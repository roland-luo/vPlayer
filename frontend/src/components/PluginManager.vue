<template>
  <div class="plugin-manager">
    <div class="plugin-header">
      <h2 class="plugin-title">Plugins</h2>
      <span class="plugin-count">{{ plugins.length }} installed</span>
    </div>

    <div v-if="plugins.length === 0" class="plugin-empty">
      <div class="plugin-empty-icon"></div>
      <p class="plugin-empty-text">No plugins installed</p>
      <p class="plugin-empty-hint">Plugins go in the plugins directory to be loaded automatically.</p>
    </div>

    <div v-else class="plugin-list">
      <div v-for="plugin in plugins" :key="plugin.name" class="plugin-item">
        <div class="plugin-info">
          <div class="plugin-name-row">
            <span class="plugin-name">{{ plugin.name }}</span>
            <span class="plugin-version">v{{ plugin.version }}</span>
          </div>
          <div class="plugin-meta">
            <span v-if="plugin.error_count > 0" class="plugin-status plugin-status-error">
              Crashed ({{ plugin.error_count }})
            </span>
            <span v-else-if="plugin.enabled" class="plugin-status plugin-status-enabled">
              Enabled
            </span>
            <span v-else class="plugin-status plugin-status-disabled">
              Disabled
            </span>
            <span v-if="plugin.last_error" class="plugin-last-error" :title="plugin.last_error">
              {{ plugin.last_error }}
            </span>
          </div>
        </div>
        <div class="plugin-toggle">
          <button
            class="toggle-btn"
            :class="{ active: plugin.enabled }"
            @click="togglePlugin(plugin)"
          >
            <span class="toggle-track">
              <span class="toggle-thumb"></span>
            </span>
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref } from "vue";
import type { PluginInfo } from "../api/player";
import { listPlugins, togglePlugin as togglePluginApi } from "../api/player";

const plugins = ref<PluginInfo[]>([]);

async function fetchPlugins() {
  try {
    plugins.value = await listPlugins();
    // Sort: enabled first, then disabled, then crashed
    plugins.value.sort((a, b) => {
      if (a.error_count > 0 && b.error_count === 0) return 1;
      if (a.error_count === 0 && b.error_count > 0) return -1;
      if (a.enabled !== b.enabled) return a.enabled ? -1 : 1;
      return a.name.localeCompare(b.name);
    });
  } catch (e) {
    console.debug("[plugin-manager] failed to load plugins", e);
  }
}

async function togglePlugin(plugin: PluginInfo) {
  const newEnabled = !plugin.enabled;
  try {
    await togglePluginApi(plugin.name, newEnabled);
    plugin.enabled = newEnabled;
  } catch (e) {
    console.debug("[plugin-manager] toggle failed", e);
  }
}

onMounted(() => {
  fetchPlugins();
});
</script>

<style scoped>
.plugin-manager {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 0;
}

.plugin-header {
  display: flex;
  align-items: baseline;
  gap: 8px;
  padding-bottom: 8px;
  border-bottom: 1px solid var(--border-subtle);
}

.plugin-title {
  font-family: var(--font-display);
  font-size: 14px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  color: var(--text-primary);
}

.plugin-count {
  font-family: var(--font-mono);
  font-size: 11px;
  color: var(--text-muted);
}

.plugin-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
  padding: 24px 0;
  text-align: center;
}

.plugin-empty-icon {
  width: 32px;
  height: 32px;
  border: 1px dashed var(--border-subtle);
  border-radius: 50%;
  margin-bottom: 4px;
}

.plugin-empty-text {
  font-size: 13px;
  color: var(--text-muted);
}

.plugin-empty-hint {
  font-size: 11px;
  color: var(--text-muted);
  opacity: 0.7;
  max-width: 240px;
}

.plugin-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.plugin-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 10px;
  border-radius: var(--radius-md);
  background: var(--bg-surface);
  border: 1px solid var(--border-subtle);
}

.plugin-item:hover {
  border-color: var(--border-glow);
}

.plugin-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}

.plugin-name-row {
  display: flex;
  align-items: baseline;
  gap: 6px;
}

.plugin-name {
  font-family: var(--font-mono);
  font-size: 12px;
  color: var(--text-primary);
  font-weight: 500;
}

.plugin-version {
  font-family: var(--font-mono);
  font-size: 10px;
  color: var(--text-muted);
}

.plugin-meta {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-wrap: wrap;
}

.plugin-status {
  font-family: var(--font-mono);
  font-size: 10px;
  padding: 1px 6px;
  border-radius: var(--radius-sm);
  text-transform: uppercase;
  letter-spacing: 0.04em;
}

.plugin-status-enabled {
  background: rgba(0, 229, 255, 0.1);
  color: var(--accent-cyan);
  border: 1px solid rgba(0, 229, 255, 0.2);
}

.plugin-status-disabled {
  background: rgba(107, 114, 128, 0.1);
  color: var(--text-muted);
  border: 1px solid rgba(107, 114, 128, 0.15);
}

.plugin-status-error {
  background: rgba(255, 0, 110, 0.1);
  color: var(--accent-magenta);
  border: 1px solid rgba(255, 0, 110, 0.2);
}

.plugin-last-error {
  font-family: var(--font-mono);
  font-size: 10px;
  color: var(--accent-magenta);
  max-width: 200px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.plugin-toggle {
  flex-shrink: 0;
}

.toggle-btn {
  background: none;
  border: none;
  cursor: pointer;
  padding: 0;
}

.toggle-track {
  display: block;
  width: 28px;
  height: 16px;
  background: var(--bg-elevated);
  border-radius: 8px;
  position: relative;
  border: 1px solid var(--border-subtle);
  transition: background 0.2s, border-color 0.2s;
}

.toggle-btn.active .toggle-track {
  background: var(--accent-cyan);
  border-color: rgba(0, 229, 255, 0.3);
}

.toggle-thumb {
  display: block;
  width: 12px;
  height: 12px;
  background: var(--text-primary);
  border-radius: 50%;
  position: absolute;
  top: 1px;
  left: 1px;
  transition: transform 0.2s;
}

.toggle-btn.active .toggle-thumb {
  transform: translateX(12px);
}
</style>
