<template>
  <Teleport to="body">
    <div v-if="visible" class="plugin-popup-overlay" @click.self="close">
      <div
        class="plugin-popup-card"
        :style="{ width: props.popupWidth + 'px', maxHeight: props.popupHeight + 'px' }"
      >
        <div class="plugin-popup-header">
          <span class="plugin-popup-title">{{ pluginName }}</span>
          <button class="plugin-popup-close" @click="close">&times;</button>
        </div>
        <div class="plugin-popup-body">
          <div v-if="loading" class="plugin-popup-loading">Loading...</div>
          <div v-else-if="error" class="plugin-popup-error">{{ error }}</div>
          <div v-else class="plugin-popup-content">
            <SubtitleSearch
              v-if="pluginName === 'subtitle-download'"
              @downloaded="onSubtitleDownloaded"
            />
            <MediaInfoView v-else-if="pluginName === 'media-info'" />
            <PlaybackSpeedView
              v-else-if="pluginName === 'playback-speed'"
              :model-value="props.playbackSpeed"
              @speed-change="onSpeedChange"
            />
            <BookmarkView
              v-else-if="pluginName === 'bookmark'"
              @seek="onBookmarkSeek"
            />
            <p v-else class="plugin-popup-placeholder">
              {{ pluginName }} plugin content appears here.
            </p>
          </div>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, watch } from "vue";
import { getPluginDetail } from "../api/player";
import SubtitleSearch from "./SubtitleSearch.vue";
import MediaInfoView from "./MediaInfoView.vue";
import PlaybackSpeedView from "./PlaybackSpeedView.vue";
import BookmarkView from "./BookmarkView.vue";

const props = withDefaults(
  defineProps<{
    pluginName: string;
    visible: boolean;
    popupWidth?: number;
    popupHeight?: number;
    playbackSpeed?: number;
  }>(),
  { pluginName: "", visible: false, popupWidth: 400, popupHeight: 300, playbackSpeed: 1.0 },
);

const emit = defineEmits<{
  close: [];
  downloaded: [path: string];
  "speed-change": [speed: number];
  seek: [position: number];
}>();

const loading = ref(false);
const error = ref("");

function close() {
  emit("close");
}

function onSubtitleDownloaded(path: string) {
  emit("downloaded", path);
  close();
}

function onSpeedChange(speed: number) {
  emit("speed-change", speed);
}

function onBookmarkSeek(position: number) {
  emit("seek", position);
}

watch(
  () => props.visible,
  async (isVisible) => {
    if (!isVisible || !props.pluginName) return;

    loading.value = true;
    error.value = "";
    try {
      await getPluginDetail(props.pluginName);
    } catch (e) {
      error.value = "Failed to load plugin data.";
      console.debug("[plugin-popup] load failed", e);
    } finally {
      loading.value = false;
    }
  },
);
</script>

<style scoped>
.plugin-popup-overlay {
  position: fixed;
  inset: 0;
  z-index: 50;
  background: rgba(6, 6, 10, 0.6);
  display: flex;
  align-items: center;
  justify-content: center;
}

.plugin-popup-card {
  background: rgba(18, 18, 26, 0.96);
  backdrop-filter: blur(16px);
  -webkit-backdrop-filter: blur(16px);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-lg);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.5);
}

.plugin-popup-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 14px;
  border-bottom: 1px solid var(--border-subtle);
}

.plugin-popup-title {
  font-family: var(--font-display);
  font-size: 13px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  color: var(--text-primary);
}

.plugin-popup-close {
  background: none;
  border: none;
  color: var(--text-muted);
  font-size: 18px;
  cursor: pointer;
  padding: 0 4px;
  line-height: 1;
}

.plugin-popup-close:hover {
  color: var(--text-primary);
}

.plugin-popup-body {
  flex: 1;
  padding: 14px;
  overflow-y: auto;
}

.plugin-popup-loading,
.plugin-popup-error {
  font-family: var(--font-mono);
  font-size: 12px;
  color: var(--text-muted);
  text-align: center;
  padding: 24px 0;
}

.plugin-popup-error {
  color: var(--accent-magenta);
}

.plugin-popup-content {
  min-height: 60px;
}

.plugin-popup-placeholder {
  font-family: var(--font-mono);
  font-size: 11px;
  color: var(--text-muted);
  text-align: center;
  padding: 24px 0;
}
</style>
