//! Регистры сведений, накопления, бухгалтерии, журналы документов, последовательности, нумераторы:
//! свойства объекта, измерения/ресурсы/реквизиты/графы в ChildObjects, команды объекта.
//! Позиции сверены с выгрузкой БП 3.0.171 по всем объектам классов (docs/research/xml-regs.md).

use super::classes::{props, P};
use super::common::*;
use super::w::W;
use crate::brace::{is_guid, Node};
use crate::meta::{self, Config, Item};
use std::collections::HashMap;

/// Свойства блока реквизита `{27,...}` в порядке выгрузки (у регистров сведений — с заполнением).
const ATTR_FILL: &[&str] = &[
    "Type", "PasswordMode", "Format", "EditFormat", "ToolTip", "MarkNegatives", "Mask", "MultiLine", "ExtendedEdit", "MinValue", "MaxValue",
    "FillFromFillingValue", "FillValue", "FillChecking", "ChoiceFoldersAndItems", "ChoiceParameterLinks", "ChoiceParameters", "QuickChoice",
    "CreateOnInput", "ChoiceForm", "LinkByType", "ChoiceHistoryOnInput",
];
/// То же без FillFromFillingValue и FillValue (регистры накопления и бухгалтерии).
const ATTR: &[&str] = &[
    "Type", "PasswordMode", "Format", "EditFormat", "ToolTip", "MarkNegatives", "Mask", "MultiLine", "ExtendedEdit", "MinValue", "MaxValue",
    "FillChecking", "ChoiceFoldersAndItems", "ChoiceParameterLinks", "ChoiceParameters", "QuickChoice", "CreateOnInput", "ChoiceForm",
    "LinkByType", "ChoiceHistoryOnInput",
];

const USE: &[(&str, &str)] = &[("0", "DontUse"), ("1", "Use")];
const INDEXING: &[(&str, &str)] = &[("0", "DontIndex"), ("1", "Index"), ("2", "IndexWithAdditionalOrder")];
const LOCK: &[(&str, &str)] = &[("0", "Automatic"), ("1", "Managed"), ("2", "AutomaticAndManaged")];

fn standard(w: &mut W, x: &Index, class: &str, n: &Node, path: &[usize]) {
    // `{0}` — элемента нет (старые объекты)
    if let Some(sa) = n.at(path) {
        super::standard::standard_attributes(w, x, class, &super::classes_refs::owner(x, n), sa);
    }
}

const INFORMATION_REGISTER: &[(&str, P)] = &[
    ("UseStandardCommands", P::Bool(&[1, 21])),
    ("EditType", P::Tok(&[1, 20], &[("0", "InList"), ("1", "InDialog"), ("2", "BothWays")])),
    ("DefaultRecordForm", P::Ref(&[1, 16])),
    ("DefaultListForm", P::Ref(&[1, 17])),
    ("AuxiliaryRecordForm", P::Ref(&[1, 27])),
    ("AuxiliaryListForm", P::Ref(&[1, 28])),
    ("StandardAttributes", P::Custom(|w, x, n| standard(w, x, "InformationRegister", n, &[1, 26, 1]))),
    (
        "InformationRegisterPeriodicity",
        P::Tok(&[1, 18], &[("0", "Nonperiodical"), ("1", "Year"), ("2", "Quarter"), ("3", "Month"), ("4", "Day"), ("5", "Second"), ("6", "RecorderPosition")]),
    ),
    ("WriteMode", P::Tok(&[1, 19], &[("0", "Independent"), ("1", "RecorderSubordinate")])),
    ("MainFilterOnPeriod", P::Custom(ir_main_filter_on_period)),
    ("IncludeHelpInContents", P::Bool(&[1, 22])),
    ("DataLockControlMode", P::Tok(&[1, 24], LOCK)),
    ("FullTextSearch", P::Tok(&[1, 25], USE)),
    ("EnableTotalsSliceFirst", P::Bool(&[1, 35])),
    ("EnableTotalsSliceLast", P::Bool(&[1, 34])),
    ("RecordPresentation", P::Ml(&[1, 29])),
    ("ExtendedRecordPresentation", P::Ml(&[1, 30])),
    ("ListPresentation", P::Ml(&[1, 31])),
    ("ExtendedListPresentation", P::Ml(&[1, 32])),
    ("Explanation", P::Ml(&[1, 33])),
    ("DataHistory", P::Tok(&[1, 36], USE)),
    ("UpdateDataHistoryImmediatelyAfterWrite", P::Bool(&[1, 37])),
    ("ExecuteAfterWriteDataHistoryVersionProcessing", P::Bool(&[1, 38])),
];

/// Флаг (1,23) хранится всегда, но выгружается только у периодического независимого регистра
/// (на каркасе у подчиненного регистратору с флагом 1 платформа пишет false).
fn ir_main_filter_on_period(w: &mut W, _: &Index, n: &Node) {
    let periodic = !matches!(atom(n, &[1, 18]), "0" | "");
    w.bool("MainFilterOnPeriod", periodic && atom(n, &[1, 19]) == "0" && atom(n, &[1, 23]) == "1");
}

/// История данных в старых записях измерения (до версии 9), ресурса (до 6) и реквизита (до 3) не хранится:
/// читатель basic FUN_1806f1fe0 / FUN_180719500 / FUN_180719cc0 ее пропускает, конструктор (FUN_1806f8440,
/// FUN_180718ea0, FUN_1807198d0) ставит 1 — Use.
const IR_DIMENSION: &[(&str, P)] = &[
    ("Master", P::Bool(&[0, 2])),
    ("MainFilter", P::Bool(&[0, 5])),
    ("DenyIncompleteValues", P::Bool(&[0, 3])),
    ("Indexing", P::Tok(&[0, 4], INDEXING)),
    ("FullTextSearch", P::Tok(&[0, 6], USE)),
    ("DataHistory", P::TokOr(&[0, 7], USE, "Use")),
    ("TypeReductionMode", P::Tok(&[0, 8], &[("0", "TransformValues"), ("1", "DeleteData"), ("2", "Deny")])),
];

const IR_RESOURCE: &[(&str, P)] = &[("Indexing", P::Tok(&[0, 2], INDEXING)), ("FullTextSearch", P::Tok(&[0, 3], USE)), ("DataHistory", P::TokOr(&[0, 4], USE, "Use"))];

const ACCUMULATION_REGISTER: &[(&str, P)] = &[
    ("UseStandardCommands", P::Bool(&[1, 16])),
    ("DefaultListForm", P::Ref(&[1, 14])),
    // в эталоне всегда пусто; позиция — по аналогии с регистром сведений
    ("AuxiliaryListForm", P::Ref(&[1, 22])),
    ("RegisterType", P::Tok(&[1, 15], &[("0", "Balance"), ("1", "Turnovers")])),
    ("IncludeHelpInContents", P::Bool(&[1, 17])),
    ("StandardAttributes", P::Custom(|w, x, n| standard(w, x, "AccumulationRegister", n, &[1, 21, 1]))),
    ("DataLockControlMode", P::Tok(&[1, 18], LOCK)),
    ("FullTextSearch", P::Tok(&[1, 19], USE)),
    ("EnableTotalsSplitting", P::Bool(&[1, 20])),
    ("ListPresentation", P::Ml(&[1, 23])),
    ("ExtendedListPresentation", P::Ml(&[1, 24])),
    ("Explanation", P::Ml(&[1, 25])),
];

/// Регистр расчета (ERP 2.5.22: Начисления и Удержания). Позиции 25–27 (нули в обоих) и формы 23/29 — по аналогии
/// с регистром накопления; периодичность встречена только Month.
const CALCULATION_REGISTER: &[(&str, P)] = &[
    ("UseStandardCommands", P::Bool(&[1, 24])),
    ("DefaultListForm", P::Ref(&[1, 23])),
    ("AuxiliaryListForm", P::Ref(&[1, 29])),
    ("Periodicity", P::Tok(&[1, 16], &[("2", "Month")])),
    ("ActionPeriod", P::Bool(&[1, 17])),
    ("BasePeriod", P::Bool(&[1, 18])),
    ("Schedule", P::Ref(&[1, 19])),
    ("ScheduleValue", P::Ref(&[1, 20])),
    ("ScheduleDate", P::Ref(&[1, 21])),
    ("ChartOfCalculationTypes", P::Ref(&[1, 22])),
    ("IncludeHelpInContents", P::Bool(&[1, 25])),
    ("StandardAttributes", P::Custom(|w, x, n| standard(w, x, "CalculationRegister", n, &[1, 28, 1]))),
    ("DataLockControlMode", P::Tok(&[1, 26], LOCK)),
    ("FullTextSearch", P::Tok(&[1, 27], USE)),
    ("ListPresentation", P::Ml(&[1, 30])),
    ("ExtendedListPresentation", P::Ml(&[1, 31])),
    ("Explanation", P::Ml(&[1, 32])),
];

const CR_DIMENSION: &[(&str, P)] = &[
    ("DenyIncompleteValues", P::Bool(&[0, 2])),
    ("BaseDimension", P::Bool(&[0, 3])),
    ("ScheduleLink", P::Ref(&[0, 4])),
    ("Indexing", P::Tok(&[0, 5], INDEXING)),
    ("FullTextSearch", P::Tok(&[0, 6], USE)),
];
const CR_ATTRIBUTE: &[(&str, P)] = &[("ScheduleLink", P::Ref(&[0, 2])), ("Indexing", P::Tok(&[0, 3], INDEXING)), ("FullTextSearch", P::Tok(&[0, 4], USE))];

const AR_DIMENSION: &[(&str, P)] = &[
    ("DenyIncompleteValues", P::Bool(&[0, 2])),
    ("Indexing", P::Tok(&[0, 3], INDEXING)),
    ("FullTextSearch", P::Tok(&[0, 4], USE)),
    ("UseInTotals", P::Bool(&[0, 5])),
];
const AR_RESOURCE: &[(&str, P)] = &[("FullTextSearch", P::Tok(&[0, 2], USE))];
const AR_ATTRIBUTE: &[(&str, P)] = &[("Indexing", P::Tok(&[0, 2], INDEXING)), ("FullTextSearch", P::Tok(&[0, 3], USE))];

/// Регистр бухгалтерии: в БП один, поэтому позиции булевых флагов с одинаковыми значениями взяты по аналогии
/// с регистром накопления и не проверены (см. открытые вопросы в docs/research/xml-regs.md).
const ACCOUNTING_REGISTER: &[(&str, P)] = &[
    ("UseStandardCommands", P::Bool(&[1, 17])),
    ("IncludeHelpInContents", P::Bool(&[1, 18])),
    ("ChartOfAccounts", P::Ref(&[1, 19])),
    ("Correspondence", P::Bool(&[1, 21])),
    ("PeriodAdjustmentLength", P::Custom(|w, _, n| w.text("PeriodAdjustmentLength", atom(n, &[1, 30])))),
    ("DefaultListForm", P::Ref(&[1, 20])),
    ("AuxiliaryListForm", P::Ref(&[1, 26])),
    ("StandardAttributes", P::Custom(|w, x, n| standard(w, x, "AccountingRegister", n, &[1, 25, 1]))),
    ("DataLockControlMode", P::Tok(&[1, 22], LOCK)),
    ("EnableTotalsSplitting", P::Bool(&[1, 24])),
    ("FullTextSearch", P::Tok(&[1, 23], USE)),
    ("ListPresentation", P::Ml(&[1, 27])),
    ("ExtendedListPresentation", P::Ml(&[1, 28])),
    ("Explanation", P::Ml(&[1, 29])),
];

const ACC_DIMENSION: &[(&str, P)] = &[
    ("Balance", P::Bool(&[0, 2])),
    ("AccountingFlag", P::Ref(&[0, 3])),
    ("DenyIncompleteValues", P::Bool(&[0, 5])),
    ("Indexing", P::Tok(&[0, 4], INDEXING)),
    ("FullTextSearch", P::Tok(&[0, 6], USE)),
];
const ACC_RESOURCE: &[(&str, P)] = &[
    ("Balance", P::Bool(&[0, 2])),
    ("AccountingFlag", P::Ref(&[0, 3])),
    ("ExtDimensionAccountingFlag", P::Ref(&[0, 4])),
    ("FullTextSearch", P::Tok(&[0, 5], USE)),
];

const DOCUMENT_JOURNAL: &[(&str, P)] = &[
    ("DefaultForm", P::Ref(&[1, 4])),
    ("AuxiliaryForm", P::Ref(&[1, 13])),
    ("UseStandardCommands", P::Bool(&[1, 5])),
    ("RegisteredDocuments", P::Custom(|w, x, n| md_list(w, "RegisteredDocuments", x, n.at(&[1, 6])))),
    ("IncludeHelpInContents", P::Bool(&[1, 7])),
    ("StandardAttributes", P::Custom(|w, x, n| standard(w, x, "DocumentJournal", n, &[1, 12, 1]))),
    ("ListPresentation", P::Ml(&[1, 14])),
    ("ExtendedListPresentation", P::Ml(&[1, 15])),
    ("Explanation", P::Ml(&[1, 16])),
];

/// Графа журнала `{4,<заголовок>,<ссылки на реквизиты>,<индексирование>}`.
const COLUMN: &[(&str, P)] = &[("Indexing", P::Tok(&[0, 3], INDEXING)), ("References", P::Custom(|w, x, n| md_list(w, "References", x, n.at(&[0, 2]))))];

const SEQUENCE: &[(&str, P)] = &[
    // в эталоне всегда DontMove при 1
    ("MoveBoundaryOnPosting", P::Tok(&[1, 10], &[("0", "Move"), ("1", "DontMove")])),
    ("Documents", P::Custom(|w, x, n| md_list(w, "Documents", x, n.at(&[1, 8])))),
    // в эталоне всегда пусто
    ("RegisterRecords", P::Custom(|w, x, n| md_list(w, "RegisterRecords", x, n.at(&[1, 9])))),
    ("DataLockControlMode", P::Tok(&[1, 11], LOCK)),
];

/// Измерение последовательности `{0,{2,<заголовок>,<типы>},<реквизиты документов>,<измерения регистров>}`.
const SEQ_DIMENSION: &[(&str, P)] = &[
    ("Type", P::Type(&[0, 1, 2])),
    ("DocumentMap", P::Custom(|w, x, n| md_list(w, "DocumentMap", x, n.at(&[0, 2])))),
    // в эталоне всегда пусто
    ("RegisterRecordsMap", P::Custom(|w, x, n| md_list(w, "RegisterRecordsMap", x, n.at(&[0, 3])))),
];

/// Нумератор `{3,<заголовок>,1,11,1,1,1}`: все флаги в единственном объекте равны 1 — порядок после длины
/// взят по документу (тип, длина, периодичность, уникальность; допустимая длина — в конце), не проверен.
const DOCUMENT_NUMERATOR: &[(&str, P)] = &[
    ("NumberType", P::Tok(&[1, 2], &[("0", "Number"), ("1", "String")])),
    ("NumberLength", P::Custom(|w, _, n| w.text("NumberLength", atom(n, &[1, 3])))),
    ("NumberAllowedLength", P::Tok(&[1, 6], &[("0", "Fixed"), ("1", "Variable")])),
    ("NumberPeriodicity", P::Tok(&[1, 4], PERIODICITY)),
    ("CheckUnique", P::Bool(&[1, 5])),
];

/// Периодичность номера: 0, 1, 4 сверены по документам; 2, 3 — по периодичности регистра сведений.
const PERIODICITY: &[(&str, &str)] = &[("0", "Nonperiodical"), ("1", "Year"), ("2", "Quarter"), ("3", "Month"), ("4", "Day")];

/// Список ссылок `{0,N,{"#",157fa490-…,{1,<uuid>}}…}` → `<xr:Item xsi:type="xr:MDObjectRef">`.
fn md_list(w: &mut W, tag: &str, x: &Index, n: Option<&Node>) {
    super::classes::md_refs(w, tag, "xr:Item", &[("xsi:type", "xr:MDObjectRef")], x, n);
}

/// Состав ChildObjects класса: (вид, свойства блока реквизита, свои свойства) в порядке выгрузки.
type Kinds = &'static [(&'static str, &'static [&'static str], &'static [(&'static str, P)])];

const IR_CHILDREN: Kinds = &[("Resource", ATTR_FILL, IR_RESOURCE), ("Attribute", ATTR_FILL, IR_RESOURCE), ("Dimension", ATTR_FILL, IR_DIMENSION)];
const AR_CHILDREN: Kinds = &[("Resource", ATTR, AR_RESOURCE), ("Attribute", ATTR, AR_ATTRIBUTE), ("Dimension", ATTR, AR_DIMENSION)];
const CR_CHILDREN: Kinds = &[("Resource", ATTR, AR_RESOURCE), ("Attribute", ATTR, CR_ATTRIBUTE), ("Dimension", ATTR, CR_DIMENSION)];
const ACC_CHILDREN: Kinds = &[("Dimension", ATTR, ACC_DIMENSION), ("Resource", ATTR, ACC_RESOURCE), ("Attribute", ATTR, AR_ATTRIBUTE)];
const DJ_CHILDREN: Kinds = &[("Column", &[], COLUMN)];
const SEQ_CHILDREN: Kinds = &[("Dimension", &[], SEQ_DIMENSION)];

/// Описание объекта класса этого семейства; `None` — класс не отсюда.
pub fn descriptor(cfg: &Config, x: &Index, class: &str, it: &Item) -> Option<Vec<u8>> {
    let (table, kinds): (&[(&str, P)], Option<Kinds>) = match class {
        "InformationRegister" => (INFORMATION_REGISTER, Some(IR_CHILDREN)),
        "AccumulationRegister" => (ACCUMULATION_REGISTER, Some(AR_CHILDREN)),
        "AccountingRegister" => (ACCOUNTING_REGISTER, Some(ACC_CHILDREN)),
        "CalculationRegister" => (CALCULATION_REGISTER, Some(CR_CHILDREN)),
        "DocumentJournal" => (DOCUMENT_JOURNAL, Some(DJ_CHILDREN)),
        "Sequence" => (SEQUENCE, Some(SEQ_CHILDREN)),
        // у нумератора нет ChildObjects
        "DocumentNumerator" => (DOCUMENT_NUMERATOR, None),
        _ => return None,
    };
    let mut root = object_brace(cfg, it)?;
    if class == "AccountingRegister" {
        upgrade_21(&mut root);
    }
    let mut w = W::metadata();
    w.open(class, &[("uuid", &it.header.id)]);
    internal_info(&mut w, class, &it.header.id, &it.header.name, &root);
    w.open("Properties", &[]);
    if let Some(h) = head(&root) {
        name_synonym_comment(&mut w, h);
    }
    props(&mut w, x, &root, table);
    w.end();
    if let Some(kinds) = kinds {
        child_objects(&mut w, x, it, &root, kinds);
    }
    Some(w.finish())
}

/// Таблицы для загрузки XML (`xml/imp/regs.rs`): свойства объекта класса и (вид, свойства элемента).
pub fn tables(class: &str) -> Option<(&'static [(&'static str, P)], &'static [(&'static str, &'static [&'static str], &'static [(&'static str, P)])])> {
    Some(match class {
        "InformationRegister" => (INFORMATION_REGISTER, IR_CHILDREN),
        "AccumulationRegister" => (ACCUMULATION_REGISTER, AR_CHILDREN),
        "AccountingRegister" => (ACCOUNTING_REGISTER, ACC_CHILDREN),
        "CalculationRegister" => (CALCULATION_REGISTER, CR_CHILDREN),
        "DocumentJournal" => (DOCUMENT_JOURNAL, DJ_CHILDREN),
        "Sequence" => (SEQUENCE, SEQ_CHILDREN),
        "DocumentNumerator" => (DOCUMENT_NUMERATOR, &[]),
        _ => return None,
    })
}

/// Регистр бухгалтерии `{21,<uuid>…}` (так 8.5 пишет новый регистр, см. каркас tools/skel) отличается от `{22,22,<uuid>…}`
/// только вторым полем; вставляем его, чтобы позиции таблиц были одни.
pub(crate) fn upgrade_21(root: &mut Node) {
    if let Node::List(r) = root {
        if let Some(Node::List(b)) = r.get_mut(1) {
            if b.first().and_then(Node::atom) == Some("21") && b.get(1).and_then(Node::atom).is_some_and(is_guid) {
                b.insert(1, Node::Atom("21".into()));
            }
        }
    }
}

/// Элементы вложенных коллекций записи по uuid из заголовка.
pub(crate) fn elements(n: &Node, out: &mut HashMap<String, Node>) {
    let l = n.list();
    let is_coll = matches!((l.first().and_then(Node::atom), l.get(1).and_then(Node::atom)), (Some(g), Some(c)) if is_guid(g) && c.parse::<usize>().ok() == Some(l.len() - 2) && meta::SUBCLASSES.iter().any(|(s, _)| *s == g));
    for c in l {
        if is_coll {
            if let Some(h) = meta::header(c) {
                out.insert(h.id, c.clone());
            }
        }
        elements(c, out);
    }
}

/// Измерения, ресурсы, реквизиты, команды — полностью; формы и макеты — именами.
fn child_objects(w: &mut W, x: &Index, it: &Item, root: &Node, kinds: Kinds) {
    let mut els = HashMap::new();
    elements(root, &mut els);
    let of = |k: &'static str| it.children.iter().filter(move |c| c.kind == k);
    if it.children.iter().all(|c| !matches!(c.kind, "Form" | "Template" | "Command") && !kinds.iter().any(|(k, _, _)| *k == c.kind)) {
        w.text("ChildObjects", "");
        return;
    }
    w.open("ChildObjects", &[]);
    for (kind, attr, table) in kinds {
        for c in of(kind) {
            element(w, x, c, els.get(&c.header.id), |w, x, n| {
                match n.at(&[0, 1]) {
                    Some(b) => attr.iter().for_each(|t| super::attr::prop(w, x, b, t)),
                    None if !attr.is_empty() => w.text("Type", "\u{1}нет блока реквизита"),
                    None => {}
                }
                props(w, x, n, table);
            });
        }
    }
    for c in of("Form") {
        w.text("Form", &c.header.name);
    }
    for c in of("Template") {
        w.text("Template", &c.header.name);
    }
    for c in of("Command") {
        super::classes_tools::command(w, x, c, root);
    }
    w.end();
}

/// `<Вид uuid=…><Properties>` Name, Synonym, Comment и свойства элемента.
fn element(w: &mut W, x: &Index, c: &Item, n: Option<&Node>, body: impl Fn(&mut W, &Index, &Node)) {
    w.open(c.kind, &[("uuid", &c.header.id)]);
    w.open("Properties", &[]);
    match n {
        Some(n) => {
            if let Some(h) = head(n) {
                name_synonym_comment(w, h);
            }
            body(w, x, n);
        }
        None => w.text("Name", "\u{1}нет элемента в записи"),
    }
    w.end();
    w.end();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::brace;

    fn xml(f: impl FnOnce(&mut W)) -> String {
        let mut w = W::metadata();
        f(&mut w);
        let s = String::from_utf8(w.finish()[3..].to_vec()).unwrap();
        let l: Vec<&str> = s.split("\r\n").collect();
        l[2..l.len() - 1].iter().map(|s| s.trim_start_matches('\t')).collect::<Vec<_>>().join("|")
    }

    fn node(t: &str) -> Node {
        brace::Doc::parse(t).unwrap().roots.remove(0)
    }

    const DIM: &str = "{{9,{27,{2,{3,{1,0,82d5ed02-654f-4b86-b995-baf64a147a12},\"Ид\",{1,\"ru\",\"Ид\"},\"\",0,0,00000000-0000-0000-0000-000000000000,0},{\"Pattern\",{\"B\"}}},0,{0},{0},0,\"\",0,{\"U\"},{\"U\"},0,00000000-0000-0000-0000-000000000000,2,0,{5006,0},{3,0,0},{0,0},0,{0},{\"U\"},0,0,0},1,0,2,0,1,1},0}";

    #[test]
    fn dimension_and_lookup() {
        let x = Index { full: HashMap::new(), types: HashMap::new() };
        let root = node(&format!("{{1,{{13134203-f60b-11d5-a3c7-0050bae0a776,1,{DIM}}}}}"));
        let mut els = HashMap::new();
        elements(&root, &mut els);
        let n = &els["82d5ed02-654f-4b86-b995-baf64a147a12"];
        assert_eq!(
            xml(|w| props(w, &x, n, IR_DIMENSION)),
            "<Master>true</Master>|<MainFilter>false</MainFilter>|<DenyIncompleteValues>false</DenyIncompleteValues>|<Indexing>IndexWithAdditionalOrder</Indexing>|<FullTextSearch>Use</FullTextSearch>|<DataHistory>Use</DataHistory>|<TypeReductionMode>TransformValues</TypeReductionMode>"
        );
        let b = n.at(&[0, 1]).unwrap();
        assert_eq!(xml(|w| super::super::attr::prop(w, &x, b, "QuickChoice")), "<QuickChoice>Auto</QuickChoice>");
        // измерение версии 6 (ЗУП 3.1.37): без истории данных и сокращения типа
        let old = node(&DIM.replacen("{{9,", "{{6,", 1).replace("},1,0,2,0,1,1},0}", "},1,0,2,0,1},0}"));
        assert!(xml(|w| props(w, &x, &old, IR_DIMENSION)).ends_with("<DataHistory>Use</DataHistory>|<TypeReductionMode>TransformValues</TypeReductionMode>"));
    }

    #[test]
    fn main_filter_only_when_periodic() {
        let x = Index { full: HashMap::new(), types: HashMap::new() };
        let r = |period: &str, mode: &str| {
            let mut l = vec!["0"; 24];
            l[18] = period;
            l[19] = mode;
            l[23] = "1";
            node(&format!("{{1,{{{}}}}}", l.join(",")))
        };
        assert_eq!(xml(|w| ir_main_filter_on_period(w, &x, &r("3", "0"))), "<MainFilterOnPeriod>true</MainFilterOnPeriod>");
        assert_eq!(xml(|w| ir_main_filter_on_period(w, &x, &r("0", "0"))), "<MainFilterOnPeriod>false</MainFilterOnPeriod>");
        assert_eq!(xml(|w| ir_main_filter_on_period(w, &x, &r("4", "1"))), "<MainFilterOnPeriod>false</MainFilterOnPeriod>");
    }
}
