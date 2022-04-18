use deno_ast::{EmitOptions, MediaType, ParseParams, SourceTextInfo};
use deno_core::{error::AnyError, ModuleSpecifier};

use crate::minify::minify_module;

pub fn compile(m: &ModuleSpecifier, code: String, minify: bool) -> Result<String, AnyError> {
    let media_type = MediaType::from(m);
    let params = ParseParams {
        specifier: m.to_string(),
        source: SourceTextInfo::from_string(code),
        media_type,
        capture_tokens: false,
        scope_analysis: false,
        maybe_syntax: None,
    };
    let parsed = if !minify {
        deno_ast::parse_module(params)?
    } else {
        deno_ast::parse_module_with_post_process(params, minify_module)?
    };
    let options = EmitOptions {
        source_map: false,
        inline_source_map: false,
        ..Default::default()
    };
    Ok(parsed.transpile(&options)?.text)
}

#[allow(dead_code)]
fn need_transpile(media_type: MediaType) -> bool {
    match media_type {
        MediaType::JavaScript | MediaType::Mjs | MediaType::Cjs | MediaType::Json => false,
        MediaType::Jsx
        | MediaType::TypeScript
        | MediaType::Mts
        | MediaType::Cts
        | MediaType::Dts
        | MediaType::Dmts
        | MediaType::Dcts
        | MediaType::Tsx => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use deno_core::resolve_url_or_path;

    use super::*;

    #[test]
    fn compile_should_work() {
        use regex::Regex;
        let re = Regex::new(r"\s+").unwrap();

        let ts_code = include_str!("../fixtures/code.ts");
        let js_code = include_str!("../fixtures/code.js");
        let m = resolve_url_or_path("foo.ts").unwrap();
        let res = compile(&m, ts_code.to_string(), false).unwrap();
        assert_eq!(re.replace_all(&res, ""), re.replace_all(js_code, ""));
    }
}
