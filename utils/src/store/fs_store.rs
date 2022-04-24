use crate::{to_hash_path, FsModuleStore, ModuleStore};
use async_trait::async_trait;
use deno_core::{anyhow::bail, error::AnyError};
use dirs::home_dir;
use std::{
    fs,
    io::{Read, Write},
    path::PathBuf,
};

impl Default for FsModuleStore {
    fn default() -> Self {
        let base = home_dir().unwrap().join(".cache/deno_fs_store");
        fs::create_dir_all(&base).unwrap();
        FsModuleStore { base }
    }
}

impl FsModuleStore {
    pub fn new(base: impl Into<PathBuf>) -> Self {
        let base = base.into();
        fs::create_dir_all(&base).unwrap();
        FsModuleStore { base }
    }
}

#[async_trait]
impl ModuleStore for FsModuleStore {
    async fn get(&self, key: &str) -> Result<String, AnyError> {
        let path = to_hash_path(&self.base, key);
        if !path.exists() {
            bail!("Module not found: {}", key);
        }
        let mut file = fs::File::open(&path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Ok(contents)
    }

    async fn put(&self, key: String, value: String) -> Result<(), AnyError> {
        let path = to_hash_path(&self.base, &key);
        fs::create_dir_all(path.parent().unwrap())?;
        let mut file = fs::File::create(&path)?;
        file.write_all(value.as_bytes())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{FsModuleStore, ModuleStore};
    use std::path::PathBuf;

    #[tokio::test]
    async fn module_store_should_work() {
        let base = PathBuf::from("/tmp/deno_fs_store");
        let store = FsModuleStore::new(base);
        store
            .put("foo".to_string(), "bar".to_string())
            .await
            .unwrap();
        let contents = store.get("foo").await.unwrap();
        assert_eq!(contents, "bar");
    }
}
