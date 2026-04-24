use tauri::{AppHandle, Runtime};

use crate::ipc::events::{self, VideoErrorPayload};

#[derive(Debug, Clone)]
pub struct RenderError {
    pub code: String,
    pub message: String,
}

impl RenderError {
    fn texture_alloc_failed(message: impl Into<String>) -> Self {
        Self {
            code: "GL_TEXTURE_ALLOC_FAILED".to_string(),
            message: message.into(),
        }
    }
}

fn should_force_texture_alloc_failure() -> bool {
    std::env::var("VPLAYER_FORCE_RENDER_FATAL")
        .ok()
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

fn emit_video_error_event<R: Runtime>(app: &AppHandle<R>, render_error: &RenderError) -> Result<(), String> {
    let payload = VideoErrorPayload {
        code: render_error.code.clone(),
        message: render_error.message.clone(),
    };
    events::emit_video_error(app, &payload)
}

pub fn render_frame<R: Runtime>(app: &AppHandle<R>, width: u32, height: u32) -> Result<(), String> {
    if width == 0 || height == 0 {
        let err = RenderError::texture_alloc_failed(format!(
            "Texture allocation failed because render target size is invalid: {width}x{height}."
        ));
        emit_video_error_event(app, &err)?;
        return Err(format!("render_frame failed: {}", err.message));
    }

    if should_force_texture_alloc_failure() {
        let err = RenderError::texture_alloc_failed(
            "Texture allocation failed in forced debug mode (VPLAYER_FORCE_RENDER_FATAL=1).",
        );
        emit_video_error_event(app, &err)?;
        return Err(format!("render_frame failed: {}", err.message));
    }

    Ok(())
}

pub fn simulate_texture_alloc_failure<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    render_frame(app, 0, 1080)
}
