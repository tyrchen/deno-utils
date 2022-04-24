mod data_channel;
mod fs_util;
mod loader;
mod store;
mod tokio_util;
mod unstable_checker;

pub use data_channel::*;
use deno_core::error::AnyError;
pub use fs_util::*;
pub use loader::*;
pub use store::*;
pub use tokio_util::*;
pub use unstable_checker::*;

use async_trait::async_trait;

#[async_trait]
pub trait ModuleStore: Send + Sync {
    async fn get(&self, specifier: &str) -> Result<String, AnyError>;
    async fn put(&self, specifier: String, code: String) -> Result<(), AnyError>;
}

#[cfg(test)]
pub mod test_util {
    use std::path::PathBuf;

    pub fn testdata_path(name: &str) -> String {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let path = path.join(format!("../fixtures/testdata/{}", name));
        path.to_string_lossy().into()
    }
}
