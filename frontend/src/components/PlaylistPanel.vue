<template>
  <div class="playlist-panel">
    <div class="playlist-head">
      <span class="playlist-title">Playlist</span>
      <button class="playlist-add-btn" @click="$emit('add-file')" title="Add files">
        <svg viewBox="0 0 24 24" width="14" height="14" fill="currentColor">
          <path d="M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6v2z"/>
        </svg>
      </button>
    </div>
    <div class="playlist-body">
      <div v-if="items.length === 0" class="playlist-empty">
        No videos in playlist
      </div>
      <div
        v-for="(item, index) in items"
        :key="item + index"
        class="playlist-item"
        :class="{ active: currentIndex === index }"
        @click="$emit('select', index)"
      >
        <span class="playlist-index">{{ String(index + 1).padStart(2, "0") }}</span>
        <span class="playlist-name">{{ formatName(item) }}</span>
        <button
          class="playlist-remove"
          @click.stop="$emit('remove', index)"
          title="Remove"
        >
          &times;
        </button>
      </div>
    </div>
    <div class="playlist-foot">
      <span class="playlist-count">{{ items.length }} item{{ items.length === 1 ? "" : "s" }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  items: string[];
  currentIndex: number | null;
}>();

defineEmits<{
  (e: "select", index: number): void;
  (e: "remove", index: number): void;
  (e: "add-file"): void;
}>();

function formatName(path: string): string {
  const parts = path.split(/[\\/]/);
  return parts[parts.length - 1] || path;
}
</script>

<style scoped>
.playlist-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.playlist-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 14px;
  border-bottom: 1px solid var(--border-subtle);
  flex-shrink: 0;
}

.playlist-title {
  font-family: var(--font-display);
  font-size: 13px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  color: var(--text-primary);
}

.playlist-add-btn {
  background: transparent;
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-sm);
  color: var(--text-muted);
  cursor: pointer;
  padding: 2px 6px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.15s;
}

.playlist-add-btn:hover {
  border-color: var(--accent-cyan);
  color: var(--accent-cyan);
}

.playlist-body {
  flex: 1;
  overflow-y: auto;
  padding: 8px 10px;
}

.playlist-empty {
  font-family: var(--font-mono);
  font-size: 11px;
  color: var(--text-muted);
  text-align: center;
  padding: 24px 0;
  opacity: 0.6;
}

.playlist-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 8px;
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: all 0.12s;
  margin-bottom: 2px;
}

.playlist-item:hover {
  background: rgba(255, 255, 255, 0.04);
}

.playlist-item.active {
  background: rgba(0, 229, 255, 0.08);
  border: 1px solid rgba(0, 229, 255, 0.2);
}

.playlist-index {
  font-family: var(--font-mono);
  font-size: 10px;
  color: var(--text-muted);
  min-width: 20px;
  text-align: right;
  flex-shrink: 0;
}

.playlist-item.active .playlist-index {
  color: var(--accent-cyan);
}

.playlist-name {
  font-family: var(--font-mono);
  font-size: 11px;
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  flex: 1;
}

.playlist-item.active .playlist-name {
  color: var(--accent-cyan);
  font-weight: 600;
}

.playlist-remove {
  background: none;
  border: none;
  color: var(--text-muted);
  font-size: 14px;
  cursor: pointer;
  padding: 0 4px;
  line-height: 1;
  opacity: 0;
  transition: opacity 0.12s;
}

.playlist-item:hover .playlist-remove {
  opacity: 1;
}

.playlist-remove:hover {
  color: var(--accent-magenta);
}

.playlist-foot {
  padding: 8px 14px;
  border-top: 1px solid var(--border-subtle);
  flex-shrink: 0;
}

.playlist-count {
  font-family: var(--font-mono);
  font-size: 10px;
  color: var(--text-muted);
}
</style>
