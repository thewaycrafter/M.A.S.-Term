//! WASM Plugin Runtime

use anyhow::{Context, Result};
use std::sync::{Arc, Mutex};
use wasmtime::*;

/// WASM Plugin host interface
pub struct WasmPlugin {
    engine: Engine,
    module: Module,
    store: Arc<Mutex<Store<()>>>,
}

impl WasmPlugin {
    /// Load a WASM plugin from file
    pub fn load(path: &std::path::Path) -> Result<Self> {
        let engine = Engine::default();
        let module = Module::from_file(&engine, path)?;
        let store = Arc::new(Mutex::new(Store::new(&engine, ())));
        
        Ok(Self {
            engine,
            module,
            store,
        })
    }

    /// Execute a function in the plugin
    pub fn call(&self, func_name: &str, _args: &[&str]) -> Result<String> {
        let mut store = self.store.lock().unwrap();
        let linker = Linker::new(&self.engine);
        let instance = linker.instantiate(&mut *store, &self.module)?;
        
        // Simple interface: plugins export a function that returns an offset/length ptr
        // Real implementation would use WASI or a more complex bindgen
        // For now, we'll try to call a simple function returning an integer status
        
        let func = instance.get_typed_func::<(), i32>(&mut *store, func_name)
            .context("Function not found or signature mismatch")?;
            
        let result = func.call(&mut *store, ())?;
        
        Ok(result.to_string())
    }
}

/// Helper to scan for plugins
pub fn scan_plugins(dir: &std::path::Path) -> Result<Vec<std::path::PathBuf>> {
    let mut plugins = Vec::new();
    if dir.exists() {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "wasm") {
                plugins.push(path);
            }
        }
    }
    Ok(plugins)
}
