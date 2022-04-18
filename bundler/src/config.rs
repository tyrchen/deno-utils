use deno_core::{
    error::AnyError,
    serde_json::{self, json, Value},
};
use serde::{Deserialize, Serialize, Serializer};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmitConfigOptions {
    pub check_js: bool,
    pub emit_decorator_metadata: bool,
    pub imports_not_used_as_values: String,
    pub inline_source_map: bool,
    pub inline_sources: bool,
    pub source_map: bool,
    pub jsx: String,
    pub jsx_factory: String,
    pub jsx_fragment_factory: String,
    pub jsx_import_source: Option<String>,
}

/// A structure for managing the configuration of TypeScript
#[derive(Debug, Clone)]
pub struct TsConfig(pub Value);

impl TsConfig {
    /// Create a new `TsConfig` with the base being the `value` supplied.
    pub fn new(value: Value) -> Self {
        TsConfig(value)
    }

    /// Merge a serde_json value into the configuration.
    pub fn merge(&mut self, value: &Value) {
        json_merge(&mut self.0, value);
    }
}

impl From<TsConfig> for deno_ast::EmitOptions {
    fn from(config: TsConfig) -> Self {
        let options: EmitConfigOptions = serde_json::from_value(config.0).unwrap();
        let imports_not_used_as_values = match options.imports_not_used_as_values.as_str() {
            "preserve" => deno_ast::ImportsNotUsedAsValues::Preserve,
            "error" => deno_ast::ImportsNotUsedAsValues::Error,
            _ => deno_ast::ImportsNotUsedAsValues::Remove,
        };
        let (transform_jsx, jsx_automatic, jsx_development) = match options.jsx.as_str() {
            "react" => (true, false, false),
            "react-jsx" => (true, true, false),
            "react-jsxdev" => (true, true, true),
            _ => (false, false, false),
        };
        deno_ast::EmitOptions {
            emit_metadata: options.emit_decorator_metadata,
            imports_not_used_as_values,
            inline_source_map: options.inline_source_map,
            inline_sources: options.inline_sources,
            source_map: options.source_map,
            jsx_automatic,
            jsx_development,
            jsx_factory: options.jsx_factory,
            jsx_fragment_factory: options.jsx_fragment_factory,
            jsx_import_source: options.jsx_import_source,
            transform_jsx,
            var_decl_imports: false,
        }
    }
}

/// Represents the "default" type library that should be used when type
/// checking the code in the module graph.  Note that a user provided config
/// of `"lib"` would override this value.
#[allow(dead_code)]
#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub enum TypeLib {
    DenoWindow,
    DenoWorker,
    UnstableDenoWindow,
    UnstableDenoWorker,
}

impl Default for TypeLib {
    fn default() -> Self {
        Self::DenoWindow
    }
}

impl Serialize for TypeLib {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let value = match self {
            Self::DenoWindow => vec!["deno.window".to_string()],
            Self::DenoWorker => vec!["deno.worker".to_string()],
            Self::UnstableDenoWindow => {
                vec!["deno.window".to_string(), "deno.unstable".to_string()]
            }
            Self::UnstableDenoWorker => {
                vec!["deno.worker".to_string(), "deno.unstable".to_string()]
            }
        };
        Serialize::serialize(&value, serializer)
    }
}

/// An enum that represents the base tsc configuration to return.
#[allow(dead_code)]
pub enum ConfigType {
    /// Return a configuration for bundling, using swc to emit the bundle. This is
    /// independent of type checking.
    Bundle,
    /// Return a configuration to use tsc to type check and optionally emit. This
    /// is independent of either bundling or just emitting via swc
    Check { lib: TypeLib, tsc_emit: bool },
    /// Return a configuration to use swc to emit single module files.
    Emit,
    /// Return a configuration as a base for the runtime `Deno.emit()` API.
    RuntimeEmit { tsc_emit: bool },
}

/// For a given configuration type and optionally a configuration file, return a
/// tuple of the resulting `TsConfig` struct and optionally any user
/// configuration options that were ignored.
pub fn get_ts_config(config_type: ConfigType) -> Result<TsConfig, AnyError> {
    let ts_config = match config_type {
        ConfigType::Bundle => TsConfig::new(json!({
          "checkJs": false,
          "emitDecoratorMetadata": false,
          "importsNotUsedAsValues": "remove",
          "inlineSourceMap": false,
          "inlineSources": false,
          "sourceMap": false,
          "jsx": "react",
          "jsxFactory": "React.createElement",
          "jsxFragmentFactory": "React.Fragment",
        })),
        ConfigType::Check { tsc_emit, lib } => {
            let mut ts_config = TsConfig::new(json!({
              "allowJs": true,
              "allowSyntheticDefaultImports": true,
              "experimentalDecorators": true,
              "incremental": true,
              "jsx": "react",
              "isolatedModules": true,
              "lib": lib,
              "module": "esnext",
              "resolveJsonModule": true,
              "strict": true,
              "target": "esnext",
              "tsBuildInfoFile": "deno:///.tsbuildinfo",
              "useDefineForClassFields": true,
              // TODO(@kitsonk) remove for Deno 2.0
              "useUnknownInCatchVariables": false,
            }));
            if tsc_emit {
                ts_config.merge(&json!({
                  "emitDecoratorMetadata": false,
                  "importsNotUsedAsValues": "remove",
                  "inlineSourceMap": true,
                  "inlineSources": true,
                  "outDir": "deno://",
                  "removeComments": true,
                }));
            } else {
                ts_config.merge(&json!({
                  "noEmit": true,
                }));
            }
            ts_config
        }
        ConfigType::Emit => TsConfig::new(json!({
          "checkJs": false,
          "emitDecoratorMetadata": false,
          "importsNotUsedAsValues": "remove",
          "inlineSourceMap": true,
          "inlineSources": true,
          "sourceMap": false,
          "jsx": "react",
          "jsxFactory": "React.createElement",
          "jsxFragmentFactory": "React.Fragment",
          "resolveJsonModule": true,
        })),
        ConfigType::RuntimeEmit { tsc_emit } => {
            let mut ts_config = TsConfig::new(json!({
              "allowJs": true,
              "allowSyntheticDefaultImports": true,
              "checkJs": false,
              "emitDecoratorMetadata": false,
              "experimentalDecorators": true,
              "importsNotUsedAsValues": "remove",
              "incremental": true,
              "isolatedModules": true,
              "jsx": "react",
              "jsxFactory": "React.createElement",
              "jsxFragmentFactory": "React.Fragment",
              "lib": TypeLib::DenoWindow,
              "module": "esnext",
              "removeComments": true,
              "inlineSourceMap": false,
              "inlineSources": false,
              "sourceMap": true,
              "strict": true,
              "target": "esnext",
              "tsBuildInfoFile": "deno:///.tsbuildinfo",
              "useDefineForClassFields": true,
              // TODO(@kitsonk) remove for Deno 2.0
              "useUnknownInCatchVariables": false,
            }));
            if tsc_emit {
                ts_config.merge(&json!({
                  "importsNotUsedAsValues": "remove",
                  "outDir": "deno://",
                }));
            } else {
                ts_config.merge(&json!({
                  "noEmit": true,
                }));
            }
            ts_config
        }
    };
    Ok(ts_config)
}

/// A function that works like JavaScript's `Object.assign()`.
fn json_merge(a: &mut Value, b: &Value) {
    match (a, b) {
        (&mut Value::Object(ref mut a), &Value::Object(ref b)) => {
            for (k, v) in b {
                json_merge(a.entry(k.clone()).or_insert(Value::Null), v);
            }
        }
        (a, b) => {
            *a = b.clone();
        }
    }
}
