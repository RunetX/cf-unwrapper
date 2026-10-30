use std::process::Command;

fn run(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_cf-unwrapper"))
        .args(args)
        .output()
        .unwrap()
}

#[test]
fn format_option_is_never_silently_ignored() {
    for args in [
        vec!["--help", "--format", "unknown"],
        vec!["--help", "--format"],
        vec!["--help", "extra"],
    ] {
        let out = run(&args);
        assert!(!out.status.success(), "accepted {args:?}");
        assert!(!out.stderr.is_empty(), "no diagnostic for {args:?}");
    }
}

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!(
            "cf-unwrapper-cli-{}-{}",
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
fn text(p: &Path) -> &str {
    p.to_str().unwrap()
}
fn success(args: &[&str]) {
    let o = run(args);
    assert!(
        o.status.success(),
        "{args:?}: {}",
        String::from_utf8_lossy(&o.stderr)
    );
}

#[test]
fn help_and_invalid_inputs() {
    let out = run(&["--help"]);
    assert!(out.status.success());
    let help = String::from_utf8(out.stdout).unwrap();
    assert!(help.starts_with("cf-unwrapper"));
    for excluded in ["dbg", "base", "res ", "bsl-check", "set-module"] {
        assert!(!help.contains(excluded));
    }
    for args in [
        vec!["bogus"],
        vec!["info", "missing.cf"],
        vec!["export", "missing.cf", "out", "--format", "9.99"],
        vec!["new-epf", "X"],
    ] {
        let out = run(&args);
        assert!(!out.status.success());
        assert!(!out.stderr.is_empty());
    }
}

#[test]
fn actual_binary_external_roundtrip_and_inspection() {
    let t = Temp::new();
    let epf = t.0.join("input.epf");
    let unpacked = t.0.join("unpacked");
    let packed = t.0.join("packed.epf");
    let xml = t.0.join("xml");
    let imported = t.0.join("imported.epf");
    success(&["new-epf", "SyntheticProcessor", text(&epf)]);
    for cmd in ["info", "ls", "tree", "verify"] {
        success(&[cmd, text(&epf)]);
    }
    success(&["cat", text(&epf), "root"]);
    success(&["search", text(&epf), "Synthetic"]);
    success(&["unpack", text(&epf), text(&unpacked)]);
    success(&["pack", text(&unpacked), text(&packed)]);
    success(&["verify", text(&packed)]);
    success(&["export", text(&epf), text(&xml)]);
    success(&["export-forms", text(&epf), text(&t.0.join("forms"))]);
    let input = xml.join("SyntheticProcessor.xml");
    success(&["import", text(&input), text(&imported)]);
    success(&["verify", text(&imported)]);
    let after = cf_unwrapper::meta::Config::open(&std::fs::read(imported).unwrap()).unwrap();
    assert_eq!(after.root.header.name, "SyntheticProcessor");
}

#[test]
fn invalid_external_name_preserves_xml_and_result() {
    let t = Temp::new();
    let stem = t.0.join("sentinel");
    let sentinel = stem.with_extension("xml");
    let dest = t.0.join("existing.epf");
    std::fs::write(&sentinel, b"preserve external XML").unwrap();
    std::fs::write(&dest, b"preserve existing result").unwrap();
    let out = run(&["new-epf", text(&stem), text(&dest)]);
    assert!(!out.status.success());
    assert!(!out.stderr.is_empty());
    assert_eq!(std::fs::read(&sentinel).unwrap(), b"preserve external XML");
    assert_eq!(std::fs::read(&dest).unwrap(), b"preserve existing result");
    let fresh = t.0.join("fresh.epf");
    assert!(!run(&["new-epf", text(&stem), text(&fresh)])
        .status
        .success());
    assert!(!fresh.exists());
}

#[test]
fn refused_import_preserves_existing_result() {
    let t = Temp::new();
    let epf = t.0.join("input.epf");
    let xml = t.0.join("xml");
    let dest = t.0.join("result.epf");
    success(&["new-epf", "SyntheticProcessor", text(&epf)]);
    success(&["export", text(&epf), text(&xml)]);
    let input = xml.join("SyntheticProcessor.xml");
    let data = std::fs::read_to_string(&input).unwrap().replace(
        "</Properties>",
        "<UnknownProperty>true</UnknownProperty></Properties>",
    );
    std::fs::write(&input, data).unwrap();
    std::fs::write(&dest, b"preserve existing result").unwrap();
    let out = run(&["import", text(&input), text(&dest)]);
    assert!(!out.status.success());
    assert!(!out.stderr.is_empty());
    assert_eq!(std::fs::read(&dest).unwrap(), b"preserve existing result");
    let fresh = t.0.join("fresh.epf");
    assert!(!run(&["import", text(&input), text(&fresh)])
        .status
        .success());
    assert!(!fresh.exists());
}

#[test]
fn binary_format_export_and_xml_down() {
    let t = Temp::new();
    let epf = t.0.join("input.epf");
    success(&["new-epf", "SyntheticProcessor", text(&epf)]);
    let src = t.0.join("source");
    std::fs::create_dir(&src).unwrap();
    std::fs::write(src.join("Object.xml"),b"<MetaDataObject version=\"2.21\">\r\n<UseInInterfaceCompatibilityMode>Any</UseInInterfaceCompatibilityMode>\r\n<LineNumberLength>5</LineNumberLength>\r\n</MetaDataObject>").unwrap();
    for v in ["2.20", "2.17"] {
        let dst = t.0.join(v);
        success(&["xml-down", text(&src), text(&dst), "--format", v]);
        let result = std::fs::read_to_string(dst.join("Object.xml")).unwrap();
        assert!(result.contains(&format!("version=\"{v}\"")));
        assert!(!result.contains("UseInInterfaceCompatibilityMode"));
        assert_eq!(result.contains("LineNumberLength"), v == "2.20");
        success(&[
            "export",
            text(&epf),
            text(&t.0.join(format!("export-{v}"))),
            "--format",
            v,
        ]);
    }
}
