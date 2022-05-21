use deno_ast::{
    get_syntax,
    swc::{
        self,
        common::{comments::SingleThreadedComments, FileName, Mark, SourceMap, Spanned},
        parser::{error::Error as SwcError, lexer::Lexer, StringInput},
    },
    Diagnostic, LineAndColumnDisplay, MediaType, SourceRangedForSpanned,
};

use deno_core::{anyhow::anyhow, error::AnyError, ModuleSpecifier};
use deno_graph::ModuleGraph;
use std::rc::Rc;

/// A module loader for swc which does the appropriate retrieval and transpiling
/// of modules from the graph.
pub struct BundleLoader<'a> {
    cm: Rc<swc::common::SourceMap>,
    emit_options: &'a deno_ast::EmitOptions,
    graph: &'a ModuleGraph,
}

impl<'a> BundleLoader<'a> {
    pub fn new(
        cm: Rc<swc::common::SourceMap>,
        emit_options: &'a deno_ast::EmitOptions,
        graph: &'a ModuleGraph,
    ) -> Self {
        Self {
            cm,
            emit_options,
            graph,
        }
    }
}

impl swc::bundler::Load for BundleLoader<'_> {
    fn load(
        &self,
        file_name: &swc::common::FileName,
    ) -> Result<swc::bundler::ModuleData, AnyError> {
        match file_name {
            swc::common::FileName::Url(specifier) => {
                if let Some(m) = self.graph.get(specifier) {
                    let (fm, module) = transpile_module(
                        specifier,
                        m.maybe_source.as_ref().map(|s| s.as_ref()).unwrap_or(""),
                        m.media_type,
                        self.emit_options,
                        self.cm.clone(),
                    )?;
                    Ok(swc::bundler::ModuleData {
                        fm,
                        module,
                        helpers: Default::default(),
                    })
                } else {
                    Err(anyhow!(
                        "Module \"{}\" unexpectedly missing when bundling.",
                        specifier
                    ))
                }
            }
            _ => unreachable!(
                "Received a request for unsupported filename {:?}",
                file_name
            ),
        }
    }
}

/// Transpiles a source module into an swc SourceFile.
fn transpile_module(
    specifier: &ModuleSpecifier,
    source: &str,
    media_type: MediaType,
    options: &deno_ast::EmitOptions,
    cm: Rc<swc::common::SourceMap>,
) -> Result<(Rc<swc::common::SourceFile>, swc::ast::Module), AnyError> {
    let source = strip_bom(source);
    let source = if media_type == MediaType::Json {
        format!(
            "export default JSON.parse(`{}`);",
            source.replace("${", "\\${").replace('`', "\\`")
        )
    } else {
        source.to_string()
    };
    let source_file = cm.new_source_file(FileName::Url(specifier.clone()), source);
    let input = StringInput::from(&*source_file);
    let comments = SingleThreadedComments::default();
    let syntax = if media_type == MediaType::Json {
        get_syntax(MediaType::JavaScript)
    } else {
        get_syntax(media_type)
    };
    let lexer = Lexer::new(syntax, deno_ast::ES_VERSION, input, Some(&comments));
    let mut parser = swc::parser::Parser::new_from(lexer);
    let module = parser
        .parse_module()
        .map_err(|e| swc_err_to_diagnostic(&cm, specifier, e))?;
    let diagnostics = parser
        .take_errors()
        .into_iter()
        .map(|e| swc_err_to_diagnostic(&cm, specifier, e))
        .collect::<Vec<_>>();

    let top_level_mark = Mark::fresh(Mark::root());
    let program = deno_ast::fold_program(
        swc::ast::Program::Module(module),
        options,
        cm,
        &comments,
        top_level_mark,
        &diagnostics,
    )?;
    let module = match program {
        swc::ast::Program::Module(module) => module,
        _ => unreachable!(),
    };

    Ok((source_file, module))
}

const BOM_CHAR: char = '\u{FEFF}';
/// Strips the byte order mark from the provided text if it exists.
fn strip_bom(text: &str) -> &str {
    if text.starts_with(BOM_CHAR) {
        &text[BOM_CHAR.len_utf8()..]
    } else {
        text
    }
}

fn swc_err_to_diagnostic(
    source_map: &SourceMap,
    specifier: &ModuleSpecifier,
    err: SwcError,
) -> Diagnostic {
    let location = source_map.lookup_char_pos(err.span().lo);
    Diagnostic {
        specifier: specifier.to_string(),
        range: err.range(),
        display_position: LineAndColumnDisplay {
            line_number: location.line,
            column_number: location.col_display + 1,
        },
        kind: err.into_kind(),
    }
}
