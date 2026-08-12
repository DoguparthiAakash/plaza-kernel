use serde::{Serialize, Deserialize};
use plaza_foundation::core::{PlazaResult, PlazaError};

/// A kernel patch to apply before building.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KernelPatch {
    pub name: String,
    pub description: String,
    pub category: PatchCategory,
    pub unified_diff: String,
    pub applied: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PatchCategory {
    Security,
    Performance,
    BugFix,
    Feature,
    PlazaVm,
}

/// Manages kernel patches for PlazaVM-specific customization.
pub struct PatchManager {
    patches: Vec<KernelPatch>,
}

impl PatchManager {
    pub fn new() -> Self {
        Self { patches: Vec::new() }
    }

    /// Load default PlazaVM kernel patches.
    pub fn plaza_defaults() -> Self {
        let mut pm = Self::new();
        pm.add(KernelPatch {
            name: "plaza-virtio-console-fix".into(),
            description: "Fix virtio-console buffer overflow in TCG mode".into(),
            category: PatchCategory::BugFix,
            unified_diff: "--- a/drivers/char/virtio_console.c\n+++ b/drivers/char/virtio_console.c\n@@ -1,3 +1,4 @@\n+// PlazaVM: patched buffer handling\n".into(),
            applied: false,
        });
        pm.add(KernelPatch {
            name: "plaza-boot-speedup".into(),
            description: "Disable unnecessary hardware probing for faster VM boot".into(),
            category: PatchCategory::Performance,
            unified_diff: "--- a/init/main.c\n+++ b/init/main.c\n@@ -1,3 +1,4 @@\n+// PlazaVM: skip PCI bus scan\n".into(),
            applied: false,
        });
        pm
    }

    pub fn add(&mut self, patch: KernelPatch) {
        self.patches.push(patch);
    }

    pub fn apply(&mut self, name: &str) -> PlazaResult<()> {
        let patch = self.patches.iter_mut().find(|p| p.name == name)
            .ok_or_else(|| PlazaError::NotFound(format!("Patch {}", name)))?;
        if patch.applied {
            return Err(PlazaError::Internal(format!("Patch '{}' already applied", name)));
        }
        patch.applied = true;
        Ok(())
    }

    pub fn revert(&mut self, name: &str) -> PlazaResult<()> {
        let patch = self.patches.iter_mut().find(|p| p.name == name)
            .ok_or_else(|| PlazaError::NotFound(format!("Patch {}", name)))?;
        patch.applied = false;
        Ok(())
    }

    pub fn list_pending(&self) -> Vec<&KernelPatch> {
        self.patches.iter().filter(|p| !p.applied).collect()
    }

    pub fn list_applied(&self) -> Vec<&KernelPatch> {
        self.patches.iter().filter(|p| p.applied).collect()
    }
}
