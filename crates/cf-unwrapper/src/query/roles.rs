//! Роли полей основной таблицы, которые генератор запроса списка дописывает в свой запрос (mngbase ULQueryGenerator):
//! FUN_18066d920 регистрирует роли полей таблицы FUN_180670870, затем FUN_180679bd0 — ключ (роль 2). Каждое поле роли
//! называет FUN_180666300: поле, уже выбранное или уже дописанное для прежней роли, — прежнее имя (трасса БП: журнал,
//! роль 0x40 «Ссылка1», ключ «Ссылка1»), поэтому повтор поля здесь отбрасывается.
//!
//! Порядок FUN_18066d920: 4 — DeletionMark части особых полей (`ede4bd22` +0x40 = 7-й аргумент слота +0x20
//! построителя, backend FUN_1803e8430 → FUN_1803e9c00, член +0x68); 0x10 — поле −7 у документа (`061d872a`); у журнала
//! (`4612bd75`) 4 — поле −4, 0x10 — поле −7; 0x20 — регистратор части `3edb7181` (+0x20 = второй аргумент слота +0x30,
//! backend FUN_1803e8860); 0x40 — ведущие поля (`ae0f5f41` +0x18 = вектор слота +0x40, FUN_1803e8b70); 0x80/0x100 —
//! иерархия (FUN_18066e130); 8 — генератор `ee411bba` +0x58 (mngbase FUN_180706ab0): первое поле таблицы с флагом
//! 0x400. Флаг 0x400 ставят построители полей: basic FUN_1801d6ff0 — Date документа (−3) и журнала (−100), basic
//! FUN_1801dd120 — Period регистра сведений (−2, флаги 0x401/0xc01 при периодичности ≠ 0), basic FUN_1801e5520 —
//! Period регистра накопления (−2, 0x404, ветка таблицы переходов 0x1801e56f7), bp FUN_180028d00 / FUN_180030ff0 —
//! Date бизнес-процесса и задачи (−3, 0x401).
//!
//! Ключ (FUN_180679bd0 → `0cbbe0d8` +0x18, mngbase FUN_180062510 = член +0x38 ULCompoundKeyProviderImpl) строит
//! FUN_1805d25b0 по основной таблице (FUN_180679280: фабрика `58e6aa68` +0x18 → FUN_1805d3d10): поле +0x38 части
//! особых полей (Ref = 6-й аргумент слота +0x20, член +0x60); если у части регистратора `3edb7181` +0x30 (backend
//! FUN_1803ea040: третий аргумент слота +0x30 ≠ 0) — у журнала первое поле вектора, иначе весь вектор слота +0x30;
//! `827aac09`/`700fe9ae` есть только у построителя другого класса (QI-карта backend 0x181427980), `743885e8`/`45b4002e`
//! (ключа нет) реализует только edb.dll.
//!
//! Части таблицы по классу — построители basic.dll (посетитель 0x180a0caf8) и bp.dll (0x180285ad0):
//! - регистр сведений FUN_1801da400: вектор +0x30 — [Period при периодичности ≠ 0] + [Recorder при подчинении
//!   регистратору и периодичности «по позиции регистратора» (6)] + все измерения по порядку; регистратор — Recorder
//!   при подчинении (режим записи 1), третий аргумент — всегда новый объект; ведущие +0x40 — [Recorder] при
//!   подчинении, иначе измерения с флагом «Ведущее» по порядку; +0x20 и +0x38 не вызываются;
//! - регистр накопления FUN_1801df930: +0x30([Recorder, LineNumber], Recorder, объект), +0x40([Recorder]);
//! - журнал документов FUN_1801d4c50: +0x30([Ref], 0, объект), +0x40([Ref]);
//! - бизнес-процесс bp FUN_1800275d0 и задача bp FUN_18002d730: только +0x20 (особые поля: Ref, DeletionMark).

use crate::xml::imp::index::Index;

/// Поля ролей основной таблицы `main` (`InformationRegister.Курсы`) в порядке регистрации, без повторов: стандартные —
/// английским именем, измерения — именем измерения.
///
/// Два прохода FUN_18066fe40 и роль 0x400 включают `gen[3]->vt+0x18` / `vt+0x88` — стратегия схемы (mngbase
/// ULSchemeGenerationStrategy, фабрика FUN_180678030 по назначению): gen[3] = список+0x10 (FUN_1806787b0 →
/// FUN_180678730, назначение — ULSchemeInfoImpl vt+0x20 = член +0x1c, FUN_1800480d0). Загрузка списка строит схему
/// mngbase FUN_1802b6b10 → FUN_180297ec0(настройки, 0, 0): назначение 0 — стратегия vtable 0x18088ed98, все признаки
/// 0x180018190 (ложь): проходов и роли 0x400 нет, vt+0x70 ложь.
pub fn main_roles(x: &Index, main: &str) -> Result<Vec<String>, String> {
    let o = x.objects.get(main).ok_or_else(|| format!("нет основной таблицы {main}"))?;
    let class = main.split('.').next().unwrap_or_default();
    let p = |k: &str| o.props.get(k).map(String::as_str).unwrap_or_default();
    let mut v: Vec<&str> = Vec::new();
    match class {
        "Document" => v.extend(["DeletionMark", "Posted", "Date", "Ref"]),
        "Catalog" => {
            v.push("DeletionMark");
            if o.owners {
                v.push("Owner");
            }
            if p("Hierarchical") == "true" {
                v.push("Parent");
                if p("HierarchyType") == "HierarchyFoldersAndItems" {
                    v.push("IsFolder");
                }
            }
            v.push("Ref");
        }
        // план видов характеристик — basic FUN_1801eb8a0 (посетитель +0xb0, поля FUN_1801ee100 — флага 0x400 нет): особые
        // поля +0x20 (Ref −2, DeletionMark −4); при иерархии (метаданные vt+0x268) — +0x38(Parent −6, IsFolder −7):
        // роль 0x80 — первый аргумент (часть 4fbf97f0 +0x18 = член +0x20, backend FUN_1803e89b0), 0x100 — второй (+0x28)
        "ChartOfCharacteristicTypes" => {
            v.push("DeletionMark");
            if p("Hierarchical") == "true" {
                v.extend(["Parent", "IsFolder"]);
            }
            v.push("Ref");
        }
        // план обмена — backend FUN_1801872c0: из слотов построителя только +0x18, +0x20 (особые поля: аргумент 6 Ref −6,
        // аргумент 7 DeletionMark −4) и +0x70; флаг 0x400 у полей нет (FUN_1801890c0: флаги полей — 1)
        "ExchangePlan" => v.extend(["DeletionMark", "Ref"]),
        // план видов расчета — calc FUN_180037670: из слотов построителя только +0x18, +0x20 (особые поля: аргумент 6
        // Ref −6, аргумент 7 DeletionMark −5) и +0x70; +0x30, +0x38, +0x40 не вызываются; флаг 0x400 ставят только
        // построители дат и периодов (см. выше) — таких полей у плана нет
        "ChartOfCalculationTypes" => v.extend(["DeletionMark", "Ref"]),
        // shortcut: как у иерархического справочника без групп (план счетов всегда иерархический), по сверке
        // ФормаСписка Хозрасчетного БП; построитель accnt.dll не разобран
        "ChartOfAccounts" => v.extend(["DeletionMark", "Parent", "Ref"]),
        // 4 −4, 0x10 −7, 0x40 [Ref], 8 Date (−100), ключ [Ref]
        "DocumentJournal" => v.extend(["DeletionMark", "Posted", "Ref", "Date", "Ref"]),
        // 4 DeletionMark, 8 Date, ключ Ref
        "BusinessProcess" | "Task" => v.extend(["DeletionMark", "Date", "Ref"]),
        // 0x20 и 0x40 Recorder, 8 Period, ключ [Recorder, LineNumber]
        "AccumulationRegister" => v.extend(["Recorder", "Recorder", "Period", "Recorder", "LineNumber"]),
        "InformationRegister" => {
            let per = p("InformationRegisterPeriodicity");
            if !matches!(per, "Nonperiodical" | "Second" | "Day" | "Month" | "Quarter" | "Year" | "RecorderPosition") {
                return Err(format!("периодичность регистра сведений {main} «{per}»: роли не выведены"));
            }
            let sub = match p("WriteMode") {
                "Independent" => false,
                "RecorderSubordinate" => true,
                w => return Err(format!("режим записи регистра сведений {main} «{w}»: роли не выведены")),
            };
            let periodic = per != "Nonperiodical";
            if sub {
                v.push("Recorder");
                v.push("Recorder");
            } else {
                // измерения с <Master>true</Master> по порядку (Index::masters)
                v.extend(x.masters.get(main).into_iter().flatten().map(String::as_str));
            }
            if periodic {
                v.push("Period");
                v.push("Period");
            }
            if sub && per == "RecorderPosition" {
                v.push("Recorder");
            }
            v.extend(o.kids.iter().filter(|(k, _)| k == "Dimension").map(|(_, n)| n.as_str()));
            // разделители (basic FUN_1802874d0 — в векторе ключа +0x30) — после измерений, по порядку общих реквизитов
            v.extend(common_fields(x, main).into_iter().filter(|(_, key)| *key).map(|(n, _)| n));
        }
        // критерий отбора: у таблицы критерия одно поле — Ссылка (resolve, `vt_field` критерия), ролью может быть только
        // оно. shortcut: регистрирует ли FUN_18066d920 роль Ref таблицы критерия, не прослежено (построитель таблицы
        // критерия не найден) — dlist принимает список, только если Ссылка выбрана (корень тогда одинаков), иначе отказ
        "FilterCriterion" => v.push("Ref"),
        c => return Err(format!("поля ролей основной таблицы класса {c} для своего запроса пока не выведены")),
    }
    let mut out: Vec<String> = Vec::new();
    for n in v {
        if !out.iter().any(|o| o == n) {
            out.push(n.to_string());
        }
    }
    Ok(out)
}

/// Роли основной таблицы — среза регистра сведений (`InformationRegister.X.SliceLast` / `SliceFirst`): у виртуальной
/// таблицы (backend, карта интерфейсов 0x181427980) нет частей ede4bd22/3edb7181/ae0f5f41/4fbf97f0/7692c826, проход ролей
/// FUN_18066d920 дает только роль 8 (поле с флагом 0x400 — Period, basic FUN_1801dd120: флаги 0xc01) и ключ (построитель
/// +0x28, basic FUN_1801db9f0 0x1801dccfb): Period, Recorder — при подчинении регистратору и периодичности по позиции
/// регистратора, измерения по порядку, разделители ключа (FUN_1802874d0).
pub fn slice_roles(x: &Index, main: &str) -> Result<Vec<String>, String> {
    let base = main.rsplit_once('.').map(|(b, _)| b).unwrap_or(main);
    let o = x.objects.get(base).ok_or_else(|| format!("нет основной таблицы {base}"))?;
    let p = |k: &str| o.props.get(k).map(String::as_str).unwrap_or_default();
    let mut v: Vec<String> = vec!["Period".into()];
    if p("WriteMode") == "RecorderSubordinate" && p("InformationRegisterPeriodicity") == "RecorderPosition" {
        v.push("Recorder".into());
    }
    v.extend(o.kids.iter().filter(|(k, _)| k == "Dimension").map(|(_, n)| n.clone()));
    v.extend(common_fields(x, base).into_iter().filter(|(_, key)| *key).map(|(n, _)| n.to_string()));
    let mut out: Vec<String> = Vec::new();
    for n in v {
        if !out.contains(&n) {
            out.push(n);
        }
    }
    Ok(out)
}

/// Источник основной таблицы в части запроса — mngbase FUN_180665950 (ULQueryEditor, по частям FUN_180665bf0 = модель
/// запроса entext +0x340): первый источник части (entext +0x370 → +0x378 FUN_18022a680: вектор part+0x38) без
/// второго объекта (`vt+0x30`, entext FUN_180037140: член +0x28) и с таблицей (`vt+0x20`, FUN_1802bd440: член +0x20),
/// равной основной (FUN_180670870 → FUN_180670010); нет такого — исключение FUN_1805b9490(2). Вектор part+0x38 заполняет
/// FUN_18026a160 в порядке источников скомпилированной части (entext FUN_18027df90).
/// `srcs` — источники части по порядку обхода `ИЗ` (с соединениями), с классом и объектом у таблиц объектов.
// shortcut: порядок источников скомпилированной части (контейнер части в FUN_18027df90) не прочитан — из нескольких
// подходящих берется только первый источник первого элемента `ИЗ` (его парсер читает раньше всех), иначе отказ.
pub fn main_source<'a>(srcs: &[(&'a crate::query::Source, Option<(String, String)>)], main: &str) -> Result<&'a crate::query::Source, String> {
    let ms: Vec<usize> = (0..srcs.len()).filter(|&i| srcs[i].1.as_ref().is_some_and(|(_, o)| o == main)).collect();
    match ms.as_slice() {
        [i] => Ok(srcs[*i].0),
        [0, ..] => Ok(srcs[0].0),
        [] => Err(format!("источник основной таблицы {main} в своем запросе не найден")),
        _ => Err(format!("источник основной таблицы {main} в своем запросе не первый из нескольких: порядок источников части не выведен")),
    }
}

/// Общие реквизиты — поля таблицы регистра сведений (basic FUN_1801da400): FUN_180287390 — backend FUN_1802aa980
/// (сервис 352cf6f8 +0x28) с режимом 1 и маской 8; FUN_1802874d0 — режим 8, маска 0xff, поле еще и в векторе ключа.
/// Отбор — backend FUN_1802aa640 по общим реквизитам конфигурации по порядку: FUN_1802aa510 — состав (`vt+0x390`:
/// Use — да, DontUse — нет, Auto или нет в составе — АвтоИспользование `vt+0x3b8` = Use); FUN_1802acd40 / FUN_1802acef0 —
/// разделение `vt+0x3a8` и использование разделяемых данных `vt+0x418`: режим 1 — разделение DontUse, режим 8 —
/// Separate и IndependentlyAndSimultaneously. Числа значений — регистрация перечислений backend FUN_180969720
/// (0x18096bcae…0x18096c164): Separate 0, DontUse 1; Independently 0, IndependentlyAndSimultaneously 1; состав Auto 0,
/// Use 1, DontUse 2; АвтоИспользование Use 0, DontUse 1. Возвращает (имя, в ключе).
// shortcut: ветка расширений (GUID владельца не пустой: FUN_1800e40e0 +0x40, FUN_1802ab4f0) не разобрана — у таблиц
// основной конфигурации ее нет; загрузка расширений — отдельная задача.
pub fn common_fields<'a>(x: &'a Index, obj: &str) -> Vec<(&'a str, bool)> {
    let mut v = Vec::new();
    for (n, auto, content) in &x.common {
        let used = match content.get(obj).map(String::as_str) {
            Some("Use") => true,
            Some("DontUse") => false,
            _ => auto == "Use",
        };
        let ca = x.objects.get(&format!("CommonAttribute.{n}"));
        let prop = |k: &str| ca.and_then(|o| o.props.get(k)).map(String::as_str).unwrap_or_default();
        match (used, prop("DataSeparation"), prop("SeparatedDataUse")) {
            (true, "DontUse", _) => v.push((n.as_str(), false)),
            (true, "Separate", "IndependentlyAndSimultaneously") => v.push((n.as_str(), true)),
            _ => {}
        }
    }
    v
}

/// Состав полей таблицы, выведенный кодом построителя: поля (имя на языке 0, на языке 1, составное с именованными
/// подполями) и имена, наличие которых не выведено (по ним — отказ, по прочим отсутствующим — «поля нет»).
pub struct TableFields {
    pub fields: Vec<(String, String, bool)>,
    pub unsure: Vec<(String, String)>,
}

impl TableFields {
    fn std(&mut self, en: &str, ru: &str) {
        self.fields.push((en.to_string(), ru.to_string(), false));
    }
    fn own(&mut self, n: &str) {
        self.fields.push((n.to_string(), n.to_string(), false));
    }
    fn unsure(&mut self, en: &str, ru: &str) {
        self.unsure.push((en.to_string(), ru.to_string()));
    }
}

/// Поля основной таблицы объекта `obj` по построителю; `None` — состав класса кодом не выведен.
/// - регистр сведений — basic FUN_1801da400: Period (периодичность `vt+0x238` ≠ 0), Recorder −3, LineNumber −4,
///   Active −5 (режим записи `vt+0x248` = 1), измерения, ресурсы, реквизиты (коллекции DAT_180a27a78, a08, a18), общие
///   реквизиты (`common_fields`), PointInTime при периодичности 6 — поле backend FUN_180401540 (QI-карта 0x181427520 с
///   ea50a1db), подполя — 9-й аргумент +0x20 (FUN_1803ebbf0 → член +0x1d8, отдает +0x68 FUN_1803ebb80): [Period, Recorder];
/// - документ — basic FUN_1801d2970: Ref −5, DataVersion −6 (совместимость FUN_180033700 vt+0x158 > 1 — всегда), DeletionMark
///   −4, Number −2 (длина номера сервиса FUN_18028d860 +0x18 ≠ 0; при нумераторе — не выведено), Date −3, Posted −7,
///   реквизиты (DAT_180a27698), общие реквизиты режима 1 (FUN_180287390: разделение DontUse), табличные части
///   (DAT_180a27688), Presentation, PointInTime (FUN_18028e120 с подполями).
pub fn table_fields(x: &Index, obj: &str) -> Option<TableFields> {
    let o = x.objects.get(obj)?;
    let class = obj.split('.').next().unwrap_or_default();
    let p = |k: &str| o.props.get(k).map(String::as_str).unwrap_or_default();
    let mut t = TableFields { fields: Vec::new(), unsure: Vec::new() };
    let kids = |kind: &'static str| o.kids.iter().filter(move |(k, _)| k == kind).map(|(_, n)| n.as_str());
    match class {
        "InformationRegister" => {
            let per = p("InformationRegisterPeriodicity");
            if !matches!(per, "Nonperiodical" | "Second" | "Day" | "Month" | "Quarter" | "Year" | "RecorderPosition") || !matches!(p("WriteMode"), "Independent" | "RecorderSubordinate") {
                return None;
            }
            if per != "Nonperiodical" {
                t.std("Period", "Период");
            }
            if p("WriteMode") == "RecorderSubordinate" {
                t.std("Recorder", "Регистратор");
                t.std("LineNumber", "НомерСтроки");
                t.std("Active", "Активность");
            }
            for kind in ["Dimension", "Resource", "Attribute"] {
                kids(kind).for_each(|n| t.own(n));
            }
            common_fields(x, obj).into_iter().for_each(|(n, _)| t.own(n));
            if per == "RecorderPosition" {
                t.fields.push(("PointInTime".into(), "МоментВремени".into(), true));
            }
        }
        "Document" => {
            t.std("Ref", "Ссылка");
            t.std("DataVersion", "ВерсияДанных");
            t.std("DeletionMark", "ПометкаУдаления");
            if number_length(x, obj) > 0 {
                t.std("Number", "Номер");
            }
            t.std("Date", "Дата");
            t.std("Posted", "Проведен");
            kids("Attribute").for_each(|n| t.own(n));
            common_fields(x, obj).into_iter().filter(|(_, sep)| !sep).for_each(|(n, _)| t.own(n));
            kids("TabularSection").for_each(|n| t.own(n));
            t.std("Presentation", "Представление");
            t.fields.push(("PointInTime".into(), "МоментВремени".into(), true));
        }
        // журнал — basic FUN_1801d4c50: Ref −0x65, Date −100, DeletionMark −4, Posted −7, графы (DAT_180a27a98), Type
        // −0xea63 (совместимость > 1 — всегда), −0x66 без имени; Number −2 — если у какого-нибудь регистрируемого документа
        // длина номера ≠ 0 (FUN_180484890: +0x58 ≥ 0); общие реквизиты — FUN_1802876b0 (режим 8, маска 0xff) по
        // метаданным первого регистрируемого документа (FUN_180484730: +0x40 пусто → коллекция vt+0x210, элемент 0)
        "DocumentJournal" => {
            let docs: Vec<&str> = p("RegisteredDocuments").lines().filter(|l| !l.is_empty()).collect();
            t.std("Ref", "Ссылка");
            t.std("Date", "Дата");
            t.std("DeletionMark", "ПометкаУдаления");
            if docs.iter().any(|d| number_length(x, d) > 0) {
                t.std("Number", "Номер");
            }
            t.std("Posted", "Проведен");
            t.std("Type", "Тип");
            kids("Column").for_each(|n| t.own(n));
            if let Some(first) = docs.first() {
                common_fields(x, first).into_iter().filter(|(_, sep)| *sep).for_each(|(n, _)| t.own(n));
            }
        }
        // таблица константы `Константа.X` — basic FUN_1801c8800 (совместимость vt+0x158 ≥ 3, иначе FUN_1801c8b60): поле
        // значения Value (FUN_1801c83c0, термины 0x180d0fb18; тип — тип константы vt+0x1f8) и общие реквизиты
        // FUN_1802876b0 (режим 8, маска 0xff — разделители, как у журнала); частей построителя нет (только +0x18)
        "Constant" => {
            t.std("Value", "Значение");
            common_fields(x, obj).into_iter().filter(|(_, sep)| *sep).for_each(|(n, _)| t.own(n));
        }
        _ => return None,
    }
    Some(t)
}

/// Тип поля Ref журнала документов — тип ссылки метаданных журнала (интерфейс ae5b864a +0x48, basic FUN_180777930;
/// поле −0x65 построителя FUN_1801d6ff0 берет его через сервис 6df9a273 +0x40 FUN_180778280): по регистрируемым документам
/// (коллекция `vt+0x210` вида 1) по порядку — тип ссылки каждого документа (`vt+0x368`, addType).
pub fn journal_ref(x: &Index, obj: &str) -> Vec<crate::query::resolve::Ty> {
    let docs = x.objects.get(obj).and_then(|o| o.props.get("RegisteredDocuments")).map(String::as_str).unwrap_or_default();
    let v: Vec<&str> = docs.lines().collect();
    // документа нет в индексе или их нет совсем — тип не выведен (отказ, а не «поля нет»)
    if v.is_empty() || v.iter().any(|d| !x.objects.contains_key(*d)) {
        return vec![crate::query::resolve::Ty::Unknown];
    }
    v.into_iter().map(|d| crate::query::resolve::Ty::Ref(d.to_string())).collect()
}

/// Длина номера документа — сервис 2bcb58d5 (basic FUN_18028d860) +0x18 = FUN_1806fb380: у документа с нумератором
/// (FUN_1806fb270) — длина номера нумератора (vt+0x1e0), иначе — документа (vt+0x478).
// shortcut: слоты vt+0x478 / vt+0x1e0 сопоставлены со свойством NumberLength по имени (у документа без нумератора
// правило корня сверено на корпусах), сопоставление по коду метаданных не прослежено.
fn number_length(x: &Index, doc: &str) -> i64 {
    let Some(o) = x.objects.get(doc) else { return 0 };
    let len = |o: &crate::xml::imp::index::Obj| o.props.get("NumberLength").and_then(|v| v.parse().ok()).unwrap_or(0);
    match o.props.get("Numerator").filter(|n| !n.is_empty()) {
        Some(n) => x.objects.get(n).map(len).unwrap_or(0),
        None => len(o),
    }
}

/// Поля среза регистра сведений (`СрезПоследних` — `last`, `СрезПервых`) — basic FUN_1801db9f0: Period (периодичность
/// ≠ 0), Recorder (подчинение регистратору), LineNumber −4 и Active −5 — при подчинении и признаке метаданных
/// `vt+0x3c0` (последних) / `vt+0x3b0` (первых) = ложь — какое это свойство, не выведено; измерения, ресурсы, реквизиты,
/// общие реквизиты (FUN_180287390, FUN_1802874d0). PointInTime в срезе нет.
pub fn slice_fields(x: &Index, obj: &str) -> Option<TableFields> {
    let o = x.objects.get(obj)?;
    let p = |k: &str| o.props.get(k).map(String::as_str).unwrap_or_default();
    let per = p("InformationRegisterPeriodicity");
    if !obj.starts_with("InformationRegister.") || per == "Nonperiodical" || !matches!(p("WriteMode"), "Independent" | "RecorderSubordinate") {
        return None;
    }
    let mut t = TableFields { fields: Vec::new(), unsure: Vec::new() };
    t.std("Period", "Период");
    if p("WriteMode") == "RecorderSubordinate" {
        t.std("Recorder", "Регистратор");
        t.unsure("LineNumber", "НомерСтроки");
        t.unsure("Active", "Активность");
    }
    for kind in ["Dimension", "Resource", "Attribute"] {
        o.kids.iter().filter(|(k, _)| k == kind).for_each(|(_, n)| t.own(n));
    }
    common_fields(x, obj).into_iter().for_each(|(n, _)| t.own(n));
    Some(t)
}

/// Поля `Т.*` источника — таблицы объекта `obj`: (имя на языке 0, на языке 1). entext FUN_18034a270 → FUN_180349c70:
/// все поля таблицы (`vt+0x40`; интерфейса 240b0eea у таблиц basic нет) по порядку, имя — `поле+0x18` на языке;
/// табличная часть — вложенной таблицей, поле с подполями (`ea50a1db`) — только если у подполей нет имен (FUN_1801dbb80):
/// PointInTime регистра сведений (подполя Period, Recorder) в `.*` не входит.
// shortcut: `.*` документа и справочника здесь — отказ: их состав (DataVersion в корне, Presentation нет) — по образцам
// в dlist.rs `query_root_fields` (ветка `Т.*`), отбор Представление в корень dcs не выведен.
pub fn star_fields(x: &Index, obj: &str) -> Result<Vec<(String, String)>, String> {
    let class = obj.split('.').next().unwrap_or_default();
    if class != "InformationRegister" {
        return Err(format!("`.*` таблицы класса {class} в своем запросе: состав полей не выведен"));
    }
    let t = table_fields(x, obj).ok_or_else(|| format!("`.*` {obj}: состав полей не выведен"))?;
    Ok(t.fields.into_iter().filter(|(_, _, composite)| !composite).map(|(a, b, _)| (a, b)).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::xml::imp::index::Obj;

    fn reg(props: &[(&str, &str)], dims: &[&str]) -> Obj {
        let mut o = Obj::default();
        o.props = props.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect();
        o.kids = dims.iter().map(|d| ("Dimension".to_string(), d.to_string())).collect();
        o
    }

    #[test]
    fn register_roles() {
        let mut x = Index::default();
        // периодический независимый: ведущие, Period, затем ключ — Period и измерения (трасса БП: СтавкаВзноса…)
        x.objects.insert("InformationRegister.А".into(), reg(&[("InformationRegisterPeriodicity", "Month"), ("WriteMode", "Independent")], &["Организация", "Вид"]));
        x.masters.insert("InformationRegister.А".into(), vec!["Организация".into()]);
        assert_eq!(main_roles(&x, "InformationRegister.А").unwrap(), ["Организация", "Period", "Вид"]);
        // подчиненный регистратору по позиции регистратора: Recorder, Period, измерения; ведущие измерения не берутся
        x.objects.insert("InformationRegister.Б".into(), reg(&[("InformationRegisterPeriodicity", "RecorderPosition"), ("WriteMode", "RecorderSubordinate")], &["Счет"]));
        x.masters.insert("InformationRegister.Б".into(), vec!["Счет".into()]);
        assert_eq!(main_roles(&x, "InformationRegister.Б").unwrap(), ["Recorder", "Period", "Счет"]);
        x.objects.insert("AccumulationRegister.В".into(), Obj::default());
        assert_eq!(main_roles(&x, "AccumulationRegister.В").unwrap(), ["Recorder", "Period", "LineNumber"]);
        x.objects.insert("ChartOfAccounts.Х".into(), Obj::default());
        assert_eq!(main_roles(&x, "ChartOfAccounts.Х").unwrap(), ["DeletionMark", "Parent", "Ref"]);
        x.objects.insert("DocumentJournal.Г".into(), Obj::default());
        assert_eq!(main_roles(&x, "DocumentJournal.Г").unwrap(), ["DeletionMark", "Posted", "Ref", "Date"]);
        x.objects.insert("InformationRegister.Д".into(), reg(&[("WriteMode", "Independent")], &[]));
        assert!(main_roles(&x, "InformationRegister.Д").is_err());
        // разделитель с IndependentlyAndSimultaneously — в ключе после измерений; Independently — не поле таблицы
        let ca = |sep: &str, sdu: &str| reg(&[("DataSeparation", sep), ("SeparatedDataUse", sdu)], &[]);
        x.objects.insert("CommonAttribute.Вспом".into(), ca("Separate", "IndependentlyAndSimultaneously"));
        x.objects.insert("CommonAttribute.Осн".into(), ca("Separate", "Independently"));
        let use_in = |o: &str| [(o.to_string(), "Use".to_string())].into_iter().collect();
        x.common = vec![("Вспом".into(), "DontUse".into(), use_in("InformationRegister.А")), ("Осн".into(), "Use".into(), Default::default())];
        assert_eq!(main_roles(&x, "InformationRegister.А").unwrap(), ["Организация", "Period", "Вид", "Вспом"]);
        let s: Vec<String> = star_fields(&x, "InformationRegister.А").unwrap().into_iter().map(|(a, b)| format!("{a}|{b}")).collect();
        assert_eq!(s, ["Period|Период", "Организация|Организация", "Вид|Вид", "Вспом|Вспом"]);
        let s: Vec<String> = star_fields(&x, "InformationRegister.Б").unwrap().into_iter().map(|(a, _)| a).collect();
        assert_eq!(s, ["Period", "Recorder", "LineNumber", "Active", "Счет"]);
        assert!(table_fields(&x, "InformationRegister.Б").unwrap().fields.iter().any(|(a, _, c)| a == "PointInTime" && *c));
        x.objects.insert("ChartOfCharacteristicTypes.Е".into(), reg(&[("Hierarchical", "true")], &[]));
        assert_eq!(main_roles(&x, "ChartOfCharacteristicTypes.Е").unwrap(), ["DeletionMark", "Parent", "IsFolder", "Ref"]);
    }
}
