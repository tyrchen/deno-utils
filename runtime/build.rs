// Copyright 2018-2022 the Deno authors. All rights reserved. MIT license.

use std::env;
#[allow(unused_imports)]
use std::path::Path;
use std::path::PathBuf;

// This is a shim that allows to generate documentation on docs.rs
#[cfg(not(feature = "docsrs"))]
mod not_docs {
    use deno_snapshot::create_snapshot;
    use std::path::PathBuf;

    pub fn build_snapshot(filename: PathBuf) {
        let data = create_snapshot(vec![], &[]).unwrap();
        std::fs::write(filename, data).unwrap();
    }
}

fn main() {
    // To debug snapshot issues uncomment:
    // op_fetch_asset::trace_serializer();

    println!("cargo:rustc-env=TARGET={}", env::var("TARGET").unwrap());
    println!("cargo:rustc-env=PROFILE={}", env::var("PROFILE").unwrap());
    let o = PathBuf::from(env::var_os("OUT_DIR").unwrap());

    // Main snapshot
    let runtime_snapshot_path = o.join("CLI_SNAPSHOT.bin");

    // If we're building on docs.rs we just create
    // and empty snapshot file and return, because `rusty_v8`
    // doesn't actually compile on docs.rs
    if env::var_os("DOCS_RS").is_some() {
        let snapshot_slice = &[];
        std::fs::write(&runtime_snapshot_path, snapshot_slice).unwrap();
    }

    #[cfg(not(feature = "docsrs"))]
    not_docs::build_snapshot(runtime_snapshot_path)
}
