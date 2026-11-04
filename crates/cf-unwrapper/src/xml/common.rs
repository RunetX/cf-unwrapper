//! Общие части описаний объектов: заголовок свойств, многоязычные строки, описания типов, ссылки, GeneratedType.

use super::w::W;
use crate::brace::{self, is_guid, Node};
use crate::meta::{Config, Item};
use std::collections::HashMap;

pub const ZERO: &str = "00000000-0000-0000-0000-000000000000";

/// Генерируемые типы класса в порядке выгрузки: (префикс имени, category, позиция TypeId в корне[1]; ValueId — следом).
/// shortcut: типы табличных частей (`CatalogTabularSection…`) не собраны — понадобятся вместе с табличными частями.
pub const GENERATED: &[(&str, &[(&str, &str, usize)])] = &[
    ("Catalog", &[("CatalogObject", "Object", 1), ("CatalogRef", "Ref", 3), ("CatalogSelection", "Selection", 5), ("CatalogList", "List", 7), ("CatalogManager", "Manager", 34)]),
    ("Document", &[("DocumentObject", "Object", 1), ("DocumentRef", "Ref", 3), ("DocumentSelection", "Selection", 5), ("DocumentList", "List", 7), ("DocumentManager", "Manager", 26)]),
    ("Enum", &[("EnumRef", "Ref", 1), ("EnumManager", "Manager", 3), ("EnumList", "List", 7)]),
    ("Constant", &[("ConstantManager", "Manager", 2), ("ConstantValueManager", "ValueManager", 4), ("ConstantValueKey", "ValueKey", 13)]),
    ("DefinedType", &[("DefinedType", "DefinedType", 1)]),
    ("SettingsStorage", &[("SettingsStorageManager", "Manager", 2)]),
    ("DataProcessor", &[("DataProcessorObject", "Object", 1), ("DataProcessorManager", "Manager", 7)]),
    ("Report", &[("ReportObject", "Object", 1), ("ReportManager", "Manager", 12)]),
    ("FilterCriterion", &[("FilterCriterionManager", "Manager", 1), ("FilterCriterionList", "List", 3)]),
    ("DocumentJournal", &[("DocumentJournalSelection", "Selection", 10), ("DocumentJournalList", "List", 1), ("DocumentJournalManager", "Manager", 8)]),
    ("Sequence", &[("SequenceRecord", "Record", 1), ("SequenceManager", "Manager", 3), ("SequenceRecordSet", "RecordSet", 5)]),
    ("WSReference", &[("WSReferenceManager", "Manager", 3)]),
    // единственная категория — 5 (Manager): integ.dll FUN_1800e0590; позиции — запись FUN_1800f2110
    ("IntegrationService", &[("IntegrationServiceManager", "Manager", 2)]),
    ("InformationRegister", &[("InformationRegisterRecord", "Record", 1), ("InformationRegisterManager", "Manager", 3), ("InformationRegisterSelection", "Selection", 5), ("InformationRegisterList", "List", 7), ("InformationRegisterRecordSet", "RecordSet", 9), ("InformationRegisterRecordKey", "RecordKey", 11), ("InformationRegisterRecordManager", "RecordManager", 13)]),
    ("AccumulationRegister", &[("AccumulationRegisterRecord", "Record", 1), ("AccumulationRegisterManager", "Manager", 3), ("AccumulationRegisterSelection", "Selection", 5), ("AccumulationRegisterList", "List", 7), ("AccumulationRegisterRecordSet", "RecordSet", 9), ("AccumulationRegisterRecordKey", "RecordKey", 11)]),
    ("CalculationRegister", &[("CalculationRegisterRecord", "Record", 1), ("CalculationRegisterManager", "Manager", 3), ("CalculationRegisterSelection", "Selection", 5), ("CalculationRegisterList", "List", 7), ("CalculationRegisterRecordSet", "RecordSet", 9), ("CalculationRegisterRecordKey", "RecordKey", 11), ("RecalculationsManager", "Recalcs", 13)]),
    ("AccountingRegister", &[("AccountingRegisterRecord", "Record", 2), ("AccountingRegisterExtDimensions", "ExtDimensions", 4), ("AccountingRegisterRecordSet", "RecordSet", 6), ("AccountingRegisterRecordKey", "RecordKey", 8), ("AccountingRegisterSelection", "Selection", 10), ("AccountingRegisterList", "List", 12), ("AccountingRegisterManager", "Manager", 14)]),
    ("ExchangePlan", &[("ExchangePlanObject", "Object", 1), ("ExchangePlanRef", "Ref", 3), ("ExchangePlanSelection", "Selection", 5), ("ExchangePlanList", "List", 7), ("ExchangePlanManager", "Manager", 9)]),
    ("ChartOfAccounts", &[("ChartOfAccountsObject", "Object", 1), ("ChartOfAccountsRef", "Ref", 3), ("ChartOfAccountsSelection", "Selection", 5), ("ChartOfAccountsList", "List", 7), ("ChartOfAccountsManager", "Manager", 9), ("ChartOfAccountsExtDimensionTypes", "ExtDimensionTypes", 11), ("ChartOfAccountsExtDimensionTypesRow", "ExtDimensionTypesRow", 13)]),
    ("ChartOfCharacteristicTypes", &[("ChartOfCharacteristicTypesObject", "Object", 1), ("ChartOfCharacteristicTypesRef", "Ref", 3), ("ChartOfCharacteristicTypesSelection", "Selection", 5), ("ChartOfCharacteristicTypesList", "List", 7), ("Characteristic", "Characteristic", 9), ("ChartOfCharacteristicTypesManager", "Manager", 11)]),
    ("ChartOfCalculationTypes", &[("ChartOfCalculationTypesObject", "Object", 2), ("ChartOfCalculationTypesRef", "Ref", 4), ("ChartOfCalculationTypesSelection", "Selection", 6), ("ChartOfCalculationTypesList", "List", 8), ("ChartOfCalculationTypesManager", "Manager", 10), ("DisplacingCalculationTypes", "DisplacingCalculationTypes", 12), ("DisplacingCalculationTypesRow", "DisplacingCalculationTypesRow", 14), ("BaseCalculationTypes", "BaseCalculationTypes", 16), ("BaseCalculationTypesRow", "BaseCalculationTypesRow", 18), ("LeadingCalculationTypes", "LeadingCalculationTypes", 20), ("LeadingCalculationTypesRow", "LeadingCalculationTypesRow", 22)]),
    ("BusinessProcess", &[("BusinessProcessObject", "Object", 3), ("BusinessProcessRef", "Ref", 5), ("BusinessProcessSelection", "Selection", 7), ("BusinessProcessList", "List", 9), ("BusinessProcessManager", "Manager", 11), ("BusinessProcessRoutePointRef", "RoutePointRef", 13)]),
    ("Task", &[("TaskObject", "Object", 3), ("TaskRef", "Ref", 5), ("TaskSelection", "Selection", 7), ("TaskList", "List", 9), ("TaskManager", "Manager", 11)]),
];

/// Типы платформы в описаниях типов `{"#",<guid>}`: (guid, элемент, значение).
pub const PLATFORM_TYPES: &[(&str, &str, &str)] = &[
    ("e199ca70-93cf-46ce-a54b-6edc88c3a296", "v8:Type", "v8:ValueStorage"),
    ("fc01b5df-97fe-449b-83d4-218a090e681e", "v8:Type", "v8:UUID"),
    ("3ee983d7-ace7-40f9-bb7e-2e916fcddd56", "v8:Type", "v8:FixedStructure"),
    ("4500381b-db30-4a10-9db4-990038032acf", "v8:Type", "v8:FixedArray"),
    ("220455ea-6c85-4513-996f-bbe79ed07774", "v8:Type", "v8:FixedMap"),
    ("4772b3b4-f4a3-49c0-a1a5-8cb5961511a3", "v8:Type", "v8:ValueListType"),
    ("e603c0f2-92fb-4d47-8f38-a44a381cf235", "v8:Type", "v8:ValueTree"),
    ("acf6192e-81ca-46ef-93a6-5a6968b78663", "v8:Type", "v8:ValueTable"),
    ("2fdc88ec-7c9b-43cd-8ba5-873f043bdd88", "v8:Type", "v8:StandardPeriod"),
    ("0387f3a2-7df5-4804-948b-4580a51e4a15", "v8:Type", "v8:StandardBeginningDate"),
    ("0dda99d9-ae9f-43d2-b7ac-44f3fb0d4059", "v8:Type", "cfg:ReportBuilder"),
    ("0a52f9de-73ea-4507-81e8-66217bead73a", "v8:TypeSet", "cfg:ExchangePlanRef"),
    ("38bfd075-3e63-4aaa-a93e-94521380d579", "v8:TypeSet", "cfg:DocumentRef"),
    ("e61ef7b8-f3e1-4f4b-8ac7-676e90524997", "v8:TypeSet", "cfg:CatalogRef"),
    ("214fa4d8-6ba4-4748-a5e1-6332b5887780", "v8:TypeSet", "cfg:BusinessProcessRef"),
    ("6291e9b3-8df5-44e1-b6b2-d9fe008016c0", "v8:TypeSet", "cfg:TaskRef"),
    ("99892482-ed55-4fb5-a7f7-20888820a758", "v8:TypeSet", "cfg:ChartOfCharacteristicTypesRef"),
    ("593cd424-0877-470d-91f9-b90a982059b4", "v8:TypeSet", "cfg:ChartOfCalculationTypesRef"),
    ("ac606d60-0209-4159-8e4c-794bc091ce38", "v8:TypeSet", "cfg:ChartOfAccountsRef"),
    ("280f5f0e-9c8a-49cc-bf6d-4d296cc17a63", "v8:TypeSet", "cfg:AnyIBRef"),
    ("474c3bf6-08b5-4ddc-a2ad-989cedf11583", "v8:TypeSet", "cfg:EnumRef"),
    ("dcfc3784-a14f-4786-ac7b-c82db5ba275f", "v8:Type", "cfg:ConstantsSet"),
];

/// Тип платформы по GUID: (`v8:Type` или `v8:TypeSet`, значение с префиксом `v8:`/`cfg:`).
pub fn platform_type(id: &str) -> Option<(&'static str, &'static str)> {
    PLATFORM_TYPES.iter().find(|(g, _, _)| *g == id).map(|(_, el, v)| (*el, *v))
}

/// Порядок наборов «любая ссылка» в выгрузке (собран по описаниям типов БП 3.0.171; AnyIBRef встречен только после простых типов).
/// EnumRef — после DocumentRef и перед ChartOfCalculationTypesRef (БП, УТ; ERP 2.5.22: DocumentRef, EnumRef, …).
pub const GENERIC_SETS: &[&str] = &[
    "cfg:ExchangePlanRef", "cfg:BusinessProcessRef", "cfg:DocumentRef", "cfg:EnumRef", "cfg:ChartOfCalculationTypesRef", "cfg:TaskRef",
    "cfg:ChartOfCharacteristicTypesRef", "cfg:ChartOfAccountsRef", "cfg:CatalogRef", "cfg:AnyIBRef",
];

/// «Любые» типы источников подписок: GUID класса (объекты, наборы записей) или свой GUID (менеджеры).
/// Порядок списка — порядок выгрузки (собран по подпискам БП 3.0.171; GUID, по которому сортирует платформа, неизвестен).
pub const ANY_TYPES: &[(&str, &str, &str)] = &[
    ("fcd3404e-1523-48ce-9bc0-ecdb822684a1", "v8:TypeSet", "BusinessProcessObject"),
    ("13134201-f60b-11d5-a3c7-0050bae0a776", "v8:TypeSet", "InformationRegisterRecordSet"),
    ("f2de87a8-64e5-45eb-a22d-b3aedab050e7", "v8:TypeSet", "CalculationRegisterRecordSet"),
    ("0dee6ca3-50a1-4f94-8c34-e70eeb802d81", "v8:Type", "AccumulationRegisterManager"),
    ("30b100d6-b29f-47ac-aec7-cb8ca8a54767", "v8:TypeSet", "ChartOfCalculationTypesObject"),
    ("2deed9b8-0056-4ffe-a473-c20a6c32a0bc", "v8:TypeSet", "AccountingRegisterRecordSet"),
    ("1aa09f48-f6d5-4999-a7f5-02a15794c795", "v8:Type", "InformationRegisterManager"),
    ("238e7e88-3c5f-48b2-8a3b-81ebbecb20ed", "v8:TypeSet", "ChartOfAccountsObject"),
    ("b64d9a40-1642-11d6-a3c7-0050bae0a776", "v8:TypeSet", "AccumulationRegisterRecordSet"),
    ("2066866d-9d38-47fe-a272-3cd416eb9c85", "v8:Type", "ChartOfAccountsManager"),
    ("82a1b659-b220-4d94-a9bd-14d757b95a48", "v8:TypeSet", "ChartOfCharacteristicTypesObject"),
    ("bc587f20-35d9-11d6-a3c7-0050bae0a776", "v8:TypeSet", "SequenceRecordSet"),
    ("26dd1dee-252a-4942-b4b5-62ea44ed8030", "v8:Type", "DocumentManager"),
    ("0195e80c-b157-11d4-9435-004095e12fc7", "v8:TypeSet", "ConstantValueManager"),
    ("857c4a91-e5f4-4fac-86ec-787626f1c108", "v8:TypeSet", "ExchangePlanObject"),
    ("274bf899-db0e-4df6-8ab5-67bf6371ec0b", "v8:TypeSet", "RecalculationRecordSet"),
    ("2d0abc8e-dede-4184-afd7-7ae8da588d47", "v8:Type", "CalculationRegisterManager"),
    ("cf4abea6-37b2-11d4-940f-008048da11f9", "v8:TypeSet", "CatalogObject"),
    ("38f1038d-8b0b-438b-bfbe-830a60a1153a", "v8:Type", "BusinessProcessManager"),
    ("3e63355c-1378-4953-be9b-1deb5fb6bec5", "v8:TypeSet", "TaskObject"),
    ("3ab47eda-6a5c-4590-9b08-0e633aa2f376", "v8:Type", "AccountingRegisterManager"),
    ("061d872a-5787-460e-95ac-ed74ea3a3e84", "v8:TypeSet", "DocumentObject"),
    ("3caa9852-d921-4ba6-8259-e1f1fc0954ad", "v8:Type", "ExchangePlanManager"),
    ("3eab4ff4-f2d1-4c96-831c-04711b093999", "v8:Type", "ChartOfCalculationTypesManager"),
    ("5e268c17-8035-458f-8041-daf9b15d05c9", "v8:Type", "TaskManager"),
    ("6ba5b296-4aa6-4099-9087-821f2d1252d1", "v8:Type", "DataProcessorManager"),
    ("7612de75-8b10-466a-b235-68572c605d92", "v8:Type", "ChartOfCharacteristicTypesManager"),
    ("82faabf3-7f9b-4b2e-b499-98876415f270", "v8:Type", "CatalogManager"),
    ("83f90ce5-0770-47df-bc7c-5156720f86df", "v8:Type", "FilterCriterionManager"),
    ("92e7f73f-bd66-4d9e-bc43-bae2acfadfd5", "v8:Type", "DocumentJournalManager"),
    ("af88b718-20c1-40e7-8a15-d0da56108fe6", "v8:Type", "EnumManager"),
    ("b63dd3d0-6d1b-4c66-bdb2-b065e282a777", "v8:Type", "ReportManager"),
];

/// Типы платформы со своим пространством имен: (GUID, пространство, префикс — `None`: `d<уровень>p1`, имя).
pub const NS_TYPES: &[(&str, &str, Option<&str>, &str)] = &[
    ("cab0d12b-3c88-4993-8edc-8c3827cadc7d", "http://v8.1c.ru/8.1/data-composition-system/settings", Some("dcsset"), "SettingsComposer"),
    ("3543ef08-3316-4f7e-9447-0cd0a1cbf1d5", "http://v8.1c.ru/8.2/data/chart", None, "Chart"),
    // ERP 2.5.22 ПомощникИсправленияОстатковТоваровОрганизаций: префикс mxl на самом элементе
    ("e603103e-a318-4edc-a014-b1c6cf94d49f", "http://v8.1c.ru/8.2/data/spreadsheet", Some("mxl"), "SpreadsheetDocument"),
];

/// Имена по uuid, собираются один раз на выгрузку.
pub struct Index {
    /// uuid объекта или элемента → `Catalog.Валюты`, `Catalog.Валюты.Attribute.Курс`, `Enum.X.EnumValue.Y`.
    pub full: HashMap<String, String>,
    /// TypeId генерируемого типа → `CatalogRef.Валюты`.
    pub types: HashMap<String, String>,
}

impl Index {
    pub fn new(cfg: &Config) -> Index {
        let mut x = Index { full: HashMap::new(), types: HashMap::new() };
        fn walk(it: &Item, path: &str, x: &mut Index) {
            for ch in &it.children {
                let p = format!("{path}.{}.{}", ch.kind, ch.header.name);
                walk(ch, &p, x);
                x.full.insert(ch.header.id.clone(), p);
            }
        }
        // Разбор записей (предопределенные, описания для типов) — по потокам; вставка — подряд, в прежнем порядке.
        let items: Vec<(&'static str, &Item)> = cfg.groups.iter().flat_map(|g| g.items.iter().map(move |it| (g.class, it))).collect();
        let parsed = crate::par::map(&items, || (), |_, &(class, it)| {
            let path = format!("{class}.{}", it.header.name);
            let (mut pre, mut types) = (HashMap::new(), Vec::new());
            // предопределенные: у справочника запись <id>.1c, у планов — с ролью Predefined
            let own = format!("{}.1c", it.header.id);
            for e in it.files.iter().filter(|(_, r)| *r == "Predefined").map(|(e, _)| e.as_str()).chain([own.as_str()]) {
                if let Some(n) = cfg.entry_text(e).and_then(|t| brace::Doc::parse(&t).ok()) {
                    n.roots.iter().for_each(|r| predefined(r, &path, &mut pre));
                }
            }
            let gen = GENERATED.iter().find(|(c, _)| *c == class).map(|(_, t)| *t).unwrap_or(&[]);
            if let Some(mut root) = (!gen.is_empty()).then(|| object_brace(cfg, it)).flatten() {
                // регистр бухгалтерии `{21,…}` — позиции как у описания (ERP 2.5.22: Международный)
                if class == "AccountingRegister" {
                    super::classes_regs::upgrade_21(&mut root);
                }
                for (prefix, _, pos) in gen {
                    if let Some(id) = root.at(&[1, *pos]).and_then(Node::atom).filter(|a| is_guid(a)) {
                        types.push((id.to_string(), format!("{prefix}.{}", it.header.name)));
                    }
                }
            }
            (path, pre, types)
        });
        for ((_, it), (path, pre, types)) in items.into_iter().zip(parsed) {
            walk(it, &path, &mut x);
            x.full.extend(pre);
            x.full.insert(it.header.id.clone(), path);
            x.types.extend(types);
        }
        if let Some(body) = external_body(cfg) {
            let (class, name) = (cfg.root.kind, &cfg.root.header.name);
            let path = format!("{class}.{name}");
            walk(&cfg.root, &path, &mut x);
            x.full.insert(cfg.root.header.id.clone(), path);
            if let Some(t) = body.at(&[1, 1]).and_then(Node::atom).filter(|a| is_guid(a)) {
                x.types.insert(t.to_string(), format!("{class}Object.{name}"));
            }
        }
        x
    }

    /// Полное имя по uuid; нулевой — пусто; неизвестный — сам uuid (виден в сверке).
    pub fn name(&self, id: &str) -> String {
        if id == ZERO || id.is_empty() {
            return String::new();
        }
        self.full.get(id).cloned().unwrap_or_else(|| id.to_string())
    }
}

const PREDEFINED_REF: &str = "ae135932-4f94-44df-92c1-c91f15a92848";

/// Строки предопределенных `{2,<n>,<колонок>,{"#",ae135932-…,{1,<id>}},…}`: имя — первая строковая колонка.
/// Ключ — `<владелец>.<id>`: один id бывает у предопределенных разных справочников (копии объектов).
pub fn predefined(n: &Node, owner: &str, out: &mut HashMap<String, String>) {
    let l = n.list();
    if l.first().and_then(Node::atom) == Some("2") && l.get(3).and_then(|r| r.get(1)).and_then(Node::atom) == Some(PREDEFINED_REF) {
        let id = l[3].get(2).and_then(|r| r.get(1)).and_then(Node::atom).unwrap_or(ZERO);
        let name = l[4..].iter().find(|c| c.get(0).and_then(Node::str) == Some("S")).and_then(|c| c.get(1)).and_then(Node::str);
        if let (true, Some(name)) = (id != ZERO && is_guid(id), name) {
            out.insert(format!("{owner}.{id}"), format!("{owner}.{name}"));
        }
    }
    l.iter().for_each(|c| predefined(c, owner, out));
}

/// Типы значения реквизита (измерения, ресурса) с uuid `id` в записи владельца: блок `{…,<заголовок>,<описание типов>}`
/// → `CatalogRef.Валюты`, `xs:string`, `v8:ValueStorage`; неизвестные GUID пропускаются.
pub fn value_types(x: &Index, root: &Node, id: &str) -> Vec<String> {
    fn find<'a>(n: &'a Node, id: &str) -> Option<&'a Node> {
        let l = n.list();
        let h = l.get(1).and_then(|h| h.get(1));
        if h.and_then(|i| i.get(2)).and_then(Node::atom) == Some(id) && h.and_then(|i| i.get(1)).and_then(Node::atom) == Some("0") {
            return l.get(2);
        }
        l.iter().find_map(|c| find(c, id))
    }
    let Some(t) = find(root, id) else { return Vec::new() };
    t.list()
        .get(1..)
        .unwrap_or_default()
        .iter()
        .filter_map(|it| match it.get(0).and_then(|n| n.atom().or(n.str()))? {
            "B" => Some("xs:boolean".into()),
            "S" => Some("xs:string".into()),
            "N" => Some("xs:decimal".into()),
            "D" => Some("xs:dateTime".into()),
            "#" => {
                let g = it.get(1).and_then(Node::atom)?;
                platform_type(g).map(|(_, v)| v.to_string()).or_else(|| x.types.get(g).cloned())
            }
            _ => None,
        })
        .collect()
}

/// Корень скобочной записи объекта (без потерь: позиции элементов как в записи).
pub fn object_brace(cfg: &Config, it: &Item) -> Option<Node> {
    let t = cfg.entry_text(&it.header.id)?;
    brace::Doc::parse(&t).ok()?.roots.into_iter().next()
}

/// Тело внешней обработки — `{1,{4,TypeId,ValueId,{заголовок},…},N,коллекции…}` внутри записи корня
/// `{1,{<uuid>},1,{<класс>,<тело>}}`; у конфигурации — `None`.
pub fn external_body(cfg: &Config) -> Option<Node> {
    if cfg.root.kind == "Configuration" {
        return None;
    }
    let t = cfg.entry_text(&cfg.root_id)?;
    brace::Doc::parse(&t).ok()?.roots.into_iter().next()?.at(&[3, 1]).cloned()
}

/// Заголовок `{N,{1,0,<uuid>},"Имя",{синоним},"комментарий",...}` — первый в обходе.
pub use crate::meta::header_node as head;

/// Заголовок элемента с данным uuid (значения перечисления, реквизиты внутри записи владельца).
pub fn head_of<'a>(n: &'a Node, id: &str) -> Option<&'a [Node]> {
    let l = n.list();
    if l.get(1).and_then(|i| i.get(2)).and_then(Node::atom) == Some(id) && l.get(1).and_then(|i| i.get(1)).and_then(Node::atom) == Some("0") {
        return Some(l);
    }
    l.iter().find_map(|c| head_of(c, id))
}

/// Name, Synonym, Comment из заголовка.
pub fn name_synonym_comment(w: &mut W, h: &[Node]) {
    w.text("Name", h.get(2).and_then(Node::str).unwrap_or_default());
    mlstring(w, "Synonym", h.get(3));
    w.text("Comment", h.get(4).and_then(Node::str).unwrap_or_default());
}

/// Многоязычная строка `{N,"ru","текст",...}` → `<v8:item>` по языкам; `{0}` → `<tag/>`.
pub fn mlstring(w: &mut W, tag: &str, n: Option<&Node>) {
    let l = n.map(Node::list).unwrap_or_default();
    let pairs: Vec<(&str, &str)> = l.get(1..).unwrap_or_default().chunks(2).filter_map(|c| Some((c.first()?.str()?, c.get(1)?.str()?))).collect();
    if pairs.is_empty() {
        w.text(tag, "");
        return;
    }
    w.open(tag, &[]);
    for (lang, text) in pairs {
        w.open("v8:item", &[]);
        w.text("v8:lang", lang);
        w.text("v8:content", text);
        w.end();
    }
    w.end();
}

/// InternalInfo с GeneratedType класса; без таблицы — ничего.
pub fn internal_info(w: &mut W, class: &str, id: &str, name: &str, root: &Node) {
    let Some((_, gen)) = GENERATED.iter().find(|(c, _)| *c == class) else { return };
    w.open("InternalInfo", &[]);
    if class == "ExchangePlan" {
        // uuid узла «этот узел» — перед генерируемыми типами
        w.text("xr:ThisNode", atom(root, &[1, 11]));
    }
    for (prefix, category, pos) in *gen {
        let (mut t, mut v) = (root.at(&[1, *pos]).and_then(Node::atom).unwrap_or("").to_string(), root.at(&[1, pos + 1]).and_then(Node::atom).unwrap_or("").to_string());
        // у констант старого формата (версия записи < 15) ValueKey в записи нет — платформа выводит его из uuid константы
        // (basic.dll 0x1806fdc16, xml-export.md «ValueKey старых констант»)
        if class == "Constant" && *category == "ValueKey" && root.at(&[1, 0]).and_then(Node::atom).and_then(|a| a.parse::<u32>().ok()).is_some_and(|ver| ver < 15) && is_guid(id) {
            (t, v) = (uuid5(id, b"typeId"), uuid5(id, b"valueId"));
        }
        let (t, v) = (t.as_str(), v.as_str());
        if !is_guid(t) || !is_guid(v) {
            w.text("xr:GeneratedType", &format!("\u{1}{prefix}.{name}: нет TypeId/ValueId в записи"));
            continue;
        }
        w.open("xr:GeneratedType", &[("name", &format!("{prefix}.{name}")), ("category", category)]);
        w.text("xr:TypeId", t);
        w.text("xr:ValueId", v);
        w.end();
    }
    w.end();
}

/// Атом по пути; нет — пусто.
pub fn atom<'a>(n: &'a Node, path: &[usize]) -> &'a str {
    n.at(path).and_then(Node::atom).unwrap_or_default()
}

pub fn string<'a>(n: &'a Node, path: &[usize]) -> &'a str {
    n.at(path).and_then(Node::str).unwrap_or_default()
}

/// 0/1 → false/true; другое значение пишется как есть, чтобы сверка его показала.
pub fn bool_at(w: &mut W, tag: &str, n: &Node, path: &[usize]) {
    token(w, tag, n, path, &[("0", "false"), ("1", "true")]);
}

/// Число из скобок → значение перечисления XML по таблице; неизвестное пишется как есть.
pub fn token(w: &mut W, tag: &str, n: &Node, path: &[usize], map: &[(&str, &str)]) {
    let a = atom(n, path);
    w.text(tag, map.iter().find(|(k, _)| *k == a).map_or(a, |(_, v)| v));
}

/// Как [`token`], но позиции нет (запись старого формата короче) — значение для «0».
pub fn token_or0(w: &mut W, tag: &str, n: &Node, path: &[usize], map: &[(&str, &str)]) {
    if n.at(path).is_none() {
        w.text(tag, map.iter().find(|(k, _)| *k == "0").map_or("\u{1}", |(_, v)| v));
    } else {
        token(w, tag, n, path, map);
    }
}

/// Скобки как есть с пометкой `?`: формат значения не восстановлен, сверка покажет.
pub fn raw(n: Option<&Node>) -> String {
    format!("\u{1}{}", n.and_then(|n| brace::write(n).ok()).unwrap_or_default())
}

/// Ссылка на объект по uuid (`CommonForm.Имя`), нулевая — пустой элемент.
pub fn reference(w: &mut W, tag: &str, x: &Index, n: &Node, path: &[usize]) {
    w.text(tag, &x.name(atom(n, path)));
}

const DESIGN_TIME_REF: &str = "5c14e26f-099b-4d37-84a6-b433d87400da";

/// Ссылка времени разработки `{"#",5c14e26f-…,{0,<тип>,<значение>}}` → `Catalog.X.Имя` (EmptyRef, `<тип>.<uuid>`);
/// `None` — не она.
pub fn design_time_ref(x: &Index, n: &Node) -> Option<String> {
    let l = n.list();
    if l.first().and_then(Node::str) != Some("#") || l.get(1).and_then(Node::atom) != Some(DESIGN_TIME_REF) {
        return None;
    }
    let r = l.get(2);
    let a = |i: usize| r.and_then(|r| r.get(i)).and_then(Node::atom).unwrap_or_default();
    match (x.types.get(a(1)), a(2)) {
        (Some(t), ZERO) => t.split_once("Ref.").map(|(c, nm)| format!("{c}.{nm}.EmptyRef")),
        // нет ни типа, ни значения
        (None, ZERO) if a(1) == ZERO => Some(String::new()),
        // тип не из этой конфигурации или значение не найдено: платформа пишет <тип>.<uuid> (180 раз в БП 3.0.171)
        (t, id) if is_guid(a(1)) && is_guid(id) => {
            let predefined = t.and_then(|t| t.split_once("Ref.")).and_then(|(c, nm)| x.full.get(&format!("{c}.{nm}.{id}")));
            predefined.or_else(|| x.full.get(id)).cloned().or_else(|| Some(format!("{}.{id}", a(1))))
        }
        _ => None,
    }
}
const FIXED_ARRAY: &str = "4500381b-db30-4a10-9db4-990038032acf";
const ACCOUNT_TYPE: &str = "872f7198-7083-4e3e-b57e-a2a9802c769e";
const TYPE_DESCRIPTION: &str = "f5c65050-3bbb-11d5-b988-0050bae0a95d";

/// Значение `{"U"}` (nil), `{"S",…}`, `{"N",…}`, `{"B",…}`, `{"D",ГГГГММДДччммсс}`, `{"#",5c14e26f-…,{0,<TypeId>,<uuid>}}`
/// (ссылка: пустая — `Catalog.Имя.EmptyRef`); прочее — `?скобки`, чтобы сверка показала.
pub fn value(w: &mut W, x: &Index, tag: &str, n: Option<&Node>) {
    let l = n.map(Node::list).unwrap_or_default();
    let s = |i: usize| l.get(i).and_then(|n| n.str().or(n.atom())).unwrap_or_default();
    let typed = |w: &mut W, t: &str, v: &str| w.text_attrs(tag, &[("xsi:type", t)], v);
    match s(0) {
        "U" => w.nil(tag),
        "S" => typed(w, "xs:string", s(1)),
        "N" => typed(w, "xs:decimal", s(1)),
        "B" => typed(w, "xs:boolean", if s(1) == "1" { "true" } else { "false" }),
        // только цифры: срез по байтам безопасен
        "D" if s(1).len() == 14 && s(1).bytes().all(|b| b.is_ascii_digit()) => {
            let d = s(1);
            typed(w, "xs:dateTime", &format!("{}-{}-{}T{}:{}:{}", &d[..4], &d[4..6], &d[6..8], &d[8..10], &d[10..12], &d[12..]))
        }
        // {"#",4500381b-…,{N,<значение>...}}
        "#" if s(1) == FIXED_ARRAY && l.get(2).is_some() => {
            w.open(tag, &[("xsi:type", "v8:FixedArray")]);
            for v in l[2].list().get(1..).unwrap_or_default() {
                value(w, x, "v8:Value", Some(v));
            }
            w.end();
        }
        // вид счета `{"#",872f7198-…,<номер>}`: в эталоне встречен только 2
        "#" if s(1) == ACCOUNT_TYPE && s(2) == "2" => typed(w, "ent:AccountType", "ActivePassive"),
        // пустое описание типов `{"#",f5c65050-…,{"Pattern"}}`
        "#" if s(1) == TYPE_DESCRIPTION && l.get(2).is_some_and(|p| p.list().len() == 1) => typed(w, "v8:TypeDescription", ""),
        "#" if s(1) == DESIGN_TIME_REF => match n.and_then(|n| design_time_ref(x, n)) {
            Some(v) => typed(w, "xr:DesignTimeRef", &v),
            None => w.text(tag, &raw(n)),
        },
        _ => w.text(tag, &raw(n)),
    }
}

/// Описание типов `{"Pattern",{"B"},{"S",10,1},{"N",15,2,1},{"D","D"},{"#",<guid>}}`.
/// Типы — в порядке записи, затем квалификаторы числа, строки, даты.
pub fn type_desc(w: &mut W, tag: &str, x: &Index, p: &Node) {
    let items = p.list().get(1..).unwrap_or_default();
    if items.is_empty() {
        w.text(tag, "");
        return;
    }
    w.open(tag, &[]);
    let (mut num, mut st, mut date) = (None, None, None);
    // (элемент, значение, место «любого» типа в порядке выгрузки)
    let mut out: Vec<(String, String, Option<usize>)> = Vec::new();
    for it in items {
        let a = |i: usize| it.get(i).and_then(|n| n.atom().or(n.str())).unwrap_or_default();
        let t = |v: &str| ("v8:Type".to_string(), v.to_string(), None);
        out.push(match a(0) {
            "B" => t("xs:boolean"),
            "S" => {
                st = Some(it);
                t("xs:string")
            }
            "N" => {
                num = Some(it);
                t("xs:decimal")
            }
            "D" => {
                date = Some(it);
                t("xs:dateTime")
            }
            "#" => {
                let g = a(1);
                if let Some((_, ns, prefix, name)) = NS_TYPES.iter().find(|(id, ..)| *id == g) {
                    // тип со своим пространством имен: объявление на самом элементе, префикс `d<уровень>p1` — по глубине
                    let p = prefix.map_or_else(|| format!("d{}p1", w.depth() + 1), str::to_string);
                    (format!("v8:Type xmlns:{p}={ns}"), format!("{p}:{name}"), None)
                } else if let Some((el, v)) = platform_type(g) {
                    (el.to_string(), v.to_string(), None)
                } else if let Some((i, (_, el, v))) = ANY_TYPES.iter().enumerate().find(|(_, (id, _, _))| *id == g) {
                    (el.to_string(), format!("cfg:{v}"), Some(i))
                } else if let Some(t) = x.types.get(g) {
                    // определяемый тип и характеристика — наборы типов
                    let el = if t.starts_with("DefinedType.") || t.starts_with("Characteristic.") { "v8:TypeSet" } else { "v8:Type" };
                    (el.to_string(), format!("cfg:{t}"), None)
                } else if crate::brace::is_guid(g) {
                    // Типа нет в конфигурации (удаленный объект) — платформа пишет его GUID на том же месте (БП 3.0.107:
                    // три подписки и параметр команды). shortcut: так же уйдет и тип платформы, которого нет в наших
                    // таблицах, — при расхождении v8:TypeId с эталоном дополнять platform_type.
                    ("v8:TypeId".to_string(), g.to_string(), None)
                } else {
                    t(&format!("\u{1}{g}"))
                }
            }
            other => t(&format!("\u{1}{other}")),
        });
    }
    // «любые» типы в записи идут по GUID класса, в выгрузке — в своем порядке: переставляются на своих же местах
    let slots: Vec<usize> = (0..out.len()).filter(|&i| out[i].2.is_some()).collect();
    let mut any: Vec<_> = slots.iter().map(|&i| out[i].clone()).collect();
    any.sort_by_key(|e| e.2);
    for (&i, e) in slots.iter().zip(any) {
        out[i] = e;
    }
    // «любой» набор (TypeSet) вместе с конкретными типами — после всех v8:Type, как наборы GENERIC_SETS (1 подписка БП:
    // CatalogObject.А, CatalogObject.Б, ExchangePlanObject); «любые» менеджеры (v8:Type) стоят среди конкретных в порядке
    // записи (сверено; наборов вместе с менеджерами в БП нет). Вместе с другими наборами (определяемый тип, GENERIC_SETS) — метка.
    if slots.iter().any(|&i| out[i].0 == "v8:TypeSet") && out.iter().any(|e| e.2.is_none() && e.1.starts_with("cfg:")) {
        if out.iter().any(|e| e.2.is_none() && e.0 == "v8:TypeSet") || slots.iter().any(|&i| out[i].0 != "v8:TypeSet") {
            out.push(("v8:Type".into(), "\u{1}порядок «любых» и конкретных типов не восстановлен".into(), None));
        } else {
            out.sort_by_key(|e| e.0 == "v8:TypeSet");
        }
    }
    // тип удаленного объекта — после прочих типов (БП 3.0.107 подписка ЗапретитьРедактированиеОбщихДанныхОбъект…)
    out.sort_by_key(|e| e.0 == "v8:TypeId");
    // наборы «любая ссылка класса» — после остальных типов, в порядке GENERIC_SETS
    out.sort_by_key(|(el, v, _)| if el == "v8:TypeSet" { GENERIC_SETS.iter().position(|g| g == v).map_or(0, |p| p + 1) } else { 0 });
    for (el, v, _) in &out {
        match el.split_once(' ').and_then(|(tag, d)| Some((tag, d.split_once('=')?))) {
            Some((tag, (k, ns))) => w.text_attrs(tag, &[(k, ns)], v),
            None => w.text(el, v),
        }
    }
    let q = |it: &Node, i: usize| it.get(i).and_then(Node::atom).unwrap_or("0").to_string();
    if let Some(it) = num {
        w.open("v8:NumberQualifiers", &[]);
        w.text("v8:Digits", &q(it, 1));
        w.text("v8:FractionDigits", &q(it, 2));
        w.text("v8:AllowedSign", if q(it, 3) == "1" { "Nonnegative" } else { "Any" });
        w.end();
    }
    if let Some(it) = st {
        w.open("v8:StringQualifiers", &[]);
        w.text("v8:Length", &q(it, 1));
        // {"S"} без квалификаторов — переменная длина
        w.text("v8:AllowedLength", if it.get(2).and_then(Node::atom) == Some("0") { "Fixed" } else { "Variable" });
        w.end();
    }
    if let Some(it) = date {
        w.open("v8:DateQualifiers", &[]);
        let f = match it.get(1).and_then(Node::str) {
            Some("D") => "Date",
            Some("T") => "Time",
            _ => "DateTime",
        };
        w.text("v8:DateFractions", f);
        w.end();
    }
    w.end();
}

const MD_REF: &str = "157fa490-4ce9-11d4-9415-008048da11f9";

/// Ссылка на объект метаданных `{"#",157fa490-…,{1,<uuid>}}` (может быть вложена глубже) → uuid.
pub fn md_ref(n: &Node) -> Option<&str> {
    let l = n.list();
    if l.get(1).and_then(Node::atom) == Some(MD_REF) {
        return l.get(2)?.get(1)?.atom();
    }
    l.iter().find_map(md_ref)
}

/// uuid версии 5 (RFC 4122: SHA-1 от 16 байт пространства в сетевом порядке и имени); `ns` — проверенный is_guid.
pub fn uuid5(ns: &str, name: &[u8]) -> String {
    let hex: String = ns.chars().filter(|c| *c != '-').collect();
    let mut m: Vec<u8> = (0..16).map(|i| u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16).unwrap_or(0)).collect();
    m.extend_from_slice(name);
    let mut d = sha1(&m);
    d[6] = d[6] & 0x0f | 0x50;
    d[8] = d[8] & 0x3f | 0x80;
    let h: String = d[..16].iter().map(|b| format!("{b:02x}")).collect();
    format!("{}-{}-{}-{}-{}", &h[..8], &h[8..12], &h[12..16], &h[16..20], &h[20..])
}

pub(crate) fn sha1(data: &[u8]) -> [u8; 20] {
    let mut h: [u32; 5] = [0x67452301, 0xefcdab89, 0x98badcfe, 0x10325476, 0xc3d2e1f0];
    let mut m = data.to_vec();
    m.push(0x80);
    while m.len() % 64 != 56 {
        m.push(0);
    }
    m.extend_from_slice(&((data.len() as u64) * 8).to_be_bytes());
    for c in m.chunks(64) {
        let mut w = [0u32; 80];
        for i in 0..80 {
            w[i] = if i < 16 { u32::from_be_bytes([c[i * 4], c[i * 4 + 1], c[i * 4 + 2], c[i * 4 + 3]]) } else { (w[i - 3] ^ w[i - 8] ^ w[i - 14] ^ w[i - 16]).rotate_left(1) };
        }
        let [mut a, mut b, mut cc, mut d, mut e] = h;
        for (i, wi) in w.iter().enumerate() {
            let (f, k) = match i {
                0..=19 => ((b & cc) | (!b & d), 0x5a827999),
                20..=39 => (b ^ cc ^ d, 0x6ed9eba1),
                40..=59 => ((b & cc) | (b & d) | (cc & d), 0x8f1bbcdc),
                _ => (b ^ cc ^ d, 0xca62c1d6),
            };
            let t = a.rotate_left(5).wrapping_add(f).wrapping_add(e).wrapping_add(k).wrapping_add(*wi);
            (e, d, cc, b, a) = (d, cc, b.rotate_left(30), a, t);
        }
        for (x, y) in h.iter_mut().zip([a, b, cc, d, e]) {
            *x = x.wrapping_add(y);
        }
    }
    let mut out = [0u8; 20];
    for (i, x) in h.iter().enumerate() {
        out[i * 4..i * 4 + 4].copy_from_slice(&x.to_be_bytes());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn xml(f: impl FnOnce(&mut W)) -> String {
        let mut w = W::metadata();
        f(&mut w);
        let b = w.finish();
        let s = String::from_utf8(b[3..].to_vec()).unwrap();
        // без заголовка и закрывающего MetaDataObject
        let l: Vec<&str> = s.split("\r\n").collect();
        l[2..l.len() - 1].iter().map(|s| s.trim_start_matches('\t')).collect::<Vec<_>>().join("|")
    }

    fn node(t: &str) -> Node {
        brace::Doc::parse(t).unwrap().roots.remove(0)
    }

    #[test]
    fn uuid5_value_key() {
        // Synthetic namespace; expected UUIDv5 values independently generated with Python uuid.
        assert_eq!(uuid5("11111111-1111-4111-8111-111111111111", b"typeId"), "87b802d1-9772-5e94-ad90-2c1510b8a44d");
        assert_eq!(uuid5("11111111-1111-4111-8111-111111111111", b"valueId"), "b2e58f97-8ddd-57aa-9c93-881c02a5a022");
    }

    #[test]
    fn types() {
        let mut x = Index { full: HashMap::new(), types: HashMap::new() };
        x.types.insert("11111111-1111-1111-1111-111111111111".into(), "CatalogRef.Валюты".into());
        x.types.insert("22222222-2222-2222-2222-222222222222".into(), "DefinedType.Деньги".into());
        let p = node("{\"Pattern\",{\"B\"},{\"S\",255,1},{\"N\",11,4,1},{\"D\",\"D\"},{\"#\",11111111-1111-1111-1111-111111111111},{\"#\",22222222-2222-2222-2222-222222222222},{\"#\",e199ca70-93cf-46ce-a54b-6edc88c3a296}}");
        assert_eq!(
            xml(|w| type_desc(w, "Type", &x, &p)),
            "<Type>|<v8:Type>xs:boolean</v8:Type>|<v8:Type>xs:string</v8:Type>|<v8:Type>xs:decimal</v8:Type>|<v8:Type>xs:dateTime</v8:Type>|<v8:Type>cfg:CatalogRef.Валюты</v8:Type>|<v8:TypeSet>cfg:DefinedType.Деньги</v8:TypeSet>|<v8:Type>v8:ValueStorage</v8:Type>\
|<v8:NumberQualifiers>|<v8:Digits>11</v8:Digits>|<v8:FractionDigits>4</v8:FractionDigits>|<v8:AllowedSign>Nonnegative</v8:AllowedSign>|</v8:NumberQualifiers>\
|<v8:StringQualifiers>|<v8:Length>255</v8:Length>|<v8:AllowedLength>Variable</v8:AllowedLength>|</v8:StringQualifiers>\
|<v8:DateQualifiers>|<v8:DateFractions>Date</v8:DateFractions>|</v8:DateQualifiers>|</Type>"
        );
        let a = node("{27,{2,{12,{1,0,44444444-4444-4444-4444-444444444444},\"Валюта\"},{\"Pattern\",{\"S\",10},{\"#\",11111111-1111-1111-1111-111111111111},{\"#\",e199ca70-93cf-46ce-a54b-6edc88c3a296}}}}");
        assert_eq!(value_types(&x, &a, "44444444-4444-4444-4444-444444444444"), ["xs:string", "CatalogRef.Валюты", "v8:ValueStorage"]);
        // «любой» набор подписки между конкретными типами — после них; вместе с «любым» менеджером — метка
        x.types.insert("00000000-0000-0000-0000-000000000002".into(), "CatalogObject.Б".into());
        x.types.insert("00000000-0000-0000-0000-000000000001".into(), "CatalogObject.А".into());
        let p = node("{\"Pattern\",{\"#\",00000000-0000-0000-0000-000000000001},{\"#\",857c4a91-e5f4-4fac-86ec-787626f1c108},{\"#\",00000000-0000-0000-0000-000000000002}}");
        assert_eq!(xml(|w| type_desc(w, "S", &x, &p)), "<S>|<v8:Type>cfg:CatalogObject.А</v8:Type>|<v8:Type>cfg:CatalogObject.Б</v8:Type>|<v8:TypeSet>cfg:ExchangePlanObject</v8:TypeSet>|</S>");
        let p = node("{\"Pattern\",{\"#\",00000000-0000-0000-0000-000000000001},{\"#\",857c4a91-e5f4-4fac-86ec-787626f1c108},{\"#\",82faabf3-7f9b-4b2e-b499-98876415f270}}");
        assert!(xml(|w| type_desc(w, "S", &x, &p)).contains(">\u{1}порядок"));
        let p = node("{\"Pattern\",{\"S\"},{\"D\"}}");
        let s = xml(|w| type_desc(w, "Type", &x, &p));
        assert!(s.contains("<v8:Length>0</v8:Length>|<v8:AllowedLength>Variable") && s.contains("DateTime"), "{s}");
        let p = node("{\"Pattern\",{\"S\",36,0},{\"#\",33333333-3333-3333-3333-333333333333}}");
        let s = xml(|w| type_desc(w, "Type", &x, &p));
        assert!(s.contains("Fixed") && s.contains("<v8:TypeId>33333333-3333-3333-3333-333333333333</v8:TypeId>"), "{s}");
    }

    #[test]
    fn head_and_strings() {
        let n = node("{1,{12,{2,{1,0,0aa19482-2d94-47dc-bcaf-5e3f2c360e13},\"Валюты\",{2,\"ru\",\"Валюты\",\"en\",\"Currencies\"},\"\",0}}}");
        let h = head(&n).unwrap();
        assert_eq!(
            xml(|w| name_synonym_comment(w, h)),
            "<Name>Валюты</Name>|<Synonym>|<v8:item>|<v8:lang>ru</v8:lang>|<v8:content>Валюты</v8:content>|</v8:item>|<v8:item>|<v8:lang>en</v8:lang>|<v8:content>Currencies</v8:content>|</v8:item>|</Synonym>|<Comment/>"
        );
        assert_eq!(xml(|w| mlstring(w, "ToolTip", Some(&node("{0}")))), "<ToolTip/>");
        let n = node("{0,1,7}");
        assert_eq!(xml(|w| bool_at(w, "A", &n, &[1])), "<A>true</A>");
        assert_eq!(xml(|w| bool_at(w, "A", &n, &[2])), "<A>7</A>");
    }

    #[test]
    fn values() {
        let mut x = Index { full: HashMap::new(), types: HashMap::new() };
        x.types.insert("11111111-1111-1111-1111-111111111111".into(), "CatalogRef.Валюты".into());
        x.full.insert("Catalog.Валюты.22222222-2222-2222-2222-222222222222".into(), "Catalog.Валюты.Рубль".into());
        let v = |t: &str| xml(|w| value(w, &x, "V", Some(&node(t))));
        assert_eq!(v("{\"U\"}"), "<V xsi:nil=\"true\"/>");
        assert_eq!(v("{\"S\",\"\"}"), "<V xsi:type=\"xs:string\"/>");
        assert_eq!(v("{\"B\",0}"), "<V xsi:type=\"xs:boolean\">false</V>");
        assert_eq!(v("{\"D\",00010101000000}"), "<V xsi:type=\"xs:dateTime\">0001-01-01T00:00:00</V>");
        assert_eq!(
            v("{\"#\",5c14e26f-099b-4d37-84a6-b433d87400da,{0,11111111-1111-1111-1111-111111111111,00000000-0000-0000-0000-000000000000}}"),
            "<V xsi:type=\"xr:DesignTimeRef\">Catalog.Валюты.EmptyRef</V>"
        );
        assert!(v("{\"Z\",1}").starts_with("<V>\u{1}{"));
        // 14 байт не-ASCII: не дата, без паники на срезе
        assert!(v("{\"D\",\"Дата123456\"}").starts_with("<V>\u{1}{"));
        // предопределенный — по ключу <владелец>.<id>; неизвестный элемент или тип — <тип>.<id>
        let r = |t: &str, id: &str| v(&format!("{{\"#\",5c14e26f-099b-4d37-84a6-b433d87400da,{{0,{t},{id}}}}}"));
        assert_eq!(r("11111111-1111-1111-1111-111111111111", "22222222-2222-2222-2222-222222222222"), "<V xsi:type=\"xr:DesignTimeRef\">Catalog.Валюты.Рубль</V>");
        assert_eq!(
            r("33333333-3333-3333-3333-333333333333", "00000000-0000-0000-0000-000000000000"),
            "<V xsi:type=\"xr:DesignTimeRef\">33333333-3333-3333-3333-333333333333.00000000-0000-0000-0000-000000000000</V>"
        );
        assert_eq!(v("{\"#\",4500381b-db30-4a10-9db4-990038032acf,{1,{\"B\",1}}}"), "<V xsi:type=\"v8:FixedArray\">|<v8:Value xsi:type=\"xs:boolean\">true</v8:Value>|</V>");
    }

    #[test]
    fn generic_type_sets_last() {
        let x = Index { full: HashMap::new(), types: HashMap::new() };
        // AnyIBRef, потом простые: в выгрузке набор «любая ссылка» — после них
        let p = node("{\"Pattern\",{\"#\",280f5f0e-9c8a-49cc-bf6d-4d296cc17a63},{\"#\",e61ef7b8-f3e1-4f4b-8ac7-676e90524997},{\"B\"}}");
        assert_eq!(xml(|w| type_desc(w, "Type", &x, &p)), "<Type>|<v8:Type>xs:boolean</v8:Type>|<v8:TypeSet>cfg:CatalogRef</v8:TypeSet>|<v8:TypeSet>cfg:AnyIBRef</v8:TypeSet>|</Type>");
    }
}
