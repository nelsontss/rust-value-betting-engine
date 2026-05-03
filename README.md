# rust-value-betting-engine

Rust project bootstrapped with Cargo.

## Layout

```text
├── Cargo.toml (project manifest: package metadata, dependencies, features, and cargo settings)
├── src (all application source code)
│   ├── lib.rs (library entry point: expose modules and public API)
│   ├── main.rs (binary entry point: keep startup thin and call into lib.rs)
│   ├── application (application layer: orchestration of business flows)
│   │   ├── mod.rs (register application submodules)
│   │   └── services (application services: coordinate workflows and integrations)
│   │      └── mod.rs (register application service modules)
│   ├── domain (core business logic and rules)
│   │   ├── mod.rs (register domain submodules)
│   │   ├── entities (stateful business objects like fixtures, markets, selections)
│   │   │   └── mod.rs (register entity modules)
│   │   ├── services (pure domain rules that do not belong to one entity)
│   │   │   └── mod.rs (register domain service modules)
│   │   └── value_objects (small immutable business types like odds or probabilities)
│   │       └── mod.rs (register value object modules)
│   ├── infrastructure (adapters for config, storage, HTTP, feeds, and other externals)
│   │   ├── mod.rs (register infrastructure submodules)
│   │   ├── config (configuration loading and startup settings)
│   │   │   └── mod.rs (register config modules)
│   │   └── repositories (database, file, API, or bookmaker adapter implementations)
│   │       └── mod.rs (register repository modules)
│   └── shared (cross-cutting technical utilities shared across layers)
│       ├── error.rs (shared error and result types)
│       └── mod.rs (register shared modules)
└── tests (integration and behavior-level tests)
    └── smoke_test.rs (example integration test against the public API)
```

## Adding Code

When you add a new folder under an existing module, create a `mod.rs` file inside that folder and register it in the parent module.

Example:

```rust
// src/domain/mod.rs
pub mod entities;
pub mod services;
pub mod value_objects;
```

If you add `src/domain/markets/mod.rs`, update `src/domain/mod.rs` with `pub mod markets;`.

## Commands

```sh
cargo run
cargo test
cargo fmt
cargo clippy --all-targets --all-features -- -D warnings
```