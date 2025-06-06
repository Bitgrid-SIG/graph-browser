
# Graph Browser

**Graph** is the canonical browser for the [Bitgrid](https://bitgrid.org) platform. It is
a reference implementation designed for embedded systems and constrained environments,
supporting deterministic content execution and minimal runtime dependencies.

---

## Crates

This repository is a Cargo workspace that includes:

* **graph-browser:** The crate that produces Graph's executable.
    * **graph-runtime:** WASM runtime that executes general wasm and runs the compiled Teal code.
        * **tl2wasm:** the Teal-to-WASM compiler used by Graph. Compiles Lua as Teal with `any`
            as the implicit type for all values (with steep runtime costs).
        * **graph-www:** (optional) feature-gated support for the World Wide Web, including a
            TypeScript by compiling.
            * **ts2wasm:** the TypeScript-to-WASM Compiler used by Graph. Like tl2wasm,
            JavaScript is compiled as Typescript with `any` as the implicit type for all
            values (with steep runtime costs).

---

## Features

* **Grid-first execution model** with no dependence on the traditional web-browser tech-stack.
* **Teal-native:** Executes Teal source compiled directly to WASM using an in-house toolchain.
* **Modular architecture:** Feature gates control inclusion of www support and JS execution.
    - `www`: enable support for the world-wide web, including a javascript engine, svg support,
        and more.
* **Embedded-oriented:** Optimized for small Linux systems and sandboxed runtimes.
    - No JIT used, even for `www` support

---

# Progress

- [ ] Browser
    - [ ] SDL3 Scaffolding
    - [ ] Grid HTML Dialect
    - [ ] Grid CSS Dialect (subset of www's dialect)
    - [ ] Runtime
        - [ ] `tl2wasm`
            - [ ] Teal Language Definition
            - [ ] Grammar + Syntax Validation
            - [ ] WASM Support for Runtime Behaviour
            - [ ] AST to WASM transformer
        - [ ] `www` feature
            - [ ] SVG
            - [ ] WWW HTML Dialect
            - [ ] WWW CSS Dialect
            - [ ] `ts2wasm`
                - [ ] Typescript Language Definition
                - [ ] Grammar + Syntax Validation
                - [ ] WASM Support for Runtime Behaviour
                - [ ] AST to WASM transformer
                


---

## Build

```sh
cargo build --release
```

To include support for the `www` feature:

```sh
cargo build --release --features www
```

