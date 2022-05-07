mod config;
mod hook;
mod loader;
mod minify;
mod options;
mod output;
mod resolver;

use askama::Template;
use config::TsConfig;
use deno_ast::swc::{
    self,
    bundler::Bundler,
    common::{FileName, FilePathMapping, Globals, SourceMap, GLOBALS},
};
use deno_core::{anyhow::Context, error::AnyError, ModuleSpecifier};
use deno_utils::{ModuleStore, UniversalModuleLoader};
use derive_builder::Builder;
use hook::BundleHook;
use loader::BundleLoader;
use minify::minify;
use output::gen_code;
use resolver::BundleResolver;
use std::{collections::HashMap, rc::Rc, sync::Arc};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BundleType {
    MainModule,
    /// Return the emitted contents of the program as a single "flattened" ES
    /// module.
    Module,
    /// Return the emitted contents of the program as a single script that
    /// executes the program using an immediately invoked function execution
    /// (IIFE).
    Classic,
}

#[derive(Builder, Clone)]
#[builder(default, pattern = "owned")]
pub struct BundleOptions {
    pub bundle_type: BundleType,
    pub ts_config: TsConfig,
    pub emit_ignore_directives: bool,
    pub module_store: Option<Arc<dyn ModuleStore>>,
    pub minify: bool,
}

#[derive(Template)]
#[template(path = "layout.j2", escape = "none")]
struct BundledJs {
    body: String,
    bundle_type: BundleType,
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

    let globals = Globals::new();
    GLOBALS.set(&globals, || {
        let emit_options: deno_ast::EmitOptions = options.ts_config.into();

        let cm = Rc::new(SourceMap::new(FilePathMapping::empty()));
        let loader = BundleLoader::new(cm.clone(), &emit_options, graph);
        let resolver = BundleResolver(graph);
        let config = swc::bundler::Config {
            module: options.bundle_type.into(),
            disable_fixer: options.minify,
            disable_hygiene: options.minify,
            ..Default::default()
        };
        // This hook will rewrite the `import.meta` when bundling to give a consistent
        // behavior between bundled and unbundled code.
        let hook = Box::new(BundleHook);
        let mut bundler = Bundler::new(&globals, cm.clone(), loader, resolver, config, hook);
        let mut entries = HashMap::new();
        entries.insert(
            "bundle".to_string(),
            FileName::Url(graph.roots[0].0.clone()),
        );
        let mut modules = bundler
            .bundle(entries)
            .context("Unable to output during bundling.")?;

        if options.minify {
            modules = minify(cm.clone(), modules);
        }

        let (mut code, may_map) = gen_code(
            cm,
            &modules[0],
            &emit_options,
            options.emit_ignore_directives,
            options.minify,
        )?;

        let tpl = BundledJs {
            body: code,
            bundle_type: options.bundle_type,
        };
        code = tpl.render()?;
        Ok((code, may_map))
    })
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use deno_core::resolve_url_or_path;

    use super::*;

    #[tokio::test]
    async fn bundle_code_module_should_work() {
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

    #[tokio::test]
    async fn bundle_code_classic_should_work() {
        let options = BundleOptions {
            bundle_type: BundleType::Classic,
            ..BundleOptions::default()
        };
        let f = Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/main.ts");
        let f = f.to_string_lossy().to_string();
        let m = resolve_url_or_path(&f).unwrap();
        let ret = bundle(m.clone(), options.clone()).await;
        assert!(ret.is_ok());
    }

    #[tokio::test]
    async fn bundle_code_main_module_should_work() {
        let options = BundleOptions {
            bundle_type: BundleType::MainModule,
            ..BundleOptions::default()
        };
        let f = Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/main.ts");
        let f = f.to_string_lossy().to_string();
        let m = resolve_url_or_path(&f).unwrap();
        let ret = bundle(m.clone(), options.clone()).await;
        assert!(ret.is_ok());
    }
}
