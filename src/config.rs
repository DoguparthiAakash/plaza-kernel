use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use plaza_foundation::core::{PlazaResult, PlazaError};

/// Linux kernel configuration option.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum KconfigValue {
    Yes,
    No,
    Module,
    String(String),
    Int(i64),
}

impl std::fmt::Display for KconfigValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Yes => write!(f, "y"),
            Self::No => write!(f, "n"),
            Self::Module => write!(f, "m"),
            Self::String(s) => write!(f, "\"{}\"", s),
            Self::Int(n) => write!(f, "{}", n),
        }
    }
}

/// Kernel configuration manager for generating .config files.
pub struct KernelConfig {
    options: HashMap<String, KconfigValue>,
}

impl KernelConfig {
    pub fn new() -> Self {
        Self { options: HashMap::new() }
    }

    /// Start from a minimal PlazaVM-optimized config.
    pub fn plaza_default() -> Self {
        let mut cfg = Self::new();
        // Core x86_64 options
        cfg.set("CONFIG_64BIT", KconfigValue::Yes);
        cfg.set("CONFIG_SMP", KconfigValue::Yes);
        cfg.set("CONFIG_PREEMPT", KconfigValue::Yes);
        cfg.set("CONFIG_HIGH_RES_TIMERS", KconfigValue::Yes);
        cfg.set("CONFIG_NO_HZ_FULL", KconfigValue::Yes);
        // VirtIO for PlazaVM guests
        cfg.set("CONFIG_VIRTIO", KconfigValue::Yes);
        cfg.set("CONFIG_VIRTIO_PCI", KconfigValue::Yes);
        cfg.set("CONFIG_VIRTIO_BLK", KconfigValue::Yes);
        cfg.set("CONFIG_VIRTIO_NET", KconfigValue::Yes);
        cfg.set("CONFIG_VIRTIO_CONSOLE", KconfigValue::Yes);
        cfg.set("CONFIG_VIRTIO_BALLOON", KconfigValue::Module);
        // Filesystem
        cfg.set("CONFIG_EXT4_FS", KconfigValue::Yes);
        cfg.set("CONFIG_TMPFS", KconfigValue::Yes);
        cfg.set("CONFIG_PROC_FS", KconfigValue::Yes);
        cfg.set("CONFIG_SYSFS", KconfigValue::Yes);
        cfg.set("CONFIG_DEVTMPFS", KconfigValue::Yes);
        cfg.set("CONFIG_DEVTMPFS_MOUNT", KconfigValue::Yes);
        // Networking
        cfg.set("CONFIG_NET", KconfigValue::Yes);
        cfg.set("CONFIG_INET", KconfigValue::Yes);
        cfg.set("CONFIG_IPV6", KconfigValue::Module);
        // Security
        cfg.set("CONFIG_SECCOMP", KconfigValue::Yes);
        cfg.set("CONFIG_NAMESPACES", KconfigValue::Yes);
        cfg.set("CONFIG_CGROUPS", KconfigValue::Yes);
        // Disable hardware KVM (PlazaVM constraint)
        cfg.set("CONFIG_KVM", KconfigValue::No);
        cfg.set("CONFIG_KVM_INTEL", KconfigValue::No);
        cfg.set("CONFIG_KVM_AMD", KconfigValue::No);
        cfg
    }

    pub fn set(&mut self, key: &str, value: KconfigValue) {
        self.options.insert(key.to_string(), value);
    }

    pub fn get(&self, key: &str) -> Option<&KconfigValue> {
        self.options.get(key)
    }

    pub fn remove(&mut self, key: &str) {
        self.options.remove(key);
    }

    /// Generate .config file content.
    pub fn to_kconfig_string(&self) -> String {
        let mut lines: Vec<String> = Vec::new();
        lines.push("# PlazaVM Kernel Configuration".into());
        lines.push(format!("# Generated: {}", chrono::Utc::now().to_rfc3339()));
        lines.push(String::new());

        let mut sorted: Vec<_> = self.options.iter().collect();
        sorted.sort_by_key(|(k, _)| k.clone());

        for (key, value) in sorted {
            match value {
                KconfigValue::No => lines.push(format!("# {} is not set", key)),
                _ => lines.push(format!("{}={}", key, value)),
            }
        }
        lines.join("\n")
    }

    /// Count of configured options.
    pub fn option_count(&self) -> usize {
        self.options.len()
    }

    /// Merge another config on top (override conflicts).
    pub fn merge(&mut self, other: &KernelConfig) {
        for (k, v) in &other.options {
            self.options.insert(k.clone(), v.clone());
        }
    }
}
