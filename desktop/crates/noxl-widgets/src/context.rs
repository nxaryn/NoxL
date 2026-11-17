//! Widget context: the sandboxed data provider for widgets.
//!
//! Widgets receive a `WidgetContext` and can only read the data it exposes.
//! This is the security boundary: widgets cannot touch the filesystem, spawn
//! processes, or access the network directly.

use std::time::Duration;

/// Data available to widgets.
pub struct WidgetContext {
    system: sysinfo::System,
}

impl WidgetContext {
    pub fn new() -> Self {
        let mut system = sysinfo::System::new_all();
        system.refresh_all();
        Self { system }
    }

    /// Refresh system data.
    pub fn refresh(&mut self) {
        self.system.refresh_all();
    }

    /// CPU usage percentage (0.0 - 100.0).
    pub fn cpu_usage(&self) -> f32 {
        self.system.global_cpu_info().cpu_usage()
    }

    /// Per-CPU usage.
    pub fn per_cpu_usage(&self) -> Vec<f32> {
        self.system.cpus().iter().map(|c| c.cpu_usage()).collect()
    }

    /// Used RAM in bytes.
    pub fn used_memory(&self) -> u64 {
        self.system.used_memory()
    }

    /// Total RAM in bytes.
    pub fn total_memory(&self) -> u64 {
        self.system.total_memory()
    }

    /// System uptime in seconds.
    pub fn uptime(&self) -> u64 {
        sysinfo::System::uptime()
    }

    /// CPU brand string.
    pub fn cpu_brand(&self) -> &str {
        self.system.cpus().first()
            .map(|c| c.brand().trim())
            .unwrap_or("Unknown CPU")
    }

    /// Number of CPUs.
    pub fn cpu_count(&self) -> usize {
        self.system.cpus().len()
    }
}

impl Default for WidgetContext {
    fn default() -> Self {
        Self::new()
    }
}
