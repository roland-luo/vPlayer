use std::collections::HashSet;

#[derive(Debug, Clone, Default)]
pub struct EmitReport {
    pub succeeded_plugins: Vec<String>,
    pub failed_plugins: Vec<String>,
}

#[derive(Debug, Default)]
pub struct PluginBus {
    plugins: Vec<String>,
    fail_on_emit: HashSet<String>,
}

impl PluginBus {
    pub fn register_plugin(&mut self, plugin_name: impl Into<String>) {
        self.plugins.push(plugin_name.into());
    }

    pub fn mark_plugin_fail_on_emit(&mut self, plugin_name: impl Into<String>) {
        self.fail_on_emit.insert(plugin_name.into());
    }

    pub fn emit(&self, event: &str) -> EmitReport {
        let mut report = EmitReport::default();

        for plugin in &self.plugins {
            if self.fail_on_emit.contains(plugin) {
                // Isolate single plugin failure: log and continue dispatching.
                eprintln!("[plugin:emit] plugin={plugin} event={event} failed and was isolated");
                report.failed_plugins.push(plugin.clone());
                continue;
            }
            report.succeeded_plugins.push(plugin.clone());
        }

        report
    }

    pub fn invoke(&self, command: &str) -> Result<String, String> {
        if self.plugins.is_empty() {
            return Ok(format!("invoke {command} skipped: no plugins registered"));
        }
        Ok(format!(
            "invoke {command} dispatched to {} plugins",
            self.plugins.len()
        ))
    }
}
