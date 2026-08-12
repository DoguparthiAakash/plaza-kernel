//! # plaza-kernel
//!
//! Linux kernel management for PlazaVM guests.
//!
//! Provides:
//! - **Config**: Kernel .config generation with PlazaVM defaults
//! - **Modules**: Kernel module dependency resolution and lifecycle
//! - **Initramfs**: Builder for generating minimal initramfs archives
//! - **Patch**: Kernel patch management for PlazaVM-specific fixes

pub mod config;
pub mod modules;
pub mod initramfs;
pub mod patch;

pub use config::{KernelConfig, KconfigValue};
pub use modules::{ModuleManager, KernelModule, ModuleState};
pub use initramfs::{InitramfsBuilder, InitramfsEntry, EntryType};
pub use patch::{PatchManager, KernelPatch, PatchCategory};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plaza_default_config() {
        let cfg = KernelConfig::plaza_default();
        assert!(cfg.option_count() > 20);
        assert_eq!(cfg.get("CONFIG_KVM"), Some(&KconfigValue::No));
        assert_eq!(cfg.get("CONFIG_VIRTIO"), Some(&KconfigValue::Yes));
        let kconfig = cfg.to_kconfig_string();
        assert!(kconfig.contains("CONFIG_VIRTIO_PCI=y"));
        assert!(kconfig.contains("# CONFIG_KVM is not set"));
    }

    #[test]
    fn test_module_dependency_resolution() {
        let mut mm = ModuleManager::new();
        mm.register("virtio", "/lib/modules/virtio.ko", vec![]);
        mm.register("virtio_net", "/lib/modules/virtio_net.ko", vec!["virtio".into()]);

        // Can't load virtio_net without virtio
        assert!(mm.load("virtio_net").is_err());

        mm.load("virtio").unwrap();
        mm.load("virtio_net").unwrap();

        assert_eq!(mm.list_loaded().len(), 2);

        // Can't unload virtio while virtio_net depends on it
        assert!(mm.unload("virtio").is_err());
        mm.unload("virtio_net").unwrap();
        mm.unload("virtio").unwrap();
    }

    #[test]
    fn test_minimal_initramfs() {
        let builder = InitramfsBuilder::plaza_minimal();
        assert!(builder.entry_count() > 10);
        assert!(builder.total_size() > 0);
        let manifest = builder.manifest();
        assert!(manifest.contains("/dev/console"));
        assert!(manifest.contains("/init"));
    }

    #[test]
    fn test_patch_lifecycle() {
        let mut pm = PatchManager::plaza_defaults();
        assert_eq!(pm.list_pending().len(), 2);
        assert_eq!(pm.list_applied().len(), 0);

        pm.apply("plaza-virtio-console-fix").unwrap();
        assert_eq!(pm.list_applied().len(), 1);
        assert_eq!(pm.list_pending().len(), 1);

        // Can't apply twice
        assert!(pm.apply("plaza-virtio-console-fix").is_err());

        pm.revert("plaza-virtio-console-fix").unwrap();
        assert_eq!(pm.list_pending().len(), 2);
    }
}
