# ToyJS Module Loading

ToyJS supports ECMAScript Modules (ESM) using V8's native module capabilities. This document explains how modules are resolved, loaded, and cached in the runtime.

## Overview

Module loading in ToyJS involves a cooperation between V8 and a custom Rust-based loader. When V8 encounters an `import` statement, it triggers a callback in Rust to resolve and load the requested module.

## Core Components

### 1. `FsModuleLoader` (`src/modules/mod.rs`)

The `FsModuleLoader` is a global singleton responsible for:
*   **Caching**: Storing compiled `v8::Module` objects to ensure each module is only loaded and compiled once.
*   **Path Mapping**: Maintaining a mapping between V8 module identity hashes and their corresponding filesystem paths. This is crucial for resolving relative imports within a module.

### 2. `module_resolver` (`src/runtime.rs`)

This is the core callback provided to V8. It is invoked whenever a module needs to be resolved. The process follows these steps:

1.  **Identity Identification**: Retrieves the path of the *referring* module using its identity hash.
2.  **Path Resolution**: Combines the referrer's path with the import specifier to determine the absolute path of the requested module.
3.  **Cache Lookup**: Checks if the module at the resolved path has already been loaded.
4.  **Compilation**: If not cached, the file is read from the disk and compiled into a `v8::Module`.
5.  **Storage**: The new module is cached in the `FsModuleLoader`.

## Resolution Logic

The path resolution logic is implemented in `FsModuleLoader::resolve_path`:

*   **Absolute Imports**: If the specifier starts with `/`, it is treated as an absolute path.
*   **Relative Imports**: If the specifier is relative (e.g., `./utils.js` or `../math.js`), it is resolved relative to the directory of the referring module.
*   **Extension Handling**: If a file doesn't exist at the exact resolved path, the loader attempts to append `.js` to the path.
*   **Canonicalization**: All paths are canonicalized to ensure that different ways of referring to the same file (e.g., `test.js` vs `./test.js`) resolve to the same cache entry.

## Module Lifecycle

### 1. Compilation
When a module is first loaded, its source code is compiled using `v8::script_compiler::compile_module`. V8 validates the syntax and creates the module record.

### 2. Instantiation
After a module and all its dependencies are compiled, the `instantiate_module` method is called. This "links" the imports and exports between modules. ToyJS provides the `module_resolver` callback here to resolve dependencies.

### 3. Evaluation
Finally, `evaluate` is called on the top-level module. This executes the module's code and its dependencies in the correct order.

## Dynamic Imports

Currently, dynamic `import()` requests trigger the `host_import_module_dynamically_callback` in `src/runtime/bindings.rs`. In the current implementation, dynamic imports are not yet supported and will return `None`.

## Example Flow

1.  `runtime.execute_module("main.js")` is called.
2.  `main.js` contains `import { add } from './math.js'`.
3.  V8 calls `module_resolver` for specifier `./math.js` with `main.js` as the referrer.
4.  Loader resolves `./math.js` to `/absolute/path/to/math.js`.
5.  Loader compiles `math.js` and returns it to V8.
6.  V8 instantiates both modules.
7.  V8 evaluates `main.js`.
