<template>
  <div class="media-info">
    <div v-if="loading" class="mi-loading">正在读取媒体信息...</div>

    <div v-else-if="error" class="mi-error">
      {{ error }}
      <button class="retry-btn" @click="fetchInfo">重试</button>
    </div>

    <div v-else-if="!info" class="mi-empty">请先播放一个视频文件。</div>

    <div v-else class="mi-table">
      <div class="mi-section">
        <div class="mi-section-title">封装格式</div>
        <div class="mi-row"><span class="mi-label">容器</span><span class="mi-value">{{ info.container }}</span></div>
        <div class="mi-row"><span class="mi-label">时长</span><span class="mi-value">{{ info.duration }}</span></div>
        <div class="mi-row"><span class="mi-label">文件大小</span><span class="mi-value">{{ info.size }}</span></div>
        <div class="mi-row"><span class="mi-label">总码率</span><span class="mi-value">{{ info.bit_rate }}</span></div>
      </div>

      <div class="mi-section">
        <div class="mi-section-title">视频</div>
        <div class="mi-row"><span class="mi-label">编码</span><span class="mi-value">{{ info.video_codec }}</span></div>
        <div class="mi-row"><span class="mi-label">分辨率</span><span class="mi-value">{{ info.video_resolution }}</span></div>
        <div class="mi-row"><span class="mi-label">码率</span><span class="mi-value">{{ info.video_bitrate }}</span></div>
        <div class="mi-row"><span class="mi-label">帧率</span><span class="mi-value">{{ info.video_fps }}</span></div>
      </div>

      <div class="mi-section">
        <div class="mi-section-title">音频</div>
        <div class="mi-row"><span class="mi-label">编码</span><span class="mi-value">{{ info.audio_codec }}</span></div>
        <div class="mi-row"><span class="mi-label">声道</span><span class="mi-value">{{ info.audio_channels }}</span></div>
        <div class="mi-row"><span class="mi-label">采样率</span><span class="mi-value">{{ info.audio_sample_rate }}</span></div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from "vue";
import { getMediaInfo, type MediaInfoResult } from "../api/player";

const loading = ref(false);
const error = ref("");
const info = ref<MediaInfoResult | null>(null);

async function fetchInfo() {
  loading.value = true;
  error.value = "";
  info.value = null;

  try {
    info.value = await getMediaInfo();
  } catch (e) {
    error.value = "读取媒体信息失败";
    console.debug("[media-info] fetch failed", e);
  } finally {
    loading.value = false;
  }
}

onMounted(() => {
  fetchInfo();
});
</script>

<style scoped>
.media-info {
  font-family: var(--font-mono);
  font-size: 11px;
}

.mi-loading,
.mi-empty {
  color: var(--text-muted);
  text-align: center;
  padding: 24px 0;
}

.mi-error {
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

.mi-section {
  margin-bottom: 12px;
}

.mi-section-title {
  font-size: 10px;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  color: var(--accent-cyan);
  margin-bottom: 4px;
  padding-bottom: 3px;
  border-bottom: 1px solid var(--border-subtle);
}

.mi-row {
  display: flex;
  justify-content: space-between;
  padding: 3px 0;
}

.mi-label {
  color: var(--text-muted);
}

.mi-value {
  color: var(--text-primary);
  text-align: right;
}
</style>
