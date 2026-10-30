//! Описания объектов «инструментальных» классов: обработки, отчеты, общие формы, макеты, картинки, подсистемы,
//! регламентные задания, XDTO-пакеты, web- и HTTP-сервисы, WS-ссылки, критерии отбора.
//! Позиции сверены со всеми объектами БП 3.0.171 (таблица — docs/research/xml-tools.md).

use super::classes::{props, P};
use super::common::*;
use super::w::W;
use crate::brace::{is_guid, Node};
use crate::meta::{Config, Item};

const COMMON_PICTURE: &[(&str, P)] = &[("AvailabilityForChoice", P::Bool(&[1, 2])), ("AvailabilityForAppearance", P::Bool(&[1, 3]))];

/// Вид макета (коды — TemplateType 1C:EDT).
pub const TEMPLATE_TYPE: &[(&str, &str)] = &[
    ("0", "SpreadsheetDocument"),
    ("1", "BinaryData"),
    ("2", "ActiveDocument"),
    ("3", "HTMLDocument"),
    ("4", "TextDocument"),
    ("5", "GeographicalSchema"),
    ("6", "DataCompositionSchema"),
    ("7", "DataCompositionAppearanceTemplate"),
    ("8", "GraphicalSchema"),
    ("9", "AddIn"),
];

const COMMON_TEMPLATE: &[(&str, P)] = &[("TemplateType", P::Tok(&[1, 2], TEMPLATE_TYPE))];

/// Форма объекта: `{13,{заголовок},<справка>,<вид>,{назначения}}` лежит в корень[1] (регистры бухгалтерии,
/// бизнес-процессы, задачи), в `{0,{13…}}` (справочники, документы, регистры…) или в `{1,{0,{13…},{расширенное
/// представление}}}` (обработки и отчеты — только у них в выгрузке есть ExtendedPresentation).
fn form_parts(n: &Node) -> (Option<&Node>, Option<&Node>) {
    let r = n.get(1);
    match r.and_then(|r| r.get(0)).and_then(Node::atom) {
        Some("1") => (r.and_then(|r| r.at(&[1, 1])), r.and_then(|r| r.at(&[1, 2]))),
        Some("0") => (r.and_then(|r| r.get(1)), None),
        _ => (r, None),
    }
}

const FORM: &[(&str, P)] = &[
    // 0 Ordinary — по двум формам планов обмена
    ("FormType", P::Custom(|w, _, n| form_prop(w, n, "FormType", 3, &[("0", "Ordinary"), ("1", "Managed")]))),
    ("IncludeHelpInContents", P::Custom(|w, _, n| form_prop(w, n, "IncludeHelpInContents", 2, &[("0", "false"), ("1", "true")]))),
    ("UsePurposes", P::Custom(|w, _, n| match form_parts(n).0.and_then(|f| f.get(4)) {
        Some(u) => use_purposes(w, Some(u)),
        // старая запись {9,…} без назначений — в выгрузке оба (одна форма плана обмена)
        None => {
            w.open("UsePurposes", &[]);
            for v in ["PlatformApplication", "MobilePlatformApplication"] {
                w.text_attrs("v8:Value", &[("xsi:type", "app:ApplicationUsePurpose")], v);
            }
            w.end();
        }
    })),
    ("UseInInterfaceCompatibilityMode", P::Const("Any")),
    ("ExtendedPresentation", P::Custom(|w, _, n| {
        if let Some(e) = form_parts(n).1 {
            mlstring(w, "ExtendedPresentation", Some(e));
        }
    })),
];

fn form_prop(w: &mut W, n: &Node, tag: &str, i: usize, map: &[(&str, &str)]) {
    match form_parts(n).0 {
        Some(f) => token(w, tag, f, &[i], map),
        None => w.text(tag, "\u{1}нет свойств формы"),
    }
}

const COMMON_FORM: &[(&str, P)] = &[
    // 1 — управляемая; обычных форм в БП нет
    ("FormType", P::Tok(&[1, 1, 3], &[("0", "Ordinary"), ("1", "Managed")])),
    ("IncludeHelpInContents", P::Bool(&[1, 1, 2])),
    ("UsePurposes", P::Custom(|w, _, n| use_purposes(w, n.at(&[1, 1, 4])))),
    // свойство 8.5 (FormPreferableInterface 1C:EDT), по каркасу — в теле формы [5]; у старых записей его нет
    ("UseInInterfaceCompatibilityMode", P::TokOr(&[1, 1, 5], super::classes_nested::FORM_INTERFACE, "Any")),
    ("UseStandardCommands", P::Bool(&[1, 4])),
    ("ExtendedPresentation", P::Ml(&[1, 2])),
    ("Explanation", P::Ml(&[1, 3])),
];

/// Назначения `{N,{"#",1708fdaa-…,<код>},...}`.
pub fn use_purposes(w: &mut W, n: Option<&Node>) {
    let items = n.map(Node::list).unwrap_or_default().get(1..).unwrap_or_default();
    if items.is_empty() {
        w.text("UsePurposes", "");
        return;
    }
    w.open("UsePurposes", &[]);
    for it in items {
        let v = match it.get(2).and_then(Node::atom) {
            Some("1") => "PlatformApplication".to_string(),
            Some("2") => "MobilePlatformApplication".to_string(),
            s => format!("\u{1}{}", s.unwrap_or_default()),
        };
        w.text_attrs("v8:Value", &[("xsi:type", "app:ApplicationUsePurpose")], &v);
    }
    w.end();
}

const DATA_PROCESSOR: &[(&str, P)] = &[
    ("UseStandardCommands", P::Bool(&[1, 5])),
    ("DefaultForm", P::Ref(&[1, 4])),
    // в эталоне всегда пусто; единственная свободная позиция с uuid
    ("AuxiliaryForm", P::Ref(&[1, 9])),
    ("IncludeHelpInContents", P::Bool(&[1, 6])),
    ("ExtendedPresentation", P::Ml(&[1, 10])),
    ("Explanation", P::Ml(&[1, 11])),
];

const REPORT: &[(&str, P)] = &[
    ("UseStandardCommands", P::Bool(&[1, 7])),
    ("DefaultForm", P::Ref(&[1, 4])),
    ("AuxiliaryForm", P::Custom(|w, _, n| report_unknown(w, "AuxiliaryForm", n))),
    ("MainDataCompositionSchema", P::Ref(&[1, 5])),
    ("DefaultSettingsForm", P::Ref(&[1, 6])),
    ("AuxiliarySettingsForm", P::Custom(|w, _, n| report_unknown(w, "AuxiliarySettingsForm", n))),
    ("DefaultVariantForm", P::Ref(&[1, 10])),
    // свойство 8.5: в выгрузке 8.3.27 его нет
    ("AuxiliaryVariantForm", P::Const("")),
    ("VariantsStorage", P::Ref(&[1, 8])),
    ("SettingsStorage", P::Ref(&[1, 9])),
    ("IncludeHelpInContents", P::Bool(&[1, 11])),
    ("ExtendedPresentation", P::Ml(&[1, 15])),
    ("Explanation", P::Ml(&[1, 16])),
];

/// AuxiliaryForm и AuxiliarySettingsForm в эталонах всегда пусты, свободных позиций с uuid две — (1,14), (1,17): какое где,
/// не установлено. Пока обе нулевые — пусто, иначе `?` со значениями. SettingsStorage — (1,9): БП 3.0.107 АнализПравДоступа
/// (там же хранилище, что и в VariantsStorage).
fn report_unknown(w: &mut W, tag: &str, n: &Node) {
    let v: Vec<&str> = [14, 17].iter().map(|&i| atom(n, &[1, i])).collect();
    if v.iter().all(|a| *a == ZERO) {
        w.text(tag, "");
    } else {
        w.text(tag, &format!("\u{1}{}", v.join(",")));
    }
}

const SCHEDULED_JOB: &[(&str, P)] = &[
    ("MethodName", P::Custom(|w, x, n| method(w, "MethodName", x, n, &[1, 6], &[1, 7]))),
    ("Description", P::Str(&[1, 3])),
    ("Key", P::Str(&[1, 2])),
    ("Use", P::Bool(&[1, 4])),
    ("Predefined", P::Bool(&[1, 5])),
    ("RestartCountOnFailure", P::Custom(|w, _, n| w.text("RestartCountOnFailure", atom(n, &[1, 8])))),
    ("RestartIntervalOnFailure", P::Custom(|w, _, n| w.text("RestartIntervalOnFailure", atom(n, &[1, 9])))),
];

/// uuid общего модуля и имя процедуры → `CommonModule.Имя.Процедура`; нулевой модуль — пусто.
fn method(w: &mut W, tag: &str, x: &Index, n: &Node, module: &[usize], proc_: &[usize]) {
    let m = x.name(atom(n, module));
    w.text(tag, &if m.is_empty() { String::new() } else { format!("{m}.{}", string(n, proc_)) });
}

const XDTO_PACKAGE: &[(&str, P)] = &[("Namespace", P::Str(&[1, 2]))];

const WS_REFERENCE: &[(&str, P)] = &[("LocationURL", P::Str(&[1, 1, 0]))];

const FILTER_CRITERION: &[(&str, P)] = &[
    ("Type", P::Type(&[1, 5, 2])),
    ("UseStandardCommands", P::Bool(&[1, 7])),
    ("Content", P::Custom(|w, x, n| md_list(w, "Content", "xr:Item", &[("xsi:type", "xr:MDObjectRef")], x, n.at(&[1, 6])))),
    // в эталоне пусты; позиции — по порядку, как у перечисления
    ("DefaultForm", P::Ref(&[1, 8])),
    ("AuxiliaryForm", P::Ref(&[1, 9])),
    ("ListPresentation", P::Ml(&[1, 10])),
    ("ExtendedListPresentation", P::Ml(&[1, 11])),
    ("Explanation", P::Ml(&[1, 12])),
];

/// Список `{0,N,<ссылка на объект метаданных>...}`.
fn md_list(w: &mut W, tag: &str, item: &str, attrs: &[(&str, &str)], x: &Index, n: Option<&Node>) {
    let items = n.map(Node::list).unwrap_or_default().get(2..).unwrap_or_default();
    if items.is_empty() {
        w.text(tag, "");
        return;
    }
    w.open(tag, &[]);
    for it in items {
        w.text_attrs(item, attrs, &x.name(md_ref(it).unwrap_or_default()));
    }
    w.end();
}

const SUBSYSTEM: &[(&str, P)] = &[
    ("IncludeHelpInContents", P::Bool(&[1, 2])),
    ("IncludeInCommandInterface", P::Bool(&[1, 4])),
    ("UseOneCommand", P::Bool(&[1, 8])),
    ("Explanation", P::Ml(&[1, 6])),
    ("Picture", P::Custom(|w, x, n| super::ui::picture(w, "Picture", x, n.at(&[1, 5])))),
    ("Content", P::Custom(|w, x, n| md_list(w, "Content", "xr:Item", &[("xsi:type", "xr:MDObjectRef")], x, n.at(&[1, 7])))),
];

/// 0 DontUse, 1 Use, 2 AutoUse (сверено по HTTP-сервисам).
const REUSE_SESSIONS: &[(&str, &str)] = &[("0", "DontUse"), ("1", "Use"), ("2", "AutoUse")];

const WEB_SERVICE: &[(&str, P)] = &[
    ("Namespace", P::Str(&[1, 1])),
    ("XDTOPackages", P::Custom(ws_packages)),
    ("DescriptorFileName", P::Str(&[1, 4])),
    ("ReuseSessions", P::Tok(&[1, 6], REUSE_SESSIONS)),
    ("SessionMaxAge", P::Custom(|w, _, n| session_max_age(w, n, &[1, 7], 3))),
];

/// Время жизни сеанса: читается из записи версии (1,0) > `since` (web-сервис — 3, HTTP-сервис — 1), иначе 20
/// (backend FUN_180f44060 и FUN_180957d70 ставят 0x14 до чтения).
fn session_max_age(w: &mut W, n: &Node, path: &[usize], since: u32) {
    let ver: u32 = n.at(&[1, 0]).and_then(Node::atom).and_then(|v| v.parse().ok()).unwrap_or(0);
    w.text("SessionMaxAge", if ver > since { atom(n, path) } else { "20" });
}

/// Пакеты web-сервиса: ссылки на XDTO-пакеты (1,3) `{0,N,<ссылка>...}`, затем пространства имен (1,5) `{N,"ns"...}`.
fn ws_packages(w: &mut W, x: &Index, n: &Node) {
    let refs = n.at(&[1, 3]).map(Node::list).unwrap_or_default().get(2..).unwrap_or_default();
    let names = n.at(&[1, 5]).map(Node::list).unwrap_or_default().get(1..).unwrap_or_default();
    if refs.is_empty() && names.is_empty() {
        w.text("XDTOPackages", "");
        return;
    }
    w.open("XDTOPackages", &[]);
    let item = |w: &mut W, t: &str, v: &str| {
        w.open("xr:Item", &[]);
        w.text("xr:Presentation", "");
        w.text("xr:CheckState", "0");
        w.text_attrs("xr:Value", &[("xsi:type", t)], v);
        w.end();
    };
    for r in refs {
        item(w, "xr:MDObjectRef", &x.name(md_ref(r).unwrap_or_default()));
    }
    for s in names {
        item(w, "xs:string", s.str().unwrap_or_default());
    }
    w.end();
}

const HTTP_SERVICE: &[(&str, P)] = &[
    ("RootURL", P::Str(&[1, 1])),
    ("ReuseSessions", P::Tok(&[1, 3], REUSE_SESSIONS)),
    ("SessionMaxAge", P::Custom(|w, _, n| session_max_age(w, n, &[1, 4], 1))),
];

/// Сервис интеграции `{1,{0,{заголовок},<TypeId>,<ValueId>,"адрес"},1,{acb7e81f-…,N,<каналы>}}`: запись — integ.dll
/// FUN_1800e0a60/FUN_1800f2110, свойства класса и их порядок — FUN_1800f2470 (e900510e), имена — FUN_1800dc080.
const INTEGRATION_SERVICE: &[(&str, P)] = &[("ExternalIntegrationServiceAddress", P::Str(&[1, 4]))];

/// Направление сообщений канала: значения перечисления 1be9d752 (integ.dll FUN_1800dc080).
pub const MESSAGE_DIRECTION: &[(&str, &str)] = &[("0", "Send"), ("1", "Receive")];

/// Канал `{1,{заголовок},<TypeId>,<ValueId>,"имя канала","обработчик",<направление>,<в транзакции>}`: запись — integ.dll
/// FUN_1800e2c30, чтение FUN_1800e2eb0 (версия 0 — без последнего поля, «в транзакции» — истина); свойства по порядку
/// класса FUN_1800f73a0, чтение значений FUN_1800f6f40 (5 имя, 6 обработчик, 7 направление, 8 Transactioned b9460636 —
/// имя из backend.dll FUN_180969720).
const INTEGRATION_CHANNEL: &[(&str, P)] = &[
    ("ExternalIntegrationServiceChannelName", P::Str(&[4])),
    ("MessageDirection", P::Tok(&[6], MESSAGE_DIRECTION)),
    ("ReceiveMessageProcessing", P::Str(&[5])),
    ("Transactioned", P::TokOr(&[7], &[("0", "false"), ("1", "true")], "true")),
];

/// Каналы сервиса интеграции: как [`open_item`], но с генерируемым типом менеджера канала (категория 5 — Manager,
/// FUN_1800e2820/FUN_1800e2a00).
fn channels(w: &mut W, x: &Index, root: &Node, it: &Item) {
    if it.children.is_empty() {
        w.text("ChildObjects", "");
        return;
    }
    w.open("ChildObjects", &[]);
    for c in &it.children {
        let own = element(root, &c.header.id).and_then(|e| e.get(0));
        w.open(c.kind, &[("uuid", &c.header.id)]);
        w.open("InternalInfo", &[]);
        let (t, v) = (own.map(|o| atom(o, &[2])).unwrap_or_default(), own.map(|o| atom(o, &[3])).unwrap_or_default());
        let name = format!("IntegrationServiceChannelManager.{}.{}", it.header.name, c.header.name);
        if is_guid(t) && is_guid(v) {
            w.open("xr:GeneratedType", &[("name", &name), ("category", "Manager")]);
            w.text("xr:TypeId", t);
            w.text("xr:ValueId", v);
            w.end();
        } else {
            w.text("xr:GeneratedType", &format!("\u{1}{name}: нет TypeId/ValueId в записи"));
        }
        w.end();
        w.open("Properties", &[]);
        match own {
            Some(o) => {
                if let Some(h) = head(o) {
                    name_synonym_comment(w, h);
                }
                props(w, x, o, INTEGRATION_CHANNEL);
            }
            None => w.text("Name", "\u{1}элемент не найден"),
        }
        w.end();
        w.end();
    }
    w.end();
}

/// Описание объекта класса из этого файла; `None` — класс не отсюда.
/// Таблица свойств класса, как ее пишет [`descriptor`] (палитра свойств — xml/edit.rs).
pub fn descriptor_table(class: &str) -> Option<&'static [(&'static str, P)]> {
    Some(match class {
        "CommonPicture" => COMMON_PICTURE,
        "CommonTemplate" => COMMON_TEMPLATE,
        // вложенные описания Forms/<имя>.xml (у любого владельца)
        "Form" => FORM,
        "CommonForm" => COMMON_FORM,
        "DataProcessor" => DATA_PROCESSOR,
        "Report" => REPORT,
        "ScheduledJob" => SCHEDULED_JOB,
        "XDTOPackage" => XDTO_PACKAGE,
        "WSReference" => WS_REFERENCE,
        "FilterCriterion" => FILTER_CRITERION,
        "Subsystem" => SUBSYSTEM,
        "WebService" => WEB_SERVICE,
        "HTTPService" => HTTP_SERVICE,
        "IntegrationService" => INTEGRATION_SERVICE,
        _ => return None,
    })
}

pub fn descriptor(cfg: &Config, x: &Index, class: &str, it: &Item) -> Option<Vec<u8>> {
    let table = descriptor_table(class)?;
    let root = object_brace(cfg, it)?;
    let mut w = W::metadata();
    w.open(class, &[("uuid", &it.header.id)]);
    internal_info(&mut w, class, &it.header.id, &it.header.name, &root);
    w.open("Properties", &[]);
    if let Some(h) = head(&root) {
        name_synonym_comment(&mut w, h);
    }
    props(&mut w, x, &root, table);
    w.end();
    match class {
        "DataProcessor" | "Report" | "FilterCriterion" => object_children(&mut w, x, class, it, &root),
        "Subsystem" => names(&mut w, it, &["Subsystem"]),
        "WebService" => nested(&mut w, x, &root, &it.children, &WS_LEVELS),
        "HTTPService" => nested(&mut w, x, &root, &it.children, &HTTP_LEVELS),
        "IntegrationService" => channels(&mut w, x, &root, it),
        _ => {}
    }
    Some(w.finish())
}

/// Свойства внешней обработки: позиции в теле записи корня (`common::external_body`).
pub const EXTERNAL_DATA_PROCESSOR: &[(&str, P)] = &[("DefaultForm", P::Ref(&[1, 4])), ("AuxiliaryForm", P::Ref(&[1, 6]))];

/// `<Имя>.xml` внешней обработки: `uuid` — запись корня, `ObjectId` — заголовок тела; реквизиты, табличные части,
/// формы и макеты — как у обработки конфигурации.
pub fn external_data_processor(cfg: &Config, x: &Index) -> Option<Vec<u8>> {
    let (class, it) = (cfg.root.kind, &cfg.root);
    let body = external_body(cfg)?;
    let mut w = W::metadata();
    w.open(class, &[("uuid", &cfg.root_id)]);
    w.open("InternalInfo", &[]);
    w.open("xr:ContainedObject", &[]);
    w.text("xr:ClassId", crate::meta::EXTERNAL_DATA_PROCESSOR);
    w.text("xr:ObjectId", &it.header.id);
    w.end();
    w.open("xr:GeneratedType", &[("name", &format!("{class}Object.{}", it.header.name)), ("category", "Object")]);
    w.text("xr:TypeId", atom(&body, &[1, 1]));
    w.text("xr:ValueId", atom(&body, &[1, 2]));
    w.end();
    w.end();
    w.open("Properties", &[]);
    if let Some(h) = head(&body) {
        name_synonym_comment(&mut w, h);
    }
    props(&mut w, x, &body, EXTERNAL_DATA_PROCESSOR);
    // shortcut: смысл строки [1,5] не известен — во всем корпусе платформы пусто; непустая — отказ, пока не встретится
    if !string(&body, &[1, 5]).is_empty() {
        w.text("Help", &raw(body.at(&[1, 5])));
    }
    w.end();
    // типы табличных частей платформа называет как у обработки конфигурации: DataProcessorTabularSection.<Имя>.<ТЧ>
    object_children(&mut w, x, "DataProcessor", it, &body);
    Some(w.finish())
}

/// Коды HTTP-методов для загрузки XML (`xml/imp/refs.rs`).
pub fn http_methods() -> &'static [(&'static str, &'static str)] {
    HTTP_METHODS
}

/// Таблица свойств класса для загрузки XML (`xml/imp/refs.rs`).
pub fn table(class: &str) -> Option<&'static [(&'static str, P)]> {
    Some(match class {
        "DataProcessor" => DATA_PROCESSOR,
        "ExternalDataProcessor" => EXTERNAL_DATA_PROCESSOR,
        "Report" => REPORT,
        "WSReference" => WS_REFERENCE,
        "FilterCriterion" => FILTER_CRITERION,
        "Subsystem" => SUBSYSTEM,
        "WebService" => WEB_SERVICE,
        "HTTPService" => HTTP_SERVICE,
        "IntegrationService" => INTEGRATION_SERVICE,
        _ => return None,
    })
}

/// ChildObjects из имен вложенных объектов в порядке видов.
fn names(w: &mut W, it: &Item, kinds: &[&str]) {
    let named: Vec<&Item> = kinds.iter().flat_map(|k| it.children.iter().filter(move |c| c.kind == *k)).collect();
    if named.is_empty() {
        w.text("ChildObjects", "");
        return;
    }
    w.open("ChildObjects", &[]);
    for c in named {
        w.text(c.kind, &c.header.name);
    }
    w.end();
}

/// Элемент вложенной коллекции `{<guid>,N,элементы...}` (на любой глубине), чей первый заголовок — `id`.
pub fn element<'a>(n: &'a Node, id: &str) -> Option<&'a Node> {
    let l = n.list();
    let coll = l.len() >= 2 && l[0].atom().is_some_and(is_guid) && l[1].atom().and_then(|a| a.parse::<usize>().ok()) == Some(l.len() - 2);
    if coll {
        for el in &l[2..] {
            if matches!(el, Node::List(_)) && head(el).and_then(|h| h.get(1)?.get(2)?.atom()) == Some(id) {
                return Some(el);
            }
        }
    }
    l.iter().find_map(|c| element(c, id))
}

/// `<Вид uuid>` и Properties элемента: Name/Synonym/Comment и свойства по таблице от узла `own` (элемент[0]).
/// Элемент остается открытым — для ChildObjects.
fn open_item(w: &mut W, x: &Index, c: &Item, own: Option<&Node>, table: &[(&str, P)]) {
    w.open(c.kind, &[("uuid", &c.header.id)]);
    w.open("Properties", &[]);
    match own {
        Some(o) => {
            if let Some(h) = head(o) {
                name_synonym_comment(w, h);
            }
            props(w, x, o, table);
        }
        None => w.text("Name", "\u{1}элемент не найден"),
    }
    w.end();
}

/// Уровень вложенных элементов с полными свойствами: операции → параметры, шаблоны URL → методы.
struct Level {
    table: &'static [(&'static str, P)],
    sub: Option<&'static Level>,
}

const WS_LEVELS: Level = Level { table: OPERATION, sub: Some(&Level { table: PARAMETER, sub: None }) };
const HTTP_LEVELS: Level = Level { table: URL_TEMPLATE, sub: Some(&Level { table: METHOD, sub: None }) };

fn nested(w: &mut W, x: &Index, scope: &Node, items: &[Item], lv: &Level) {
    if items.is_empty() {
        w.text("ChildObjects", "");
        return;
    }
    w.open("ChildObjects", &[]);
    for c in items {
        let el = element(scope, &c.header.id);
        open_item(w, x, c, el.and_then(|e| e.get(0)), lv.table);
        if let (Some(sub), Some(el)) = (lv.sub, el) {
            nested(w, x, el, &c.children, sub);
        }
        w.end();
    }
    w.end();
}

/// Операция `{1,{заголовок},{0,"ns","тип"},<Nillable>,<Transactioned>,"процедура",<блокировка>}`.
const OPERATION: &[(&str, P)] = &[
    ("XDTOReturningValueType", P::Custom(|w, _, n| xdto_type(w, "XDTOReturningValueType", n.get(2)))),
    ("Nillable", P::Bool(&[3])),
    // в эталоне всегда false и Managed
    ("Transactioned", P::Bool(&[4])),
    ("ProcedureName", P::Str(&[5])),
    ("DataLockControlMode", P::Tok(&[6], &[("0", "Automatic"), ("1", "Managed"), ("2", "AutomaticAndManaged")])),
];

/// Параметр `{0,{заголовок},{0,"ns","тип"},<Nillable>,<направление>}`.
const PARAMETER: &[(&str, P)] = &[
    ("XDTOValueType", P::Custom(|w, _, n| xdto_type(w, "XDTOValueType", n.get(2)))),
    ("Nillable", P::Bool(&[3])),
    ("TransferDirection", P::Tok(&[4], &[("0", "In"), ("1", "Out"), ("2", "InOut")])),
];

/// Шаблон URL `{0,"шаблон",{заголовок}}`.
const URL_TEMPLATE: &[(&str, P)] = &[("Template", P::Str(&[1]))];

/// Метод `{0,"обработчик",<HTTP-метод>,{заголовок}}`.
const METHOD: &[(&str, P)] = &[("HTTPMethod", P::Tok(&[2], HTTP_METHODS)), ("Handler", P::Str(&[1]))];

/// Коды — HTTPMethod 1C:EDT (2, 3, 11, 14 встречены в БП).
const HTTP_METHODS: &[(&str, &str)] = &[
    ("0", "CONNECT"),
    ("1", "COPY"),
    ("2", "DELETE"),
    ("3", "GET"),
    ("5", "LOCK"),
    ("6", "MERGE"),
    ("7", "MKCOL"),
    ("8", "MOVE"),
    ("9", "OPTIONS"),
    ("10", "PATCH"),
    ("11", "POST"),
    ("12", "PROPFIND"),
    ("13", "PROPPATCH"),
    ("14", "PUT"),
    ("15", "TRACE"),
    ("16", "UNLOCK"),
    ("1000", "Any"),
];

/// Тип XDTO `{0,"пространство имен","имя"}` → `xs:string`; пространство не из корня — `xmlns:d<глубина>p1`.
fn xdto_type(w: &mut W, tag: &str, n: Option<&Node>) {
    let s = |i: usize| n.and_then(|n| n.get(i)).and_then(Node::str).unwrap_or_default();
    let (ns, name) = (s(1), s(2));
    if name.is_empty() {
        w.text(tag, "");
        return;
    }
    match super::w::prefix(ns) {
        Some(p) => w.text(tag, &format!("{p}:{name}")),
        None => {
            let p = format!("d{}p1", w.depth() + 1);
            w.text_attrs(tag, &[(&format!("xmlns:{p}"), ns)], &format!("{p}:{name}"));
        }
    }
}

/// Реквизит обработки и отчета: блок `{27,...}` — элемент[0][1].
const ATTRIBUTE: &[(&str, P)] = &[
    ("Type", P::Attr(&[1])),
    ("PasswordMode", P::Attr(&[1])),
    ("Format", P::Attr(&[1])),
    ("EditFormat", P::Attr(&[1])),
    ("ToolTip", P::Attr(&[1])),
    ("MarkNegatives", P::Attr(&[1])),
    ("Mask", P::Attr(&[1])),
    ("MultiLine", P::Attr(&[1])),
    ("ExtendedEdit", P::Attr(&[1])),
    ("MinValue", P::Attr(&[1])),
    ("MaxValue", P::Attr(&[1])),
    ("FillChecking", P::Attr(&[1])),
    ("ChoiceFoldersAndItems", P::Attr(&[1])),
    ("ChoiceParameterLinks", P::Attr(&[1])),
    ("ChoiceParameters", P::Attr(&[1])),
    ("QuickChoice", P::Attr(&[1])),
    ("CreateOnInput", P::Attr(&[1])),
    ("ChoiceForm", P::Attr(&[1])),
    ("LinkByType", P::Attr(&[1])),
    ("ChoiceHistoryOnInput", P::Attr(&[1])),
];

/// Реквизит табличной части: то же и FillFromFillingValue, FillValue.
const TS_ATTRIBUTE: &[(&str, P)] = &[
    ("Type", P::Attr(&[1])),
    ("PasswordMode", P::Attr(&[1])),
    ("Format", P::Attr(&[1])),
    ("EditFormat", P::Attr(&[1])),
    ("ToolTip", P::Attr(&[1])),
    ("MarkNegatives", P::Attr(&[1])),
    ("Mask", P::Attr(&[1])),
    ("MultiLine", P::Attr(&[1])),
    ("ExtendedEdit", P::Attr(&[1])),
    ("MinValue", P::Attr(&[1])),
    ("MaxValue", P::Attr(&[1])),
    ("FillFromFillingValue", P::Attr(&[1])),
    ("FillValue", P::Attr(&[1])),
    ("FillChecking", P::Attr(&[1])),
    ("ChoiceFoldersAndItems", P::Attr(&[1])),
    ("ChoiceParameterLinks", P::Attr(&[1])),
    ("ChoiceParameters", P::Attr(&[1])),
    ("QuickChoice", P::Attr(&[1])),
    ("CreateOnInput", P::Attr(&[1])),
    ("ChoiceForm", P::Attr(&[1])),
    ("LinkByType", P::Attr(&[1])),
    ("ChoiceHistoryOnInput", P::Attr(&[1])),
];

/// Табличная часть `{0,{11,<TypeId>,<ValueId>,<TypeId строки>,<ValueId строки>,{0,{заголовок}},<FillChecking>,
/// {1,{стандартные реквизиты}}|{0},<ToolTip>}}`.
const TABULAR_SECTION: &[(&str, P)] = &[
    ("ToolTip", P::Ml(&[1, 8])),
    ("FillChecking", P::Tok(&[1, 6], &[("0", "DontCheck"), ("1", "ShowError")])),
    // {0} — элемента нет
    ("StandardAttributes", P::Custom(|w, x, n| {
        if let Some(sa) = n.at(&[1, 7, 1]) {
            // владелец путей связей — объект (`DataProcessor.Имя`), а не сама табличная часть
            let ts = x.name(head(n).and_then(|h| h[1].get(2)).and_then(Node::atom).unwrap_or_default());
            let owner = ts.splitn(3, '.').take(2).collect::<Vec<_>>().join(".");
            super::standard::standard_attributes(w, x, "TabularSection", &owner, sa);
        }
    })),
];

/// Команда объекта: узел `{<версия>,{2,<uuid>,…},{9,<картинка>,<представление>,<подсказка>,…}}` в записи владельца
/// (у перечисления и критерия отбора — внутри `{0,0,0,…}`); позиции в `{9,…}` те же, что у общей команды.
const COMMAND: &[(&str, P)] = &[
    ("Group", P::Custom(|w, x, n| {
        let id = atom(n, &[2, 7, 1]);
        let g = super::classes::STD_COMMAND_GROUPS.iter().find(|(g, _)| *g == id).map_or_else(|| x.name(id), |(_, nm)| nm.to_string());
        w.text("Group", &g);
    })),
    ("CommandParameterType", P::Type(&[2, 8])),
    ("ParameterUseMode", P::Tok(&[2, 11], &[("0", "Single"), ("1", "Multiple")])),
    ("ModifiesData", P::Bool(&[2, 10])),
    ("Representation", P::Tok(&[2, 2], super::classes::REPRESENTATION)),
    ("ToolTip", P::Ml(&[2, 3])),
    ("Picture", P::Custom(|w, x, n| super::ui::picture(w, "Picture", x, n.at(&[2, 1])))),
    ("Shortcut", P::Custom(|w, _, n| w.text("Shortcut", &super::ui::shortcut(n.at(&[2, 5]))))),
    ("OnMainServerUnavalableBehavior", P::Tok(&[2, 12], super::classes::ON_MAIN_SERVER_UNAVAILABLE)),
];

/// Команда объекта по uuid в записи владельца — полностью.
pub fn command(w: &mut W, x: &Index, c: &Item, root: &Node) {
    w.open("Command", &[("uuid", &c.header.id)]);
    w.open("Properties", &[]);
    match (head_of(root, &c.header.id), command_node(root, &c.header.id)) {
        (Some(h), Some(n)) => {
            name_synonym_comment(w, h);
            props(w, x, n, COMMAND);
        }
        _ => w.text("Name", "\u{1}нет записи команды"),
    }
    w.end();
    w.end();
}

fn command_node<'a>(n: &'a Node, id: &str) -> Option<&'a Node> {
    let l = n.list();
    if l.get(1).is_some_and(|h| atom(h, &[0]) == "2" && atom(h, &[1]) == id) {
        return Some(n);
    }
    l.iter().find_map(|c| command_node(c, id))
}

/// Реквизиты, табличные части (полностью), формы и макеты (именами), команды (полностью).
fn object_children(w: &mut W, x: &Index, class: &str, it: &Item, root: &Node) {
    let kinds = ["Attribute", "TabularSection", "Form", "Template", "Command"];
    let kids: Vec<&Item> = kinds.iter().flat_map(|k| it.children.iter().filter(move |c| c.kind == *k)).collect();
    if kids.is_empty() {
        w.text("ChildObjects", "");
        return;
    }
    w.open("ChildObjects", &[]);
    for c in kids {
        let el = || element(root, &c.header.id);
        match c.kind {
            "Form" | "Template" => w.text(c.kind, &c.header.name),
            "Attribute" => {
                open_item(w, x, c, el().and_then(|e| e.get(0)), ATTRIBUTE);
                w.end();
            }
            "Command" => command(w, x, c, root),
            _ => tabular_section(w, x, class, &it.header.name, c, el()),
        }
    }
    w.end();
}

fn tabular_section(w: &mut W, x: &Index, class: &str, owner: &str, c: &Item, el: Option<&Node>) {
    let own = el.and_then(|e| e.get(0));
    w.open("TabularSection", &[("uuid", &c.header.id)]);
    w.open("InternalInfo", &[]);
    let name = format!("{owner}.{}", c.header.name);
    for (suffix, pos) in [("TabularSection", 1), ("TabularSectionRow", 3)] {
        let (t, v) = own.map(|o| (atom(o, &[1, pos]), atom(o, &[1, pos + 1]))).unwrap_or_default();
        w.open("xr:GeneratedType", &[("name", &format!("{class}{suffix}.{name}")), ("category", suffix)]);
        w.text("xr:TypeId", t);
        w.text("xr:ValueId", v);
        w.end();
    }
    w.end();
    w.open("Properties", &[]);
    if let Some(o) = own {
        if let Some(h) = head(o) {
            name_synonym_comment(w, h);
        }
        props(w, x, o, TABULAR_SECTION);
    }
    w.end();
    let attrs: Vec<&Item> = c.children.iter().filter(|a| a.kind == "Attribute").collect();
    if attrs.is_empty() {
        w.text("ChildObjects", "");
    } else {
        w.open("ChildObjects", &[]);
        for a in attrs {
            open_item(w, x, a, el.and_then(|e| element(e, &a.header.id)).and_then(|e| e.get(0)), TS_ATTRIBUTE);
            w.end();
        }
        w.end();
    }
    w.end();
}

// ---------------------------------------------------------------------------------------------------------------
// Configuration.xml: корень `{2,{<uuid>},<N>,{<ClassId>,{…}}×N,…}`; свойства — в корень[3][1][1].
// Эталон один (БП), второй образец скобок — УТ 11.5.27 с выгрузкой 8.3.27; где значения совпадают у всех
// образцов, позиция выбрана по соседству и пишется только встреченное значение (иное — числом, сверка покажет).

/// Порядок видов в ChildObjects (выгрузки БП и УТ; Bot и IntegrationService — из УТ, meta их пока не знает).
const CONFIG_ORDER: &[&str] = &[
    "Language", "Subsystem", "StyleItem", "Style", "CommonPicture", "SessionParameter", "Role", "CommonTemplate", "FilterCriterion",
    "CommonModule", "CommonAttribute", "ExchangePlan", "XDTOPackage", "WebService", "HTTPService", "WSReference",
    "EventSubscription", "ScheduledJob", "SettingsStorage", "FunctionalOption", "FunctionalOptionsParameter", "DefinedType",
    "Bot", "CommonCommand", "CommandGroup", "Constant", "CommonForm", "Catalog", "Document", "DocumentNumerator", "Sequence",
    "DocumentJournal", "Enum", "Report", "DataProcessor", "InformationRegister", "AccumulationRegister",
    "ChartOfCharacteristicTypes", "ChartOfAccounts", "AccountingRegister", "ChartOfCalculationTypes", "CalculationRegister", "BusinessProcess", "Task",
    "IntegrationService",
];

/// Функциональности мобильного приложения в порядке выгрузки. Код в скобках `{2,N,{<код>,<0|1>}...}` — номер,
/// с 28-го — номер + 4 (коды 28–31 не встречаются; Videoconferences = 34 по УТ). Нет кода — false.
const MOBILE_FUNCTIONALITIES: &[&str] = &[
    "Biometrics", "Location", "BackgroundLocation", "BluetoothPrinters", "WiFiPrinters", "Contacts", "Calendars",
    "PushNotifications", "LocalNotifications", "InAppPurchases", "PersonalComputerFileExchange", "Ads", "NumberDialing",
    "CallProcessing", "CallLog", "AutoSendSMS", "ReceiveSMS", "SMSLog", "Camera", "Microphone", "MusicLibrary",
    "PictureAndVideoLibraries", "AudioPlaybackAndVibration", "BackgroundAudioPlaybackAndVibration", "InstallPackages", "OSBackup",
    "ApplicationUsageStatistics", "BarcodeScanning", "BackgroundAudioRecording", "AllFilesAccess", "Videoconferences", "NFC",
    "DocumentScanning", "SpeechToText", "Geofences", "IncomingShareRequests", "AllIncomingShareRequestsTypesProcessing",
    "TextToSpeech",
];

/// Пустые во всех образцах ссылки: позиции (корень[3][1][1]) есть, какая чья — не установлено.
const CONFIG_ZERO_REFS: &[usize] = &[9, 11, 34, 35, 37, 46, 47, 48, 50, 55];
/// Пустые во всех образцах списки `{0,0}`.
const CONFIG_EMPTY_LISTS: &[usize] = &[18, 20, 27, 52];
/// Пустые во всех образцах строки.
const CONFIG_EMPTY_STRINGS: &[usize] = &[2, 42, 56];

/// Свойство из группы неразличимых позиций: пока все пусты — пусто, иначе `?` со значениями.
fn group(w: &mut W, tag: &str, n: &Node, slots: &[usize], empty: &str) {
    let vals: Vec<String> = slots.iter().map(|&i| n.get(i).map(|v| crate::brace::write(v).unwrap_or_default()).unwrap_or_default()).collect();
    if vals.iter().all(|v| v == empty || v.is_empty()) {
        w.text(tag, "");
    } else {
        w.text(tag, &format!("\u{1}{}", vals.join(";")));
    }
}

/// `80324` → `Version8_3_24`; у старых конфигураций — малые коды: 2 → `Version8_2_13` (выгрузка платформой 8.5.1 тестовой
/// конфигурации отладчика). shortcut: прочие малые коды пишутся как есть (платформа их не загрузит) — свести по
/// перечислению платформы, когда встретятся.
fn compat(w: &mut W, tag: &str, n: &Node, i: usize) {
    let a = atom(n, &[i]);
    match a.parse::<u32>() {
        Ok(v) if v >= 80000 => w.text(tag, &version_name(v)),
        Ok(2) => w.text(tag, "Version8_2_13"),
        _ => w.text(tag, a),
    }
}

/// Режим совместимости конфигурации кодом записи (`80316`, у старых — малые коды); `None` — не конфигурация.
pub fn compat_mode(cfg: &Config) -> Option<u32> {
    let rid = cfg.entry_text("root")?.split(',').nth(1)?.trim().to_string();
    let root = crate::brace::Doc::parse(&cfg.entry_text(&rid)?).ok()?.roots.into_iter().next()?;
    atom(root.at(&[3, 1, 1])?, &[26]).parse().ok()
}

fn version_name(v: u32) -> String {
    format!("Version{}_{}_{}", v / 10000, v / 100 % 100, v % 100)
}

const ZREF: &str = ZERO;
const ELIST: &str = "{0,0}";
const ESTR: &str = "\"\"";

/// Свойства конфигурации; пути — от корень[3][1][1]. `*` в docs/research/xml-tools.md — позиция не проверена.
pub(crate) const CONFIGURATION: &[(&str, P)] = &[
    ("NamePrefix", P::Custom(|w, _, n| group(w, "NamePrefix", n, CONFIG_EMPTY_STRINGS, ESTR))),
    // в скобках БП 80324, а выгрузка 8.5.1 пишет Version8_5_1 (у УТ: 80327 и Version8_3_27 в выгрузке 8.3.27):
    // похоже, ниже своей версии платформа не пишет
    ("ConfigurationExtensionCompatibilityMode", P::Custom(|w, _, n| {
        let v: u32 = atom(n, &[43]).parse().unwrap_or(0);
        w.text("ConfigurationExtensionCompatibilityMode", &version_name(v.max(80501)));
    })),
    // коды не как в EDT (ClientRunMode 0 Auto): на каркасе Ordinary — 0, Auto — 2
    ("DefaultRunMode", P::Tok(&[21], &[("0", "OrdinaryApplication"), ("1", "ManagedApplication"), ("2", "Auto")])),
    ("UsePurposes", P::Custom(|w, _, n| use_purposes(w, n.get(33)))),
    ("ScriptVariant", P::Tok(&[3], &[("0", "English"), ("1", "Russian")])),
    ("DefaultRoles", P::Custom(|w, x, n| md_list(w, "DefaultRoles", "xr:Item", &[("xsi:type", "xr:MDObjectRef")], x, n.get(39)))),
    ("Vendor", P::Str(&[14])),
    ("Version", P::Str(&[15])),
    ("UpdateCatalogAddress", P::Str(&[16])),
    ("IncludeHelpInContents", P::Bool(&[13])),
    ("UseManagedFormInOrdinaryApplication", P::Bool(&[28])),
    ("UseOrdinaryFormInManagedApplication", P::Bool(&[29])),
    ("AdditionalFullTextSearchDictionaries", P::Custom(|w, _, n| group(w, "AdditionalFullTextSearchDictionaries", n, CONFIG_EMPTY_LISTS, ELIST))),
    ("CommonSettingsStorage", P::Ref(&[22])),
    ("ReportsUserSettingsStorage", P::Ref(&[23])),
    ("ReportsVariantsStorage", P::Ref(&[24])),
    ("FormDataSettingsStorage", P::Ref(&[25])),
    ("DynamicListsUserSettingsStorage", P::Custom(|w, _, n| group(w, "DynamicListsUserSettingsStorage", n, CONFIG_ZERO_REFS, ZREF))),
    ("URLExternalDataStorage", P::Custom(|w, _, n| group(w, "URLExternalDataStorage", n, CONFIG_ZERO_REFS, ZREF))),
    ("Content", P::Custom(|w, _, n| group(w, "Content", n, CONFIG_EMPTY_LISTS, ELIST))),
    ("DefaultReportForm", P::Ref(&[30])),
    ("DefaultReportVariantForm", P::Ref(&[31])),
    ("DefaultReportSettingsForm", P::Ref(&[32])),
    ("DefaultReportAppearanceTemplate", P::Custom(|w, _, n| group(w, "DefaultReportAppearanceTemplate", n, CONFIG_ZERO_REFS, ZREF))),
    ("DefaultDynamicListSettingsForm", P::Custom(|w, _, n| group(w, "DefaultDynamicListSettingsForm", n, CONFIG_ZERO_REFS, ZREF))),
    ("DefaultSearchForm", P::Custom(|w, _, n| group(w, "DefaultSearchForm", n, CONFIG_ZERO_REFS, ZREF))),
    ("DefaultDataHistoryChangeHistoryForm", P::Custom(|w, _, n| group(w, "DefaultDataHistoryChangeHistoryForm", n, CONFIG_ZERO_REFS, ZREF))),
    ("DefaultDataHistoryVersionDataForm", P::Custom(|w, _, n| group(w, "DefaultDataHistoryVersionDataForm", n, CONFIG_ZERO_REFS, ZREF))),
    ("DefaultDataHistoryVersionDifferencesForm", P::Custom(|w, _, n| group(w, "DefaultDataHistoryVersionDifferencesForm", n, CONFIG_ZERO_REFS, ZREF))),
    ("DefaultCollaborationSystemUsersChoiceForm", P::Custom(|w, _, n| group(w, "DefaultCollaborationSystemUsersChoiceForm", n, CONFIG_ZERO_REFS, ZREF))),
    // свойства 8.5: в выгрузке 8.3.27 их нет, в скобках 8.3.24 — тоже
    ("AuxiliaryReportForm", P::Const("")),
    ("AuxiliaryReportVariantForm", P::Const("")),
    ("AuxiliaryReportSettingsForm", P::Const("")),
    ("AuxiliaryDynamicListSettingsForm", P::Const("")),
    ("AuxiliaryDataHistoryChangeHistoryForm", P::Const("")),
    ("AuxiliaryDataHistoryVersionDataForm", P::Const("")),
    ("AuxiliaryDataHistoryVersionDifferencesForm", P::Const("")),
    ("AuxiliaryCollaborationSystemUsersChoiceForm", P::Const("")),
    // в обоих эталонах пусто, хотя списки (корень[3][1][1][40], [51]) не пусты — что выгружается, не установлено
    ("RequiredMobileApplicationPermissions", P::Const("")),
    ("UsedMobileApplicationFunctionalities", P::Custom(|w, _, n| mobile_functionalities(w, n.get(53)))),
    ("StandaloneConfigurationRestrictionRoles", P::Custom(|w, _, n| group(w, "StandaloneConfigurationRestrictionRoles", n, CONFIG_EMPTY_LISTS, ELIST))),
    ("MobileApplicationURLs", P::Custom(|w, _, n| group(w, "MobileApplicationURLs", n, CONFIG_EMPTY_LISTS, ELIST))),
    ("AllowedIncomingShareRequestTypes", P::Custom(|w, _, n| share_types(w, n.get(59)))),
    ("MainClientApplicationWindowInterfaceVariant", P::TokOr(&[66], &[("0", "NavigationLeft"), ("1", "NavigationTop")], "NavigationLeft")),
    // Поля 63, 66, 67 есть только в записи новой версии (77 полей, у БП — 60); без них — значения по умолчанию.
    ("ClientApplicationTheme", P::TokOr(&[63], &[("0", "Auto"), ("1", "Light"), ("2", "Dark")], "Auto")),
    ("MainClientApplicationWindowMode", P::Tok(&[45], &[("0", "Normal"), ("1", "Workplace"), ("2", "FullscreenWorkplace"), ("3", "Kiosk"), ("4", "EmbeddedWorkplace")])),
    ("ClientApplicationWindowsOpenVariant", P::TokOr(&[67], &[("0", "OpenDataInTabs"), ("1", "OpenDataInDialogs")], "OpenDataInDialogs")),
    ("DefaultInterface", P::Custom(|w, _, n| group(w, "DefaultInterface", n, CONFIG_ZERO_REFS, ZREF))),
    ("Caption", P::Const("")),
    ("ShortCaption", P::Const("")),
    ("DefaultStyle", P::Custom(|w, _, n| group(w, "DefaultStyle", n, CONFIG_ZERO_REFS, ZREF))),
    ("DefaultLanguage", P::Ref(&[10])),
    // [4] и [5] во всех образцах одинаковы
    ("BriefInformation", P::Ml(&[4])),
    ("DetailedInformation", P::Ml(&[5])),
    ("Copyright", P::Ml(&[6])),
    ("VendorInformationAddress", P::Ml(&[7])),
    ("ConfigurationInformationAddress", P::Ml(&[8])),
    ("DataLockControlMode", P::Tok(&[17], &[("0", "Automatic"), ("1", "Managed"), ("2", "AutomaticAndManaged")])),
    ("ObjectAutonumerationMode", P::Tok(&[19], &[("0", "AutoFree"), ("1", "NotAutoFree")])),
    ("ModalityUseMode", P::Tok(&[36], USE_MODE)),
    ("SynchronousPlatformExtensionAndAddInCallUseMode", P::Tok(&[41], USE_MODE)),
    // режимы 8.5 есть только в [62] (на каркасе Taxi → Version8_5EnableTaxi меняет лишь его); у записи старой версии — [38]
    ("InterfaceCompatibilityMode", P::Custom(|w, _, n| token(w, "InterfaceCompatibilityMode", n, if n.get(62).is_some() { &[62] } else { &[38] }, INTERFACE_MODE))),
    // на каркасе DontUse — 1, Use — 0 (в EDT наоборот); у записи старой версии позиции нет
    ("Version85InterfaceMigrationMode", P::TokOr(&[68], &[("0", "Use"), ("1", "DontUse")], "DontUse")),
    // shortcut: на каркасе Use не меняет запись конфигурации — где хранится, не найдено; [57] не проверен, перепроверить опытом
    ("DatabaseTablespacesUseMode", P::Tok(&[57], &[("0", "DontUse")])),
    ("CompatibilityMode", P::Custom(|w, _, n| compat(w, "CompatibilityMode", n, 26))),
    ("DefaultConstantsForm", P::Custom(|w, _, n| group(w, "DefaultConstantsForm", n, CONFIG_ZERO_REFS, ZREF))),
];

/// Коды перечислений — как в модели метаданных 1C:EDT (там они совпадают с записью .cf); позиции проверены опытом на каркасе.
const USE_MODE: &[(&str, &str)] = &[("0", "Use"), ("1", "UseWithWarnings"), ("2", "DontUse")];
const INTERFACE_MODE: &[(&str, &str)] = &[
    ("0", "Version8_2"),
    ("1", "Version8_2EnableTaxi"),
    ("2", "TaxiEnableVersion8_2"),
    ("3", "Taxi"),
    ("4", "TaxiEnableVersion8_5"),
    ("5", "Version8_5"),
    ("6", "Version8_5EnableTaxi"),
];

/// `{2,N,{<код>,<0|1>}...,0}` → `<app:functionality>` по всему списку платформы.
fn mobile_functionalities(w: &mut W, n: Option<&Node>) {
    let codes: Vec<(&str, &str)> = n
        .map(Node::list)
        .unwrap_or_default()
        .iter()
        .filter_map(|c| Some((c.get(0)?.atom()?, c.get(1)?.atom()?)))
        .collect();
    w.open("UsedMobileApplicationFunctionalities", &[]);
    for (i, name) in MOBILE_FUNCTIONALITIES.iter().enumerate() {
        let code = (if i < 28 { i } else { i + 4 }).to_string();
        let used = match codes.iter().find(|(c, _)| *c == code).map(|(_, u)| *u) {
            None | Some("0") => "false".to_string(),
            Some("1") => "true".to_string(),
            Some(u) => format!("\u{1}{u}"),
        };
        w.open("app:functionality", &[]);
        w.text("app:functionality", name);
        w.text("app:use", &used);
        w.end();
    }
    w.end();
}

/// `{N,{"#",f251d17e-…,{0,"mime","uti","ext",<вариант>,<свой>}}...}`.
fn share_types(w: &mut W, n: Option<&Node>) {
    let items = n.map(Node::list).unwrap_or_default().get(1..).unwrap_or_default();
    if items.is_empty() {
        w.text("AllowedIncomingShareRequestTypes", "");
        return;
    }
    w.open("AllowedIncomingShareRequestTypes", &[]);
    for it in items {
        let v = it.get(2);
        let s = |i: usize| v.and_then(|v| v.get(i)).and_then(Node::str).unwrap_or_default();
        w.open("v8:Value", &[("xsi:type", "app:AllowedIncomingShareRequestType")]);
        w.text("app:mime", s(1));
        w.text("app:uti", s(2));
        w.text("app:ext", s(3));
        w.text_attrs("app:processingVariant", &[("xsi:type", "xs:decimal")], v.map_or("", |v| atom(v, &[4])));
        w.text("app:isCustom", if v.map_or("", |v| atom(v, &[5])) == "1" { "true" } else { "false" });
        w.end();
    }
    w.end();
}

/// Первая тройка `{1,0,<uuid>}` в обходе — ObjectId части корня.
fn first_id(n: &Node) -> Option<&str> {
    let l = n.list();
    if let [Node::Atom(a), Node::Atom(b), Node::Atom(id)] = l {
        if a == "1" && b == "0" && is_guid(id) {
            return Some(id);
        }
    }
    l.iter().find_map(first_id)
}

/// Uuid элементов первой в обходе коллекции `{<guid>,N,<uuid>...}`.
fn collection_ids<'a>(n: &'a Node, guid: &str) -> Option<Vec<&'a str>> {
    let l = n.list();
    if l.len() >= 2 && l[0].atom() == Some(guid) && l[1].atom().and_then(|a| a.parse::<usize>().ok()) == Some(l.len() - 2) {
        return Some(l[2..].iter().filter_map(Node::atom).collect());
    }
    l.iter().find_map(|c| collection_ids(c, guid))
}

/// Configuration.xml по корневой записи.
pub fn configuration(cfg: &Config, x: &Index) -> Option<Vec<u8>> {
    let rid = cfg.entry_text("root")?.split(',').nth(1)?.trim().to_string();
    let root = crate::brace::Doc::parse(&cfg.entry_text(&rid)?).ok()?.roots.into_iter().next()?;
    let p = root.at(&[3, 1, 1])?;
    let mut w = W::metadata();
    w.open("Configuration", &[("uuid", &rid)]);
    w.open("InternalInfo", &[]);
    let parts: usize = atom(&root, &[2]).parse().unwrap_or(0);
    for k in 3..3 + parts {
        w.open("xr:ContainedObject", &[]);
        w.text("xr:ClassId", atom(&root, &[k, 0]));
        w.text("xr:ObjectId", root.at(&[k, 1]).and_then(first_id).unwrap_or("\u{1}"));
        w.end();
    }
    w.end();
    w.open("Properties", &[]);
    if let Some(h) = head(p) {
        name_synonym_comment(&mut w, h);
    }
    props(&mut w, x, p, CONFIGURATION);
    w.end();
    w.open("ChildObjects", &[]);
    let known = |c: &str| CONFIG_ORDER.contains(&c);
    // виды не из таблицы порядка — в конце (в БП таких нет)
    let kinds = CONFIG_ORDER.iter().copied().chain(cfg.groups.iter().map(|g| g.class).filter(|c| !known(c)));
    for kind in kinds {
        let Some(&(guid, ..)) = crate::meta::CLASSES.iter().find(|c| c.1 == kind) else { continue };
        for id in collection_ids(&root, guid).unwrap_or_default() {
            let full = x.name(id);
            w.text(kind, full.split_once('.').map_or(full.as_str(), |(_, nm)| nm));
        }
    }
    w.end();
    Some(w.finish())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn node(t: &str) -> Node {
        crate::brace::Doc::parse(t).unwrap().roots.remove(0)
    }

    /// Строки без заголовка и отступов, через `|`.
    fn xml(f: impl FnOnce(&mut W)) -> String {
        let mut w = W::metadata();
        f(&mut w);
        let s = String::from_utf8(w.finish()[3..].to_vec()).unwrap();
        let l: Vec<&str> = s.split("\r\n").collect();
        l[2..l.len() - 1].iter().map(|s| s.trim_start_matches('\t')).collect::<Vec<_>>().join("|")
    }

    fn index() -> Index {
        Index { full: HashMap::new(), types: HashMap::new() }
    }

    const A: &str = "11111111-1111-1111-1111-111111111111";
    const B: &str = "22222222-2222-2222-2222-222222222222";

    #[test]
    fn command_inside_enum_wrapper() {
        // команда перечисления: `{1,{2,…},{9,…}}` внутри `{0,0,0,…}`; изменяет данные, MakeDisable
        let t = "{1,{6d8d73a7-ba29-401d-9032-3872ec2d6433,1,{{0,{0,0,0,{1,{2,B,078a6af8-d22c-4248-9c33-7e90075a3d2c},{9,{4,0,{0},\"\",-1,-1,1,0,\"\"},3,{0},1,{0,0,0},0,{1,00000000-0000-0000-0000-000000000000},{\"Pattern\"},{3,{1,0,B},\"К\",{0},\"\",0,0,00000000-0000-0000-0000-000000000000,0},1,0,1}}}},0}}}";
        let root = node(&t.replace('B', B));
        let item = Item { kind: "Command", header: crate::meta::Header { id: B.into(), ..Default::default() }, files: vec![], children: vec![] };
        let s = xml(|w| command(w, &index(), &item, &root));
        assert!(s.contains("<Name>К</Name>") && s.contains("<ModifiesData>true</ModifiesData>") && s.contains("<Representation>Auto</Representation>"), "{s}");
        assert!(s.contains("<OnMainServerUnavalableBehavior>MakeDisable</OnMainServerUnavalableBehavior>"), "{s}");
    }

    #[test]
    fn element_by_head() {
        // владелец A с коллекцией из двух элементов: реквизит B и элемент без заголовка
        let t = "{1,{0,{2,{1,0,A},\"Вл\",{0},\"\"}},1,{5d24a9d1-098e-11d6-b9b8-0050bae0a95d,2,{{0,{27,{2,{2,{1,0,B},\"Р\",{0},\"к\"}}}},0},{{0,{1,0,A}}}}}";
        let n = node(&t.replace('A', A).replace('B', B));
        let el = element(&n, B).unwrap();
        assert_eq!(head(el).unwrap()[2].str(), Some("Р"));
        assert!(element(&n, "33333333-3333-3333-3333-333333333333").is_none());
    }

    #[test]
    fn xdto_types() {
        let s = |t: &str| xml(|w| xdto_type(w, "T", Some(&node(t))));
        assert_eq!(s("{0,\"http://www.w3.org/2001/XMLSchema\",\"string\"}"), "<T>xs:string</T>");
        assert_eq!(s("{0,\"http://v8.1c.ru/8.1/data/core\",\"Array\"}"), "<T>v8:Array</T>");
        // внутри MetaDataObject глубина 1 — d2p1 (у параметра web-сервиса 7 — d8p1)
        assert_eq!(s("{0,\"http://x/1\",\"Id\"}"), "<T xmlns:d2p1=\"http://x/1\">d2p1:Id</T>");
        assert_eq!(s("{0,\"\",\"\"}"), "<T/>");
    }

    #[test]
    fn mobile_codes_shift_after_27() {
        // код 27 — BarcodeScanning (номер 27), код 34 — Videoconferences (номер 30)
        let s = xml(|w| mobile_functionalities(w, Some(&node("{2,3,{0,1},{27,1},{34,1},0}"))));
        let used: Vec<&str> = s.split("<app:functionality>").filter(|p| p.contains("<app:use>true")).map(|p| p.split('<').next().unwrap()).collect();
        assert_eq!(used, ["Biometrics", "BarcodeScanning", "Videoconferences"]);
        assert_eq!(s.matches("<app:use>").count(), MOBILE_FUNCTIONALITIES.len());
    }

    #[test]
    fn form_layouts() {
        let h = format!("{{3,{{1,0,{A}}},\"Ф\",{{0}},\"\",0,0,00000000-0000-0000-0000-000000000000,0}}");
        let p = "{2,{\"#\",1708fdaa-cbce-4289-b373-07a5a74bee91,1}}";
        let x = index();
        let props = |t: &str| xml(|w| props(w, &x, &node(t), FORM));
        let ext = props(&format!("{{1,{{1,{{0,{{13,{h},1,1,{p}}},{{1,\"ru\",\"Р\"}}}}}},0}}"));
        assert!(ext.starts_with("<FormType>Managed</FormType>|<IncludeHelpInContents>true</IncludeHelpInContents>|<UsePurposes>"), "{ext}");
        assert!(ext.ends_with("<ExtendedPresentation>|<v8:item>|<v8:lang>ru</v8:lang>|<v8:content>Р</v8:content>|</v8:item>|</ExtendedPresentation>"), "{ext}");
        let plain = props(&format!("{{1,{{0,{{13,{h},0,0,{p}}}}},0}}"));
        assert!(plain.starts_with("<FormType>Ordinary</FormType>|<IncludeHelpInContents>false") && !plain.contains("Extended"), "{plain}");
        let bare = props(&format!("{{1,{{13,{h},0,1,{p}}},0}}"));
        assert!(bare.starts_with("<FormType>Managed</FormType>") && bare.contains("PlatformApplication</v8:Value>|</UsePurposes>"), "{bare}");
    }

    #[test]
    fn report_slots_and_versions() {
        let z = ZERO;
        let r = |a: &str| node(&format!("{{1,{{19,0,0,0,0,0,0,0,0,{z},0,0,0,0,{a},0,0,{z}}}}}"));
        assert_eq!(xml(|w| report_unknown(w, "AuxiliaryForm", &r(z))), "<AuxiliaryForm/>");
        assert!(xml(|w| report_unknown(w, "AuxiliaryForm", &r(A))).starts_with("<AuxiliaryForm>\u{1}1111"));
        assert_eq!(version_name(80324), "Version8_3_24");
        assert_eq!(xml(|w| compat(w, "C", &node("{0,2}"), 1)), "<C>Version8_2_13</C>");
    }

    #[test]
    fn integration_channels() {
        // канал версии 1 (передача, не в транзакции) и версии 0 — без последнего поля: «в транзакции» истина (FUN_1800e2eb0)
        let h = |id: &str, n: &str| format!("{{3,{{1,0,{id}}},\"{n}\",{{0}},\"\",0,0,{ZERO},0}}");
        let t = format!(
            "{{1,{{0,{},{A},{B},\"\"}},1,{{acb7e81f-0637-4ebd-88ff-954ba075ae51,2,{{{{1,{},{A},{B},\"out\",\"\",0,0}},0}},{{{{0,{},{A},{B},\"in\",\"П\",1}},0}}}}}}",
            h(ZERO, "С"),
            h(A, "К1"),
            h(B, "К2")
        );
        let ch = |id: &str, n: &str| Item { kind: "IntegrationServiceChannel", header: crate::meta::Header { id: id.into(), name: n.into(), ..Default::default() }, files: vec![], children: vec![] };
        let it = Item { kind: "IntegrationService", header: crate::meta::Header { name: "С".into(), ..Default::default() }, files: vec![], children: vec![ch(A, "К1"), ch(B, "К2")] };
        let s = xml(|w| channels(w, &index(), &node(&t), &it));
        assert!(s.contains("name=\"IntegrationServiceChannelManager.С.К1\" category=\"Manager\""), "{s}");
        assert!(s.contains("<ExternalIntegrationServiceChannelName>out</ExternalIntegrationServiceChannelName>|<MessageDirection>Send</MessageDirection>|<ReceiveMessageProcessing/>|<Transactioned>false</Transactioned>"), "{s}");
        assert!(s.contains("<MessageDirection>Receive</MessageDirection>|<ReceiveMessageProcessing>П</ReceiveMessageProcessing>|<Transactioned>true</Transactioned>"), "{s}");
    }

    #[test]
    fn contained_object_id() {
        let n = node(&format!("{{6,{{1,{{{{1,0,{A}}},00000000-0000-0000-0000-000000000000}}}},{B}}}"));
        assert_eq!(first_id(&n), Some(A));
        let c = node(&format!("{{x,{{0fe48980-252d-11d6-a3c7-0050bae0a776,2,{A},{B}}}}}"));
        assert_eq!(collection_ids(&c, "0fe48980-252d-11d6-a3c7-0050bae0a776"), Some(vec![A, B]));
    }

    /// Время жизни сеанса web-сервиса версии 3 — 20, версии 4 — из записи.
    #[test]
    fn session_max_age_by_version() {
        let ws = |v: &str| xml(|w| session_max_age(w, &node(&format!("{{1,{{{v},\"\",{{0}},{{0,0}},\"\",{{0}},1,30}}}}")), &[1, 7], 3));
        assert_eq!(ws("3"), "<SessionMaxAge>20</SessionMaxAge>");
        assert_eq!(ws("4"), "<SessionMaxAge>30</SessionMaxAge>");
    }
}
