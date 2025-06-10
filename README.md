
# Graph Browser

**Graph** is the canonical browser for the [Bitgrid](https://bitgrid.org) platform. It is
a reference implementation designed for embedded systems and constrained environments,
supporting deterministic content execution and minimal runtime dependencies.

---

## Crates

This repository is a Cargo workspace that includes:

* **browser:** The application of the **graph-engine** crate to produce the Graph Browser.
* **graph-common:** Functionality shared between Graph's various crates.
    * **imgui module:** A wrapper module through which the rest of the workspace may access the
    `imgui` crate. This wrapper module also adds in the `imgui_sdl3_support` as `sdl3_support`
    and `renderers` module.
        * **renderers:** The module through which the rest of the workspace may access various
        renderer backends for the `imgui` crate.
            * **glow:** A wrapper module through which the rest of the workspace may access the
            `imgui_glow_renderer` crate, with `imgui_glow_renderer::glow` aliased as
            `common::renderer::imgui::renderers::glow::inner`.
    * **sdl3 module:** an alias through which the rest of the workspace may access the `sdl3` crate.
* **graph-engine:** The combination of the Graph Workspace's various crates into a single,
    unified engine.

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
    - [X] SDL3 + DearImGui Scaffolding
        - [X] Code
        - [X] Documentation
    - [ ] Graph's Chrome
        ([What is a browser's chrome?](https://developer.mozilla.org/en-US/docs/Glossary/Chrome))
    - [ ] Page Rendering
        - [ ] **B**it**g**rid **M**arkup **L**anguage (BGML)
            - [ ] Grammar Spec
            - [ ] Parser
            - [ ] Tag Spec
                - [ ] Default Rendering (Style) Attributes
            - [ ] Renderer
        - [ ] CSS
            - [ ] Rendering Model Specs
            - [X] Parser
                - Crest
            - [ ] Attribute Spec
                - [ ] How each CSS attribute affects each BGML tag
            - [ ] Modify engine to apply css before rendering
    - [ ] Runtime
        - [ ] Grid API for Lua Runtime
        - [ ] Teal API for Lua Runtime
        - [ ] Grid API for JS Runtime
        - [ ] Typescript API for JS Runtime
        - [ ] Teal-Types to WASM Compiler
        - [ ] Typescript-Types to WASM Compiler
        - [ ] Teal-Any to WASM Compiler
        - [ ] Typescript-Any to WASM Compiler

---

## Build

```sh
cargo build --release
```

To include support for the `www` feature:

```sh
cargo build --release --features www
```

