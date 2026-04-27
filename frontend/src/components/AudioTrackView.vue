<template>
  <div class="audio-track-view">
    <div v-if="tracks.length === 0" class="audio-track-empty">
      No audio tracks available
    </div>
    <div v-else class="audio-track-list">
      <div
        v-for="track in tracks"
        :key="track.id"
        class="audio-track-item"
        :class="{ active: track.enabled }"
        @click="$emit('select-track', track.id)"
      >
        <span class="track-label">{{ track.label }}</span>
        <span v-if="track.language" class="track-lang">{{ track.language }}</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { AudioTrackInfo } from "./PlayerView.vue";

defineProps<{
  tracks: AudioTrackInfo[];
}>();

defineEmits<{
  (e: "select-track", id: string): void;
}>();
</script>

<style scoped>
.audio-track-view {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.audio-track-empty {
  font-family: var(--font-mono);
  font-size: 11px;
  color: var(--text-muted);
  text-align: center;
  padding: 24px 0;
}

.audio-track-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.audio-track-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 10px;
  border-radius: var(--radius-md);
  background: var(--bg-surface);
  border: 1px solid var(--border-subtle);
  cursor: pointer;
  transition: all 0.12s;
}

.audio-track-item:hover {
  border-color: var(--border-glow);
}

.audio-track-item.active {
  background: rgba(0, 229, 255, 0.08);
  border-color: var(--accent-cyan);
}

.track-label {
  font-family: var(--font-mono);
  font-size: 12px;
  color: var(--text-primary);
  flex: 1;
}

.audio-track-item.active .track-label {
  color: var(--accent-cyan);
  font-weight: 600;
}

.track-lang {
  font-family: var(--font-mono);
  font-size: 10px;
  color: var(--text-muted);
  text-transform: uppercase;
}
</style>
