mod config;
mod hook;
mod loader;
mod resolver;

use askama::Template;
use config::{get_ts_config, ConfigType, TsConfig};
use deno_ast::swc;
use deno_core::{anyhow::Context, error::AnyError, ModuleSpecifier};
use deno_utils::{FsModuleStore, ModuleStore, UniversalModuleLoader};
use derive_builder::Builder;
use hook::BundleHook;
use loader::BundleLoader;
use resolver::BundleResolver;
use std::{collections::HashMap, rc::Rc, sync::Arc};

const IGNORE_DIRECTIVES: &[&str] = &[
    "// deno-fmt-ignore-file",
    "// deno-lint-ignore-file",
    "// This code was bundled using `deno bundle` and it's not recommended to edit it manually",
    "",
];

#[derive(Debug, Clone, Copy)]
pub enum BundleType {
    /// Return the emitted contents of the program as a single "flattened" ES
    /// module.
    Module,
    /// Return the emitted contents of the program as a single script that
    /// executes the program using an immediately invoked function execution
    /// (IIFE).
    Classic,
}

impl From<BundleType> for swc::bundler::ModuleType {
    fn from(bundle_type: BundleType) -> Self {
        match bundle_type {
            BundleType::Classic => Self::Iife,
            BundleType::Module => Self::Es,
        }
    }
}

#[derive(Builder, Clone)]
#[builder(default, pattern = "owned")]
pub struct BundleOptions {
    pub bundle_type: BundleType,
    pub ts_config: TsConfig,
    pub emit_ignore_directives: bool,
    pub module_store: Option<Arc<dyn ModuleStore>>,
    pub global_this: bool,
}

impl Default for BundleOptions {
    fn default() -> Self {
        Self {
            bundle_type: BundleType::Classic,
            ts_config: get_ts_config(ConfigType::Bundle).unwrap(),
            emit_ignore_directives: true,
            module_store: Some(Arc::new(FsModuleStore::default())),
            global_this: false,
        }
    }
}

#[derive(Template)]
#[template(path = "layout.j2", escape = "none")]
struct BundledJs {
    body: String,
}

/// Given a module graph, generate and return a bundle of the graph and
/// optionally its source map. Unlike emitting with `check_and_maybe_emit` and
/// `emit`, which store the emitted modules in the cache, this function simply
/// returns the output.
pub async fn bundle(
    // graph: &ModuleGraph,
    root: ModuleSpecifier,
    options: BundleOptions,
) -> Result<(String, Option<String>), AnyError> {
    let mut loader = UniversalModuleLoader::new(options.module_store, false);
    let graph_owned = deno_graph::create_graph(
        vec![(root, deno_graph::ModuleKind::Esm)],
        false,
        None,
        &mut loader,
        None,
        None,
        None,
        None,
    )
    .await;
    let graph = &graph_owned;

    let globals = swc::common::Globals::new();
    deno_ast::swc::common::GLOBALS.set(&globals, || {
        let emit_options: deno_ast::EmitOptions = options.ts_config.into();
        let source_map_config = deno_ast::SourceMapConfig {
            inline_sources: emit_options.inline_sources,
        };

        let cm = Rc::new(swc::common::SourceMap::new(
            swc::common::FilePathMapping::empty(),
        ));
        let loader = BundleLoader::new(cm.clone(), &emit_options, graph);
        let resolver = BundleResolver(graph);
        let config = swc::bundler::Config {
            module: options.bundle_type.into(),
            ..Default::default()
        };
        // This hook will rewrite the `import.meta` when bundling to give a consistent
        // behavior between bundled and unbundled code.
        let hook = Box::new(BundleHook);
        let mut bundler =
            swc::bundler::Bundler::new(&globals, cm.clone(), loader, resolver, config, hook);
        let mut entries = HashMap::new();
        entries.insert(
            "bundle".to_string(),
            swc::common::FileName::Url(graph.roots[0].0.clone()),
        );
        let output = bundler
            .bundle(entries)
            .context("Unable to output during bundling.")?;
        let mut buf = Vec::new();
        let mut srcmap = Vec::new();
        {
            let cfg = swc::codegen::Config { minify: true };
            let mut wr = Box::new(swc::codegen::text_writer::JsWriter::new(
                cm.clone(),
                "\n",
                &mut buf,
                Some(&mut srcmap),
            ));

            if options.emit_ignore_directives {
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
                .emit_module(&output[0].module)
                .context("Unable to emit during bundling.")?;
        }
        let mut code = String::from_utf8(buf).context("Emitted code is an invalid string.")?;
        let mut maybe_map: Option<String> = None;
        {
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

        if options.global_this {
            let tpl = BundledJs { body: code };
            code = tpl.render()?;
        }
        Ok((code, maybe_map))
    })
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use deno_core::resolve_url_or_path;

    use super::*;

    #[tokio::test]
    async fn bundle_code_should_work() {
        let options = BundleOptions::default();
        let f = Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/main.ts");
        let f = f.to_string_lossy().to_string();
        let m = resolve_url_or_path(&f).unwrap();
        let (_bundle, _) = bundle(m.clone(), options.clone()).await.unwrap();
        let store = options.module_store.unwrap();
        let ret = store.get(m.as_str()).await;
        assert!(ret.is_ok());
        let imported = resolve_url_or_path("https://deno.land/std@0.134.0/http/server.ts").unwrap();
        let ret = store.get(imported.as_str()).await;
        assert!(ret.is_ok());
    }
}
