<template>
  <div class="chapter-view">
    <div v-if="loading" class="ch-loading">正在读取章节信息...</div>

    <div v-else-if="error" class="ch-error">
      {{ error }}
      <button class="retry-btn" @click="fetchChapters">重试</button>
    </div>

    <div v-else-if="chapters.length === 0" class="ch-empty">
      当前视频没有章节标记。
    </div>

    <div v-else class="ch-list">
      <div
        v-for="ch in chapters"
        :key="ch.id"
        class="ch-item"
        @click="seekToChapter(ch)"
      >
        <div class="ch-index">{{ ch.id + 1 }}</div>
        <div class="ch-info">
          <span class="ch-title">{{ ch.title }}</span>
          <span class="ch-time">{{ formatTime(ch.start) }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from "vue";
import { listChapters, type ChapterEntry } from "../api/player";

const chapters = ref<ChapterEntry[]>([]);
const loading = ref(false);
const error = ref("");

const emit = defineEmits<{
  (e: "seek", position: number): void;
}>();

function formatTime(seconds: number): string {
  if (!seconds || seconds < 0) return "00:00:00";
  const h = Math.floor(seconds / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  const s = Math.floor(seconds % 60);
  return `${String(h).padStart(2, "0")}:${String(m).padStart(2, "0")}:${String(s).padStart(2, "0")}`;
}

async function fetchChapters() {
  loading.value = true;
  error.value = "";
  chapters.value = [];

  try {
    chapters.value = await listChapters();
  } catch (e) {
    error.value = "读取章节失败";
    console.debug("[chapter] list failed", e);
  } finally {
    loading.value = false;
  }
}

function seekToChapter(ch: ChapterEntry) {
  emit("seek", ch.start);
}

onMounted(() => {
  fetchChapters();
});
</script>

<style scoped>
.chapter-view {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.ch-loading,
.ch-empty {
  font-family: var(--font-mono);
  font-size: 11px;
  color: var(--text-muted);
  text-align: center;
  padding: 24px 0;
}

.ch-error {
  font-family: var(--font-mono);
  font-size: 11px;
  color: var(--accent-magenta);
  text-align: center;
  padding: 24px 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
}

.retry-btn {
  background: transparent;
  border: 1px solid var(--accent-magenta);
  color: var(--accent-magenta);
  font-family: var(--font-mono);
  font-size: 10px;
  padding: 3px 10px;
  border-radius: var(--radius-sm);
  cursor: pointer;
}

.ch-list {
  display: flex;
  flex-direction: column;
  gap: 3px;
}

.ch-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 7px 10px;
  border-radius: var(--radius-md);
  background: var(--bg-surface);
  border: 1px solid var(--border-subtle);
  cursor: pointer;
  transition: border-color 0.15s;
}

.ch-item:hover {
  border-color: var(--accent-cyan);
}

.ch-index {
  font-family: var(--font-mono);
  font-size: 11px;
  font-weight: 700;
  color: var(--accent-cyan);
  width: 20px;
  text-align: center;
  flex-shrink: 0;
}

.ch-info {
  display: flex;
  justify-content: space-between;
  align-items: center;
  flex: 1;
  min-width: 0;
  gap: 8px;
}

.ch-title {
  font-family: var(--font-mono);
  font-size: 11px;
  color: var(--text-primary);
  font-weight: 500;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.ch-time {
  font-family: var(--font-mono);
  font-size: 10px;
  color: var(--text-muted);
  flex-shrink: 0;
}
</style>
