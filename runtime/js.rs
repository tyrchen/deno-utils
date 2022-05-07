// Copyright 2018-2022 the Deno authors. All rights reserved. MIT license.
use deno_core::Snapshot;
use log::debug;
use once_cell::sync::Lazy;

pub static CLI_SNAPSHOT: Lazy<Box<[u8]>> = Lazy::new(
    #[allow(clippy::uninit_vec)]
    #[cold]
    #[inline(never)]
    || {
        static COMPRESSED_CLI_SNAPSHOT: &[u8] =
            include_bytes!(concat!(env!("OUT_DIR"), "/CLI_SNAPSHOT.bin"));

        deno_snapshot::decode(COMPRESSED_CLI_SNAPSHOT)
    },
);

pub fn deno_isolate_init() -> Snapshot {
    debug!("Deno isolate init with snapshots.");
    Snapshot::Static(&*CLI_SNAPSHOT)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_snapshot() {
        let mut js_runtime = deno_core::JsRuntime::new(deno_core::RuntimeOptions {
            startup_snapshot: Some(deno_isolate_init()),
            ..Default::default()
        });
        js_runtime
            .execute_script(
                "<anon>",
                r#"
      if (!(bootstrap.mainRuntime && bootstrap.workerRuntime)) {
        throw Error("bad");
      }
      console.log("we have console.log!!!");
    "#,
            )
            .unwrap();
    }
}
