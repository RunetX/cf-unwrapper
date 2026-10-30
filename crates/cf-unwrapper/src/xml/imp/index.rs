//! Имена → uuid по всей выгрузке (один проход по описаниям объектов): ссылки в XML — полные имена
//! (`Catalog.Валюты.Form.ФормаЭлемента`), в записях — uuid; типы `cfg:CatalogRef.Валюты` → TypeId.

use super::dom::{self, El};
use std::collections::HashMap;
use std::path::Path;

#[derive(Default)]
pub struct Index {
    /// `Catalog.Валюты`, `Catalog.Валюты.Attribute.Курс`, `Catalog.Валюты.Form.ФормаЭлемента` → uuid
    pub full: HashMap<String, String>,
    /// `CatalogRef.Валюты` → (TypeId, ValueId) из GeneratedType
    pub types: HashMap<String, (String, String)>,
    /// Предопределенные `Catalog.Валюты.Рубль` → id (из `Ext/Predefined.xml`)
    pub predefined: HashMap<String, String>,
    /// Типы `<v8:Type>` реквизитов, измерений, ресурсов: `Catalog.Банки.Attribute.Город` → [`cfg:CatalogRef.Города`]
    pub attr_types: HashMap<String, Vec<String>>,
    /// Наборы типов `<v8:TypeSet>` тех же реквизитов (`cfg:DefinedType.X`, `cfg:CatalogRef`…), если есть
    pub type_sets: HashMap<String, Vec<String>>,
    /// uuid и имя конфигурации (права роли ссылаются на `Configuration.<Имя>` — запись по uuid корня).
    pub root: (String, String),
    /// Ведущие измерения регистра сведений по порядку: `InformationRegister.X` → имена (команды `OpenByValue.<измерение>`).
    pub masters: HashMap<String, Vec<String>>,
    /// Объекты верхнего уровня: `Catalog.Банки` → свойства и подчиненные по порядку (поля основной таблицы
    /// динамического списка).
    pub objects: HashMap<String, Obj>,
    /// Общие реквизиты: имя, AutoUse и состав (`Catalog.Банки` → `Use`/`DontUse`/`Auto`).
    pub common: Vec<(String, String, HashMap<String, String>)>,
    /// Режим совместимости `a·10000 + b·100 + c` (`Version8_3_24` → 80324; так его сравнивает код платформы:
    /// 0x139c1 = 8.3.21); `None` — нет или `DontUse`.
    pub compat: Option<u32>,
    /// Имена (в нижнем регистре) реквизитов, измерений, ресурсов с `<PasswordMode>true` — у любого объекта.
    pub password: std::collections::HashSet<String>,
    /// Те же элементы полными путями (`ExchangePlan.X.Attribute.Пароль`).
    pub password_full: std::collections::HashSet<String>,
    /// Графы журнала: `DocumentJournal.X.Column.Y` → реквизиты документов `<References>` по порядку.
    pub column_refs: HashMap<String, Vec<String>>,
    /// Измерения и ресурсы с `<Balance>false</Balance>` (регистр бухгалтерии): полные имена.
    pub unbalanced: std::collections::HashSet<String>,
    /// Измерения с `<MainFilter>true</MainFilter>` (основной отбор): полные имена.
    pub main_filter: std::collections::HashSet<String>,
}

/// Объект метаданных для полей динамического списка.
#[derive(Default)]
pub struct Obj {
    /// простые свойства `<Properties>` (текст)
    pub props: HashMap<String, String>,
    /// подчиненные верхнего уровня по порядку: (вид, имя) — реквизиты, измерения, ресурсы, табличные части, графы…
    pub kids: Vec<(String, String)>,
    /// есть владельцы (`<Owners>` не пуст)
    pub owners: bool,
    /// владельцы справочника по порядку (`Catalog.Номенклатура`) — тип стандартного реквизита Owner
    pub owner_list: Vec<String>,
}

/// Режим совместимости `Version8_3_24` → 80324; `DontUse` и незнакомое — `None`.
pub fn compat_of(t: &str) -> Option<u32> {
    let v: Vec<u32> = t.strip_prefix("Version")?.split('_').map(|p| p.parse().ok()).collect::<Option<_>>()?;
    match v[..] {
        [a, b, c] if b < 100 && c < 100 => a.checked_mul(10000)?.checked_add(b * 100 + c),
        _ => None,
    }
}

impl Index {
    pub fn build(dir: &Path, conf: &El) -> Result<Index, String> {
        Self::build_from(conf, &|rel| std::fs::read(dir.join(rel)).ok())
    }

    /// То же по файлам из `read` (путь от корня выгрузки через `/`): выгрузка в памяти `xml::index_files`.
    pub fn build_from(conf: &El, read: &dyn Fn(&str) -> Option<Vec<u8>>) -> Result<Index, String> {
        let mut x = Index::default();
        let cobj = conf.child("Configuration").ok_or("Configuration.xml: нет <Configuration>")?;
        // uuid конфигурации: ссылки `urn:form:md:…:<uuid>` в словаре формы отчета
        if let Some(id) = cobj.peek_attr("uuid") {
            x.full.insert("Configuration".into(), id.to_string());
        }
        let name = cobj.child("Properties").and_then(|p| p.child("Name")).and_then(|n| n.text().ok());
        x.root = (cobj.attr("uuid").unwrap_or_default().to_string(), name.unwrap_or_default());
        x.compat = cobj.peek_child("Properties").and_then(|p| p.peek_child("CompatibilityMode")).and_then(El::peek_text).and_then(|t| compat_of(&t));
        let co = cobj.child("ChildObjects").ok_or("Configuration.xml: нет <ChildObjects>")?;
        for ch in co.elems() {
            let name = ch.text()?;
            let base = super::super::plural(&ch.name);
            x.object(read, &ch.name, &name, &base, &ch.name.clone());
        }
        Ok(x)
    }

    /// Индекс выгрузки внешней обработки: `<Имя>.xml` и каталог `<Имя>` в `dir`.
    pub fn build_external(dir: &Path, class: &str, name: &str) -> Index {
        Self::build_external_from(&|rel| std::fs::read(dir.join(rel)).ok(), class, name)
    }

    /// То же по файлам из `read` (путь от каталога выгрузки, `./<Имя>.xml`): выгрузка в памяти `xml::index_files`.
    pub fn build_external_from(read: &dyn Fn(&str) -> Option<Vec<u8>>, class: &str, name: &str) -> Index {
        let mut x = Index::default();
        // база «.»: пути только для чтения файлов (`./<Имя>.xml`, `./<Имя>/Forms/…`)
        x.object(read, class, name, ".", class);
        x.root = (x.full.get(&format!("{class}.{name}")).cloned().unwrap_or_default(), name.to_string());
        x
    }

    /// Описание объекта; нечитаемое пропускается (сам файл отвергнет загрузка объекта).
    fn object(&mut self, read_bytes: &dyn Fn(&str) -> Option<Vec<u8>>, class: &str, name: &str, base: &str, full: &str) {
        let full = if full == class { format!("{class}.{name}") } else { full.to_string() };
        let read = |rel: &str| read_bytes(rel).and_then(|b| dom::parse(std::str::from_utf8(&b).ok()?).ok());
        let Some(d) = read(&format!("{base}/{name}.xml")) else { return };
        let Some(obj) = d.child(class) else { return };
        if let Some(id) = obj.attr("uuid") {
            self.full.insert(full.clone(), id.to_string());
        }
        // тип константы (для путей `НаборКонстант.<константа>.…`)
        if let Some(t) = obj.child("Properties").and_then(|p| p.child("Type")) {
            self.attr_types.insert(full.clone(), t.all("v8:Type").filter_map(|c| c.text().ok()).collect());
            self.type_sets(&full, t);
        }
        if let Some(ii) = obj.child("InternalInfo") {
            for g in ii.all("xr:GeneratedType") {
                if let (Some(n), Some(t), Some(v)) = (g.attr("name"), g.child("xr:TypeId").and_then(|e| e.text().ok()), g.child("xr:ValueId").and_then(|e| e.text().ok())) {
                    self.types.insert(n.to_string(), (t, v));
                }
            }
        }
        if full == format!("{class}.{name}") {
            self.object_info(class, obj, &full);
        }
        let own = format!("{base}/{name}");
        if let Some(p) = read(&format!("{own}/Ext/Predefined.xml")) {
            self.predefined_items(&p, &full);
        }
        // состав плана обмена (объекты по порядку, через перевод строки): таблицы изменений в запросе — dlist
        if class == "ExchangePlan" {
            if let (Some(c), Some(o)) = (read(&format!("{own}/Ext/Content.xml")), self.objects.get_mut(&full)) {
                o.props.insert("Content".into(), c.all("Item").filter_map(|i| i.child("Metadata")?.text().ok()).collect::<Vec<_>>().join("\n"));
            }
        }
        if let Some(co) = obj.child("ChildObjects") {
            self.children(co, &full);
            for e in co.elems() {
                let Ok(n) = e.text() else { continue };
                let sub = match e.name.as_str() {
                    "Form" => "Forms",
                    "Template" => "Templates",
                    "Subsystem" => "Subsystems",
                    _ => continue,
                };
                let f = format!("{full}.{}.{n}", e.name);
                if e.name == "Subsystem" {
                    self.object(read_bytes, "Subsystem", &n, &format!("{own}/{sub}"), &f);
                } else if let Some(id) = read(&format!("{own}/{sub}/{n}.xml")).and_then(|d| d.child(&e.name).and_then(|o| o.attr("uuid").map(str::to_string))) {
                    self.full.insert(f, id);
                }
            }
        }
    }

    /// Свойства и состав объекта верхнего уровня; у общего реквизита — его состав.
    pub(crate) fn object_info(&mut self, class: &str, obj: &El, full: &str) {
        let mut o = Obj::default();
        if let Some(p) = obj.child("Properties") {
            for e in p.elems() {
                if let Some(t) = e.peek_text() {
                    o.props.insert(e.name.clone(), t);
                }
            }
            o.owner_list = p.child("Owners").into_iter().flat_map(|w| w.elems()).filter_map(|e| e.text().ok()).collect();
            // регистрируемые документы журнала по порядку, через перевод строки (состав полей журнала — query::roles)
            // движения документа (регистры), через перевод строки: тип поля Recorder регистра — query/dlist
            if let Some(r) = p.child("RegisterRecords").filter(|_| class == "Document") {
                o.props.insert("RegisterRecords".into(), r.elems().filter_map(|e| e.text().ok()).collect::<Vec<_>>().join("
"));
            }
            if let Some(r) = p.child("RegisteredDocuments") {
                o.props.insert("RegisteredDocuments".into(), r.elems().filter_map(|e| e.text().ok()).collect::<Vec<_>>().join("\n"));
            }
            o.owners = !o.owner_list.is_empty();
            // состав критерия отбора (реквизиты объектов), через перевод строки: тип поля Ссылка в запросе
            if class == "FilterCriterion" {
                o.props.insert("Content".into(), p.child("Content").into_iter().flat_map(|c| c.elems()).filter_map(|e| e.text().ok()).collect::<Vec<_>>().join("
"));
            }
            if class == "CommonAttribute" {
                let content = p.child("Content").into_iter().flat_map(|c| c.all("xr:Item")).filter_map(|it| Some((it.child("xr:Metadata")?.text().ok()?, it.child("xr:Use")?.text().ok()?))).collect();
                let name = full.split_once('.').map(|(_, n)| n.to_string()).unwrap_or_default();
                self.common.push((name, o.props.get("AutoUse").cloned().unwrap_or_default(), content));
            }
        }
        if let Some(co) = obj.child("ChildObjects") {
            for e in co.elems() {
                if let Some(n) = e.child("Properties").and_then(|p| p.child("Name")).and_then(|n| n.text().ok()) {
                    o.kids.push((e.name.clone(), n));
                }
            }
        }
        self.objects.insert(full.to_string(), o);
    }

    /// Элементы с uuid в ChildObjects (реквизиты, табличные части с их реквизитами, значения, команды…).
    fn children(&mut self, co: &El, full: &str) {
        for e in co.elems() {
            let (Some(id), Some(n)) = (e.attr("uuid"), e.child("Properties").and_then(|p| p.child("Name")).and_then(|n| n.text().ok())) else { continue };
            let f = format!("{full}.{}.{n}", e.name);
            if let Some(sub) = e.child("ChildObjects") {
                self.children(sub, &f);
            }
            if let Some(t) = e.child("Properties").and_then(|p| p.child("Type")) {
                self.attr_types.insert(f.clone(), t.all("v8:Type").filter_map(|c| c.text().ok()).collect());
                self.type_sets(&f, t);
            }
            if e.child("Properties").and_then(|p| p.child("Balance")).and_then(|b| b.text().ok()).as_deref() == Some("false") {
                self.unbalanced.insert(f.clone());
            }
            if e.name == "Dimension" && e.child("Properties").and_then(|p| p.child("MainFilter")).and_then(|b| b.text().ok()).as_deref() == Some("true") {
                self.main_filter.insert(f.clone());
            }
            if e.name == "Column" {
                let refs = e.child("Properties").and_then(|p| p.child("References")).into_iter().flat_map(|r| r.elems()).filter_map(|i| i.text().ok()).collect();
                self.column_refs.insert(f.clone(), refs);
            }
            if e.child("Properties").and_then(|p| p.child("PasswordMode")).and_then(|m| m.text().ok()).as_deref() == Some("true") {
                self.password.insert(n.to_lowercase());
                self.password_full.insert(f.clone());
            }
            if e.name == "Dimension" && e.child("Properties").and_then(|p| p.child("Master")).and_then(|m| m.text().ok()).as_deref() == Some("true") {
                self.masters.entry(full.to_string()).or_default().push(n.clone());
            }
            self.full.insert(f, id.to_string());
        }
    }

    /// Элементы `<Item id><Name>…` на любой глубине (`ChildItems`).
    fn predefined_items(&mut self, e: &El, owner: &str) {
        for it in e.elems() {
            if let (Some(id), Some(n)) = (it.attr("id"), it.child("Name").and_then(|n| n.text().ok())) {
                self.predefined.insert(format!("{owner}.{n}"), id.to_string());
            }
            self.predefined_items(it, owner);
        }
    }

    fn type_sets(&mut self, full: &str, t: &El) {
        let v: Vec<String> = t.all("v8:TypeSet").filter_map(|c| c.text().ok()).collect();
        if !v.is_empty() {
            self.type_sets.insert(full.to_string(), v);
        }
    }

    pub fn id(&self, full: &str) -> Option<&str> {
        self.full.get(full).map(String::as_str)
    }
}

