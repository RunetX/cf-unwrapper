//! Права роли: запись `<id роли>.0` (роль файла Rights) → `Ext/Rights.xml`.
//! Устройство записи и правила выгрузки — docs/research/xml-rights.md.

use crate::brace::{self, Node};
use crate::meta::{Config, Item};
use std::collections::{HashMap, HashSet};

/// Коллекции стандартных реквизитов, стандартных табличных частей и субконто регистра бухгалтерии.
const STD_ATTR: &str = "03f171e8-326f-41c6-9fa5-932a0b12cddf";
const STD_TAB: &str = "28db313d-dbc2-4b83-8c4a-d2aeee708062";
const EXT_DIM: &str = "91162600-3161-4326-89a0-4a7cecd5092a";
const EXT_DIM_TYPE: &str = "b3b48b29-d652-47ab-9d21-7e06768c31b5";

/// GUID права → имя в XML (сопоставлено по выгрузке БП 3.0.171).
pub const RIGHTS: &[(&str, &str)] = &[
    ("fd05f656-7a23-43a4-8996-f480a806fb97", "ActiveUsers"),
    ("900e3c92-6e18-4874-846a-b28780b5b54c", "Administration"),
    ("f7c6a0bb-bca6-4cd3-9146-832971cd7073", "AnalyticsSystemClient"),
    ("07ef4641-f7da-417a-bd75-35c40a17c2f7", "Automation"),
    ("3762abec-3836-446a-83ce-3e05001bca8b", "CollaborationSystemInfoBaseRegistration"),
    ("399d7390-8d83-4a57-b4d7-c902c15b701f", "ConfigurationExtensionsAdministration"),
    ("10b8ce49-ae3d-4a2e-afe7-1e3648bd59f7", "DataAdministration"),
    ("c0028105-4cc1-41ca-aef1-bfbd8fc8f8c4", "Delete"),
    ("b7bab52d-c1b1-4bd8-8276-02db08d42352", "Edit"),
    ("8497054a-ffd1-4ca7-bdfe-340b9ddc050a", "EditDataHistoryVersionComment"),
    ("1c799cf9-342d-4bf7-9b6f-951a009228ce", "EventLog"),
    ("8fb221e3-0d4f-43f2-ad71-1984cad63375", "ExclusiveMode"),
    ("74fd69fa-368e-4292-956a-65eb2f9877bd", "Execute"),
    ("02119c69-f08a-4142-9426-3725d74b7719", "ExternalConnection"),
    ("499e8968-ca89-43f0-9955-8756058b1b53", "Get"),
    ("b5f861d3-d9c5-45ec-98bf-0ed4d489a351", "InputByString"),
    ("33200740-82b0-4de7-8556-d3fb25ca4328", "Insert"),
    ("3b869658-ebc9-49ff-9bb3-e7c59686f538", "InteractiveActivate"),
    ("b0c0cbfc-f2cc-4b80-8460-5d5d7a599d9d", "InteractiveChangeOfPosted"),
    ("798cf688-ad74-44fe-a464-236b49e910e0", "InteractiveClearDeletionMark"),
    ("e7f9daf9-eac2-4ada-9c26-c380858f3589", "InteractiveClearDeletionMarkPredefinedData"),
    ("b53db6ed-6e5b-4035-8d24-f10083d646ed", "InteractiveDelete"),
    ("fa6dbe86-856a-4ac4-b8ac-bce99f8b8b22", "InteractiveDeleteMarked"),
    ("65e5f92c-40ff-4130-9652-c0e7612d0609", "InteractiveDeleteMarkedPredefinedData"),
    ("013a262e-165f-4815-bdae-7a1bed6a68e4", "InteractiveDeletePredefinedData"),
    ("5e664189-f0ee-439c-bdc5-eb81cca41ddf", "InteractiveExecute"),
    ("fb88c756-91c9-4351-9cdf-e027879886c6", "InteractiveInsert"),
    ("7b8359dd-7d4e-4bcd-a61c-b4b26eae19c6", "InteractiveOpenExtDataProcessors"),
    ("eb29e198-c338-4a20-a253-be6fc3dd44d9", "InteractiveOpenExtReports"),
    ("5d167fcc-b11f-403a-9a37-1eda64c19df1", "InteractivePosting"),
    ("21b4742a-d335-4234-bf0f-a3074a0e31ac", "InteractivePostingRegular"),
    ("d76b72ba-5388-4b7f-af64-1b351f63a1e1", "InteractiveSetDeletionMark"),
    ("408c56c0-e210-4e2e-8e82-610050a08a39", "InteractiveSetDeletionMarkPredefinedData"),
    ("84487e82-eb6c-4c51-ae16-3a6db17e886d", "InteractiveStart"),
    ("4d0d77ec-8511-430d-bd77-8407f27bc8f4", "InteractiveUndoPosting"),
    ("b9b44b51-3ac9-47cd-8b5a-df51afdcceb0", "MainWindowModeEmbeddedWorkplace"),
    ("818fc6c3-4691-44e3-a80c-e8d424730ead", "MainWindowModeFullscreenWorkplace"),
    ("155a0b35-4343-4047-989b-d385373b063e", "MainWindowModeKiosk"),
    ("d066966a-ff6a-4a41-bd68-6191cab083bc", "MainWindowModeNormal"),
    ("f6168734-8b8d-4a88-ab39-ef6b51758e83", "MainWindowModeWorkplace"),
    ("1e50809b-73ed-4935-bb77-2616c4cabdf5", "MobileClient"),
    ("31c3d4f6-7d02-4654-a14e-06aacafcb4fa", "Output"),
    ("e060de25-bffd-42fd-bb09-f3a788d65760", "Posting"),
    ("1c87578f-9e09-4ec0-a991-5629c87b1588", "Read"),
    ("64319ca1-f3d8-472e-82ce-5da233e6daaa", "ReadDataHistory"),
    ("1b762bf9-df7f-4255-bbe6-f7578f41368d", "ReadDataHistoryOfMissingData"),
    ("d8682bbb-7800-4aa0-8590-d3cb11fe2a29", "SaveUserData"),
    ("1d306db2-d97e-4b57-9b28-5d21e838cd9e", "Set"),
    ("65b6855f-85d5-4d33-ab75-be4485326dd5", "Start"),
    ("479a42c0-c3e9-4ae7-bf4a-75cebc14fec4", "SwitchToDataHistoryVersion"),
    ("265eec41-3ce1-4a07-bc3b-253d44c9a4f4", "TechnicalSpecialistMode"),
    ("29da0973-3b85-40e5-89da-bce02dbab08e", "ThickClient"),
    ("3c00c6ee-844e-4620-85e4-671e72f114d9", "ThinClient"),
    ("24abfe06-289a-48c5-8bb4-032c733e45c5", "TotalsControl"),
    ("f55a8f7f-2c65-404f-b530-093d9006adba", "UndoPosting"),
    ("287b74b8-3a66-4a76-ba27-4f1f6a93770e", "Update"),
    ("4d87a22d-ca7f-40ba-a367-a4eae62f4a7f", "UpdateDataBaseConfiguration"),
    ("b162ff57-0296-483e-9af8-dc37576802cb", "UpdateDataHistory"),
    ("c4ab1331-e58d-4a46-ad2e-fe6d80b72aa4", "UpdateDataHistoryOfMissingData"),
    ("a679c969-8ea1-4b8b-9e61-8a414ba448f4", "UpdateDataHistorySettings"),
    ("5b3ea0e2-fdb9-41f6-bf6c-25747906b4cb", "UpdateDataHistoryVersionComment"),
    ("c6de80da-a4f7-4ce9-bbeb-0b00ea564ec1", "Use"),
    ("aa6448f2-be0f-42ea-ba26-1af7f52b5b65", "View"),
    ("9342b152-a7ae-4c79-9b7b-f4f028a36479", "ViewDataHistory"),
    ("bd33c881-192c-4ef7-a51d-b146e38c5078", "WebClient"),
];

/// Права конфигурации в порядке платформы. Права, добавленные в поздних версиях, в старых ролях не записаны;
/// для них отсутствие значит «истина» (CONFIG_TRUE). Права 4df6d046-… в выгрузке не встречалось, имя неизвестно.
const CONFIG_ORDER: &[&str] = &[
    "Administration", "DataAdministration", "UpdateDataBaseConfiguration", "ExclusiveMode", "ActiveUsers", "EventLog",
    "ThinClient", "WebClient", "MobileClient", "ThickClient", "ExternalConnection", "Automation", "TechnicalSpecialistMode",
    "CollaborationSystemInfoBaseRegistration", "MainWindowModeNormal", "MainWindowModeWorkplace", "MainWindowModeEmbeddedWorkplace",
    "MainWindowModeFullscreenWorkplace", "MainWindowModeKiosk", "AnalyticsSystemClient", "\u{1}4df6d046-3bf8-4dda-991c-53ba664296a5",
    "SaveUserData", "ConfigurationExtensionsAdministration", "InteractiveOpenExtDataProcessors", "InteractiveOpenExtReports", "Output",
];
pub const CONFIG_TRUE: &[&str] =
    &["MainWindowModeNormal", "MainWindowModeWorkplace", "MainWindowModeEmbeddedWorkplace", "MainWindowModeFullscreenWorkplace", "MainWindowModeKiosk", "AnalyticsSystemClient"];

/// Стандартная табличная часть плана видов расчета по номеру (как в описании, classes_refs).
fn calc_tab(n: i64) -> Option<&'static str> {
    Some(match n {
        -30 => "LeadingCalculationTypes",
        -20 => "DisplacingCalculationTypes",
        -10 => "BaseCalculationTypes",
        _ => return None,
    })
}

/// Номер стандартного реквизита → имя; номера свои у каждого класса.
fn std_attr(class: &str, n: i64) -> Option<&'static str> {
    std_table(class).iter().find(|(k, _)| *k == n).map(|(_, v)| *v)
}

/// Путь к стандартному реквизиту по имени (`StandardAttribute.Code`…) — обратное [`std_name`].
pub fn std_key(class: &str, rest: &str) -> Option<Vec<(i64, &'static str)>> {
    let mut cands: Vec<Vec<(i64, &'static str)>> = std_table(class).iter().map(|(n, _)| vec![(*n, STD_ATTR)]).collect();
    if class == "AccountingRegister" {
        for n in 0..32 {
            cands.push(vec![(n, STD_ATTR), (n, EXT_DIM)]);
            cands.push(vec![(n, STD_ATTR), (n, EXT_DIM_TYPE)]);
        }
    }
    if class == "ChartOfCalculationTypes" {
        for t in [-30, -20, -10] {
            cands.push(vec![(t, STD_TAB)]);
            cands.extend(std_table("CalculationTypes").iter().map(|(n, _)| vec![(t, STD_TAB), (*n, STD_ATTR)]));
        }
    }
    if class == "ChartOfAccounts" {
        cands.push(vec![(-12, STD_TAB)]);
        cands.extend(std_table("ExtDimensionTypes").iter().map(|(n, _)| vec![(-12, STD_TAB), (*n, STD_ATTR)]));
    }
    cands.into_iter().find(|p| std_name(class, p).as_deref() == Some(rest))
}

fn std_table(class: &str) -> &'static [(i64, &'static str)] {
    match class {
        "Catalog" => &[(-2, "Code"), (-3, "Description"), (-4, "Parent"), (-5, "Owner"), (-6, "IsFolder"), (-7, "DeletionMark"), (-8, "Ref"), (-10, "Predefined"), (-13, "PredefinedDataName")],
        "Document" => &[(-2, "Number"), (-3, "Date"), (-4, "DeletionMark"), (-5, "Ref"), (-7, "Posted")],
        "DocumentJournal" => &[(-2, "Number"), (-4, "DeletionMark"), (-7, "Posted"), (-100, "Date"), (-101, "Ref"), (-60003, "Type")],
        "ExchangePlan" => &[(-2, "Code"), (-3, "Description"), (-4, "DeletionMark"), (-6, "Ref"), (-9, "SentNo"), (-10, "ReceivedNo"), (-13, "ThisNode")],
        "ChartOfCharacteristicTypes" => &[(-2, "Ref"), (-4, "DeletionMark"), (-5, "Predefined"), (-6, "Parent"), (-7, "IsFolder"), (-8, "Code"), (-9, "Description"), (-11, "ValueType"), (-14, "PredefinedDataName")],
        "ChartOfAccounts" => &[(-2, "Ref"), (-4, "DeletionMark"), (-5, "Predefined"), (-6, "Parent"), (-7, "Code"), (-8, "Description"), (-10, "Type"), (-11, "OffBalance"), (-17, "Order"), (-28, "PredefinedDataName")],
        // Стандартная табличная часть ВидыСубконто плана счетов.
        "ExtDimensionTypes" => &[(-12, "LineNumber"), (-13, "ExtDimensionType"), (-14, "Predefined"), (-15, "TurnoversOnly")],
        "InformationRegister" => &[(-2, "Period"), (-3, "Recorder"), (-4, "LineNumber"), (-5, "Active")],
        "AccumulationRegister" => &[(-2, "Period"), (-3, "Recorder"), (-4, "LineNumber"), (-5, "Active"), (-9, "RecordType")],
        "AccountingRegister" => &[(-2, "Period"), (-3, "Recorder"), (-4, "LineNumber"), (-5, "Active"), (-10, "Account")],
        // Номера — как в описаниях (standard::NAMES); права на них — в БП 3.0.107.
        "Task" => &[(-2, "Number"), (-3, "Date"), (-4, "DeletionMark"), (-5, "Ref"), (-7, "BusinessProcess"), (-8, "RoutePoint"), (-9, "Description"), (-10, "Executed")],
        "BusinessProcess" => &[(-2, "Number"), (-3, "Date"), (-4, "DeletionMark"), (-5, "Ref"), (-7, "Completed"), (-8, "HeadTask"), (-9, "Started")],
        "ChartOfCalculationTypes" => &[(-2, "Code"), (-3, "Description"), (-4, "ActionPeriodIsBasic"), (-5, "DeletionMark"), (-6, "Ref"), (-8, "Predefined"), (-11, "PredefinedDataName")],
        // Стандартные табличные части плана видов расчета (ведущие, вытесняющие, базовые).
        "CalculationTypes" => &[(-100, "LineNumber"), (-101, "CalculationType"), (-102, "Predefined")],
        _ => &[],
    }
}

/// Путь к стандартному реквизиту `{n,<коллекция>},...` → `StandardAttribute.Code`, `StandardTabularSection.X.StandardAttribute.Y`.
fn std_name(class: &str, path: &[(i64, &str)]) -> Option<String> {
    match path {
        [(n, STD_ATTR)] => std_attr(class, *n).map(|a| format!("StandardAttribute.{a}")),
        // Субконто регистра бухгалтерии: номер субконто с нуля.
        [(n, STD_ATTR), (m, EXT_DIM)] if n == m && *n >= 0 && class == "AccountingRegister" => Some(format!("StandardAttribute.ExtDimension{}", n + 1)),
        [(n, STD_ATTR), (m, EXT_DIM_TYPE)] if n == m && *n >= 0 && class == "AccountingRegister" => Some(format!("StandardAttribute.ExtDimensionType{}", n + 1)),
        [(-12, STD_TAB)] if class == "ChartOfAccounts" => Some("StandardTabularSection.ExtDimensionTypes".into()),
        [(-12, STD_TAB), (n, STD_ATTR)] if class == "ChartOfAccounts" => {
            std_attr("ExtDimensionTypes", *n).map(|a| format!("StandardTabularSection.ExtDimensionTypes.StandardAttribute.{a}"))
        }
        [(t, STD_TAB)] if class == "ChartOfCalculationTypes" => calc_tab(*t).map(|t| format!("StandardTabularSection.{t}")),
        [(t, STD_TAB), (n, STD_ATTR)] if class == "ChartOfCalculationTypes" => {
            Some(format!("StandardTabularSection.{}.StandardAttribute.{}", calc_tab(*t)?, std_attr("CalculationTypes", *n)?))
        }
        _ => None,
    }
}

/// Объект метаданных по GUID: полное имя, вид элемента и класс владельца верхнего уровня.
#[derive(Clone, Debug)]
pub struct Named {
    pub full: String,
    pub kind: &'static str,
    pub class: &'static str,
}

/// Полные имена только для нужных GUID: обход всего дерева без выделения памяти на каждый узел.
fn names(cfg: &Config, want: &HashSet<&str>) -> HashMap<String, Named> {
    fn walk(it: &Item, class: &'static str, path: &mut String, want: &HashSet<&str>, out: &mut HashMap<String, Named>) {
        let len = path.len();
        if path.is_empty() {
            path.push_str(class);
        } else {
            path.push('.');
            path.push_str(it.kind);
        }
        path.push('.');
        path.push_str(&it.header.name);
        if want.contains(it.header.id.as_str()) {
            out.insert(it.header.id.clone(), Named { full: path.clone(), kind: it.kind, class });
        }
        for ch in &it.children {
            walk(ch, class, path, want, out);
        }
        path.truncate(len);
    }
    let mut out = HashMap::new();
    let mut path = String::new();
    walk(&cfg.root, "Configuration", &mut path, want, &mut out);
    // Роль ссылается на конфигурацию по имени записи корня (`root` или `configinfo`), а не по id из ее заголовка.
    if want.contains(cfg.root_id.as_str()) {
        out.insert(cfg.root_id.clone(), Named { full: format!("Configuration.{}", cfg.root.header.name), kind: "Configuration", class: "Configuration" });
    }
    for g in &cfg.groups {
        for it in &g.items {
            walk(it, g.class, &mut path, want, &mut out);
        }
    }
    out
}

/// Ext/Rights.xml по тексту записи прав роли.
/// `old` — формат 8.3.24 (2.17) и старше: явно заданные права интерфейса пишутся всегда, как в расширении.
pub fn xml(cfg: &Config, text: &str, old: bool) -> Result<Vec<u8>, String> {
    let r = brace::parse(text)?;
    let mut want = HashSet::new();
    guids(&r, &mut want);
    let idx = names(cfg, &want);
    let extension = cfg.index.contains_key("configinfo") && !cfg.index.contains_key("root");
    render(&r, extension || old, &|id| idx.get(id).cloned())
}

fn guids<'a>(n: &'a Node, out: &mut HashSet<&'a str>) {
    match n {
        Node::Atom(a) if brace::is_guid(a) => {
            out.insert(a);
        }
        Node::List(l) => l.iter().for_each(|c| guids(c, out)),
        _ => {}
    }
}

fn atom(n: Option<&Node>, what: &str) -> Result<String, String> {
    n.and_then(Node::atom).map(str::to_string).ok_or_else(|| format!("права: нет {what}"))
}

fn count(n: Option<&Node>, what: &str) -> Result<usize, String> {
    atom(n, what)?.parse().map_err(|_| format!("права: {what} — не число"))
}

/// Текст условия или шаблона: платформа пишет переводы строк как LF (в записи — CRLF).
fn esc(s: &str) -> String {
    s.replace("\r\n", "\n").replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

/// Флаг роли: 1 — да; 0 и 4294967295 (-1) — нет. Иное — с меткой (файл не выгружается).
fn flag(v: &str) -> String {
    match v {
        "1" => "true".into(),
        "0" | "4294967295" => "false".into(),
        v => format!("{}{v}", super::UNK),
    }
}

fn right_name(g: &str) -> String {
    RIGHTS.iter().find(|(k, _)| *k == g).map(|(_, v)| v.to_string()).unwrap_or_else(|| format!("\u{1}{g}"))
}

struct Object<'a> {
    key: (String, Vec<(i64, String)>),
    name: String,
    rights: Vec<(String, &'a str)>,
    rls: HashMap<String, &'a [Node]>,
}

/// Запись `{10,{N,{{1,<guid>,<глубина>,{n,<коллекция>}...,<поле>},{права}}...},{шаблоны},<флаги>}`.
/// `extension` — права роли расширения (.cfe): там явно заданные права интерфейса конфигурации пишутся всегда.
pub fn render(r: &Node, explicit_ui: bool, name_of: &dyn Fn(&str) -> Option<Named>) -> Result<Vec<u8>, String> {
    let set_new = atom(r.get(3), "setForNewObjects")?;
    let set_attr = atom(r.get(4), "setForAttributesByDefault")?;
    let indep = atom(r.get(5), "independentRightsOfChildObjects")?;
    let mut objects = Vec::new();
    let objs = r.get(1).ok_or("права: нет списка объектов")?;
    for o in objs.list().iter().skip(1) {
        let (k, b) = (o.get(0).ok_or("права: нет ключа")?, o.get(1).ok_or("права: нет прав объекта")?);
        let base = atom(k.get(1), "GUID объекта")?;
        let depth = count(k.get(2), "глубина")?;
        let path: Vec<(i64, String)> = (0..depth)
            .map(|i| {
                let p = k.get(3 + i);
                Ok((atom(p.and_then(|p| p.get(0)), "номер")?.parse::<i64>().map_err(|_| "права: номер — не число")?, atom(p.and_then(|p| p.get(1)), "коллекция")?))
            })
            .collect::<Result<_, String>>()?;
        let field = atom(k.get(3 + depth), "признак поля")? == "1";
        // Объекты, удаленные из конфигурации, платформа не выгружает.
        let Some(owner) = name_of(&base) else { continue };
        let name = if depth == 0 {
            owner.full.clone()
        } else {
            // Стандартные реквизиты табличных частей платформа тоже не выгружает.
            if owner.kind == "TabularSection" {
                continue;
            }
            let p: Vec<(i64, &str)> = path.iter().map(|(n, g)| (*n, g.as_str())).collect();
            let s = if owner.kind == owner.class { std_name(owner.class, &p) } else { None };
            format!("{}.{}", owner.full, s.unwrap_or_else(|| format!("\u{1}{p:?}")))
        };
        let b = b.list();
        let (pairs, rls_list) = if b.first().and_then(Node::atom) == Some("0") {
            (&b[1..], &[][..])
        } else {
            let n = count(b.get(1), "число прав")?;
            let end = (2 + 2 * n).min(b.len());
            (&b[2..end], b.get(end + 1..).unwrap_or(&[]))
        };
        let mut rights: Vec<(String, &str)> = pairs.chunks(2).map(|c| Ok((right_name(&atom(c.first(), "право")?), c.get(1).and_then(Node::atom).unwrap_or("\u{1}")))).collect::<Result<_, String>>()?;
        let mut rls = HashMap::new();
        for x in rls_list {
            rls.insert(right_name(&atom(x.get(0), "право ограничения")?), x.get(1).map(Node::list).unwrap_or(&[]));
        }
        let config = depth == 0 && owner.kind == "Configuration";
        // В расширении права интерфейса конфигурации (CONFIG_TRUE), заданные явно, платформа пишет всегда, даже если
        // это умолчание роли; в основной конфигурации — нет (kontur_billing.cfe и БП против ibcmd config export, 24.10.2026).
        // shortcut: правило выведено по одному расширению; проверить на других .cfe, когда появятся их эталоны.
        // 8.3.24 так пишет их и в основной конфигурации (БП 3.0.107 ПолныеПрава; 8.3.27 и 8.5 — нет).
        let always: Vec<String> = if config && explicit_ui { rights.iter().filter(|(r, v)| *v == "1" && CONFIG_TRUE.contains(&r.as_str())).map(|(r, _)| r.clone()).collect() } else { Vec::new() };
        if config {
            let mut ordered: Vec<(String, &str)> = CONFIG_ORDER
                .iter()
                // «0» (не задано) — как отсутствие права: у прав CONFIG_TRUE это «истина».
                .filter_map(|n| rights.iter().find(|(r, v)| r == n && *v != "0").map(|(_, v)| *v).or(CONFIG_TRUE.contains(n).then_some("1")).map(|v| (n.to_string(), v)))
                .collect();
            ordered.extend(rights.into_iter().filter(|(r, _)| !CONFIG_ORDER.contains(&r.as_str())));
            rights = ordered;
        }
        // Пишутся значения, отличные от умолчания (у полей — setForAttributesByDefault, у объектов — setForNewObjects),
        // и права с ограничением.
        // «0» — право не задано (бывает в расширениях): платформа его не пишет при любом умолчании
        // (проверено на kontur_billing.cfe против ibcmd config export, 24.10.2026).
        let default = if field { set_attr == "1" } else { set_new == "1" };
        rights.retain(|(r, v)| *v != "0" && ((*v == "1") != default || always.contains(r) || rls.get(r).is_some_and(|l| l.first().and_then(Node::atom) != Some("0"))));
        if rights.is_empty() {
            continue;
        }
        objects.push(Object { key: (base, path), name, rights, rls });
    }
    objects.sort_by(|a, b| a.key.cmp(&b.key));

    let mut x = String::from("\u{feff}<?xml version=\"1.0\" encoding=\"UTF-8\"?>\r\n<Rights xmlns=\"http://v8.1c.ru/8.2/roles\" xmlns:xs=\"http://www.w3.org/2001/XMLSchema\" xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\" xsi:type=\"Rights\" version=\"2.21\">\r\n");
    x.push_str(&format!("\t<setForNewObjects>{}</setForNewObjects>\r\n", flag(&set_new)));
    x.push_str(&format!("\t<setForAttributesByDefault>{}</setForAttributesByDefault>\r\n", flag(&set_attr)));
    x.push_str(&format!("\t<independentRightsOfChildObjects>{}</independentRightsOfChildObjects>\r\n", flag(&indep)));
    for o in &objects {
        x.push_str(&format!("\t<object>\r\n\t\t<name>{}</name>\r\n", o.name));
        for (r, v) in &o.rights {
            let v = match *v {
                "1" => "true".into(),
                "-1" => "false".into(),
                v => format!("{}{v}", super::UNK),
            };
            x.push_str(&format!("\t\t<right>\r\n\t\t\t<name>{r}</name>\r\n\t\t\t<value>{v}</value>\r\n"));
            for c in o.rls.get(r).map(|l| l.get(1..).unwrap_or(&[])).unwrap_or(&[]) {
                restriction(c, name_of, &mut x)?;
            }
            x.push_str("\t\t</right>\r\n");
        }
        x.push_str("\t</object>\r\n");
    }
    for t in r.get(2).map(Node::list).unwrap_or(&[]).iter().skip(1) {
        let (n, c) = (t.get(0).and_then(Node::str).ok_or("права: нет имени шаблона")?, t.get(1).and_then(Node::str).ok_or("права: нет текста шаблона")?);
        x.push_str(&format!("\t<restrictionTemplate>\r\n\t\t<name>{}</name>\r\n\t\t<condition>{}</condition>\r\n\t</restrictionTemplate>\r\n", esc(n), esc(c)));
    }
    x.push_str("</Rights>");
    Ok(x.into_bytes())
}

/// Ограничение `{1,"условие",<число полей>,{{0},{0,<guid реквизита>}}...}`; пустое (`{1,"",0}`) не пишется.
fn restriction(c: &Node, name_of: &dyn Fn(&str) -> Option<Named>, x: &mut String) -> Result<(), String> {
    let cond = c.get(1).and_then(Node::str).ok_or("права: нет условия ограничения")?;
    let n = count(c.get(2), "число полей")?;
    if n == 0 && cond.is_empty() {
        return Ok(());
    }
    x.push_str("\t\t\t<restrictionByCondition>\r\n");
    for f in c.list().iter().skip(3).take(n) {
        // Поле — реквизит объекта; пишется его имя. Другой вид ссылки оставлен видимым.
        let id = match (f.get(0).map(Node::list), f.get(1).map(Node::list)) {
            (Some([Node::Atom(z)]), Some([Node::Atom(z2), Node::Atom(id)])) if z == "0" && z2 == "0" => Some(id.as_str()),
            _ => None,
        };
        let name = id.and_then(name_of).map(|n| n.full.rsplit('.').next().unwrap_or_default().to_string()).unwrap_or_else(|| format!("\u{1}{f:?}"));
        x.push_str(&format!("\t\t\t\t<field>{name}</field>\r\n"));
    }
    x.push_str(&format!("\t\t\t\t<condition>{}</condition>\r\n\t\t\t</restrictionByCondition>\r\n", esc(cond)));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const CAT: &str = "11111111-1111-1111-1111-111111111111";
    const ATTR: &str = "22222222-2222-2222-2222-222222222222";
    const CONF: &str = "33333333-3333-3333-3333-333333333333";

    fn name_of(id: &str) -> Option<Named> {
        let (full, kind, class) = match id {
            CAT => ("Catalog.Валюты", "Catalog", "Catalog"),
            ATTR => ("Catalog.Валюты.Attribute.Код", "Attribute", "Catalog"),
            CONF => ("Configuration.Мини", "Configuration", "Configuration"),
            _ => return None,
        };
        Some(Named { full: full.into(), kind, class })
    }

    fn out(text: &str) -> String {
        String::from_utf8(render(&brace::parse(text).unwrap(), false, &name_of).unwrap()).unwrap()
    }

    #[test]
    fn rights_rls_templates() {
        let read = "1c87578f-9e09-4ec0-a991-5629c87b1588";
        let view = "aa6448f2-be0f-42ea-ba26-1af7f52b5b65";
        let edit = "b7bab52d-c1b1-4bd8-8276-02db08d42352";
        let update = "287b74b8-3a66-4a76-ba27-4f1f6a93770e";
        let gone = "99999999-9999-9999-9999-999999999999";
        let t = format!(
            "{{10,{{5,\
             {{{{1,{ATTR},0,1}},{{0,{view},1,{edit},-1}}}},\
             {{{{1,{CAT},1,{{-2,{STD_ATTR}}},1}},{{0,{view},-1,{edit},-1}}}},\
             {{{{1,{CAT},0,0}},{{1,3,{read},1,{update},-1,{view},-1,2,{{{read},{{2,{{1,\"\",0}},{{1,\"ГДЕ А < 1 &\r\nБ\",1,{{{{0}},{{0,{ATTR}}}}}}}}}}},{{{update},{{0}}}}}}}},\
             {{{{1,{gone},0,0}},{{0,{read},1}}}},\
             {{{{1,{CONF},0,0}},{{0,{read},-1}}}}}},\
             {{1,{{\"Шаблон(П)\",\"// а\r\nб\"}}}},4294967295,1,0,4294967295}}"
        );
        let x = out(&t);
        let body = x.split_once("</independentRightsOfChildObjects>\r\n").unwrap().1;
        assert!(x.starts_with("\u{feff}<?xml version=\"1.0\" encoding=\"UTF-8\"?>\r\n<Rights "));
        assert!(x.contains("\t<setForNewObjects>false</setForNewObjects>\r\n\t<setForAttributesByDefault>true</setForAttributesByDefault>\r\n"));
        // Порядок — по GUID и пути; по умолчанию (View=1 у реквизита, -1 у объекта) не пишется; Read с ограничением пишется.
        assert_eq!(
            body,
            "\t<object>\r\n\t\t<name>Catalog.Валюты</name>\r\n\
             \t\t<right>\r\n\t\t\t<name>Read</name>\r\n\t\t\t<value>true</value>\r\n\
             \t\t\t<restrictionByCondition>\r\n\t\t\t\t<field>Код</field>\r\n\t\t\t\t<condition>ГДЕ А &lt; 1 &amp;\nБ</condition>\r\n\t\t\t</restrictionByCondition>\r\n\
             \t\t</right>\r\n\t</object>\r\n\
             \t<object>\r\n\t\t<name>Catalog.Валюты.StandardAttribute.Code</name>\r\n\
             \t\t<right>\r\n\t\t\t<name>View</name>\r\n\t\t\t<value>false</value>\r\n\t\t</right>\r\n\
             \t\t<right>\r\n\t\t\t<name>Edit</name>\r\n\t\t\t<value>false</value>\r\n\t\t</right>\r\n\t</object>\r\n\
             \t<object>\r\n\t\t<name>Catalog.Валюты.Attribute.Код</name>\r\n\
             \t\t<right>\r\n\t\t\t<name>Edit</name>\r\n\t\t\t<value>false</value>\r\n\t\t</right>\r\n\t</object>\r\n\
             \t<object>\r\n\t\t<name>Configuration.Мини</name>\r\n\
             \t\t<right>\r\n\t\t\t<name>MainWindowModeNormal</name>\r\n\t\t\t<value>true</value>\r\n\t\t</right>\r\n\
             \t\t<right>\r\n\t\t\t<name>MainWindowModeWorkplace</name>\r\n\t\t\t<value>true</value>\r\n\t\t</right>\r\n\
             \t\t<right>\r\n\t\t\t<name>MainWindowModeEmbeddedWorkplace</name>\r\n\t\t\t<value>true</value>\r\n\t\t</right>\r\n\
             \t\t<right>\r\n\t\t\t<name>MainWindowModeFullscreenWorkplace</name>\r\n\t\t\t<value>true</value>\r\n\t\t</right>\r\n\
             \t\t<right>\r\n\t\t\t<name>MainWindowModeKiosk</name>\r\n\t\t\t<value>true</value>\r\n\t\t</right>\r\n\
             \t\t<right>\r\n\t\t\t<name>AnalyticsSystemClient</name>\r\n\t\t\t<value>true</value>\r\n\t\t</right>\r\n\t</object>\r\n\
             \t<restrictionTemplate>\r\n\t\t<name>Шаблон(П)</name>\r\n\t\t<condition>// а\nб</condition>\r\n\t</restrictionTemplate>\r\n</Rights>"
        );
    }

    #[test]
    fn std_names() {
        assert_eq!(std_name("Catalog", &[(-13, STD_ATTR)]).as_deref(), Some("StandardAttribute.PredefinedDataName"));
        assert_eq!(std_name("AccountingRegister", &[(1, STD_ATTR), (1, EXT_DIM_TYPE)]).as_deref(), Some("StandardAttribute.ExtDimensionType2"));
        assert_eq!(std_name("ChartOfAccounts", &[(-12, STD_TAB), (-15, STD_ATTR)]).as_deref(), Some("StandardTabularSection.ExtDimensionTypes.StandardAttribute.TurnoversOnly"));
        assert_eq!(std_name("Task", &[(-10, STD_ATTR)]).as_deref(), Some("StandardAttribute.Executed"));
        assert_eq!(std_name("ChartOfCalculationTypes", &[(-20, STD_TAB), (-101, STD_ATTR)]).as_deref(), Some("StandardTabularSection.DisplacingCalculationTypes.StandardAttribute.CalculationType"));
        assert_eq!(std_key("ChartOfCalculationTypes", "StandardTabularSection.BaseCalculationTypes"), Some(vec![(-10, STD_TAB)]));
    }
}
