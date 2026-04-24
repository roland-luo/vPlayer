<template>
  <div class="player-app">
    <TopHud :title="currentTitle" :meta="mediaMeta" :is-playing="isPlaying" :playlist-status="playlistStatus"
      @open-file="handleOpenFile" @debug-error="handleDebugError" @debug-fatal="handleDebugFatal"
      @prev-file="handlePrevFile" @next-file="handleNextFile" />
    <PlayerView ref="playerViewRef" :is-playing="isPlaying" :source-path="currentMediaPath" :volume="volume"
      @play="handlePlay" @pause="handlePause" @progress="handleViewProgress" @loaded-metadata="handleLoadedMetadata"
      @ended="handleViewEnded" @video-error="handleVideoElementError" />
    <ControlBar :is-playing="isPlaying" :current-time="currentTime" :duration="duration" :volume="volume"
      @toggle-play="togglePlay" @seek="handleSeek" @volume-change="handleVolumeChange" />
    <div v-if="videoError" class="video-error-overlay">
      <div class="video-error-card">
        <div class="video-error-title">Video Error</div>
        <div class="video-error-code">{{ videoError.code }}</div>
        <div class="video-error-message">{{ videoError.message }}</div>
        <button class="video-error-dismiss" @click="clearVideoError">Dismiss</button>
      </div>
    </div>
    <div v-if="fatalError" class="fatal-error-overlay">
      <div class="fatal-error-card">
        <div class="fatal-error-title">Fatal Startup Error</div>
        <div class="fatal-error-code">{{ fatalError.code }}</div>
        <div class="fatal-error-message">{{ fatalError.message }}</div>
        <div class="fatal-error-meta">Stage: {{ fatalError.stage }}</div>
        <div class="fatal-error-meta">Suggestion: {{ fatalError.suggestion }}</div>
        <div v-if="logDirectory" class="fatal-error-meta">Logs: {{ logDirectory }}</div>
        <div v-if="retryFailureCount > 0" class="fatal-error-meta">
          Retry Failures: {{ retryFailureCount }}
        </div>
        <div v-if="lastRetryFailureLabel" class="fatal-error-meta">
          Last Retry Failure: {{ lastRetryFailureLabel }}
        </div>
        <div class="fatal-error-actions">
          <button class="fatal-error-btn" @click="copyFatalErrorInfo">Copy Error Info</button>
          <button class="fatal-error-btn" @click="copyFatalDiagnosticReport">
            Copy Full Report
          </button>
          <button class="fatal-error-btn" @click="saveFatalDiagnosticReportToFile">
            Save Report
          </button>
          <button class="fatal-error-btn" :disabled="retryingStartup" @click="handleRetryStartup">
            {{ retryingStartup ? "Retrying..." : "Retry Startup" }}
          </button>
          <button class="fatal-error-btn" @click="copyLogDirectory">Copy Log Path</button>
          <button class="fatal-error-btn" @click="handleOpenLogsDir">Open Logs</button>
          <button class="fatal-error-dismiss" @click="clearFatalError">Dismiss</button>
        </div>
        <div v-if="fatalActionHint" class="fatal-error-hint">{{ fatalActionHint }}</div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, type ComponentPublicInstance } from "vue";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import TopHud from "./components/TopHud.vue";
import PlayerView from "./components/PlayerView.vue";
import ControlBar from "./components/ControlBar.vue";
import {
  type AppFatalError,
  emitDebugVideoError,
  emitDebugFatalError,
  getLogDirectory,
  getPlaylistState,
  getPlayerState,
  getStartupFatalError,
  openLogDirectory,
  pause,
  pickAndPlayFile,
  playlistNext,
  playlistPrev,
  retryStartupProbe,
  resume,
  saveFatalDiagnosticReport,
  seek,
  setVolume,
  type PlaylistState,
  type PlayerState,
} from "./api/player";

const playerState = ref<PlayerState>({
  state: "idle",
  position: 0,
  duration: 0,
  volume: 100,
});
const currentTitle = ref("vPlayer");
const mediaMeta = ref({
  codec: "--",
  resolution: "--",
  fps: "--",
  audio: "--",
});
const currentMediaPath = ref("");
const videoError = ref<{ code: string; message: string } | null>(null);
const fatalError = ref<AppFatalError | null>(null);
const fatalActionHint = ref("");
const retryingStartup = ref(false);
const retryFailureCount = ref(0);
const lastRetryFailureAt = ref<number | null>(null);
const logDirectory = ref("");
const listenerStateChangeReady = ref(false);
const listenerProgressReady = ref(false);
const listenerVideoErrorReady = ref(false);
const listenerFatalErrorReady = ref(false);
const lastPlayerStateChangeAt = ref<number | null>(null);
const lastPlayerProgressAt = ref<number | null>(null);
const lastVideoErrorAt = ref<number | null>(null);
const lastFatalErrorAt = ref<number | null>(null);
const playlistState = ref<PlaylistState>({
  items: [],
  current_index: null,
});
const rafId = ref<number | null>(null);
const playbackAnchorPosition = ref(0);
const playbackAnchorTs = ref(0);
type PlayerViewExpose = ComponentPublicInstance & {
  seekTo: (position: number) => void;
  tryPlay: () => Promise<void>;
};
const playerViewRef = ref<PlayerViewExpose | null>(null);

const isPlaying = computed(() => playerState.value.state === "playing");
const isIdle = computed(() => playerState.value.state === "idle");
const currentTime = computed(() => playerState.value.position);
const duration = computed(() => playerState.value.duration);
const volume = computed(() => playerState.value.volume);
const playlistStatus = computed(() => {
  const total = playlistState.value.items.length;
  if (total === 0) return "0 / 0";
  const current = (playlistState.value.current_index ?? 0) + 1;
  return `${current} / ${total}`;
});
const lastRetryFailureLabel = computed(() => {
  if (!lastRetryFailureAt.value) return "";
  return new Date(lastRetryFailureAt.value).toLocaleString();
});

const isTauriRuntime =
  typeof window !== "undefined" &&
  ("__TAURI_INTERNALS__" in window || "__TAURI__" in window);

function applyPlayerState(next: Partial<PlayerState>) {
  playerState.value = {
    ...playerState.value,
    ...next,
    volume: Math.max(0, Math.min(100, next.volume ?? playerState.value.volume)),
  };
}

function clearVideoError() {
  videoError.value = null;
}

function setCurrentMediaPath(path: string) {
  currentMediaPath.value = path;
  clearVideoError();
}

function clearFatalError() {
  fatalError.value = null;
  fatalActionHint.value = "";
  retryFailureCount.value = 0;
  lastRetryFailureAt.value = null;
}

function formatEventTimestamp(ts: number | null) {
  if (!ts) return "(none)";
  return new Date(ts).toLocaleString();
}

function stopUiCompensation() {
  if (rafId.value !== null) {
    cancelAnimationFrame(rafId.value);
    rafId.value = null;
  }
}

function startUiCompensation() {
  stopUiCompensation();

  const loop = () => {
    if (!isPlaying.value) {
      stopUiCompensation();
      return;
    }

    const now = performance.now();
    const elapsedSeconds = (now - playbackAnchorTs.value) / 1000;
    let nextPosition = playbackAnchorPosition.value + elapsedSeconds;

    if (duration.value > 0) {
      nextPosition = Math.min(nextPosition, duration.value);
    }

    applyPlayerState({ position: nextPosition });
    rafId.value = requestAnimationFrame(loop);
  };

  playbackAnchorPosition.value = currentTime.value;
  playbackAnchorTs.value = performance.now();
  rafId.value = requestAnimationFrame(loop);
}

async function handlePlay() {
  if (isIdle.value) {
    await handleOpenFile();
    return;
  }

  if (!isTauriRuntime) {
    applyPlayerState({ state: "playing" });
    return;
  }
  await resume();
}

async function handlePause() {
  if (!isTauriRuntime) {
    applyPlayerState({ state: "paused" });
    return;
  }
  await pause();
}

async function togglePlay() {
  if (isPlaying.value) {
    await handlePause();
  } else {
    await handlePlay();
  }
}

async function handleSeek(position: number) {
  playerViewRef.value?.seekTo(position);
  applyPlayerState({ position });
  if (!isTauriRuntime) {
    return;
  }
  await seek(position);
}

async function handleVolumeChange(v: number) {
  if (!isTauriRuntime) {
    applyPlayerState({ volume: v });
    return;
  }
  await setVolume(v);
}

async function handleOpenFile() {
  if (!isTauriRuntime) return;
  try {
    const selectedPath = await pickAndPlayFile();
    if (!selectedPath) return;
    setCurrentMediaPath(selectedPath);
    updateTitleByPath(selectedPath);
    await refreshPlaylistState();
    await ensureVideoAutoplay();
  } catch (error) {
    console.debug("[open-file] failed to select/play file", error);
  }
}

function updateTitleByPath(path: string) {
  const parts = path.split(/[\\/]/);
  currentTitle.value = parts[parts.length - 1] || path;
}

async function handleDebugError() {
  if (!isTauriRuntime) return;
  await emitDebugVideoError();
}

async function handleDebugFatal() {
  if (!isTauriRuntime) return;
  await emitDebugFatalError();
}

async function copyFatalErrorInfo() {
  if (!fatalError.value) return;
  const text = [
    `Code: ${fatalError.value.code}`,
    `Stage: ${fatalError.value.stage}`,
    `Message: ${fatalError.value.message}`,
    `Suggestion: ${fatalError.value.suggestion}`,
  ].join("\n");

  try {
    await navigator.clipboard.writeText(text);
    fatalActionHint.value = "Copied error info.";
  } catch {
    fatalActionHint.value = "Copy failed. Clipboard is not available.";
  }
}

function buildFatalDiagnosticReport() {
  if (!fatalError.value) return "";
  const lines = [
    "# vPlayer Fatal Diagnostic Report",
    `Timestamp: ${new Date().toISOString()}`,
    `Current Title: ${currentTitle.value}`,
    `Playlist Status: ${playlistStatus.value}`,
    `Media Codec: ${mediaMeta.value.codec}`,
    `Media Resolution: ${mediaMeta.value.resolution}`,
    `Media FPS: ${mediaMeta.value.fps}`,
    `Media Audio: ${mediaMeta.value.audio}`,
    `Runtime: ${isTauriRuntime ? "tauri" : "web"}`,
    `Video Error Present: ${videoError.value ? "yes" : "no"}`,
    `Video Error Code: ${videoError.value?.code || "(none)"}`,
    `Video Error Message: ${videoError.value?.message || "(none)"}`,
    `Listener player:state_change: ${listenerStateChangeReady.value ? "ready" : "not_ready"}`,
    `Listener player:progress: ${listenerProgressReady.value ? "ready" : "not_ready"}`,
    `Listener video:error: ${listenerVideoErrorReady.value ? "ready" : "not_ready"}`,
    `Listener app:fatal_error: ${listenerFatalErrorReady.value ? "ready" : "not_ready"}`,
    `Last Event player:state_change: ${formatEventTimestamp(lastPlayerStateChangeAt.value)}`,
    `Last Event player:progress: ${formatEventTimestamp(lastPlayerProgressAt.value)}`,
    `Last Event video:error: ${formatEventTimestamp(lastVideoErrorAt.value)}`,
    `Last Event app:fatal_error: ${formatEventTimestamp(lastFatalErrorAt.value)}`,
    `Retry In Progress: ${retryingStartup.value ? "yes" : "no"}`,
    `Fatal Hint: ${fatalActionHint.value || "(none)"}`,
    `Player State: ${playerState.value.state}`,
    `Playback Progress: ${playerState.value.position.toFixed(3)} / ${playerState.value.duration.toFixed(3)}`,
    `Volume: ${playerState.value.volume.toFixed(2)}`,
    `Code: ${fatalError.value.code}`,
    `Stage: ${fatalError.value.stage}`,
    `Message: ${fatalError.value.message}`,
    `Suggestion: ${fatalError.value.suggestion}`,
    `Log Directory: ${logDirectory.value || "(unknown)"}`,
    `Retry Failures: ${retryFailureCount.value}`,
    `Last Retry Failure: ${lastRetryFailureLabel.value || "(none)"}`,
    `User Agent: ${typeof navigator !== "undefined" ? navigator.userAgent : "unknown"}`,
  ];
  return lines.join("\n");
}

async function copyFatalDiagnosticReport() {
  if (!fatalError.value) return;
  const report = buildFatalDiagnosticReport();
  try {
    await navigator.clipboard.writeText(report);
    fatalActionHint.value = "Copied full diagnostic report.";
  } catch {
    fatalActionHint.value = "Copy failed. Clipboard is not available.";
  }
}

async function saveFatalDiagnosticReportToFile() {
  if (!fatalError.value) return;
  const report = buildFatalDiagnosticReport();
  try {
    const path = await saveFatalDiagnosticReport(report);
    fatalActionHint.value = `Saved and revealed report: ${path}`;
  } catch (error) {
    fatalActionHint.value = "Save report failed.";
    console.debug("[fatal-error] save report failed", error);
  }
}

async function copyLogDirectory() {
  if (!logDirectory.value) return;
  try {
    await navigator.clipboard.writeText(logDirectory.value);
    fatalActionHint.value = "Copied log path.";
  } catch {
    fatalActionHint.value = "Copy failed. Clipboard is not available.";
  }
}

async function handleOpenLogsDir() {
  if (!isTauriRuntime) return;
  try {
    const path = await openLogDirectory();
    logDirectory.value = path;
    fatalActionHint.value = `Opened logs: ${path}`;
  } catch (error) {
    fatalActionHint.value = "Open logs failed.";
    console.debug("[fatal-error] open logs failed", error);
  }
}

async function handleRetryStartup() {
  if (!isTauriRuntime || retryingStartup.value) return;
  retryingStartup.value = true;
  fatalActionHint.value = "Retrying startup probe...";
  try {
    const nextFatal = await retryStartupProbe();
    fatalError.value = nextFatal;
    if (nextFatal) {
      retryFailureCount.value += 1;
      lastRetryFailureAt.value = Date.now();
      fatalActionHint.value = "Startup probe still failing.";
      return;
    }
    fatalActionHint.value = "Startup probe recovered. Closing fatal overlay...";
    setTimeout(() => {
      clearFatalError();
    }, 900);
  } catch (error) {
    retryFailureCount.value += 1;
    lastRetryFailureAt.value = Date.now();
    fatalActionHint.value = "Retry startup failed.";
    console.debug("[fatal-error] retry startup failed", error);
  } finally {
    retryingStartup.value = false;
  }
}

async function handlePrevFile() {
  if (!isTauriRuntime) return;
  try {
    const path = await playlistPrev();
    if (path) {
      setCurrentMediaPath(path);
      updateTitleByPath(path);
    }
    await refreshPlaylistState();
    await ensureVideoAutoplay();
  } catch (error) {
    console.debug("[playlist] prev failed", error);
  }
}

async function handleNextFile() {
  if (!isTauriRuntime) return;
  try {
    const path = await playlistNext();
    if (path) {
      setCurrentMediaPath(path);
      updateTitleByPath(path);
    }
    await refreshPlaylistState();
    await ensureVideoAutoplay();
  } catch (error) {
    console.debug("[playlist] next failed", error);
  }
}

async function refreshPlaylistState() {
  if (!isTauriRuntime) return;
  playlistState.value = await getPlaylistState();
  const index = playlistState.value.current_index;
  if (index !== null && playlistState.value.items[index]) {
    setCurrentMediaPath(playlistState.value.items[index]);
  }
}

function handleViewProgress(payload: { position: number; duration: number }) {
  applyPlayerState(payload);
  playbackAnchorPosition.value = payload.position;
  playbackAnchorTs.value = performance.now();
}

function handleLoadedMetadata(payload: { duration: number; width: number; height: number }) {
  applyPlayerState({ duration: payload.duration });
  mediaMeta.value = {
    ...mediaMeta.value,
    codec: "HTML5",
    resolution: `${payload.width}x${payload.height}`,
    fps: "--",
    audio: "HTML5 Media",
  };
}

function handleViewEnded() {
  stopUiCompensation();
  applyPlayerState({ state: "paused" });
}

function handleVideoElementError(payload: { code: string; message: string }) {
  videoError.value = payload;
}

async function ensureVideoAutoplay() {
  await nextTick();
  try {
    await playerViewRef.value?.tryPlay();
  } catch (error) {
    console.debug("[autoplay] failed to start playback after file selection", error);
  }
}

const unlistenFns: UnlistenFn[] = [];

onMounted(async () => {
  if (!isTauriRuntime) return;

  try {
    applyPlayerState(await getPlayerState());
    await refreshPlaylistState();
    fatalError.value = await getStartupFatalError();
    logDirectory.value = await getLogDirectory();

    const unlistenState = await listen<PlayerState>("player:state_change", (event) => {
      applyPlayerState(event.payload);
      lastPlayerStateChangeAt.value = Date.now();
      if (event.payload.state === "playing") {
        clearVideoError();
        playbackAnchorPosition.value = event.payload.position;
        playbackAnchorTs.value = performance.now();
        startUiCompensation();
      } else {
        stopUiCompensation();
      }
    });
    unlistenFns.push(unlistenState);
    listenerStateChangeReady.value = true;

    const unlistenProgress = await listen<{ position: number; duration: number }>(
      "player:progress",
      (event) => {
        applyPlayerState(event.payload);
        lastPlayerProgressAt.value = Date.now();
        playbackAnchorPosition.value = event.payload.position;
        playbackAnchorTs.value = performance.now();
      },
    );
    unlistenFns.push(unlistenProgress);
    listenerProgressReady.value = true;

    const unlistenVideoError = await listen<{ code: string; message: string }>(
      "video:error",
      (event) => {
        videoError.value = event.payload;
        lastVideoErrorAt.value = Date.now();
      },
    );
    unlistenFns.push(unlistenVideoError);
    listenerVideoErrorReady.value = true;

    const unlistenFatalError = await listen<AppFatalError>("app:fatal_error", (event) => {
      fatalError.value = event.payload;
      lastFatalErrorAt.value = Date.now();
    });
    unlistenFns.push(unlistenFatalError);
    listenerFatalErrorReady.value = true;
  } catch (error) {
    console.debug("[state-sync] failed to initialize tauri event sync", error);
  }
});

onUnmounted(() => {
  stopUiCompensation();
  for (const unlisten of unlistenFns) {
    unlisten();
  }
  listenerStateChangeReady.value = false;
  listenerProgressReady.value = false;
  listenerVideoErrorReady.value = false;
  listenerFatalErrorReady.value = false;
  lastPlayerStateChangeAt.value = null;
  lastPlayerProgressAt.value = null;
  lastVideoErrorAt.value = null;
  lastFatalErrorAt.value = null;
});
</script>

<style scoped>
.player-app {
  width: 100%;
  height: 100%;
  position: relative;
  background: var(--bg-base);
  overflow: hidden;
}

.video-error-overlay {
  position: absolute;
  inset: 0;
  z-index: 30;
  background: rgba(8, 8, 12, 0.52);
  display: flex;
  align-items: center;
  justify-content: center;
}

.video-error-card {
  width: min(540px, 88vw);
  border: 1px solid rgba(255, 82, 82, 0.45);
  background: rgba(15, 10, 10, 0.9);
  border-radius: 10px;
  padding: 16px;
  color: #ffd6d6;
}

.video-error-title {
  font-size: 14px;
  font-weight: 700;
  margin-bottom: 8px;
  text-transform: uppercase;
  letter-spacing: 0.06em;
}

.video-error-code {
  font-family: var(--font-mono);
  font-size: 11px;
  color: #ff8a80;
  margin-bottom: 8px;
}

.video-error-message {
  font-size: 13px;
  line-height: 1.45;
  color: #ffd6d6;
  margin-bottom: 12px;
}

.video-error-dismiss {
  border: 1px solid rgba(255, 138, 128, 0.6);
  background: transparent;
  color: #ffb4ab;
  font-size: 12px;
  padding: 4px 10px;
  border-radius: 6px;
  cursor: pointer;
}

.fatal-error-overlay {
  position: absolute;
  inset: 0;
  z-index: 40;
  background: rgba(6, 6, 10, 0.94);
  display: flex;
  align-items: center;
  justify-content: center;
}

.fatal-error-card {
  width: min(620px, 90vw);
  border: 1px solid rgba(255, 138, 128, 0.5);
  background: rgba(20, 12, 12, 0.95);
  border-radius: 12px;
  padding: 20px;
  color: #ffd6d6;
}

.fatal-error-title {
  font-size: 16px;
  font-weight: 700;
  margin-bottom: 10px;
  text-transform: uppercase;
  letter-spacing: 0.06em;
}

.fatal-error-code {
  font-family: var(--font-mono);
  font-size: 12px;
  color: #ff8a80;
  margin-bottom: 8px;
}

.fatal-error-message {
  font-size: 13px;
  line-height: 1.45;
  margin-bottom: 8px;
}

.fatal-error-meta {
  font-family: var(--font-mono);
  font-size: 11px;
  color: #ffb4ab;
  margin-bottom: 6px;
}

.fatal-error-dismiss {
  border: 1px solid rgba(255, 138, 128, 0.6);
  background: transparent;
  color: #ffb4ab;
  font-size: 12px;
  padding: 4px 10px;
  border-radius: 6px;
  cursor: pointer;
}

.fatal-error-actions {
  margin-top: 10px;
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.fatal-error-btn {
  border: 1px solid rgba(255, 138, 128, 0.6);
  background: transparent;
  color: #ffb4ab;
  font-size: 12px;
  padding: 4px 10px;
  border-radius: 6px;
  cursor: pointer;
}

.fatal-error-hint {
  margin-top: 8px;
  font-family: var(--font-mono);
  font-size: 11px;
  color: #ffd1cb;
}
</style>
