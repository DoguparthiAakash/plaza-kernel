use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use plaza_foundation::core::{PlazaResult, PlazaError};

/// Represents a loadable kernel module.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KernelModule {
    pub name: String,
    pub path: String,
    pub state: ModuleState,
    pub size_bytes: u64,
    pub dependencies: Vec<String>,
    pub parameters: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModuleState {
    Unloaded,
    Loading,
    Loaded,
    Unloading,
    Failed,
}

/// Kernel module manager for tracking loaded/available modules.
pub struct ModuleManager {
    modules: HashMap<String, KernelModule>,
    load_order: Vec<String>,
}

impl ModuleManager {
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
            load_order: Vec::new(),
        }
    }

    /// Register a module as available (not yet loaded).
    pub fn register(&mut self, name: &str, path: &str, deps: Vec<String>) {
        self.modules.insert(name.to_string(), KernelModule {
            name: name.to_string(),
            path: path.to_string(),
            state: ModuleState::Unloaded,
            size_bytes: 0,
            dependencies: deps,
            parameters: HashMap::new(),
        });
    }

    /// Simulate loading a module (resolves dependencies first).
    pub fn load(&mut self, name: &str) -> PlazaResult<()> {
        // Check existence
        if !self.modules.contains_key(name) {
            return Err(PlazaError::NotFound(format!("Module {}", name)));
        }

        // Check dependencies
        let deps = self.modules[name].dependencies.clone();
        for dep in &deps {
            if let Some(m) = self.modules.get(dep) {
                if m.state != ModuleState::Loaded {
                    return Err(PlazaError::Internal(format!(
                        "Dependency '{}' not loaded for module '{}'", dep, name
                    )));
                }
            } else {
                return Err(PlazaError::NotFound(format!("Dependency module {}", dep)));
            }
        }

        let module = self.modules.get_mut(name).unwrap();
        module.state = ModuleState::Loaded;
        self.load_order.push(name.to_string());
        Ok(())
    }

    /// Unload a module (checks for dependents first).
    pub fn unload(&mut self, name: &str) -> PlazaResult<()> {
        // Check if anything depends on this module
        for (other_name, other_mod) in &self.modules {
            if other_mod.state == ModuleState::Loaded && other_mod.dependencies.contains(&name.to_string()) {
                return Err(PlazaError::Internal(format!(
                    "Cannot unload '{}': '{}' depends on it", name, other_name
                )));
            }
        }

        let module = self.modules.get_mut(name)
            .ok_or_else(|| PlazaError::NotFound(format!("Module {}", name)))?;
        module.state = ModuleState::Unloaded;
        self.load_order.retain(|n| n != name);
        Ok(())
    }

    /// Set module parameters before loading.
    pub fn set_parameter(&mut self, module: &str, key: &str, value: &str) -> PlazaResult<()> {
        let m = self.modules.get_mut(module)
            .ok_or_else(|| PlazaError::NotFound(format!("Module {}", module)))?;
        m.parameters.insert(key.to_string(), value.to_string());
        Ok(())
    }

    pub fn list_loaded(&self) -> Vec<&KernelModule> {
        self.modules.values().filter(|m| m.state == ModuleState::Loaded).collect()
    }

    pub fn list_all(&self) -> Vec<&KernelModule> {
        self.modules.values().collect()
    }

    pub fn get(&self, name: &str) -> Option<&KernelModule> {
        self.modules.get(name)
    }
}
