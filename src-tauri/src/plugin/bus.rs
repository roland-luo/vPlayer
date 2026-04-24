use std::panic;
use std::panic::AssertUnwindSafe;

use super::{PluginEvent, PluginInstance, PluginState};

#[derive(Debug, Clone, Default)]
pub struct EmitReport {
    pub succeeded_plugins: Vec<String>,
    pub failed_plugins: Vec<String>,
}

#[derive(Default)]
pub struct PluginBus {
    plugins: Vec<PluginInstance>,
}

impl PluginBus {
    /// Register a loaded plugin instance.
    pub fn register_plugin(&mut self, instance: PluginInstance) {
        if let Some(existing) = self
            .plugins
            .iter_mut()
            .find(|p| p.name() == instance.name())
        {
            *existing = instance;
            return;
        }
        self.plugins.push(instance);
    }

    /// Unregister a plugin by name.
    pub fn unregister_plugin(&mut self, name: &str) {
        self.plugins.retain(|p| p.name() != name);
    }

    /// Get a mutable reference to a plugin instance.
    pub fn get_mut(&mut self, name: &str) -> Option<&mut PluginInstance> {
        self.plugins.iter_mut().find(|p| p.name() == name)
    }

    /// Get all plugin names.
    pub fn plugin_names(&self) -> Vec<String> {
        self.plugins.iter().map(|p| p.name().to_string()).collect()
    }

    /// Dispatch an event to all enabled plugins with catch_unwind isolation.
    ///
    /// Each plugin's `on_event` is wrapped in `catch_unwind` so that a single
    /// plugin panic does not crash the player or affect other plugins.
    pub fn emit(&mut self, event: &PluginEvent) -> EmitReport {
        let mut report = EmitReport::default();

        // We need to split borrows — collect names first to avoid borrow issues.
        let target_names: Vec<String> = self
            .plugins
            .iter()
            .filter(|p| p.is_enabled())
            .map(|p| p.name().to_string())
            .collect();

        for name in &target_names {
            let plugin = match self.plugins.iter_mut().find(|p| p.name() == name) {
                Some(p) => p,
                None => continue,
            };

            let result = {
                let mut plugin_ref = AssertUnwindSafe(&mut *plugin);
                panic::catch_unwind(panic::AssertUnwindSafe(|| {
                    plugin_ref.plugin.on_event(event)
                }))
            };

            match result {
                Ok(Ok(())) => {
                    report.succeeded_plugins.push(name.clone());
                }
                Ok(Err(e)) => {
                    let err_msg = e.to_string();
                    eprintln!(
                        "[plugin:emit] plugin={name} event={} error={err_msg}",
                        event.name()
                    );
                    plugin.error_count = plugin.error_count.saturating_add(1);
                    plugin.last_error = Some(err_msg.clone());
                    plugin.state = PluginState::Crashed;
                    report.failed_plugins.push(name.clone());
                }
                Err(panic_payload) => {
                    let panic_msg = if let Some(s) = panic_payload.downcast_ref::<&str>() {
                        s.to_string()
                    } else if let Some(s) = panic_payload.downcast_ref::<String>() {
                        s.clone()
                    } else {
                        "unknown panic".to_string()
                    };
                    eprintln!(
                        "[plugin:emit] plugin={name} event={} PANICKED: {panic_msg}",
                        event.name()
                    );
                    plugin.error_count = plugin.error_count.saturating_add(1);
                    plugin.last_error = Some(format!("panic: {panic_msg}"));
                    plugin.state = PluginState::Crashed;
                    report.failed_plugins.push(name.clone());
                }
            }
        }

        report
    }

    /// Invoke a command across all plugins.
    ///
    /// Currently dispatches as an event. In Phase 2+ this may support
    /// request/response patterns.
    pub fn invoke(&mut self, command: &str) -> Result<String, String> {
        if self.plugins.is_empty() {
            return Ok(format!("invoke {command} skipped: no plugins registered"));
        }

        let event = PluginEvent::named(command);
        let report = self.emit(&event);

        Ok(format!(
            "invoke {command}: {} succeeded, {} failed",
            report.succeeded_plugins.len(),
            report.failed_plugins.len()
        ))
    }
}
