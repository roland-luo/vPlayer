<template>
  <div class="player-view">
    <div class="video-surface">
      <video
        ref="videoEl"
        class="video-element"
        :src="sourceUrl || undefined"
        playsinline
        preload="metadata"
        @loadedmetadata="handleLoadedMetadata"
        @timeupdate="handleTimeUpdate"
        @ended="handleEnded"
        @error="handleError"
      />
      <div class="video-overlay">
        <div
          class="play-overlay"
          v-if="!isPlaying || !sourceUrl"
          @click="$emit('play')"
        >
          <div class="play-button">
            <svg viewBox="0 0 24 24" width="28" height="28">
              <polygon points="5,3 19,12 5,21" fill="white" />
            </svg>
          </div>
        </div>
        <div
          v-else-if="isPlaying"
          class="pause-overlay"
          @click="$emit('pause')"
        >
          <!-- Invisible click target for pausing -->
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { captureScreenshot as captureScreenshotApi } from "../api/player";
import { convertFileSrc } from "@tauri-apps/api/core";

export interface TextTrackInfo {
  id: string;
  kind: string;
  label: string;
  language: string;
  mode: string;
  src?: string;
  external?: boolean;
}

export interface AudioTrackInfo {
  id: string;
  kind: string;
  label: string;
  language: string;
  enabled: boolean;
}

const props = defineProps<{
  isPlaying: boolean;
  sourcePath: string;
  volume: number;
  playbackRate?: number;
}>();

const emit = defineEmits<{
  (e: "play"): void;
  (e: "pause"): void;
  (e: "progress", payload: { position: number; duration: number }): void;
  (e: "loaded-metadata", payload: { duration: number; width: number; height: number }): void;
  (e: "ended"): void;
  (e: "video-error", payload: { code: string; message: string }): void;
  (e: "tracks-change", tracks: TextTrackInfo[]): void;
  (e: "audio-tracks-change", tracks: AudioTrackInfo[]): void;
}>();

const videoEl = ref<HTMLVideoElement | null>(null);
let externalTrackCount = 0;

const sourceUrl = computed(() => {
  if (!props.sourcePath) return "";
  return convertFileSrc(props.sourcePath);
});

function readTextTracks(): TextTrackInfo[] {
  const el = videoEl.value;
  if (!el) return [];
  const tracks: TextTrackInfo[] = [];
  for (let i = 0; i < el.textTracks.length; i++) {
    const t = el.textTracks[i];
    tracks.push({
      id: (t as any).id || `track-${i}`,
      kind: t.kind,
      label: t.label || `Track ${i + 1}`,
      language: t.language || "",
      mode: t.mode,
    });
  }
  // Also read <track> elements to detect external tracks.
  const trackEls = el.querySelectorAll("track");
  trackEls.forEach((tr, idx) => {
    const existing = tracks.find((t) => t.id === tr.id);
    if (!existing) {
      tracks.push({
        id: tr.id || `ext-${idx}`,
        kind: tr.kind || "subtitles",
        label: tr.label || `External ${idx + 1}`,
        language: tr.srclang || "",
        mode: "disabled",
        src: tr.src,
        external: true,
      });
    }
  });
  return tracks;
}

function emitTracks() {
  emit("tracks-change", readTextTracks());
}

function setupTrackListeners() {
  const el = videoEl.value;
  if (!el) return;
  const handler = () => emitTracks();
  el.textTracks.addEventListener("addtrack", handler);
  el.textTracks.addEventListener("removetrack", handler);
  el.textTracks.addEventListener("change", handler);
  // Initial emit.
  emitTracks();
}

function removeTrackListeners() {
  const el = videoEl.value;
  if (!el) return;
  const handler = () => emitTracks();
  el.textTracks.removeEventListener("addtrack", handler);
  el.textTracks.removeEventListener("removetrack", handler);
  el.textTracks.removeEventListener("change", handler);
}

/** Toggle a text track on/off. */
function setTextTrackEnabled(trackId: string, enabled: boolean) {
  const el = videoEl.value;
  if (!el) return;
  for (let i = 0; i < el.textTracks.length; i++) {
    const t = el.textTracks[i];
    const id = (t as any).id || `track-${i}`;
    if (id === trackId) {
      t.mode = enabled ? "showing" : "disabled";
      emitTracks();
      return;
    }
  }
  // Check external <track> elements.
  const trackEl = el.querySelector(`track#${CSS.escape(trackId)}`);
  if (trackEl) {
    trackEl.setAttribute("default", enabled ? "" : "false");
    // Re-load to apply default change.
    if (enabled) {
      const clone = trackEl.cloneNode(true) as HTMLTrackElement;
      clone.default = true;
      trackEl.parentNode?.replaceChild(clone, trackEl);
    }
    emitTracks();
  }
}

/** Add an external subtitle file as a <track> element. */
async function addSubtitleTrack(path: string, label?: string, language?: string): Promise<void> {
  const el = videoEl.value;
  if (!el) return;
  const url = convertFileSrc(path);
  const track = document.createElement("track");
  externalTrackCount++;
  track.id = `ext-sub-${externalTrackCount}`;
  track.kind = "subtitles";
  track.label = label || `Subtitle ${externalTrackCount}`;
  track.srclang = language || "";
  track.src = url;
  track.default = true;
  el.appendChild(track);
  emitTracks();
}

/** Remove all external subtitle tracks. */
function clearExternalSubtitles() {
  const el = videoEl.value;
  if (!el) return;
  el.querySelectorAll("track").forEach((t) => t.remove());
  externalTrackCount = 0;
  emitTracks();
}

// --- Audio tracks ---

function readAudioTracks(): AudioTrackInfo[] {
  const el = videoEl.value;
  if (!el) return [];
  const list = (el as any).audioTracks;
  if (!list) return [];
  const tracks: AudioTrackInfo[] = [];
  for (let i = 0; i < list.length; i++) {
    const t = list[i];
    tracks.push({
      id: String(t.id || i),
      kind: t.kind || "",
      label: t.label || `Audio ${i + 1}`,
      language: t.language || "",
      enabled: Boolean(t.enabled),
    });
  }
  return tracks;
}

function emitAudioTracks() {
  emit("audio-tracks-change", readAudioTracks());
}

function setupAudioTrackListeners() {
  const el = videoEl.value;
  if (!el) return;
  const list = (el as any).audioTracks;
  if (!list) return;
  const handler = () => emitAudioTracks();
  list.addEventListener?.("addtrack", handler);
  list.addEventListener?.("removetrack", handler);
  list.addEventListener?.("change", handler);
  emitAudioTracks();
}

function removeAudioTrackListeners() {
  const el = videoEl.value;
  if (!el) return;
  const list = (el as any).audioTracks;
  if (!list) return;
  const handler = () => emitAudioTracks();
  list.removeEventListener?.("addtrack", handler);
  list.removeEventListener?.("removetrack", handler);
  list.removeEventListener?.("change", handler);
}

function setAudioTrackEnabled(trackId: string) {
  const el = videoEl.value;
  if (!el) return;
  const list = (el as any).audioTracks;
  if (!list) return;
  for (let i = 0; i < list.length; i++) {
    const t = list[i];
    t.enabled = String(t.id || i) === trackId;
  }
  emitAudioTracks();
}

async function tryPlay() {
  if (!videoEl.value || !sourceUrl.value) return;
  try {
    await videoEl.value.play();
  } catch (error) {
    // `AbortError` is expected during source switching (load() interrupts play()).
    // Treat it as a transient race instead of a user-visible playback failure.
    if (error instanceof DOMException && error.name === "AbortError") {
      return;
    }
    emit("video-error", {
      code: "video_play_failed",
      message: `Video play failed: ${String(error)}`,
    });
  }
}

function pausePlayback() {
  if (!videoEl.value) return;
  videoEl.value.pause();
}

function seekTo(position: number) {
  if (!videoEl.value || Number.isNaN(position)) return;
  videoEl.value.currentTime = Math.max(0, position);
}

function handleLoadedMetadata() {
  if (!videoEl.value) return;
  emit("loaded-metadata", {
    duration: Number.isFinite(videoEl.value.duration) ? videoEl.value.duration : 0,
    width: videoEl.value.videoWidth,
    height: videoEl.value.videoHeight,
  });
  setupTrackListeners();
  setupAudioTrackListeners();
}

function handleTimeUpdate() {
  if (!videoEl.value) return;
  emit("progress", {
    position: videoEl.value.currentTime,
    duration: Number.isFinite(videoEl.value.duration) ? videoEl.value.duration : 0,
  });
}

function handleEnded() {
  emit("ended");
}

function handleError() {
  const err = videoEl.value?.error;
  emit("video-error", {
    code: `video_element_error_${err?.code ?? "unknown"}`,
    message: err?.message || "Video element failed to decode or load media.",
  });
}

watch(
  () => props.isPlaying,
  async (playing) => {
    if (playing) {
      await tryPlay();
    } else {
      pausePlayback();
    }
  },
  { immediate: true },
);

watch(
  () => sourceUrl.value,
  async () => {
    if (!videoEl.value) return;
    removeTrackListeners();
    removeAudioTrackListeners();
    clearExternalSubtitles();
    videoEl.value.load();
    if (props.isPlaying) {
      await tryPlay();
    }
  },
);

watch(
  () => props.volume,
  (v) => {
    if (!videoEl.value) return;
    videoEl.value.volume = Math.max(0, Math.min(1, v / 100));
  },
  { immediate: true },
);

watch(
  () => props.playbackRate,
  (rate) => {
    if (!videoEl.value || !rate) return;
    videoEl.value.playbackRate = Math.max(0.1, Math.min(16, rate));
  },
  { immediate: false },
);

async function captureScreenshot(): Promise<string> {
  return captureScreenshotApi();
}

function setPlaybackSpeed(rate: number) {
  if (!videoEl.value) return;
  videoEl.value.playbackRate = Math.max(0.1, Math.min(16, rate));
}

defineExpose({
  seekTo,
  tryPlay,
  pausePlayback,
  captureScreenshot,
  setPlaybackSpeed,
  addSubtitleTrack,
  clearExternalSubtitles,
  setTextTrackEnabled,
  readTextTracks,
  setAudioTrackEnabled,
  readAudioTracks,
});
</script>

<style scoped>
.player-view {
  position: absolute;
  inset: 0;
  z-index: 1;
}

.video-surface {
  width: 100%;
  height: 100%;
  background: var(--bg-base);
  position: relative;
  overflow: hidden;
}

.video-element {
  width: 100%;
  height: 100%;
  object-fit: contain;
  background: #000;
}

.video-overlay {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  pointer-events: none;
}

.video-overlay::before {
  content: "";
  position: absolute;
  inset: 0;
  background:
    radial-gradient(ellipse 80% 50% at 50% 50%, rgba(0, 229, 255, 0.03) 0%, transparent 70%),
    repeating-linear-gradient(
      0deg,
      transparent,
      transparent 2px,
      rgba(0, 229, 255, 0.012) 2px,
      rgba(0, 229, 255, 0.012) 4px
    );
  pointer-events: none;
}

.play-overlay {
  position: relative;
  z-index: 2;
  cursor: pointer;
  pointer-events: auto;
}

.play-button {
  width: 80px;
  height: 80px;
  border: 2px solid rgba(255, 255, 255, 0.3);
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.3s ease;
}

.play-button:hover {
  border-color: var(--accent-cyan);
  box-shadow: 0 0 30px rgba(0, 229, 255, 0.2);
}

.play-button svg {
  margin-left: 4px;
}

.pause-overlay {
  position: absolute;
  inset: 0;
  z-index: 2;
  cursor: pointer;
  pointer-events: auto;
}
</style>
