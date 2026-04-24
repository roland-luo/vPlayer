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
};

export async function playFile(path: string): Promise<string> {
  return invoke("play_file", { path });
}

export async function pickAndPlayFile(): Promise<string | null> {
  return invoke("pick_and_play_file");
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
