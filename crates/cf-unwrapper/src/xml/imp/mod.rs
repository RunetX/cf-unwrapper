//! Загрузка XML-выгрузки (`ibcmd config export`, «Выгрузить конфигурацию в файлы») в записи .cf — обратная выгрузке.
//! Правила и покрытие — docs/research/xml-import.md; сверка с загрузкой платформы — `cfkit import-check`.
//!
//! Без потерь или отказ: файл, в котором что-то не разобрано (непрочитанный элемент или атрибут, незнакомое значение),
//! не дает ни одной записи и попадает в `refused` с причиной; файл, для которого загрузки еще нет, — тоже.

pub mod attr;
pub mod check;
pub(crate) mod config;
mod dcs;
pub mod desc;
pub(crate) mod extension;
pub mod dom;
pub(crate) mod ext;
mod flowchart;
pub(crate) mod form;
mod hashorder;
pub mod html;
pub mod index;
pub(crate) mod module;
mod chart;
mod mxl;
mod predef;
pub mod payload;
pub mod refs;
pub(crate) mod rights;
pub mod standard;
pub mod tpl;
pub mod ui;

use crate::meta::own_role;
use dom::El;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::sync::Arc;

/// Содержимое записи: как есть или вложенный контейнер (имя, данные) в порядке записей.
#[derive(Clone, Debug, PartialEq)]
pub enum Rec {
    Raw(Vec<u8>),
    Dir(Vec<(String, Vec<u8>)>),
}

#[derive(Default)]
pub struct Import {
    /// Имя записи → (вид для отчета, содержимое).
    pub recs: BTreeMap<String, (String, Rec)>,
    /// (файл относительно каталога выгрузки, причина)
    pub refused: Vec<(String, String)>,
    /// Файлы, ставшие записями.
    pub files: usize,
}

/// Готовые записи из одного файла.
type Out = Vec<(String, String, Rec)>;

pub const BOM: &[u8] = &[0xef, 0xbb, 0xbf];

/// Загрузка каталога; `only` — пути объектов относительно каталога (`CommonModules/Имя`, как в списке `export`).
pub fn import(dir: &Path, only: Option<&[String]>) -> Result<Import, String> {
    if is_external(dir) {
        return external(dir);
    }
    // для выборки — только каталоги выбранных объектов (обход всех 90 тыс. файлов БП — секунды)
    let all = match only {
        None => walk(dir, "")?,
        Some(o) => {
            let mut v = Vec::new();
            for p in o.iter().map(String::as_str).chain(["Ext"]) {
                if dir.join(format!("{p}.xml")).is_file() {
                    v.push(format!("{p}.xml"));
                }
                if dir.join(p).is_dir() {
                    v.extend(walk(dir, p)?);
                }
            }
            v.extend(["Configuration.xml", "ConfigDumpInfo.xml"].map(String::from));
            v.sort();
            v.dedup();
            v
        }
    };
    let mut x = Ctx { dir, out: Import::default(), claimed: BTreeSet::new(), index: None };
    let conf = x.parse("Configuration.xml")?;
    let cobj = conf.req("Configuration")?;
    let root_id = cobj.req_attr("uuid")?.to_string();
    // модули приложения названы по ObjectId объекта конфигурации (ClassId 9cd510cd-…)
    let own = cobj
        .req("InternalInfo")?
        .all("xr:ContainedObject")
        .find(|c| c.child("xr:ClassId").and_then(|e| e.text().ok()).as_deref() == Some(CONFIG_CLASS))
        .and_then(|c| c.child("xr:ObjectId").and_then(|e| e.text().ok()))
        .ok_or("Configuration.xml: нет ContainedObject конфигурации")?;
    let wanted = |rel: &str| only.is_none_or(|o| o.iter().any(|p| rel == p.as_str() || rel.starts_with(&format!("{p}/")) || rel == format!("{p}.xml")));
    let root = wanted("Configuration") || wanted("Ext");
    if root {
        // версии объектов: платформа при загрузке назначает новые (запись versions), файл не загружается
        x.claimed.insert("ConfigDumpInfo.xml".into());
        x.ext("Ext", "Configuration", &own);
        let r = x.idx().map(|_| ()).and_then(|_| {
            let ix = x.index.as_deref().unwrap();
            x.xml("Configuration.xml", |d| config::record(d, ix))
        });
        let r = r.map(|b| vec![(root_id.clone(), "описание: Configuration".to_string(), Rec::Raw(b))]);
        x.take("Configuration.xml", r);
    }
    let mut objs = Vec::new();
    for ch in cobj.req("ChildObjects")?.elems() {
        let name = ch.text()?;
        let base = super::plural(&ch.name);
        if wanted(&format!("{base}/{name}")) {
            objs.push((ch.name.clone(), name, base));
        }
    }
    // объекты не зависят друг от друга: каждый — в своем контексте над общим индексом, итоги — в порядке объектов
    let _ = x.idx();
    let index = x.index.clone();
    let parts = crate::par::map(&objs, || (), |_, (class, name, base)| {
        let mut c = Ctx { dir, out: Import::default(), claimed: BTreeSet::new(), index: index.clone() };
        c.object(class, name, base, None);
        (c.out, c.claimed)
    });
    for (out, claimed) in parts {
        x.out.recs.extend(out.recs);
        x.out.refused.extend(out.refused);
        x.out.files += out.files;
        x.claimed.extend(claimed);
    }
    // файлы без загрузки — отказ с причиной, ничего не пропадает молча
    for f in &all {
        let in_scope = wanted(f) || root && (f.starts_with("Ext/") || f == "Configuration.xml" || f == "ConfigDumpInfo.xml");
        if in_scope && !x.claimed.contains(f) {
            x.out.refused.push((f.clone(), "загрузка этого вида файлов не поддержана".into()));
        }
    }
    Ok(x.out)
}

const CONFIG_CLASS: &str = "9cd510cd-abfc-11d4-9434-004095e12fc7";

/// Выгрузка внешней обработки задается своим `<Имя>.xml` (рядом — каталог `<Имя>`), как у `config import`
/// платформы; выгрузка конфигурации — каталогом.
pub fn is_external(path: &Path) -> bool {
    path.extension().is_some_and(|e| e.eq_ignore_ascii_case("xml"))
}

const EXTERNAL: &str = "ExternalDataProcessor";

/// Набор типов конфигурации → (класс части записи конфигурации, номер категории) — владелец типа в `copyinfo`;
/// снято с 32 сборок .epf платформой 8.5.1.1529 (QueryWizard — 10 наборов, StandardFindByReference — AnyIBRef).
const TYPE_SET_OWNERS: &[(&str, &str, u8)] = &[
    ("280f5f0e-9c8a-49cc-bf6d-4d296cc17a63", CONFIG_CLASS, 0), // AnyIBRef
    ("0a52f9de-73ea-4507-81e8-66217bead73a", CONFIG_CLASS, 1), // ExchangePlanRef
    ("e61ef7b8-f3e1-4f4b-8ac7-676e90524997", "9fcd25a0-4822-11d4-9414-008048da11f9", 0), // CatalogRef
    ("474c3bf6-08b5-4ddc-a2ad-989cedf11583", "9fcd25a0-4822-11d4-9414-008048da11f9", 1), // EnumRef
    ("38bfd075-3e63-4aaa-a93e-94521380d579", "9fcd25a0-4822-11d4-9414-008048da11f9", 4), // DocumentRef
    ("99892482-ed55-4fb5-a7f7-20888820a758", "9fcd25a0-4822-11d4-9414-008048da11f9", 5), // ChartOfCharacteristicTypesRef
    ("214fa4d8-6ba4-4748-a5e1-6332b5887780", "51f2d5d8-ea4d-4064-8892-82951750031e", 0), // BusinessProcessRef
    ("6291e9b3-8df5-44e1-b6b2-d9fe008016c0", "51f2d5d8-ea4d-4064-8892-82951750031e", 1), // TaskRef
    ("593cd424-0877-470d-91f9-b90a982059b4", "9de14907-ec23-4a07-96f0-85521cb6b53b", 0), // ChartOfCalculationTypesRef
    ("ac606d60-0209-4159-8e4c-794bc091ce38", "e3687481-0a87-462c-a166-9f34594f9bba", 0), // ChartOfAccountsRef
];

fn external_temp_name(uuid: &str) -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(0);
    // The format GUID can repeat within one clock second. Never reuse a path:
    // on Windows a deleted directory may still be pending deletion (os error 5).
    let sequence = NEXT.fetch_add(1, Ordering::Relaxed);
    format!("cfkit-new-{}-{sequence}-{uuid}", std::process::id())
}

#[cfg(test)]
mod temp_directory_tests {
    use super::external_temp_name;
    use std::collections::HashSet;

    #[test]
    fn temporary_names_stay_unique_with_identical_uuid() {
        // A fixed GUID models calls seeded by the same one-second clock tick.
        let barrier = std::sync::Barrier::new(32);
        let names = std::thread::scope(|scope| {
            let workers: Vec<_> = (0..32)
                .map(|_| {
                    let barrier = &barrier;
                    scope.spawn(move || {
                        barrier.wait();
                        (0..4)
                            .map(|_| external_temp_name("fixed-timer-guid"))
                            .collect::<Vec<_>>()
                    })
                })
                .collect();
            workers
                .into_iter()
                .flat_map(|worker| worker.join().unwrap())
                .collect::<Vec<_>>()
        });
        assert_eq!(names.len(), 128);
        assert_eq!(
            names.into_iter().collect::<HashSet<_>>().len(),
            128,
            "simultaneous calls must never reuse a temporary directory name"
        );
    }
}

/// Новая внешняя обработка `name` без реквизитов и форм: описание с новыми uuid собирается загрузкой.
pub fn new_external(name: &str) -> Result<Vec<u8>, String> {
    // Validate before UUID generation or filesystem access; names become XML and paths.
    let mut chars = name.chars();
    let valid = chars.next().is_some_and(|c| c == '_' || c.is_alphabetic())
        && chars.all(|c| c == '_' || c.is_alphanumeric());
    let upper = name.to_ascii_uppercase();
    let reserved = matches!(upper.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || ["COM", "LPT"].iter().any(|prefix| upper.strip_prefix(prefix)
            .is_some_and(|n| n.len() == 1 && n.as_bytes()[0].is_ascii_digit() && n != "0"));
    if !valid || reserved {
        return Err("имя обработки: ожидается идентификатор из букв, цифр и _, начинающийся с буквы или _; системные имена файлов запрещены".into());
    }
    let mut rng = Rng::new(ticks());
    let mut g = || rng.guid();
    let (uuid, object, type_id, value_id) = (g(), g(), g(), g());
    let ns = r#"xmlns="http://v8.1c.ru/8.3/MDClasses" xmlns:app="http://v8.1c.ru/8.2/managed-application/core" xmlns:cfg="http://v8.1c.ru/8.1/data/enterprise/current-config" xmlns:cmi="http://v8.1c.ru/8.2/managed-application/cmi" xmlns:ent="http://v8.1c.ru/8.1/data/enterprise" xmlns:lf="http://v8.1c.ru/8.2/managed-application/logform" xmlns:pal="http://v8.1c.ru/8.1/data/ui/colors/palette" xmlns:style="http://v8.1c.ru/8.1/data/ui/style" xmlns:sys="http://v8.1c.ru/8.1/data/ui/fonts/system" xmlns:v8="http://v8.1c.ru/8.1/data/core" xmlns:v8ui="http://v8.1c.ru/8.1/data/ui" xmlns:web="http://v8.1c.ru/8.1/data/ui/colors/web" xmlns:win="http://v8.1c.ru/8.1/data/ui/colors/windows" xmlns:xen="http://v8.1c.ru/8.3/xcf/enums" xmlns:xpr="http://v8.1c.ru/8.3/xcf/predef" xmlns:xr="http://v8.1c.ru/8.3/xcf/readable" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance""#;
    let class = crate::meta::EXTERNAL_DATA_PROCESSOR;
    let xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<MetaDataObject {ns} version="2.21">
<{EXTERNAL} uuid="{uuid}">
<InternalInfo>
<xr:ContainedObject>
<xr:ClassId>{class}</xr:ClassId>
<xr:ObjectId>{object}</xr:ObjectId>
</xr:ContainedObject>
<xr:GeneratedType name="ExternalDataProcessorObject.{name}" category="Object">
<xr:TypeId>{type_id}</xr:TypeId>
<xr:ValueId>{value_id}</xr:ValueId>
</xr:GeneratedType>
</InternalInfo>
<Properties>
<Name>{name}</Name>
<Synonym/>
<Comment/>
<DefaultForm/>
<AuxiliaryForm/>
</Properties>
<ChildObjects/>
</{EXTERNAL}>
</MetaDataObject>"#
    );
    // загрузка читает выгрузку с диска: описание — во временный каталог
    let dir = loop {
        let candidate = std::env::temp_dir().join(external_temp_name(&g()));
        match std::fs::create_dir(&candidate) {
            Ok(()) => break candidate,
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(format!("reserve external temp directory {}: {e}", candidate.display())),
        }
    };
    let path = dir.join(format!("{name}.xml"));
    let r = std::fs::write(&path, xml).map_err(|e| e.to_string()).and_then(|_| import(&path, None)).and_then(|imp| match imp.refused.first() {
        Some((f, why)) => Err(format!("{f}: {why}")),
        None => cf(&path, &imp),
    });
    std::fs::remove_dir_all(&dir).ok();
    r
}

/// `<Имя>.xml` внешней обработки → записи: описание корня (`<uuid>`), модуль и справка (`<ObjectId>.N`), формы, макеты.
fn external(path: &Path) -> Result<Import, String> {
    let dir = path.parent().ok_or("нет каталога выгрузки")?;
    let name = path.file_stem().and_then(|s| s.to_str()).ok_or("имя файла не в Юникоде")?.to_string();
    let rel = format!("{name}.xml");
    let mut all = vec![rel.clone()];
    if dir.join(&name).is_dir() {
        all.extend(walk(dir, &name)?);
    }
    let mut x = Ctx { dir, out: Import::default(), claimed: BTreeSet::new(), index: Some(Arc::new(index::Index::build_external(dir, EXTERNAL, &name))) };
    let doc = x.parse(&rel)?;
    let obj = doc.child(EXTERNAL).ok_or(format!("{rel}: нет <{EXTERNAL}> (внешние отчеты не поддержаны)"))?;
    let uuid = obj.peek_attr("uuid").ok_or(format!("{rel}: нет uuid"))?.to_string();
    let own = desc::contained_object(obj, crate::meta::EXTERNAL_DATA_PROCESSOR).map_err(|e| format!("{rel}: {e}"))?;
    x.ext(&format!("{name}/Ext"), EXTERNAL, &own);
    for (sub, kind) in [("Forms", "Form"), ("Templates", "Template")] {
        let names: Vec<String> = obj.peek_child("ChildObjects").map(|co| co.all(kind).filter_map(|e| e.peek_text()).collect()).unwrap_or_default();
        for n in names {
            x.nested(kind, &n, &format!("{name}/{sub}"), EXTERNAL);
        }
    }
    let r = x.descriptor(&doc, EXTERNAL, None, None, &uuid);
    x.take(&rel, r);
    for f in &all {
        if !x.claimed.contains(f) {
            x.out.refused.push((f.clone(), "загрузка этого вида файлов не поддержана".into()));
        }
    }
    Ok(x.out)
}

struct Ctx<'a> {
    dir: &'a Path,
    out: Import,
    claimed: BTreeSet<String>,
    /// Имена → uuid по всей выгрузке: строится при первом описании объекта.
    index: Option<Arc<index::Index>>,
}

impl Ctx<'_> {
    fn read(&self, rel: &str) -> Result<Vec<u8>, String> {
        std::fs::read(self.dir.join(rel)).map_err(|e| format!("{rel}: {e}"))
    }

    fn parse(&self, rel: &str) -> Result<El, String> {
        let b = self.read(rel)?;
        let t = std::str::from_utf8(&b).map_err(|_| format!("{rel}: не UTF-8"))?;
        dom::parse(t).map_err(|e| format!("{rel}: {e}"))
    }

    /// Разбор XML-файла и загрузка `f`; что-то в файле осталось непрочитанным — отказ.
    fn xml<T>(&self, rel: &str, f: impl FnOnce(&El) -> Result<T, String>) -> Result<T, String> {
        let d = self.parse(rel)?;
        let r = f(&d)?;
        match d.unread() {
            Some(u) => Err(format!("не разобрано: {u}")),
            None => Ok(r),
        }
    }

    /// Файл загружен в записи или отказ с причиной; файл отмечается обработанным в любом случае.
    fn take(&mut self, rel: &str, r: Result<Out, String>) {
        self.claimed.insert(rel.to_string());
        match r {
            Ok(recs) => {
                self.out.files += 1;
                for (name, kind, rec) in recs {
                    self.out.recs.insert(name, (kind, rec));
                }
            }
            Err(e) => self.out.refused.push((rel.to_string(), e)),
        }
    }

    /// `full` — полное имя вложенной подсистемы (`Subsystem.А.Subsystem.Б`); у объектов верхнего уровня `None`.
    fn object(&mut self, class: &str, name: &str, base: &str, full: Option<&str>) {
        let rel = format!("{base}/{name}.xml");
        let doc = match self.parse(&rel) {
            Ok(d) => d,
            Err(e) => return self.take(&rel, Err(e)),
        };
        let Some(obj) = doc.child(class) else { return self.take(&rel, Err(format!("нет элемента <{class}>"))) };
        let Ok(id) = obj.req_attr("uuid").map(str::to_string) else { return self.take(&rel, Err("нет uuid".into())) };
        let own = format!("{base}/{name}");
        self.ext(&format!("{own}/Ext"), class, &id);
        // команды — в описании владельца: uuid по имени
        if let Some(co) = obj.child("ChildObjects") {
            for c in co.all("Command") {
                let cname = c.child("Properties").and_then(|p| p.child("Name")).and_then(|n| n.text().ok());
                if let (Some(cname), Some(cid)) = (cname, c.attr("uuid")) {
                    self.ext(&format!("{own}/Commands/{cname}/Ext"), "Command", cid);
                }
            }
        }
        for (dir, kind) in [("Forms", "Form"), ("Templates", "Template"), ("Subsystems", "Subsystem")] {
            let names: Vec<String> = match obj.child("ChildObjects") {
                Some(co) => co.all(kind).filter_map(|e| e.text().ok()).collect(),
                None => Vec::new(),
            };
            for n in names {
                let sub = format!("{own}/{dir}");
                if kind == "Subsystem" {
                    let me = full.map_or_else(|| format!("{class}.{name}"), str::to_string);
                    self.object("Subsystem", &n, &sub, Some(&format!("{me}.Subsystem.{n}")));
                } else {
                    self.nested(kind, &n, &sub, class);
                }
            }
        }
        let r = self.descriptor(&doc, class, None, full, &id);
        self.take(&rel, r);
    }

    /// Индекс имен: строится при первом обращении.
    fn idx(&mut self) -> Result<&index::Index, String> {
        if self.index.is_none() {
            let conf = self.parse("Configuration.xml")?;
            self.index = Some(Arc::new(index::Index::build(self.dir, &conf)?));
        }
        Ok(self.index.as_deref().unwrap())
    }

    fn descriptor(&mut self, doc: &El, class: &str, owner: Option<&str>, full: Option<&str>, id: &str) -> Result<Out, String> {
        let x = self.idx()?;
        match desc::record(doc, class, owner, full, x)? {
            Some(b) => Ok(vec![(id.to_string(), format!("описание: {class}"), Rec::Raw(b))]),
            None => Err(format!("описание {class}: загрузка не поддержана")),
        }
    }

    /// Форма или макет: свое описание `<Имя>.xml` и каталог `<Имя>/Ext`.
    fn nested(&mut self, kind: &str, name: &str, base: &str, owner: &str) {
        let rel = format!("{base}/{name}.xml");
        let doc = match self.parse(&rel) {
            Ok(d) => d,
            Err(e) => return self.take(&rel, Err(e)),
        };
        let Some(id) = doc.child(kind).and_then(|o| o.attr("uuid").map(str::to_string)) else { return self.take(&rel, Err(format!("нет uuid <{kind}>"))) };
        self.ext(&format!("{base}/{name}/Ext"), kind, &id);
        let r = self.descriptor(&doc, kind, Some(owner), None, &id);
        self.take(&rel, r);
    }

    /// Файлы `Ext` объекта: модули; остальное останется необработанным (отказ в конце загрузки).
    fn ext(&mut self, ext: &str, class: &str, id: &str) {
        let Ok(rd) = std::fs::read_dir(self.dir.join(ext)) else { return };
        let mut files: Vec<String> = rd.filter_map(|e| e.ok()).filter(|e| e.path().is_file()).filter_map(|e| e.file_name().into_string().ok()).collect();
        files.sort();
        for f in files {
            let rel = format!("{ext}/{f}");
            let Some((role, kind)) = f.rsplit_once('.') else { continue };
            let Some(suffix) = suffix_of(class, role) else { continue };
            let name = format!("{id}.{suffix}");
            let one = |r: Result<Rec, String>, k: &str| r.map(|rec| vec![(name.clone(), k.to_string(), rec)]);
            let r = match (role, kind) {
                (r, "bsl") => one(self.read(&rel).and_then(|b| module::record(&b)), module_kind(r)),
                // вложенный контейнер — байт в байт: страницы платформа не перепаковывает (у упаковщика — по 512, круг БП)
                (r, "bin") if r.ends_with("Module") => one(self.read(&rel).and_then(|b| payload::container(&b).map(|_| Rec::Raw(b))), "модуль без исходного текста"),
                // обычная форма: файл — сам контейнер записи (form, module)
                ("Form", "bin") => one(self.read(&rel).and_then(|b| payload::container(&b).map(|_| Rec::Raw(b))), "обычная форма"),
                ("Package" | "ParentConfigurations" | "MobileClientSignature", "bin") => one(self.read(&rel).map(Rec::Raw), "двоичный файл как есть"),
                ("Template", "bin") => one(self.read(&rel).and_then(|b| payload::binary(&b)), "макет: двоичные данные"),
                ("Template", "txt") => one(self.read(&rel).and_then(|b| payload::text(&b)), "макет: текст"),
                ("Schedule", "xml") => one(self.xml(&rel, ext::schedule), "расписание"),
                ("AdditionalIndexes","xml") => {
                    let r=self.idx().map(|_|()).and_then(|_|self.xml(&rel,|d|super::indexes::import(d,self.index.as_deref().unwrap()))).map(Rec::Raw);
                    one(r,"дополнительные индексы")
                },
                ("Style", "xml") => {
                    let r = self.idx().map(|_| ()).and_then(|_| {
                        let ix = self.index.as_deref().unwrap();
                        self.xml(&rel, |d| ext::style(d, ix))
                    });
                    one(r, "стиль")
                }
                ("CommandInterface" | "MainSectionCommandInterface", "xml") => {
                    let r = self.idx().map(|_| ()).and_then(|_| {
                        let ix = self.index.as_deref().unwrap();
                        self.xml(&rel, |d| ext::command_interface(d, ix))
                    });
                    one(r, "командный интерфейс")
                }
                ("HomePageWorkArea", "xml") => {
                    let r = self.idx().map(|_| ()).and_then(|_| {
                        let ix = self.index.as_deref().unwrap();
                        self.xml(&rel, |d| ext::home_page(d, ix))
                    });
                    one(r, "начальная страница и интерфейс клиента")
                }
                ("StandaloneConfigurationContent", "bin") => {
                    let r = self.idx().map(|_| ()).and_then(|_| {
                        let ix = self.index.as_deref().unwrap();
                        self.xml(&rel, |d| ext::standalone(d, ix))
                    });
                    one(r, "состав автономной конфигурации")
                }
                ("ClientApplicationInterface", "xml") => one(self.xml(&rel, ext::client_interface), "начальная страница и интерфейс клиента"),
                // WS-ссылка: контейнер `0.wsdl` (файл без BOM) и схемы `1.xsd`… как есть
                ("WSDefinition", "xml") => {
                    let r = self.read(&rel).and_then(|b| {
                        let wsdl = b.strip_prefix(BOM).ok_or("WSDefinition.xml без BOM")?.to_vec();
                        let mut v = vec![("0.wsdl".to_string(), wsdl)];
                        for k in 1.. {
                            let x = format!("{ext}/{k}.xsd");
                            if !self.dir.join(&x).is_file() {
                                break;
                            }
                            v.push((format!("{k}.xsd"), self.read(&x)?));
                            self.claimed.insert(x);
                        }
                        Ok(Rec::Dir(v))
                    });
                    one(r, "WS-ссылка: описание")
                }
                // управляемая форма: Form.xml и модуль Form/Module.bsl — одна запись
                ("Form", "xml") => {
                    let mrel = format!("{ext}/Form/Module.bsl");
                    let module = std::fs::read(self.dir.join(&mrel)).ok();
                    let items = self.dir.join(ext).join("Form/Items");
                    let pics = |f: &str| std::fs::read(items.join(f)).map_err(|e| format!("Form/Items/{f}: {e}"));
                    let r = self.idx().map(|_| ()).and_then(|_| {
                        let ix = self.index.as_deref().unwrap();
                        self.xml(&rel, |d| form::record(d, module.as_deref(), ix, &rel, &pics))
                    });
                    if r.is_ok() && module.is_some() {
                        self.claimed.insert(mrel);
                    }
                    let r = r.map(|(rec, used)| {
                        self.claimed.extend(used.into_iter().map(|u| format!("{ext}/Form/Items/{u}")));
                        rec
                    });
                    one(r, "форма")
                }
                ("Flowchart", "xml") => {
                    let r = self.idx().map(|_| ()).and_then(|_| {
                        let ix = self.index.as_deref().unwrap();
                        self.xml(&rel, |d| flowchart::record(d, ix, &|n, _| Err(format!("картинка элемента {n} карты маршрута: не встречена"))).map(|r| r.0))
                    });
                    one(r, "карта маршрута")
                }
                ("Predefined", "xml") => {
                    let owner = ext.strip_suffix("/Ext").map(|o| format!("{o}.xml")).unwrap_or_default();
                    let r = self.idx().map(|_| ()).and_then(|_| {
                        let od = self.parse(&owner)?;
                        let props = od.child(class).ok_or(format!("владелец: нет <{class}>"))?;
                        let ix = self.index.as_deref().unwrap();
                        self.xml(&rel, |d| match class {
                            "Catalog" => predef::catalog(d, props),
                            "ChartOfCharacteristicTypes" => predef::characteristic_types(d, props, ix),
                            "ChartOfAccounts" => predef::accounts(d, props, ix),
                            "ChartOfCalculationTypes" => predef::calc_types(d, props),
                            _ => Err(format!("предопределенные {class}: загрузка не поддержана")),
                        })
                    });
                    one(r, "предопределенные")
                }
                ("Rights", "xml") => {
                    let r = self.idx().map(|_| ()).and_then(|_| {
                        let ix = self.index.as_deref().unwrap();
                        self.xml(&rel, |d| rights::record(d, ix))
                    });
                    one(r, "права роли")
                }
                ("Content", "xml") => {
                    let r = self.idx().map(|_| ()).and_then(|_| {
                        let ix = self.index.as_deref().unwrap();
                        self.xml(&rel, |d| ext::content(d, ix))
                    });
                    one(r, "состав плана обмена")
                }
                // справка и HTML-макет (Template.xml с корнем <Help>); макеты других видов — дальше
                ("Help", "xml") | ("Template", "xml") => {
                    let doc = match self.parse(&rel) {
                        Ok(d) => d,
                        Err(e) => {
                            self.take(&rel, Err(e));
                            continue;
                        }
                    };
                    if doc.name == "DataCompositionSchema" {
                        doc.touch_all();
                        // встроенная картинка — `Template/value.<расширение>` (выгрузка `dcs::schema_files`)
                        let pic = std::fs::read_dir(self.dir.join(format!("{ext}/Template"))).ok().and_then(|d| {
                            d.filter_map(|e| e.ok()?.file_name().into_string().ok()).find(|n| n.starts_with("value."))
                        });
                        let svg_rel = format!("{ext}/Template/{}", pic.as_deref().unwrap_or("value.svg"));
                        let svg = std::fs::read(self.dir.join(&svg_rel)).ok();
                        let r = self.read(&rel).and_then(|b| String::from_utf8(b).map_err(|_| "не UTF-8".to_string())).and_then(|t| {
                            let x = self.idx()?;
                            dcs::record(&t, x, svg.as_deref())
                        });
                        let r = r.map(|(rec, used)| {
                            if used {
                                self.claimed.insert(svg_rel);
                            }
                            rec
                        });
                        self.take(&rel, one(r, "макет: схема компоновки данных"));
                        continue;
                    }
                    // макет оформления компоновки: платформа хранит файл как есть (корень — как у выгрузки)
                    if doc.name == "AppearanceTemplate" {
                        doc.touch_all();
                        const ROOT: &str = "\u{feff}<?xml version=\"1.0\" encoding=\"UTF-8\"?>\r\n<AppearanceTemplate xmlns=\"http://v8.1c.ru/8.1/data-composition-system/appearance-template\" xmlns:dcscor=\"http://v8.1c.ru/8.1/data-composition-system/core\" xmlns:pal=\"http://v8.1c.ru/8.1/data/ui/colors/palette\" xmlns:style=\"http://v8.1c.ru/8.1/data/ui/style\" xmlns:sys=\"http://v8.1c.ru/8.1/data/ui/fonts/system\" xmlns:v8ui=\"http://v8.1c.ru/8.1/data/ui\" xmlns:web=\"http://v8.1c.ru/8.1/data/ui/colors/web\" xmlns:win=\"http://v8.1c.ru/8.1/data/ui/colors/windows\" xmlns:xs=\"http://www.w3.org/2001/XMLSchema\" xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\">";
                        let r = self.read(&rel).and_then(|b| if b.starts_with(ROOT.as_bytes()) { Ok(Rec::Raw(b)) } else { Err("макет оформления: корень не как у выгрузки".into()) });
                        self.take(&rel, one(r, "макет: оформление компоновки"));
                        continue;
                    }
                    if doc.name == "document" {
                        let r = self.idx().and_then(|x| mxl::record(&doc, x)).and_then(|r| match doc.unread() {
                            Some(u) => Err(format!("не разобрано: {u}")),
                            None => Ok(r),
                        });
                        self.take(&rel, one(r, "макет: табличный документ"));
                        continue;
                    }
                    if doc.name == "GraphicalSchema" {
                        let items = self.dir.join(ext).join("Template/Items");
                        let pics = |n: &str, f: &str| std::fs::read(items.join(n).join(f)).map_err(|e| format!("Template/Items/{n}/{f}: {e}"));
                        let r = self.idx().and_then(|x| flowchart::record(&doc, x, &pics)).and_then(|r| match doc.unread() {
                            Some(u) => Err(format!("не разобрано: {u}")),
                            None => Ok(r),
                        });
                        let r = match r {
                            Ok((rec, used)) => {
                                for u in used {
                                    self.claimed.insert(format!("{ext}/Template/Items/{u}"));
                                }
                                one(Ok(rec), "макет: графическая схема")
                            }
                            Err(e) => Err(e),
                        };
                        self.take(&rel, r);
                        continue;
                    }
                    if doc.name != "Help" {
                        continue;
                    }
                    let (sub, guid, k) = if role == "Help" { ("Help", html::HELP_FILES, "справка") } else { ("Template", html::TEMPLATE_FILES, "макет: HTML") };
                    let dir = self.dir.join(ext).join(sub);
                    let r = self.idx().and_then(|x| html::record(&doc, x, guid, &dir)).and_then(|(rec, used)| match doc.unread() {
                        Some(u) => Err(format!("не разобрано: {u}")),
                        None => Ok((rec, used)),
                    });
                    match r {
                        Ok((rec, used)) => {
                            for u in used {
                                self.claimed.insert(format!("{ext}/{sub}/{u}"));
                            }
                            one(Ok(rec), k)
                        }
                        Err(e) => Err(e),
                    }
                }
                // логотип (роль 1 контейнера конфигурации): загрузка платформы у mobileapp пишет `<ObjectId>.1` как картинку,
                // у каркаса (без MobilePlatformApplication) не сохраняет — от чего зависит, в коде не найдено: отказ всегда
                ("Logo", "xml") => Err("логотип конфигурации: когда загрузка платформы его сохраняет, не выведено из кода".into()),
                ("Picture" | "Splash" | "MainSectionPicture", "xml") => {
                    let dir = self.dir.join(ext);
                    let read = |p: &str| std::fs::read(dir.join(p)).map_err(|e| format!("{p}: {e}"));
                    match self.xml(&rel, |d| payload::picture(d, role, &read)) {
                        Ok((rec, file)) => {
                            self.claimed.insert(format!("{ext}/{file}"));
                            one(Ok(rec), "картинка")
                        }
                        Err(e) => Err(e),
                    }
                }
                _ => continue,
            };
            self.take(&rel, r);
        }
    }
}

/// Суффикс записи по роли файла (таблица выгрузки [`own_role`] в обратную сторону).
fn suffix_of(class: &str, role: &str) -> Option<&'static str> {
    const SUFFIXES: &[&str] = &["0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "14", "15", "16", "1c", "a", "b", "c", "f"];
    SUFFIXES.iter().copied().find(|s| own_role(class, s) == Some(role))
}

fn module_kind(role: &str) -> &'static str {
    match role {
        "Module" => "модуль: Module",
        "ObjectModule" => "модуль: ObjectModule",
        "ManagerModule" => "модуль: ManagerModule",
        "RecordSetModule" => "модуль: RecordSetModule",
        "ValueManagerModule" => "модуль: ValueManagerModule",
        "CommandModule" => "модуль: CommandModule",
        _ => "модуль: модули конфигурации",
    }
}

/// Все файлы подкаталога `sub` (пусто — всего каталога): пути относительно `dir` через `/`.
fn walk(dir: &Path, sub: &str) -> Result<Vec<String>, String> {
    fn go(root: &Path, d: &Path, out: &mut Vec<String>) -> Result<(), String> {
        for e in std::fs::read_dir(d).map_err(|e| format!("{}: {e}", d.display()))? {
            let p = e.map_err(|e| e.to_string())?.path();
            if p.is_dir() {
                go(root, &p, out)?;
            } else {
                out.push(p.strip_prefix(root).unwrap().to_string_lossy().replace('\\', "/"));
            }
        }
        Ok(())
    }
    let mut out = Vec::new();
    go(dir, &dir.join(sub), &mut out)?;
    out.sort();
    Ok(out)
}

/// Первая часть .cf из `config save` — 32-битный контейнер (записи `30ffe4cc-…`, `ae3cd3da-…`, `root`, `version`,
/// `versions`, время 2019 год), побайтно одна и та же у всех загрузок (БП 3.0.171, 8.5, каркас: md5 5f1c12f3…). Пересжатая
/// заново (те же записи, другие байты deflate) платформа ее с частью 1 не читает — загружает пустую конфигурацию части 0.
const PART0: &[u8] = include_bytes!("part0/part0.cf");

/// Файл .cf: постоянная первая часть и записи загрузки с `root`, `version`, `versions`, `deleted`.
/// `versions` — новые случайные GUID на каждую запись, как у платформы при каждой загрузке.
pub fn cf(dir: &Path, imp: &Import) -> Result<Vec<u8>, String> {
    use crate::container::{deflate, write, Entry};
    if is_external(dir) {
        return epf(dir, imp);
    }
    let conf = dom::parse(&String::from_utf8_lossy(&std::fs::read(dir.join("Configuration.xml")).map_err(|e| e.to_string())?))?;
    if conf.req("Configuration")?.peek_child("Properties").is_some_and(|p|p.peek_child("ConfigurationExtensionPurpose").is_some()) {
        return extension::cfe(&conf,imp);
    }
    let root_id = conf.req("Configuration")?.req_attr("uuid")?.to_string();
    if !imp.recs.contains_key(&root_id) {
        return Err(format!("нет записи конфигурации {root_id}"));
    }
    let now = ticks();
    let entry = |name: &str, data: Vec<u8>, t: u64| Entry { name: name.into(), created: t, modified: t, data: deflate(&data) };
    let mut rng = Rng::new(now);
    let mut versions = format!("{{1,{},\"\",{}", imp.recs.len() + 1, rng.guid());
    let mut p1 = Vec::new();
    for (name, (_, rec)) in &imp.recs {
        versions.push_str(&format!(",\"{name}\",{}", rng.guid()));
        let data = match rec {
            Rec::Raw(b) => b.clone(),
            Rec::Dir(v) => write(&v.iter().map(|(n, d)| Entry { name: n.clone(), created: now, modified: now, data: d.clone() }).collect::<Vec<_>>()),
        };
        p1.push(entry(name, data, now));
    }
    versions.push('}');
    let bom = |s: &str| [BOM, s.as_bytes()].concat();
    p1.push(entry("deleted", bom("0"), now));
    p1.push(entry("root", bom(&format!("{{2,{root_id},}}")), now));
    // запись version `{216,0,{N,0}}`: backend FUN_18031c320 → FUN_180270c10 — пара DAT_1819c5598/559c, N (+0x28 версии
    // хранилища, FUN_180325230), число и список GUID (пустой). shortcut: N = режим совместимости по загрузкам платформы
    // (8.3.16, 8.3.24, 8.3.27 — БП, ЭДО, ERP; 8.5.1 — каркас), откуда +0x28 — не прослежено; пересмотреть при расхождении
    let compat = index::Index::build(dir, &conf)?.compat.ok_or("запись version: режим совместимости не задан — N не выведено")?;
    p1.push(entry("version", bom(&format!("{{\r\n{{216,0,\r\n{{{compat},0}}\r\n}}\r\n}}")), now));
    p1.push(entry("versions", bom(&versions), now));
    p1.sort_by(|a, b| a.name.as_bytes().cmp(b.name.as_bytes()));
    // часть 1 у платформы всегда 64-битная (каркас и БП): 32-битную на ее месте платформа не читает (`write_as`)
    Ok([PART0, &crate::container::write_as(&p1, crate::container::Width::W64)].concat())
}

/// Файл .epf, как у `config import --out` платформы: одна 32-битная часть, записи загрузки и `copyinfo`, `root`,
/// `version`, `versions` (в `versions` — все записи файла, и сама она), по имени.
fn epf(path: &Path, imp: &Import) -> Result<Vec<u8>, String> {
    use crate::container::{deflate, write, write_as, Entry, Width};
    let dir = path.parent().ok_or("нет каталога выгрузки")?;
    let name = path.file_stem().and_then(|s| s.to_str()).ok_or("имя файла не в Юникоде")?;
    let doc = dom::parse(&String::from_utf8_lossy(&std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?))?;
    let obj = doc.req(EXTERNAL)?;
    let uuid = obj.req_attr("uuid")?.to_string();
    if !imp.recs.contains_key(&uuid) {
        return Err(format!("нет записи внешней обработки {uuid}"));
    }
    let own = desc::contained_object(obj, crate::meta::EXTERNAL_DATA_PROCESSOR)?;
    let x = index::Index::build_external(dir, EXTERNAL, name);
    let type_id = &x.types.get(&format!("{EXTERNAL}Object.{name}")).ok_or("нет GeneratedType Object")?.0;
    // copyinfo — объекты и типы, на которые ссылаются записи: основная и дополнительная формы (из записи корня), сам
    // объект с его типом — когда тип встречается в других записях (реквизит формы «Объект»), наборы типов конфигурации
    let refs = |pat: &str, skip: &str| imp.recs.iter().any(|(n, (_, r))| n != skip && matches!(r, Rec::Raw(b) if b.windows(pat.len()).any(|w| w == pat.as_bytes())));
    let props = obj.req("Properties")?;
    let mut objs = Vec::new();
    let mut types = Vec::new();
    if refs(&format!("{{\"#\",{type_id}}}"), &uuid) {
        objs.push((own.clone(), crate::meta::EXTERNAL_DATA_PROCESSOR, name.to_string()));
        types.push((type_id.clone(), own.clone(), 0));
    }
    for tag in ["DefaultForm", "AuxiliaryForm"] {
        let f = props.peek_child(tag).and_then(El::peek_text).unwrap_or_default();
        if let Some((_, form)) = f.rsplit_once(".Form.") {
            let id = x.full.get(&f).ok_or(format!("<{tag}>: нет формы {f}"))?;
            objs.push((id.clone(), "d5b0e5ed-256d-401c-9c36-f630cafd8a62", form.to_string()));
        }
    }
    // наборы типов конфигурации (`cfg:CatalogRef`…) в записях: их владельцы — части записи конфигурации, у файла
    // без конфигурации платформа дает частям новые случайные ObjectId при каждой сборке
    let now = ticks();
    let mut rng = Rng::new(now);
    let mut parts: Vec<(&str, String)> = Vec::new();
    for (set, class, cat) in TYPE_SET_OWNERS {
        if !refs(&format!("{{\"#\",{set}}}"), "") {
            continue;
        }
        let part = match parts.iter().find(|(c, _)| c == class) {
            Some((_, id)) => id.clone(),
            None => {
                parts.push((class, rng.guid()));
                parts.last().unwrap().1.clone()
            }
        };
        types.push((set.to_string(), part, *cat));
    }
    for (class, id) in &parts {
        let n = if *class == CONFIG_CLASS { "Конфигурация" } else { "" };
        objs.push((id.clone(), class, n.to_string()));
    }
    objs.sort();
    objs.dedup();
    types.sort();
    // раздел `{N,элемент…}`; пустой — `{0}`
    let section = |items: Vec<String>| if items.is_empty() { "{0}".to_string() } else { format!("{{{},{}\r\n}}", items.len(), items.join(",")) };
    let s1 = section(objs.iter().map(|(id, class, n)| format!("\r\n{{{id},{id},1,\r\n{{{class},\"{n}\"}}\r\n}}")).collect());
    let s2 = section(types.iter().map(|(t, owner, cat)| format!("\r\n{{{t},{owner},{cat}}}")).collect());
    // shortcut: третий раздел (у ТаблицаЗначений/ДеревоЗначений — `{k},<тип>,{k}`, смысл k не выведен) пишется пустым;
    // платформа такой файл читает и выгружает так же — влияние на клиент проверить при отладке (шаг 6)
    let copyinfo = format!("{{4,\r\n{s1},\r\n{s2},\r\n{{0}},\r\n{{0,0}},\r\n{{0}}\r\n}}");
    let bom = |s: &str| [BOM, s.as_bytes()].concat();
    let mut recs: Vec<(String, Vec<u8>)> = imp
        .recs
        .iter()
        .map(|(n, (_, rec))| {
            let data = match rec {
                Rec::Raw(b) => b.clone(),
                Rec::Dir(v) => write(&v.iter().map(|(n, d)| Entry { name: n.clone(), created: now, modified: now, data: d.clone() }).collect::<Vec<_>>()),
            };
            (n.clone(), data)
        })
        .collect();
    recs.push(("copyinfo".into(), bom(&copyinfo)));
    recs.push(("root".into(), bom(&format!("{{2,{uuid},}}"))));
    // shortcut: режима совместимости у .epf нет — платформа 8.5.1 пишет 80501; другие версии платформы не проверены
    recs.push(("version".into(), bom("{\r\n{216,0,\r\n{80501,0}\r\n}\r\n}")));
    let mut names: Vec<String> = recs.iter().map(|(n, _)| n.clone()).chain(["versions".to_string()]).collect();
    names.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));
    let mut versions = format!("{{1,{},\"\",{}", names.len() + 1, rng.guid());
    for n in &names {
        versions.push_str(&format!(",\"{n}\",{}", rng.guid()));
    }
    versions.push('}');
    recs.push(("versions".into(), bom(&versions)));
    recs.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));
    let entries: Vec<Entry> = recs.into_iter().map(|(n, d)| Entry { name: n, created: now, modified: now, data: deflate(&d) }).collect();
    Ok(write_as(&entries, Width::W32))
}

/// Время записи контейнера: десятитысячные доли секунды от 01.01.0001.
fn ticks() -> u64 {
    let unix = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    (unix + 62_135_596_800) * 10_000
}

/// xorshift64*: GUID версий должны только различаться, криптостойкость не нужна.
struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Rng {
        Rng(seed ^ 0x9e37_79b9_7f4a_7c15 ^ (std::process::id() as u64) << 32)
    }
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_f491_4f6c_dd1d)
    }
    /// GUID версии 4.
    fn guid(&mut self) -> String {
        let (a, b) = (self.next(), self.next());
        let b = (b & 0x3fff_ffff_ffff_ffff) | 0x8000_0000_0000_0000;
        let a = (a & !0xf000) | 0x4000;
        format!("{:08x}-{:04x}-{:04x}-{:04x}-{:012x}", a >> 32, (a >> 16) & 0xffff, a & 0xffff, b >> 48, b & 0xffff_ffff_ffff)
    }
}
