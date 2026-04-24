<template>
  <div class="top-hud">
    <div class="hud-left">
      <button class="hud-open-btn" @click="$emit('open-file')">Open</button>
      <button class="hud-nav-btn" @click="$emit('prev-file')">Prev</button>
      <button class="hud-nav-btn" @click="$emit('next-file')">Next</button>
      <span class="hud-title">{{ title }}</span>
    </div>
    <div class="hud-right">
      <span class="hud-item active">
        <span class="hud-pulse"></span>
        {{ isPlaying ? "Playing" : "Ready" }}
      </span>
      <button class="hud-debug-btn" @click="$emit('debug-error')">DebugError</button>
      <button class="hud-debug-btn" @click="$emit('debug-fatal')">DebugFatal</button>
      <span class="hud-item">PL {{ playlistStatus }}</span>
      <span class="hud-item">{{ meta.audio }}</span>
      <span class="hud-item">{{ meta.codec }}</span>
      <span class="hud-item">{{ meta.resolution }}</span>
      <span class="hud-item">{{ meta.fps }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
interface MediaMeta {
  codec: string;
  resolution: string;
  fps: string;
  audio: string;
}

defineProps<{
  title: string;
  meta: MediaMeta;
  isPlaying?: boolean;
  playlistStatus: string;
}>();

defineEmits<{
  (e: "open-file"): void;
  (e: "debug-error"): void;
  (e: "debug-fatal"): void;
  (e: "prev-file"): void;
  (e: "next-file"): void;
}>();
</script>

<style scoped>
.top-hud {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 36px;
  background: linear-gradient(
    to bottom,
    rgba(8, 8, 12, 0.9) 0%,
    rgba(8, 8, 12, 0.5) 70%,
    transparent 100%
  );
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 20px;
  z-index: 10;
  pointer-events: none;
}

.hud-left,
.hud-right {
  display: flex;
  align-items: center;
  gap: 16px;
}

.hud-left {
  pointer-events: auto;
}

.hud-title {
  font-family: var(--font-display);
  font-size: 14px;
  font-weight: 600;
  letter-spacing: 0.04em;
  text-transform: uppercase;
}

.hud-open-btn {
  pointer-events: auto;
  background: transparent;
  border: 1px solid rgba(0, 229, 255, 0.35);
  color: var(--accent-cyan);
  font-family: var(--font-mono);
  font-size: 10px;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  padding: 3px 8px;
  border-radius: 4px;
  cursor: pointer;
}

.hud-open-btn:hover {
  border-color: rgba(0, 229, 255, 0.7);
  box-shadow: 0 0 10px rgba(0, 229, 255, 0.2);
}

.hud-nav-btn {
  pointer-events: auto;
  background: transparent;
  border: 1px solid rgba(255, 255, 255, 0.25);
  color: var(--text-primary);
  font-family: var(--font-mono);
  font-size: 10px;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  padding: 3px 8px;
  border-radius: 4px;
  cursor: pointer;
}

.hud-nav-btn:hover {
  border-color: rgba(255, 255, 255, 0.6);
}

.hud-debug-btn {
  pointer-events: auto;
  background: transparent;
  border: 1px solid rgba(255, 82, 82, 0.45);
  color: #ff8a80;
  font-family: var(--font-mono);
  font-size: 10px;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  padding: 3px 8px;
  border-radius: 4px;
  cursor: pointer;
}

.hud-debug-btn:hover {
  border-color: rgba(255, 82, 82, 0.75);
  box-shadow: 0 0 10px rgba(255, 82, 82, 0.2);
}

.hud-item {
  font-family: var(--font-mono);
  font-size: 10px;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.hud-item.active {
  color: var(--accent-cyan);
}

.hud-pulse {
  width: 5px;
  height: 5px;
  background: var(--accent-cyan);
  border-radius: 50%;
  display: inline-block;
  animation: breathe 3s ease-in-out infinite;
  box-shadow: 0 0 6px rgba(0, 229, 255, 0.4);
  margin-right: 6px;
  vertical-align: middle;
}
</style>
