use std::sync::OnceLock;
use tauri::{AppHandle, Emitter};

static APP_HANDLE: OnceLock<AppHandle> = OnceLock::new();

pub fn init_app_handle(handle: AppHandle) {
    APP_HANDLE
        .set(handle)
        .unwrap_or_else(|_| eprintln!("[app_handle] already initialised"));
}

pub fn get_app_handle() -> &'static AppHandle {
    APP_HANDLE
        .get()
        .expect("[app_handle] not initialised — call init_app_handle() first")
}

pub fn emit<T: serde::Serialize + Clone>(event: &str, payload: T) {
    if let Some(handle) = APP_HANDLE.get() {
        if let Err(e) = handle.emit(event, payload) {
            eprintln!("[app_handle] emit error on '{event}': {e}");
        }
    } else {
        eprintln!("[app_handle] tried to emit '{event}' before init");
    }
}
