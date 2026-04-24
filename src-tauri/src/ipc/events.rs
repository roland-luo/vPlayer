#![allow(dead_code)]

use serde::Serialize;
use tauri::{AppHandle, Emitter, Runtime};

pub const EVT_PLAYER_STATE_CHANGE: &str = "player:state_change";
pub const EVT_PLAYER_PROGRESS: &str = "player:progress";
pub const EVT_VIDEO_ERROR: &str = "video:error";
pub const EVT_APP_FATAL_ERROR: &str = "app:fatal_error";

#[derive(Debug, Clone, Serialize)]
pub struct PlayerStateChangePayload {
    pub state: String,
    pub position: f64,
    pub duration: f64,
    /// Volume range is fixed to 0~100 in Week 1.
    pub volume: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlayerProgressPayload {
    pub position: f64,
    pub duration: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct VideoErrorPayload {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AppFatalErrorPayload {
    pub stage: String,
    pub code: String,
    pub message: String,
    pub suggestion: String,
}

fn emit_event<R: Runtime, T: Serialize>(
    app: &AppHandle<R>,
    event_name: &str,
    payload: &T,
) -> Result<(), String> {
    app.emit(event_name, payload)
        .map_err(|e| format!("emit {event_name} failed: {e}"))
}

pub fn emit_player_state_change<R: Runtime>(
    app: &AppHandle<R>,
    payload: &PlayerStateChangePayload,
) -> Result<(), String> {
    emit_event(app, EVT_PLAYER_STATE_CHANGE, payload)
}

pub fn emit_player_progress<R: Runtime>(
    app: &AppHandle<R>,
    payload: &PlayerProgressPayload,
) -> Result<(), String> {
    emit_event(app, EVT_PLAYER_PROGRESS, payload)
}

pub fn emit_video_error<R: Runtime>(
    app: &AppHandle<R>,
    payload: &VideoErrorPayload,
) -> Result<(), String> {
    emit_event(app, EVT_VIDEO_ERROR, payload)
}

pub fn emit_app_fatal_error<R: Runtime>(
    app: &AppHandle<R>,
    payload: &AppFatalErrorPayload,
) -> Result<(), String> {
    emit_event(app, EVT_APP_FATAL_ERROR, payload)
}
