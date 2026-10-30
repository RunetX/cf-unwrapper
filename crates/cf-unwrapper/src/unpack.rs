//! Распаковка контейнера в каталог и обратная сборка.
//!
//! В каждом каталоге лежит `.cfkit`: строки `имя\tcreated\tmodified\tфлаги` в исходном порядке.
//! Флаги: `z` — данные были сжаты deflate, `c` — запись является контейнером и развернута в подкаталог.
//! Файл из нескольких частей (крупные .cf) раскладывается в `part0/`, `part1/`, …; манифест частей —
//! `parts\tN\tширины` (`W32,W64`): платформа не читает конфигурацию из части, собранной другой ширины.

use crate::container::{self, Entry};
use std::fs;
use std::path::Path;

const MANIFEST: &str = ".cfkit";

fn safe_name(name: &str) -> Result<&str, String> {
    if name.is_empty() || name == "." || name == ".." || name == MANIFEST || name.contains(['/', '\\', ':']) {
        return Err(format!("недопустимое имя записи: {name:?}"));
    }
    Ok(name)
}

pub fn unpack_file(bytes: &[u8], dir: &Path) -> Result<usize, String> {
    let parts = container::read_all(bytes).map_err(|e| e.to_string())?;
    fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    if parts.len() == 1 && parts[0].width == container::Width::W32 {
        return unpack_entries(&parts[0].entries, dir, true);
    }
    let mut n = 0;
    for (i, p) in parts.iter().enumerate() {
        n += unpack_entries(&p.entries, &dir.join(format!("part{i}")), true)?;
    }
    let widths: Vec<_> = parts.iter().map(|p| format!("{:?}", p.width)).collect();
    fs::write(dir.join(MANIFEST), format!("parts\t{}\t{}\n", parts.len(), widths.join(","))).map_err(|e| e.to_string())?;
    Ok(n)
}

fn unpack_entries(entries: &[Entry], dir: &Path, top: bool) -> Result<usize, String> {
    fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let mut manifest = String::new();
    let mut n = 0;
    for e in entries {
        let name = safe_name(&e.name)?;
        let inflated = if top { container::inflate(&e.data) } else { None };
        let z = inflated.is_some();
        let data = inflated.unwrap_or_else(|| e.data.clone());
        let nested = container::is_container(&data) && container::read_at(&data, 0).is_ok();
        let flags = format!("{}{}", if z { "z" } else { "" }, if nested { "c" } else { "" });
        manifest.push_str(&format!("{name}\t{}\t{}\t{flags}\n", e.created, e.modified));
        if nested {
            let c = container::read_at(&data, 0).map_err(|e| e.to_string())?;
            n += unpack_entries(&c.entries, &dir.join(name), false)?;
        } else {
            fs::write(dir.join(name), &data).map_err(|e| format!("{}: {e}", dir.join(name).display()))?;
            n += 1;
        }
    }
    fs::write(dir.join(MANIFEST), manifest).map_err(|e| e.to_string())?;
    Ok(n)
}

pub fn pack_dir(dir: &Path) -> Result<Vec<u8>, String> {
    let m = fs::read_to_string(dir.join(MANIFEST)).map_err(|e| format!("{}: {e}", dir.join(MANIFEST).display()))?;
    if let Some(rest) = m.strip_prefix("parts\t") {
        let mut f = rest.trim().split('\t');
        let cnt: usize = f.next().unwrap_or("").parse().map_err(|_| "плохой манифест частей")?;
        // без ширин — манифест прежних версий: ширина по объему
        let widths: Vec<&str> = f.next().map(|w| w.split(',').collect()).unwrap_or_default();
        let mut out = Vec::new();
        for i in 0..cnt {
            let e = pack_entries(&dir.join(format!("part{i}")))?;
            out.extend(match widths.get(i) {
                Some(&"W64") => container::write_as(&e, container::Width::W64),
                Some(&"W32") => container::write_as(&e, container::Width::W32),
                Some(w) => return Err(format!("плохая ширина части в манифесте: {w}")),
                None => container::write(&e),
            });
        }
        return Ok(out);
    }
    Ok(container::write(&pack_entries(dir)?))
}

fn pack_entries(dir: &Path) -> Result<Vec<Entry>, String> {
    let m = fs::read_to_string(dir.join(MANIFEST)).map_err(|e| format!("{}: {e}", dir.join(MANIFEST).display()))?;
    let mut entries = Vec::new();
    for line in m.lines().filter(|l| !l.is_empty()) {
        let f: Vec<&str> = line.split('\t').collect();
        let [name, created, modified, flags] = f[..] else { return Err(format!("плохая строка манифеста: {line}")) };
        let name = safe_name(name)?;
        let num = |s: &str| s.parse::<u64>().map_err(|_| format!("плохое время в манифесте: {line}"));
        let mut data = if flags.contains('c') {
            container::write(&pack_entries(&dir.join(name))?)
        } else {
            fs::read(dir.join(name)).map_err(|e| format!("{}: {e}", dir.join(name).display()))?
        };
        if flags.contains('z') {
            data = container::deflate(&data);
        }
        entries.push(Entry { name: name.to_string(), created: num(created)?, modified: num(modified)?, data });
    }
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::container::{deflate, write};

    #[test]
    fn unpack_pack_roundtrip() {
        let inner = write(&[
            Entry { name: "info".into(), created: 1, modified: 1, data: b"{3,1,0,\"\",0}".to_vec() },
            Entry { name: "text".into(), created: 1, modified: 1, data: "Процедура Тест() КонецПроцедуры".as_bytes().to_vec() },
        ]);
        let file = write(&[
            Entry { name: "root".into(), created: 7, modified: 8, data: deflate(b"{2,abc,}") },
            Entry { name: "abc.0".into(), created: 9, modified: 10, data: deflate(&inner) },
        ]);
        let dir = std::env::temp_dir().join(format!("cfkit-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        assert_eq!(unpack_file(&file, &dir).unwrap(), 3);
        assert_eq!(fs::read_to_string(dir.join("abc.0").join("text")).unwrap(), "Процедура Тест() КонецПроцедуры");
        let again = pack_dir(&dir).unwrap();
        let (a, b) = (container::read_at(&file, 0).unwrap(), container::read_at(&again, 0).unwrap());
        for (x, y) in a.entries.iter().zip(&b.entries) {
            assert_eq!((&x.name, x.created, x.modified), (&y.name, y.created, y.modified));
            assert_eq!(container::content(x), container::content(y));
        }
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn parts_keep_width() {
        let e = |n: &str| vec![Entry { name: n.into(), created: 1, modified: 1, data: b"x".to_vec() }];
        let file = [container::write_as(&e("a"), container::Width::W32), container::write_as(&e("b"), container::Width::W64)].concat();
        let dir = std::env::temp_dir().join(format!("cfkit-parts-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        unpack_file(&file, &dir).unwrap();
        let w: Vec<_> = container::read_all(&pack_dir(&dir).unwrap()).unwrap().iter().map(|p| p.width).collect();
        assert_eq!(w, [container::Width::W32, container::Width::W64]);
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn rejects_path_traversal() {
        let file = write(&[Entry { name: "..".into(), created: 0, modified: 0, data: vec![1] }]);
        let dir = std::env::temp_dir().join(format!("cfkit-trav-{}", std::process::id()));
        assert!(unpack_file(&file, &dir).is_err());
        let _ = fs::remove_dir_all(&dir);
    }
}
