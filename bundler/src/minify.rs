use deno_ast::swc;
use swc::{
    bundler::Bundle,
    common::{sync::Lrc, Mark, SourceMap},
    transforms::fixer,
    visit::VisitMutWith,
};
use swc_ecma_minifier::{
    optimize,
    option::{ExtraOptions, MinifyOptions},
};

const MINIFY_CONFIG: &str = include_str!("config.json");

pub fn minify(cm: Lrc<SourceMap>, modules: Vec<Bundle>) -> Vec<Bundle> {
    let options: MinifyOptions = serde_json::from_str(MINIFY_CONFIG).unwrap();
    modules
        .into_iter()
        .map(|mut b| {
            b.module = optimize(
                b.module.into(),
                cm.clone(),
                None,
                None,
                &options,
                &ExtraOptions {
                    unresolved_mark: Mark::fresh(Mark::root()),
                    top_level_mark: Mark::fresh(Mark::root()),
                },
            )
            .expect_module();
            b.module.visit_mut_with(&mut fixer(None));
            b
        })
        .collect()
}
