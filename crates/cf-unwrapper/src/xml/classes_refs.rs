//! Ссылочные объекты: справочники, документы, планы видов характеристик, счетов, видов расчета, обмена,
//! бизнес-процессы, задачи. Свойства объекта — таблицы позиций в корне[1]; вложенные элементы (реквизиты,
//! табличные части, команды) ищутся по заголовку внутри записи владельца. Таблицы — docs/research/xml-refs.md.

use super::classes::{md_refs, P};
use super::common::*;
use super::w::W;
use crate::brace::Node;
use crate::meta::Item;

const NUM: &[(&str, &str)] = &[];
const USE: &[(&str, &str)] = &[("0", "DontUse"), ("1", "Use")];
const CHECK: &[(&str, &str)] = &[("0", "DontCheck"), ("1", "ShowError")];
const LOCK: &[(&str, &str)] = &[("0", "Automatic"), ("1", "Managed"), ("2", "AutomaticAndManaged")];
const EDIT_TYPE: &[(&str, &str)] = &[("0", "InList"), ("1", "InDialog"), ("2", "BothWays")];
const CHOICE_MODE: &[(&str, &str)] = &[("0", "FromForm"), ("1", "QuickChoice"), ("2", "BothWays")];
const CREATE_ON_INPUT: &[(&str, &str)] = &[("0", "Auto"), ("1", "DontUse"), ("2", "Use")];
const HISTORY_ON_INPUT: &[(&str, &str)] = &[("0", "Auto"), ("1", "DontUse")];
const ALLOWED_LENGTH: &[(&str, &str)] = &[("0", "Fixed"), ("1", "Variable")];
const STRING_OR_NUMBER: &[(&str, &str)] = &[("0", "Number"), ("1", "String")];
const PRESENTATION: &[(&str, &str)] = &[("0", "AsCode"), ("1", "AsDescription")];
const PREDEFINED_UPDATE: &[(&str, &str)] = &[("0", "Auto"), ("1", "AutoUpdate"), ("2", "DontAutoUpdate")];
const INDEXING: &[(&str, &str)] = &[("0", "DontIndex"), ("1", "Index"), ("2", "IndexWithAdditionalOrder")];
const ITEM_USE: &[(&str, &str)] = &[("0", "ForItem"), ("1", "ForFolder"), ("2", "ForFolderAndItem")];
// режимы ввода по строке `{<поиск>,<полнотекстовый>,<получение данных>}`; у старых записей списка нет — Begin, DontUse, Directly
const SEARCH_MODE: &[(&str, &str)] = &[("1", "Begin"), ("2", "AnyPart")];
const FULL_TEXT_ON_INPUT: &[(&str, &str)] = &[("1", "Use"), ("2", "DontUse")];
const DATA_GET_MODE: &[(&str, &str)] = &[("0", "Directly"), ("1", "Background")];

/// Таблица свойств класса; `None` — класс не из этого семейства.
pub fn table(class: &str) -> Option<&'static [(&'static str, P)]> {
    Some(match class {
        "Catalog" => CATALOG,
        "Document" => DOCUMENT,
        "ChartOfCharacteristicTypes" => CHART_OF_CHARACTERISTIC_TYPES,
        "ChartOfAccounts" => CHART_OF_ACCOUNTS,
        "ChartOfCalculationTypes" => CHART_OF_CALCULATION_TYPES,
        "ExchangePlan" => EXCHANGE_PLAN,
        "BusinessProcess" => BUSINESS_PROCESS,
        "Task" => TASK,
        _ => return None,
    })
}

const CATALOG: &[(&str, P)] = &[
    ("Hierarchical", P::Bool(&[1, 37])),
    ("HierarchyType", P::Tok(&[1, 36], &[("0", "HierarchyFoldersAndItems"), ("1", "HierarchyOfItems")])),
    ("LimitLevelCount", P::Bool(&[1, 38])),
    ("LevelCount", P::Tok(&[1, 10], NUM)),
    ("FoldersOnTop", P::Bool(&[1, 13])),
    ("UseStandardCommands", P::Bool(&[1, 31])),
    ("Owners", P::Custom(|w, x, n| md_refs(w, "Owners", "xr:Item", &[("xsi:type", "xr:MDObjectRef")], x, n.at(&[1, 12])))),
    // в эталоне всегда 0 (ToItems): позиция — единственный постоянный 0 среди соседних свойств иерархии
    ("SubordinationUse", P::Tok(&[1, 39], &[("0", "ToItems"), ("1", "ToFolders"), ("2", "ToFoldersAndItems")])),
    ("CodeLength", P::Tok(&[1, 17], NUM)),
    ("DescriptionLength", P::Tok(&[1, 19], NUM)),
    ("CodeType", P::Tok(&[1, 18], STRING_OR_NUMBER)),
    ("CodeAllowedLength", P::Tok(&[1, 51], ALLOWED_LENGTH)),
    ("CodeSeries", P::Tok(&[1, 16], &[("0", "WholeCatalog"), ("1", "WithinSubordination"), ("2", "WithinOwnerSubordination")])),
    ("CheckUnique", P::Bool(&[1, 14])),
    ("Autonumbering", P::Bool(&[1, 15])),
    ("DefaultPresentation", P::Tok(&[1, 20], PRESENTATION)),
    ("StandardAttributes", P::Custom(|w, x, n| standard(w, x, "Catalog", n, n.at(&[1, 45, 1])))),
    ("Characteristics", P::Custom(|w, x, n| characteristics(w, x, n.at(&[1, 52])))),
    ("PredefinedDataUpdate", P::Tok(&[1, 55], PREDEFINED_UPDATE)),
    ("EditType", P::Tok(&[1, 11], EDIT_TYPE)),
    ("QuickChoice", P::Bool(&[1, 41])),
    ("ChoiceMode", P::Tok(&[1, 40], CHOICE_MODE)),
    ("InputByString", P::Custom(|w, x, n| fields(w, x, "InputByString", n, &[1, 42]))),
    ("SearchStringModeOnInputByString", P::TokOr(&[1, 56, 0], SEARCH_MODE, "Begin")),
    ("FullTextSearchOnInputByString", P::TokOr(&[1, 56, 1], FULL_TEXT_ON_INPUT, "DontUse")),
    ("ChoiceDataGetModeOnInputByString", P::TokOr(&[1, 56, 2], DATA_GET_MODE, "Directly")),
    ("DefaultObjectForm", P::Ref(&[1, 21])),
    ("DefaultFolderForm", P::Ref(&[1, 22])),
    ("DefaultListForm", P::Ref(&[1, 23])),
    ("DefaultChoiceForm", P::Ref(&[1, 24])),
    ("DefaultFolderChoiceForm", P::Ref(&[1, 25])),
    ("AuxiliaryObjectForm", P::Ref(&[1, 26])),
    ("AuxiliaryFolderForm", P::Ref(&[1, 27])),
    ("AuxiliaryListForm", P::Ref(&[1, 28])),
    ("AuxiliaryChoiceForm", P::Ref(&[1, 29])),
    ("AuxiliaryFolderChoiceForm", P::Ref(&[1, 30])),
    ("IncludeHelpInContents", P::Bool(&[1, 33])),
    ("BasedOn", P::Custom(|w, x, n| md_refs(w, "BasedOn", "xr:Item", &[("xsi:type", "xr:MDObjectRef")], x, n.at(&[1, 32])))),
    ("DataLockFields", P::Custom(|w, x, n| fields(w, x, "DataLockFields", n, &[1, 54]))),
    ("DataLockControlMode", P::Tok(&[1, 43], LOCK)),
    ("FullTextSearch", P::Tok(&[1, 44], USE)),
    ("ObjectPresentation", P::Ml(&[1, 46])),
    ("ExtendedObjectPresentation", P::Ml(&[1, 47])),
    ("ListPresentation", P::Ml(&[1, 48])),
    ("ExtendedListPresentation", P::Ml(&[1, 49])),
    ("Explanation", P::Ml(&[1, 50])),
    ("CreateOnInput", P::Tok(&[1, 53], CREATE_ON_INPUT)),
    ("ChoiceHistoryOnInput", P::Tok(&[1, 57], HISTORY_ON_INPUT)),
    // три последних позиции записи (нет у старой версии 53); в эталоне всегда 0
    ("DataHistory", P::Tok(&[1, 58], USE)),
    ("UpdateDataHistoryImmediatelyAfterWrite", P::Bool(&[1, 59])),
    ("ExecuteAfterWriteDataHistoryVersionProcessing", P::Bool(&[1, 60])),
];

/// Запись справочника старой версии достраивается, как при загрузке (basic FUN_180738320), чтобы выгрузка
/// и таблица загрузки читали одни позиции:
/// - до версии 53 (≤ 0x34) в стандартные реквизиты (1,45,1), если они есть, добавляется PredefinedDataName (-13)
///   со свойствами по умолчанию; в std::map по коду (backend FUN_180d417e0 → FUN_180d44950, CompositeID::operator<)
///   -13 — наименьший код, поэтому первым;
/// - создание при вводе (1,53) пишется с версии 48; до нее загрузка ставит 2 − QuickChoice (1,41): Use,
///   при быстром выборе DontUse.
/// shortcut: создание при вводе — только версия 47 (длина 53); у версий < 47 раскладка другая (поля по версиям 0x2b,
/// 0x2d в FUN_180738320), там CreateOnInput выйдет Auto; пересмотреть, если встретится справочник версии < 47
pub(crate) fn upgrade_old_catalog(root: &mut Node) {
    let ver: u32 = root.at(&[1, 0]).and_then(Node::atom).and_then(|v| v.parse().ok()).unwrap_or(u32::MAX);
    if let (true, Some(Node::List(sa))) = (ver <= 52, root.at_mut(&[1, 45, 1])) {
        let count: Option<usize> = sa.get(1).and_then(Node::atom).and_then(|a| a.parse().ok());
        let has = (0..count.unwrap_or(0)).any(|k| sa.get(2 + 3 * k).and_then(|c| c.get(0)).and_then(Node::atom) == Some("-13"));
        if let (Some(c), false) = (count, has) {
            sa[1] = Node::Atom((c + 1).to_string());
            let code = Node::List(vec![Node::Atom("-13".into())]);
            sa.splice(2..2, [code, Node::Atom("510405d3-2a0c-4fea-960a-7fee59b32f9b".into()), Node::List(vec![])]);
        }
    }
    if let (47, Some(Node::List(b))) = (ver, root.at_mut(&[1])) {
        if b.len() == 53 {
            let v = if b.get(41).and_then(Node::atom) == Some("1") { "1" } else { "2" };
            b.push(Node::Atom(v.into()));
        }
    }
}

const DOCUMENT: &[(&str, P)] = &[
    ("UseStandardCommands", P::Bool(&[1, 23])),
    ("Numerator", P::Ref(&[1, 10])),
    ("NumberType", P::Tok(&[1, 11], STRING_OR_NUMBER)),
    ("NumberLength", P::Tok(&[1, 12], NUM)),
    ("NumberAllowedLength", P::Tok(&[1, 44], ALLOWED_LENGTH)),
    ("NumberPeriodicity", P::Tok(&[1, 13], PERIODICITY)),
    ("CheckUnique", P::Bool(&[1, 14])),
    ("Autonumbering", P::Bool(&[1, 15])),
    ("StandardAttributes", P::Custom(|w, x, n| standard(w, x, "Document", n, n.at(&[1, 32, 1])))),
    ("Characteristics", P::Custom(|w, x, n| characteristics(w, x, n.at(&[1, 45])))),
    ("BasedOn", P::Custom(|w, x, n| md_refs(w, "BasedOn", "xr:Item", &[("xsi:type", "xr:MDObjectRef")], x, n.at(&[1, 22])))),
    ("InputByString", P::Custom(|w, x, n| fields(w, x, "InputByString", n, &[1, 29]))),
    ("CreateOnInput", P::Tok(&[1, 46], CREATE_ON_INPUT)),
    ("SearchStringModeOnInputByString", P::TokOr(&[1, 48, 0], SEARCH_MODE, "Begin")),
    ("FullTextSearchOnInputByString", P::TokOr(&[1, 48, 1], FULL_TEXT_ON_INPUT, "DontUse")),
    ("ChoiceDataGetModeOnInputByString", P::TokOr(&[1, 48, 2], DATA_GET_MODE, "Directly")),
    ("DefaultObjectForm", P::Ref(&[1, 16])),
    ("DefaultListForm", P::Ref(&[1, 17])),
    ("DefaultChoiceForm", P::Ref(&[1, 18])),
    // в эталоне всегда пусты: три нулевых uuid подряд
    ("AuxiliaryObjectForm", P::Ref(&[1, 35])),
    ("AuxiliaryListForm", P::Ref(&[1, 36])),
    ("AuxiliaryChoiceForm", P::Ref(&[1, 37])),
    ("Posting", P::Tok(&[1, 19], ALLOW_DENY)),
    ("RealTimePosting", P::Tok(&[1, 21], ALLOW_DENY)),
    ("RegisterRecordsDeletion", P::Tok(&[1, 20], &[("0", "AutoDelete"), ("1", "AutoDeleteOff"), ("2", "AutoDeleteOnUnpost")])),
    ("RegisterRecordsWritingOnPost", P::Tok(&[1, 43], &[("0", "WriteSelected"), ("1", "WriteModified")])),
    ("SequenceFilling", P::Tok(&[1, 28], &[("0", "AutoFill"), ("1", "AutoFillOff")])),
    ("RegisterRecords", P::Custom(|w, x, n| md_refs(w, "RegisterRecords", "xr:Item", &[("xsi:type", "xr:MDObjectRef")], x, n.at(&[1, 24])))),
    ("PostInPrivilegedMode", P::Bool(&[1, 33])),
    ("UnpostInPrivilegedMode", P::Bool(&[1, 34])),
    ("IncludeHelpInContents", P::Bool(&[1, 25])),
    ("DataLockFields", P::Custom(|w, x, n| fields(w, x, "DataLockFields", n, &[1, 47]))),
    ("DataLockControlMode", P::Tok(&[1, 30], LOCK)),
    ("FullTextSearch", P::Tok(&[1, 31], USE)),
    ("ObjectPresentation", P::Ml(&[1, 38])),
    ("ExtendedObjectPresentation", P::Ml(&[1, 39])),
    ("ListPresentation", P::Ml(&[1, 40])),
    ("ExtendedListPresentation", P::Ml(&[1, 41])),
    ("Explanation", P::Ml(&[1, 42])),
    ("ChoiceHistoryOnInput", P::Tok(&[1, 49], HISTORY_ON_INPUT)),
    // три последних позиции записи (нет у старой версии 37); в эталоне всегда 0
    ("DataHistory", P::Tok(&[1, 50], USE)),
    ("UpdateDataHistoryImmediatelyAfterWrite", P::Bool(&[1, 51])),
    ("ExecuteAfterWriteDataHistoryVersionProcessing", P::Bool(&[1, 52])),
];

// Классы ниже: в БП 3.0.171 от 1 до 26 объектов; позиции свойств, не меняющихся ни в БП, ни в УТ 11.5.27,
// взяты по соседству (порядок свойств в записях классов общий) — список в docs/research/xml-refs.md.

const CHART_OF_CHARACTERISTIC_TYPES: &[(&str, P)] = &[
    ("UseStandardCommands", P::Bool(&[1, 14])),
    ("IncludeHelpInContents", P::Bool(&[1, 16])),
    ("CharacteristicExtValues", P::Ref(&[1, 17])),
    ("Type", P::Type(&[1, 18])),
    ("Hierarchical", P::Bool(&[1, 19])),
    ("FoldersOnTop", P::Bool(&[1, 20])),
    ("CodeLength", P::Tok(&[1, 21], NUM)),
    ("CodeAllowedLength", P::Tok(&[1, 49], ALLOWED_LENGTH)),
    ("DescriptionLength", P::Tok(&[1, 23], NUM)),
    ("CodeSeries", P::Tok(&[1, 35], &[("0", "WholeCharacteristicKind"), ("1", "WithinSubordination")])),
    ("CheckUnique", P::Bool(&[1, 34])),
    ("Autonumbering", P::Bool(&[1, 22])),
    ("DefaultPresentation", P::Tok(&[1, 24], PRESENTATION)),
    ("StandardAttributes", P::Custom(|w, x, n| standard(w, x, "ChartOfCharacteristicTypes", n, n.at(&[1, 38, 1])))),
    ("Characteristics", P::Custom(|w, x, n| characteristics(w, x, n.at(&[1, 50])))),
    ("PredefinedDataUpdate", P::Tok(&[1, 53], PREDEFINED_UPDATE)),
    ("EditType", P::Tok(&[1, 25], EDIT_TYPE)),
    ("QuickChoice", P::Bool(&[1, 32])),
    ("ChoiceMode", P::Tok(&[1, 31], CHOICE_MODE)),
    ("InputByString", P::Custom(|w, x, n| fields(w, x, "InputByString", n, &[1, 33]))),
    ("CreateOnInput", P::Tok(&[1, 51], CREATE_ON_INPUT)),
    ("SearchStringModeOnInputByString", P::TokOr(&[1, 54, 0], SEARCH_MODE, "Begin")),
    ("ChoiceDataGetModeOnInputByString", P::TokOr(&[1, 54, 2], DATA_GET_MODE, "Directly")),
    ("FullTextSearchOnInputByString", P::TokOr(&[1, 54, 1], FULL_TEXT_ON_INPUT, "DontUse")),
    ("ChoiceHistoryOnInput", P::Tok(&[1, 55], HISTORY_ON_INPUT)),
    ("DefaultObjectForm", P::Ref(&[1, 26])),
    ("DefaultFolderForm", P::Ref(&[1, 27])),
    ("DefaultListForm", P::Ref(&[1, 28])),
    ("DefaultChoiceForm", P::Ref(&[1, 29])),
    ("DefaultFolderChoiceForm", P::Ref(&[1, 30])),
    ("AuxiliaryObjectForm", P::Ref(&[1, 39])),
    ("AuxiliaryFolderForm", P::Ref(&[1, 40])),
    ("AuxiliaryListForm", P::Ref(&[1, 41])),
    ("AuxiliaryChoiceForm", P::Ref(&[1, 42])),
    ("AuxiliaryFolderChoiceForm", P::Ref(&[1, 43])),
    ("BasedOn", P::Custom(|w, x, n| refs(w, x, "BasedOn", n.at(&[1, 15])))),
    ("DataLockFields", P::Custom(|w, x, n| fields(w, x, "DataLockFields", n, &[1, 52]))),
    ("DataLockControlMode", P::Tok(&[1, 36], LOCK)),
    ("FullTextSearch", P::Tok(&[1, 37], USE)),
    ("ObjectPresentation", P::Ml(&[1, 44])),
    ("ExtendedObjectPresentation", P::Ml(&[1, 45])),
    ("ListPresentation", P::Ml(&[1, 46])),
    ("ExtendedListPresentation", P::Ml(&[1, 47])),
    ("Explanation", P::Ml(&[1, 48])),
    ("DataHistory", P::Tok(&[1, 56], USE)),
    ("UpdateDataHistoryImmediatelyAfterWrite", P::Bool(&[1, 57])),
    ("ExecuteAfterWriteDataHistoryVersionProcessing", P::Bool(&[1, 58])),
];

const CHART_OF_ACCOUNTS: &[(&str, P)] = &[
    ("UseStandardCommands", P::Bool(&[1, 16])),
    ("IncludeHelpInContents", P::Bool(&[1, 17])),
    ("BasedOn", P::Custom(|w, x, n| refs(w, x, "BasedOn", n.at(&[1, 18])))),
    ("ExtDimensionTypes", P::Ref(&[1, 19])),
    ("MaxExtDimensionCount", P::Tok(&[1, 20], NUM)),
    ("CodeMask", P::Str(&[1, 21])),
    ("CodeLength", P::Tok(&[1, 22], NUM)),
    ("DescriptionLength", P::Tok(&[1, 23], NUM)),
    ("CodeSeries", P::Tok(&[1, 35], &[("0", "WholeChartOfAccounts"), ("1", "WithinSubordination")])),
    ("CheckUnique", P::Bool(&[1, 34])),
    ("DefaultPresentation", P::Tok(&[1, 26], PRESENTATION)),
    ("StandardAttributes", P::Custom(|w, x, n| standard(w, x, "ChartOfAccounts", n, n.at(&[1, 38, 1])))),
    ("Characteristics", P::Custom(|w, x, n| characteristics(w, x, n.at(&[1, 48])))),
    ("StandardTabularSections", P::Custom(|w, x, n| standard_tabular_sections(w, x, "ChartOfAccounts", n, n.at(&[1, 39, 1])))),
    ("PredefinedDataUpdate", P::Tok(&[1, 51], PREDEFINED_UPDATE)),
    ("EditType", P::Tok(&[1, 27], EDIT_TYPE)),
    ("QuickChoice", P::Bool(&[1, 32])),
    ("ChoiceMode", P::Tok(&[1, 31], CHOICE_MODE)),
    ("InputByString", P::Custom(|w, x, n| fields(w, x, "InputByString", n, &[1, 33]))),
    ("SearchStringModeOnInputByString", P::TokOr(&[1, 52, 0], SEARCH_MODE, "Begin")),
    ("FullTextSearchOnInputByString", P::TokOr(&[1, 52, 1], FULL_TEXT_ON_INPUT, "DontUse")),
    ("ChoiceDataGetModeOnInputByString", P::TokOr(&[1, 52, 2], DATA_GET_MODE, "Directly")),
    ("CreateOnInput", P::Tok(&[1, 49], CREATE_ON_INPUT)),
    ("ChoiceHistoryOnInput", P::Tok(&[1, 53], HISTORY_ON_INPUT)),
    ("DefaultObjectForm", P::Ref(&[1, 28])),
    ("DefaultListForm", P::Ref(&[1, 29])),
    ("DefaultChoiceForm", P::Ref(&[1, 30])),
    ("AuxiliaryObjectForm", P::Ref(&[1, 40])),
    ("AuxiliaryListForm", P::Ref(&[1, 41])),
    ("AuxiliaryChoiceForm", P::Ref(&[1, 42])),
    ("AutoOrderByCode", P::Bool(&[1, 24])),
    ("OrderLength", P::Tok(&[1, 25], NUM)),
    ("DataLockFields", P::Custom(|w, x, n| fields(w, x, "DataLockFields", n, &[1, 50]))),
    ("DataLockControlMode", P::Tok(&[1, 36], LOCK)),
    ("FullTextSearch", P::Tok(&[1, 37], USE)),
    ("DataHistory", P::Tok(&[1, 54], USE)),
    ("UpdateDataHistoryImmediatelyAfterWrite", P::Bool(&[1, 55])),
    ("ExecuteAfterWriteDataHistoryVersionProcessing", P::Bool(&[1, 56])),
    ("ObjectPresentation", P::Ml(&[1, 43])),
    ("ExtendedObjectPresentation", P::Ml(&[1, 44])),
    ("ListPresentation", P::Ml(&[1, 45])),
    ("ExtendedListPresentation", P::Ml(&[1, 46])),
    ("Explanation", P::Ml(&[1, 47])),
];

const CHART_OF_CALCULATION_TYPES: &[(&str, P)] = &[
    ("UseStandardCommands", P::Bool(&[1, 35])),
    ("CodeLength", P::Tok(&[1, 25], NUM)),
    ("DescriptionLength", P::Tok(&[1, 30], NUM)),
    ("CodeType", P::Tok(&[1, 26], STRING_OR_NUMBER)),
    ("CodeAllowedLength", P::Tok(&[1, 53], ALLOWED_LENGTH)),
    ("DefaultPresentation", P::Tok(&[1, 31], PRESENTATION)),
    ("EditType", P::Tok(&[1, 24], EDIT_TYPE)),
    ("QuickChoice", P::Bool(&[1, 39])),
    ("ChoiceMode", P::Tok(&[1, 38], CHOICE_MODE)),
    ("InputByString", P::Custom(|w, x, n| fields(w, x, "InputByString", n, &[1, 40]))),
    ("SearchStringModeOnInputByString", P::TokOr(&[1, 58, 0], SEARCH_MODE, "Begin")),
    ("FullTextSearchOnInputByString", P::TokOr(&[1, 58, 1], FULL_TEXT_ON_INPUT, "DontUse")),
    ("ChoiceDataGetModeOnInputByString", P::TokOr(&[1, 58, 2], DATA_GET_MODE, "Directly")),
    ("CreateOnInput", P::Tok(&[1, 55], CREATE_ON_INPUT)),
    ("ChoiceHistoryOnInput", P::Tok(&[1, 59], HISTORY_ON_INPUT)),
    ("DefaultObjectForm", P::Ref(&[1, 32])),
    ("DefaultListForm", P::Ref(&[1, 33])),
    ("DefaultChoiceForm", P::Ref(&[1, 34])),
    ("AuxiliaryObjectForm", P::Ref(&[1, 45])),
    ("AuxiliaryListForm", P::Ref(&[1, 46])),
    ("AuxiliaryChoiceForm", P::Ref(&[1, 47])),
    ("BasedOn", P::Custom(|w, x, n| refs(w, x, "BasedOn", n.at(&[1, 36])))),
    ("DependenceOnCalculationTypes", P::Tok(&[1, 27], &[("0", "DontUse"), ("1", "OnActionPeriod"), ("2", "OnRegistrationPeriod")])),
    ("BaseCalculationTypes", P::Custom(|w, x, n| refs(w, x, "BaseCalculationTypes", n.at(&[1, 28])))),
    ("ActionPeriodUse", P::Bool(&[1, 29])),
    ("StandardAttributes", P::Custom(|w, x, n| standard(w, x, "ChartOfCalculationTypes", n, n.at(&[1, 43, 1])))),
    ("Characteristics", P::Custom(|w, x, n| characteristics(w, x, n.at(&[1, 54])))),
    ("StandardTabularSections", P::Custom(|w, x, n| standard_tabular_sections(w, x, "ChartOfCalculationTypes", n, n.at(&[1, 44, 1])))),
    ("PredefinedDataUpdate", P::Tok(&[1, 57], PREDEFINED_UPDATE)),
    ("IncludeHelpInContents", P::Bool(&[1, 37])),
    ("DataLockFields", P::Custom(|w, x, n| fields(w, x, "DataLockFields", n, &[1, 56]))),
    ("DataLockControlMode", P::Tok(&[1, 41], LOCK)),
    ("FullTextSearch", P::Tok(&[1, 42], USE)),
    ("ObjectPresentation", P::Ml(&[1, 48])),
    ("ExtendedObjectPresentation", P::Ml(&[1, 49])),
    ("ListPresentation", P::Ml(&[1, 50])),
    ("ExtendedListPresentation", P::Ml(&[1, 51])),
    ("Explanation", P::Ml(&[1, 52])),
    ("DataHistory", P::Tok(&[1, 60], USE)),
    ("UpdateDataHistoryImmediatelyAfterWrite", P::Bool(&[1, 61])),
    ("ExecuteAfterWriteDataHistoryVersionProcessing", P::Bool(&[1, 62])),
];

const EXCHANGE_PLAN: &[(&str, P)] = &[
    ("UseStandardCommands", P::Bool(&[1, 13])),
    ("CodeLength", P::Tok(&[1, 15], NUM)),
    ("CodeAllowedLength", P::Tok(&[1, 39], ALLOWED_LENGTH)),
    ("DescriptionLength", P::Tok(&[1, 17], NUM)),
    ("DefaultPresentation", P::Tok(&[1, 25], PRESENTATION)),
    ("EditType", P::Tok(&[1, 21], EDIT_TYPE)),
    ("QuickChoice", P::Bool(&[1, 23])),
    ("ChoiceMode", P::Tok(&[1, 22], CHOICE_MODE)),
    ("InputByString", P::Custom(|w, x, n| fields(w, x, "InputByString", n, &[1, 27]))),
    ("SearchStringModeOnInputByString", P::TokOr(&[1, 43, 0], SEARCH_MODE, "Begin")),
    ("FullTextSearchOnInputByString", P::TokOr(&[1, 43, 1], FULL_TEXT_ON_INPUT, "DontUse")),
    ("ChoiceDataGetModeOnInputByString", P::TokOr(&[1, 43, 2], DATA_GET_MODE, "Directly")),
    ("DefaultObjectForm", P::Ref(&[1, 14])),
    ("DefaultListForm", P::Ref(&[1, 19])),
    ("DefaultChoiceForm", P::Ref(&[1, 20])),
    ("AuxiliaryObjectForm", P::Ref(&[1, 31])),
    ("AuxiliaryListForm", P::Ref(&[1, 32])),
    ("AuxiliaryChoiceForm", P::Ref(&[1, 33])),
    ("StandardAttributes", P::Custom(|w, x, n| standard(w, x, "ExchangePlan", n, n.at(&[1, 30, 1])))),
    ("Characteristics", P::Custom(|w, x, n| characteristics(w, x, n.at(&[1, 40])))),
    ("BasedOn", P::Custom(|w, x, n| refs(w, x, "BasedOn", n.at(&[1, 24])))),
    ("DistributedInfoBase", P::Bool(&[1, 26])),
    ("IncludeConfigurationExtensions", P::Bool(&[1, 45])),
    // у записей версии 28 позиции нет — DontUse
    ("CreateOnInput", P::TokOr(&[1, 41], CREATE_ON_INPUT, "DontUse")),
    ("ChoiceHistoryOnInput", P::Tok(&[1, 44], HISTORY_ON_INPUT)),
    ("IncludeHelpInContents", P::Bool(&[1, 18])),
    ("DataLockFields", P::Custom(|w, x, n| fields(w, x, "DataLockFields", n, &[1, 42]))),
    ("DataLockControlMode", P::Tok(&[1, 28], LOCK)),
    ("FullTextSearch", P::Tok(&[1, 29], USE)),
    ("ObjectPresentation", P::Ml(&[1, 34])),
    ("ExtendedObjectPresentation", P::Ml(&[1, 35])),
    ("ListPresentation", P::Ml(&[1, 36])),
    ("ExtendedListPresentation", P::Ml(&[1, 37])),
    ("Explanation", P::Ml(&[1, 38])),
    ("DataHistory", P::Tok(&[1, 46], USE)),
    ("UpdateDataHistoryImmediatelyAfterWrite", P::Bool(&[1, 47])),
    ("ExecuteAfterWriteDataHistoryVersionProcessing", P::Bool(&[1, 48])),
];

const BUSINESS_PROCESS: &[(&str, P)] = &[
    ("UseStandardCommands", P::Bool(&[1, 2])),
    ("EditType", P::Tok(&[1, 21], EDIT_TYPE)),
    ("InputByString", P::Custom(|w, x, n| fields(w, x, "InputByString", n, &[1, 27]))),
    ("CreateOnInput", P::Tok(&[1, 42], CREATE_ON_INPUT)),
    ("SearchStringModeOnInputByString", P::TokOr(&[1, 44, 0], SEARCH_MODE, "Begin")),
    ("ChoiceDataGetModeOnInputByString", P::TokOr(&[1, 44, 2], DATA_GET_MODE, "Directly")),
    ("FullTextSearchOnInputByString", P::TokOr(&[1, 44, 1], FULL_TEXT_ON_INPUT, "DontUse")),
    ("DefaultObjectForm", P::Ref(&[1, 22])),
    ("DefaultListForm", P::Ref(&[1, 23])),
    ("DefaultChoiceForm", P::Ref(&[1, 24])),
    ("AuxiliaryObjectForm", P::Ref(&[1, 32])),
    ("AuxiliaryListForm", P::Ref(&[1, 33])),
    ("AuxiliaryChoiceForm", P::Ref(&[1, 34])),
    ("ChoiceHistoryOnInput", P::Tok(&[1, 45], HISTORY_ON_INPUT)),
    ("NumberType", P::Tok(&[1, 16], STRING_OR_NUMBER)),
    ("NumberLength", P::Tok(&[1, 18], NUM)),
    ("NumberAllowedLength", P::Tok(&[1, 40], ALLOWED_LENGTH)),
    ("CheckUnique", P::Bool(&[1, 19])),
    ("StandardAttributes", P::Custom(|w, x, n| standard(w, x, "BusinessProcess", n, n.at(&[1, 30, 1])))),
    ("Characteristics", P::Custom(|w, x, n| characteristics(w, x, n.at(&[1, 41])))),
    ("Autonumbering", P::Bool(&[1, 20])),
    ("BasedOn", P::Custom(|w, x, n| refs(w, x, "BasedOn", n.at(&[1, 15])))),
    ("NumberPeriodicity", P::Tok(&[1, 17], PERIODICITY)),
    ("Task", P::Ref(&[1, 25])),
    ("CreateTaskInPrivilegedMode", P::Bool(&[1, 31])),
    ("DataLockFields", P::Custom(|w, x, n| fields(w, x, "DataLockFields", n, &[1, 43]))),
    ("DataLockControlMode", P::Tok(&[1, 28], LOCK)),
    ("IncludeHelpInContents", P::Bool(&[1, 26])),
    ("FullTextSearch", P::Tok(&[1, 29], USE)),
    ("ObjectPresentation", P::Ml(&[1, 35])),
    ("ExtendedObjectPresentation", P::Ml(&[1, 36])),
    ("ListPresentation", P::Ml(&[1, 37])),
    ("ExtendedListPresentation", P::Ml(&[1, 38])),
    ("Explanation", P::Ml(&[1, 39])),
    ("DataHistory", P::Tok(&[1, 46], USE)),
    ("UpdateDataHistoryImmediatelyAfterWrite", P::Bool(&[1, 47])),
    ("ExecuteAfterWriteDataHistoryVersionProcessing", P::Bool(&[1, 48])),
];

const TASK: &[(&str, P)] = &[
    ("UseStandardCommands", P::Bool(&[1, 2])),
    ("NumberType", P::Tok(&[1, 18], STRING_OR_NUMBER)),
    ("NumberLength", P::Tok(&[1, 19], NUM)),
    ("NumberAllowedLength", P::Tok(&[1, 43], ALLOWED_LENGTH)),
    ("CheckUnique", P::Bool(&[1, 20])),
    ("Autonumbering", P::Bool(&[1, 21])),
    // БП: 0 — BusinessProcessNumber, УТ: 1 — DontUse
    ("TaskNumberAutoPrefix", P::Tok(&[1, 31], &[("0", "BusinessProcessNumber"), ("1", "DontUse")])),
    ("DescriptionLength", P::Tok(&[1, 22], NUM)),
    ("Addressing", P::Ref(&[1, 25])),
    ("MainAddressingAttribute", P::Ref(&[1, 26])),
    ("CurrentPerformer", P::Ref(&[1, 29])),
    ("BasedOn", P::Custom(|w, x, n| refs(w, x, "BasedOn", n.at(&[1, 30])))),
    ("StandardAttributes", P::Custom(|w, x, n| standard(w, x, "Task", n, n.at(&[1, 34, 1])))),
    ("Characteristics", P::Custom(|w, x, n| characteristics(w, x, n.at(&[1, 44])))),
    // у задачи AsNumber вместо AsCode (TaskMainPresentation 1C:EDT), позиция — по каркасу
    ("DefaultPresentation", P::Tok(&[1, 27], &[("0", "AsNumber"), ("1", "AsDescription")])),
    ("EditType", P::Tok(&[1, 23], EDIT_TYPE)),
    ("InputByString", P::Custom(|w, x, n| fields(w, x, "InputByString", n, &[1, 28]))),
    ("SearchStringModeOnInputByString", P::TokOr(&[1, 47, 0], SEARCH_MODE, "Begin")),
    ("FullTextSearchOnInputByString", P::TokOr(&[1, 47, 1], FULL_TEXT_ON_INPUT, "DontUse")),
    ("ChoiceDataGetModeOnInputByString", P::TokOr(&[1, 47, 2], DATA_GET_MODE, "Directly")),
    ("CreateOnInput", P::Tok(&[1, 45], CREATE_ON_INPUT)),
    ("DefaultObjectForm", P::Ref(&[1, 15])),
    ("DefaultListForm", P::Ref(&[1, 16])),
    ("DefaultChoiceForm", P::Ref(&[1, 17])),
    ("AuxiliaryObjectForm", P::Ref(&[1, 35])),
    ("AuxiliaryListForm", P::Ref(&[1, 36])),
    ("AuxiliaryChoiceForm", P::Ref(&[1, 37])),
    ("ChoiceHistoryOnInput", P::Tok(&[1, 48], HISTORY_ON_INPUT)),
    ("IncludeHelpInContents", P::Bool(&[1, 24])),
    ("DataLockFields", P::Custom(|w, x, n| fields(w, x, "DataLockFields", n, &[1, 46]))),
    ("DataLockControlMode", P::Tok(&[1, 33], LOCK)),
    ("FullTextSearch", P::Tok(&[1, 32], USE)),
    ("ObjectPresentation", P::Ml(&[1, 38])),
    ("ExtendedObjectPresentation", P::Ml(&[1, 39])),
    ("ListPresentation", P::Ml(&[1, 40])),
    ("ExtendedListPresentation", P::Ml(&[1, 41])),
    ("Explanation", P::Ml(&[1, 42])),
    ("DataHistory", P::Tok(&[1, 49], USE)),
    ("UpdateDataHistoryImmediatelyAfterWrite", P::Bool(&[1, 50])),
    ("ExecuteAfterWriteDataHistoryVersionProcessing", P::Bool(&[1, 51])),
];

/// Список ссылок на объекты метаданных `{0,N,{"#",157fa490-…,{1,<uuid>}}...}` → `<xr:Item xsi:type="xr:MDObjectRef">`.
fn refs(w: &mut W, x: &Index, tag: &str, n: Option<&Node>) {
    md_refs(w, tag, "xr:Item", &[("xsi:type", "xr:MDObjectRef")], x, n);
}

/// Стандартные табличные части `{0,N,<код>,{3,<синоним>,"комментарий",?,<проверка заполнения>,{1,N,<реквизиты>},
/// <подсказка>},...}`: виды субконто плана счетов, ведущие/вытесняющие/базовые виды расчета.
fn standard_tabular_sections(w: &mut W, x: &Index, class: &str, root: &Node, n: Option<&Node>) {
    const NAMES: &[(&str, &str)] = &[("-12", "ExtDimensionTypes"), ("-30", "LeadingCalculationTypes"), ("-20", "DisplacingCalculationTypes"), ("-10", "BaseCalculationTypes")];
    let items = n.map(Node::list).and_then(|l| l.get(2..)).unwrap_or_default();
    // без стандартных табличных частей (нет видов субконто / зависимостей видов расчета) платформа элемент не пишет
    if items.is_empty() {
        return;
    }
    w.open("StandardTabularSections", &[]);
    for c in items.chunks(2) {
        let code = c[0].atom().unwrap_or_default();
        let name = NAMES.iter().find(|(k, _)| *k == code).map_or_else(|| format!("\u{1}{code}"), |(_, nm)| nm.to_string());
        let t = c.get(1).unwrap_or(&c[0]);
        w.open("xr:StandardTabularSection", &[("name", &name)]);
        mlstring(w, "xr:Synonym", t.get(1));
        w.text("xr:Comment", string(t, &[2]));
        mlstring(w, "xr:ToolTip", t.get(6));
        token_or0(w, "xr:FillChecking", t, &[4], CHECK);
        match t.get(5) {
            Some(sa) => super::standard::standard_attributes_tag(w, x, &format!("{class}.StandardTabularSection"), &owner(x, root), sa, "xr:StandardAttributes"),
            None => w.text("xr:StandardAttributes", "\u{1}нет реквизитов"),
        }
        w.end();
    }
    w.end();
}

const ALLOW_DENY: &[(&str, &str)] = &[("0", "Allow"), ("1", "Deny")];
/// Коды — DocumentNumberPeriodicity 1C:EDT (3 Month встречено и в УТ 11.5.27).
const PERIODICITY: &[(&str, &str)] = &[("0", "Nonperiodical"), ("1", "Year"), ("2", "Quarter"), ("3", "Month"), ("4", "Day")];

/// Стандартные реквизиты `{1,N,...}` объекта `root`; `{0}` — элемента нет.
fn standard(w: &mut W, x: &Index, class: &str, root: &Node, n: Option<&Node>) {
    if let Some(n) = n {
        super::standard::standard_attributes(w, x, class, &owner(x, root), n);
    }
}

/// Полное имя объекта по его записи.
pub fn owner(x: &Index, root: &Node) -> String {
    x.name(head(root).and_then(|h| h[1].get(2)).and_then(Node::atom).unwrap_or_default())
}

const FIELD_REF: &str = "60ea359f-3a6e-48bb-8e71-d2a457572918";

/// Поля `{1,{0,N,{"#",60ea359f-…,{-код}|{0,<uuid>}},...}}` (ввод по строке, поля блокировки) → `<xr:Field>`.
fn fields(w: &mut W, x: &Index, tag: &str, root: &Node, path: &[usize]) {
    let owner = owner(x, root);
    let items = root.at(path).and_then(|n| n.get(1)).map(Node::list).and_then(|l| l.get(2..)).unwrap_or_default();
    if items.is_empty() {
        w.text(tag, "");
        return;
    }
    w.open(tag, &[]);
    for f in items {
        let v = match (f.get(1).and_then(Node::atom), f.get(2)) {
            (Some(FIELD_REF), Some(r)) => field(x, &owner, r),
            _ => raw(Some(f)),
        };
        w.text("xr:Field", &v);
    }
    w.end();
}

/// Ссылка на поле `{-код}` (стандартный реквизит `table`), `{0,<uuid>}`, `{-1}` / `{0}` (нет поля) → текст.
fn field(x: &Index, table: &str, r: &Node) -> String {
    match r.list() {
        [Node::Atom(z), Node::Atom(id)] if z == "0" => x.name(id),
        [Node::Atom(c)] if c == "-1" || c == "0" => c.clone(),
        [Node::Atom(c)] if c.starts_with('-') => {
            // стандартный реквизит табличной части владельца назван по коду класса владельца
            let class = table.split('.').next().unwrap_or_default();
            let names = super::standard::NAMES.iter().find(|(k, _)| *k == class).map(|(_, t)| *t).unwrap_or(&[]);
            match names.iter().find(|(k, _)| k == c) {
                Some((_, nm)) => format!("{table}.StandardAttribute.{nm}"),
                None => format!("{table}.StandardAttribute.\u{1}{c}"),
            }
        }
        _ => raw(Some(r)),
    }
}

/// Характеристики `{0,{N,{"#",fe839d42-…,{<версия>,{1,<типы>},{1,<значения>},<поля>...}},...}}`.
/// Поле версии 1 — `{0,<ссылка>}`, версии 4 — `{1,<ссылка>,0}`; у версии 1 нет полей 9–12 (в XML -1).
pub fn characteristics(w: &mut W, x: &Index, n: Option<&Node>) {
    let items = n.and_then(|n| n.get(1)).map(Node::list).and_then(|l| l.get(1..)).unwrap_or_default();
    if items.is_empty() {
        w.text("Characteristics", "");
        return;
    }
    w.open("Characteristics", &[]);
    for c in items {
        let Some(v) = c.get(2) else {
            w.text("xr:Characteristic", &raw(Some(c)));
            continue;
        };
        let types = x.name(atom(v, &[1, 1]));
        let values = x.name(atom(v, &[2, 1]));
        let f = |i: usize, table: &str| v.get(i).and_then(|f| f.get(1)).map_or_else(|| "-1".to_string(), |r| field(x, table, r));
        w.open("xr:Characteristic", &[]);
        w.open("xr:CharacteristicTypes", &[("from", &types)]);
        w.text("xr:KeyField", &f(8, &types));
        w.text("xr:TypesFilterField", &f(6, &types));
        value(w, x, "xr:TypesFilterValue", v.get(7));
        w.text("xr:DataPathField", &f(9, &types));
        w.text("xr:MultipleValuesUseField", &f(10, &types));
        w.end();
        w.open("xr:CharacteristicValues", &[("from", &values)]);
        w.text("xr:ObjectField", &f(3, &values));
        w.text("xr:TypeField", &f(4, &values));
        w.text("xr:ValueField", &f(5, &values));
        w.text("xr:MultipleValuesKeyField", &f(11, &values));
        w.text("xr:MultipleValuesOrderField", &f(12, &values));
        w.end();
        w.end();
    }
    w.end();
}

/// Порядок вложенных элементов в ChildObjects по классам.
fn child_order(class: &str) -> Option<&'static [&'static str]> {
    Some(match class {
        "Catalog" => &["Attribute", "TabularSection", "Form", "Template", "Command"],
        "Document" => &["Attribute", "Form", "TabularSection", "Template", "Command"],
        // место TabularSection у плана счетов — по каркасу (tools/skel/nest.py), Template у задачи — по аналогии
        "ChartOfAccounts" => &["Attribute", "TabularSection", "AccountingFlag", "ExtDimensionAccountingFlag", "Form", "Template", "Command"],
        "Task" => &["Attribute", "TabularSection", "Form", "AddressingAttribute", "Template", "Command"],
        "ChartOfCharacteristicTypes" | "ChartOfCalculationTypes" | "ExchangePlan" | "BusinessProcess" => {
            &["Attribute", "TabularSection", "Form", "Template", "Command"]
        }
        _ => return None,
    })
}

/// ChildObjects объекта этого семейства; `false` — класс не отсюда.
pub fn child_objects(w: &mut W, x: &Index, class: &str, it: &Item, root: &Node) -> bool {
    let Some(order) = child_order(class) else { return false };
    let all: Vec<&Item> = order.iter().flat_map(|k| it.children.iter().filter(move |c| c.kind == *k)).collect();
    if all.is_empty() {
        w.text("ChildObjects", "");
        return true;
    }
    w.open("ChildObjects", &[]);
    for c in all {
        // узел элемента — на k уровней выше его заголовка; формы и макеты — отдельные записи, только имя
        let up = |k: usize| header_path(root, &c.header.id).and_then(|p| root.at(&p[..p.len().saturating_sub(k)]));
        match c.kind {
            "Attribute" | "AccountingFlag" | "ExtDimensionAccountingFlag" | "AddressingAttribute" => attribute(w, x, class, false, c, up(3)),
            "TabularSection" => tabular_section(w, x, class, root, c, up(4)),
            "Command" => super::classes_tools::command(w, x, c, root),
            k => w.text(k, &c.header.name),
        }
    }
    w.end();
    true
}

/// Путь к заголовку элемента с данным uuid.
fn header_path(n: &Node, id: &str) -> Option<Vec<usize>> {
    let l = n.list();
    if l.get(1).and_then(|i| i.get(2)).and_then(Node::atom) == Some(id) && l.get(1).and_then(|i| i.get(1)).and_then(Node::atom) == Some("0") {
        return Some(Vec::new());
    }
    l.iter().enumerate().find_map(|(k, c)| {
        let mut p = header_path(c, id)?;
        p.insert(0, k);
        Some(p)
    })
}

/// Name, Synonym, Comment по заголовку внутри узла элемента.
fn names(w: &mut W, n: &Node, id: &str) {
    match head_of(n, id) {
        Some(h) => name_synonym_comment(w, h),
        None => w.text("Name", "\u{1}нет заголовка"),
    }
}

/// Свойства блока реквизита по порядку выгрузки; FillFromFillingValue и FillValue — только у реквизитов объекта.
const ATTR_PROPS: &[&str] = &[
    "Type", "PasswordMode", "Format", "EditFormat", "ToolTip", "MarkNegatives", "Mask", "MultiLine", "ExtendedEdit", "MinValue", "MaxValue",
    "FillFromFillingValue", "FillValue", "FillChecking", "ChoiceFoldersAndItems", "ChoiceParameterLinks", "ChoiceParameters", "QuickChoice",
    "CreateOnInput", "ChoiceForm", "LinkByType", "ChoiceHistoryOnInput",
];

/// Реквизит `{<версия>,{27,...},<индексирование>,[<использование>],<полнотекстовый поиск>,<история данных>}`.
/// Использование (для групп/элементов) есть только у иерархических классов; нет позиции истории данных — Use.
/// Признак учета: `{6,{27,...},<история данных>}`; реквизит адресации: `{4,{27,...},<индексирование>,<измерение>,
/// <полнотекстовый поиск>,<история данных>}`.
fn attribute(w: &mut W, x: &Index, class: &str, in_ts: bool, c: &Item, n: Option<&Node>) {
    w.open(c.kind, &[("uuid", &c.header.id)]);
    w.open("Properties", &[]);
    let Some(n) = n else {
        w.text("Name", "\u{1}нет записи реквизита");
        w.end();
        w.end();
        return;
    };
    names(w, n, &c.header.id);
    let block = n.get(1).unwrap_or(n);
    for tag in ATTR_PROPS {
        if in_ts && matches!(*tag, "FillFromFillingValue" | "FillValue") {
            continue;
        }
        super::attr::prop(w, x, block, tag);
    }
    let history = |w: &mut W, i: usize| match n.get(i) {
        Some(_) => token(w, "DataHistory", n, &[i], USE),
        None => w.text("DataHistory", "Use"),
    };
    match c.kind {
        "AccountingFlag" | "ExtDimensionAccountingFlag" => {
            history(w, 2);
            w.end();
            w.end();
            return;
        }
        "AddressingAttribute" => {
            token_or0(w, "Indexing", n, &[2], INDEXING);
            reference(w, "AddressingDimension", x, n, &[3]);
            token_or0(w, "FullTextSearch", n, &[4], USE);
            history(w, 5);
            w.end();
            w.end();
            return;
        }
        _ => {}
    }
    let with_use = !in_ts && matches!(class, "Catalog" | "ChartOfCharacteristicTypes");
    let (fts, dh) = if with_use { (4, 5) } else { (3, 4) };
    let use_ = |w: &mut W| token_or0(w, "Use", n, &[3], ITEM_USE);
    if with_use && class == "Catalog" {
        use_(w);
    }
    token_or0(w, "Indexing", n, &[2], INDEXING);
    if with_use && class != "Catalog" {
        use_(w);
    }
    token_or0(w, "FullTextSearch", n, &[fts], USE);
    history(w, dh);
    w.end();
    w.end();
}

/// Табличная часть `{{<0..2>,{11,<TypeId>,<ValueId>,<TypeId строки>,<ValueId строки>,{0,<заголовок>},
/// <проверка заполнения>,<стандартные реквизиты>,<подсказка>},[<использование>],[<длина номера строки>]},<?>,{<реквизиты>}}`.
fn tabular_section(w: &mut W, x: &Index, class: &str, root: &Node, c: &Item, n: Option<&Node>) {
    w.open("TabularSection", &[("uuid", &c.header.id)]);
    let Some(n) = n else {
        w.text("Properties", "\u{1}нет записи табличной части");
        w.end();
        return;
    };
    let t = n.at(&[0, 1]).unwrap_or(n);
    let owner = it_owner(x, &c.header.id);
    w.open("InternalInfo", &[]);
    for (k, (suffix, category)) in [("TabularSection", "TabularSection"), ("TabularSectionRow", "TabularSectionRow")].iter().enumerate() {
        w.open("xr:GeneratedType", &[("name", &format!("{class}{suffix}.{owner}.{}", c.header.name)), ("category", category)]);
        w.text("xr:TypeId", atom(t, &[1 + 2 * k]));
        w.text("xr:ValueId", atom(t, &[2 + 2 * k]));
        w.end();
    }
    w.end();
    w.open("Properties", &[]);
    names(w, n, &c.header.id);
    mlstring(w, "ToolTip", t.get(8));
    token_or0(w, "FillChecking", t, &[6], CHECK);
    standard(w, x, "TabularSection", root, t.at(&[7, 1]));
    let with_use = matches!(class, "Catalog" | "ChartOfCharacteristicTypes");
    if with_use {
        token_or0(w, "Use", n, &[0, 2], ITEM_USE);
    }
    // последний элемент заголовка (версия 1 у документа, 2 у справочника; в БП 3.0.171 его нет — 5, в УТ встречено 9)
    let lnl = if with_use { 3 } else { 2 };
    w.text("LineNumberLength", n.at(&[0, lnl]).and_then(Node::atom).unwrap_or("5"));
    w.end();
    let attrs: Vec<&Item> = c.children.iter().filter(|a| a.kind == "Attribute").collect();
    if attrs.is_empty() {
        w.text("ChildObjects", "");
    } else {
        w.open("ChildObjects", &[]);
        for a in attrs {
            let p = header_path(n, &a.header.id);
            let node = p.as_ref().and_then(|p| n.at(&p[..p.len().saturating_sub(3)]));
            attribute(w, x, class, true, a, node);
        }
        w.end();
    }
    w.end();
}

/// Имя объекта-владельца по полному имени элемента (`Catalog.Валюты.TabularSection.Т` → `Валюты`).
fn it_owner(x: &Index, id: &str) -> String {
    x.name(id).split('.').nth(1).unwrap_or_default().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn xml(f: impl FnOnce(&mut W)) -> String {
        let mut w = W::metadata();
        f(&mut w);
        let s = String::from_utf8(w.finish()[3..].to_vec()).unwrap();
        let l: Vec<&str> = s.split("\r\n").collect();
        l[2..l.len() - 1].iter().map(|s| s.trim_start_matches('\t')).collect::<Vec<_>>().join("|")
    }

    fn node(t: &str) -> Node {
        crate::brace::Doc::parse(t).unwrap().roots.remove(0)
    }

    fn item(kind: &'static str, id: &str) -> Item {
        let header = crate::meta::Header { id: id.into(), name: "Р".into(), ..Default::default() };
        Item { kind, header, files: vec![], children: vec![] }
    }

    const A: &str = "00000000-0000-0000-0000-00000000000a";
    const B: &str = "00000000-0000-0000-0000-00000000000b";
    const Z: &str = "00000000-0000-0000-0000-000000000000";

    #[test]
    fn input_fields() {
        let mut x = Index { full: HashMap::new(), types: HashMap::new() };
        x.full.insert(A.into(), "Catalog.С".into());
        x.full.insert(B.into(), "Catalog.С.Attribute.ИНН".into());
        let f = |r: &str| format!("{{\"#\",60ea359f-3a6e-48bb-8e71-d2a457572918,{r}}}");
        let root = node(&format!("{{1,{{56,{{0,{{3,{{1,0,{A}}},\"С\",{{0}},\"\",0}}}},{{1,{{0,2,{},{}}}}},{{1,{{0,0}}}}}}}}", f("{-3}"), f(&format!("{{0,{B}}}"))));
        assert_eq!(
            xml(|w| fields(w, &x, "InputByString", &root, &[1, 2])),
            "<InputByString>|<xr:Field>Catalog.С.StandardAttribute.Description</xr:Field>|<xr:Field>Catalog.С.Attribute.ИНН</xr:Field>|</InputByString>"
        );
        assert_eq!(xml(|w| fields(w, &x, "DataLockFields", &root, &[1, 3])), "<DataLockFields/>");
    }

    /// Ссылка на отсутствующий объект: в составе ФО пропускается, в списке MDListType — uuid;
    /// PredefinedDataName справочника до версии 53 — первым; создание при вводе версии 47 — по быстрому выбору;
    /// Неопределено ChoiceHistoryOnInput стандартного реквизита — Auto.
    #[test]
    fn old_and_dangling() {
        let mut x = Index { full: HashMap::new(), types: HashMap::new() };
        x.full.insert(A.into(), "Catalog.С".into());
        let r = |id: &str| format!("{{\"#\",3ea29ea5-66f6-4e3b-8595-d8940db766a2,{{0,{{\"#\",157fa490-4ce9-11d4-9415-008048da11f9,{{1,{id}}}}}}}}}");
        let fo = |refs: &[&str]| node(&format!("{{1,{{0,{{0}},{{0}},{{0,{}{}}}}}}}", refs.len(), refs.iter().map(|id| format!(",{}", r(id))).collect::<String>()));
        let content = |n: &Node| xml(|w| super::super::classes::fo_content(w, &x, n));
        assert_eq!(content(&fo(&[A, B])), "<Content>|<xr:Object>Catalog.С</xr:Object>|</Content>");
        assert_eq!(content(&fo(&[B])), "<Content/>");
        let n = node(&format!("{{0,2,{},{}}}", r(A), r(B)));
        let t = "<xr:Item xsi:type=\"xr:MDObjectRef\">";
        assert_eq!(xml(|w| md_refs(w, "Owners", "xr:Item", &[("xsi:type", "xr:MDObjectRef")], &x, Some(&n))), format!("<Owners>|{t}Catalog.С</xr:Item>|{t}{B}</xr:Item>|</Owners>"));

        let catalog = |ver: &str| {
            let mut l = vec!["0".to_string(); 53];
            l[0] = ver.into();
            l[41] = "1".into();
            l[45] = "{0,{1,2,{-10},510405d3-2a0c-4fea-960a-7fee59b32f9b,{13,0},{-2},510405d3-2a0c-4fea-960a-7fee59b32f9b,{13,1,7ba608f2-e654-42a3-8885-334fe88ca910,{\"U\"}}}}".into();
            let mut root = node(&format!("{{1,{{{}}}}}", l.join(",")));
            upgrade_old_catalog(&mut root);
            root
        };
        let std_names = |root: &Node| {
            let s = xml(|w| super::super::standard::standard_attributes(w, &x, "Catalog", "Catalog.С", root.at(&[1, 45, 1]).unwrap()));
            s.split('|').filter_map(|l| l.strip_prefix("<xr:StandardAttribute name=\"")?.strip_suffix("\">").map(String::from)).collect::<Vec<_>>()
        };
        let old = catalog("47");
        assert_eq!(std_names(&old), ["PredefinedDataName", "Predefined", "Code"]);
        assert_eq!(old.at(&[1, 53]).and_then(Node::atom), Some("1"));
        let new = catalog("53");
        assert_eq!(std_names(&new), ["Predefined", "Code"]);
        assert_eq!(new.at(&[1, 53]), None);
        let s = xml(|w| super::super::standard::standard_attributes(w, &x, "Catalog", "Catalog.С", new.at(&[1, 45, 1]).unwrap()));
        assert_eq!(s.matches("<xr:ChoiceHistoryOnInput>Auto</xr:ChoiceHistoryOnInput>").count(), 2);
    }

    /// Свойства после ChoiceHistoryOnInput у реквизита класса `class` с хвостом обертки `rest`.
    fn attr_tail(class: &str, rest: &str) -> String {
        let x = Index { full: HashMap::new(), types: HashMap::new() };
        let n = node(&format!("{{5,{{27,{{2,{{3,{{1,0,{A}}},\"Р\",{{0}},\"\",0}},{{\"Pattern\",{{\"B\"}}}}}}}},{rest}}}"));
        let s = xml(|w| attribute(w, &x, class, false, &item("Attribute", A), Some(&n)));
        s.split('|').skip_while(|t| !t.starts_with("<ChoiceHistoryOnInput")).skip(1).collect::<Vec<_>>().join("|")
    }

    #[test]
    fn attribute_tail_by_class_and_version() {
        // справочник: Use перед Indexing, история данных пятой
        assert_eq!(attr_tail("Catalog", "2,1,0,0"), "<Use>ForFolder</Use>|<Indexing>IndexWithAdditionalOrder</Indexing>|<FullTextSearch>DontUse</FullTextSearch>|<DataHistory>DontUse</DataHistory>|</Properties>|</Attribute>");
        // документ без позиции истории данных (старая версия) — Use
        assert_eq!(attr_tail("Document", "1,1"), "<Indexing>Index</Indexing>|<FullTextSearch>Use</FullTextSearch>|<DataHistory>Use</DataHistory>|</Properties>|</Attribute>");
    }

    #[test]
    fn line_number_length_from_header() {
        let x = Index { full: HashMap::new(), types: HashMap::new() };
        let lnl = |class: &str, head: &str| {
            let t = format!("{{11,{Z},{Z},{Z},{Z},{{0,{{3,{{1,0,{A}}},\"Т\",{{0}},\"\",0}}}},0,{{0}},{{0}}}}");
            let n = node(&format!("{{{{{}}},1,{{888744e1-b616-11d4-9436-004095e12fc7,0}}}}", head.replace('T', &t)));
            let s = xml(|w| tabular_section(w, &x, class, &node("{1}"), &item("TabularSection", A), Some(&n)));
            s.split('|').find(|t| t.starts_with("<LineNumberLength>")).unwrap_or_default().to_string()
        };
        assert_eq!(lnl("Document", "1,T"), "<LineNumberLength>5</LineNumberLength>");
        assert_eq!(lnl("Document", "2,T,9"), "<LineNumberLength>9</LineNumberLength>");
        assert_eq!(lnl("Catalog", "2,T,0,9"), "<LineNumberLength>9</LineNumberLength>");
    }
}
