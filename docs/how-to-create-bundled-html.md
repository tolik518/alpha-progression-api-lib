# Building index.bundle.html

## Prerequisites

```bash
cargo install wasm-pack
```

## Build

```bash
# 1. Compile WASM (no-modules target — required for file:// loading)
wasm-pack build alpha-progression-wasm --target no-modules

# 2. Bundle WASM + JS glue + app into a single self-contained HTML
python3 bundle.py
```

Open `index.bundle.html` directly in your browser — no server needed.

## Why no-modules?

`--target web` uses ES module `import` syntax which browsers block on `file://`.  
`--target no-modules` generates a plain IIFE (`wasm_bindgen` global) that works anywhere.  
The WASM binary is base64-encoded inside the HTML so there are no external file fetches.

## Re-bundling after code changes

Only step 1 is needed if you change Rust code. Only step 2 if you change `index.html`.