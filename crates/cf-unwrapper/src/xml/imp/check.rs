//! Сверка загрузки с записями платформы (`cfkit unpack` ее .cf): по видам — сколько записей совпало байт в байт,
//! сколько отличается (с первым отличием) и сколько файлов отвергнуто (с причинами).

use super::{Import, Rec};
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Default)]
pub struct Row {
    pub same: usize,
    pub differ: usize,
    /// Записи нет у платформы.
    pub extra: usize,
    pub examples: Vec<String>,
}

/// Строки отчета: вид записи → итог; отказы — вид файла → (число, причины с примером).
pub fn compare(imp: &Import, reference: &Path, examples: usize) -> (BTreeMap<String, Row>, BTreeMap<String, (usize, BTreeMap<String, (usize, String)>)>) {
    // чтение эталона упирается в открытие файлов, не в процессор: записи сверяются на всех ядрах, итог — в порядке записей
    let recs: Vec<_> = imp.recs.iter().collect();
    let found = crate::par::map(&recs, || (), |_, (name, (kind, rec))| {
        let generated = kind == "описание: Configuration";
        let r = read_ref(&reference.join(name)).map(|r| if generated { mask_hash(r) } else { r });
        // вложенный контейнер, перенесенный байт в байт (модуль без текста, обычная форма), — по записям, как эталон
        let unpacked;
        let rec = match (rec, &r) {
            (Rec::Raw(b), Some(Rec::Dir(_))) if crate::container::is_container(b) => {
                unpacked = super::payload::container(b).unwrap_or_else(|_| rec.clone());
                &unpacked
            }
            _ => rec,
        };
        match r {
            None => Some(None),
            Some(r) if r == *rec => None,
            Some(r) => {
                // для разбора: CFKIT_IMPORT_DUMP=<каталог> — свои отличающиеся записи туда
                if let (Ok(d), Rec::Raw(b)) = (std::env::var("CFKIT_IMPORT_DUMP"), rec) {
                    let _ = std::fs::create_dir_all(&d);
                    let _ = std::fs::write(Path::new(&d).join(name), b);
                }
                Some(Some(first_diff(rec, &r)))
            }
        }
    });
    let mut rows: BTreeMap<String, Row> = BTreeMap::new();
    // None — совпала; Some(None) — нет у платформы; Some(Some(отличие))
    for ((name, (kind, _)), f) in recs.iter().zip(found) {
        let row = rows.entry(kind.clone()).or_default();
        match f {
            None => row.same += 1,
            Some(None) => {
                row.extra += 1;
                if row.examples.len() < examples {
                    row.examples.push(format!("{name}: нет у платформы"));
                }
            }
            Some(Some(d)) => {
                row.differ += 1;
                if row.examples.len() < examples {
                    row.examples.push(format!("{name}: {d}"));
                }
            }
        }
    }
    let mut refused: BTreeMap<String, (usize, BTreeMap<String, (usize, String)>)> = BTreeMap::new();
    // для разбора: CFKIT_IMPORT_REFUSED=<файл> — все отказы `файл\tпричина`
    if let Ok(f) = std::env::var("CFKIT_IMPORT_REFUSED") {
        let _ = std::fs::write(f, imp.refused.iter().map(|(file, why)| format!("{file}\t{}\n", why.replace(['\r', '\n'], " "))).collect::<String>());
    }
    for (file, why) in &imp.refused {
        let e = refused.entry(super::super::diff::kind(file)).or_default();
        e.0 += 1;
        // причина без номеров строк — чтобы одинаковые собирались вместе
        let key: String = why.split("(строка").next().unwrap_or(why).chars().filter(|c| !c.is_ascii_digit()).take(120).collect();
        let r = e.1.entry(key).or_insert((0, format!("{file}: {why}")));
        r.0 += 1;
    }
    (rows, refused)
}

/// Число в конце записи конфигурации `{{1,"",""},{<число>}}` платформа пишет новое при каждой загрузке:
/// при сверке оно заменяется нулем, как пишет загрузка.
fn mask_hash(r: Rec) -> Rec {
    let Rec::Raw(b) = r else { return r };
    let t = String::from_utf8_lossy(&b).into_owned();
    let tail = "\r\n}\r\n}";
    match t.strip_suffix(tail).and_then(|h| h.rfind("\r\n{").map(|i| (h, i))) {
        Some((h, i)) if h[i + 3..].strip_suffix('}').is_some_and(|n| n.trim_start_matches('-').bytes().all(|c| c.is_ascii_digit())) => {
            Rec::Raw(format!("{}\r\n{{0}}{tail}", &h[..i]).into_bytes())
        }
        _ => Rec::Raw(b),
    }
}

fn read_ref(p: &Path) -> Option<Rec> {
    if p.is_dir() {
        let m = std::fs::read_to_string(p.join(".cfkit")).ok()?;
        let mut v = Vec::new();
        for l in m.lines().filter(|l| !l.is_empty()) {
            let name = l.split('\t').next()?;
            v.push((name.to_string(), std::fs::read(p.join(name)).ok()?));
        }
        return Some(Rec::Dir(v));
    }
    std::fs::read(p).ok().map(Rec::Raw)
}

fn first_diff(ours: &Rec, theirs: &Rec) -> String {
    match (ours, theirs) {
        (Rec::Raw(a), Rec::Raw(b)) => bytes_diff(a, b),
        (Rec::Dir(a), Rec::Dir(b)) => {
            let (na, nb): (Vec<&String>, Vec<&String>) = (a.iter().map(|e| &e.0).collect(), b.iter().map(|e| &e.0).collect());
            if na != nb {
                return format!("записи контейнера {na:?} у платформы {nb:?}");
            }
            a.iter().zip(b).find(|(x, y)| x.1 != y.1).map_or("?".into(), |(x, y)| format!("{}: {}", x.0, bytes_diff(&x.1, &y.1)))
        }
        (Rec::Raw(_), Rec::Dir(_)) => "у платформы вложенный контейнер".into(),
        (Rec::Dir(_), Rec::Raw(_)) => "у платформы не контейнер".into(),
    }
}

/// Позиция первого отличия и окрестность с обеих сторон.
pub fn bytes_diff(a: &[u8], b: &[u8]) -> String {
    let i = a.iter().zip(b).position(|(x, y)| x != y).unwrap_or(a.len().min(b.len()));
    let ctx = |s: &[u8]| {
        let from = i.saturating_sub(60);
        let to = (i + 60).min(s.len());
        String::from_utf8_lossy(&s[from..to]).replace("\r\n", "⏎").replace('\n', "↵")
    };
    format!("байт {i} (длина {} / {}):\n        наш: {}\n        эталон: {}", a.len(), b.len(), ctx(a), ctx(b))
}
