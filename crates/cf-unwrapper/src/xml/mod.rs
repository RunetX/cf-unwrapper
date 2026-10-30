//! Выгрузка в каталог XML, как «Выгрузить конфигурацию в файлы» (`ibcmd config export`, format="Hierarchical").
//! Пути и устройство файлов — docs/research/xml-export.md; сверка с платформой — `cfkit xml-diff`.

pub mod attr;
pub mod captions;
pub mod chart;
pub mod classes;
pub mod classes_nested;
pub mod classes_refs;
pub mod classes_regs;
pub mod classes_tools;
pub mod common;
pub mod dcs;
pub mod diff;
pub mod down;
pub mod edit;
pub mod newform;
pub mod ext;
pub mod flowchart;
pub mod form;
pub mod formedit;
pub mod form_props;
pub mod form_dcs;
pub mod form_v28;
pub mod form_v59;
pub mod graph;
pub mod imp;
pub mod mxl;
pub mod payload;
pub mod predefined;
pub mod rights;
pub mod standard;
pub mod xdto_table;
pub mod events_table;
pub mod commands_table;
pub mod ui;
pub mod w;

use crate::meta::{Config, Item};
use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;
use w::BOM;

/// Каталог класса: Catalog → Catalogs, ChartOfAccounts → ChartsOfAccounts, FilterCriterion → FilterCriteria.
pub fn plural(class: &str) -> String {
    if let Some(rest) = class.strip_prefix("ChartOf") {
        return format!("ChartsOf{rest}");
    }
    match class {
        "FilterCriterion" => "FilterCriteria".into(),
        "BusinessProcess" => "BusinessProcesses".into(),
        c => format!("{c}s"),
    }
}

/// Вложенные объекты со своим каталогом внутри владельца.
fn nested_dir(kind: &str) -> Option<&'static str> {
    match kind {
        "Form" => Some("Forms"),
        "Command" => Some("Commands"),
        "Template" => Some("Templates"),
        "Subsystem" => Some("Subsystems"),
        _ => None,
    }
}

#[derive(Debug, Default)]
pub struct Report {
    pub files: usize,
    /// Роль файла → сколько записей не выгружено (формат еще не восстановлен).
    pub skipped: BTreeMap<String, usize>,
    /// Записи, которые не удалось выгрузить: «запись (роль): причина».
    pub errors: Vec<String>,
    /// Файлы в памяти (режим [`Mode::Index`]): путь от корня выгрузки через `/` → содержимое.
    pub mem: Vec<(String, Vec<u8>)>,
}

/// Что выгружать: все, только Form.xml, только то, что читает индекс загрузки (`imp::index`) — в память.
#[derive(Clone, Copy, PartialEq)]
enum Mode {
    All,
    Forms,
    Index,
}

pub fn export(cfg: &Config, dir: &Path) -> Result<Report, String> {
    export_some(cfg, dir, Mode::All, None, None)
}

/// Выгрузка в формате старой версии (`down::Format`, таблица `formats/<версия>.txt`); `only` — как у [`export_only`].
pub fn export_as(cfg: &Config, dir: &Path, only: Option<&[PathBuf]>, fmt: &down::Format) -> Result<Report, String> {
    export_some(cfg, dir, Mode::All, only.map(|o| o.iter().map(|p| dir.join(p)).collect()), Some(fmt))
}

/// Только объекты, к которым относятся `only` (пути файлов относительно `dir`), — сверка отдельных объектов.
pub fn export_only(cfg: &Config, dir: &Path, only: &[PathBuf]) -> Result<Report, String> {
    export_some(cfg, dir, Mode::All, Some(only.iter().map(|p| dir.join(p)).collect()), None)
}

/// Только Form.xml управляемых форм, с причинами отказа в `errors` — быстрая сверка форм.
/// `only` — пути Form.xml относительно `dir`: выгрузить только их (выборка форм, tools/skel/forms_sample.py).
pub fn export_forms(cfg: &Config, dir: &Path, only: Option<&[PathBuf]>) -> Result<Report, String> {
    export_some(cfg, dir, Mode::Forms, only.map(|o| o.iter().map(|p| dir.join(p)).collect()), None)
}

/// Описания объектов, Configuration.xml и Predefined.xml — в памяти, без записи на диск: индекс загрузки по `.cf`
/// (правка формы в окне). Незнакомое значение — файла нет, как у выгрузки.
pub fn index_files(cfg: &Config) -> Result<std::collections::HashMap<String, Vec<u8>>, String> {
    Ok(export_some(cfg, Path::new(""), Mode::Index, None, None)?.mem.into_iter().collect())
}

fn export_some(cfg: &Config, dir: &Path, mode: Mode, only: Option<HashSet<PathBuf>>, fmt: Option<&down::Format>) -> Result<Report, String> {
    let (index, names, form_names) = (common::Index::new(cfg), OnceLock::new(), OnceLock::new());
    // Объекты выгружаются по потокам независимо; отчеты частей сливаются в порядке объектов — как при обходе подряд.
    let units: Vec<Option<(&'static str, &Item)>> = std::iter::once(None).chain(cfg.groups.iter().flat_map(|g| g.items.iter().map(move |it| Some((g.class, it))))).collect();
    // Индекс загрузки читает выгрузку нового режима (AnyIBRef) — правила режима ему не нужны.
    let compat = if mode == Mode::Index { None } else { compat_format(cfg)? };
    let compat = compat.as_ref();
    let failed = AtomicBool::new(false);
    let init = || Exporter { cfg, index: &index, report: Report::default(), names: &names, form_names: &form_names, mode, only: &only, dirs: HashSet::new(), fmt, compat };
    // Крупные объекты (регламентированные отчеты — по 3–5 с) раздаются первыми, иначе последний из них держит выгрузку.
    let mut order: Vec<usize> = (0..units.len()).collect();
    order.sort_by_cached_key(|&i| std::cmp::Reverse(units[i].map_or(usize::MAX, |(_, it)| weight(cfg, it))));
    let done = crate::par::map(&order, init, |x, &i| {
        if failed.load(Ordering::Relaxed) {
            return Ok(Report::default());
        }
        let r = x.unit(dir, units[i]).map(|_| std::mem::take(&mut x.report));
        failed.fetch_or(r.is_err(), Ordering::Relaxed);
        r
    });
    let mut parts: Vec<Option<Result<Report, String>>> = (0..units.len()).map(|_| None).collect();
    for (i, r) in order.into_iter().zip(done) {
        parts[i] = Some(r);
    }
    let mut report = Report::default();
    for p in parts.into_iter().flatten() {
        let p = p?;
        report.files += p.files;
        for (k, n) in p.skipped {
            *report.skipped.entry(k).or_default() += n;
        }
        report.errors.extend(p.errors);
        report.mem.extend(p.mem);
    }
    Ok(report)
}

/// Правила выгрузки старого режима совместимости (`formats/compat-8.3.16.txt`): выведены сверкой выгрузок 8.5 одной БП
/// в режимах 8.3.16 и 8.3.24; режимы между ними не сверены — отказ, а не догадка.
fn compat_format(cfg: &Config) -> Result<Option<down::Format>, String> {
    match classes_tools::compat_mode(cfg) {
        Some(c) if c <= 80316 => down::Format::load("compat-8.3.16").map(Some),
        Some(c) if c < 80324 => Err(format!("режим совместимости {c}: правила выгрузки выведены для 8.3.16 и ниже и для 8.3.24 и выше")),
        _ => Ok(None),
    }
}

/// Сжатый объем записей объекта со всеми вложенными — оценка времени его выгрузки.
fn weight(cfg: &Config, it: &Item) -> usize {
    let own: usize = it.files.iter().filter_map(|(e, _)| cfg.index.get(e)).map(|&i| cfg.entries[i].data.len()).sum();
    own + it.children.iter().map(|c| weight(cfg, c)).sum::<usize>()
}

/// Объект с каталогом `obj` (и описанием `obj.xml`) попадает в выборку; `None` — все.
fn wanted(only: &Option<HashSet<PathBuf>>, obj: &Path) -> bool {
    only.as_ref().is_none_or(|o| o.iter().any(|p| p.starts_with(obj) || p.with_extension("") == obj))
}

/// Состояние одного потока выгрузки; индексы общие, строятся один раз.
struct Exporter<'a> {
    cfg: &'a Config,
    index: &'a common::Index,
    report: Report,
    names: &'a OnceLock<ext::Names>,
    /// Имена объектов по GUID — для форм, строятся при первой форме.
    form_names: &'a OnceLock<form::Names>,
    mode: Mode,
    /// Полные пути файлов, объекты которых выгружать (в режиме форм — сами Form.xml); None — все.
    only: &'a Option<HashSet<PathBuf>>,
    /// Уже созданные этим потоком каталоги: create_dir_all на каждый из 90 тысяч файлов заметно дорог.
    dirs: HashSet<PathBuf>,
    /// Формат старой версии; None — 2.21.
    fmt: Option<&'a down::Format>,
    /// Правила старого режима совместимости (применяются до `fmt`); None — режим 8.3.24 и новее.
    compat: Option<&'a down::Format>,
}

/// Имя из файла как часть пути: только идентификатор 1С (буквы, цифры, `_`) или код языка (`pt-BR`)
/// и не имя устройства Windows, иначе `..`, `C:`, `a/b` или `CON` вывели бы запись за пределы каталога выгрузки.
pub fn safe_name(name: &str) -> bool {
    let upper = name.to_uppercase();
    let last = upper.chars().nth(3).filter(|_| upper.chars().count() == 4);
    let numbered = (upper.starts_with("COM") || upper.starts_with("LPT")) && last.is_some_and(|c| c.is_ascii_digit() || "¹²³".contains(c));
    let device = numbered || ["CON", "PRN", "AUX", "NUL"].contains(&upper.as_str());
    !name.is_empty() && !device && name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-')
}

impl Exporter<'_> {
    /// Часть выгрузки: `None` — файлы корня и Configuration.xml, иначе объект верхнего уровня с описанием.
    fn unit(&mut self, dir: &Path, unit: Option<(&'static str, &Item)>) -> Result<(), String> {
        let Some((class, it)) = unit else {
            // внешняя обработка: `<Имя>.xml` и каталог `<Имя>` рядом, как у ibcmd `config export --file`
            if self.cfg.root.kind != "Configuration" {
                let it = &self.cfg.root;
                if !self.safe(&it.header.name) || !wanted(self.only, &dir.join(&it.header.name)) {
                    return Ok(());
                }
                if self.mode != Mode::Forms {
                    match classes_tools::external_data_processor(self.cfg, self.index) {
                        Some(b) => self.write(&dir.join(format!("{}.xml", it.header.name)), &b)?,
                        None => *self.report.skipped.entry(format!("{}.xml", it.kind)).or_default() += 1,
                    }
                }
                return self.object(it, dir);
            }
            let root = wanted(self.only, &dir.join("Ext")) || wanted(self.only, &dir.join("Configuration"));
            if root && self.mode != Mode::Index {
                self.files(&self.cfg.root, dir)?;
            }
            if let Some(b) = (self.mode != Mode::Forms && root).then(|| classes_tools::configuration(self.cfg, self.index)).flatten() {
                self.write(&dir.join("Configuration.xml"), &b)?;
            }
            return Ok(());
        };
        let base = dir.join(plural(class));
        if !self.safe(&it.header.name) || !wanted(self.only, &base.join(&it.header.name)) {
            return Ok(());
        }
        match (self.mode != Mode::Forms).then(|| classes::descriptor(self.cfg, self.index, class, it)) {
            None => {}
            Some(Some(b)) => self.write(&base.join(format!("{}.xml", it.header.name)), &b)?,
            Some(None) => *self.report.skipped.entry(format!("{class}.xml")).or_default() += 1,
        }
        self.object(it, &base)
    }

    /// Небезопасное имя — объект пропускается и попадает в отчет.
    fn safe(&mut self, name: &str) -> bool {
        let ok = safe_name(name);
        if !ok {
            *self.report.skipped.entry("<недопустимое имя>".into()).or_default() += 1;
        }
        ok
    }

    fn object(&mut self, it: &Item, base: &Path) -> Result<(), String> {
        if !self.safe(&it.header.name) {
            return Ok(());
        }
        let own = base.join(&it.header.name);
        self.files(it, &own)?;
        for ch in &it.children {
            if let Some(d) = nested_dir(ch.kind) {
                // у команд своего файла описания нет — они в описании владельца
                if ch.kind != "Command" && self.mode != Mode::Forms {
                    match classes::descriptor(self.cfg, self.index, ch.kind, ch) {
                        Some(b) => self.write(&own.join(d).join(format!("{}.xml", ch.header.name)), &b)?,
                        None => *self.report.skipped.entry(format!("{}.xml", ch.kind)).or_default() += 1,
                    }
                }
                self.object(ch, &own.join(d))?;
            }
        }
        Ok(())
    }

    fn files(&mut self, it: &Item, own: &Path) -> Result<(), String> {
        let ext = own.join("Ext");
        for (entry, role) in &it.files {
            if self.mode == Mode::Forms {
                if *role == "Form" {
                    self.form(entry, &ext)?;
                }
                continue;
            }
            if self.mode == Mode::Index {
                if *role == "Predefined" {
                    self.ext_file(entry, role, &ext).ok();
                }
                continue;
            }
            for (rel, data) in payload::extra(self.cfg, entry, role) {
                self.write(&ext.join(rel), &data)?;
            }
            let mut rejected = Vec::new();
            let out = payload::files(self.cfg, self.index, self.fmt.is_some(), it, entry, role, &mut rejected);
            let reasoned = !rejected.is_empty();
            self.report.errors.extend(rejected.into_iter().map(|e| format!("{entry} ({role}): {e}")));
            if let Some(out) = out {
                for (rel, data) in out {
                    self.write(&ext.join(rel), &data)?;
                }
                continue;
            }
            let r = match *role {
                // Пустой модуль формы платформа не выгружает, пустые модули объектов — выгружает (один BOM).
                "Form" => self.module(entry, &ext.join("Form").join("Module.bsl"), false).and_then(|_| self.form(entry, &ext)),
                "Rights" => match self.cfg.entry_text(entry) {
                    Some(t) => rights::xml(self.cfg, &t, self.older().is_some_and(|v| v <= 17)).and_then(|x| self.write(&ext.join("Rights.xml"), &x)).map(|_| true),
                    None => Ok(false),
                },
                r @ ("Schedule" | "Content" | "CommandInterface" | "MainSectionCommandInterface" | "HomePageWorkArea" | "ClientApplicationInterface" | "Predefined" | "Flowchart") => self.ext_file(entry, r, &ext),
                r if r.ends_with("Module") => self.module(entry, &ext.join(format!("{r}.bsl")), true),
                _ => Ok(false),
            };
            // Нечитаемая запись не обрывает выгрузку остальных: ошибка попадает в отчет.
            let done = r.unwrap_or_else(|e| {
                self.report.errors.push(format!("{entry} ({role}): {e}"));
                true
            });
            // отказ с причиной уже в errors — не считаем его второй раз безымянным пропуском
            if !done && !reasoned {
                *self.report.skipped.entry(role.to_string()).or_default() += 1;
            }
        }
        Ok(())
    }

    /// Единственное место записи файлов выгрузки: файл с меткой [`UNK`] не пишется — отказ с причиной в `errors`.
    /// Целевая версия формата ниже 2.21: минорный номер (`2.17` → 17).
    fn older(&self) -> Option<u32> {
        self.fmt.and_then(|f| f.version.strip_prefix("2.")?.parse().ok()).filter(|v| *v < 21)
    }

    fn write(&mut self, path: &Path, data: &[u8]) -> Result<(), String> {
        if let Some(line) = unknown_mark(path, data) {
            self.report.errors.push(format!("{}: незнакомое значение: {line}", path.display()));
            return Ok(());
        }
        let mut data = std::borrow::Cow::Borrowed(data);
        for f in [self.compat, self.fmt].into_iter().flatten() {
            match f.down(path, &data) {
                Ok(d) => data = d.into(),
                Err(e) => {
                    self.report.errors.push(format!("{}: {e}", path.display()));
                    return Ok(());
                }
            }
        }
        let data = &*data;
        if self.mode == Mode::Index {
            self.report.mem.push((path.to_string_lossy().replace('\\', "/"), data.to_vec()));
            return Ok(());
        }
        if let Some(p) = path.parent().filter(|p| !self.dirs.contains(*p)) {
            std::fs::create_dir_all(p).map_err(|e| format!("{}: {e}", p.display()))?;
            self.dirs.insert(p.to_path_buf());
        }
        std::fs::write(path, data).map_err(|e| format!("{}: {e}", path.display()))?;
        self.report.files += 1;
        Ok(())
    }

    fn module(&mut self, entry: &str, path: &Path, empty: bool) -> Result<bool, String> {
        let Some(text) = self.cfg.module_text(entry) else { return Ok(false) };
        if text.is_empty() && !empty {
            return Ok(true);
        }
        self.write(path, &[BOM, text.as_bytes()].concat())?;
        Ok(true)
    }

    fn form(&mut self, entry: &str, ext: &Path) -> Result<bool, String> {
        if self.only.as_ref().is_some_and(|o| !o.contains(&ext.join("Form.xml"))) {
            return Ok(true);
        }
        // Форма обычного приложения — вложенный контейнер (Form.bin, `payload`), Form.xml у нее нет.
        if self.cfg.index.get(entry).is_some_and(|&i| crate::container::is_container(&crate::container::content(&self.cfg.entries[i]))) {
            return Ok(true);
        }
        let Some(text) = self.cfg.entry_text(entry) else { return Ok(false) };
        let names = self.form_names.get_or_init(|| {
            let mut n = form::Names::new(self.cfg);
            n.older = self.older();
            // индекс загрузки — около 13 с на ERP: только когда модель корня применяется
            n.index = if n.needs_index() { formedit::index(self.cfg).ok() } else { None };
            n
        });
        match form::form_xml(names, &text) {
            Ok(Some(x)) => self.write(&ext.join("Form.xml"), &x).map(|_| true),
            Err(e) if self.mode == Mode::Forms => {
                self.report.errors.push(format!("{entry} ({}): {e}", ext.display()));
                Ok(false)
            }
            Err(e) => Err(e),
            _ => Ok(false),
        }
    }
}

/// Метка значения, которое правило не смогло вывести (незнакомый код, неразрешенная ссылка, сырая запись).
/// U+0001 в XML 1.0 недопустим ни в каком виде, поэтому в верно восстановленном файле его не бывает.
pub const UNK: char = '\u{1}';

/// Строка с меткой (метка показана как `?`): в пути или в тексте .xml/.html; в остальных файлах
/// (картинки, .bin, модули — данные из .cf как есть) байт 1 — обычные данные.
fn unknown_mark(path: &Path, data: &[u8]) -> Option<String> {
    let p = path.to_string_lossy();
    if p.contains(UNK) {
        return Some(p.replace(UNK, "?"));
    }
    matches!(path.extension().and_then(|e| e.to_str()), Some("xml" | "html")).then_some(())?;
    data.contains(&(UNK as u8)).then_some(())?;
    String::from_utf8_lossy(data).lines().find(|l| l.contains(UNK)).map(|l| l.trim().replace(UNK, "?"))
}

/// base64 с переводами строк внутри, как в `{#base64:...}`.
pub fn base64(s: &str) -> Result<Vec<u8>, String> {
    let mut out = Vec::with_capacity(s.len() * 3 / 4);
    let (mut acc, mut bits) = (0u32, 0u32);
    for c in s.bytes() {
        let v = match c {
            b'A'..=b'Z' => c - b'A',
            b'a'..=b'z' => c - b'a' + 26,
            b'0'..=b'9' => c - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            b'=' | b'\r' | b'\n' | b' ' => continue,
            _ => return Err(format!("base64: недопустимый символ {:?}", c as char)),
        };
        acc = acc << 6 | v as u32;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((acc >> bits) as u8);
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_decodes_with_newlines() {
        assert_eq!(base64("0J/R\r\ngA==").unwrap(), "Пр".as_bytes());
        assert!(base64("a*").is_err());
    }

    #[test]
    fn unsafe_names_rejected() {
        for bad in ["", "..", "a/b", "a\\b", "C:", "C:x", "..\\x", "CON", "con", "Lpt1", "COM9", "COM¹", "NUL", "a.b", "a b", "x\0"] {
            assert!(!safe_name(bad), "{bad:?}");
        }
        for good in ["Валюты", "ФормаСписка_en", "ru", "pt-BR", "Console", "COM10", "_1"] {
            assert!(safe_name(good), "{good:?}");
        }
    }

    #[test]
    fn unknown_marks() {
        let x = Path::new("a/Ext/Form.xml");
        assert_eq!(unknown_mark(x, "<a>\r\n\t<b>web:\u{1}97</b>\r\n</a>".as_bytes()).as_deref(), Some("<b>web:?97</b>"));
        assert!(unknown_mark(Path::new("a/Help/ru.html"), "<p>\u{1}x</p>".as_bytes()).is_some());
        assert!(unknown_mark(Path::new("a/Ext/Picture/Picture.\u{1}"), b"").is_some());
        // знак вопроса — обычный текст; байт 1 в двоичном файле — данные
        assert!(unknown_mark(x, b"<a x=\"?\">?y</a>").is_none());
        assert!(unknown_mark(Path::new("a/Ext/Template.bin"), b"\x01\x02").is_none());
    }

    #[test]
    fn marked_file_not_written() {
        let root = Item { kind: "Configuration", header: Default::default(), files: vec![], children: vec![] };
        let cfg = Config { entries: vec![], index: Default::default(), root, groups: vec![], version: String::new(), parts: vec![], other_parts: vec![], main_part: 0, root_id: String::new() };
        let (index, names, form_names) = (common::Index::new(&cfg), OnceLock::new(), OnceLock::new());
        let mut x = Exporter { cfg: &cfg, index: &index, report: Report::default(), names: &names, form_names: &form_names, mode: Mode::All, only: &None, dirs: HashSet::new(), fmt: None, compat: None };
        let dir = std::env::temp_dir().join(format!("cfkit-unk-{}", std::process::id()));
        x.write(&dir.join("a.xml"), "<a>\u{1}1</a>".as_bytes()).unwrap();
        x.write(&dir.join("b.xml"), b"<a>1</a>").unwrap();
        assert!(!dir.join("a.xml").exists() && dir.join("b.xml").exists());
        assert_eq!((x.report.files, x.report.errors.len()), (1, 1));
        assert!(x.report.errors[0].contains("a.xml: незнакомое значение: <a>?1</a>"), "{:?}", x.report.errors);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn plural_names() {
        assert_eq!(plural("Catalog"), "Catalogs");
        assert_eq!(plural("ChartOfCharacteristicTypes"), "ChartsOfCharacteristicTypes");
        assert_eq!(plural("FilterCriterion"), "FilterCriteria");
        assert_eq!(plural("BusinessProcess"), "BusinessProcesses");
    }
}
