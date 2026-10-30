use cf_unwrapper::container;
use cf_unwrapper::meta::{role_ru, Config, Item, SUBKINDS};
use std::path::Path;
use std::process::ExitCode;

/// Как println!, но закрытый вывод (`cf_unwrapper tree | head`) — тихий выход, а не паника.
macro_rules! out {
    ($($t:tt)*) => {{
        use std::io::Write;
        if writeln!(std::io::stdout(), $($t)*).is_err() {
            std::process::exit(0);
        }
    }};
}

const HELP: &str = "cf-unwrapper — 1C container and XML format engine\n\n  cf-unwrapper info|ls|tree <file> [--all]\n  cf-unwrapper cat|module <file> <entry>\n  cf-unwrapper search <file> <text>\n  cf-unwrapper verify <file>\n  cf-unwrapper unpack <file> <directory>\n  cf-unwrapper pack <directory> <file>\n  cf-unwrapper export <file> <directory> [selection] [--format 2.20|2.17]\n  cf-unwrapper export-forms <file> <directory> [selection]\n  cf-unwrapper import <directory|external.xml> <file>\n  cf-unwrapper new-epf <name> <file.epf>\n  cf-unwrapper xml-down <source> <destination> --format 2.20|2.17\n  cf-unwrapper xml-diff <ours> <reference> [N]\n  cf-unwrapper import-check <xml> <unpacked> [selection|-] [N]";

fn main() -> ExitCode {
    let mut a: Vec<String> = std::env::args().skip(1).collect();
    let format = if let Some(i) = a.iter().position(|x| x == "--format") {
        if i + 1 >= a.len() || !matches!(a.first().map(String::as_str), Some("export" | "xml-down"))
        {
            eprintln!("--format <version> is supported by export and xml-down");
            return ExitCode::from(2);
        }
        let version = a.remove(i + 1);
        a.remove(i);
        if version != "2.21" {
            if let Err(e) = cf_unwrapper::xml::down::Format::load(&version) {
                eprintln!("{e}");
                return ExitCode::from(2);
            }
        } else if a.first().is_some_and(|c| c == "xml-down") {
            eprintln!("xml-down requires an older target format");
            return ExitCode::from(2);
        }
        Some(version)
    } else {
        None
    };
    let r = match a.iter().map(String::as_str).collect::<Vec<_>>()[..] {
        ["info", f] => info(f),
        ["ls", f] => ls(f),
        ["tree", f] => tree(f, false),
        ["tree", f, "--all"] => tree(f, true),
        ["cat", f, e] => cat(f, e),
        ["module", f, e] => module(f, e),
        ["search", f, t] => search(f, t),
        ["verify", f] => verify(f),
        ["unpack", f, d] => read(f)
            .and_then(|b| cf_unwrapper::unpack::unpack_file(&b, Path::new(d)))
            .map(|n| out!("распаковано файлов: {n}")),
        ["pack", d, f] => cf_unwrapper::unpack::pack_dir(Path::new(d)).and_then(|b| {
            std::fs::write(f, &b)
                .map_err(|e| e.to_string())
                .map(|_| out!("собрано: {f}, {} байт", b.len()))
        }),
        ["export", f, d] => export(f, d, false, None, format.as_deref()),
        ["export", f, d, l] => export(f, d, false, Some(l), format.as_deref()),
        ["export-forms", f, d] => export(f, d, true, None, None),
        ["export-forms", f, d, l] => export(f, d, true, Some(l), None),
        ["xml-diff", a, b] => xml_diff(a, b, 0),
        ["xml-down", a, b] => xml_down(a, b, format.as_deref()),
        ["import", d, f] => import(d, f),
        ["new-epf", name, f] => cf_unwrapper::xml::imp::new_external(name)
            .and_then(|b| std::fs::write(f, &b).map_err(|e| e.to_string()))
            .map(|_| out!("новая внешняя обработка {name}: {f}")),
        ["import-check", d, r] => import_check(d, r, "-", 3),
        ["import-check", d, r, l] => import_check(d, r, l, 3),
        ["import-check", d, r, l, n] => n
            .parse()
            .map_err(|_| "N — число".to_string())
            .and_then(|n| import_check(d, r, l, n)),
        ["xml-diff", a, b, n] => n
            .parse()
            .map_err(|_| "N — число".to_string())
            .and_then(|n| xml_diff(a, b, n)),
        _ => {
            if a.is_empty() || (a.len() == 1 && matches!(a[0].as_str(), "help" | "--help")) {
                out!("{HELP}");
            } else {
                eprintln!("invalid arguments; use --help");
            }
            return ExitCode::from(
                if a.is_empty() || (a.len() == 1 && matches!(a[0].as_str(), "help" | "--help")) {
                    0
                } else {
                    2
                },
            );
        }
    };
    match r {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("ошибка: {e}");
            ExitCode::FAILURE
        }
    }
}

fn read(f: &str) -> Result<Vec<u8>, String> {
    std::fs::read(f).map_err(|e| format!("{f}: {e}"))
}

fn open(f: &str) -> Result<Config, String> {
    Config::open(&read(f)?)
}

fn info(f: &str) -> Result<(), String> {
    let c = open(f)?;
    for (i, (w, n)) in c.parts.iter().enumerate() {
        out!("часть {i}: {w:?}, записей {n}");
    }
    out!(
        "версия формата: {}",
        if c.version.is_empty() {
            "—"
        } else {
            &c.version
        }
    );
    let h = &c.root.header;
    let what = if c.root.kind == "ExternalDataProcessor" {
        "внешняя обработка"
    } else {
        "конфигурация"
    };
    out!("{what}: {} ({}) {}", h.name, h.synonym, h.id);
    let total: usize = c.groups.iter().map(|g| g.items.len()).sum();
    out!("объектов: {total}");
    for g in &c.groups {
        out!("  {:<32} {}", g.title, g.items.len());
    }
    Ok(())
}

fn ls(f: &str) -> Result<(), String> {
    for (i, p) in container::read_all(&read(f)?)
        .map_err(|e| e.to_string())?
        .iter()
        .enumerate()
    {
        for e in &p.entries {
            out!("{i}\t{}\t{}", e.data.len(), e.name);
        }
    }
    Ok(())
}

fn tree(f: &str, all: bool) -> Result<(), String> {
    let c = open(f)?;
    out!("{}", c.root.header.name);
    for (file, role) in &c.root.files {
        out!("  · {} [{file}]", role_ru(role));
    }
    // у внешней обработки реквизиты, табличные части, формы и макеты — дети корня
    for (kind, title) in SUBKINDS {
        let ch: Vec<&Item> = c.root.children.iter().filter(|x| x.kind == *kind).collect();
        if !ch.is_empty() {
            out!("  {title} ({})", ch.len());
            for it in ch {
                print_item(it, 2, all);
            }
        }
    }
    for g in &c.groups {
        out!("  {} ({})", g.title, g.items.len());
        for it in &g.items {
            print_item(it, 2, all);
        }
    }
    Ok(())
}

fn print_item(it: &Item, depth: usize, all: bool) {
    let pad = "  ".repeat(depth);
    let syn = if it.header.synonym.is_empty() || it.header.synonym == it.header.name {
        String::new()
    } else {
        format!(" — {}", it.header.synonym)
    };
    out!("{pad}{}{syn}", it.header.name);
    if !all {
        return;
    }
    for (file, role) in &it.files {
        out!("{pad}  · {} [{file}]", role_ru(role));
    }
    for (kind, title) in SUBKINDS {
        let ch: Vec<&Item> = it.children.iter().filter(|c| c.kind == *kind).collect();
        if !ch.is_empty() {
            out!("{pad}  {title}");
            for c in ch {
                print_item(c, depth + 2, all);
            }
        }
    }
}

fn cat(f: &str, e: &str) -> Result<(), String> {
    // Любой контейнер (.hbk, .epf), не только конфигурация: ищем запись во всех частях файла.
    let b = read(f)?;
    let parts = container::read_all(&b).map_err(|e| e.to_string())?;
    let entry = parts
        .iter()
        .flat_map(|p| &p.entries)
        .find(|x| x.name == e)
        .ok_or(format!("нет записи {e}"))?;
    out!("{}", cf_unwrapper::brace::text(&container::content(entry)));
    Ok(())
}

fn module(f: &str, e: &str) -> Result<(), String> {
    let c = open(f)?;
    out!(
        "{}",
        c.module_text(e)
            .ok_or(format!("в записи {e} нет текста модуля"))?
    );
    Ok(())
}

fn verify(f: &str) -> Result<(), String> {
    let b = read(f)?;
    let parts = container::read_all(&b).map_err(|e| e.to_string())?;
    let rebuilt: Vec<u8> = parts
        .iter()
        .flat_map(|p| container::write(&p.entries))
        .collect();
    let again = container::read_all(&rebuilt)
        .map_err(|e| format!("пересобранный файл не читается: {e}"))?;
    let (mut n, mut nested) = (0usize, 0usize);
    for (p, q) in parts.iter().zip(&again) {
        if p.entries.len() != q.entries.len() {
            return Err(format!(
                "число записей {} != {}",
                p.entries.len(),
                q.entries.len()
            ));
        }
        for (x, y) in p.entries.iter().zip(&q.entries) {
            if (&x.name, x.created, x.modified, &x.data)
                != (&y.name, y.created, y.modified, &y.data)
            {
                return Err(format!("запись {} отличается", x.name));
            }
            let raw = container::content(x);
            if container::is_container(&raw) {
                let c = container::read_at(&raw, 0).map_err(|e| format!("{}: {e}", x.name))?;
                let w = container::read_at(&container::write(&c.entries), 0)
                    .map_err(|e| format!("{}: {e}", x.name))?;
                if c.entries
                    .iter()
                    .zip(&w.entries)
                    .any(|(a, b)| a.data != b.data || a.name != b.name)
                {
                    return Err(format!("вложенный контейнер {} отличается", x.name));
                }
                nested += 1;
            }
            n += 1;
        }
    }
    out!(
        "совпало записей: {n}, вложенных контейнеров: {nested}; исходно {} байт, после сборки {}",
        b.len(),
        rebuilt.len()
    );
    Ok(())
}

fn search(f: &str, t: &str) -> Result<(), String> {
    let c = open(f)?;
    let hits = c.search(t, 500);
    for (e, n, line) in &hits {
        out!("{}:{n}: {line}", c.describe(e));
    }
    out!("найдено: {}", hits.len());
    Ok(())
}

fn export(
    f: &str,
    d: &str,
    forms: bool,
    list: Option<&str>,
    format: Option<&str>,
) -> Result<(), String> {
    let c = open(f)?;
    let only = match list {
        Some(l) => Some(
            std::fs::read_to_string(l)
                .map_err(|e| format!("{l}: {e}"))?
                .lines()
                .filter(|s| !s.is_empty())
                .map(std::path::PathBuf::from)
                .collect::<Vec<_>>(),
        ),
        None => None,
    };
    let r = if let Some(v) = format.filter(|v| *v != "2.21") {
        cf_unwrapper::xml::export_as(
            &c,
            Path::new(d),
            only.as_deref(),
            &cf_unwrapper::xml::down::Format::load(v)?,
        )?
    } else if forms {
        cf_unwrapper::xml::export_forms(&c, Path::new(d), only.as_deref())?
    } else if let Some(o) = &only {
        cf_unwrapper::xml::export_only(&c, Path::new(d), o)?
    } else {
        cf_unwrapper::xml::export(&c, Path::new(d))?
    };
    out!("записано файлов: {}", r.files);
    for (role, n) in &r.skipped {
        out!("  не выгружено ({role}): {n}");
    }
    if r.errors.is_empty() {
        return Ok(());
    }
    for e in &r.errors {
        eprintln!("  {e}");
    }
    Err(format!("записей с ошибками: {}", r.errors.len()))
}

fn xml_down(src: &str, dst: &str, format: Option<&str>) -> Result<(), String> {
    let f = cf_unwrapper::xml::down::Format::load(format.ok_or("нужен --format <версия>")?)?;
    let (src, dst) = (Path::new(src), Path::new(dst));
    let (mut files, mut errors) = (0, 0);
    let mut stack = vec![src.to_path_buf()];
    while let Some(d) = stack.pop() {
        for e in std::fs::read_dir(&d)
            .map_err(|e| format!("{}: {e}", d.display()))?
            .flatten()
        {
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
                continue;
            }
            let rel = p.strip_prefix(src).unwrap();
            let data = std::fs::read(&p).map_err(|e| format!("{}: {e}", p.display()))?;
            match f.down(&p, &data) {
                Ok(b) => {
                    let to = dst.join(rel);
                    std::fs::create_dir_all(to.parent().unwrap()).map_err(|e| e.to_string())?;
                    std::fs::write(&to, b).map_err(|e| format!("{}: {e}", to.display()))?;
                    files += 1;
                }
                Err(e) => {
                    eprintln!("  {}: {e}", rel.display());
                    errors += 1;
                }
            }
        }
    }
    out!("записано файлов: {files} (формат {})", f.version);
    if errors > 0 {
        return Err(format!(
            "файлов, которые формат {} не выражает: {errors}",
            f.version
        ));
    }
    Ok(())
}

fn xml_diff(a: &str, b: &str, examples: usize) -> Result<(), String> {
    let rows = cf_unwrapper::xml::diff::compare(Path::new(a), Path::new(b), examples)?;
    let (mut total, mut same) = (0, 0);
    out!(
        "{:<40} {:>7} {:>7} {:>7} {:>7} {:>7}",
        "вид",
        "эталон",
        "равно",
        "иначе",
        "нет",
        "лишние"
    );
    for (k, r) in &rows {
        out!(
            "{k:<40} {:>7} {:>7} {:>7} {:>7} {:>7}",
            r.reference,
            r.same,
            r.differ,
            r.missing,
            r.extra
        );
        for (p, d) in &r.examples {
            out!(
                "    {p}
      {d}"
            );
        }
        total += r.reference;
        same += r.same;
    }
    out!(
        "итого: {same} из {total} файлов совпали ({:.1}%)",
        100.0 * same as f64 / total.max(1) as f64
    );
    Ok(())
}

fn list_file(l: &str) -> Result<Option<Vec<String>>, String> {
    if l == "-" {
        return Ok(None);
    }
    let t = std::fs::read_to_string(l).map_err(|e| format!("{l}: {e}"))?;
    Ok(Some(
        t.lines()
            .map(|s| s.trim().trim_end_matches(".xml").replace('\\', "/"))
            .filter(|s| !s.is_empty())
            .collect(),
    ))
}

fn import(d: &str, f: &str) -> Result<(), String> {
    let imp = cf_unwrapper::xml::imp::import(Path::new(d), None)?;
    if !imp.refused.is_empty() {
        for (file, why) in &imp.refused {
            eprintln!("  {file}: {why}");
        }
        return Err(format!(
            "не загружено файлов: {} (загружено {}); .cf не записан",
            imp.refused.len(),
            imp.files
        ));
    }
    let b = cf_unwrapper::xml::imp::cf(Path::new(d), &imp)?;
    let tmp = format!("{f}.cf_unwrapper-tmp");
    std::fs::write(&tmp, &b)
        .and_then(|_| std::fs::rename(&tmp, f))
        .map_err(|e| format!("{f}: {e}"))?;
    out!(
        "загружено файлов: {}, записей: {}; {f}, {} байт",
        imp.files,
        imp.recs.len(),
        b.len()
    );
    Ok(())
}

fn import_check(d: &str, r: &str, l: &str, n: usize) -> Result<(), String> {
    let only = list_file(l)?;
    let t = std::time::Instant::now();
    let imp = cf_unwrapper::xml::imp::import(Path::new(d), only.as_deref())?;
    let t_imp = t.elapsed();
    if let Ok(p) = std::env::var("CFKIT_REFUSED") {
        // все отказы по файлам — для выборок «что еще не загружается»
        let t: String = imp
            .refused
            .iter()
            .map(|(f, w)| {
                format!(
                    "{f}	{w}
"
                )
            })
            .collect();
        std::fs::write(&p, t).map_err(|e| format!("{p}: {e}"))?;
    }
    let t = std::time::Instant::now();
    let (rows, refused) = cf_unwrapper::xml::imp::check::compare(&imp, Path::new(r), n);
    // время фаз — в stderr, чтобы не мешать сравнению отчетов
    eprintln!(
        "время: загрузка {:.1} с, сверка {:.1} с",
        t_imp.as_secs_f64(),
        t.elapsed().as_secs_f64()
    );
    out!(
        "{:<44} {:>7} {:>7} {:>7}",
        "вид записи",
        "равно",
        "иначе",
        "лишние"
    );
    let (mut same, mut total) = (0, 0);
    for (k, row) in &rows {
        out!("{k:<44} {:>7} {:>7} {:>7}", row.same, row.differ, row.extra);
        for e in &row.examples {
            out!("    {e}");
        }
        same += row.same;
        total += row.same + row.differ + row.extra;
    }
    out!("записей: {same} из {total} совпали");
    out!("отказы (вид файла: число):");
    for (k, (cnt, why)) in &refused {
        out!("  {k}: {cnt}");
        let mut w: Vec<_> = why.values().collect();
        w.sort_by(|a, b| b.0.cmp(&a.0));
        for (c, ex) in w.iter().take(n.max(1)) {
            out!("    {c} × {ex}");
        }
    }
    out!("файлов отвергнуто: {}", imp.refused.len());
    Ok(())
}
