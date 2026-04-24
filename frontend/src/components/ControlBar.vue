<template>
  <div class="control-bar" :class="{ visible: showControls }">
    <div class="progress-bar" @click="handleProgressClick">
      <div class="progress-fill" :style="{ width: progressPercent + '%' }">
        <div class="progress-thumb"></div>
      </div>
    </div>
    <div class="controls-row">
      <div class="controls-left">
        <button class="control-btn" @click="$emit('toggle-play')">
          <svg v-if="!isPlaying" viewBox="0 0 24 24" width="20" height="20" fill="currentColor">
            <path d="M8 5v14l11-7z" />
          </svg>
          <svg v-else viewBox="0 0 24 24" width="20" height="20" fill="currentColor">
            <path d="M6 19h4V5H6v14zm8-14v14h4V5h-4z" />
          </svg>
        </button>
        <button class="control-btn" @click="skipBackward">
          <svg viewBox="0 0 24 24" width="20" height="20" fill="currentColor">
            <path d="M11 18V6l-8.5 6 8.5 6zm.5-6l8.5 6V6l-8.5 6z" />
          </svg>
        </button>
        <button class="control-btn" @click="skipForward">
          <svg viewBox="0 0 24 24" width="20" height="20" fill="currentColor">
            <path d="M4 18l8.5-6L4 6v12zm9-12v12l8.5-6L13 6z" />
          </svg>
        </button>
        <div class="time-display">
          <span class="current">{{ formatTime(currentTime) }}</span>
          <span class="separator"> / </span>
          <span>{{ formatTime(duration) }}</span>
        </div>
      </div>
      <div class="controls-right">
        <div class="volume-control">
          <button class="control-btn" @click="toggleMute">
            <svg viewBox="0 0 24 24" width="20" height="20" fill="currentColor">
              <path
                d="M3 9v6h4l5 5V4L7 9H3zm13.5 3c0-1.77-1.02-3.29-2.5-4.03v8.05c1.48-.73 2.5-2.25 2.5-4.02zM14 3.23v2.06c2.89.86 5 3.54 5 6.71s-2.11 5.85-5 6.71v2.06c4.01-.91 7-4.49 7-8.77s-2.99-7.86-7-8.77z" />
            </svg>
          </button>
          <input type="range" class="volume-slider" min="0" max="100" :value="volume" @input="handleVolumeInput" />
        </div>
        <button class="control-btn" @click="openSettings">
          <svg viewBox="0 0 24 24" width="20" height="20" fill="currentColor">
            <path
              d="M19.43 12.98c.04-.32.07-.64.07-.98s-.03-.66-.07-.98l2.11-1.65c.19-.15.24-.42.12-.64l-2-3.46c-.12-.22-.39-.3-.61-.22l-2.49 1c-.52-.4-1.08-.73-1.69-.98l-.38-2.65C14.46 2.18 14.25 2 14 2h-4c-.25 0-.46.18-.5.42l-.38 2.65c-.61.25-1.17.59-1.69.98l-2.49-1c-.23-.09-.49 0-.61.22l-2 3.46c-.13.22-.07.49.12.64l2.11 1.65c-.04.32-.07.65-.07.98s.03.66.07.98l-2.11 1.65c-.19.15-.24.42-.12.64l2 3.46c.12.22.39.3.61.22l2.49-1.01c.52.4 1.08.73 1.69.98l.38 2.65c.04.24.25.42.5.42h4c.25 0 .46-.18.5-.42l.38-2.65c.61-.25 1.17-.59 1.69-.98l2.49 1.01c.22.08.49 0 .61-.22l2-3.46c.12-.22.07-.49-.12-.64l-2.11-1.65zM12 15.5c-1.93 0-3.5-1.57-3.5-3.5s1.57-3.5 3.5-3.5 3.5 1.57 3.5 3.5-1.57 3.5-3.5 3.5z" />
          </svg>
        </button>
        <button class="control-btn" @click="toggleFullscreen">
          <svg viewBox="0 0 24 24" width="20" height="20" fill="currentColor">
            <path d="M7 14H5v5h5v-2H7v-3zm-2-4h2V7h3V5H5v5zm12 7h-3v2h5v-5h-2v3zM14 5v2h3v3h2V5h-5z" />
          </svg>
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from "vue";
import { getCurrentWindow } from "@tauri-apps/api/window";

const props = defineProps<{
  isPlaying: boolean;
  currentTime: number;
  duration: number;
  volume: number;
}>();

const emit = defineEmits<{
  (e: "toggle-play"): void;
  (e: "seek", position: number): void;
  (e: "volume-change", volume: number): void;
}>();

const showControls = ref(true);
const isMuted = ref(false);

const progressPercent = computed(() => {
  if (props.duration <= 0) return 0;
  return (props.currentTime / props.duration) * 100;
});

function formatTime(seconds: number): string {
  if (!seconds || seconds < 0) return "00:00:00";
  const h = Math.floor(seconds / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  const s = Math.floor(seconds % 60);
  return `${String(h).padStart(2, "0")}:${String(m).padStart(2, "0")}:${String(s).padStart(2, "0")}`;
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

function openSettings() {
  // TODO: open settings panel
}

async function toggleFullscreen() {
  // Always try Tauri window fullscreen first.
  // Runtime sniffing via window globals is unreliable across Tauri/WebView versions.
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

  if (!request || !exit) {
    console.debug("[fullscreen] Fullscreen API is not available in current runtime.");
    return;
  }

  if (!isFullscreen) {
    await Promise.resolve(request.call(el));
  } else {
    await Promise.resolve(exit.call(doc));
  }
}
</script>

<style scoped>
.control-bar {
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  background: linear-gradient(to top,
      rgba(8, 8, 12, 0.95) 0%,
      rgba(8, 8, 12, 0.6) 50%,
      transparent 100%);
  padding: 48px 24px 16px;
  z-index: 10;
  opacity: 0;
  transform: translateY(8px);
  transition: opacity 200ms ease-out, transform 200ms ease-out;
  pointer-events: none;
}

.control-bar.visible {
  opacity: 1;
  transform: translateY(0);
  pointer-events: auto;
}

.progress-bar {
  width: 100%;
  height: 4px;
  background: rgba(255, 255, 255, 0.08);
  border-radius: 2px;
  position: relative;
  margin-bottom: 16px;
  cursor: pointer;
  transition: height 100ms ease;
}

.progress-bar:hover {
  height: 6px;
}

.progress-fill {
  height: 100%;
  background: var(--accent-cyan);
  border-radius: 2px;
  position: relative;
  animation: bar-glow 3s ease-in-out infinite;
}

.progress-thumb {
  position: absolute;
  right: -7px;
  top: 50%;
  transform: translateY(-50%);
  width: 14px;
  height: 14px;
  background: var(--accent-cyan);
  border-radius: 50%;
  box-shadow: 0 0 16px rgba(0, 229, 255, 0.6);
}

.controls-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.controls-left,
.controls-right {
  display: flex;
  align-items: center;
  gap: 16px;
}

.control-btn {
  background: none;
  border: none;
  color: var(--text-primary);
  cursor: pointer;
  padding: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: color 150ms ease-out;
}

.control-btn:hover {
  color: var(--accent-cyan);
}

.time-display {
  font-family: var(--font-mono);
  font-size: 13px;
  color: var(--text-muted);
  letter-spacing: 0.02em;
  margin-left: 8px;
}

.time-display .current {
  color: var(--text-primary);
}

.time-display .separator {
  margin: 0 4px;
}

.volume-control {
  display: flex;
  align-items: center;
  gap: 8px;
}

.volume-slider {
  -webkit-appearance: none;
  appearance: none;
  width: 80px;
  height: 4px;
  background: rgba(255, 255, 255, 0.1);
  border-radius: 2px;
  outline: none;
  cursor: pointer;
}

.volume-slider::-webkit-slider-thumb {
  -webkit-appearance: none;
  appearance: none;
  width: 12px;
  height: 12px;
  background: var(--accent-cyan);
  border-radius: 50%;
  box-shadow: 0 0 8px rgba(0, 229, 255, 0.4);
  cursor: pointer;
}

.volume-slider::-moz-range-thumb {
  width: 12px;
  height: 12px;
  background: var(--accent-cyan);
  border-radius: 50%;
  box-shadow: 0 0 8px rgba(0, 229, 255, 0.4);
  cursor: pointer;
  border: none;
}
</style>
