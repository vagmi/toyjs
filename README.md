# ToyJS

ToyJS is a minimal JavaScript runtime built with Rust and V8. It serves as an educational project to understand how JavaScript runtimes (like Node.js or Deno) work under the hood, featuring a custom event loop, native bindings, and ES module support.

## Overview

- **V8 Engine**: Powered by the V8 JavaScript engine.
- **ES Modules**: Support for `import` and `export` syntax.
- **Native Bindings**:
  - `print(msg)`: Print to stdout.
  - `add(a, b)`: Simple synchronous addition.
- **Async Support**:
  - `setTimeout` / `setInterval`: Timer operations.
  - `fetch`: Basic HTTP requests (returns a Promise).
- **Event Loop**: Custom implementation using `tokio` to handle asynchronous tasks.


## Test it

```sh
$ cargo run exec -- js/index.js
```
