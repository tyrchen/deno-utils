use std::sync::Arc;

use deno_ast::swc;
use deno_utils::FsModuleStore;

use crate::{
    config::{get_ts_config, ConfigType},
    BundleOptions, BundleType,
};

impl Default for BundleType {
    fn default() -> Self {
        BundleType::Module
    }
}

impl From<BundleType> for swc::bundler::ModuleType {
    fn from(bundle_type: BundleType) -> Self {
        match bundle_type {
            BundleType::Classic => Self::Iife,
            BundleType::Module => Self::Es,
            BundleType::MainModule => Self::Es,
        }
    }
}

impl Default for BundleOptions {
    fn default() -> Self {
        Self {
            bundle_type: BundleType::Module,
            ts_config: get_ts_config(ConfigType::Bundle).unwrap(),
            emit_ignore_directives: false,
            module_store: Some(Arc::new(FsModuleStore::default())),
            minify: true,
        }
    }
}
