use serde::{Deserialize, Serialize};
use tauri::State;

use crate::ipc::state::AppState;
use crate::plugin::PluginEvent;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaInfoResult {
    pub container: String,
    pub duration: String,
    pub size: String,
    pub bit_rate: String,
    pub video_codec: String,
    pub video_resolution: String,
    pub video_bitrate: String,
    pub video_fps: String,
    pub audio_codec: String,
    pub audio_channels: String,
    pub audio_sample_rate: String,
}

fn format_duration(secs_str: &str) -> String {
    let secs: f64 = secs_str.parse().unwrap_or(0.0);
    let h = (secs / 3600.0) as u32;
    let m = ((secs % 3600.0) / 60.0) as u32;
    let s = secs % 60.0;
    format!("{:02}:{:02}:{:05.2}", h, m, s)
}

fn format_size(bytes_str: &str) -> String {
    let bytes: f64 = bytes_str.parse().unwrap_or(0.0);
    if bytes >= 1_073_741_824.0 {
        format!("{:.2} GiB", bytes / 1_073_741_824.0)
    } else if bytes >= 1_048_576.0 {
        format!("{:.2} MiB", bytes / 1_048_576.0)
    } else {
        format!("{:.0} B", bytes)
    }
}

fn format_bitrate(bps_str: &str) -> String {
    let bps: f64 = bps_str.parse().unwrap_or(0.0);
    if bps >= 1_000_000.0 {
        format!("{:.0} Mbps", bps / 1_000_000.0)
    } else if bps > 0.0 {
        format!("{:.0} Kbps", bps / 1_000.0)
    } else {
        "N/A".to_string()
    }
}

fn parse_fps(avg_frame_rate: &str) -> String {
    let parts: Vec<&str> = avg_frame_rate.split('/').collect();
    if parts.len() == 2 {
        let num: f64 = parts[0].parse().unwrap_or(0.0);
        let den: f64 = parts[1].parse().unwrap_or(1.0);
        if den > 0.0 {
            format!("{:.2} fps", num / den)
        } else {
            avg_frame_rate.to_string()
        }
    } else {
        avg_frame_rate.to_string()
    }
}

/// Tauri command: run ffprobe on the current video and return parsed metadata.
#[tauri::command]
pub async fn get_media_info(
    app_state: State<'_, AppState>,
) -> Result<MediaInfoResult, String> {
    let video_path = {
        let playlist = app_state.playlist.lock().map_err(|e| format!("{e}"))?;
        let idx = playlist
            .current_index
            .ok_or_else(|| "no video is playing".to_string())?;
        playlist
            .items
            .get(idx)
            .cloned()
            .ok_or_else(|| "invalid playlist index".to_string())?
    };

    let output = std::process::Command::new("ffprobe")
        .arg("-v")
        .arg("quiet")
        .arg("-print_format")
        .arg("json")
        .arg("-show_format")
        .arg("-show_streams")
        .arg(&video_path)
        .output()
        .map_err(|e| format!("ffprobe failed: {e}"))?;

    if !output.status.success() {
        return Err("ffprobe returned non-zero exit status".to_string());
    }

    let json: serde_json::Value = serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("parse ffprobe output: {e}"))?;

    let format_val = &json["format"];
    let streams = json["streams"].as_array().cloned().unwrap_or_default();

    let video_stream = streams.iter().find(|s| s["codec_type"] == "video");
    let audio_stream = streams.iter().find(|s| s["codec_type"] == "audio");

    // Emit event to media-info plugin for tracking.
    {
        let mut bus = app_state.plugin_bus.lock().map_err(|e| format!("{e}"))?;
        let event =
            PluginEvent::with_payload("mediainfo:request", serde_json::json!({ "video": video_path }));
        let _report = bus.emit(&event);
    }

    Ok(MediaInfoResult {
        container: format_val["format_name"]
            .as_str()
            .unwrap_or("?")
            .to_string(),
        duration: format_duration(format_val["duration"].as_str().unwrap_or("0")),
        size: format_size(format_val["size"].as_str().unwrap_or("0")),
        bit_rate: format_bitrate(format_val["bit_rate"].as_str().unwrap_or("0")),
        video_codec: video_stream
            .map(|s| {
                s["codec_name"]
                    .as_str()
                    .unwrap_or("?")
                    .to_string()
            })
            .unwrap_or_else(|| "N/A".to_string()),
        video_resolution: video_stream
            .map(|s| {
                let w = s["width"].as_u64().unwrap_or(0);
                let h = s["height"].as_u64().unwrap_or(0);
                format!("{}x{}", w, h)
            })
            .unwrap_or_else(|| "N/A".to_string()),
        video_bitrate: video_stream
            .and_then(|s| s["bit_rate"].as_str())
            .map(format_bitrate)
            .unwrap_or_else(|| "N/A".to_string()),
        video_fps: video_stream
            .and_then(|s| s["avg_frame_rate"].as_str())
            .map(parse_fps)
            .unwrap_or_else(|| "N/A".to_string()),
        audio_codec: audio_stream
            .map(|s| {
                s["codec_name"]
                    .as_str()
                    .unwrap_or("?")
                    .to_string()
            })
            .unwrap_or_else(|| "N/A".to_string()),
        audio_channels: audio_stream
            .map(|s| {
                s["channels"]
                    .as_u64()
                    .unwrap_or(0)
                    .to_string()
            })
            .unwrap_or_else(|| "N/A".to_string()),
        audio_sample_rate: audio_stream
            .and_then(|s| s["sample_rate"].as_str())
            .map(|r| format!("{} Hz", r))
            .unwrap_or_else(|| "N/A".to_string()),
    })
}
