# nexus-cog-core

Foundation library for the [Nexus Cog](https://github.com/nexus-cognitive-tech) cognitive stack — shared types, storage abstractions, and runtime configuration.

## Scope

- **Types** — serde-friendly data shapes used across every Nexus Cog engine (memory, graph, causal, intent, etc.). No logic, only shapes.
- **Storage** — atomic write primitives, batch operations, lock abstractions, JSON store, migrations.
- **Configuration** — runtime paths, settings, error types, environment.

This crate is the foundation that every engine and interface builds on. Engines import only the type modules they need:

```rust
use nexus_cog_core::palace::MemoryPalace;
use nexus_cog_core::store::JsonStore;
use nexus_cog_core::config::NexusConfig;
```

## License

Apache-2.0.
