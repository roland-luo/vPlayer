use tauri::AppHandle;

use crate::ipc::{
    events::{self, AppFatalErrorPayload},
    state::{AppState, StartupFatalState},
};
use crate::mpv::core::StartupError;

pub fn emit_startup_fatal_error(
    app: &AppHandle,
    stage: &str,
    code: &str,
    message: &str,
    suggestion: &str,
) -> Result<(), String> {
    let payload = AppFatalErrorPayload {
        stage: stage.to_string(),
        code: code.to_string(),
        message: message.to_string(),
        suggestion: suggestion.to_string(),
    };
    events::emit_app_fatal_error(app, &payload)
}

pub fn handle_startup_error(
    app: &AppHandle,
    app_state: &AppState,
    startup_error: &StartupError,
) -> Result<(), String> {
    let fatal_state = StartupFatalState {
        stage: startup_error.stage.clone(),
        code: startup_error.code.clone(),
        message: startup_error.message.clone(),
        suggestion: startup_error.suggestion.clone(),
    };

    {
        let mut stored = app_state
            .startup_fatal
            .lock()
            .map_err(|e| format!("startup fatal lock poisoned: {e}"))?;
        *stored = Some(fatal_state.clone());
    }

    emit_startup_fatal_error(
        app,
        &fatal_state.stage,
        &fatal_state.code,
        &fatal_state.message,
        &fatal_state.suggestion,
    )
}
