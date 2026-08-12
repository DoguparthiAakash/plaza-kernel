use std::path::{Path, PathBuf};
use serde::{Serialize, Deserialize};
use plaza_foundation::core::{PlazaResult, PlazaError};

/// An entry to be included in the initramfs archive.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitramfsEntry {
    pub path: String,
    pub entry_type: EntryType,
    pub mode: u32,
    pub content: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EntryType {
    Directory,
    File,
    Symlink(String),
    CharDevice { major: u32, minor: u32 },
    BlockDevice { major: u32, minor: u32 },
}

/// Builder for generating initramfs (cpio) archives.
pub struct InitramfsBuilder {
    entries: Vec<InitramfsEntry>,
}

impl InitramfsBuilder {
    pub fn new() -> Self {
        Self { entries: Vec::new() }
    }

    /// Create a PlazaVM-optimized minimal initramfs.
    pub fn plaza_minimal() -> Self {
        let mut builder = Self::new();

        // Root directories
        for dir in &["/", "/bin", "/dev", "/etc", "/lib", "/proc", "/sys", "/tmp", "/run", "/var", "/sbin"] {
            builder.add_directory(dir, 0o755);
        }

        // Essential device nodes
        builder.add_entry(InitramfsEntry {
            path: "/dev/console".into(),
            entry_type: EntryType::CharDevice { major: 5, minor: 1 },
            mode: 0o600,
            content: None,
        });
        builder.add_entry(InitramfsEntry {
            path: "/dev/null".into(),
            entry_type: EntryType::CharDevice { major: 1, minor: 3 },
            mode: 0o666,
            content: None,
        });
        builder.add_entry(InitramfsEntry {
            path: "/dev/zero".into(),
            entry_type: EntryType::CharDevice { major: 1, minor: 5 },
            mode: 0o666,
            content: None,
        });

        // Minimal /init script
        let init_script = b"#!/bin/sh\nmount -t proc proc /proc\nmount -t sysfs sysfs /sys\nmount -t devtmpfs devtmpfs /dev\nexec /sbin/init\n";
        builder.add_file("/init", init_script, 0o755);

        builder
    }

    pub fn add_directory(&mut self, path: &str, mode: u32) {
        self.entries.push(InitramfsEntry {
            path: path.to_string(),
            entry_type: EntryType::Directory,
            mode,
            content: None,
        });
    }

    pub fn add_file(&mut self, path: &str, content: &[u8], mode: u32) {
        self.entries.push(InitramfsEntry {
            path: path.to_string(),
            entry_type: EntryType::File,
            mode,
            content: Some(content.to_vec()),
        });
    }

    pub fn add_symlink(&mut self, path: &str, target: &str) {
        self.entries.push(InitramfsEntry {
            path: path.to_string(),
            entry_type: EntryType::Symlink(target.to_string()),
            mode: 0o777,
            content: None,
        });
    }

    pub fn add_entry(&mut self, entry: InitramfsEntry) {
        self.entries.push(entry);
    }

    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    pub fn total_size(&self) -> usize {
        self.entries.iter()
            .filter_map(|e| e.content.as_ref())
            .map(|c| c.len())
            .sum()
    }

    /// Generate a manifest listing all entries (for debugging).
    pub fn manifest(&self) -> String {
        self.entries.iter()
            .map(|e| format!("{:o} {:?} {}", e.mode, e.entry_type, e.path))
            .collect::<Vec<_>>()
            .join("\n")
    }
}
