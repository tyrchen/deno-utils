use crate::{
    colors::{cyan, italic_bold, red, yellow},
    errors,
    permissions::Permissions,
    worker::WorkerOptions,
    BootstrapOptions,
};
use deno_broadcast_channel::InMemoryBroadcastChannel;
use deno_core::{
    error::{format_file_name, AnyError, JsError, JsStackFrame},
    FsModuleLoader, Snapshot,
};
use deno_web::BlobStore;
use std::fmt::Write;
use std::{rc::Rc, sync::Arc};

const TS_VERSION: &str = "3.7.2";
const USER_AGENT: &str = "deno-simple-runtime";
const SOURCE_ABBREV_THRESHOLD: usize = 150;

impl Default for BootstrapOptions {
    fn default() -> Self {
        Self {
            args: vec![],
            cpu_count: 1,
            debug_flag: false,
            enable_testing_features: false,
            location: None,
            runtime_version: env!("CARGO_PKG_VERSION").to_string(),
            ts_version: TS_VERSION.to_string(),
            unstable: false,
            no_color: false,
            is_tty: false,
            user_agent: USER_AGENT.to_string(),
        }
    }
}

impl Default for WorkerOptions {
    fn default() -> Self {
        Self {
            bootstrap: BootstrapOptions::default(),
            unsafely_ignore_certificate_errors: None,
            root_cert_store: None,
            seed: None,
            format_js_error_fn: Some(Arc::new(format_js_error)),
            module_loader: Rc::new(FsModuleLoader),
            create_web_worker_cb: Arc::new(|_| {
                panic!("Web workers are not supported");
            }),
            web_worker_preload_module_cb: Arc::new(|_| {
                panic!("Web workers are not supported");
            }),
            js_error_create_fn: None,
            maybe_inspector_server: None,
            should_break_on_first_statement: false,
            get_error_class_fn: Some(&get_error_class_name),
            origin_storage_dir: None,
            blob_store: BlobStore::default(),
            broadcast_channel: InMemoryBroadcastChannel::default(),
            shared_array_buffer_store: None,
            compiled_wasm_module_store: None,
            stdio: Default::default(),

            main_module: None,
            permissions: Permissions::default(),
            startup_snapshot: None,
            runtime_options_callback: None,
        }
    }
}

#[derive(Clone)]
pub enum StartSnapshot {
    Static(&'static [u8]),
    Dynamic(Box<[u8]>),
}

impl From<StartSnapshot> for Snapshot {
    fn from(snapshot: StartSnapshot) -> Self {
        match snapshot {
            StartSnapshot::Static(data) => Snapshot::Static(data),
            StartSnapshot::Dynamic(data) => Snapshot::Boxed(data),
        }
    }
}

fn get_error_class_name(e: &AnyError) -> &'static str {
    errors::get_error_class_name(e).unwrap_or("Error")
}

fn format_js_error(js_error: &JsError) -> String {
    format_js_error_inner(js_error, false)
}

fn format_js_error_inner(js_error: &JsError, is_child: bool) -> String {
    let mut s = String::new();
    s.push_str(&js_error.exception_message);
    if let Some(aggregated) = &js_error.aggregated {
        for aggregated_error in aggregated {
            let error_string = format_js_error_inner(aggregated_error, true);
            for line in error_string.trim_start_matches("Uncaught ").lines() {
                write!(&mut s, "\n    {}", line).unwrap();
            }
        }
    }
    let column_number = js_error
        .source_line_frame_index
        .and_then(|i| js_error.frames.get(i).unwrap().column_number);
    s.push_str(&format_maybe_source_line(
        if is_child {
            None
        } else {
            js_error.source_line.as_deref()
        },
        column_number,
        true,
        0,
    ));
    for frame in &js_error.frames {
        write!(&mut s, "\n    at {}", format_frame(frame)).unwrap();
    }
    if let Some(cause) = &js_error.cause {
        let error_string = format_js_error_inner(cause, true);
        write!(
            &mut s,
            "\nCaused by: {}",
            error_string.trim_start_matches("Uncaught ")
        )
        .unwrap();
    }
    s
}

/// Take an optional source line and associated information to format it into
/// a pretty printed version of that line.
fn format_maybe_source_line(
    source_line: Option<&str>,
    column_number: Option<i64>,
    is_error: bool,
    level: usize,
) -> String {
    if source_line.is_none() || column_number.is_none() {
        return "".to_string();
    }

    let source_line = source_line.unwrap();
    // sometimes source_line gets set with an empty string, which then outputs
    // an empty source line when displayed, so need just short circuit here.
    // Also short-circuit on error line too long.
    if source_line.is_empty() || source_line.len() > SOURCE_ABBREV_THRESHOLD {
        return "".to_string();
    }
    if source_line.contains("Couldn't format source line: ") {
        return format!("\n{}", source_line);
    }

    let mut s = String::new();
    let column_number = column_number.unwrap();

    if column_number as usize > source_line.len() {
        return format!(
        "\n{} Couldn't format source line: Column {} is out of bounds (source may have changed at runtime)",
        crate::colors::yellow("Warning"), column_number,
      );
    }

    for _i in 0..(column_number - 1) {
        if source_line.chars().nth(_i as usize).unwrap() == '\t' {
            s.push('\t');
        } else {
            s.push(' ');
        }
    }
    s.push('^');
    let color_underline = if is_error {
        red(&s).to_string()
    } else {
        cyan(&s).to_string()
    };

    let indent = format!("{:indent$}", "", indent = level);

    format!("\n{}{}\n{}{}", indent, source_line, indent, color_underline)
}

fn format_frame(frame: &JsStackFrame) -> String {
    let _internal = frame
        .file_name
        .as_ref()
        .map_or(false, |f| f.starts_with("deno:"));
    let is_method_call = !(frame.is_top_level.unwrap_or_default() || frame.is_constructor);
    let mut result = String::new();
    if frame.is_async {
        result += "async ";
    }
    if frame.is_promise_all {
        result += &italic_bold(&format!(
            "Promise.all (index {})",
            frame.promise_index.unwrap_or_default()
        ))
        .to_string();
        return result;
    }
    if is_method_call {
        let mut formatted_method = String::new();
        if let Some(function_name) = &frame.function_name {
            if let Some(type_name) = &frame.type_name {
                if !function_name.starts_with(type_name) {
                    write!(&mut formatted_method, "{}.", type_name).unwrap();
                }
            }
            formatted_method += function_name;
            if let Some(method_name) = &frame.method_name {
                if !function_name.ends_with(method_name) {
                    write!(&mut formatted_method, " [as {}]", method_name).unwrap();
                }
            }
        } else {
            if let Some(type_name) = &frame.type_name {
                write!(&mut formatted_method, "{}.", type_name).unwrap();
            }
            if let Some(method_name) = &frame.method_name {
                formatted_method += method_name
            } else {
                formatted_method += "<anonymous>";
            }
        }
        result += &italic_bold(&formatted_method).to_string();
    } else if frame.is_constructor {
        result += "new ";
        if let Some(function_name) = &frame.function_name {
            result += &italic_bold(&function_name).to_string();
        } else {
            result += &cyan("<anonymous>").to_string();
        }
    } else if let Some(function_name) = &frame.function_name {
        result += &italic_bold(&function_name).to_string();
    } else {
        result += &format_location(frame);
        return result;
    }
    write!(&mut result, " ({})", format_location(frame)).unwrap();
    result
}

// Keep in sync with `/core/error.js`.
pub fn format_location(frame: &JsStackFrame) -> String {
    let _internal = frame
        .file_name
        .as_ref()
        .map_or(false, |f| f.starts_with("deno:"));
    if frame.is_native {
        return cyan("native").to_string();
    }
    let mut result = String::new();
    let file_name = frame.file_name.clone().unwrap_or_default();
    if !file_name.is_empty() {
        result += &cyan(&format_file_name(&file_name)).to_string();
    } else {
        if frame.is_eval {
            result += &(cyan(&frame.eval_origin.as_ref().unwrap()).to_string() + ", ");
        }
        result += &cyan("<anonymous>").to_string();
    }
    if let Some(line_number) = frame.line_number {
        write!(&mut result, ":{}", yellow(&line_number.to_string())).unwrap();
        if let Some(column_number) = frame.column_number {
            write!(&mut result, ":{}", yellow(&column_number.to_string())).unwrap();
        }
    }
    result
}
