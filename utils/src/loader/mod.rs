mod universal_loader;

pub use universal_loader::UniversalModuleLoader;

pub trait ModuleStore: Send + Sync {
    fn get(&self, specifier: &str) -> Option<String>;
    fn put(&self, specifier: String, code: String);
}
