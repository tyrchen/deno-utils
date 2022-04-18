use anyhow::Result;
use deno_broadcast_channel::InMemoryBroadcastChannel;
use deno_core::{JsRuntime, RuntimeOptions};
use deno_web::BlobStore;
use std::{
    env,
    path::{Path, PathBuf},
};

use crate::permissions::Permissions;

const JS_PATHS: &[&str] = &["js/**/*.js"];

pub fn create_snapshot_with_main_module(
    files: &[PathBuf],
    code: Option<String>,
) -> Result<Vec<u8>> {
    _create_snapshot(files, code, false)
}

pub fn create_snapshot(files: &[PathBuf]) -> Result<Vec<u8>> {
    _create_snapshot(files, None, true)
}

pub fn _create_snapshot(files: &[PathBuf], code: Option<String>, build: bool) -> Result<Vec<u8>> {
    // Order matters!
    let extensions = vec![
        deno_webidl::init(),
        deno_console::init(),
        deno_url::init(),
        deno_web::init::<Permissions>(BlobStore::default(), Default::default()),
        deno_fetch::init::<Permissions>(Default::default()),
        deno_websocket::init::<Permissions>("".to_owned(), None, None),
        deno_webstorage::init(None),
        deno_crypto::init(None),
        deno_webgpu::init(false),
        deno_broadcast_channel::init(InMemoryBroadcastChannel::default(), false),
        deno_tls::init(),
        deno_ffi::init::<Permissions>(false),
        deno_net::init::<Permissions>(
            None, false, // No --unstable.
            None,
        ),
        deno_http::init(),
    ];

    let rt = JsRuntime::new(RuntimeOptions {
        will_snapshot: true,
        extensions,
        ..Default::default()
    });
    _gen_snapshot(rt, files, code, build)
}

pub fn get_js_files(base_dir: &str, paths: &[&str]) -> Vec<PathBuf> {
    let mut files: Vec<_> = paths
        .iter()
        .flat_map(|p| {
            glob::glob(format!("{}/{}", base_dir, p).as_str())
                .unwrap()
                .filter_map(Result::ok)
                .collect::<Vec<_>>()
        })
        .collect();

    files.sort();
    files
}

fn _gen_snapshot(
    mut rt: JsRuntime,
    files: &[PathBuf],
    code: Option<String>,
    build: bool,
) -> Result<Vec<u8>> {
    let base_dir = env!("CARGO_MANIFEST_DIR");
    let display_root = Path::new(base_dir).parent().unwrap();
    let mut all_files = get_js_files(base_dir, JS_PATHS);
    all_files.extend_from_slice(files);
    for file in all_files {
        if build {
            println!("cargo:rerun-if-changed={}", file.display());
        }
        let display_path = file.strip_prefix(display_root).unwrap();
        let display_path_str = display_path.display().to_string();
        rt.execute_script(
            &("deno:".to_string() + &display_path_str.replace('\\', "/")),
            &std::fs::read_to_string(&file).unwrap(),
        )?;
    }
    if let Some(v) = code {
        rt.execute_script("deno:main", &v)?;
    }

    let snapshot = rt.snapshot();
    let snapshot_slice: &[u8] = &*snapshot;

    Ok(zstd::encode_all(snapshot_slice, 7)?)
}
