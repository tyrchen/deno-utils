use deno_ast::{
    swc::{self, bundler::Bundle, common::sync::Lrc, common::SourceMap},
    EmitOptions,
};
use deno_core::{anyhow::Context, error::AnyError};

const IGNORE_DIRECTIVES: &[&str] = &[
    "// deno-fmt-ignore-file",
    "// deno-lint-ignore-file",
    "// This code was bundled using `deno-bundler` and it's not recommended to edit it manually",
    "",
];

pub fn gen_code(
    cm: Lrc<SourceMap>,
    bundle: &Bundle,
    emit_options: &EmitOptions,
    ignore_directive: bool,
    minify: bool,
) -> Result<(String, Option<String>), AnyError> {
    let source_map_config = deno_ast::SourceMapConfig {
        inline_sources: emit_options.inline_sources,
    };
    let mut buf = Vec::new();
    let mut srcmap = Vec::new();
    {
        let cfg = swc::codegen::Config { minify };
        let mut wr = Box::new(swc::codegen::text_writer::JsWriter::new(
            cm.clone(),
            "\n",
            &mut buf,
            Some(&mut srcmap),
        ));

        if ignore_directive {
            // write leading comments in bundled file
            use swc::codegen::text_writer::WriteJs;
            let cmt = IGNORE_DIRECTIVES.join("\n") + "\n";
            wr.write_comment(&cmt)?;
        }

        let mut emitter = swc::codegen::Emitter {
            cfg,
            cm: cm.clone(),
            comments: None,
            wr,
        };
        emitter
            .emit_module(&bundle.module)
            .context("Unable to emit during bundling.")?;
    }
    let mut code = String::from_utf8(buf).context("Emitted code is an invalid string.")?;

    let mut maybe_map: Option<String> = None;
    if emit_options.source_map || emit_options.inline_source_map {
        let mut buf = Vec::new();
        cm.build_source_map_with_config(&mut srcmap, None, source_map_config)
            .to_writer(&mut buf)?;
        if emit_options.inline_source_map {
            let encoded_map = format!(
                "//# sourceMappingURL=data:application/json;base64,{}\n",
                base64::encode(buf)
            );
            code.push_str(&encoded_map);
        } else if emit_options.source_map {
            maybe_map = Some(String::from_utf8(buf)?);
        }
    }

    Ok((code, maybe_map))
}
