//! Стандартные реквизиты `<StandardAttributes>`: в скобках `{1,N,{-код},510405d3-…,{13,24,<guid свойства>,<значение>,...},...}`.
//! Код реквизита → имя зависит от класса; свойства — словарь по GUID, в выгрузке — в своем постоянном порядке.

use super::common::*;
use super::w::W;
use crate::brace::{self, Node};

/// Код стандартного реквизита → имя, по классам (сверка с выгрузкой БП 3.0.171).
/// У табличных частей LineNumber — -10, у табличных частей плана видов расчета — -100, у обработок и отчетов — -3.
pub const NAMES: &[(&str, &[(&str, &str)])] = &[
    ("Enum", &[("-3", "Order"), ("-2", "Ref")]),
    ("Catalog", &[("-13", "PredefinedDataName"), ("-10", "Predefined"), ("-8", "Ref"), ("-7", "DeletionMark"), ("-6", "IsFolder"), ("-5", "Owner"), ("-4", "Parent"), ("-3", "Description"), ("-2", "Code")]),
    ("Document", &[("-7", "Posted"), ("-5", "Ref"), ("-4", "DeletionMark"), ("-3", "Date"), ("-2", "Number")]),
    ("InformationRegister", &[("-5", "Active"), ("-4", "LineNumber"), ("-3", "Recorder"), ("-2", "Period")]),
    ("TabularSection", &[("-100", "LineNumber"), ("-10", "LineNumber"), ("-3", "LineNumber")]),
    ("AccumulationRegister", &[("-9", "RecordType"), ("-5", "Active"), ("-4", "LineNumber"), ("-3", "Recorder"), ("-2", "Period")]),
    ("CalculationRegister", &[("-13", "RegistrationPeriod"), ("-11", "ReversingEntry"), ("-10", "Active"), ("-9", "EndOfBasePeriod"), ("-8", "BegOfBasePeriod"), ("-7", "EndOfActionPeriod"), ("-6", "BegOfActionPeriod"), ("-5", "ActionPeriod"), ("-4", "CalculationType"), ("-3", "LineNumber"), ("-2", "Recorder")]),
    ("AccountingRegister", &[("-30", "PeriodAdjustment"), ("-10", "Account"), ("-9", "RecordType"), ("-5", "Active"), ("-4", "LineNumber"), ("-3", "Recorder"), ("-2", "Period")]),
    ("DocumentJournal", &[("-60003", "Type"), ("-101", "Ref"), ("-100", "Date"), ("-7", "Posted"), ("-4", "DeletionMark"), ("-2", "Number")]),
    ("ChartOfCharacteristicTypes", &[("-14", "PredefinedDataName"), ("-11", "ValueType"), ("-9", "Description"), ("-8", "Code"), ("-7", "IsFolder"), ("-6", "Parent"), ("-5", "Predefined"), ("-4", "DeletionMark"), ("-2", "Ref")]),
    ("ChartOfAccounts", &[("-28", "PredefinedDataName"), ("-17", "Order"), ("-11", "OffBalance"), ("-10", "Type"), ("-8", "Description"), ("-7", "Code"), ("-6", "Parent"), ("-5", "Predefined"), ("-4", "DeletionMark"), ("-2", "Ref")]),
    ("ChartOfCalculationTypes", &[("-11", "PredefinedDataName"), ("-8", "Predefined"), ("-6", "Ref"), ("-5", "DeletionMark"), ("-4", "ActionPeriodIsBasic"), ("-3", "Description"), ("-2", "Code")]),
    ("ExchangePlan", &[("-14", "ExchangeDate"), ("-13", "ThisNode"), ("-10", "ReceivedNo"), ("-9", "SentNo"), ("-6", "Ref"), ("-4", "DeletionMark"), ("-3", "Description"), ("-2", "Code")]),
    ("BusinessProcess", &[("-9", "Started"), ("-8", "HeadTask"), ("-7", "Completed"), ("-5", "Ref"), ("-4", "DeletionMark"), ("-3", "Date"), ("-2", "Number")]),
    ("Task", &[("-10", "Executed"), ("-9", "Description"), ("-8", "RoutePoint"), ("-7", "BusinessProcess"), ("-5", "Ref"), ("-4", "DeletionMark"), ("-3", "Date"), ("-2", "Number")]),
    // стандартные табличные части
    ("ChartOfAccounts.StandardTabularSection", &[("-15", "TurnoversOnly"), ("-14", "Predefined"), ("-13", "ExtDimensionType"), ("-12", "LineNumber")]),
    ("ChartOfCalculationTypes.StandardTabularSection", &[("-102", "Predefined"), ("-101", "CalculationType"), ("-100", "LineNumber")]),
];

const EXT_DIMENSION: &str = "91162600-3161-4326-89a0-4a7cecd5092a";
const EXT_DIMENSION_TYPE: &str = "b3b48b29-d652-47ab-9d21-7e06768c31b5";
const SYNONYM: &str = "cf4abea3-37b2-11d4-940f-008048da11f9";
const COMMENT: &str = "cf4abea4-37b2-11d4-940f-008048da11f9";
const TOOLTIP: &str = "4690ff70-e3fa-4914-9127-6a9acc5fc949";
const FORMAT: &str = "580c29e2-8af4-4258-882a-7cf8073e61c8";
const EDIT_FORMAT: &str = "88149a78-9448-4767-867b-0e650d165d2e";
const MASK: &str = "f49e4ced-4033-4e6c-8755-9fbaaccd6078";
const FILL_CHECKING: &str = "2723eb98-b4c1-498a-a6f3-70444757902f";
const FILL_FROM_FILLING_VALUE: &str = "2c8143d5-4248-4c43-8bfb-307c0be2e415";
const DATA_HISTORY: &str = "9288a8ed-b259-46d0-a8e3-70d87956ff2d";
const FULL_TEXT_SEARCH: &str = "d4232326-022b-421e-b6d3-88e418f74327";
const FILL_VALUE: &str = "e6b3f5f3-bdf3-4ad0-bc60-7323b3feb208";
const CHOICE_PARAMETER_LINKS: &str = "e3da683b-c54a-457a-a243-b9b4f9bf76dd";
const CHOICE_PARAMETERS: &str = "fcf503b8-1c06-454a-970c-06413e64aee5";
const LINK_BY_TYPE: &str = "1183c14f-f814-49c6-9233-a3c26b3f64cf";

/// Свойства, которые в эталоне не меняются ни разу: какой GUID какому свойству отвечает, не установлено.
/// Пишется значение эталона, если в скобках значение по умолчанию; иначе — скобки как есть (сверка покажет).
const UNRESOLVED: &[(&str, &str, &str)] = &[
    // MultiLine, ExtendedEdit, PasswordMode, MarkNegatives — четыре {"B",0}
    ("2bbba66b-fabf-4863-8ba3-54b3c64c896e", "{\"B\",0}", "MultiLine"),
    ("4de03908-56f4-4396-a61e-17253afca9ac", "{\"B\",0}", "ExtendedEdit"),
    ("90ae4b5d-e0fd-49ef-a008-d67c1e75038c", "{\"B\",0}", "PasswordMode"),
    ("b02800e9-a8d1-42ab-9a12-f673e92be968", "{\"B\",0}", "MarkNegatives"),
    // MinValue, MaxValue — два {"U"}
    ("3eaf5a8b-06d6-47b0-ac7d-a9698247f499", "{\"U\"}", "MinValue"),
    ("c65a541f-0b91-4f33-bc88-fbaaa57f9992", "{\"U\"}", "MaxValue"),
    ("6c4f7074-e7d4-48eb-b31b-132873666262", "{\"#\",157fa490-4ce9-11d4-9415-008048da11f9,\r\n{1,00000000-0000-0000-0000-000000000000}\r\n}", "ChoiceForm"),
    // CreateOnInput, QuickChoice, ChoiceHistoryOnInput — все Auto
    ("33c74a4d-561f-4bc0-9eaa-8d21c893c0a9", "{\"#\",ad3615c5-aae6-4725-89be-91827523abd9,\r\n{ad3615c5-aae6-4725-89be-91827523abd9,0}\r\n}", "CreateOnInput"),
    ("6e3a1131-37a3-4da5-8895-572d9d0c9db6", "{\"#\",ace3fd07-11b2-477e-ab7f-36f0ea37c8dd,\r\n{ace3fd07-11b2-477e-ab7f-36f0ea37c8dd,2}\r\n}", "QuickChoice"),
    ("7ba608f2-e654-42a3-8885-334fe88ca910", "{\"#\",12ca4003-ac70-450e-b897-37faf86bd313,0}", "ChoiceHistoryOnInput"),
];

/// `<StandardAttributes>` из узла `{1,N,...}`; `class` выбирает имена кодов, `owner` (`Catalog.Имя`) — для путей связей.
pub fn standard_attributes(w: &mut W, x: &Index, class: &str, owner: &str, n: &Node) {
    standard_attributes_tag(w, x, class, owner, n, "StandardAttributes");
}

/// То же с другим внешним тегом (`xr:StandardAttributes` у стандартных табличных частей).
pub fn standard_attributes_tag(w: &mut W, x: &Index, class: &str, owner: &str, n: &Node, tag: &str) {
    let names = NAMES.iter().find(|(c, _)| *c == class).map(|(_, t)| *t).unwrap_or(&[]);
    let l = n.list();
    let count: usize = l.get(1).and_then(Node::atom).and_then(|a| a.parse().ok()).unwrap_or(0);
    // Субконто регистра бухгалтерии: код `{<N-1>,<guid>}`, guid — ExtDimension или ExtDimensionType.
    let ext_dim = |c: &Node| -> Option<String> {
        let kind = match c.get(1)?.atom()? {
            EXT_DIMENSION => "ExtDimension",
            EXT_DIMENSION_TYPE => "ExtDimensionType",
            _ => return None,
        };
        Some(format!("{kind}{}", c.get(0)?.atom()?.parse::<usize>().ok()? + 1))
    };
    let mut items: Vec<(std::borrow::Cow<str>, &[Node])> = (0..count)
        .map(|k| {
            let c = l.get(2 + 3 * k);
            let code = c.and_then(|c| (class == "AccountingRegister").then(|| ext_dim(c)).flatten()).map(Into::into).unwrap_or_else(|| c.and_then(|c| c.get(0)).and_then(Node::atom).unwrap_or_default().into());
            (code, l.get(4 + 3 * k).map(Node::list).unwrap_or_default())
        })
        .collect();
    // в старых записях плана обмена нет ThisNode (-13): платформа выгружает его первым со свойствами по умолчанию
    // (у справочника до версии 53 так же PredefinedDataName — classes_refs::upgrade_old_catalog)
    if class == "ExchangePlan" && !items.iter().any(|(c, _)| *c == "-13") {
        items.insert(0, ("-13".into(), &[]));
    }
    w.open(tag, &[]);
    for (code, props) in items {
        let unknown = if code.starts_with("ExtDimension") { code.to_string() } else { format!("\u{1}{code}") };
        let name = names.iter().find(|(c, _)| *c == code).map_or(&*unknown, |(_, nm)| nm);
        let get = |g: &str| props.get(2..).unwrap_or_default().chunks(2).find(|c| c[0].atom() == Some(g)).and_then(|c| c.get(1));
        w.open("xr:StandardAttribute", &[("name", name)]);
        // {"#",<тип>,<число>} или {"#",<тип>,{<тип>,<число>}} → число
        let num = |g: &str| {
            let v = get(g)?.get(2)?;
            v.atom().or_else(|| v.get(1)?.atom())
        };
        let fixed = |w: &mut W, tag: &str, xml: &str| {
            let (g, def, _) = UNRESOLVED.iter().find(|(_, _, t)| *t == tag).unwrap();
            match get(g) {
                Some(v) if brace::write(v).ok().as_deref() != Some(def) => w.text(&format!("xr:{tag}"), &raw(Some(v))),
                _ if xml == "nil" => w.nil(&format!("xr:{tag}")),
                _ => w.text(&format!("xr:{tag}"), xml),
            }
        };
        let empty_or_raw = |w: &mut W, tag: &str, g: &str, empty: &str| match get(g) {
            Some(v) if brace::write(v).ok().as_deref() != Some(empty) => w.text(tag, &raw(Some(v))),
            _ => w.text(tag, ""),
        };
        // Связь по типу со стандартным реквизитом `{3,1,{<код>},<элемент связи>}` (субконто регистра бухгалтерии → Account).
        let link = get(LINK_BY_TYPE).and_then(|v| v.get(2)).map(Node::list).and_then(|l| match l {
            [a, b, c, d] if a.atom() == Some("3") && b.atom() == Some("1") => Some((names.iter().find(|(k, _)| Some(*k) == c.get(0).and_then(Node::atom))?.1, d.atom()?)),
            _ => None,
        });
        match link {
            Some((target, item)) => {
                w.open("xr:LinkByType", &[]);
                w.text("xr:DataPath", &format!("{owner}.StandardAttribute.{target}"));
                w.text("xr:LinkItem", item);
                w.end();
            }
            None => empty_or_raw(w, "xr:LinkByType", LINK_BY_TYPE, "{\"#\",9ad557b1-249e-48dc-824b-3e149ecf10a6,\r\n{3,0,0}\r\n}"),
        }
        w.text("xr:FillChecking", if num(FILL_CHECKING) == Some("1") { "ShowError" } else { "DontCheck" });
        fixed(w, "MultiLine", "false");
        w.text("xr:FillFromFillingValue", if get(FILL_FROM_FILLING_VALUE).and_then(|v| v.get(1)).and_then(Node::atom) == Some("1") { "true" } else { "false" });
        fixed(w, "CreateOnInput", "Auto");
        // не хранится: у владельца справочника — Deny
        w.text("xr:TypeReductionMode", if class == "Catalog" && name == "Owner" { "Deny" } else { "TransformValues" });
        fixed(w, "MaxValue", "nil");
        mlstring(w, "xr:ToolTip", get(TOOLTIP).and_then(|v| v.get(2)));
        fixed(w, "ExtendedEdit", "false");
        mlstring(w, "xr:Format", get(FORMAT).and_then(|v| v.get(2)));
        // форма выбора `{"#",157fa490-…,{1,<uuid формы>}}` (ERP 2.5.22 ГруппыСотрудников.Parent); пустая — по умолчанию
        let choice_form = UNRESOLVED.iter().find(|(_, _, t)| *t == "ChoiceForm").and_then(|(g, ..)| get(g)).and_then(|v| v.get(2)).and_then(|r| r.get(1)).and_then(Node::atom);
        match choice_form.and_then(|id| x.full.get(id)) {
            Some(f) => w.text("xr:ChoiceForm", f),
            None => fixed(w, "ChoiceForm", ""),
        }
        fixed(w, "QuickChoice", "Auto");
        // Неопределено (план обмена ЗУП 3.1.37) — Auto: чтение словаря версии < 11 заменяет его значением 0
        // (backend FUN_180d38ae0, свойство 7ba608f2 реестра FUN_180969720).
        // shortcut: у словаря версии ≥ 11 (в ЗУП — 13) замену делает писатель XML — не прослежен; пересмотреть, если Неопределено встретится в другом свойстве
        let history = UNRESOLVED.iter().find(|(_, _, t)| *t == "ChoiceHistoryOnInput").and_then(|(g, ..)| get(g));
        match history.and_then(|v| brace::write(v).ok()) {
            Some(s) if s == "{\"U\"}" => w.text("xr:ChoiceHistoryOnInput", "Auto"),
            _ => fixed(w, "ChoiceHistoryOnInput", "Auto"),
        }
        mlstring(w, "xr:EditFormat", get(EDIT_FORMAT).and_then(|v| v.get(2)));
        fixed(w, "PasswordMode", "false");
        w.text("xr:DataHistory", if num(DATA_HISTORY) == Some("0") { "DontUse" } else { "Use" });
        fixed(w, "MarkNegatives", "false");
        fixed(w, "MinValue", "nil");
        mlstring(w, "xr:Synonym", get(SYNONYM).and_then(|v| v.get(2)));
        w.text("xr:Comment", get(COMMENT).and_then(|v| v.get(1)).and_then(Node::str).unwrap_or_default());
        w.text("xr:FullTextSearch", if num(FULL_TEXT_SEARCH) == Some("0") { "DontUse" } else { "Use" });
        // свойства нет в словаре (реквизит добавлен платформой) — значение по умолчанию
        match get(CHOICE_PARAMETER_LINKS) {
            Some(v) => super::attr::links(w, x, "xr:ChoiceParameterLinks", owner, v.get(2)),
            None => w.text("xr:ChoiceParameterLinks", ""),
        }
        match get(FILL_VALUE) {
            Some(v) => value(w, x, "xr:FillValue", Some(v)),
            None => w.nil("xr:FillValue"),
        }
        w.text("xr:Mask", get(MASK).and_then(|v| v.get(1)).and_then(Node::str).unwrap_or_default());
        match get(CHOICE_PARAMETERS) {
            Some(v) => super::attr::choice_parameters(w, x, "xr:ChoiceParameters", v.get(2)),
            None => w.text("xr:ChoiceParameters", ""),
        }
        w.end();
    }
    w.end();
}
