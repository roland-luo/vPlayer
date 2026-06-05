<template>
  <header class="top-hud">
    <div class="hud-left">
      <button class="hud-btn" title="Open File" @click="$emit('open-file')">
        <svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="1.8">
          <path d="M4 7h6l2 2h8v8a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2z" />
          <path d="M4 10V7a2 2 0 0 1 2-2h4" />
        </svg>
      </button>
      <button class="hud-btn" title="Previous" @click="$emit('prev-file')">
        <svg viewBox="0 0 24 24" width="18" height="18" fill="currentColor">
          <path d="M18 6v12l-8.5-6zM6 6h2v12H6z" />
        </svg>
      </button>
      <button class="hud-btn" title="Next" @click="$emit('next-file')">
        <svg viewBox="0 0 24 24" width="18" height="18" fill="currentColor">
          <path d="M6 6v12l8.5-6zM16 6h2v12h-2z" />
        </svg>
      </button>
      <div class="hud-title-wrap">
        <span class="hud-title">{{ title }}</span>
      </div>
    </div>

    <div class="hud-right">
      <span class="hud-status" :class="{ playing: isPlaying }">
        <span class="dot"></span>
        {{ isPlaying ? "Playing" : "Paused" }}
      </span>
      <span class="hud-meta">{{ meta.resolution }}</span>
      <span class="hud-meta">{{ meta.fps }}</span>
      <button class="hud-chip" @click="$emit('toggle-plugins')">Plugins</button>
      <button class="hud-danger" title="Debug Error" @click="$emit('debug-error')">Err</button>
      <button class="hud-danger" title="Debug Fatal" @click="$emit('debug-fatal')">Fatal</button>
    </div>
  </header>
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
  (e: "toggle-plugins"): void;
}>();
</script>

<style scoped>
.top-hud {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  z-index: 20;
  height: 56px;
  padding: 0 14px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  background: linear-gradient(to bottom, rgba(0, 0, 0, 0.72) 0%, rgba(0, 0, 0, 0.22) 70%, transparent 100%);
  pointer-events: none;
}

.hud-left,
.hud-right {
  display: flex;
  align-items: center;
  gap: 8px;
}

.hud-left,
.hud-right button {
  pointer-events: auto;
}

.hud-btn {
  width: 34px;
  height: 34px;
  border: none;
  border-radius: 999px;
  background: rgba(24, 24, 24, 0.88);
  color: #fff;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: background 160ms ease-out, transform 120ms ease-out;
}

.hud-btn:hover {
  background: rgba(56, 56, 56, 0.95);
}

.hud-btn:active {
  transform: scale(0.96);
}

.hud-title-wrap {
  min-width: 0;
  max-width: 34vw;
  margin-left: 6px;
}

.hud-title {
  font-family: var(--font-body);
  font-size: 14px;
  font-weight: 600;
  color: #fff;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.hud-status {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 6px 10px;
  border-radius: 999px;
  font-family: var(--font-body);
  font-size: 12px;
  color: #d0d0d0;
  background: rgba(22, 22, 22, 0.88);
}

.hud-status .dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  background: #ff4e45;
}

.hud-status.playing .dot {
  background: #00e5ff;
}

.hud-meta {
  font-family: var(--font-mono);
  font-size: 11px;
  color: #c7c7c7;
  padding: 4px 8px;
  border-radius: 999px;
  background: rgba(30, 30, 30, 0.7);
}

.hud-chip,
.hud-danger {
  height: 32px;
  border: none;
  border-radius: 999px;
  padding: 0 12px;
  font-family: var(--font-body);
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
}

.hud-chip {
  background: rgba(24, 24, 24, 0.9);
  color: #fff;
}

.hud-chip:hover {
  background: rgba(52, 52, 52, 0.95);
}

.hud-danger {
  background: rgba(255, 0, 110, 0.18);
  color: #ff8fb8;
}

.hud-danger:hover {
  background: rgba(255, 0, 110, 0.28);
}

@media (max-width: 1080px) {
  .hud-meta,
  .hud-danger {
    display: none;
  }

  .hud-title-wrap {
    max-width: 42vw;
  }
}
</style>
