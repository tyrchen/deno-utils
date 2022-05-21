use deno_ast::swc;
use swc::{
    bundler::Bundle,
    common::{sync::Lrc, Mark, SourceMap},
    minifier::{
        optimize,
        option::{ExtraOptions, MangleOptions, MinifyOptions},
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
                    // FIXME: use compress option
                    // currently it would cause module compiling issue
                    compress: None,
                    mangle: Some(MangleOptions {
                        top_level: true,
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                &ExtraOptions {
                    unresolved_mark: Mark::fresh(Mark::root()),
                    top_level_mark: Mark::fresh(Mark::root()),
                },
            );
            b.module.visit_mut_with(&mut fixer(None));
            b
        })
        .collect()
}
