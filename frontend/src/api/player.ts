import { invoke } from "@tauri-apps/api/core";

// Week 1 contract: volume is fixed to 0~100 across backend/frontend.
export type PlayerState = {
  state: string;
  position: number;
  duration: number;
  volume: number;
};

export type PlaylistState = {
  items: string[];
  current_index: number | null;
};

export type AppFatalError = {
  stage: string;
  code: string;
  message: string;
  suggestion: string;
};

export type PluginInfo = {
  name: string;
  version: string;
  enabled: boolean;
  error_count: number;
  last_error: string | null;
  ui_button_label: string | null;
  ui_button_icon: string | null;
  ui_popup_width: number | null;
  ui_popup_height: number | null;
};

export async function playFile(path: string): Promise<string> {
  return invoke("play_file", { path });
}

export async function pickAndPlayFile(): Promise<string | null> {
  return invoke("pick_and_play_file");
}

export async function pickSubtitleFile(): Promise<string | null> {
  return invoke("pick_subtitle_file");
}

export async function pause(): Promise<void> {
  return invoke("pause");
}

export async function resume(): Promise<void> {
  return invoke("resume");
}

export async function seek(position: number): Promise<void> {
  return invoke("seek", { position });
}

export async function setVolume(volume: number): Promise<void> {
  return invoke("set_volume", { volume });
}

export async function getPlayerState(): Promise<PlayerState> {
  return invoke("get_player_state");
}

export async function getPlaylistState(): Promise<PlaylistState> {
  return invoke("get_playlist_state");
}

export async function listPlugins(): Promise<PluginInfo[]> {
  return invoke("list_plugins");
}

export async function getStartupFatalError(): Promise<AppFatalError | null> {
  return invoke("get_startup_fatal_error");
}

export async function retryStartupProbe(): Promise<AppFatalError | null> {
  return invoke("retry_startup_probe");
}

export async function emitDebugVideoError(): Promise<void> {
  return invoke("emit_debug_video_error");
}

export async function emitDebugFatalError(): Promise<void> {
  return invoke("emit_debug_fatal_error");
}

export async function openLogDirectory(): Promise<string> {
  return invoke("open_log_directory");
}

export async function getLogDirectory(): Promise<string> {
  return invoke("get_log_directory");
}

export async function saveFatalDiagnosticReport(report: string): Promise<string> {
  return invoke("save_fatal_diagnostic_report", { report });
}

export async function playlistNext(): Promise<string | null> {
  return invoke("playlist_next");
}

export async function playlistPrev(): Promise<string | null> {
  return invoke("playlist_prev");
}

export async function togglePlugin(name: string, enabled: boolean): Promise<void> {
  return invoke("toggle_plugin", { name, enabled });
}

export async function getPluginDetail(name: string): Promise<PluginInfo> {
  return invoke("get_plugin_detail", { name });
}

export async function captureScreenshot(): Promise<string> {
  return invoke("capture_screenshot");
}

export type ChapterEntry = {
  id: number;
  title: string;
  start: number;
  end: number;
};

export async function listChapters(): Promise<ChapterEntry[]> {
  return invoke("list_chapters");
}

export type SubtitleSearchResult = {
  name: string;
  language: string;
  format: string;
  source: string;
  path: string;
};

export async function searchSubtitles(query?: string): Promise<SubtitleSearchResult[]> {
  return invoke("search_subtitles", { query: query ?? null });
}

export async function downloadSubtitle(sourcePath: string): Promise<string> {
  return invoke("download_subtitle", { sourcePath });
}

export type MediaInfoResult = {
  container: string;
  duration: string;
  size: string;
  bit_rate: string;
  video_codec: string;
  video_resolution: string;
  video_bitrate: string;
  video_fps: string;
  audio_codec: string;
  audio_channels: string;
  audio_sample_rate: string;
};

export async function getMediaInfo(): Promise<MediaInfoResult> {
  return invoke("get_media_info");
}

export type BookmarkEntry = {
  id: string;
  name: string;
  video: string;
  position: number;
  created_at: number;
};

export async function listBookmarks(): Promise<BookmarkEntry[]> {
  return invoke("list_bookmarks");
}

export async function addBookmark(name: string): Promise<BookmarkEntry> {
  return invoke("add_bookmark", { name });
}

export async function deleteBookmark(id: string): Promise<void> {
  return invoke("delete_bookmark", { id });
}

export type PlayerSettings = {
  volume: number;
  playback_speed: number;
  window_size: { width: number; height: number } | null;
  last_playlist: string[];
  last_playlist_index: number | null;
  last_position: number;
  preferred_subtitle_lang: string | null;
  preferred_audio_lang: string | null;
};

export async function loadPlayerSettings(): Promise<PlayerSettings> {
  return invoke("load_player_settings");
}

export async function savePlayerSettings(settings: PlayerSettings): Promise<void> {
  return invoke("save_player_settings", { settings });
}

export async function getPlayerSettings(): Promise<PlayerSettings> {
  return invoke("get_player_settings");
}

export async function updatePlayerSettings(settings: PlayerSettings): Promise<void> {
  return invoke("update_player_settings", { settings });
}
