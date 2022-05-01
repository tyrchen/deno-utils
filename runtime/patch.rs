use crate::{permissions::Permissions, worker::WorkerOptions, BootstrapOptions};
use deno_broadcast_channel::InMemoryBroadcastChannel;
use deno_core::{FsModuleLoader, Snapshot};
use deno_web::BlobStore;
use std::{rc::Rc, sync::Arc};

const TS_VERSION: &str = "3.7.2";
const USER_AGENT: &str = "deno-simple-runtime";

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
        }
    }
}

impl Default for WorkerOptions {
    fn default() -> Self {
        Self {
            bootstrap: BootstrapOptions::default(),
            unsafely_ignore_certificate_errors: None,
            root_cert_store: None,
            user_agent: USER_AGENT.to_string(),
            seed: None,
            format_js_error_fn: None,
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
            get_error_class_fn: None,
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
