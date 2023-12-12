use self::ipc::{GetUserErrorCode, LoginErrorCode, LogoutErrorCode, SelectHandleErrorCode, User};

tauri_bindgen_host::generate!({
    path: "ipc.wit",
    async: false,
    tracing: true,
});

#[derive(Clone, Copy)]
struct IpcCtx;

impl ipc::Ipc for IpcCtx {
    fn login(
        &self,
        username: String,
        password: String,
        code: Option<String>,
    ) -> Option<LoginErrorCode> {
        todo!()
    }

    fn logout(&self) -> Option<LogoutErrorCode> {
        todo!()
    }

    fn get_user(&self) -> Result<User, GetUserErrorCode> {
        todo!()
    }

    fn select_handle(&self, handle: String) -> Option<SelectHandleErrorCode> {
        todo!()
    }
}
