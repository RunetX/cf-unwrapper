# CLI and library reference

See the [project overview](../README.md) and [contribution guide](../CONTRIBUTING.md). Run the shell commands below from the repository root; CLI examples assume the executable is on `PATH`.

Standalone Rust engine for 1C container files and XML exports. The workspace has a format library (`cf-unwrapper`, imported as `cf_unwrapper`) and a file CLI (`cf-unwrapper-cli`, binary `cf-unwrapper`). It requires Rust stable and no installed 1C platform. The library's only external runtime dependency is flate2; the CLI directly depends only on that library.

```sh
cargo build --workspace --locked
cargo run -p cf-unwrapper-cli -- --help
python tools/check.py
```

```sh
cf-unwrapper info input.cf
cf-unwrapper tree input.epf --all
cf-unwrapper ls input.cfe
cf-unwrapper cat input.cf root
cf-unwrapper module input.cf ENTRY
cf-unwrapper search input.cf text
cf-unwrapper verify input.cf
cf-unwrapper unpack input.cf unpacked
cf-unwrapper pack unpacked rebuilt.cf
cf-unwrapper export input.cf xml
cf-unwrapper export input.cf xml-old --format 2.20
cf-unwrapper export input.cf xml-older --format 2.17
cf-unwrapper export-forms input.cf forms
cf-unwrapper import xml rebuilt.cf
cf-unwrapper new-epf SyntheticProcessor fresh.epf
cf-unwrapper export fresh.epf external
cf-unwrapper import external/SyntheticProcessor.xml rebuilt.epf
cf-unwrapper unpack rebuilt.epf external-unpacked
cf-unwrapper xml-down xml xml-old --format 2.17
cf-unwrapper xml-diff xml reference 3
cf-unwrapper import-check external/SyntheticProcessor.xml external-unpacked - 3
```

`import-check <xml> <records-dir> [selection|-] [N]` compares imported XML with the directory containing the unpacked metadata records themselves. A flat `.epf` normally uses the unpack root, as in the external-processing example above. A multipart `.cf` commonly uses `unpacked/part1`, but the part number is not universal: inspect the root and per-part `.cfkit` manifests and `cf-unwrapper ls input.cf` to identify the metadata part. Pass that part directory, not the multipart unpack root; for a confirmed `part1` layout, use `cf-unwrapper import-check xml unpacked/part1 - 3`.

The `differ`, `extra` and `refused` counts are comparison results; inspect them even when the command exits successfully. Import can normalize records, so comparing original container records against imported XML can report differences (including a platform generator version change). This alone does not prove XML roundtrip loss. To check the normalized representation, import the XML, export the rebuilt container, and compare those XML trees with `xml-diff`; unpack the rebuilt container for a matching `import-check` baseline.

`export` and `export-forms` accept an optional UTF-8 selection file (one object path per line). `import-check` accepts a selection file or `-` for all objects. Native XML is 2.21; `export --format 2.21` explicitly requests it. Downgrades support embedded 2.20 and chained 2.17 tables. Files under `formats/<version>.txt` beside the executable override the embedded tables; the packaged tables live in `crates/cf-unwrapper/formats/`. Invalid arguments, unreadable files, unsupported formats, and refusals from `import` return nonzero with stderr diagnostics. `import` refuses to replace or create its result when any input file is refused.

```rust
use cf_unwrapper::{meta::Config, xml};

fn main() -> Result<(), String> {
    let bytes = xml::imp::new_external("SyntheticProcessor")?;
    let config = Config::open(&bytes)?;
    let output = std::path::Path::new("xml");
    let report = xml::export(&config, output)?;
    if !report.errors.is_empty() { return Err(format!("{:?}", report.errors)); }
    let input = output.join("SyntheticProcessor.xml");
    let imported = xml::imp::import(&input, None)?;
    if !imported.refused.is_empty() { return Err(format!("{:?}", imported.refused)); }
    let rebuilt = xml::imp::cf(&input, &imported)?;
    let _reopened = Config::open(&rebuilt)?;
    Ok(())
}
```

Container APIs preserve entries, timestamps and W32/W64 multipart layout. Unpack manifests retain the existing `.cfkit` format. A single W32 stream unpacks flat; a single W64 stream uses `part0/` and `parts\t1\tW64` in the root `.cfkit`, so repacking keeps its width. The manifest and format diagnostics retain legacy names for compatibility.

Coverage includes managed forms, MXL, flowcharts, command interfaces, registers, query parsing needed by dynamic-list import, and the remaining extracted XML serializers. This is a reverse engineered implementation with explicit unsupported cases, not a guarantee for every platform version or every configuration. Synthetic checks cover only the stated behaviors; no proprietary corpus is distributed or silently skipped in CI. `xml-diff` and `import-check` are comparison reports; their exit status signals an execution error, not the presence of differences. Export/downconversion may leave successfully written files when another file is refused.

No platform runner, GUI, debugger, BSL server or constructor integration is included. Static captions work without platform resources. Advanced library editing primitives remain because they belong to the XML representation; there are no model editing CLI commands.

For a local corpus check, choose your own paths under ignored `local/`: run `unpack`, `export`, `import`, `verify` and `xml-diff` with those explicit paths. Compare the result with a trusted platform export locally. No paths, corpus files, dumps or reports should be committed. Such a run is optional and is separate from the deterministic CI tests.

Opt-in legacy diagnostics use explicit caller-selected paths: `CFKIT_IMPORT_DUMP`, `CFKIT_IMPORT_REFUSED`, `CFKIT_DP_DUMP`, `CFKIT_QP_DUMP`, and CLI `CFKIT_REFUSED`. `CFKIT_V59_TAILS` changes form diagnostics. Keep them unset for normal operation and checks; their output may contain your input data.

See [extraction audit](extraction-audit.md) for source selection, sanitized test data, embedded asset provenance and limitations. Source metadata declares MIT. Platform-derived default assets and the fixed CF seed are identified separately in that audit; a technical content review does not establish redistribution rights.
