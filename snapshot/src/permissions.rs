use deno_ffi::FfiPermissions;
use deno_net::NetPermissions;
use deno_web::TimersPermission;
use deno_websocket::WebSocketPermissions;
use std::path::Path;

pub struct Permissions;

impl deno_fetch::FetchPermissions for Permissions {
    fn check_net_url(
        &mut self,
        _url: &deno_core::url::Url,
    ) -> Result<(), deno_core::error::AnyError> {
        unreachable!("snapshotting!")
    }

    fn check_read(&mut self, _p: &Path) -> Result<(), deno_core::error::AnyError> {
        unreachable!("snapshotting!")
    }
}

impl WebSocketPermissions for Permissions {
    fn check_net_url(
        &mut self,
        _url: &deno_core::url::Url,
    ) -> Result<(), deno_core::error::AnyError> {
        unreachable!("snapshotting!")
    }
}

impl TimersPermission for Permissions {
    fn allow_hrtime(&mut self) -> bool {
        unreachable!("snapshotting!")
    }

    fn check_unstable(&self, _state: &deno_core::OpState, _api_name: &'static str) {
        unreachable!("snapshotting!")
    }
}

impl FfiPermissions for Permissions {
    fn check(&mut self, _path: Option<&Path>) -> Result<(), deno_core::error::AnyError> {
        unreachable!("snapshotting!")
    }
}

impl NetPermissions for Permissions {
    fn check_net<T: AsRef<str>>(
        &mut self,
        _host: &(T, Option<u16>),
    ) -> Result<(), deno_core::error::AnyError> {
        unreachable!("snapshotting!")
    }

    fn check_read(&mut self, _p: &Path) -> Result<(), deno_core::error::AnyError> {
        unreachable!("snapshotting!")
    }

    fn check_write(&mut self, _p: &Path) -> Result<(), deno_core::error::AnyError> {
        unreachable!("snapshotting!")
    }
}
