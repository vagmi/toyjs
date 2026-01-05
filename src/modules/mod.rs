use std::collections::HashMap;
use std::num::NonZero;
use std::path::Path;
use std::sync::{Mutex, OnceLock};
use v8;

static MODULE_LOADER: OnceLock<Mutex<FsModuleLoader>> = OnceLock::new();

pub struct FsModuleLoader {
    pub modules: HashMap<String, v8::Global<v8::Module>>,
    pub paths: HashMap<NonZero<i32>, String>,
}

// SAFETY: We only access FsModuleLoader from the V8 isolate thread
unsafe impl Send for FsModuleLoader {}

impl FsModuleLoader {
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
            paths: HashMap::new(),
        }
    }

    pub fn global() -> &'static Mutex<FsModuleLoader> {
        MODULE_LOADER.get_or_init(|| Mutex::new(FsModuleLoader::new()))
    }

    pub fn store_module(&mut self, path: String, module: v8::Global<v8::Module>, identity_hash: NonZero<i32>) {
        self.paths.insert(identity_hash, path.clone());
        self.modules.insert(path, module);
    }

    pub fn get_module(&self, path: &str) -> Option<&v8::Global<v8::Module>> {
        self.modules.get(path)
    }

    pub fn get_path_by_hash(&self, hash: NonZero<i32>) -> Option<&String> {
        self.paths.get(&hash)
    }

    pub fn resolve_path(base: &str, specifier: &str) -> Option<String> {
        // Simple resolution: if absolute, use it. If relative, join.
        let path = if specifier.starts_with('/') {
            std::path::PathBuf::from(specifier)
        } else {
            let base_path = Path::new(base);
            let base_dir = if base_path.is_dir() {
                base_path
            } else {
                base_path.parent().unwrap_or(Path::new("/"))
            };
            base_dir.join(specifier)
        };

        // For this toy, we just use the string path, assuming it exists or we'll fail to read later.
        // Canonicalize ensures we don't have duplicates for "./a.js" and "a.js"
        match path.canonicalize() {
            Ok(p) => p.to_str().map(|s| s.to_string()),
            Err(_) => {
                // If file doesn't exist, canonicalize fails.
                // For valid imports, it should exist.
                None
            }
        }
    }
}
