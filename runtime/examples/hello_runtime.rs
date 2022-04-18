// Copyright 2018-2022 the Deno authors. All rights reserved. MIT license.

use deno_core::error::AnyError;
use deno_core::FsModuleLoader;
use deno_simple_runtime::permissions::Permissions;
use deno_simple_runtime::worker::MainWorker;
use deno_simple_runtime::WorkerOptionsBuilder;
use std::path::Path;
use std::rc::Rc;

fn get_error_class_name(e: &AnyError) -> &'static str {
    deno_simple_runtime::errors::get_error_class_name(e).unwrap_or("Error")
}

#[tokio::main]
async fn main() -> Result<(), AnyError> {
    let js_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/hello_runtime.js");
    let permissions = Permissions::allow_all();

    let options = WorkerOptionsBuilder::default()
        .main_module(Some(&js_path.to_string_lossy()))
        .permissions(permissions)
        .module_loader(Rc::new(FsModuleLoader))
        .get_error_class_fn(Some(&get_error_class_name))
        .build()
        .unwrap();

    let mut worker = MainWorker::bootstrap_from_options(options, vec![]);
    worker.execute_main_module().await?;
    worker.run_event_loop(false).await?;
    Ok(())
}
