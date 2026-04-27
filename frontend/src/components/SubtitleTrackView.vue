<template>
  <div class="subtitle-track-view">
    <div v-if="tracks.length === 0" class="subtitle-empty">
      No subtitle tracks available
    </div>
    <div v-else class="subtitle-list">
      <div
        v-for="track in tracks"
        :key="track.id"
        class="subtitle-track-item"
        :class="{ active: track.mode === 'showing' }"
        @click="toggleTrack(track.id)"
      >
        <span class="track-label">{{ track.label }}</span>
        <span v-if="track.language" class="track-lang">{{ track.language }}</span>
        <span v-if="track.external" class="track-badge">EXT</span>
      </div>
    </div>

    <div class="subtitle-actions">
      <button class="subtitle-action-btn" @click="$emit('load-external')">
        Load External Subtitle
      </button>
      <button class="subtitle-action-btn" @click="$emit('clear-external')">
        Clear External
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { TextTrackInfo } from "./PlayerView.vue";

defineProps<{
  tracks: TextTrackInfo[];
}>();

const emit = defineEmits<{
  (e: "toggle-track", id: string): void;
  (e: "load-external"): void;
  (e: "clear-external"): void;
}>();

function toggleTrack(id: string) {
  emit("toggle-track", id);
}
</script>

<style scoped>
.subtitle-track-view {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.subtitle-empty {
  font-family: var(--font-mono);
  font-size: 11px;
  color: var(--text-muted);
  text-align: center;
  padding: 24px 0;
}

.subtitle-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.subtitle-track-item {
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

.subtitle-track-item:hover {
  border-color: var(--border-glow);
}

.subtitle-track-item.active {
  background: rgba(0, 229, 255, 0.08);
  border-color: var(--accent-cyan);
}

.track-label {
  font-family: var(--font-mono);
  font-size: 12px;
  color: var(--text-primary);
  flex: 1;
}

.subtitle-track-item.active .track-label {
  color: var(--accent-cyan);
  font-weight: 600;
}

.track-lang {
  font-family: var(--font-mono);
  font-size: 10px;
  color: var(--text-muted);
  text-transform: uppercase;
}

.track-badge {
  font-family: var(--font-mono);
  font-size: 9px;
  padding: 1px 5px;
  border-radius: 2px;
  background: rgba(0, 229, 255, 0.1);
  color: var(--accent-cyan);
}

.subtitle-actions {
  display: flex;
  gap: 8px;
}

.subtitle-action-btn {
  flex: 1;
  background: transparent;
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-sm);
  padding: 6px 8px;
  font-family: var(--font-mono);
  font-size: 11px;
  color: var(--text-primary);
  cursor: pointer;
  text-align: center;
  transition: all 0.15s;
}

.subtitle-action-btn:hover {
  border-color: var(--accent-cyan);
  color: var(--accent-cyan);
}
</style>
