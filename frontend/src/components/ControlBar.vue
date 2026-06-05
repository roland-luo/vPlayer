<template>
  <div class="control-bar" :class="{ visible: showControls }">
    <div class="seek-wrap" @click="handleProgressClick">
      <div class="seek-track">
        <div class="seek-fill" :style="{ width: progressPercent + '%' }"></div>
        <div class="seek-thumb" :style="{ left: progressPercent + '%' }"></div>
      </div>
    </div>

    <div class="controls-row">
      <div class="cluster left">
        <button class="icon-btn" @click="$emit('toggle-play')" :title="isPlaying ? 'Pause' : 'Play'">
          <svg v-if="!isPlaying" viewBox="0 0 24 24" width="22" height="22" fill="currentColor"><path d="M8 5v14l11-7z"/></svg>
          <svg v-else viewBox="0 0 24 24" width="22" height="22" fill="currentColor"><path d="M6 19h4V5H6v14zm8-14v14h4V5h-4z"/></svg>
        </button>
        <button class="icon-btn" @click="skipBackward" title="Back 10s">
          <svg viewBox="0 0 24 24" width="20" height="20" fill="currentColor"><path d="M11 18V6l-8.5 6 8.5 6zm.5-6l8.5 6V6l-8.5 6z"/></svg>
        </button>
        <button class="icon-btn" @click="skipForward" title="Forward 10s">
          <svg viewBox="0 0 24 24" width="20" height="20" fill="currentColor"><path d="M4 18l8.5-6L4 6v12zm9-12v12l8.5-6L13 6z"/></svg>
        </button>
        <div class="time-display" aria-live="polite">
          <span class="current">{{ formatTime(currentTime) }}</span>
          <span class="sep">/</span>
          <span class="total">{{ formatTime(duration) }}</span>
        </div>
      </div>

      <div class="cluster right">
        <div class="volume-wrap">
          <button class="icon-btn" @click="toggleMute" title="Mute">
            <svg viewBox="0 0 24 24" width="20" height="20" fill="currentColor">
              <path d="M3 9v6h4l5 5V4L7 9H3zm13.5 3c0-1.77-1.02-3.29-2.5-4.03v8.05c1.48-.73 2.5-2.25 2.5-4.02zM14 3.23v2.06c2.89.86 5 3.54 5 6.71s-2.11 5.85-5 6.71v2.06c4.01-.91 7-4.49 7-8.77s-2.99-7.86-7-8.77z"/>
            </svg>
          </button>
          <input type="range" class="volume-slider" min="0" max="100" :value="volume" @input="handleVolumeInput" />
        </div>

        <button class="icon-btn" @click="$emit('screenshot')" title="Screenshot">
          <svg viewBox="0 0 24 24" width="20" height="20" fill="currentColor">
            <path d="M9 2L7.17 4H4c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V6c0-1.1-.9-2-2-2h-3.17L15 2H9zm3 15c-2.76 0-5-2.24-5-5s2.24-5 5-5 5 2.24 5 5-2.24 5-5 5z"/>
          </svg>
        </button>

        <button class="icon-btn" @click="toggleMoreActions" title="More">
          <svg viewBox="0 0 24 24" width="20" height="20" fill="currentColor">
            <circle cx="5" cy="12" r="2"/><circle cx="12" cy="12" r="2"/><circle cx="19" cy="12" r="2"/>
          </svg>
        </button>

        <button class="icon-btn" @click="toggleFullscreen" title="Fullscreen">
          <svg viewBox="0 0 24 24" width="20" height="20" fill="currentColor"><path d="M7 14H5v5h5v-2H7v-3zm-2-4h2V7h3V5H5v5zm12 7h-3v2h5v-5h-2v3zM14 5v2h3v3h2V5h-5z"/></svg>
        </button>
      </div>
    </div>

    <div v-if="showMoreActions" class="more-actions">
      <button class="more-btn" @click="$emit('toggle-playlist')">Playlist</button>
      <button v-if="hasSubtitles" class="more-btn" @click="$emit('toggle-subtitles')">Subtitles</button>
      <button v-if="hasAudioTracks" class="more-btn" @click="$emit('toggle-audio-tracks')">Audio</button>
      <button
        v-for="plugin in pluginButtons"
        :key="plugin.name"
        class="more-btn"
        @click="$emit('plugin-click', plugin.name)"
      >
        {{ plugin.ui_button_label ?? plugin.name }}
      </button>
      <button class="more-btn" @click="openSettings">Settings</button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from "vue";
import { getCurrentWindow } from "@tauri-apps/api/window";
import type { PluginInfo } from "../api/player";

const props = defineProps<{
  isPlaying: boolean;
  currentTime: number;
  duration: number;
  volume: number;
  plugins?: PluginInfo[];
  hasSubtitles?: boolean;
  hasAudioTracks?: boolean;
}>();

const emit = defineEmits<{
  (e: "toggle-play"): void;
  (e: "seek", position: number): void;
  (e: "volume-change", volume: number): void;
  (e: "screenshot"): void;
  (e: "plugin-click", name: string): void;
  (e: "toggle-playlist"): void;
  (e: "toggle-subtitles"): void;
  (e: "toggle-audio-tracks"): void;
}>();

const showControls = ref(true);
const isMuted = ref(false);
const showMoreActions = ref(false);
let hideTimer: ReturnType<typeof setTimeout> | null = null;
const AUTO_HIDE_MS = 2500;

const pluginButtons = computed(() => {
  return (props.plugins ?? []).filter((p) => p.ui_button_label);
});

const progressPercent = computed(() => {
  if (props.duration <= 0) return 0;
  return (props.currentTime / props.duration) * 100;
});

function formatTime(seconds: number): string {
  if (!seconds || seconds < 0) return "00:00";
  const h = Math.floor(seconds / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  const s = Math.floor(seconds % 60);
  if (h > 0) {
    return `${String(h).padStart(2, "0")}:${String(m).padStart(2, "0")}:${String(s).padStart(2, "0")}`;
  }
  return `${String(m).padStart(2, "0")}:${String(s).padStart(2, "0")}`;
}

function handleProgressClick(event: MouseEvent) {
  const bar = event.currentTarget as HTMLElement;
  const rect = bar.getBoundingClientRect();
  const percent = (event.clientX - rect.left) / rect.width;
  const newPosition = percent * props.duration;
  emit("seek", newPosition);
}

function skipForward() {
  emit("seek", Math.min(props.currentTime + 10, props.duration));
}

function skipBackward() {
  emit("seek", Math.max(props.currentTime - 10, 0));
}

function handleVolumeInput(event: Event) {
  const target = event.target as HTMLInputElement;
  emit("volume-change", parseInt(target.value, 10));
}

function toggleMute() {
  isMuted.value = !isMuted.value;
  emit("volume-change", isMuted.value ? 0 : props.volume);
}

function toggleMoreActions() {
  showMoreActions.value = !showMoreActions.value;
  if (showMoreActions.value) {
    showControls.value = true;
    clearHideTimer();
  } else {
    scheduleAutoHide();
  }
}

function openSettings() {
  // TODO: open settings panel
}

async function toggleFullscreen() {
  try {
    const appWindow = getCurrentWindow();
    const isFullscreen = await appWindow.isFullscreen();
    await appWindow.setFullscreen(!isFullscreen);
    return;
  } catch (error) {
    console.debug("[fullscreen] Tauri API unavailable, fallback to DOM API", error);
  }

  const doc = document as Document & {
    webkitExitFullscreen?: () => Promise<void> | void;
    webkitFullscreenElement?: Element | null;
  };
  const el = document.documentElement as HTMLElement & {
    webkitRequestFullscreen?: () => Promise<void> | void;
  };

  const request = el.requestFullscreen ?? el.webkitRequestFullscreen;
  const exit = doc.exitFullscreen ?? doc.webkitExitFullscreen;
  const isFullscreen = Boolean(doc.fullscreenElement || doc.webkitFullscreenElement);

  if (!request || !exit) return;
  if (!isFullscreen) {
    await Promise.resolve(request.call(el));
  } else {
    await Promise.resolve(exit.call(doc));
  }
}

function clearHideTimer() {
  if (hideTimer) {
    clearTimeout(hideTimer);
    hideTimer = null;
  }
}

function scheduleAutoHide() {
  clearHideTimer();
  if (!props.isPlaying || showMoreActions.value) {
    showControls.value = true;
    return;
  }
  hideTimer = setTimeout(() => {
    if (props.isPlaying && !showMoreActions.value) {
      showControls.value = false;
    }
  }, AUTO_HIDE_MS);
}

function revealControls() {
  showControls.value = true;
  scheduleAutoHide();
}

watch(
  () => props.isPlaying,
  (playing) => {
    if (!playing) {
      showControls.value = true;
      clearHideTimer();
      return;
    }
    scheduleAutoHide();
  },
  { immediate: true },
);

onMounted(() => {
  window.addEventListener("mousemove", revealControls);
  window.addEventListener("click", revealControls);
});

onUnmounted(() => {
  window.removeEventListener("mousemove", revealControls);
  window.removeEventListener("click", revealControls);
  clearHideTimer();
});
</script>

<style scoped>
.control-bar {
  position: absolute;
  left: 0;
  right: 0;
  bottom: 0;
  z-index: 20;
  padding: 14px 16px 12px;
  background: linear-gradient(to top, rgba(0, 0, 0, 0.84) 0%, rgba(0, 0, 0, 0.42) 58%, transparent 100%);
  opacity: 0;
  transform: translateY(8px);
  transition: opacity 180ms ease-out, transform 180ms ease-out;
  pointer-events: none;
}

.control-bar.visible {
  opacity: 1;
  transform: translateY(0);
  pointer-events: auto;
}

.seek-wrap {
  cursor: pointer;
  padding: 8px 0;
}

.seek-track {
  position: relative;
  height: 4px;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.26);
}

.seek-fill {
  height: 100%;
  border-radius: 999px;
  background: #ff2d2d;
}

.seek-thumb {
  position: absolute;
  top: 50%;
  width: 12px;
  height: 12px;
  border-radius: 50%;
  background: #ff2d2d;
  transform: translate(-50%, -50%) scale(0);
  transition: transform 140ms ease-out;
}

.seek-wrap:hover .seek-thumb {
  transform: translate(-50%, -50%) scale(1);
}

.controls-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 14px;
}

.cluster {
  display: flex;
  align-items: center;
  gap: 6px;
}

.icon-btn {
  width: 36px;
  height: 36px;
  border: none;
  border-radius: 999px;
  background: transparent;
  color: #fff;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: background 140ms ease-out;
}

.icon-btn:hover {
  background: rgba(255, 255, 255, 0.18);
}

.time-display {
  display: inline-flex;
  align-items: baseline;
  gap: 6px;
  margin-left: 6px;
  font-family: var(--font-body);
  font-size: 13px;
  color: #f1f1f1;
  font-variant-numeric: tabular-nums;
}

.time-display .total {
  color: #b8b8b8;
}

.time-display .sep {
  color: #999;
}

.volume-wrap {
  display: flex;
  align-items: center;
  gap: 4px;
}

.volume-slider {
  -webkit-appearance: none;
  appearance: none;
  width: 88px;
  height: 4px;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.35);
  cursor: pointer;
}

.volume-slider::-webkit-slider-thumb {
  -webkit-appearance: none;
  appearance: none;
  width: 10px;
  height: 10px;
  border-radius: 50%;
  background: #fff;
}

.volume-slider::-moz-range-thumb {
  width: 10px;
  height: 10px;
  border: none;
  border-radius: 50%;
  background: #fff;
}

.more-actions {
  margin-top: 10px;
  padding: 10px;
  border-radius: 12px;
  background: rgba(20, 20, 20, 0.9);
  border: 1px solid rgba(255, 255, 255, 0.12);
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.more-btn {
  border: none;
  border-radius: 999px;
  padding: 8px 12px;
  background: rgba(255, 255, 255, 0.08);
  color: #f3f3f3;
  font-family: var(--font-body);
  font-size: 12px;
  cursor: pointer;
}

.more-btn:hover {
  background: rgba(255, 255, 255, 0.2);
}

@media (max-width: 900px) {
  .volume-slider {
    display: none;
  }

  .time-display {
    font-size: 12px;
  }
}
</style>
