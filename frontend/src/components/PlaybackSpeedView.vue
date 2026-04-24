<template>
  <div class="speed-view">
    <div class="speed-current">
      <span class="speed-label">当前速度</span>
      <span class="speed-value">{{ (props.modelValue || 1.0).toFixed(2) }}x</span>
    </div>

    <!-- <div class="speed-presets">
      <button v-for="preset in presets" :key="preset" class="speed-btn" :class="{ active: currentSpeed === preset }"
        @click="setSpeed(preset)">
        {{ preset }}x
      </button>
    </div> -->

    <div class="speed-slider-row">
      <span class="speed-slider-label">0.1x</span>
      <input type="range" class="speed-slider" min="0.1" max="16.0" step="0.05" :value="props.modelValue || 1.0"
        @input="handleSlider" />
      <span class="speed-slider-label">16.0x</span>
    </div>

    <div class="speed-hint">
      快捷键: ↑ 加速 · ↓ 减速 · R 重置
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from "vue";

const presets = [0.5, 0.75, 1.0, 1.25, 1.5, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0];

const props = withDefaults(
  defineProps<{
    modelValue?: number;
  }>(),
  { modelValue: 1.0 },
);

const emit = defineEmits<{
  (e: "speed-change", speed: number): void;
  (e: "update:modelValue", speed: number): void;
}>();

function setSpeed(speed: number) {
  emit("update:modelValue", speed);
  emit("speed-change", speed);
}

function handleSlider(event: Event) {
  const target = event.target as HTMLInputElement;
  const speed = parseFloat(target.value);
  setSpeed(speed);
}
</script>

<style scoped>
.speed-view {
  display: flex;
  flex-direction: column;
  gap: 16px;
  padding: 4px 0;
}

.speed-current {
  display: flex;
  justify-content: space-between;
  align-items: baseline;
  padding-bottom: 8px;
  border-bottom: 1px solid var(--border-subtle);
}

.speed-label {
  font-family: var(--font-mono);
  font-size: 11px;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.speed-value {
  font-family: var(--font-mono);
  font-size: 24px;
  font-weight: 700;
  color: var(--accent-cyan);
  letter-spacing: 0.02em;
}

.speed-presets {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
}

.speed-btn {
  flex: 1;
  min-width: 48px;
  background: var(--bg-surface);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  padding: 8px 6px;
  font-family: var(--font-mono);
  font-size: 12px;
  color: var(--text-primary);
  cursor: pointer;
  text-align: center;
  transition: all 0.15s;
}

.speed-btn:hover {
  border-color: var(--border-glow);
  color: var(--accent-cyan);
}

.speed-btn.active {
  background: rgba(0, 229, 255, 0.12);
  border-color: var(--accent-cyan);
  color: var(--accent-cyan);
  font-weight: 600;
}

.speed-slider-row {
  display: flex;
  align-items: center;
  gap: 10px;
}

.speed-slider-label {
  font-family: var(--font-mono);
  font-size: 10px;
  color: var(--text-muted);
  white-space: nowrap;
}

.speed-slider {
  -webkit-appearance: none;
  appearance: none;
  flex: 1;
  height: 4px;
  background: rgba(255, 255, 255, 0.1);
  border-radius: 2px;
  outline: none;
  cursor: pointer;
}

.speed-slider::-webkit-slider-thumb {
  -webkit-appearance: none;
  appearance: none;
  width: 14px;
  height: 14px;
  background: var(--accent-cyan);
  border-radius: 50%;
  box-shadow: 0 0 8px rgba(0, 229, 255, 0.4);
  cursor: pointer;
}

.speed-slider::-moz-range-thumb {
  width: 14px;
  height: 14px;
  background: var(--accent-cyan);
  border-radius: 50%;
  box-shadow: 0 0 8px rgba(0, 229, 255, 0.4);
  cursor: pointer;
  border: none;
}

.speed-hint {
  font-family: var(--font-mono);
  font-size: 10px;
  color: var(--text-muted);
  opacity: 0.6;
  text-align: center;
  padding-top: 4px;
}
</style>
