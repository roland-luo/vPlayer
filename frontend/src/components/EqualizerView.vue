<template>
  <div class="equalizer-view">
    <!-- Presets -->
    <div class="eq-presets">
      <button
        v-for="preset in PRESETS"
        :key="preset.name"
        class="eq-preset-btn"
        :class="{ active: activePreset === preset.name }"
        @click="onPreset(preset.name)"
      >
        {{ preset.label }}
      </button>
    </div>

    <!-- Band sliders -->
    <div class="eq-bands">
      <div v-for="band in bands" :key="band.index" class="eq-band">
        <span class="eq-db">{{ band.gain > 0 ? "+" : "" }}{{ band.gain.toFixed(1) }}</span>
        <div class="eq-slider-wrap">
          <input
            type="range"
            class="eq-slider"
            min="-12"
            max="12"
            step="0.5"
            :value="band.gain"
            @input="onSlider(band.index, ($event.target as HTMLInputElement).value)"
          />
        </div>
        <span class="eq-freq">{{ band.label }}</span>
      </div>
    </div>

    <!-- Footer: connect status / error -->
    <div class="eq-footer">
      <span v-if="!isConnected" class="eq-status eq-status-init">正在初始化...</span>
      <span v-else class="eq-status eq-status-ok">已连接</span>
      <button class="eq-reset-btn" :disabled="!isConnected" @click="onReset">重置</button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useAudioEqualizer, PRESETS } from "../composables/useAudioEqualizer";

const { bands, activePreset, isConnected, connect, setGain, applyPreset, reset } =
  useAudioEqualizer();

const initError = ref("");

onMounted(async () => {
  try {
    await connect();
  } catch (e) {
    initError.value = "无法连接音频均衡器";
    console.debug("[equalizer] connect failed", e);
  }
});

function onSlider(index: number, raw: string) {
  setGain(index, parseFloat(raw));
}

function onPreset(name: string) {
  applyPreset(name);
}

function onReset() {
  reset();
}
</script>

<style scoped>
.equalizer-view {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

/* ── Presets ── */
.eq-presets {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
}

.eq-preset-btn {
  flex: 1;
  min-width: 56px;
  background: var(--bg-surface);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  padding: 6px 8px;
  font-family: var(--font-mono);
  font-size: 11px;
  color: var(--text-primary);
  cursor: pointer;
  text-align: center;
  transition: all 0.15s;
}

.eq-preset-btn:hover {
  border-color: var(--border-glow);
  color: var(--accent-cyan);
}

.eq-preset-btn.active {
  background: rgba(0, 229, 255, 0.12);
  border-color: var(--accent-cyan);
  color: var(--accent-cyan);
  font-weight: 600;
}

/* ── Bands ── */
.eq-bands {
  display: flex;
  gap: 6px;
  justify-content: space-between;
}

.eq-band {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  flex: 1;
  min-width: 0;
}

.eq-db {
  font-family: var(--font-mono);
  font-size: 10px;
  color: var(--text-primary);
  font-weight: 600;
  min-height: 1.4em;
  text-align: center;
}

.eq-slider-wrap {
  height: 120px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.eq-slider {
  -webkit-appearance: none;
  appearance: none;
  writing-mode: vertical-lr;
  direction: rtl;
  width: 4px;
  height: 120px;
  background: rgba(255, 255, 255, 0.1);
  border-radius: 2px;
  outline: none;
  cursor: pointer;
}

.eq-slider::-webkit-slider-thumb {
  -webkit-appearance: none;
  appearance: none;
  width: 14px;
  height: 14px;
  background: var(--accent-cyan);
  border-radius: 50%;
  box-shadow: 0 0 8px rgba(0, 229, 255, 0.4);
  cursor: pointer;
}

.eq-slider::-moz-range-thumb {
  width: 14px;
  height: 14px;
  background: var(--accent-cyan);
  border-radius: 50%;
  box-shadow: 0 0 8px rgba(0, 229, 255, 0.4);
  cursor: pointer;
  border: none;
}

.eq-freq {
  font-family: var(--font-mono);
  font-size: 9px;
  color: var(--text-muted);
  text-align: center;
}

/* ── Footer ── */
.eq-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding-top: 4px;
  border-top: 1px solid var(--border-subtle);
}

.eq-status {
  font-family: var(--font-mono);
  font-size: 10px;
}

.eq-status-init {
  color: var(--text-muted);
}

.eq-status-ok {
  color: var(--accent-cyan);
}

.eq-reset-btn {
  background: transparent;
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-sm);
  padding: 3px 10px;
  font-family: var(--font-mono);
  font-size: 10px;
  color: var(--text-muted);
  cursor: pointer;
  transition: all 0.15s;
}

.eq-reset-btn:hover:not(:disabled) {
  border-color: var(--accent-cyan);
  color: var(--accent-cyan);
}

.eq-reset-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}
</style>
