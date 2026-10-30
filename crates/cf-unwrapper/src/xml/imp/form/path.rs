//! Путь к данным элемента формы `Объект.Товары.Номенклатура` → `{N,{<id реквизита>},<часть>…}` (обратное `data_path`
//! выгрузки): колонка реквизита формы — `{<id>}`, дополнительная колонка — `{<id>,5bdad865…}`, реквизит метаданных —
//! `{0,<uuid>}`, стандартный реквизит — `{<код>}` по классу, поле типа платформы — `{<код>}`, итог колонки —
//! `{101000000,<uuid>}`, поле регистра бухгалтерии по счету — `{2|3,<uuid>}`, субконто — `{<номер>,<guid Дт|Кт>}`;
//! текущие данные таблицы `Items.<таблица>.CurrentData.…` — `{N,{<id таблицы>,02023637…},<части от пути таблицы>…}`.
//! Индекс строки `[N]` — `{N,e67e2953…}`, неразрешенный путь выгрузки `1/0:<uuid>` — части как есть.
//! Незнакомое (поля динамического списка, недоступные поля `~…`, неоднозначные имена) — отказ.

use super::*;

const ADDED_COLUMN: &str = "5bdad865-f2c5-434b-8041-ba4aad3b6687";
/// Индекс строки `[N]` — `{N,ROW_INDEX}`.
const ROW_INDEX: &str = "e67e2953-cebe-4d97-bb93-12b17e6384f8";
/// Компоновщик настроек отчета (стандартный реквизит объекта отчета).
const REPORT_COMPOSER: &str = "b9754f01-29e9-11d6-a3c7-0050bae0a776";
/// Субконто Дт/Кт записи регистра бухгалтерии (`{<номер с нуля>,<guid>}`).
const EXT_DIM_DR: &str = "1ab44b24-3315-40a9-b495-f1f1227ac205";
const EXT_DIM_CR: &str = "f77758c9-9fcd-490f-9bbd-1e446541f536";
/// Субконто регистра без корреспонденции.
const EXT_DIM: &str = "91162600-3161-4326-89a0-4a7cecd5092a";
/// Итог колонки табличной части `Total<колонка>`.
const TOTAL: &str = "101000000";

/// Где продолжается путь.
enum Ctx<'a> {
    /// реквизит или колонка формы: свои колонки и типы
    Form { cols: &'a [Col], types: &'a [String] },
    /// объект метаданных, табличная часть, набор записей, набор констант: полное имя и класс
    Meta { full: String, class: String },
    /// тип платформы с полями по номерам (`v8:ValueListType`, компоновщик настроек…)
    Plat(&'static str),
    /// значение без известного состава (дальше путь не продолжается)
    Leaf,
}

/// Поля типов платформы: (класс, имя, код, класс следующей части; пусто — дальше не продолжается). Коды — как читает
/// выгрузка (`composer_field`).
const PLAT: &[(&str, &str, i64, &str)] = &[
    ("dcsset:SettingsComposer", "Settings", 0, "Settings"),
    ("dcsset:SettingsComposer", "UserSettings", 1, "UserSettings"),
    ("dcsset:SettingsComposer", "FixedSettings", 2, "Settings"),
    ("SettingsComposer", "Settings", 0, "Settings"),
    ("SettingsComposer", "FixedSettings", 2, "Settings"),
    ("SettingsComposer", "UserSettings", 1, "UserSettings"),
    ("Settings", "DataParameters", 0, "DataParameters"),
    ("Settings", "Filter", 1, "Filter"),
    ("Settings", "Selection", 2, "Selection"),
    ("Settings", "Order", 3, "Order"),
    ("Settings", "ConditionalAppearance", 4, "ConditionalAppearance"),
    ("Settings", "ItemDataParameters", 10007, "DataParameters"),
    ("Settings", "ItemFilter", 10008, "Filter"),
    ("Settings", "ItemGroupFields", 10009, "GroupFields"),
    ("Settings", "ItemSelection", 10010, "Selection"),
    ("Settings", "ItemOrder", 10011, "Order"),
    ("Settings", "ItemConditionalAppearance", 10012, "ConditionalAppearance"),
    ("Settings", "ItemOutputParameters", 10013, "DataParameters"),
    ("Settings", "ItemUserFields", 10014, "UserFields"),
    ("Filter", "Use", 10000, ""),
    ("Filter", "LeftValuePicture", 10001, ""),
    ("Filter", "LeftValue", 10002, ""),
    ("Filter", "ComparisonType", 10003, ""),
    ("Filter", "RightValuePicture", 10004, ""),
    ("Filter", "RightValue", 10005, ""),
    ("Filter", "Date", 10006, ""),
    ("Filter", "GroupType", 10007, ""),
    ("Filter", "Application", 10008, ""),
    ("Filter", "ViewMode", 10009, ""),
    ("Filter", "Presentation", 10010, ""),
    ("Filter", "FilterAvailableFields", 0, "FilterAvailableFields"),
    ("FilterAvailableFields", "FieldPicture", 10000, ""),
    ("FilterAvailableFields", "Title", 10001, ""),
    ("ConditionalAppearance", "Use", 10000, ""),
    ("ConditionalAppearance", "Appearance", 10001, "Appearance"),
    ("Appearance", "Use", 10000, ""),
    ("Appearance", "Parameter", 10001, ""),
    ("Appearance", "ValuePicture", 10002, ""),
    ("Appearance", "Value", 10003, ""),
    ("ConditionalAppearance", "Filter", 10002, "Filter"),
    ("ConditionalAppearance", "Fields", 10003, "Fields"),
    ("ConditionalAppearance", "Presentation", 10004, ""),
    ("ConditionalAppearance", "UseArea", 10005, ""),
    ("Fields", "Use", 10000, ""),
    ("Fields", "FieldPicture", 10001, ""),
    ("Fields", "Field", 10002, ""),
    ("Order", "Use", 10000, ""),
    ("Order", "Field", 10002, ""),
    ("Order", "OrderType", 10003, ""),
    ("Order", "OrderAvailableFields", 0, ""),
    ("Selection", "SelectionAvailableFields", 0, ""),
    ("GroupFields", "GroupFieldsAvailableFields", 0, ""),
    ("UserSettings", "Use", 10000, ""),
    ("UserSettings", "SettingPicture", 10001, ""),
    ("UserSettings", "Setting", 10002, ""),
    ("UserSettings", "ComparisonType", 10003, ""),
    ("UserSettings", "ValuePicture", 10004, ""),
    ("UserSettings", "Value", 10005, ""),
    ("UserSettings", "EditInReportForm", 10006, ""),
    ("UserSettings", "Filter", 10007, "Filter"),
    ("UserSettings", "Order", 10008, "Order"),
    ("UserSettings", "ConditionalAppearance", 10010, "ConditionalAppearance"),
    ("UserSettings", "Structure", 10011, ""),
    ("v8:ValueListType", "ValueType", -1, ""),
    ("v8:ValueListType", "Value", 0, ""),
    ("v8:ValueListType", "Presentation", 1, ""),
    ("v8:ValueListType", "Check", 2, ""),
    ("v8:ValueListType", "Picture", 3, ""),
    // прочие номера свойств компоновки — getPropInfo контекстов dcscore, как у выгрузки (`form::composer_field`)
    ("Settings", "OutputParameters", 5, "DataParameters"),
    ("Settings", "UserFields", 6, "UserFields"),
    ("Settings", "Use", 10000, ""),
    ("Settings", "ReportStructure", 10001, ""),
    ("Settings", "HasSelection", 10002, ""),
    ("Settings", "HasFilter", 10003, ""),
    ("Settings", "HasOrder", 10004, ""),
    ("Settings", "HasConditionalAppearance", 10005, ""),
    ("Settings", "HasOutputParameters", 10006, ""),
    ("Settings", "ReportStructurePicture", 10015, ""),
    ("DataParameters", "Use", 10000, ""),
    ("DataParameters", "Parameter", 10001, ""),
    ("DataParameters", "ValuePicture", 10002, ""),
    ("DataParameters", "Value", 10003, ""),
    ("DataParameters", "Date", 10004, ""),
    ("DataParameters", "StartDate", 10005, ""),
    ("DataParameters", "EndDate", 10006, ""),
    ("Filter", "UsedCount", 1, ""),
    ("Selection", "UsedCount", 1, ""),
    ("Order", "UsedCount", 1, ""),
    ("GroupFields", "UsedCount", 1, ""),
    ("ConditionalAppearance", "UsedCount", 0, ""),
    ("UserSettings", "UsedCount", 0, ""),
    ("UserSettings", "Selection", 10009, "Selection"),
    ("Selection", "Use", 10000, ""),
    ("Selection", "TitlePicture", 10001, ""),
    ("Selection", "Title", 10002, ""),
    ("Selection", "Placement", 10003, ""),
    ("Selection", "FieldPicture", 10004, ""),
    ("Selection", "Field", 10005, ""),
    ("Order", "FieldPicture", 10001, ""),
    ("GroupFields", "Use", 10000, ""),
    ("GroupFields", "FieldPicture", 10001, ""),
    ("GroupFields", "Field", 10002, ""),
    ("GroupFields", "GroupType", 10003, ""),
    ("GroupFields", "AdditionType", 10004, ""),
    ("GroupFields", "BeginOfPeriodPicture", 10005, ""),
    ("GroupFields", "BeginOfPeriod", 10006, ""),
    ("GroupFields", "EndOfPeriodPicture", 10007, ""),
    ("GroupFields", "EndOfPeriod", 10008, ""),
    ("Fields", "AppearanceFieldsAvailableFields", 0, ""),
    ("UserFields", "Title", 10000, ""),
    ("v8:StandardPeriod", "Variant", 0, ""),
    ("v8:StandardBeginningDate", "Variant", 0, ""),
    ("v8:StandardBeginningDate", "Date", 1, ""),
    ("v8:StandardPeriod", "StartDate", 1, ""),
    ("CalculationTypes", "CalculationType", -101, ""),
    ("v8:StandardPeriod", "EndDate", 2, ""),
    ("d5p1:GanttChart", "Point", 0, ""),
    ("d5p1:GanttChart", "Text", 1, ""),
];

fn plat(class: &str, n: &str) -> Option<(Node, Ctx<'static>)> {
    let (_, _, code, next) = PLAT.iter().find(|(c, m, ..)| *c == class && *m == n)?;
    Some((l(vec![an(code)]), if next.is_empty() { Ctx::Leaf } else { Ctx::Plat(next) }))
}

/// Класс метаданных и полное имя объекта по типу `cfg:CatalogObject.Банки` → (`Catalog`, `Catalog.Банки`).
fn meta_of(t: &str) -> Option<(String, String)> {
    let t = t.strip_prefix("cfg:")?;
    let (head, name) = t.split_once('.')?;
    for suffix in ["RecordManager", "RecordSet", "Object", "Ref"] {
        if let Some(c) = head.strip_suffix(suffix) {
            if !c.is_empty() {
                return Some((c.to_string(), format!("{c}.{name}")));
            }
        }
    }
    None
}

/// Контекст значения по типам (один тип; несколько — дальше путь не продолжается).
fn typed(types: &[String]) -> Ctx<'static> {
    let [t] = types else { return Ctx::Leaf };
    if let Some(c) = PLAT.iter().map(|r| r.0).find(|c| c.contains(':') && c == t) {
        return Ctx::Plat(c);
    }
    // любой отчет (общая форма отчета): стандартные реквизиты отчета (компоновщик настроек)
    if t == "cfg:ReportObject" {
        return Ctx::Meta { full: "Report".into(), class: "Report".into() };
    }
    if t == "cfg:ConstantsSet" {
        return Ctx::Meta { full: "Constant".into(), class: "ConstantsSet".into() };
    }
    match meta_of(t) {
        Some((class, full)) => Ctx::Meta { full, class },
        None => Ctx::Leaf,
    }
}

/// Стандартный реквизит → код по классу (сверка с записями загрузки БП).
fn std_code(class: &str, name: &str) -> Option<i64> {
    const T: &[(&str, &str, i64)] = &[
        ("Catalog", "Code", -2),
        ("Catalog", "Description", -3),
        ("Catalog", "Parent", -4),
        ("Catalog", "Owner", -5),
        ("Catalog", "Ref", -8),
        ("Catalog", "PredefinedDataName", -13),
        // форма Конфигуратора 8.5 с «Расположения» (layout.cf 28.10.2026)
        ("Catalog", "DeletionMark", -7),
        ("Catalog", "Predefined", -10),
        ("Document", "Number", -2),
        ("Document", "Date", -3),
        ("Document", "Ref", -5),
        // как у выгрузки (`std_field`)
        ("Document", "DeletionMark", -4),
        ("Document", "Posted", -7),
        ("Document", "RegisterRecords", -8),
        ("ChartOfCalculationTypes", "LeadingCalculationTypes", -30),
        ("ChartOfCalculationTypes", "BaseCalculationTypes", -10),
        ("ChartOfCalculationTypes", "DisplacingCalculationTypes", -20),
        ("BusinessProcess", "Number", -2),
        ("BusinessProcess", "Date", -3),
        ("Task", "Number", -2),
        ("Task", "Date", -3),
        ("Task", "Ref", -5),
        ("Task", "BusinessProcess", -7),
        ("Task", "RoutePoint", -8),
        ("Task", "Description", -9),
        ("Task", "Executed", -10),
        ("ChartOfAccounts", "Parent", -6),
        ("ChartOfAccounts", "Code", -7),
        ("ChartOfAccounts", "Description", -8),
        ("ChartOfAccounts", "Type", -10),
        ("ChartOfAccounts", "OffBalance", -11),
        ("ChartOfAccounts", "ExtDimensionTypes", -12),
        ("ChartOfCharacteristicTypes", "Code", -8),
        ("ChartOfCharacteristicTypes", "Description", -9),
        ("ChartOfCharacteristicTypes", "ValueType", -11),
        ("ChartOfCalculationTypes", "Code", -2),
        ("ChartOfCalculationTypes", "Description", -3),
        ("ExchangePlan", "Code", -2),
        ("ExchangePlan", "Description", -3),
        ("ExchangePlan", "Ref", -6),
        ("ExchangePlan", "SentNo", -9),
        ("ExchangePlan", "ReceivedNo", -10),
        ("TabularSection", "LineNumber", -2),
        ("InformationRegister", "Period", -2),
        // русские имена — в полях условного оформления (компоновка)
        ("InformationRegister", "Период", -2),
        ("InformationRegister", "Активность", -5),
        ("InformationRegister", "LineNumber", -4),
        ("InformationRegister", "Active", -5),
        ("AccumulationRegister", "LineNumber", -4),
        ("AccumulationRegister", "Active", -5),
        ("AccountingRegister", "Period", -2),
        ("AccountingRegister", "LineNumber", -4),
        ("AccountingRegister", "AccountDr", -6),
        ("AccountingRegister", "AccountCr", -7),
        // русские имена (accnt.dll: пары 0x180469e78 СчетДт/AccountDr, 0x180469ea0 СчетКт/AccountCr) — в полях условного
        // оформления (ПроверкаАналитикиСчетовУчетаКапитала БП)
        ("AccountingRegister", "СчетДт", -6),
        ("AccountingRegister", "СчетКт", -7),
        ("AccountingRegister", "PeriodAdjustment", -30),
        ("AccountingRegister", "Account", -10),
        // отбор набора записей (как у выгрузки `std_field`)
        ("InformationRegister", "Filter", -60001),
        ("AccumulationRegister", "Filter", -60001),
        ("AccountingRegister", "Filter", -60001),
    ];
    T.iter().find(|(c, n, _)| *c == class && *n == name).map(|(.., v)| *v).or_else(|| {
        // прочие — коды стандартных реквизитов описания (как у выгрузки `std_field`)
        crate::xml::standard::NAMES.iter().find(|(c, _)| *c == class)?.1.iter().find(|(_, n)| *n == name)?.0.parse().ok()
    })
}

/// Подчиненные объекты метаданных, у которых имя — часть пути.
const KINDS: &[&str] = &["Attribute", "TabularSection", "Dimension", "Resource", "AccountingFlag", "ExtDimensionAccountingFlag", "AddressingAttribute"];

impl F<'_> {
    /// Отладка (CFKIT_DP_DUMP=<файл>): все пути элементов формы `файл\tэлемент\tпуть\tузел|!ошибка` в конец файла.
    pub(super) fn dump_paths(&self, doc: &El, rel: &str, file: &str) {
        fn walk(f: &F, e: &El, rel: &str, out: &mut String) {
            for c in e.elems() {
                if let (Some(n), Some(d)) = (c.attr("name"), c.child("DataPath")) {
                    let r = f.data_path(d).and_then(|v| Doc::new(true, vec![v]).write().map_err(|e| e));
                    let r = r.map(|s| s.trim_start_matches('\u{feff}').trim().to_string()).unwrap_or_else(|e| format!("!{e}")).replace(['\r', '\n', '\t'], "");
                    out.push_str(&format!("{rel}\t{n}\t{}\t{r}\n", d.text().unwrap_or_default()));
                }
                walk(f, c, rel, out);
            }
        }
        let mut out = String::new();
        walk(self, doc, rel, &mut out);
        use std::io::Write;
        if let Ok(mut fh) = std::fs::OpenOptions::new().create(true).append(true).open(file) {
            let _ = fh.write_all(out.as_bytes());
        }
    }

    /// Путь к данным (текст `<DataPath>`) → узел записи.
    pub(super) fn data_path(&self, e: &El) -> R<Node> {
        let t = e.text()?;
        self.path_text(&t).map_err(|m| err(e, &m))
    }

    /// Путь к данным по тексту; ошибка — без строки XML.
    pub(super) fn path_text(&self, t: &str) -> Result<Node, String> {
        let bad = |m: &str| format!("путь {t:?}: {m}");
        // неразрешенный путь выгрузка пишет частями записи: `1/0:<uuid>` — `{2,{1},{0,<uuid>}}`
        // одна часть `58` — тоже неразрешенный путь (имя реквизита с цифры не начинается)
        if t.contains('/') || t.parse::<u64>().is_ok() {
            let raw = |p: &str| match p.split_once(':') {
                None => crate::xml::imp::tpl::int(p).map(|n| l(vec![a(n)])),
                Some((n, g)) => crate::xml::imp::tpl::int(n).filter(|_| crate::brace::is_guid(g)).map(|n| l(vec![a(n), a(g)])),
            };
            let v: Option<Vec<Node>> = t.split('/').map(raw).collect();
            let mut v = v.ok_or_else(|| bad("часть неразрешенного пути не «число» или «число:guid»"))?;
            v.insert(0, an(v.len()));
            return Ok(l(v));
        }
        // Ведущую `~` разбор пути снимает и сначала разрешает путь как обычный (mngbase FUN_18075f8c0); картинка строки
        // списка — постоянный код без номера FieldsMap, так что узел тот же (`~` выгрузка ставит по метаданным).
        if let Some(r) = t.strip_prefix('~').filter(|r| !r.contains('~') && r.split('.').count() == 2 && r.rsplit('.').next().is_some_and(super::dlist::is_default_picture)) {
            return self.path_text(r);
        }
        // `~Список.Поле[~Список.Запасное]` — поле корня или недоступное поле списка (номер — `List::unavailable_id`);
        // `~Items.Т.CurrentData.Поле[~…]` — mngbase FUN_1802c8a30, ветка Items (первая часть не реквизит, 4 части,
        // `Items`, `CurrentData`): элемент по имени (+0x48), CompositeID элемента (+0x90, GUID DAT_1808628e8 — ELEM),
        // первые три части основного и запасного пути заменяются путем таблицы (FUN_1802d1e60), дальше — как у поля
        // списка. Путь таблицы из нескольких частей (номер через FieldsMap префикса) — не выведен.
        if let Some((main, fb)) = t.strip_prefix('~').and_then(|r| self.tilde_split(r)) {
            let parts: Vec<&str> = main.split('.').collect();
            let fparts: Option<Vec<&str>> = fb.map(|f| f.split('.').collect());
            let one = |p: &[&str], n: usize| p.len() == n && !p[n - 1].contains('[');
            if let ["Items", item, "CurrentData", n] = parts.as_slice() {
                let it = self.items.get(*item).filter(|it| it.tag == "Table" && it.dynamic_list && !it.data_path.contains('.'));
                let fm = fparts.as_ref().map(|f| match f.as_slice() { ["Items", i2, "CurrentData", m] if i2 == item => Some(*m), _ => None });
                if let (Some(it), true, false) = (it, one(&parts, 4), fm.is_some_and(|m| m.is_none())) {
                    if let Some(node) = self.list_field_unavailable(&it.data_path, n, fm.flatten()) {
                        return Ok(l(vec![an(2), l(vec![a(it.id.clone()), a(ELEM)]), node.map_err(|m| bad(&m))?]));
                    }
                }
            } else if let ([at, n], true) = (parts.as_slice(), one(&parts, 2)) {
                let fm = fparts.as_ref().map(|f| match f.as_slice() { [a2, m] if a2 == at => Some(*m), _ => None });
                if !fm.is_some_and(|m| m.is_none()) {
                    if let Some(node) = self.list_field_unavailable(at, n, fm.flatten()) {
                        let node = node.map_err(|m| bad(&m))?;
                        let (mut v, _) = self.walk(at).map_err(|m| bad(&m))?;
                        v.push(node);
                        v.insert(0, an(v.len()));
                        return Ok(l(v));
                    }
                }
            }
        }
        if t.contains('~') {
            return Err(bad("недоступные поля пока не загружаются"));
        }
        let parts: Vec<&str> = t.split('.').collect();
        if let ["Items", item, "CurrentData", rest @ ..] = parts.as_slice() {
            let it = self.items.get(*item).ok_or_else(|| bad("нет элемента"))?;
            if it.tag != "Table" || rest.is_empty() {
                return Err(bad("текущие данные не таблицы"));
            }
            let (mut ctx, base) = self.table_ctx(&it.data_path).map_err(|m| bad(&format!("путь таблицы: {m}")))?;
            let mut v = vec![an(rest.len() + 1), l(vec![a(it.id.clone()), a(ELEM)])];
            if it.dynamic_list {
                let nodes = match rest {
                    [n] => self.list_field(&it.data_path, n).map(|n| vec![n]),
                    [n, m] => self.list_field_nested(&it.data_path, n, m).map(Vec::from),
                    [n, m, k] => self.list_field_nested3(&it.data_path, n, m, k).map(Vec::from),
                    _ => Err("путь через поле динамического списка глубже двух частей пока не загружается".into()),
                };
                v.extend(nodes.map_err(|m| bad(&m))?);
                return Ok(l(v));
            }
            let mut p = base;
            for n in rest {
                let (node, next) = self.step(ctx, &p, n).map_err(|m| bad(&m))?;
                v.push(node);
                ctx = next;
                p = format!("{p}.{n}");
            }
            return Ok(l(v));
        }
        let (mut v, _) = self.walk(t).map_err(|m| bad(&m))?;
        v.insert(0, an(v.len()));
        Ok(l(v))
    }

    /// Контекст за путем таблицы; путь таблицы через текущие данные другой таблицы (`Items.Т.CurrentData.Часть…`) —
    /// контекст пути той таблицы и части за ним (части записи идут от пути таблицы, как у `Items…` выше). Второе —
    /// путь от реквизита формы (ключ дополнительных колонок `<AdditionalColumns table=…>`).
    fn table_ctx(&self, t: &str) -> Result<(Ctx<'_>, String), String> {
        let parts: Vec<&str> = t.split('.').collect();
        let ["Items", item, "CurrentData", rest @ ..] = parts.as_slice() else { return Ok((self.walk(t)?.1, t.to_string())) };
        let it = self.items.get(*item).ok_or_else(|| format!("нет элемента {item}"))?;
        if it.tag != "Table" || it.dynamic_list || rest.is_empty() {
            return Err("текущие данные не таблицы".into());
        }
        let (mut ctx, mut p) = self.table_ctx(&it.data_path)?;
        for n in rest {
            ctx = self.step(ctx, &p, n)?.1;
            p = format!("{p}.{n}");
        }
        Ok((ctx, p))
    }

    /// Путь `t` ведет к коллекции строк (таблица, дерево, список значений, табличная часть, набор записей).
    pub(super) fn is_collection(&self, t: &str) -> bool {
        match self.walk(t) {
            Ok((_, Ctx::Form { types, .. })) => {
                types.iter().any(|t| matches!(t.as_str(), "v8:ValueTable" | "v8:ValueTree" | "v8:ValueListType" | "cfg:DynamicList") || (t.starts_with("cfg:") && t.contains("RecordSet.")))
            }
            // набор записей движений документа (`Объект.Движения.Хозрасчетный`: ОперацияБух ERP)
            Ok((_, Ctx::Meta { class, .. })) => class == "TabularSection" || (class.ends_with("Register") && t.split('.').nth(1).and_then(|s| super::dlist::std_en(s).or(Some(s))) == Some("RegisterRecords")),
            _ => false,
        }
    }

    /// Части пути от реквизита формы и контекст за последней частью.
    /// Индекс строки `[N]` после части — `{N,e67e2953…}` (контекст не меняется: дальше — колонка той же коллекции).
    fn walk(&self, t: &str) -> Result<(Vec<Node>, Ctx<'_>), String> {
        let index = |n: &'_ str| -> Result<(String, Option<Node>), String> {
            let Some((b, i)) = n.split_once('[') else { return Ok((n.to_string(), None)) };
            let i = i.strip_suffix(']').and_then(crate::xml::imp::tpl::int).ok_or_else(|| format!("индекс строки {n:?}"))?;
            Ok((b.to_string(), Some(l(vec![a(i), a(ROW_INDEX)]))))
        };
        let mut parts = t.split('.');
        let (root, ri) = index(parts.next().unwrap_or_default())?;
        let at = self.attrs.get(&root).ok_or_else(|| format!("нет реквизита формы {root}"))?;
        let mut v = vec![l(vec![a(at.id.clone())])];
        v.extend(ri);
        let mut ctx = Ctx::Form { cols: &at.cols, types: &at.types };
        let rest: Vec<&str> = parts.collect();
        let (mut p, mut tail) = (root.clone(), &rest[..]);
        // компоновщик настроек списка — свойство −6 (обратное `std_field` выгрузки), дальше — по таблице платформенных
        // путей (ВыборОбъектовОтбором Клиента ЭДО: `СписокДанных.SettingsComposer.Settings.Filter` → `{-6},{0},{1}`)
        if let ("DynamicList", ["SettingsComposer", more @ ..]) = (at.class.as_str(), tail) {
            v.push(l(vec![a("-6")]));
            (ctx, p, tail) = (Ctx::Plat("SettingsComposer"), format!("{root}.SettingsComposer"), more);
        } else if at.class == "DynamicList" {
            return match tail {
                [] => Ok((v, ctx)),
                [n] if !n.contains('[') => {
                    v.push(self.list_field(&root, n)?);
                    Ok((v, Ctx::Leaf))
                }
                [n, m] if !n.contains('[') && !m.contains('[') => {
                    v.extend(self.list_field_nested(&root, n, m)?);
                    Ok((v, Ctx::Leaf))
                }
                [n, m, k] if ![n, m, k].iter().any(|p| p.contains('[')) => {
                    v.extend(self.list_field_nested3(&root, n, m, k)?);
                    Ok((v, Ctx::Leaf))
                }
                _ => Err("путь через поле динамического списка глубже двух частей пока не загружается".into()),
            };
        }
        for n in tail {
            let (n, ri) = index(n)?;
            let (node, next) = self.step(ctx, &p, &n)?;
            v.push(node);
            v.extend(ri);
            ctx = next;
            p = format!("{p}.{n}");
        }
        Ok((v, ctx))
    }

    /// Одна часть пути `n` в контексте `ctx` (путь до нее — `p`, для дополнительных колонок).
    fn step<'s>(&'s self, ctx: Ctx<'s>, p: &str, n: &str) -> Result<(Node, Ctx<'s>), String> {
        // ключ колонок — путь с английскими именами стандартных реквизитов (`Объект.RegisterRecords.Х` для `Объект.Движения.Х`)
        let pe = p.split('.').map(|s| super::dlist::std_en(s).unwrap_or(s)).collect::<Vec<_>>().join(".");
        let extra: Vec<&Col> = self.attrs.values().flat_map(|a| a.extra.iter()).filter(|(t, c)| (t == p || *t == pe) && c.name == n).map(|(_, c)| c).collect();
        let found = match ctx {
            Ctx::Form { cols, types } => {
                let own = cols.iter().find(|c| c.name == n);
                // определяемый тип — его типы (выгрузка разрешает путь по классу типа реквизита)
                let by_type = match types {
                    [t] if t.starts_with("cfg:DefinedType.") => self.in_ctx(self.typed_of(&t["cfg:".len()..]), n)?,
                    _ => self.in_ctx(typed(types), n)?,
                };
                match (own, by_type) {
                    (Some(_), Some(_)) => return Err(format!("{n}: и колонка реквизита формы, и поле типа")),
                    (Some(c), None) => Some((l(vec![a(c.id.clone())]), Ctx::Form { cols: &[], types: &c.types })),
                    (None, r) => r,
                }
            }
            c => self.in_ctx(c, n)?,
        };
        match (found, extra.as_slice()) {
            (Some(_), [_, ..]) => Err(format!("{n}: и поле, и дополнительная колонка")),
            (Some(r), []) => Ok(r),
            (None, [c]) => Ok((l(vec![a(c.id.clone()), a(ADDED_COLUMN)]), Ctx::Form { cols: &[], types: &c.types })),
            // количество строк коллекции платформенного типа (список значений: ВыборКодовМаркировки Клиента ЭДО
            // `{2,{4},{100000000}}`) — тот же код, что у табличной части
            (None, []) if n == "RowsCount" || n == "КоличествоСтрок" => Ok((l(vec![a("100000000")]), Ctx::Leaf)),
            _ => Err(format!("поле {n} не найдено")),
        }
    }

    fn in_ctx<'s>(&'s self, ctx: Ctx<'s>, n: &str) -> Result<Option<(Node, Ctx<'s>)>, String> {
        Ok(match ctx {
            Ctx::Meta { full, class } => self.meta_step(&full, &class, n)?,
            Ctx::Plat(c) => plat(c, n),
            Ctx::Form { .. } | Ctx::Leaf => None,
        })
    }

    fn typed_of(&self, full: &str) -> Ctx<'static> {
        self.x.attr_types.get(full).map_or(Ctx::Leaf, |t| typed(t))
    }

    /// Поле объекта метаданных `full` класса `class`.
    fn meta_step<'s>(&'s self, full: &str, class: &str, n: &str) -> Result<Option<(Node, Ctx<'s>)>, String> {
        // движения документа: набор записей регистра по имени (обратное sub_path выгрузки: `{0,<uuid регистра>}`)
        if class == "RegisterRecords" {
            return Ok(["AccountingRegister", "AccumulationRegister", "InformationRegister", "CalculationRegister"].iter().find_map(|c| {
                let f = format!("{c}.{n}");
                self.x.id(&f).map(|g| (l(vec![a("0"), a(g)]), Ctx::Meta { full: f.clone(), class: c.to_string() }))
            }));
        }
        // строка видов субконто: −12 LineNumber, −13 ExtDimensionType, −14 Predefined, −15 TurnoversOnly
        // (standard.rs, StandardTabularSection), признак учета субконто — `{0,<uuid>}`
        if class == "ExtDimensionTypes" {
            let code = match n {
                "LineNumber" => Some(-12),
                "ExtDimensionType" => Some(-13),
                "Predefined" => Some(-14),
                "TurnoversOnly" => Some(-15),
                _ => None,
            };
            if let Some(c) = code {
                return Ok(Some((l(vec![an(c)]), Ctx::Leaf)));
            }
            let f = format!("{full}.ExtDimensionAccountingFlag.{n}");
            return Ok(self.x.id(&f).map(|g| (l(vec![a("0"), a(g)]), Ctx::Leaf)));
        }
        if class == "ConstantsSet" {
            let f = format!("Constant.{n}");
            return Ok(self.x.id(&f).map(|g| (l(vec![a("0"), a(g)]), self.typed_of(&f))));
        }
        if let Some(r) = self.direct(full, class, n)? {
            return Ok(Some(r));
        }
        if class == "Report" && n == "SettingsComposer" {
            return Ok(Some((l(vec![a("0"), a(REPORT_COMPOSER)]), Ctx::Plat("SettingsComposer"))));
        }
        // количество строк — код 100000000 у любого класса (обратное `sub_path` выгрузки: табличная часть, набор записей)
        // русское имя — из таблицы пар имен mngbase (0x180a833f0: RowsCount / КоличествоСтрок, Total / Итог, …)
        if n == "RowsCount" || n == "КоличествоСтрок" {
            return Ok(Some((l(vec![a("100000000")]), Ctx::Leaf)));
        }
        // общий реквизит в составе объекта: «Использовать», или «Авто» при «Автоиспользовании» = Использовать
        // (обратное выгрузке: `{0,<uuid общего реквизита>}`, `foreign` его пропускает)
        if class != "TabularSection" {
            let f = format!("CommonAttribute.{n}");
            let used = self.x.common.iter().find(|(c, ..)| c == n).is_some_and(|(_, auto, content)| match content.get(full).map(String::as_str) {
                Some("Use") => true,
                None | Some("Auto") => auto == "Use",
                _ => false,
            });
            if let Some(g) = self.x.id(&f).filter(|_| used) {
                return Ok(Some((l(vec![a("0"), a(g)]), self.typed_of(&f))));
            }
        }
        // итог поля набора записей — `{101000000,<uuid поля>}` у любого класса (обратное sub_path выгрузки)
        if let Some(c) = n.strip_prefix("Total").filter(|_| class != "TabularSection") {
            if let Some((_, _, g)) = self.child(full, c)? {
                return Ok(Some((l(vec![a(TOTAL), a(g)]), Ctx::Leaf)));
            }
        }
        if class == "TabularSection" {
            if let Some(c) = n.strip_prefix("Total") {
                if let Some(g) = self.x.id(&format!("{full}.Attribute.{c}")) {
                    return Ok(Some((l(vec![a(TOTAL), a(g)]), Ctx::Leaf)));
                }
            }
        }
        if class == "AccountingRegister" {
            if let Some(Ok(k @ 1..=3)) = n.strip_prefix("ExtDimension").map(str::parse::<i64>) {
                return Ok(Some((l(vec![an(k - 1), a(EXT_DIM)]), Ctx::Leaf)));
            }
            for (side, code, ext) in [("Dr", 2, EXT_DIM_DR), ("Cr", 3, EXT_DIM_CR)] {
                if let Some(k) = n.strip_prefix(&format!("ExtDimension{side}")) {
                    if let Ok(k @ 1..=3) = k.parse::<i64>() {
                        return Ok(Some((l(vec![an(k - 1), a(ext)]), Ctx::Leaf)));
                    }
                }
                if let Some(b) = n.strip_suffix(side) {
                    if let Some((_, f, g)) = self.child(full, b)? {
                        return Ok(Some((l(vec![an(code), a(g)]), self.typed_of(&f))));
                    }
                }
            }
        }
        Ok(None)
    }

    /// Подчиненный объект `n` объекта `full`: (вид, полное имя, uuid); несколько — отказ.
    fn child(&self, full: &str, n: &str) -> Result<Option<(&'static str, String, String)>, String> {
        let mut hits: Vec<_> = KINDS.iter().filter_map(|k| {
            let f = format!("{full}.{k}.{n}");
            self.x.id(&f).map(|g| (*k, f.clone(), g.to_string()))
        }).collect();
        // признак учета и одноименный признак учета субконто: у самого счета — признак учета (субконто — у строк
        // видов субконто). shortcut: по сверке ФормаСчета БП (`Объект.Валютный`)
        if hits.len() == 2 && hits.iter().any(|h| h.0 == "AccountingFlag") && hits.iter().any(|h| h.0 == "ExtDimensionAccountingFlag") {
            hits.retain(|h| h.0 == "AccountingFlag");
        }
        if hits.len() > 1 {
            return Err(format!("{n}: несколько подчиненных объектов {full}"));
        }
        Ok(hits.pop())
    }

    /// Подчиненный объект или стандартный реквизит (вместе — отказ).
    fn direct<'s>(&'s self, full: &str, class: &str, n: &str) -> Result<Option<(Node, Ctx<'s>)>, String> {
        // стандартный реквизит и русским именем (`Объект.Дата`, `Объект.Движения` в условном оформлении: ФормаДокумента
        // НачальнаяШтатнаяРасстановка и ОперацияБух ERP)
        let n = match std_code(class, n) {
            None => super::dlist::std_en(n).filter(|e| std_code(class, e).is_some()).unwrap_or(n),
            Some(_) => n,
        };
        match (self.child(full, n)?, std_code(class, n)) {
            (Some(_), Some(_)) => Err(format!("{n}: и реквизит, и стандартный реквизит {full}")),
            (Some((k, f, g)), None) => {
                let next = if k == "TabularSection" { Ctx::Meta { full: f, class: "TabularSection".into() } } else { self.typed_of(&f) };
                Ok(Some((l(vec![a("0"), a(g)]), next)))
            }
            (None, Some(c)) => {
                // виды субконто счета — стандартная табличная часть (выгрузка: `std_key` ChartOfAccounts −12)
                if class == "ChartOfAccounts" && n == "ExtDimensionTypes" {
                    return Ok(Some((l(vec![an(c)]), Ctx::Meta { full: full.to_string(), class: "ExtDimensionTypes".into() })));
                }
                // ведущие, вытесняющие, базовые виды расчета — таблицы с колонкой CalculationType
                if class == "ChartOfCalculationTypes" && [-10, -20, -30].contains(&c) {
                    return Ok(Some((l(vec![an(c)]), Ctx::Plat("CalculationTypes"))));
                }
                if class == "Document" && n == "RegisterRecords" {
                    return Ok(Some((l(vec![an(c)]), Ctx::Meta { full: full.to_string(), class: "RegisterRecords".into() })));
                }
                // владелец справочника с одним владельцем — путь продолжается его полями (выгрузка: `{-5},{0,<uuid>}`)
                let owner = (class == "Catalog" && n == "Owner").then(|| self.x.objects.get(full)).flatten().and_then(|o| match o.owner_list.as_slice() {
                    [w] => w.split_once('.').map(|(c, _)| Ctx::Meta { full: w.clone(), class: c.to_string() }),
                    _ => None,
                });
                Ok(Some((l(vec![an(c)]), owner.unwrap_or(Ctx::Leaf))))
            }
            (None, None) => Ok(None),
        }
    }
}
