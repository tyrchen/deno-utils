use deno_ast::swc;
use deno_core::{anyhow::anyhow, error::AnyError};
use deno_graph::ModuleGraph;

/// A resolver implementation for swc that resolves specifiers from the graph.
pub struct BundleResolver<'a>(pub &'a ModuleGraph);

impl swc::bundler::Resolve for BundleResolver<'_> {
    fn resolve(
        &self,
        referrer: &swc::common::FileName,
        specifier: &str,
    ) -> Result<swc::common::FileName, AnyError> {
        let referrer = if let swc::common::FileName::Url(referrer) = referrer {
            referrer
        } else {
            unreachable!(
                "An unexpected referrer was passed when bundling: {:?}",
                referrer
            );
        };
        if let Some(specifier) = self.0.resolve_dependency(specifier, referrer, false) {
            Ok(deno_ast::swc::common::FileName::Url(specifier.clone()))
        } else {
            Err(anyhow!(
                "Cannot resolve \"{}\" from \"{}\".",
                specifier,
                referrer
            ))
        }
    }
}
