use deno_ast::swc::{
    ast::Module,
    common::{Mark, SourceMap},
};
use std::rc::Rc;
use swc_ecma_minifier::{
    optimize,
    option::{ExtraOptions, MangleOptions, MinifyOptions},
};

pub(crate) fn minify_module(program: Module) -> Module {
    let top_level_mark = Mark::fresh(Mark::root());
    let unresolved_mark = Mark::fresh(Mark::root());
    let cm = Rc::new(SourceMap::default());
    let options = MinifyOptions {
        compress: Some(Default::default()),
        mangle: Some(MangleOptions {
            top_level: true,
            ..Default::default()
        }),
        ..Default::default()
    };
    optimize(
        program.into(),
        cm,
        None,
        None,
        &options,
        &ExtraOptions {
            top_level_mark,
            unresolved_mark,
        },
    )
    .expect_module()
}
