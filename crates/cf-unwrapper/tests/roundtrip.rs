use cf_unwrapper::{
    container::{self, Entry, Width},
    meta::Config,
    unpack, xml,
};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
static NEXT: AtomicUsize = AtomicUsize::new(0);

#[test]
fn adopted_extension_roundtrip_and_single_part_container() {
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/extension");
    let imported = xml::imp::import(&source, None).unwrap();
    assert!(imported.refused.is_empty(), "{:?}", imported.refused);
    let bytes = xml::imp::cf(&source, &imported).unwrap();
    let parts = container::read_all(&bytes).unwrap();
    assert_eq!(parts.len(), 1);
    assert_eq!(parts[0].width, Width::W32);
    assert!(parts[0].entries.iter().any(|e| e.name == "configinfo"));
    assert!(!parts[0]
        .entries
        .iter()
        .any(|e| e.name == "root" || e.name == "version"));
    let cfg = Config::open(&bytes).unwrap();
    assert!(xml::extension::is_extension(&cfg));
    let temp = Temp::new();
    let report = xml::export(&cfg, &temp.0).unwrap();
    assert!(report.errors.is_empty(), "{:?}", report.errors);
    assert!(report.skipped.is_empty(), "{:?}", report.skipped);
    fn same(source: &Path, exported: &Path) {
        for entry in std::fs::read_dir(source).unwrap() {
            let path = entry.unwrap().path();
            let other = exported.join(path.file_name().unwrap());
            if path.is_dir() {
                same(&path, &other);
            } else {
                assert_eq!(
                    std::fs::read_to_string(&path).unwrap(),
                    std::fs::read_to_string(&other).unwrap(),
                    "{}",
                    path.display()
                );
            }
        }
    }
    same(&source, &temp.0);
}
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!(
            "cf-unwrapper-roundtrip-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&p).unwrap();
        Self(p)
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn container_parts_and_extensions() {
    let t = Temp::new();
    let inner = container::write(&[Entry {
        name: "payload".into(),
        created: 3,
        modified: 4,
        data: b"synthetic".to_vec(),
    }]);
    let entries = vec![
        Entry {
            name: "nested".into(),
            created: 5,
            modified: 6,
            data: container::deflate(&inner),
        },
        Entry {
            name: "text".into(),
            created: 7,
            modified: 8,
            data: container::deflate(b"test data"),
        },
    ];
    for ext in ["cf", "cfe", "epf"] {
        let bytes = [
            container::write_as(&entries, Width::W32),
            container::write_as(&entries, Width::W64),
        ]
        .concat();
        let path = t.0.join(format!("input.{ext}"));
        std::fs::write(&path, &bytes).unwrap();
        let dir = t.0.join(ext);
        unpack::unpack_file(&std::fs::read(&path).unwrap(), &dir).unwrap();
        let rebuilt = container::read_all(&unpack::pack_dir(&dir).unwrap()).unwrap();
        assert_eq!(rebuilt.len(), 2);
        assert_eq!(rebuilt[0].width, Width::W32);
        assert_eq!(rebuilt[1].width, Width::W64);
        for part in rebuilt {
            for (got, want) in part.entries.iter().zip(&entries) {
                assert_eq!(
                    (
                        &got.name,
                        got.created,
                        got.modified,
                        container::content(got)
                    ),
                    (
                        &want.name,
                        want.created,
                        want.modified,
                        container::content(want)
                    )
                );
            }
        }
    }
}

#[test]
fn external_name_rejects_paths_without_overwriting_xml() {
    let t = Temp::new();
    let stem = t.0.join("sentinel");
    let sentinel = stem.with_extension("xml");
    std::fs::write(&sentinel, b"preserve external XML").unwrap();
    let invalid = [
        stem.to_str().unwrap().to_string(),
        format!("../{}/sentinel", t.0.file_name().unwrap().to_str().unwrap()),
        "../sentinel".into(),
        "..\\sentinel".into(),
        "/sentinel".into(),
        "\\sentinel".into(),
        ["C", ":sentinel"].concat(),
        ["C", ":\\sentinel"].concat(),
        "\\\\server\\share\\sentinel".into(),
        "folder/sentinel".into(),
        "folder\\sentinel".into(),
        "".into(),
        ".".into(),
        "..".into(),
        "1Name".into(),
        "has space".into(),
        "Name.xml".into(),
        "Bad<name".into(),
        "CON".into(),
        "nul".into(),
        "COM1".into(),
        "Lpt9".into(),
    ];
    for name in invalid {
        let result = xml::imp::new_external(&name);
        assert!(result.is_err(), "accepted invalid name {name:?}");
        assert_eq!(
            std::fs::read(&sentinel).unwrap(),
            b"preserve external XML",
            "{name:?}"
        );
        assert!(
            result.unwrap_err().contains("имя"),
            "missing name diagnostic: {name:?}"
        );
    }
}

#[test]
fn simultaneous_external_creation_keeps_each_temporary_directory_private() {
    for round in 0..4 {
        let barrier = std::sync::Arc::new(std::sync::Barrier::new(32));
        let workers: Vec<_> = (0..32)
            .map(|i| {
                let barrier = barrier.clone();
                std::thread::spawn(move || {
                    let name = format!("Обработка_{round}_{i}");
                    barrier.wait();
                    let config = Config::open(&xml::imp::new_external(&name).unwrap()).unwrap();
                    assert_eq!(config.root.header.name, name);
                })
            })
            .collect();
        for worker in workers {
            worker.join().unwrap();
        }
    }
}

#[test]
fn external_xml_roundtrip() {
    let t = Temp::new();
    let before = Config::open(&xml::imp::new_external("SyntheticProcessor").unwrap()).unwrap();
    assert!(xml::export(&before, &t.0).unwrap().errors.is_empty());
    let input = t.0.join("SyntheticProcessor.xml");
    let imp = xml::imp::import(&input, None).unwrap();
    assert!(imp.refused.is_empty(), "{:?}", imp.refused);
    let after = Config::open(&xml::imp::cf(&input, &imp).unwrap()).unwrap();
    assert_eq!(after.root.header.name, "SyntheticProcessor");
    for entry in before.entries.iter().filter(|e| e.name != "versions") {
        assert_eq!(
            container::content(entry),
            container::content(&after.entries[after.index[&entry.name]]),
            "{}",
            entry.name
        );
    }
    for name in ["Обработка_1", "_Processor2"] {
        let config = Config::open(&xml::imp::new_external(name).unwrap()).unwrap();
        assert_eq!(config.root.header.name, name);
    }
}

#[test]
fn minimal_configuration_xml_roundtrip() {
    let t = Temp::new();
    std::fs::write(
        t.0.join("Configuration.xml"),
        include_str!("fixtures/Configuration.xml"),
    )
    .unwrap();
    let imp = xml::imp::import(&t.0, None).unwrap();
    assert!(imp.refused.is_empty(), "{:?}", imp.refused);
    let bytes = xml::imp::cf(&t.0, &imp).unwrap();
    let parts = container::read_all(&bytes).unwrap();
    assert_eq!(parts.len(), 2);
    assert_eq!(parts[1].width, Width::W64);
    let c = Config::open(&bytes).unwrap();
    assert_eq!(c.root.header.name, "SyntheticConfig");
    let out = t.0.join("export");
    let report = xml::export(&c, &out).unwrap();
    assert!(report.errors.is_empty(), "{:?}", report.errors);
    let again = xml::imp::import(&out, None).unwrap();
    assert!(again.refused.is_empty(), "{:?}", again.refused);
    let reloaded = Config::open(&xml::imp::cf(&out, &again).unwrap()).unwrap();
    assert_eq!(reloaded.root.header.name, "SyntheticConfig");
}

#[test]
fn builtin_downgrade_tables() {
    let source=b"<MetaDataObject version=\"2.21\">\r\n<UseInInterfaceCompatibilityMode>Any</UseInInterfaceCompatibilityMode>\r\n<LineNumberLength>5</LineNumberLength>\r\n</MetaDataObject>";
    let v20 = xml::down::Format::load("2.20")
        .unwrap()
        .down(Path::new("Catalog.xml"), source)
        .unwrap();
    let v20 = String::from_utf8(v20).unwrap();
    assert!(v20.contains("version=\"2.20\""));
    assert!(!v20.contains("UseInInterfaceCompatibilityMode"));
    assert!(v20.contains("LineNumberLength"));
    let v17 = xml::down::Format::load("2.17")
        .unwrap()
        .down(Path::new("Catalog.xml"), source)
        .unwrap();
    let v17 = String::from_utf8(v17).unwrap();
    assert!(v17.contains("version=\"2.17\""));
    assert!(!v17.contains("UseInInterfaceCompatibilityMode"));
    assert!(!v17.contains("LineNumberLength"));
    assert!(xml::down::Format::load("unknown").is_err());
}

#[test]
fn single_w64_unpack_preserves_container_width() {
    let t = Temp::new();
    let bytes = container::write_as(
        &[Entry {
            name: "data".into(),
            created: 1,
            modified: 2,
            data: container::deflate(b"example"),
        }],
        Width::W64,
    );
    unpack::unpack_file(&bytes, &t.0).unwrap();
    let after = container::read_all(&unpack::pack_dir(&t.0).unwrap()).unwrap();
    assert_eq!(after[0].width, Width::W64);
}
