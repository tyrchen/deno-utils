mod universal_loader;

use crate::ModuleStore;
use data_url::DataUrl;
use deno_core::{anyhow::bail, error::AnyError, ModuleSpecifier};
use std::sync::Arc;

#[derive(Clone)]
pub struct UniversalModuleLoader {
    store: Option<Arc<dyn ModuleStore>>,
    compile: bool,
}

pub async fn get_source_code(m: &ModuleSpecifier) -> Result<String, AnyError> {
    let code = match m.scheme() {
        "http" | "https" => {
            let res = reqwest::get(m.to_owned()).await?;
            // TODO: The HTML spec says to fail if the status is not
            // 200-299, but `error_for_status()` fails if the status is
            // 400-599.
            let res = res.error_for_status()?;
            res.text().await?
        }
        "file" => {
            let path = match m.to_file_path() {
                Ok(path) => path,
                Err(_) => bail!("Invalid file URL."),
            };
            tokio::fs::read_to_string(path).await?
        }
        "data" => {
            let url = match DataUrl::process(m.as_str()) {
                Ok(url) => url,
                Err(_) => bail!("Not a valid data URL."),
            };
            let bytes = match url.decode_to_vec() {
                Ok((bytes, _)) => bytes,
                Err(_) => bail!("Not a valid data URL."),
            };
            match String::from_utf8(bytes) {
                Ok(s) => s,
                Err(_) => bail!("Not a valid data URL code."),
            }
        }
        schema => bail!("Invalid schema {}", schema),
    };
    Ok(code)
}
