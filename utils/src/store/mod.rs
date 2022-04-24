mod fs_store;

use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct FsModuleStore {
    base: PathBuf,
}
