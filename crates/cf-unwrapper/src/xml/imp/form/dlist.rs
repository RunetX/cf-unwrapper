//! Динамический список: настройки реквизита (`<Settings xsi:type="DynamicList">` → словарь `{0,N,"ключ",значение…}`),
//! номера полей (FieldsMap) и словарь его таблицы.
//!
//! Номера полей — как их дает загрузка платформы (mngbase.dll): поля списка — дерево, узлы которого платформа загружает
//! по требованию (0x1802b76a0: дети узла получают номер из FieldsMap по имени, иначе — следующий из счетчика); при
//! каждом перестроении (0x1802b85b0) счетчик = наибольший номер FieldsMap + 1 (пустой — 1), корень загружается заново.
//! Новое поле формы попадает в FieldsMap с номером, который получило при перестроении: номер = счетчик + место среди
//! полей корня, которых в FieldsMap еще нет. Порядок полей корня — доступные поля компоновки (схема строится по основной
//! таблице, 0x1802b6b10 `ULDataSource`/`ULDataSet`): по имени без учета регистра. Состав стандартных полей по классу
//! основной таблицы снят сверкой (БП, 835 из 863 форм без своего запроса), прочие классы — отказ.

use super::*;

/// Основная таблица `{"#",fc01b5df…,<uuid>}`.
const MAIN_TABLE: &str = "fc01b5df-97fe-449b-83d4-218a090e681e";
const KEY_FIELDS: &str = "51e7a0d2-530b-11d4-b98a-008048da3034";
const DATA_PARAMETERS: &str = "6217eee1-6289-49d2-9315-d87888cd2d62";
/// Стандартная картинка строки (`Список.DefaultPicture`) — поле без номера FieldsMap.
pub(super) const DEFAULT_PICTURE: &str = "10000000";

/// Имя стандартной картинки — пара терминов mngbase 0x180a83440 (DefaultPicture / СтандартнаяКартинка): поле дерева
/// списка с обоими именами (FUN_1802998a0, `term_string(…, 0)` и `(…, 1)`), разрешитель `~` — `compare_term` (FUN_1802c8a30).
pub(super) fn is_default_picture(n: &str) -> bool {
    n == "DefaultPicture" || n == "СтандартнаяКартинка"
}

/// Стандартные поля: имя и второе (русское) имя.
pub(super) const STD: &[(&str, &str)] = &[
    ("Ref", "Ссылка"),
    ("DeletionMark", "ПометкаУдаления"),
    ("Predefined", "Предопределенный"),
    ("PredefinedDataName", "ИмяПредопределенныхДанных"),
    ("Code", "Код"),
    ("Description", "Наименование"),
    ("Parent", "Родитель"),
    ("IsFolder", "ЭтоГруппа"),
    ("Owner", "Владелец"),
    ("ValueType", "ТипЗначения"),
    ("Number", "Номер"),
    ("Date", "Дата"),
    ("Posted", "Проведен"),
    ("PointInTime", "МоментВремени"),
    ("Period", "Период"),
    ("Recorder", "Регистратор"),
    ("LineNumber", "НомерСтроки"),
    ("Active", "Активность"),
    ("RecordType", "ВидДвижения"),
    ("Type", "Тип"),
    ("BusinessProcess", "БизнесПроцесс"),
    ("RoutePoint", "ТочкаМаршрута"),
    ("Executed", "Выполнена"),
    ("Order", "Порядок"),
    ("OffBalance", "Забалансовый"),
    ("ExtDimensionTypes", "ВидыСубконто"),
    ("ThisNode", "ЭтотУзел"),
    ("SentNo", "НомерОтправленного"),
    ("ReceivedNo", "НомерПринятого"),
    ("ExchangeDate", "ДатаОбмена"),
    // bp.dll: пары имен 0x1803659a8…
    ("Completed", "Завершен"),
    ("HeadTask", "ВедущаяЗадача"),
    ("Started", "Стартован"),
    // calc.dll: пары имен 0x180547430…
    ("LeadingCalculationTypes", "ВедущиеВидыРасчета"),
    ("ActionPeriodIsBasic", "ПериодДействияБазовый"),
    ("BaseCalculationTypes", "БазовыеВидыРасчета"),
    ("DisplacingCalculationTypes", "ВытесняющиеВидыРасчета"),
];

/// Поле списка: имя и второе имя (у стандартного поля — русское).
#[derive(Clone)]
pub(super) struct Field {
    pub name: String,
    pub second: String,
}

/// Динамический список реквизита формы: поля корня по порядку и FieldsMap `(номер, поле)`.
#[derive(Clone)]
pub(super) struct List {
    pub main: String,
    pub fields: Vec<Field>,
    pub map: Vec<(i64, Field)>,
    /// свой запрос (за полями корня может идти `DataParameters`, `query::dcs::data_parameters`)
    pub custom: bool,
    /// свой запрос не компилируется: корня нет (ни полей, ни `DataParameters`), недоступные поля — счетчиком
    pub no_root: bool,
    /// свой запрос, у которого `DataParameters` в корне не выведен: причина (номер недоступного поля — отказ)
    pub dp: Option<String>,
    /// поля только из условного оформления формы и их номера (`F::list_paths_in_load_order`): в FieldsMap не пишутся
    pub ca: Vec<(String, Option<i64>)>,
    /// узлы корня раскрыты (путь `Список.А.Б`, уже бывший в FieldsMap): счетчик дерева ушел за детей — до следующей
    /// записи в FieldsMap (перестроение) номер недоступного поля не выведен
    pub expanded: bool,
    /// свой запрос: поле корня → объект и поле объекта (`query_origins`)
    pub origins: Vec<(String, String, String)>,
    /// свой запрос: типы выходных полей по проверке запроса (`query_types`) — для полей не из таблицы объекта
    pub types: crate::query::resolve::Out,
    /// свой запрос: поля ролей основной таблицы (`query::roles::main_roles`) — путь по имени роли (`Список.Owner`)
    pub roles: Vec<String>,
    /// типы полей корня не из таблиц: `DataParameters` своего запроса (`query::dcs::parameter_fields`), вычисляемые поля
    /// (`with_calc`) — или почему не выведены
    pub kinds: Vec<(String, Result<Kind, String>)>,
    /// свой запрос со СГРУППИРОВАТЬ ПО: поля вывода «не чистые» (dcs FUN_18030f4b0 снимает флаги использования +0x7f…
    /// +0x85), дети-табличные части их детей не проходят (FUN_180301cf0) — у раскрытия детей без табличных частей
    pub grouped: bool,
}

fn num_prop(o: &super::super::index::Obj, k: &str) -> i64 {
    o.props.get(k).and_then(|v| v.parse().ok()).unwrap_or(0)
}

/// Основной запрос пакета (последний без ПОМЕСТИТЬ) со СГРУППИРОВАТЬ ПО. shortcut: агрегаты без группировки и
/// выражения по нескольким полям («не чистые» поля по одному) не выделены — при отличиях уточнить по FUN_18030f4b0
fn grouped(text: &str) -> bool {
    crate::query::parse(text).ok().and_then(|b| b.queries.into_iter().rev().find(|q| q.parts.iter().all(|p| p.into.is_none()))).is_some_and(|q| q.parts.iter().any(|p| !p.group.is_empty()))
}

/// Поля корня списка с основной таблицей `main` (`Catalog.Банки`) по порядку доступных полей.
fn root_fields(x: &Index, main: &str) -> R<Vec<Field>> {
    // движения с субконто регистра бухгалтерии — поля построителя accnt (`records_ext_fields`); генератор текста берет
    // поля с именем (−8 без имени выпадает, см. план обмена)
    if let Some((base, vt)) = main.rsplit_once('.').filter(|(b, _)| b.contains('.')) {
        // реквизит в режиме пароля (у прочих таблиц ниже его в корне нет, dcs FUN_1800c4b90) — пара Dr/Cr небалансового
        // поля не выведена; общий реквизит — маска 0xff accnt FUN_1800d86f0 не прослежена: состав корня не выведен — отказ
        // (vt_field своего запроса их видит: поле, которого нет, там — отказ)
        if x.objects.get(base).is_some_and(|o| o.kids.iter().any(|(k, n)| x.password_full.contains(&format!("{base}.{k}.{n}")))) {
            return Err(format!("реквизит в режиме пароля у виртуальной основной таблицы {main}: поле корня не выведено"));
        }
        if !crate::query::roles::common_fields(x, base).is_empty() {
            return Err(format!("общий реквизит у виртуальной основной таблицы {main}: поле корня не выведено"));
        }
        let fs = records_ext_fields(x, base, vt).ok_or_else(|| format!("поля виртуальной основной таблицы {main} не выведены"))?;
        return sorted_root(main, fs.into_iter().map(|(name, second, _)| Field { name, second }).collect());
    }
    let o = x.objects.get(main).ok_or_else(|| format!("нет основной таблицы {main}"))?;
    let class = main.split('.').next().unwrap_or_default();
    let p = |k: &str| o.props.get(k).map(String::as_str).unwrap_or_default();
    let mut std: Vec<&str> = Vec::new();
    match class {
        "Catalog" | "ChartOfCharacteristicTypes" => {
            std.extend(["Ref", "DeletionMark", "Predefined", "PredefinedDataName"]);
            if num_prop(o, "CodeLength") > 0 {
                std.push("Code");
            }
            if num_prop(o, "DescriptionLength") > 0 {
                std.push("Description");
            }
            if p("Hierarchical") == "true" {
                std.push("Parent");
                if class == "ChartOfCharacteristicTypes" || p("HierarchyType") == "HierarchyFoldersAndItems" {
                    std.push("IsFolder");
                }
            }
            if class == "Catalog" && o.owners {
                std.push("Owner");
            }
            if class == "ChartOfCharacteristicTypes" {
                std.push("ValueType");
            }
        }
        "Document" => {
            std.extend(["Ref", "DeletionMark", "Date", "Posted", "PointInTime"]);
            if num_prop(o, "NumberLength") > 0 {
                std.push("Number");
            }
        }
        // backend FUN_1801872c0 (источник категории 0a7fea78, vtable 0x181393758 +0x20): поля таблицы плана обмена — Ref,
        // [-0xb], DataVersion, DeletionMark, [-0xc], ThisNode (-0xd, совместимость > 0x139b0), Code, Description, SentNo,
        // ReceivedNo, ExchangeDate (-0xe, совместимость > 0x139c1), реквизиты, табличные части, Presentation. Текст
        // запроса (mngbase FUN_180704800) берет поля с именем (-0xb, -0xc — с пустым, &DAT_1819a5e00) без Presentation и
        // DataVersion (ede4bd22 +0x58, +0x60: backend FUN_1803e9c00 — члены +0x78, +0x88).
        "ExchangePlan" => {
            let c = x.compat.ok_or("поля плана обмена без режима совместимости: не выведены")?;
            std.extend(["Ref", "DeletionMark"]);
            if c > 0x139b0 {
                std.push("ThisNode");
            }
            std.extend(["Code", "Description", "SentNo", "ReceivedNo"]);
            if c > 0x139c1 {
                std.push("ExchangeDate");
            }
        }
        // basic.dll FUN_1801ccf80 (имена стандартных полей таблиц по номеру): у перечисления — Ref (−2) и Order (−3);
        // Presentation генератор текста не берет (см. план обмена)
        "Enum" => std.extend(["Ref", "Order"]),
        // calc.dll FUN_180037670 (основная таблица плана видов расчета): −6 Ref, [−9, −7 DataVersion], −5 DeletionMark,
        // −8 Predefined при совместимости < 0x139af, иначе −0x6e — поле без имени (фабрика FUN_180039900: имя —
        // глобальная пустая строка DAT_1805538f8) с подполями по составному номеру (FUN_180038e50: −0xb
        // PredefinedDataName, −8 Predefined, −7 DataVersion) — в корень идут Predefined и PredefinedDataName, −2 Code и −3 Description при длине > 0, −4 ActionPeriodIsBasic и
        // −0x14 DisplacingCalculationTypes при +0x240, −10 BaseCalculationTypes при +0x220 ≠ 0, −0x1e
        // LeadingCalculationTypes всегда, реквизиты, табличные части, Presentation (имена — FUN_18031a9b0).
        // Что такое +0x240/+0x220 интерфейса объекта, не прослежено (calc FUN_180319360: GUID свойства → номер —
        // DependenceOnCalculationTypes 9, ActionPeriodUse 0xb; слоты vtable интерфейса с номерами не связаны) — у
        // плана не с умолчаниями этих свойств отказ.
        "ChartOfCalculationTypes" => {
            if !x.compat.is_some_and(|c| c >= 0x139af) {
                return Err("поля плана видов расчета при совместимости до 8.3.3 (−8 Predefined вместо −0x6e) не выведены".into());
            }
            std.extend(["Ref", "DeletionMark", "Predefined", "PredefinedDataName", "LeadingCalculationTypes"]);
            // +0x240 — использование периода действия (ActionPeriodUse), +0x220 — зависимость от видов расчета: shortcut —
            // слоты сопоставлены со свойствами по смыслу (вытесняющие и базовость периода — при периоде действия, базовые —
            // при зависимости), сверено на планах Начисления/Удержания ERP
            match p("ActionPeriodUse") {
                "true" => std.extend(["ActionPeriodIsBasic", "DisplacingCalculationTypes"]),
                "false" => {}
                v => return Err(format!("ActionPeriodUse {v:?} плана видов расчета не выведено")),
            }
            match p("DependenceOnCalculationTypes") {
                "DontUse" => {}
                "OnActionPeriod" | "ByActionPeriod" | "NotByActionPeriod" => std.push("BaseCalculationTypes"),
                v => return Err(format!("DependenceOnCalculationTypes {v:?} плана видов расчета не выведено")),
            }
            if num_prop(o, "CodeLength") > 0 {
                std.push("Code");
            }
            if num_prop(o, "DescriptionLength") > 0 {
                std.push("Description");
            }
        }
        // bp.dll FUN_1800275d0 (основная таблица бизнес-процесса): −5 Ref, [−0xe, −6 DataVersion], −4 DeletionMark,
        // −2 Number при длине номера (+0x40) ≠ 0, −3 Date, −7 Completed, −8 HeadTask, −9 Started (имена — FUN_180231ba0),
        // реквизиты, табличные части, Presentation; DataVersion и Presentation текст запроса не берет (см. план обмена)
        "BusinessProcess" => {
            std.extend(["Ref", "DeletionMark", "Date", "Completed", "HeadTask", "Started"]);
            if num_prop(o, "NumberLength") > 0 {
                std.push("Number");
            }
        }
        "Task" => {
            std.extend(["Ref", "DeletionMark", "Number", "Date", "Description", "Executed"]);
            // BusinessProcess и RoutePoint — только у задачи какого-нибудь бизнес-процесса (их тип — ссылки на процессы
            // этой задачи; БюджетнаяЗадача ERP без процесса — номера FieldsMap трех форм)
            if x.objects.iter().any(|(k, b)| k.starts_with("BusinessProcess.") && b.props.get("Task").map(String::as_str) == Some(main)) {
                std.extend(["BusinessProcess", "RoutePoint"]);
            }
        }
        "DocumentJournal" => std.extend(["Ref", "Type", "Date", "Number", "Posted", "DeletionMark"]),
        "InformationRegister" => {
            // basic FUN_1801da400 (таблица регистра сведений): периодичность (+0x238) ≠ 0 — Period, при 6 (по позиции
            // регистратора) — еще PointInTime (пара терминов 0x180d0f9f8); подчинение регистратору (+0x248 == 1) —
            // Recorder, LineNumber, Active
            if p("InformationRegisterPeriodicity") == "RecorderPosition" {
                std.push("PointInTime");
            }
            if p("InformationRegisterPeriodicity") != "Nonperiodical" {
                std.push("Period");
            }
            if p("WriteMode") == "RecorderSubordinate" {
                std.extend(["Recorder", "LineNumber", "Active"]);
            }
        }
        "AccumulationRegister" => {
            std.extend(["Period", "Recorder", "LineNumber", "Active", "PointInTime"]);
            if p("RegisterType") == "Balance" {
                std.push("RecordType");
            }
        }
        // shortcut: состав по номерам стандартных полей выгрузки (−6 Parent, −7 Code, −8 Description, −10 Type,
        // −11 OffBalance) и сверке КорреспонденцииСчетов БП; код построителя accnt.dll не разобран
        "ChartOfAccounts" => {
            std.extend(["Ref", "DeletionMark", "Predefined", "PredefinedDataName", "Parent", "Type", "OffBalance"]);
            if num_prop(o, "CodeLength") > 0 {
                std.push("Code");
            }
            if num_prop(o, "DescriptionLength") > 0 {
                std.push("Description");
            }
            if num_prop(o, "OrderLength") > 0 {
                std.push("Order");
            }
            if num_prop(o, "MaxExtDimensionCount") > 0 {
                std.push("ExtDimensionTypes");
            }
        }
        // критерий отбора: поле Ссылка — ссылки на объекты состава (как таблица критерия в запросе, `IndexMeta::vt_field`)
        "FilterCriterion" => std.push("Ref"),
        c => return Err(format!("поля основной таблицы динамического списка класса {c} пока не выведены")),
    }
    let second = |n: &str| std_ru_in(class, n).unwrap_or_default();
    let mut v: Vec<Field> = std.iter().map(|n| Field { name: n.to_string(), second: second(n).to_string() }).collect();
    for (k, n) in &o.kids {
        // реквизит в режиме пароля в поля набора не попадает (как `query::dcs::password_dropped`; Пароль плана обмена
        // МобильноеПриложениеТорговыйПредставитель ERP), кроме схемы старой совместимости (там же)
        if matches!(k.as_str(), "Attribute" | "Dimension" | "Resource" | "TabularSection" | "Column" | "AddressingAttribute" | "AccountingFlag") && (old_schema(x) || !x.password_full.contains(&format!("{main}.{k}.{n}"))) {
            v.push(Field { name: n.clone(), second: n.clone() });
        }
    }
    // общие реквизиты: регистр сведений — из кода (roles::common_fields), прочие классы — явное «использовать» в составе,
    // план обмена — еще Auto / нет в составе при АвтоИспользовании Use (разделитель МиграцияПриложений Клиента ЭДО)
    // shortcut: правило по сверке (Auto у справочников и документов БП не попадает, у плана обмена ЭДО попадает), код
    // построителя таблиц не разобран — смотреть basic при первом расхождении
    if class == "InformationRegister" {
        v.extend(crate::query::roles::common_fields(x, main).into_iter().map(|(n, _)| Field { name: n.to_string(), second: n.to_string() }));
    } else {
        for (n, auto, content) in &x.common {
            let used = match content.get(main).map(String::as_str) {
                Some("Use") => true,
                None | Some("Auto") => class == "ExchangePlan" && auto == "Use",
                _ => false,
            };
            if used {
                v.push(Field { name: n.clone(), second: n.clone() });
            }
        }
    }
    sorted_root(main, v)
}

/// Поля корня по порядку доступных полей (без учета регистра, латиница первой); имена, равные без учета регистра, — отказ.
fn sorted_root(main: &str, mut v: Vec<Field>) -> R<Vec<Field>> {
    if let Some(f) = v.iter().find(|f| !nocase_ok(&f.name)) {
        return Err(format!("поле {}: сравнение без учета регистра с символами вне ASCII и кириллицы (ICU) не воспроизведено", f.name));
    }
    v.sort_by(|a, b| nocase(&a.name, &b.name));
    if v.windows(2).any(|w| nocase(&w[0].name, &w[1].name).is_eq()) {
        return Err(format!("поля {main} с одинаковым именем без учета регистра"));
    }
    Ok(v)
}

/// Поле корня списка для раскрытия узла дерева полей.
#[derive(Clone)]
pub(super) enum Kind {
    /// типы поля (`cfg:CatalogRef.X`, `xs:dateTime`…)
    Types(Vec<String>),
    /// табличная часть основной таблицы (`Catalog.X.TabularSection.T`)
    Ts(String),
    /// узел `DataParameters` своего запроса: имена параметров набора
    Params(Vec<String>),
}

/// Классы ссылочных таблиц, у которых поля таблицы — `root_fields` (у перечисления нет DataVersion).
const REF_TABLES: &[&str] = &["Catalog", "Document", "Enum", "ChartOfCharacteristicTypes", "ExchangePlan", "BusinessProcess", "Task", "ChartOfCalculationTypes"];

/// Типы реквизита `full` (`Catalog.X.Attribute.Y`): определяемый тип — его типы; прочие наборы типов — отказ.
fn attr_types(x: &Index, full: &str) -> R<Vec<String>> {
    let mut v = x.attr_types.get(full).cloned().ok_or_else(|| format!("нет типа {full}"))?;
    for s in x.type_sets.get(full).into_iter().flatten() {
        // все ссылки класса (`cfg:DocumentRef`) — все объекты класса
        if let Some(c) = s.strip_prefix("cfg:").and_then(|r| r.strip_suffix("Ref")).filter(|c| REF_TABLES.contains(c)) {
            let p = format!("{c}.");
            v.extend(x.objects.keys().filter_map(|k| k.strip_prefix(&p)).filter(|n| !n.contains('.')).map(|n| format!("cfg:{c}Ref.{n}")));
            continue;
        }
        // характеристика — типы значения характеристик плана видов характеристик (его Type)
        if let Some(c) = s.strip_prefix("cfg:Characteristic.") {
            v.extend(attr_types(x, &format!("ChartOfCharacteristicTypes.{c}"))?);
            continue;
        }
        let d = s.strip_prefix("cfg:DefinedType.").ok_or_else(|| format!("набор типов {s} у {full}: состав полей не выведен"))?;
        let d = format!("DefinedType.{d}");
        if x.type_sets.contains_key(&d) {
            return Err(format!("набор типов в {d}: состав полей не выведен"));
        }
        v.extend(x.attr_types.get(&d).cloned().ok_or_else(|| format!("нет типа {d}"))?);
    }
    Ok(v)
}

/// Тип поля корня `name` основной таблицы `main`: стандартное поле по классу, реквизит, измерение, ресурс, табличная
/// часть, общий реквизит.
fn root_kind(x: &Index, main: &str, name: &str) -> R<Kind> {
    let class = main.split('.').next().unwrap_or_default();
    let o = x.objects.get(main).ok_or_else(|| format!("нет основной таблицы {main}"))?;
    let reft = |full: &str| full.split_once('.').map(|(c, n)| format!("cfg:{c}Ref.{n}")).ok_or_else(|| format!("тип ссылки {full}"));
    // колонка своего запроса — под русским именем стандартного поля (`Т.ВерсияДанных`, ERP СтруктураРабочихЦентров)
    let own = o.kids.iter().any(|(_, n)| n == name);
    let name = if own { name } else { std_en(name).or((name == "ВерсияДанных").then_some("DataVersion")).unwrap_or(name) };
    match name {
        // ссылка журнала — ссылки регистрируемых документов (roles::journal_ref)
        "Ref" if class == "DocumentJournal" => return xml_types(&crate::query::roles::journal_ref(x, main)).map(Kind::Types).map_err(|m| format!("ссылка журнала {main}: {m}")),
        // ссылка критерия отбора — ссылки на объекты его состава (`IndexMeta::vt_field`)
        "Ref" if class == "FilterCriterion" => {
            use crate::query::resolve::Meta;
            return xml_types(&IndexMeta(x).vt_field(main, "", "Ref")?.unwrap_or_default()).map(Kind::Types);
        }
        "Ref" => return Ok(Kind::Types(vec![reft(main)?])),
        "Parent" if matches!(class, "Catalog" | "ChartOfCharacteristicTypes") => return Ok(Kind::Types(vec![reft(main)?])),
        "Owner" if class == "Catalog" => return Ok(Kind::Types(o.owner_list.iter().map(|w| reft(w)).collect::<R<_>>()?)),
        "Date" | "Period" | "ExchangeDate" => return Ok(Kind::Types(vec!["xs:dateTime".into()])),
        "Recorder" if class.ends_with("Register") => {
            let docs: Vec<String> = recorders(x, main).iter().map(|k| reft(k)).collect::<R<_>>()?;
            if docs.is_empty() {
                return Err(format!("регистратор {main}: документов-регистраторов нет"));
            }
            return Ok(Kind::Types(docs));
        }
        // строка, число, булево, момент времени, описание типов: таблиц нет, даты нет
        "DeletionMark" | "Predefined" | "PredefinedDataName" | "Code" | "Description" | "IsFolder" | "Number" | "Posted" | "PointInTime" | "LineNumber" | "Active" | "ThisNode" | "SentNo" | "ReceivedNo" | "Order" | "Completed" | "Started" | "Executed" | "ValueType" | "DataVersion" => return Ok(Kind::Types(Vec::new())),
        // вид движения регистра накопления — системное перечисление, детей нет
        "RecordType" if class == "AccumulationRegister" => return Ok(Kind::Types(Vec::new())),
        // тип документа журнала — значение типа (basic FUN_1801ccf80: −0xea63 Type), полей нет
        "Type" if class == "DocumentJournal" => return Ok(Kind::Types(Vec::new())),
        n if STD.iter().any(|(e, _)| *e == n) => return Err(format!("стандартное поле {n} ({class}): дети узла не выведены")),
        _ => {}
    }
    if let Some((k, _)) = o.kids.iter().find(|(_, n)| n == name) {
        return match k.as_str() {
            "TabularSection" => Ok(Kind::Ts(format!("{main}.TabularSection.{name}"))),
            "Attribute" | "Dimension" | "Resource" | "AddressingAttribute" => Ok(Kind::Types(attr_types(x, &format!("{main}.{k}.{name}"))?)),
            // графа журнала — типы реквизитов, на которые она ссылается (как `IndexMeta::field`)
            "Column" => {
                let refs = x.column_refs.get(&format!("{main}.Column.{name}")).filter(|r| !r.is_empty()).ok_or_else(|| format!("графа {name}: ссылок на реквизиты нет"))?;
                let mut v: Vec<String> = Vec::new();
                for r in refs {
                    for t in attr_types(x, r)? {
                        if !v.contains(&t) {
                            v.push(t);
                        }
                    }
                }
                Ok(Kind::Types(v))
            }
            k => Err(format!("{k} {name}: дети узла не выведены")),
        };
    }
    Ok(Kind::Types(attr_types(x, &format!("CommonAttribute.{name}"))?))
}

/// Регистратор регистра `reg` — ссылки на документы, в движениях которых он есть (по имени).
fn recorders(x: &Index, reg: &str) -> Vec<String> {
    let mut v: Vec<String> = x.objects.iter().filter(|(k, o)| k.starts_with("Document.") && o.props.get("RegisterRecords").is_some_and(|r| r.lines().any(|l| l == reg))).map(|(k, _)| k.clone()).collect();
    v.sort();
    v
}

/// Тип значения субконто регистра бухгалтерии `reg`: тип значений плана видов характеристик — видов субконто его плана
/// счетов (поле запроса не сужено видом субконто счета).
fn ext_dim_types(x: &Index, reg: &str) -> Vec<crate::query::resolve::Ty> {
    let cct = x.objects.get(reg).and_then(|o| o.props.get("ChartOfAccounts")).and_then(|c| x.objects.get(c)).and_then(|c| c.props.get("ExtDimensionTypes"));
    cct.map_or(vec![crate::query::resolve::Ty::Unknown], |c| meta_types(x, c))
}

/// Поля виртуальной таблицы «ДвиженияССубконто» регистра бухгалтерии: (язык 0, язык 1, тип) — accnt FUN_1800537b0
/// (поле по номеру — FUN_18004c340, имена номеров — FUN_18036c540): −2 Period, −0x1e PeriodAdjustment (длина уточнения
/// периода `vt+0x230` ≠ 0), −3 Recorder, −4 LineNumber, −5 Active; с корреспонденцией (`vt+0x220`) — −6 AccountDr,
/// по каждому субконто (максимум субконто плана, `+0x90`) ExtDimensionDr и ExtDimensionTypeDr, −7 AccountCr и то же
/// Cr; без корреспонденции — −10 Account, ExtDimension и ExtDimensionType; измерения, ресурсы (с корреспонденцией
/// небалансовые, `+0x388` ложь, — парой Dr/Cr, суффиксы Дт/Кт), реквизиты (FUN_1800d1ad0), общие реквизиты режима 1
/// (FUN_1800d86f0), без корреспонденции — −8 без имени, PointInTime (100), без корреспонденции — RecordType (−9).
/// План счетов не задан (−8 и RecordType без счетов) — `None`.
/// shortcut: маска 0xff общих реквизитов (у basic — 8) не прослежена — взяты реквизиты с разделением DontUse; корень
/// основной таблицы с общим реквизитом — отказ (`root_fields`), имена своего запроса — по этому составу; пересмотреть
/// по FUN_1800d86f0, когда общий реквизит у регистра бухгалтерии встретится
fn records_ext_fields(x: &Index, obj: &str, vt: &str) -> Option<Vec<(String, String, Vec<crate::query::resolve::Ty>)>> {
    use crate::query::resolve::Ty;
    if !obj.starts_with("AccountingRegister.") || !(nocase(vt, "ДвиженияССубконто").is_eq() || nocase(vt, "RecordsWithExtDimensions").is_eq()) {
        return None;
    }
    let o = x.objects.get(obj)?;
    let corr = match o.props.get("Correspondence").map(String::as_str) {
        Some("true") => true,
        Some("false") => false,
        _ => return None,
    };
    let chart = o.props.get("ChartOfAccounts").filter(|c| !c.is_empty())?;
    let c = x.objects.get(chart)?;
    let cct = c.props.get("ExtDimensionTypes").map(|t| vec![Ty::Ref(t.clone())]).unwrap_or(vec![Ty::Unknown]);
    let max = num_prop(c, "MaxExtDimensionCount");
    let f = |e: &str, r: &str, t: Vec<Ty>| (e.to_string(), r.to_string(), t);
    let mut v = vec![f("Period", "Период", vec![Ty::Date])];
    if num_prop(o, "PeriodAdjustmentLength") > 0 {
        v.push(f("PeriodAdjustment", "УточнениеПериода", vec![Ty::Prim]));
    }
    v.extend([f("Recorder", "Регистратор", vec![Ty::Unknown]), f("LineNumber", "НомерСтроки", vec![Ty::Prim]), f("Active", "Активность", vec![Ty::Prim])]);
    let sides: &[(&str, &str)] = if corr { &[("Dr", "Дт"), ("Cr", "Кт")] } else { &[("", "")] };
    for (en, ru) in sides {
        v.push(f(&format!("Account{en}"), &format!("Счет{ru}"), vec![Ty::Ref(chart.clone())]));
        for i in 1..=max {
            v.push(f(&format!("ExtDimension{en}{i}"), &format!("Субконто{ru}{i}"), ext_dim_types(x, obj)));
            v.push(f(&format!("ExtDimensionType{en}{i}"), &format!("ВидСубконто{ru}{i}"), cct.clone()));
        }
    }
    for kind in ["Dimension", "Resource", "Attribute"] {
        for (_, n) in o.kids.iter().filter(|(k, _)| k == kind) {
            let full = format!("{obj}.{kind}.{n}");
            let t = || if kind == "Resource" { vec![Ty::Prim] } else { meta_types(x, &full) };
            if corr && kind != "Attribute" && x.unbalanced.contains(&full) {
                v.push(f(&format!("{n}Dr"), &format!("{n}Дт"), t()));
                v.push(f(&format!("{n}Cr"), &format!("{n}Кт"), t()));
            } else {
                v.push(f(n, n, t()));
            }
        }
    }
    for (n, _) in crate::query::roles::common_fields(x, obj).into_iter().filter(|(_, key)| !key) {
        v.push(f(n, n, meta_types(x, &format!("CommonAttribute.{n}"))));
    }
    v.push(f("PointInTime", "МоментВремени", vec![Ty::Prim]));
    if !corr {
        v.push(f("RecordType", "ВидДвижения", vec![Ty::Prim]));
    }
    Some(v)
}

/// Поля виртуальной таблицы «ВидыСубконто» плана счетов (язык 0, язык 1); `None` — таблица не она.
/// shortcut: состав по справке платформы и сверке ФормаСчета БП (ВидСубконто, ТолькоОбороты, признак учета субконто)
fn ext_dimension_fields(x: &Index, obj: &str, vt: &str) -> Option<Vec<(String, String)>> {
    if !obj.starts_with("ChartOfAccounts.") || !(nocase(vt, "ВидыСубконто").is_eq() || nocase(vt, "ExtDimensionTypes").is_eq()) {
        return None;
    }
    let mut v: Vec<(String, String)> = [("Ref", "Ссылка"), ("LineNumber", "НомерСтроки"), ("ExtDimensionType", "ВидСубконто"), ("TurnoversOnly", "ТолькоОбороты")].iter().map(|(e, r)| (e.to_string(), r.to_string())).collect();
    v.extend(x.objects.get(obj)?.kids.iter().filter(|(k, _)| k == "ExtDimensionAccountingFlag").map(|(_, n)| (n.clone(), n.clone())));
    Some(v)
}

/// Виртуальная таблица задачи «ЗадачиПоИсполнителю».
fn is_tasks_by_executive(obj: &str, vt: &str) -> bool {
    obj.starts_with("Task.") && (nocase(vt, "ЗадачиПоИсполнителю").is_eq() || nocase(vt, "TasksByExecutive").is_eq())
}

/// Источники своего запроса: псевдоним → класс и объект (`Catalog.Банки`), только таблицы объектов
fn walk<'a>(x: &Index, src: &'a crate::query::Source, out: &mut Vec<(&'a crate::query::Source, Option<(String, String)>)>) {
    let t = match &src.table {
        // критерий отбора `КритерийОтбора.Имя[(&Значение)]` — таблица критерия (resolve: Src::Vt без имени)
        crate::query::Table::Name { path, .. } if path.len() == 2 && (nocase(&path[0], "КритерийОтбора").is_eq() || nocase(&path[0], "FilterCriterion").is_eq()) => {
            obj_key(x, &format!("FilterCriterion.{}", path[1])).map(|o| ("FilterCriterion".to_string(), o))
        }
        crate::query::Table::Name { path, params: None } if path.len() == 2 => query_class(&path[0]).and_then(|c| obj_key(x, &format!("{c}.{}", path[1])).map(|o| (c.to_string(), o))),
        // задачи по исполнителю — таблица задачи (поля те же)
        crate::query::Table::Name { path, .. } if path.len() == 3 => query_class(&path[0]).and_then(|c| obj_key(x, &format!("{c}.{}", path[1])).filter(|o| is_tasks_by_executive(o, &path[2])).map(|o| (c.to_string(), o))),
        crate::query::Table::Group(g) => {
            walk(x, g, out);
            None
        }
        _ => None,
    };
    out.push((src, t));
    // соединения, в том числе необязательные компоновки `{… СОЕДИНЕНИЕ …}`: поля корня дает только список выборки
    for j in &src.joins {
        walk(x, &j.source, out);
    }
}

/// Поля корня своего запроса `Псевдоним.Поле` таблицы объекта (последний запрос, первая часть): имя в корне на языке
/// запроса, объект, имя поля объекта (стандартное — английское) — для типов узла при раскрытии; литерал — объект пуст,
/// вместо поля — тип XML; прочие поля — нет.
fn query_origins(x: &Index, text: &str) -> R<Vec<(String, String, String)>> {
    let b = crate::query::parse(text)?;
    let Some(s) = b.queries.last().and_then(|q| q.parts.first()) else { return Ok(Vec::new()) };
    let mut srcs = Vec::new();
    for f in &s.from {
        walk(x, f, &mut srcs);
    }
    // литерал (число, строка, булево; `ВЫБОР` с ветвями одного вида) без ОБЪЕДИНИТЬ — примитивный тип, объект пуст
    // `ЗНАЧЕНИЕ(Справочник.Х.Элемент)` ссылочного класса — ссылка `Х`
    fn lit(x: &Index, e: &crate::query::Expr) -> Option<String> {
        use crate::query::Expr;
        match e {
            Expr::Num(_) | Expr::Neg(_) => Some("xs:decimal".into()),
            Expr::Str(_) => Some("xs:string".into()),
            Expr::Const("True" | "False") => Some("xs:boolean".into()),
            Expr::Value(p) if p.len() == 3 => {
                let c = query_class(&p[0]).filter(|c| REF_TABLES.contains(c))?;
                let o = obj_key(x, &format!("{c}.{}", p[1]))?;
                Some(format!("cfg:{c}Ref.{}", o.split_once('.')?.1))
            }
            Expr::Case { when, other, .. } => {
                let mut v = when.iter().map(|(_, t)| lit(x, t)).chain(other.iter().map(|o| lit(x, o)));
                let first = v.next()??;
                v.all(|t| t.as_ref() == Some(&first)).then_some(first)
            }
            _ => None,
        }
    }
    let single = b.queries.last().is_some_and(|q| q.parts.len() == 1);
    let mut out = Vec::new();
    for f in &s.fields {
        if let (Some(t), Some(a), true) = (lit(x, &f.expr), &f.alias, single) {
            out.push((a.clone(), String::new(), t));
            continue;
        }
        // вложенная табличная часть `Т.ТЧ.(…)` — узел табличной части, если выбраны все ее реквизиты (дети — как у нее:
        // СведенияОЗастрахованномЛицеФСС БП); выбраны не все — состав детей не выведен
        let (p, nested) = match &f.expr {
            crate::query::Expr::Path(p) => (p, None),
            crate::query::Expr::Nested(p, fs) if f.alias.is_some() => (p, Some(fs)),
            _ => continue,
        };
        let [a, n] = p.as_slice() else { continue };
        let Some((_, obj)) = srcs.iter().find(|(s, _)| s.alias.as_deref().is_some_and(|x| nocase(x, a).is_eq())).and_then(|(_, t)| t.clone()) else { continue };
        if let Some(fs) = nested {
            let pre = format!("{obj}.TabularSection.{n}.Attribute.");
            let picked = |k: &str| fs.iter().any(|g| matches!(&g.expr, crate::query::Expr::Path(q) if q.last().is_some_and(|l| nocase(l, k).is_eq())));
            if !x.attr_types.keys().filter_map(|k| k.strip_prefix(&pre)).all(picked) {
                continue;
            }
        }
        // свое поле объекта — раньше стандартного с тем же русским именем (измерение Владелец регистра сведений —
        // не Owner: ДанныеБизнесПроцессов БП; измерение ВидДвижения — не RecordType: ПравилаУточненияСчетовВМеждународномУчете
        // ERP); у класса со стандартным полем такое имя платформа не допускает
        // только поля (значения перечисления и команды в kids — не поля выборки)
        let field = |k: &str| matches!(k, "Attribute" | "Dimension" | "Resource" | "TabularSection" | "AddressingAttribute" | "AccountingFlag" | "Column");
        let own = x.objects.get(&obj).and_then(|o| o.kids.iter().find(|(c, k)| field(c) && nocase(k, n).is_eq()).map(|(_, k)| k.clone()));
        let en = own.or_else(|| STD.iter().find(|(e, r)| nocase(e, n).is_eq() || nocase(r, n).is_eq()).map(|(e, _)| e.to_string())).unwrap_or(n.clone());
        out.push((f.alias.clone().unwrap_or(n.clone()), obj, en));
    }
    Ok(out)
}

/// Типы выходных полей своего запроса (последний запрос): имя — как в первой части, типы — объединение по позиции
/// всех частей ОБЪЕДИНИТЬ (тип поля объединения — все типы его частей). Не проверенный запрос — пусто. `untyped` —
/// параметры, объявленные в схеме без типа (`Untyped`).
fn query_types(x: &Index, text: &str, untyped: &[String]) -> crate::query::resolve::Out {
    let Ok(parts) = crate::query::parse(text).and_then(|b| crate::query::resolve::check_parts(&Untyped(IndexMeta(x), untyped), &b)) else { return Vec::new() };
    let mut out = parts.first().cloned().unwrap_or_default();
    for p in parts.iter().skip(1) {
        for (o, (_, t)) in out.iter_mut().zip(p) {
            match (&mut o.1, t) {
                (Some(a), Some(t)) => a.extend(t.iter().filter(|x| !a.contains(x)).cloned().collect::<Vec<_>>()),
                (a, _) => *a = None,
            }
        }
    }
    out
}

/// Типы XML поля по типам запроса: ссылка — `cfg:<класс>Ref.<имя>`, дата — `xs:dateTime`, прочие примитивы — лист.
fn xml_types(t: &[crate::query::resolve::Ty]) -> R<Vec<String>> {
    use crate::query::resolve::Ty;
    t.iter()
        .map(|t| match t {
            Ty::Ref(o) => o.split_once('.').map(|(c, n)| format!("cfg:{c}Ref.{n}")).ok_or_else(|| format!("ссылка {o}")),
            Ty::Date => Ok("xs:dateTime".into()),
            Ty::Prim | Ty::Uuid => Ok("xs:string".into()),
            Ty::Storage => Ok("v8:ValueStorage".into()),
            Ty::Table | Ty::Unknown => Err("тип не выведен".into()),
        })
        .collect()
}

/// Дети узла дерева полей списка по порядку — как их дает источник полей компоновки (dcs.dll, режим 9):
/// - поле с типами: dcs FUN_180300a70 (источник полей набора-запроса, слот +0x20 vtable 0x1805a3cf0) → FUN_180304e80
///   (таблицы по типам поля, +0x30 метаданных запроса) → FUN_180301cf0 (поля каждой таблицы `ede4bd22` +0x40 по
///   порядку, кроме Ref (+0x38) и Presentation (+0x58) и полей с пустым именем; поля таблицы те же, что берет генератор
///   текста корня, `root_fields`, плюс DataVersion (+0x60), которого генератор не берет); табличная часть — только если
///   у родителя нет признаков +0x228 (поле табличной части), +0x230 (FUN_1802fe440: таблиц у типов больше одной) и
///   +0x91; одноименные поля разных таблиц — одно (FUN_180061190: фильтр +0x20 FUN_1800aaea0 по имени);
/// - порядок — дерево полей компоновщика (+0x40, ключ — полный путь, сравнение без учета регистра; FUN_1800aebd0 обходит
///   его по порядку, сортировки FUN_1800d4ce0 при раскрытии нет — глубина 1 из mngbase FUN_1802b76a0);
/// - у даты за ними — BeginDates, EndDates, DateParts (dcs FUN_1800b29c0 → FUN_1800b2f50, после полей дерева).
/// Поля характеристик (FUN_180305770, провайдер характеристик FUN_1802ec3b0) берутся из данных информационной базы —
/// у загрузки их нет. Табличная часть основной таблицы: `LineNumber`, `Ref` (ссылка владельца: фильтр +0x38 ее не
/// убирает — номера FieldsMap СписокСтатейЗатрат… и ВыгрузкаРегламентированныхОтчетов БП) и реквизиты.
fn children(x: &Index, kind: &Kind, in_ts: bool) -> R<Vec<Field>> {
    let fld = |n: &str, r: &str| Field { name: n.to_string(), second: r.to_string() };
    let mut v: Vec<Field> = Vec::new();
    let mut date = false;
    match kind {
        Kind::Ts(full) => {
            v.extend([fld("LineNumber", "НомерСтроки"), fld("Ref", "Ссылка")]);
            let p = format!("{full}.Attribute.");
            v.extend(x.attr_types.keys().filter_map(|k| k.strip_prefix(&p)).map(|n| fld(n, n)));
        }
        Kind::Params(names) => v.extend(names.iter().map(|n| fld(n, n))),
        Kind::Types(types) => {
            let mut tables = Vec::new();
            for t in types {
                match t.as_str() {
                    "xs:dateTime" => date = true,
                    // хранилище значения — лист (сверка ОчередьСообщенийВЕТИС ЭДО)
                    "xs:string" | "xs:decimal" | "xs:boolean" | "v8:ValueStorage" | "v8:UUID" => {}
                    _ => {
                        let r = t.strip_prefix("cfg:").and_then(|r| r.split_once('.')).and_then(|(k, n)| Some((k.strip_suffix("Ref")?, n)));
                        match r {
                            Some((c, n)) if REF_TABLES.contains(&c) || c == "ChartOfAccounts" => tables.push(format!("{c}.{n}")),
                            _ => return Err(format!("тип {t}: дети узла дерева полей не выведены")),
                        }
                    }
                }
            }
            let no_ts = in_ts || tables.len() > 1;
            for tb in &tables {
                let o = x.objects.get(tb).ok_or_else(|| format!("нет таблицы {tb}"))?;
                // стандартные табличные части плана видов расчета (`calc_std_ts`) — тоже табличные части
                let ts = |n: &str| o.kids.iter().any(|(k, m)| k == "TabularSection" && m == n) || calc_std_ts(x, tb, n).is_some();
                let mut f: Vec<Field> = root_fields(x, tb)?.into_iter().filter(|f| f.name != "Ref" && !(no_ts && ts(&f.name))).collect();
                // DataVersion — пара терминов backend 0x18197f508 (общий создатель поля FUN_180188940)
                if !tb.starts_with("Enum.") {
                    f.push(fld("DataVersion", "ВерсияДанных"));
                }
                for n in f {
                    if !v.iter().any(|m| nocase(&m.name, &n.name).is_eq()) {
                        v.push(n);
                    }
                }
            }
        }
    }
    if let Some(n) = v.iter().find(|n| !nocase_ok(&n.name)) {
        return Err(format!("поле {}: сравнение без учета регистра вне ASCII и кириллицы не воспроизведено", n.name));
    }
    v.sort_by(|a, b| nocase(&a.name, &b.name));
    // имена — пары dcs 0x180750248…0x180750270; элементы частей даты dcs FUN_1800b3ad0 дает с байтом +0x71, при
    // совместимости < 0x139bf такие mngbase FUN_1802b76a0 пропускает
    if date && !old_schema(x) {
        v.extend([fld("BeginDates", "ДатыНачала"), fld("EndDates", "ДатыКонца"), fld("DateParts", "ЧастиДат")]);
    }
    Ok(v)
}

/// Тип ребенка `k` узла `parent` — для детей следующего уровня: у табличной части LineNumber, Ref (ссылка владельца)
/// и реквизиты; у поля с типами — поле той таблицы типа, где оно есть (в нескольких — отказ), DataVersion — лист.
fn kid_kind(x: &Index, parent: &Kind, k: &str) -> R<Kind> {
    match parent {
        Kind::Ts(full) => match k {
            "LineNumber" => Ok(Kind::Types(Vec::new())),
            "Ref" => {
                let owner = full.split(".TabularSection.").next().unwrap_or_default();
                Ok(Kind::Types(vec![owner.split_once('.').map(|(c, n)| format!("cfg:{c}Ref.{n}")).unwrap_or_default()]))
            }
            _ => Ok(Kind::Types(attr_types(x, &format!("{full}.Attribute.{k}"))?)),
        },
        Kind::Params(_) => Err(format!("тип параметра {k} набора: дети не выведены")),
        Kind::Types(types) => {
            if k == "DataVersion" {
                return Ok(Kind::Types(Vec::new()));
            }
            if matches!(k, "BeginDates" | "EndDates" | "DateParts") {
                return Err(format!("дети поля {k} даты не выведены"));
            }
            let mut found = Vec::new();
            for t in types {
                let Some((c, n)) = t.strip_prefix("cfg:").and_then(|r| r.split_once('.')).and_then(|(c, n)| Some((c.strip_suffix("Ref")?, n))) else { continue };
                let tb = format!("{c}.{n}");
                if root_fields(x, &tb)?.iter().any(|f| f.name == k) {
                    found.push(root_kind(x, &tb, k)?);
                }
            }
            match found.len() {
                1 => Ok(found.pop().unwrap_or(Kind::Types(Vec::new()))),
                0 => Err(format!("поле {k} не найдено в таблицах типа")),
                _ => Err(format!("поле {k} в нескольких таблицах типа: дети не выведены")),
            }
        }
    }
}

/// Класс таблицы запроса по первой части имени (`Справочник` / `Catalog`).
const QUERY_CLASSES: &[(&str, &str)] = &[
    ("Справочник", "Catalog"),
    ("Документ", "Document"),
    ("ПланВидовХарактеристик", "ChartOfCharacteristicTypes"),
    ("РегистрСведений", "InformationRegister"),
    ("РегистрНакопления", "AccumulationRegister"),
    ("Перечисление", "Enum"),
    ("ЖурналДокументов", "DocumentJournal"),
    ("ПланСчетов", "ChartOfAccounts"),
    ("ПланОбмена", "ExchangePlan"),
    ("БизнесПроцесс", "BusinessProcess"),
    ("Задача", "Task"),
    ("РегистрБухгалтерии", "AccountingRegister"),
    ("ПланВидовРасчета", "ChartOfCalculationTypes"),
];

fn query_class(c: &str) -> Option<&'static str> {
    QUERY_CLASSES.iter().find(|(r, e)| r.eq_ignore_ascii_case(c) || nocase(r, c).is_eq() || nocase(e, c).is_eq()).map(|(_, e)| *e)
}

/// Стандартные поля, которые поле выборки без псевдонима называет по-английски (второе имя — русское), по классу
/// таблицы-источника. Список — поля основной таблицы `root_fields` плюс DataVersion и Presentation (в корень
/// основной таблицы их не берет генератор, в своем запросе они бывают).
fn std_of(class: &str) -> &'static [&'static str] {
    match class {
        "Catalog" => &["Ref", "DeletionMark", "Predefined", "PredefinedDataName", "Code", "Description", "Parent", "IsFolder", "Owner", "DataVersion", "Presentation"],
        "ChartOfCharacteristicTypes" => &["Ref", "DeletionMark", "Predefined", "PredefinedDataName", "Code", "Description", "Parent", "IsFolder", "ValueType", "DataVersion", "Presentation"],
        "Document" => &["Ref", "DeletionMark", "Date", "Posted", "PointInTime", "Number", "DataVersion", "Presentation"],
        "InformationRegister" => &["Period", "Recorder", "LineNumber", "Active", "PointInTime"],
        "AccumulationRegister" => &["Period", "Recorder", "LineNumber", "Active", "RecordType", "PointInTime"],
        "Enum" => &["Ref", "Order", "Presentation"],
        // backend FUN_1801872c0 (поля таблицы плана обмена, см. root_fields)
        "ExchangePlan" => &["Ref", "DeletionMark", "ThisNode", "Code", "Description", "SentNo", "ReceivedNo", "ExchangeDate", "DataVersion", "Presentation"],
        // basic FUN_1801ccf80 (имена по классу 4612bd75): −0x65 Ref, −100 Date, −0xea63 Type, −4 DeletionMark, −2 Number,
        // −7 Posted — поля построителя журнала FUN_1801d4c50
        "DocumentJournal" => &["Ref", "Type", "Date", "Number", "Posted", "DeletionMark"],
        // bp FUN_180025280 (имена по классу fcd3404e / 3e63355c) и поля построителей FUN_1800275d0 / FUN_18002d730
        "BusinessProcess" => &["Ref", "DeletionMark", "Number", "Date", "Completed", "HeadTask", "Started", "DataVersion", "Presentation"],
        "Task" => &["Ref", "DeletionMark", "Number", "Date", "BusinessProcess", "RoutePoint", "Description", "Executed", "DataVersion", "Presentation"],
        // как root_fields (номера выгрузки) и ссылочные DataVersion, Presentation
        "ChartOfAccounts" => &["Ref", "DeletionMark", "Predefined", "PredefinedDataName", "Parent", "Type", "OffBalance", "Code", "Description", "Order", "ExtDimensionTypes", "DataVersion", "Presentation"],
        // calc FUN_18031a9b0 (имена по номерам) и поля построителя FUN_180037670
        "ChartOfCalculationTypes" => &["Ref", "DeletionMark", "Predefined", "PredefinedDataName", "Code", "Description", "ActionPeriodIsBasic", "BaseCalculationTypes", "DisplacingCalculationTypes", "LeadingCalculationTypes", "DataVersion", "Presentation"],
        _ => &[],
    }
}

/// Русское имя стандартного поля таблицы класса `class`: у плана счетов Type — «Вид» (у журнала — «Тип»).
fn std_ru_in(class: &str, en: &str) -> Option<&'static str> {
    if class == "ChartOfAccounts" && en == "Type" {
        return Some("Вид");
    }
    std_ru(en)
}

/// Стандартная табличная часть `ts` (имя без учета регистра) плана видов расчета `obj`, если она есть у плана (условия —
/// как у поля корня, `root_fields`): английское имя.
fn calc_std_ts(x: &Index, obj: &str, ts: &str) -> Option<&'static str> {
    const TS: &[&str] = &["BaseCalculationTypes", "DisplacingCalculationTypes", "LeadingCalculationTypes"];
    if !obj.starts_with("ChartOfCalculationTypes.") {
        return None;
    }
    let e = TS.iter().find(|e| nocase(e, ts).is_eq() || std_ru(e).is_some_and(|r| nocase(r, ts).is_eq()))?;
    root_fields(x, obj).ok()?.iter().any(|f| f.name == *e).then_some(*e)
}

/// Поле роли основной таблицы (английское имя стандартного поля).
/// Английское имя стандартного поля по русскому (обратное `std_ru`; `Движения` — RegisterRecords).
pub(super) fn std_en(ru: &str) -> Option<&'static str> {
    if ru == "Движения" {
        return Some("RegisterRecords");
    }
    STD.iter().find(|(_, r)| *r == ru).map(|(e, _)| *e)
}

fn std_ru(en: &str) -> Option<&'static str> {
    STD.iter().find(|(e, _)| *e == en).map(|(_, r)| *r).or(match en {
        "DataVersion" => Some("ВерсияДанных"),
        "Presentation" => Some("Представление"),
        _ => None,
    })
}

/// Метаданные для проверки текста запроса (`query::resolve`) по индексу загрузки.
struct IndexMeta<'a>(&'a Index);

/// Метаданные проверки, у которых параметр, объявленный в схеме без типа (`<Parameter>` без `valueType`), — выражение без
/// типов: тип параметра набора — тип параметра запроса, объявленный пустой его не меняет (dcs FUN_1800ba550: +0x50
/// объявления пуст — тип +0x48 записи запроса). shortcut: тип самого выражения `&Имя` в тексте (entext) не прослежен —
/// пустой по двум формам ERP (СменноеЗадание ФормаДокумента, ОтчетКомитенту ФормаПодбораДокументаПродажи: номера
/// FieldsMap); объявленный с типом и необъявленный — не выведены. Только параметр, который во всем пакете стоит лишь
/// целым выражением поля выборки или веткой ТОГДА/ИНАЧЕ (`whole_only`): тип параметра запроса entext выводит из его
/// употребления (`Т.Дата <= &П`, `В (&П)`), такой — не выведен
struct Untyped<'a>(IndexMeta<'a>, &'a [String]);

/// Параметр `name` стоит в пакете `b` (текст `text`) только целым выражением поля выборки запроса пакета или веткой
/// ТОГДА/ИНАЧЕ такого выражения: все вхождения `&name` текста — эти.
fn whole_only(text: &str, b: &crate::query::Batch, name: &str) -> bool {
    use crate::query::Expr;
    fn n(e: &Expr, name: &str) -> usize {
        match e {
            Expr::Param(p) => usize::from(nocase(p, name).is_eq()),
            Expr::Case { when, other, .. } => when.iter().map(|(_, t)| n(t, name)).sum::<usize>() + other.as_deref().map_or(0, |o| n(o, name)),
            _ => 0,
        }
    }
    let Ok(t) = crate::query::lex::tokens(text) else { return false };
    let all = t.iter().filter(|t| matches!(t, crate::query::lex::Tok::Param(p) if nocase(p, name).is_eq())).count();
    all == b.queries.iter().flat_map(|q| &q.parts).flat_map(|s| &s.fields).map(|f| n(&f.expr, name)).sum::<usize>()
}

impl crate::query::resolve::Meta for Untyped<'_> {
    fn object(&self, obj: &str) -> Option<String> {
        self.0.object(obj)
    }
    fn field(&self, obj: &str, name: &str) -> R<Option<Vec<crate::query::resolve::Ty>>> {
        self.0.field(obj, name)
    }
    fn ts(&self, obj: &str, ts: &str) -> bool {
        self.0.ts(obj, ts)
    }
    fn ts_field(&self, obj: &str, ts: &str, name: &str) -> R<Option<Vec<crate::query::resolve::Ty>>> {
        self.0.ts_field(obj, ts, name)
    }
    fn vt_field(&self, obj: &str, vt: &str, name: &str) -> R<Option<Vec<crate::query::resolve::Ty>>> {
        self.0.vt_field(obj, vt, name)
    }
    fn vt_param(&self, obj: &str, vt: &str, name: &str) -> R<Option<Vec<crate::query::resolve::Ty>>> {
        self.0.vt_param(obj, vt, name)
    }
    fn value(&self, obj: &str, name: &str) -> bool {
        self.0.value(obj, name)
    }
    fn param(&self, name: &str) -> Option<Vec<crate::query::resolve::Ty>> {
        self.1.iter().any(|n| nocase(n, name).is_eq()).then(Vec::new)
    }
}

fn meta_types(x: &Index, key: &str) -> Vec<crate::query::resolve::Ty> {
    use crate::query::resolve::Ty;
    // наборы «любая ссылка класса» (`cfg:DocumentRef`) — ссылки на все объекты класса; прочие наборы (кроме определяемого
    // типа, `query::settings::types_of`) — тип не выведен
    let mut sets: Vec<Ty> = Vec::new();
    for t in x.type_sets.get(key).into_iter().flatten() {
        if t.starts_with("cfg:DefinedType.") {
            continue;
        }
        let Some(c) = t.strip_prefix("cfg:").and_then(|c| c.strip_suffix("Ref")).filter(|c| REF_TABLES.contains(c) || *c == "ChartOfAccounts") else { return vec![Ty::Unknown] };
        let mut objs: Vec<&String> = x.objects.keys().filter(|k| k.split_once('.').is_some_and(|(k, _)| k == c)).collect();
        objs.sort();
        sets.extend(objs.into_iter().map(|o| Ty::Ref(o.clone())));
    }
    let Some(v) = crate::query::settings::types_of(x, key) else { return if sets.is_empty() { vec![Ty::Unknown] } else { sets } };
    let mut v: Vec<Ty> = v.into_iter()
        .map(|t| {
            let refs = [
                ("cfg:CatalogRef.", "Catalog"),
                ("cfg:DocumentRef.", "Document"),
                ("cfg:EnumRef.", "Enum"),
                ("cfg:ChartOfCharacteristicTypesRef.", "ChartOfCharacteristicTypes"),
                ("cfg:ChartOfAccountsRef.", "ChartOfAccounts"),
                ("cfg:ChartOfCalculationTypesRef.", "ChartOfCalculationTypes"),
                ("cfg:ExchangePlanRef.", "ExchangePlan"),
                ("cfg:BusinessProcessRef.", "BusinessProcess"),
                ("cfg:TaskRef.", "Task"),
            ];
            if let Some((c, n)) = refs.iter().find_map(|(p, c)| t.strip_prefix(p).map(|n| (c, n))) {
                return Ty::Ref(format!("{c}.{n}"));
            }
            if t == "v8:ValueStorage" {
                return Ty::Storage;
            }
            if t == "v8:UUID" {
                return Ty::Uuid;
            }
            if t == "xs:dateTime" {
                return Ty::Date;
            }
            if t.starts_with("xs:") || t == "v8:Null" {
                return Ty::Prim;
            }
            Ty::Unknown
        })
        .collect();
    v.extend(sets);
    v
}

/// Поля периода виртуальных таблиц оборотов (справка платформы: «Обороты», «ОстаткиИОбороты»), тип — дата.
const PERIOD_FIELDS: &[&str] = &[
    "ПериодСекунда", "ПериодМинута", "ПериодЧас", "ПериодДень", "ПериодНеделя", "ПериодДекада", "ПериодМесяц", "ПериодКвартал", "ПериодПолугодие", "ПериодГод",
    "SecondPeriod", "MinutePeriod", "HourPeriod", "DayPeriod", "WeekPeriod", "TenDaysPeriod", "MonthPeriod", "QuarterPeriod", "HalfYearPeriod", "YearPeriod",
];

/// Ключ `x.full` с последней частью `name` без учета регистра: точное имя, иначе перебор.
fn full_key(x: &Index, prefix: &str, name: &str) -> Option<String> {
    let k = format!("{prefix}{name}");
    if x.full.contains_key(&k) {
        return Some(k);
    }
    x.full.keys().find(|k| k.strip_prefix(prefix).is_some_and(|n| !n.contains('.') && nocase(n, name).is_eq())).cloned()
}

/// Ключ объекта `Catalog.Банки` в индексе: имя в запросе без учета регистра.
fn obj_key(x: &Index, obj: &str) -> Option<String> {
    if x.objects.contains_key(obj) {
        return Some(obj.to_string());
    }
    x.objects.keys().find(|k| nocase(k, obj).is_eq()).cloned()
}

impl crate::query::resolve::Meta for IndexMeta<'_> {
    fn object(&self, obj: &str) -> Option<String> {
        obj_key(self.0, obj)
    }

    fn field(&self, obj: &str, name: &str) -> R<Option<Vec<crate::query::resolve::Ty>>> {
        use crate::query::resolve::Ty;
        let x = self.0;
        let Some(o) = x.objects.get(obj) else { return Ok(None) };
        let class = obj.split('.').next().unwrap_or_default();
        // состав из кода построителя (roles::table_fields): поля нет — `Ok(None)` (запрос не компилируется наверняка);
        // у классов без выведенного состава отсутствующее поле — отказ (Err в конце)
        let tf = crate::query::roles::table_fields(x, obj);
        if let Some(t) = &tf {
            let is = |e: &str, r: &str| nocase(e, name).is_eq() || nocase(r, name).is_eq();
            if t.unsure.iter().any(|(e, r)| is(e, r)) {
                return Err(format!("поле {name} таблицы {obj}: наличие не выведено"));
            }
            let Some((e, _, _)) = t.fields.iter().find(|(e, r, _)| is(e, r)) else { return Ok(None) };
            // константа (roles::table_fields): Value — тип константы, прочее — разделители
            if class == "Constant" {
                return Ok(Some(meta_types(x, &if e == "Value" { obj.to_string() } else { format!("CommonAttribute.{e}") })));
            }
        }
        // стандартные поля — как у корня основной таблицы (root_fields), плюс DataVersion и Presentation ссылочных
        let mut std: Vec<String> = root_fields(x, obj)?.into_iter().filter(|f| STD.iter().any(|(e, _)| *e == f.name)).map(|f| f.name).collect();
        if matches!(class, "Catalog" | "Document" | "ChartOfCharacteristicTypes" | "ExchangePlan" | "BusinessProcess" | "Task" | "ChartOfAccounts" | "ChartOfCalculationTypes") {
            std.extend(["DataVersion".to_string(), "Presentation".to_string()]);
        }
        if class == "Enum" {
            std.push("Presentation".into());
        }
        if let Some(e) = std.iter().find(|e| nocase(e, name).is_eq() || std_ru_in(class, e).is_some_and(|r| nocase(r, name).is_eq())) {
            return Ok(Some(match e.as_str() {
                "Ref" if class == "DocumentJournal" => crate::query::roles::journal_ref(x, obj),
                "Ref" | "Parent" => vec![Ty::Ref(obj.to_string())],
                "Owner" => o.owner_list.iter().map(|w| Ty::Ref(w.clone())).collect(),
                // регистратор — ссылки документов-регистраторов (`recorders`: полный состав по движениям документов
                // индекса); поля пути нет ни у одного — запрос не компилируется
                "Recorder" if class.ends_with("Register") => {
                    let docs = recorders(x, obj);
                    if docs.is_empty() { vec![Ty::Unknown] } else { docs.into_iter().map(Ty::Ref).collect() }
                }
                "Recorder" | "ValueType" | "Type" | "PointInTime" | "BusinessProcess" | "RoutePoint" | "HeadTask" => vec![Ty::Unknown],
                "Date" | "Period" | "ExchangeDate" => vec![Ty::Date],
                // виды субконто плана счетов — стандартная табличная часть
                "ExtDimensionTypes" => vec![Ty::Table],
                _ => vec![Ty::Prim],
            }));
        }
        for (k, n) in &o.kids {
            if !nocase(n, name).is_eq() {
                continue;
            }
            match k.as_str() {
                "TabularSection" => return Ok(Some(vec![Ty::Table])),
                "Attribute" | "Dimension" | "Resource" | "AddressingAttribute" => return Ok(Some(meta_types(x, &format!("{obj}.{k}.{n}")))),
                // признак учета счета — булево
                "AccountingFlag" => return Ok(Some(vec![Ty::Prim])),
                // графа журнала — типы реквизитов, на которые она ссылается; ссылка не на реквизит — тип не выведен
                "Column" => {
                    let refs = x.column_refs.get(&format!("{obj}.Column.{n}")).filter(|r| !r.is_empty() && r.iter().all(|r| x.attr_types.contains_key(r)));
                    let mut v: Vec<Ty> = Vec::new();
                    for t in refs.into_iter().flatten().flat_map(|r| meta_types(x, r)) {
                        if !v.contains(&t) {
                            v.push(t);
                        }
                    }
                    return Ok(Some(if v.is_empty() { vec![Ty::Unknown] } else { v }));
                }
                _ => {}
            }
        }
        // у перечисления полей таблицы только Ref и Order (basic FUN_1801ccf80) — общих реквизитов нет
        if class == "Enum" {
            return Ok(None);
        }
        for (n, auto, content) in &x.common {
            if !nocase(n, name).is_eq() {
                continue;
            }
            let used = match content.get(obj).map(String::as_str) {
                Some("Use") => true,
                Some("DontUse") => false,
                _ if class == "DocumentJournal" && auto == "Use" => return Err(format!("общий реквизит {n} журнала документов в своем запросе не выведен")),
                _ => auto == "Use",
            };
            if used {
                return Ok(Some(meta_types(x, &format!("CommonAttribute.{n}"))));
            }
        }
        Err(match tf {
            Some(_) => format!("поле {name} таблицы {obj}: в составе есть, тип не выведен"),
            None => format!("поле {name} не найдено в {obj}, состав полей таблицы класса {class} не выведен"),
        })
    }

    fn ts(&self, obj: &str, ts: &str) -> bool {
        self.0.objects.get(obj).is_some_and(|o| o.kids.iter().any(|(k, n)| k == "TabularSection" && nocase(n, ts).is_eq())) || calc_std_ts(self.0, obj, ts).is_some()
    }

    fn ts_field(&self, obj: &str, ts: &str, name: &str) -> R<Option<Vec<crate::query::resolve::Ty>>> {
        use crate::query::resolve::Ty;
        let x = self.0;
        // стандартная табличная часть плана видов расчета: calc FUN_180039900 (ветки −10, −0x14, −0x1e) — −6 Ref (ссылка
        // плана), −100 LineNumber (число 5,0), −0x65 CalculationType (тип — базовые планы +0x398 / свой +0x3e0 интерфейса
        // плана: не выведен), −0x66 Predefined (булево); имена — FUN_18031a9b0
        if calc_std_ts(x, obj, ts).is_some() {
            let is = |e: &str| nocase(name, e).is_eq() || std_ru(e).is_some_and(|r| nocase(name, r).is_eq()) || (e == "CalculationType" && nocase(name, "ВидРасчета").is_eq());
            return Ok(match ["Ref", "LineNumber", "CalculationType", "Predefined"].into_iter().find(|e| is(e)) {
                Some("Ref") => Some(vec![Ty::Ref(obj.to_string())]),
                Some("CalculationType") => Some(vec![Ty::Unknown]),
                Some(_) => Some(vec![Ty::Prim]),
                None => None,
            });
        }
        let Some(tn) = x.objects.get(obj).and_then(|o| o.kids.iter().find(|(k, n)| k == "TabularSection" && nocase(n, ts).is_eq())).map(|(_, n)| n.clone()) else { return Ok(None) };
        if nocase(name, "Ссылка").is_eq() || nocase(name, "Ref").is_eq() {
            return Ok(Some(vec![Ty::Ref(obj.to_string())]));
        }
        if nocase(name, "НомерСтроки").is_eq() || nocase(name, "LineNumber").is_eq() {
            return Ok(Some(vec![Ty::Prim]));
        }
        match full_key(x, &format!("{obj}.TabularSection.{tn}.Attribute."), name) {
            Some(k) => Ok(Some(meta_types(x, &k))),
            None => Err(format!("поле {name} не найдено в табличной части {obj}.{tn}, состав ее полей не выведен")),
        }
    }

    fn vt_field(&self, obj: &str, vt: &str, name: &str) -> R<Option<Vec<crate::query::resolve::Ty>>> {
        use crate::query::resolve::Ty;
        let x = self.0;
        // таблица констант: поле — константа, тип — ее тип
        if obj == crate::query::resolve::CONSTANTS {
            if name.is_empty() {
                return Ok(None);
            }
            return Ok(x.objects.keys().find(|k| k.strip_prefix("Constant.").is_some_and(|n| nocase(n, name).is_eq())).map(|k| meta_types(x, k)));
        }
        let o = x.objects.get(obj).ok_or_else(|| format!("нет таблицы {obj}"))?;
        // критерий отбора: поле Ссылка — ссылки на объекты его состава
        if obj.starts_with("FilterCriterion.") {
            if name.is_empty() {
                return Ok(None);
            }
            if !nocase(name, "Ссылка").is_eq() && !nocase(name, "Ref").is_eq() {
                return Err(format!("поле {name} критерия отбора {obj} не выведено"));
            }
            let mut v: Vec<String> = o.props.get("Content").into_iter().flat_map(|c| c.lines()).map(|i| i.splitn(3, '.').take(2).collect::<Vec<_>>().join(".")).collect();
            v.sort();
            v.dedup();
            return Ok(Some(v.into_iter().map(Ty::Ref).collect()));
        }
        // остатки регистра остатков: измерения и `<ресурс>Остаток` (состав полей — справка платформы, «Остатки»)
        if obj.starts_with("AccumulationRegister.") && (nocase(vt, "Остатки").is_eq() || nocase(vt, "Balance").is_eq()) && o.props.get("RegisterType").is_some_and(|t| t == "Balance") {
            if name.is_empty() {
                return Ok(None);
            }
            if let Some((k, n)) = o.kids.iter().find(|(k, n)| k == "Dimension" && nocase(n, name).is_eq()) {
                return Ok(Some(meta_types(x, &format!("{obj}.{k}.{n}"))));
            }
            if o.kids.iter().any(|(k, n)| k == "Resource" && (nocase(&format!("{n}Остаток"), name).is_eq() || nocase(&format!("{n}Balance"), name).is_eq())) {
                return Ok(Some(vec![Ty::Prim]));
            }
            return Err(format!("поле {name} не найдено в {obj}.{vt}, состав полей остатков не выведен"));
        }
        // остатки регистра бухгалтерии: Счет, Субконто1…N, измерения, `<ресурс>Остаток[Дт|Кт]`,
        // `<ресурс>РазвернутыйОстаток{Дт|Кт}` (справка платформы, «Остатки»). shortcut: суффиксы у всех ресурсов, признаки
        // учета (Balance, AccountingFlag) не учитываются; сверено на ИнвентаризацияРасчетовСКонтрагентами БП
        // субконто регистра бухгалтерии: поля записей субконто (справка платформы, «Субконто»). shortcut: сверено на
        // Номенклатура.ФормаИсторияДвижений БП (Период, Регистратор, Вид, Значение)
        if obj.starts_with("AccountingRegister.") && (nocase(vt, "Субконто").is_eq() || nocase(vt, "ExtDimensions").is_eq()) {
            if name.is_empty() {
                return Ok(None);
            }
            let is = |e: &str, r: &str| nocase(e, name).is_eq() || nocase(r, name).is_eq();
            let cct = o.props.get("ChartOfAccounts").and_then(|c| x.objects.get(c)).and_then(|c| c.props.get("ExtDimensionTypes")).cloned();
            return Ok(Some(if is("Period", "Период") {
                vec![Ty::Date]
            } else if is("LineNumber", "НомерСтроки") || is("Active", "Активность") || is("RecordType", "ВидДвижения") {
                vec![Ty::Prim]
            } else if is("Kind", "Вид") {
                cct.map(|c| vec![Ty::Ref(c)]).unwrap_or(vec![Ty::Unknown])
            } else if is("Value", "Значение") {
                ext_dim_types(x, obj)
            } else if is("Recorder", "Регистратор") {
                vec![Ty::Unknown]
            } else {
                return Err(format!("поле {name} не найдено в {obj}.{vt}, состав полей субконто не выведен"));
            }));
        }
        // остатки и обороты — еще начальный и конечный остаток, обороты и поля периода (справка «ОстаткиИОбороты»)
        let acc_vt = ["Остатки", "Balance", "ОстаткиИОбороты", "BalanceAndTurnovers"].iter().position(|v| nocase(v, vt).is_eq()).map(|i| i >= 2);
        if let Some(turn) = acc_vt.filter(|_| obj.starts_with("AccountingRegister.")) {
            if name.is_empty() {
                return Ok(None);
            }
            if turn && ["Период", "Period", "Регистратор", "Recorder", "НомерСтроки", "LineNumber"].iter().any(|p| nocase(p, name).is_eq()) {
                return Ok(Some(vec![if nocase("Регистратор", name).is_eq() || nocase("Recorder", name).is_eq() { Ty::Unknown } else if nocase("Период", name).is_eq() || nocase("Period", name).is_eq() { Ty::Date } else { Ty::Prim }]));
            }
            if nocase(name, "Счет").is_eq() || nocase(name, "Account").is_eq() {
                return Ok(Some(o.props.get("ChartOfAccounts").map(|c| vec![Ty::Ref(c.clone())]).unwrap_or(vec![Ty::Unknown])));
            }
            let max = o.props.get("ChartOfAccounts").and_then(|c| x.objects.get(c)).map_or(0, |c| num_prop(c, "MaxExtDimensionCount"));
            if (1..=max).any(|i| nocase(&format!("Субконто{i}"), name).is_eq() || nocase(&format!("ExtDimension{i}"), name).is_eq()) {
                return Ok(Some(ext_dim_types(x, obj)));
            }
            if let Some((k, n)) = o.kids.iter().find(|(k, n)| k == "Dimension" && nocase(n, name).is_eq()) {
                return Ok(Some(meta_types(x, &format!("{obj}.{k}.{n}"))));
            }
            let bal = ["Остаток", "ОстатокДт", "ОстатокКт", "РазвернутыйОстатокДт", "РазвернутыйОстатокКт", "Balance", "BalanceDr", "BalanceCr", "ExtendedBalanceDr", "ExtendedBalanceCr"];
            let suf: Vec<String> = if turn {
                let mut v: Vec<String> = ["Начальный", "Конечный"].iter().flat_map(|p| bal[..5].iter().map(move |b| format!("{p}{}", b))).collect();
                v.extend(["Opening", "Closing"].iter().flat_map(|p| bal[5..].iter().map(move |b| format!("{p}{}", b))));
                v.extend(["Оборот", "ОборотДт", "ОборотКт", "Turnover", "TurnoverDr", "TurnoverCr"].iter().map(|s| s.to_string()));
                v
            } else {
                bal.iter().map(|s| s.to_string()).collect()
            };
            if o.kids.iter().any(|(k, n)| k == "Resource" && suf.iter().any(|s| nocase(&format!("{n}{s}"), name).is_eq())) {
                return Ok(Some(vec![Ty::Prim]));
            }
            return Err(format!("поле {name} не найдено в {obj}.{vt}, состав полей остатков не выведен"));
        }
        // остатки и обороты регистра накопления: измерения, поля периода и ресурсы с суффиксами НачальныйОстаток, Оборот,
        // Приход, Расход, КонечныйОстаток (basic.dll FUN_1801e3a20, пары суффиксов 0x1809fb118)
        if obj.starts_with("AccumulationRegister.") && (nocase(vt, "ОстаткиИОбороты").is_eq() || nocase(vt, "BalanceAndTurnovers").is_eq()) {
            if name.is_empty() {
                return Ok(None);
            }
            if let Some((k, n)) = o.kids.iter().find(|(k, n)| k == "Dimension" && nocase(n, name).is_eq()) {
                return Ok(Some(meta_types(x, &format!("{obj}.{k}.{n}"))));
            }
            if ["Период", "Period"].iter().chain(PERIOD_FIELDS).any(|p| nocase(p, name).is_eq()) {
                return Ok(Some(vec![Ty::Date]));
            }
            if ["НомерСтроки", "LineNumber"].iter().any(|p| nocase(p, name).is_eq()) {
                return Ok(Some(vec![Ty::Prim]));
            }
            if nocase("Регистратор", name).is_eq() || nocase("Recorder", name).is_eq() {
                return Ok(Some(vec![Ty::Unknown]));
            }
            let suf = ["НачальныйОстаток", "OpeningBalance", "Оборот", "Turnover", "Приход", "Receipt", "Расход", "Expense", "КонечныйОстаток", "ClosingBalance"];
            if o.kids.iter().any(|(k, n)| k == "Resource" && suf.iter().any(|s| nocase(&format!("{n}{s}"), name).is_eq())) {
                return Ok(Some(vec![Ty::Prim]));
            }
            return Err(format!("поле {name} не найдено в {obj}.{vt}, состав полей остатков и оборотов не выведен"));
        }
        // обороты регистра накопления: измерения и `<ресурс>Оборот`, у регистра остатков еще `Приход` / `Расход`
        // (справка платформы, «Обороты»)
        if obj.starts_with("AccumulationRegister.") && (nocase(vt, "Обороты").is_eq() || nocase(vt, "Turnovers").is_eq()) {
            if name.is_empty() {
                return Ok(None);
            }
            if let Some((k, n)) = o.kids.iter().find(|(k, n)| k == "Dimension" && nocase(n, name).is_eq()) {
                return Ok(Some(meta_types(x, &format!("{obj}.{k}.{n}"))));
            }
            // поля периода — при любой периодичности. shortcut: их набор зависит от периодичности параметра, а
            // vt_field ее не получает; запрос с полем не своей периодичности платформа не компилирует
            if ["Период", "Period"].iter().chain(PERIOD_FIELDS).any(|p| nocase(p, name).is_eq()) {
                return Ok(Some(vec![Ty::Date]));
            }
            if ["НомерСтроки", "LineNumber"].iter().any(|p| nocase(p, name).is_eq()) {
                return Ok(Some(vec![Ty::Prim]));
            }
            if nocase("Регистратор", name).is_eq() || nocase("Recorder", name).is_eq() {
                return Ok(Some(vec![Ty::Unknown]));
            }
            let balance = o.props.get("RegisterType").is_some_and(|t| t == "Balance");
            let suf: &[&str] = if balance { &["Оборот", "Turnover", "Приход", "Receipt", "Расход", "Expense"] } else { &["Оборот", "Turnover"] };
            if o.kids.iter().any(|(k, n)| k == "Resource" && suf.iter().any(|s| nocase(&format!("{n}{s}"), name).is_eq())) {
                return Ok(Some(vec![Ty::Prim]));
            }
            return Err(format!("поле {name} не найдено в {obj}.{vt}, состав полей оборотов не выведен"));
        }
        if let Some(fs) = records_ext_fields(x, obj, vt) {
            if name.is_empty() {
                return Ok(None);
            }
            return fs.into_iter().find(|(e, r, _)| nocase(e, name).is_eq() || nocase(r, name).is_eq()).map(|(_, _, t)| Some(t)).ok_or_else(|| format!("поле {name} не найдено в {obj}.{vt}"));
        }
        // виды субконто счета: Ref, LineNumber, ExtDimensionType, TurnoversOnly, признаки учета субконто
        if let Some(names) = ext_dimension_fields(x, obj, vt) {
            if name.is_empty() {
                return Ok(None);
            }
            let (e, _) = names.into_iter().find(|(e, r)| nocase(e, name).is_eq() || nocase(r, name).is_eq()).ok_or_else(|| format!("поле {name} не найдено в {obj}.{vt}"))?;
            return Ok(Some(match e.as_str() {
                "Ref" => vec![Ty::Ref(obj.to_string())],
                "ExtDimensionType" => o.props.get("ExtDimensionTypes").map(|t| vec![Ty::Ref(t.clone())]).unwrap_or(vec![Ty::Unknown]),
                _ => vec![Ty::Prim],
            }));
        }
        // задачи по исполнителю — поля самой задачи (справка платформы)
        if is_tasks_by_executive(obj, vt) {
            return if name.is_empty() { Ok(None) } else { self.field(obj, name) };
        }
        let periodic = o.props.get("InformationRegisterPeriodicity").is_some_and(|p| p != "Nonperiodical");
        // таблица изменений регистра сведений — basic FUN_1801d98f0: Node и MessageNo (сервис FUN_18028e000 +0xa0 →
        // backend FUN_180486fe0; тип узла — ссылки планов обмена с регистром в составе), при подчинении регистратору
        // (`vt+0x248` = 1) Recorder, иначе Period (периодичность `vt+0x238` ≠ 0 и `vt+0x268`) и измерения с `+0x3c0`,
        // общие реквизиты режима 8 (FUN_1802876b0). Регистра нет ни в одном плане обмена — таблицы нет.
        // shortcut: `vt+0x268` и `+0x3c0` измерения сопоставлены с MainFilterOnPeriod и MainFilter по смыслу — период и
        // измерение вне основного отбора дают отказ, а не «поля нет» (запрос с ним считался бы некомпилируемым, корень —
        // пустым); пересмотреть по чтению слотов, когда такое поле встретится
        if obj.starts_with("InformationRegister.") && (nocase(vt, "Изменения").is_eq() || nocase(vt, "Changes").is_eq()) {
            let mut nodes: Vec<&String> = x.objects.iter().filter(|(k, p)| k.starts_with("ExchangePlan.") && p.props.get("Content").is_some_and(|c| c.lines().any(|l| l == obj))).map(|(k, _)| k).collect();
            nodes.sort();
            if nodes.is_empty() {
                return Err(format!("таблица изменений {obj}.{vt}: регистра нет в составе планов обмена"));
            }
            if name.is_empty() {
                return Ok(None);
            }
            let is = |e: &str, r: &str| nocase(e, name).is_eq() || nocase(r, name).is_eq();
            let p = |k: &str| o.props.get(k).map(String::as_str).unwrap_or_default();
            if is("Node", "Узел") {
                return Ok(Some(nodes.into_iter().map(|n| Ty::Ref(n.clone())).collect()));
            }
            if is("MessageNo", "НомерСообщения") {
                return Ok(Some(vec![Ty::Prim]));
            }
            match p("WriteMode") {
                "RecorderSubordinate" if is("Recorder", "Регистратор") => return Ok(Some(vec![Ty::Unknown])),
                "RecorderSubordinate" => {}
                "Independent" => {
                    if periodic && is("Period", "Период") {
                        return if p("MainFilterOnPeriod") == "true" { Ok(Some(vec![Ty::Date])) } else { Err(format!("период таблицы изменений {obj} без основного отбора по периоду: поле не выведено")) };
                    }
                    if let Some((k, n)) = o.kids.iter().find(|(k, n)| k == "Dimension" && nocase(n, name).is_eq()) {
                        let full = format!("{obj}.{k}.{n}");
                        return if x.main_filter.contains(&full) { Ok(Some(meta_types(x, &full))) } else { Err(format!("измерение {n} таблицы изменений {obj} вне основного отбора: поле не выведено")) };
                    }
                }
                w => return Err(format!("таблица изменений {obj}: режим записи {w:?} не выведен")),
            }
            return Ok(crate::query::roles::common_fields(x, obj).into_iter().find(|(n, key)| *key && nocase(n, name).is_eq()).map(|(n, _)| meta_types(x, &format!("CommonAttribute.{n}"))));
        }
        let slice = ["СрезПоследних", "СрезПервых", "SliceLast", "SliceFirst"].iter().any(|v| nocase(v, vt).is_eq());
        if !(obj.starts_with("InformationRegister.") && slice && periodic) {
            return Err(format!("виртуальная таблица {obj}.{vt} в своем запросе пока не проверяется"));
        }
        if name.is_empty() {
            return Ok(None);
        }
        // состав среза — roles::slice_fields (basic FUN_1801db9f0)
        let t = crate::query::roles::slice_fields(x, obj).ok_or_else(|| format!("срез {obj}.{vt}: состав полей не выведен"))?;
        let is = |e: &str, r: &str| nocase(e, name).is_eq() || nocase(r, name).is_eq();
        if t.unsure.iter().any(|(e, r)| is(e, r)) {
            return Err(format!("поле {name} среза регистра в своем запросе: наличие не выведено"));
        }
        let Some((e, _, _)) = t.fields.iter().find(|(e, r, _)| is(e, r)) else { return Ok(None) };
        if e == "Period" {
            return Ok(Some(vec![Ty::Date]));
        }
        if e == "Recorder" {
            return Ok(Some(vec![Ty::Unknown]));
        }
        if let Some((k, n)) = o.kids.iter().find(|(k, n)| matches!(k.as_str(), "Dimension" | "Resource" | "Attribute") && n == e) {
            return Ok(Some(meta_types(x, &format!("{obj}.{k}.{n}"))));
        }
        Ok(Some(meta_types(x, &format!("CommonAttribute.{e}"))))
    }

    fn vt_param(&self, obj: &str, vt: &str, name: &str) -> R<Option<Vec<crate::query::resolve::Ty>>> {
        // shortcut: в параметрах проверяются только измерения регистра (они есть в условии и среза, и остатков);
        // прочие поля условия — отказ, пока не выведен их состав у каждой виртуальной таблицы
        let x = self.0;
        let o = x.objects.get(obj).ok_or_else(|| format!("нет таблицы {obj}"))?;
        // счет в условии счета и субконто в условии виртуальных таблиц регистра бухгалтерии
        if obj.starts_with("AccountingRegister.") && (nocase(name, "Счет").is_eq() || nocase(name, "Account").is_eq()) {
            return Ok(Some(o.props.get("ChartOfAccounts").map(|c| vec![crate::query::resolve::Ty::Ref(c.clone())]).unwrap_or(vec![crate::query::resolve::Ty::Unknown])));
        }
        let max = o.props.get("ChartOfAccounts").and_then(|c| x.objects.get(c)).map_or(0, |c| num_prop(c, "MaxExtDimensionCount"));
        if obj.starts_with("AccountingRegister.") && (1..=max).any(|i| nocase(&format!("Субконто{i}"), name).is_eq() || nocase(&format!("ExtDimension{i}"), name).is_eq()) {
            return Ok(Some(ext_dim_types(x, obj)));
        }
        // у среза регистра сведений в условии — и ресурсы, реквизиты, регистратор (ERP: ресурсы СоставЕГДС.ЕГДС,
        // НДССостояниеРеализации0.Состояние, Регистратор УсловияДоговоровАренды — эталонная загрузка запросы приняла).
        // shortcut: состав условия среза по образцам, не из кода; Period и прочие стандартные — отказ
        let slice = obj.starts_with("InformationRegister.") && ["СрезПоследних", "СрезПервых", "SliceLast", "SliceFirst"].iter().any(|v| nocase(v, vt).is_eq());
        if slice && o.props.get("WriteMode").is_some_and(|w| w == "RecorderSubordinate") && (nocase(name, "Регистратор").is_eq() || nocase(name, "Recorder").is_eq()) {
            return Ok(Some(vec![crate::query::resolve::Ty::Unknown]));
        }
        let kinds: &[&str] = if slice { &["Dimension", "Resource", "Attribute"] } else { &["Dimension"] };
        match o.kids.iter().find(|(k, n)| kinds.contains(&k.as_str()) && nocase(n, name).is_eq()) {
            Some((k, n)) => Ok(Some(meta_types(x, &format!("{obj}.{k}.{n}")))),
            None => Err(format!("поле {name} в параметрах виртуальной таблицы {obj}.{vt} не измерение: не проверено")),
        }
    }

    fn value(&self, obj: &str, name: &str) -> bool {
        let x = self.0;
        if nocase(name, "ПустаяСсылка").is_eq() || nocase(name, "EmptyRef").is_eq() {
            return true;
        }
        if obj.starts_with("Enum.") {
            return full_key(x, &format!("{obj}.EnumValue."), name).is_some();
        }
        x.predefined.keys().any(|k| k.strip_prefix(obj).and_then(|r| r.strip_prefix('.')).is_some_and(|n| nocase(n, name).is_eq()))
    }
}

/// Корень дерева полей списка со своим запросом: выходные поля текста, который получает набор компоновки после того,
/// как генератор запроса (mngbase FUN_18067a560: текст набора — ULQueryGenerator, FUN_180665660 +0x128) дописал поля
/// ролей основной таблицы. Поле выборки — псевдоним; без псевдонима `Источник.Поле` — имя поля (стандартное —
/// английское, второе имя русское). Роль, поле которой уже выбрано из источника основной таблицы, ничего не добавляет
/// (FUN_180666300 → FUN_18066a6f0); иначе поле добавляется (FUN_18066a8a0): имя — русское имя поля, занятое — с
/// суффиксом 1, 2… (`core::string_ito`); в корне добавленное без суффикса — как поле без псевдонима, с суффиксом — под
/// этим именем (трасса БП). Порядок корня — словарь полей набора dcs по имени без учета регистра (FUN_1800aefe0, обход
/// дерева, сравнение FUN_18003aed0).
fn query_root_fields(x: &Index, main: Option<&str>, text: &str, declared: &[(String, String)]) -> R<Option<Vec<Field>>> {
    query_root(x, main, text, declared, true)
}

/// `autofill` — AutoFillAvailableFields набора (`List::from_query`).
fn query_root(x: &Index, main: Option<&str>, text: &str, declared: &[(String, String)], autofill: bool) -> R<Option<Vec<Field>>> {
    let b = crate::query::parse(text)?;
    // запрос с таблицей, которой нет в конфигурации, платформа не компилирует: ошибку компиляции (SelectException,
    // entext RegentException) mngbase FUN_180680580 перебрасывает как mngcore::UniversalListException (FUN_1804317e0),
    // ее ловит catch 0x1807965bc вызова FUN_1802b7570 в FUN_1802b73c0 (0x1802b743e): флаг кадра +0x90 поднят при +0x360
    // формы = 0 (только так FUN_1802b7570 идет в схему FUN_1802b6b10), — результат пуст, продолжение 0x1802b7552; объект
    // исключения d5aed2e0 — переброс (GUID в entext нет). FUN_1802b76a0 оставляет корень без полей; пути `~Список.X`
    // нумерует счетчик FUN_1802c8a30 (unavailable_id). Прочие ошибки — отказ.
    let parts = match crate::query::resolve::check_parts(&IndexMeta(x), &b) {
        Err(e) if e.contains(crate::query::resolve::ABSENT) => return Ok(None),
        r => r?,
    };
    let outs = parts.first().cloned().unwrap_or_default();
    let q = b.queries.last().ok_or("пустой запрос")?;
    // ОБЪЕДИНИТЬ: поля запроса — поля первой части (entext FUN_18034f420: части +0x398 разбираются отдельно, имена —
    // FUN_18037ce90 первой части, +0x168). Поля ролей — см. ниже (только если каждое уже выбрано в каждой части).
    if q.parts.len() > 1 {
        // `{ГДЕ …}` части — как в первой (ниже): записи полей набора строятся по всем частям (dcs FUN_18030a0b0, части
        // `+0x48`, разбор каждой +0x70), поле только для условия корень режима 9 отбрасывает
        let every: Vec<_> = q.parts.iter().flat_map(|p| p.fields.iter().cloned()).collect();
        for p in q.parts.iter().skip(1) {
            if !p.dcs_select.is_empty() {
                return Err("расширение компоновки {ВЫБРАТЬ} в части ОБЪЕДИНИТЬ: флаг «поле» записей не выведен".into());
            }
            let mut ps = Vec::new();
            for f in &p.from {
                walk(x, f, &mut ps);
            }
            for f in &p.fields {
                // выражение без имени в части (`*`, подзапрос, кортеж): отбор не разбирался; `ПРЕДСТАВЛЕНИЕ(…)` —
                // `presentation_dropped` ниже: колонка остается, если аргумент не выбран ни в этой части, ни в других (по
                // тексту пути). shortcut: правило первой части перенесено по аналогии — по полям какой части dcs решает
                // отбор (FUN_18030a0b0, разбор частей +0x70), не прослежено, поэтому аргумент, выбранный хоть в одной
                // части, — отказ; сверено на АвансовыйОтчет.ФормаДокумента ERP, пересмотреть по разбору частей в dcs
                let presentation = matches!(&f.expr, crate::query::Expr::Call { name, .. } if matches!(name.as_str(), "ПРЕДСТАВЛЕНИЕ" | "PRESENTATION" | "ПРЕДСТАВЛЕНИЕССЫЛКИ" | "REFPRESENTATION"));
                if !presentation && matches!(crate::query::names::unnamed(&f.expr), crate::query::names::Unnamed::No) {
                    return Err("поле-выражение части ОБЪЕДИНИТЬ: отбор в корень не выведен".into());
                }
                if crate::query::dcs::password_dropped(x, f) {
                    return Err("часть ОБЪЕДИНИТЬ: поле реквизита в режиме пароля — отбор в корень не выведен".into());
                }
                if crate::query::dcs::presentation_dropped(x, f, &p.fields, &ps, &|t| ts_source(x, t)) || presentation && crate::query::dcs::presentation_dropped(x, f, &every, &ps, &|t| ts_source(x, t)) {
                    return Err("часть ОБЪЕДИНИТЬ: поле-представление — отбор в корень не выведен".into());
                }
            }
        }
    }
    let s = &q.parts[0];
    // `{ГДЕ …}` полей корня не дает: поле только для условия (флаг использования «поле» записи +0x7e = 0) режим 9
    // отбрасывает (dcs FUN_180062e80: +0x8e описания = 0); `{ВЫБРАТЬ …}` — после списка выборки
    if s.into.is_some() {
        return Err("последний запрос пакета с ПОМЕСТИТЬ: набор списка не выведен".into());
    }
    let mut srcs = Vec::new();
    for f in &s.from {
        walk(x, f, &mut srcs);
    }
    // пустое имя — единственный источник без псевдонима (поля без имени источника: `ВЫБРАТЬ Ссылка ИЗ Справочник.X`)
    let alias_of = |a: &str| srcs.iter().find(|(s, _)| match s.alias.as_deref() { Some(x) => nocase(x, a).is_eq(), None => a.is_empty() && srcs.len() == 1 }).and_then(|(_, t)| t.clone());
    let mut fields = Vec::new();
    // имена в языке запроса для совпадений при дописывании ролей (FUN_18066a8a0)
    let mut ru = Vec::new();
    let mut plain = Vec::new();
    // последнее имя пути поля выборки (`Т.Владелец.Ссылка` → Ссылка) и псевдоним таблицы, если поле `Псевдоним.Поле`
    let mut tails = Vec::new();
    let mut numbered = 0;
    // начало полей корня каждого поля выборки и есть ли у него псевдоним (для имен-дублей ниже)
    let mut starts = Vec::new();
    let mut implicit: Vec<String>;
    for (f, _) in s.fields.iter().zip(&outs) {
        use crate::query::Expr;
        starts.push((fields.len(), f.alias.is_some()));
        let path = match &f.expr {
            Expr::Path(p) => Some(p),
            _ => None,
        };
        if let Some(p) = path.filter(|p| p.len() == 2) {
            plain.push((p[0].clone(), p[1].clone(), f.alias.clone()));
        }
        // `Поле` единственного источника без псевдонима — как `Источник.Поле` (имя источника пустое, `alias_of`): поле
        // роли ищется по выражению колонки, а не по тексту (mngbase FUN_180666300: выражение роли — модель +0x2e8 по
        // источнику части; FUN_18066a6f0 — сравнение выражений vtable +0x30, хеш +0xe0). shortcut: что модель дает `Поле`
        // и `Источник.Поле` одно выражение, не прослежено — сверено на ИнтегрируемыеОбъекты
        // ПравилаИнтеграцииС1СДокументооборотом ERP; у источника с псевдонимом `Поле` — по-прежнему отказ
        if let Some(p) = path.filter(|p| p.len() == 1 && alias_of("").is_some()) {
            plain.push((String::new(), p[0].clone(), f.alias.clone()));
        }
        // `Т.ТЧ.Реквизит` в выборке (resolve принимает): имя и раскрытие поля корня не выведены
        if let Some(p) = path.filter(|p| p.len() > 2) {
            use crate::query::resolve::Meta;
            if alias_of(&p[0]).is_some_and(|(_, o)| IndexMeta(x).ts(&o, &p[1])) {
                return Err(format!("поле выборки {} через табличную часть: поле корня не выведено", p.join(".")));
            }
        }
        let tail = match &f.expr {
            Expr::Path(p) => p.last(),
            Expr::Dot(_, v) => v.last(),
            _ => None,
        };
        if let Some(t) = tail {
            let unaliased = path.is_some_and(|p| p.len() == 1 && alias_of("").is_some());
            tails.push((t.clone(), path.filter(|p| p.len() == 2 && srcs.iter().any(|(s, _)| s.alias.as_deref().is_some_and(|x| nocase(x, &p[0]).is_eq()))).map(|p| p[0].clone()).or(unaliased.then(String::new))));
        }
        // хранилище значения — обычное поле корня: ограничения по типу (dcs FUN_18030f4b0: типы d47d59f8…,
        // ДвоичныеДанные 87126200… без длины, строка без длины) снимают флаги +0x80, +0x81, +0x84, +0x85 записи, но не
        // флаг «поле» +0x7e, а корень режима 9 смотрит только его (FUN_180062e80: +0x8e описания); ХранилищеЗначения
        // (e199ca70…, dcs 0x1805627e0) в этих проверках нет (трасса qp-3: E3 [1,[1,0,…]], поле в корне D1)
        if crate::query::dcs::password_dropped(x, f) {
            continue;
        }
        // ПРЕДСТАВЛЕНИЕССЫЛКИ, названное в расширении {ВЫБРАТЬ}, — поле корня (ЗаказыТорговыхПлощадок ERP)
        let in_ext = matches!(&f.expr, crate::query::parse::Expr::Call { name, .. } if name == "ПРЕДСТАВЛЕНИЕССЫЛКИ" || name == "REFPRESENTATION")
            && f.alias.as_deref().is_some_and(|a| s.dcs_select.iter().any(|g| matches!(&g.expr, crate::query::parse::Expr::Path(p) if p.len() == 1 && nocase(&p[0], a).is_eq())));
        if !in_ext && crate::query::dcs::presentation_dropped(x, f, &s.fields, &srcs, &|t| ts_source(x, t)) {
            continue;
        }
        if let Some(a) = &f.alias {
            fields.push(Field { name: a.clone(), second: a.clone() });
            ru.push(a.clone());
            continue;
        }
        // `Т.*`: поля таблицы источника (roles::star_fields); выбраны как `Т.Поле` — для ролей поле выбрано
        if let Expr::Star(p) = &f.expr {
            // `Т.ТЧ.*` — одна колонка вложенной таблицы с именем табличной части (entext FUN_18034a270: QI ca360 поля
            // → выражение FUN_18046cc40 «Т.ТЧ», поля FUN_180349b00, имя +0x18 — имя ТЧ на языке запроса)
            // виды субконто плана счетов — стандартная табличная часть, имя на языке запроса. shortcut: язык имени по
            // номерам FieldsMap не различим (Форма НастройкаСчетовМеждународногоУчета ERP), FUN_18034a270 для
            // стандартной табличной части не прочитан
            let std_ts = |o: &str, t: &str| (o.starts_with("ChartOfAccounts.") && (nocase(t, "ВидыСубконто").is_eq() || nocase(t, "ExtDimensionTypes").is_eq())).then(|| if nocase(t, "ВидыСубконто").is_eq() { "ВидыСубконто" } else { "ExtDimensionTypes" }.to_string());
            let ts = match p.as_slice() {
                [a, t] => alias_of(a).and_then(|(_, o)| std_ts(&o, t).or_else(|| x.objects.get(&o)?.kids.iter().find(|(k, n)| k == "TabularSection" && nocase(n, t).is_eq()).map(|(_, n)| n.clone()))),
                _ => None,
            };
            if let Some(n) = ts {
                fields.push(Field { name: n.clone(), second: n.clone() });
                ru.push(n);
                continue;
            }
            // `ВЫБРАТЬ *` (все поля всех источников) с одним источником — его `Т.*`; у источника без псевдонима — пустое
            // имя (`alias_of`). shortcut: разбор `*` в entext (FUN_18034a270) не прочитан, по справке платформы; несколько
            // источников — отказ
            let one = match (p.as_slice(), srcs.as_slice()) {
                ([], [(s, _)]) => s.alias.clone().unwrap_or_default(),
                ([a], _) => a.clone(),
                _ => return Err("`*` при нескольких источниках или `Т.Поле.*` в своем запросе: состав полей не выведен".into()),
            };
            let a = &one;
            let (_, obj) = alias_of(a).ok_or_else(|| format!("`{a}.*` не из таблицы объекта"))?;
            // `Справочник.*` — поля таблицы и DataVersion, без Presentation (все 3 номера FieldsMap
            // ПодключаемоеОборудование.ФормаВыбораИзСписка БП). shortcut: по одной форме; документ — ниже, прочие ссылочные
            // классы — отказ (`roles::star_fields`)
            // `Документ.*` (и `ВЫБРАТЬ *`, и `Д.*`) — поля таблицы по построителю (`roles::table_fields`) без поля с подполями
            // (PointInTime, как у `star_fields`) и без Presentation (Ref выбрана — `query::dcs::presentation_dropped`).
            // shortcut: DataVersion в корне и Presentation нет — по одной форме (ЭтапПроизводства2_2.ФормаСписка ERP: корень
            // совпал, запись отвергнута по ServerState), принятой записью не проверено; пересмотреть по первой принятой
            let star = if obj.starts_with("Catalog.") {
                let mut v: Vec<_> = root_fields(x, &obj)?.into_iter().map(|f| (f.name, f.second)).collect();
                v.push(("DataVersion".into(), "ВерсияДанных".into()));
                v
            } else if obj.starts_with("Document.") {
                let t = crate::query::roles::table_fields(x, &obj).ok_or_else(|| format!("`.*` {obj}: состав полей не выведен"))?;
                t.fields.into_iter().filter(|(e, _, composite)| !composite && e != "Presentation").map(|(e, r, _)| (e, r)).collect()
            } else {
                crate::query::roles::star_fields(x, &obj)?
            };
            for (n, s) in star {
                plain.push((a.clone(), n.clone(), None));
                fields.push(Field { name: n, second: s.clone() });
                ru.push(s);
            }
            continue;
        }
        use crate::query::names::{unnamed, Unnamed};
        let p = match unnamed(&f.expr) {
            // агрегат поля: выбрано ли поле роли (mngbase FUN_18066a6f0), не выведено
            Unnamed::Field(_) if main.is_some() && !matches!(f.expr, crate::query::Expr::Path(_)) => return Err("агрегат без псевдонима в своем запросе с основной таблицей: совпадение с полем роли не выведено".into()),
            Unnamed::Field([n]) if alias_of("").is_some() => {
                implicit = vec![String::new(), n.clone()];
                &implicit[..]
            }
            Unnamed::Field(p) => p,
            // entext FUN_18037ce90: «Field»/«Поле» + номер по порядку полей, где +0xf8 ложь
            Unnamed::Numbered => {
                numbered += 1;
                fields.push(Field { name: format!("Field{numbered}"), second: format!("Поле{numbered}") });
                ru.push(format!("Поле{numbered}"));
                continue;
            }
            Unnamed::Fixed(n) => {
                fields.push(Field { name: n.to_string(), second: n.to_string() });
                ru.push(n.to_string());
                continue;
            }
            // имя табличной части по метаданным (resolve проверил: `Источник.ТЧ` объекта); вложенное поле с номером
            // делит счетчик с полями выборки (FUN_18037ce90) — не выведено
            Unnamed::Table(_) if matches!(&f.expr, crate::query::Expr::Nested(_, fs) if fs.iter().any(|g| g.alias.is_none() && !matches!(unnamed(&g.expr), Unnamed::Field(_) | Unnamed::Fixed(_)))) => {
                return Err("вложенное поле табличной части без псевдонима — выражение: номер имени не выведен".into())
            }
            // стандартная табличная часть плана видов расчета — пара имен стандартного поля (calc FUN_18031a9b0)
            Unnamed::Table(p) if alias_of(&p[0]).is_some_and(|(_, o)| calc_std_ts(x, &o, &p[1]).is_some()) => {
                let e = alias_of(&p[0]).and_then(|(_, o)| calc_std_ts(x, &o, &p[1])).unwrap_or_default();
                let r = std_ru(e).unwrap_or(e);
                fields.push(Field { name: e.to_string(), second: r.to_string() });
                ru.push(r.to_string());
                continue;
            }
            Unnamed::Table(p) => {
                let n = alias_of(&p[0]).and_then(|(_, o)| x.objects.get(&o)?.kids.iter().find(|(k, n)| k == "TabularSection" && nocase(n, &p[1]).is_eq()).map(|(_, n)| n.clone())).ok_or_else(|| format!("табличная часть {} без псевдонима не из таблицы объекта", p.join(".")))?;
                fields.push(Field { name: n.clone(), second: n.clone() });
                ru.push(n);
                continue;
            }
            Unnamed::No => return Err("выражение без псевдонима в своем запросе: имя поля не выведено".into()),
        };
        // поле виртуальной таблицы или табличной части-источника (`Т.Поле`, путь источника из трех частей): пара имен —
        // имена поля источника (entext FUN_18037ce90 → FUN_1802e1120): ресурс с суффиксом — basic.dll FUN_18007a960
        // (таблица суффиксов 0x1809fb118), стандартное — его пара, прочие — одно имя на оба языка; `None` — источник не такой
        let vt = srcs.iter().find(|(s, _)| s.alias.as_deref().is_some_and(|a| nocase(a, &p[0]).is_eq())).and_then(|(s, _)| match &s.table {
            crate::query::Table::Name { path, .. } if path.len() == 3 => query_class(&path[0]).and_then(|c| obj_key(x, &format!("{c}.{}", path[1]))).map(|o| (o, path[2].clone(), ts_source(x, &s.table))),
            _ => None,
        });
        let vt_pair = |n: &String| -> R<Option<(String, String)>> {
            let Some((o, t, ts)) = vt.clone().filter(|_| alias_of(&p[0]).is_none()) else { return Ok(None) };
            use crate::query::resolve::Meta;
            let found = if ts { IndexMeta(x).ts_field(&o, &t, n) } else { IndexMeta(x).vt_field(&o, &t, n) };
            found?.ok_or_else(|| format!("поле {} не найдено в {o}.{t}", p.join(".")))?;
            const SUFFIX: &[(&str, &str)] = &[("Balance", "Остаток"), ("Turnover", "Оборот"), ("Receipt", "Приход"), ("Expense", "Расход"), ("OpeningBalance", "НачальныйОстаток"), ("ClosingBalance", "КонечныйОстаток")];
            let kids = x.objects.get(&o).map(|ob| ob.kids.clone()).unwrap_or_default();
            let res = kids.iter().filter(|(k, _)| k == "Resource").find_map(|(_, r)| SUFFIX.iter().find(|(e, ru)| nocase(&format!("{r}{ru}"), n).is_eq() || nocase(&format!("{r}{e}"), n).is_eq()).map(|(e, ru)| (format!("{r}{e}"), format!("{r}{ru}"))));
            let std = STD.iter().find(|(e, r)| nocase(e, n).is_eq() || nocase(r, n).is_eq()).map(|(e, r)| (e.to_string(), r.to_string()));
            let own = kids.iter().find(|(_, k)| nocase(k, n).is_eq()).map(|(_, k)| (k.clone(), k.clone()));
            res.or(std).or(own).map(Some).ok_or_else(|| format!("поле {} виртуальной таблицы: имя не выведено", p.join(".")))
        };
        // `Т.А.Б` без псевдонима — склейка имен частей после источника на каждом языке (Файлы: `…УчетнаяЗапись.Сервис`
        // → УчетнаяЗаписьСервис). shortcut: по сверке одной формы, имя стандартного поля в середине пути не проверено
        // у виртуальной таблицы первая часть — пара имен ее поля (`vt_pair`)
        // у табличной части-источника путь через ее Ссылку — без имени Ссылки: entext FUN_1802e1120 начинает имя с поля
        // пути, только если его слот +0x28 (с источником и путем) не пуст, иначе — только имена частей пути (Статус в
        // корне ВыборПозицииЗаказаНаПроизводство ERP: `Т.Ссылка.Статус`, номера FieldsMap). shortcut: слот +0x28 не
        // прочитан — правило только для Ссылки табличной части; при расхождении — разбирать реализацию +0x28 полей
        let ts_ref = p.len() > 2 && alias_of(&p[0]).is_none() && vt.as_ref().is_some_and(|(_, _, ts)| *ts) && (nocase(&p[1], "Ссылка").is_eq() || nocase(&p[1], "Ref").is_eq());
        let first = if p.len() > 2 && !ts_ref { vt_pair(&p[1])? } else { None };
        if p.len() > 2 && (alias_of(&p[0]).is_some() || first.is_some() || ts_ref) {
            let part = |n: &String| STD.iter().find(|(e, r)| nocase(e, n).is_eq() || nocase(r, n).is_eq()).map_or((n.clone(), n.clone()), |(e, r)| (e.to_string(), r.to_string()));
            let (n0, n1): (Vec<_>, Vec<_>) = first.into_iter().chain(p[1 + usize::from(vt.is_some() && alias_of(&p[0]).is_none())..].iter().map(part)).unzip();
            ru.push(n1.concat());
            fields.push(Field { name: n0.concat(), second: n1.concat() });
            continue;
        }
        if p.len() != 2 {
            return Err("поле через точку без псевдонима в своем запросе пока не загружается".into());
        }
        let n = &p[1];
        // реквизит табличной части-источника: имя поля на обоих языках — имя реквизита (entext FUN_1802e1120 без
        // подпути: имя поля, как у реквизита объекта); стандартные поля табличной части — не выведены
        let ts_attr = srcs.iter().find(|(s, _)| s.alias.as_deref().is_some_and(|a| nocase(a, &p[0]).is_eq())).and_then(|(s, _)| match &s.table {
            crate::query::Table::Name { path, params: None } if path.len() == 3 && ts_source(x, &s.table) => {
                let o = query_class(&path[0]).and_then(|c| obj_key(x, &format!("{c}.{}", path[1])))?;
                let tn = x.objects.get(&o)?.kids.iter().find(|(k, t)| k == "TabularSection" && nocase(t, &path[2]).is_eq()).map(|(_, t)| t.clone())?;
                full_key(x, &format!("{o}.TabularSection.{tn}.Attribute."), n).and_then(|k| k.rsplit('.').next().map(str::to_string))
            }
            _ => None,
        });
        if let Some(a) = ts_attr {
            fields.push(Field { name: a.clone(), second: a.clone() });
            ru.push(a);
            continue;
        }
        if let Some((n0, n1)) = crate::query::names::derived(x, &b, s, p, STD)? {
            ru.push(n1.clone());
            fields.push(Field { name: n0, second: n1 });
            continue;
        }
        // поле видов субконто счета без псевдонима — имя поля таблицы на двух языках
        let ext = srcs.iter().find(|(s, _)| s.alias.as_deref().is_some_and(|a| nocase(a, &p[0]).is_eq())).and_then(|(s, _)| match &s.table {
            crate::query::Table::Name { path, .. } if path.len() == 3 => query_class(&path[0]).and_then(|c| obj_key(x, &format!("{c}.{}", path[1]))).and_then(|o| {
                ext_dimension_fields(x, &o, &path[2]).or_else(|| records_ext_fields(x, &o, &path[2]).map(|v| v.into_iter().map(|(e, r, _)| (e, r)).collect()))
            }),
            _ => None,
        });
        if let Some(names) = ext {
            let (e, r) = names.into_iter().find(|(e, r)| nocase(e, n).is_eq() || nocase(r, n).is_eq()).ok_or_else(|| format!("поле {n} видов субконто не найдено"))?;
            ru.push(r.clone());
            fields.push(Field { name: e, second: r });
            continue;
        }
        if let Some(pair) = vt_pair(&p[1])? {
            ru.push(pair.1.clone());
            fields.push(Field { name: pair.0, second: pair.1 });
            continue;
        }
        let (class, obj) = alias_of(&p[0]).ok_or_else(|| format!("поле {} без псевдонима не из таблицы объекта", p.join(".")))?;
        let o = x.objects.get(&obj).ok_or_else(|| format!("нет таблицы {obj}"))?;
        let en = STD.iter().chain([("DataVersion", "ВерсияДанных"), ("Presentation", "Представление")].iter()).map(|(e, _)| *e).find(|e| nocase(e, n).is_eq() || std_ru_in(&class, e).is_some_and(|r| nocase(r, n).is_eq()));
        // реквизит объекта или общий реквизит, в составе которого объект (Use; Auto — по AutoUse)
        let common = x.common.iter().any(|(cn, auto, by)| {
            nocase(cn, n).is_eq() && match by.get(&obj).map(String::as_str) {
                Some("Use") => true,
                Some("DontUse") => false,
                _ => auto == "Use",
            }
        });
        let attr = o.kids.iter().any(|(_, k)| nocase(k, n).is_eq()) || common;
        match en.filter(|e| std_of(&class).contains(e)) {
            Some(e) if !attr => fields.push(Field { name: e.to_string(), second: std_ru_in(&class, e).unwrap_or(e).to_string() }),
            Some(_) => return Err(format!("поле {n} — и стандартное, и реквизит {obj}")),
            None if attr => fields.push(Field { name: n.clone(), second: n.clone() }),
            None if !std_of(&class).is_empty() => return Err(format!("поле {n} без псевдонима не найдено в {obj}")),
            None => return Err(format!("стандартные поля таблицы класса {class} в своем запросе пока не выведены")),
        }
        ru.push(n.clone());
    }
    // имя поля без псевдонима, уже занятое полем раньше, — с номером 1, 2… (ПодключаемоеОборудование БП: `Т.*` с
    // реквизитом Организация и `Р.Организация` — в корне на одно поле больше); с псевдонимом — не выведено
    if fields.len() != ru.len() {
        return Err("поля корня и имена в языке запроса разошлись".into());
    }
    for i in 0..ru.len() {
        if !ru[..i].iter().any(|b| nocase(&ru[i], b).is_eq()) {
            continue;
        }
        if starts.iter().rev().find(|(k, _)| *k <= i).is_none_or(|(_, aliased)| *aliased) {
            return Err("поля своего запроса с одинаковым именем без учета регистра".into());
        }
        let n = (1..).map(|k| format!("{}{k}", ru[i])).find(|c| !ru.iter().any(|b| nocase(b, c).is_eq())).unwrap_or_default();
        fields[i] = Field { name: n.clone(), second: n.clone() };
        ru[i] = n;
    }
    // `{ВЫБРАТЬ Имя}` по полю выборки корень не меняет: запись поля остается с флагом «поле» +0x7e = 1 (трасса
    // qp-dcs-14-3, ИнтерфейсДокументовЭДО: КонтрольОтраженияВУчете [+0x7e 1, +0x7f 0]; корень B2 — все поля выборки),
    // режим 9 пропускает ее (dcs FUN_180062e80: +0x8e ≠ 0). Псевдоним, равный имени (`{ВЫБРАТЬ Ссылка.* КАК Ссылка}`), —
    // тот же элемент: элементы расширения редактор запроса различает по имени (mngbase FUN_180669e30 / FUN_1806692a0:
    // модель +0x578(вид, часть), имя элемента +0x40 — псевдоним или имя поля, хеш без учета регистра). shortcut: что и запись поля
    // dcs строится по одному имени, не прослежено — сверено на СписокСвязанныхДоговоров НаправленияДеятельности и
    // ВыборЗаказаВОтчетПереработчика ЗаказПереработчику ERP. Прочее (выражение, другой псевдоним, поле не из выборки)
    // — не выведено
    for f in &s.dcs_select {
        match &f.expr {
            crate::query::Expr::Path(p) | crate::query::Expr::Star(p) if p.len() == 1 && f.alias.as_deref().is_none_or(|a| nocase(a, &p[0]).is_eq()) && ru.iter().any(|n| nocase(n, &p[0]).is_eq()) => {}
            _ => return Err("расширение компоновки {ВЫБРАТЬ} не по имени поля выборки: флаг «поле» записи не выведен".into()),
        }
    }
    // основная таблица — виртуальная таблица регистра (`AccumulationRegister.Имя.Balance`): полей ролей нет (сверка
    // ПередачаВРегистр2ЕГАИС ЭДО). shortcut: одна форма, все поля с псевдонимами; роли виртуальных таблиц не прочитаны
    // сверены остатки (ПередачаВРегистр2ЕГАИС ЭДО) и движения с субконто (ФормаСписка Хозрасчетного БП: поля ролей все
    // выбраны), задачи по исполнителю (МоиЗадачи БП: корень — выбранные поля с Presentation); прочие не выведены
    // срез регистра сведений — роли по построителю backend (`roles::slice_roles`); источник — виртуальная таблица
    let slice = main.is_some_and(|m| m.starts_with("InformationRegister.") && (m.ends_with(".SliceLast") || m.ends_with(".SliceFirst")));
    // совместимость < 0x139bf: схему дерева полей строит mngbase FUN_1802b6b10 сам — набор ULDataSet с текстом своего
    // запроса как есть, без генератора ULQueryEditor (FUN_180297ec0 → FUN_180680580 → FUN_1806689c0): полей ролей нет
    let main = main.filter(|_| !old_schema(x));
    if let Some(m) = main.filter(|m| m.matches('.').count() > 1 && !slice && !m.ends_with(".Balance") && !m.ends_with(".RecordsWithExtDimensions") && !m.ends_with(".TasksByExecutive")) {
        return Err(format!("виртуальная основная таблица {m}: состав корня не выведен"));
    }
    if let Some(main) = main.filter(|m| m.matches('.').count() == 1 || slice) {
        let ms = if slice {
            let (base, vt) = main.rsplit_once('.').unwrap_or_default();
            let names: &[&str] = if vt == "SliceLast" { &["СрезПоследних", "SliceLast"] } else { &["СрезПервых", "SliceFirst"] };
            if q.parts.len() > 1 {
                return Err("основная таблица — срез, запрос с ОБЪЕДИНИТЬ: роли по частям не выведены".into());
            }
            srcs.iter().map(|(s, _)| *s).find(|s| match &s.table {
                crate::query::Table::Name { path, .. } if path.len() == 3 => query_class(&path[0]) == Some("InformationRegister") && obj_key(x, &format!("InformationRegister.{}", path[1])).as_deref() == Some(base) && names.iter().any(|n| nocase(n, &path[2]).is_eq()),
                _ => false,
            }).ok_or_else(|| format!("источник основной таблицы {main} в своем запросе не найден"))?
        } else {
            crate::query::roles::main_source(&srcs, main)?
        };
        // источник без псевдонима — только единственный (поля `Поле` — `plain` с пустым именем источника)
        let malias = ms.alias.clone().or_else(|| (srcs.len() == 1).then(String::new)).ok_or("источник основной таблицы без псевдонима")?;
        // ОБЪЕДИНИТЬ: источник основной таблицы нужен в каждой части (mngbase FUN_180665950 по частям FUN_180665bf0, иначе
        // исключение). ULQueryEditor FUN_18066a8a0 ищет колонку (+0x4c0) по вектору выражений частей, нет — новая колонка
        // (+0x458, выражение по частям +0x278). Реализация +0x4c0 не прочитана: поле роли считается выбранным, только если
        // одна и та же колонка в каждой части — `Псевдоним.Поле` источника основной таблицы (так при любом сравнении);
        // иначе отказ
        let mut others = Vec::new();
        for p in q.parts.iter().skip(1) {
            let mut ps = Vec::new();
            for f in &p.from {
                walk(x, f, &mut ps);
            }
            let m: Vec<_> = ps.iter().filter(|(_, t)| t.as_ref().is_some_and(|(_, o)| o == main)).collect();
            let a = match m.as_slice() {
                [(s, _)] => s.alias.clone().ok_or("источник основной таблицы без псевдонима в части ОБЪЕДИНИТЬ")?,
                _ => return Err(format!("источник основной таблицы {main} в части ОБЪЕДИНИТЬ не найден однозначно")),
            };
            others.push((a, p));
        }
        // поле `Псевдоним.Поле` источника основной таблицы — имя поля в колонке j части
        let col = |fs: &[crate::query::Field], j: usize, a: &str| -> Option<String> {
            match fs.get(j).map(|f| &f.expr) {
                Some(crate::query::Expr::Path(v)) if v.len() == 2 && nocase(&v[0], a).is_eq() => Some(v[1].clone()),
                _ => None,
            }
        };
        // имена редактора запроса (mngbase ULQueryEditor, конструктор FUN_180668da0): колонки модели (entext +0x448 —
        // все колонки по порядку, и выброшенные из корня; колонка — vtable 0x1805d0610, выражения частей +0x20, имя
        // +0x60) под своим именем, кроме колонки, чей ключ (выражения по частям, FUN_18066a2e0) уже есть в +0x78:
        // хеш — текст выражения (FUN_18066b960, +0xe0), сравнение FUN_180668d10 → +0x30 выражения. Списки 0 и 1
        // (+0x578, entext FUN_1802817f0: part+0x1b0 — `{ВЫБРАТЬ}`, part+0x1c8 — `{ГДЕ}`) учтены ниже, у полей ролей. Поле роли с
        // русским именем из этого множества FUN_18066a8a0 добавляет с явным псевдонимом и суффиксом, иначе — без
        // псевдонима, в корне английским именем (`НЕОПРЕДЕЛЕНО КАК Номер, НЕОПРЕДЕЛЕНО КАК Дата` — роль Date).
        // shortcut: +0x30 выражений entext не прочитана; колонка выпадает как повтор только при константе (литерал,
        // ЗНАЧЕНИЕ) и выбранном прямо поле источника основной таблицы — так в трех формах ERP (ФормированиеЗаказов…,
        // ПодборТоваров… ×2), а повтор поля другой таблицы (ТаможеннаяДекларацияИмпорт.РабочееМесто, `Ссылка1`) и
        // константы без полей таблиц (ПрименениеНоменклатурыВПроизводстве, `Проведен1`) имя оставляют; пересмотреть
        // по разбору +0x30 классов выражений entext
        let direct = s.fields.iter().any(|f| matches!(&f.expr, crate::query::Expr::Path(p) if p.len() == 2 && nocase(&p[0], &malias).is_eq()));
        let mut keys: Vec<Vec<&crate::query::Expr>> = Vec::new();
        let mut taken = Vec::new();
        for (k, f) in s.fields.iter().enumerate().take(starts.len()) {
            use crate::query::Expr::{Const, Num, Star, Str, Value};
            let (lo, hi) = (starts[k].0, starts.get(k + 1).map_or(ru.len(), |x| x.0));
            if matches!(f.expr, Star(_)) {
                taken.extend(ru[lo..hi].iter().cloned());
                continue;
            }
            let key: Vec<_> = std::iter::once(&f.expr).chain(q.parts.iter().skip(1).filter_map(|p| p.fields.get(k).map(|g| &g.expr))).collect();
            if direct && key.iter().all(|e| matches!(e, Const(_) | Num(_) | Str(_) | Value(_))) && keys.contains(&key) {
                continue;
            }
            keys.push(key);
            match (ru.get(lo).filter(|_| hi > lo), &f.alias, &f.expr) {
                (Some(n), _, _) => taken.push(n.clone()),
                (None, Some(a), _) => taken.push(a.clone()),
                (None, None, crate::query::Expr::Path(p)) => taken.extend(p.last().cloned()),
                _ => {}
            }
        }
        let roles = if slice { crate::query::roles::slice_roles(x, main)? } else { crate::query::roles::main_roles(x, main)? };
        for r in roles.iter().map(String::as_str) {
            let rru = std_ru(r).unwrap_or(r);
            let is = |n: &str| nocase(n, r).is_eq() || nocase(n, rru).is_eq();
            let listed = |o: &str| s.dcs_select.iter().any(|g| matches!(&g.expr, crate::query::Expr::Path(p) | crate::query::Expr::Star(p) if p.len() == 1 && nocase(&p[0], o).is_eq()));
            let found = if others.is_empty() {
                // записи колонок редактора запроса (mngbase ULQueryEditor): элемент {ВЫБРАТЬ} первой части — запись без
                // выражения расширения, флаг +0x28 = 0 (FUN_180668da0 → FUN_180669e30, vtbl+0x578(0, …)); при
                // автозаполнении — колонка вне {ВЫБРАТЬ}, выражение которой еще не записано, флаг 1 (vtbl+0x448). Элемент
                // {ГДЕ} с именем колонки (FUN_1806692a0, vtbl+0x578(1, …)) ставит флаг = равенство выражений колонки и
                // элемента; записи вне {ВЫБРАТЬ} — при совместимости > 0x139bb (vtbl+0x158 сервиса b1c6a8a5). Поле роли
                // выбрано, если флаг 1 у записи колонки с выражением роли (FUN_18066a6f0, поиск по выражению +0x78).
                // НастройкаПоддержанияЗапасов ERP: ГруппировкаТоварныхОграничений в {ВЫБРАТЬ}, но не в {ГДЕ} — поле роли
                // добавлено с суффиксом 1. vtbl+0x578 — entext FUN_1802817f0: вид 0 — part+0x1b0 (при vtbl+0xe8 — первой
                // части), 1 — part+0x1c8; что это {ВЫБРАТЬ} и {ГДЕ} — по устройству элементов (+0x60: колонка выборки /
                // выражение), сверено на всех корпусах
                let cols: Vec<&str> = plain.iter().filter(|(a, n, _)| nocase(a, &malias).is_eq() && is(n)).map(|(_, n, al)| al.as_deref().unwrap_or(n)).collect();
                if cols.iter().any(|c| listed(c)) {
                    // FUN_18066a6f0 берет одну запись по выражению (+0x78): какую из нескольких колонок — не выведено
                    let mut hits = cols.iter().filter(|c| listed(c)).map(|c| Ok(dcs_where_item(&s.dcs_where, c, &malias, &is)? == Some(Some(true)))).collect::<R<Vec<bool>>>()?;
                    hits.dedup();
                    match hits.as_slice() {
                        [h] => *h,
                        _ => return Err(format!("поле роли {rru}: колонки в {{ВЫБРАТЬ}} с разным флагом по {{ГДЕ}} — запись по выражению не выведена")),
                    }
                } else {
                    // колонка вне {ВЫБРАТЬ} и одноименный элемент {ГДЕ}-выражение (КорректировкаПриобретения ERP:
                    // `(ВЫРАЗИТЬ(Р.Контрагент.Ключ КАК …)).* КАК Контрагент`) — по номерам FieldsMap поле роли выбрано;
                    // одноименное поле другого пути — не выведено, отказ. shortcut: по механизму запись колонки вне
                    // {ВЫБРАТЬ} флаг по {ГДЕ} получает (совместимость > 0x139bb) и выражение роли с выражением элемента не
                    // равно — расходится с двумя формами ERP; пересмотреть по сервису b1c6a8a5 (vtbl+0x158, config.dll)
                    // или по составу вида 1 entext. Без автозаполнения (AutoFillAvailableFields = false) записи колонки
                    // вне {ВЫБРАТЬ} нет (FUN_180668da0, байт +0x20 владельца): поле роли добавляется, имя добавленного
                    // поля (FUN_18066a8a0) не выведено — отказ
                    match cols.first() {
                        Some(c) if !autofill => return Err(format!("поле роли {rru}: без автозаполнения колонка {c} вне {{ВЫБРАТЬ}} — имя добавленного поля не выведено")),
                        Some(c) => match dcs_where_item(&s.dcs_where, c, &malias, &is)? {
                            Some(Some(false)) if x.compat.is_none_or(|v| v > 0x139bb) => {
                                return Err(format!("поле роли {rru}: колонка {c} вне {{ВЫБРАТЬ}}, в {{ГДЕ}} одноименное поле другого пути — флаг записи не выведен"))
                            }
                            _ => true,
                        },
                        None => false,
                    }
                }
            } else {
                // ОБЪЕДИНИТЬ: флаг записи из элементов {ГДЕ} всех частей (FUN_1806692a0) — не выведен
                if (0..s.fields.len()).any(|j| col(&s.fields, j, &malias).is_some_and(|n| is(&n) && listed(s.fields[j].alias.as_deref().unwrap_or(&n)))) {
                    return Err(format!("ОБЪЕДИНИТЬ: поле роли {rru} в {{ВЫБРАТЬ}} — флаг записи по {{ГДЕ}} частей не выведен"));
                }
                (0..s.fields.len()).any(|j| col(&s.fields, j, &malias).is_some_and(|n| is(&n)) && others.iter().all(|(a, p)| col(&p.fields, j, a).is_some_and(|n| is(&n))))
            };
            if found {
                continue;
            }
            if main.starts_with("FilterCriterion.") {
                return Err(format!("поле Ссылка критерия отбора {main} не выбрано: есть ли роль Ref, не выведено"));
            }
            if !others.is_empty() {
                return Err(format!("ОБЪЕДИНИТЬ: поле роли {rru} не в одной колонке каждой части — колонка по частям (+0x4c0) не выведена"));
            }
            // поле роли выбрано не как `Псевдоним.Поле` (без псевдонима таблицы, через ссылку, выражением): считает
            // ли его FUN_18066a6f0 выбранным из источника основной таблицы — не выведено; `Другой.Поле` — не из него
            if tails.iter().any(|(t, a)| (nocase(t, r).is_eq() || nocase(t, rru).is_eq()) && a.is_none()) {
                return Err(format!("поле роли {rru} выбрано не как Псевдоним.Поле: совпадение с основной таблицей не выведено"));
            }
            // поле роли под агрегатом выборки (`КОЛИЧЕСТВО(Т.КодМаркировки) КАК Количество`): колонку для поля ищет
            // mngbase FUN_18066a8a0 через +0x4c0 редактора запроса (entext, реализация не прочитана) и новую добавляет,
            // только если не нашел; по номерам FieldsMap ЭДО ПулКодовМаркировкиСУЗ поля роли в корне нет — поле
            // считается выбранным. shortcut: по сверке одной формы, реализация +0x4c0 не прочитана
            let role = |p: &[String]| (p.len() == 2 && nocase(&p[0], &malias).is_eq() && is(&p[1])) || (p.len() == 1 && malias.is_empty() && is(&p[0]));
            if s.fields.iter().any(|f| under_aggregate(&f.expr, &role, false)) {
                continue;
            }
            if !taken.iter().any(|n| nocase(n, rru).is_eq()) {
                fields.push(Field { name: r.to_string(), second: rru.to_string() });
                taken.push(rru.to_string());
                continue;
            }
            let n = (1..).map(|i| format!("{rru}{i}")).find(|c| !taken.iter().any(|n| nocase(n, c).is_eq())).unwrap_or_default();
            fields.push(Field { name: n.clone(), second: n.clone() });
            taken.push(n);
        }
    }
    // поля набора, объявленные в схеме (`<Field>` настроек): поле запроса с именем или вторым именем, равным полю
    // объявления, становится полем по пути объявления (dcs FUN_180305770: поиск в словаре полей набора по имени, затем
    // по второму имени, сравнение FUN_18003aed0)
    for (path, fld) in declared {
        if path.contains('.') || fld.contains('.') {
            // объявление через точку регистрируется под узлом своего пути (dcs FUN_180305770 с непустым путем: +0x28
            // разрешителя), корень не меняет; с тем же путем — ничего не переименовывает
            if path == fld {
                continue;
            }
            return Err(format!("поле набора {path} через точку в своем запросе пока не загружается"));
        }
        let i = fields.iter().position(|f| nocase(&f.name, fld).is_eq()).or_else(|| fields.iter().position(|f| nocase(&f.second, fld).is_eq()));
        match i {
            Some(i) => fields[i] = Field { name: path.clone(), second: path.clone() },
            // поля, которого нет в выборке, объявление в корень не добавляет: записи набора-запроса — только поля
            // текста (dcs FUN_180306bb0; объявленные `*(+8)+0x18` дают там лишь тип, записи из них строит обработчик
            // набора без запроса FUN_180306650), а невыбранные поля основной таблицы генератор дописывает в `{ГДЕ}`
            // (mngbase FUN_1806689c0 → FUN_18066a8a0), такие режим 9 корня отбрасывает (dcs FUN_180062e80)
            None => {}
        }
    }
    fields.sort_by(|a, b| nocase(&a.name, &b.name));
    if fields.windows(2).any(|w| nocase(&w[0].name, &w[1].name).is_eq()) {
        return Err("поля своего запроса с одинаковым именем без учета регистра".into());
    }
    if let Ok(p) = std::env::var("CFKIT_QP_DUMP") {
        use std::io::Write;
        let key: String = text.replace("\r\n", "\n").chars().take(150).collect();
        let line = serde_line(&key, &fields);
        if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(p) {
            let _ = writeln!(f, "{line}");
        }
    }
    Ok(Some(fields))
}

/// Схема дерева полей списка — по совместимости < 0x139bf (mngbase FUN_1802b6b10: источник ULDataSource, набор
/// ULDataSet), а не генератором схемы (FUN_180297ec0).
fn old_schema(x: &Index) -> bool {
    x.compat.is_some_and(|c| c < 0x139bf)
}

/// Есть ли в выражении путь `hit` под агрегатной функцией (`inside` — уже под ней).
fn under_aggregate(e: &crate::query::Expr, hit: &dyn Fn(&[String]) -> bool, inside: bool) -> bool {
    use crate::query::Expr::*;
    let f = |x: &crate::query::Expr| under_aggregate(x, hit, inside);
    match e {
        Path(p) => inside && hit(p),
        Call { name, args, .. } => {
            let agg = inside || crate::query::names::AGGREGATES.contains(&name.as_str());
            args.iter().any(|a| under_aggregate(a, hit, agg))
        }
        Neg(x) | Not(x) | Cast(x, _) | Refs(x, _) | Dot(x, _) | IsNull { x, .. } => f(x),
        Bin(_, a, b) => f(a) || f(b),
        Case { of, when, other } => of.as_deref().is_some_and(f) || when.iter().any(|(a, b)| f(a) || f(b)) || other.as_deref().is_some_and(f),
        In { x, list, .. } => f(x) || list.iter().any(f),
        Like { x, pat, esc, .. } => f(x) || f(pat) || esc.as_deref().is_some_and(f),
        Between { x, lo, hi, .. } => f(x) || f(lo) || f(hi),
        Tuple(v) => v.iter().any(f),
        Nested(_, fs) | Dcs(fs) => fs.iter().any(|g| f(&g.expr)),
        Star(_) | Num(_) | Str(_) | Param(_) | Const(_) | TypeOf(_) | Value(_) | Sub(_) => false,
    }
}

/// Поля выборки запроса, которых нет в его `{ВЫБРАТЬ}` (`Поле` или `Поле.*`); у `ОБЪЕДИНИТЬ` — поля и `{ВЫБРАТЬ}` первой
/// части (поля запроса — ее, entext FUN_18034f420); `None` — `{ВЫБРАТЬ}` нет, он не только в первой части или поле без
/// имени.
fn dcs_unlisted(text: &str) -> Option<Vec<String>> {
    use crate::query::Expr;
    fn name(f: &crate::query::Field) -> Option<&str> {
        match (&f.alias, &f.expr) {
            (Some(a), _) => Some(a),
            (None, Expr::Path(p) | Expr::Star(p)) => p.last().map(String::as_str),
            _ => None,
        }
    }
    let b = crate::query::parse(text).ok()?;
    let [p, rest @ ..] = b.queries.last()?.parts.as_slice() else { return None };
    if p.dcs_select.is_empty() || rest.iter().any(|r| !r.dcs_select.is_empty()) {
        return None;
    }
    let listed: Vec<&str> = p.dcs_select.iter().map(name).collect::<Option<_>>()?;
    let mut out = Vec::new();
    for f in &p.fields {
        let n = name(f)?;
        if !listed.iter().any(|m| nocase(m, n).is_eq()) {
            out.push(n.to_string());
        }
    }
    Some(out)
}

/// Элемент `{ГДЕ}` с именем колонки `name` (mngbase FUN_1806692a0: имя элемента +0x40, выражение +0x60; из одноименных
/// — последний): `Some(Some(р))` — поле, р — равно ли оно полю роли `alias.<поле>` (`is` — имя поля роли);
/// `Some(None)` — выражение не поле; `None` — элемента нет. Выражение без имени — условие, не элемент. Элемент `Поле`
/// без источника при источнике с псевдонимом и `{ГДЕ Т.*}` (состав элементов) не выведены — отказ, если от них зависит
/// ответ.
fn dcs_where_item(w: &[crate::query::Field], name: &str, alias: &str, is: &dyn Fn(&str) -> bool) -> R<Option<Option<bool>>> {
    use crate::query::Expr;
    let mut hit = None;
    let mut star = false;
    for f in w {
        let n = match (&f.alias, &f.expr) {
            (Some(a), _) => a,
            (None, Expr::Star(p)) if p.len() == 1 && !alias.is_empty() => {
                star = true;
                continue;
            }
            (None, Expr::Path(p) | Expr::Star(p)) => &p[p.len() - 1],
            _ => continue,
        };
        if !nocase(n, name).is_eq() {
            continue;
        }
        let e = match &f.expr {
            Expr::Dot(e, s) if s == &["*"] => e.as_ref(),
            e => e,
        };
        hit = Some(match e {
            // поле единственного источника без псевдонима (`alias` пуст); при псевдониме — не выведено
            Expr::Path(p) | Expr::Star(p) if p.len() == 1 => match alias.is_empty() {
                true => Some(is(&p[0])),
                false => return Err(format!("элемент {{ГДЕ}} {name} без источника: равенство выражению колонки не выведено")),
            },
            Expr::Path(p) | Expr::Star(p) => Some(p.len() == 2 && nocase(&p[0], alias).is_eq() && is(&p[1])),
            _ => None,
        });
    }
    // и при явном элементе: перекрывает ли его `Т.*` (из одноименных — последний), не выведено
    if star {
        return Err(format!("{{ГДЕ Т.*}} и колонка {name} в своем запросе: элементы расширения не выведены"));
    }
    Ok(hit)
}

/// Источник запроса — табличная часть объекта (`Справочник.Контрагенты.КонтактнаяИнформация`).
fn ts_source(x: &Index, t: &crate::query::Table) -> bool {
    use crate::query::resolve::Meta;
    match t {
        crate::query::Table::Name { path, params: None } if path.len() == 3 => {
            query_class(&path[0]).and_then(|c| obj_key(x, &format!("{c}.{}", path[1]))).is_some_and(|o| IndexMeta(x).ts(&o, &path[2]))
        }
        _ => false,
    }
}

/// Строка выгрузки корня для сверки с трассой: ключ (начало текста, `\n` как `\\n`) и поля `имя|второе`.
fn serde_line(key: &str, f: &[Field]) -> String {
    let k = key.replace('\\', "\\\\").replace('\n', "\\n").replace('\t', "\\t");
    let v: Vec<String> = f.iter().map(|f| format!("{}|{}", f.name, f.second)).collect();
    format!("{k}\t{}", v.join(";"))
}

/// Сравнение имен без учета регистра — core85 `core::compare_nocase` (0x1802d8370): по единицам UTF-16; у несовпавших —
/// верхний регистр, при несовпадении — нижний от него, разность кодов; общая часть равна — разность длин. ASCII и
/// 0x410–0x44F — свои ветки, прочее — ICU `u_toupper`/`u_tolower` (простое отображение); здесь из прочего — только
/// 0x400–0x45F (`nocase_ok`), у остальных символов имя отвергается.
fn nocase(a: &str, b: &str) -> std::cmp::Ordering {
    fn up(c: u16) -> u16 {
        match c {
            0x61..=0x7a | 0x430..=0x44f => c - 0x20,
            0..=0x7f | 0x410..=0x42f => c,
            _ => simple(c, char::to_uppercase),
        }
    }
    fn low(c: u16) -> u16 {
        match c {
            0x41..=0x5a | 0x410..=0x42f => c + 0x20,
            0..=0x7f | 0x430..=0x44f => c,
            _ => simple(c, char::to_lowercase),
        }
    }
    fn simple<I: ExactSizeIterator<Item = char>>(c: u16, f: fn(char) -> I) -> u16 {
        let ch = char::from_u32(c as u32).unwrap_or_default();
        let mut m = f(ch);
        match (m.len(), m.next()) {
            (1, Some(r)) if (r as u32) < 0x10000 => r as u32 as u16,
            _ => c,
        }
    }
    let (a, b): (Vec<u16>, Vec<u16>) = (a.encode_utf16().collect(), b.encode_utf16().collect());
    for (&x, &y) in a.iter().zip(&b) {
        if x != y {
            let (x, y) = (up(x), up(y));
            if x != y {
                let (x, y) = (low(x), low(y));
                if x != y {
                    return x.cmp(&y);
                }
            }
        }
    }
    a.len().cmp(&b.len())
}

/// Вычисляемые поля настроек `st` — в начало корня (`query::settings::calc_root`); поле набора с тем же именем выпадает.
/// Поле выражения `А.Б` — ребенок поля корня `А` (дети — `kids`, дерево полей набора); глубже — не выведено. Результат —
/// выведенные типы вычисляемых полей корня (`calc_kind`).
fn with_calc(x: &Index, fields: &mut Vec<Field>, st: &El, kids: &dyn Fn(&Field) -> R<Vec<Field>>) -> R<Vec<(String, R<Kind>)>> {
    let is = |f: &Field, n: &str| nocase(&f.name, n).is_eq();
    let has = |p: &str| -> R<bool> {
        let parts: Vec<&str> = p.split('.').collect();
        let Some(f) = fields.iter().find(|f| is(f, parts[0])) else { return Ok(false) };
        match parts.as_slice() {
            [_] => Ok(true),
            [_, b] => Ok(kids(f)?.iter().any(|k| is(k, b))),
            _ => Err("путь глубже двух частей: поиск в дереве полей не выведен".into()),
        }
    };
    let calc = crate::query::settings::calc_root(st, &has, nocase)?;
    if let Some(c) = calc.iter().find(|c| !nocase_ok(c)) {
        return Err(format!("вычисляемое поле {c}: сравнение без учета регистра с символами вне ASCII и кириллицы (ICU) не воспроизведено"));
    }
    fields.retain(|f| !calc.iter().any(|c| nocase(c, &f.name).is_eq()));
    // тип не выведен — отказ при раскрытии; выражение — само одноименное поле набора без valueType — тип этого поля (по
    // таблице, как поле корня)
    let kinds = calc.iter().filter_map(|c| {
        let e = st.peek_elems().find(|e| e.name == "CalculatedField" && e.peek_child("dcssch:dataPath").and_then(El::peek_text).as_deref() == Some(c))?;
        match calc_kind(x, e) {
            Ok(k) => Some((c.clone(), Ok(k))),
            Err(_) if e.peek_child("dcssch:valueType").is_none() && e.peek_child("dcssch:expression").and_then(El::peek_text).is_some_and(|v| nocase(v.trim(), c).is_eq()) => None,
            Err(m) => Some((c.clone(), Err(format!("вычисляемое поле {c}: {m}")))),
        }
    }).collect();
    fields.splice(0..0, calc.into_iter().map(|c| Field { second: c.clone(), name: c }));
    Ok(kinds)
}

/// Тип вычисляемого поля для раскрытия — его `valueType` (тип значения поля схемы): типы по имени и `TypeId`
/// определяемого типа (типы его описания). Без типа (тип по выражению), наборы типов, прочие `TypeId` — не выведены.
fn calc_kind(x: &Index, c: &El) -> R<Kind> {
    let vt = c.peek_child("dcssch:valueType").ok_or("тип вычисляемого поля без valueType (по выражению) не выведен")?;
    let mut v: Vec<String> = Vec::new();
    for t in vt.peek_elems() {
        let id = t.peek_text().unwrap_or_default();
        let add: Vec<String> = match t.name.as_str() {
            "v8:Type" => vec![id],
            "v8:TypeId" => {
                let d = x.types.iter().find(|(k, (tid, _))| *tid == id && k.starts_with("DefinedType.")).map(|(k, _)| k.clone()).ok_or_else(|| format!("TypeId {id} вычисляемого поля не определяемый тип"))?;
                attr_types(x, &d)?
            }
            n if n.ends_with("Qualifiers") => Vec::new(),
            n => return Err(format!("{n} в типе вычисляемого поля не выведен")),
        };
        v.extend(add.into_iter().filter(|a| !v.contains(a)).collect::<Vec<_>>());
    }
    Ok(Kind::Types(v))
}

/// Символы, у которых отображение регистра ICU совпадает с `char::to_*case` наверняка (ASCII, кириллица 0x400–0x45F).
fn nocase_ok(n: &str) -> bool {
    n.encode_utf16().all(|c| c < 0x80 || (0x400..=0x45f).contains(&c))
}

/// Имена, которые обычный путь `Список.<имя>` списка со своим запросом (настройки `st`) находит в корне до записи в
/// FieldsMap (как `List::id`: имя и второе имя поля корня, Owner — колонкой роли), — для признака `~` выгрузки
/// (`xml::form`): путь, который не разрешается, платформа пишет недоступным. Запрос не компилируется — имен нет.
pub(crate) fn path_names(x: &Index, st: &El) -> R<Vec<String>> {
    let l = List::from_query(x, st)?;
    let mut v: Vec<String> = l.fields.iter().flat_map(|f| [f.name.clone(), f.second.clone()]).collect();
    if l.roles.iter().any(|r| r == "Owner") && l.origins.iter().any(|(c, o, e)| *o == l.main && e == "Owner" && l.fields.iter().any(|f| f.name == *c)) {
        v.push("Owner".into());
    }
    Ok(v)
}

impl List {
    pub(super) fn new(x: &Index, main: &str, st: &El) -> R<List> {
        let mut fields = root_fields(x, main)?;
        let kinds = with_calc(x, &mut fields, st, &|f| children(x, &root_kind(x, main, &f.name)?, false))?;
        Ok(List { main: main.to_string(), fields, map: Vec::new(), custom: false, no_root: false, dp: None, ca: Vec::new(), expanded: false, origins: Vec::new(), roles: Vec::new(), types: Vec::new(), kinds, grouped: false })
    }

    /// Список со своим запросом: настройки `<Settings xsi:type="DynamicList">` (текст, основная таблица, поля набора).
    pub(super) fn from_query(x: &Index, st: &El) -> R<List> {
        let text = st.peek_child("QueryText").and_then(El::peek_text).ok_or("свой запрос без QueryText")?;
        let main = st.peek_child("MainTable").and_then(El::peek_text);
        let off = x.compat.is_none_or(|c| c >= 0x139b4) && st.peek_child("AutoFillAvailableFields").and_then(El::peek_text).as_deref() == Some("false");
        let unlisted = if off { dcs_unlisted(&text) } else { None };
        let mut declared = Vec::new();
        for f in st.peek_elems().filter(|e| e.name == "Field") {
            let g = |k: &str| f.peek_child(k).and_then(El::peek_text).unwrap_or_default();
            match f.peek_attr("xsi:type") {
                // при выключенном автозаполнении с `{ВЫБРАТЬ}` папки и поля через точку корень не меняют (ниже)
                Some("dcssch:DataSetFieldFolder") if unlisted.is_some() => continue,
                Some("dcssch:DataSetFieldField") if unlisted.is_some() && g("dcssch:dataPath").contains('.') => continue,
                Some("dcssch:DataSetFieldField") => {}
                // вложенный набор (dcs FUN_1800c2840: объявление QI DAT_180562c20, имя +0x28) — поле запроса-таблицы с тем
                // же путем; переименование вложенного набора не выведено
                Some("dcssch:DataSetFieldNestedDataSet") if g("dcssch:dataPath") == g("dcssch:field") => {}
                t => return Err(format!("поле набора {} в своем запросе пока не загружается", t.unwrap_or_default())),
            }
            declared.push((g("dcssch:dataPath"), g("dcssch:field")));
        }
        // автозаполнение доступных полей набора: настройки списка +0x278 → mngbase FUN_180297ec0 → ULSchemeInfoImpl
        // FUN_18067f990 (+0x88, геттер +0x70 FUN_180680050) → построитель FUN_1806787b0 (байт +0x20: при совместимости
        // < 0x139b4 — 1, иначе это значение) → FUN_18067a560 (+0x58 набора) → dcs FUN_1800c8760 (+0x50 набора в +0x30
        // описания текста) → FUN_180306bb0 (+0x10 посетителя): от него зависят поля и параметры набора
        // (FUN_18030c790). Выключено и полей набора не объявлено — корень пуст (сверка АнализПравДоступа ЭДО); все поля
        // с `{ВЫБРАТЬ}` — корень как при включенном без полей выборки, которых в `{ВЫБРАТЬ}` нет; поля ролей остаются,
        // объявленные папки и поля через точку не добавляются (все 11 номеров FieldsMap ФормаПодбораДокументовЗатрат БП,
        // ФормаВыбораОснованияУвольнения БП; с `ОБЪЕДИНИТЬ` и `{ВЫБРАТЬ}` первой части — ФормированиеУведомленийОВвозеПеремещении
        // ERP). shortcut: по четырем формам; прочие сочетания — отказ
        if off && unlisted.is_none() {
            if !declared.is_empty() || text.contains('&') {
                return Err("автозаполнение доступных полей своего запроса выключено, поля или параметры набора есть: корень не выведен".into());
            }
            return Ok(List { main: main.unwrap_or_default(), fields: Vec::new(), map: Vec::new(), custom: true, no_root: false, dp: None, ca: Vec::new(), expanded: false, origins: Vec::new(), roles: Vec::new(), types: Vec::new(), kinds: Vec::new(), grouped: false });
        }
        // некомпилируемый запрос: корень пуст целиком, без `DataParameters`
        let Some(mut fields) = query_root(x, main.as_deref(), &text, &declared, !off)? else {
            return Ok(List { main: main.unwrap_or_default(), fields: Vec::new(), map: Vec::new(), custom: true, no_root: true, dp: None, ca: Vec::new(), expanded: false, origins: Vec::new(), roles: Vec::new(), types: Vec::new(), kinds: Vec::new(), grouped: false });
        };
        for n in unlisted.iter().flatten() {
            fields.retain(|f| !nocase(&f.name, n).is_eq() && !nocase(&f.second, n).is_eq());
        }
        // поле набора с ограничением «поле» (`useRestriction/field` истина) в корень режима 9 не попадает (dcs
        // FUN_1800a9840 / FUN_180062dd0, слот +0x18; ДиспетчированиеСборкаИДоставка ERP: ДатаСобытия, Просрочен)
        let restricted: Vec<String> = st.peek_elems().filter(|e| e.name == "Field").filter(|f| {
            f.peek_child("dcssch:useRestriction").and_then(|u| u.peek_child("dcssch:field")).and_then(El::peek_text).as_deref() == Some("true")
        }).filter_map(|f| f.peek_child("dcssch:dataPath").and_then(El::peek_text)).collect();
        fields.retain(|f| !restricted.iter().any(|r| nocase(&f.name, r).is_eq()));
        let mut kinds = with_calc(x, &mut fields, st, &|f| Err(format!("дети поля {} своего запроса для выражения не выведены", f.name)))?;
        let params: Vec<_> = st.peek_elems().filter(|e| e.name == "Parameter").map(|p| (p.peek_child("dcssch:name").and_then(El::peek_text).unwrap_or_default(), p.peek_child("dcssch:availableAsField").and_then(El::peek_text).as_deref() != Some("false"))).collect();
        let b = crate::query::parse(&text)?;
        let untyped: Vec<String> = st.peek_elems().filter(|e| e.name == "Parameter" && e.peek_child("dcssch:valueType").is_none()).filter_map(|p| p.peek_child("dcssch:name").and_then(El::peek_text)).filter(|n| whole_only(&text, &b, n)).collect();
        // совместимость < 0x139bf: элемент дерева с байтом +0x71 и без +0x72 mngbase FUN_1802b76a0 пропускает, а
        // `DataParameters` dcs FUN_1800b1e00 дает именно таким (+0x71 = 1)
        let dp = if old_schema(x) { Ok(false) } else { crate::query::dcs::data_parameters(&IndexMeta(x), &text, &b, &params) };
        if dp == Ok(true) {
            fields.push(Field { name: "DataParameters".into(), second: "ПараметрыДанных".into() });
        }
        // роль Owner для пути `Список.Owner` (List::id) — только при пункте `{ГДЕ Т.Владелец.*}` без псевдонима
        // (ДокументыПодтверждающиеЛьготы… БП); без него путь не разрешается и `~Список.Owner` берет номер счетчика
        // (Сотрудники.ФормаСписка ERP, список Подразделения: Owner 58, Владелец 85). shortcut: по двум формам, слияние
        // пункта `{ГДЕ}` с полем выборки в dcs не прочитано
        let owner_where = b.queries.last().and_then(|q| q.parts.first()).is_some_and(|s| {
            s.dcs_where.iter().any(|f| f.alias.is_none() && matches!(&f.expr, crate::query::Expr::Path(p) | crate::query::Expr::Star(p) if p.len() == 2 && (nocase(&p[1], "Владелец").is_eq() || nocase(&p[1], "Owner").is_eq())))
        });
        let roles = main.as_deref().filter(|_| !old_schema(x)).and_then(|m| crate::query::roles::main_roles(x, m).ok()).unwrap_or_default().into_iter().filter(|r| r != "Owner" || owner_where).collect();
        kinds.push(("DataParameters".into(), crate::query::dcs::parameter_fields(&IndexMeta(x), &text, &b, &params).map(Kind::Params)));
        Ok(List { main: main.clone().unwrap_or_default(), fields, map: Vec::new(), custom: true, no_root: false, dp: dp.err(), ca: Vec::new(), expanded: false, origins: query_origins(x, &text)?, roles, types: query_types(x, &text, &untyped), kinds, grouped: grouped(&text) })
    }

    /// Поле корня уже в FieldsMap: по имени или (поле роли — `List::id`) по второму имени записи, чье имя не поле корня.
    fn covered(&self, f: &Field) -> bool {
        self.map.iter().any(|(_, m)| m.name == f.name || (m.second == f.name && !self.fields.iter().any(|g| g.name == m.name)))
    }

    /// Номер поля корня `n` (по FieldsMap или новый — как дает перестроение платформы). Обычный путь разрешается по
    /// дереву полей (mngbase FUN_1802cbc20): поле вне корня, которое в FieldsMap записал путь `~` (FUN_1802c8a30), им
    /// не находится (БП ДокументыПоТребованиюФНС: `Список.Выбран` условного оформления у платформы без номера).
    pub(super) fn id(&mut self, n: &str) -> Result<i64, String> {
        // поиск поля корня по имени, затем по второму имени (dcs FUN_180305770): `Список.Период` → Period
        // (ОтветственныеЛицаОрганизаций БП: в FieldsMap Period/Период)
        let by_name = self.fields.iter().any(|f| f.name == n) || self.map.iter().any(|(_, f)| f.name == n);
        let canon = self.fields.iter().find(|f| !by_name && f.second == n).map(|f| f.name.clone());
        // поле роли основной таблицы, выбранное колонкой под своим именем (`Т.Владелец КАК Владелец`): путь `Список.Owner`
        // дает номер колонки, в FieldsMap — роль и колонка вторым именем (Owner/Владелец, ДокументыПодтверждающиеЛьготы…
        // БП: номера следующих полей колонку уже не считают); `Список.Ref` так не разрешается (ФормыСписка БП с
        // `Т.Ссылка КАК Ссылка`). shortcut: только Owner, по одной форме, при `{ГДЕ Т.Владелец.*}` (роль отбирает
        // `from_query`); механизм в mngbase не найден
        if !by_name && canon.is_none() && n == "Owner" && self.roles.iter().any(|r| r == n) {
            if let Some((col, ..)) = self.origins.iter().find(|(c, o, e)| *o == self.main && e == n && self.fields.iter().any(|f| f.name == *c)) {
                let col = col.clone();
                let id = match self.map.iter().find(|(_, f)| f.name == col) {
                    Some((id, _)) => *id,
                    None => self.peek(&col).ok_or_else(|| format!("поле {col} не найдено среди полей {}", self.main))?,
                };
                self.map.push((id, Field { name: n.to_string(), second: col }));
                self.expanded = false;
                return Ok(id);
            }
        }
        let n = canon.as_deref().unwrap_or(n);
        if let Some((id, m)) = self.map.iter().find(|(_, f)| f.name == n) {
            // поле корня или запись поля роли (выше) с колонкой корня
            if self.fields.iter().any(|f| f.name == n || (n == "Owner" && self.roles.iter().any(|r| r == n) && f.name == m.second)) {
                return Ok(*id);
            }
            // обычный путь не разрешен: mngbase FUN_18075f8c0 без `~` дает 1 (ошибка), читатели путей оставляют путь
            // пустым — условное оформление пишет поле без пути (`conditional_appearance`, «не найдено»), прочие — отказ
            return Err(format!("поле {n} вне корня {} (есть только по пути `~`): обычным путем не найдено", self.main));
        }
        let mut c = self.map.iter().map(|(i, _)| *i).max().unwrap_or(0) + 1;
        for f in &self.fields {
            if self.covered(f) {
                continue;
            }
            if f.name == n {
                self.map.push((c, f.clone()));
                self.expanded = false;
                return Ok(c);
            }
            c += 1;
        }
        Err(format!("поле {n} не найдено среди полей {}", self.main))
    }

    /// Номер поля корня `n` без записи в FieldsMap (как `id`): `None` — поля нет в корне.
    fn peek(&self, n: &str) -> Option<i64> {
        // по имени, затем по второму имени (как `id`)
        let by_name = self.fields.iter().any(|f| f.name == n) || self.map.iter().any(|(_, f)| f.name == n);
        let canon = self.fields.iter().find(|f| !by_name && f.second == n).map(|f| f.name.clone());
        let n = canon.as_deref().unwrap_or(n);
        if let Some((id, _)) = self.map.iter().find(|(_, f)| f.name == n) {
            return self.fields.iter().any(|f| f.name == n).then_some(*id);
        }
        let mut c = self.map.iter().map(|(i, _)| *i).max().unwrap_or(0) + 1;
        for f in self.fields.iter().filter(|f| !self.covered(f)) {
            if f.name == n {
                return Some(c);
            }
            c += 1;
        }
        None
    }

    /// Номер вложенного поля `a.b`. Объект поля формы (mngbase FUN_18029a6f0, байт формы +0x360 нулевой) раскрывает
    /// каждого ребенка узла уровня по порядку — для `a.b` всех детей корня (путь `<имя>` → FUN_1802b76a0); раскрытие
    /// нумерует сразу всех детей узла: номер из FieldsMap по полному имени (`+0x50`), иначе счетчик дерева (+0x20).
    /// Счетчик — после перестроения (FUN_1802b85b0: наибольший номер FieldsMap + 1, затем поля корня). Новое поле
    /// пишется в FieldsMap (FUN_1805f18b0) — перестроение; путь, уже бывший в FieldsMap, оставляет дерево раскрытым.
    /// Совместимость < 0x139bf: FUN_1802b76a0 пропускает элементы с байтом +0x71 без +0x72 (части даты — `children`,
    /// `DataParameters`); свой запрос — типы полей корня не известны.
    pub(super) fn nested_id(&mut self, x: &Index, a: &str, b: &str) -> Result<(i64, i64), String> {
        // номер `a` — по корню на момент перестроения; оба поля пишутся в FieldsMap после разбора пути
        let ida = self.peek(a).ok_or_else(|| format!("поле {a} не найдено среди полей {}", self.main))?;
        let (idab, f) = self.nested(x, a, b)?;
        let fa = self.fields.iter().find(|m| m.name == a).cloned().ok_or("поле корня")?;
        if !self.covered(&fa) {
            self.map.push((ida, fa));
        }
        match f {
            Some(f) => {
                self.map.push((idab, f));
                self.expanded = false;
            }
            None => self.expanded = true,
        }
        Ok((ida, idab))
    }

    /// Тип поля корня `f` для раскрытия. Свой запрос: тип поля `Псевдоним.Поле` таблицы объекта, недописанные роли —
    /// основной таблицы; прочие (выражения, производные источники) — не выведены.
    fn field_kind(&self, x: &Index, f: &Field) -> R<Kind> {
        // `DataParameters` — только дописанный набором (второе имя ПараметрыДанных), не поле запроса с этим именем
        if let Some((_, k)) = self.kinds.iter().find(|(n, _)| *n == f.name && (n != "DataParameters" || f.second == "ПараметрыДанных")) {
            return k.clone();
        }
        if !self.custom {
            return root_kind(x, &self.main, &f.name);
        }
        // поле роли, дописанное под английским именем (Ref при колонке `0 КАК Ссылка`), по второму имени чужую колонку не
        // берет: его тип — роль основной таблицы
        let by = |k: &str| self.origins.iter().find(|(n, _, _)| nocase(n, k).is_eq());
        let role = self.roles.iter().any(|r| *r == f.name);
        match by(&f.name).or_else(|| if role { None } else { by(&f.second) }) {
            Some((_, obj, t)) if obj.is_empty() => Ok(Kind::Types(vec![t.clone()])),
            Some((_, obj, n)) => root_kind(x, obj, n),
            // поле роли, дописанное генератором (`query_root_fields`): русское имя стандартного поля роли
            None if let Some(r) = self.roles.iter().find(|r| **r == f.name || STD.iter().any(|(e, ru)| e == r && nocase(ru, &f.name).is_eq())) => root_kind(x, &self.main, r),
            None => match self.types.iter().find(|(n, _)| nocase(n, &f.name).is_eq() || nocase(n, &f.second).is_eq()) {
                Some((_, Some(t))) => xml_types(t).map(Kind::Types).map_err(|m| format!("поле {} своего запроса: {m}", f.name)),
                _ => Err(format!("поле {} своего запроса не из таблицы объекта: тип не выведен", f.name)),
            },
        }
    }

    /// Поиск в FieldsMap (+0x50 FUN_1805f1830): по имени, затем по второму имени.
    fn in_map(&self, n: &str, r: &str) -> Result<Option<i64>, String> {
        for k in [n, r] {
            if let Some((id, _)) = self.map.iter().find(|(_, f)| f.name == k || (!f.second.is_empty() && f.second == k)) {
                return Ok(Some(*id));
            }
        }
        if self.map.iter().any(|(_, f)| [n, r].iter().any(|k| nocase(&f.name, k).is_eq() || (!f.second.is_empty() && nocase(&f.second, k).is_eq()))) {
            return Err(format!("поле {n}: в FieldsMap только без учета регистра — сравнение не выведено"));
        }
        Ok(None)
    }

    /// Номер `a.b.c` (как `nested_id`, уровнем глубже): раскрытие второго уровня нумерует детей всех полей корня,
    /// третьего — детей каждого ребенка `a` по порядку (FUN_1802b76a0 на узле уровня); номера — счетчик дерева после
    /// перестроения, поля из FieldsMap его не двигают. shortcut: по сверке двух форм БП (СписокСтатейЗатрат…,
    /// ВыгрузкаРегламентированныхОтчетов), дети ребенка с полем в нескольких таблицах типа — отказ
    pub(super) fn nested3_id(&mut self, x: &Index, a: &str, b: &str, c: &str) -> Result<(i64, i64, i64), String> {
        let ida = self.peek(a).ok_or_else(|| format!("поле {a} не найдено среди полей {}", self.main))?;
        let (idab, fab) = self.nested(x, a, b)?;
        let full = format!("{a}.{b}.{c}");
        let mut new = Vec::new();
        let idabc = match self.in_map(&full, &full)? {
            Some(id) => id,
            None => {
                let max = self.map.iter().map(|(i, _)| *i).max().unwrap_or(0);
                let mut n = max + 1 + self.fields.iter().filter(|f| !self.covered(f)).count() as i64;
                let mut at = None;
                for f in &self.fields {
                    let kind = self.field_kind(x, f).map_err(|m| format!("раскрытие {}: {m}", f.name))?;
                    for k in children(x, &kind, false).map_err(|m| format!("раскрытие {}: {m}", f.name))? {
                        if self.in_map(&format!("{}.{}", f.name, k.name), &format!("{}.{}", f.second, k.second))?.is_none() {
                            n += 1;
                        }
                    }
                    if f.name == a {
                        at = Some((f.clone(), kind));
                    }
                }
                let (fa, kind) = at.ok_or_else(|| format!("поле {a} не найдено среди полей {}", self.main))?;
                let in_ts = matches!(kind, Kind::Ts(_));
                let mut found = None;
                for k in children(x, &kind, false)? {
                    let kk = kid_kind(x, &kind, &k.name).map_err(|m| format!("раскрытие {a}.{}: {m}", k.name))?;
                    let kids = children(x, &kk, in_ts).map_err(|m| format!("раскрытие {a}.{}: {m}", k.name))?;
                    for g in kids {
                        let (pn, pr) = (format!("{a}.{}.{}", k.name, g.name), format!("{}.{}.{}", fa.second, k.second, g.second));
                        if k.name == b && g.name == c {
                            found = Some((n, Field { name: pn, second: pr }));
                            break;
                        }
                        if self.in_map(&pn, &pr)?.is_none() {
                            n += 1;
                        }
                    }
                    if found.is_some() || k.name == b {
                        break;
                    }
                }
                let (id, f) = found.ok_or_else(|| format!("поле {c} не найдено среди детей {a}.{b} ({})", self.main))?;
                new.push((id, f));
                id
            }
        };
        let fa = self.fields.iter().find(|m| m.name == a).cloned().ok_or("поле корня")?;
        if !self.covered(&fa) {
            self.map.push((ida, fa));
        }
        new.extend(fab.map(|f| (idab, f)));
        self.expanded = new.is_empty();
        self.map.extend(new);
        Ok((ida, idab, idabc))
    }

    /// Номер `a.b` и запись FieldsMap для него (`None` — уже есть): имя — полный путь, второе — вторые имена частей.
    fn nested(&self, x: &Index, a: &str, b: &str) -> Result<(i64, Option<Field>), String> {
        if x.compat.is_none() {
            return Err("поле динамического списка через поле без режима совместимости: отбор детей узла не выведен".into());
        }
        let in_map = |n: &str, r: &str| self.in_map(n, r);
        let full = format!("{a}.{b}");
        if let Some(id) = in_map(&full, &full)? {
            return Ok((id, None));
        }
        let max = self.map.iter().map(|(i, _)| *i).max().unwrap_or(0);
        // shortcut: у сгруппированного запроса `DataParameters` счетчик раскрытия не занимает — по двум формам ERP
        // (ЦеныНоменклатуры2_5, МониторингЗаказаНормативныйГрафик), в dcs не прослежено; при отличиях — FUN_1800b1e00
        let mut c = max + 1 + self.fields.iter().filter(|f| !self.covered(f) && !(self.grouped && f.name == "DataParameters")).count() as i64;
        for f in &self.fields {
            let kind = self.field_kind(x, f).map_err(|m| format!("раскрытие {}: {m}", f.name))?;
            let kids = children(x, &kind, self.grouped).map_err(|m| format!("раскрытие {}: {m}", f.name))?;
            let path = |k: &Field| (format!("{}.{}", f.name, k.name), format!("{}.{}", f.second, k.second));
            if f.name != a {
                for k in &kids {
                    let (n, r) = path(k);
                    if in_map(&n, &r)?.is_none() {
                        c += 1;
                    }
                }
                continue;
            }
            let pos = |kids: &[&Field]| -> Result<Option<(i64, Field)>, String> {
                let mut c = c;
                for k in kids {
                    let (n, r) = path(k);
                    if k.name == b {
                        return Ok(Some((c, Field { name: n, second: r })));
                    }
                    if in_map(&n, &r)?.is_none() {
                        c += 1;
                    }
                }
                Ok(None)
            };
            let all: Vec<&Field> = kids.iter().collect();
            let (id, fb) = pos(&all)?.ok_or_else(|| format!("поле {b} не найдено среди детей {a} ({})", self.main))?;
            return Ok((id, Some(fb)));
        }
        Err(format!("поле {a} не найдено среди полей {}", self.main))
    }

    /// Номер поля `~Список.n`. Разбор пути с `~` (mngbase FUN_18075f8c0) сначала разрешает его как обычный (поле
    /// корня); иначе — +0x5f8 разрешителя пути (класс CLSID eb0b9e48…, vtable 0x180835bd0, создает backend
    /// FUN_1803712c0 для контекста читателя +0x130): mngbase FUN_1802c8a30 ищет поле в FieldsMap по именам (+0x50
    /// словаря), а нет — дает номер счетчика дерева полей (+0x20 узла, после перестроения — за полями корня) и
    /// добавляет поле в FieldsMap (+0x60: имена пути и запасного пути — без `~…` второго имени нет).
    /// Обычный разбор (+0x5e8 → FUN_1802cbb80 → +0x5f0) берет только основной путь; поиск в FieldsMap (+0x50
    /// FUN_1805f1830) — сначала по основному имени, затем по запасному, в общем указателе имен и вторых имен
    /// (FUN_1805f1560); новое поле (+0x60 FUN_1805f18b0) — имя основного пути, второе — запасного, если отличается без
    /// учета регистра (FUN_1805ce200).
    fn unavailable_id(&mut self, n: &str, fb: Option<&str>) -> Result<i64, String> {
        if let Ok(id) = self.id(n) {
            return Ok(id);
        }
        for k in std::iter::once(n).chain(fb) {
            if let Some(id) = self.known(k)? {
                return Ok(id);
            }
        }
        if self.expanded {
            return Err(format!("недоступное поле {n} после раскрытия узлов корня без перестроения: счетчик дерева не выведен"));
        }
        let max = self.map.iter().map(|(i, _)| *i).max().unwrap_or(0);
        let id = max + 1 + self.fields.iter().filter(|f| !self.covered(f)).count() as i64;
        let second = fb.filter(|f| !nocase(f, n).is_eq()).unwrap_or_default();
        self.map.push((id, Field { name: n.to_string(), second: second.to_string() }));
        Ok(id)
    }

    /// Номер поля FieldsMap по имени или второму имени. shortcut: сравнение указателя (FUN_1805f1560) не прослежено —
    /// совпадение только без учета регистра — отказ.
    fn known(&self, k: &str) -> Result<Option<i64>, String> {
        let hit = |f: &Field, eq: &dyn Fn(&str) -> bool| eq(&f.name) || (!f.second.is_empty() && eq(&f.second));
        if let Some((id, _)) = self.map.iter().find(|(_, f)| hit(f, &|x| x == k)) {
            return Ok(Some(*id));
        }
        if self.map.iter().any(|(_, f)| hit(f, &|x| nocase(x, k).is_eq())) {
            return Err(format!("поле {k}: в FieldsMap только без учета регистра — сравнение не выведено"));
        }
        Ok(None)
    }
}

/// Свойства логической формы типа `LFEDataPath` (модель XDTO logform), которые читатель разбирает отложенно.
const LFE_PATHS: &[&str] = &[
    "DataPath",
    "RowPictureDataPath",
    "HeaderDataPath",
    "FooterDataPath",
    "TitleDataPath",
    "MultipleValueDataPath",
    "MultipleValuePictureDataPath",
    "MultipleValuePresentDataPath",
    "Field",
    "ReportResult",
    "DetailsData",
    "VariantAppearance",
];

impl F<'_> {
    /// Поля динамического списка (любого: ФормаСписка ПравилаЛимитовПоДаннымБюджетирования ERP — `Список.ЭтоГруппа`
    /// у списка без своего запроса), которые есть только в условном оформлении формы (`List::ca`): номер им дает
    /// задача условного оформления (`list_paths_in_load_order`), в FieldsMap они не пишутся.
    pub(super) fn list_ca_only(&self, doc: &El) -> R<()> {
        fn walk(e: &El, ca: bool, paths: &mut Vec<String>, fields: &mut Vec<String>) {
            for c in e.peek_elems() {
                let in_ca = ca || c.name == "ConditionalAppearance";
                match c.peek_text() {
                    Some(t) if in_ca && c.peek_attr("xsi:type") == Some("dcscor:Field") => fields.push(t),
                    Some(t) if !in_ca && LFE_PATHS.contains(&c.name.as_str()) => paths.push(t),
                    _ => walk(c, in_ca, paths, fields),
                }
            }
        }
        let mut lists = self.lists.borrow_mut();
        if lists.is_empty() {
            return Ok(());
        }
        let (mut paths, mut fields) = (Vec::new(), Vec::new());
        walk(doc, false, &mut paths, &mut fields);
        for f in fields {
            let Some((attr, n)) = f.split_once('.') else { continue };
            if !lists.contains_key(attr) || n == "DefaultPicture" {
                continue;
            }
            // текущие данные — по имени элемента-таблицы списка (как разрешает путь `path_text`)
            let used = paths.iter().any(|p| {
                let p = p.trim_start_matches('~');
                p == f
                    || p.strip_prefix("Items.").and_then(|r| r.split_once(".CurrentData.")).is_some_and(|(it, r)| {
                        r == n && self.items.get(it).is_some_and(|i| i.tag == "Table" && i.data_path == attr)
                    })
            });
            let l = lists.get_mut(attr).ok_or("список")?;
            if !used && !l.ca.iter().any(|(m, _)| m == n) {
                l.ca.push((n.to_string(), None));
            }
        }
        Ok(())
    }

    /// Номера полей динамических списков — в порядке, в котором их дает загрузка платформы. Путь к данным читатель
    /// не разбирает сразу (mngbase FUN_180760000): задача откладывается в очередь читателя (+0x48) с приоритетом 2,
    /// если первая часть пути `Items` или `~Items`, иначе 1. Очередь (backend FUN_1803746c0, FUN_180377b30) —
    /// `std::map<приоритет, deque>`: исполняется по возрастанию приоритета (FUN_1803747d0), внутри — по порядку
    /// постановки (FUN_18036bbd0), то есть по порядку документа. Поля «использовать всегда» (`<UseAlways><Field>`,
    /// mngbase FUN_18073cf60) идут той же очередью. Здесь пути разрешаются заранее в этом порядке; номера полей
    /// (FieldsMap) потом берутся из списка, ошибки — забота самой загрузки элемента. Условное оформление формы
    /// разбирает задача той же очереди с приоритетом 1 (mngbase FUN_18073b5c0: задача FUN_1807515d0 ставится через +0x48
    /// читателя с приоритетом 1 при чтении условного оформления; тело FUN_18074fdf0 → FUN_18073b170 разбирает поля через
    /// +0x5e8 FUN_1802cbb80 без +0x538 — без записи в FieldsMap): номер поля, которого нет в FieldsMap, — по дереву
    /// полей в этот момент (`List::peek`).
    pub(super) fn list_paths_in_load_order(&self, doc: &El) {
        fn walk<'e>(e: &'e El, out: &mut Vec<(u8, String)>) {
            for c in e.peek_elems() {
                if c.name == "ConditionalAppearance" && e.name == "Attributes" {
                    out.push((1, String::new()));
                    continue;
                }
                match c.peek_text() {
                    Some(t) if LFE_PATHS.contains(&c.name.as_str()) => {
                        let first = t.split('.').next().unwrap_or_default();
                        out.push((if first == "Items" || first == "~Items" { 2 } else { 1 }, t));
                    }
                    _ => walk(c, out),
                }
            }
        }
        if self.lists.borrow().is_empty() {
            return;
        }
        let mut v = Vec::new();
        walk(doc, &mut v);
        v.sort_by_key(|(p, _)| *p);
        for (_, t) in v {
            if t.is_empty() {
                for l in self.lists.borrow_mut().values_mut() {
                    let ids: Vec<_> = l.ca.iter().map(|(n, _)| l.peek(n)).collect();
                    for ((_, id), p) in l.ca.iter_mut().zip(ids) {
                        *id = p;
                    }
                }
                continue;
            }
            let _ = self.path_text(&t);
        }
    }

    /// Номер поля `n` динамического списка реквизита `attr`.
    pub(super) fn list_field(&self, attr: &str, n: &str) -> Result<Node, String> {
        let mut lists = self.lists.borrow_mut();
        // поле корня с этим именем (вычисляемое `СтандартнаяКартинка`) — раньше картинки: дети узла списка — поля дерева,
        // картинка строки — после них (mngbase FUN_180299750: номер 10000000 за последним ребенком)
        if is_default_picture(n) && !lists.get(attr).is_some_and(|l| l.fields.iter().any(|f| f.name == n)) {
            return Ok(l(vec![a(DEFAULT_PICTURE)]));
        }
        // свойства самого списка (обратное `std_field` выгрузки, ("DynamicList", −1/−2/−3)): ОбработчикиОбновления Клиента ЭДО
        // `Список.Order` → `{2,{1},{-1}}`, в FieldsMap не пишется
        let own = match n {
            "Order" => Some("-1"),
            "Filter" => Some("-2"),
            // ERP ТоварныеОграничения ФормаСписка, НастройкаПоддержанияЗапасов (свой запрос): `Список.Group` → `{2,{1},{-3}}`
            "Group" => Some("-3"),
            _ => None,
        };
        if let Some(c) = own.filter(|_| !lists.get(attr).is_some_and(|l| l.fields.iter().any(|f| f.name == n))) {
            return Ok(l(vec![a(c)]));
        }
        let list = lists.get_mut(attr).ok_or_else(|| format!("динамический список {attr} без основной таблицы пока не загружается"))?;
        // обычный путь у некомпилируемого запроса не разрешается (mngbase FUN_1802cbc20 — по дереву, оно пусто), разбор
        // пути без `~` (FUN_18075f8c0) дает ошибку — как поле, которого нет: условное оформление формы оставляет путь
        // пустым (`conditional_appearance`), прочие места — отказ
        if list.no_root {
            return Err(format!("поле {n} списка {attr}: запрос не компилируется, в пустом корне не найдено"));
        }
        // поле только из условного оформления: номер задачи условного оформления, без записи в FieldsMap; поля нет в
        // корне — разбор не удался, условное оформление пишет поле без пути («не найдено», как `Список.Выбран`)
        if let Some((_, id)) = list.ca.iter().find(|(m, _)| m == n) {
            if let Some(id) = id {
                return Ok(l(vec![an(*id)]));
            }
            return Err(format!("поле {n} списка {attr} только в условном оформлении: в дереве полей не найдено"));
        }
        Ok(l(vec![an(list.id(n)?)]))
    }

    /// Части пути вложенного поля `a.b` динамического списка реквизита `attr`: `{<номер a>},{<номер a.b>}` (см.
    /// `List::nested_id`).
    pub(super) fn list_field_nested3(&self, attr: &str, a: &str, b: &str, c: &str) -> Result<[Node; 3], String> {
        let mut lists = self.lists.borrow_mut();
        let list = lists.get_mut(attr).ok_or_else(|| format!("динамический список {attr} без основной таблицы пока не загружается"))?;
        if list.ca.iter().any(|(m, _)| m == a) || is_default_picture(a) || list.no_root {
            return Err(format!("поле {a}.{b}.{c} списка {attr}: поле {a} только в условном оформлении, картинка или нет корня — не выведено"));
        }
        let (ida, idab, idabc) = list.nested3_id(self.x, a, b, c)?;
        Ok([l(vec![an(ida)]), l(vec![an(idab)]), l(vec![an(idabc)])])
    }

    pub(super) fn list_field_nested(&self, attr: &str, a: &str, b: &str) -> Result<[Node; 2], String> {
        let mut lists = self.lists.borrow_mut();
        let list = lists.get_mut(attr).ok_or_else(|| format!("динамический список {attr} без основной таблицы пока не загружается"))?;
        if list.ca.iter().any(|(m, _)| m == a) || is_default_picture(a) || list.no_root {
            return Err(format!("поле {a}.{b} списка {attr}: поле {a} только в условном оформлении, картинка или нет корня — не выведено"));
        }
        let (ida, idab) = list.nested_id(self.x, a, b)?;
        Ok([l(vec![an(ida)]), l(vec![an(idab)])])
    }

    /// Номер поля `~attr.n` динамического списка (см. `List::unavailable_id`); `None` — `attr` не список.
    /// Недоступное поле `n` (запасное `fb`) списка `attr`; `None` — `attr` не динамический список.
    pub(super) fn list_field_unavailable(&self, attr: &str, n: &str, fb: Option<&str>) -> Option<Result<Node, String>> {
        let mut lists = self.lists.borrow_mut();
        let list = lists.get_mut(attr)?;
        // `DataParameters` в конце корня не выведен (`List::dp`) — недоступное поле вне корня: отказ
        if list.dp.is_some() && list.id(n).is_err() {
            return Some(Err(format!("недоступное поле {n} списка {attr}: {}", list.dp.clone().unwrap_or_default())));
        }
        Some(list.unavailable_id(n, fb).map(|id| l(vec![an(id)])))
    }

    /// Путь недоступного поля без ведущей `~`: основной и запасной (вторая `~` — при совместимости > 0x139b6, mngbase
    /// FUN_18075f8c0); `None` — режим совместимости не известен.
    pub(super) fn tilde_split<'t>(&self, r: &'t str) -> Option<(&'t str, Option<&'t str>)> {
        match r.split_once('~') {
            None => Some((r, None)),
            Some((a, b)) => self.x.compat.filter(|c| *c > 0x139b6).map(|_| (a, Some(b))),
        }
    }

    /// Поля «использовать всегда» динамического списка: номера (платформа читает их вместе с реквизитом — номера новых
    /// полей идут за полями элементов).
    pub(super) fn list_use_always(&self, attr: &str, u: &El) -> R<()> {
        let mut v = Vec::new();
        for f in u.elems() {
            if f.name != "Field" {
                return Err(err(f, "ожидалось <Field>"));
            }
            let t = f.text()?;
            let (tilde, t) = t.strip_prefix('~').map_or((false, t.as_str()), |r| (true, r));
            let (t, fb) = if tilde { self.tilde_split(t).ok_or_else(|| err(f, "запасной путь без режима совместимости"))? } else { (t, None) };
            let field = |t: &str| t.strip_prefix(&format!("{attr}.")).filter(|n| !n.contains(['.', '~'])).map(str::to_string);
            let n = field(t).ok_or_else(|| err(f, "поле «использовать всегда» не поле списка"))?;
            let fb = fb.map(|b| field(b).ok_or_else(|| err(f, "запасной путь поля «использовать всегда» не поле списка"))).transpose()?;
            let n = n.as_str();
            let node = if tilde && !is_default_picture(n) {
                self.list_field_unavailable(attr, n, fb.as_deref()).ok_or_else(|| err(f, &format!("недоступное поле {n}: {attr} не динамический список")))?.map_err(|m| err(f, &m))?
            } else {
                self.list_field(attr, n).map_err(|m| err(f, &m))?
            };
            v.push(node.list().first().and_then(Node::atom).and_then(|x| x.parse().ok()).ok_or_else(|| err(f, "номер поля"))?);
        }
        self.list_req.borrow_mut().insert(attr.to_string(), v);
        Ok(())
    }

    /// Словарь настроек динамического списка `{0,N,"ключ",значение…}` (после всех путей формы: FieldsMap полон).
    pub(super) fn list_settings(&self, attr: &str, st: &El) -> R<Node> {
        let mut manual = None;
        let mut dynamic_read = None;
        let mut main = None;
        let mut list_settings = None;
        let mut auto_save = true;
        let mut query = None;
        let mut parts = Vec::new();
        for c in st.elems() {
            match c.name.as_str() {
                n if crate::query::settings::ELEMENTS.contains(&n) => parts.push(c),
                "ManualQuery" => manual = Some(boolean(c)?),
                "QueryText" => query = Some(c.text()?),
                "DynamicDataRead" => dynamic_read = Some(boolean(c)?),
                "MainTable" => main = Some(c.text()?),
                "ListSettings" => list_settings = Some(c),
                // обратное выгрузке: пишет только false
                "AutoSaveUserSettings" => auto_save = boolean(c)?,
                o => return Err(err(c, &format!("{o} в настройках динамического списка пока не загружается"))),
            }
        }
        // свой запрос: текст как есть, поля корня — из текста (List::from_query); текст без своего запроса читатель
        // тоже хранит как есть, а в построитель он не идет (FUN_18067f990: текст +0x20 — только при своем запросе),
        // поля — по основной таблице (List::new)
        let query = match (manual, query) {
            (Some(false), None) => String::new(),
            (Some(_), Some(q)) => super::super::tpl::crlf(&q),
            _ => return Err(err(st, "сочетание ManualQuery и QueryText в настройках динамического списка не встречено")),
        };
        // mngbase FUN_1802b4660 ветвится по совместимости: KeyType/KeyFields при > 0x139b3, ReqMapFieldId… при < 0x139b7
        // иначе, AutoSaveUserSettings/Filter/Order при > 0x139ae, Group при < 0x139af, ServerState и DataParameters при
        // > 0x139be; словарь ниже — ветки > 0x139b6 (до 0x139be без двух последних ключей)
        let state_kept = match self.x.compat {
            Some(c) if c > 0x139be => true,
            Some(c) if c > 0x139b6 => false,
            _ => return Err(err(st, "настройки динамического списка при совместимости 8.3.10 и ниже (или без нее) не выведены")),
        };
        if !state_kept && parts.iter().any(|e| matches!(e.name.as_str(), "Field" | "CalculatedField" | "Parameter")) {
            return Err(err(st, "поля и параметры схемы динамического списка при совместимости 8.3.18 и ниже: ключа ServerState в записи нет"));
        }
        // без основной таблицы читатель ее не ставит (mngbase FUN_18073f980 только при тексте): пустая ссылка, вид 0
        // виртуальная таблица — суффикс имени и категория (обратно выгрузке, xml::form)
        let (gid, category) = match &main {
            Some(m) => {
                let (t, cat) = match m.rsplit_once('.').filter(|(t, _)| t.contains('.')) {
                    Some((t, v)) => match (t.split('.').next().unwrap_or_default(), v) {
                        ("AccountingRegister", "RecordsWithExtDimensions") | ("AccumulationRegister", "Balance") => (t, "3"),
                        ("AccountingRegister", "Balance") => (t, "4"),
                        ("Task", "TasksByExecutive") | ("InformationRegister", "SliceFirst") => (t, "2"),
                        // basic.dll FUN_1801db9f0: виртуальные таблицы регистра сведений 1 SliceFirst, 2 SliceLast (+1)
                        ("InformationRegister", "SliceLast") => (t, "3"),
                        _ => return Err(err(st, &format!("виртуальная основная таблица {m} не выведена"))),
                    },
                    None => (m.as_str(), "1"),
                };
                (self.x.id(t).ok_or_else(|| err(st, &format!("нет основной таблицы {t}")))?, cat)
            }
            None => (super::ZERO, "0"),
        };
        let parts = crate::query::settings::parts(&parts, super::xml_text, super::ROOT_NS)?;
        let state = super::super::dcs::server_state_part(&parts.state, self.x).map_err(|m| err(st, &m))?;
        let ls = list_settings.map(|ls| super::dlist_settings::list_settings(self.x, ls)).transpose()?;
        // ReqMapFieldId — `std::set<int>`: запись идет по возрастанию номера (mngbase FUN_1802b4660, обход дерева
        // FUN_180045c20), чтение вставляет в множество (FUN_1802b1a50)
        let mut ids = self.list_req.borrow().get(attr).cloned().unwrap_or_default();
        ids.sort_unstable();
        ids.dedup();
        let hash = |g: &str, v: Node| l(vec![s("#"), a(g), v]);
        let b = |v: bool| l(vec![s("B"), a(if v { "1" } else { "0" })]);
        let mut kv: Vec<(String, Node)> = vec![
            ("QueryText".into(), l(vec![s("S"), s(query)])),
            ("MainTable".into(), l(vec![s("#"), a(MAIN_TABLE), a(gid)])),
            ("MainTableCategory".into(), l(vec![s("N"), a(category)])),
            ("KeyType".into(), l(vec![s("N"), an(parts.key_type)])),
            ("KeyFields".into(), hash(KEY_FIELDS, l(std::iter::once(an(parts.key_fields.len())).chain(parts.key_fields.iter().map(|k| l(vec![s("S"), s(k.clone())]))).collect()))),
            ("DynamicalDataSelection".into(), b(dynamic_read == Some(false))),
            ("AutoFillAvailableFields".into(), b(parts.auto_fill)),
            ("ManualQuery".into(), b(manual == Some(true))),
        ];
        let lists = self.lists.borrow();
        let list = lists.get(attr).ok_or_else(|| err(st, "динамический список не подготовлен"))?;
        let mut map = list.map.clone();
        map.sort_by_key(|(i, _)| *i);
        // FieldsMapSecondaryNamesLoaded — байт FieldsMap +0x22 (в словарь его пишет FUN_1805f1d80) того объекта настроек,
        // который последним записан в реквизит (+0x4a0): читатель FUN_18073d190 — новый объект (конструктор FUN_1805f39f0:
        // 0); дерево полей (FUN_1802b16a0 → FUN_1802b1a50) — свой объект с «совместимость > 0x139b6», в реквизит его
        // пишут FUN_1802b8a10 после новой записи FieldsMap (FUN_1802b8ac0 возвращает реквизит только после +0x60; номер
        // 10000000 картинки строки и особые −1…−6 не регистрирует; `~X.DefaultPicture` FUN_1802c8a30 дает 10000000 без
        // +0x60) и задача «использовать всегда» (FUN_18074ff50 при непустом векторе путей → FUN_18073d720: +0x488, +0x4a0)
        let names_loaded = (!map.is_empty() || !ids.is_empty()) && self.x.compat.is_some_and(|c| c > 0x139b6);
        for (k, (id, f)) in map.iter().enumerate() {
            kv.push((format!("FieldsMapItemId{k}"), l(vec![s("N"), an(id)])));
            kv.push((format!("FieldsMapItemName{k}"), l(vec![s("S"), s(f.name.clone())])));
            if !f.second.is_empty() && f.second != f.name {
                kv.push((format!("FieldsMapItemSecondaryName{k}"), l(vec![s("S"), s(f.second.clone())])));
            }
            kv.push((format!("FiledsMapItemId{k}"), l(vec![s("N"), an(id)])));
            kv.push((format!("FiledsMapItemName{k}"), l(vec![s("S"), s(f.name.clone())])));
        }
        kv.push(("FieldsMapSecondaryNamesLoaded".into(), b(names_loaded)));
        for (k, id) in ids.iter().enumerate() {
            kv.push((format!("ReqMapFieldId{k}"), l(vec![s("N"), an(id)])));
        }
        kv.push(("AutoSaveUserSettings".into(), b(auto_save)));
        let (keys, params) = ls.map(|l| (l.keys, l.data_parameters)).unwrap_or_default();
        kv.extend(keys);
        kv.push(("GetInvisibleFieldPresentations".into(), b(parts.invisible)));
        if state_kept {
            kv.push(("ServerState".into(), l(vec![s("S"), s(server_state(&state)?)])));
            kv.push(("DataParameters".into(), hash(DATA_PARAMETERS, l(vec![super::super::payload::base64_atom(params.as_deref().unwrap_or(DATA_PARAMETERS_EMPTY).as_bytes())]))));
        } else if params.is_some() {
            return Err(err(st, "параметры данных динамического списка при совместимости 8.3.18 и ниже: ключа DataParameters в записи нет"));
        }
        let mut v = vec![a("0"), an(kv.len())];
        for (k, n) in kv {
            v.extend([s(k), n]);
        }
        Ok(l(v))
    }
}

#[cfg(test)]
const SERVER_STATE_EMPTY: &str = "\u{feff}<?xml version=\"1.0\" encoding=\"UTF-8\"?>\r\n<UniversalListServerOnlyState xmlns=\"\" xmlns:xs=\"http://www.w3.org/2001/XMLSchema\" xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\"/>";

const DATA_PARAMETERS_EMPTY: &str = "\u{feff}<?xml version=\"1.0\" encoding=\"UTF-8\"?>\r\n<DataParameterValues xmlns=\"http://v8.1c.ru/8.1/data-composition-system/settings\" xmlns:xs=\"http://www.w3.org/2001/XMLSchema\" xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\"/>";

/// Состояние сервера списка: `41 C1`, затем текст кусками (`9A <длина>` до 255 байт, `9B <длина u16>`), в конце два
/// пробела; строка — base64 строками по 64 знака. Куски — записи писателя XML (mngbase FUN_180702cc0: писатель
/// FUN_1802925d0 в поток FUN_1805464d0): каждый сброс буфера писателя пишется по 32 768 байт, у первого сброса первые
/// два байта BOM — отдельно. Буфер — 65 536 знаков UTF-16 (с BOM): вызов, который в него не помещается, идет после
/// сброса (ЭтапПроизводства2_2.ФормаСписка ERP: 124 674 байта — сброс на 65 514-м знаке перед текстом поля в 51 знак,
/// куски 2, 32 768, 32 768, 14 270 и 32 768, 12 098).
// shortcut: разбиение кусками снято с записей загрузки (1 842 + 211 состояний БП, все такие, и ERP); емкость 65 536 —
// внутри сверенного одной записью промежутка 65 515…65 565, а гранулярность вызовов писателя не прочитана: граница сброса
// принимается только перед текстовым узлом без сущностей (как в образце), иначе — отказ
fn server_state(xml: &str) -> R<String> {
    const BUF: usize = 65536;
    let b = xml.as_bytes();
    // начала сбросов (байтовые смещения): перед текстовым узлом, на котором буфер переполнился бы
    let mut flushes = vec![0usize];
    let (mut at, mut used) = (0usize, 0usize);
    for (i, c) in xml.char_indices() {
        let w = c.len_utf16();
        if used + w <= BUF {
            used += w;
            continue;
        }
        // текстовый узел, в котором лежит знак i: от последнего `>` до следующего `<`
        let open = xml[at..i].rfind(['<', '>']).map(|k| at + k);
        let text_from = match open {
            Some(k) if b[k] == b'>' => k + 1,
            _ => return Err(format!("ServerState списка {} байт: граница сброса буфера писателя внутри разметки — разбиение не выведено", xml.len())),
        };
        let text_to = xml[i..].find('<').map_or(xml.len(), |k| i + k);
        let text = &xml[text_from..text_to];
        if text_from <= at || text.contains('&') || text.encode_utf16().count() > BUF {
            return Err(format!("ServerState списка {} байт: сброс буфера писателя на тексте {} знаков — разбиение не выведено", xml.len(), text.chars().count()));
        }
        flushes.push(text_from);
        at = text_from;
        used = xml[text_from..i].encode_utf16().count() + w;
    }
    flushes.push(b.len());
    let mut out = vec![0x41, 0xc1];
    let mut push = |c: &[u8]| {
        if c.len() <= 255 {
            out.extend([0x9a, c.len() as u8]);
        } else {
            out.push(0x9b);
            out.extend((c.len() as u16).to_le_bytes());
        }
        out.extend_from_slice(c);
    };
    for (k, w) in flushes.windows(2).enumerate() {
        let mut seg = &b[w[0]..w[1]];
        if k == 0 {
            push(&seg[..2.min(seg.len())]);
            seg = &seg[2.min(seg.len())..];
        }
        for c in seg.chunks(32768) {
            push(c);
        }
    }
    out.extend(b"  ");
    let n = super::super::payload::base64_atom(&out);
    Ok(n.atom().unwrap_or_default().strip_prefix("#base64:").unwrap_or_default().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Длины кусков состояния сервера (обратное `server_state`).
    fn state_chunks(st: &str) -> Vec<usize> {
        let b = crate::xml::base64(st).unwrap();
        let (mut p, mut v) = (2, Vec::new());
        while p + 2 < b.len() {
            let n = if b[p] == 0x9a { p += 2; b[p - 1] as usize } else { p += 3; u16::from_le_bytes([b[p - 2], b[p - 1]]) as usize };
            v.push(n);
            p += n;
        }
        v
    }

    #[test]
    fn long_server_state_flushes_before_text() {
        // 1 + 3 + 65 520 + 4 + 3 = 65 531 знак; текст в 10 знаков уже не помещается в буфер 65 536 — сброс перед ним
        let xml = format!("\u{feff}<r>{}</r><b>{}</b>", "y".repeat(65520), "z".repeat(10));
        assert_eq!(state_chunks(&server_state(&xml).unwrap()), vec![2, 32768, 32763, 14]);
        // граница внутри разметки — отказ
        let xml = format!("\u{feff}<r>{}</r><bbbbbbbbbbbb>z</bbbbbbbbbbbb>", "y".repeat(65527));
        assert!(server_state(&xml).is_err());
        // короткое — как раньше
        assert_eq!(state_chunks(&server_state(SERVER_STATE_EMPTY).unwrap()), vec![2, SERVER_STATE_EMPTY.len() - 2]);
    }

    #[test]
    fn empty_server_state_as_platform() {
        // запись загрузки БП (ВидыНалоговыхОрганов.ФормаСписка)
        let want = "QcGaAu+7mrO/PD94bWwgdmVyc2lvbj0iMS4wIiBlbmNvZGluZz0iVVRGLTgiPz4N\r\r\nCjxVbml2ZXJzYWxMaXN0U2VydmVyT25seVN0YXRlIHhtbG5zPSIiIHhtbG5zOnhz\r\r\nPSJodHRwOi8vd3d3LnczLm9yZy8yMDAxL1hNTFNjaGVtYSIgeG1sbnM6eHNpPSJo\r\r\ndHRwOi8vd3d3LnczLm9yZy8yMDAxL1hNTFNjaGVtYS1pbnN0YW5jZSIvPiAg";
        assert_eq!(server_state(SERVER_STATE_EMPTY).unwrap(), want);
    }

    #[test]
    fn old_schema_root() {
        // совместимость 8.3.16: схема дерева полей без генератора — ролей нет, реквизит в режиме пароля остается
        let mut x = Index { compat: Some(80316), ..Default::default() };
        let mut o = super::super::super::index::Obj::default();
        o.kids = vec![("Attribute".into(), "Сумма".into()), ("Attribute".into(), "Пароль".into())];
        x.objects.insert("Document.Счет".into(), o);
        x.password.insert("пароль".into());
        let f = query_root_fields(&x, Some("Document.Счет"), "ВЫБРАТЬ Д.Ссылка, Д.Сумма, Д.Пароль ИЗ Документ.Счет КАК Д", &[]).unwrap().unwrap();
        assert_eq!(f.iter().map(|f| f.name.as_str()).collect::<Vec<_>>(), ["Ref", "Пароль", "Сумма"]);
    }

    #[test]
    fn query_root_with_roles() {
        // Документ: роли DeletionMark, Posted, Date, Ref; Ссылка выбрана с псевдонимом — не дописывается
        let mut x = Index::default();
        let mut o = super::super::super::index::Obj::default();
        o.kids = vec![("Attribute".into(), "Сумма".into())];
        x.objects.insert("Document.Счет".into(), o);
        let q = "ВЫБРАТЬ Д.Ссылка КАК Ссылка, Д.Дата, Д.Сумма, 1 КАК Дата1 ИЗ Документ.Счет КАК Д";
        let f = query_root_fields(&x, Some("Document.Счет"), q, &[]).unwrap().unwrap();
        let n: Vec<String> = f.iter().map(|f| format!("{}|{}", f.name, f.second)).collect();
        assert_eq!(n, ["Date|Дата", "DeletionMark|ПометкаУдаления", "Posted|Проведен", "Дата1|Дата1", "Ссылка|Ссылка", "Сумма|Сумма"]);
        // объявленное поле набора дает путь объявления
        let f = query_root_fields(&x, Some("Document.Счет"), q, &[("ДатаДок".into(), "Дата".into())]).unwrap().unwrap();
        assert!(f.iter().any(|f| f.name == "ДатаДок"));
        // занятое имя роли — с суффиксом
        let q = "ВЫБРАТЬ Д.Ссылка КАК Ссылка, 0 КАК ПометкаУдаления ИЗ Документ.Счет КАК Д";
        let f = query_root_fields(&x, Some("Document.Счет"), q, &[]).unwrap().unwrap();
        assert!(f.iter().any(|f| f.name == "ПометкаУдаления1"));
        // колонка-повтор константы при выбранном поле основной таблицы имя роли не занимает; без поля таблицы — занимает
        let q = "ВЫБРАТЬ Д.Ссылка КАК Ссылка, 0 КАК Сумма, 0 КАК ПометкаУдаления ИЗ Документ.Счет КАК Д";
        let f = query_root_fields(&x, Some("Document.Счет"), q, &[]).unwrap().unwrap();
        assert!(f.iter().any(|f| f.name == "DeletionMark") && !f.iter().any(|f| f.name == "ПометкаУдаления1"));
        let q = "ВЫБРАТЬ 0 КАК Сумма, 0 КАК ПометкаУдаления ИЗ Документ.Счет КАК Д";
        let f = query_root_fields(&x, Some("Document.Счет"), q, &[]).unwrap().unwrap();
        assert!(f.iter().any(|f| f.name == "ПометкаУдаления1"));
        // ПРЕДСТАВЛЕНИЕ выбранного поля в корне нет; невыбранного поля и агрегата — поле
        let q = "ВЫБРАТЬ Д.Сумма, ПРЕДСТАВЛЕНИЕ(Д.Сумма) КАК П, ПРЕДСТАВЛЕНИЕ(Д.Дата) КАК Н, ПРЕДСТАВЛЕНИЕ(МАКСИМУМ(Д.Сумма)) КАК М ИЗ Документ.Счет КАК Д";
        let f = query_root_fields(&x, None, q, &[]).unwrap().unwrap();
        assert!(!f.iter().any(|f| f.name == "П") && f.iter().any(|f| f.name == "Н") && f.iter().any(|f| f.name == "М"));
        // хранилище значения — поле корня; реквизит в режиме пароля — поля нет; реквизит с именем Представление — поле
        x.objects.get_mut("Document.Счет").unwrap().kids.extend([("Attribute".into(), "Файл".into()), ("Attribute".into(), "Представление".into())]);
        x.attr_types.insert("Document.Счет.Attribute.Файл".into(), vec!["v8:ValueStorage".into()]);
        let f = query_root_fields(&x, None, "ВЫБРАТЬ Д.Файл, Д.Представление КАК П ИЗ Документ.Счет КАК Д", &[]).unwrap().unwrap();
        assert_eq!(f.iter().map(|f| f.name.as_str()).collect::<Vec<_>>(), ["П", "Файл"]);
        x.password.insert("файл".into());
        let f = query_root_fields(&x, None, "ВЫБРАТЬ Д.Файл, Д.Сумма ИЗ Документ.Счет КАК Д", &[]).unwrap().unwrap();
        assert_eq!(f.iter().map(|f| f.name.as_str()).collect::<Vec<_>>(), ["Сумма"]);
        x.password.clear();
        // имя-дубль без псевдонима — с номером; с псевдонимом — отказ
        let f = query_root_fields(&x, None, "ВЫБРАТЬ Д.Сумма, Е.Сумма ИЗ Документ.Счет КАК Д, Документ.Счет КАК Е", &[]).unwrap().unwrap();
        assert_eq!(f.iter().map(|f| f.name.as_str()).collect::<Vec<_>>(), ["Сумма", "Сумма1"]);
        assert!(query_root_fields(&x, None, "ВЫБРАТЬ Д.Сумма, Е.Ссылка КАК Сумма ИЗ Документ.Счет КАК Д, Документ.Счет КАК Е", &[]).is_err());
        // происхождение полей корня: `Псевдоним.Поле` таблицы объекта, литерал — тип XML; прочее выражение — нет
        let o = query_origins(&x, "ВЫБРАТЬ Д.сумма КАК С, Д.Ссылка, 1 КАК Е, Д.Сумма + 1 КАК Ж ИЗ Документ.Счет КАК Д").unwrap();
        assert_eq!(o, [("С".into(), "Document.Счет".into(), "Сумма".into()), ("Ссылка".into(), "Document.Счет".into(), "Ref".into()), ("Е".into(), "".into(), "xs:decimal".into())]);
        // `Т.А.Б` без псевдонима — склейка частей
        let f = query_root_fields(&x, None, "ВЫБРАТЬ Д.Ссылка.Дата ИЗ Документ.Счет КАК Д", &[]).unwrap().unwrap();
        assert_eq!(f.iter().map(|f| format!("{}|{}", f.name, f.second)).collect::<Vec<_>>(), ["RefDate|СсылкаДата"]);
        // `{ВЫБРАТЬ Имя}` по полю выборки и `{ГДЕ …}` части ОБЪЕДИНИТЬ корень не меняют; `{ВЫБРАТЬ}` по полю таблицы — отказ
        let q = "ВЫБРАТЬ Д.Сумма КАК С {ВЫБРАТЬ С} ИЗ Документ.Счет КАК Д ОБЪЕДИНИТЬ ВСЕ ВЫБРАТЬ Д.Сумма ИЗ Документ.Счет КАК Д {ГДЕ Д.Дата КАК Х}";
        let f = query_root_fields(&x, None, q, &[]).unwrap().unwrap();
        assert_eq!(f.iter().map(|f| f.name.as_str()).collect::<Vec<_>>(), ["С"]);
        assert!(query_root_fields(&x, None, "ВЫБРАТЬ Д.Сумма КАК С {ВЫБРАТЬ Д.Дата} ИЗ Документ.Счет КАК Д", &[]).is_err());
        // без псевдонима: параметр — его имя, `Д.ТЧ.(…)` — имя табличной части по метаданным
        x.objects.get_mut("Document.Счет").unwrap().kids.push(("TabularSection".into(), "Товары".into()));
        let q = "ВЫБРАТЬ Д.Ссылка КАК Ссылка, &Основной, Д.товары.* ИЗ Документ.Счет КАК Д";
        let f = query_root_fields(&x, None, q, &[]).unwrap().unwrap();
        assert!(f.iter().any(|f| f.name == "Товары"));
        let q = "ВЫБРАТЬ Д.Ссылка КАК Ссылка, &Основной, Д.товары.(НомерСтроки) ИЗ Документ.Счет КАК Д";
        let f = query_root_fields(&x, None, q, &[]).unwrap().unwrap();
        let n: Vec<String> = f.iter().map(|f| format!("{}|{}", f.name, f.second)).collect();
        assert_eq!(n, ["Основной|Основной", "Ссылка|Ссылка", "Товары|Товары"]);
        // ОБЪЕДИНИТЬ без основной таблицы: имена первой части; хранилище/представление в другой части — отказ
        let q = "ВЫБРАТЬ Д.Ссылка КАК Ссылка, 1 КАК Н ИЗ Документ.Счет КАК Д ОБЪЕДИНИТЬ ВСЕ ВЫБРАТЬ Д.Ссылка, 2 ИЗ Документ.Счет КАК Д";
        let f = query_root_fields(&x, None, q, &[]).unwrap().unwrap();
        assert_eq!(f.iter().map(|f| f.name.as_str()).collect::<Vec<_>>(), ["Н", "Ссылка"]);
        // с основной таблицей: роль не в одной колонке каждой части — отказ; все роли в одних колонках — корень первой части
        assert!(query_root_fields(&x, Some("Document.Счет"), q, &[]).is_err());
        let q2 = "ВЫБРАТЬ Д.Ссылка КАК Ссылка, Д.Дата КАК Дата, Д.ПометкаУдаления КАК ПометкаУдаления, Д.Проведен КАК Проведен ИЗ Документ.Счет КАК Д ОБЪЕДИНИТЬ ВСЕ ВЫБРАТЬ Д.Ссылка, Д.Дата, Д.ПометкаУдаления, Д.Проведен ИЗ Документ.Счет КАК Д";
        let f = query_root_fields(&x, Some("Document.Счет"), q2, &[]).unwrap().unwrap();
        assert_eq!(f.iter().map(|f| f.name.as_str()).collect::<Vec<_>>(), ["Дата", "ПометкаУдаления", "Проведен", "Ссылка"]);
        // представление невыбранного поля в другой части — колонка как есть (АвансовыйОтчет ERP); выбранного — отказ
        let f = query_root_fields(&x, None, "ВЫБРАТЬ Д.Сумма КАК С ИЗ Документ.Счет КАК Д ОБЪЕДИНИТЬ ВЫБРАТЬ ПРЕДСТАВЛЕНИЕ(Д.Ссылка) ИЗ Документ.Счет КАК Д", &[]).unwrap().unwrap();
        assert_eq!(f.iter().map(|f| f.name.as_str()).collect::<Vec<_>>(), ["С"]);
        assert!(query_root_fields(&x, None, "ВЫБРАТЬ Д.Ссылка КАК С, 1 КАК Н ИЗ Документ.Счет КАК Д ОБЪЕДИНИТЬ ВЫБРАТЬ Д.Ссылка, ПРЕДСТАВЛЕНИЕ(Д.Ссылка) ИЗ Документ.Счет КАК Д", &[]).is_err());
        // аргумент выбран в другой части — отказ: по полям какой части решает dcs, не прослежено
        assert!(query_root_fields(&x, None, "ВЫБРАТЬ Д.Ссылка КАК С ИЗ Документ.Счет КАК Д ОБЪЕДИНИТЬ ВЫБРАТЬ ПРЕДСТАВЛЕНИЕ(Д.Ссылка) ИЗ Документ.Счет КАК Д", &[]).is_err());
        // колонка роли в {ВЫБРАТЬ}: выбрана, только если в {ГДЕ} элемент с ее именем и тем же выражением; вне {ВЫБРАТЬ} —
        // выбрана; {ГДЕ Т.*} при колонке в {ВЫБРАТЬ} — отказ
        let q = "ВЫБРАТЬ Д.Ссылка КАК Ссылка, Д.Дата КАК Дата, Д.ПометкаУдаления КАК ПометкаУдаления, Д.Проведен {ВЫБРАТЬ Ссылка.*, Дата} ИЗ Документ.Счет КАК Д {ГДЕ (Д.Сумма > 0), Д.Ссылка.* КАК Ссылка}";
        let f = query_root_fields(&x, Some("Document.Счет"), q, &[]).unwrap().unwrap();
        assert_eq!(f.iter().map(|f| f.name.as_str()).collect::<Vec<_>>(), ["Posted", "Дата", "Дата1", "ПометкаУдаления", "Ссылка"]);
        assert!(query_root_fields(&x, Some("Document.Счет"), &q.replace("(Д.Сумма > 0), Д.Ссылка.* КАК Ссылка", "Д.*"), &[]).is_err());
        // вне {ВЫБРАТЬ}: одноименный элемент {ГДЕ}-выражение — поле роли выбрано; поле другого пути — отказ
        let q = "ВЫБРАТЬ Д.Ссылка КАК Ссылка, Д.Дата КАК Дата, Д.ПометкаУдаления КАК ПометкаУдаления, Д.Проведен КАК Проведен ИЗ Документ.Счет КАК Д {ГДЕ (ВЫРАЗИТЬ(Д.Ссылка КАК Документ.Счет)).* КАК Ссылка}";
        assert_eq!(query_root_fields(&x, Some("Document.Счет"), q, &[]).unwrap().unwrap().len(), 4);
        assert!(query_root_fields(&x, Some("Document.Счет"), &q.replace("(ВЫРАЗИТЬ(Д.Ссылка КАК Документ.Счет)).*", "Д.Дата"), &[]).is_err());
        // без автозаполнения колонка роли вне {ВЫБРАТЬ} — отказ; `Т.*` рядом с явным элементом — отказ; две колонки
        // роли в {ВЫБРАТЬ} с разным флагом по {ГДЕ} — отказ
        let q = "ВЫБРАТЬ Д.Ссылка КАК Ссылка, Д.Дата КАК Дата, Д.ПометкаУдаления КАК ПометкаУдаления, Д.Проведен КАК Проведен {ВЫБРАТЬ Дата} ИЗ Документ.Счет КАК Д";
        assert!(query_root(&x, Some("Document.Счет"), q, &[], true).is_ok() && query_root(&x, Some("Document.Счет"), q, &[], false).is_err());
        let q = "ВЫБРАТЬ Д.Ссылка КАК Ссылка, Д.Дата КАК Дата, Д.ПометкаУдаления КАК ПометкаУдаления, Д.Проведен {ВЫБРАТЬ Ссылка.*} ИЗ Документ.Счет КАК Д {ГДЕ Д.Ссылка.* КАК Ссылка, Д.*}";
        assert!(query_root_fields(&x, Some("Document.Счет"), q, &[]).is_err());
        let q = "ВЫБРАТЬ Д.Ссылка КАК Ссылка, Д.Ссылка КАК Док, Д.Дата КАК Дата, Д.ПометкаУдаления КАК ПометкаУдаления, Д.Проведен {ВЫБРАТЬ Ссылка.*, Док.*} ИЗ Документ.Счет КАК Д {ГДЕ Д.Ссылка.* КАК Ссылка}";
        assert!(query_root_fields(&x, Some("Document.Счет"), q, &[]).is_err_and(|e| e.contains("разным флагом")));
        // выражения без псевдонима: «Field»/«Поле» + номер по порядку, агрегат — имя аргумента (без основной таблицы)
        let q = "ВЫБРАТЬ Д.Ссылка КАК Ссылка, 1, МАКСИМУМ(Д.Сумма), ВЫБОР КОГДА ИСТИНА ТОГДА 1 КОНЕЦ ИЗ Документ.Счет КАК Д";
        let f = query_root_fields(&x, None, q, &[]).unwrap().unwrap();
        let n: Vec<String> = f.iter().map(|f| format!("{}|{}", f.name, f.second)).collect();
        assert_eq!(n, ["Field1|Поле1", "Field2|Поле2", "Ссылка|Ссылка", "Сумма|Сумма"]);
        assert!(query_root_fields(&x, Some("Document.Счет"), q, &[]).is_err());
        // таблицы нет в конфигурации — корня нет; поля нет у документа (состав выведен кодом, roles::table_fields) —
        // корня нет; у справочника (состав не выведен) — отказ
        assert!(query_root_fields(&x, None, "ВЫБРАТЬ Р.Поле КАК П ИЗ РегистрСведений.Нет КАК Р", &[]).is_ok_and(|r| r.is_none()));
        assert!(query_root_fields(&x, None, "ВЫБРАТЬ Д.Нет КАК П ИЗ Документ.Счет КАК Д", &[]).is_ok_and(|r| r.is_none()));
        // АВТОУПОРЯДОЧИВАНИЕ — не псевдоним: `Д` — не источник и не поле документа — запрос не компилируется
        assert!(query_root_fields(&x, Some("Document.Счет"), "ВЫБРАТЬ Д.Ссылка КАК Ссылка ИЗ Документ.счет АВТОУПОРЯДОЧИВАНИЕ", &[]).is_ok_and(|r| r.is_none()));
        x.objects.insert("Catalog.Банки".into(), Default::default());
        assert!(query_root_fields(&x, None, "ВЫБРАТЬ С.Нет КАК П ИЗ Справочник.Банки КАК С", &[]).is_err());
        // `ВЫБРАТЬ *` — поля документа без Presentation и PointInTime, роли выбраны; поля единственного источника без
        // псевдонима — как `Источник.Поле`, невыбранные роли дописаны
        let f = query_root_fields(&x, Some("Document.Счет"), "ВЫБРАТЬ * ИЗ Документ.Счет КАК Д", &[]).unwrap().unwrap();
        let n: Vec<String> = f.iter().map(|f| format!("{}|{}", f.name, f.second)).collect();
        assert_eq!(n, ["DataVersion|ВерсияДанных", "Date|Дата", "DeletionMark|ПометкаУдаления", "Posted|Проведен", "Ref|Ссылка", "Представление|Представление", "Сумма|Сумма", "Товары|Товары", "Файл|Файл"]);
        let f = query_root_fields(&x, Some("Document.Счет"), "ВЫБРАТЬ Ссылка, ПометкаУдаления, Сумма ИЗ Документ.Счет", &[]).unwrap().unwrap();
        let n: Vec<String> = f.iter().map(|f| format!("{}|{}", f.name, f.second)).collect();
        assert_eq!(n, ["Date|Дата", "DeletionMark|ПометкаУдаления", "Posted|Проведен", "Ref|Ссылка", "Сумма|Сумма"]);
        // `Представление` источника без псевдонима — как `Источник.Представление`: реквизит — поле корня; стандартное —
        // поле, только пока Ссылка не выбрана
        let mut o = super::super::super::index::Obj::default();
        o.props = [("InformationRegisterPeriodicity", "Month"), ("WriteMode", "Independent")].iter().map(|(k, v)| (k.to_string(), v.to_string())).collect();
        o.kids = vec![("Dimension".into(), "Вид".into()), ("Resource".into(), "Представление".into())];
        x.objects.insert("InformationRegister.Р".into(), o);
        let f = query_root_fields(&x, Some("InformationRegister.Р"), "ВЫБРАТЬ Период, Вид, Представление ИЗ РегистрСведений.Р", &[]).unwrap().unwrap();
        assert!(f.iter().any(|f| f.name == "Представление"));
        let mut o = super::super::super::index::Obj::default();
        o.kids = vec![("Attribute".into(), "Сумма".into())];
        x.objects.insert("Document.Акт".into(), o);
        let f = query_root_fields(&x, None, "ВЫБРАТЬ Сумма, Представление ИЗ Документ.Акт", &[]).unwrap().unwrap();
        assert_eq!(f.len(), 2);
        let f = query_root_fields(&x, None, "ВЫБРАТЬ Ссылка, Представление ИЗ Документ.Акт", &[]).unwrap().unwrap();
        assert_eq!(f.len(), 1);
        // `{ВЫБРАТЬ Имя.* КАК Имя}` — как `{ВЫБРАТЬ Имя.*}`; другой псевдоним — отказ
        let f = query_root_fields(&x, None, "ВЫБРАТЬ Д.Ссылка КАК С {ВЫБРАТЬ С.* КАК С} ИЗ Документ.Счет КАК Д", &[]).unwrap().unwrap();
        assert_eq!(f.iter().map(|f| f.name.as_str()).collect::<Vec<_>>(), ["С"]);
        assert!(query_root_fields(&x, None, "ВЫБРАТЬ Д.Ссылка КАК С {ВЫБРАТЬ С.* КАК Х} ИЗ Документ.Счет КАК Д", &[]).is_err());
        // отказы: поле роли без псевдонима таблицы, два поля с одним именем, ПОМЕСТИТЬ в последнем запросе
        for q in [
            "ВЫБРАТЬ Ссылка КАК Ссылка ИЗ Документ.Счет КАК Д",
            "ВЫБРАТЬ Д.Ссылка, Д.Сумма КАК Ссылка ИЗ Документ.Счет КАК Д",
            "ВЫБРАТЬ Д.Ссылка КАК Ссылка ПОМЕСТИТЬ ВТ ИЗ Документ.Счет КАК Д",
        ] {
            assert!(query_root_fields(&x, Some("Document.Счет"), q, &[]).is_err(), "{q}");
        }
        // табличная часть-источник: путь через ее Ссылку — без имени Ссылки
        let f = query_root_fields(&x, None, "ВЫБРАТЬ Т.Ссылка.Дата ИЗ Документ.Счет.Товары КАК Т", &[]).unwrap().unwrap();
        assert_eq!(f.iter().map(|f| format!("{}|{}", f.name, f.second)).collect::<Vec<_>>(), ["Date|Дата"]);
        // поле объекта с именем стандартного поля другого класса — поле объекта
        let mut r = super::super::super::index::Obj::default();
        r.kids = vec![("Dimension".into(), "ВидДвижения".into())];
        x.objects.insert("InformationRegister.Р".into(), r);
        let o = query_origins(&x, "ВЫБРАТЬ Р.ВидДвижения ИЗ РегистрСведений.Р КАК Р").unwrap();
        assert_eq!(o, [("ВидДвижения".into(), "InformationRegister.Р".into(), "ВидДвижения".into())]);
    }

    #[test]
    fn rebuild_numbering() {
        // СведенияОДоходахНДФЛ: Period 3, затем ДатаПолученияДохода 10 — как в записи загрузки
        let f = |n: &str| Field { name: n.into(), second: String::new() };
        let mut l = List { main: "x".into(), fields: ["Active", "LineNumber", "Period", "PointInTime", "Recorder", "Включать", "Головная", "ДатаПолученияДохода"].map(f).to_vec(), map: Vec::new(), custom: false, no_root: false, dp: None, ca: Vec::new(), expanded: false, origins: Vec::new(), roles: Vec::new(), types: Vec::new(), kinds: Vec::new(), grouped: false };
        assert_eq!(l.id("Period"), Ok(3));
        assert_eq!(l.id("ДатаПолученияДохода"), Ok(10));
        assert_eq!(l.id("Period"), Ok(3));
        assert_eq!(l.id("Active"), Ok(11));
        // недоступное поле — счетчик за полями корня: max 11 + 1 + 5 полей вне FieldsMap
        assert_eq!(l.unavailable_id("SWIFT", None), Ok(17));
        assert_eq!(l.unavailable_id("SWIFT", None), Ok(17));
        assert_eq!(l.unavailable_id("Period", None), Ok(3));
        assert_eq!(l.id("Recorder"), Ok(20));
        // обычный путь не находит поле вне корня, записанное путем `~`
        assert!(l.id("SWIFT").is_err());
        // запасной путь: второе имя; поиск по второму имени; совпадение только без учета регистра — отказ
        assert_eq!(l.unavailable_id("Number", Some("Номер")), Ok(25));
        assert_eq!(l.unavailable_id("Номер", None), Ok(25));
        assert_eq!(l.unavailable_id("Ref", Some("Номер")), Ok(25));
        assert!(l.unavailable_id("swift", None).is_err());
    }

    #[test]
    fn nocase_as_compare_nocase() {
        use std::cmp::Ordering::*;
        assert_eq!(nocase("ДатаА", "датаа"), Equal);
        assert_eq!(nocase("Ёлка", "ёлка"), Equal);
        // '_' (0x5f) меньше строчной буквы, но больше прописной до свертки — сравниваются строчные
        assert_eq!(nocase("A_B", "AaB"), Less);
        assert_eq!(nocase("е", "ё"), Less);
        assert_eq!(nocase("Код", "КодX"), Less);
        assert!(nocase_ok("Ref_Код1") && !nocase_ok("Ref\u{3b1}"));
    }

    #[test]
    fn role_under_aggregate() {
        let b = crate::query::parse("ВЫБРАТЬ Т.Код, КОЛИЧЕСТВО(Т.Код) - СУММА(ВЫБОР КОГДА Т.Имя = 1 ТОГДА 1 КОНЕЦ) КАК К ИЗ Справочник.Банки КАК Т").unwrap();
        let fs = &b.queries[0].parts[0].fields;
        let hit = |n: &'static str| move |p: &[String]| p.len() == 2 && p[0] == "Т" && p[1] == n;
        assert!(!under_aggregate(&fs[0].expr, &hit("Код"), false));
        assert!(under_aggregate(&fs[1].expr, &hit("Код"), false));
        assert!(under_aggregate(&fs[1].expr, &hit("Имя"), false));
        assert!(!under_aggregate(&fs[1].expr, &hit("Наименование"), false));
    }

    #[test]
    fn dcs_unlisted_fields() {
        let q = "ВЫБРАТЬ Ж.Дата КАК Дата, Ж.Ссылка, Ж.Организация КАК Организация {ВЫБРАТЬ Дата, Ссылка.*} ИЗ ЖурналДокументов.Ж КАК Ж";
        assert_eq!(dcs_unlisted(q), Some(vec!["Организация".to_string()]));
        assert_eq!(dcs_unlisted("ВЫБРАТЬ Ж.Дата ИЗ ЖурналДокументов.Ж КАК Ж"), None);
        // ОБЪЕДИНИТЬ: поля и `{ВЫБРАТЬ}` первой части; `{ВЫБРАТЬ}` в другой части — не выведено
        let u = "ВЫБРАТЬ Ж.Дата КАК Дата, Ж.Номер КАК Номер {ВЫБРАТЬ Дата} ИЗ ЖурналДокументов.Ж КАК Ж ОБЪЕДИНИТЬ ВСЕ ВЫБРАТЬ Ж.Дата, Ж.Номер ИЗ ЖурналДокументов.Ж КАК Ж";
        assert_eq!(dcs_unlisted(u), Some(vec!["Номер".to_string()]));
        assert_eq!(dcs_unlisted(&format!("{u} {{ВЫБРАТЬ Номер}}")), None);
    }

    #[test]
    fn kid_kinds_of_ts() {
        let mut x = Index::default();
        x.attr_types.insert("Document.Д.TabularSection.Т.Attribute.Статья".into(), vec!["cfg:CatalogRef.С".into()]);
        let ts = Kind::Ts("Document.Д.TabularSection.Т".into());
        let types = |k: Kind| match k {
            Kind::Types(t) => t,
            Kind::Ts(_) | Kind::Params(_) => vec!["ts".into()],
        };
        assert_eq!(types(kid_kind(&x, &ts, "Ref").unwrap()), ["cfg:DocumentRef.Д"]);
        assert!(types(kid_kind(&x, &ts, "LineNumber").unwrap()).is_empty());
        assert_eq!(types(kid_kind(&x, &ts, "Статья").unwrap()), ["cfg:CatalogRef.С"]);
    }

    #[test]
    fn kinds_of_recorder_journal_calc_params() {
        use crate::query::resolve::{Meta, Ty};
        let types = |k: R<Kind>| match k.unwrap() {
            Kind::Types(t) => t,
            Kind::Ts(_) | Kind::Params(_) => vec!["?".into()],
        };
        let obj = |kids: &[(&str, &str)], props: &[(&str, &str)]| {
            let mut o = super::super::super::index::Obj::default();
            o.kids = kids.iter().map(|(k, n)| (k.to_string(), n.to_string())).collect();
            o.props = props.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect();
            o
        };
        let mut x = Index::default();
        x.objects.insert("Document.Д".into(), obj(&[("Attribute", "Склад")], &[("RegisterRecords", "AccumulationRegister.Р")]));
        x.objects.insert("Document.Е".into(), obj(&[], &[]));
        x.objects.insert("AccumulationRegister.Р".into(), obj(&[], &[("RegisterType", "Turnovers")]));
        x.attr_types.insert("Document.Д.Attribute.Склад".into(), vec!["cfg:CatalogRef.С".into()]);
        // регистратор — ссылки документов, в движениях которых есть регистр; поля пути нет ни у одного — не компилируется
        assert_eq!(IndexMeta(&x).field("AccumulationRegister.Р", "Регистратор").unwrap(), Some(vec![Ty::Ref("Document.Д".into())]));
        assert!(query_root_fields(&x, None, "ВЫБРАТЬ Т.Регистратор.Склад КАК С ИЗ РегистрНакопления.Р КАК Т", &[]).unwrap().is_some());
        assert!(query_root_fields(&x, None, "ВЫБРАТЬ Т.Регистратор.Нет КАК Н ИЗ РегистрНакопления.Р КАК Т", &[]).unwrap().is_none());
        // журнал: Ref — ссылки регистрируемых документов, Type — лист, графа — типы реквизитов, на которые ссылается
        x.objects.insert("DocumentJournal.Ж".into(), obj(&[("Column", "Склад")], &[("RegisteredDocuments", "Document.Д")]));
        x.column_refs.insert("DocumentJournal.Ж.Column.Склад".into(), vec!["Document.Д.Attribute.Склад".into()]);
        assert_eq!(types(root_kind(&x, "DocumentJournal.Ж", "Ref")), ["cfg:DocumentRef.Д"]);
        assert!(types(root_kind(&x, "DocumentJournal.Ж", "Type")).is_empty());
        assert_eq!(types(root_kind(&x, "DocumentJournal.Ж", "Склад")), ["cfg:CatalogRef.С"]);
        // набор типов характеристики — типы значения плана видов характеристик
        x.attr_types.insert("ChartOfCharacteristicTypes.Х".into(), vec!["cfg:CatalogRef.С".into()]);
        x.attr_types.insert("Document.Е.Attribute.З".into(), Vec::new());
        x.type_sets.insert("Document.Е.Attribute.З".into(), vec!["cfg:Characteristic.Х".into()]);
        assert_eq!(attr_types(&x, "Document.Е.Attribute.З").unwrap(), ["cfg:CatalogRef.С"]);
        // тип вычисляемого поля: TypeId определяемого типа — его типы; без valueType — не выведен
        x.types.insert("DefinedType.О".into(), ("48fa9d68".into(), "0".into()));
        x.attr_types.insert("DefinedType.О".into(), vec!["xs:decimal".into()]);
        let c = crate::xml::imp::dom::parse("<CalculatedField><dcssch:valueType><v8:TypeId>48fa9d68</v8:TypeId><v8:NumberQualifiers/></dcssch:valueType></CalculatedField>").unwrap();
        assert_eq!(types(calc_kind(&x, &c)), ["xs:decimal"]);
        assert!(calc_kind(&x, &crate::xml::imp::dom::parse("<CalculatedField/>").unwrap()).is_err());
        // параметр без типа — выражение без типов, только целым выражением поля выборки или веткой ТОГДА/ИНАЧЕ
        x.objects.insert("Catalog.С".into(), obj(&[], &[]));
        let q = "ВЫБРАТЬ &П КАК П, ВЫБОР КОГДА Т.Ссылка = &Х ТОГДА &п ИНАЧЕ НЕОПРЕДЕЛЕНО КОНЕЦ КАК Ветка ИЗ Справочник.С КАК Т";
        let b = crate::query::parse(q).unwrap();
        assert!(whole_only(q, &b, "П") && !whole_only(q, &b, "Х"));
        let w = format!("{q} ГДЕ Т.Ссылка В (&П)");
        assert!(!whole_only(&w, &crate::query::parse(&w).unwrap(), "П"));
        assert_eq!(query_types(&x, q, &["П".into()])[0], ("П".into(), Some(Vec::new())));
        assert_eq!(query_types(&x, q, &[])[0], ("П".into(), None));
    }

    #[test]
    fn records_ext_changes_calc_ts() {
        use crate::query::resolve::{Meta, Ty};
        let obj = |kids: &[(&str, &str)], props: &[(&str, &str)]| {
            let mut o = super::super::super::index::Obj::default();
            o.kids = kids.iter().map(|(k, n)| (k.to_string(), n.to_string())).collect();
            o.props = props.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect();
            o
        };
        let mut x = Index::default();
        x.objects.insert("ChartOfAccounts.П".into(), obj(&[], &[("MaxExtDimensionCount", "1")]));
        // без корреспонденции: Account, ExtDimension1, RecordType; уточнения периода нет при длине 0
        x.objects.insert("AccountingRegister.Б".into(), obj(&[("Dimension", "Орг"), ("Resource", "Сумма")], &[("Correspondence", "false"), ("ChartOfAccounts", "ChartOfAccounts.П"), ("PeriodAdjustmentLength", "0")]));
        let names = |x: &Index, m: &str| root_fields(x, m).unwrap().into_iter().map(|f| format!("{}|{}", f.name, f.second)).collect::<Vec<_>>();
        assert_eq!(names(&x, "AccountingRegister.Б.RecordsWithExtDimensions"), ["Account|Счет", "Active|Активность", "ExtDimension1|Субконто1", "ExtDimensionType1|ВидСубконто1", "LineNumber|НомерСтроки", "Period|Период", "PointInTime|МоментВремени", "Recorder|Регистратор", "RecordType|ВидДвижения", "Орг|Орг", "Сумма|Сумма"]);
        // с корреспонденцией: небалансовое измерение — парой Dr/Cr
        x.objects.insert("AccountingRegister.К".into(), obj(&[("Dimension", "Орг")], &[("Correspondence", "true"), ("ChartOfAccounts", "ChartOfAccounts.П"), ("PeriodAdjustmentLength", "1")]));
        x.unbalanced.insert("AccountingRegister.К.Dimension.Орг".into());
        let k = names(&x, "AccountingRegister.К.RecordsWithExtDimensions");
        assert!(k.contains(&"ОргDr|ОргДт".to_string()) && k.contains(&"PeriodAdjustment|УточнениеПериода".to_string()) && !k.iter().any(|n| n.starts_with("RecordType")));
        // реквизит в режиме пароля и общий реквизит у регистра — состав корня не выведен, отказ
        x.password_full.insert("AccountingRegister.Б.Dimension.Орг".into());
        assert!(root_fields(&x, "AccountingRegister.Б.RecordsWithExtDimensions").is_err());
        x.password_full.clear();
        x.common.push(("Общий".into(), "Use".into(), Default::default()));
        x.objects.insert("CommonAttribute.Общий".into(), obj(&[], &[("DataSeparation", "DontUse")]));
        assert!(root_fields(&x, "AccountingRegister.Б.RecordsWithExtDimensions").is_err());
        x.common.clear();
        // таблица изменений: Узел — планы обмена с регистром в составе, измерения основного отбора
        x.objects.insert("InformationRegister.Р".into(), obj(&[("Dimension", "А"), ("Dimension", "Б")], &[("WriteMode", "Independent"), ("InformationRegisterPeriodicity", "Nonperiodical")]));
        x.main_filter.insert("InformationRegister.Р.Dimension.А".into());
        x.attr_types.insert("InformationRegister.Р.Dimension.А".into(), vec!["xs:string".into()]);
        assert!(IndexMeta(&x).vt_field("InformationRegister.Р", "Изменения", "").is_err());
        x.objects.insert("ExchangePlan.О".into(), obj(&[], &[("Content", "Catalog.Х\nInformationRegister.Р")]));
        let m = IndexMeta(&x);
        assert_eq!(m.vt_field("InformationRegister.Р", "Изменения", "Узел").unwrap(), Some(vec![Ty::Ref("ExchangePlan.О".into())]));
        assert_eq!(m.vt_field("InformationRegister.Р", "Изменения", "А").unwrap(), Some(vec![Ty::Prim]));
        // измерение вне основного отбора — отказ (сопоставление слота не прочитано), не поле — «поля нет»
        assert!(m.vt_field("InformationRegister.Р", "Изменения", "Б").is_err());
        assert_eq!(m.vt_field("InformationRegister.Р", "Изменения", "Нет").unwrap(), None);
        // стандартная табличная часть плана видов расчета — только при зависимости от базовых
        x.compat = Some(0x139be);
        x.objects.insert("ChartOfCalculationTypes.Н".into(), obj(&[], &[("ActionPeriodUse", "false"), ("DependenceOnCalculationTypes", "DontUse")]));
        let m = IndexMeta(&x);
        assert!(m.ts("ChartOfCalculationTypes.Н", "ВедущиеВидыРасчета") && !m.ts("ChartOfCalculationTypes.Н", "БазовыеВидыРасчета"));
        assert_eq!(m.ts_field("ChartOfCalculationTypes.Н", "ВедущиеВидыРасчета", "НомерСтроки").unwrap(), Some(vec![Ty::Prim]));
        assert_eq!(m.ts_field("ChartOfCalculationTypes.Н", "LeadingCalculationTypes", "Нет").unwrap(), None);
        // вложенное поле стандартной табличной части в своем запросе — пара имен стандартного поля
        let q = "ВЫБРАТЬ Т.Ссылка, Т.ВедущиеВидыРасчета.(Ссылка, НомерСтроки, ВидРасчета) ИЗ ПланВидовРасчета.Н КАК Т";
        let f = query_root_fields(&x, None, q, &[]).unwrap().unwrap();
        assert!(f.iter().any(|f| f.name == "LeadingCalculationTypes" && f.second == "ВедущиеВидыРасчета"));
        assert!(query_root_fields(&x, None, "ВЫБРАТЬ Т.БазовыеВидыРасчета.(Ссылка) ИЗ ПланВидовРасчета.Н КАК Т", &[]).is_err());
        // раскрытие ссылки на план внутри табличной части: стандартная табличная часть отсекается, как свои
        let kids = |in_ts: bool| children(&x, &Kind::Types(vec!["cfg:ChartOfCalculationTypesRef.Н".into()]), in_ts).unwrap().into_iter().any(|f| f.name == "LeadingCalculationTypes");
        assert!(kids(false) && !kids(true));
    }

    #[test]
    fn filter_criterion_in_query() {
        let obj = |kids: &[(&str, &str)], props: &[(&str, &str)]| {
            let mut o = super::super::super::index::Obj::default();
            o.kids = kids.iter().map(|(k, n)| (k.to_string(), n.to_string())).collect();
            o.props = props.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect();
            o
        };
        let mut x = Index::default();
        x.objects.insert("FilterCriterion.К".into(), obj(&[], &[("Content", "Task.З.Attribute.Э\nDocument.Д.Attribute.Э")]));
        x.objects.insert("Task.З".into(), obj(&[("Attribute", "Э"), ("Attribute", "Исп")], &[]));
        x.objects.insert("Document.Д".into(), obj(&[("Attribute", "Э")], &[]));
        // Ссылка критерия — ссылки объектов состава
        assert!(matches!(root_kind(&x, "FilterCriterion.К", "Ref").unwrap(), Kind::Types(t) if t == ["cfg:DocumentRef.Д", "cfg:TaskRef.З"]));
        // источник `КритерийОтбора.Имя` — основная таблица; Ссылка выбрана — роль Ref есть, не выбрана — отказ
        let q = "ВЫБРАТЬ К.Ссылка КАК Ссылка, З.Исп ИЗ КритерийОтбора.К КАК К ЛЕВОЕ СОЕДИНЕНИЕ Задача.З КАК З ПО К.Ссылка = З.Ссылка";
        let f = query_root_fields(&x, Some("FilterCriterion.К"), q, &[]).unwrap().unwrap();
        assert_eq!(f.iter().map(|f| f.name.as_str()).collect::<Vec<_>>(), ["Исп", "Ссылка"]);
        let q = "ВЫБРАТЬ З.Исп ИЗ КритерийОтбора.К КАК К ЛЕВОЕ СОЕДИНЕНИЕ Задача.З КАК З ПО К.Ссылка = З.Ссылка";
        assert!(query_root_fields(&x, Some("FilterCriterion.К"), q, &[]).is_err_and(|e| e.contains("не выбрано")));
    }
}
