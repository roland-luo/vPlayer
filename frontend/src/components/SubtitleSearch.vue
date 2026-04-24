<template>
  <div class="subtitle-search">
    <div class="search-row">
      <input
        v-model="searchQuery"
        class="search-input"
        placeholder="搜索字幕..."
        @keyup.enter="handleSearch"
      />
      <button class="search-btn" :disabled="loading" @click="handleSearch">
        {{ loading ? "搜索中..." : "搜索" }}
      </button>
    </div>

    <div v-if="loading" class="subtitle-loading">正在搜索字幕...</div>

    <div v-else-if="error" class="subtitle-error">
      {{ error }}
      <button class="retry-btn" @click="handleSearch">重试</button>
    </div>

    <div v-else-if="results.length === 0" class="subtitle-empty">
      未找到字幕文件，尝试修改搜索关键词。
    </div>

    <div v-else class="subtitle-results">
      <div v-for="(result, index) in results" :key="index" class="subtitle-item">
        <div class="subtitle-info">
          <span class="subtitle-name">{{ result.name }}</span>
          <span class="subtitle-meta">
            {{ result.language }} · {{ result.format }}
            <span class="subtitle-source" :class="result.source">{{ result.source }}</span>
          </span>
        </div>
        <button
          class="download-btn"
          :disabled="downloading === result.path"
          @click="handleDownload(result)"
        >
          {{ downloading === result.path ? "下载中..." : "下载" }}
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from "vue";
import { searchSubtitles, downloadSubtitle, type SubtitleSearchResult } from "../api/player";

const searchQuery = ref("");
const results = ref<SubtitleSearchResult[]>([]);
const loading = ref(false);
const error = ref("");
const downloading = ref("");

const emit = defineEmits<{
  (e: "downloaded", path: string): void;
}>();

async function handleSearch() {
  loading.value = true;
  error.value = "";
  results.value = [];

  try {
    results.value = await searchSubtitles(searchQuery.value || undefined);
  } catch (e) {
    error.value = "搜索失败";
    console.debug("[subtitle] search failed", e);
  } finally {
    loading.value = false;
  }
}

async function handleDownload(result: SubtitleSearchResult) {
  downloading.value = result.path;
  try {
    const savedPath = await downloadSubtitle(result.path);
    emit("downloaded", savedPath);
  } catch (e) {
    console.debug("[subtitle] download failed", e);
  } finally {
    downloading.value = "";
  }
}

onMounted(() => {
  handleSearch();
});
</script>

<style scoped>
.subtitle-search {
  display: flex;
  flex-direction: column;
  gap: 12px;
  min-height: 100px;
}

.search-row {
  display: flex;
  gap: 8px;
}

.search-input {
  flex: 1;
  background: var(--bg-elevated);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-sm);
  padding: 6px 10px;
  font-family: var(--font-mono);
  font-size: 12px;
  color: var(--text-primary);
  outline: none;
  transition: border-color 0.15s;
}

.search-input:focus {
  border-color: var(--accent-cyan);
}

.search-btn {
  background: transparent;
  border: 1px solid var(--accent-cyan);
  color: var(--accent-cyan);
  font-family: var(--font-mono);
  font-size: 11px;
  padding: 4px 12px;
  border-radius: var(--radius-sm);
  cursor: pointer;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  white-space: nowrap;
}

.search-btn:hover {
  background: rgba(0, 229, 255, 0.08);
}

.search-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.subtitle-loading,
.subtitle-empty {
  font-family: var(--font-mono);
  font-size: 11px;
  color: var(--text-muted);
  text-align: center;
  padding: 24px 0;
}

.subtitle-error {
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

.subtitle-results {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.subtitle-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 10px;
  border-radius: var(--radius-md);
  background: var(--bg-surface);
  border: 1px solid var(--border-subtle);
}

.subtitle-item:hover {
  border-color: var(--border-glow);
}

.subtitle-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
  flex: 1;
}

.subtitle-name {
  font-family: var(--font-mono);
  font-size: 11px;
  color: var(--text-primary);
  font-weight: 500;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.subtitle-meta {
  font-family: var(--font-mono);
  font-size: 10px;
  color: var(--text-muted);
  display: flex;
  align-items: center;
  gap: 4px;
}

.subtitle-source {
  font-size: 9px;
  padding: 0 4px;
  border-radius: 2px;
  text-transform: uppercase;
  letter-spacing: 0.03em;
}

.subtitle-source.local {
  background: rgba(0, 229, 255, 0.1);
  color: var(--accent-cyan);
}

.subtitle-source.online {
  background: rgba(0, 200, 83, 0.1);
  color: #00c853;
}

.download-btn {
  background: transparent;
  border: 1px solid var(--border-subtle);
  color: var(--text-primary);
  font-family: var(--font-mono);
  font-size: 10px;
  padding: 3px 10px;
  border-radius: var(--radius-sm);
  cursor: pointer;
  flex-shrink: 0;
  text-transform: uppercase;
  letter-spacing: 0.03em;
}

.download-btn:hover {
  border-color: var(--accent-cyan);
  color: var(--accent-cyan);
}

.download-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
