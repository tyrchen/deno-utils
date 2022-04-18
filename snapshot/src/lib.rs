#[cfg(feature = "build")]
mod builder;
#[cfg(feature = "build")]
mod permissions;

#[cfg(feature = "build")]
pub use builder::{create_snapshot, create_snapshot_with_main_module, get_js_files};

pub fn decode(compressed: &[u8]) -> Box<[u8]> {
    zstd::decode_all(compressed).unwrap().into_boxed_slice()
}
