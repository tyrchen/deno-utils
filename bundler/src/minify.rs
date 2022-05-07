use deno_ast::swc;
use swc::{
    bundler::Bundle,
    common::{sync::Lrc, Mark, SourceMap},
    minifier::{
        optimize,
        option::{CompressOptions, ExtraOptions, MangleOptions, MinifyOptions, TopLevelOptions},
    },
    transforms::fixer,
    visit::VisitMutWith,
};

pub fn minify(cm: Lrc<SourceMap>, modules: Vec<Bundle>) -> Vec<Bundle> {
    modules
        .into_iter()
        .map(|mut b| {
            b.module = optimize(
                b.module,
                cm.clone(),
                None,
                None,
                &MinifyOptions {
                    compress: Some(CompressOptions {
                        top_level: Some(TopLevelOptions { functions: true }),
                        ..Default::default()
                    }),
                    mangle: Some(MangleOptions {
                        top_level: true,
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                &ExtraOptions {
                    top_level_mark: Mark::new(),
                },
            );
            b.module.visit_mut_with(&mut fixer(None));
            b
        })
        .collect()
}
