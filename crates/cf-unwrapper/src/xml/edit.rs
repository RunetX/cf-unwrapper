//! Палитра свойств: правка описания одного объекта через его XML, как в Конфигураторе.
//!
//! Объект → описание `<Класс>/<Имя>.xml` выгрузкой в память ([`export`]) → свойства ([`props`]) → новое значение
//! пишется в текст XML ([`set`]) → запись объекта собирается загрузкой ([`import`], та же, что у `cfkit import`) →
//! заменяет свою в конфигурации ([`store`]). Остальные записи не трогаются, .cf пишет их байт в байт.
//! Загрузка — проверка правки: значение, которое она не принимает, не попадает в конфигурацию.

use super::classes::P;
use super::common;
use super::imp::{desc, dom, index};
use crate::meta::{Config, Item};
use std::ops::Range;

/// Имена ↔ uuid для выгрузки и загрузки одного объекта: индекс выгрузки и обратный ему индекс загрузки.
pub struct Names {
    pub exp: common::Index,
    pub imp: index::Index,
}

impl Names {
    pub fn new(cfg: &Config) -> Names {
        let exp = common::Index::new(cfg);
        let mut imp = index::Index::default();
        for (id, full) in &exp.full {
            if crate::brace::is_guid(id) {
                imp.full.insert(full.clone(), id.clone());
            } else if let Some((_, pid)) = id.rsplit_once('.') {
                // предопределенные: ключ выгрузки `<владелец>.<id>`, значение `<владелец>.<имя>`
                imp.predefined.insert(full.clone(), pid.to_string());
            }
        }
        for (t, name) in &exp.types {
            imp.types.insert(name.clone(), (t.clone(), String::new()));
        }
        // права роли ссылаются на `Configuration.<Имя>` — запись корня (как `rights::names` выгрузки)
        imp.root = (cfg.root_id.clone(), cfg.root.header.name.clone());
        Names { exp, imp }
    }
}

/// Объект с собственным описанием: класс, элемент дерева, класс владельца (форма, макет), полное имя
/// вложенной подсистемы (`Subsystem.А.Subsystem.Б`), запись в .cf; `sub` — вложенный элемент описания
/// (реквизит, табличная часть, измерение, команда…): его свойства правятся в XML владельца, запись — владельца.
pub struct Obj<'a> {
    pub class: &'static str,
    pub item: &'a Item,
    pub owner: Option<&'static str>,
    pub full: Option<String>,
    pub entry: String,
    pub sub: Option<&'a Item>,
}

impl Obj<'_> {
    /// Класс свойств палитры: вид вложенного элемента или класс объекта.
    pub fn kind(&self) -> &'static str {
        self.sub.map_or(self.class, |s| s.kind)
    }
}

/// Объект по uuid: корень (`root` или uuid записи корня), верхнего уровня, форма или макет объекта, вложенная
/// подсистема, вложенный элемент описания объекта.
pub fn find<'a>(cfg: &'a Config, id: &str) -> Option<Obj<'a>> {
    fn sub<'a>(it: &'a Item, full: &str, id: &str) -> Option<Obj<'a>> {
        for ch in it.children.iter().filter(|c| c.kind == "Subsystem") {
            let f = format!("{full}.Subsystem.{}", ch.header.name);
            if ch.header.id == id {
                return Some(Obj { class: "Subsystem", item: ch, owner: None, full: Some(f), entry: id.to_string(), sub: None });
            }
            if let Some(o) = sub(ch, &f, id) {
                return Some(o);
            }
        }
        None
    }
    // формы, макеты и подсистемы — свои записи, остальные потомки — в описании владельца
    fn nested<'a>(it: &'a Item, id: &str) -> Option<&'a Item> {
        it.children.iter().filter(|c| !matches!(c.kind, "Form" | "Template" | "Subsystem")).find_map(|c| if c.header.id == id { Some(c) } else { nested(c, id) })
    }
    // объект `it` класса `class` с записью `entry`: он сам, его форма или макет, вложенный элемент
    fn within<'a>(class: &'static str, it: &'a Item, entry: &str, id: &str) -> Option<Obj<'a>> {
        let own = |sub| Obj { class, item: it, owner: None, full: None, entry: entry.to_string(), sub };
        if it.header.id == id {
            return Some(own(None));
        }
        if let Some(ch) = it.children.iter().find(|c| c.header.id == id && matches!(c.kind, "Form" | "Template")) {
            return Some(Obj { class: ch.kind, item: ch, owner: Some(class), full: None, entry: id.to_string(), sub: None });
        }
        nested(it, id).map(|n| own(Some(n)))
    }
    // внешняя обработка — корень-объект: запись описания — запись корня, заголовок — по ObjectId
    if cfg.root.kind == "ExternalDataProcessor" {
        let id = if id == "root" || id == cfg.root_id { cfg.root.header.id.as_str() } else { id };
        return within("ExternalDataProcessor", &cfg.root, &cfg.root_id, id);
    }
    if id == "root" || id == cfg.root_id {
        return Some(Obj { class: "Configuration", item: &cfg.root, owner: None, full: None, entry: cfg.root_id.clone(), sub: None });
    }
    for g in &cfg.groups {
        for it in &g.items {
            if g.class == "Subsystem" {
                if it.header.id == id {
                    return Some(Obj { class: g.class, item: it, owner: None, full: None, entry: id.to_string(), sub: None });
                }
                if let Some(ch) = it.children.iter().find(|c| c.header.id == id && matches!(c.kind, "Form" | "Template")) {
                    return Some(Obj { class: ch.kind, item: ch, owner: Some(g.class), full: None, entry: id.to_string(), sub: None });
                }
                if let Some(o) = sub(it, &format!("Subsystem.{}", it.header.name), id) {
                    return Some(o);
                }
            } else if let Some(o) = within(g.class, it, &it.header.id, id) {
                return Some(o);
            }
        }
    }
    None
}

/// Описание объекта в памяти, как файл выгрузки (с BOM); незнакомое значение — ошибка, как у выгрузки.
pub fn export(cfg: &Config, names: &Names, o: &Obj) -> Result<String, String> {
    let b = match o.class {
        "Configuration" => super::classes_tools::configuration(cfg, &names.exp),
        "ExternalDataProcessor" => super::classes_tools::external_data_processor(cfg, &names.exp),
        _ => super::classes::descriptor(cfg, &names.exp, o.class, o.item),
    };
    let b = b.ok_or(format!("выгрузка описания {} не поддержана", o.class))?;
    if let Some(l) = super::unknown_mark(std::path::Path::new("x.xml"), &b) {
        return Err(format!("незнакомое значение: {l}"));
    }
    String::from_utf8(b).map_err(|_| "описание не UTF-8".into())
}

/// Запись объекта из описания — загрузка XML; что-то не разобрано — ошибка.
pub fn import(xml: &str, names: &Names, o: &Obj) -> Result<Vec<u8>, String> {
    let doc = dom::parse(xml)?;
    if o.class == "Configuration" {
        return super::imp::config::record(&doc, &names.imp);
    }
    desc::record(&doc, o.class, o.owner, o.full.as_deref(), &names.imp)?.ok_or(format!("загрузка описания {} не поддержана", o.class))
}

/// Вид значения свойства — какой редактор у него в палитре.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Kind {
    Bool,
    /// Значения: (код записи, значение XML).
    Enum(&'static [(&'static str, &'static str)]),
    Num,
    Str,
    /// Многоязычная строка.
    Ml,
    /// Ссылка на объект метаданных (полное имя): классы для выбора; `FORM` — формы объекта и общие формы,
    /// `TEMPLATE` — макеты объекта.
    Ref(&'static [&'static str]),
    /// Список ссылок на объекты метаданных (`xr:Item`) — классы для выбора; пустой — список полей объекта (`xr:Field`).
    List(&'static [&'static str]),
    /// Описание типов (диалог «Редактирование типа данных»).
    Type,
    /// Пока не редактируется: ссылки, составные значения.
    ReadOnly,
}

/// Классы ссылки на форму и на макет объекта (см. [`Kind::Ref`]).
pub const FORM: &[&str] = &["Form"];
pub const TEMPLATE: &[&str] = &["Template"];
/// Ссылочные классы (ввод на основании).
const REF_CLASSES: &[&str] = &["Catalog", "Document", "ChartOfCharacteristicTypes", "ChartOfAccounts", "ChartOfCalculationTypes", "ExchangePlan", "BusinessProcess", "Task"];
const REGISTERS: &[&str] = &["InformationRegister", "AccumulationRegister", "AccountingRegister", "CalculationRegister"];

/// Свойства-списки, которые палитра правит: свойство → классы для выбора (пусто — поля объекта).
fn list_of(tag: &str) -> Option<&'static [&'static str]> {
    Some(match tag {
        "Owners" => &["Catalog", "ExchangePlan", "ChartOfCharacteristicTypes", "ChartOfAccounts", "ChartOfCalculationTypes"],
        "BasedOn" => REF_CLASSES,
        "RegisterRecords" => REGISTERS,
        "RegisteredDocuments" | "Documents" => &["Document"],
        "BaseCalculationTypes" => &["ChartOfCalculationTypes"],
        "InputByString" | "DataLockFields" => &[],
        _ => return None,
    })
}

/// Свойства-ссылки, которые палитра правит: свойство → классы для выбора.
fn ref_of(tag: &str) -> Option<&'static [&'static str]> {
    Some(match tag {
        t if (t.starts_with("Default") || t.starts_with("Auxiliary")) && t.ends_with("Form") => FORM,
        "MainDataCompositionSchema" => TEMPLATE,
        "Numerator" => &["DocumentNumerator"],
        "Task" => &["Task"],
        "VariantsStorage" | "SettingsStorage" => &["SettingsStorage"],
        "ChartOfAccounts" => &["ChartOfAccounts"],
        "ExtDimensionTypes" => &["ChartOfCharacteristicTypes"],
        _ => return None,
    })
}

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Text(String),
    /// (язык, текст)
    Ml(Vec<(String, String)>),
    /// Полные имена объектов или полей.
    List(Vec<String>),
    /// Описание типов.
    Type(Types),
}

/// Описание типов `<Type>` — как пишет выгрузка: типы по порядку, затем квалификаторы числа, строки, даты.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Types {
    /// (`v8:Type` или `v8:TypeSet`, значение: `xs:string`, `cfg:CatalogRef.Валюты`, `cfg:DefinedType.Деньги`…)
    pub items: Vec<(String, String)>,
    /// (разрядность, дробная часть, неотрицательное)
    pub number: Option<(u32, u32, bool)>,
    /// (длина, фиксированная)
    pub string: Option<(u32, bool)>,
    /// `Date`, `Time`, `DateTime`
    pub date: Option<String>,
}

impl Types {
    pub fn has(&self, t: &str) -> bool {
        self.items.iter().any(|(e, v)| e == "v8:Type" && v == t)
    }

    /// Квалификаторы — ровно у примитивных типов описания (в записи они часть элемента типа). Нового типа —
    /// как у `ibcmd config import` без квалификаторов (опыт 27.10.2026): Число(0, 0, любой), Строка(0, переменная),
    /// Дата — дата и время.
    pub fn normalize(&mut self) {
        self.number = self.has("xs:decimal").then(|| self.number.unwrap_or((0, 0, false)));
        self.string = self.has("xs:string").then(|| self.string.unwrap_or((0, false)));
        self.date = self.has("xs:dateTime").then(|| self.date.clone().unwrap_or_else(|| "DateTime".into()));
    }

    /// Из элемента `<Type>`; чего здесь нет (атрибуты, `v8:TypeId`, двоичные данные…) — `None`, свойство только для чтения.
    fn of(e: &dom::El) -> Option<Types> {
        if !e.attrs.is_empty() {
            return None;
        }
        let mut t = Types::default();
        let leaf = |e: &dom::El, n: &str| e.peek_child(n).filter(|c| c.attrs.is_empty()).and_then(|c| c.peek_text());
        let num = |e: &dom::El, n: &str| leaf(e, n)?.parse::<u32>().ok();
        for q in e.peek_elems() {
            let n = q.peek_elems().count();
            match q.name.as_str() {
                "v8:Type" | "v8:TypeSet" if q.attrs.is_empty() && n == 0 => t.items.push((q.name.clone(), q.peek_text()?)),
                "v8:NumberQualifiers" if n == 3 => {
                    let sign = match leaf(q, "v8:AllowedSign")?.as_str() {
                        "Any" => false,
                        "Nonnegative" => true,
                        _ => return None,
                    };
                    t.number = Some((num(q, "v8:Digits")?, num(q, "v8:FractionDigits")?, sign));
                }
                "v8:StringQualifiers" if n == 2 => {
                    let fixed = match leaf(q, "v8:AllowedLength")?.as_str() {
                        "Variable" => false,
                        "Fixed" => true,
                        _ => return None,
                    };
                    t.string = Some((num(q, "v8:Length")?, fixed));
                }
                "v8:DateQualifiers" if n == 1 => t.date = Some(leaf(q, "v8:DateFractions").filter(|f| ["Date", "Time", "DateTime"].contains(&f.as_str()))?),
                _ => return None,
            }
        }
        Some(t)
    }

    /// Текст элемента `<tag>`: строки с отступом `ind` (отступ самого элемента), CRLF.
    fn xml(&self, tag: &str, ind: &str) -> String {
        if self.items.is_empty() && self.number.is_none() && self.string.is_none() && self.date.is_none() {
            return format!("<{tag}/>");
        }
        let esc = |t: &str| t.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;");
        let mut s = format!("<{tag}>");
        for (e, v) in &self.items {
            s += &format!("\r\n{ind}\t<{e}>{}</{e}>", esc(v));
        }
        let mut q = |name: &str, kids: &[(&str, String)]| {
            s += &format!("\r\n{ind}\t<v8:{name}>");
            for (k, v) in kids {
                s += &format!("\r\n{ind}\t\t<v8:{k}>{v}</v8:{k}>");
            }
            s += &format!("\r\n{ind}\t</v8:{name}>");
        };
        if let Some((d, f, nonneg)) = self.number {
            q("NumberQualifiers", &[("Digits", d.to_string()), ("FractionDigits", f.to_string()), ("AllowedSign", (if nonneg { "Nonnegative" } else { "Any" }).into())]);
        }
        if let Some((len, fixed)) = self.string {
            q("StringQualifiers", &[("Length", len.to_string()), ("AllowedLength", (if fixed { "Fixed" } else { "Variable" }).into())]);
        }
        if let Some(d) = &self.date {
            q("DateQualifiers", &[("DateFractions", d.clone())]);
        }
        s + &format!("\r\n{ind}</{tag}>")
    }

    /// Представление, как в дереве диалога «Редактирование типа данных»: имена типов через запятую.
    // shortcut: в палитре Конфигуратора представление типа (квалификаторы, составной тип) со снимком не сверено.
    pub fn shown(&self) -> String {
        self.items.iter().map(|(_, v)| type_ru(v)).collect::<Vec<_>>().join(", ")
    }
}

/// Описание типов из текста `<Type>…</Type>` (командная строка).
pub fn parse_types(xml: &str) -> Result<Types, String> {
    Types::of(&dom::parse(xml)?).ok_or("описание типов: неизвестный элемент".into())
}

/// Примитивные типы: (XML, имя в дереве диалога, снимок 07-type-dialog Конфигуратора 8.5).
pub const PRIMITIVES: &[(&str, &str)] = &[
    ("xs:decimal", "Число"),
    ("xs:string", "Строка"),
    ("xs:dateTime", "Дата"),
    ("xs:boolean", "Булево"),
    ("v8:ValueStorage", "ХранилищеЗначения"),
    ("v8:UUID", "УникальныйИдентификатор"),
];

/// Ссылочные типы по классам: (префикс `cfg:`, имя в дереве диалога, класс объектов), порядок снимка 07.
pub const REF_TYPES: &[(&str, &str, &str)] = &[
    ("CatalogRef", "СправочникСсылка", "Catalog"),
    ("DocumentRef", "ДокументСсылка", "Document"),
    ("EnumRef", "ПеречислениеСсылка", "Enum"),
    ("ChartOfCharacteristicTypesRef", "ПланВидовХарактеристикСсылка", "ChartOfCharacteristicTypes"),
    ("ChartOfAccountsRef", "ПланСчетовСсылка", "ChartOfAccounts"),
    ("ChartOfCalculationTypesRef", "ПланВидовРасчетаСсылка", "ChartOfCalculationTypes"),
    ("BusinessProcessRef", "БизнесПроцессСсылка", "BusinessProcess"),
    ("BusinessProcessRoutePointRef", "ТочкаМаршрутаБизнесПроцессаСсылка", "BusinessProcess"),
    ("TaskRef", "ЗадачаСсылка", "Task"),
    ("ExchangePlanRef", "ПланОбменаСсылка", "ExchangePlan"),
];

/// Имя типа по-русски: `cfg:CatalogRef.Валюты` → `СправочникСсылка.Валюты`.
// shortcut: только типы дерева диалога и определяемые; прочие (объекты, наборы записей подписок) — именем XML.
pub fn type_ru(v: &str) -> String {
    if let Some((_, r)) = PRIMITIVES.iter().find(|(x, _)| *x == v) {
        return r.to_string();
    }
    let Some(rest) = v.strip_prefix("cfg:") else { return v.to_string() };
    let (head, tail) = rest.split_once('.').map_or((rest, None), |(h, t)| (h, Some(t)));
    let ru = match head {
        "AnyRef" => "ЛюбаяСсылка",
        "Characteristic" => "Характеристика",
        "DefinedType" => "ОпределяемыйТип",
        h => match REF_TYPES.iter().find(|r| r.0 == h) {
            Some(r) => r.1,
            // объект (основной реквизит формы): CatalogObject → СправочникОбъект — по виду ссылки
            None => match h.strip_suffix("Object").and_then(|k| REF_TYPES.iter().find(|r| r.2 == k)) {
                Some(r) => return format!("{}Объект{}", r.1.trim_end_matches("Ссылка"), tail.map_or(String::new(), |t| format!(".{t}"))),
                None => return v.to_string(),
            },
        },
    };
    tail.map_or(ru.to_string(), |t| format!("{ru}.{t}"))
}

#[derive(Clone, Debug)]
pub struct Prop {
    pub tag: String,
    pub kind: Kind,
    pub value: Value,
    /// Составное значение (элемент с вложенными элементами): в палитре — «Открыть», как у Конфигуратора.
    pub compound: bool,
    /// Байты элемента в тексте описания: от `<` до конца закрывающего тега.
    span: Range<usize>,
    /// Отступ строки элемента (для многострочного значения).
    indent: String,
    /// Значение, которого выгрузка не пишет (свойство элемента формы): такое значение убирает элемент из XML. Пустой
    /// `span` — элемента в XML нет, правка вставляет его в начало `span`.
    pub default: Option<String>,
}

impl Prop {
    /// Значение одной строкой: текст, русский (или первый) вариант многоязычной строки, сводка составного.
    pub fn shown(&self) -> String {
        match &self.value {
            Value::Text(t) => t.clone(),
            Value::Ml(v) => v.iter().find(|(l, _)| l == "ru").or(v.first()).map(|(_, t)| t.clone()).unwrap_or_default(),
            Value::List(v) => v.join(", "),
            Value::Type(t) => t.shown(),
        }
    }

    /// Пустое значение: пустая строка, список без элементов.
    pub fn is_empty(&self) -> bool {
        match &self.value {
            Value::Text(t) => t.is_empty(),
            Value::Ml(v) => v.is_empty(),
            Value::List(v) => v.is_empty(),
            Value::Type(t) => t.items.is_empty(),
        }
    }
}

/// Доступно ли свойство при текущих значениях остальных (как у Конфигуратора: недоступное — серое).
/// Правила — по снимку палитры справочника (docs/ui-ref/properties.png) и по смыслу свойств (docs/editor.md).
pub fn available(props: &[Prop], tag: &str) -> bool {
    let get = |t: &str| props.iter().find(|p| p.tag == t);
    let is = |t: &str, v: &str| get(t).is_some_and(|p| p.shown() == v);
    let empty = |t: &str| get(t).is_none_or(Prop::is_empty);
    let has = |t: &str| get(t).is_some();
    let folders = !is("Hierarchical", "false") && !is("HierarchyType", "HierarchyOfItems");
    match tag {
        "HierarchyType" | "LimitLevelCount" => !is("Hierarchical", "false"),
        "LevelCount" => !is("Hierarchical", "false") && !is("LimitLevelCount", "false"),
        "FoldersOnTop" | "DefaultFolderForm" | "DefaultFolderChoiceForm" | "AuxiliaryFolderForm" | "AuxiliaryFolderChoiceForm" => folders,
        "SubordinationUse" => !empty("Owners"),
        // нумератор задает тип, длину, периодичность и уникальность номера
        "NumberType" | "NumberLength" | "NumberAllowedLength" | "NumberPeriodicity" | "CheckUnique" if !empty("Numerator") => false,
        // нет кода (номера) — нечего настраивать
        "CodeType" | "CodeAllowedLength" | "CodeSeries" | "CheckUnique" | "Autonumbering" if has("CodeLength") => !is("CodeLength", "0"),
        "NumberType" | "NumberAllowedLength" | "NumberPeriodicity" | "CheckUnique" | "Autonumbering" if has("NumberLength") => !is("NumberLength", "0"),
        "RealTimePosting" | "RegisterRecordsDeletion" | "RegisterRecordsWritingOnPost" | "SequenceFilling" | "PostInPrivilegedMode" | "UnpostInPrivilegedMode" => !is("Posting", "Deny"),
        "UpdateDataHistoryImmediatelyAfterWrite" | "ExecuteAfterWriteDataHistoryVersionProcessing" => !is("DataHistory", "DontUse"),
        "FullTextSearchOnInputByString" => !is("FullTextSearch", "DontUse"),
        "MainFilterOnPeriod" | "EnableTotalsSliceFirst" | "EnableTotalsSliceLast" => !is("InformationRegisterPeriodicity", "Nonperiodical"),
        "ServerCall" | "Privileged" => !is("Server", "false"),
        "UseOneCommand" => !is("IncludeInCommandInterface", "false"),
        "IncludeConfigurationExtensions" => !is("DistributedInfoBase", "false"),
        "MaxExtDimensionCount" => !empty("ExtDimensionTypes"),
        "BaseCalculationTypes" => !is("DependenceOnCalculationTypes", "DontUse"),
        "SeparatedDataUse" | "DataSeparationValue" | "DataSeparationUse" | "ConditionalSeparation" | "UsersSeparation" | "AuthenticationSeparation" | "ConfigurationExtensionsSeparation" => !is("DataSeparation", "DontUse"),
        _ => true,
    }
}

/// Имена стандартных реквизитов объекта (свойство StandardAttributes, без табличных частей).
pub fn standard_attributes(xml: &str) -> Vec<String> {
    let head = &xml[..xml.find("<ChildObjects").unwrap_or(xml.len())];
    head.split("<xr:StandardAttribute name=\"").skip(1).filter_map(|s| s.split('"').next()).map(String::from).collect()
}

/// Вложенные объекты описания (ChildObjects): (вид, имя) — формы, макеты, реквизиты, табличные части, команды.
pub fn children(xml: &str) -> Vec<(String, String)> {
    let Some(s) = xml.find("<ChildObjects>") else { return Vec::new() };
    let Some(e) = element_end(xml, s) else { return Vec::new() };
    let Ok(co) = dom::parse(&xml[s..e]) else { return Vec::new() };
    co.elems()
        .filter_map(|c| {
            let name = c.text().ok().or_else(|| c.child("Properties")?.child("Name")?.text().ok())?;
            Some((c.name.clone(), name))
        })
        .collect()
}

/// Таблица свойств класса из таблиц выгрузки.
fn table(class: &str) -> Option<&'static [(&'static str, P)]> {
    super::classes::table(class)
        .or_else(|| super::classes_refs::table(class))
        .or_else(|| super::classes_regs::tables(class).map(|t| t.0))
        .or_else(|| super::classes_tools::descriptor_table(class))
        .or_else(|| (class == "Configuration").then_some(super::classes_tools::CONFIGURATION))
}

fn kind_of(class: &str, tag: &str) -> Option<Kind> {
    Some(match tag {
        "Name" | "Comment" => Kind::Str,
        "Synonym" => Kind::Ml,
        t if list_of(t).is_some() => Kind::List(list_of(t)?),
        t if ref_of(t).is_some() => Kind::Ref(ref_of(t)?),
        _ => match table(class).and_then(|t| t.iter().find(|(t, _)| *t == tag)).map(|p| &p.1) {
            Some(P::Bool(_)) => Kind::Bool,
            Some(P::Tok(_, d)) if d.is_empty() => Kind::Num,
            Some(P::Tok(_, d) | P::TokOr(_, d, _)) => Kind::Enum(d),
            Some(P::Ml(_)) => Kind::Ml,
            Some(P::Str(_)) => Kind::Str,
            Some(P::Type(_)) => Kind::Type,
            _ => attr_kind(tag),
        },
    })
}

/// Вид свойства, которое таблица выгрузки пишет своей функцией (свойства реквизита у константы и общего
/// реквизита, числа служб и регламентных заданий, свойства вложенных элементов: реквизитов, табличных частей,
/// измерений, команд); остальное — только для чтения. Значение все равно проверяет загрузка.
fn attr_kind(tag: &str) -> Kind {
    const AUTO_USE: &[(&str, &str)] = &[("", "Auto"), ("", "Use"), ("", "DontUse")];
    const USE: &[(&str, &str)] = &[("", "DontUse"), ("", "Use")];
    match tag {
        "Indexing" => Kind::Enum(&[("", "DontIndex"), ("", "Index"), ("", "IndexWithAdditionalOrder")]),
        "FullTextSearch" | "DataHistory" => Kind::Enum(USE),
        "Use" => Kind::Enum(&[("", "ForItem"), ("", "ForFolder"), ("", "ForFolderAndItem")]),
        "ParameterUseMode" => Kind::Enum(&[("", "Single"), ("", "Multiple")]),
        "LineNumberLength" => Kind::Num,
        "Type" => Kind::Type,
        "Representation" => Kind::Enum(&[("", "Text"), ("", "Picture"), ("", "PictureAndText"), ("", "Auto")]),
        "FillChecking" => Kind::Enum(&[("", "DontCheck"), ("", "ShowError")]),
        "QuickChoice" | "CreateOnInput" => Kind::Enum(AUTO_USE),
        "ChoiceHistoryOnInput" => Kind::Enum(&[("", "Auto"), ("", "DontUse")]),
        "ChoiceFoldersAndItems" => Kind::Enum(&[("", "Items"), ("", "Folders"), ("", "FoldersAndItems")]),
        "Format" | "EditFormat" | "ToolTip" => Kind::Ml,
        "Mask" => Kind::Str,
        "PeriodAdjustmentLength" | "NumberLength" | "SessionMaxAge" | "RestartCountOnFailure" | "RestartIntervalOnFailure" => Kind::Num,
        _ => Kind::ReadOnly,
    }
}

/// Свойства объекта — элементы `<Properties>` корневого объекта описания, по порядку.
pub fn props(xml: &str, class: &str) -> Result<Vec<Prop>, String> {
    props_of(xml, class, None)
}

/// Свойства объекта или его вложенного элемента `sub` (uuid) — элементы его `<Properties>`, по порядку.
pub fn props_of(xml: &str, class: &str, sub: Option<&str>) -> Result<Vec<Prop>, String> {
    let at = match sub {
        None => properties(xml),
        Some(id) => xml.find(&format!(" uuid=\"{id}\"")).and_then(|u| xml[..u].rfind('<')).and_then(|s| properties_at(xml, s)),
    };
    let (body, end) = at.ok_or("в описании нет <Properties>")?;
    props_in(xml, body, end, &|tag| kind_of(class, tag), &[])
}

/// Свойства элемента управляемой формы (`id` — атрибут id элемента; `None` — сама форма) по тексту ее Form.xml: все
/// свойства вида в порядке выгрузки (`form_props::ITEMS`) — записанные в XML со значением, прочие с умолчанием
/// (`Prop::default`, пустой `span` — место вставки); состав (подчиненные, контекстное меню, подсказка, события,
/// коллекции формы) — не свойства.
pub fn form_props(xml: &str, id: Option<&str>) -> Result<Vec<Prop>, String> {
    let start = match id {
        None => xml.find("<Form ").ok_or("нет <Form>")?,
        // у реквизитов, команд, колонок свои id: элементы — до коллекций формы (выгрузка пишет их первыми)
        Some(id) => {
            let items = &xml[..xml.find("\r\n\t<Attributes>").unwrap_or(xml.len())];
            items.find(&format!(" id=\"{id}\"")).and_then(|u| items[..u].rfind('<')).ok_or(format!("нет элемента id={id}"))?
        }
    };
    let body = start + xml[start..].find('>').ok_or("незакрытый тег")? + 1;
    let elem = xml[start + 1..].split([' ', '>', '/']).next().unwrap_or_default().to_string();
    if xml[..body].ends_with("/>") {
        return Ok(Vec::new());
    }
    let end = element_end(xml, start).ok_or("незакрытый элемент")?;
    let end = xml[..end].rfind("</").ok_or("незакрытый элемент")?;
    const SKIP: &[&str] = &[
        "ChildItems", "ContextMenu", "ExtendedTooltip", "Events", "AutoCommandBar", "SearchStringAddition", "ViewStatusAddition", "SearchControlAddition",
        "Attributes", "Commands", "Parameters", "CommandSet", "CommandInterface", "ConditionalAppearance",
    ];
    let mut v = props_in(xml, body, end, &form_kind, SKIP)?;
    // простое значение без своего вида (DataPath) — строкой: недопустимое отвергнет загрузка формы
    for p in v.iter_mut().filter(|p| p.kind == Kind::ReadOnly && !p.compound) {
        p.kind = Kind::Str;
    }
    let Some((_, order)) = super::form_props::ITEMS.iter().find(|(t, _)| *t == elem) else { return Ok(v) };
    // место вставки — конец строки перед первым узлом состава (контекстное меню, подсказка…) или перед закрывающим
    // тегом; вставляется «перевод строки, отступ, элемент»
    let line = xml[..start].rfind('\n').map_or(0, |n| n + 1);
    let indent = format!("{}\t", &xml[line..start]);
    let mut next = end;
    let mut i = body;
    while let Some(off) = xml[i..end].find('<') {
        let s = i + off;
        let name = xml[s + 1..].split([' ', '>', '/']).next().unwrap_or_default();
        if SKIP.contains(&name) {
            next = s;
            break;
        }
        i = element_end(xml, s).ok_or("незакрытый элемент")?;
    }
    let at = xml[body..next].rfind("\r\n").map_or(body, |n| body + n);
    // у формы — только свойства ее вида (по типу основного реквизита): справочника, документа, списка, отчета
    let main = xml.split("<MainAttribute>true</MainAttribute>").next().and_then(|h| h.rsplit("<v8:Type>").next()).unwrap_or_default();
    let own = |tag: &str| match tag {
        "UseForFoldersAndItems" => main.starts_with("cfg:CatalogObject") || main.starts_with("cfg:ChartOfCharacteristicTypesObject"),
        "AutoTime" | "UsePostingMode" | "RepostOnWrite" => main.starts_with("cfg:DocumentObject"),
        "GroupList" => main.starts_with("cfg:DynamicList"),
        "ReportResult" | "DetailsData" | "ReportFormType" | "VariantAppearance" | "AutoShowState" | "CustomSettingsFolder" | "ReportResultViewMode" | "ViewModeApplicationOnSetReportResult" => {
            main.starts_with("cfg:ReportObject")
        }
        _ => true,
    };
    let mut out = Vec::new();
    let present: Vec<String> = v.iter().map(|p| p.tag.clone()).collect();
    for tag in order.iter().filter(|t| elem != "Form" || own(t) || present.iter().any(|p| p == **t)) {
        let (kind, default) = form_kind_of(tag);
        match v.iter().position(|p| p.tag == *tag) {
            Some(k) => {
                let mut p = v.remove(k);
                if p.kind != Kind::ReadOnly && kind != Kind::ReadOnly && !matches!((&p.value, &kind), (Value::Text(_), Kind::Ml)) {
                    p.kind = kind;
                }
                p.default = Some(default.to_string());
                out.push(p);
            }
            None => {
                let value = if kind == Kind::Ml { Value::Ml(Vec::new()) } else { Value::Text(default.to_string()) };
                out.push(Prop { tag: tag.to_string(), kind, value, compound: false, span: at..at, indent: indent.clone(), default: Some(default.to_string()) });
            }
        }
    }
    out.extend(v);
    Ok(out)
}

/// Русское имя свойства элемента формы (палитра Конфигуратора 8.5), строки события `Event.<имя>` — имя события;
/// нет — XML-имя.
pub fn form_caption(tag: &str) -> &str {
    if let Some(ev) = tag.strip_prefix("Event.") {
        return super::form_props::EVENTS.iter().flat_map(|(_, e)| e.iter()).find(|e| e.0 == ev).map_or(ev, |e| e.1);
    }
    super::form_props::NAMES.iter().find(|(t, _)| *t == tag).map_or(tag, |(_, r)| r)
}

/// Строка палитры события элемента формы `Event.<имя>`: значение — процедура-обработчик. Правка — не по тексту
/// (`set`), а `formedit::set_event` с заготовкой процедуры в модуле.
pub fn form_event_prop(event: &str, handler: Option<String>) -> Prop {
    Prop { tag: format!("Event.{event}"), kind: Kind::Str, value: Value::Text(handler.unwrap_or_default()), compound: false, span: 0..0, indent: String::new(), default: None }
}

/// Вид и умолчание свойства элемента формы по таблице `form_props`.
fn form_kind_of(tag: &str) -> (Kind, &'static str) {
    let Some(&(_, k, d)) = super::form_props::KINDS.iter().find(|(t, _, _)| *t == tag) else { return (Kind::Str, "") };
    let kind = match k {
        'B' => Kind::Bool,
        'T' => Kind::Enum(&[("", "Auto"), ("", "true"), ("", "false")]),
        'E' => super::form_props::ENUMS.iter().find(|(t, _)| *t == tag).map_or(Kind::Str, |(_, v)| Kind::Enum(v)),
        'N' => Kind::Num,
        'M' => Kind::Ml,
        'R' => Kind::ReadOnly,
        _ => Kind::Str,
    };
    (kind, d)
}

/// Вид свойства элемента формы, записанного в XML, до таблицы `form_props` (свойства вне таблицы).
fn form_kind(tag: &str) -> Option<Kind> {
    Some(match tag {
        "Title" | "ToolTip" | "InputHint" | "Format" | "EditFormat" | "FooterText" | "HeaderText" => Kind::Ml,
        "Width" | "Height" | "MaxWidth" | "MaxHeight" | "MinWidth" => Kind::Num,
        _ => return None,
    })
}

/// Свойства — дочерние элементы текста `xml[body..end]` по порядку, кроме `skip`.
fn props_in(xml: &str, body: usize, end: usize, kind_of: &dyn Fn(&str) -> Option<Kind>, skip: &[&str]) -> Result<Vec<Prop>, String> {
    let mut out = Vec::new();
    let mut i = body;
    while let Some(off) = xml[i..end].find('<') {
        let start = i + off;
        let stop = element_end(xml, start).ok_or("описание: незакрытый элемент")?;
        let el = dom::parse(&xml[start..stop])?;
        let line = xml[..start].rfind('\n').map_or(0, |n| n + 1);
        let indent = xml[line..start].to_string();
        let leaf = el.text().ok();
        let items: Option<Vec<(String, String)>> = el.elems().map(|it| Some((it.child("v8:lang")?.text().ok()?, it.child("v8:content")?.text().ok()?))).collect();
        // список: только элементы `xr:Item` (ссылка на объект метаданных) или `xr:Field` с текстом
        let list = |fields: bool| -> Option<Vec<String>> {
            el.elems()
                .map(|it| {
                    let ok = if fields { it.name == "xr:Field" && it.attrs.is_empty() } else { it.name == "xr:Item" && it.attrs == [("xsi:type".to_string(), "xr:MDObjectRef".to_string())] };
                    ok.then(|| it.text().ok()).flatten()
                })
                .collect()
        };
        if skip.contains(&el.name.as_str()) {
            i = stop;
            continue;
        }
        let (kind, value) = match (kind_of(&el.name), leaf) {
            (Some(k @ Kind::List(_)), Some(t)) if t.is_empty() && el.attrs.is_empty() => (k, Value::List(Vec::new())),
            (Some(k @ Kind::List(c)), None) if el.attrs.is_empty() && list(c.is_empty()).is_some() => (k, Value::List(list(c.is_empty()).unwrap())),
            (Some(k @ Kind::Ref(_)), Some(t)) if el.attrs.is_empty() => (k, Value::Text(t)),
            (Some(Kind::Type), _) if Types::of(&el).is_some() => (Kind::Type, Value::Type(Types::of(&el).unwrap())),
            (Some(Kind::Ml), Some(t)) if t.is_empty() => (Kind::Ml, Value::Ml(Vec::new())),
            (Some(Kind::Ml), _) if items.is_some() && el.attrs.is_empty() => (Kind::Ml, Value::Ml(items.unwrap())),
            (Some(Kind::Enum(d)), Some(t)) if el.attrs.is_empty() && d.iter().any(|(_, v)| *v == t) => (Kind::Enum(d), Value::Text(t)),
            (Some(k @ (Kind::Bool | Kind::Num | Kind::Str)), Some(t)) if el.attrs.is_empty() => (k, Value::Text(t)),
            // свойство не из таблицы класса, но с булевым значением — флажок (недопустимое отвергнет загрузка)
            // или в таблице, но со своей выгрузкой (основной отбор по периоду регистра сведений)
            (None | Some(Kind::ReadOnly), Some(t)) if el.attrs.is_empty() && (t == "true" || t == "false") => (Kind::Bool, Value::Text(t)),
            (_, Some(t)) => (Kind::ReadOnly, Value::Text(t)),
            (_, None) => (Kind::ReadOnly, Value::Text(summary(&el))),
        };
        let compound = el.text().is_err() && kind == Kind::ReadOnly;
        out.push(Prop { tag: el.name.clone(), kind, value, compound, span: start..stop, indent, default: None });
        i = stop;
    }
    Ok(out)
}

/// Описание с новым значением свойства; остальной текст как был.
pub fn set(xml: &str, p: &Prop, v: &Value) -> String {
    // свойство элемента формы: умолчание выгрузка не пишет — элемента нет; нет элемента — вставка
    if let Some(d) = &p.default {
        let is_default = match v {
            Value::Text(t) => t == d,
            Value::Ml(items) => items.is_empty() && d.is_empty(),
            _ => false,
        };
        match (p.span.is_empty(), is_default) {
            (true, true) => return xml.to_string(),
            (false, true) => {
                let from = xml[..p.span.start].rfind("\r\n").unwrap_or(p.span.start);
                return format!("{}{}", &xml[..from], &xml[p.span.end..]);
            }
            (true, false) => {
                let one = Prop { span: 0..0, default: None, ..p.clone() };
                let frag = set("", &one, v);
                return format!("{}\r\n{}{frag}{}", &xml[..p.span.start], p.indent, &xml[p.span.start..]);
            }
            (false, false) => {}
        }
    }
    // текст значения — как в файле (`dom` не меняет переводы строк), поэтому без замены CRLF, как у `w::escape`
    let escape = |t: &str| t.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;");
    let leaf = |tag: &str, t: &str| if t.is_empty() { format!("<{tag}/>") } else { format!("<{tag}>{}</{tag}>", escape(t)) };
    let frag = match v {
        Value::Text(t) => leaf(&p.tag, t),
        Value::Ml(items) if items.is_empty() => format!("<{}/>", p.tag),
        Value::List(items) if items.is_empty() => format!("<{}/>", p.tag),
        Value::Type(t) => t.xml(&p.tag, &p.indent),
        Value::List(items) => {
            let ind = &p.indent;
            let item = if p.kind == Kind::List(&[]) { "<xr:Field>" } else { "<xr:Item xsi:type=\"xr:MDObjectRef\">" };
            let close = if p.kind == Kind::List(&[]) { "</xr:Field>" } else { "</xr:Item>" };
            let mut s = format!("<{}>", p.tag);
            for it in items {
                s += &format!("\r\n{ind}\t{item}{}{close}", escape(it));
            }
            s + &format!("\r\n{ind}</{}>", p.tag)
        }
        Value::Ml(items) => {
            let ind = &p.indent;
            let mut s = format!("<{}>", p.tag);
            for (lang, text) in items {
                s += &format!("\r\n{ind}\t<v8:item>\r\n{ind}\t\t{}\r\n{ind}\t\t{}\r\n{ind}\t</v8:item>", leaf("v8:lang", lang), leaf("v8:content", text));
            }
            s + &format!("\r\n{ind}</{}>", p.tag)
        }
    };
    let out = format!("{}{frag}{}", &xml[..p.span.start], &xml[p.span.end..]);
    match v {
        Value::Type(t) => match &p.value {
            Value::Type(old) => fit_fill(out, p.span.start, old, t),
            _ => out,
        },
        _ => out,
    }
}

/// Значение заполнения того же элемента, примитивного типа, убранного из описания, — пустое (`xsi:nil`): так
/// делает загрузка платформы (опыт 27.10.2026: Строка → Число и Дата — `<FillValue xsi:type="xs:string"/>` стал nil).
// Только убранный тип: в БП есть значения заполнения строкой у типа без xs:string (определяемый тип).
// shortcut: только примитивные значения заполнения; ссылочные и Min/MaxValue не проверены платформой.
fn fit_fill(xml: String, from: usize, old: &Types, t: &Types) -> String {
    let end = xml[from..].find("</Properties>").map_or(xml.len(), |e| from + e);
    let Some(s) = xml[from..end].find("<FillValue ").map(|i| from + i) else { return xml };
    let Some(e) = element_end(&xml, s) else { return xml };
    let head = &xml[s..s + xml[s..].find('>').unwrap_or(0)];
    let ty = head.split("xsi:type=\"").nth(1).and_then(|r| r.split('"').next());
    match ty {
        Some(ty) if old.has(ty) && !t.has(ty) => format!("{}<FillValue xsi:nil=\"true\"/>{}", &xml[..s], &xml[e..]),
        _ => xml,
    }
}

/// Сводка составного значения: тексты листьев через запятую.
fn summary(e: &dom::El) -> String {
    fn leaves(e: &dom::El, out: &mut Vec<String>) {
        match e.text() {
            Ok(t) if !t.is_empty() => out.push(t),
            Ok(_) => {}
            Err(_) => e.elems().for_each(|c| leaves(c, out)),
        }
    }
    let mut v = Vec::new();
    leaves(e, &mut v);
    let s = v.join(", ");
    if s.chars().count() > 200 {
        s.chars().take(200).collect::<String>() + "…"
    } else {
        s
    }
}

/// Тело `<Properties>` корневого объекта: (после открывающего тега, начало закрывающего).
fn properties(xml: &str) -> Option<(usize, usize)> {
    // корень MetaDataObject → объект → его первый дочерний <Properties> (InternalInfo своего Properties не имеет)
    let obj = xml.find("<MetaDataObject").and_then(|m| xml[m..].find('>').map(|e| m + e + 1))?;
    properties_at(xml, obj + xml[obj..].find('<')?)
}

/// Тело первого дочернего `<Properties>` элемента, открывающий тег которого начинается в `obj`.
fn properties_at(xml: &str, obj: usize) -> Option<(usize, usize)> {
    let mut i = obj + xml[obj..].find('>')? + 1;
    loop {
        let s = i + xml[i..].find('<')?;
        if xml[s..].starts_with("</") {
            return None;
        }
        let e = element_end(xml, s)?;
        if xml[s..].starts_with("<Properties>") {
            return Some((s + "<Properties>".len(), e - "</Properties>".len()));
        }
        i = e;
    }
}

/// Конец элемента, начинающегося в `start` (после `/>` или парного закрывающего тега).
fn element_end(xml: &str, start: usize) -> Option<usize> {
    let mut depth = 0usize;
    let mut i = start;
    loop {
        let s = i + xml[i..].find('<')?;
        let e = s + xml[s..].find('>')? + 1;
        let tag = &xml[s..e];
        if tag.starts_with("</") {
            depth = depth.checked_sub(1)?;
        } else if !tag.ends_with("/>") {
            depth += 1;
        }
        if depth == 0 {
            return Some(e);
        }
        i = e;
    }
}

/// Замена записи: (запись, данные до, данные после) — для отмены и повтора.
#[derive(Clone, Debug)]
pub struct Change {
    pub entry: String,
    pub before: Vec<u8>,
    pub after: Vec<u8>,
}

/// Новая запись объекта `id` в конфигурации; совпадает со старой — `None`.
pub fn store(cfg: &mut Config, id: &str, rec: &[u8]) -> Result<Option<Change>, String> {
    let &i = cfg.index.get(id).ok_or(format!("нет записи {id}"))?;
    if crate::container::content(&cfg.entries[i]) == rec {
        return Ok(None);
    }
    let before = cfg.entries[i].data.clone();
    let after = crate::container::deflate(rec);
    put(cfg, id, &after)?;
    Ok(Some(Change { entry: id.to_string(), before, after }))
}

/// Объект `id` (верхнего уровня) в составе подсистемы или функциональной опции `holder`: включить (`on`) или
/// исключить — правка `<Content>` описания `holder` через загрузку. Новый элемент — в конец состава.
pub fn content(cfg: &mut Config, names: &Names, holder: &str, id: &str, on: bool) -> Result<Option<Change>, String> {
    let o = find(cfg, id).ok_or("объект не найден")?;
    let full = format!("{}.{}", o.class, o.item.header.name);
    let h = find(cfg, holder).ok_or("подсистема не найдена")?;
    let item = match h.class {
        "Subsystem" => "<xr:Item xsi:type=\"xr:MDObjectRef\">",
        "FunctionalOption" => "<xr:Object>",
        c => return Err(format!("{c}: не подсистема и не функциональная опция")),
    };
    let close = if h.class == "Subsystem" { "</xr:Item>" } else { "</xr:Object>" };
    let xml = export(cfg, names, &h)?;
    let props = props_of(&xml, h.class, None)?;
    let p = props.iter().find(|p| p.tag == "Content").ok_or("в описании нет <Content>")?;
    let el = dom::parse(&xml[p.span.clone()])?;
    let mut items: Vec<String> = el.elems().map(|e| e.text()).collect::<Result<_, _>>()?;
    let has = items.contains(&full);
    if has == on {
        return Ok(None);
    }
    if on {
        items.push(full);
    } else {
        items.retain(|x| *x != full);
    }
    let esc = |t: &str| t.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;");
    let ind = &p.indent;
    let frag = if items.is_empty() { "<Content/>".to_string() } else { format!("<Content>{}\r\n{ind}</Content>", items.iter().map(|i| format!("\r\n{ind}\t{item}{}{close}", esc(i))).collect::<String>()) };
    let new = format!("{}{frag}{}", &xml[..p.span.start], &xml[p.span.end..]);
    let rec = import(&new, names, &h)?;
    let entry = h.entry;
    store(cfg, &entry, &rec)
}

/// Права справочника по порядку списка «Права» Конфигуратора и что отмечается вместе с каждым — снято с
/// Конфигуратора 8.5.1.1529 перебором (редактор роли: отметить право, посмотреть флажки, снять «Чтение» — снимаются
/// все; 27.10.2026). Снятое право снимает те, что от него зависят (строки, где оно есть).
pub const CATALOG_RIGHTS: &[(&str, &[&str])] = &[
    ("Read", &[]),
    ("Insert", &["Read"]),
    ("Update", &["Read"]),
    ("Delete", &["Read"]),
    ("View", &["Read"]),
    ("InteractiveInsert", &["Read", "Insert", "Update", "View", "Edit"]),
    ("Edit", &["Read", "Update", "View"]),
    ("InteractiveDelete", &["Read", "Update", "Delete", "View", "Edit"]),
    ("InteractiveSetDeletionMark", &["Read", "Update", "View", "Edit"]),
    ("InteractiveClearDeletionMark", &["Read", "Update", "View", "Edit"]),
    ("InteractiveDeleteMarked", &["Read", "Update", "Delete", "View", "Edit"]),
    ("InputByString", &["Read", "View"]),
    ("InteractiveDeletePredefinedData", &["Read", "Update", "Delete", "View", "Edit", "InteractiveDelete"]),
    ("InteractiveSetDeletionMarkPredefinedData", &["Read", "Update", "View", "Edit", "InteractiveSetDeletionMark"]),
    ("InteractiveClearDeletionMarkPredefinedData", &["Read", "Update", "View", "Edit", "InteractiveClearDeletionMark"]),
    ("InteractiveDeleteMarkedPredefinedData", &["Read", "Update", "Delete", "View", "Edit", "InteractiveDeleteMarked"]),
    ("ReadDataHistory", &["Read"]),
    ("ReadDataHistoryOfMissingData", &["Read", "ReadDataHistory"]),
    ("UpdateDataHistory", &["Read", "ReadDataHistory"]),
    ("UpdateDataHistoryOfMissingData", &["Read", "ReadDataHistory", "ReadDataHistoryOfMissingData", "UpdateDataHistory"]),
    ("UpdateDataHistorySettings", &[]),
    ("UpdateDataHistoryVersionComment", &["Read", "ReadDataHistory"]),
    ("ViewDataHistory", &["Read", "View", "ReadDataHistory"]),
    ("EditDataHistoryVersionComment", &["Read", "View", "ReadDataHistory", "UpdateDataHistoryVersionComment"]),
    ("SwitchToDataHistoryVersion", &["Read", "View"]),
];

/// Отмеченные права объекта `full` в роли `role` (запись прав) — по Ext/Rights.xml.
pub fn rights_of(cfg: &Config, role: &str, full: &str) -> Result<Vec<String>, String> {
    let xml = role_xml(cfg, role)?;
    Ok(object_span(&xml, full).map(|(s, e)| right_blocks(&xml[s..e]).into_iter().filter(|b| b.1).map(|b| b.0).collect()).unwrap_or_default())
}

fn role_xml(cfg: &Config, role: &str) -> Result<String, String> {
    let text = cfg.entry_text(role).ok_or(format!("нет записи прав {role}"))?;
    String::from_utf8(super::rights::xml(cfg, &text, false)?).map_err(|e| e.to_string())
}

/// Блок `<object>` объекта `full` в Rights.xml: от `\t<object>` до конца `\t</object>\r\n`.
fn object_span(xml: &str, full: &str) -> Option<(usize, usize)> {
    let at = xml.find(&format!("\t\t<name>{full}</name>\r\n"))?;
    let s = xml[..at].rfind("\t<object>\r\n")?;
    let e = at + xml[at..].find("\t</object>\r\n")? + "\t</object>\r\n".len();
    Some((s, e))
}

/// Права блока объекта: (имя, значение, текст `\t\t<right>…</right>\r\n`, только имя и значение).
fn right_blocks(obj: &str) -> Vec<(String, bool, String, bool)> {
    let mut out = Vec::new();
    let mut i = 0;
    while let Some(s) = obj[i..].find("\t\t<right>\r\n").map(|k| i + k) {
        let Some(e) = obj[s..].find("\t\t</right>\r\n").map(|k| s + k + "\t\t</right>\r\n".len()) else { break };
        let b = &obj[s..e];
        let name = b.split("<name>").nth(1).and_then(|r| r.split("</name>").next()).unwrap_or_default().to_string();
        let value = b.contains("<value>true</value>");
        out.push((name, value, b.to_string(), b.matches('\n').count() == 4));
        i = e;
    }
    out
}

/// Права вида объекта по порядку списка «Права», если их зависимости известны целиком (графы прав платформы,
/// `imp::rights::kind_deps`: справочник, документ, регистры, константа, отчет, обработка…); иначе — пусто.
pub fn right_names(class: &str) -> Vec<&'static str> {
    super::imp::rights::kind_deps(class).map(|(n, _)| n.to_vec()).unwrap_or_default()
}

/// Право `right` объекта `id` в роли `role` (запись прав `<роль>.0`): отметить или снять — как флажок «Права»
/// Конфигуратора: вместе отмечается то, от чего право зависит, и снимается то, что зависит от него (графы прав
/// платформы; у справочника сверено с перебором в Конфигураторе — [`CATALOG_RIGHTS`], тест `rights_like_designer`).
/// Ext/Rights.xml пишет только отмеченные права (так пишет Конфигуратор), запись — загрузкой.
pub fn set_right(cfg: &mut Config, names: &Names, role: &str, id: &str, right: &str, on: bool) -> Result<Option<Change>, String> {
    let o = find(cfg, id).ok_or("объект не найден")?;
    let (names_k, rows) = super::imp::rights::kind_deps(o.class).filter(|_| o.sub.is_none()).ok_or(format!("права {}: зависимости прав вида не известны", o.class))?;
    let full = format!("{}.{}", o.class, o.item.header.name);
    let k = names_k.iter().position(|r| *r == right).ok_or(format!("у {} нет права {right}", o.class))?;
    let xml = role_xml(cfg, role)?;
    if xml.contains("<setForNewObjects>true</setForNewObjects>") {
        return Err("роль с «Устанавливать права для новых объектов»: снятие прав у Конфигуратора не сверено".into());
    }
    let span = object_span(&xml, &full);
    let blocks = span.map(|(s, e)| right_blocks(&xml[s..e])).unwrap_or_default();
    if blocks.iter().any(|b| !b.3) {
        return Err(format!("{full}: у прав есть ограничения доступа — их правка не поддержана"));
    }
    let mut set: Vec<&str> = blocks.iter().filter(|b| b.1).map(|b| b.0.as_str()).collect();
    // строка F, символ R: «+» — R зависит от F (снятие F снимает R)
    let dep = |f: usize, r: usize| rows[f].as_bytes()[r] == b'+';
    if on {
        set.extend((0..names_k.len()).filter(|&f| dep(f, k)).map(|f| names_k[f]).chain([right]));
    } else {
        set.retain(|r| names_k.iter().position(|n| n == r).is_none_or(|i| i != k && !dep(k, i)));
    }
    // по порядку списка Конфигуратора, как в его Rights.xml
    let rights: Vec<&str> = names_k.iter().copied().filter(|r| set.contains(r)).collect();
    let block = if rights.is_empty() {
        String::new()
    } else {
        let body: String = rights.iter().map(|r| format!("\t\t<right>\r\n\t\t\t<name>{r}</name>\r\n\t\t\t<value>true</value>\r\n\t\t</right>\r\n")).collect();
        format!("\t<object>\r\n\t\t<name>{full}</name>\r\n{body}\t</object>\r\n")
    };
    // новый объект — перед первым объектом (у Конфигуратора — перед Configuration; порядок записи — загрузки)
    let new = match span {
        Some((s, e)) => format!("{}{block}{}", &xml[..s], &xml[e..]),
        None => {
            let at = xml.find("\t<object>\r\n").or_else(|| xml.rfind("</Rights>")).ok_or("Rights.xml без </Rights>")?;
            format!("{}{block}{}", &xml[..at], &xml[at..])
        }
    };
    if new == xml {
        return Ok(None);
    }
    let rec = match super::imp::rights::record(&dom::parse(&new)?, &names.imp)? {
        super::imp::Rec::Raw(b) => b,
        super::imp::Rec::Dir(_) => return Err("права роли: запись не одна".into()),
    };
    let &i = cfg.index.get(role).ok_or(format!("нет записи {role}"))?;
    if crate::container::content(&cfg.entries[i]) == rec {
        return Ok(None);
    }
    let ch = Change { entry: role.to_string(), before: cfg.entries[i].data.clone(), after: crate::container::deflate(&rec) };
    apply(cfg, std::slice::from_ref(&ch), false)?;
    Ok(Some(ch))
}

/// Данные записи как есть (отмена, повтор) и заголовок объекта в дереве по ним.
/// Вложенные элементы (реквизиты, табличные части…) — из той же записи заново: их имена могли измениться.
pub fn put(cfg: &mut Config, entry: &str, data: &[u8]) -> Result<(), String> {
    let &i = cfg.index.get(entry).ok_or(format!("нет записи {entry}"))?;
    cfg.entries[i].data = data.to_vec();
    let text = crate::brace::text(&crate::container::content(&cfg.entries[i]));
    let node = crate::brace::parse(&text)?;
    let Some(h) = crate::meta::header(&node) else {
        // запись не объекта дерева (права роли…) — заголовка нет, дерево то же
        fn has(items: &[Item], id: &str) -> bool {
            items.iter().any(|i| i.header.id == id || has(&i.children, id))
        }
        if entry != cfg.root_id && !cfg.groups.iter().any(|g| has(&g.items, entry)) {
            return Ok(());
        }
        return Err("в записи нет заголовка".into());
    };
    if entry == cfg.root_id {
        cfg.root.header = crate::meta::Header { id: cfg.root.header.id.clone(), ..h };
        // у внешней обработки реквизиты и ТЧ — в записи корня
        if cfg.root.kind != "Configuration" {
            let mut kids = Vec::new();
            cfg.children_in(&node, &mut kids);
            cfg.root.children = kids;
        }
        return Ok(());
    }
    let kids = cfg.object(entry, "").map(|o| o.children).unwrap_or_default();
    fn walk(it: &mut Item, id: &str, h: &crate::meta::Header, kids: &mut Option<Vec<Item>>) -> bool {
        if it.header.id == id {
            it.header = crate::meta::Header { id: it.header.id.clone(), ..h.clone() };
            it.children = kids.take().unwrap_or_default();
            return true;
        }
        it.children.iter_mut().any(|c| walk(c, id, h, kids))
    }
    let mut kids = Some(kids);
    let found = cfg.groups.iter_mut().flat_map(|g| g.items.iter_mut()).any(|it| walk(it, entry, &h, &mut kids));
    if !found {
        return Err(format!("объект {entry} не найден в дереве"));
    }
    Ok(())
}

/// Записи по списку замен (пустые данные — записи нет): `undo` — данные «до» в обратном порядке. Запись добавлена
/// или удалена — дерево строится из записей заново, иначе обновляются заголовки, как у [`put`].
pub fn apply(cfg: &mut Config, changes: &[Change], undo: bool) -> Result<(), String> {
    let data = |c: &Change| if undo { c.before.clone() } else { c.after.clone() };
    let structural = changes.iter().any(|c| c.before.is_empty() || c.after.is_empty());
    if !structural {
        for c in changes {
            put(cfg, &c.entry, &data(c))?;
        }
        return Ok(());
    }
    let order: Vec<&Change> = if undo { changes.iter().rev().collect() } else { changes.iter().collect() };
    let stamp = cfg.index.get(&cfg.root_id).map(|&i| (cfg.entries[i].created, cfg.entries[i].modified)).unwrap_or_default();
    for c in order {
        let d = data(c);
        match (cfg.index.get(&c.entry).copied(), d.is_empty()) {
            (Some(i), false) => cfg.entries[i].data = d,
            (Some(i), true) => {
                cfg.entries.remove(i);
                cfg.index = cfg.entries.iter().enumerate().map(|(i, e)| (e.name.clone(), i)).collect();
            }
            (None, false) => {
                cfg.index.insert(c.entry.clone(), cfg.entries.len());
                cfg.entries.push(crate::container::Entry { name: c.entry.clone(), created: stamp.0, modified: stamp.1, data: d });
            }
            (None, true) => {}
        }
    }
    // разбор корня — единственный отказ сборки дерева: проверить до того, как записи уйдут в нее
    let root = cfg.entry_text(&cfg.root_id).ok_or("нет записи корня")?;
    crate::brace::parse(&root)?;
    let entries = std::mem::take(&mut cfg.entries);
    let mut n = Config::from_entries(entries, cfg.parts.clone())?;
    n.other_parts = std::mem::take(&mut cfg.other_parts);
    n.main_part = cfg.main_part;
    *cfg = n;
    Ok(())
}

/// Пустой модуль роли `role` («ObjectModule») у объекта `id`, которого у объекта еще нет: запись `<объект>.<суффикс>`
/// с текстом из одного BOM — как «Открыть модуль объекта» у новой внешней обработки в Конфигураторе.
pub fn new_module(cfg: &Config, id: &str, role: &str) -> Result<Change, String> {
    let o = find(cfg, id).filter(|o| o.sub.is_none()).ok_or(format!("нет объекта {id}"))?;
    const SUFFIXES: &[&str] = &["0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "a", "b", "c"];
    let suffix = SUFFIXES.iter().find(|s| crate::meta::own_role(o.class, s) == Some(role)).ok_or(format!("у {} нет модуля {role}", o.class))?;
    let entry = format!("{}.{suffix}", o.entry);
    if cfg.index.contains_key(&entry) {
        return Err(format!("модуль {role} уже есть"));
    }
    let super::imp::Rec::Dir(v) = super::imp::module::record(b"\xef\xbb\xbf")? else { return Err("модуль — не каталог".into()) };
    let (created, modified) = cfg.index.get(&o.entry).map(|&i| (cfg.entries[i].created, cfg.entries[i].modified)).unwrap_or_default();
    let data = crate::container::write(&v.into_iter().map(|(name, data)| crate::container::Entry { name, created, modified, data }).collect::<Vec<_>>());
    Ok(Change { entry, before: Vec::new(), after: crate::container::deflate(&data) })
}

/// Эталоны новых объектов —`ibcmd config export` объектов, добавленных в Конфигураторе 8.5.1.1529 клавишей Insert
/// с именами по умолчанию (вложенные — из `ChildObjects` того же эталона). Без эталона добавление — отказ.
const NEW: &[(&str, &str)] = &[
    ("Catalog", include_str!("new/Catalog.xml")),
    ("Document", include_str!("new/Document.xml")),
    ("Enum", include_str!("new/Enum.xml")),
    ("Constant", include_str!("new/Constant.xml")),
    ("Report", include_str!("new/Report.xml")),
    ("DataProcessor", include_str!("new/DataProcessor.xml")),
    ("ChartOfCharacteristicTypes", include_str!("new/ChartOfCharacteristicTypes.xml")),
    ("InformationRegister", include_str!("new/InformationRegister.xml")),
    ("AccumulationRegister", include_str!("new/AccumulationRegister.xml")),
    ("Subsystem", include_str!("new/Subsystem.xml")),
    ("CommonModule", include_str!("new/CommonModule.xml")),
];
/// Модуль команды справочника, который Конфигуратор создает вместе с командой.
const COMMAND_MODULE: &str = include_str!("new/CommandModule.bsl");

/// Текст эталона с CRLF (git может хранить LF).
fn template(class: &str) -> Result<String, String> {
    // эталон обработки — без вложенных: реквизиты и ТЧ внешней обработки — справочника без его свойств, типы ТЧ —
    // DataProcessorTabularSection, у ТЧ — стандартный реквизит НомерСтроки (так у всех 137 ТЧ обработок ЭДО)
    if class == "ExternalDataProcessor" {
        let own = ["<Use>", "<Indexing>", "<FullTextSearch>", "<DataHistory>", "<LineNumberLength>"];
        let t = template("Catalog")?;
        let mut t = t.split("\r\n").filter(|l| !own.iter().any(|p| l.trim_start().starts_with(p))).collect::<Vec<_>>().join("\r\n").replace("CatalogTabularSection", "DataProcessorTabularSection");
        let ts = t.find("<TabularSection ").ok_or("в эталоне нет табличной части")?;
        let end = ts + t[ts..].find("\t\t\t\t</Properties>").ok_or("в эталоне нет свойств табличной части")?;
        t.insert_str(end, &include_str!("new/TabularSectionLineNumber.xml").replace("\r\n", "\n").replace('\n', "\r\n"));
        return Ok(t);
    }
    let t = NEW.iter().find(|(c, _)| *c == class).map(|(_, t)| *t).ok_or(format!("добавление {class}: нет эталона Конфигуратора"))?;
    Ok(t.replace("\r\n", "\n").replace('\n', "\r\n"))
}

/// Есть ли эталон для «Добавить»: объект класса (`kind` нет) или его вложенный элемент вида `kind` (команда — у
/// справочника: эталон ее модуля снят у него).
pub fn can_add(class: &str, kind: Option<&str>) -> bool {
    // форма — «Конструктор формы» справочника и документа (`add_form`), макет — у любого владельца (`add_template`)
    match kind {
        Some("Form") => return matches!(class, "Catalog" | "Document"),
        Some("Template") => return !matches!(class, "Configuration" | "" ),
        // внешняя обработка — единственный объект файла
        None if class == "ExternalDataProcessor" => return false,
        _ => {}
    }
    template(class).is_ok_and(|t| kind.is_none_or(|k| t.contains(&format!("\r\n\t\t\t<{k} uuid=")) && (k != "Command" || class == "Catalog")))
}

/// Случайный uuid v4 (ключи `RandomState` — из случайности ОС).
fn uuid4() -> String {
    use std::hash::{BuildHasher, Hasher};
    let r = || std::collections::hash_map::RandomState::new().build_hasher().finish();
    let (a, b) = (r(), r());
    let b = (b & 0x3fff_ffff_ffff_ffff) | 0x8000_0000_0000_0000;
    format!("{:08x}-{:04x}-4{:03x}-{:04x}-{:012x}", a >> 32, (a >> 16) & 0xffff, a & 0xfff, b >> 48, b & 0xffff_ffff_ffff)
}

/// Все uuid текста — новые: одинаковые одинаково, нулевой как был.
fn fresh_uuids(t: &str) -> String {
    let is = |s: &[u8]| s.len() == 36 && s.iter().enumerate().all(|(i, c)| if matches!(i, 8 | 13 | 18 | 23) { *c == b'-' } else { c.is_ascii_hexdigit() });
    let mut map: std::collections::HashMap<&str, String> = std::collections::HashMap::new();
    let (b, mut out, mut i) = (t.as_bytes(), String::with_capacity(t.len()), 0);
    while i < b.len() {
        if i + 36 <= b.len() && is(&b[i..i + 36]) && t.is_char_boundary(i) {
            let u = &t[i..i + 36];
            out += if u.bytes().all(|c| c == b'0' || c == b'-') { u } else { map.entry(u).or_insert_with(uuid4) };
            i += 36;
        } else {
            let c = t[i..].chars().next().unwrap();
            out.push(c);
            i += c.len_utf8();
        }
    }
    out
}

/// Имя по умолчанию: основа имени эталона и первый свободный номер (`Справочник1`, `Реквизит2`).
fn free_name(stem: &str, taken: &[String]) -> String {
    (1..).map(|n| format!("{stem}{n}")).find(|n| !taken.iter().any(|t| t.to_lowercase() == n.to_lowercase())).unwrap()
}

fn first_name(t: &str) -> Result<&str, String> {
    t.split_once("<Name>").and_then(|(_, r)| r.split_once("</Name>")).map(|(n, _)| n).ok_or("в эталоне нет <Name>".into())
}

/// Элемент `<tag …>` с отступом `ind`: (начало строки, конец элемента).
fn element_at(t: &str, ind: &str, tag: &str) -> Option<(usize, usize)> {
    let s = t.find(&format!("\r\n{ind}<{tag}"))? + 2;
    Some((s, element_end(t, s + ind.len())?))
}

/// `ChildObjects` с отступом `ind` пустым (`<ChildObjects/>`).
fn without_children(t: &str, ind: &str) -> String {
    match element_at(t, ind, "ChildObjects>") {
        Some((s, e)) => format!("{}{ind}<ChildObjects/>{}", &t[..s], &t[e..]),
        None => t.to_string(),
    }
}

/// Элемент в конец `ChildObjects` с отступом `ind`, начиная с `from`.
fn insert_child(t: &str, from: usize, ind: &str, el: &str) -> Result<String, String> {
    let rel = &t[from..];
    let empty = rel.find(&format!("\r\n{ind}<ChildObjects/>"));
    let full = element_at(rel, ind, "ChildObjects>");
    // первый из двух — свой (дальше могут быть ChildObjects следующих табличных частей)
    match (empty, full) {
        (Some(p), f) if f.is_none_or(|(s, _)| p + 2 < s) => {
            let s = from + p + 2;
            Ok(format!("{}{ind}<ChildObjects>\r\n{el}\r\n{ind}</ChildObjects>{}", &t[..s], &t[s + ind.len() + "<ChildObjects/>".len()..]))
        }
        (_, Some((_, e))) => {
            let close = from + e - "</ChildObjects>".len() - ind.len() - 2;
            Ok(format!("{}\r\n{el}{}", &t[..close], &t[close..]))
        }
        _ => Err("в описании нет ChildObjects".into()),
    }
}

/// Соседи элемента `id` того же вида в `ChildObjects` описания владельца: (владелец, его XML, места соседей, номер
/// элемента). Вложенный элемент — `<Вид uuid=…>…</Вид>`; форма, макет — строка `<Form>Имя</Form>` владельца; объект
/// верхнего уровня — строка `<Catalog>Имя</Catalog>` корня.
fn siblings<'a>(cfg: &'a Config, names: &Names, id: &str) -> Result<(Obj<'a>, String, Vec<(usize, usize)>, usize), String> {
    let o = find(cfg, id).ok_or(format!("нет объекта {id}"))?;
    let owner = if o.sub.is_some() {
        None
    } else if o.owner.is_some() {
        Some(cfg.groups.iter().flat_map(|g| &g.items).find(|it| it.children.iter().any(|c| c.header.id == id)).ok_or("нет владельца")?.header.id.clone())
    } else if o.full.is_none() && o.class != "Configuration" {
        Some("root".to_string())
    } else {
        return Err("подсистемы и корень не перемещаются".into());
    };
    let (holder, line_kind) = match &owner {
        Some(h) => (find(cfg, h).ok_or("нет владельца")?, Some(o.class)),
        None => (find(cfg, &o.entry).ok_or("нет владельца")?, None),
    };
    let holder = Obj { sub: None, ..holder };
    let xml = export(cfg, names, &holder)?;
    let (line, ind, head) = match line_kind {
        Some(kind) => {
            let head = format!("\n\t\t\t<{kind}>");
            let at = xml.find(&format!("{head}{}</{kind}>", o.item.header.name)).ok_or("нет элемента в описании")? + 1;
            (at, "\t\t\t".to_string(), head)
        }
        None => {
            let at = xml.find(&format!(" uuid=\"{id}\"")).ok_or("нет элемента в описании")?;
            let start = xml[..at].rfind('<').ok_or("нет элемента в описании")?;
            let line = xml[..start].rfind('\n').map_or(0, |n| n + 1);
            let ind = xml[line..start].to_string();
            let head = format!("\n{ind}<{} uuid=\"", &xml[start + 1..at]);
            (line, ind, head)
        }
    };
    let parent = ind.strip_suffix('\t').ok_or("нет ChildObjects")?;
    let open = xml[..line].rfind(&format!("\n{parent}<ChildObjects>")).ok_or("нет ChildObjects")? + 1;
    let close = element_end(&xml, open + parent.len()).ok_or("нет конца ChildObjects")?;
    let mut spans = Vec::new();
    let mut i = open;
    while let Some(k) = xml[i..close].find(&head) {
        let s = i + k + 1;
        let e = element_end(&xml, s + ind.len()).ok_or("нет конца элемента")?;
        spans.push((s, e));
        i = e;
    }
    let me = spans.iter().position(|&(s, _)| s == line).ok_or("нет элемента среди соседей")?;
    Ok((holder, xml, spans, me))
}

/// Описание владельца `o` после правки → замена его записи (через загрузку).
fn holder_change(cfg: &Config, names: &Names, o: &Obj, xml: &str) -> Result<Change, String> {
    let rec = import(xml, names, o)?;
    let &oi = cfg.index.get(&o.entry).ok_or(format!("нет записи {}", o.entry))?;
    Ok(Change { entry: o.entry.clone(), before: cfg.entries[oi].data.clone(), after: crate::container::deflate(&rec) })
}

/// Элемент `id` на место соседа того же вида выше (`up`) или ниже: обмен местами в `ChildObjects` владельца через
/// загрузку — вложенный элемент, форма, макет, объект верхнего уровня (как «Переместить вверх/вниз» Конфигуратора:
/// у форм — снимок 27.10.2026). Крайний элемент — `None`.
pub fn move_child(cfg: &Config, names: &Names, id: &str, up: bool) -> Result<Option<Change>, String> {
    let (o, xml, spans, me) = siblings(cfg, names, id)?;
    let Some(other) = (if up { me.checked_sub(1) } else { Some(me + 1).filter(|&k| k < spans.len()) }) else { return Ok(None) };
    let ((a, ae), (b, be)) = (spans[me.min(other)], spans[me.max(other)]);
    let xml = format!("{}{}{}{}{}", &xml[..a], &xml[b..be], &xml[ae..b], &xml[a..ae], &xml[be..]);
    holder_change(cfg, names, &o, &xml).map(Some)
}

/// «Сортировка метаданных» Конфигуратора (снимок 27.10.2026): элементы вида элемента `id` у его владельца — по имени,
/// синониму (русскому) или комментарию (`by`: Name, Synonym, Comment), по возрастанию или убыванию; места в
/// `ChildObjects` те же, порядок — новый. Порядок не изменился — `None`.
/// shortcut: сравнение строк без учета регистра (`to_lowercase`), правило Конфигуратора для равных и «ё» не сверено.
pub fn sort_children(cfg: &Config, names: &Names, id: &str, by: &str, desc: bool) -> Result<Option<Change>, String> {
    let (o, xml, spans, _) = siblings(cfg, names, id)?;
    let key = |s: usize, e: usize| -> Result<String, String> {
        let el = &xml[s..e];
        // строка формы, макета, объекта корня — свое описание; вложенный — свойства в элементе
        let text = match el.split_once(" uuid=\"") {
            Some((_, r)) => {
                let u = r.split('"').next().unwrap_or_default();
                let x = find(cfg, u).ok_or("нет элемента")?;
                let props = props_of(&xml, x.kind(), x.sub.map(|s| s.header.id.as_str()))?;
                props
            }
            None => {
                let name = el.split('>').nth(1).and_then(|r| r.split('<').next()).unwrap_or_default();
                let tag = el.trim_start().trim_start_matches('<').split('>').next().unwrap_or_default();
                let item = if o.class == "Configuration" {
                    cfg.groups.iter().filter(|g| g.class == tag).flat_map(|g| &g.items).find(|i| i.header.name == name)
                } else {
                    o.item.children.iter().find(|c| c.kind == tag && c.header.name == name)
                };
                let x = item.and_then(|i| find(cfg, &i.header.id)).ok_or(format!("нет объекта {name}"))?;
                props_of(&export(cfg, names, &x)?, x.kind(), None)?
            }
        };
        Ok(text.iter().find(|p| p.tag == by).map(|p| match &p.value {
            Value::Ml(items) => items.iter().find(|i| i.0 == "ru").or(items.first()).map(|i| i.1.clone()).unwrap_or_default(),
            _ => p.shown(),
        }).unwrap_or_default().to_lowercase())
    };
    let mut order: Vec<(String, usize)> = spans.iter().enumerate().map(|(k, &(s, e))| key(s, e).map(|t| (t, k))).collect::<Result<_, _>>()?;
    order.sort_by(|a, b| if desc { b.0.cmp(&a.0) } else { a.0.cmp(&b.0) });
    if order.iter().enumerate().all(|(k, x)| x.1 == k) {
        return Ok(None);
    }
    let mut out = String::new();
    let mut at = 0;
    for (slot, (_, k)) in spans.iter().zip(&order) {
        out.push_str(&xml[at..slot.0]);
        out.push_str(&xml[spans[*k].0..spans[*k].1]);
        at = slot.1;
    }
    out.push_str(&xml[at..]);
    holder_change(cfg, names, &o, &out).map(Some)
}

/// Новый объект верхнего уровня класса `class` по эталону Конфигуратора: (uuid, замены записей — объект и корень).
/// `names` дополняется новым именем (после применения замен индекс строится заново).
pub fn add(cfg: &Config, names: &mut Names, class: &'static str) -> Result<(String, Vec<Change>), String> {
    let t = template(class)?;
    let old = first_name(&t)?.to_string();
    let stem = old.trim_end_matches(|c: char| c.is_ascii_digit());
    let taken: Vec<String> = cfg.groups.iter().filter(|g| g.class == class).flat_map(|g| g.items.iter().map(|i| i.header.name.clone())).collect();
    let name = free_name(stem, &taken);
    let xml = fresh_uuids(&without_children(&t, "\t\t").replace(&old, &name));
    let id = xml.split_once(&format!("<{class} uuid=\"")).and_then(|(_, r)| r.split_once('"')).map(|(u, _)| u.to_string()).ok_or("в эталоне нет uuid объекта")?;
    names.imp.full.insert(format!("{class}.{name}"), id.clone());
    let rec = desc::record(&dom::parse(&xml)?, class, None, None, &names.imp)?.ok_or(format!("загрузка описания {class} не поддержана"))?;
    // корень: имя в ChildObjects (порядок в записи — по классам, внутри класса — по XML: новый последним)
    let root = find(cfg, "root").ok_or("нет корня")?;
    let rx = export(cfg, names, &root)?;
    let own = format!("\r\n\t\t\t<{class}>");
    let el = format!("\t\t\t<{class}>{}</{class}>", super::w::escape(&name));
    let rx = match rx.rfind(&own).and_then(|p| rx[p + 2..].find("\r\n").map(|q| p + 2 + q)) {
        Some(e) => format!("{}\r\n{el}{}", &rx[..e], &rx[e..]),
        None => insert_child(&rx, 0, "\t\t", &el)?,
    };
    let rrec = import(&rx, names, &root)?;
    let &ri = cfg.index.get(&cfg.root_id).ok_or("нет записи корня")?;
    Ok((
        id.clone(),
        vec![
            Change { entry: id, before: Vec::new(), after: crate::container::deflate(&rec) },
            Change { entry: cfg.root_id.clone(), before: cfg.entries[ri].data.clone(), after: crate::container::deflate(&rrec) },
        ],
    ))
}

/// Новый вложенный элемент вида `kind` (реквизит, табличная часть, команда, значение перечисления, измерение,
/// ресурс) у объекта `owner` (uuid объекта или его табличной части) по эталону Конфигуратора: (uuid, замены записей).
pub fn add_child(cfg: &Config, names: &mut Names, owner: &str, kind: &str) -> Result<(String, Vec<Change>), String> {
    let o = find(cfg, owner).ok_or(format!("нет объекта {owner}"))?;
    if kind == "Template" && o.sub.is_none() && o.owner.is_none() {
        return add_template(cfg, names, &o, "SpreadsheetDocument", None);
    }
    let in_ts = match o.sub {
        Some(s) if s.kind == "TabularSection" => true,
        None if o.owner.is_none() && o.full.is_none() && o.class != "Configuration" => false,
        _ => return Err("вложенные элементы добавляются в объект или его табличную часть".into()),
    };
    let t = template(o.class)?;
    let (owner_old, ind) = (first_name(&t)?.to_string(), if in_ts { "\t\t\t\t\t" } else { "\t\t\t" });
    let no = || format!("{kind} в {}: нет эталона Конфигуратора", if in_ts { "табличной части" } else { o.class });
    let scope = if in_ts { element_at(&t, "\t\t\t", "TabularSection ").ok_or_else(no)? } else { (0, t.len()) };
    let (s, e) = element_at(&t[scope.0..scope.1], ind, &format!("{kind} ")).ok_or_else(no)?;
    let mut el = t[scope.0 + s..scope.0 + e].to_string();
    if kind == "TabularSection" {
        el = without_children(&el, "\t\t\t\t");
    }
    let old = first_name(&el)?.to_string();
    // имена соседей того же уровня (всех видов): shortcut — Конфигуратор может проверять уникальность иначе (с
    // реквизитами владельца у реквизита табличной части); тогда номер выйдет больше, чем у него
    let level = o.sub.unwrap_or(o.item);
    let taken: Vec<String> = level.children.iter().map(|c| c.header.name.clone()).collect();
    let name = free_name(old.trim_end_matches(|c: char| c.is_ascii_digit()), &taken);
    let owner_name = o.item.header.name.clone();
    el = el.replace(&old, &name).replace(&format!(".{owner_old}."), &format!(".{owner_name}."));
    let ts_name = o.sub.map(|s| s.header.name.clone());
    if let Some(ts) = &ts_name {
        let ts_old = first_name(&t[scope.0..scope.1])?;
        el = el.replace(&format!(".{ts_old}."), &format!(".{ts}."));
    }
    let el = fresh_uuids(&el);
    let id = el.split_once(" uuid=\"").and_then(|(_, r)| r.split_once('"')).map(|(u, _)| u.to_string()).ok_or("в эталоне нет uuid элемента")?;
    let xml = export(cfg, names, &o)?;
    let from = match o.sub {
        Some(s) => xml.find(&format!(" uuid=\"{}\"", s.header.id)).ok_or("нет табличной части в описании")?,
        None => 0,
    };
    let xml = insert_child(&xml, from, &ind[..ind.len() - 1], &el)?;
    let rec = import(&xml, names, &o)?;
    let &oi = cfg.index.get(&o.entry).ok_or(format!("нет записи {}", o.entry))?;
    let mut ch = vec![Change { entry: o.entry.clone(), before: cfg.entries[oi].data.clone(), after: crate::container::deflate(&rec) }];
    // команда — с модулем, как у Конфигуратора (эталон — у справочника)
    if kind == "Command" {
        if o.class != "Catalog" {
            return Err(format!("модуль новой команды {}: нет эталона Конфигуратора", o.class));
        }
        let text = COMMAND_MODULE.replace("\r\n", "\n").replace('\n', "\r\n").replace(&format!(".{owner_old}."), &format!(".{owner_name}."));
        let data = match super::imp::module::record(text.as_bytes())? {
            super::imp::Rec::Raw(b) => b,
            super::imp::Rec::Dir(v) => crate::container::write(&v.into_iter().map(|(name, data)| crate::container::Entry { name, created: cfg.entries[oi].created, modified: cfg.entries[oi].modified, data }).collect::<Vec<_>>()),
        };
        ch.push(Change { entry: format!("{id}.2"), before: Vec::new(), after: crate::container::deflate(&data) });
    }
    Ok((id, ch))
}

/// Строка `<Вид>Имя</Вид>` в `ChildObjects` описания владельца — за последней строкой того же вида или вида до него
/// (порядок выгрузки: Attribute, TabularSection, Form, Template, Command…), иначе перед первым элементом вида после.
fn insert_line(xml: &str, kind: &str, name: &str) -> Result<String, String> {
    const ORDER: &[&str] = &["Form", "Template"];
    let line = format!("\t\t\t<{kind}>{}</{kind}>", super::w::escape(name));
    let me = ORDER.iter().position(|k| *k == kind).ok_or("вид строки")?;
    // за последней строкой своего вида или предыдущих
    for k in ORDER[..=me].iter().rev() {
        if let Some(p) = xml.rfind(&format!("\r\n\t\t\t<{k}>")) {
            let e = p + 2 + xml[p + 2..].find("\r\n").ok_or("описание владельца")?;
            return Ok(format!("{}\r\n{line}{}", &xml[..e], &xml[e..]));
        }
    }
    // форм и макетов нет: за последним реквизитом или табличной частью верхнего уровня, иначе в начало ChildObjects
    let open = xml.find("\r\n\t\t<ChildObjects>").map(|p| p + 2);
    let Some(open) = open else { return insert_child(xml, 0, "\t\t", &line) };
    let close = element_end(xml, open).ok_or("описание владельца")?;
    let mut at = open + "\t\t<ChildObjects>".len();
    let mut i = at;
    while let Some(k) = xml[i..close].find("\r\n\t\t\t<") {
        let s = i + k + 2;
        let tag = xml[s + 4..].split([' ', '>']).next().unwrap_or_default();
        if !matches!(tag, "Attribute" | "TabularSection") {
            break;
        }
        at = element_end(xml, s + 3).ok_or("описание владельца")?;
        i = at;
    }
    Ok(format!("{}\r\n{line}{}", &xml[..at], &xml[at..]))
}

/// Типы «Конструктора макета» (Active document — объект OLE с машины, не делается) и запись содержимого пустого
/// макета, как ее пишет Конфигуратор 8.5 (эталоны 27.10.2026; у табличного документа, двоичных данных, HTML и внешней
/// компоненты записи нет).
pub const TEMPLATE_TYPES: &[(&str, &str, Option<&[u8]>)] = &[
    ("SpreadsheetDocument", "Табличный документ", None),
    ("TextDocument", "Текстовый документ", Some(include_bytes!("new/templates/TextDocument.bin"))),
    ("BinaryData", "Двоичные данные", None),
    ("HTMLDocument", "HTML документ", None),
    ("GeographicalSchema", "Географическая схема", Some(include_bytes!("new/templates/GeographicalSchema.bin"))),
    ("GraphicalSchema", "Графическая схема", Some(include_bytes!("new/templates/GraphicalSchema.bin"))),
    ("DataCompositionSchema", "Схема компоновки данных", Some(include_bytes!("new/templates/DataCompositionSchema.bin"))),
    ("DataCompositionAppearanceTemplate", "Макет оформления компоновки данных", Some(include_bytes!("new/templates/DataCompositionAppearanceTemplate.bin"))),
    ("AddIn", "Внешняя компонента", None),
];

/// Новый макет объекта `owner` типа `ttype` ([`TEMPLATE_TYPES`]), как «Конструктор макета» Конфигуратора 8.5;
/// `named` — имя, синоним, комментарий из окна конструктора (нет — по умолчанию).
pub fn add_template_of(cfg: &Config, names: &mut Names, owner: &str, ttype: &str, named: Option<(&str, &str, &str)>) -> Result<(String, Vec<Change>), String> {
    let o = find(cfg, owner).filter(|o| o.sub.is_none() && o.owner.is_none() && o.class != "Configuration").ok_or(format!("нет объекта {owner}"))?;
    add_template(cfg, names, &o, ttype, named)
}

/// Имя, синоним и комментарий нового элемента вида `kind` из окна конструктора (`named`) или по умолчанию (`stem`,
/// `syn`; занятое имя — с номером): имя — идентификатор, не занятое у владельца без учета регистра.
fn wizard_names(o: &Obj, kind: &str, named: Option<(&str, &str, &str)>, stem: &str, syn: &str) -> Result<(String, String, String), String> {
    let taken: Vec<String> = o.item.children.iter().filter(|c| c.kind == kind).map(|c| c.header.name.clone()).collect();
    let is_taken = |n: &str| taken.iter().any(|t| t.to_lowercase() == n.to_lowercase());
    match named {
        Some((n, s, c)) => {
            if n.is_empty() || n.chars().next().is_some_and(|c| c.is_ascii_digit()) || !n.chars().all(|c| c.is_alphanumeric() || c == '_') {
                return Err(format!("имя «{n}» — не идентификатор"));
            }
            if is_taken(n) {
                return Err(format!("имя «{n}» уже занято"));
            }
            Ok((n.to_string(), s.to_string(), c.to_string()))
        }
        // shortcut: занятое имя — с номером, как у реквизитов; у Конфигуратора не снято
        None => Ok((if is_taken(stem) { free_name(stem, &taken) } else { stem.to_string() }, syn.to_string(), String::new())),
    }
}

/// Новый макет объекта по эталону «Конструктора макета» Конфигуратора 8.5 (27.10.2026: имя «Макет», следующие —
/// «Макет1», «Макет2»…, синоним — как имя): (uuid, замены — описание, содержимое, если у типа есть, и владелец).
fn add_template(cfg: &Config, names: &mut Names, o: &Obj, ttype: &str, named: Option<(&str, &str, &str)>) -> Result<(String, Vec<Change>), String> {
    let &(_, _, body) = TEMPLATE_TYPES.iter().find(|t| t.0 == ttype).ok_or(format!("тип макета {ttype}"))?;
    let (name, syn, comment) = match named {
        Some(_) => wizard_names(o, "Template", named, "", "")?,
        None => {
            let (n, _, _) = wizard_names(o, "Template", None, "Макет", "")?;
            (n.clone(), n, String::new())
        }
    };
    let t = include_str!("new/Template.xml").replace("\r\n", "\n");
    let t = super::newform::synonym_comment(&t, "Макет", &syn, &comment).replace("<TemplateType>SpreadsheetDocument</TemplateType>", &format!("<TemplateType>{ttype}</TemplateType>")).replace('\n', "\r\n");
    let xml = fresh_uuids(&t.replace("<Name>Макет</Name>", &format!("<Name>{name}</Name>")));
    let id = xml.split_once(" uuid=\"").and_then(|(_, r)| r.split_once('"')).map(|(u, _)| u.to_string()).ok_or("в эталоне нет uuid")?;
    let rec = desc::record(&dom::parse(&xml)?, "Template", Some(o.class), None, &names.imp)?.ok_or("загрузка макета не поддержана")?;
    let mut ch = vec![Change { entry: id.clone(), before: Vec::new(), after: crate::container::deflate(&rec) }];
    if let Some(b) = body {
        ch.push(Change { entry: format!("{id}.0"), before: Vec::new(), after: crate::container::deflate(b) });
    }
    let ox = insert_line(&export(cfg, names, o)?, "Template", &name)?;
    names.imp.full.insert(format!("{}.{}.Template.{name}", o.class, o.item.header.name), id.clone());
    ch.push(holder_change(cfg, names, o, &ox)?);
    Ok((id, ch))
}

/// Виды форм «Конструктора формы»: (вид, имя, синоним, свойство основной формы владельца).
pub const FORM_KINDS: &[(&str, &str, &str, &str)] = &[
    ("Item", "ФормаЭлемента", "Форма элемента", "DefaultObjectForm"),
    ("Folder", "ФормаГруппы", "Форма группы", "DefaultFolderForm"),
    ("List", "ФормаСписка", "Форма списка", "DefaultListForm"),
    ("Choice", "ФормаВыбора", "Форма выбора", "DefaultChoiceForm"),
    ("FolderChoice", "ФормаВыбораГруппы", "Форма выбора группы", "DefaultFolderChoiceForm"),
];

/// Новая форма внешней обработки по эталону «Конструктора формы обработки» Конфигуратора 8.5 (28.10.2026,
/// локальный синтетический эксперимент): «Форма обработки» и «Произвольная форма» дают одну форму — автокомандная
/// панель и основной реквизит «Объект» (ExternalDataProcessorObject), без модуля; в описании — ExtendedPresentation;
/// `main` — основная форма обработки (первая форма — «Форма», дальше «Форма1»…).
pub fn add_ext_form(cfg: &Config, names: &mut Names, main: bool, named: Option<(&str, &str, &str)>) -> Result<(String, Vec<Change>), String> {
    let o = find(cfg, "root").filter(|o| o.class == "ExternalDataProcessor").ok_or("не внешняя обработка")?;
    let (name, mut syn, comment) = wizard_names(&o, "Form", named, "Форма", "Форма")?;
    // без окна синоним — как имя («Форма1», wiz-x)
    if named.is_none() {
        syn = name.clone();
    }
    let id = uuid4();
    let full = format!("ExternalDataProcessor.{}.Form.{name}", o.item.header.name);
    let desc = super::newform::props(&id, &name, &syn, &comment).replace("\t\t</Properties>", "\t\t\t<ExtendedPresentation/>\r\n\t\t</Properties>");
    let desc_rec = desc::record(&dom::parse(&desc)?, "Form", Some(o.class), None, &names.imp)?.ok_or("загрузка описания формы не поддержана")?;
    let form = super::newform::custom_form().replace(
        "\t<Attributes/>",
        &format!("\t<Attributes>\r\n\t\t<Attribute name=\"Объект\" id=\"1\">\r\n\t\t\t<Type>\r\n\t\t\t\t<v8:Type>cfg:ExternalDataProcessorObject.{}</v8:Type>\r\n\t\t\t</Type>\r\n\t\t\t<MainAttribute>true</MainAttribute>\r\n\t\t</Attribute>\r\n\t</Attributes>", o.item.header.name),
    );
    let body = match super::imp::form::record(&dom::parse(&form)?, None, &names.imp, &format!("{full}/Form.xml"), &|f| Err(format!("картинка {f}")))?.0 {
        super::imp::Rec::Raw(b) => b,
        super::imp::Rec::Dir(v) => crate::container::write(&v.into_iter().map(|(name, data)| crate::container::Entry { name, created: 0, modified: 0, data }).collect::<Vec<_>>()),
    };
    let mut ch = vec![
        Change { entry: id.clone(), before: Vec::new(), after: crate::container::deflate(&desc_rec) },
        Change { entry: format!("{id}.0"), before: Vec::new(), after: crate::container::deflate(&body) },
    ];
    let mut ox = insert_line(&export(cfg, names, &o)?, "Form", &name)?;
    names.imp.full.insert(full.clone(), id.clone());
    if main {
        let p = props(&ox, o.class)?.into_iter().find(|p| p.tag == "DefaultForm").ok_or("нет свойства DefaultForm")?;
        ox = set(&ox, &p, &Value::Text(full));
    }
    ch.push(holder_change(cfg, names, &o, &ox)?);
    Ok((id, ch))
}

/// Имя и синоним новой формы по умолчанию (конструктор формы): у формы документа — «ФормаДокумента».
pub fn form_default(class: &str, kind: &str) -> (&'static str, &'static str) {
    match (class, kind) {
        ("Document", "Item") => ("ФормаДокумента", "Форма документа"),
        _ => FORM_KINDS.iter().find(|k| k.0 == kind).map_or(("Форма", "Форма"), |k| (k.1, k.2)),
    }
}

/// Строка списка «Реквизиты» страницы «Расположение» конструктора формы: поле, отмечено ли; у табличной части —
/// колонки (у поля — пусто).
#[derive(Clone, Debug)]
pub struct LayoutRow {
    pub field: super::newform::Field,
    pub on: bool,
    pub cols: Vec<LayoutRow>,
}

/// Список «Реквизиты» конструктора формы элемента, группы или документа, как у Конфигуратора (снимки 27, 34
/// 28.10.2026): стандартные реквизиты (отмечены поля формы по умолчанию), реквизиты и табличные части с «НомерСтроки»
/// (отмечены). У других видов — пусто. shortcut: место «Владелец» и «Родитель» (после «Наименование») и отсутствие
/// «ЭтоГруппа» выведены по форме по умолчанию — снимка иерархического справочника нет; реквизиты формы группы — по
/// «Использованию» (ForFolder, ForFolderAndItem).
pub fn form_layout(cfg: &Config, names: &Names, owner: &str, kind: &str) -> Result<Vec<LayoutRow>, String> {
    if !matches!(kind, "Item" | "Folder") {
        return Ok(Vec::new());
    }
    let o = find(cfg, owner).ok_or(format!("нет объекта {owner}"))?;
    let doc = o.class == "Document";
    let xml = export(cfg, names, &o)?;
    let ps = props(&xml, o.class)?;
    let row = |name: &str, path: &str, boolean: bool, on: bool| LayoutRow { field: super::newform::Field { name: name.into(), path: path.into(), boolean }, on, cols: Vec::new() };
    let mut rows = vec![row("Ссылка", "Ref", false, false)];
    if doc {
        rows.extend([row("Номер", "Number", false, true), row("Дата", "Date", false, true), row("Проведен", "Posted", true, false), row("ПометкаУдаления", "DeletionMark", true, false)]);
    } else {
        rows.extend([row("Код", "Code", false, true), row("Наименование", "Description", false, true)]);
        if matches!(ps.iter().find(|p| p.tag == "Owners").map(|p| &p.value), Some(Value::List(v)) if !v.is_empty()) {
            rows.push(row("Владелец", "Owner", false, true));
        }
        if ps.iter().any(|p| p.tag == "Hierarchical" && p.shown() == "true") {
            rows.push(row("Родитель", "Parent", false, true));
        }
        rows.extend([row("ПометкаУдаления", "DeletionMark", true, false), row("Предопределенный", "Predefined", true, false), row("ИмяПредопределенныхДанных", "PredefinedDataName", false, false)]);
    }
    let boolean = |id: &str| {
        props_of(&xml, "Attribute", Some(id)).ok().and_then(|p| p.into_iter().find(|p| p.tag == "Type")).is_some_and(|p| matches!(&p.value, Value::Type(t) if t.items.len() == 1 && t.items[0].1 == "xs:boolean"))
    };
    let used = |what: &str, id: &str| {
        let u = props_of(&xml, what, Some(id)).ok().and_then(|p| p.into_iter().find(|p| p.tag == "Use")).map(|p| p.shown()).unwrap_or_default();
        doc || if kind == "Folder" { u == "ForFolder" || u == "ForFolderAndItem" } else { u != "ForFolder" }
    };
    for c in o.item.children.iter().filter(|c| c.kind == "Attribute" && used("Attribute", &c.header.id)) {
        rows.push(row(&c.header.name, &c.header.name, boolean(&c.header.id), true));
    }
    for t in o.item.children.iter().filter(|c| c.kind == "TabularSection" && used("TabularSection", &c.header.id)) {
        let mut r = row(&t.header.name, &t.header.name, false, true);
        r.cols.push(row("НомерСтроки", "LineNumber", false, true));
        r.cols.extend(t.children.iter().filter(|c| c.kind == "Attribute").map(|c| row(&c.header.name, &c.header.name, boolean(&c.header.id), true)));
        rows.push(r);
    }
    Ok(rows)
}

/// Новая форма объекта, как «Конструктор формы» Конфигуратора с настройками по умолчанию (`newform`; эталоны
/// справочника и документа 27.10.2026): (uuid, замены — описание формы, форма, владелец). `main` — «Назначить
/// форму основной» (флажок конструктора, по умолчанию отмечен): свойство основной формы владельца — новая форма.
/// `named` — имя, синоним, комментарий из окна конструктора (нет — по умолчанию, занятое имя — с номером).
/// `layout` — страница «Расположение» (`form_layout` с флажками и число колонок; нет — по умолчанию).
/// Отказ с причиной: не справочник и не документ, форма группы у неиерархического, длина кода или наименования 0.
pub fn add_form(cfg: &Config, names: &mut Names, owner: &str, kind: &str, main: bool, named: Option<(&str, &str, &str)>, layout: Option<(&[LayoutRow], usize)>) -> Result<(String, Vec<Change>), String> {
    let o = find(cfg, owner).filter(|o| o.sub.is_none() && o.owner.is_none()).ok_or(format!("нет объекта {owner}"))?;
    if !matches!(o.class, "Catalog" | "Document") {
        return Err(format!("форма {}: нет эталона Конфигуратора (есть у справочника и документа)", o.class));
    }
    // "" — «Произвольная форма»: основной не назначается
    let prop = match kind {
        "" => None,
        _ => Some(FORM_KINDS.iter().find(|k| k.0 == kind).ok_or(format!("вид формы {kind}"))?.3),
    };
    let doc = o.class == "Document";
    let (stem, syn) = form_default(o.class, kind);
    let xml = export(cfg, names, &o)?;
    let ps = props(&xml, o.class)?;
    let get = |t: &str| ps.iter().find(|p| p.tag == t).map(Prop::shown).unwrap_or_default();
    let tree = get("Hierarchical") == "true";
    if matches!(kind, "Folder" | "FolderChoice") && (doc || !tree) {
        return Err("форма группы — у иерархического справочника".into());
    }
    let num = |t: &str| get(t).parse::<u32>().unwrap_or(0);
    if doc && num("NumberLength") == 0 || !doc && (num("CodeLength") == 0 || num("DescriptionLength") == 0) {
        return Err("длина кода, наименования или номера 0: нет эталона Конфигуратора".into());
    }
    let folder = kind == "Folder";
    let form = match kind {
        "Item" | "Folder" => {
            let rows = match layout {
                Some((rows, _)) => rows.to_vec(),
                None => form_layout(cfg, names, owner, kind)?,
            };
            let on = |r: &&LayoutRow| r.on;
            let head = rows.iter().filter(|r| r.cols.is_empty()).filter(on).map(|r| r.field.clone()).collect();
            let tables = rows.iter().filter(|r| !r.cols.is_empty()).filter(on).map(|r| (r.field.name.clone(), r.cols.iter().filter(on).map(|c| c.field.clone()).collect())).collect();
            let columns = layout.map_or(1, |l| l.1.max(1));
            super::newform::object_form(&super::newform::Spec { class: o.class, name: o.item.header.name.clone(), head, tables, columns }, folder)
        }
        "" => super::newform::custom_form(),
        _ => super::newform::list_form(o.class, &o.item.header.name, kind, tree)?,
    };
    let (name, syn, comment) = wizard_names(&o, "Form", named, stem, syn)?;
    let id = uuid4();
    let full = format!("{}.{}.Form.{name}", o.class, o.item.header.name);
    let desc_rec = desc::record(&dom::parse(&super::newform::props(&id, &name, &syn, &comment))?, "Form", Some(o.class), None, &names.imp)?.ok_or("загрузка описания формы не поддержана")?;
    // поля основной таблицы динамического списка — по описанию владельца (индекс из XML строит их сам)
    let od = dom::parse(&xml)?;
    let oe = od.child(o.class).ok_or("описание владельца")?;
    names.imp.object_info(o.class, oe, &format!("{}.{}", o.class, o.item.header.name));
    // режим совместимости — из описания корня (настройки динамического списка зависят от него)
    let root = find(cfg, "root").ok_or("нет корня")?;
    let cm = props(&export(cfg, names, &root)?, "Configuration")?.into_iter().find(|p| p.tag == "CompatibilityMode").map(|p| p.shown());
    names.imp.compat = cm.as_deref().and_then(super::imp::index::compat_of);
    let body = match super::imp::form::record(&dom::parse(&form)?, None, &names.imp, &format!("{full}/Form.xml"), &|f| Err(format!("картинка {f}")))?.0 {
        super::imp::Rec::Raw(b) => b,
        super::imp::Rec::Dir(v) => crate::container::write(&v.into_iter().map(|(name, data)| crate::container::Entry { name, created: 0, modified: 0, data }).collect::<Vec<_>>()),
    };
    let mut ch = vec![
        Change { entry: id.clone(), before: Vec::new(), after: crate::container::deflate(&desc_rec) },
        Change { entry: format!("{id}.0"), before: Vec::new(), after: crate::container::deflate(&body) },
    ];
    let mut ox = insert_line(&xml, "Form", &name)?;
    names.imp.full.insert(full.clone(), id.clone());
    if let Some(prop) = prop.filter(|_| main) {
        let p = props(&ox, o.class)?.into_iter().find(|p| p.tag == prop).ok_or(format!("нет свойства {prop}"))?;
        ox = set(&ox, &p, &Value::Text(full));
    }
    ch.push(holder_change(cfg, names, &o, &ox)?);
    Ok((id, ch))
}

/// Свое у объекта: uuid (сам, вложенные элементы, генераторы типов) и записи (сам, его формы, макеты, модули).
/// У вложенного элемента записей нет — он в записи владельца.
fn own(cfg: &Config, names: &Names, o: &Obj) -> Result<(std::collections::HashSet<String>, Vec<String>), String> {
    fn walk(it: &Item, ids: &mut std::collections::HashSet<String>, entries: &mut Vec<String>) {
        ids.insert(it.header.id.clone());
        entries.extend(it.files.iter().map(|(f, _)| f.clone()));
        it.children.iter().for_each(|c| walk(c, ids, entries));
    }
    let (mut ids, mut entries) = (std::collections::HashSet::new(), Vec::new());
    let it = o.sub.unwrap_or(o.item);
    walk(it, &mut ids, &mut entries);
    if o.sub.is_none() {
        entries.extend(ids.iter().filter(|i| cfg.index.contains_key(*i)).cloned());
    }
    // генераторы типов — из InternalInfo описания (у вложенного — в его элементе)
    let xml = export(cfg, names, o)?;
    let part = match o.sub {
        Some(s) => xml.find(&format!(" uuid=\"{}\"", s.header.id)).and_then(|u| xml[..u].rfind('<').and_then(|b| Some(&xml[b..element_end(&xml, b)?]))).unwrap_or(""),
        None => &xml,
    };
    for tag in ["<xr:TypeId>", "<xr:ValueId>"] {
        ids.extend(part.split(tag).skip(1).filter_map(|r| r.split('<').next()).filter(|u| crate::brace::is_guid(u)).map(String::from));
    }
    entries.sort();
    entries.dedup();
    Ok((ids, entries))
}

/// Записи, где встречаются uuid объекта `id` (сам, вложенные элементы, генераторы типов), кроме его собственных,
/// записи владельца вложенного элемента и `versions`: (запись, найденный uuid). Один проход по распакованным записям в
/// потоках: каждый фрагмент вида uuid ищется в наборе.
pub fn refs(cfg: &Config, names: &Names, id: &str) -> Result<Vec<(String, String)>, String> {
    let o = find(cfg, id).ok_or(format!("нет объекта {id}"))?;
    let (ids, mine) = own(cfg, names, &o)?;
    let skip = |e: &str| e == "versions" || mine.iter().any(|m| m == e) || (o.sub.is_some() && e == o.entry);
    let all: Vec<&crate::container::Entry> = cfg.entries.iter().filter(|e| !skip(&e.name)).collect();
    let hits = crate::par::map(&all, || (), |_, e| {
        let b = crate::container::content(e);
        let hex = |c: &u8| c.is_ascii_hexdigit();
        let mut i = 0;
        while i + 36 <= b.len() {
            let w = &b[i..i + 36];
            if w[8] == b'-' && w[13] == b'-' && w[18] == b'-' && w[23] == b'-' && w.iter().enumerate().all(|(k, c)| matches!(k, 8 | 13 | 18 | 23) || hex(c)) {
                if let Some(u) = std::str::from_utf8(w).ok().filter(|u| ids.contains(*u)) {
                    return Some((e.name.clone(), u.to_string()));
                }
                i += 36;
            } else {
                i += 1;
            }
        }
        None
    });
    Ok(hits.into_iter().flatten().collect())
}

/// Объект записи (для сообщений): «Документ.Документ1» — по uuid в начале имени записи; не нашелся — имя записи.
pub fn whose(cfg: &Config, entry: &str) -> String {
    let id = entry.split('.').next().unwrap_or(entry);
    fn walk(it: &Item, id: &str, path: &mut Vec<String>) -> bool {
        if it.header.id == id || it.files.iter().any(|(f, _)| f.starts_with(id)) {
            return true;
        }
        for c in &it.children {
            path.push(c.header.name.clone());
            if walk(c, id, path) {
                return true;
            }
            path.pop();
        }
        false
    }
    if entry == cfg.root_id {
        return "Конфигурация".into();
    }
    for g in &cfg.groups {
        for it in &g.items {
            let mut p = vec![crate::meta::CLASSES.iter().find(|c| c.1 == g.class).map_or(g.class, |c| c.3).to_string(), it.header.name.clone()];
            if walk(it, id, &mut p) {
                return p.join(".");
            }
        }
    }
    entry.to_string()
}

/// Первая строка отказа удаления — текст окна Конфигуратора (снимок 08).
pub const DELETE_REFUSED: &str = "Объект не может быть удален, так как на него имеются ссылки в других объектах!";

/// Удалить объект верхнего уровня или вложенный элемент, как Конфигуратор 8.5 (эталон — `before-del`/`after-del`):
/// ссылки из других объектов (типы, владельцы…) — отказ со списком «использован в»; права ролей на него снимаются;
/// состав подсистем остается со ссылкой на uuid удаленного (так оставляет Конфигуратор). Формы, макеты, подсистемы —
/// отказ (эталона нет). Замены записей: свои — удалены, корень или владелец — без элемента, права ролей.
pub fn delete(cfg: &Config, names: &Names, id: &str) -> Result<Vec<Change>, String> {
    let o = find(cfg, id).ok_or(format!("нет объекта {id}"))?;
    if o.class == "Configuration" || o.full.is_some() || o.class == "Subsystem" {
        return Err("удаление подсистем и корня: нет эталона Конфигуратора".into());
    }
    if o.owner.is_some() {
        return delete_form(cfg, names, &o);
    }
    let (ids, mine) = own(cfg, names, &o)?;
    let role_rights = |e: &str| cfg.groups.iter().filter(|g| g.class == "Role").flat_map(|g| &g.items).any(|r| r.files.iter().any(|(f, role)| f == e && *role == "Rights"));
    let subsystem = |e: &str| {
        fn has(it: &Item, e: &str) -> bool {
            it.header.id == e || it.children.iter().filter(|c| c.kind == "Subsystem").any(|c| has(c, e))
        }
        cfg.groups.iter().filter(|g| g.class == "Subsystem").flat_map(|g| &g.items).any(|s| has(s, e))
    };
    let mut rights = Vec::new();
    let mut used: Vec<String> = Vec::new();
    for (e, _) in refs(cfg, names, id)? {
        if e == cfg.root_id && o.sub.is_none() {
            continue;
        } else if role_rights(&e) {
            rights.push(e);
        } else if !subsystem(&e) {
            let w = whose(cfg, &e);
            if !used.contains(&w) {
                used.push(w);
            }
        }
    }
    let name = o.sub.unwrap_or(o.item).header.name.clone();
    if !used.is_empty() {
        // как у Конфигуратора (снимки 08, 09): вопрос-отказ и по строке служебного сообщения на объект со ссылкой
        // shortcut: место ссылки — до объекта («Документ.Документ1»), у Конфигуратора — до свойства («….Реквизит.Реквизит1.Тип»)
        let me = format!("{}.{}{}", crate::meta::CLASSES.iter().find(|c| c.1 == o.class).map_or(o.class, |c| c.3), o.item.header.name, o.sub.map(|s| format!(".{}", s.header.name)).unwrap_or_default());
        let lines: Vec<String> = used.iter().map(|u| format!("Объект \"{me}\" использован в: {u}")).collect();
        return Err(format!("{DELETE_REFUSED}\n{}", lines.join("\n")));
    }
    let data = |e: &str| cfg.index.get(e).map(|&i| cfg.entries[i].data.clone()).ok_or(format!("нет записи {e}"));
    let mut ch: Vec<Change> = mine.iter().filter(|e| cfg.index.contains_key(*e)).map(|e| Ok(Change { entry: e.clone(), before: data(e)?, after: Vec::new() })).collect::<Result<_, String>>()?;
    // корень или владелец — описание без элемента, через загрузку
    let (host, xml) = match o.sub {
        None => {
            let root = find(cfg, "root").ok_or("нет корня")?;
            let rx = export(cfg, names, &root)?;
            let line = format!("\r\n\t\t\t<{}>{}</{}>", o.class, super::w::escape(&name), o.class);
            let p = rx.find(&line).ok_or("в корне нет объекта")?;
            (root, format!("{}{}", &rx[..p], &rx[p + line.len()..]))
        }
        Some(s) => {
            let host = find(cfg, &o.entry).ok_or("нет владельца")?;
            let ox = export(cfg, names, &host)?;
            let u = ox.find(&format!(" uuid=\"{}\"", s.header.id)).ok_or("нет элемента в описании владельца")?;
            let b = ox[..u].rfind("\r\n").ok_or("описание владельца")?;
            let e = element_end(&ox, b + 2).ok_or("описание владельца")?;
            let mut x = format!("{}{}", &ox[..b], &ox[e..]);
            // удален последний элемент — `<ChildObjects/>`, как пишет выгрузка
            if let Some(p) = x[..b].rfind("<ChildObjects>") {
                let inner = p + "<ChildObjects>".len();
                if let Some(q) = x[inner..].find("</ChildObjects>").map(|q| inner + q).filter(|&q| x[inner..q].trim().is_empty()) {
                    x = format!("{}<ChildObjects/>{}", &x[..p], &x[q + "</ChildObjects>".len()..]);
                }
            }
            (host, x)
        }
    };
    let rec = import(&xml, names, &host)?;
    ch.push(Change { entry: host.entry.clone(), before: data(&host.entry)?, after: crate::container::deflate(&rec) });
    // права ролей: пары `{{1,<uuid>,0,0},{права}}` объекта и вложенных — прочь, число пар меньше
    for e in rights {
        let text = cfg.entry_text(&e).ok_or(format!("нет записи {e}"))?;
        let mut d = crate::brace::Doc::parse(&text)?;
        fn strip(n: &mut crate::brace::Node, ids: &std::collections::HashSet<String>) -> usize {
            let mut gone = 0;
            if let Some(l) = n.list_mut() {
                let pair = |c: &crate::brace::Node| c.at(&[0, 0]).and_then(crate::brace::Node::atom) == Some("1") && c.at(&[0, 1]).and_then(crate::brace::Node::atom).is_some_and(|u| ids.contains(u));
                let counted = l.first().and_then(crate::brace::Node::atom).and_then(|a| a.parse::<usize>().ok()) == Some(l.len() - 1);
                if counted && l.iter().skip(1).any(pair) {
                    let before = l.len();
                    let mut k = 1;
                    while k < l.len() {
                        if pair(&l[k]) {
                            l.remove(k);
                        } else {
                            k += 1;
                        }
                    }
                    gone += before - l.len();
                    l[0] = crate::brace::Node::Atom((l.len() - 1).to_string());
                }
                for c in l.iter_mut() {
                    gone += strip(c, ids);
                }
            }
            gone
        }
        if d.roots.iter_mut().map(|r| strip(r, &ids)).sum::<usize>() == 0 {
            return Err(format!("права роли {}: место объекта в записи не найдено", whose(cfg, &e)));
        }
        let t = d.write()?;
        if ids.iter().any(|u| t.contains(u.as_str())) {
            return Err(format!("права роли {}: ссылка на объект осталась вне списка прав — не разобрано", whose(cfg, &e)));
        }
        // BOM — как было у записи (`entry_text` его снимает)
        let bom: &[u8] = if crate::container::content(&cfg.entries[cfg.index[&e]]).starts_with(&[0xef, 0xbb, 0xbf]) { &[0xef, 0xbb, 0xbf] } else { &[] };
        ch.push(Change { entry: e.clone(), before: data(&e)?, after: crate::container::deflate(&[bom, t.as_bytes()].concat()) });
    }
    Ok(ch)
}

/// Объект `id` в составе плана обмена `plan` (страница «Обмен данными» окна объекта): `Some(авторегистрация)` —
/// включить или сменить авторегистрацию, `None` — исключить. Как у Конфигуратора (эталон 27.10.2026): флажок
/// включает с «Разрешить» — `Some(true)`; новый элемент — в конец Content.xml. Правка — через загрузку состава.
pub fn exchange(cfg: &mut Config, names: &Names, plan: &str, id: &str, auto: Option<bool>) -> Result<Option<Change>, String> {
    let o = find(cfg, id).filter(|o| o.sub.is_none() && o.owner.is_none()).ok_or("нет объекта")?;
    let full = format!("{}.{}", o.class, o.item.header.name);
    let p = find(cfg, plan).filter(|p| p.class == "ExchangePlan").ok_or("нет плана обмена")?;
    // у плана с пустым составом записи нет (эталон: ПланОбмена1 без «Состава») — первая отметка ее создает
    let entry = p.item.files.iter().find(|f| f.1 == "Content").map(|f| f.0.clone());
    let fresh = entry.is_none();
    let entry = entry.unwrap_or_else(|| format!("{plan}.1"));
    let text = if fresh { "{2,0,0}".to_string() } else { cfg.entry_text(&entry).ok_or(format!("нет записи {entry}"))? };
    let node = crate::brace::parse(&text)?;
    let xml = String::from_utf8(super::ext::content(&node, &super::ext::Names::new(cfg))?).map_err(|e| e.to_string())?;
    let item = |a: bool| format!("\t<Item>\r\n\t\t<Metadata>{full}</Metadata>\r\n\t\t<AutoRecord>{}</AutoRecord>\r\n\t</Item>\r\n", if a { "Allow" } else { "Deny" });
    let span = xml.find(&format!("\t<Item>\r\n\t\t<Metadata>{full}</Metadata>")).map(|s| (s, s + xml[s..].find("\t</Item>\r\n").map_or(0, |e| e + "\t</Item>\r\n".len())));
    let new = match (span, auto) {
        (Some((s, e)), None) => format!("{}{}", &xml[..s], &xml[e..]),
        (Some((s, e)), Some(a)) => format!("{}{}{}", &xml[..s], item(a), &xml[e..]),
        (None, None) => return Ok(None),
        (None, Some(a)) => match xml.rfind("</ExchangePlanContent>") {
            Some(at) => format!("{}{}{}", &xml[..at], item(a), &xml[at..]),
            None => {
                let at = xml.rfind("/>").ok_or("состав без корня")?;
                format!("{}>\r\n{}</ExchangePlanContent>", &xml[..at], item(a))
            }
        },
    };
    // пустой состав — `<ExchangePlanContent …/>`, как у выгрузки
    let new = match new.find(">\r\n</ExchangePlanContent>").filter(|_| !new.contains("<Item>")) {
        Some(at) => format!("{}/>", &new[..at]),
        None => new,
    };
    let rec = match super::imp::ext::content(&dom::parse(&new)?, &names.imp)? {
        super::imp::Rec::Raw(b) => b,
        super::imp::Rec::Dir(_) => return Err("состав плана: запись не одна".into()),
    };
    if fresh {
        let ch = Change { entry, before: Vec::new(), after: crate::container::deflate(&rec) };
        apply(cfg, std::slice::from_ref(&ch), false)?;
        return Ok(Some(ch));
    }
    store(cfg, &entry, &rec)
}

/// Классы со свойством «Вводится на основании» — кандидаты «Является основанием для».
pub fn based_classes() -> Vec<&'static str> {
    crate::meta::CLASSES.iter().map(|c| c.1).filter(|c| table(c).is_some_and(|t| t.iter().any(|p| p.0 == "BasedOn"))).collect()
}

/// «Является основанием для» объекта `id` — список `to` (полные имена XML), как «Изменить» Конфигуратора (эталон
/// 27.10.2026): объекту из списка без этого основания оно дописывается в конец «Вводится на основании», у объекта
/// не из списка — снимается; правка каждого — через загрузку его описания.
pub fn set_based_to(cfg: &Config, names: &Names, id: &str, to: &[String]) -> Result<Vec<Change>, String> {
    let o = find(cfg, id).filter(|o| o.sub.is_none() && o.owner.is_none()).ok_or("нет объекта")?;
    let me = format!("{}.{}", o.class, o.item.header.name);
    let mut ch = Vec::new();
    for class in based_classes() {
        for it in cfg.groups.iter().filter(|g| g.class == class).flat_map(|g| &g.items) {
            let x = find(cfg, &it.header.id).ok_or("нет объекта")?;
            let xml = export(cfg, names, &x)?;
            let Some(p) = props(&xml, x.class)?.into_iter().find(|p| p.tag == "BasedOn") else { continue };
            let Value::List(items) = &p.value else { continue };
            let want = to.contains(&format!("{class}.{}", it.header.name));
            let has = items.contains(&me);
            if want == has {
                continue;
            }
            let v = if want { items.iter().cloned().chain([me.clone()]).collect() } else { items.iter().filter(|i| **i != me).cloned().collect() };
            ch.push(holder_change(cfg, names, &x, &set(&xml, &p, &Value::List(v)))?);
        }
    }
    Ok(ch)
}

/// Удаление формы или макета, как у Конфигуратора (снимки 27.10.2026): свойство владельца со ссылкой на нее
/// (основная форма…) или ссылка из другой записи — отказ со строкой «использован в: Справочник.Справочник1.
/// ОсновнаяФормаОбъекта»; иначе записи формы прочь и строка `<Form>Имя</Form>` из описания владельца.
fn delete_form(cfg: &Config, names: &Names, o: &Obj) -> Result<Vec<Change>, String> {
    let owner = cfg.groups.iter().flat_map(|g| &g.items).find(|it| it.children.iter().any(|c| c.header.id == o.item.header.id)).ok_or("нет владельца")?;
    let host = find(cfg, &owner.header.id).ok_or("нет владельца")?;
    let ox = export(cfg, names, &host)?;
    let name = &o.item.header.name;
    let full = format!("{}.{}.{}.{name}", host.class, owner.header.name, o.class);
    let ru = |class: &str| crate::meta::CLASSES.iter().find(|c| c.1 == class).map_or(class.to_string(), |c| c.3.to_string());
    let camel = |s: &str| s.split(' ').map(|w| w.chars().next().map(|f| f.to_uppercase().collect::<String>() + &w[f.len_utf8()..]).unwrap_or_default()).collect::<String>();
    let me = format!("{}.{}.{}.{name}", camel(&ru(host.class)), owner.header.name, if o.class == "Form" { "Форма" } else { "Макет" });
    let mut used: Vec<String> = Vec::new();
    // свойства владельца со ссылкой: элемент `<Тег>Catalog.X.Form.Имя</Тег>` описания
    for (p, _) in ox.match_indices(&format!(">{full}</")) {
        let tag = ox[..p].rsplit('<').next().unwrap_or_default();
        used.push(format!("{}.{}.{}", camel(&ru(host.class)), owner.header.name, camel(super::captions::prop_ru(tag).unwrap_or(tag))));
    }
    let (_, mine) = own(cfg, names, o)?;
    for (e, _) in refs(cfg, names, &o.item.header.id)? {
        if e != owner.header.id {
            let w = whose(cfg, &e);
            if !used.contains(&w) {
                used.push(w);
            }
        }
    }
    if !used.is_empty() {
        let lines: Vec<String> = used.iter().map(|u| format!("Объект \"{me}\" использован в: {u}")).collect();
        return Err(format!("{DELETE_REFUSED}\n{}", lines.join("\n")));
    }
    let data = |e: &str| cfg.index.get(e).map(|&i| cfg.entries[i].data.clone()).ok_or(format!("нет записи {e}"));
    let mut ch: Vec<Change> = mine.iter().filter(|e| cfg.index.contains_key(*e)).map(|e| Ok(Change { entry: e.clone(), before: data(e)?, after: Vec::new() })).collect::<Result<_, String>>()?;
    let line = format!("\r\n\t\t\t<{}>{}</{}>", o.class, super::w::escape(name), o.class);
    let p = ox.find(&line).ok_or("в описании владельца нет формы")?;
    let mut x = format!("{}{}", &ox[..p], &ox[p + line.len()..]);
    if let Some(q) = x[..p].rfind("<ChildObjects>") {
        let inner = q + "<ChildObjects>".len();
        if let Some(r) = x[inner..].find("</ChildObjects>").map(|r| inner + r).filter(|&r| x[inner..r].trim().is_empty()) {
            x = format!("{}<ChildObjects/>{}", &x[..q], &x[r + "</ChildObjects>".len()..]);
        }
    }
    ch.push(holder_change(cfg, names, &host, &x)?);
    Ok(ch)
}

/// Синоним из имени, как у Конфигуратора при переименовании объекта с пустым синонимом (эталон after-rename:
/// «Справочник1Новый» → «Справочник1 новый»): слово — с заглавной после строчной или цифры, следующие слова — со
/// строчной. shortcut: правило по одному образцу; аббревиатуры («ЗагрузкаЭДО») и «_» не сверены — пишутся как
/// слово из заглавных без изменения.
pub fn synonym_of(name: &str) -> String {
    let mut out = String::new();
    let cs: Vec<char> = name.chars().collect();
    for (i, &c) in cs.iter().enumerate() {
        let prev = i.checked_sub(1).map(|k| cs[k]);
        if c.is_uppercase() && prev.is_some_and(|p| p.is_lowercase() || p.is_ascii_digit()) {
            out.push(' ');
            let abbr = cs.get(i + 1).is_some_and(|n| n.is_uppercase());
            out.extend(if abbr { vec![c] } else { c.to_lowercase().collect() });
        } else {
            out.push(c);
        }
    }
    out
}

/// Менеджер объекта в тексте модуля: `Справочники.`, `Catalogs.`; у общего модуля — само имя.
fn managers(class: &str) -> Vec<String> {
    const MANAGED: &[&str] = &[
        "Catalog", "Document", "Enum", "Constant", "InformationRegister", "AccumulationRegister", "AccountingRegister", "CalculationRegister",
        "ChartOfCharacteristicTypes", "ChartOfAccounts", "ChartOfCalculationTypes", "ExchangePlan", "BusinessProcess", "Task", "DataProcessor", "Report", "DocumentJournal",
    ];
    if class == "CommonModule" {
        return vec![String::new()];
    }
    if !MANAGED.contains(&class) {
        return Vec::new();
    }
    let ru = crate::meta::CLASSES.iter().find(|c| c.1 == class).map(|c| c.2.replace(' ', "")).unwrap_or_default();
    vec![format!("{ru}."), format!("{}.", super::plural(class))]
}

/// Замены `<менеджер><имя>` в тексте модуля вне строк и комментариев, без учета регистра, по границе слова:
/// (новый текст, найдено).
fn replace_in_code(text: &str, prefixes: &[String], old: &str, new: &str) -> (String, usize) {
    let ident = |c: char| c.is_alphanumeric() || c == '_';
    let low: Vec<char> = text.to_lowercase().chars().collect();
    let cs: Vec<char> = text.chars().collect();
    // to_lowercase у кириллицы и латиницы — та же длина в символах; иначе без замен
    if low.len() != cs.len() {
        return (text.to_string(), 0);
    }
    let pats: Vec<Vec<char>> = prefixes.iter().map(|p| format!("{p}{old}").to_lowercase().chars().collect()).collect();
    let (mut out, mut n, mut i) = (String::with_capacity(text.len()), 0, 0);
    let (mut in_str, mut in_comment) = (false, false);
    while i < cs.len() {
        let c = cs[i];
        if in_comment {
            in_comment = c != '\n';
        } else if c == '"' {
            in_str = !in_str;
        } else if !in_str && c == '/' && cs.get(i + 1) == Some(&'/') {
            in_comment = true;
        } else if !in_str && (i == 0 || !ident(cs[i - 1]) && cs[i - 1] != '.') {
            if let Some((p, pat)) = prefixes.iter().zip(&pats).find(|(_, pat)| low[i..].starts_with(pat) && !cs.get(i + pat.len()).is_some_and(|&c| ident(c))) {
                out.extend(&cs[i..i + p.chars().count()]);
                out += new;
                i += pat.len();
                n += 1;
                continue;
            }
        }
        out.push(c);
        i += 1;
    }
    (out, n)
}

/// Переименовать объект верхнего уровня как Конфигуратор 8.5 (эталон before-rename/after-rename): имя в описании
/// (ссылки других объектов — по uuid, в выгрузке сами дают новое имя), пустой синоним — из имени ([`synonym_of`]),
/// в текстах модулей `<менеджер>.<старое имя>` — на новое (окно «Результаты поиска» у Конфигуратора: «Найдено: 1,
/// Заменено автоматически: 1»). Применяет замены к `cfg`: (замены для отмены, замен в модулях).
pub fn rename(cfg: &mut Config, names: &mut Names, id: &str, new: &str) -> Result<(Vec<Change>, usize), String> {
    let o = find(cfg, id).ok_or(format!("нет объекта {id}"))?;
    if o.sub.is_some() || o.owner.is_some() || o.full.is_some() || o.class == "Configuration" {
        return Err("переименование с правкой модулей — у объектов верхнего уровня; вложенные — свойством «Имя»".into());
    }
    let ok = new.chars().next().is_some_and(|c| c.is_alphabetic() || c == '_') && new.chars().all(|c| c.is_alphanumeric() || c == '_');
    if !ok {
        // текст Конфигуратора (frntend_ru.res)
        return Err("Неверное имя! Имя должно состоять из одного слова, начинаться с буквы и не содержать специальных символов кроме \"_\".".into());
    }
    let (class, old) = (o.class, o.item.header.name.clone());
    if cfg.groups.iter().filter(|g| g.class == class).flat_map(|g| &g.items).any(|i| i.header.id != id && i.header.name.to_lowercase() == new.to_lowercase()) {
        return Err(format!("{class}.{new} уже есть"));
    }
    let mut xml = export(cfg, names, &o)?;
    let p = props(&xml, class)?;
    if let Some(s) = p.iter().find(|p| p.tag == "Synonym").filter(|s| s.is_empty()) {
        xml = set(&xml, s, &Value::Ml(vec![("ru".into(), synonym_of(new))]));
    }
    let p = props(&xml, class)?;
    xml = set(&xml, p.iter().find(|p| p.tag == "Name").ok_or("нет имени")?, &Value::Text(new.into()));
    // свои пути (`Catalog.Старое.Form.…`, поля ввода по строке) и имена генераторов (`CatalogRef.Старое`) — с новым
    // именем, и в индексе загрузки — новые ключи к тем же uuid
    let (from, to) = (format!("{class}.{old}"), format!("{class}.{new}"));
    for end in [".", "<"] {
        xml = xml.replace(&format!("{from}{end}"), &format!("{to}{end}"));
    }
    xml = xml.split('\n').map(|l| if l.contains("<xr:GeneratedType name=\"") { l.replace(&format!(".{old}\""), &format!(".{new}\"")).replace(&format!(".{old}."), &format!(".{new}.")) } else { l.to_string() }).collect::<Vec<_>>().join("\n");
    let moved = |k: &str| k == from || k.starts_with(&format!("{from}."));
    let add: Vec<(String, String)> = names.imp.full.iter().filter(|(k, _)| moved(k)).map(|(k, v)| (format!("{to}{}", &k[from.len()..]), v.clone())).collect();
    names.imp.full.extend(add);
    let gen: Vec<(String, (String, String))> = names.imp.types.iter().filter_map(|(k, v)| k.split_once('.').filter(|(_, r)| *r == old || r.starts_with(&format!("{old}."))).map(|(g, r)| (format!("{g}.{new}{}", &r[old.len()..]), v.clone()))).collect();
    names.imp.types.extend(gen);
    let rec = import(&xml, names, &o)?;
    let entry = o.entry.clone();
    let mut ch: Vec<Change> = store(cfg, &entry, &rec)?.into_iter().collect();
    // модули: все тексты конфигурации
    let prefixes = managers(class);
    let mut found = 0;
    let modules: Vec<String> = {
        fn walk(it: &Item, out: &mut Vec<String>) {
            out.extend(it.files.iter().filter(|(_, r)| crate::meta::is_module_role(r)).map(|(f, _)| f.clone()));
            it.children.iter().for_each(|c| walk(c, out));
        }
        let mut v = Vec::new();
        walk(&cfg.root, &mut v);
        cfg.groups.iter().flat_map(|g| &g.items).for_each(|it| walk(it, &mut v));
        v
    };
    // тексты — в потоках (на БП тысячи модулей), запись замен — по одной
    let edits = if prefixes.is_empty() {
        Vec::new()
    } else {
        let c: &Config = cfg;
        crate::par::map(&modules, || (), |_, m| c.module_text(m).map(|t| replace_in_code(&t, &prefixes, &old, new)).filter(|(_, n)| *n > 0).map(|(t, n)| (m.clone(), t, n)))
    };
    for (m, nt, n) in edits.into_iter().flatten() {
        let before = cfg.entries[cfg.index[&m]].data.clone();
        cfg.set_module_text(&m, &nt)?;
        ch.push(Change { entry: m.clone(), before, after: cfg.entries[cfg.index[&m]].data.clone() });
        found += n;
    }
    Ok((ch, found))
}

/// uuid в байтах — по таблице (та же длина: вложенный контейнер записи не ломается); прочее как было.
fn remap_bytes(b: &[u8], map: &std::collections::HashMap<String, String>) -> Vec<u8> {
    let mut out = b.to_vec();
    let hex = |c: &u8| c.is_ascii_hexdigit();
    let mut i = 0;
    while i + 36 <= out.len() {
        let w = &out[i..i + 36];
        if w[8] == b'-' && w[13] == b'-' && w[18] == b'-' && w[23] == b'-' && w.iter().enumerate().all(|(k, c)| matches!(k, 8 | 13 | 18 | 23) || hex(c)) {
            if let Some(n) = std::str::from_utf8(w).ok().and_then(|u| map.get(u)) {
                out[i..i + 36].copy_from_slice(n.as_bytes());
            }
            i += 36;
        } else {
            i += 1;
        }
    }
    out
}

/// Копия справочника по F9, как у Конфигуратора 8.5 (эталон copy.cf, constructor-ff): имя — основа и первый
/// свободный номер («Справочник1» → «Справочник2»), синоним и тексты модулей — как были, все uuid (объект, вложенные,
/// генераторы, формы, команды) — новые; в описании — явные стандартные реквизиты по умолчанию у объекта и табличных
/// частей, где их не было (так пишет Конфигуратор при копировании). Записи форм и модулей — те же байты с новыми uuid
/// (ссылки записей — по uuid). Других видов эталона копирования нет — отказ. (uuid копии, замены записей).
pub fn copy(cfg: &Config, names: &mut Names, id: &str) -> Result<(String, Vec<Change>), String> {
    let o = find(cfg, id).ok_or(format!("нет объекта {id}"))?;
    if o.class != "Catalog" || o.sub.is_some() {
        return Err(format!("копирование {}: нет эталона Конфигуратора (есть у справочника)", o.class));
    }
    let (class, old) = (o.class, o.item.header.name.clone());
    let taken: Vec<String> = cfg.groups.iter().filter(|g| g.class == class).flat_map(|g| g.items.iter().map(|i| i.header.name.clone())).collect();
    let name = free_name(old.trim_end_matches(|c: char| c.is_ascii_digit()), &taken);
    let (ids, mine) = own(cfg, names, &o)?;
    let map: std::collections::HashMap<String, String> = ids.iter().map(|u| (u.clone(), uuid4())).collect();
    let new_id = map[id].clone();
    // описание: пути и генераторы — с новым именем (как у rename), стандартные реквизиты — явно, uuid — по таблице
    let mut xml = export(cfg, names, &o)?;
    let (from, to) = (format!("{class}.{old}"), format!("{class}.{name}"));
    for end in [".", "<"] {
        xml = xml.replace(&format!("{from}{end}"), &format!("{to}{end}"));
    }
    xml = xml.split('\n').map(|l| if l.contains("<xr:GeneratedType name=\"") { l.replace(&format!(".{old}\""), &format!(".{name}\"")).replace(&format!(".{old}."), &format!(".{name}.")) } else { l.to_string() }).collect::<Vec<_>>().join("\n");
    let p = props(&xml, class)?;
    xml = set(&xml, p.iter().find(|p| p.tag == "Name").ok_or("нет имени")?, &Value::Text(name.clone()));
    let tpl = include_str!("new/CatalogCopy.xml").replace("\r\n", "\n").replace('\n', "\r\n");
    let block = |ind: &str| element_at(&tpl, ind, "StandardAttributes>").map(|(s, e)| tpl[s..e].to_string()).ok_or("эталон копии: нет StandardAttributes");
    let (obj_sa, ts_sa) = (block("\t\t\t")?, block("\t\t\t\t\t")?);
    let props_end = xml.find("\r\n\t\t</Properties>").ok_or("описание: нет Properties")?;
    if !xml[..props_end].contains("\r\n\t\t\t<StandardAttributes>") {
        let at = xml.find("\r\n\t\t\t<DefaultPresentation>").and_then(|p| xml[p + 2..].find("\r\n").map(|q| p + 2 + q)).ok_or("описание: нет DefaultPresentation")?;
        xml.insert_str(at, &format!("\r\n{obj_sa}"));
    }
    // табличные части без стандартных реквизитов — после FillChecking своих Properties
    let mut from_pos = 0;
    while let Some(p) = xml[from_pos..].find("\r\n\t\t\t<TabularSection uuid=").map(|p| from_pos + p + 2) {
        let end = element_end(&xml, p + 3).ok_or("описание: табличная часть")?;
        let pe = p + xml[p..end].find("</Properties>").unwrap_or(end - p);
        if !xml[p..pe].contains("<StandardAttributes>") {
            if let Some(fc) = xml[p..pe].find("\r\n\t\t\t\t\t<FillChecking>").and_then(|f| xml[p + f + 2..pe].find("\r\n").map(|q| p + f + 2 + q)) {
                xml.insert_str(fc, &format!("\r\n{ts_sa}"));
            }
        }
        from_pos = element_end(&xml, p + 3).unwrap_or(xml.len());
    }
    let xml = String::from_utf8(remap_bytes(xml.as_bytes(), &map)).map_err(|_| "описание не UTF-8")?;
    // индекс загрузки: новые полные имена (объект, вложенные, формы) → новые uuid, генераторы — новые TypeId
    let moved: Vec<(String, String)> = names.imp.full.iter().filter(|(k, _)| *k == &from || k.starts_with(&format!("{from}."))).filter_map(|(k, v)| Some((format!("{to}{}", &k[from.len()..]), map.get(v)?.clone()))).collect();
    names.imp.full.extend(moved);
    let gen: Vec<(String, (String, String))> = names.imp.types.iter().filter_map(|(k, (t, v))| {
        let (g, r) = k.split_once('.')?;
        (r == old || r.starts_with(&format!("{old}."))).then(|| (format!("{g}.{name}{}", &r[old.len()..]), (map.get(t).cloned().unwrap_or_default(), map.get(v).cloned().unwrap_or_default())))
    }).collect();
    names.imp.types.extend(gen);
    let rec = desc::record(&dom::parse(&xml)?, class, None, None, &names.imp)?.ok_or("загрузка описания не поддержана")?;
    let mut ch = vec![Change { entry: new_id.clone(), before: Vec::new(), after: crate::container::deflate(&rec) }];
    // прочие записи объекта: те же данные с новыми uuid, имя записи — с новым uuid
    for e in mine.iter().filter(|e| e.as_str() != id) {
        let Some(&i) = cfg.index.get(e) else { continue };
        let (head, tail) = e.split_once('.').map_or((e.as_str(), ""), |(h, t)| (h, t));
        let ne = match (map.get(head), tail) {
            (Some(h), "") => h.clone(),
            (Some(h), t) => format!("{h}.{t}"),
            _ => return Err(format!("запись {e}: uuid не из объекта")),
        };
        let data = remap_bytes(&crate::container::content(&cfg.entries[i]), &map);
        ch.push(Change { entry: ne, before: Vec::new(), after: crate::container::deflate(&data) });
    }
    // корень: новый элемент ChildObjects за последним того же класса
    let root = find(cfg, "root").ok_or("нет корня")?;
    let rx = export(cfg, names, &root)?;
    let own = format!("\r\n\t\t\t<{class}>");
    let el = format!("\t\t\t<{class}>{}</{class}>", super::w::escape(&name));
    let rx = match rx.rfind(&own).and_then(|p| rx[p + 2..].find("\r\n").map(|q| p + 2 + q)) {
        Some(e) => format!("{}\r\n{el}{}", &rx[..e], &rx[e..]),
        None => insert_child(&rx, 0, "\t\t", &el)?,
    };
    names.imp.full.insert(to, new_id.clone());
    let rrec = import(&rx, names, &root)?;
    let &ri = cfg.index.get(&cfg.root_id).ok_or("нет записи корня")?;
    ch.push(Change { entry: cfg.root_id.clone(), before: cfg.entries[ri].data.clone(), after: crate::container::deflate(&rrec) });
    Ok((new_id, ch))
}

#[cfg(test)]
mod tests {
    #[test]
    fn form_element_props() {
        let x = include_str!("new/designer/c1-item.xml");
        let p = super::form_props(x, Some("1")).unwrap();
        let get = |t: &str| p.iter().find(|p| p.tag == t).unwrap();
        assert_eq!(p[0].tag, "DataPath");
        assert_eq!(get("ExtendedEditMultipleValues").kind, super::Kind::Bool);
        assert_eq!(get("Wrap").shown(), "true");
        assert!(p.len() > 100, "{}", p.len());
        assert_eq!(super::form_caption("Wrap"), "Переносить");
        let y = super::set(x, get("DataPath"), &super::Value::Text("Объект.Description".into()));
        assert!(y.contains("<InputField name=\"Код\" id=\"1\">\r\n\t\t\t<DataPath>Объект.Description</DataPath>"));
        // нет в XML — вставка последней строкой свойств; умолчание — строка уходит
        let y = super::set(x, get("Wrap"), &super::Value::Text("false".into()));
        assert!(y.contains("<ExtendedEditMultipleValues>true</ExtendedEditMultipleValues>\r\n\t\t\t<Wrap>false</Wrap>\r\n\t\t\t<ContextMenu name=\"КодКонтекстноеМеню\""), "{}", &y[..1800]);
        let y = super::set(x, get("ExtendedEditMultipleValues"), &super::Value::Text("false".into()));
        assert!(y.contains("<EditMode>EnterOnInput</EditMode>\r\n\t\t\t<ContextMenu name=\"КодКонтекстноеМеню\""));
        assert_eq!(super::set(x, get("Wrap"), &super::Value::Text("true".into())), x);
        let root = super::form_props(x, None).unwrap();
        assert_eq!(root.iter().find(|p| p.tag == "WindowOpeningMode").unwrap().shown(), "LockOwner");
    }

    /// Зависимости прав справочника из графа платформы = снятым перебором в Конфигураторе (порядок и состав).
    #[test]
    fn rights_like_designer() {
        let (n, rows) = super::super::imp::rights::kind_deps("Catalog").unwrap();
        assert_eq!(n.to_vec(), super::CATALOG_RIGHTS.iter().map(|r| r.0).collect::<Vec<_>>());
        for (k, (r, implied)) in super::CATALOG_RIGHTS.iter().enumerate() {
            let from_graph: Vec<&str> = (0..n.len()).filter(|&f| rows[f].as_bytes()[k] == b'+').map(|f| n[f]).collect();
            let mut want = implied.to_vec();
            want.sort();
            let mut got = from_graph.clone();
            got.sort();
            assert_eq!(got, want, "{r}");
        }
        assert!(super::right_names("Document").contains(&"Posting"));
    }

    use super::*;

    const XML: &str = "\u{feff}<?xml version=\"1.0\" encoding=\"UTF-8\"?>\r\n<MetaDataObject xmlns=\"x\" version=\"2.21\">\r\n\t<Catalog uuid=\"u\">\r\n\t\t<InternalInfo>\r\n\t\t\t<xr:GeneratedType name=\"CatalogObject.В\" category=\"Object\">\r\n\t\t\t\t<xr:TypeId>t</xr:TypeId>\r\n\t\t\t</xr:GeneratedType>\r\n\t\t</InternalInfo>\r\n\t\t<Properties>\r\n\t\t\t<Name>В</Name>\r\n\t\t\t<Synonym>\r\n\t\t\t\t<v8:item>\r\n\t\t\t\t\t<v8:lang>ru</v8:lang>\r\n\t\t\t\t\t<v8:content>Валюты &amp; курсы</v8:content>\r\n\t\t\t\t</v8:item>\r\n\t\t\t</Synonym>\r\n\t\t\t<Comment/>\r\n\t\t\t<Hierarchical>false</Hierarchical>\r\n\t\t\t<HierarchyType>HierarchyFoldersAndItems</HierarchyType>\r\n\t\t\t<CodeLength>3</CodeLength>\r\n\t\t\t<Owners/>\r\n\t\t\t<DefaultObjectForm>Catalog.В.Form.Ф</DefaultObjectForm>\r\n\t\t\t<MaxValue xsi:nil=\"true\"/>\r\n\t\t</Properties>\r\n\t\t<ChildObjects/>\r\n\t</Catalog>\r\n</MetaDataObject>";

    #[test]
    fn props_and_set_roundtrip() {
        let p = props(XML, "Catalog").unwrap();
        let tags: Vec<(&str, Kind)> = p.iter().map(|p| (p.tag.as_str(), p.kind)).collect();
        assert_eq!(tags[..4], [("Name", Kind::Str), ("Synonym", Kind::Ml), ("Comment", Kind::Str), ("Hierarchical", Kind::Bool)]);
        assert!(matches!(p[4].kind, Kind::Enum(d) if d.iter().any(|(_, v)| *v == "HierarchyOfItems")));
        assert!(matches!((p[5].kind, p[6].kind, p[7].kind, p[8].kind), (Kind::Num, Kind::List(_), Kind::Ref(FORM), Kind::ReadOnly)));
        assert_eq!((&p[6].value, &p[7].value), (&Value::List(vec![]), &Value::Text("Catalog.В.Form.Ф".into())));
        assert_eq!(p[1].value, Value::Ml(vec![("ru".into(), "Валюты & курсы".into())]));
        // то же значение — тот же текст
        for q in p.iter().filter(|q| q.kind != Kind::ReadOnly) {
            assert_eq!(set(XML, q, &q.value), XML, "{}", q.tag);
        }
        let x = set(XML, &p[1], &Value::Ml(vec![("ru".into(), "Новые <валюты>".into()), ("en".into(), "Currencies".into())]));
        assert!(x.contains("\t\t\t<Synonym>\r\n\t\t\t\t<v8:item>\r\n\t\t\t\t\t<v8:lang>ru</v8:lang>\r\n\t\t\t\t\t<v8:content>Новые &lt;валюты&gt;</v8:content>\r\n\t\t\t\t</v8:item>\r\n\t\t\t\t<v8:item>\r\n\t\t\t\t\t<v8:lang>en</v8:lang>"), "{x}");
        let q = props(&x, "Catalog").unwrap();
        assert_eq!(q[1].shown(), "Новые <валюты>");
        let y = set(&x, &q[3], &Value::Text("true".into()));
        assert!(y.contains("<Hierarchical>true</Hierarchical>"));
        assert_eq!(set(&y, &props(&y, "Catalog").unwrap()[1], &Value::Ml(vec![])).matches("<Synonym/>").count(), 1);
        // список владельцев: запись и чтение обратно, пустой — `<Owners/>`
        let owners = Value::List(vec!["Catalog.А".into(), "Catalog.Б".into()]);
        let z = set(&y, &props(&y, "Catalog").unwrap()[6], &owners);
        assert!(z.contains("\t\t\t<Owners>\r\n\t\t\t\t<xr:Item xsi:type=\"xr:MDObjectRef\">Catalog.А</xr:Item>\r\n\t\t\t\t<xr:Item xsi:type=\"xr:MDObjectRef\">Catalog.Б</xr:Item>\r\n\t\t\t</Owners>\r\n"), "{z}");
        let q = props(&z, "Catalog").unwrap();
        assert_eq!(q[6].value, owners);
        assert_eq!(set(&z, &q[6], &q[6].value), z);
        assert_eq!(set(&z, &q[6], &Value::List(vec![])), y);
    }

    #[test]
    fn rename_text() {
        assert_eq!(synonym_of("Справочник1Новый"), "Справочник1 новый");
        assert_eq!(synonym_of("ЗагрузкаЭДО"), "Загрузка ЭДО");
        let p = managers("Catalog");
        let t = "Процедура Тест() Экспорт\r\n\tСсылка = Справочники.Справочник1.ПустаяСсылка(); // Справочники.Справочник1\r\n\tА = справочники.СПРАВОЧНИК1;\r\n\tБ = Справочники.Справочник10; В = \"Справочники.Справочник1\";\r\n\tГ = Х.Справочники.Справочник1;\r\nКонецПроцедуры";
        let (r, n) = replace_in_code(t, &p, "Справочник1", "Новый");
        assert_eq!(n, 2, "{r}");
        assert!(r.contains("Справочники.Новый.ПустаяСсылка(); // Справочники.Справочник1") && r.contains("справочники.Новый;"), "{r}");
        assert!(r.contains("Справочники.Справочник10;") && r.contains("\"Справочники.Справочник1\"") && r.contains("Х.Справочники.Справочник1"), "{r}");
    }

    #[test]
    fn nested_props() {
        let x = XML.replace(
            "<ChildObjects/>",
            "<ChildObjects>
			<Attribute uuid=\"a1\">
				<Properties>
					<Name>Р</Name>
					<Indexing>Index</Indexing>
				</Properties>
			</Attribute>
		</ChildObjects>",
        );
        let p = props_of(&x, "Attribute", Some("a1")).unwrap();
        assert_eq!(p.iter().map(|p| p.tag.as_str()).collect::<Vec<_>>(), ["Name", "Indexing"]);
        assert!(matches!(p[1].kind, Kind::Enum(_)));
        let y = set(&x, &p[1], &Value::Text("DontIndex".into()));
        assert_eq!(y, x.replace("<Indexing>Index</Indexing>", "<Indexing>DontIndex</Indexing>"));
        // свойства владельца не задеты, нет элемента — ошибка
        assert_eq!(props(&y, "Catalog").unwrap()[0].shown(), "В");
        assert!(props_of(&x, "Attribute", Some("нет")).is_err());
    }

    #[test]
    fn types_edit() {
        let xml = "<P>\r\n\t<Properties>\r\n\t\t<Type>\r\n\t\t\t<v8:Type>xs:string</v8:Type>\r\n\t\t\t<v8:StringQualifiers>\r\n\t\t\t\t<v8:Length>10</v8:Length>\r\n\t\t\t\t<v8:AllowedLength>Variable</v8:AllowedLength>\r\n\t\t\t</v8:StringQualifiers>\r\n\t\t</Type>\r\n\t\t<FillValue xsi:type=\"xs:string\"/>\r\n\t</Properties>\r\n</P>";
        let at = xml.find("<Type>").unwrap();
        let el = dom::parse(&xml[at..element_end(xml, at).unwrap()]).unwrap();
        let t = Types::of(&el).unwrap();
        assert_eq!((t.shown(), t.string), ("Строка".to_string(), Some((10, false))));
        // тот же текст при записи без правки
        assert_eq!(t.xml("Type", "\t\t"), &xml[at..element_end(xml, at).unwrap()]);
        let mut n = parse_types("<Type><v8:Type>cfg:CatalogRef.Валюты</v8:Type><v8:Type>xs:decimal</v8:Type><v8:Type>xs:dateTime</v8:Type></Type>").unwrap();
        n.normalize();
        assert_eq!((n.number, n.string, n.date.as_deref()), (Some((0, 0, false)), None, Some("DateTime")));
        assert_eq!(n.shown(), "СправочникСсылка.Валюты, Число, Дата");
        let prop = Prop { tag: "Type".into(), kind: Kind::Type, value: Value::Type(t), compound: false, span: at..element_end(xml, at).unwrap(), indent: "\t\t".into(), default: None };
        let out = set(xml, &prop, &Value::Type(n));
        assert!(out.contains("<FillValue xsi:nil=\"true\"/>") && out.contains("<v8:DateFractions>DateTime</v8:DateFractions>"), "{out}");
        assert!(parse_types("<Type><v8:TypeId>x</v8:TypeId></Type>").is_err());
    }

    #[test]
    fn availability_rules() {
        let p = props(XML, "Catalog").unwrap();
        // не иерархический: вид иерархии и количество уровней недоступны; нет владельцев — использование подчинения
        assert!(!available(&p, "HierarchyType") && !available(&p, "LevelCount") && !available(&p, "SubordinationUse"));
        assert!(available(&p, "CodeLength") && available(&p, "Name"));
        let x = set(XML, &p[3], &Value::Text("true".into()));
        let q = props(&x, "Catalog").unwrap();
        // LimitLevelCount в описании нет — количество уровней не ограничено этим флажком
        assert!(available(&q, "HierarchyType") && available(&q, "LevelCount"));
        assert!(available(&q, "DefaultFolderForm"));
        assert_eq!(children(XML), Vec::<(String, String)>::new());
    }
}
