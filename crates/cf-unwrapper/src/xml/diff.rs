//! Сверка своей выгрузки с выгрузкой платформы: по видам файлов — сколько совпало байт в байт.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

#[derive(Debug, Default, Clone)]
pub struct Row {
    pub reference: usize,
    pub same: usize,
    pub differ: usize,
    pub missing: usize,
    pub extra: usize,
    /// Первые несовпавшие файлы: (путь, первая отличающаяся строка эталона).
    pub examples: Vec<(String, String)>,
}

/// Вид файла: имя без имени объекта (`Catalogs/<obj>.xml`, `Ext/ObjectModule.bsl`, `Help/<lang>.html`).
pub fn kind(rel: &str) -> String {
    let parts: Vec<&str> = rel.split('/').collect();
    let name = parts[parts.len() - 1];
    let parent = if parts.len() >= 2 { parts[parts.len() - 2] } else { "" };
    if parts.len() == 1 {
        return name.to_string();
    }
    match parent {
        "Help" => format!("Help/<lang>{}", ext(name)),
        "Forms" | "Commands" | "Templates" => format!("{parent}/<obj>.xml"),
        "Ext" | "Form" => name.to_string(),
        _ if parts.len() == 2 => format!("{parent}/<obj>.xml"),
        _ => format!("<other>{}", ext(name)),
    }
}

fn ext(name: &str) -> &str {
    name.rfind('.').map(|i| &name[i..]).unwrap_or("")
}

fn walk(root: &Path, dir: &Path, out: &mut BTreeSet<String>) -> std::io::Result<()> {
    for e in std::fs::read_dir(dir)? {
        let e = e?;
        // тип из записи каталога, без отдельного запроса к файлу
        let (p, is_dir) = (e.path(), e.file_type()?.is_dir());
        if is_dir {
            walk(root, &p, out)?;
        } else {
            out.insert(p.strip_prefix(root).unwrap().to_string_lossy().replace('\\', "/"));
        }
    }
    Ok(())
}

/// Сравнивает каталоги; `examples` — сколько примеров несовпадений хранить на вид.
pub fn compare(ours: &Path, reference: &Path, examples: usize) -> Result<BTreeMap<String, Row>, String> {
    let (mut a, mut b) = (BTreeSet::new(), BTreeSet::new());
    walk(ours, ours, &mut a).map_err(|e| format!("{}: {e}", ours.display()))?;
    walk(reference, reference, &mut b).map_err(|e| format!("{}: {e}", reference.display()))?;
    // служебный файл выгрузки: загрузка без него работает, на содержимое конфигурации не влияет
    b.remove("ConfigDumpInfo.xml");
    let mut rows: BTreeMap<String, Row> = BTreeMap::new();
    let read = |root: &Path, rel: &str| std::fs::read(root.join(rel)).map_err(|e| format!("{rel}: {e}"));
    // Файлы читаются по потокам; None — нет у нас, Some(None) — совпал, Some(Some(строка)) — отличается.
    let refs: Vec<&String> = b.iter().collect();
    let cmp = crate::par::map(&refs, || (), |_, rel| -> Result<Option<Option<String>>, String> {
        if !a.contains(*rel) {
            return Ok(None);
        }
        let (x, y) = (read(ours, rel)?, read(reference, rel)?);
        Ok(Some((x != y).then(|| first_diff(&x, &y))))
    });
    for (rel, c) in refs.into_iter().zip(cmp) {
        let row = rows.entry(kind(rel)).or_default();
        row.reference += 1;
        match c? {
            None => row.missing += 1,
            Some(None) => row.same += 1,
            Some(Some(d)) => {
                row.differ += 1;
                if row.examples.len() < examples {
                    row.examples.push((rel.clone(), d));
                }
            }
        }
    }
    for rel in a.difference(&b) {
        rows.entry(kind(rel)).or_default().extra += 1;
    }
    Ok(rows)
}

/// Номер и текст первой отличающейся строки: «12: эталон | наше».
fn first_diff(ours: &[u8], reference: &[u8]) -> String {
    let (x, y) = (String::from_utf8_lossy(ours), String::from_utf8_lossy(reference));
    let (mut xl, mut yl) = (x.split('\n'), y.split('\n'));
    for n in 1.. {
        match (xl.next(), yl.next()) {
            (Some(p), Some(q)) if p == q => continue,
            (p, q) => {
                let cut = |s: Option<&str>| s.map(|s| s.trim_end_matches('\r').chars().take(160).collect::<String>()).unwrap_or_else(|| "<конец>".into());
                return format!("{n}: {} | {}", cut(q), cut(p));
            }
        }
    }
    unreachable!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kinds() {
        assert_eq!(kind("Configuration.xml"), "Configuration.xml");
        assert_eq!(kind("Catalogs/Валюты.xml"), "Catalogs/<obj>.xml");
        assert_eq!(kind("Catalogs/Валюты/Ext/ObjectModule.bsl"), "ObjectModule.bsl");
        assert_eq!(kind("Catalogs/Валюты/Forms/ФормаСписка.xml"), "Forms/<obj>.xml");
        assert_eq!(kind("Catalogs/Валюты/Forms/ФормаСписка/Ext/Form/Module.bsl"), "Module.bsl");
        assert_eq!(kind("Catalogs/Валюты/Ext/Help/ru.html"), "Help/<lang>.html");
    }

    #[test]
    fn first_diff_shows_line() {
        assert_eq!(first_diff(b"a\r\nb\r\n", b"a\r\nc\r\n"), "2: c | b");
        assert_eq!(first_diff(b"a", b"a\nb"), "2: b | <конец>");
    }
}
