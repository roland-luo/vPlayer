<template>
  <div class="note-view">
    <!-- Header -->
    <div class="note-header">
      <div class="note-header-left">
        <span class="note-header-title">NOTES</span>
        <span class="note-header-meta" v-if="currentVideoName">
          &middot; {{ currentVideoName }}
        </span>
        <span class="note-header-count" v-if="bookmarks.length > 0">
          {{ bookmarks.length }} 条
        </span>
      </div>
      <button
        v-if="bookmarks.length > 0"
        class="note-export-btn"
        type="button"
        :aria-label="`导出 ${bookmarks.length} 条笔记`"
        @click="handleExport"
      >
        <svg class="note-export-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
          <rect x="9" y="3" width="6" height="4" rx="1"/>
          <path d="M9 7v10a2 2 0 0 0 2 2h2a2 2 0 0 0 2-2V7"/>
          <path d="M5 15h14"/>
          <path d="M12 15v4"/>
        </svg>
        EXPORT
      </button>
    </div>
    <div class="note-session-stats">
      本次会话: 新增 {{ sessionAddCount }} · 回跳 {{ sessionSeekCount }}
    </div>

    <!-- Position chip -->
    <div class="note-chip" :class="{ paused: isPausedByNote }">
      <svg class="note-chip-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
        <path d="M12 2C8.13 2 5 5.13 5 9c0 5.25 7 13 7 13s7-7.75 7-13c0-3.87-3.13-7-7-7z"/>
        <circle cx="12" cy="9" r="2.5"/>
      </svg>
      <span class="note-chip-time" role="status" aria-live="polite">{{ formatTime(currentPosition ?? 0) }}</span>
      <span v-if="isPausedByNote" class="note-chip-status">PAUSED</span>
    </div>

    <!-- Input area -->
    <div class="note-input-wrap">
      <label for="note-input" class="visually-hidden">笔记内容</label>
      <textarea
        id="note-input"
        ref="textareaRef"
        v-model="newName"
        class="note-textarea"
        :class="{ disabled: !hasVideo }"
        :placeholder="hasVideo ? '记一笔（Enter 保存，Shift+Enter 换行）' : '请先打开视频'"
        :disabled="!hasVideo"
        :maxlength="2000"
        rows="2"
        aria-label="笔记内容"
        @focus="onTextareaFocus"
        @blur="onTextareaBlur"
        @keydown="onTextareaKeydown"
        @input="onTextareaInput"
      />
      <div v-if="showHint" class="note-hint">
        ENTER 保存 &middot; SHIFT+ENTER 换行 &middot; ESC 取消
      </div>
    </div>

    <!-- Action row -->
    <div class="note-actions">
      <button
        class="note-add-btn"
        type="button"
        :aria-disabled="!canAdd"
        :disabled="!canAdd"
        @click="handleAdd"
      >
        {{ adding ? "..." : "ADD" }}
      </button>
      <button class="note-toggle-btn" type="button" :aria-pressed="autoPauseOnFocus" @click="toggleAutoPause">
        {{ autoPauseOnFocus ? "AUTO-PAUSE ON" : "AUTO-PAUSE OFF" }}
      </button>
    </div>

    <!-- List -->
    <div v-if="loading" class="note-loading">加载中...</div>

    <div v-else-if="error" class="note-error">
      {{ error }}
      <button class="note-retry-btn" type="button" @click="fetchBookmarks">重试</button>
    </div>

    <div v-else-if="!hasVideo" class="note-empty">
      请先打开视频，开始记笔记。
    </div>

    <div v-else-if="bookmarks.length === 0" class="note-empty">
      暂无笔记，按 N 聚焦输入框开始记录。
    </div>

    <ul v-else class="note-list" role="list" aria-label="笔记列表">
      <li v-for="bm in bookmarks" :key="bm.id" class="note-list-item">
        <button
          class="note-item-btn"
          type="button"
          @click="seekToNote(bm)"
        >
          <svg class="note-item-play-icon" viewBox="0 0 24 24" fill="currentColor">
            <polygon points="5,3 19,12 5,21"/>
          </svg>
          <div class="note-item-body">
            <span class="note-item-time">{{ formatTime(bm.position) }}</span>
            <span class="note-item-content">{{ bm.name }}</span>
          </div>
        </button>
        <button
          class="note-item-delete"
          type="button"
          :aria-label="`删除笔记 ${bm.name}`"
          @click="handleDelete(bm.id)"
        >
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" class="note-delete-icon">
            <line x1="18" y1="6" x2="6" y2="18"/>
            <line x1="6" y1="6" x2="18" y2="18"/>
          </svg>
        </button>
      </li>
    </ul>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch, nextTick } from "vue";
import {
  listBookmarks,
  addBookmark,
  deleteBookmark,
  type BookmarkEntry,
} from "../api/player";

const props = defineProps<{
  currentPosition?: number;
  hasVideo?: boolean;
  currentVideoName?: string;
  currentVideoPath?: string;
  isPlaying?: boolean;
}>();

const emit = defineEmits<{
  (e: "seek", position: number): void;
  (e: "pause"): void;
  (e: "resume"): void;
  (e: "open-exporter"): void;
  (e: "bookmarks-change", bookmarks: BookmarkEntry[]): void;
}>();

const bookmarks = ref<BookmarkEntry[]>([]);
const newName = ref("");
const loading = ref(false);
const adding = ref(false);
const error = ref("");
const textareaRef = ref<HTMLTextAreaElement | null>(null);
const isPausedByNote = ref(false);
const wasPlayingBeforeFocus = ref(false);

const SEEK_OFFSET = 3;
const AUTO_PAUSE_SETTING_KEY = "vplayer:note-auto-pause";
const autoPauseOnFocus = ref(true);
const sessionAddCount = ref(0);
const sessionSeekCount = ref(0);

const canAdd = computed(() => !adding.value && newName.value.trim().length > 0 && props.hasVideo);

const showHint = computed(() => {
  // Persist hint dismissal in localStorage
  if (typeof window === "undefined") return true;
  return localStorage.getItem("vplayer:note-hint-dismissed") !== "1";
});

function formatTime(seconds: number): string {
  if (!seconds || seconds < 0) return "00:00:00";
  const h = Math.floor(seconds / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  const s = Math.floor(seconds % 60);
  return `${String(h).padStart(2, "0")}:${String(m).padStart(2, "0")}:${String(s).padStart(2, "0")}`;
}

async function fetchBookmarks() {
  if (!props.hasVideo) {
    bookmarks.value = [];
    emit("bookmarks-change", []);
    return;
  }
  loading.value = true;
  error.value = "";
  try {
    bookmarks.value = await listBookmarks();
    emit("bookmarks-change", bookmarks.value);
  } catch (e) {
    error.value = "加载笔记失败";
    console.debug("[note] list failed", e);
  } finally {
    loading.value = false;
  }
}

async function handleAdd() {
  const name = newName.value.trim();
  if (!name || !props.hasVideo) return;
  adding.value = true;
  try {
    await addBookmark(name, props.currentPosition ?? 0);
    sessionAddCount.value += 1;
    newName.value = "";
    const el = textareaRef.value;
    if (el) el.style.height = "auto";
    await fetchBookmarks();
    // Dismiss hint after first successful add
    if (typeof window !== "undefined") {
      localStorage.setItem("vplayer:note-hint-dismissed", "1");
    }
  } catch (e) {
    console.debug("[note] add failed", e);
  } finally {
    adding.value = false;
  }
}

async function handleDelete(id: string) {
  try {
    await deleteBookmark(id);
    bookmarks.value = bookmarks.value.filter((b) => b.id !== id);
    emit("bookmarks-change", bookmarks.value);
  } catch (e) {
    console.debug("[note] delete failed", e);
  }
}

function seekToNote(bm: BookmarkEntry) {
  const target = Math.max(0, bm.position - SEEK_OFFSET);
  sessionSeekCount.value += 1;
  emit("seek", target);
  emit("resume");
}

function onTextareaFocus() {
  if (!props.hasVideo) return;
  if (!autoPauseOnFocus.value) return;
  wasPlayingBeforeFocus.value = props.isPlaying ?? false;
  isPausedByNote.value = true;
  emit("pause");
}

function onTextareaBlur() {
  if (!props.hasVideo) return;
  if (!autoPauseOnFocus.value) return;
  isPausedByNote.value = false;
  if (newName.value.trim().length === 0 && wasPlayingBeforeFocus.value) {
    emit("resume");
  }
}

function onTextareaKeydown(event: KeyboardEvent) {
  if (event.key === "Enter" && !event.shiftKey) {
    event.preventDefault();
    handleAdd();
    return;
  }
  if (event.key === "Escape") {
    event.preventDefault();
    newName.value = "";
    textareaRef.value?.blur();
    if (wasPlayingBeforeFocus.value) {
      emit("resume");
    }
    return;
  }
}

function onTextareaInput() {
  const el = textareaRef.value;
  if (!el) return;
  el.style.height = "auto";
  const lineHeight = 21; // 14px * 1.5
  const minHeight = lineHeight * 2;
  const maxHeight = lineHeight * 4;
  const scrollHeight = el.scrollHeight;
  const nextHeight = Math.min(Math.max(scrollHeight, minHeight), maxHeight);
  el.style.height = `${nextHeight}px`;
}

function handleExport() {
  if (bookmarks.value.length === 0) return;
  emit("open-exporter");
}

function toggleAutoPause() {
  autoPauseOnFocus.value = !autoPauseOnFocus.value;
  if (typeof window !== "undefined") {
    localStorage.setItem(AUTO_PAUSE_SETTING_KEY, autoPauseOnFocus.value ? "1" : "0");
  }
}

// Public method for App.vue hotkey
function focusInput() {
  nextTick(() => {
    textareaRef.value?.focus();
  });
}

watch(() => [props.hasVideo, props.currentVideoPath], ([hasVideo]) => {
  if (hasVideo) {
    fetchBookmarks();
  } else {
    bookmarks.value = [];
    emit("bookmarks-change", []);
  }
  newName.value = "";
  isPausedByNote.value = false;
  const el = textareaRef.value;
  if (el) el.style.height = "auto";
});

onMounted(() => {
  if (typeof window !== "undefined") {
    autoPauseOnFocus.value = localStorage.getItem(AUTO_PAUSE_SETTING_KEY) !== "0";
  }
  if (props.hasVideo) {
    fetchBookmarks();
  }
});

onUnmounted(() => {
  // nothing to clean up
});

defineExpose({
  focusInput,
});
</script>

<style scoped>
.note-view {
  display: flex;
  flex-direction: column;
  gap: 12px;
  min-height: 80px;
  position: relative;
}

/* Header */
.note-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 0;
  gap: 4px;
}

.note-header-left {
  display: flex;
  align-items: center;
  gap: 4px;
  min-width: 0;
}

.note-header-title {
  font-family: var(--font-display);
  font-size: 11px;
  letter-spacing: 0.08em;
  font-weight: 600;
  text-transform: uppercase;
  color: var(--text-primary);
}

.note-header-meta {
  font-family: var(--font-body);
  font-size: 12px;
  color: var(--text-muted);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.note-header-count {
  font-family: var(--font-mono);
  font-size: 11px;
  letter-spacing: 0.04em;
  font-weight: 500;
  color: var(--text-muted);
}

.note-session-stats {
  font-family: var(--font-mono);
  font-size: 10px;
  letter-spacing: 0.03em;
  color: var(--text-muted);
}

/* Export button */
.note-export-btn {
  display: flex;
  align-items: center;
  gap: 4px;
  background: transparent;
  border: none;
  color: var(--text-muted);
  font-family: var(--font-display);
  font-size: 11px;
  letter-spacing: 0.06em;
  font-weight: 600;
  text-transform: uppercase;
  padding: 4px 8px;
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: color 0.15s ease-out, box-shadow 0.15s ease-out;
}

.note-export-btn:hover {
  color: var(--text-primary);
}

.note-export-btn:focus-visible {
  box-shadow: 0 0 0 2px var(--accent-cyan);
  outline: 2px solid transparent;
  outline-offset: 2px;
}

.note-export-icon {
  width: 14px;
  height: 14px;
  color: var(--accent-cyan);
}

/* Chip */
.note-chip {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  align-self: flex-start;
  padding: 4px 12px;
  border-radius: 9999px;
  background: rgba(0, 229, 255, 0.10);
  border: 1px solid rgba(0, 229, 255, 0.20);
  transition: background 0.2s ease-in-out, border-color 0.2s ease-in-out;
}

.note-chip.paused {
  background: rgba(255, 0, 110, 0.10);
  border-color: rgba(255, 0, 110, 0.20);
}

.note-chip-icon {
  width: 12px;
  height: 12px;
  color: var(--accent-cyan);
  transition: color 0.2s ease-in-out;
}

.note-chip.paused .note-chip-icon {
  color: var(--accent-magenta);
}

.note-chip-time {
  font-family: var(--font-mono);
  font-size: 12px;
  font-variant-numeric: tabular-nums;
  letter-spacing: 0.04em;
  font-weight: 500;
  color: var(--accent-cyan);
  transition: color 0.2s ease-in-out;
}

.note-chip.paused .note-chip-time {
  color: var(--accent-magenta);
}

.note-chip-status {
  font-family: var(--font-display);
  font-size: 10px;
  letter-spacing: 0.08em;
  font-weight: 600;
  text-transform: uppercase;
  color: var(--accent-magenta);
}

/* Textarea */
.note-input-wrap {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.note-textarea {
  width: 100%;
  background: var(--bg-elevated);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-sm);
  padding: 8px 12px;
  font-family: var(--font-body);
  font-size: 14px;
  color: var(--text-primary);
  outline: none;
  resize: vertical;
  min-height: 44px;
  max-height: 120px;
  line-height: 1.5;
  transition: border-color 0.15s ease-out, box-shadow 0.15s ease-out;
}

.note-textarea::placeholder {
  font-family: var(--font-body);
  font-size: 13px;
  color: var(--text-muted);
}

.note-textarea:hover {
  border-color: rgba(255, 255, 255, 0.12);
}

.note-textarea:focus {
  border-color: var(--accent-cyan);
  box-shadow: 0 0 8px rgba(0, 229, 255, 0.20);
}

.note-textarea.disabled,
.note-textarea:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* Hint */
.note-hint {
  font-family: var(--font-mono);
  font-size: 10px;
  letter-spacing: 0.04em;
  font-weight: 400;
  color: var(--text-muted);
}

/* Actions */
.note-actions {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.note-add-btn {
  background: transparent;
  border: 1px solid var(--accent-cyan);
  color: var(--accent-cyan);
  font-family: var(--font-display);
  font-size: 11px;
  letter-spacing: 0.06em;
  font-weight: 600;
  text-transform: uppercase;
  padding: 4px 12px;
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: background 0.15s ease-out, box-shadow 0.15s ease-out;
  white-space: nowrap;
}

.note-add-btn:hover {
  background: rgba(0, 229, 255, 0.08);
}

.note-add-btn:focus-visible {
  box-shadow: 0 0 0 2px var(--accent-cyan);
  outline: 2px solid transparent;
  outline-offset: 2px;
}

.note-add-btn:active {
  background: rgba(0, 229, 255, 0.16);
}

.note-add-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.note-toggle-btn {
  background: transparent;
  border: 1px solid var(--border-subtle);
  color: var(--text-muted);
  font-family: var(--font-display);
  font-size: 10px;
  letter-spacing: 0.05em;
  font-weight: 600;
  text-transform: uppercase;
  padding: 4px 10px;
  border-radius: var(--radius-sm);
  cursor: pointer;
}

.note-toggle-btn[aria-pressed="true"] {
  border-color: var(--accent-cyan);
  color: var(--accent-cyan);
}

/* Loading / Error / Empty */
.note-loading,
.note-empty {
  font-family: var(--font-body);
  font-size: 12px;
  color: var(--text-muted);
  text-align: center;
  padding: 20px 0;
}

.note-error {
  font-family: var(--font-body);
  font-size: 11px;
  color: var(--accent-magenta);
  text-align: center;
  padding: 20px 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
}

.note-retry-btn {
  background: transparent;
  border: 1px solid var(--accent-magenta);
  color: var(--accent-magenta);
  font-family: var(--font-mono);
  font-size: 10px;
  padding: 3px 10px;
  border-radius: var(--radius-sm);
  cursor: pointer;
}

/* List */
.note-list {
  list-style: none;
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding: 0;
  margin: 0;
}

.note-list-item {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 7px 10px;
  border-radius: var(--radius-md);
  background: var(--bg-surface);
  border: 1px solid var(--border-subtle);
  transition: border-color 0.15s ease-out;
}

.note-list-item:hover {
  border-color: var(--accent-cyan);
}

.note-item-btn {
  display: flex;
  align-items: flex-start;
  gap: 6px;
  flex: 1;
  min-width: 0;
  background: none;
  border: none;
  color: inherit;
  text-align: left;
  cursor: pointer;
  padding: 0;
  margin: 0;
}

.note-item-btn:focus-visible {
  outline: none;
}

.note-list-item:focus-within {
  box-shadow: 0 0 0 2px var(--accent-cyan);
}

.note-item-play-icon {
  width: 10px;
  height: 10px;
  margin-top: 3px;
  flex-shrink: 0;
  color: var(--text-muted);
  transition: color 0.15s ease-out;
}

.note-list-item:hover .note-item-play-icon {
  color: var(--accent-cyan);
}

.note-item-body {
  display: flex;
  flex-direction: column;
  gap: 1px;
  min-width: 0;
  flex: 1;
}

.note-item-time {
  font-family: var(--font-mono);
  font-size: 11px;
  font-variant-numeric: tabular-nums;
  letter-spacing: 0.04em;
  font-weight: 500;
  color: var(--text-primary);
}

.note-item-content {
  font-family: var(--font-body);
  font-size: 12px;
  color: var(--text-primary);
  white-space: pre-wrap;
  word-break: break-word;
  line-height: 1.4;
}

.note-item-delete {
  background: none;
  border: none;
  color: var(--text-muted);
  cursor: pointer;
  padding: 2px;
  line-height: 1;
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--radius-sm);
  transition: color 0.15s ease-out;
}

.note-item-delete:hover {
  color: var(--accent-magenta);
}

.note-item-delete:focus-visible {
  box-shadow: 0 0 0 2px var(--accent-cyan);
  outline: 2px solid transparent;
  outline-offset: 2px;
}

.note-delete-icon {
  width: 14px;
  height: 14px;
}

/* Visually hidden for a11y */
.visually-hidden {
  position: absolute;
  width: 1px;
  height: 1px;
  padding: 0;
  margin: -1px;
  overflow: hidden;
  clip: rect(0, 0, 0, 0);
  white-space: nowrap;
  border: 0;
}
</style>
