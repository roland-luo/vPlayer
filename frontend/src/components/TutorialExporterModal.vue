<template>
  <Teleport to="body">
    <div v-if="visible" class="tutor-export-overlay" @click.self="close">
      <div class="tutor-export-card">
        <div class="tutor-export-header">
          <span class="tutor-export-title">EXPORT TUTORIAL</span>
          <button class="tutor-export-close" type="button" @click="close">×</button>
        </div>

        <div v-if="!hasVideo" class="tutor-export-empty">
          请先打开视频。
        </div>

        <div v-else class="tutor-export-body">
          <div class="tutor-export-field">
            <label for="export-filename" class="tutor-export-label">File name</label>
            <input
              id="export-filename"
              v-model="filename"
              class="tutor-export-input"
              :class="{ invalid: filenameInvalid }"
              type="text"
            />
            <span v-if="filenameError" class="tutor-export-error">{{ filenameError }}</span>
          </div>

          <div class="tutor-export-options">
            <label class="tutor-export-option">
              <input v-model="includeScreenshots" type="checkbox" />
              Include screenshots
            </label>
            <label class="tutor-export-option">
              <input v-model="groupByChapters" type="checkbox" />
              Group by chapters
            </label>
          </div>

          <div class="tutor-export-list-header">
            <span>Select notes</span>
            <button class="tutor-export-link" type="button" @click="toggleAll">
              {{ allSelected ? "Deselect all" : "Select all" }}
            </button>
          </div>

          <div class="tutor-export-list">
            <label
              v-for="bm in bookmarks"
              :key="bm.id"
              class="tutor-export-item"
            >
              <input v-model="selectedIds" type="checkbox" :value="bm.id" />
              <span class="tutor-export-item-time">{{ formatTime(bm.position) }}</span>
              <span class="tutor-export-item-content">{{ bm.name }}</span>
            </label>
          </div>

          <div class="tutor-export-actions">
            <button class="tutor-export-cancel" type="button" @click="close">Cancel</button>
            <button
              class="tutor-export-submit"
              type="button"
              :disabled="!canExport || exporting"
              @click="handleExport"
            >
              {{ exporting ? "..." : "Export .md" }}
            </button>
          </div>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, computed, watch } from "vue";
import { save } from "@tauri-apps/plugin-dialog";
import { open } from "@tauri-apps/plugin-shell";
import { dirname } from "@tauri-apps/api/path";
import type { BookmarkEntry } from "../api/player";
import { exportNotesToMarkdown } from "../api/tutorialExport";

const props = defineProps<{
  visible: boolean;
  hasVideo?: boolean;
  currentVideoPath?: string;
  currentVideoName?: string;
  bookmarks: BookmarkEntry[];
}>();

const emit = defineEmits<{
  close: [];
  success: [message: string];
  error: [message: string];
}>();

const filename = ref("");
const includeScreenshots = ref(true);
const groupByChapters = ref(true);
const selectedIds = ref<string[]>([]);
const exporting = ref(false);

const allSelected = computed(() =>
  props.bookmarks.length > 0 && selectedIds.value.length === props.bookmarks.length,
);

const canExport = computed(() => {
  if (!filename.value.trim()) return false;
  if (selectedIds.value.length === 0) return false;
  if (filenameInvalid.value) return false;
  return true;
});

const filenameInvalid = computed(() => {
  const name = filename.value;
  if (!name) return false;
  if (name.includes("/") || name.includes("\\")) return true;
  if (!name.endsWith(".md")) return true;
  return false;
});

const filenameError = computed(() => {
  const name = filename.value;
  if (!name) return "";
  if (name.includes("/") || name.includes("\\")) return "文件名不能包含路径分隔符";
  if (!name.endsWith(".md")) return "文件名需以 .md 结尾";
  return "";
});

function formatTime(seconds: number): string {
  if (!seconds || seconds < 0) return "00:00:00";
  const h = Math.floor(seconds / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  const s = Math.floor(seconds % 60);
  return `${String(h).padStart(2, "0")}:${String(m).padStart(2, "0")}:${String(s).padStart(2, "0")}`;
}

function toggleAll() {
  if (allSelected.value) {
    selectedIds.value = [];
  } else {
    selectedIds.value = props.bookmarks.map((bm) => bm.id);
  }
}

function close() {
  emit("close");
}

watch(
  () => props.visible,
  (isVisible) => {
    if (isVisible) {
      const base = props.currentVideoName || "notes";
      filename.value = `${base.replace(/\.[^.]+$/, "")}-notes.md`;
      selectedIds.value = props.bookmarks.map((bm) => bm.id);
      includeScreenshots.value = true;
      groupByChapters.value = true;
    }
  },
);

async function handleExport() {
  if (!canExport.value) return;
  if (!props.currentVideoPath) {
    emit("error", "视频路径为空");
    return;
  }

  const selectedDir = await save({
    defaultPath: filename.value,
    filters: [{ name: "Markdown", extensions: ["md"] }],
  });

  if (!selectedDir) return;

  const outputDir = await dirname(selectedDir);

  exporting.value = true;
  try {
    const result = await exportNotesToMarkdown({
      video_path: props.currentVideoPath,
      note_ids: selectedIds.value,
      output_dir: outputDir,
      filename: filename.value,
      include_screenshots: includeScreenshots.value,
      group_by_chapters: groupByChapters.value,
    });

    emit(
      "success",
      `已导出 ${result.note_count} 条笔记，${result.screenshot_count} 张截图`,
    );

    await open(outputDir);
    close();
  } catch (e) {
    const message = e instanceof Error ? e.message : String(e);
    emit("error", `导出失败：${message}`);
  } finally {
    exporting.value = false;
  }
}
</script>

<style scoped>
.tutor-export-overlay {
  position: fixed;
  inset: 0;
  z-index: 50;
  background: rgba(6, 6, 10, 0.6);
  display: flex;
  align-items: center;
  justify-content: center;
}

.tutor-export-card {
  width: 420px;
  max-height: 520px;
  background: rgba(18, 18, 26, 0.96);
  backdrop-filter: blur(16px);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-lg);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.5);
}

.tutor-export-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 14px;
  border-bottom: 1px solid var(--border-subtle);
}

.tutor-export-title {
  font-family: var(--font-display);
  font-size: 13px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  color: var(--text-primary);
}

.tutor-export-close {
  background: none;
  border: none;
  color: var(--text-muted);
  font-size: 18px;
  cursor: pointer;
}

.tutor-export-close:hover {
  color: var(--text-primary);
}

.tutor-export-body {
  padding: 14px;
  display: flex;
  flex-direction: column;
  gap: 12px;
  overflow-y: auto;
}

.tutor-export-empty {
  padding: 24px;
  text-align: center;
  font-family: var(--font-body);
  font-size: 12px;
  color: var(--text-muted);
}

.tutor-export-field {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.tutor-export-label {
  font-family: var(--font-body);
  font-size: 11px;
  color: var(--text-muted);
}

.tutor-export-input {
  background: var(--bg-elevated);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-sm);
  padding: 6px 10px;
  color: var(--text-primary);
  font-family: var(--font-body);
  font-size: 13px;
  outline: none;
}

.tutor-export-input:focus {
  border-color: var(--accent-cyan);
  box-shadow: 0 0 8px rgba(0, 229, 255, 0.2);
}

.tutor-export-input.invalid {
  border-color: var(--accent-magenta);
}

.tutor-export-error {
  font-family: var(--font-mono);
  font-size: 10px;
  color: var(--accent-magenta);
}

.tutor-export-options {
  display: flex;
  flex-direction: column;
  gap: 6px;
  font-family: var(--font-body);
  font-size: 12px;
  color: var(--text-muted);
}

.tutor-export-option {
  display: flex;
  align-items: center;
  gap: 6px;
  cursor: pointer;
}

.tutor-export-list-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-family: var(--font-body);
  font-size: 11px;
  color: var(--text-muted);
}

.tutor-export-link {
  background: none;
  border: none;
  color: var(--accent-cyan);
  font-family: var(--font-mono);
  font-size: 10px;
  cursor: pointer;
  padding: 0;
}

.tutor-export-list {
  max-height: 160px;
  overflow-y: auto;
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  padding: 6px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.tutor-export-item {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  padding: 6px;
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: background 0.15s ease-out;
}

.tutor-export-item:hover {
  background: rgba(255, 255, 255, 0.04);
}

.tutor-export-item-time {
  font-family: var(--font-mono);
  font-size: 11px;
  font-variant-numeric: tabular-nums;
  color: var(--accent-cyan);
  flex-shrink: 0;
}

.tutor-export-item-content {
  font-family: var(--font-body);
  font-size: 12px;
  color: var(--text-primary);
  line-height: 1.4;
  word-break: break-word;
}

.tutor-export-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 4px;
}

.tutor-export-cancel {
  background: transparent;
  border: 1px solid var(--border-subtle);
  color: var(--text-muted);
  font-family: var(--font-display);
  font-size: 11px;
  letter-spacing: 0.06em;
  font-weight: 600;
  text-transform: uppercase;
  padding: 4px 12px;
  border-radius: var(--radius-sm);
  cursor: pointer;
}

.tutor-export-cancel:hover {
  color: var(--text-primary);
  border-color: rgba(255, 255, 255, 0.12);
}

.tutor-export-submit {
  background: var(--accent-cyan);
  border: none;
  color: var(--bg-base);
  font-family: var(--font-display);
  font-size: 11px;
  letter-spacing: 0.06em;
  font-weight: 600;
  text-transform: uppercase;
  padding: 4px 12px;
  border-radius: var(--radius-sm);
  cursor: pointer;
}

.tutor-export-submit:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
