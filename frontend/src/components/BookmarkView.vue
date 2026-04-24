<template>
  <div class="bookmark-view">
    <!-- Add bookmark form -->
    <div class="add-row">
      <input
        v-model="newName"
        class="add-input"
        placeholder="书签名称..."
        maxlength="60"
        @keyup.enter="handleAdd"
      />
      <button class="add-btn" :disabled="adding || !newName.trim()" @click="handleAdd">
        {{ adding ? "..." : "添加" }}
      </button>
    </div>

    <div v-if="loading" class="bm-loading">加载中...</div>

    <div v-else-if="error" class="bm-error">
      {{ error }}
      <button class="retry-btn" @click="fetchBookmarks">重试</button>
    </div>

    <div v-else-if="bookmarks.length === 0" class="bm-empty">
      暂无书签，播放视频时添加书签标记精彩位置。
    </div>

    <div v-else class="bm-list">
      <div
        v-for="bm in bookmarks"
        :key="bm.id"
        class="bm-item"
        @click="seekToBookmark(bm)"
      >
        <div class="bm-info">
          <span class="bm-name">{{ bm.name }}</span>
          <span class="bm-time">{{ formatTime(bm.position) }}</span>
        </div>
        <button class="bm-delete" @click.stop="handleDelete(bm.id)" title="删除书签">
          &times;
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from "vue";
import {
  listBookmarks,
  addBookmark,
  deleteBookmark,
  type BookmarkEntry,
} from "../api/player";

const bookmarks = ref<BookmarkEntry[]>([]);
const newName = ref("");
const loading = ref(false);
const adding = ref(false);
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

async function fetchBookmarks() {
  loading.value = true;
  error.value = "";
  try {
    bookmarks.value = await listBookmarks();
  } catch (e) {
    error.value = "加载书签失败";
    console.debug("[bookmark] list failed", e);
  } finally {
    loading.value = false;
  }
}

async function handleAdd() {
  const name = newName.value.trim();
  if (!name) return;
  adding.value = true;
  try {
    await addBookmark(name);
    newName.value = "";
    await fetchBookmarks();
  } catch (e) {
    console.debug("[bookmark] add failed", e);
  } finally {
    adding.value = false;
  }
}

async function handleDelete(id: string) {
  try {
    await deleteBookmark(id);
    bookmarks.value = bookmarks.value.filter((b) => b.id !== id);
  } catch (e) {
    console.debug("[bookmark] delete failed", e);
  }
}

function seekToBookmark(bm: BookmarkEntry) {
  emit("seek", bm.position);
}

onMounted(() => {
  fetchBookmarks();
});
</script>

<style scoped>
.bookmark-view {
  display: flex;
  flex-direction: column;
  gap: 10px;
  min-height: 80px;
}

.add-row {
  display: flex;
  gap: 8px;
}

.add-input {
  flex: 1;
  background: var(--bg-elevated);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-sm);
  padding: 6px 10px;
  font-family: var(--font-mono);
  font-size: 12px;
  color: var(--text-primary);
  outline: none;
}

.add-input:focus {
  border-color: var(--accent-cyan);
}

.add-btn {
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

.add-btn:hover {
  background: rgba(0, 229, 255, 0.08);
}

.add-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.bm-loading,
.bm-empty {
  font-family: var(--font-mono);
  font-size: 11px;
  color: var(--text-muted);
  text-align: center;
  padding: 20px 0;
}

.bm-error {
  font-family: var(--font-mono);
  font-size: 11px;
  color: var(--accent-magenta);
  text-align: center;
  padding: 20px 0;
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

.bm-list {
  display: flex;
  flex-direction: column;
  gap: 3px;
}

.bm-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 7px 10px;
  border-radius: var(--radius-md);
  background: var(--bg-surface);
  border: 1px solid var(--border-subtle);
  cursor: pointer;
  transition: border-color 0.15s;
}

.bm-item:hover {
  border-color: var(--accent-cyan);
}

.bm-info {
  display: flex;
  flex-direction: column;
  gap: 1px;
  min-width: 0;
  flex: 1;
}

.bm-name {
  font-family: var(--font-mono);
  font-size: 11px;
  color: var(--text-primary);
  font-weight: 500;
}

.bm-time {
  font-family: var(--font-mono);
  font-size: 10px;
  color: var(--text-muted);
}

.bm-delete {
  background: none;
  border: none;
  color: var(--text-muted);
  font-size: 16px;
  cursor: pointer;
  padding: 0 2px;
  line-height: 1;
  flex-shrink: 0;
}

.bm-delete:hover {
  color: var(--accent-magenta);
}
</style>
