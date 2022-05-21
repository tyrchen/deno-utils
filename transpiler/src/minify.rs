use std::rc::Rc;

use deno_ast::swc::{
    ast::Module,
    common::{Mark, SourceMap},
    minifier::{
        optimize,
        option::{ExtraOptions, MangleOptions, MinifyOptions},
    },
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
        program,
        cm,
        None,
        None,
        &options,
        &ExtraOptions {
            top_level_mark,
            unresolved_mark,
        },
    )
}
