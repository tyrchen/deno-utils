// Copyright 2018-2022 the Deno authors. All rights reserved. MIT license.
#![allow(clippy::derive_partial_eq_without_eq)]

pub use deno_broadcast_channel;
pub use deno_console;
pub use deno_core;
pub use deno_crypto;
pub use deno_fetch;
#[cfg(feature = "ext_ffi")]
pub use deno_ffi;
pub use deno_http;
pub use deno_net;
pub use deno_node;
pub use deno_tls;
pub use deno_url;
pub use deno_web;
#[cfg(feature = "ext_webgpu")]
pub use deno_webgpu;
pub use deno_webidl;
pub use deno_websocket;
pub use deno_webstorage;

pub mod colors;
pub mod errors;
pub mod fs_util;
pub mod inspector_server;
pub mod js;
pub mod ops;
pub mod permissions;
pub mod tokio_util;
pub mod web_worker;
pub mod worker;

mod patch;
mod worker_bootstrap;
pub use worker_bootstrap::BootstrapOptions;

pub use patch::StartSnapshot;
pub use worker::{MainWorker, WorkerOptions, WorkerOptionsBuilder, WorkerOptionsBuilderError};

#[cfg(test)]
pub mod test_util {
    use crate::{permissions::Permissions, MainWorker, WorkerOptionsBuilder};
    use std::path::PathBuf;

    pub fn testdata_path(name: &str) -> String {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let path = path.join(format!("../fixtures/testdata/{}", name));
        path.to_string_lossy().into()
    }

    pub fn create_test_worker(file: impl AsRef<str>) -> MainWorker {
        let options = WorkerOptionsBuilder::default()
            .main_module(Some(file.as_ref()))
            .permissions(Permissions::allow_all())
            .build()
            .unwrap();

        MainWorker::bootstrap_from_options(options, vec![])
    }
}
