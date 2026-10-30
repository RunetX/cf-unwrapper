# cf-unwrapper library

Standalone 1C container and XML format engine. Rust stable, edition 2021; only external runtime dependency: flate2.

```rust
use cf_unwrapper::{meta::Config, xml};
let bytes = xml::imp::new_external("SyntheticProcessor").unwrap();
let config = Config::open(&bytes).unwrap();
assert_eq!(config.root.header.name, "SyntheticProcessor");
```

Public format modules include container, brace, meta, unpack, xml and the supporting query parser. Form DTOs live in `xml::form`. Builtin downgrade tables are bundled under `formats/`. Unsupported inputs are reported explicitly. No installed platform, GUI, debugger or BSL server is required. See the repository README and `docs/extraction-audit.md` for CLI examples, coverage limits and the provenance of platform-derived default assets. MIT follows the source metadata declaration; technical asset review does not establish third-party redistribution rights.
