//! Управляемая форма: запись `<форма>.0` = `{4,{дерево формы},"модуль",{реквизиты},{параметры},{команды},...}` → Ext/Form.xml.
//! Устройство записи — docs/research/xml-form.md. Незнакомые значения пишутся как `?<что>:<значение>`,
//! чтобы сверка с платформой их показала, а не прятала.

use crate::brace::{self, is_guid, Node};
use crate::xml::form_dcs as dcs;
use crate::meta::{Config, Item};
use std::collections::{HashMap, HashSet};

#[path = "form_settings.rs"]
mod settings;

/// Имена объектов конфигурации по GUID: для путей к данным, типов, картинок, команд.
#[derive(Default)]
pub struct Names {
    /// GUID объекта или вложенного элемента → полное имя (`Catalog.Банки`, `Catalog.Банки.Attribute.Код`).
    full: HashMap<String, String>,
    /// GUID типа → имя типа (`CatalogRef.Банки`).
    types: HashMap<String, String>,
    /// Объекты с непустым «Вводится на основании» — только у них есть команда CreateBasedOn.
    based_on: HashSet<String>,
    /// Справочники с владельцами: только у них есть команда OpenByValue (basic FUN_1801a0ae0 создает `{4,guid}`,
    /// если список владельцев не пуст).
    owned: HashSet<String>,
    /// Объекты со снятым «Использовать стандартные команды»: их стандартные команды платформа пишет `N:guid`.
    no_std_commands: HashSet<String>,
    /// Документы с запрещенным проведением: у их списков нет команд Post и UndoPosting.
    no_posting: HashSet<String>,
    /// Периодические регистры сведений: у их списков SetDateInterval есть.
    periodic: HashSet<String>,
    /// Регистр сведений → ведущие измерения по порядку (команды OpenByValue.<измерение>).
    masters: HashMap<String, Vec<String>>,
    /// Регистр сведений → все измерения в нижнем регистре: ключ записи, в списке своего запроса есть всегда.
    dims: HashMap<String, HashSet<String>>,
    /// Неиерархические справочники и планы видов характеристик: у их списков нет IsFolder и Parent.
    flat: HashSet<String>,
    /// Справочники без групп (неиерархические и с иерархией элементов): у их списков нет IsFolder.
    no_folders: HashSet<String>,
    /// Реквизит объекта → GUID его единственного типа (для стандартных полей в пути к данным).
    attr_type: HashMap<String, String>,
    /// Объект → имена всех вложенных (реквизиты, измерения, ТЧ...) в нижнем регистре: поля основной таблицы списка.
    fields: HashMap<String, HashSet<String>>,
    /// Предопределенные: `<владелец>.<guid>` → `<владелец>.<имя>` (как в common::Index).
    predef: HashMap<String, String>,
    /// `класс.имя` объекта в нижнем регистре (`catalog.банки`) → GUID: таблицы в текстах запросов.
    objects: HashMap<String, String>,
    /// Константа → признак n[1][11] записи версии 16 (в XML его нет): у `true` UseAlways набора констант обратный.
    /// Нет константы — запись другой версии, признак не прочитан.
    const_inverted: HashMap<String, bool>,
    /// Константы типа ХранилищеЗначения (для `constants_use_always` старых форматов).
    const_storage: HashSet<String>,
    /// GUID типа определяемого типа → GUID его единственного типа (n[1][4] записи): класс реквизита формы такого типа.
    defined: HashMap<String, String>,
    /// Объект → имена его элементов (реквизиты, измерения, ТЧ...) в нижнем регистре → как в описании.
    cased: HashMap<String, HashMap<String, String>>,
    /// Объект → поле в нижнем регистре → GUID ссылочных типов его описания (`#`); пусто — только примитивные.
    field_refs: HashMap<String, HashMap<String, Vec<String>>>,
    /// Определяемые типы (GUID типа) только из примитивных типов.
    prim_defined: HashSet<String>,
    /// Режим совместимости 8.3.16 и ниже: поле списка со своим запросом доступно, только если оно есть в выборке.
    old_lists: bool,
    /// Режим совместимости 8.3.10 и ниже: загрузка второй `~` пути не разбирает (dlist tilde_split, > 0x139b6).
    one_path: bool,
    /// Целевая версия формата старше 2.21 (`--format`): номер после «2.» (17, 20). То, что выгрузка 8.5 из записи
    /// не выводит, а старые платформы пишут, — здесь, а не в таблицах formats/ (из текста 2.21 его не восстановить).
    pub older: Option<u32>,
    /// Индекс загрузки по той же конфигурации (`formedit::index`): корень списка со своим запросом — моделью загрузки
    /// (`path_names`); нет — признак `~` только по тексту запроса (`unavailable`).
    pub index: Option<crate::xml::imp::index::Index>,
}

/// GUID ссылочных типов описания типов `{"Pattern", {…}…}`; `None` — не описание типов.
fn pattern_refs(p: &Node) -> Option<Vec<String>> {
    let l = p.list();
    if l.first().and_then(Node::str) != Some("Pattern") {
        return None;
    }
    Some(l[1..].iter().filter(|t| t.get(0).and_then(Node::str) == Some("#")).map(|t| atom_at(t, 1).to_string()).collect())
}

fn nested_names(it: &Item, out: &mut HashSet<String>) {
    for ch in &it.children {
        out.insert(ch.header.name.to_lowercase());
        nested_names(ch, out);
    }
}

/// Имена полей выборки запроса (в нижнем регистре): первый запрос пакета без ПОМЕСТИТЬ, первая часть объединения,
/// псевдоним после КАК или последнее имя выражения.
fn query_fields(q: &str) -> HashSet<String> {
    query_select(query_main(&query_clean(q)))
}

/// Первый запрос пакета без ПОМЕСТИТЬ (очищенного `query_clean`).
fn query_main(t: &str) -> &str {
    let words = |s: &str| -> Vec<String> { s.split(|c: char| !(c.is_alphanumeric() || c == '_')).filter(|w| !w.is_empty()).map(str::to_string).collect() };
    let parts: Vec<&str> = t.split(';').filter(|p| !p.trim().is_empty()).collect();
    // Пропускаются и УНИЧТОЖИТЬ временных таблиц.
    parts
        .iter()
        .find(|p| {
            let w = words(p);
            w.iter().any(|w| w == "выбрать" || w == "select") && !w.iter().any(|w| w == "поместить" || w == "into")
        })
        .or(parts.last())
        .copied()
        .unwrap_or("")
}

/// Стандартные поля основной таблицы динамического списка: имя в пути (латиницей) и русское (в своем запросе).
const STD_LIST_FIELDS: &[(&str, &str)] = &[
    ("Ref", "Ссылка"), ("Code", "Код"), ("Description", "Наименование"), ("DeletionMark", "ПометкаУдаления"),
    ("Predefined", "Предопределенный"), ("PredefinedDataName", "ИмяПредопределенныхДанных"), ("Parent", "Родитель"),
    ("Owner", "Владелец"), ("IsFolder", "ЭтоГруппа"), ("Number", "Номер"), ("Date", "Дата"), ("Posted", "Проведен"),
    ("PointInTime", "МоментВремени"), ("Period", "Период"), ("Recorder", "Регистратор"), ("LineNumber", "НомерСтроки"),
    ("Active", "Активность"), ("DefaultPicture", ""), ("Type", "Тип"), ("ValueType", "ТипЗначения"),
    ("BusinessProcess", "БизнесПроцесс"), ("Completed", "Завершен"), ("Started", "Стартован"), ("HeadTask", "ВедущаяЗадача"),
    ("Executed", "Выполнена"), ("RoutePoint", "ТочкаМаршрута"), ("ReceivedNo", "НомерПринятого"), ("SentNo", "НомерОтправленного"),
    ("ThisNode", "ЭтотУзел"), ("Order", "Порядок"), ("OffBalance", "Забалансовый"), ("Kind", "Вид"), ("Account", "Счет"),
    ("CalculationType", "ВидРасчета"), ("RegistrationPeriod", "ПериодРегистрации"), ("ActionPeriod", "ПериодДействия"),
    ("BegOfActionPeriod", "ПериодДействияНачало"), ("EndOfActionPeriod", "ПериодДействияКонец"),
    ("BegOfBasePeriod", "БазовыйПериодНачало"), ("EndOfBasePeriod", "БазовыйПериодКонец"), ("ReversingEntry", "Сторно"),
    ("DataVersion", "ВерсияДанных"), ("ExtDimensionType", "ВидСубконто"), ("TurnoversOnly", "ТолькоОбороты"),
    // план видов расчета (ERP Начисления.ФормаСписка — свой запрос с табличными частями)
    ("ActionPeriodIsBasic", "ПериодДействияБазовый"), ("BaseCalculationTypes", "БазовыеВидыРасчета"),
    ("LeadingCalculationTypes", "ВедущиеВидыРасчета"), ("DisplacingCalculationTypes", "ВытесняющиеВидыРасчета"),
];

/// Стандартное поле `eng` (из STD_LIST_FIELDS) есть у таблицы класса `class`; класс не из списка — считается, что есть.
/// Состав — стандартные реквизиты объектов платформы (ПравилаФинОтчетности: `Активность` у справочника недоступна).
fn std_applies(class: &str, eng: &str) -> bool {
    const REF: &[&str] = &["Ref", "DeletionMark", "DataVersion", "DefaultPicture"];
    const CATALOG: &[&str] = &["Code", "Description", "Predefined", "PredefinedDataName", "Parent", "Owner", "IsFolder"];
    const DOC: &[&str] = &["Number", "Date", "Posted", "PointInTime"];
    let own: &[&str] = match class {
        "Catalog" => CATALOG,
        "Document" => DOC,
        "ChartOfCharacteristicTypes" => &["Code", "Description", "Predefined", "PredefinedDataName", "Parent", "IsFolder", "ValueType"],
        "ExchangePlan" => &["Code", "Description", "ReceivedNo", "SentNo", "ThisNode"],
        // у регистра сведений ссылочных полей нет (КлиентБанк.ЗагрузкаВыписки ERP: `~ЗагруженныеДокументы.Posted`)
        "InformationRegister" => return matches!(eng, "Period" | "Recorder" | "LineNumber" | "Active" | "PointInTime" | "DefaultPicture"),
        _ => return true,
    };
    REF.contains(&eng) || own.contains(&eng)
}

/// Стандартного поля `eng` у таблицы класса `class` нет вовсе: в своем запросе платформа пишет его двумя путями
/// (ЖурналСтатусовФинОтчетностиВБанки: `~Список.Ref~Список.Ссылка`).
/// shortcut: только Ref регистра сведений (одна форма БП); прочие поля регистров — когда встретятся.
fn std_absent(class: &str, eng: &str) -> bool {
    class == "InformationRegister" && eng == "Ref"
}

/// Класс первой таблицы источника первого запроса пакета без ПОМЕСТИТЬ (`ИЗ Документ.Х` → `Document`); `None` — не
/// таблица конфигурации.
fn first_source(q: &str) -> Option<&'static str> {
    let t = query_clean(q);
    let mut depth = 0i32;
    let mut words = query_main(&t).split(|c: char| {
        depth += match c { '(' => 1, ')' => -1, _ => 0 };
        !(c.is_alphanumeric() || c == '_' || c == '.') || depth > 0
    });
    words.find(|w| *w == "из" || *w == "from")?;
    Some(match query_class(words.find(|w| !w.is_empty())?.split('.').next()?)? {
        "catalog" => "Catalog",
        "document" => "Document",
        "chartofcharacteristictypes" => "ChartOfCharacteristicTypes",
        "exchangeplan" => "ExchangePlan",
        "informationregister" => "InformationRegister",
        _ => "",
    })
}

/// Русское имя стандартного поля в нижнем регистре сравнивается с `lname`.
fn std_ru_is(r: &str, lname: &str) -> bool {
    !r.is_empty() && r.to_lowercase() == lname
}

/// Таблица метаданных в тексте запроса: русское или английское имя класса → класс конфигурации.
fn query_class(w: &str) -> Option<&'static str> {
    Some(match w {
        "справочник" | "catalog" => "catalog",
        "документ" | "document" => "document",
        "регистрсведений" | "informationregister" => "informationregister",
        "регистрнакопления" | "accumulationregister" => "accumulationregister",
        "регистрбухгалтерии" | "accountingregister" => "accountingregister",
        "регистррасчета" | "calculationregister" => "calculationregister",
        "плансчетов" | "chartofaccounts" => "chartofaccounts",
        "планвидовхарактеристик" | "chartofcharacteristictypes" => "chartofcharacteristictypes",
        "планвидоврасчета" | "chartofcalculationtypes" => "chartofcalculationtypes",
        "планобмена" | "exchangeplan" => "exchangeplan",
        "бизнеспроцесс" | "businessprocess" => "businessprocess",
        "задача" | "task" => "task",
        "журналдокументов" | "documentjournal" => "documentjournal",
        "перечисление" | "enum" => "enum",
        "константа" | "constant" => "constant",
        _ => return None,
    })
}

/// Текст запроса в нижнем регистре без комментариев и строковых литералов (чтобы не мешали поиску слов).
fn query_clean(q: &str) -> String {
    query_strip(q).chars().flat_map(char::to_lowercase).collect()
}

/// Текст запроса без комментариев и строковых литералов, регистр как в тексте.
fn query_strip(q: &str) -> String {
    let mut t = String::new();
    let mut it = q.chars().peekable();
    while let Some(c) = it.next() {
        match c {
            '/' if it.peek() == Some(&'/') => {
                while it.peek().is_some_and(|&c| c != '\n') {
                    it.next();
                }
            }
            '"' => {
                while let Some(c) = it.next() {
                    if c == '"' && it.peek() != Some(&'"') {
                        break;
                    }
                    if c == '"' {
                        it.next();
                    }
                }
                t.push_str("\"\"");
            }
            _ => t.push(c),
        }
    }
    t
}

/// Имена полей выборки одного запроса `main` (очищенного `query_clean`).
fn query_select(main: &str) -> HashSet<String> {
    // Токены верхнего уровня: скобки, запятые, слова.
    let mut out = HashSet::new();
    let (mut depth, mut started, mut item) = (0i32, false, Vec::<String>::new());
    let mut word = String::new();
    let flush = |word: &mut String, item: &mut Vec<String>, depth: i32| {
        if !word.is_empty() {
            if depth == 0 {
                item.push(std::mem::take(word));
            } else {
                word.clear();
            }
        }
    };
    let mut raw = String::new();
    let end_item = |item: &mut Vec<String>, out: &mut HashSet<String>, raw: &mut String| {
        // Путь без псевдонима из трех и более частей (`Т.УчетнаяЗапись.Сервис`) платформа называет склейкой частей
        // после первой (`УчетнаяЗаписьСервис`, Файлы.Файлы).
        let parts: Vec<&str> = raw.trim().split('.').collect();
        if parts.len() >= 3 && parts.iter().all(|w| !w.is_empty() && w.chars().all(|c| c.is_alphanumeric() || c == '_')) {
            out.insert(parts[1..].concat());
        }
        raw.clear();
        if let Some(last) = item.last() {
            out.insert(last.clone());
            // `НЕОПРЕДЕЛЕНО КАК Ссылка` — еще и `=неопределено:ссылка` (поле без типа-ссылки).
            if let [v, k, a] = &item[..] {
                if matches!(v.as_str(), "неопределено" | "undefined" | "null") && matches!(k.as_str(), "как" | "as") {
                    out.insert(format!("=неопределено:{a}"));
                }
            }
        }
        item.clear();
    };
    for c in main.chars().chain(std::iter::once(' ')) {
        // `{ВЫБРАТЬ ...}`, `{ГДЕ ...}` компоновки — за выборкой (НовыйИнтерфейсЭДО: поле перед `{ВЫБРАТЬ` — в выборке).
        if c == '{' && depth == 0 && started {
            flush(&mut word, &mut item, depth);
            break;
        }
        if depth == 0 && started && c != ',' {
            raw.push(c);
        }
        if c.is_alphanumeric() || c == '_' {
            word.push(c);
            continue;
        }
        flush(&mut word, &mut item, depth);
        if !started {
            if item.iter().any(|w| w == "выбрать" || w == "select") {
                started = true;
                item.clear();
                raw.clear();
            }
            continue;
        }
        if let Some(w) = item.last() {
            if depth == 0 && ["из", "from", "где", "where", "объединить", "union", "сгруппировать", "упорядочить", "имеющие", "индексировать"].contains(&w.as_str()) {
                item.pop();
                break;
            }
        }
        match c {
            '(' => depth += 1,
            ')' => depth -= 1,
            ',' if depth == 0 => end_item(&mut item, &mut out, &mut raw),
            _ => {}
        }
    }
    end_item(&mut item, &mut out, &mut raw);
    out
}

/// Слова каждого поля выборки первого запроса пакета (регистр как в тексте, без РАЗРЕШЕННЫЕ/РАЗЛИЧНЫЕ/ПЕРВЫЕ N).
/// `None` — `*` в выборке или `{` компоновки (с `brace_ends` — `{` заканчивает выборку, как в `query_select`).
fn select_words(q: &str, brace_ends: bool) -> Option<Vec<Vec<String>>> {
    let lower = query_clean(q);
    let cased = query_strip(q);
    let main = query_main(&lower);
    // Тот же текст без смены регистра: у кириллицы и латиницы длина в байтах при этом не меняется.
    let off = main.as_ptr() as usize - lower.as_ptr() as usize;
    let main = cased.get(off..off + main.len()).filter(|_| cased.len() == lower.len())?;
    let (mut depth, mut items, mut cur, mut started) = (0i32, Vec::new(), String::new(), false);
    let mut word = String::new();
    for c in main.chars().chain(std::iter::once(' ')) {
        if c.is_alphanumeric() || c == '_' {
            word.push(c);
            cur.push(c);
            continue;
        }
        let w = std::mem::take(&mut word).to_lowercase();
        if !started {
            if w == "выбрать" || w == "select" {
                started = true;
                cur.clear();
            }
            continue;
        }
        if depth == 0 && ["из", "from", "где", "where", "объединить", "union", "сгруппировать", "упорядочить", "имеющие", "индексировать", "поместить", "into"].contains(&w.as_str()) {
            cur.truncate(cur.len() - w.len());
            break;
        }
        match c {
            '{' if depth == 0 && brace_ends => break,
            '{' | '*' => return None,
            '(' => depth += 1,
            ')' => depth -= 1,
            ',' if depth == 0 => {
                items.push(std::mem::take(&mut cur));
                continue;
            }
            _ => {}
        }
        cur.push(c);
    }
    items.push(cur);
    let mut out = Vec::new();
    for (i, it) in items.iter().enumerate() {
        let mut words: Vec<String> = it.split_whitespace().map(str::to_string).collect();
        if i == 0 {
            // ВЫБРАТЬ РАЗРЕШЕННЫЕ / РАЗЛИЧНЫЕ / ПЕРВЫЕ N.
            while let Some(w) = words.first().map(|w| w.to_lowercase()) {
                match w.as_str() {
                    "разрешенные" | "allowed" | "различные" | "distinct" => words.remove(0),
                    "первые" | "top" if words.len() > 1 => {
                        words.drain(..2);
                        continue;
                    }
                    _ => break,
                };
            }
        }
        out.push(words);
    }
    Some(out)
}

/// Поля выборки первого запроса пакета под именами, которые им дает платформа: псевдоним; без псевдонима — последнее
/// имя пути (стандартное — латиницей, как в FieldsMap: `Т.Проведен` → `Posted`), путь из трех и более частей — склейка
/// частей после первой. `None` — выборку не разобрать однозначно (выражение без псевдонима, `*`, `{` компоновки).
fn query_items(q: &str) -> Option<Vec<String>> {
    let mut out = Vec::new();
    for words in select_words(q, false)? {
        let words: Vec<&str> = words.iter().map(String::as_str).collect();
        let ident = |s: &str| !s.is_empty() && s.chars().all(|c| c.is_alphanumeric() || c == '_');
        let name = match &words[..] {
            [.., k, a] if matches!(k.to_lowercase().as_str(), "как" | "as") && ident(a) => a.to_string(),
            [p] => {
                let parts: Vec<&str> = p.split('.').collect();
                match &parts[..] {
                    [_, f] if ident(f) => STD_LIST_FIELDS.iter().find(|(_, r)| std_ru_is(r, &f.to_lowercase())).map_or(f.to_string(), |(e, _)| e.to_string()),
                    [_, rest @ ..] if rest.len() >= 2 && rest.iter().all(|s| ident(s)) => rest.concat(),
                    _ => return None,
                }
            }
            _ => return None,
        };
        out.push(name);
    }
    Some(out)
}

/// Поля своего запроса динамического списка, которых нет в FieldsMap: платформа при загрузке дает им номера за
/// наибольшим по алфавиту имен (ПередачаОС: 158 при 151 — седьмое из 18 по алфавиту, не по порядку выборки; ПутевойЛист,
/// РегламентныеОперацииНДС, ПрослеживаемостьУведомления). Возвращает (наибольший номер FieldsMap, имена за ним); имя `None`,
/// если место зависит от сравнения строк (регистр, ё). Запрос, который не строится, полей не дает (ПередачаОС каркаса:
/// 158 не пишется). `None` — выборка не разобрана или запрос не свой.
fn list_extension(names: &Names, st: &Node) -> Option<(u32, Vec<Option<String>>)> {
    let kv = st.list().get(2..).unwrap_or(&[]);
    let get = |k: &str| kv.chunks(2).find(|c| c[0].str() == Some(k)).and_then(|c| c.get(1));
    let q = get("QueryText").and_then(|v| v.get(1)).and_then(Node::str)?;
    if get("ManualQuery").map(|v| atom_at(v, 1)) != Some("1") {
        return None;
    }
    let mut max = 0;
    let mut mapped = HashSet::new();
    for k in 0.. {
        let Some(i) = get(&format!("FieldsMapItemId{k}")).and_then(|v| v.get(1)).and_then(Node::atom) else { break };
        max = max.max(i.parse::<u32>().ok()?);
        mapped.insert(get(&format!("FieldsMapItemName{k}")).and_then(|v| v.get(1)).and_then(Node::str)?.to_lowercase());
    }
    if names.query_broken(q) {
        return Some((max, Vec::new()));
    }
    let mut seen = HashSet::new();
    let rest: Vec<String> = query_items(q)?.into_iter().filter(|n| !mapped.contains(&n.to_lowercase()) && seen.insert(n.to_lowercase())).collect();
    // Порядки, между которыми данные не выбирают: с регистром и без, ё как е и отдельно. Латиница — раньше кириллицы
    // (ПрослеживаемостьУведомления.ФормаСпискаРНПТ: 26 — DeletionMark раньше ДокументУведомление).
    let orders: Vec<Vec<&String>> = [0, 1, 4, 5]
        .into_iter()
        .map(|v| {
            let key = |s: &String| -> Vec<char> {
                s.chars()
                    .map(|c| {
                        let c = if v & 1 == 1 { c.to_lowercase().next().unwrap_or(c) } else { c };
                        if v & 4 == 4 { match c { 'ё' => 'е', 'Ё' => 'Е', c => c } } else { c }
                    })
                    .collect()
            };
            let mut o: Vec<&String> = rest.iter().collect();
            o.sort_by_key(|s| key(s));
            o
        })
        .collect();
    let names = (0..rest.len()).map(|k| orders.iter().all(|o| o[k] == orders[0][k]).then(|| orders[0][k].clone())).collect();
    Some((max, names))
}

/// Позиция списка «Вводится на основании» в n[1] описания объекта (как в classes_refs).
fn based_on_pos(class: &str) -> Option<usize> {
    Some(match class {
        "Catalog" => 32,
        "Document" => 22,
        "ChartOfCharacteristicTypes" | "BusinessProcess" => 15,
        "ChartOfAccounts" => 18,
        "ChartOfCalculationTypes" => 36,
        "ExchangePlan" => 24,
        "Task" => 30,
        _ => return None,
    })
}

/// Позиция флага UseStandardCommands в n[1] описания объекта (те же, что в таблицах classes_*.rs).
/// shortcut: у AccountingRegister позиция зависит от версии записи (upgrade_21) — не проверяется.
fn use_std_commands_pos(class: &str) -> Option<usize> {
    Some(match class {
        "Catalog" => 31,
        "Document" => 23,
        "ChartOfCharacteristicTypes" => 14,
        "ChartOfAccounts" => 16,
        "ChartOfCalculationTypes" => 35,
        "ExchangePlan" => 13,
        "BusinessProcess" | "Task" => 2,
        "InformationRegister" => 21,
        "AccumulationRegister" => 16,
        "CalculationRegister" => 24,
        "DocumentJournal" | "DataProcessor" => 5,
        "Report" | "FilterCriterion" => 7,
        _ => return None,
    })
}

/// Порядок пар (тип, значение) в начале описания объекта: (позиция типа в n[1], имя типа).
/// Позиции получены сверкой с InternalInfo выгрузки платформы (BP 3.0.171).
fn generated_types(class: &str) -> &'static [(usize, &'static str)] {
    match class {
        "Catalog" => &[(1, "CatalogObject"), (3, "CatalogRef"), (5, "CatalogSelection"), (7, "CatalogList"), (34, "CatalogManager")],
        "Document" => &[(1, "DocumentObject"), (3, "DocumentRef"), (5, "DocumentSelection"), (7, "DocumentList"), (26, "DocumentManager")],
        "Enum" => &[(1, "EnumRef"), (3, "EnumManager"), (7, "EnumList")],
        "InformationRegister" => &[
            (1, "InformationRegisterRecord"),
            (3, "InformationRegisterManager"),
            (5, "InformationRegisterSelection"),
            (7, "InformationRegisterList"),
            (9, "InformationRegisterRecordSet"),
            (11, "InformationRegisterRecordKey"),
            (13, "InformationRegisterRecordManager"),
        ],
        "AccumulationRegister" => &[
            (1, "AccumulationRegisterRecord"),
            (3, "AccumulationRegisterManager"),
            (5, "AccumulationRegisterSelection"),
            (7, "AccumulationRegisterList"),
            (9, "AccumulationRegisterRecordSet"),
            (11, "AccumulationRegisterRecordKey"),
        ],
        "AccountingRegister" => &[
            (2, "AccountingRegisterRecord"),
            (4, "AccountingRegisterExtDimensions"),
            (6, "AccountingRegisterRecordSet"),
            (8, "AccountingRegisterRecordKey"),
            (10, "AccountingRegisterSelection"),
            (12, "AccountingRegisterList"),
            (14, "AccountingRegisterManager"),
        ],
        "DataProcessor" => &[(1, "DataProcessorObject"), (7, "DataProcessorManager")],
        "Report" => &[(1, "ReportObject"), (12, "ReportManager")],
        "ChartOfAccounts" => &[
            (1, "ChartOfAccountsObject"),
            (3, "ChartOfAccountsRef"),
            (5, "ChartOfAccountsSelection"),
            (7, "ChartOfAccountsList"),
            (9, "ChartOfAccountsManager"),
            (11, "ChartOfAccountsExtDimensionTypes"),
            (13, "ChartOfAccountsExtDimensionTypesRow"),
        ],
        "ChartOfCharacteristicTypes" => &[
            (1, "ChartOfCharacteristicTypesObject"),
            (3, "ChartOfCharacteristicTypesRef"),
            (5, "ChartOfCharacteristicTypesSelection"),
            (7, "ChartOfCharacteristicTypesList"),
            (9, "Characteristic"),
            (11, "ChartOfCharacteristicTypesManager"),
        ],
        "ChartOfCalculationTypes" => &[
            (2, "ChartOfCalculationTypesObject"),
            (4, "ChartOfCalculationTypesRef"),
            (6, "ChartOfCalculationTypesSelection"),
            (8, "ChartOfCalculationTypesList"),
            (10, "ChartOfCalculationTypesManager"),
            (12, "DisplacingCalculationTypes"),
            (14, "DisplacingCalculationTypesRow"),
            (16, "BaseCalculationTypes"),
            (18, "BaseCalculationTypesRow"),
            (20, "LeadingCalculationTypes"),
            (22, "LeadingCalculationTypesRow"),
        ],
        "ExchangePlan" => &[(1, "ExchangePlanObject"), (3, "ExchangePlanRef"), (5, "ExchangePlanSelection"), (7, "ExchangePlanList"), (9, "ExchangePlanManager")],
        "BusinessProcess" => &[
            (3, "BusinessProcessObject"),
            (5, "BusinessProcessRef"),
            (7, "BusinessProcessSelection"),
            (9, "BusinessProcessList"),
            (11, "BusinessProcessManager"),
            (13, "BusinessProcessRoutePointRef"),
        ],
        "Task" => &[(3, "TaskObject"), (5, "TaskRef"), (7, "TaskSelection"), (9, "TaskList"), (11, "TaskManager")],
        "DocumentJournal" => &[(1, "DocumentJournalList"), (8, "DocumentJournalManager"), (10, "DocumentJournalSelection")],
        "Constant" => &[(2, "ConstantManager"), (4, "ConstantValueManager"), (13, "ConstantValueKey")],
        "DefinedType" => &[(1, "DefinedType")],
        "FilterCriterion" => &[(1, "FilterCriterionManager"), (3, "FilterCriterionList")],
        "Sequence" => &[(1, "SequenceRecord"), (3, "SequenceManager"), (5, "SequenceRecordSet")],
        "SettingsStorage" => &[(2, "SettingsStorageManager")],
        "WSReference" => &[(3, "WSReferenceManager")],
        _ => &[],
    }
}

impl Names {
    /// Нужен ли индекс загрузки: при совместимости 8.3.16 и ниже модель корня не применяется (`path_names`).
    pub fn needs_index(&self) -> bool {
        !self.old_lists
    }

    /// UseAlways набора констант: константу с признаком `const_inverted` платформа пишет, когда ее нет в записи формы,
    /// и не пишет, когда есть (36 из 36 форм БП 3.0.171; в БП признак у 6 констант). `None` — у какой-то константы
    /// признак не прочитан или в записи не одиночный путь `{1,{0,guid}}`.
    fn constants_use_always(&self, rec: Option<&Node>) -> Option<Node> {
        let mut on: HashSet<&str> = HashSet::new();
        for f in rec.map_or(&[][..], |r| r.list().get(2..).unwrap_or(&[])) {
            match f.list() {
                [k, Node::List(g)] if k.atom() == Some("1") && g.len() == 2 && g[0].atom() == Some("0") => on.insert(g[1].atom()?),
                // Подполе `{-2}` константы без признака (и константы, которой нет в конфигурации) платформа не пишет.
                [k, Node::List(g), Node::List(s)]
                    if k.atom() == Some("2") && g.len() == 2 && g[0].atom() == Some("0") && s.len() == 1 && s[0].atom() == Some("-2")
                        && self.const_inverted.get(g[1].atom()?) != Some(&true) =>
                {
                    continue
                }
                _ => return None,
            };
        }
        if self.const_inverted.is_empty() || on.iter().any(|g| !self.const_inverted.contains_key(*g)) {
            return None;
        }
        // 8.3.27 и старше не обращают признак у константы-хранилища значения (БП 3.0.107: УдалитьНастройкиКолонтитулов —
        // 32 формы; прочие константы с признаком обращают, как 8.5).
        for (g, &inv) in self.const_inverted.iter().filter(|(g, _)| self.older.is_none() || !self.const_storage.contains(*g)) {
            if inv && !on.remove(g.as_str()) {
                on.insert(g);
            }
        }
        let mut v = vec![Node::Atom("0".into()), Node::Atom(on.len().to_string())];
        v.extend(on.iter().map(|g| Node::List(vec![Node::Atom("1".into()), Node::List(vec![Node::Atom("0".into()), Node::Atom(g.to_string())])])));
        Some(Node::List(v))
    }

    pub fn new(cfg: &Config) -> Names {
        let compat = super::classes_tools::compat_mode(cfg);
        let mut n = Names { old_lists: compat.is_some_and(|c| c <= 80316), one_path: compat.is_some_and(|c| c <= 80310), ..Names::default() };
        for g in &cfg.groups {
            let pos = generated_types(g.class);
            for it in &g.items {
                let full = format!("{}.{}", g.class, it.header.name);
                n.children(it, &full);
                let mut f = HashSet::new();
                nested_names(it, &mut f);
                n.fields.insert(it.header.id.clone(), f);
                n.cased.insert(it.header.id.clone(), it.children.iter().map(|c| (c.header.name.to_lowercase(), c.header.name.clone())).collect());
                let own = format!("{}.1c", it.header.id);
                for e in it.files.iter().filter(|(_, r)| *r == "Predefined").map(|(e, _)| e.as_str()).chain([own.as_str()]) {
                    if let Some(d) = cfg.entry_text(e).and_then(|t| brace::Doc::parse(&t).ok()) {
                        d.roots.iter().for_each(|r| super::common::predefined(r, &full, &mut n.predef));
                    }
                }
                n.objects.insert(full.to_lowercase(), it.header.id.clone());
                n.full.insert(it.header.id.clone(), full);
                if pos.is_empty() {
                    continue;
                }
                let Some(node) = cfg.entry_text(&it.header.id).and_then(|t| brace::parse(&t).ok()) else { continue };
                if based_on_pos(g.class).and_then(|i| node.get(1).and_then(|h| h.get(i))).is_some_and(|b| b.list().len() > 2) {
                    n.based_on.insert(it.header.id.clone());
                }
                // Owners справочника — n[1][12] (classes_refs).
                if g.class == "Catalog" && node.at(&[1, 12]).is_some_and(|o| o.list().len() > 2) {
                    n.owned.insert(it.header.id.clone());
                }
                if use_std_commands_pos(g.class).and_then(|i| node.get(1).and_then(|h| h.get(i))).and_then(Node::atom) == Some("0") {
                    n.no_std_commands.insert(it.header.id.clone());
                }
                // Hierarchical (classes_refs): Catalog n[1][37], ChartOfCharacteristicTypes n[1][19].
                let hier = match g.class { "Catalog" => Some(37), "ChartOfCharacteristicTypes" => Some(19), _ => None };
                if hier.and_then(|i| node.get(1).and_then(|h| h.get(i))).and_then(Node::atom) == Some("0") {
                    n.flat.insert(it.header.id.clone());
                    n.no_folders.insert(it.header.id.clone());
                }
                // HierarchyType справочника n[1][36]: 1 — HierarchyOfItems.
                if g.class == "Catalog" && node.get(1).and_then(|h| h.get(36)).and_then(Node::atom) == Some("1") {
                    n.no_folders.insert(it.header.id.clone());
                }
                if g.class == "DefinedType" {
                    if let (Some(id), Some([_, Node::List(x)])) = (node.get(1).and_then(|h| h.get(1)).and_then(Node::atom), node.get(1).and_then(|h| h.get(4)).map(Node::list)) {
                        if let [s, i] = &x[..] {
                            if s.str() == Some("#") {
                                n.defined.insert(id.to_string(), i.atom().unwrap_or("").to_string());
                            }
                        }
                    }
                    let id = node.get(1).and_then(|h| h.get(1)).and_then(Node::atom);
                    if let (Some(id), Some(r)) = (id, node.get(1).and_then(|h| h.get(4)).and_then(pattern_refs)) {
                        if r.is_empty() {
                            n.prim_defined.insert(id.to_string());
                        }
                    }
                }
                if g.class == "Constant" {
                    if let Some(h) = node.get(1).filter(|h| matches!(atom_at(h, 0), "14" | "16")) {
                        n.const_inverted.insert(it.header.id.clone(), atom_at(h, 11) == "1");
                        if node.at(&[1, 1, 1, 2]).is_some_and(|t| t.list().iter().any(|x| atom_at(x, 1) == "e199ca70-93cf-46ce-a54b-6edc88c3a296")) {
                            n.const_storage.insert(it.header.id.clone());
                        }
                    }
                }
                // Posting (classes_refs DOCUMENT): 1 — Deny.
                if g.class == "Document" && node.get(1).and_then(|h| h.get(19)).and_then(Node::atom) == Some("1") {
                    n.no_posting.insert(it.header.id.clone());
                }
                let mut els = HashMap::new();
                super::classes_regs::elements(&node, &mut els);
                for (id, e) in &els {
                    if let (Some(h), Some(r)) = (crate::meta::header(e), e.at(&[0, 1, 1, 2]).and_then(pattern_refs)) {
                        n.field_refs.entry(it.header.id.clone()).or_default().insert(h.name.to_lowercase(), r);
                    }
                    let p = e.at(&[0, 1, 1, 2]).map(Node::list).unwrap_or_default();
                    if let [_, t] = p {
                        if t.get(0).and_then(Node::str) == Some("#") {
                            n.attr_type.insert(id.clone(), atom_at(t, 1).to_string());
                        }
                    }
                }
                if g.class == "InformationRegister" {
                    // InformationRegisterPeriodicity n[1][18] (classes_regs): 0 — Nonperiodical.
                    if !matches!(node.get(1).and_then(|h| h.get(18)).and_then(Node::atom), Some("0") | None) {
                        n.periodic.insert(it.header.id.clone());
                    }
                    let d = it.children.iter().filter(|c| c.kind == "Dimension").map(|c| c.header.name.to_lowercase());
                    n.dims.insert(it.header.id.clone(), d.collect());
                    let m =it.children.iter().filter(|c| c.kind == "Dimension" && els.get(&c.header.id).and_then(|d| d.at(&[0, 2])).and_then(Node::atom) == Some("1"));
                    n.masters.insert(it.header.id.clone(), m.map(|c| c.header.name.clone()).collect());
                }
                // позиции — от первого порождаемого типа: у старых версий заголовка поля перед ним нет (регистр
                // бухгалтерии версии 21 ERP 2.5.22 — без второго номера, у 22 БП он есть)
                let first = node.get(1).map(Node::list).and_then(|h| h.iter().position(|a| a.atom().is_some_and(is_guid)));
                let min = pos.iter().map(|(i, _)| *i).min().unwrap_or(0);
                for &(i, prefix) in pos {
                    let i = first.map_or(i, |f| i + f - min);
                    if let Some(id) = node.get(1).and_then(|h| h.get(i)).and_then(Node::atom).filter(|a| is_guid(a)) {
                        n.types.insert(id.to_string(), format!("{prefix}.{}", it.header.name));
                    }
                }
            }
        }
        if let Some(body) = super::common::external_body(cfg) {
            n.external(cfg, &body);
        }
        n
    }

    /// Внешняя обработка — объект без группы: корень и тело его записи.
    fn external(&mut self, cfg: &Config, body: &Node) {
        let it = &cfg.root;
        let full = format!("{}.{}", it.kind, it.header.name);
        self.children(it, &full);
        let mut f = HashSet::new();
        nested_names(it, &mut f);
        self.fields.insert(it.header.id.clone(), f);
        self.cased.insert(it.header.id.clone(), it.children.iter().map(|c| (c.header.name.to_lowercase(), c.header.name.clone())).collect());
        self.objects.insert(full.to_lowercase(), it.header.id.clone());
        self.full.insert(it.header.id.clone(), full);
        let mut els = HashMap::new();
        super::classes_regs::elements(body, &mut els);
        for (id, e) in &els {
            if let (Some(h), Some(r)) = (crate::meta::header(e), e.at(&[0, 1, 1, 2]).and_then(pattern_refs)) {
                self.field_refs.entry(it.header.id.clone()).or_default().insert(h.name.to_lowercase(), r);
            }
            if let [_, t] = e.at(&[0, 1, 1, 2]).map(Node::list).unwrap_or_default() {
                if t.get(0).and_then(Node::str) == Some("#") {
                    self.attr_type.insert(id.clone(), atom_at(t, 1).to_string());
                }
            }
        }
        if let Some(id) = body.at(&[1, 1]).and_then(Node::atom).filter(|a| is_guid(a)) {
            self.types.insert(id.to_string(), format!("{}Object.{}", it.kind, it.header.name));
        }
    }

    fn children(&mut self, it: &Item, full: &str) {
        for ch in &it.children {
            let f = format!("{full}.{}.{}", ch.kind, ch.header.name);
            self.children(ch, &f);
            self.full.insert(ch.header.id.clone(), f);
        }
    }

    /// Имена, которые обычный путь находит в корне списка со своим запросом `st`, — моделью загрузки (`dlist::path_names`
    /// по `<Settings>` этой же выгрузки); `None` — индекса нет, режим 8.3.16 и ниже (корень там не сверен) или модель
    /// корень не вывела.
    fn path_names(&self, st: &Node) -> Option<Vec<String>> {
        let x = self.index.as_ref().filter(|_| !self.old_lists)?;
        let kv = st.list().get(2..).unwrap_or(&[]);
        if !kv.chunks(2).any(|c| c[0].str() == Some("ManualQuery") && c.get(1).is_some_and(|v| atom_at(v, 1) == "1")) {
            return None;
        }
        let mut w = W { s: String::new(), depth: 0 };
        dynamic_list(&mut w, self, kv);
        let el = crate::xml::imp::dom::parse(&w.s).ok()?;
        crate::xml::imp::form::dlist::path_names(x, &el).ok()
    }

    /// Поле `name` недоступно в динамическом списке `st` (платформа пишет его с `~`, вычисляя по метаданным при выгрузке):
    /// при своем запросе — запрос ссылается на то, чего нет в конфигурации, или обычный путь не находит его в корне
    /// (`root` — `path_names`); корня нет — поля нет среди полей выборки и полей основной таблицы; без своего запроса —
    /// нет в основной таблице.
    /// shortcut: без корня псевдонимы берутся разбором текста запроса без схемы компоновки; на БП 3.0.171 неверно ~2% полей.
    fn unavailable(&self, st: &Node, name: &str, root: Option<&[String]>) -> Option<bool> {
        let kv = st.list().get(2..).unwrap_or(&[]);
        let get = |k: &str| kv.chunks(2).find(|c| c[0].str() == Some(k)).and_then(|c| c.get(1));
        // Поле с точкой (`Владелец.Description`) — по первой части.
        let name = name.split('.').next().unwrap_or(name);
        let main = get("MainTable").map_or("", |v| atom_at(v, 2));
        let manual = get("ManualQuery").map(|v| atom_at(v, 1)) == Some("1");
        // Настройки не динамического списка (тип элементов списка значений...).
        let Some(q) = get("QueryText").map(|v| v.get(1).and_then(Node::str).unwrap_or("")) else { return Some(false) };
        // Картинка по умолчанию — без основной таблицы или у перечисления; от запроса не зависит. В старом режиме — всегда
        // доступна (БП 3.0.107: 47 путей без `~`).
        if name == "DefaultPicture" {
            // и у критерия отбора (ЗадачиПоЭкземпляруБюджета ERP)
            return Some(!self.old_lists && self.full.get(main).is_none_or(|m| m.starts_with("Enum.") || m.starts_with("FilterCriterion.")));
        }
        if (name == "IsFolder" && self.no_folders.contains(main)) || (name == "Parent" && self.flat.contains(main)) {
            return Some(true);
        }
        let lname = name.to_lowercase();
        // Ресурс и реквизит регистра бухгалтерии — с признаком Dr/Cr (`СуммаDr`).
        let base = lname.strip_suffix("dr").or_else(|| lname.strip_suffix("cr")).unwrap_or(&lname);
        let has_main = self.full.contains_key(main);
        let main_class = self.full.get(main).map_or("", |f| f.split('.').next().unwrap_or(""));
        // shortcut: состав общего реквизита не проверяется — считается, что он есть у всех объектов.
        let in_main = self.fields.get(main).is_some_and(|f| f.contains(&lname) || f.contains(base))
            || self.objects.contains_key(&format!("commonattribute.{lname}"))
            || (has_main && STD_LIST_FIELDS.iter().any(|(e, r)| std_ru_is(r, &lname) && std_applies(main_class, e)))
            // стандартное поле регистра бухгалтерии с признаком Dr/Cr (`AccountDr`: Международный.ФормаСписка ERP)
            || (main_class == "AccountingRegister" && {
                let n = lname.trim_end_matches(|c: char| c.is_ascii_digit());
                matches!(n.strip_suffix("dr").or_else(|| n.strip_suffix("cr")), Some("account" | "extdimension" | "extdimensiontype"))
            });
        if manual {
            if self.query_broken(q) {
                return Some(true);
            }
            // Путь к полю, которого нет в корне по имени и второму имени, недоступен: так его разбирает и загрузка (mngbase
            // FUN_18075f8c0 сначала ищет путь `~` как обычный, FUN_1802cbc20). Совпадение только без учета регистра — по
            // тексту запроса ниже (сравнение имен дерева не выведено).
            if let Some(r) = root {
                if r.iter().any(|n| n == name) {
                    return Some(false);
                }
                if !r.iter().any(|n| n.to_lowercase() == lname) {
                    return Some(true);
                }
            }
            // Старый режим: доступно только поле выборки — по имени или вторичному имени FieldsMap (`Т.Ссылка` дает Ref со
            // вторичным Ссылка); стандартных полей основной таблицы вне выборки нет (БП 3.0.107 Контрагенты: Ref без
            // вторичного имени при `Т.Ссылка КАК Ссылка` — `~Список.Ref`).
            if self.old_lists {
                let sel = query_fields(q);
                let second = (0..)
                    .map_while(|k| get(&format!("FieldsMapItemName{k}")).map(|v| (k, v)))
                    .find(|(_, v)| v.get(1).and_then(Node::str).is_some_and(|n| n.eq_ignore_ascii_case(name)))
                    .and_then(|(k, _)| get(&format!("FieldsMapItemSecondaryName{k}")))
                    .and_then(|v| v.get(1).and_then(Node::str))
                    .map(str::to_lowercase);
                let dr = lname.strip_suffix("dr").map(|b| format!("{b}дт")).or_else(|| lname.strip_suffix("cr").map(|b| format!("{b}кт")));
                // Стандартное поле без псевдонима (`Т.ПометкаУдаления`) — под латинским именем (ПрослеживаемостьУведомления:
                // `Items.Список.CurrentData.DeletionMark` без `~`).
                let ru = STD_LIST_FIELDS.iter().find(|(e, r)| *e == name && !r.is_empty()).map(|(_, r)| r.to_lowercase());
                let unaliased = ru.is_some_and(|r| select_words(q, true).is_some_and(|items| items.iter().any(|w| w.len() == 1 && w[0].rsplit('.').next().is_some_and(|l| l.to_lowercase() == r))));
                if unaliased || [Some(lname.clone()), second, dr].into_iter().flatten().any(|n| sel.contains(&n)) || self.star_field(q, &lname) {
                    return Some(false);
                }
            }
            // Стандартные поля (латиницей: Ref, Code...) и измерения регистра сведений (ключ записи) у основной таблицы
            // есть всегда, прочие ее реквизиты — только из выборки. Без основной таблицы стандартное поле — тоже из
            // выборки (`Т.Ссылка КАК Ссылка` дает и Ref).
            let ru = STD_LIST_FIELDS.iter().find(|(e, _)| *e == name).map(|(_, r)| r.to_lowercase());
            let sel = query_fields(q);
            let dr = lname.strip_suffix("dr").map(|b| format!("{b}дт")).or_else(|| lname.strip_suffix("cr").map(|b| format!("{b}кт")));
            let key = self.dims.get(main).is_some_and(|d| d.contains(&lname)) || self.objects.contains_key(&format!("commonattribute.{lname}"));
            // Стандартное поле без основной таблицы по русскому имени — только если оно не НЕОПРЕДЕЛЕНО (УправлениеОбменом).
            let ru = ru.filter(|r| !sel.contains(&format!("=неопределено:{r}")));
            // Запрос, начинающийся с комментария, латинских стандартных полей не дает (ПодборНоменклатуры: `~Список.Code`,
            // 1 из 1; русские из выборки — есть).
            let commented = q.trim_start().starts_with("//");
            let latin = !commented && !std_absent(main_class, name) && (!STD_LIST_FIELDS.iter().any(|(e, _)| *e == name) || std_applies(main_class, name));
            let ru = ru.filter(|_| !commented);
            let has_ru = ru.is_some();
            if !self.old_lists && ((name.is_ascii() && has_main && latin) || key || sel.contains(&lname) || dr.is_some_and(|d| sel.contains(&d)) || ru.is_some_and(|r| sel.contains(&r)) || self.star_field(q, &lname)) {
                return Some(false);
            }
            // Стандартное поле основной таблицы по русскому имени вне выборки: выбрано под другим псевдонимом
            // (`Т.Ссылка КАК ЭлектронныйДокумент`, ТекущиеДелаПоЭДО: `~ВходящиеЭД.Ссылка`, `~ИсходящиеЭД.Ссылка`) — недоступно,
            // не выбрано вовсе (ЗаменаЭлементов: `Список.Ссылка`, `Список.ПометкаУдаления`) — доступно; поля нет среди
            // стандартных реквизитов класса (ПометкаУдаления у регистра сведений, РегламентированнаяОтчетность каркаса) — недоступно.
            if let Some((e, r)) = STD_LIST_FIELDS.iter().find(|(e, r)| !self.old_lists && has_main && std_ru_is(r, &lname) && std_applies(main_class, e)) {
                let in_md = super::standard::NAMES.iter().find(|(c, _)| *c == main_class).is_none_or(|(_, l)| l.iter().any(|(_, n)| n == e));
                if !in_md {
                    return Some(true);
                }
                let r = r.to_lowercase();
                let as_other = |w: &Vec<String>| match &w[..] {
                    [p, k, a] => matches!(k.to_lowercase().as_str(), "как" | "as") && p.rsplit('.').next().is_some_and(|l| l.to_lowercase() == r) && a.to_lowercase() != r,
                    _ => false,
                };
                return select_words(q, true).map(|items| items.iter().any(as_other));
            }
            // Стандартное поле без основной таблицы вне выборки — недоступно, если источник — таблица конфигурации;
            // незнакомый класс — отказ. НЕОПРЕДЕЛЕНО КАК Ссылка и запрос с комментарием — свои правила выше.
            if !has_main && !commented && has_ru && STD_LIST_FIELDS.iter().any(|(e, _)| *e == name) {
                return first_source(q).map(|_| true);
            }
        } else {
            // Виртуальная таблица (остатки...) — свои поля, по метаданным не проверяются.
            if in_main || get("MainTableCategory").and_then(Node::atom).is_some_and(|c| c != "0" && c != "1") {
                return Some(false);
            }
            // shortcut: применимость прочих стандартных полей (Owner у неподчиненного...) не проверяется — в каркасе таких нет.
            if STD_LIST_FIELDS.iter().any(|(e, _)| *e == name && std_applies(main_class, e)) {
                return Some(false);
            }
        }
        // Вычисляемое поле списка — в схеме настроек (ServerState).
        let schema = get("ServerState").and_then(|v| v.get(1)).and_then(Node::str).and_then(|b| crate::xml::base64(b).and_then(|b| dcs::server_state(&b)).and_then(|x| dcs::parse(&x)).ok());
        // Поля набора (`Field`) — описания полей запроса, сохраненные с формой; с метаданными не сверяются.
        Some(!schema.is_some_and(|st| st.elements().filter(|e| e.local() == "ExpressionField").any(|e| e.elements().any(|d| d.local() == "dataPath" && d.kids.iter().any(|k| matches!(k, dcs::Kid::Text(t) if t.trim().eq_ignore_ascii_case(name) || t.trim().to_lowercase() == lname))))))
    }

    /// Путь `path` к полю `name` списка `st` с признаком недоступности: `~путь`; у записи FieldsMap со вторым именем —
    /// `~путь~путь ко второму имени` (`~Список.Ref~Список.Ссылка`); `ru_col` — есть ли у списка колонка с таким именем,
    /// `root` — имена корня (`path_names`).
    fn mark(&self, st: &Node, name: &str, path: String, ru_col: &dyn Fn(&str) -> bool, root: Option<&[String]>) -> String {
        match self.unavailable(st, name, root) {
            Some(false) => return path,
            // Без основной таблицы, но со своей колонкой русского имени (`Регистратор` рядом с `Recorder`) — один путь `~`
            // (3 формы каркаса).
            None if STD_LIST_FIELDS.iter().any(|(e, r)| *e == name && !r.is_empty() && ru_col(r)) => {}
            None => return format!("\u{1}доступность поля {path} (стандартное по русскому имени вне выборки) не выведена"),
            Some(true) => {}
        }
        // Второй путь — к второму имени записи FieldsMap (`~Список.Ref~Список.Ссылка`): обратное загрузке, где путь `~А~Б`
        // дает запись (А, второе имя Б) — mngbase FUN_1802c8a30 (+0x60, FUN_1805f18b0); второго имени нет или оно то же
        // без учета регистра (FUN_1805ce200) — путь один (записи ERP вторых имен не хранят — ни одного второго пути).
        // shortcut: писатель пути `~` (обратный FUN_18075f8c0) в mngbase не найден, правило — обращение чтения, сверено на
        // четырех корпусах (два вторых пути БП); пересмотреть, если встретится второй путь без второго имени записи
        let kv = st.list().get(2..).unwrap_or(&[]);
        let get = |k: &str| kv.chunks(2).find(|c| c[0].str() == Some(k)).and_then(|c| c.get(1)?.get(1)?.str());
        let lname = name.to_lowercase();
        let second = (0..)
            .map_while(|k| get(&format!("FieldsMapItemName{k}")).map(|n| (k, n)))
            .find(|(_, n)| n.to_lowercase() == lname)
            .and_then(|(k, _)| get(&format!("FieldsMapItemSecondaryName{k}")))
            .filter(|s| s.to_lowercase() != lname);
        match (second, path.strip_suffix(name)) {
            (None, _) => format!("~{path}"),
            (Some(_), _) if self.one_path => format!("\u{1}второй путь недоступного поля {path} при совместимости 8.3.10 и ниже не выведен"),
            (Some(s), Some(head)) => format!("~{path}~{head}{s}"),
            (Some(_), None) => format!("\u{1}второй путь недоступного поля {path} (второе имя {name} в FieldsMap) не выведен"),
        }
    }

    /// Запрос ссылается на таблицу или значение перечисления, которых нет в конфигурации: платформа не строит
    /// его схему, и все поля списка недоступны.
    /// Поле `Т.Поле` по псевдониму таблицы — тоже: псевдоним, связанный только с таблицами конфигурации, а поля нет
    /// ни у одной из них.
    fn query_broken(&self, q: &str) -> bool {
        let t = query_clean(q);
        let (missing, words, bound) = self.query_scan(&t);
        if missing {
            return true;
        }
        words.iter().any(|w| {
            let mut p = w.split('.');
            // `Т.*` — все поля таблицы, не поле.
            let (Some(a), Some(f)) = (p.next(), p.next().filter(|f| !f.is_empty())) else { return false };
            let Some(Some(srcs)) = bound.get(a) else { return false };
            // подполе у поля только примитивных типов (`Т.Строка.Ссылка`, НастройкиОбменаСУЗ Клиента ЭДО) — поля нет
            let prim = |id: &String| self.field_refs.get(id).and_then(|m| m.get(f)).is_some_and(|r| r.iter().all(|g| self.prim_defined.contains(g)));
            let sub = p.next().is_some_and(|g| !g.is_empty());
            !srcs.is_empty() && srcs.iter().all(|&(id, virt)| !self.table_field(id, virt, f) || (sub && !virt && prim(id)))
        })
    }

    /// Поле `lname` выбрано звездочкой `Т.*` по псевдониму таблицы конфигурации, у которой оно есть.
    /// shortcut: звездочка ищется во всем тексте, а не только в выборке первого запроса пакета.
    fn star_field(&self, q: &str, lname: &str) -> bool {
        let t = query_clean(q);
        let (_, words, bound) = self.query_scan(&t);
        words.iter().filter(|w| w.ends_with('.') && t.contains(&format!("{w}*"))).any(|w| {
            matches!(bound.get(w.trim_end_matches('.')), Some(Some(srcs)) if srcs.iter().any(|&(id, virt)| self.table_field(id, virt, lname)))
        })
    }

    /// Разбор очищенного запроса `t`: (ссылается на то, чего нет в конфигурации; слова; псевдоним → таблицы
    /// (GUID объекта, виртуальная ли), `None` — связан и с чем-то иным).
    #[allow(clippy::type_complexity)]
    fn query_scan<'s, 't>(&'s self, t: &'t str) -> (bool, Vec<&'t str>, HashMap<&'t str, Option<Vec<(&'s String, bool)>>>) {
        // Слова и скобки (скобки — чтобы пропустить параметры виртуальной таблицы перед КАК).
        let mut tokens: Vec<&str> = Vec::new();
        let mut start = 0;
        for (i, c) in t.char_indices().chain(std::iter::once((t.len(), ' '))) {
            if c.is_alphanumeric() || c == '_' || c == '.' || c == '&' {
                continue;
            }
            tokens.extend([&t[start..i]].into_iter().filter(|w| !w.is_empty()));
            if c == '(' || c == ')' {
                tokens.push(&t[i..i + 1]);
            }
            start = i + c.len_utf8();
        }
        let words: Vec<&str> = tokens.iter().copied().filter(|w| *w != "(" && *w != ")").collect();
        let aliases: HashSet<&str> = words.windows(2).filter(|w| w[0] == "как" || w[0] == "as").map(|w| w[1]).collect();
        // Таблица `класс.имя[.часть]`: (класс, GUID объекта, если он есть, третья часть).
        // `any` — и когда первая часть совпадает с псевдонимом (источник `РегистрСведений.Х КАК РегистрСведений`).
        let table = |w: &str, any: bool| -> Option<(&'static str, Option<&String>, Option<String>)> {
            let p: Vec<&str> = w.split('.').collect();
            let class = p.first().filter(|c| any || !aliases.contains(*c)).and_then(|c| query_class(c))?;
            let name = p.get(1).filter(|n| !n.is_empty())?;
            Some((class, self.objects.get(&format!("{class}.{name}")), p.get(2).map(|s| s.to_string())))
        };
        let missing = words.iter().filter(|w| !w.starts_with('&')).any(|w| {
            let Some((class, id, v)) = table(w, false) else { return false };
            let Some(id) = id else { return true };
            class == "enum" && v.is_some_and(|v| !matches!(v.as_str(), "пустаяссылка" | "emptyref") && !self.fields.get(id).is_some_and(|f| f.contains(&v)))
        });
        // Псевдоним → таблицы (GUID объекта, виртуальная ли); `None` — связан и с чем-то иным (подзапрос, временная таблица).
        let mut bound: HashMap<&str, Option<Vec<(&String, bool)>>> = HashMap::new();
        // Источники, первая часть которых — псевдоним: это не поля.
        let mut named: HashSet<&str> = HashSet::new();
        let mut i = 0;
        while i < tokens.len() {
            let tab = |any| table(tokens[i], any).and_then(|(class, id, v)| Some((id?, class != "enum" && v.is_some())));
            let mut src = tab(false);
            if src.is_none() && matches!(tokens.get(i + 1), Some(&"как" | &"as")) {
                src = tab(true);
                if src.is_some() {
                    named.insert(tokens[i]);
                }
            }
            let mut j = i + 1;
            if src.is_some() && tokens.get(j) == Some(&"(") {
                let mut depth = 0;
                while j < tokens.len() {
                    depth += match tokens[j] { "(" => 1, ")" => -1, _ => 0 };
                    j += 1;
                    if depth == 0 {
                        break;
                    }
                }
            }
            if matches!(tokens.get(j), Some(&"как" | &"as")) {
                if let Some(a) = tokens.get(j + 1) {
                    let e = bound.entry(a).or_insert(Some(Vec::new()));
                    match (src, e.as_mut()) {
                        (Some(s), Some(v)) => v.push(s),
                        _ => *e = None,
                    }
                }
            }
            i += 1;
        }
        (missing, words.into_iter().filter(|w| !named.contains(w)).collect(), bound)
    }

    /// У таблицы объекта `id` (виртуальной — `virt`) есть поле `f` (в нижнем регистре).
    fn table_field(&self, id: &str, virt: bool, f: &str) -> bool {
        // Сверх полей списка: поля таблиц констант, регистров и их виртуальных таблиц (Субконто: Вид, Значение).
        // таблица изменений (`РегистрСведений.Х.Изменения`): Узел, НомерСообщения (ОбменСПодключаемымОборудованиемOffline ERP)
        const STD: &[&str] = &["представление", "presentation", "виддвижения", "recordtype", "уточнениепериода", "periodadjustment", "значение", "value", "узел", "node", "номерсообщения", "messageno", "видысубконто", "extdimensiontypes"];
        let known = |f: &str| {
            self.fields.get(id).is_some_and(|s| s.contains(f))
                || STD.contains(&f)
                || STD_LIST_FIELDS.iter().any(|(e, r)| (std_ru_is(r, f) || e.eq_ignore_ascii_case(f)) && (virt || std_applies(self.full.get(id).map_or("", |c| c.split('.').next().unwrap_or("")), e)))
                || self.objects.contains_key(&format!("commonattribute.{f}"))
        };
        if known(f) {
            return true;
        }
        if !virt {
            return false;
        }
        // Поля виртуальных таблиц: периоды и ресурсы с суффиксами (`СуммаОстаток`, `КоличествоОборотДт`).
        // shortcut: состав полей по виду виртуальной таблицы не различается — суффиксы любые из списка.
        const PERIODS: &[&str] = &["секунда", "минута", "час", "день", "неделя", "декада", "месяц", "квартал", "полугодие", "год", "регистратор", "моментвремени"];
        const SUFFIXES: &[&str] = &[
            "начальныйостатокдт", "начальныйостатоккт", "конечныйостатокдт", "конечныйостатоккт", "начальныйразвернутыйостатокдт",
            "начальныйразвернутыйостатоккт", "конечныйразвернутыйостатокдт", "конечныйразвернутыйостатоккт", "начальныйостаток",
            "конечныйостаток", "остатокдт", "остатоккт", "развернутыйостатокдт", "развернутыйостатоккт", "остаток", "оборотдт", "обороткт",
            "оборот", "приход", "расход", "дт", "кт",
        ];
        // Счет и субконто регистра бухгалтерии: `СчетДт`, `СубконтоКт2`, `ВидСубконтоДт1`.
        let f0 = f.trim_end_matches(|c: char| c.is_ascii_digit());
        let f0 = f0.strip_suffix("дт").or_else(|| f0.strip_suffix("кт")).unwrap_or(f0);
        if PERIODS.contains(&f) || ["счет", "субконто", "видсубконто"].contains(&f0) {
            return true;
        }
        SUFFIXES.iter().any(|s| f.strip_suffix(s).is_some_and(|b| !b.is_empty() && known(b)))
    }

    /// Последняя часть полного имени — имя самого элемента.
    fn short(&self, id: &str) -> Option<&str> {
        self.full.get(id).map(|f| f.rsplit('.').next().unwrap_or(f))
    }
}

/// Текст XML с отступами табуляцией и CRLF, как у платформы.
struct W {
    s: String,
    depth: usize,
}

/// Переводы строк внутри текста платформа пишет как LF.
fn esc(s: &str) -> String {
    s.replace("\r\n", "\n").replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

fn esc_attr(s: &str) -> String {
    esc(s).replace('"', "&quot;")
}

impl W {
    fn line(&mut self, t: &str) {
        self.s.push_str("\r\n");
        for _ in 0..self.depth {
            self.s.push('\t');
        }
        self.s.push_str(t);
    }
    fn open(&mut self, tag: &str) {
        self.line(&format!("<{tag}>"));
        self.depth += 1;
    }
    fn close(&mut self, tag: &str) {
        self.depth -= 1;
        self.line(&format!("</{tag}>"));
    }
    fn leaf(&mut self, tag: &str, text: &str) {
        if text.is_empty() {
            self.line(&format!("<{tag}/>"));
        } else {
            self.line(&format!("<{tag}>{}</{tag}>", esc(text)));
        }
    }
}

/// Свойства элемента: копятся в любом порядке, пишутся в порядке платформы.
struct P {
    depth: usize,
    items: Vec<(&'static str, String)>,
    /// Дополнительные атрибуты открывающего тега (DisplayImportance).
    head: String,
}

impl P {
    fn new(depth: usize) -> P {
        P { depth, items: Vec::new(), head: String::new() }
    }
    fn w(&self) -> W {
        W { s: String::new(), depth: self.depth }
    }
    fn push(&mut self, name: &'static str, w: W) {
        if !w.s.is_empty() {
            self.items.push((name, w.s));
        }
    }
    fn leaf(&mut self, name: &'static str, text: &str) {
        let mut w = self.w();
        w.leaf(name, text);
        self.push(name, w);
    }
    fn local(&mut self, name: &'static str, n: &Node) {
        let mut w = self.w();
        local(&mut w, name, n);
        self.push(name, w);
    }
    /// Незнакомые свойства — в конце, с пометкой (у меток отказа `\u{1}…` своя пометка, порядок у них не вопрос).
    fn emit(mut self, w: &mut W, order: &[&str]) {
        self.items.sort_by_key(|(n, _)| order.iter().position(|o| o == n).unwrap_or(usize::MAX));
        for (n, s) in self.items {
            if !order.contains(&n) && !n.starts_with('\u{1}') {
                w.s.push_str(&format!("<!--\u{1}порядок {n}-->"));
            }
            w.s.push_str(&s);
        }
    }
}

/// Порядок записи свойств формы — mngbase.dll FUN_180728d50 (номера модели XDTO logform:Form), затем свойства вида
/// формы FUN_180748890 по ключу словаря 0…7, 0x14, 0x15, 0x17, 0x1b, 0x1d (см. порядок элементов ниже, у `COMMAND_BAR_ORDER`).
const FORM_ORDER: &[&str] = &[
    "Title", "Width", "Height", "WindowOpeningMode", "EnterKeyBehavior", "AutoSaveDataInSettings", "SaveDataInSettings", "SaveWindowSettings",
    "SettingsStorage", "AutoTitle", "AutoURL", "Group", "ChildrenAlign", "HorizontalSpacing", "VerticalSpacing", "HorizontalAlign",
    "VerticalAlign", "ChildItemsWidth", "ChildItemsTitleLocation", "AutoFillCheck", "Customizable", "Enabled", "ReadOnly", "CommandBarLocation",
    "VerticalScroll", "ScalingMode", "Scale", "ScaleVariant", "ConversationsRepresentation", "MobileDeviceCommandBarContent", "CommandSet",
    "ShowTitle", "ShowCloseButton", "CollapseItemsByImportanceVariant", "CreateButtonsGroupTitle", "CreateButtonsGroupPicture", "WindowViewMode",
    "ShowCommandBar", "UseForFoldersAndItems", "GroupList", "AutoTime", "UsePostingMode", "RepostOnWrite", "ReportResult", "DetailsData",
    "ReportFormType", "VariantAppearance", "AutoShowState", "CustomSettingsFolder", "ReportResultViewMode", "ViewModeApplicationOnSetReportResult",
];

/// Курсор по списку скобочной записи.
struct R<'a> {
    v: &'a [Node],
    i: usize,
    what: &'static str,
}

impl<'a> R<'a> {
    fn at(n: &'a Node, i: usize, what: &'static str) -> R<'a> {
        R { v: n.list(), i, what }
    }
    fn err(&self, m: &str) -> String {
        format!("{}[{}]: {m}", self.what, self.i)
    }
    fn node(&mut self) -> Result<&'a Node, String> {
        let n = self.v.get(self.i).ok_or_else(|| self.err("нет элемента"))?;
        self.i += 1;
        Ok(n)
    }
    fn atom(&mut self) -> Result<&'a str, String> {
        match self.node()? {
            Node::Atom(a) => Ok(a),
            _ => Err(format!("{}[{}]: ожидался атом", self.what, self.i - 1)),
        }
    }
    fn num(&mut self) -> Result<i64, String> {
        let a = self.atom()?;
        a.parse().map_err(|_| format!("{}[{}]: не число {a:?}", self.what, self.i - 1))
    }
    fn str(&mut self) -> Result<&'a str, String> {
        match self.node()? {
            Node::Str(s) => Ok(s),
            _ => Err(format!("{}[{}]: ожидалась строка", self.what, self.i - 1)),
        }
    }
}

fn atom_at(n: &Node, i: usize) -> &str {
    n.get(i).and_then(Node::atom).unwrap_or("")
}

fn num_at(n: &Node, i: usize) -> i64 {
    atom_at(n, i).parse().unwrap_or(i64::MIN)
}

/// Незнакомое значение — видимой меткой в выводе.
fn unk(what: &str, n: &Node) -> String {
    format!("\u{1}{what}:{}", brace::write(n).unwrap_or_default().replace("\r\n", ""))
}

fn bool_text(b: bool) -> &'static str {
    if b {
        "true"
    } else {
        "false"
    }
}

const HEAD: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\r\n<Form xmlns=\"http://v8.1c.ru/8.3/xcf/logform\" xmlns:app=\"http://v8.1c.ru/8.2/managed-application/core\" xmlns:cfg=\"http://v8.1c.ru/8.1/data/enterprise/current-config\" xmlns:dcscor=\"http://v8.1c.ru/8.1/data-composition-system/core\" xmlns:dcssch=\"http://v8.1c.ru/8.1/data-composition-system/schema\" xmlns:dcsset=\"http://v8.1c.ru/8.1/data-composition-system/settings\" xmlns:ent=\"http://v8.1c.ru/8.1/data/enterprise\" xmlns:lf=\"http://v8.1c.ru/8.2/managed-application/logform\" xmlns:pal=\"http://v8.1c.ru/8.1/data/ui/colors/palette\" xmlns:style=\"http://v8.1c.ru/8.1/data/ui/style\" xmlns:sys=\"http://v8.1c.ru/8.1/data/ui/fonts/system\" xmlns:v8=\"http://v8.1c.ru/8.1/data/core\" xmlns:v8ui=\"http://v8.1c.ru/8.1/data/ui\" xmlns:web=\"http://v8.1c.ru/8.1/data/ui/colors/web\" xmlns:win=\"http://v8.1c.ru/8.1/data/ui/colors/windows\" xmlns:xr=\"http://v8.1c.ru/8.3/xcf/readable\" xmlns:xs=\"http://www.w3.org/2001/XMLSchema\" xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\" version=\"2.21\">";


/// Реквизиты и элементы управляемой формы для подсказки — из ее же Form.xml.
/// Реквизит управляемой формы: имя, типы (`DocumentObject.Реализация`, `v8:ValueTable`), основной ли, колонки.
#[derive(Clone, Debug, Default)]
pub struct FormAttr {
    pub name: String,
    pub types: Vec<String>,
    pub main: bool,
    pub columns: Vec<(String, Vec<String>)>,
}

/// Описание формы для подсказки в ее модуле: реквизиты и элементы (имя, вид элемента в Form.xml: `InputField`…).
#[derive(Clone, Debug, Default)]
pub struct FormInfo {
    pub attributes: Vec<FormAttr>,
    pub items: Vec<(String, String)>,
}

pub fn outline(names: &Names, text: &str) -> Option<FormInfo> {
    use crate::xml::imp::dom::{self, El};
    let xml = String::from_utf8(form_xml(names, text).ok()??).ok()?;
    let root = dom::parse(&xml).ok()?;
    let types = |e: &El| -> Vec<String> {
        e.peek_child("Type").map(|t| t.peek_elems().filter_map(|v| v.peek_text()).map(|v| v.trim_start_matches("cfg:").to_string()).collect()).unwrap_or_default()
    };
    let mut info = FormInfo::default();
    for a in root.peek_child("Attributes").into_iter().flat_map(|a| a.peek_elems()).filter(|a| a.name == "Attribute") {
        let columns = a.peek_child("Columns").into_iter().flat_map(|c| c.peek_elems()).filter_map(|c| Some((c.peek_attr("name")?.to_string(), types(c)))).collect();
        info.attributes.push(FormAttr {
            name: a.peek_attr("name").unwrap_or_default().to_string(),
            types: types(a),
            main: a.peek_child("MainAttribute").and_then(El::peek_text).as_deref() == Some("true"),
            columns,
        });
    }
    // элементы — все с именем и номером под ChildItems и в командной панели формы, со вложенными подсказками и меню
    fn walk(e: &El, out: &mut Vec<(String, String)>) {
        for k in e.peek_elems() {
            if let (Some(n), Some(_)) = (k.peek_attr("name"), k.peek_attr("id")) {
                out.push((n.to_string(), k.name.clone()));
            }
            walk(k, out);
        }
    }
    for part in ["AutoCommandBar", "ChildItems"] {
        if let Some(e) = root.peek_child(part) {
            if let Some(n) = e.peek_attr("name") {
                info.items.push((n.to_string(), e.name.clone()));
            }
            walk(e, &mut info.items);
        }
    }
    Some(info)
}

/// Ext/Form.xml по тексту записи формы. `None` — запись не управляемая форма.
pub fn form_xml(names: &Names, text: &str) -> Result<Option<Vec<u8>>, String> {
    let mut doc = brace::Doc::parse(text)?;
    let v59 = doc.roots.first().is_some_and(super::form_v59::is_v59);
    let mut v59_flags = HashMap::new();
    if let Some(r) = doc.roots.first_mut().filter(|r| super::form_v59::is_v59(r)) {
        v59_flags = super::form_v59::downgrade(r)?;
    }
    let root = doc.roots.first().ok_or("пустая запись формы")?;
    if !matches!(root.get(1), Some(Node::List(_))) {
        return Ok(None);
    }
    let mut items = HashMap::new();
    collect_items(root.get(1).unwrap(), &mut items);
    let mut commands = HashMap::new();
    if let Some(c) = root.get(5) {
        for cmd in c.list().iter().skip(2) {
            if let (Some(id), Some(name)) = (cmd.get(1).and_then(|i| i.get(0)).and_then(Node::atom), cmd.get(2).and_then(Node::str)) {
                commands.insert(id, name);
            }
        }
    }
    // Поля своего запроса списков за наибольшим номером FieldsMap (`list_extension`): (список, номер, имя) и
    // список → (наибольший номер FieldsMap, последний номер с полями запроса).
    let mut synth = Vec::new();
    let mut extent = HashMap::new();
    if let Some(a) = root.get(3) {
        for at in a.list().iter().skip(2).take(num_at(a, 1).max(0) as usize) {
            let id = at.get(1).map_or("", |i| atom_at(i, 0));
            let Some((max, ext)) = at.get(14 + num_at(at, 13).max(0) as usize).and_then(|st| list_extension(names, st)) else { continue };
            extent.insert(id, (max, max + ext.len() as u32));
            synth.extend(ext.into_iter().enumerate().filter_map(|(k, n)| Some((id, (max + 1 + k as u32).to_string(), n?))));
        }
    }
    let mut attrs = HashMap::new();
    let mut cols = HashMap::new();
    let mut col_class = HashMap::new();
    let mut owners = HashMap::new();
    let mut prim = HashSet::new();
    let mut lists = HashMap::new();
    if let Some(a) = root.get(3) {
        for at in a.list().iter().skip(2).take(num_at(a, 1).max(0) as usize) {
            let id = at.get(1).map_or("", |i| atom_at(i, 0));
            let name = at.get(3).and_then(Node::str).unwrap_or("").to_string();
            let class_of = |t: Option<&Node>| {
                t.and_then(|t| t.list().iter().find_map(|x| (x.get(0).and_then(Node::str) == Some("#")).then(|| atom_at(x, 1))))
                    // Определяемый тип из одного типа — класс этого типа (МастерНастройкиВнутреннегоЭДО: `Организация.Ref`).
                    .map(|g| names.defined.get(g).map_or(g, String::as_str))
                    .map(|g| match (platform_type(g), names.types.get(g)) {
                        _ if g == DYNAMIC_LIST => "DynamicList".to_string(),
                        (_, Some(t)) => type_class(t).to_string(),
                        (Some(p), _) => p.to_string(),
                        _ => String::new(),
                    })
                    .unwrap_or_default()
            };
            let class = class_of(at.get(5));
            let tname = at
                .get(5)
                .and_then(|t| t.list().iter().find_map(|x| (x.get(0).and_then(Node::str) == Some("#")).then(|| atom_at(x, 1))))
                .and_then(|g| names.types.get(g).map(|t| t.split('.').next().unwrap_or("").to_string()).or_else(|| (g == DYNAMIC_LIST).then(|| "DynamicList".to_string())))
                .unwrap_or_default();
            if let Some(t) = at.get(5).and_then(|t| t.list().iter().find_map(|x| (x.get(0).and_then(Node::str) == Some("#")).then(|| atom_at(x, 1)))).and_then(|g| names.types.get(g)) {
                owners.insert(id, format!("{}.{}", type_class(t), t.split_once('.').map_or("", |x| x.1)));
            }
            if at.get(5).is_some_and(|p| p.list().len() > 1 && pattern_refs(p).is_some_and(|r| r.is_empty())) {
                prim.insert(id);
            }
            attrs.insert(id, (name, class, tname));
            let ncol = num_at(at, 13).max(0) as usize;
            for col in at.list().iter().skip(14).take(ncol) {
                if let (Some(c), Some(n)) = (col.get(1).and_then(Node::atom), col.get(3).and_then(Node::str)) {
                    cols.insert((id, c), n);
                    col_class.insert((id, c), class_of(col.get(5)));
                }
            }
            // Поля динамического списка: пары FieldsMapItemIdK → FieldsMapItemNameK в настройках.
            if let Some(st) = at.get(14 + ncol) {
                lists.insert(id, st);
                let kv = st.list().get(2..).unwrap_or(&[]);
                let get = |k: &str| kv.chunks(2).find(|c| c[0].str() == Some(k)).and_then(|c| c.get(1));
                // Без своего запроса имя поля основной таблицы платформа пишет как в описании, а не как в FieldsMap
                // (ПрочиеДоходыИРасходы: `ПринятиеКналоговомуУчету` → `ПринятиеКНалоговомуУчету`).
                let main = get("MainTable").map_or("", |v| atom_at(v, 2));
                let auto = get("ManualQuery").map(|v| atom_at(v, 1)) == Some("0");
                for k in 0.. {
                    let Some(i) = get(&format!("FieldsMapItemId{k}")).and_then(|v| v.get(1)).and_then(Node::atom) else { break };
                    if let Some(n) = get(&format!("FieldsMapItemName{k}")).and_then(|v| v.get(1)).and_then(Node::str) {
                        let n = names.cased.get(main).filter(|_| auto).and_then(|m| m.get(&n.to_lowercase())).map_or(n, String::as_str);
                        cols.insert((id, i), n);
                    }
                }
            }
        }
    }
    for (id, i, n) in &synth {
        cols.entry((*id, i.as_str())).or_insert(n.as_str());
    }
    // Дополнительные колонки табличных частей `{0,{2,{реквизит},{0,guid ТЧ}},K,колонка...}` в хвосте реквизитов.
    if let Some(a) = root.get(3) {
        let tail = a.list().get(2 + num_at(a, 1).max(0) as usize..).unwrap_or(&[]);
        let extra = tail.first().and_then(Node::atom).and_then(|x| x.parse::<usize>().ok()).unwrap_or(0);
        for e in tail.iter().skip(1).take(extra) {
            let Some(path) = e.get(1) else { continue };
            let ts = path.get(num_at(path, 0).max(0) as usize).map_or("", |x| atom_at(x, 1));
            for c in e.list().iter().skip(3).take(num_at(e, 2).max(0) as usize) {
                if let (Some(i), Some(n)) = (c.get(1).and_then(Node::atom), c.get(3).and_then(Node::str)) {
                    cols.insert((ts, i), n);
                }
            }
        }
    }
    let mut tables = HashMap::new();
    let mut table_nodes = HashMap::new();
    let mut plain = HashSet::new();
    collect_tables(root.get(1).unwrap(), &mut tables, &mut table_nodes, &mut plain);
    let mut generated = Vec::new();
    if root.get(1).is_some_and(|f| num_at(f, 0) != 28) {
        generated_items(root.get(1).unwrap(), None, &mut generated);
    }
    // Запись 8.5 из загрузки XML: элементы, порожденные загрузкой (панели выделенных строк), — с id `{0}`.
    let mut zero_items = Vec::new();
    if v59 {
        collect_zero_items(root.get(1).unwrap(), &mut zero_items);
    }
    let roots = lists.iter().filter_map(|(id, st)| Some((*id, names.path_names(st)?))).collect();
    let mut x = Form { names, w: W { s: String::from(HEAD), depth: 1 }, items, commands, attrs, cols, tables, lists, roots, excluded: HashSet::new(), scopes: [true; 7], table_nodes, plain, xcols: HashMap::new(), col_class, owners, prim, extent, generated, zero_items, v59_flags };
    // Дополнительные колонки по пути таблицы: вложенные (`СвУпПред.Пред.СведОрг`) идут после родительских.
    if let Some(a) = root.get(3) {
        let tail = a.list().get(2 + num_at(a, 1).max(0) as usize..).unwrap_or(&[]);
        let extra = tail.first().and_then(Node::atom).and_then(|x| x.parse::<usize>().ok()).unwrap_or(0);
        for e in tail.iter().skip(1).take(extra) {
            let Some(table) = e.get(1).and_then(|d| x.data_path(d)) else { continue };
            for c in e.list().iter().skip(3).take(num_at(e, 2).max(0) as usize) {
                if let (Some(i), Some(n)) = (c.get(1).and_then(Node::atom), c.get(3).and_then(Node::str)) {
                    x.xcols.insert((table.clone(), i.to_string()), n);
                }
            }
        }
    }
    x.form(root)?;
    if root.get(1).is_some_and(|f| num_at(f, 0) == 28) {
        super::form_v28::tooltips(&mut x.w.s)?;
    }
    // Незнакомое значение помечено `\u{1}`: такой XML платформа не примет — форма не выгружается, причина — в ошибке.
    if let Some(i) = x.w.s.find('\u{1}') {
        let a = x.w.s[..i].rfind('\n').map_or(0, |a| a + 1);
        let b = x.w.s[i..].find('\r').map_or(x.w.s.len(), |b| i + b);
        return Err(format!("незнакомое значение: {}", x.w.s[a..b].trim().replace('\u{1}', "?")));
    }
    x.w.s.push_str("\r\n</Form>");
    Ok(Some([&[0xef, 0xbb, 0xbf][..], x.w.s.as_bytes()].concat()))
}

struct Form<'a> {
    names: &'a Names,
    w: W,
    /// id → имя: элементы, команды формы; реквизиты — (имя, класс типа).
    items: HashMap<&'a str, &'a str>,
    commands: HashMap<&'a str, &'a str>,
    attrs: HashMap<&'a str, (String, String, String)>,
    /// (id реквизита, id колонки) → имя колонки.
    cols: HashMap<(&'a str, &'a str), &'a str>,
    /// id реквизита → настройки динамического списка (словарь `{…,"ключ",значение...}`).
    lists: HashMap<&'a str, &'a Node>,
    /// id реквизита-списка со своим запросом → имена, которые обычный путь находит в корне (`Names::path_names`).
    roots: HashMap<&'a str, Vec<String>>,
    /// id таблицы → ее путь к данным (для `Items.<таблица>.CurrentData...`).
    tables: HashMap<&'a str, &'a Node>,
    /// GUID исключенных стандартных команд формы: ссылка на такую команду пишется `0:guid`.
    excluded: HashSet<String>,
    /// Группы стандартных команд формы (`command_scope`), которые у нее есть: ссылка на команду чужой группы — `0:guid`.
    scopes: [bool; 7],
    /// id таблицы → ее узел (исключенные команды, режим выбора — для ссылок на ее стандартные команды).
    table_nodes: HashMap<&'a str, &'a Node>,
    /// id групп, декораций и кнопок: стандартных команд у них нет — ссылку платформа пишет `id:guid`.
    plain: HashSet<&'a str>,
    /// (путь таблицы без индексов строк, id колонки) → имя дополнительной колонки (`AdditionalColumns table=...`).
    xcols: HashMap<(String, String), &'a str>,
    /// (id реквизита, id колонки) → класс типа колонки (для стандартных полей колонки-ссылки).
    col_class: HashMap<(&'a str, &'a str), String>,
    /// id реквизита формы → объект метаданных его типа (`Document.Имя`): поле чужого объекта платформа не разрешает.
    owners: HashMap<&'a str, String>,
    /// id реквизитов формы только примитивных типов: полей метаданных у них нет, путь `{0,guid}` от такого реквизита
    /// платформа не разрешает (ВыплатаЗарплаты ЗУП: `10/0:guid` у булева реквизита).
    /// shortcut: по смыслу типа и одному образцу, функция разрешения пути в mngbase не прослежена — при расхождении пути
    /// от примитивного реквизита найти ее и сверить.
    prim: HashSet<&'a str>,
    /// Динамический список → (наибольший номер FieldsMap, последний номер с полями своего запроса): номер за последним
    /// платформа не разрешает и путь не пишет (ВыплатыСамозанятым 58 при 24 и двух полях запроса).
    extent: HashMap<&'a str, (u32, u32)>,
    /// Имена элементов, которые платформа порождает при загрузке, по порядку их id за наибольшим ([`generated_items`]).
    generated: Vec<String>,
    /// Имена элементов с id `{0}` (запись 8.5 из загрузки XML): ссылка на элемент с id 0 — на такой элемент.
    zero_items: Vec<String>,
    /// Свойства 8.5 по id элемента (`rec` — форма): `form_v59::downgrade`.
    v59_flags: HashMap<String, u8>,
}

/// Хвост корня формы после навигатора: позиция → (свойство, значения по коду, код по умолчанию).
/// Значения выписаны из сверки с выгрузкой платформы.
const TAIL_ENUMS: &[(usize, &str, &[&str], i64)] = &[
    (3, "ScalingMode", &["", "Normal", "Compact"], 0),
    (6, "HorizontalSpacing", &["Auto", "None", "Half", "Single", "OneAndHalf", "Double"], 0),
    (7, "VerticalSpacing", &["Auto", "None", "Half", "Single", "OneAndHalf", "Double"], 0),
    (8, "HorizontalAlign", &["Left", "Center", "Right"], 3),
    (9, "VerticalAlign", &["Top", "Center", "Bottom"], 3),
    (10, "ChildrenAlign", &["Auto", "None", "ItemsLeftTitlesLeft", "ItemsRightTitlesLeft", "ItemsLeftTitlesRight", "ItemsRightTitlesRight", "TitlesLeftDataAuto"], 0),
    (12, "VerticalScroll", &["", "", "useIfNecessary", "useWithoutStretch"], 0),
    (16, "ConversationsRepresentation", &["", "Show", "DontShow"], 0),
    (17, "CollapseItemsByImportanceVariant", &["Auto", "Use", "DontUse"], 0), // mngcore 0x180789280
];

fn enum_name(t: &[&'static str], code: i64) -> Option<&'static str> {
    usize::try_from(code).ok().and_then(|c| t.get(c)).copied().filter(|s| !s.is_empty())
}

impl Form<'_> {
    /// Флаг свойства 8.5 элемента (`rec` — формы).
    fn flag(&self, id: &str, f: u8) -> bool {
        self.v59_flags.get(id).is_some_and(|x| x & f != 0)
    }

    fn form(&mut self, root: &Node) -> Result<(), String> {
        let f = root.get(1).unwrap();
        let ver = num_at(f, 0);
        if !(28..=50).contains(&ver) {
            return Err(format!("версия формы {ver}"));
        }
        // Словарь дополнительных свойств: {число, (ключ, значение)...}.
        let k = num_at(f, 18);
        let mut dict: HashMap<&str, &Node> = HashMap::new();
        for j in 0..k.max(0) as usize {
            dict.insert(atom_at(f, 19 + 2 * j), f.get(20 + 2 * j).ok_or("словарь формы")?);
        }
        let mut r = R::at(f, 19 + 2 * k as usize, "форма");
        let events = r.node()?;
        let excluded = r.node()?;
        self.excluded = excluded.list().iter().skip(1).filter_map(Node::atom).map(str::to_string).collect();
        // Без Customizable команды CustomizeForm нет: кнопка на нее — `0:guid`, как у исключенной.
        if num_at(f, 14) == 0 {
            self.excluded.insert("198ea630-fda2-4cda-8a23-f999f4c67ee6".into());
        }
        self.scopes = self.scopes(root);
        r.num()?;
        let acb = r.node()?;
        let nchild = r.num()?;
        let mut children = Vec::new();
        for _ in 0..nchild {
            children.push((r.atom()?, r.node()?));
        }
        r.str()?;
        r.str()?;
        // Навигатор — за флагом 1 (у старых версий бывает 0 без навигатора).
        if r.num()? != 0 {
            r.node()?;
        }
        let tail = &r.v[r.i..];
        // Версия 28 (ПодробнаяИнформация планов обмена, 6 форм БП) — всегда без хвоста: VerticalScroll useIfNecessary,
        // расширенные подсказки — `form_v28`. С хвостом не встречалась.
        if ver == 28 && !tail.is_empty() {
            return Err("версия формы 28 с хвостом: не сверено".into());
        }
        // У версий до 42 в хвосте только AutoURL и VerticalScroll (в 2); у старых хвост короче,
        // недостающие поля — по умолчанию.
        let tg = |i: usize| match (ver < 42, i) {
            (false, _) | (true, 0) => tail.get(i),
            (true, 12) if ver == 28 => Some(super::form_v28::scroll()),
            (true, 12) => tail.get(2),
            _ => None,
        };
        let has = |i: usize| tg(i).is_some();
        let t = |i: usize| tg(i).and_then(Node::atom).and_then(|a| a.parse::<i64>().ok()).unwrap_or(i64::MIN);
        let mut p = P::new(self.w.depth);
        if let Some(n) = f.get(10) {
            p.local("Title", n);
        }
        if num_at(f, 3) != 0 {
            p.leaf("Width", atom_at(f, 3));
        }
        if num_at(f, 4) != 0 {
            p.leaf("Height", atom_at(f, 4));
        }
        if !self.flag("rec", super::form_v59::UNSET_WINDOW_MODE) {
            p.leaf("WindowOpeningMode", match num_at(f, 2) {
                0 => "DontBlock",
                1 => "LockOwner",
                2 => "LockWholeInterface",
                _ => "\u{1}",
            });
        }
        if num_at(f, 5) == 0 {
            p.leaf("EnterKeyBehavior", "DefaultButton");
        }
        if num_at(f, 7) == 1 {
            p.leaf("AutoSaveDataInSettings", "Use");
        }
        if num_at(f, 6) == 1 {
            p.leaf("SaveDataInSettings", "UseList");
        }
        if t(20) == 0 {
            p.leaf("SaveWindowSettings", "false");
        }
        if atom_at(f, 8) != "00000000-0000-0000-0000-000000000000" {
            let id = atom_at(f, 8);
            p.leaf("SettingsStorage", self.names.full.get(id).map_or(id, |s| s.as_str()));
        }
        if num_at(f, 9) == 0 {
            p.leaf("AutoTitle", "false");
        }
        if t(0) == 0 {
            p.leaf("AutoURL", "false");
        }
        // Без поля 18 хвоста (версии до 48) группировка — поле 11 корня (0/1); поле 11 хвоста (0/1/2) с ним совпадает
        // (БП: 59 форм версий 38–46, у остальных 7 458 — тоже).
        let group = match has(18) {
            true => t(18),
            false if has(11) && t(11) != num_at(f, 11) => -1,
            false => num_at(f, 11),
        };
        if !self.flag("rec", super::form_v59::UNSET_GROUP) {
            p.leaf("Group", match group {
                0 => "Vertical",
                1 => "Horizontal",
                2 => "HorizontalIfPossible",
                3 => "AlwaysHorizontal",
                _ => "\u{1}",
            });
        }
        for &(i, name, vals, def) in TAIL_ENUMS {
            if has(i) && t(i) != def {
                p.leaf(name, enum_name(vals, t(i)).unwrap_or("\u{1}"));
            }
        }
        if num_at(f, 12) != 0 {
            p.leaf("ChildItemsWidth", ["", "Equal", "LeftWide", "LeftWidest", "LeftNarrow", "LeftNarrowest"].get(num_at(f, 12) as usize).copied().unwrap_or("\u{1}"));
        }
        if num_at(f, 13) == 0 {
            p.leaf("AutoFillCheck", "false");
        }
        if num_at(f, 14) == 0 {
            p.leaf("Customizable", "false");
        }
        if num_at(f, 15) == 0 {
            p.leaf("Enabled", "false");
        }
        match num_at(f, 17) {
            1 => {}
            0 => p.leaf("CommandBarLocation", "None"),
            2 => p.leaf("CommandBarLocation", "Top"),
            3 => p.leaf("CommandBarLocation", "Bottom"),
            _ => p.leaf("CommandBarLocation", "\u{1}"),
        }
        if has(13) && t(13) != 100 {
            p.leaf("Scale", &t(13).to_string());
        }
        if t(14) == 0 {
            p.leaf("ShowTitle", "false");
        }
        if t(15) == 0 {
            p.leaf("ShowCloseButton", "false");
        }
        match num_at(f, 17) {
            0 => p.leaf("ShowCommandBar", "false"),
            2 | 3 => p.leaf("ShowCommandBar", "true"),
            _ => {}
        }
        let main_id = root.get(3).and_then(|a| a.list().iter().skip(2).find(|at| num_at(at, 10) == 1)).map_or("", |at| at.get(1).map_or("", |i| atom_at(i, 0)));
        let main_class = self.attrs.get(main_id).map_or("", |a| a.2.as_str());
        // Форма объекта справочника без ключа 0: платформа пишет UseForFoldersAndItems по умолчанию — Items (каркас и БП:
        // 270+ форм справочников без групп; форма элемента иерархического справочника, созданная Конфигуратором 8.5).
        if main_class == "CatalogObject" && !dict.contains_key("0") {
            p.leaf("UseForFoldersAndItems", "Items");
        }
        // Форма документа без ключей 2–4 (конструктор 8.5, 28.10.2026): платформа пишет умолчания.
        if main_class == "DocumentObject" && !["2", "3", "4"].iter().any(|k| dict.contains_key(*k)) {
            p.leaf("AutoTime", "CurrentOrLast");
            p.leaf("UsePostingMode", "Auto");
            p.leaf("RepostOnWrite", "true");
        }
        let mut keys: Vec<_> = dict.iter().collect();
        keys.sort_by_key(|(k, _)| k.parse::<i64>().unwrap_or(0));
        // ViewModeApplicationOnSetReportResult по умолчанию (без ключа 29)
        let mut vm_default = false;
        // ReportResultViewMode по умолчанию (ключ 7 без 27)
        let mut rr_default = false;
        // Свойства отчета (ключи 7, 21, 27, 29) у формы с основным реквизитом примитивного типа платформа не пишет
        // (ЗУП ОбщиеОбъектыРеглОтчетности.УведомлениеОбИзмененияхВФормеОтчета: реквизит xs:string, ключи в записи есть).
        // Ключи 5, 6, 20 в образце пустые (`{0,""}`, не пишутся и так); непустые у такой формы не сверены — отказ.
        // shortcut: правило по образцу, расширение формы отчета в mngcore не снято; при основном реквизите другого
        // не отчетного типа (обработка, справочник) или без типа — пишутся, как раньше; пересмотреть, когда встретится.
        let foreign = root
            .get(3)
            .and_then(|a| a.list().iter().skip(2).find(|at| num_at(at, 10) == 1))
            .and_then(|at| at.get(5))
            .is_some_and(|t| t.list().len() > 1 && !t.list().iter().any(|x| x.get(0).and_then(Node::str) == Some("#")));
        for (key, v) in keys {
            match *key {
                "7" | "21" | "27" | "29" if foreign => {}
                "5" | "6" | "20" if foreign && num_at(v.get(2).unwrap_or(&EMPTY), 0) != 0 => {
                    p.leaf("\u{1}свойство", &unk(&format!("ключ {key} отчета у формы с примитивным основным реквизитом"), v))
                }
                // Значения — номер в перечислении XDTO-модели enums.dll (FoldersAndItemsUse, AutoTimeMode, PostingModeUse).
                "0" => p.en("UseForFoldersAndItems", FOLDERS_AND_ITEMS_USE, num_at(v, 2), i64::MIN),
                "1" => self.item_ref(&mut p, "GroupList", v),
                "23" => self.item_ref(&mut p, "CustomSettingsFolder", v),
                "2" => p.en("AutoTime", AUTO_TIME_MODE, num_at(v, 2), i64::MIN),
                "3" => p.en("UsePostingMode", POSTING_MODE_USE, num_at(v, 2), i64::MIN),
                "4" => p.leaf("RepostOnWrite", bool_text(atom_at(v, 1) == "1")),
                "5" => self.attr_ref(&mut p, "ReportResult", v),
                "6" => self.attr_ref(&mut p, "DetailsData", v),
                "20" => self.attr_ref(&mut p, "VariantAppearance", v),
                "7" => {
                    p.en("ReportFormType", &["Main", "Settings", "Variant"], num_at(v, 2), -1);
                    // Формы версий 42/46 без ключа 27 (3 из 3 БП): платформа пишет оба свойства по умолчанию.
                    if !dict.contains_key("27") && !dict.contains_key("29") {
                        p.leaf("ReportResultViewMode", "Auto");
                        p.leaf("ViewModeApplicationOnSetReportResult", "Auto");
                        vm_default = true;
                        rr_default = true;
                    }
                }
                // enums.dll FUN_180017e10
                "21" => p.en("AutoShowState", &["Auto", "DontShow", "Show", "ShowOnComposition"], num_at(v, 2), -1),
                "27" => {
                    p.en("ReportResultViewMode", &["Auto"], num_at(v, 2), -1);
                    if !dict.contains_key("29") {
                        p.leaf("ViewModeApplicationOnSetReportResult", "Auto");
                        vm_default = true;
                    }
                }
                "29" => p.en("ViewModeApplicationOnSetReportResult", &["Auto"], num_at(v, 2), -1),
                // Служебные ключи без свойства в XML (адреса urn:form:md, настройки вариантов отчета).
                "8" | "9" | "10" | "11" | "12" | "13" | "14" | "15" | "16" | "17" | "18" | "19" | "22" | "24" | "25" | "26" => {}
                // форма компоновщика настроек: 28 `{"B",0}` — постоянное, в XML нет (imp/form.rs, НастройкаОтборовСписка)
                "28" if v.get(0).and_then(Node::str) == Some("B") && atom_at(v, 1) == "0" => {}
                _ => p.leaf("\u{1}dict", &unk(key, v)),
            }
        }
        self.command_set(&mut p, excluded, root);
        // 8.3.24 пишет умолчания, только если у формы есть исключенные команды (БП 3.0.107: 55 из 55 с CommandSet, 0 из 82 без).
        if vm_default && self.names.older.is_some_and(|v| v <= 17) && !p.items.iter().any(|(n, _)| *n == "CommandSet") {
            p.items.retain(|(n, _)| *n != "ViewModeApplicationOnSetReportResult" && !(rr_default && *n == "ReportResultViewMode"));
        }
        if let Some(m) = tg(19).filter(|m| num_at(m, 1) > 0) {
            let mut w = p.w();
            w.open("MobileDeviceCommandBarContent");
            for k in 0..num_at(m, 1) as usize {
                let v = m.get(3 + 2 * k).unwrap_or(&EMPTY);
                w.open("xr:Item");
                w.leaf("xr:Presentation", m.get(2 + 2 * k).and_then(Node::str).unwrap_or(""));
                w.leaf("xr:CheckState", "0");
                let name = match v.get(0).and_then(Node::str) {
                    // 0 — пусто; нет элемента — `id:02023637-...`, как у GroupList.
                    Some("N") if atom_at(v, 1) == "0" => String::new(),
                    Some("N") => self.items.get(atom_at(v, 1)).map_or_else(|| format!("{}:{ELEM}", atom_at(v, 1)), |s| s.to_string()),
                    _ => unk("значение", v),
                };
                if name.is_empty() {
                    w.line("<xr:Value xsi:type=\"xs:string\"/>");
                } else {
                    w.line(&format!("<xr:Value xsi:type=\"xs:string\">{}</xr:Value>", esc(&name)));
                }
                w.close("xr:Item");
            }
            w.close("MobileDeviceCommandBarContent");
            p.push("MobileDeviceCommandBarContent", w);
        }
        p.emit(&mut self.w, FORM_ORDER);
        let d = self.w.depth;
        let acb = self.item(None, acb, d, Some("AutoCommandBar"))?;
        self.w.s.push_str(&acb);
        // Часть событий формы есть только при определенном основном реквизите, иначе платформа пишет GUID (сверка БП 3.0.171
        // по GUID записи); основной реквизит, для которого данных нет, — отказ.
        let main = root.get(3).and_then(|a| a.list().iter().skip(2).find(|at| num_at(at, 10) == 1)).and_then(|at| self.attrs.get(at.get(1).map_or("", |i| atom_at(i, 0))));
        let (mc, mt) = main.map_or(("", ""), |x| (x.1.as_str(), x.2.as_str()));
        let object = (mt.ends_with("Object") && mt != "ReportObject" && mt != "DataProcessorObject") || mt.ends_with("RecordManager") || mt.ends_with("RecordSet") || mc == "cfg:ConstantsSet";
        let document = mt == "DocumentObject";
        let report = mt == "ReportObject" || mc == "cfg:ReportObject";
        let pick = |name: bool, guid: bool, g: &str| {
            if name {
                form_event(g).map(String::from)
            } else if guid {
                Some(g.to_string())
            } else {
                Some(format!("\u{1}событие {g}"))
            }
        };
        events_xml(&mut self.w, &[events], &|g| match g {
            // Неприменимый вариант BeforeWrite[AtServer] — GUID: и без основного реквизита, и «документный» у справочника
            // (БП 3.0.107: 21 форма, НастройкиОбменСБанками.ФормаЭлемента — оба варианта сразу).
            "9cc34712-da5f-4faa-a653-343d2085fbe8" | "bf0ac0e1-bcbb-4dfe-8fc4-0b1923b461a6" => pick(object && !document, !(object && !document), g),
            "8a5894c9-d2ff-4c1d-b433-89cc352bbfbc" | "8f42e083-be92-4102-b1f0-fa58452c1a63" => pick(document, !document, g),
            // OnWriteAtServer, OnReadAtServer, AfterWrite, AfterWriteAtServer (без основного реквизита — БП 3.0.107, 50 форм;
            // с основным реквизитом-обработкой — ЗУП СозданиеВедомостейНаВыплатуЗарплаты.Ведомость и СозданиеРеестров...Реестр).
            // mngcore FUN_1802a8810: имя события записи регистрируется, только если его поддерживает расширение основного
            // реквизита (vtable+0x18 первого параметра; без реквизита — NULL); вызов обработчика есть только у объектов (basic).
            "c1bc0d3e-d35e-4207-a06b-ece68ed25314" | "390d5e4b-e732-4c88-8748-9e211a416984" | "047d4d09-961c-4bdc-8519-eef10674c35b"
            | "213d1900-dcad-4616-9f20-3f077156a40f" => pick(object, main.is_none() || mt == "DataProcessorObject", g),
            "7b15b3db-1cd0-4e1d-a74b-2c972c9e2226" | "87ce636e-9de6-4e42-9395-f0f189d08397" | "961ee7c6-0327-422b-adcb-97a90c46753d" => {
                pick(report, main.is_none() || mt == "DataProcessorObject", g)
            }
            _ => form_event(g).map(String::from),
        })?;
        let ch = self.children(&children, d + 1)?;
        if !ch.is_empty() {
            self.w.open("ChildItems");
            self.w.s.push_str(&ch);
            self.w.close("ChildItems");
        }
        let attrs = root.get(3).ok_or("нет реквизитов")?;
        self.attributes(attrs)?;
        self.commands_xml(root.get(5).unwrap_or(&EMPTY));
        self.parameters(root.get(4).unwrap_or(&EMPTY));
        self.command_interface(root.get(6).unwrap_or(&EMPTY), root.get(7).unwrap_or(&EMPTY));
        Ok(())
    }

    /// Командный интерфейс: панель навигации `[6]` и командная панель `[7]`, каждая `{0,N,элементы}`;
    /// элемент `{3,номер,команда,параметр (путь к данным),тип (0 Auto, 1 Added),{0,группа},Index,
    /// видимость по умолчанию (1 — свою видимость не пишет),видимость}`.
    fn command_interface(&mut self, nav: &Node, bar: &Node) {
        if num_at(nav, 1) <= 0 && num_at(bar, 1) <= 0 {
            return;
        }
        self.w.open("CommandInterface");
        for (tag, n) in [("NavigationPanel", nav), ("CommandBar", bar)] {
            let cnt = num_at(n, 1).max(0) as usize;
            if cnt == 0 {
                continue;
            }
            self.w.open(tag);
            for it in n.list().iter().skip(2).take(cnt) {
                let mut p = P::new(self.w.depth + 1);
                let c = it.get(2).unwrap_or(&EMPTY);
                p.leaf("Command", &if c.get(1).is_some() { self.command(c) } else { atom_at(c, 0).to_string() });
                p.leaf("Type", match num_at(it, 4) {
                    0 => "Auto",
                    1 => "Added",
                    _ => "\u{1}",
                });
                if let Some(d) = it.get(3).and_then(|d| self.data_path(d)) {
                    p.leaf("Attribute", &d);
                }
                let g = it.get(5).map_or("", |g| atom_at(g, 1));
                if !g.is_empty() {
                    let std = super::classes::STD_COMMAND_GROUPS.iter().find(|(id, _)| *id == g).map(|(_, n)| *n);
                    p.leaf("CommandGroup", std.or_else(|| self.names.full.get(g).map(String::as_str)).unwrap_or(g));
                }
                p.num_nz("Index", atom_at(it, 6));
                if num_at(it, 7) == 0 {
                    p.leaf("DefaultVisible", "false");
                    p.adjustable("Visible", it.get(8), self.names);
                }
                self.w.open("Item");
                p.emit(&mut self.w, CI_ITEM_ORDER);
                self.w.close("Item");
            }
            self.w.close(tag);
        }
        self.w.close("CommandInterface");
    }

    /// Команды `{0,N,{ver,{id,409b9a53…},"имя",Title,ToolTip,Use,Shortcut,Picture,"действие",Representation,ModifiesSavedData,0,
    /// {ФО},?,AssociatedTableElementId,?,?,?,CurrentRowUse}...}`; старые версии короче.
    fn commands_xml(&mut self, n: &Node) {
        let cnt = num_at(n, 1).max(0) as usize;
        if cnt == 0 {
            return;
        }
        let nm = self.names;
        self.w.open("Commands");
        for c in n.list().iter().skip(2).take(cnt) {
            // Запись из 21 поля: поле 15 = 2 и за ним `2,""` (НастройкиОбновления РегламентированнаяОтчетность — БП и
            // каркас 8.5, 2 из 2) в XML не попадают; дальше — как у записи из 19 полей. Иные значения — меткой ниже.
            let norm;
            let c = match c.list() {
                l if l.len() == 21 && atom_at(c, 15) == "2" && atom_at(c, 16) == "2" && l[17].str() == Some("") => {
                    let mut v = l.to_vec();
                    v.splice(15..18, [Node::Atom("1".into())]);
                    norm = Node::List(v);
                    &norm
                }
                _ => c,
            };
            let id = c.get(1).map_or("", |i| atom_at(i, 0));
            let name = c.get(2).and_then(Node::str).unwrap_or("");
            let mut p = P::new(self.w.depth + 1);
            p.local("Title", c.get(3).unwrap_or(&EMPTY));
            p.local("ToolTip", c.get(4).unwrap_or(&EMPTY));
            p.adjustable("Use", c.get(5), nm);
            if let Some(s) = c.get(6).and_then(shortcut) {
                p.leaf("Shortcut", &s);
            }
            p.picture("Picture", c.get(7), nm);
            // Пустое действие платформа не пишет.
            if let Some(a) = c.get(8).and_then(Node::str).filter(|a| !a.is_empty()) {
                p.leaf("Action", a);
            }
            p.functional_options(c.get(12), nm);
            p.en("Representation", &["Text", "Picture", "TextPicture"], num_at(c, 9), 3);
            p.flag_if("ModifiesSavedData", num_at(c, 10) == 1, "true");
            let l = c.list();
            let cur = if l.len() > 18 { num_at(c, 18) } else if l.len() > 13 { num_at(c, 13) } else { 2 };
            p.en("CurrentRowUse", &["Use", "DontUse"], cur, 2);
            let t = atom_at(c, 14);
            if !t.is_empty() && t != "0" {
                let name = self.items.get(t).map_or_else(|| format!("{t}:{ELEM}"), |s| s.to_string());
                p.raw("AssociatedTableElementId", format!("\r\n{}<AssociatedTableElementId xsi:type=\"xs:string\">{}</AssociatedTableElementId>", "\t".repeat(p.depth), esc(&name)));
            }
            // Поля 11, 15–17 и хвост длиннее 19 в БП почти всегда одинаковы; иное — меткой. Поле 16 = 2 в XML не попадает
            // (ВходящиеСообщенияСЭДОФСС.ФормаЗаписи: две команды, в эталоне без лишних свойств).
            for (i, def) in [(11, "0"), (15, "1"), (16, "0"), (17, "0")] {
                if l.len() > i && atom_at(c, i) != def && !(i == 16 && atom_at(c, i) == "2") {
                    p.leaf("\u{1}команда", &unk(&i.to_string(), &l[i]));
                }
            }
            if l.len() > 19 {
                p.leaf("\u{1}команда", &unk("хвост", c));
            }
            // Команда без свойств — пустым элементом (ВыплатыСамозанятым.ФормаВыбораКодировки).
            if p.items.is_empty() {
                self.w.line(&format!("<Command name=\"{}\" id=\"{id}\"/>", esc_attr(name)));
                continue;
            }
            self.w.line(&format!("<Command name=\"{}\" id=\"{id}\">", esc_attr(name)));
            self.w.depth += 1;
            p.emit(&mut self.w, COMMAND_ORDER);
            self.w.close("Command");
        }
        self.w.close("Commands");
    }

    /// Параметры `{0,N,{0,"имя",{тип},ключевой}...}`.
    fn parameters(&mut self, n: &Node) {
        let cnt = num_at(n, 1).max(0) as usize;
        if cnt == 0 {
            return;
        }
        self.w.open("Parameters");
        for a in n.list().iter().skip(2).take(cnt) {
            let w = &mut self.w;
            w.line(&format!("<Parameter name=\"{}\">", esc_attr(a.get(1).and_then(Node::str).unwrap_or(""))));
            w.depth += 1;
            type_xml(w, self.names, a.get(2).unwrap_or(&EMPTY));
            if num_at(a, 3) == 1 {
                w.leaf("KeyParameter", "true");
            }
            w.close("Parameter");
        }
        self.w.close("Parameters");
    }

    /// `{"N",id}` — ссылка на элемент формы; 0 — нет.
    fn item_ref(&self, p: &mut P, name: &'static str, v: &Node) {
        let id = atom_at(v, 1);
        if id != "0" {
            p.leaf(name, &self.item_name(id));
        }
    }

    /// Имя элемента по id. Элемента нет — платформа пишет `id:guid`, как и для ненайденных команд; но id за наибольшим
    /// в форме она отдает элементу, порожденному при загрузке ([`generated_items`]; UserSettingsGroup 26 →
    /// ПериодПанельДействийВыделенныхСтрок, БухучетЗарплатыСотрудников). Порожденных нет (версия 28) или id за
    /// ними — отказ.
    fn item_name(&self, id: &str) -> String {
        if let Some(s) = self.items.get(id) {
            return s.to_string();
        }
        let max = self.items.keys().filter_map(|k| k.parse::<i64>().ok()).max().unwrap_or(0);
        match id.parse::<i64>() {
            Ok(n) if n <= max => format!("{id}:{ELEM}"),
            // Порожденные элементы — у 8.5; 8.3.24 и 8.3.27 пишут `id:guid` (БухучетЗарплатыСотрудников).
            Ok(n) if n > max && ((n - max - 1) as usize) < self.generated.len() && self.names.older.is_some() => format!("{id}:{ELEM}"),
            Ok(n) if n > max && ((n - max - 1) as usize) < self.generated.len() => self.generated[(n - max - 1) as usize].clone(),
            _ => format!("\u{1}элемент {id} за наибольшим id {max}"),
        }
    }

    /// `{"#",guid,{1,{id},""}}` — ссылка на реквизит формы; `{0,""}` — нет.
    fn attr_ref(&self, p: &mut P, name: &'static str, v: &Node) {
        let r = v.get(2).unwrap_or(&EMPTY);
        if num_at(r, 0) == 0 {
            return;
        }
        let id = r.get(1).map_or("", |i| atom_at(i, 0));
        p.leaf(name, self.attrs.get(id).map_or("\u{1}", |a| a.0.as_str()));
    }

    /// Какие группы стандартных команд формы (`command_scope`) есть у формы с таким основным реквизитом
    /// (объект, список, отчет, задача).
    fn scopes(&self, root: &Node) -> [bool; 7] {
        let main_id = root.get(3).and_then(|a| a.list().iter().skip(2).find(|at| num_at(at, 10) == 1)).map_or("", |at| at.get(1).map_or("", |i| atom_at(i, 0)));
        let (main_class, main) = self.attrs.get(main_id).map_or(("", ""), |x| (x.1.as_str(), x.2.as_str()));
        let object = (main.ends_with("Object") && main != "ReportObject" && main != "DataProcessorObject")
            || main.ends_with("RecordManager")
            || main.ends_with("RecordSet")
            || main_class == "cfg:ConstantsSet";
        let list = main == "DynamicList";
        let report = main == "ReportObject" || main_class == "cfg:ReportObject";
        [true, object, list, object || list, report, main == "TaskObject", report || main_class == "dcsset:SettingsComposer"]
    }

    /// Исключенные стандартные команды `{N,guid...}`: пишутся по имени, по алфавиту, только применимые к форме.
    fn command_set(&self, p: &mut P, ex: &Node, root: &Node) {
        let cnt = num_at(ex, 0).max(0) as usize;
        if cnt == 0 {
            return;
        }
        // Тип основного реквизита определяет набор команд формы.
        let main_id = root.get(3).and_then(|a| a.list().iter().skip(2).find(|at| num_at(at, 10) == 1)).map_or("", |at| at.get(1).map_or("", |i| atom_at(i, 0)));
        let main = self.attrs.get(main_id).map_or(("", ""), |x| (x.1.as_str(), x.2.as_str()));
        // Документ основного реквизита (объект или основная таблица списка): при Posting = Deny нет Post/UndoPosting.
        let doc = self.owners.get(main_id).and_then(|o| self.names.objects.get(&o.to_lowercase())).map(String::as_str).or_else(|| {
            self.lists.get(main_id).and_then(|st| st.list().get(2..)?.chunks(2).find(|c| c[0].str() == Some("MainTable"))?.get(1)).map(|v| atom_at(v, 2))
        });
        let no_posting = doc.is_some_and(|d| self.names.no_posting.contains(d));
        // Список без основной таблицы — без команд изменения и иерархии (каркас: 12 таких команд в записи, в XML ни одной);
        // таблица основного списка без ChangeRowSet — без команд изменения состава, как у самой таблицы.
        let list_main = self.lists.get(main_id).and_then(|st| st.list().get(2..)?.chunks(2).find(|c| c[0].str() == Some("MainTable"))?.get(1)).map_or("", |v| atom_at(v, 2));
        let no_main = main.1 == "DynamicList" && no_main(list_main);
        // (ChangeRowSet, выделение одной строки) таблиц основного списка
        let rows: Vec<(bool, bool)> = self
            .tables
            .iter()
            .filter(|(_, d)| num_at(d, 0) == 1 && d.get(1).is_some_and(|i| atom_at(i, 0) == main_id))
            .filter_map(|(id, _)| self.table_nodes.get(id))
            .map(|t| (It::new(t, 4).num(17) == 1, It::new(t, 4).num(24) == 0))
            .collect();
        let no_rows = main.1 == "DynamicList" && !rows.is_empty() && rows.iter().all(|r| !r.0);
        // Как у таблицы (`table_lacks`): при выделении одной строки во всех таблицах основного списка нет и команды формы
        // ShowMultipleSelection (ЗУП: 3 формы; в эталонах пяти конфигураций ни одной формы с ней при SingleRow).
        // shortcut: правило по образцам, отбор команд формы в mngbase не снят; таблицы с разным выделением — отказ.
        let single = main.1 == "DynamicList" && !rows.is_empty() && rows.iter().all(|r| r.1);
        let mixed = main.1 == "DynamicList" && rows.iter().any(|r| r.1) && !single;
        let task = main.1 == "TaskObject";
        // У формы без Customizable команды «Изменить форму» нет (282 таких формы каркаса — ни одной с ней).
        let customizable = root.get(1).is_none_or(|f| num_at(f, 14) != 0);
        let mut names: Vec<String> = Vec::new();
        // Неполный набор неразличимых команд, которых у формы нет при любом соответствии (CreateInitialImage, ReadChanges,
        // WriteChanges у формы без основного реквизита-объекта: ОбменЗарплата3Бухгалтерия3.ПомощникНастройки...), — не пишется.
        let absent = |g: &str| COMMAND_GROUPS.iter().any(|(all, names)| all.contains(&g) && names.iter().all(|n| !self.scopes[command_scope(n) as usize]));
        let guids = ex.list().iter().skip(1).take(cnt).filter_map(Node::atom).filter(|g| !absent(g));
        for n in grouped(guids, |g| std_form_command2(g).map(str::to_string)) {
            let keep = self.scopes[command_scope(&n) as usize]
                && (customizable || n != "CustomizeForm")
                && !(no_posting && matches!(n.as_str(), "Post" | "UndoPosting"))
                && !(no_rows && matches!(n.as_str(), "Copy" | "Create" | "CreateFolder" | "Delete" | "SetDeletionMark" | "MoveItem"))
                && !(no_main && NO_MAIN_COMMANDS.contains(&n.as_str()));
            // Команды списка, про которые без основной таблицы данных нет.
            // Choose, OutputList — пишутся (ERP 2.5.22: 6 форм выбора и 8 расшифровок со списком без основной таблицы).
            // shortcut: отбор команд формы платформой (mngbase) не снят с DLL — прочие команды без основной таблицы отказом
            if no_main && matches!(n.as_str(), "Post" | "UndoPosting" | "SetDateInterval" | "FindByCurrentValue" | "ChangeHistory" | "GetURL" | "ShowMultipleSelection") {
                names.push(format!("\u{1}{n}"));
                continue;
            }
            if n == "ShowMultipleSelection" && (single || mixed) {
                if mixed {
                    names.push(format!("\u{1}{n}"));
                }
                continue;
            }
            if keep {
                // У задачи «Записать и закрыть» — «Выполнить и закрыть».
                names.push(if task && n == "WriteAndClose" { "ExecuteAndClose".into() } else { n });
            }
        }
        if names.is_empty() {
            return;
        }
        names.sort();
        let mut w = p.w();
        w.open("CommandSet");
        for n in names {
            w.leaf("ExcludedCommand", &n);
        }
        w.close("CommandSet");
        p.push("CommandSet", w);
    }

    /// Реквизиты `{4,N,реквизит...,M,(доп. колонки)×M,0,{#base64:условное оформление}}`.
    fn attributes(&mut self, n: &Node) -> Result<(), String> {
        let cnt = num_at(n, 1).max(0) as usize;
        let l = n.list();
        let tail = l.get(2 + cnt..).unwrap_or(&[]);
        // Дополнительные колонки табличных частей: `{0,{путь},K,колонка...}`, путь начинается с реквизита.
        let extra = tail.first().and_then(Node::atom).and_then(|a| a.parse::<usize>().ok()).unwrap_or(0);
        let mut add: HashMap<&str, Vec<&Node>> = HashMap::new();
        for e in tail.iter().skip(1).take(extra) {
            let id = e.get(1).and_then(|p| p.get(1)).map_or("", |i| atom_at(i, 0));
            add.entry(id).or_default().push(e);
        }
        // Платформа пишет таблицы доп. колонок реквизита по возрастанию пути: ТЧ `{0,guid}` — по GUID строкой
        // (109 из 109 форм каркаса), колонки реквизита формы `{N}` и вложенные `{N,5bdad865}` — по N.
        // Смесь GUID и номеров на одном уровне не встречалась — форма отвергается.
        let mut mixed = None;
        for v in add.values_mut() {
            let key = |e: &&Node| -> Vec<(u8, i64, String)> {
                let p = e.get(1).map_or(&[][..], Node::list);
                p.iter()
                    .skip(2)
                    .map(|x| match (atom_at(x, 0), x.get(1).and_then(Node::atom)) {
                        ("0", Some(g)) => (1, 0, g.to_string()),
                        (n, _) => (0, n.parse().unwrap_or(i64::MIN), String::new()),
                    })
                    .collect()
            };
            let keys: Vec<_> = v.iter().map(key).collect();
            let depth = keys.iter().map(Vec::len).max().unwrap_or(0);
            for d in 0..depth {
                let kinds: HashSet<u8> = keys.iter().filter_map(|k| k.get(d)).map(|k| k.0).collect();
                if kinds.len() > 1 {
                    mixed = v.first().copied();
                }
            }
            v.sort_by_cached_key(key);
        }
        if let Some(e) = mixed {
            self.w.leaf("\u{1}реквизиты", &unk("порядок доп. колонок", e));
        }
        let ca = self.conditional_appearance(tail.last())?;
        if cnt == 0 && ca.is_empty() {
            self.w.line("<Attributes/>");
            return Ok(());
        }
        self.w.open("Attributes");
        for a in &l[2..2 + cnt] {
            let id = atom_at(a.get(1).unwrap_or(a), 0);
            self.attribute(a, add.get(id).map_or(&[][..], |v| v.as_slice()))?;
        }
        // Дальше M четверок (имя поля условного оформления ×2, два пути к данным) — в XML не попадают.
        let m = tail.get(1 + extra).and_then(Node::atom).and_then(|a| a.parse::<usize>().ok());
        if m.is_none_or(|m| tail.len() < 2 + extra + 4 * m) {
            self.w.leaf("\u{1}реквизиты", &unk("хвост", n));
        }
        self.w.s.push_str(&ca);
        self.w.close("Attributes");
        Ok(())
    }

    /// `{9,{id},0,"имя",Title,Type,View,Edit,UseAlways,Save,Main,SavedData,FillCheck,N,колонка...,Settings,ФО}`.
    fn attribute(&mut self, a: &Node, extra: &[&Node]) -> Result<(), String> {
        let id = atom_at(a.get(1).unwrap_or(a), 0);
        let name = a.get(3).and_then(Node::str).ok_or("реквизит: нет имени")?;
        let nm = self.names;
        let ncol = num_at(a, 13).max(0) as usize;
        let mut p = P::new(self.w.depth + 1);
        p.local("Title", a.get(4).unwrap_or(&EMPTY));
        let mut w = p.w();
        type_xml(&mut w, nm, a.get(5).ok_or("реквизит: нет типа")?);
        p.push("Type", w);
        p.adjustable("View", a.get(6), nm);
        p.adjustable("Edit", a.get(7), nm);
        p.flag_if("MainAttribute", num_at(a, 10) == 1, "true");
        p.flag_if("SavedData", num_at(a, 11) == 1, "true");
        p.flag_if("FillCheck", num_at(a, 12) == 1, "ShowError");
        let set_ua;
        let mut ua = a.get(8);
        if self.attrs.get(id).is_some_and(|x| x.1 == "cfg:ConstantsSet") {
            match nm.constants_use_always(ua) {
                Some(n) => {
                    set_ua = n;
                    ua = Some(&set_ua);
                }
                None => p.leaf("\u{1}реквизит", "\u{1}UseAlways набора констант"),
            }
        }
        self.fields(&mut p, "UseAlways", ua, name, id);
        self.list_use_always(&mut p, a.get(14 + ncol), name, id);
        self.fields(&mut p, "Save", a.get(9), name, id);
        p.functional_options(a.get(15 + ncol), nm);
        if ncol > 0 || !extra.is_empty() {
            let mut w = p.w();
            w.open("Columns");
            for c in a.list().iter().skip(14).take(ncol) {
                column(&mut w, nm, c);
            }
            for e in extra {
                let table = e.get(1).and_then(|d| self.data_path(d)).unwrap_or_default();
                let k = num_at(e, 2).max(0) as usize;
                if k == 0 {
                    w.line(&format!("<AdditionalColumns table=\"{}\"/>", esc_attr(&table)));
                    continue;
                }
                w.line(&format!("<AdditionalColumns table=\"{}\">", esc_attr(&table)));
                w.depth += 1;
                for c in e.list().iter().skip(3).take(k) {
                    column(&mut w, nm, c);
                }
                w.close("AdditionalColumns");
            }
            w.close("Columns");
            p.push("Columns", w);
        }
        if let Some(st) = a.get(14 + ncol).filter(|s| num_at(s, 1) > 0) {
            let mut w = p.w();
            settings(&mut w, nm, st);
            p.push("Settings", w);
        }
        if a.list().len() != 16 + ncol {
            p.leaf("\u{1}реквизит", &unk("длина", a));
        }
        self.w.line(&format!("<Attribute name=\"{}\" id=\"{id}\">", esc_attr(name)));
        self.w.depth += 1;
        p.emit(&mut self.w, ATTRIBUTE_ORDER);
        self.w.close("Attribute");
        Ok(())
    }

    /// «Использовать всегда» динамического списка — `ReqMapFieldIdK` в настройках; платформа сортирует пути.
    fn list_use_always(&self, p: &mut P, st: Option<&Node>, name: &str, id: &str) {
        let Some(st) = st.filter(|s| s.get(2).and_then(Node::str) == Some("QueryText")) else { return };
        let mut v: Vec<String> = st
            .list()
            .chunks(2)
            .filter(|c| c[0].str().is_some_and(|k| k.starts_with("ReqMapFieldId")))
            .filter_map(|c| c.get(1).and_then(|v| v.get(1)).and_then(Node::atom))
            // Дыра в FieldsMap и номер за последним полем своего запроса — как у путей элементов, не пишутся
            // (КорреспонденцииСчетов 90/94 при 117, ПодборОсновныхСредств, НастройкиУчетаЗарплаты 231–233 при 73).
            .filter(|f| std_field("DynamicList", f.parse().unwrap_or(0)).is_some() || !self.list_gone(id, f))
            .map(|f| match (self.cols.get(&(id, f)), std_field("DynamicList", f.parse().unwrap_or(0))) {
                (Some(c), _) if self.twin_extra(id, f) => format!("~{name}.{c}"),
                (Some(c), _) => self.names.mark(st, c, format!("{name}.{c}"), &self.ru_col(id), self.roots.get(id).map(Vec::as_slice)),
                // порядок, отбор, группировка — свойства самого списка, не поля таблицы: доступны всегда
                // (ERP ПравилаГруппировкиАктивовОбязательств, БП 3.0.171: `Список.Order`)
                (None, Some(c @ ("Order" | "Filter" | "Group"))) => format!("{name}.{c}"),
                (None, Some(c)) => self.names.mark(st, c, format!("{name}.{c}"), &self.ru_col(id), self.roots.get(id).map(Vec::as_slice)),
                _ => format!("\u{1}поле:{f}"),
            })
            .collect();
        if v.is_empty() {
            return;
        }
        v.sort();
        let mut w = p.w();
        w.open("UseAlways");
        for f in v {
            w.leaf("Field", &f);
        }
        w.close("UseAlways");
        p.push("UseAlways", w);
    }

    /// Поля реквизита `{0,N,{K,часть...}...}` (UseAlways, Save): пути от самого реквизита.
    fn fields(&self, p: &mut P, tag: &'static str, n: Option<&Node>, name: &str, id: &str) {
        let Some(n) = n.filter(|n| num_at(n, 1) > 0) else { return };
        let class = self.attrs.get(id).map_or("", |a| a.1.as_str());
        // Прочие поля настроек компоновщика (`{2,{0},{код}}`) встречались только все 11 разом (НастройкаЗаполнения*ФормСтатистики,
        // 3 формы): какой код какому имени отвечает, не установлено, но поля пишутся по алфавиту — набор пишется целиком.
        const REST: [&str; 11] = ["0", "5", "6", "10000", "10001", "10002", "10003", "10004", "10005", "10006", "10015"];
        const REST_NAMES: [&str; 11] = [
            "DataParameters", "HasConditionalAppearance", "HasFilter", "HasOrder", "HasOutputParameters", "HasSelection",
            "OutputParameters", "ReportStructure", "ReportStructurePicture", "Use", "UserFields",
        ];
        fn rest(f: &Node) -> Option<&str> {
            match f.list() {
                [k, Node::List(a), Node::List(b)] if k.atom() == Some("2") && a.len() == 1 && a[0].atom() == Some("0") && b.len() == 1 => {
                    b[0].atom().filter(|c| REST.contains(c))
                }
                _ => None,
            }
        }
        let found: HashSet<&str> = n.list().iter().skip(2).filter_map(rest).collect();
        let settings = (found.len() == REST.len())
            .then(|| self.sub_path(name.to_string(), class, id, &[Node::List(vec![Node::Atom("0".into())])]))
            .flatten();
        let mut v: Vec<String> = settings.iter().flat_map(|s| REST_NAMES.iter().map(move |r| format!("{s}.{r}"))).collect();
        v.extend(n.list().iter().skip(2).filter(|f| settings.is_none() || rest(f).is_none()).map(|f| {
            let k = num_at(f, 0).max(0) as usize;
            let parts = f.list().get(1..1 + k).unwrap_or(&[]);
            let mut all = vec![Node::List(vec![Node::Atom(id.to_string())])];
            all.extend(parts.iter().cloned());
            self.sub_path(name.to_string(), class, id, parts).or_else(|| self.lost(&all)).unwrap_or_else(|| unk("поле", f))
        }));
        // Платформа пишет поля по алфавиту.
        v.sort();
        let mut w = p.w();
        w.open(tag);
        for s in v {
            w.leaf("Field", &s);
        }
        w.close(tag);
        p.push(tag, w);
    }

    /// Условное оформление: настройки СКД в base64; в Form.xml — те же элементы с префиксом dcsset.
    fn conditional_appearance(&self, n: Option<&Node>) -> Result<String, String> {
        let Some(b) = n.and_then(|n| n.get(0)).and_then(Node::atom).and_then(|a| a.strip_prefix("#base64:")) else {
            return Ok(String::new());
        };
        let x = String::from_utf8_lossy(&crate::xml::base64(b)?).into_owned();
        // Без оформления там бывает и не XML (`AA==`).
        if !x.contains("<conditionalAppearance>") {
            return Ok(String::new());
        }
        let st = dcs::parse(&x)?;
        let Some(ca) = st.elements().find(|e| e.local() == "conditionalAppearance").filter(|e| e.elements().next().is_some()) else {
            return Ok(String::new());
        };
        let names = self.names;
        let style = |g: &str| names.types.get(g).map(|t| format!("cfg:{t}")).or_else(|| names.short(g).map(|s| format!("style:{s}")));
        let d = self.w.depth + 1;
        let mut out = format!("\r\n{}<ConditionalAppearance>", "\t".repeat(d));
        for e in ca.elements() {
            dcs::write_in(&mut out, Some(&st), e, None, d + 1, &style);
        }
        out += &format!("\r\n{}</ConditionalAppearance>", "\t".repeat(d));
        Ok(out)
    }
}

/// Колонка `{5,id,0,"имя",Title,Type,View,Edit,ФО,?}`.
fn column(w: &mut W, names: &Names, c: &Node) {
    let id = atom_at(c, 1);
    w.line(&format!("<Column name=\"{}\" id=\"{id}\">", esc_attr(c.get(3).and_then(Node::str).unwrap_or(""))));
    let mut p = P::new(w.depth + 1);
    p.local("Title", c.get(4).unwrap_or(&EMPTY));
    let mut t = p.w();
    type_xml(&mut t, names, c.get(5).unwrap_or(&EMPTY));
    p.push("Type", t);
    p.adjustable("View", c.get(6), names);
    p.adjustable("Edit", c.get(7), names);
    p.functional_options(c.get(8), names);
    match num_at(c, 9) {
        0 => {}
        1 => p.leaf("FillCheck", "ShowError"),
        _ => p.leaf("\u{1}колонка", &unk("9", c)),
    }
    w.depth += 1;
    // Место FillCheck — как у реквизита (в БП рядом с View/Edit/FunctionalOptions не встречалась).
    p.emit(w, &["Title", "Type", "View", "Edit", "FillCheck", "FunctionalOptions"]);
    w.close("Column");
}

/// Настройки реквизита `{0,N,("ключ",значение)...}`: тип элементов списка значений; остальное пока меткой.
fn settings(w: &mut W, names: &Names, n: &Node) {
    if num_at(n, 1) == 1 && n.get(2).and_then(Node::str) == Some("ElementType") {
        let mut t = W { s: String::new(), depth: w.depth + 1 };
        type_inner(&mut t, names, n.get(3).and_then(|v| v.get(2)).unwrap_or(&EMPTY));
        if t.s.is_empty() {
            w.line("<Settings xsi:type=\"v8:TypeDescription\"/>");
        } else {
            w.line("<Settings xsi:type=\"v8:TypeDescription\">");
            w.s.push_str(&t.s);
            w.line("</Settings>");
        }
        return;
    }
    if n.get(2).and_then(Node::str) == Some("QueryText") {
        dynamic_list(w, names, n.list().get(2..).unwrap_or(&[]));
        return;
    }
    // Пустые настройки списка у таблицы значений (ЕдиныйНалоговыйСчетЛичныйКабинет.ТаблицаОперацииЕНП, 10 в БП) — в XML
    // их нет; другие значения (и вариант с ServerState) — отказ.
    const EMPTY_LIST: &str = r##"{0,5,"KeyType",{"N",0},"KeyFields",{"#",51e7a0d2-530b-11d4-b98a-008048da3034,{0}},"AutoFillAvailableFields",{"B",1},"FieldsMapSecondaryNamesLoaded",{"B",1},"GetInvisibleFieldPresentations",{"B",1}}"##;
    if brace::write(n).is_ok_and(|s| s.replace("\r\n", "") == EMPTY_LIST) || settings::server_state_list(n, EMPTY_LIST) {
        return;
    }
    // Только старое соответствие полей `FiledsMapItemIdK`/`FiledsMapItemNameK` (таблица значений,
    // СканированныеДокументы...ФормаЭлементаНовая) — в XML настроек нет.
    let kv = n.list().get(2..).unwrap_or(&[]);
    if !kv.is_empty() && kv.chunks(2).all(|c| c[0].str().is_some_and(|k| k.starts_with("FiledsMapItemId") || k.starts_with("FiledsMapItemName"))) {
        return;
    }
    let key = n.get(2).and_then(Node::str).unwrap_or("");
    let r = match key {
        "Moxel" if num_at(n, 1) == 1 => settings::moxel(w, names, n.get(3).unwrap_or(&EMPTY)),
        "Planner" if num_at(n, 1) == 1 => settings::planner(w, n.get(3).unwrap_or(&EMPTY)),
        "Chart" if num_at(n, 1) == 1 => settings::chart(w, n.get(3).unwrap_or(&EMPTY)),
        "GanttChart" if num_at(n, 1) == 1 => settings::gantt(w, n.get(3).unwrap_or(&EMPTY)),
        // Диаграмма, планировщик... пока не разбираются.
        _ => Err(String::new()),
    };
    if let Err(e) = r {
        w.leaf("Settings", &format!("\u{1}настройки:{key}{}", if e.is_empty() { e } else { format!(": {e}") }));
    }
}

/// Ключи настроек динамического списка, которые в XML не попадают или разбираются ниже.
fn dl_known(k: &str) -> bool {
    let base = k.trim_end_matches(|c: char| c.is_ascii_digit());
    matches!(
        base,
        "QueryText" | "MainTable" | "MainTableCategory" | "KeyType" | "KeyFields" | "DynamicalDataSelection" | "AutoFillAvailableFields"
            | "ManualQuery" | "FieldsMapItemId" | "FieldsMapItemName" | "FieldsMapItemSecondaryName" | "FiledsMapItemId"
            | "FiledsMapItemName" | "FieldsMapSecondaryNamesLoaded" | "ReqMapFieldId" | "AutoSaveUserSettings"
            | "GetInvisibleFieldPresentations" | "ServerState" | "DataParameters" | "Order" | "Filter" | "Appearance" | "Group"
            | "GroupSelectedSettingId" | "GroupSelectedSettingViewMode" | "GroupSelectedSettingPresentation"
    )
}

/// Настройки динамического списка `("ключ",{тип,значение})...` → `<Settings xsi:type="DynamicList">`.
/// Порядок свойств — как у платформы; настройки СКД лежат XML-ом в base64.
fn dynamic_list(w: &mut W, names: &Names, kv: &[Node]) {
    let get = |k: &str| kv.chunks(2).find(|c| c[0].str() == Some(k)).and_then(|c| c.get(1));
    let val = |k: &str| get(k).and_then(|v| v.get(1));
    let flag = |k: &str| val(k).and_then(Node::atom).map(|a| a == "1");
    let xml = |k: &str| -> Option<Result<dcs::El, String>> {
        let b = get(k)?.get(2)?.get(0)?.atom()?.strip_prefix("#base64:")?;
        Some(crate::xml::base64(b).and_then(|b| dcs::parse(&String::from_utf8_lossy(&b))))
    };
    let style = |g: &str| names.types.get(g).map(|t| format!("cfg:{t}")).or_else(|| names.short(g).map(|s| format!("style:{s}")));
    w.line("<Settings xsi:type=\"DynamicList\">");
    w.depth += 1;
    if flag("AutoFillAvailableFields") == Some(false) {
        w.leaf("AutoFillAvailableFields", "false");
    }
    w.leaf("ManualQuery", bool_text(flag("ManualQuery") == Some(true)));
    // Флаг хранится обратным: «динамическая выборка выключена».
    w.leaf("DynamicDataRead", bool_text(flag("DynamicalDataSelection") != Some(true)));
    if let Some(q) = val("QueryText").and_then(Node::str).filter(|q| !q.is_empty()) {
        w.leaf("QueryText", q);
    }
    if let Some(b) = val("ServerState").and_then(Node::str) {
        let st = crate::xml::base64(b).and_then(|b| dcs::server_state(&b)).and_then(|x| dcs::parse(&x));
        match st {
            Ok(st) => {
                // Поля, вычисляемые поля, параметры — группами, как у платформы.
                for (src, dst) in [("Field", None), ("ExpressionField", Some("CalculatedField")), ("Parameter", None)] {
                    for e in st.elements().filter(|e| e.local() == src) {
                        let mut e = e.clone();
                        // У вычисляемых полей и параметров в форме xsi:type не пишется.
                        if dst.is_some() || src == "Parameter" {
                            e.attrs.retain(|(k, _)| k != "xsi:type");
                        }
                        dcs::write_in(&mut w.s, Some(&st), &e, dst, w.depth, &style);
                    }
                }
                for e in st.elements().filter(|e| !matches!(e.local(), "Field" | "ExpressionField" | "Parameter")) {
                    w.leaf("\u{1}ServerState", e.local());
                }
            }
            Err(e) => w.leaf("\u{1}ServerState", &e),
        }
    }
    if let Some(g) = get("MainTable").map(|t| atom_at(t, 2)).filter(|g| !g.is_empty() && *g != ZERO_GUID) {
        let mut t = names.full.get(g).map_or_else(|| format!("\u{1}{g}"), String::clone);
        // Виртуальная таблица — по MainTableCategory (сверка БП 3.0.171: 0 и 1 — сама таблица).
        let cat = val("MainTableCategory").and_then(Node::atom).unwrap_or("1");
        match (t.split('.').next().unwrap_or(""), cat) {
            (_, "0" | "1") => {}
            ("AccountingRegister", "3") => t += ".RecordsWithExtDimensions",
            ("AccountingRegister", "4") | ("AccumulationRegister", "3") => t += ".Balance",
            ("Task", "2") => t += ".TasksByExecutive",
            // basic.dll FUN_1801db9f0: у регистра сведений виртуальные 1 SliceFirst, 2 SliceLast; категория — на 1 больше
            ("InformationRegister", "2") => t += ".SliceFirst",
            ("InformationRegister", "3") => t += ".SliceLast",
            _ => t += &format!(".\u{1}категория{cat}"),
        }
        w.leaf("MainTable", &t);
    }
    match val("KeyType").and_then(Node::atom).unwrap_or("0") {
        "0" => {}
        k => w.leaf("KeyType", enum_name(&["", "FieldValue", "RowKey", "RowNumber"], k.parse().unwrap_or(-1)).unwrap_or(&format!("\u{1}{k}"))),
    }
    if let Some(f) = get("KeyFields").and_then(|v| v.get(2)) {
        for x in f.list().iter().skip(1).take(num_at(f, 0).max(0) as usize) {
            w.leaf("KeyField", x.get(1).and_then(Node::str).unwrap_or("\u{1}"));
        }
    }
    if flag("AutoSaveUserSettings") == Some(false) {
        w.leaf("AutoSaveUserSettings", "false");
    }
    if flag("GetInvisibleFieldPresentations") == Some(false) {
        w.leaf("GetInvisibleFieldPresentations", "false");
    }
    w.open("ListSettings");
    let d = w.depth;
    // Без ключа — настройки по умолчанию с постоянными идентификаторами пользовательских настроек.
    for (key, tag, user_id) in [
        ("Filter", "filter", Some("dfcece9d-5077-440b-b6b3-45a5cb4538eb")),
        ("DataParameters", "dataParameters", None),
        ("Order", "order", Some("88619765-ccb3-46c6-ac52-38e9c992ebd4")),
        ("Appearance", "conditionalAppearance", Some("b75fecce-942b-4aed-abc9-e6a02e460fb3")),
    ] {
        let from_xml = matches!(xml(key), Some(Ok(_)));
        let mut e = match xml(key) {
            Some(Ok(e)) => e,
            Some(Err(e)) => {
                w.leaf("\u{1}ListSettings", &format!("{key}: {e}"));
                continue;
            }
            None => match user_id {
                Some(id) => dcs::El { name: tag.into(), attrs: vec![("xmlns".into(), DCSSET.into())], kids: vec![dcs::Kid::El(text_el("userSettingID", id))] },
                None => continue,
            },
        };
        if (user_id.is_none() || from_xml) && e.elements().next().is_none() {
            continue;
        }
        // Режим просмотра — только из записи (XML настройки в base64); дописывается лишь у настройки по умолчанию (ключа
        // в записи нет). Каркас 4 466: ни одной формы с потерянным viewMode; БП, 1 787 форм с ListSettings, — тоже.
        if !from_xml {
            let at = e.kids.iter().position(|k| matches!(k, dcs::Kid::El(x) if x.local().starts_with("userSetting"))).unwrap_or(e.kids.len());
            e.kids.insert(at, dcs::Kid::El(text_el("viewMode", "Normal")));
        }
        dcs::write_root(&mut w.s, &e, Some(tag), d, &style);
    }
    match xml("Group") {
        Some(Ok(mut g)) if g.elements().next().is_some() => {
            // группировки списка — цепочкой: каждое поле записи — своя вложенная группировка (ДеревоВидыЦен ERP)
            g.attrs.retain(|(k, _)| k.starts_with("xmlns"));
            let items: Vec<dcs::Kid> = std::mem::take(&mut g.kids).into_iter().filter(|k| matches!(k, dcs::Kid::El(_))).collect();
            let n = items.len();
            for (i, it) in items.into_iter().enumerate() {
                w.line("<dcsset:item xsi:type=\"dcsset:StructureItemGroup\">");
                w.depth += 1;
                let one = dcs::El { name: g.name.clone(), attrs: g.attrs.clone(), kids: vec![it] };
                dcs::write_root(&mut w.s, &one, Some("groupItems"), d + 1 + i, &style);
            }
            for _ in 0..n {
                w.depth -= 1;
                w.line("</dcsset:item>");
            }
        }
        Some(Err(e)) => w.leaf("\u{1}ListSettings", &format!("Group: {e}")),
        _ => {}
    }
    match val("GroupSelectedSettingViewMode").and_then(Node::atom) {
        None | Some("0") => w.leaf("dcsset:itemsViewMode", "Normal"),
        Some("1") => {}
        Some(m) => w.leaf("dcsset:itemsViewMode", &format!("\u{1}{m}")),
    }
    match val("GroupSelectedSettingId").and_then(Node::str) {
        None => w.leaf("dcsset:itemsUserSettingID", "911b6018-f537-43e8-a417-da56b22f9aec"),
        Some("") => {}
        Some(id) => w.leaf("dcsset:itemsUserSettingID", id),
    }
    if let Some(p) = get("GroupSelectedSettingPresentation").and_then(|v| v.get(2)).filter(|p| num_at(p, 0) > 0) {
        local_typed(w, "dcsset:itemsUserSettingPresentation", p);
    }
    if w.s.ends_with("<ListSettings>") {
        w.s.truncate(w.s.len() - "<ListSettings>".len());
        w.s.push_str("<ListSettings/>");
        w.depth -= 1;
    } else {
        w.close("ListSettings");
    }
    for k in kv.chunks(2).filter_map(|c| c[0].str()).filter(|k| !dl_known(k)) {
        w.leaf("\u{1}DynamicList", k);
    }
    w.depth -= 1;
    w.line("</Settings>");
}

const DCSSET: &str = "http://v8.1c.ru/8.1/data-composition-system/settings";
const ZERO_GUID: &str = "00000000-0000-0000-0000-000000000000";

fn text_el(name: &str, t: &str) -> dcs::El {
    dcs::El { name: name.into(), attrs: Vec::new(), kids: vec![dcs::Kid::Text(t.into())] }
}

/// Многоязычная строка с `xsi:type="v8:LocalStringType"` (в настройках СКД).
/// Пустой вариант платформа не пишет; без вариантов — пустой элемент (БП 3.0.107 ВводДанныхДляРасчетаСреднегоЗаработкаОбщий).
fn local_typed(w: &mut W, tag: &str, n: &Node) {
    let l = n.list();
    let ks: Vec<usize> = (0..num_at(n, 0).max(0) as usize).filter(|k| l.get(2 + 2 * k).and_then(Node::str) != Some("")).collect();
    if ks.is_empty() {
        w.line(&format!("<{tag} xsi:type=\"v8:LocalStringType\"/>"));
        return;
    }
    w.line(&format!("<{tag} xsi:type=\"v8:LocalStringType\">"));
    w.depth += 1;
    for k in ks {
        w.open("v8:item");
        w.leaf("v8:lang", l.get(1 + 2 * k).and_then(Node::str).unwrap_or("\u{1}"));
        w.leaf("v8:content", l.get(2 + 2 * k).and_then(Node::str).unwrap_or("\u{1}"));
        w.close("v8:item");
    }
    w.depth -= 1;
    w.line(&format!("</{tag}>"));
}

/// Многоязычная строка `{1,N,{"ru","текст"},...}`; пустая (`{1,0}`) не пишется.
fn local(w: &mut W, tag: &str, n: &Node) {
    let l = n.list();
    let cnt = num_at(n, 1).max(0) as usize;
    if cnt == 0 {
        return;
    }
    w.open(tag);
    for p in l.iter().skip(2).take(cnt) {
        w.open("v8:item");
        w.leaf("v8:lang", p.get(0).and_then(Node::str).unwrap_or(""));
        w.leaf("v8:content", p.get(1).and_then(Node::str).unwrap_or(""));
        w.close("v8:item");
    }
    w.close(tag);
}

/// События `{N,(guid,"обработчик")×N,1,0,(guid,0,1)×N}`; пустые обработчики платформа не пишет.
fn events_xml(w: &mut W, nodes: &[&Node], name_of: &dyn Fn(&str) -> Option<String>) -> Result<(), String> {
    let mut out = Vec::new();
    for (n, k) in nodes.iter().flat_map(|n| (0..num_at(n, 0).max(0) as usize).map(move |k| (*n, k))) {
        let g = atom_at(n, 1 + 2 * k);
        let h = n.get(2 + 2 * k).and_then(Node::str).ok_or("события: нет обработчика")?;
        if h.is_empty() {
            continue;
        }
        let name = name_of(g).unwrap_or_else(|| format!("\u{1}{g}"));
        out.push((name, h));
    }
    if out.is_empty() {
        return Ok(());
    }
    w.open("Events");
    for (name, h) in out {
        w.line(&format!("<Event name=\"{name}\">{}</Event>", esc(h)));
    }
    w.close("Events");
    Ok(())
}

/// GUID события формы → имя в XML (из сверки с выгрузкой платформы).
fn form_event(g: &str) -> Option<&'static str> {
    Some(match g {
        "047d4d09-961c-4bdc-8519-eef10674c35b" => "AfterWrite",
        "213d1900-dcad-4616-9f20-3f077156a40f" => "AfterWriteAtServer",
        "52dbb775-1631-4fd5-8c55-1615b5881dac" => "BeforeClose",
        "e773807c-0c0c-4689-a093-231ddcd6409f" => "BeforeLoadDataFromSettingsAtServer",
        "40925042-2517-455b-a600-d68282829334" => "BeforeLoadUserSettingsAtServer",
        "1dd89674-8b50-4240-9899-e3426b79cb02" => "BeforeLoadVariantAtServer",
        "9cc34712-da5f-4faa-a653-343d2085fbe8" | "8a5894c9-d2ff-4c1d-b433-89cc352bbfbc" => "BeforeWrite",
        "bf0ac0e1-bcbb-4dfe-8fc4-0b1923b461a6" | "8f42e083-be92-4102-b1f0-fa58452c1a63" => "BeforeWriteAtServer",
        "1d632984-de3c-4b4b-ad9f-d69682a10182" => "ChoiceProcessing",
        "5426e344-5740-4f23-99c1-99179a200dc5" => "ExternalEvent",
        "e73d6384-49d2-4885-a752-a674d6ff7742" => "FillCheckProcessingAtServer",
        "93dfba16-26db-46f8-acb5-4f92f50c855f" => "NavigationProcessing",
        "3b644f4f-055f-4808-bdc6-a50ce895e4d9" => "NewWriteProcessing",
        "3699f6a3-9a2a-4c82-a775-6ff4824a08ca" => "NotificationProcessing",
        "ca21cd18-35b2-4281-b5c8-016ecc8da8ac" => "OnClose",
        "9f2e5ddb-3492-4f5d-8f0d-416b8d1d5c5b" => "OnCreateAtServer",
        "79cea13e-f6fb-4483-905d-713326405771" => "OnLoadDataFromSettingsAtServer",
        "7b15b3db-1cd0-4e1d-a74b-2c972c9e2226" => "OnLoadUserSettingsAtServer",
        "87ce636e-9de6-4e42-9395-f0f189d08397" => "OnLoadVariantAtServer",
        "d6b86f20-722b-4fe6-83fa-85c6aa4c1fe5" => "OnMainServerAvailabilityChange",
        "3ccc650e-f631-4cae-8e33-3eaac610b5f9" => "OnOpen",
        "390d5e4b-e732-4c88-8748-9e211a416984" => "OnReadAtServer",
        "6b3175a5-c143-4179-a670-ef231dc0a688" => "OnReopen",
        "1952a54f-35ad-4928-902f-df212ab38ca3" => "OnSaveDataInSettingsAtServer",
        "961ee7c6-0327-422b-adcb-97a90c46753d" => "OnSaveUserSettingsAtServer",
        "499bb7af-6262-4de4-819f-ef264d1a20ec" => "OnSaveVariantAtServer",
        "d817bccf-504e-4133-a79a-dd16e3a4df73" => "OnUpdateUserSettingSetAtServer",
        "c1bc0d3e-d35e-4207-a06b-ece68ed25314" => "OnWriteAtServer",
        "674956b3-e469-4fdc-acf5-24ebf88cf7ab" => "URLGetProcessing",
        "44498116-1641-4bfa-ae33-86e53c205797" => "URLListGetProcessing",
        "e0cd9bdf-88fa-428c-9f1f-86f7f73b11e2" => "URLProcessing",
        // прочие — по таблице событий DLL
        _ => return super::events_table::EVENTS.iter().find(|(id, ..)| *id == g).map(|(_, n, _)| *n),
    })
}

const DYNAMIC_LIST: &str = "65abad24-838b-4987-8b35-ed9e2bd4d9c8";

/// GUID платформенного типа → имя в XML.
fn platform_type(g: &str) -> Option<&'static str> {
    Some(match g {
        "acf6192e-81ca-46ef-93a6-5a6968b78663" => "v8:ValueTable",
        "e603c0f2-92fb-4d47-8f38-a44a381cf235" => "v8:ValueTree",
        "e603103e-a318-4edc-a014-b1c6cf94d49f" => "mxl:SpreadsheetDocument",
        "4772b3b4-f4a3-49c0-a1a5-8cb5961511a3" => "v8:ValueListType",
        "fc01b5df-97fe-449b-83d4-218a090e681e" => "v8:UUID",
        "140b5ff4-37b1-4df5-b5ec-a0bfd2b94f8f" => "v8ui:FormattedString",
        "9cd510c7-abfc-11d4-9434-004095e12fc7" => "v8ui:Color",
        "9cd510c8-abfc-11d4-9434-004095e12fc7" => "v8ui:Font",
        "ebf766b1-f32c-11d3-9851-008048da1252" => "d5p1:TextDocument",
        "151f8778-e2d0-496a-9f02-d9ffd93b57ec" => "fd:FormattedDocument",
        "48510817-200c-48c2-9973-06cf90840514" => "pdfdoc:PDFDocument",
        "3543ef08-3316-4f7e-9447-0cd0a1cbf1d5" => "d5p1:Chart",
        "43dc7f37-5b1d-42a7-8f28-f545080d0255" => "pl:Planner",
        "4af83795-fc2a-48cd-9bea-ce665789a62c" => "d5p1:FlowchartContextType",
        "e6f51714-91cb-4dce-94fe-90ae3e3e1ad1" => "v8ui:Picture",
        "f5c65050-3bbb-11d5-b988-0050bae0a95d" => "v8:TypeDescription",
        "2fdc88ec-7c9b-43cd-8ba5-873f043bdd88" => "v8:StandardPeriod",
        "cab0d12b-3c88-4993-8edc-8c3827cadc7d" => "dcsset:SettingsComposer",
        "52616226-8ccf-4d1d-a3da-827eeb4f9cf9" => "v8ui:VerticalAlign",
        "7a0135c7-5b77-4de2-839e-25c5e63a2142" => "v8ui:SizeChangeMode",
        "dcfc3784-a14f-4786-ac7b-c82db5ba275f" => "cfg:ConstantsSet",
        "1dd6fdb9-553d-40d4-b2d1-c7fc31f497bb" => "cfg:ReportObject",
        "f6841c6b-6c71-4c82-ae9e-d08b49db326c" => "dcsset:Filter",
        "98ea8e5a-b586-442b-b944-6e3447734aa7" => "v8:FillChecking",
        "872f7198-7083-4e3e-b57e-a2a9802c769e" => "ent:AccountType",
        "dcbf2698-3c1f-4a22-997f-48070ae9bd64" => "dcsset:DataCompositionComparisonType",
        "a090004e-b706-453f-aa10-090a77b53757" => "dcsset:DataCompositionFieldPlacement",
        "913e8016-6e90-47a0-b2a0-4513f4edad61" => "dcscor:Field",
        "af4a19b5-da3d-406f-be0c-81143e400452" => "dcscor:DataCompositionSortDirection",
        // Колонки ФормаНастроекОтчета (по одной неизвестной на колонку).
        "0e0850cf-0634-414e-85ba-9a88a8bd44c4" => "dcscor:DataCompositionGroupType",
        "c6a52555-d20f-452c-bfc2-1b53e9a56063" => "dcscor:DataCompositionPeriodAdditionType",
        // ERP 2.5.22: по единственной незнакомой на форму (БизнесРегионы, ОперацияМеждународный, ЭлементыФинансовыхОтчетов)
        "95de81b0-81c3-4936-9dbb-6400e5c90378" => "d5p1:GeographicalSchema",
        // XDTO chart.dll (сериализатор по GUID)
        "3a6e63bf-16aa-42eb-b48c-2fff9670ad2f" => "d5p1:GanttChart",
        "741ae838-6e42-4ac0-b6a4-17e5604b0669" => "ent:AccountingRecordType",
        "43f9c095-40e8-441a-8fad-20a45798c71b" => "v8ui:HorizontalAlign",
        "0387f3a2-7df5-4804-948b-4580a51e4a15" => "v8:StandardBeginningDate",
        DYNAMIC_LIST => "cfg:DynamicList",
        // прочие — по таблице XDTO, снятой с DLL
        _ => return super::xdto_table::XDTO.iter().find(|(id, ..)| *id == g).map(|(_, t, _)| *t),
    })
}

/// Наборы типов `{"#",guid}` → `v8:TypeSet`.
fn platform_type_set(g: &str) -> Option<&'static str> {
    Some(match g {
        "e61ef7b8-f3e1-4f4b-8ac7-676e90524997" => "cfg:CatalogRef",
        "38bfd075-3e63-4aaa-a93e-94521380d579" => "cfg:DocumentRef",
        "474c3bf6-08b5-4ddc-a2ad-989cedf11583" => "cfg:EnumRef",
        "0a52f9de-73ea-4507-81e8-66217bead73a" => "cfg:ExchangePlanRef",
        "214fa4d8-6ba4-4748-a5e1-6332b5887780" => "cfg:BusinessProcessRef",
        "280f5f0e-9c8a-49cc-bf6d-4d296cc17a63" => "cfg:AnyIBRef",
        // Прочие «любые ссылки» — по таблице описаний объектов (TaskRef, ChartOf*Ref).
        _ => return super::common::platform_type(g).filter(|(el, _)| *el == "v8:TypeSet").map(|x| x.1),
    })
}

/// Тип вне пространств имен шапки: платформа объявляет пространство прямо в элементе.
fn type_leaf(w: &mut W, t: &str) {
    let ns = match t.split(':').next() {
        Some("mxl") => "http://v8.1c.ru/8.2/data/spreadsheet",
        Some("fd") => "http://v8.1c.ru/8.2/data/formatted-document",
        Some("pdfdoc") => "http://v8.1c.ru/8.3/data/pdf",
        Some("pl") => "http://v8.1c.ru/8.3/data/planner",
        Some("d5p1") => match t {
            "d5p1:TextDocument" => "http://v8.1c.ru/8.1/data/txtedt",
            "d5p1:Chart" | "d5p1:GanttChart" => "http://v8.1c.ru/8.2/data/chart",
            "d5p1:GeographicalSchema" => "http://v8.1c.ru/8.2/data/geo",
            _ => super::xdto_table::XDTO.iter().find(|(_, x, _)| *x == t).map_or("http://v8.1c.ru/8.2/data/graphscheme", |(.., ns)| *ns),
        },
        _ => return w.leaf("v8:Type", t),
    };
    let p = t.split(':').next().unwrap_or("");
    w.line(&format!("<v8:Type xmlns:{p}=\"{ns}\">{t}</v8:Type>"));
}

/// Описание типов `{"Pattern",{"S",длина,переменная},{"N",цифры,дробь,неотрицательное},{"B"},{"D","D"},{"#",guid}...}`.
fn type_xml(w: &mut W, names: &Names, p: &Node) {
    let mut t = W { s: String::new(), depth: w.depth + 1 };
    type_inner(&mut t, names, p);
    if t.s.is_empty() {
        w.line("<Type/>");
    } else {
        w.line("<Type>");
        w.s.push_str(&t.s);
        w.line("</Type>");
    }
}

/// Содержимое описания типов без обертки.
fn type_inner(w: &mut W, names: &Names, p: &Node) {
    let l = p.list();
    if l.len() <= 1 {
        return;
    }
    let (mut num, mut strq, mut date) = (None, None, None);
    // Наборы типов платформа пишет после всех типов (в эталоне БП ни разу не наоборот).
    let mut sets = W { s: String::new(), depth: w.depth };
    for t in &l[1..] {
        match t.get(0).and_then(Node::str) {
            Some("S") => {
                w.leaf("v8:Type", "xs:string");
                strq = Some((atom_at(t, 1).to_string(), atom_at(t, 2).to_string()));
            }
            Some("N") => {
                w.leaf("v8:Type", "xs:decimal");
                num = Some((atom_at(t, 1).to_string(), atom_at(t, 2).to_string(), atom_at(t, 3).to_string()));
            }
            Some("B") => w.leaf("v8:Type", "xs:boolean"),
            Some("L") => w.leaf("v8:Type", "v8:Null"),
            Some("D") => {
                w.leaf("v8:Type", "xs:dateTime");
                date = Some(t.get(1).and_then(Node::str).unwrap_or("").to_string());
            }
            Some("#") => {
                let g = atom_at(t, 1);
                match (platform_type(g), names.types.get(g), platform_type_set(g)) {
                    (Some(x), _, _) => type_leaf(w, x),
                    // Определяемый тип и характеристика — наборы типов.
                    (None, Some(x), _) if x.starts_with("DefinedType.") || x.starts_with("Characteristic.") => sets.leaf("v8:TypeSet", &format!("cfg:{x}")),
                    (None, Some(x), _) => w.leaf("v8:Type", &format!("cfg:{x}")),
                    (None, None, Some(x)) => sets.leaf("v8:TypeSet", x),
                    _ => w.leaf("v8:Type", &unk("тип", t)),
                }
            }
            _ => w.leaf("v8:Type", &unk("тип", t)),
        }
    }
    w.s.push_str(&sets.s);
    if let Some((d, f, s)) = num {
        w.open("v8:NumberQualifiers");
        w.leaf("v8:Digits", if d.is_empty() { "0" } else { &d });
        w.leaf("v8:FractionDigits", if f.is_empty() { "0" } else { &f });
        w.leaf("v8:AllowedSign", if s == "1" { "Nonnegative" } else { "Any" });
        w.close("v8:NumberQualifiers");
    }
    if let Some((len, var)) = strq {
        w.open("v8:StringQualifiers");
        w.leaf("v8:Length", if len.is_empty() { "0" } else { &len });
        w.leaf("v8:AllowedLength", if var == "0" { "Fixed" } else { "Variable" });
        w.close("v8:StringQualifiers");
    }
    if let Some(d) = date {
        w.open("v8:DateQualifiers");
        w.leaf("v8:DateFractions", match d.as_str() {
            "D" => "Date",
            "T" => "Time",
            _ => "DateTime",
        });
        w.close("v8:DateQualifiers");
    }
}

// ---------------------------------------------------------------- элементы формы

const GROUP: &str = "cd5394d0-7dda-4b56-8927-93ccbe967a01";
const DECORATION: &str = "3d3cb80c-508b-41fa-8a18-680cdf5f1712";

/// Элемент после вставки «видимости по ролям»: при `[4]=1` за флагом идет значение, остальное сдвинуто на 1.
struct It<'a> {
    l: &'a [Node],
    off: usize,
    /// Позиция флага: у кнопок 3, у остальных 4.
    flag: usize,
}

impl<'a> It<'a> {
    fn new(n: &'a Node, flag: usize) -> It<'a> {
        let l = n.list();
        let off = usize::from(l.get(flag).and_then(Node::atom) == Some("1"));
        It { l, off, flag }
    }
    fn n(&self, i: usize) -> Option<&'a Node> {
        self.l.get(if i <= self.flag { i } else { i + self.off })
    }
    fn a(&self, i: usize) -> &'a str {
        self.n(i).and_then(Node::atom).unwrap_or("")
    }
    fn num(&self, i: usize) -> i64 {
        self.a(i).parse().unwrap_or(i64::MIN)
    }
    fn uv(&self) -> Option<&'a Node> {
        (self.off == 1).then(|| &self.l[self.flag + 1])
    }
    fn id(&self) -> &'a str {
        self.n(1).map_or("", |n| atom_at(n, 0))
    }
}

const TOOLTIP_REPR: &[&str] = &["Auto", "None", "Balloon", "Button", "ShowAuto", "ShowTop", "ShowLeft", "ShowBottom", "ShowRight"];
const SPACING: &[&str] = &["Auto", "None", "Half", "Single", "OneAndHalf", "Double"];
const H_ALIGN: &[&str] = &["Left", "Center", "Right", "Auto"];
const V_ALIGN: &[&str] = &["Top", "Center", "Bottom"];
// Код 3 — ItemsRightTitlesLeft (БП и каркас: 18 из 18), код 4 не встречался.
const CHILDREN_ALIGN: &[&str] = &["Auto", "None", "ItemsLeftTitlesLeft", "ItemsRightTitlesLeft", "ItemsLeftTitlesRight", "ItemsRightTitlesRight", "TitlesLeftDataAuto"];
const CHILD_WIDTH: &[&str] = &["Auto", "Equal", "LeftWide", "LeftWidest", "LeftNarrow", "LeftNarrowest"];
const GROUPING: &[&str] = &["Vertical", "Horizontal", "HorizontalIfPossible", "AlwaysHorizontal", "Auto", "AutoScreenTypeSensitive"];

static EMPTY: Node = Node::List(Vec::new());

impl P {
    /// Перечисление: код по умолчанию не пишется, незнакомый код — меткой.
    fn en(&mut self, name: &'static str, t: &[&str], code: i64, def: i64) {
        if code == def || code == i64::MIN {
            return;
        }
        match usize::try_from(code).ok().and_then(|c| t.get(c)).filter(|s| !s.is_empty()) {
            Some(v) => self.leaf(name, v),
            None => self.leaf(name, &format!("\u{1}{code}")),
        }
    }
    /// 0/1/2 = false/true/авто.
    fn tri(&mut self, name: &'static str, code: i64) {
        match code {
            0 => self.leaf(name, "false"),
            1 => self.leaf(name, "true"),
            _ => {}
        }
    }
    fn flag_if(&mut self, name: &'static str, cond: bool, text: &str) {
        if cond {
            self.leaf(name, text);
        }
    }
    fn num_nz(&mut self, name: &'static str, v: &str) {
        if !v.is_empty() && v != "0" {
            self.leaf(name, v);
        }
    }
    fn color(&mut self, name: &'static str, n: Option<&Node>, names: &Names) {
        if let Some(c) = n.and_then(|n| color(n, names)) {
            self.leaf(name, &c);
        }
    }
    fn font(&mut self, name: &'static str, n: Option<&Node>, names: &Names) {
        if let Some(f) = n.and_then(|n| font(n, names)) {
            self.items.push((name, format!("\r\n{}<{name}{f}/>", "\t".repeat(self.depth))));
        }
    }
    fn picture(&mut self, name: &'static str, n: Option<&Node>, names: &Names) {
        if let Some(n) = n {
            let mut w = self.w();
            picture(&mut w, name, n, names);
            self.push(name, w);
        }
    }
    fn raw(&mut self, name: &'static str, s: String) {
        if !s.is_empty() {
            self.items.push((name, s));
        }
    }
    /// Настраиваемое булево `{0,{0,{"B",общее},N,(роль,{"B",значение})...}}`.
    fn adjustable(&mut self, name: &'static str, n: Option<&Node>, names: &Names) {
        let Some(v) = n.and_then(|n| n.get(1)) else { return };
        let common = v.get(1).and_then(|b| b.get(1)).and_then(Node::atom) == Some("1");
        let cnt = num_at(v, 2).max(0) as usize;
        if common && cnt == 0 {
            return;
        }
        let mut w = self.w();
        w.open(name);
        w.leaf("xr:Common", bool_text(common));
        for k in 0..cnt {
            let role = atom_at(v, 3 + 2 * k);
            let val = v.get(4 + 2 * k).and_then(|b| b.get(1)).and_then(Node::atom) == Some("1");
            let rn = names.full.get(role).map_or(role, |s| s.as_str());
            w.line(&format!("<xr:Value name=\"{}\">{}</xr:Value>", esc_attr(rn), bool_text(val)));
        }
        w.close(name);
        self.push(name, w);
    }
    /// Рамка `{3,0,{0},стиль,толщина,0,guid}`: пишется, если отличается от рамки вида по умолчанию
    /// или задана явно (нулевой GUID вместо ссылки на стиль).
    fn border(&mut self, n: Option<&Node>, def_style: i64) {
        let Some(n) = n else { return };
        let (style, width) = (num_at(n, 3), atom_at(n, 4));
        if style == def_style && width == "1" && atom_at(n, 6) != "00000000-0000-0000-0000-000000000000" {
            return;
        }
        let name = match style {
            0 => "WithoutBorder",
            1 => "Single",
            2 => "Embossed",
            3 => "Indented",
            4 => "Underline",
            7 => "Overline",
            200 => "Double",
            _ => "\u{1}",
        };
        let mut w = self.w();
        w.line(&format!("<Border width=\"{width}\">"));
        w.depth += 1;
        w.line(&format!("<v8ui:style xsi:type=\"v8ui:ControlBorderType\">{name}</v8ui:style>"));
        w.close("Border");
        self.push("Border", w);
    }
    /// Функциональные опции `{0,N,guid...}`.
    fn functional_options(&mut self, n: Option<&Node>, names: &Names) {
        let Some(n) = n.filter(|n| num_at(n, 1) > 0) else { return };
        let mut w = self.w();
        w.open("FunctionalOptions");
        for g in n.list().iter().skip(2).filter_map(Node::atom) {
            // Ненайденная функциональная опция — GUID как есть, так пишет и платформа; нулевой — пустой `<Item/>`
            // (3 из 3 форм каркаса, платформа с нулевым GUID форму не загружает).
            let zero = g.trim_start_matches(['0', '-']).is_empty();
            w.leaf("Item", if zero { "" } else { names.full.get(g).map_or(g, String::as_str) });
        }
        w.close("FunctionalOptions");
        self.push("FunctionalOptions", w);
    }
    /// Многоязычная строка с признаком форматирования `{1,{строка},форматированная}`.
    fn formatted(&mut self, name: &'static str, t: Option<&Node>) {
        let Some(t) = t else { return };
        let Some(s) = t.get(1).filter(|s| num_at(s, 1) > 0) else {
            // Пустой заголовок пишется только форматированный.
            if num_at(t, 2) == 1 {
                let d = self.depth;
                self.raw(name, format!("\r\n{}<{name} formatted=\"true\"/>", "\t".repeat(d)));
            }
            return;
        };
        let mut w = self.w();
        local(&mut w, name, s);
        let f = bool_text(num_at(t, 2) == 1);
        w.s = w.s.replacen(&format!("<{name}>"), &format!("<{name} formatted=\"{f}\">"), 1);
        self.push(name, w);
    }
}

/// Цвета стиля `{3,3,{-N}}`, Windows `{3,1,{N}}`, web `{3,2,{N}}` — таблицы `mxl` (идентификаторы из ресурсов платформы).
pub(super) fn std_style_color(code: i64) -> Option<&'static str> {
    super::mxl::color_name(super::mxl::STYLE, code)
}

fn win_color(code: i64) -> Option<&'static str> {
    super::mxl::color_name(super::mxl::WINDOWS, code)
}

fn web_color(code: i64) -> Option<&'static str> {
    super::mxl::color_name(super::mxl::WEB, code)
}

/// Ссылка на элемент стиля: `{-N}` — стандартный, `{0,guid}` — из конфигурации.
fn style_ref(n: &Node, names: &Names, std: fn(i64) -> Option<&'static str>) -> String {
    let code = num_at(n, 0);
    if code < 0 {
        return std(code).map_or_else(|| format!("\u{1}style{code}"), |s| format!("style:{s}"));
    }
    let id = atom_at(n, 1);
    match names.short(id) {
        Some(s) => format!("style:{s}"),
        None if id.is_empty() => code.to_string(),
        None => format!("\u{1}style:{id}"),
    }
}

/// Цвет `{3,вид,{значение}}`; `{3,4,{0}}` — авто (не пишется).
fn color(n: &Node, names: &Names) -> Option<String> {
    let v = n.get(2)?;
    let code = num_at(v, 0);
    Some(match num_at(n, 1) {
        // «Авто» с остатком прежнего цвета (код или ссылка на стиль, РозничныеПродажи) 8.3.27 и старше пишут `auto`, 8.5 — нет (БП 3.0.107
        // МастерФормированияЗаявкиНаПодключениеУпрощенное: всплывающее меню).
        4 if names.older.is_some() && (code != 0 || v.list().len() > 1) => "auto".into(),
        4 => return None,
        0 => format!("#{:02X}{:02X}{:02X}", code & 0xff, (code >> 8) & 0xff, (code >> 16) & 0xff),
        1 => win_color(code).map_or_else(|| format!("\u{1}win{code}"), |s| format!("win:{s}")),
        2 => web_color(code).map_or_else(|| format!("\u{1}web{code}"), |s| format!("web:{s}")),
        3 => style_ref(v, names, std_style_color),
        _ => unk("цвет", n),
    })
}

pub(super) fn std_style_font(code: i64) -> Option<&'static str> {
    Some(match code {
        -20 => "TextFont",
        -30 => "SmallTextFont",
        -31 => "NormalTextFont",
        -32 => "LargeTextFont",
        -33 => "ExtraLargeTextFont",
        _ => return None,
    })
}

/// Шрифт `{7,вид,флаги,[ссылка],значения...,1,масштаб}` → атрибуты элемента; `{7,3,0,1,100}` — авто.
/// Флаги: 1 — гарнитура, 2 — размер, 4 — жирный, 8 — курсив, 16 — подчеркнутый, 32 — зачеркнутый, 512 — масштаб.
fn font(n: &Node, names: &Names) -> Option<String> {
    let l = n.list();
    let kind = num_at(n, 1);
    let flags = num_at(n, 2);
    if kind == 3 && flags == 0 {
        return None;
    }
    let b = |v: Option<&Node>| bool_text(v.and_then(Node::atom) == Some("1"));
    if kind == 0 {
        // Абсолютный шрифт: все поля на месте.
        let face = l.iter().rev().find_map(Node::str).unwrap_or("");
        return Some(format!(
            " faceName=\"{}\" height=\"{}\" bold=\"{}\" italic=\"{}\" underline=\"{}\" strikeout=\"{}\" kind=\"Absolute\" scale=\"{}\"",
            esc_attr(face),
            num_at(n, 3) / 10,
            bool_text(num_at(n, 7) >= 700),
            b(l.get(8)),
            b(l.get(9)),
            b(l.get(10)),
            atom_at(n, l.len() - 1)
        ));
    }
    let mut a = String::new();
    let start = match kind {
        1 => {
            // Системный шрифт `{N}`: 0 DefaultGUIFont, 2 ANSIFixedFont (ОплатаСервисаБП); прочие не встречались.
            match l.get(3).map_or("0", |x| atom_at(x, 0)) {
                "0" => a += " ref=\"sys:DefaultGUIFont\"",
                "2" => a += " ref=\"sys:ANSIFixedFont\"",
                k => a += &format!(" ref=\"\u{1}системный шрифт:{k}\""),
            }
            4
        }
        2 => {
            a += &format!(" ref=\"{}\"", esc_attr(&style_ref(l.get(3)?, names, std_style_font)));
            4
        }
        _ => 3,
    };
    let mut vals = l.get(start..).unwrap_or(&[]).iter();
    let mut take = |bit: i64| if flags & bit != 0 { vals.next() } else { None };
    let height = take(2).and_then(Node::atom).and_then(|x| x.parse::<i64>().ok());
    let bold = take(4).map(|v| v.atom() == Some("700"));
    let it = take(8).map(|v| b(Some(v)));
    let un = take(16).map(|v| b(Some(v)));
    let st = take(32).map(|v| b(Some(v)));
    let face = take(1).and_then(Node::str);
    if let Some(f) = face {
        a += &format!(" faceName=\"{}\"", esc_attr(f));
    }
    if let Some(h) = height {
        a += &format!(" height=\"{}\"", h / 10);
    }
    if let Some(x) = bold {
        a += &format!(" bold=\"{}\"", bool_text(x));
    }
    for (nm, v) in [("italic", it), ("underline", un), ("strikeout", st)] {
        if let Some(v) = v {
            a += &format!(" {nm}=\"{v}\"");
        }
    }
    a += match kind {
        1 => " kind=\"WindowsFont\"",
        2 => " kind=\"StyleItem\"",
        _ => " kind=\"AutoFont\"",
    };
    if flags & 512 != 0 {
        a += &format!(" scale=\"{}\"", atom_at(n, l.len() - 1));
    }
    Some(a)
}

/// Стандартные картинки по GUID.
/// Картинки по GUID, которых нет ни в конфигурации, ни среди стандартных: GUID не встречается ни в одном файле платформы
/// 8.5.1.1529 (стандартные лежат там в двоичном виде, например Back в core85.dll) — платформа пишет `0:guid`
/// (ПлатежноеПоручение, ПлатежноеТребование, СчетНаОплатуПокупателю).
/// В справке такая картинка остается ссылкой `../../mdpicture/id<guid>/…` (Клиент ЭДО, МакетыПечатныхФорм).
pub(crate) const NOT_STD_PICTURES: &[&str] = &["05f4dd91-2d70-4f80-accc-4f1980cba51a", "bb5c3a09-6d63-4e1c-8fe9-2310a267e732", "75378d9c-a364-4b46-a6d1-b745604c46a2",
    // БП 3.0.107: справка ЗадачаИсполнителя
    "c5b8bf21-5c8f-4e5b-8b85-bfac387ce99a"];

pub(crate) const FORM_STD_PICTURES: &[(&str, &str)] = &[
    ("5289d9a4-b012-4d54-9bce-50473fe29b57", "DialogExclamation"),
    ("8bdf1079-8fad-4d21-ad7f-4b2e4ecdce3d", "DialogInformation"),
    ("ef27ae9e-7040-4374-b93c-0d276de2ea23", "DialogQuestion"),
    ("83db1f8a-41bd-4016-bdb2-a28e3a8d6dcc", "DialogStop"),
    ("75a40cc4-c719-4c3f-91ea-fc5787bc34ca", "UserWithAuthentication"),
    ("20175a6d-7510-46a8-9077-75a5d0d97802", "AccountingRegister"),
    ("70f51581-87b6-41cb-a21b-c9dcdcc7fa93", "AccumulationRegister"),
    ("47f01799-7968-4f44-9acc-fe1bdde8beb2", "ActiveUsers"),
    ("2a0c2238-cb59-4473-ada6-352b60f3c0a9", "AddListItem"),
    ("1001ae3e-9289-4303-9699-3c0c17e20e61", "AddToFavorites"),
    ("7a9cd2fd-6372-4342-9a9e-3ebbd754fd83", "AppearanceCheckBox"),
    ("85998f14-805b-4e2b-ba19-9d79b0464042", "AppearanceCheckIcon"),
    ("b2202798-23e0-4165-9982-24878f432488", "AppearanceCross"),
    ("9ef73565-2250-4a35-9fb3-470bd19ca9ca", "AppearanceCrossIcon"),
    ("501b8c1d-8062-408e-bde8-b6549324713e", "AppearanceExclamationMark"),
    ("b39aa431-a32f-4447-984a-45606474c82d", "AppearanceExclamationMarkIcon"),
    ("4bf588d5-b8d8-47fd-8f41-eb9b668981ce", "SetListItemDeletionMark"),
    ("b0dd988f-2d9f-4364-b1f4-a4d5f45ffb78", "AppearanceFlagRed"),
    ("f6e88116-03d8-4400-9d88-791895d7031a", "Attach"),
    ("0c1f7756-6143-4903-a94c-8f22c85e44de", "Attribute"),
    ("3c904ff7-1195-4a7c-9a38-7b1f6ca49cce", "Back"),
    ("509c4a7f-6406-4388-bb8c-bc81fb5131aa", "BusinessProcess"),
    ("9fecbaff-2a05-4da6-9ef1-807e754b928d", "BusinessProcessStart"),
    ("fada8a16-8b14-4151-87a9-775099f37832", "Calculator"),
    ("9e808d29-787b-4825-863a-13c6844ce91d", "CancelSearch"),
    ("069b8324-7c51-4c73-a6a8-c06d4fc383b5", "Catalog"),
    ("97b2cc97-d5c6-45fb-9824-9d6d73db21fe", "Change"),
    ("dcd23a32-5c7c-43f2-9021-80d98128556f", "CheckSyntax"),
    ("7168f070-087c-448e-ae3a-6740f424076a", "ChooseFromList"),
    ("2f130057-bb2a-4e22-bba5-e108fac26940", "ChooseValue"),
    ("479470e0-ea0f-4266-8549-e2b1e8c06534", "ClearFilter"),
    ("448d6f55-d885-496c-870d-d1bd78374745", "CloneListItem"),
    ("f6532868-30b9-44ab-803c-78f0f0b06b02", "CloneObject"),
    ("1377931c-5744-4948-bade-cb35117b5f63", "Close"),
    ("27ee3053-952c-49e5-8261-9215098e0e9c", "CollapseAll"),
    ("4ab0e87f-7d9b-4aa8-ac4b-680a78522da8", "CreateFolder"),
    ("4d2570b5-205f-413c-b4cc-b2097f61684f", "CreateInitialImage"),
    ("977e831a-0e73-4d60-af51-091a6fa8612e", "CreateListItem"),
    ("6511326b-20c3-4bf8-8503-c2c2c9072c6c", "CustomizeForm"),
    ("f04794cb-c198-4172-86c3-649386013c85", "CustomizeList"),
    ("984b0a3e-daa2-4a9e-b75c-1f230a6e592a", "DataCompositionConditionalAppearance"),
    ("a075c3ef-bc4e-4c96-bdad-2245ec09c28e", "DataCompositionDataParameters"),
    ("d90a7482-9a1d-4d3d-ae96-6db440214d96", "DataCompositionFilter"),
    ("ad8cb448-a6bb-43b4-886a-7d6a8367eef2", "DataCompositionGroupFields"),
    ("732f9dd3-5baf-47ff-af7b-edfe16dac2a1", "DataCompositionNewChart"),
    ("6c2759b1-2b63-4fa5-8d13-75786c7e1e89", "DataCompositionNewTable"),
    ("efda7350-6cd7-4416-b188-f5ca9baf66c2", "DataCompositionOrder"),
    ("ee7c4a5b-2d9b-4087-ae3e-947792085f09", "DataCompositionOutputParameters"),
    ("2c732bfa-f734-48bc-a18b-7554db8a3888", "DataCompositionSelection"),
    ("affb1617-24bc-4170-9c84-0902cc3ef206", "DataCompositionSettingsWizard"),
    ("b68eb29c-2372-46e1-b84e-13843899ccf6", "DataCompositionUserFields"),
    ("a6cbfd77-fcf0-40f4-a8de-ee0d3e580fe6", "DataProcessor"),
    ("08a45a70-c221-4339-b3b1-9f11cb22147d", "Delete"),
    ("60643198-e4b2-4c39-9de1-53cca3fff382", "DeleteDirectly"),
    ("167a160b-fa48-4337-87ab-7e0fe95c4b5a", "DeleteListItem"),
    ("c78c9f3d-e92c-4f38-bb72-d8bd7fa5dbe3", "Dimension"),
    ("894afc03-9904-465d-b671-f555ffb9b21c", "Document"),
    ("e51185a4-d915-45b8-b201-1c46cc2d8104", "DocumentJournal"),
    ("ed0bec43-4633-416c-8c08-0384ca444e32", "EndEdit"),
    ("723765ab-0b92-4745-a621-1ba0f77c92c9", "EventLog"),
    ("4fddea39-5129-4b4c-83fe-4e443cd61940", "EventLogByUser"),
    ("544fdbe8-5956-4512-bc62-93b4c022d291", "ExchangePlan"),
    ("003024ed-fa25-42ac-9f53-f5014e383801", "ExecuteTask"),
    ("fb7e9fb5-110b-41cb-adc6-753969ae1c81", "ExpandAll"),
    ("fad46a2b-2e56-47cc-b90c-3c2d4b061937", "ExternalDataSourceCube"),
    ("2954e819-f3fc-40de-9769-292efce9a355", "ExternalDataSourceFunction"),
    ("5b612c21-e223-4997-9e61-86f7a67ec945", "ExternalDataSourceTable"),
    ("b1406535-6cc2-4410-95ea-753556e8460f", "FilterByCurrentValue"),
    ("2ef82795-06fe-4365-bd0c-44b486264620", "FilterCriterion"),
    ("ffab30f1-da11-44b5-b34c-24da22badcf4", "Find"),
    ("c7cdd3c0-3879-436a-b145-5e2615e9b3e1", "FindInList"),
    ("38bbcebe-e456-461b-8457-07c9a72344a3", "FindInTree"),
    ("5182f57f-e834-4d11-9c9f-4aedc002b6e9", "FixTable"),
    ("fc34a694-e99b-4d1c-a526-63f5571bdb09", "Form"),
    ("b7c81c62-d6ad-4eae-9cea-0e203182db67", "FormHelp"),
    ("f874b0cc-db1d-4577-8c77-d4ba206eb05d", "Forward"),
    ("0ce78048-0196-4f80-a781-9829cdb7f43e", "GenerateReport"),
    ("892196a9-c94f-4e50-8224-3c0cea4ea6b8", "GoBack"),
    ("7562cef7-0e57-4f63-a754-b61128a4f3ae", "GoForward"),
    ("85cc7dd0-44fc-41aa-967f-f52f202ee2e6", "GoToBegin"),
    ("14b24498-e49c-4713-be64-75101d0abfb9", "GoToEnd"),
    ("31bf709f-3b50-4137-9b51-ebc7fb802a7c", "GotoExternalURL"),
    ("2a7e58e2-a6c5-4387-a459-5249c441947a", "GroupConversation"),
    ("6ecee038-9722-4d80-bb91-7ee7046ec4c7", "Help"),
    ("c283cd1c-3187-451d-8ef2-7df55daeef06", "History"),
    ("4b54770b-d069-4c0e-9b17-5cc2a01134d9", "Information"),
    ("5b87ad1b-d8cc-43c1-b5c4-dc43613c518c", "InformationRegister"),
    ("01ec9d9a-7497-4d88-b93f-066c633a4866", "InputOnBasis"),
    ("cb34c423-3d6a-4202-a809-3b3f45fb14ab", "LevelUp"),
    ("31b93f03-0ba2-4631-a171-0d3a3d2ecc48", "ListSettings"),
    ("549a2c45-4fce-493f-94ee-9a3a4f426551", "ListViewMode"),
    ("3d4ad3b1-17de-4cf1-a2e4-0c2c83a5b5c2", "ListViewModeHierarchicalList"),
    ("c757209d-a87f-4410-b1a3-76000178f1f0", "ListViewModeList"),
    ("283ecabd-aaed-41d1-ad46-6cca91c29120", "LoadReportSettings"),
    ("18492a87-2fe4-44af-b218-304897fed020", "MarkToDelete"),
    ("78da2c47-172f-4d57-ab52-a06e40548136", "Message"),
    ("37e91e77-93ce-4c3b-8d30-a9d8cfd3d3b0", "MoveItem"),
    ("9c7f3c8c-a4b4-4a0a-84c8-d60d9feeca16", "NewConversation"),
    ("928075d1-b90b-416c-b0b2-c3104cf084aa", "Notifications"),
    ("785362cb-3756-48ed-87d2-292ded17054a", "OpenFile"),
    ("c2e2d966-5b7f-4699-903b-28a6f50d5471", "OutputList"),
    ("64837726-d2a2-4682-a788-737423e80013", "Picture"),
    ("20ebc47b-f4d9-439c-acd3-fdc624fbac2a", "Post"),
    ("0abdab67-5c90-4296-8168-239d22024d11", "PrintImmediately"),
    ("b4c7ab2c-bcda-4468-a28f-5fee93838c4e", "Properties"),
    ("7df3febb-2640-41b7-ad8b-7a23b7ad4aec", "QueryWizardCreateTempTableDropQuery"),
    ("83c8f18d-8701-41f3-bef4-53f88adbb868", "ReadChanges"),
    ("fc4f29e0-d168-4fe0-8e64-e982fabf2595", "Refresh"),
    ("6cb69e7f-fe19-4f64-bfb5-1a4fad6c2ef9", "Replace"),
    ("db817ee1-fd28-4e7f-bb4a-53686b2b153c", "Report"),
    ("942e0303-a3ec-4fe8-887c-5aea8516d424", "ReportSettings"),
    ("8f29e0e2-d5e6-41e8-a34d-9a0288156322", "Reread"),
    ("a7707ed1-39b0-418f-974d-4d500d27a9c6", "RestoreValues"),
    ("c7f70aa3-b944-4efe-97e3-0fa3bda3cd88", "RotateClockwise"),
    ("a43fcd1b-ad8d-4318-9d55-fd1ba086e65b", "RotateCounterclockwise"),
    ("818ab7d0-4654-4542-bd5e-fd9d1352b5a1", "SaveFile"),
    ("b5a0aaba-3a83-4a71-b6f9-24aae1574681", "SaveReportSettings"),
    ("23f940bf-7381-4c2b-85a1-e541ed428042", "SaveValues"),
    ("1970a480-9b38-405e-9d9e-8209f3fad5f1", "ScheduledJob"),
    ("6206a729-16e5-4e32-b53c-122de4e30c8d", "ScheduledJobs"),
    ("fc6a06a8-1308-4385-b1b2-9d302d2054ed", "SearchControl"),
    ("be23a908-fe1b-44df-be94-d0f6e8353abe", "SendMessage"),
    ("58174855-39be-462e-8723-cb2d95182146", "SetDateInterval"),
    ("55ef0776-5ee4-4daf-9a9b-70d63643ab8d", "SetTime"),
    ("caf2e58b-ca3d-4b63-82c9-f21f1c9bc9eb", "Setting"),
    ("a064544f-6037-48ca-b19f-8ad63e43af23", "ShowData"),
    ("9c96aa25-d656-4b3d-ab3e-81d9718da238", "ShowInList"),
    ("91022b99-b610-48ad-954e-a297848081ce", "SortListAsc"),
    ("1fa32fdb-a180-418f-a6eb-db7516b7a30b", "SortListDesc"),
    ("7c75b1df-1fdc-471f-aae6-6b7870318cd4", "SpreadsheetDeleteComment"),
    ("03665ff1-3a05-41d1-96d3-04bda2d8ede3", "SpreadsheetInsertComment"),
    ("26518e18-e364-475a-8026-e41134658b2a", "SpreadsheetInsertPageBreak"),
    ("2846af8d-af84-47e3-82b9-01b01f960426", "SpreadsheetReadOnly"),
    ("1cd7b762-ec6a-4e92-ac9a-1832be228ec3", "Stop"),
    ("6e3687cf-a8d1-446a-833a-bfaf38516353", "SwitchActivity"),
    ("3bdc16c8-6a96-4467-9442-a8e4804b3fa2", "SyncContents"),
    ("37cf7cc0-abad-4385-b597-6fd2d8dc085a", "Task"),
    ("8ca4ea33-603d-4992-8a41-c7924b5bd40b", "UndoPosting"),
    ("6ff3ddbd-56e3-4ddf-a5bf-048c1e2dfb2f", "User"),
    ("894cf65b-4109-4533-a1d7-c87b1fcc80a3", "Write"),
    ("e6fc55a0-3d58-4b15-bdd3-717453929598", "WriteAndClose"),
    ("97c5a6d5-47ed-43f9-8c8c-10e9903c23d2", "Calendar"),
    ("87d032df-0956-47e9-bead-4e15330f1983", "AppearanceUpArrowGreen"),
    ("c1a61df2-f280-49d0-a8b3-7e5fc6f56ff7", "AppearanceCircleRed"),
    ("2721abfb-fbff-4a3a-98ac-b7c9eb29cd85", "AppearanceCircleEmpty"),
    ("35bc8caa-f7ce-4158-87da-d9bf785afa39", "DataSearch"),
    ("1a4342a5-fa06-4556-8a85-e8738fc25821", "GetURL"),
];

fn std_picture(id: &str) -> Option<&'static str> {
    FORM_STD_PICTURES.iter().find(|(g, _)| *g == id).map(|(_, n)| *n).or_else(|| crate::xml::imp::form::platform::std_picture(id))
}

/// Стандартные картинки по отрицательному коду `{-N}`.
fn std_picture_code(code: i64) -> Option<&'static str> {
    Some(match code {
        -1 => "InputFieldSelect",
        -2 => "InputFieldClear",
        -3 => "MoveUp",
        -4 => "MoveDown",
        -5 => "InputFieldCalendar",
        -7 => "InputFieldOpen",
        -8 => "MoveLeft",
        -9 => "MoveRight",
        -10 => "CheckAll",
        -11 => "UncheckAll",
        -13 => "Print",
        -14 => "InputFieldChooseType",
        -200 => "Clear",
        _ => return crate::xml::imp::form::platform::std_picture(&code.to_string()),
    })
}

/// Картинка `{4,вид,{ссылка},"",x,y,прозрачность,...}`; вид 0 — нет картинки.
fn picture(w: &mut W, tag: &str, n: &Node, names: &Names) {
    let kind = num_at(n, 1);
    if kind == 0 {
        return;
    }
    w.open(tag);
    let r = n.get(2).unwrap_or(n);
    match kind {
        1 => {
            let code = num_at(r, 0);
            let id = atom_at(r, 1);
            let name = if code < 0 {
                std_picture_code(code).map_or_else(|| format!("\u{1}StdPicture{code}"), |s| format!("StdPicture.{s}"))
            } else if code == 0 && r.list().len() == 1 {
                // `{0}` без ссылки — `0` (БП 3.0.107 ФормаНастроекОтчета: команда ВыбратьПериод1).
                "0".into()
            } else if let Some(s) = std_picture(id) {
                format!("StdPicture.{s}")
            } else if let Some(f) = names.full.get(id) {
                f.clone()
            } else if NOT_STD_PICTURES.contains(&id) {
                format!("0:{id}")
            } else {
                format!("\u{1}картинка:{id}")
            };
            w.leaf("xr:Ref", &name);
        }
        // Встроенная картинка лежит рядом файлом Form/Items/<элемент>/<свойство>.<расширение> (см. payload).
        _ => match super::payload::picture_value(n).as_deref().and_then(super::payload::picture_ext) {
            Some(ext) => w.leaf("xr:Abs", &format!("{tag}.{ext}")),
            None => w.leaf("xr:Abs", &unk("картинка", n)),
        },
    }
    w.leaf("xr:LoadTransparent", bool_text(num_at(n, 6) == 1));
    let (x, y) = (num_at(n, 4), num_at(n, 5));
    if x >= 0 && y >= 0 {
        w.line(&format!("<xr:TransparentPixel x=\"{x}\" y=\"{y}\"/>"));
    }
    w.close(tag);
}

impl Form<'_> {
    /// Дочерние элементы `(guid вида, элемент)...` на глубине `depth`.
    fn children(&self, ch: &[(&str, &Node)], depth: usize) -> Result<String, String> {
        let mut s = String::new();
        for (g, n) in ch {
            s += &self.item(Some(g), n, depth, None)?;
        }
        Ok(s)
    }

    /// Дети элемента: число на позиции `at`, затем пары (guid, элемент). Возвращает позицию за ними.
    fn pairs<'n>(l: &It<'n>, at: usize) -> Result<(Vec<(&'n str, &'n Node)>, usize), String> {
        let cnt = l.num(at);
        if cnt < 0 {
            return Err(format!("нет числа детей на {at}"));
        }
        let mut v = Vec::new();
        for k in 0..cnt as usize {
            let g = l.a(at + 1 + 2 * k);
            let n = l.n(at + 2 + 2 * k).ok_or("нет дочернего элемента")?;
            v.push((g, n));
        }
        Ok((v, at + 1 + 2 * cnt as usize))
    }

    /// `guid` — вид элемента в списке детей; встроенные (контекстное меню, подсказка) идут без него.
    fn item(&self, guid: Option<&str>, n: &Node, depth: usize, tag: Option<&'static str>) -> Result<String, String> {
        let it = It::new(n, if guid == Some(BUTTON) { 3 } else { 4 });
        match (guid, tag) {
            (_, Some("ExtendedTooltip")) => self.decoration(&it, depth, tag),
            (Some(BUTTON), _) => self.button(&it, depth),
            (Some(FIELD), _) => self.field(&it, depth),
            (Some(TABLE), _) => self.table(&it, depth),
            (Some(ADDITION), _) => self.addition(&it, depth),
            (Some(GROUP) | None, _) => self.group(&it, depth),
            (Some(DECORATION), _) => self.decoration(&it, depth, None),
            (Some(g), _) => Err(format!("элемент вида {g} пока не выгружается")),
        }
    }

    /// Открывающий тег, свойства и закрывающий; без свойств — пустой элемент.
    fn element(tag: &str, it: &It, depth: usize, p: P, order: &[&str]) -> String {
        let name = it.n(6).and_then(Node::str).unwrap_or("");
        let mut w = W { s: String::new(), depth };
        let head = format!("<{tag} name=\"{}\" id=\"{}\"{}", esc_attr(name), it.id(), p.head);
        if p.items.is_empty() {
            w.line(&format!("{head}/>"));
            return w.s;
        }
        w.line(&format!("{head}>"));
        w.depth += 1;
        p.emit(&mut w, order);
        w.close(tag);
        w.s
    }

    fn sub_items(&self, p: &mut P, depth: usize, et: Option<&Node>, ch: &[(&str, &Node)]) -> Result<(), String> {
        if let Some(e) = et {
            p.raw("ExtendedTooltip", self.item(None, e, depth + 1, Some("ExtendedTooltip"))?);
        }
        let c = self.children(ch, depth + 2)?;
        if !c.is_empty() {
            let ind = "\t".repeat(depth + 1);
            p.raw("ChildItems", format!("\r\n{ind}<ChildItems>{c}\r\n{ind}</ChildItems>"));
        }
        Ok(())
    }

    fn group(&self, it: &It, depth: usize) -> Result<String, String> {
        let kind = it.num(5);
        let tag = match kind {
            0 => "CommandBar",
            1 => "Popup",
            2 => "ColumnGroup",
            3 => "Pages",
            4 => "Page",
            5 => "UsualGroup",
            6 => "ButtonGroup",
            8 => "ContextMenu",
            9 => "AutoCommandBar",
            _ => return Err(format!("группа вида {kind}")),
        };
        let nm = self.names;
        let mut p = P::new(depth + 1);
        let sub = it.n(20).ok_or("группа: нет свойств вида")?;
        let (ch, t) = Self::pairs(it, 21)?;
        let mut et = None;
        if !matches!(kind, 8 | 9) {
            p.local("Title", it.n(7).unwrap_or(&EMPTY));
            p.local("ToolTip", it.n(8).unwrap_or(&EMPTY));
            p.flag_if("EnableContentChange", it.num(9) == 1, "true");
            p.flag_if("Enabled", it.num(10) == 0, "false");
            p.flag_if("ReadOnly", it.num(11) == 1, "true");
            p.num_nz("Width", it.a(12));
            p.num_nz("Height", it.a(13));
            p.tri("HorizontalStretch", it.num(14));
            p.tri("VerticalStretch", it.num(15));
            p.color("TitleTextColor", it.n(16), nm);
            p.font("TitleFont", it.n(17), nm);
            // Поле 18 `{0,код,модификаторы}` — Shortcut (сворачиваемая группа; каркас: Ctrl+S и др.).
            if let Some(s) = it.n(18).filter(|n| n.list().len() == 3 && atom_at(n, 0) == "0").and_then(shortcut) {
                p.leaf("Shortcut", &s);
            }
            p.adjustable("UserVisible", it.uv(), nm);
            p.flag_if("Visible", it.num(t) == 0, "false");
            p.en("ToolTipRepresentation", TOOLTIP_REPR, it.num(t + 1), 0);
            let mut k = t + 2;
            if it.num(k) == 1 {
                et = it.n(k + 1);
                k += 1;
            }
            p.en("GroupHorizontalAlign", H_ALIGN, it.num(k + 2), 3);
            p.en("GroupVerticalAlign", V_ALIGN, it.num(k + 3), 3);
        }
        let mut k = t + 2;
        if it.num(k) == 1 {
            k += 1;
        }
        p.importance(it.num(k + 4));
        let s = |i: usize| num_at(sub, i);
        let consts = |p: &mut P, list: &[(usize, &str)]| {
            for &(i, def) in list {
                p.unk_if("\u{1}группа", sub.get(i), def);
            }
        };
        match kind {
            0 => {
                p.en("HorizontalLocation", &["", "Center", "Right", "Auto"], s(1), 0);
                self.command_source(&mut p, sub.get(2));
            }
            1 => {
                p.picture("Picture", sub.get(1), nm);
                self.command_source(&mut p, sub.get(2));
                p.en("Representation", &["Text", "Picture", "PictureAndText"], s(4), 3);
                p.en("Shape", &["", "Usual", "Oval"], s(5), 0);
                p.en("ShapeRepresentation", &["", "Always", "WhenActive", "None"], s(6), 0);
                p.color("BackColor", sub.get(7), nm);
                p.color("BorderColor", sub.get(8), nm);
                consts(&mut p, &[(3, "2")]);
            }
            2 => {
                p.en("Group", &["Horizontal", "", "InCell"], s(1), 1);
                p.flag_if("ShowTitle", s(2) == 0, "false");
                p.flag_if("ShowInHeader", s(3) == 1, "true");
                p.en("HeaderHorizontalAlign", H_ALIGN, s(4), 3);
                p.picture("HeaderPicture", sub.get(5), nm);
                p.color("TitleBackColor", sub.get(6), nm);
                if let Some(d) = sub.get(7).and_then(|d| self.data_path(d)) {
                    p.leaf("HeaderDataPath", &d);
                }
                p.local("HeaderFormat", sub.get(10).unwrap_or(&EMPTY));
                p.en("FixingInTable", &["", "Left", "Right"], s(11), 0);
                consts(&mut p, &[(8, "{\"Pattern\"}"), (9, "\"\"")]);
            }
            3 => {
                let r = if sub.list().len() > 5 { s(5) } else { s(1) };
                p.en("PagesRepresentation", &["None", "TabsOnTop", "TabsOnBottom", "TabsOnLeftHorizontal", "TabsOnRightHorizontal", "Swipe", "Auto"], r, 6);
                let mut w = p.w();
                events_xml(&mut w, &[sub.get(2).unwrap_or(&EMPTY)], &|g| item_event(g).map(String::from))?;
                p.push("Events", w);
                p.en("CurrentRowUse", &["Use", "DontUse"], s(3), 2);
                consts(&mut p, &[(4, "0")]);
            }
            4 => {
                p.picture("Picture", sub.get(1), nm);
                p.en("ChildItemsWidth", CHILD_WIDTH, s(3), 0);
                if let Some(d) = sub.get(4).and_then(|d| self.data_path(d)) {
                    p.leaf("TitleDataPath", &d);
                }
                p.local("Format", sub.get(5).unwrap_or(&EMPTY));
                p.flag_if("ShowTitle", s(6) == 0, "false");
                p.color("BackColor", sub.get(9), nm);
                p.en("HorizontalSpacing", SPACING, s(10), 0);
                p.en("VerticalSpacing", SPACING, s(11), 0);
                p.en("HorizontalAlign", H_ALIGN, s(12), 3);
                p.en("VerticalAlign", V_ALIGN, s(13), 3);
                p.en("ChildrenAlign", CHILDREN_ALIGN, s(14), 0);
                if sub.get(15).is_some() && !self.flag(it.id(), super::form_v59::UNSET_SCROLL) {
                    p.leaf("ScrollOnCompress", bool_text(s(15) == 1));
                }
                // Страница из 17 полей (формы версий 42/46): группировка в 16 (0/1), платформа пишет ее всегда (43 из 43).
                let g = match sub.list().len() {
                    17 => Some(s(16)).filter(|g| *g <= 1),
                    n if n > 17 => Some(s(17)),
                    _ => None,
                };
                if sub.list().len() >= 17 && !self.flag(it.id(), super::form_v59::UNSET_GROUP) {
                    p.leaf("Group", g.and_then(|g| GROUPING.get(g as usize)).copied().unwrap_or("\u{1}"));
                }
                consts(&mut p, &[(7, "{\"Pattern\"}"), (8, "\"\""), (18, "{3,4,{0}}"), (19, "{7,3,0,1,100}")]);
            }
            6 => {
                self.command_source(&mut p, sub.get(1));
                p.en("Representation", &["", "Usual", "Compact"], s(3), 0);
                consts(&mut p, &[(2, "2")]);
            }
            5 => {
                p.en("ChildItemsWidth", CHILD_WIDTH, s(2), 0);
                if !self.flag(it.id(), super::form_v59::UNSET_REPRESENTATION) {
                    p.leaf("Representation", ["None", "StrongSeparation", "WeakSeparation", "NormalSeparation"].get(s(3) as usize).copied().unwrap_or("\u{1}"));
                }
                p.flag_if("ShowTitle", s(4) == 0, "false");
                p.flag_if("ControlRepresentation", s(11) == 1, "Button");
                p.flag_if("Collapsed", s(12) == 1, "true");
                p.flag_if("ShowLeftMargin", s(13) == 0, "false");
                p.local("CollapsedRepresentationTitle", sub.get(14).unwrap_or(&EMPTY));
                p.local("Format", sub.get(6).unwrap_or(&EMPTY));
                p.en("HorizontalSpacing", SPACING, s(15), 0);
                p.en("VerticalSpacing", SPACING, s(16), 0);
                p.en("HorizontalAlign", H_ALIGN, s(17), 3);
                p.en("VerticalAlign", V_ALIGN, s(18), 3);
                p.en("ThroughAlign", &["Use", "DontUse"], s(19), 2);
                p.en("ChildrenAlign", CHILDREN_ALIGN, s(20), 0);
                p.flag_if("United", s(21) == 0, "false");
                p.color("BackColor", sub.get(9), nm);
                // Поле 23 — HiddenStateTitleBackColor с версии 25: mngcore FUN_180351660 (чтение, +0x15c), FUN_180351380 (запись).
                if s(0) > 0x18 {
                    p.color("HiddenStateTitleBackColor", sub.get(23), nm);
                }
                p.en("CurrentRowUse", &["", "DontUse"], s(25), 2);
                // У старых версий (23 поля) группировка в 22, у новых она же продублирована в 27.
                // Совсем старые (15 полей) — в 1 (ВыборПериода, ФормаСообщение: 1 Horizontal, 0 Vertical); версия 12
                // (9 полей, форма версии 28 ЗУП ВидыОтправляемыхДокументов.ФормаГруппы) — тоже в 1, поля Behavior нет.
                // shortcut: раскладка версии 12 — по двум группам одной формы (поля 1, 3, 4 сверены на двух значениях,
                // остальные одинаковы); при других значениях полей 2, 5–8 и группировке не 0/1 — отказ; пересмотреть,
                // когда будет снято чтение группы старой версии в mngcore или встретится другая такая форма.
                if sub.list().len() == 9 {
                    consts(&mut p, &[(2, "0"), (5, "{0}"), (6, "{1,0}"), (7, "{\"Pattern\"}"), (8, "\"\"")]);
                }
                let g = match sub.list().len() {
                    9 if s(1) > 1 => i64::MIN,
                    9 | 15 => s(1),
                    n if n > 27 => s(27),
                    _ => s(22),
                };
                if !self.flag(it.id(), super::form_v59::UNSET_GROUP) {
                    p.leaf("Group", &GROUPING.get(g as usize).map_or_else(|| format!("\u{1}{g}:{}", sub.list().len()), |v| v.to_string()));
                }
                if s(10) == 0 {
                    p.leaf("Behavior", "Usual");
                } else if sub.list().len() != 9 {
                    // У версии 12 (9 полей) поля нет — Behavior не пишется (ЗУП ВидыОтправляемыхДокументов.ФормаГруппы, 2 из 2).
                    // Без поля 28 (28 полей) — поле 24: 1 Collapsible, 2 PopUp (у 29 полей 24 с 28 совпадает; БП 18 из 18).
                    let b = match (sub.get(28), sub.get(24)) {
                        (Some(_), _) => s(28),
                        (None, Some(_)) => s(24),
                        (None, None) => 1,
                    };
                    match b {
                        1 => p.leaf("Behavior", "Collapsible"),
                        2 => p.leaf("Behavior", "PopUp"),
                        3 => {}
                        c => p.leaf("Behavior", &format!("\u{1}{c}")),
                    }
                }
                if let Some(d) = sub.get(5).and_then(|d| self.data_path(d)) {
                    p.leaf("TitleDataPath", &d);
                }
            }
            8 => p.flag_if("Autofill", s(1) == 0, "false"),
            9 => {
                p.en("HorizontalAlign", H_ALIGN, s(1), 0);
                p.flag_if("Autofill", s(2) == 0, "false");
            }
            _ => p.leaf("\u{1}группа", &unk("", sub)),
        }
        self.sub_items(&mut p, depth, et, &ch)?;
        let order: &[&str] = match kind {
            0 => COMMAND_BAR_ORDER,
            1 => POPUP_ORDER,
            2 => COLUMN_GROUP_ORDER,
            3 => PAGES_ORDER,
            4 => PAGE_ORDER,
            6 => BUTTON_GROUP_ORDER,
            5 => USUAL_GROUP_ORDER,
            8 => &["Autofill", "ChildItems"],
            9 => &["HorizontalAlign", "Autofill", "ChildItems"],
            _ => &[],
        };
        Ok(Self::element(tag, it, depth, p, order))
    }

    /// Надпись и расширенная подсказка (подсказка — надпись без контекстного меню и своей подсказки).
    fn decoration(&self, it: &It, depth: usize, tag: Option<&'static str>) -> Result<String, String> {
        let kind = it.num(5);
        let tag = tag.unwrap_or(match kind {
            0 => "LabelDecoration",
            1 => "PictureDecoration",
            _ => return Err(format!("декорация вида {kind}")),
        });
        let nm = self.names;
        let mut p = P::new(depth + 1);
        p.local("ToolTip", it.n(8).unwrap_or(&EMPTY));
        p.flag_if("Enabled", it.num(9) == 0, "false");
        p.num_nz("Width", it.a(10));
        p.num_nz("Height", it.a(11));
        p.tri("HorizontalStretch", it.num(12));
        p.tri("VerticalStretch", it.num(13));
        let sub = it.n(18).ok_or("декорация: нет свойств")?;
        // Картинка версии 59: поле 14 — TextColor, PictureColor — поле 13 свойств вида (`form_v59`).
        let v59_picture = kind == 1 && sub.list().len() > 13;
        // У картинки прежних версий поле 14 — PictureColor, и платформа пишет его же еще и TextColor (БизнесСеть.ОтправкаДокумента).
        p.color("TextColor", it.n(14), nm);
        if kind == 1 && !v59_picture {
            p.color("PictureColor", it.n(14), nm);
        }
        p.font("Font", it.n(15), nm);
        // Поле 16 `{0,код,модификаторы}` — Shortcut (ФинансыОрганизации: Ctrl+S у надписи; терялось).
        if let Some(s) = it.n(16).filter(|n| n.list().len() == 3 && atom_at(n, 0) == "0").and_then(shortcut) {
            p.leaf("Shortcut", &s);
        }
        p.adjustable("UserVisible", it.uv(), nm);
        p.flag_if("Hyperlink", num_at(sub, if kind == 1 { 2 } else { 1 }) == 1, "true");
        let ev = if kind == 1 {
            // Картинка: {4,Picture,Hyperlink,PictureSize,Zoomable,NonselectedPictureText,BorderColor,Border,
            // EnableStartDrag,EnableDrag,события,FileDragMode,ImageScale}.
            p.picture("Picture", sub.get(1), nm);
            p.en("PictureSize", PICTURE_SIZE, num_at(sub, 3), 0);
            p.flag_if("Zoomable", num_at(sub, 4) == 1, "true");
            p.local("NonselectedPictureText", sub.get(5).unwrap_or(&EMPTY));
            p.color("BorderColor", sub.get(6), nm);
            p.border(sub.get(7), 0);
            p.flag_if("EnableStartDrag", num_at(sub, 8) == 1, "true");
            p.flag_if("EnableDrag", num_at(sub, 9) == 1, "true");
            // У старой версии без поля 11 — тоже AsFile.
            p.flag_if("FileDragMode", matches!(num_at(sub, 11), 0 | i64::MIN), "AsFile");
            p.num_def("ImageScale", atom_at(sub, 12), "100");
            if v59_picture {
                p.color("PictureColor", sub.get(13), nm);
            }
            sub.get(10)
        } else {
            p.en("HorizontalAlign", H_ALIGN, num_at(sub, 2), 0);
            p.en("VerticalAlign", V_ALIGN, num_at(sub, 3), 3);
            p.num_nz("TitleHeight", atom_at(sub, 4));
            p.color("BackColor", sub.get(6), nm);
            p.color("BorderColor", sub.get(7), nm);
            p.border(sub.get(8), 0);
            sub.get(5)
        };
        let mut k = 19;
        let mut cm = None;
        if it.num(k) == 1 {
            cm = it.n(k + 1);
            k += 1;
        }
        k += 1;
        p.flag_if("Visible", it.num(k) == 0, "false");
        p.tri("SkipOnInput", it.num(k + 1));
        p.formatted("Title", it.n(k + 2));
        p.en("ToolTipRepresentation", TOOLTIP_REPR, it.num(k + 3), 0);
        k += 4;
        let mut et = None;
        if it.num(k) == 1 {
            et = it.n(k + 1);
            k += 1;
        }
        k += 1;
        p.flag_if("AutoMaxWidth", it.num(k) == 0, "false");
        p.num_nz("MaxWidth", it.a(k + 1));
        p.flag_if("AutoMaxHeight", it.num(k + 3) == 0, "false");
        p.num_nz("MaxHeight", it.a(k + 4));
        p.en("GroupHorizontalAlign", H_ALIGN, it.num(k + 5), 3);
        p.en("GroupVerticalAlign", V_ALIGN, it.num(k + 6), 3);
        // важность — за GroupVerticalAlign (номер сдвигают необязательные контекстное меню и подсказка)
        if it.a(0) == "12" {
            p.importance(it.num(k + 7));
        }
        if let Some(e) = ev {
            let mut w = p.w();
            events_xml(&mut w, &[e], &|g| item_event(g).map(String::from))?;
            p.push("Events", w);
        }
        if let Some(c) = cm {
            p.raw("ContextMenu", self.item(None, c, depth + 1, None)?);
        }
        self.sub_items(&mut p, depth, et, &[])?;
        Ok(Self::element(tag, it, depth, p, if kind == 1 { PICTURE_DECORATION_ORDER } else { DECORATION_ORDER }))
    }
}

/// Перечисления словаря формы: порядок значений XDTO-модели enums.dll (ресурс model.xdto) — номер в записи.
pub(crate) const FOLDERS_AND_ITEMS_USE: &[&str] = &["Items", "Folders", "FoldersAndItems"];
pub(crate) const AUTO_TIME_MODE: &[&str] = &["DontUse", "Last", "First", "CurrentOrLast", "CurrentOrFirst"];
pub(crate) const POSTING_MODE_USE: &[&str] = &["Regular", "RealTime", "Ask", "Auto"];

const CI_ITEM_ORDER: &[&str] = &["Command", "Type", "Attribute", "CommandGroup", "Index", "DefaultVisible", "Visible"];

const ATTRIBUTE_ORDER: &[&str] = &[
    "Title", "Type", "View", "Edit", "MainAttribute", "SavedData", "FillCheck", "UseAlways", "Save", "FunctionalOptions", "Columns", "Settings",
];

const COMMAND_ORDER: &[&str] = &[
    "Title", "ToolTip", "Use", "Shortcut", "Picture", "Action", "FunctionalOptions", "Representation", "ModifiesSavedData", "CurrentRowUse",
    "AssociatedTableElementId",
];

// Порядок свойств элементов (таблицы *_ORDER групп, кнопки, таблицы, декораций, дополнений и полей) — порядок записи
// XML формы в mngbase.dll 8.5.1.1529: писатель элемента (FUN_180756d90 обычная группа, FUN_18074ef10 кнопка,
// FUN_1807526f0 таблица, FUN_180750cf0 поле ввода …) вызывает писатель базового типа (FUN_180724450 GroupBase,
// FUN_180711e80 Field, FUN_18072b980 Decoration, FUN_18072d2f0 Addition), затем свой (FUN_180726190 UsualGroup,
// FUN_18072f280 Button, FUN_1807317c0 Table + FUN_180748e70 доп. свойства списка, FUN_180713f60 InputField …); каждый
// пишет свойства по номеру модели XDTO logform (model.xdto из mngbase_root.res, базовые типы впереди) в порядке кода,
// пропуская умолчания. Затем предопределенные элементы (FUN_180710f40, в порядке списка элемента), Events
// (FUN_180752ff0), ChildItems. Сверка: инверсий нет ни в выгрузке БП 3.0.171 (≈300 тыс. элементов), ни в Клиенте ЭДО.

const COMMAND_BAR_ORDER: &[&str] = &[
    "Visible", "UserVisible", "Enabled", "ReadOnly", "EnableContentChange", "Title", "TitleTextColor", "TitleFont", "ToolTip",
    "ToolTipRepresentation", "Shortcut", "Width", "Height", "HorizontalStretch", "VerticalStretch", "GroupHorizontalAlign", "GroupVerticalAlign",
    "HorizontalLocation", "AppearanceMode", "CommandSource", "ContextMenu", "AutoCommandBar", "ExtendedTooltip", "SearchStringAddition",
    "ViewStatusAddition", "SearchControlAddition", "Events", "ChildItems",
];

const POPUP_ORDER: &[&str] = &[
    "Visible", "UserVisible", "Enabled", "ReadOnly", "EnableContentChange", "Title", "TitleTextColor", "TitleFont", "ToolTip",
    "ToolTipRepresentation", "Shortcut", "Width", "Height", "HorizontalStretch", "VerticalStretch", "GroupHorizontalAlign", "GroupVerticalAlign",
    "Picture", "CommandSource", "Representation", "PlacementArea", "Shape", "ShapeRepresentation", "BackColor", "BorderColor", "Importance",
    "ContextMenu", "AutoCommandBar", "ExtendedTooltip", "SearchStringAddition", "ViewStatusAddition", "SearchControlAddition", "Events", "ChildItems",
];

const COLUMN_GROUP_ORDER: &[&str] = &[
    "Visible", "UserVisible", "Enabled", "ReadOnly", "EnableContentChange", "Title", "TitleTextColor", "TitleFont", "ToolTip",
    "ToolTipRepresentation", "Shortcut", "Width", "Height", "HorizontalStretch", "VerticalStretch", "GroupHorizontalAlign", "GroupVerticalAlign",
    "Group", "ShowTitle", "TitleBackColor", "ShowInHeader", "HeaderDataPath", "HeaderHorizontalAlign", "HeaderPicture", "HeaderFormat",
    "FixingInTable", "ShowInCard", "FixInCard", "ShowTitleInCard", "ContextMenu", "AutoCommandBar", "ExtendedTooltip", "SearchStringAddition",
    "ViewStatusAddition", "SearchControlAddition", "Events", "ChildItems",
];

const PAGES_ORDER: &[&str] = &[
    "Visible", "UserVisible", "Enabled", "ReadOnly", "EnableContentChange", "Title", "TitleTextColor", "TitleFont", "ToolTip",
    "ToolTipRepresentation", "Shortcut", "Width", "Height", "HorizontalStretch", "VerticalStretch", "GroupHorizontalAlign", "GroupVerticalAlign",
    "PagesRepresentation", "CurrentRowUse", "AssociatedTableElementId", "ContextMenu", "AutoCommandBar", "ExtendedTooltip", "SearchStringAddition",
    "ViewStatusAddition", "SearchControlAddition", "Events", "ChildItems",
];

const PAGE_ORDER: &[&str] = &[
    "Visible", "UserVisible", "Enabled", "ReadOnly", "EnableContentChange", "Title", "TitleTextColor", "TitleFont", "ToolTip",
    "ToolTipRepresentation", "Shortcut", "Width", "Height", "HorizontalStretch", "VerticalStretch", "GroupHorizontalAlign", "GroupVerticalAlign",
    "Picture", "Group", "ChildrenAlign", "HorizontalSpacing", "VerticalSpacing", "HorizontalAlign", "VerticalAlign", "ChildItemsWidth",
    "ChildItemsTitleLocation", "Format", "ShowTitle", "TitleDataPath", "BackColor", "ScrollOnCompress", "ContextMenu", "AutoCommandBar",
    "ExtendedTooltip", "SearchStringAddition", "ViewStatusAddition", "SearchControlAddition", "Events", "ChildItems",
];

const BUTTON_GROUP_ORDER: &[&str] = &[
    "Visible", "UserVisible", "Enabled", "ReadOnly", "EnableContentChange", "Title", "TitleTextColor", "TitleFont", "ToolTip",
    "ToolTipRepresentation", "Shortcut", "Width", "Height", "HorizontalStretch", "VerticalStretch", "GroupHorizontalAlign", "GroupVerticalAlign",
    "CommandSource", "PlacementArea", "Representation", "ContextMenu", "AutoCommandBar", "ExtendedTooltip", "SearchStringAddition",
    "ViewStatusAddition", "SearchControlAddition", "Events", "ChildItems",
];

const USUAL_GROUP_ORDER: &[&str] = &[
    "Visible", "UserVisible", "Enabled", "ReadOnly", "EnableContentChange", "Title", "TitleTextColor", "TitleFont", "ToolTip",
    "ToolTipRepresentation", "Shortcut", "Width", "Height", "HorizontalStretch", "VerticalStretch", "GroupHorizontalAlign", "GroupVerticalAlign",
    "Group", "ChildrenAlign", "HorizontalSpacing", "VerticalSpacing", "HorizontalAlign", "VerticalAlign", "Behavior", "CollapsedRepresentationTitle",
    "CollapsedRepresentationItem", "Collapsed", "ControlRepresentation", "Representation", "ShowLeftMargin", "United", "ChildItemsWidth",
    "ChildItemsTitleLocation", "Format", "ShowTitle", "TitleDataPath", "BackColor", "HiddenStateTitleBackColor", "ThroughAlign", "CurrentRowUse",
    "AssociatedTableElementId", "ShowAsCard", "BackPictureEffect", "CardRepresentationType", "Hyperlink", "BackPicture", "ScrollOnCompress",
    "ContextMenu", "AutoCommandBar", "ExtendedTooltip", "SearchStringAddition", "ViewStatusAddition", "SearchControlAddition", "Events", "ChildItems",
];

const PICTURE_SIZE: &[&str] = &["RealSize", "Stretch", "Proportionally", "Tile", "AutoSize", "RealSizeIgnoreScale", "AutoSizeIgnoreScale", "ByFontSize"];

const PICTURE_DECORATION_ORDER: &[&str] = &[
    "Visible", "UserVisible", "Enabled", "Width", "AutoMaxWidth", "MaxWidth", "MinWidth", "Height", "AutoMaxHeight", "MaxHeight", "HorizontalStretch",
    "VerticalStretch", "SkipOnInput", "TextColor", "Font", "Shortcut", "Title", "ToolTip", "ToolTipRepresentation", "GroupHorizontalAlign",
    "GroupVerticalAlign", "OnMainServerUnavalableBehavior", "Hyperlink", "PictureSize", "Zoomable", "ImageScale", "NonselectedPictureText",
    "EnableStartDrag", "EnableDrag", "Picture", "BorderColor", "Border", "FileDragMode", "BackgroundShowMode", "PictureColor", "ContextMenu",
    "AutoCommandBar", "ExtendedTooltip", "SearchStringAddition", "ViewStatusAddition", "SearchControlAddition", "Events", "ChildItems",
];

const DECORATION_ORDER: &[&str] = &[
    "Visible", "UserVisible", "Enabled", "Width", "AutoMaxWidth", "MaxWidth", "MinWidth", "Height", "AutoMaxHeight", "MaxHeight", "HorizontalStretch",
    "VerticalStretch", "SkipOnInput", "TextColor", "Font", "Shortcut", "Title", "ToolTip", "ToolTipRepresentation", "GroupHorizontalAlign",
    "GroupVerticalAlign", "OnMainServerUnavalableBehavior", "Hyperlink", "HorizontalAlign", "VerticalAlign", "TitleHeight", "BackColor",
    "BorderColor", "Border", "ContextMenu", "AutoCommandBar", "ExtendedTooltip", "SearchStringAddition", "ViewStatusAddition",
    "SearchControlAddition", "Events", "ChildItems",
];

// ---------------------------------------------------------------- кнопки, команды, пути к данным

const BUTTON: &str = "a9f3b1ac-f51b-431e-b102-55a69acdecad";
const ELEM: &str = "02023637-7868-4a5f-8576-835a76e0c9ba";
const FORM_COMMANDS: &str = "409b9a53-7f7e-4178-86c1-33176c7c7a7a";
/// Метка дополнительной колонки табличной части в пути к данным `{id колонки,ADDED_COLUMN}`.
const ADDED_COLUMN: &str = "5bdad865-f2c5-434b-8041-ba4aad3b6687";
/// Индекс строки в пути к данным `{N,ROW_INDEX}` → `[N]`.
const ROW_INDEX: &str = "e67e2953-cebe-4d97-bb93-12b17e6384f8";

/// Элементы, которые 8.5 порождает при загрузке формы прежней версии, по порядку id (наибольший id + 1, ...): обход
/// в глубину, после потомков элемента — у поля панель действий выделенных строк, у таблицы — действия строки, панель
/// действий выделенных строк, ее строка поиска и управление поиском (с контекстным меню и расширенной подсказкой).
/// Сверено с каркасом (те же формы, сохраненные 8.5): 4 238 форм из 4 244 — имена и id порожденных совпали; 6 форм
/// версии 28 (им платформа еще порождает расширенные подсказки) сюда не попадают.
fn generated_items<'n>(n: &'n Node, guid: Option<&str>, out: &mut Vec<String>) {
    let l = n.list();
    let mut g = None;
    for x in l {
        match x {
            Node::Atom(a) if is_guid(a) => g = Some(a.as_str()),
            Node::List(_) => {
                generated_items(x, g, out);
                g = None;
            }
            _ => {}
        }
    }
    if !matches!(l.get(1), Some(Node::List(id)) if id.len() == 2 && id[1].atom() == Some(ELEM)) || l.len() <= 2 {
        return;
    }
    let name = l.iter().skip(2).take(10).find_map(Node::str).unwrap_or("");
    let tails: &[&str] = match guid {
        Some(FIELD) => &["ПанельДействийВыделенныхСтрок"],
        Some(TABLE) => &[
            "ДействияСтроки", "ПанельДействийВыделенныхСтрок", "ПанельДействийВыделенныхСтрокСтрокаПоиска",
            "ПанельДействийВыделенныхСтрокСтрокаПоискаКонтекстноеМеню", "ПанельДействийВыделенныхСтрокСтрокаПоискаРасширеннаяПодсказка",
            "ПанельДействийВыделенныхСтрокУправлениеПоиском", "ПанельДействийВыделенныхСтрокУправлениеПоискомКонтекстноеМеню",
            "ПанельДействийВыделенныхСтрокУправлениеПоискомРасширеннаяПодсказка",
        ],
        _ => &[],
    };
    out.extend(tails.iter().map(|t| format!("{name}{t}")));
}

/// Имена элементов `{вид,{0},…,"Имя",…}` — id 0 у порожденных загрузкой XML 8.5.
fn collect_zero_items(n: &Node, out: &mut Vec<String>) {
    let l = n.list();
    if l.len() > 6 && matches!(l.get(1), Some(Node::List(id)) if id.len() == 1 && id[0].atom() == Some("0")) {
        if let Some(name) = l.iter().skip(3).find_map(Node::str) {
            out.push(name.to_string());
        }
    }
    for x in l {
        if matches!(x, Node::List(_)) {
            collect_zero_items(x, out);
        }
    }
}

/// Имена элементов формы по id (для ссылок «Item.<имя>» и стандартных команд элементов).
fn collect_items<'n>(n: &'n Node, out: &mut HashMap<&'n str, &'n str>) {
    let l = n.list();
    if let (Some(Node::List(id)), true) = (l.get(1), l.len() > 6) {
        if id.len() == 2 && id[1].atom() == Some(ELEM) {
            if let (Some(i), Some(name)) = (id[0].atom(), l.iter().skip(3).find_map(Node::str)) {
                out.entry(i).or_insert(name);
            }
        }
    }
    for x in l {
        collect_items(x, out);
    }
}

/// Таблицы: id → путь к данным и id → узел самой таблицы.
fn collect_tables<'n>(n: &'n Node, out: &mut HashMap<&'n str, &'n Node>, ex: &mut HashMap<&'n str, &'n Node>, plain: &mut HashSet<&'n str>) {
    let l = n.list();
    for (i, x) in l.iter().enumerate() {
        if matches!(x.atom(), Some(GROUP | DECORATION | BUTTON)) {
            if let Some(Node::List(id)) = l.get(i + 1).and_then(|t| t.get(1)) {
                if id.len() == 2 && id[1].atom() == Some(ELEM) {
                    plain.extend(id[0].atom());
                }
            }
        }
        // Встроенные элементы идут без вида перед собой: контекстное меню и командная панель — группы (виды 8, 9), расширенная
        // подсказка — декорация, а также дополнения таблицы. Имя команды элемента — у источника его команд (mngbase
        // FUN_1802e6650 → FUN_1802e6050 → +0x20 FUN_1802e9740): свой источник — только при непустом vtbl+0x50 или +0x30
        // элемента, иначе общий +0x80, как у групп, декораций и кнопок, — ссылка `id:guid` (ЗУП: Find подсказки поля). Имя в x[3..],
        // как в collect_items, отличает элемент от пути к текущим данным `{N,{T,ELEM},{0,g}...}`.
        // shortcut: что vtbl+0x50/+0x30 пусты у встроенных элементов, не прослежено (классы элементов не найдены); проверено
        // только на подсказке (ЗУП, 2 ссылки), ссылок на команды контекстного меню, командной панели и дополнений в корпусах
        // нет — при такой ссылке сверить с платформой и проследить vtbl элемента.
        if let Some(Node::List(id)) = x.get(1).filter(|_| x.list().len() > 6 && !i.checked_sub(1).and_then(|k| l[k].atom()).is_some_and(is_guid) && x.list().iter().skip(3).find_map(Node::str).is_some()) {
            if id.len() == 2 && id[1].atom() == Some(ELEM) {
                plain.extend(id[0].atom());
            }
        }
        // Поля надписи, ввода, флажка и переключателя стандартных команд не имеют (БП 3.0.107: `163:guid` OutputList у надписи).
        if x.atom() == Some(FIELD) {
            if let Some(it) = l.get(i + 1).map(|t| It::new(t, 4)).filter(|it| matches!(it.num(5), 1 | 2 | 3 | 5)) {
                plain.insert(it.id());
            }
        }
        if x.atom() == Some(TABLE) {
            if let Some(t) = l.get(i + 1) {
                let it = It::new(t, 4);
                if let Some(d) = it.n(11) {
                    out.insert(it.id(), d);
                }
                ex.insert(it.id(), t);
            }
        }
        collect_tables(x, out, ex, plain);
    }
}

/// Класс объекта по имени типа: `CatalogObject.Банки` → `Catalog`.
fn type_class(t: &str) -> &str {
    let head = t.split('.').next().unwrap_or(t);
    for suffix in ["RecordManager", "RecordSet", "RecordKey", "Record", "Object", "Ref", "Selection", "List", "Manager"] {
        if let Some(c) = head.strip_suffix(suffix) {
            if !c.is_empty() {
                return c;
            }
        }
    }
    head
}

/// Стандартные реквизиты `{-N}` по классу владельца (из сверки с выгрузкой платформы).
fn std_field(class: &str, code: i64) -> Option<&'static str> {
    Some(match (class, code) {
        ("Catalog", -2) => "Code",
        ("Catalog", -3) => "Description",
        ("Catalog", -4) => "Parent",
        ("Catalog", -5) => "Owner",
        ("Catalog", -8) => "Ref",
        ("Catalog", -13) => "PredefinedDataName",
        // форма Конфигуратора 8.5 с «Расположения» (layout.cf 28.10.2026)
        ("Catalog", -7) => "DeletionMark",
        ("Catalog", -10) => "Predefined",
        ("Document" | "BusinessProcess" | "Task", -2) => "Number",
        ("Document" | "BusinessProcess" | "Task", -3) => "Date",
        // Как в стандартных реквизитах описания (`standard::NAMES`): БольничныйЛист, Отпуск.ФормаПодробнееОРасчете.
        ("Document", -4) => "DeletionMark",
        ("Document", -7) => "Posted",
        ("Document" | "Task", -5) => "Ref",
        ("Document", -8) => "RegisterRecords",
        ("Task", -7) => "BusinessProcess",
        ("Task", -8) => "RoutePoint",
        ("Task", -9) => "Description",
        ("Task", -10) => "Executed",
        ("ChartOfAccounts", -6) => "Parent",
        ("ChartOfAccounts", -7) => "Code",
        ("ChartOfAccounts", -8) => "Description",
        ("ChartOfAccounts", -10) => "Type",
        ("ChartOfAccounts", -11) => "OffBalance",
        ("ChartOfCharacteristicTypes", -8) => "Code",
        ("ChartOfCharacteristicTypes", -9) => "Description",
        ("ChartOfCharacteristicTypes", -11) => "ValueType",
        ("ChartOfCalculationTypes", -2) => "Code",
        ("ChartOfCalculationTypes", -3) => "Description",
        ("ChartOfCalculationTypes", -30) => "LeadingCalculationTypes",
        ("ExchangePlan", -2) => "Code",
        ("ExchangePlan", -3) => "Description",
        ("ExchangePlan", -6) => "Ref",
        ("ExchangePlan", -9) => "SentNo",
        ("ExchangePlan", -10) => "ReceivedNo",
        ("TabularSection", -2) => "LineNumber",
        ("InformationRegister" | "AccountingRegister", -2) => "Period",
        ("InformationRegister" | "AccumulationRegister" | "AccountingRegister", -4) => "LineNumber",
        ("InformationRegister" | "AccumulationRegister", -5) => "Active",
        ("AccountingRegister", -6) => "AccountDr",
        ("AccountingRegister", -7) => "AccountCr",
        ("AccountingRegister", -30) => "PeriodAdjustment",
        // регистр без корреспонденции (ERP ТиповыеОперацииМеждународныйУчет)
        ("AccountingRegister", -10) => "Account",
        // Отбор набора записей (БП 3.0.107 ПроверкаАналитикиСчетовУчетаКапитала: `НаборЗаписейХозрасчетный.Filter`).
        ("InformationRegister" | "AccumulationRegister" | "AccountingRegister", -60001) => "Filter",
        ("DynamicList", -1) => "Order",
        ("DynamicList", 10000000) => "DefaultPicture",
        ("DynamicList", -2) => "Filter",
        ("DynamicList", -3) => "Group",
        ("ChartOfCalculationTypes", -10) => "BaseCalculationTypes",
        ("ChartOfCalculationTypes", -20) => "DisplacingCalculationTypes",
        ("CalculationTypes", -101) => "CalculationType",
        // прочие — коды стандартных реквизитов описания те же (ERP: Parent ПВХ, ThisNode, ActionPeriodIsBasic)
        _ => return super::standard::NAMES.iter().find(|(c, _)| *c == class)?.1.iter().find(|(k, _)| k.parse() == Ok(code)).map(|(_, n)| *n),
    })
}

/// Субконто Дт/Кт записи регистра бухгалтерии в пути к данным (`{N,guid}`, N — номер субконто с нуля).
const EXT_DIM_DR: &str = "1ab44b24-3315-40a9-b495-f1f1227ac205";
const EXT_DIM_CR: &str = "f77758c9-9fcd-490f-9bbd-1e446541f536";
/// Субконто записи регистра бухгалтерии без корреспонденции (номер — с нуля).
const EXT_DIM: &str = "91162600-3161-4326-89a0-4a7cecd5092a";

/// Компоновщик настроек отчета (стандартный реквизит ReportObject) в пути к данным.
const REPORT_COMPOSER: &str = "b9754f01-29e9-11d6-a3c7-0050bae0a776";

/// Поля платформенных типов в пути к данным: (класс, код) → (имя, класс следующей части).
fn composer_field(class: &str, code: i64) -> Option<(&'static str, &'static str)> {
    Some(match (class, code) {
        ("SettingsComposer" | "dcsset:SettingsComposer", 0) => ("Settings", "Settings"),
        ("SettingsComposer" | "dcsset:SettingsComposer", 2) => ("FixedSettings", "Settings"),
        ("DynamicList", -6) => ("SettingsComposer", "SettingsComposer"),
        ("Settings", 0) => ("DataParameters", "DataParameters"),
        ("Settings", 1) => ("Filter", "Filter"),
        ("Settings", 2) => ("Selection", "Selection"),
        ("Settings", 3) => ("Order", "Order"),
        ("Settings", 4) => ("ConditionalAppearance", "ConditionalAppearance"),
        ("Filter" | "ConditionalAppearance" | "Order", 10000) => ("Use", ""),
        ("Filter", 10001) => ("LeftValuePicture", ""),
        ("Filter", 10002) => ("LeftValue", ""),
        ("Filter", 10003) => ("ComparisonType", ""),
        ("Filter", 10004) => ("RightValuePicture", ""),
        ("Filter", 10005) => ("RightValue", ""),
        ("Filter", 10006) => ("Date", ""),
        ("Filter", 10007) => ("GroupType", ""),
        ("Filter", 10008) => ("Application", ""),
        ("Filter", 10009) => ("ViewMode", ""),
        ("Filter", 10010) => ("Presentation", ""),
        ("ConditionalAppearance", 10001) => ("Appearance", "Appearance"),
        ("ConditionalAppearance", 10002) => ("Filter", "Filter"),
        ("ConditionalAppearance", 10003) => ("Fields", "Fields"),
        ("ConditionalAppearance", 10004) => ("Presentation", ""),
        ("ConditionalAppearance", 10005) => ("UseArea", ""),
        // Элемент оформления (ЭлементУсловногоОформленияОтчета).
        ("Appearance" | "Fields", 10000) => ("Use", ""),
        ("Appearance", 10001) => ("Parameter", ""),
        ("Appearance", 10002) => ("ValuePicture", ""),
        ("Appearance", 10003) => ("Value", ""),
        ("Fields", 10001) => ("FieldPicture", ""),
        ("Fields", 10002) => ("Field", ""),
        ("Order", 10002) => ("Field", ""),
        ("Order", 10003) => ("OrderType", ""),
        // Текущая строка дерева настроек (`Items.<дерево>.CurrentData...`).
        ("Settings", 10007) => ("ItemDataParameters", "DataParameters"),
        ("Settings", 10008) => ("ItemFilter", "Filter"),
        ("Settings", 10009) => ("ItemGroupFields", "GroupFields"),
        ("Settings", 10010) => ("ItemSelection", "Selection"),
        ("Settings", 10011) => ("ItemOrder", "Order"),
        ("Settings", 10012) => ("ItemConditionalAppearance", "ConditionalAppearance"),
        ("Settings", 10013) => ("ItemOutputParameters", "DataParameters"),
        ("Settings", 10014) => ("ItemUserFields", "UserFields"),
        ("GroupFields", 0) => ("GroupFieldsAvailableFields", ""),
        ("Filter", 0) => ("FilterAvailableFields", "FilterAvailableFields"),
        ("FilterAvailableFields", 10000) => ("FieldPicture", ""),
        ("Selection", 0) => ("SelectionAvailableFields", ""),
        ("Order", 0) => ("OrderAvailableFields", ""),
        ("v8:ValueListType", -1) => ("ValueType", ""),
        ("v8:ValueListType", 0) => ("Value", ""),
        ("v8:ValueListType", 1) => ("Presentation", ""),
        ("v8:ValueListType", 2) => ("Check", ""),
        ("v8:ValueListType", 3) => ("Picture", ""),
        ("v8:StandardPeriod", 0) => ("Variant", ""),
        // стандартная дата начала: core85 имя свойства по номеру (около 0x1801492d0) — 0 Variant, 1 Date
        ("v8:StandardBeginningDate", 0) => ("Variant", ""),
        ("v8:StandardBeginningDate", 1) => ("Date", ""),
        ("v8:StandardPeriod", 1) => ("StartDate", ""),
        ("v8:StandardPeriod", 2) => ("EndDate", ""),
        // колонки таблицы диаграммы Ганта (ERP МониторингЗаказа: эталон загрузки XML)
        ("d5p1:GanttChart", 0) => ("Point", ""),
        ("d5p1:GanttChart", 1) => ("Text", ""),
        ("FilterAvailableFields", 10001) => ("Title", ""),
        ("SettingsComposer" | "dcsset:SettingsComposer", 1) => ("UserSettings", "UserSettings"),
        ("UserSettings", 10000) => ("Use", ""),
        ("UserSettings", 10001) => ("SettingPicture", ""),
        ("UserSettings", 10002) => ("Setting", ""),
        ("UserSettings", 10003) => ("ComparisonType", ""),
        ("UserSettings", 10004) => ("ValuePicture", ""),
        ("UserSettings", 10005) => ("Value", ""),
        ("UserSettings", 10006) => ("EditInReportForm", ""),
        ("UserSettings", 10007) => ("Filter", "Filter"),
        ("UserSettings", 10008) => ("Order", "Order"),
        ("UserSettings", 10009) => ("Selection", "Selection"),
        ("UserSettings", 10010) => ("ConditionalAppearance", "ConditionalAppearance"),
        ("UserSettings", 10011) => ("Structure", ""),
        // Остальные номера свойств компоновки — getPropInfo контекстов dcscore (номер → пара имен): настройки
        // FUN_1801d5a50/FUN_1801d5da0, параметры данных FUN_18001e0f0, выбор FUN_18019fd30/FUN_18019fe50, порядок
        // FUN_180173e30/FUN_180173f50, группировка FUN_180164a90/FUN_180164bb0, поля оформления FUN_180124250/320,
        // пользовательские поля FUN_180220200, отбор и оформление — UsedCount (FUN_18014f4f0, FUN_1800c7c40);
        // динамический список mngbase FUN_180299370 (-1…-6).
        ("Settings", 5) => ("OutputParameters", "DataParameters"),
        ("Settings", 6) => ("UserFields", "UserFields"),
        ("Settings", 10000) => ("Use", ""),
        ("Settings", 10001) => ("ReportStructure", ""),
        ("Settings", 10002) => ("HasSelection", ""),
        ("Settings", 10003) => ("HasFilter", ""),
        ("Settings", 10004) => ("HasOrder", ""),
        ("Settings", 10005) => ("HasConditionalAppearance", ""),
        ("Settings", 10006) => ("HasOutputParameters", ""),
        ("Settings", 10015) => ("ReportStructurePicture", ""),
        ("DataParameters", 10000) => ("Use", ""),
        ("DataParameters", 10001) => ("Parameter", ""),
        ("DataParameters", 10002) => ("ValuePicture", ""),
        ("DataParameters", 10003) => ("Value", ""),
        ("DataParameters", 10004) => ("Date", ""),
        ("DataParameters", 10005) => ("StartDate", ""),
        ("DataParameters", 10006) => ("EndDate", ""),
        ("Filter" | "Selection" | "Order" | "GroupFields", 1) => ("UsedCount", ""),
        ("ConditionalAppearance" | "UserSettings", 0) => ("UsedCount", ""),
        ("Selection", 10000) => ("Use", ""),
        ("Selection", 10001) => ("TitlePicture", ""),
        ("Selection", 10002) => ("Title", ""),
        ("Selection", 10003) => ("Placement", ""),
        ("Selection", 10004) => ("FieldPicture", ""),
        ("Selection", 10005) => ("Field", ""),
        ("Order", 10001) => ("FieldPicture", ""),
        ("GroupFields", 10000) => ("Use", ""),
        ("GroupFields", 10001) => ("FieldPicture", ""),
        ("GroupFields", 10002) => ("Field", ""),
        ("GroupFields", 10003) => ("GroupType", ""),
        ("GroupFields", 10004) => ("AdditionType", ""),
        ("GroupFields", 10005) => ("BeginOfPeriodPicture", ""),
        ("GroupFields", 10006) => ("BeginOfPeriod", ""),
        ("GroupFields", 10007) => ("EndOfPeriodPicture", ""),
        ("GroupFields", 10008) => ("EndOfPeriod", ""),
        ("Fields", 0) => ("AppearanceFieldsAvailableFields", ""),
        ("UserFields", 10000) => ("Title", ""),
        ("DynamicList", -4) => ("ConditionalAppearance", ""),
        ("DynamicList", -5) => ("Parameters", ""),
        ("ChartOfAccounts", -12) => ("ExtDimensionTypes", "ExtDimensionTypes"),
        ("ExtDimensionTypes", -13) => ("ExtDimensionType", ""),
        ("ExtDimensionTypes", -15) => ("TurnoversOnly", ""),
        _ => return None,
    })
}

/// Команды динамического списка, которых без основной таблицы нет: у формы (каркас: 12 таких в записи, в XML ни одной)
/// и у таблицы (исключенные не пишутся, кнопки на них — `id:guid`). ChangeHistory таблицы — ЗУП ВидыДокументовЭДОБЗК
/// (в эталонах пяти конфигураций у 548 таблиц списков без основной таблицы ни одной; у формы она — отказ, `command_set`).
/// shortcut: ChangeHistory — по образцам, отбор команд таблицы в mngbase не снят; кнопка на нее — отказ (`command`);
/// пересмотреть, когда будет снят отбор или встретится кнопка.
const NO_MAIN_COMMANDS: &[&str] = &["Change", "Copy", "Create", "CreateFolder", "Delete", "SetDeletionMark", "MoveItem", "HierarchicalList", "List", "Tree", "LevelDown", "LevelUp", "ChangeHistory"];

/// GUID команд таблицы, которые в каркасе остаются только у динамического списка: настройки списка (у дерева значений
/// платформа их выбрасывает)...
const DL_SETTINGS_COMMANDS: &[&str] = &[
    "33b7b9cd-6979-4435-8c58-d9bc8250edec", // DynamicListStandardSettings
    "14559f7c-853c-42a4-9ea1-01546107747b", // ListSettings
    "182a793b-22a5-4625-b316-6a5be7f88078", // LoadDynamicListSettings
    "825c1c15-ef8f-47ab-b002-e6b84b3e5b10", // OutputList
    "403bc6e6-b98e-4181-9f43-9c75cbbf82cf", // Refresh
    "95b4bc12-2ece-4d7a-b3e2-6f9293620a06", // SaveDynamicListSettings
];
/// ...и прочие (у коллекций значений не встречались).
const DL_ONLY_COMMANDS: &[&str] = &[
    "11761e12-cf32-4826-a175-b23213e3b229", // ChangeHistory
    "0f8d6d98-2f8b-405a-b8b3-0538e9d95da5", // Create
    "b59f3c87-e213-4947-abae-9dbaffaef147", // CreateByParameter
    "d82ca05c-2966-4d77-9a39-a1eea087bfa7", // CreateFolder
    "ec576e13-1e76-4c33-98aa-a33204514227", // Delete
    "dc118d99-b351-4e30-9310-e864f2e53ec0", // LevelDown
    "0e9b637d-cf6e-4330-8a8f-cd44842e34bb", // LevelUp
    "27bd521a-51c6-4fe7-846d-a98f988774b5", // MoveItem
    "e3dd8850-fc3c-41b1-bbb3-7c66af082608", // Post
    "daa306cd-a78a-4e74-a14c-739daba624cb", // SetDateInterval
    "a2f737a8-0114-4e86-a214-45e5c213fa65", // SetDeletionMark
    "04ac7211-e74f-4776-9749-35a9282b1d52", // UndoPosting
];
/// Команды коллекций значений (свои GUID), которых у таблицы динамического списка нет: исключенные платформа выбрасывает,
/// кнопки на них — `id:guid` (ВыборПрисоединенногоФайла, ОшибкиЗаполненияОбъектов, ДокументыПоТребованиюФНС).
const COLL_ONLY_COMMANDS: &[&str] = &[
    "b0016a68-ec64-4e6d-b905-c71fd62efc4c", // Add
    "8d772f97-c0ef-47c0-9cb0-efea28c61341", // Delete
    "9ef79140-3de6-436a-8dda-610bb963f5db", // EndEdit
    "fa51b106-eae6-44c7-8054-76cbb3100603", // MoveDown
    "37740564-9e86-44a0-bea9-3f485a5a3f91", // MoveUp
    "8af6ebff-cd02-4bfe-a984-44a292623708", // ShowRowRearrangement
    "2bbe4e12-06d2-409b-a972-eea585125d83", // SortListAsc
    "58b2a785-23f6-4b0e-a324-9a1323285595", // SortListDesc
];

/// У таблицы динамического списка с основной таблицей класса `class` нет команды `g`. Delete у списка регистра сведений —
/// GUID коллекций `8d772f97`, а `ec576e13` нет (ЖурналСтатусовФинОтчетностиВБанки); у прочих — наоборот.
fn dl_lacks(g: &str, class: &str, older: bool) -> bool {
    match g {
        "ec576e13-1e76-4c33-98aa-a33204514227" => class == "InformationRegister",
        // SelectAll таблицы коллекции: 8.5 пишет его и у списка, 8.3.24 и 8.3.27 — нет (БП 3.0.107 ФинОтчетВБанк).
        "51c99108-107c-43e1-8918-e48835bf2495" => older,
        "8d772f97-c0ef-47c0-9cb0-efea28c61341" => class != "InformationRegister",
        _ => COLL_ONLY_COMMANDS.contains(&g),
    }
}
/// Иерархия таблицы (у динамического списка и дерева; у таблицы значений платформа их выбрасывает).
const TREE_COMMANDS: &[&str] = &[
    "01833a5a-6553-4c49-b445-095018107bb5", // HierarchicalList
    "0d0249a4-2b2f-4fc0-a66f-b36f9494b3cc", // List
    "05468165-f954-45a5-84f2-6641c51f9f23", // Tree
];

/// Основная таблица динамического списка не задана (нет ключа или нулевой GUID).
fn no_main(main: &str) -> bool {
    main.trim_start_matches(['0', '-']).is_empty()
}

/// Стандартные команды объектов: (класс, номер, имя) — номер из `CompositeID(N, uuid объекта)` функции команд класса
/// (basic.dll: справочник 0x1801a0ae0, ПВХ 0x18019a190, документ 0x18019bcd0, журнал 0x1801a0340, перечисление
/// 0x18019d1e0, регистр сведений 0x18019db10 (3… — открыть по ведущему измерению), регистр накопления 0x180198960,
/// отчет 0x1801a32d0, обработка 0x1801a4c10, константа 0x1801a3bb0; accnt.dll план счетов 0x180016b90, регистр
/// бухгалтерии 0x1800184d0; calc.dll ПВР 0x180011930, регистр расчета 0x180013270; bp.dll бизнес-процесс 0x18000f260,
/// задача 0x18000c9c0; backend.dll план обмена 0x1801052b0, критерий отбора 0x180106e00). Общая форма — 100 Open.
pub(crate) const STD_COMMANDS: &[(&str, i64, &str)] = &[
    ("Catalog", 0, "OpenList"), ("Catalog", 1, "Create"), ("Catalog", 2, "CreateFolder"), ("Catalog", 3, "CreateBasedOn"), ("Catalog", 4, "OpenByValue"),
    ("ChartOfCharacteristicTypes", 0, "OpenList"), ("ChartOfCharacteristicTypes", 1, "CreateFolder"), ("ChartOfCharacteristicTypes", 2, "Create"),
    ("ChartOfCharacteristicTypes", 3, "CreateBasedOn"),
    ("Document", 0, "OpenList"), ("Document", 1, "Create"), ("Document", 2, "CreateBasedOn"),
    ("DocumentJournal", 0, "OpenList"), ("Enum", 0, "OpenList"),
    ("InformationRegister", 0, "OpenList"), ("InformationRegister", 1, "Create"), ("InformationRegister", 2, "OpenByRecorder"),
    ("AccumulationRegister", 0, "OpenList"), ("AccumulationRegister", 1, "OpenByRecorder"),
    ("Report", 0, "Open"), ("DataProcessor", 0, "Open"), ("Constant", 100, "Open"), ("CommonForm", 100, "Open"),
    ("ChartOfAccounts", 0, "OpenList"), ("ChartOfAccounts", 1, "Create"), ("ChartOfAccounts", 2, "CreateBasedOn"),
    ("AccountingRegister", 0, "OpenList"), ("AccountingRegister", 1, "OpenByRecorder"),
    ("ChartOfCalculationTypes", 0, "OpenList"), ("ChartOfCalculationTypes", 1, "Create"), ("ChartOfCalculationTypes", 2, "CreateBasedOn"),
    ("CalculationRegister", 0, "OpenList"), ("CalculationRegister", 1, "OpenByRecorder"),
    ("BusinessProcess", 0, "OpenList"), ("BusinessProcess", 1, "Create"), ("BusinessProcess", 2, "CreateBasedOn"),
    ("Task", 0, "OpenList"), ("Task", 1, "Create"), ("Task", 2, "CreateBasedOn"), ("Task", 3, "OpenByValue"),
    ("ExchangePlan", 0, "OpenList"), ("ExchangePlan", 1, "Create"), ("ExchangePlan", 2, "CreateBasedOn"),
    ("FilterCriterion", 0, "OpenByValue"),
];

/// Номер стандартной команды объекта `{N,<guid объекта>}` → имя (`STD_COMMANDS`).
pub(crate) fn object_std_command(class: &str, n: i64) -> Option<&'static str> {
    STD_COMMANDS.iter().find(|(c, k, _)| *c == class && *k == n).map(|(.., s)| *s)
}

impl Form<'_> {
    /// Команда `{N,guid}`: команда формы, стандартная команда формы или элемента, общая команда или команда объекта.
    fn command(&self, c: &Node) -> String {
        let (n, g) = (atom_at(c, 0), atom_at(c, 1));
        let num: i64 = n.parse().unwrap_or(i64::MIN);
        if g == FORM_COMMANDS {
            return match self.commands.get(n) {
                Some(name) => format!("Form.Command.{name}"),
                None => format!("{n}:{g}"),
            };
        }
        if num == 0 && !self.excluded.contains(g) {
            // Команды чужой группы (SetDeletionMark у обработки) у формы нет — `0:guid`.
            if let Some(s) = std_form_command2(g).filter(|s| self.scopes[command_scope(s) as usize]) {
                return format!("Form.StandardCommand.{s}");
            }
        } else if let (Some(s), Some(item)) = (std_item_command(g), self.items.get(n)) {
            // Таблица без пути к данным: в БП у нее Add/Change/Delete/EndEdit нет (`id:guid`), прочие не встречались.
            if self.path_less(n) && !matches!(s, "Add" | "Change" | "Delete" | "EndEdit") {
                return format!("\u{1}команда таблицы без пути к данным:{s}");
            }
            // shortcut: кнопок на ChangeHistory таблицы списка без основной таблицы (`NO_MAIN_COMMANDS`) в эталонах пяти
            // конфигураций нет ни именем, ни `id:guid` — отказ; снять, когда встретится.
            if s == "ChangeHistory" && self.no_main_list(n) {
                return format!("\u{1}кнопка ChangeHistory таблицы списка без основной таблицы:{item}");
            }
            if !self.table_lacks(n, g) {
                return format!("Form.Item.{item}.StandardCommand.{s}");
            }
        }
        if let Some(full) = self.names.full.get(g) {
            if full.contains(".Command.") || full.starts_with("CommonCommand.") {
                return full.clone();
            }
            if self.names.no_std_commands.contains(g) {
                return format!("{n}:{g}");
            }
            let class = full.split('.').next().unwrap_or("");
            match object_std_command(class, num) {
                // Без «Вводится на основании» команды нет — платформа пишет `N:guid`.
                Some("CreateBasedOn") if !self.names.based_on.contains(g) => {}
                // Справочник без владельцев команды «открыть по владельцу» не имеет.
                Some("OpenByValue") if class == "Catalog" && !self.names.owned.contains(g) => {}
                Some(s) => return format!("{full}.StandardCommand.{s}"),
                None if class == "InformationRegister" && num >= 3 => {
                    if let Some(d) = self.names.masters.get(g).and_then(|m| m.get(num as usize - 3)) {
                        return format!("{full}.StandardCommand.OpenByValue.{d}");
                    }
                }
                None => {}
            }
        }
        // Так пишет и платформа, если команду не нашла.
        format!("{n}:{g}")
    }

    /// У списка с основной таблицей `main` нет SetDateInterval: справочник или непериодический регистр сведений.
    fn no_date_interval(&self, main: &str) -> bool {
        match self.names.full.get(main).map_or("", |f| f.split('.').next().unwrap_or("")) {
            "Catalog" => true,
            "InformationRegister" => !self.names.periodic.contains(main),
            _ => false,
        }
    }

    /// Таблица `id` — самого динамического списка без основной таблицы (у таблицы его отбора — нет).
    fn no_main_list(&self, id: &str) -> bool {
        let Some(it) = self.table_nodes.get(id).map(|t| It::new(t, 4)) else { return false };
        let attr = it.n(11).and_then(|d| d.get(1)).map_or("", |f| atom_at(f, 0));
        let main = self.lists.get(attr).and_then(|st| st.list().get(2..)?.chunks(2).find(|c| c[0].str() == Some("MainTable"))?.get(1)).map_or("", |v| atom_at(v, 2));
        self.attrs.get(attr).is_some_and(|a| a.1 == "DynamicList") && it.n(11).is_some_and(|d| num_at(d, 0) == 1) && no_main(main)
    }

    /// Таблица `id` без пути к данным.
    fn path_less(&self, id: &str) -> bool {
        self.table_nodes.get(id).is_some_and(|t| It::new(t, 4).n(11).is_none_or(|d| num_at(d, 0) <= 0))
    }

    /// У таблицы `id` нет стандартной команды `g`: исключена или «Выбрать» вне режима выбора — платформа пишет `id:guid`.
    /// shortcut: SearchHistory таблицы не списка с SearchControlLocation None (в CommandSet ее нет) не учтена — кнопок на нее
    /// в БП нет; учесть, когда встретится.
    fn table_lacks(&self, id: &str, g: &str) -> bool {
        if self.plain.contains(id) || self.path_less(id) {
            return true;
        }
        let Some(it) = self.table_nodes.get(id).map(|t| It::new(t, 4)) else { return false };
        let ex = it.n(55 + 2 * it.num(54).max(0) as usize + 1);
        // Как в CommandSet таблицы: у списка справочника или непериодического регистра сведений нет SetDateInterval.
        let attr = it.n(11).and_then(|d| d.get(1)).map_or("", |f| atom_at(f, 0));
        let main = self.lists.get(attr).and_then(|st| st.list().get(2..)?.chunks(2).find(|c| c[0].str() == Some("MainTable"))?.get(1)).map_or("", |v| atom_at(v, 2));
        ex.is_some_and(|e| e.list().iter().skip(1).any(|x| x.atom() == Some(g)))
            || (std_item_command(g) == Some("Choose") && it.num(22) != 1)
            // Как в CommandSet: при выделении одной строки нет команд множественного выделения (БП 3.0.107 СтраныМира).
            || (std_item_command(g).is_some_and(|c| matches!(c, "SelectAll" | "ShowMultipleSelection")) && it.num(24) == 0)
            || (std_item_command(g) == Some("SetDateInterval") && self.no_date_interval(main))
            || (self.attrs.get(attr).is_some_and(|a| a.1 == "DynamicList") && it.n(11).is_some_and(|d| num_at(d, 0) == 1) && (dl_lacks(g, self.names.full.get(main).map_or("", |f| f.split('.').next().unwrap_or("")), self.names.older.is_some())
                    // Как в CommandSet таблицы: без изменения состава строк нет и команд изменения состава.
                    || it.num(17) != 1 && std_item_command(g).is_some_and(|c| matches!(c, "Copy" | "Create" | "CreateFolder" | "Delete" | "SetDeletionMark" | "MoveItem"))
                    || no_main(main) && std_item_command(g).is_some_and(|c| NO_MAIN_COMMANDS.contains(&c))))
    }

    /// Путь к данным `{N,{реквизит формы},{поле}...}`; `None` — пути нет.
    fn data_path(&self, d: &Node) -> Option<String> {
        let cnt = num_at(d, 0);
        if cnt <= 0 {
            return None;
        }
        let parts = &d.list()[1..];
        let first = parts.first()?;
        let id = atom_at(first, 0);
        let rest = parts.get(1..cnt as usize).unwrap_or(&[]);
        let path = match (first.get(1), self.attrs.get(id)) {
            // Недоступное поле динамического списка — с `~`.
            (None, Some((name, cls, _))) => self.sub_path(name.clone(), cls, id, rest).map(|s| {
                let n = rest.first().filter(|p| p.get(1).is_none()).map(|p| atom_at(p, 0));
                let col = n.and_then(|n| self.cols.get(&(id, n)));
                match (self.lists.get(id), col) {
                    (Some(_), Some(_)) if self.twin_extra(id, n.unwrap_or("")) => format!("~{s}"),
                    (Some(st), Some(c)) => self.names.mark(st, c, s, &self.ru_col(id), self.roots.get(id).map(Vec::as_slice)),
                    _ => s,
                }
            }),
            // Текущие данные таблицы: путь продолжает путь к данным таблицы.
            (Some(g), _) if g.atom() == Some(ELEM) => self.current_data(id, rest),
            _ => None,
        };
        Some(path.or_else(|| self.lost(parts.get(..cnt as usize)?)).unwrap_or_else(|| unk("путь", d)))
    }

    /// Путь к данным элемента: поле динамического списка (или текущих данных его таблицы), которого нет в FieldsMapItemId,
    /// платформа не пишет вовсе (ВедомостьНаВыплатуЗарплатыВБанк — кнопка, Обмен1С_КАМИН — поле надписи).
    fn item_path(&self, d: &Node) -> Option<String> {
        let path = self.data_path(d)?;
        // Только то, что иначе не разрешается: стандартных полей (DefaultPicture, Posted...) в FieldsMap тоже нет.
        if !path.starts_with('\u{1}') {
            return Some(path);
        }
        let parts = d.list().get(1..).unwrap_or(&[]);
        if let ([first, f], "2") = (parts, atom_at(d, 0)) {
            let id = atom_at(first, 0);
            let list = match atom_at(first, 1) {
                "" => Some(id),
                ELEM => self.tables.get(id).filter(|t| atom_at(t, 0) == "1").and_then(|t| t.get(1)).filter(|p| p.get(1).is_none()).map(|p| atom_at(p, 0)),
                _ => None,
            };
            // Дыра в номерах FieldsMap — поле удалено, путь не пишется; номер за наибольшим — поле своего запроса
            // (`list_extension`, разрешено в `cols`), а за последним из них — тоже не пишется.
            let dl = |l: &str| self.lists.get(l).is_some_and(|st| st.get(2).and_then(Node::str) == Some("QueryText"));
            if list.is_some_and(|l| dl(l) && self.list_gone(l, atom_at(f, 0))) && f.get(1).is_none() {
                return None;
            }
        }
        Some(path)
    }

    /// Второе и следующие поля своего запроса с одним именем в FieldsMap (ВедомостьНаВыплатуЗарплатыВБанк: Дата 1/133,
    /// Номер 3/140, Ссылка 117/154) — недоступны, `~` одним путем; доступно поле с наименьшим номером. Без своего
    /// запроса (ВидыОтправляемыхДокументов) — как есть.
    fn twin_extra(&self, l: &str, n: &str) -> bool {
        let Some(c) = self.cols.get(&(l, n)) else { return false };
        let manual = self.lists.get(l).is_some_and(|st| st.list().get(2..).unwrap_or(&[]).chunks(2).any(|c| c[0].str() == Some("ManualQuery") && c.get(1).is_some_and(|v| atom_at(v, 1) == "1")));
        let num = |i: &str| i.parse::<u32>().unwrap_or(u32::MAX);
        // Латинское имя стандартного поля при русском с меньшим номером — тоже двойник (ТорговыеПредложения Клиента ЭДО:
        // Ссылка 8, Ref 25 → `~Список.Ref`); обратное — нет (БП ДокументооборотыОбменаСБанками: Description, затем
        // Наименование — доступно). shortcut: правило по двум образцам, не из кода — при новом расхождении искать в mngbase.
        let c: &str = c;
        let same = |x: &str| x == c || STD_LIST_FIELDS.iter().any(|&(e, r)| !r.is_empty() && x == r && c == e);
        manual && self.cols.iter().any(|((a, i), x)| *a == l && same(x) && num(i) < num(n))
    }

    /// Номер `n` поля динамического списка `l` не из FieldsMap: дыра (поле удалено) или за последним полем своего
    /// запроса — платформа его не пишет (ни путь элемента, ни поле UseAlways).
    fn list_gone(&self, l: &str, n: &str) -> bool {
        if self.cols.contains_key(&(l, n)) {
            return false;
        }
        let max = self.cols.keys().filter(|(a, _)| *a == l).filter_map(|(_, i)| i.parse::<u32>().ok()).max();
        // Без своего запроса за последним номером полей нет вовсе (БП 3.0.107 ПодборНематериальныхАктивов: ReqMap 70–74 при 33).
        let auto = self.lists.get(l).is_some_and(|st| st.list().get(2..).unwrap_or(&[]).chunks(2).any(|c| c[0].str() == Some("ManualQuery") && c.get(1).is_some_and(|v| atom_at(v, 1) == "0")));
        n.parse::<u32>().is_ok_and(|n| match self.extent.get(l) {
            Some(&(m, e)) => n < m || n > e,
            None => max.is_some_and(|m| n < m || auto && n > m),
        })
    }

    /// Путь, который не разрешила и платформа (нет реквизита формы или элемента, реквизита метаданных в конфигурации,
    /// или он чужого объекта): части через `/`, каждая `код` или `код:guid`.
    fn lost(&self, parts: &[Node]) -> Option<String> {
        let first = parts.first()?;
        let id = atom_at(first, 0);
        let missing = match atom_at(first, 1) {
            "" => !self.attrs.contains_key(id),
            ELEM => !self.items.contains_key(id),
            _ => false,
        };
        let special = [ROW_INDEX, ADDED_COLUMN, REPORT_COMPOSER, ELEM];
        let unknown = parts.iter().skip(1).map(|p| atom_at(p, 1)).any(|g| !g.is_empty() && !special.contains(&g) && !self.names.full.contains_key(g));
        // У текущих данных таблицы — колонка не из табличной части, на которую указывает путь таблицы.
        let ts = (atom_at(first, 1) == ELEM).then(|| self.tables.get(id)).flatten().and_then(|d| d.list().get(1..=num_at(d, 0).max(0) as usize)?.last().map(|p| atom_at(p, 1)));
        let foreign_col = |g: &str| ts.and_then(|t| self.names.full.get(t)).is_some_and(|t| t.rsplit('.').nth(1) == Some("TabularSection") && self.names.full.get(g).is_some_and(|f| !f.starts_with(&format!("{t}."))));
        let prim = atom_at(first, 1).is_empty() && self.prim.contains(id) && parts.get(1).is_some_and(|p| !atom_at(p, 1).is_empty());
        let foreign = prim || parts.get(1).is_some_and(|p| self.foreign(id, atom_at(p, 1)) || foreign_col(atom_at(p, 1)));
        let raw = |p: &Node| match atom_at(p, 1) {
            "" => atom_at(p, 0).to_string(),
            g => format!("{}:{g}", atom_at(p, 0)),
        };
        (missing || unknown || foreign).then(|| parts.iter().map(raw).collect::<Vec<_>>().join("/"))
    }

    /// Реквизит метаданных `g` не из объекта типа реквизита формы `attr` (ссылка осталась от копирования формы).
    fn foreign(&self, attr: &str, g: &str) -> bool {
        match (self.owners.get(attr), self.names.full.get(g)) {
            // Общий реквизит входит в состав многих объектов.
            (Some(o), Some(f)) => !f.starts_with(&format!("{o}.")) && !f.starts_with("CommonAttribute."),
            _ => false,
        }
    }

    /// Есть ли у реквизита-списка `attr` колонка с именем `ru` (для Names::mark).
    fn ru_col<'s>(&'s self, attr: &'s str) -> impl Fn(&str) -> bool + 's {
        // Без полей своего запроса за наибольшим номером FieldsMap (`list_extension`): их платформа добавляет при загрузке.
        let own = move |a: &str, i: &str| !self.extent.get(a).is_some_and(|&(m, _)| i.parse::<u32>().is_ok_and(|n| n > m));
        move |ru| self.cols.iter().any(|((a, i), c)| *a == attr && *c == ru && own(a, i))
    }

    fn current_data(&self, item: &str, rest: &[Node]) -> Option<String> {
        let name = self.items.get(item)?;
        let d = self.tables.get(item)?;
        let k = num_at(d, 0);
        let base = self.data_path(d)?;
        let mut l = d.list().get(..=k as usize)?.to_vec();
        l[0] = Node::Atom((k + rest.len() as i64).to_string());
        l.extend(rest.iter().cloned());
        let full = self.data_path(&Node::List(l))?;
        // Недоступное поле — `~путь` или `~путь~путь` (см. Names::mark): каждая часть — от текущих данных.
        let base = base.trim_start_matches('~');
        let parts: Option<Vec<String>> = full.split('~').map(|p| p.strip_prefix(base).map(|tail| format!("Items.{name}.CurrentData{tail}")).or(p.is_empty().then(String::new))).collect();
        (!full.starts_with('\u{1}')).then_some(parts?.join("~"))
    }

    /// Продолжение пути от реквизита формы: `{0,guid}` — реквизит метаданных, `{N}` — стандартное поле по классу
    /// (отрицательные у объектов, положительные у платформенных типов) или колонка реквизита формы.
    fn sub_path(&self, mut out: String, class: &str, attr: &str, parts: &[Node]) -> Option<String> {
        let mut class = class;
        let mut prev = "";
        let root = out.clone();
        for p in parts {
            let code = num_at(p, 0);
            let g = atom_at(p, 1);
            if g == ROW_INDEX {
                out = format!("{out}[{code}]");
                continue;
            }
            let mut side = "";
            let s = if g == ADDED_COLUMN {
                // Дополнительная колонка: номер — id колонки, таблица — путь до нее (без индексов) или предыдущая ТЧ.
                let table: String = out.split('[').enumerate().map(|(i, s)| if i == 0 { s } else { s.split_once(']').map_or(s, |x| x.1) }).collect();
                let c = match self.xcols.get(&(table, atom_at(p, 0).to_string())) {
                    Some(c) => *c,
                    None => self.cols.get(&(prev, atom_at(p, 0))).copied()?,
                };
                // дальше — реквизиты типа колонки, не табличной части (`Объект.Наработки.ОбъектФормы.ИнвентарныйНомер`)
                prev = "";
                c
            } else if g == REPORT_COMPOSER {
                class = "SettingsComposer";
                "SettingsComposer"
            } else if code == 101000000 && !g.is_empty() {
                // Итог колонки табличной части.
                out = format!("{out}.Total{}", self.names.short(g)?);
                continue;
            } else if let Some((s, next)) = g.is_empty().then(|| composer_field(class, code)).flatten() {
                class = next;
                s
            } else if class == "AccountingRegister" && g == EXT_DIM && (0..3).contains(&code) {
                out = format!("{out}.ExtDimension{}", code + 1);
                class = "";
                continue;
            } else if class == "AccountingRegister" && (g == EXT_DIM_DR || g == EXT_DIM_CR) && (0..3).contains(&code) {
                // Субконто записи регистра бухгалтерии: GUID платформы, номер — с нуля (ОперацияБух, КорректировкаДвижений).
                out = format!("{out}.ExtDimension{}{}", if g == EXT_DIM_DR { "Dr" } else { "Cr" }, code + 1);
                class = "";
                continue;
            } else if !g.is_empty() {
                if out == root && (self.foreign(attr, g) || self.prim.contains(attr)) {
                    return None;
                }
                // Колонка чужой табличной части (форма скопирована из другого объекта) — путь не разрешается.
                if self.names.full.get(prev).is_some_and(|t| t.rsplit('.').nth(1) == Some("TabularSection") && self.names.full.get(g).is_some_and(|f| !f.starts_with(&format!("{t}.")))) {
                    return None;
                }
                // Поле записи регистра бухгалтерии по счету: код 2 — Dr, 3 — Cr (`КоличествоDr`); другой код не встречался.
                side = match (class == "AccountingRegister", code) {
                    (false, _) | (true, 0) => "",
                    (true, 2) => "Dr",
                    (true, 3) => "Cr",
                    _ => return None,
                };
                let full = self.names.full.get(g)?;
                class = match full.rsplit('.').nth(1) {
                    Some("TabularSection") => "TabularSection",
                    // набор записей регистра в движениях документа (`Объект.RegisterRecords.Хозрасчетный.Active`)
                    Some(r @ ("InformationRegister" | "AccumulationRegister" | "AccountingRegister" | "CalculationRegister")) if full.matches('.').count() == 1 => r,
                    _ => self.names.attr_type.get(g).and_then(|t| self.names.types.get(t).map(|t| type_class(t)).or_else(|| platform_type(t))).unwrap_or(""),
                };
                prev = g;
                self.names.short(g)?
            } else if let Some(s) = std_field(class, code) {
                // ведущие, вытесняющие, базовые виды расчета — таблицы с колонкой CalculationType
                if class == "ChartOfCalculationTypes" && [-10, -20, -30].contains(&code) {
                    class = "CalculationTypes";
                }
                s
            } else if code == 100000000 {
                "RowsCount"
            } else if code > 0 {
                let c = self.cols.get(&(attr, atom_at(p, 0))).copied()?;
                class = self.col_class.get(&(attr, atom_at(p, 0))).map_or("", String::as_str);
                // Поле динамического списка с точкой — полный путь от списка (`Владелец.Description`).
                if c.contains('.') {
                    out = format!("{root}.{c}");
                    continue;
                }
                c
            } else {
                return None;
            };
            out = format!("{out}.{s}{side}");
        }
        Some(out)
    }

    fn button(&self, it: &It, depth: usize) -> Result<String, String> {
        let nm = self.names;
        let mut p = P::new(depth + 1);
        p.local("Title", it.n(6).unwrap_or(&EMPTY));
        p.flag_if("Enabled", it.num(7) == 0, "false");
        if let Some(c) = it.n(8).filter(|c| c.get(1).is_some()) {
            p.leaf("CommandName", &self.command(c));
        } else if let Some(c) = it.n(8) {
            p.leaf("CommandName", atom_at(c, 0));
        }
        if let Some(d) = it.n(9).and_then(|d| self.item_path(d)) {
            p.leaf("DataPath", &d);
        }
        p.en("Representation", &["Text", "Picture", "PictureAndText"], it.num(10), 3);
        if it.num(11) == 1 {
            p.leaf("DefaultButton", "true");
            p.leaf("ButtonImportance", "Main");
        }
        p.flag_if("DefaultItem", it.num(13) == 1, "true");
        p.num_nz("Width", it.a(16));
        p.num_nz("Height", it.a(17));
        p.num_nz("TitleHeight", it.a(18));
        p.color("BackColor", it.n(19), nm);
        p.color("TextColor", it.n(20), nm);
        p.color("BorderColor", it.n(21), nm);
        p.font("Font", it.n(22), nm);
        p.flag_if("Check", it.num(24) == 1, "true");
        p.picture("Picture", it.n(25), nm);
        p.flag_if("Visible", it.num(26) == 0, "false");
        p.tri("SkipOnInput", it.num(29));
        p.en("ToolTipRepresentation", TOOLTIP_REPR, it.num(30), 0);
        p.adjustable("UserVisible", it.uv(), nm);
        let mut k = 31;
        let mut et = None;
        if it.num(k) == 1 {
            et = it.n(k + 1);
            k += 1;
        }
        k += 1;
        if let Some(v) = it.n(k).filter(|v| v.get(0).and_then(Node::str) != Some("U")) {
            let mut w = p.w();
            match (v.get(0).and_then(Node::str), atom_at(v, 1)) {
                // Описание типов и ссылка на объект метаданных (UUID объекта).
                (Some("#"), "f5c65050-3bbb-11d5-b988-0050bae0a95d") => {
                    w.line("<Parameter xsi:type=\"v8:TypeDescription\">");
                    w.depth += 1;
                    type_inner(&mut w, nm, v.get(2).unwrap_or(&EMPTY));
                    w.close("Parameter");
                }
                (Some("#"), "fc01b5df-97fe-449b-83d4-218a090e681e") if nm.full.contains_key(atom_at(v, 2)) => {
                    w.line(&format!("<Parameter xsi:type=\"xr:MDObjectRef\">{}</Parameter>", esc(&nm.full[atom_at(v, 2)])));
                }
                _ => w.leaf("Parameter", &unk("значение", v)),
            }
            p.push("Parameter", w);
        }
        p.flag_if("AutoMaxWidth", it.num(k + 1) == 0, "false");
        p.num_nz("MaxWidth", it.a(k + 2));
        p.flag_if("AutoMaxHeight", it.num(k + 4) == 0, "false");
        p.num_nz("MaxHeight", it.a(k + 5));
        p.flag_if("HorizontalStretch", it.num(k + 6) == 1, "true");
        p.flag_if("VerticalStretch", it.num(k + 7) == 1, "true");
        p.en("GroupHorizontalAlign", H_ALIGN, it.num(k + 8), 3);
        p.en("GroupVerticalAlign", V_ALIGN, it.num(k + 9), 3);
        p.en("RepresentationInContextMenu", &["None", "AdditionalInContextMenu", "OnlyInContextMenu"], it.num(k + 10), 3);
        p.en("Shape", &["", "Usual", "Oval"], it.num(k + 11), 0);
        match it.num(k + 12) {
            3 => {
                p.leaf("ShapeRepresentation", "None");
                p.leaf("ButtonImportance", "Supplementary");
            }
            c => p.en("ShapeRepresentation", &["", "Always", "WhenActive"], c, 0),
        }
        // У старых версий кнопки (25) хвоста нет, вид — только в начале (4).
        let ty = if it.n(k + 13).is_some() { it.num(k + 13) } else { it.num(4) };
        p.leaf("Type", ["CommandBarButton", "UsualButton", "Hyperlink", "CommandBarHyperlink"].get(ty as usize).copied().unwrap_or("\u{1}"));
        p.en("PictureLocation", &["", "Left", "Right"], it.num(k + 14), 0);
        if it.n(k + 16).is_some() {
            p.en("LocationInCommandBar", &["", "InAdditionalSubmenu", "InCommandBar", "InCommandBarAndInAdditionalSubmenu"], it.num(k + 16), 0);
        } else {
            // У версий до 30 — в 15 и в своей нумерации (2 — авто).
            p.en("LocationInCommandBar", &["InAdditionalSubmenu", "InCommandBarAndInAdditionalSubmenu"], it.num(15), 2);
        }
        if matches!(it.a(0), "30" | "31") {
            p.importance(it.num(48));
        }
        self.sub_items(&mut p, depth, et, &[])?;
        Ok(Self::element_named("Button", it.n(5).and_then(Node::str).unwrap_or(""), it.id(), depth, p, BUTTON_ORDER))
    }

    fn element_named(tag: &str, name: &str, id: &str, depth: usize, p: P, order: &[&str]) -> String {
        let mut w = W { s: String::new(), depth };
        let head = format!("<{tag} name=\"{}\" id=\"{id}\"{}", esc_attr(name), p.head);
        if p.items.is_empty() {
            w.line(&format!("{head}/>"));
            return w.s;
        }
        w.line(&format!("{head}>"));
        w.depth += 1;
        p.emit(&mut w, order);
        w.close(tag);
        w.s
    }
}

const BUTTON_ORDER: &[&str] = &[
    "Type", "Visible", "UserVisible", "TitleHeight", "Representation", "DefaultButton", "SkipOnInput", "Enabled", "DefaultItem", "Width",
    "AutoMaxWidth", "MaxWidth", "MinWidth", "Height", "AutoMaxHeight", "MaxHeight", "HorizontalStretch", "VerticalStretch", "GroupHorizontalAlign",
    "GroupVerticalAlign", "PlacementArea", "Check", "CommandName", "Parameter", "DataPath", "TextColor", "BackColor", "BorderColor", "Font",
    "Shortcut", "Picture", "Title", "ToolTipRepresentation", "RepresentationInContextMenu", "Shape", "ShapeRepresentation", "PictureLocation",
    "LocationInCommandBar", "CommandUniqueness", "OnMainServerUnavalableBehavior", "ShowAsCard", "BackPicture", "BackPictureEffect",
    "CardPictureAndTitleAlign", "PictureHeight", "ButtonImportance", "ContextMenu", "AutoCommandBar", "ExtendedTooltip", "SearchStringAddition",
    "ViewStatusAddition", "SearchControlAddition", "Events", "ChildItems",
];

fn std_form_command(g: &str) -> Option<&'static str> {
    Some(match g {
        "679b62d9-ff72-4329-bf3a-c0c32b311dd2" => "Cancel",
        "8149a06a-dbf3-4d4d-a275-5385a4196fc7" => "CancelEdit",
        "96e0bc70-f8ff-4732-8119-060923203629" => "CancelSearch",
        "6886601d-276c-4d3f-af0a-05c586025608" => "Change",
        "c9abb6b0-eafd-4505-8312-9a7b6888cbf3" => "ChangeHistory",
        "174e58ce-82ad-4787-b956-9367937f7971" => "ChangeHistory",
        "8e2b82cf-d1ea-46b2-afdf-a8d64e66ea2b" => "Choose",
        "3772996b-41f4-4c47-a5a8-ea397db424ae" => "Close",
        "342c531d-dc73-458a-8ac4-6a746916a33b" => "Copy",
        "68baa1bc-edd1-4d9b-ad80-1d53fb8a7988" => "Copy",
        "4f834c38-add1-45e4-a9f3-cefe3efac5c9" => "Create",
        "d8772fd1-a3bf-417d-8334-c49968dbb45e" => "CreateFolder",
        "198ea630-fda2-4cda-8a23-f999f4c67ee6" => "CustomizeForm",
        "3dd3bd8a-ac1e-44d6-ac83-e7802642a5e2" => "Delete",
        "c32d43de-b820-49d0-bf7a-d70829f48f40" => "Delete",
        "1cc781aa-f32b-4dc7-996a-6c38c3deda5c" => "Delete",
        "d603a249-6eb3-4e38-bb2d-a8a86a8ab156" => "DynamicListStandardSettings",
        "bdefa701-6685-453e-a02a-3683d0cc16d3" => "Find",
        "b520ca45-d8db-4982-b128-bb42a6afd911" => "FindByCurrentValue",
        "9885f4b6-d830-435f-a0e3-6b70ffe0f85c" => "GetURL",
        "39bb0fe9-771d-4dd5-8a6e-2d16984523af" => "Help",
        "ffc5e8d5-40a7-4893-a590-49bd588f9466" => "HierarchicalList",
        "d7e9e72c-8fa7-430c-a3e9-aeadfd57dfc7" => "Ignore",
        "aa042316-63ba-4f10-8d39-3935474562d0" => "LevelDown",
        "e44f9b41-bf53-4837-b4d4-f0ff9cdf0feb" => "LevelUp",
        "a2b927a1-35af-43e3-af73-4af22ac2c0fa" => "List",
        "1c00edb8-a826-4855-9bde-94dbc5f620e5" => "ListSettings",
        "952c2984-9955-415a-8235-5c710aabe732" => "LoadDynamicListSettings",
        "b0c9afb6-320c-4e36-be21-8f6d48116415" => "LoadReportSettings",
        "b08b7a35-583a-4756-b814-0436ff9139c0" => "LoadVariant",
        "39c6a2fb-45cc-41b1-853f-967fb68aa1df" => "MoveItem",
        "03df6ee5-883c-4cc6-b319-d886d1a9b2c8" => "NewWindow",
        "06ee6a21-061e-47f8-81c5-92ae8b8f3b5d" => "No",
        "f3613d5c-20c6-46e5-b4d5-7d712ece1296" => "OK",
        "9758d344-4b1d-4dc9-80bd-81060bc18b2a" => "OutputList",
        "4c569466-1af5-4fc1-9b63-7bf6493097bf" => "Post",
        "3b8cedbc-8e74-4017-b901-d14b09f32f7a" => "Post",
        "87317f86-057f-477e-9045-2da4e4980199" => "PostAndClose",
        "a11fe36e-0b45-4c07-80b3-2346b660a51e" => "Print",
        "fd8f031f-c168-4e1b-8b0c-15eb3057e688" => "Refresh",
        "0fb774df-ec1c-4e23-9ed1-e089974f74bf" => "ReportSettings",
        "1f317795-c420-4a30-b594-c492abc55f7a" => "Reread",
        "71e0226e-ebb2-4e33-8745-0a94a01bbf15" => "RestoreValues",
        "d5c3842d-7252-4370-9174-756a6cc553e5" => "SaveDynamicListSettings",
        "7910bb04-ddcc-4e5d-89f0-104c6ad0f187" => "SaveReportSettings",
        "239f0103-8de9-4fdf-b485-eb5531da7e51" => "SaveValues",
        "9bffcf73-7b1d-4a8d-bf23-5e051af3ee29" => "SaveVariant",
        "eb880cb2-a91f-4ad6-afb7-f0e6d7a1b111" => "SetDateInterval",
        "827b541d-30c1-4f06-aecf-92aa496a0835" => "SetDeletionMark",
        "2e86453d-8958-4c9a-a1b4-b15215eedc2e" => "SetDeletionMark",
        "3a17e914-ec6a-4280-b4df-78914f40522b" => "ShowInList",
        "8d7bcd38-1bbb-4dc1-a9ad-cc9d5966ca8e" => "Start",
        "e6a9041f-4d43-4f06-8e17-e95753531565" => "StartAndClose",
        "0b83270d-7f95-4cdd-93c3-342d7991fed5" => "Tree",
        "441362c1-0c86-4f73-bf50-6e1048a2db73" => "UndoPosting",
        "389ef1f1-97ce-4326-adf5-886b2dead75c" => "UndoPosting",
        "fe558fde-99b3-45d0-a060-9fc2905309f6" => "Write",
        "32df4349-2607-4c2b-a4b9-bca4a1a28bd7" => "WriteAndClose",
        "5d41082e-9619-42ec-b96f-98b082b3a2f0" => "Yes",
        _ => return None,
    })
}

fn std_item_command(g: &str) -> Option<&'static str> {
    Some(match g {
        "b0016a68-ec64-4e6d-b905-c71fd62efc4c" => "Add",
        // Сопоставлены по таблицам БП с одним незнакомым GUID и одним недостающим именем.
        "8af6ebff-cd02-4bfe-a984-44a292623708" => "ShowRowRearrangement",
        "11761e12-cf32-4826-a175-b23213e3b229" => "ChangeHistory",
        "d96b0c03-b209-4d01-a3fc-17a14f873b64" => "SearchHistory",
        "7d4db5ed-0981-4020-b3b8-886b7165ba05" => "SetPresentation",
        "e6900951-1a42-4397-bf00-cabb2cd7ad6d" => "Detailed",
        "fca750bc-4fb6-40e2-ae0f-e818939a32e7" => "AddFilterItem",
        "a5fdef31-bbf0-4a9d-98aa-fd5fd8f1344a" => "AddFilterItemGroup",
        "62ff963c-9426-43af-bb23-1d2ef3a9a0c1" => "AddOrderItem",
        "ab0ebc39-68ee-4034-b2f4-43eee55bd651" => "AlignCenter",
        "56ae90b6-588f-406e-919c-cc5cc7f86297" => "AlignJustify",
        "87ecfbdd-8e2b-4ba2-a315-0897020f382f" => "AlignLeft",
        "e428af27-c4f7-4577-b80e-95a79f94322d" => "AlignRight",
        "17724105-6e59-4d52-8a42-cf0fb4838037" => "BackColor",
        "f20eefc2-f819-4ab1-be67-87b3ca2e26e6" => "Bold",
        "4efdcc95-9f24-4652-a5ed-febfaa51f135" => "BorderAll",
        "5ff37d88-ba18-428c-bf49-9ccde4c46268" => "BorderBottom",
        "c1c95e8a-37b4-477c-9df1-7bc0fdbc1bd3" => "BorderColor",
        "c4713141-07d1-411f-8804-172d4b8c4f01" => "BorderInside",
        "635136cc-3a47-43d1-8893-d7d536bf37c4" => "BorderLeft",
        "1177792e-7da2-4755-81a9-997f2d0e3dce" => "BorderNone",
        "05183b65-aa1f-401c-ac34-d99a6ff4ff60" => "BorderOutline",
        "e05dddd1-40ad-44a2-8ae7-0a551f0b1809" => "BorderRight",
        "81be7ad1-a93f-4936-92d0-d59c10213f28" => "BorderTop",
        "a0033f06-56f7-4855-b901-7ac66fe1bb99" => "BulletedList",
        "44ad3ec9-f3c2-4913-9224-5f9fb6418743" => "CancelSearch",
        "b41f5bbc-ba5d-4888-8cd1-db246a371418" => "Change",
        "871100d5-049d-4b22-a46a-fabf54bd64c3" => "Char",
        "18248aa8-e621-4e19-a611-54fb8923644c" => "CheckAll",
        "8969c93a-23e5-4bef-941d-aaef315858d2" => "Choose",
        "59e67a77-8141-42cf-b062-7cb92e210b6d" => "ClearAll",
        "7eae9c22-db31-4f27-a56a-b4dd62d21a2c" => "ClearContent",
        "ff5c34f8-b172-4ef2-91d3-48283a66a725" => "CollapseAllGroups",
        "97407339-2c9f-400b-bd5b-3d97b6d00c21" => "ColumnWidth",
        "0ae4bea5-23be-42a7-b69e-97b11b29c453" => "Copy",
        "88078230-1f6b-415f-99e4-ad2ff73810cf" => "CopyToClipboard",
        "7a294bdc-b86b-4b73-abc4-df9c811f61ef" => "CopyToClipboard",
        "1ba33890-92e9-42a3-95bd-a5c783f46d55" => "CopyToClipboard",
        "0f8d6d98-2f8b-405a-b8b3-0538e9d95da5" => "Create",
        "b59f3c87-e213-4947-abae-9dbaffaef147" => "CreateByParameter",
        "d82ca05c-2966-4d77-9a39-a1eea087bfa7" => "CreateFolder",
        "83670388-2e45-439e-9968-587eca6c7f8d" => "CutToClipboard",
        "ec647dcc-2be7-486c-9046-d8b371f9909e" => "DecreaseFontSize",
        "39f6b9f1-7aa1-4a03-a01b-e127d51bc228" => "DecreaseIndent",
        "8d772f97-c0ef-47c0-9cb0-efea28c61341" => "Delete",
        "ec576e13-1e76-4c33-98aa-a33204514227" => "Delete",
        "202efa3a-32e6-482c-8bc9-1a596bdd57f4" => "DeleteColumns",
        "531b9a9b-e4b3-4fe2-9562-1ff678c6d73d" => "DeleteComment",
        "7b63d2c2-5b6b-472e-8f6c-7438d952be73" => "DeleteRows",
        "33b7b9cd-6979-4435-8c58-d9bc8250edec" => "DynamicListStandardSettings",
        "5c52ec51-6000-4190-b8c8-bd6201a271f5" => "Edit",
        "93d90e38-02a4-42f8-828a-2798f51c4500" => "Ellipse",
        "9ef79140-3de6-436a-8dda-610bb963f5db" => "EndEdit",
        "12acffde-8389-4e5e-bd86-ff248262d84a" => "ExpandAllGroups",
        "c0519548-2a9a-44de-a25e-faf01e089d4d" => "Find",
        "0cf34151-92d3-42fd-954f-5938433908a4" => "Find",
        "714d44cc-63da-4431-b33a-428e398d2a08" => "FindByCurrentValue",
        "29fc1bfc-7bf8-4849-9b4b-3e42d12ebcbb" => "FindNext",
        "474180f3-c5bd-49bf-bfd4-fddb03918295" => "FindPrevious",
        "b34f83ab-1cd7-41b1-89bd-dd0804a47b26" => "FixTable",
        "a8f6b59e-b712-4d3e-a974-55a3be4eb295" => "Font",
        "05e95a55-947e-4dde-a657-16ec11750a2f" => "Font",
        "0e36114c-5b59-4005-9426-374a6c067e4a" => "GetURL",
        "32a87619-85ce-495a-a195-2719f5e9c71e" => "GoBack",
        "85a1b7ee-e94d-4783-b519-4af123f58596" => "GoForward",
        "a4e92b1d-5e86-4a86-b276-00e71ecf3fd4" => "GoToBegin",
        "25d773e7-9961-49fc-a9c8-527079090143" => "GoToCell",
        "3e80231d-e169-456e-8373-0b9d4c15c8ab" => "GoToEnd",
        "e406e2a0-f06b-4402-b8c3-9017c95df44c" => "Group",
        "4a817da0-5797-4e16-906f-02fb869e1873" => "GroupFilterItems",
        "b573b54a-ce87-4078-bd21-4f06709157c6" => "Hide",
        "01833a5a-6553-4c49-b445-095018107bb5" => "HierarchicalList",
        // Таблицы доступных полей компоновщика (БП: 3 из 3, у прочих таблиц не встречались).
        "15664824-eedc-4a92-9f6b-c89a2dead157" => "ChooseAll",
        "d77e5787-b130-4355-8f8f-01ecec82f843" => "Choose",
        "6d83186a-5838-40a5-95e7-8990193adf0a" => "Hyperlink",
        "0bdc43a3-79f0-48d6-bce4-1142542e1a59" => "IncreaseFontSize",
        "d0a4d953-115b-4059-a6cb-6e67f903a4f3" => "IncreaseIndent",
        "468dca2b-17be-4657-bae8-64b94fcf6187" => "InsertColumnsLeft",
        "0a2d962b-5178-4fce-983b-19068b919f41" => "InsertColumnsRight",
        "ed6630f2-c296-43dd-b408-d370513fcebc" => "InsertComment",
        "e5c3a5a6-695d-41bf-9c88-4367fd2a2a6e" => "InsertRows",
        "1d6dbce7-a813-437b-89b8-450319ed13bd" => "InsertRowsBottom",
        "4ecc8cf1-2a26-446f-9fb6-93db4ffee068" => "InsertRowsTop",
        "a8631f01-318a-4da2-80a9-9075c7524463" => "Italic",
        "dc118d99-b351-4e30-9310-e864f2e53ec0" => "LevelDown",
        "0e9b637d-cf6e-4330-8a8f-cd44842e34bb" => "LevelUp",
        "c9b9e671-7c9b-44b5-97e9-dd1ee51a1bfd" => "Line",
        "408f351e-0536-46be-8916-a891db9bfbe6" => "LineSpacing",
        "0d0249a4-2b2f-4fc0-a66f-b36f9494b3cc" => "List",
        "14559f7c-853c-42a4-9ea1-01546107747b" => "ListSettings",
        "182a793b-22a5-4625-b316-6a5be7f88078" => "LoadDynamicListSettings",
        "f4df676f-eefb-48bd-9117-83ee6c207cb5" => "Merge",
        "fa51b106-eae6-44c7-8054-76cbb3100603" => "MoveDown",
        "27bd521a-51c6-4fe7-846d-a98f988774b5" => "MoveItem",
        "37740564-9e86-44a0-bea9-3f485a5a3f91" => "MoveUp",
        "2b5d0007-b74c-4786-a904-37e64bda8414" => "NumberedList",
        "825c1c15-ef8f-47ab-b002-e6b84b3e5b10" => "OutputList",
        "49602716-fea6-497f-8047-726404038857" => "OutputList",
        "41c1bb40-1027-4fd1-a19e-17976100e64b" => "PageSetup",
        "01db2225-b62d-4112-a4b6-d39d627bf79f" => "PageSetup",
        "905692d2-c3e7-4433-8f10-8d2ce35f652b" => "PasteFromClipboard",
        "edf14e37-e755-4d1c-970c-48ed776e3a0e" => "PasteFromClipboard",
        "59b4387d-f5be-4658-901f-bd3068217469" => "Pickup",
        "9d8a3915-de52-4227-91cd-2fce22e09972" => "Picture",
        "a97ea34e-7af2-412c-aa9d-b3393b1914ac" => "Picture",
        "e3dd8850-fc3c-41b1-bbb3-7c66af082608" => "Post",
        "4ca32834-6f9f-4dfb-89ce-6db36931c89b" => "Preview",
        "5aa38159-2001-42ae-8451-f8cabe0762c3" => "Preview",
        "1d13f9a3-402a-46cb-9c68-1709356840f2" => "Preview",
        "a8483976-8b13-416a-9680-133b306dc6b0" => "Print",
        "d673d512-f71a-48a6-ae5d-527a64ffd813" => "Print",
        "e2d6f793-b786-4640-a91b-8d77f73860f1" => "Print",
        "0e355e57-d603-4ac6-998b-c522c43d3668" => "PrintImmediately",
        "7b53bef7-3811-4375-810a-672bf817ace4" => "PrintImmediately",
        "be8800c3-8ccf-444a-bbf0-8f3078ff0ded" => "Properties",
        "852f0fba-4338-4c43-a2da-851fcffd07bb" => "Rectangle",
        "6f1ea963-0807-4de8-b544-b5666f500b05" => "Redo",
        "403bc6e6-b98e-4181-9f43-9c75cbbf82cf" => "Refresh",
        "adc3d2d0-4d84-4038-a453-ab5d693a60bd" => "RemoveName",
        "30f0b852-3284-4ae4-838b-f89b78388bdc" => "RotateClockwise",
        "e28575de-9fe8-4a8e-a285-13250de33d49" => "RotateCounterclockwise",
        "17b9f6bb-74b3-439d-b719-eb236b2fe001" => "RowHeight",
        "d8e20c4d-3519-49aa-80e5-d6d66fee741a" => "Save",
        "5a331cec-bf93-4af5-8f51-80fd7118db47" => "SaveAs",
        "999b786e-0534-4fca-8b62-f706d5336798" => "SaveAs",
        "95b4bc12-2ece-4d7a-b3e2-6f9293620a06" => "SaveDynamicListSettings",
        "1e5ebd8b-32ee-4af9-b85c-3a0417000660" => "ScaleDown",
        "d9117be6-f436-40fd-9670-8d71b99b2477" => "ScaleUp",
        "7b683784-b474-441a-ba63-3d757bd0ffd4" => "SearchEverywhere",
        "ff533ae0-46a9-4e1d-aa3a-6dffa27e076b" => "SearchEverywhere",
        "b67f202a-dcf8-41f3-bda8-1ff9bed5f2ef" => "SelectAll",
        "51c99108-107c-43e1-8918-e48835bf2495" => "SelectAll",
        "cca9b248-5f35-477f-a49d-95da3b7becad" => "SelectAll",
        "daa306cd-a78a-4e74-a14c-739daba624cb" => "SetDateInterval",
        "a2f737a8-0114-4e86-a214-45e5c213fa65" => "SetDeletionMark",
        "5ce7de77-4796-41a0-beb3-7dc420053639" => "SetName",
        "1b680da5-a5ca-4ea7-8db9-df079de39b61" => "Show",
        "83c3121e-60cc-4b47-a296-0c1976f0d766" => "ShowGrid",
        "b55ad06a-ee91-4435-a747-6f51884772d9" => "ShowGroups",
        "4a9ec98e-c814-47b6-911f-694effc10fc5" => "ShowHeaders",
        "e7216412-03ac-4a81-99c2-1d7c28e88e31" => "ShowMultipleSelection",
        "2bbe4e12-06d2-409b-a972-eea585125d83" => "SortListAsc",
        "58b2a785-23f6-4b0e-a324-9a1323285595" => "SortListDesc",
        "02bdf755-7bc7-4e41-a199-f65cef10e6bc" => "SplitCell",
        "db1cd9b3-bdf4-43f5-abd6-c2e4bd85d709" => "Strikeout",
        "80455469-5f1c-4817-a992-756dfee9138f" => "Text",
        "71007f7d-1995-44aa-9125-9926e70a35b5" => "TextColor",
        "65ee8b9b-8a0a-4de8-9a71-3a7a8c43c4ba" => "ThickBorderBottom",
        "08872d94-c57a-470f-a050-0d4d81765df2" => "ThickBorderOutline",
        "a01654df-d7f1-4ec5-8b03-258d953de2e7" => "ThickBorderTop",
        "05468165-f954-45a5-84f2-6641c51f9f23" => "Tree",
        "5048cc44-702b-44e3-8445-9af75c02724d" => "UncheckAll",
        "85bd789b-0047-46f9-9b2e-845907fc1b1d" => "Underline",
        "f5814962-2bef-43dd-b633-a193d4b0970e" => "Undo",
        "04ac7211-e74f-4776-9749-35a9282b1d52" => "UndoPosting",
        "88a56d46-abff-4925-91a2-6592a4664912" => "Ungroup",
        "82b88a24-2856-484a-afd9-55a15bdf9785" => "Ungroup",
        "d7e55d2e-bfea-4d80-b4ad-a1bb31ec2147" => "UseFieldAsValue",
        "1f1e900a-8488-4159-81be-9704eb96906d" => "UserSettingItemProperties",
        // прочие — по таблице описателей команд DLL
        _ => return super::commands_table::ITEM.iter().find(|(id, _)| *id == g).map(|(_, n)| *n),
    })
}



/// Назначение стандартной команды формы: от него зависит, пишет ли платформа ее в CommandSet
/// (команды объекта — только при основном реквизите-объекте, списка — при динамическом списке, отчета — при отчете).
fn command_scope(name: &str) -> u8 {
    match name {
        "Write" | "WriteAndClose" | "PostAndClose" | "Reread" | "ShowInList" => 1,
        "Find" | "CancelSearch" | "FindByCurrentValue" | "Change" | "Create" | "CreateFolder" | "Choose" | "List" | "Tree" | "HierarchicalList"
        | "MoveItem" | "ListSettings" | "OutputList" | "Refresh" | "SetDateInterval" | "GetURL" | "DynamicListStandardSettings"
        | "SaveDynamicListSettings" | "LoadDynamicListSettings" => 2,
        "Copy" | "Delete" | "Post" | "UndoPosting" | "SetDeletionMark" | "ChangeHistory" => 3,
        "ShowMultipleSelection" | "LevelDown" | "LevelUp" => 2,
        // Отмена и окончание редактирования — еще у формы с основным реквизитом-компоновщиком (НастройкаОтборовСписка).
        "CancelEdit" | "EndEdit" => 6,
        "ReportSettings" | "Print" | "LoadVariant" | "SaveVariant" | "NewWindow" | "LoadReportSettings" | "SaveReportSettings"
        | "ChangeVariant" | "Generate" | "Save" | "ChangeSettingsStructure" | "StandardSettings" | "CompactViewMode" => 4,
        "CreateInitialImage" | "ReadChanges" | "WriteChanges" => 1,
        "Execute" => 5,
        _ => 0,
    }
}

/// Команды, которые в БП исключаются только вместе: имена известны, но какой GUID какое имя — нет.
/// Пишутся по алфавиту, поэтому при полном наборе соответствие не нужно; неполный набор — `?guid`.
const COMMAND_GROUPS: &[(&[&str], &[&str])] = &[
    (&["aa042316-63ba-4f10-8d39-3935474562d0", "e44f9b41-bf53-4837-b4d4-f0ff9cdf0feb"], &["LevelDown", "LevelUp"]),
    (
        &["3328a951-c3c8-4f22-b99e-814f7cea6b82", "8b81add7-25af-4df7-a69c-144e3e3e4c8e", "d82e191e-f052-40ee-8691-00cac5b34629"],
        &["CreateInitialImage", "ReadChanges", "WriteChanges"],
    ),
    (&["48e12019-0fd6-46eb-aab6-2acba716a623", "fc120c02-7f39-469b-b357-b2dd8d4b0765"], &["AddAutoOrderItem", "Expand"]),
    // Команды табличного документа: исключаются только всем набором (19 и еще 13 — у полей с полным набором).
    (
        &["0e8c7cb4-f146-4208-af36-b3f8c7d71b66", "14bd1c58-da9d-41db-a515-75f8b39fdc52", "4402cb7a-f68e-44cd-9478-52a695b18a25", "49a22a23-d2cf-4f84-97ae-66f94f863145", "5ccf1fce-3fab-4fb6-ac04-a9b2cf689cee", "60abcc40-dc62-4d03-833b-7b8ab8232d2c", "69333d9f-28d1-446b-bd9a-cf8f85cf1704", "719daaab-c2d0-473d-b373-faf18ebe7d9d", "7e79f8d3-6cab-49d5-aac0-43f5056ed958", "7f3f496d-506c-4239-98fb-58e1ea6ba54a", "80a0b41c-24df-40e4-8269-683fb557214d", "9f71febd-8c22-4471-8410-31f455bb3c57", "b383fa5a-2324-4e7e-a166-aabb5d64aea3", "c50fd6b2-51a1-47e0-8cd3-84b16823287c", "ee0aab77-fd5f-4594-9c5e-e989a953642f", "f2b6b156-d929-4be2-af5b-9c9b792524bb", "f5773ab5-4036-49ca-8286-7a4ea2c354d7", "f9395bfa-9301-4cec-8c1e-e2b62fb3abd6", "fd523437-4160-4a52-a70b-9166c7eebcf0"],
        &["AlignDrawingBottom", "AlignDrawingCenter", "AlignDrawingLeft", "AlignDrawingMiddle", "AlignDrawingRight", "AlignDrawingTop", "BringDrawingForward", "BringDrawingToFront", "CombineToGroup", "DistributeDrawingsHorizontally", "DistributeDrawingsVertically", "EqualDrawingHeight", "EqualDrawingSize", "EqualDrawingWidth", "RemoveFromGroup", "RemoveRepeatOnEachPage", "RepeatOnEachPage", "SendDrawingBackward", "SendDrawingToBack"],
    ),
    (
        &["08fdfb5b-192a-41a9-b57a-9781cd3ef7b6", "0c66c888-7512-402c-941d-96bec0e5749a", "1c7e6bb5-54ac-4ebf-8823-e92b3cf629da", "2da58c85-ae4d-403f-b0e2-c50027a5467f", "3a7ef674-f589-4734-9b22-954ea64dc79f", "3e15759b-551a-46c4-8d24-8d6df22a1a64", "41f3fbde-476a-4984-bd12-b32e990af811", "6728e5c7-8f67-4b0d-bd6f-90b728218fe3", "952af05e-0771-4c26-adb6-a3418a262e4a", "95dbc17e-d11e-4008-b9a9-24d5f5b1d061", "9e525e9b-99ed-4d89-9f02-2bf449ba65e6", "e1ae173a-22c3-4909-a72c-5454b64c6446", "feb51db7-bc1f-4b9f-a6e6-db24d5f812ab"],
        &["BlackAndWhiteView", "HeaderFooter", "InsertPageBreak", "Names", "NextComment", "PageViewMode", "PreviousComment", "RemovePageBreak", "RemovePrintArea", "SetPrintArea", "ShowCellNames", "ShowComments", "ShowRowAndColumnNames"],
    ),
];

/// Имена исключенных команд по GUID: `name` для одиночных, `COMMAND_GROUPS` для неразличимых.
/// `name` возвращает `None`, если одиночную команду писать не нужно.
fn grouped<'a>(guids: impl Iterator<Item = &'a str>, name: impl Fn(&str) -> Option<String>) -> Vec<String> {
    let guids: Vec<&str> = guids.collect();
    let mut out = Vec::new();
    for g in &guids {
        if let Some((all, names)) = COMMAND_GROUPS.iter().find(|(all, _)| all.contains(g)) {
            if all.iter().all(|a| guids.contains(a)) {
                if all[0] == *g {
                    out.extend(names.iter().map(|n| n.to_string()));
                }
            } else {
                out.push(format!("\u{1}{g}"));
            }
        } else if let Some(n) = name(g) {
            out.push(n);
        }
    }
    out
}

/// Стандартные команды, которых нет среди команд кнопок.
fn std_form_command2(g: &str) -> Option<&'static str> {
    std_form_command(g).or(match g {
        // mngbase FUN_1802fb3c0 (описатель команд формы)
        "3f01ed62-97f8-465b-b4f7-6517ac2bc994" => Some("Abort"),
        "5174ad3f-0569-42fd-8adf-011d8206db6c" => Some("Retry"),
        "6f959e83-23ec-4991-901d-575d7ea98868" => Some("Activate"),
        "0ea1a92b-3477-44dd-b152-ea7d411f1c5d" => Some("OpenFromStandaloneServer"),
        "573e81b7-57eb-45f0-ba4d-ada7c2537a2d" => Some("OpenFromMainServer"),
        "0ce53bd5-a3c5-43e0-b051-54c835a87be5" => Some("CreateByParameter"),
        "f4613f71-5449-48ed-aea5-de005b272a1d" => Some("SwitchActivity"),
        // Сопоставлены по формам БП, где у GUID ровно одно имя без пары.
        "2cacadf7-8fb3-4ec6-ae2b-0ca3fd311c9e" => Some("Execute"),
        "9fea4ba9-7d33-47d4-a271-cb54df4a9b74" => Some("ShowMultipleSelection"),
        "fb9d7977-258a-440a-9b59-0a650c86f6a2" => Some("ChangeVariant"),
        "74c1abd6-b274-4654-baf0-7b8418b792ea" => Some("EndEdit"),
        "b5e6da6b-cec4-450c-876a-6a5f0837f6cc" => Some("Generate"),
        "a6d73055-3730-42e7-8934-3145ee987141" => Some("Save"),
        "3ea8bf45-5f33-4545-a3bb-29f80666b627" => Some("ChangeSettingsStructure"),
        "c8f1bd8c-b4d1-46d5-97b3-929b5606b6c3" => Some("StandardSettings"),
        "45bea91e-d9e5-4756-b5e6-375ab84b6903" => Some("CompactViewMode"),
        // прочие — по таблице описателей команд формы DLL
        _ => super::commands_table::FORM.iter().find(|(id, _)| *id == g).map(|(_, n)| *n),
    })
}

/// GUID события элемента формы → имя в XML (из сверки с выгрузкой платформы).
fn item_event(g: &str) -> Option<&'static str> {
    Some(match g {
        "0b8dc702-d001-4637-a215-9f35613e096c" => "AdditionalDetailProcessing",
        "de65638d-a806-4a76-bc10-f62bbc86e0e7" => "AfterDeleteRow",
        "178a97c4-0ffe-4fcc-93e6-505369939da5" => "AutoComplete",
        "2391e7b8-7235-45d7-ab7e-6ff3dc086396" => "BeforeAddRow",
        "a7a9dc42-29b6-4c5b-8980-6d0b87149bdd" => "BeforeCollapse",
        "2c8ad9e0-53f0-406d-9c8b-460f6cb6fd74" => "BeforeCreate",
        "6f758c30-b414-4949-a236-584bccb0102a" => "BeforeDelete",
        "2ccfdec5-583d-4eca-8319-e55de492665a" => "BeforeDeleteRow",
        "4d88756d-bad4-4fde-92e1-c1f1402ac6b2" => "BeforeEditEnd",
        "7c39b7bc-db0f-4410-9d98-8e5b7896995e" => "BeforeExpand",
        "c41e7b98-098c-433e-8ac3-56ec2a2c49e2" => "BeforeLoadUserSettingsAtServer",
        "61455593-0982-4415-bc2e-2e8722a7abd0" => "BeforePrint",
        "ab930362-ff94-4dcb-ad16-188805d23e3c" => "BeforeRowChange",
        "82f5f464-5aa5-4efc-9813-51432c74ff8e" => "BeforeStartEdit",
        "b56fa930-855b-4e67-a87c-8ea917d1dfab" => "BeforeStartQuickEdit",
        "b7646583-04d3-4905-8f04-8985914bd1b7" => "BeforeWrite",
        "8bfdb5eb-62dc-4851-8a2c-e983526356bf" => "ChoiceProcessing",
        "f72043b8-2d79-414e-bc4e-3972fe9dbca1" => "ChoiceProcessing",
        "b50dc41b-c15a-4ebe-a17f-d01e51c47de6" => "Clearing",
        "11707a99-4eb9-4373-bc8c-84891483a034" => "Click",
        "9874537f-454c-40ae-83e9-3b9cefbc6d08" => "Click",
        "996b8c30-7a89-4973-8d56-2c9ce2976695" => "Click",
        "eba5f295-c611-4dd9-84b5-22911ad60c53" => "Click",
        "b34abb58-ee89-4119-945c-6432e9ef04a9" => "CommandGenerateProcessing",
        "aeba313d-c467-44b3-b4a2-956340932c8f" => "Creating",
        "2988b2a5-c887-4928-94ae-5d0c9c31e999" => "DetailProcessing",
        // поле диаграммы Ганта (mngcore FUN_1802b6d70)
        "8724b8d4-140d-4357-8ac9-46e29ba7b168" => "DetailProcessing",
        "53325f0c-b112-4c44-ab12-5d1ee0b1f07b" => "DocumentComplete",
        "8ad48496-8d0b-4f6c-ae48-99d95227884b" => "Drag",
        "0d644ff6-443b-4390-86fa-7f9105e42711" => "DragCheck",
        "cb286ab3-3a1c-40d2-a232-6e64f624ccec" => "DragEnd",
        "6d4d6747-a823-4f61-ab31-a426572f2c6c" => "DragStart",
        "14256303-d2b7-4a58-bfab-e77493d10a59" => "EditTextChange",
        "e6e68c5f-dc83-43c1-b6ca-438be00b77c1" => "MultipleValueOpening",
        "ce67decf-16b8-4d61-b347-4e6a063580dc" => "NewWriteProcessing",
        "2042ec93-3108-4190-b767-ec6c10dd9ff4" => "OnActivate",
        "f228b12f-d892-4925-b338-695617357b32" => "OnActivateCell",
        "3793cac5-9f9a-4b7c-adda-386e5cccf794" => "OnActivateDate",
        "6e973761-8683-47fa-a609-4e230950294d" => "OnActivateField",
        "60edb81d-887b-478e-94ee-7fef2b13393d" => "OnActivateRow",
        "fe115cc8-9e33-4684-a166-bd5136fe7a9f" => "OnChange",
        "411a4578-276c-4f4a-b56a-b3b01181c997" => "OnChangeAreaContent",
        "da8dfb86-c5d1-4e35-a8a4-01b167a60ad3" => "OnClick",
        "526c501f-ed3f-4db4-8731-fd0324707501" => "OnCurrentPageChange",
        "2971b9a9-1724-4f34-aaa4-f3db584c3ca0" => "OnCurrentParentChange",
        "5e6c8466-44d0-4fd0-8d87-ac749c16ff60" => "OnCurrentRepresentationPeriodChange",
        "01d80ddd-dce5-4db3-beb5-f63c97cb05b9" => "OnEditEnd",
        "68fc3ed6-a517-4843-a7a4-473cf4e27209" => "OnEditEnd",
        "97365900-eadf-4dfd-a9aa-fbb9ecabd079" => "OnGetDataAtServer",
        "336b3ee5-d67f-4651-b098-e2c53f8317e2" => "OnLoadUserSettingsAtServer",
        "1490ede6-6f33-4c6d-b971-53b2541331ea" => "OnPeriodOutput",
        "a73dae96-734d-42e4-8ae7-b70249ecd233" => "OnSaveUserSettingsAtServer",
        "b3c10170-c5ff-4cba-b537-679e1c872b45" => "OnStartEdit",
        "e91128e6-621d-4dc8-b12e-bd65aeb37e2d" => "OnUpdateUserSettingSetAtServer",
        "ac5a9c5a-5f1d-4fc5-b88c-a187038c16d1" => "Opening",
        "ff33c4d6-a0db-4906-992e-37b3f44cd97a" => "RefreshRequestProcessing",
        "1282f000-23b6-4887-87f4-9e8e79db3d32" => "Selection",
        "22287505-97d8-4258-a318-209e2493f7eb" => "Selection",
        "2feb1ee9-b750-4352-bb4c-67ba1c608dc6" => "Selection",
        "3c3da18f-fc18-4f77-8c2d-96c25bec40a5" => "Selection",
        "b6aaed5c-8a5a-4a03-bd5b-1f14d0f099ec" => "Selection",
        "1960479b-4d89-4eba-8b39-0aa802020558" => "StartChoice",
        "b3b65989-73ac-4db3-b6cb-398cb41a062f" => "StartListChoice",
        "c331eb1b-d32b-4533-844c-1276600b64e3" => "TextEditEnd",
        "70636369-514c-4662-977e-1c3976c9756c" => "Tuning",
        "509eca20-d6e4-4fef-a0f8-3a6b44c64178" => "URLProcessing",
        "d710ea07-5c96-4c43-ab6e-e138d3653780" => "URLProcessing",
        "0d8cf5b0-55eb-4d1e-960a-22c160210945" => "ValueChoice",
        // регистрация событий mngcore.dll (FUN_1802c4880 GUID → обработчик, FUN_1802c48c0 имя → тот же обработчик):
        // таблица FUN_1802ba8a0, поле табличного документа FUN_1802b2b90, поле ввода FUN_1802afc00
        "674956b3-e469-4fdc-acf5-24ebf88cf7ab" => "URLGetProcessing",
        "44498116-1641-4bfa-ae33-86e53c205797" => "URLListGetProcessing",
        "06d41ccc-4e8a-46f8-aeff-b3303cf753d2" => "URLProcessing",
        "9d59f117-9183-4044-be98-829c9cd48754" => "MultipleValuesAdd",
        "49ede602-af78-4a50-b821-ec81f6778f2d" => "MultipleValuesDelete",
        "26b8cded-3de1-4118-90ea-08e3cde193ec" => "MultipleValueURLProcessing",
        // прочие — по таблице событий DLL
        _ => return super::events_table::EVENTS.iter().find(|(id, ..)| *id == g).map(|(_, n, _)| *n),
    })
}

// ---------------------------------------------------------------- таблица и дополнения

const TABLE: &str = "143c00f7-a42d-4cd7-9189-88e4467dc768";
const ADDITION: &str = "c5259a1d-518a-4afd-b98d-0176027e4feb";

const IMPORTANCE: &[&str] = &["Auto", "VeryHigh", "High", "Usual", "Low", "VeryLow"];
const SEARCH_STRING_LOCATION: &[&str] = &["Auto", "None", "CommandBar", "Top", "Bottom", "FormCaption", "PullFromTop"];

impl P {
    /// Важность отображения — атрибут открывающего тега; 0 (авто) не пишется.
    fn importance(&mut self, code: i64) {
        if code > 0 {
            let v = enum_name(IMPORTANCE, code).map_or_else(|| format!("\u{1}{code}"), str::to_string);
            self.head = format!(" DisplayImportance=\"{v}\"");
        }
    }
}

impl Form<'_> {
    /// Источник команд `{N,guid}`: 0 и GUID элемента — форма, N — элемент, особый GUID — глобальные команды.
    fn command_source(&self, p: &mut P, n: Option<&Node>) {
        let Some(n) = n.filter(|n| n.get(1).is_some()) else { return };
        let (id, g) = (atom_at(n, 0), atom_at(n, 1));
        let v = match (id, g) {
            ("0", ELEM) => "Form".to_string(),
            ("0", "2ef6d6fa-847a-485e-8684-d37a3ab5efb8") => "FormCommandPanelGlobalCommands".to_string(),
            // Нет элемента — `id:02023637-...`, как у GroupList (ЗастрахованныеЛицаСЭДО, НастройкиПрямыхВыплатФСС).
            (_, ELEM) => self.items.get(id).map_or_else(|| format!("{id}:{ELEM}"), |s| format!("Item.{s}")),
            _ => unk("источник", n),
        };
        p.leaf("CommandSource", &v);
    }

    /// Путь `{K,часть...}` от пути к данным элемента `d` (картинка строк таблицы).
    fn data_path_from(&self, d: Option<&Node>, rel: &Node) -> Option<String> {
        let k = num_at(rel, 0);
        if k <= 0 {
            return None;
        }
        let d = d?;
        let mut l = d.list().to_vec();
        l[0] = Node::Atom((num_at(d, 0) + k).to_string());
        l.extend(rel.list().iter().skip(1).take(k as usize).cloned());
        self.data_path(&Node::List(l))
    }

    fn table(&self, it: &It, depth: usize) -> Result<String, String> {
        let nm = self.names;
        let mut p = P::new(depth + 1);
        p.en("Representation", &["List", "", "Tree"], it.num(3), 1);
        p.adjustable("UserVisible", it.uv(), nm);
        p.en("TitleLocation", &["None", "Auto", "Left", "Top", "Right", "Bottom"], it.num(6), 0);
        p.num_nz("TitleHeight", it.a(7));
        p.en("CommandBarLocation", &["None", "", "Top", "Bottom"], it.num(8), 1);
        p.local("Title", it.n(9).unwrap_or(&EMPTY));
        p.local("ToolTip", it.n(10).unwrap_or(&EMPTY));
        if let Some(d) = it.n(11).and_then(|d| self.item_path(d)) {
            // Таблица пользовательских настроек компоновщика: ComplexSettingsViewMode Show — в записи свойства нет,
            // платформа пишет всегда (РассылкиОтчетов.ФормаЭлемента, ИнтерактивноеИзменениеВыгрузки: 2 из 2 таких таблиц БП).
            if d.ends_with(".UserSettings") {
                p.leaf("ComplexSettingsViewMode", "Show");
            }
            p.leaf("DataPath", &d);
        }
        p.flag_if("Autofill", it.num(12) == 1, "true");
        p.flag_if("Enabled", it.num(13) == 0, "false");
        p.flag_if("ReadOnly", it.num(14) == 1, "true");
        p.flag_if("DefaultItem", it.num(16) == 1, "true");
        p.flag_if("ChangeRowSet", it.num(17) == 0, "false");
        p.flag_if("ChangeRowOrder", it.num(18) == 0, "false");
        p.num_nz("Width", it.a(19));
        p.num_nz("Height", it.a(20));
        p.num_nz("HeightInTableRows", it.a(21));
        p.flag_if("ChoiceMode", it.num(22) == 1, "true");
        p.en("RowInputMode", &["", "EndOfWindow", "AfterCurrentRow", "BeforeCurrentRow"], it.num(23), 0);
        p.en("SelectionMode", &["SingleRow"], it.num(24), 1);
        if !self.flag(it.id(), super::form_v59::UNSET_ROW_SELECTION) {
            p.leaf("RowSelectionMode", enum_name(&["Cell", "Row"], it.num(25)).unwrap_or("\u{1}"));
        }
        if self.flag(it.id(), super::form_v59::SHOW_COMMAND_BAR) {
            p.leaf("ShowCommandBar", ["false", "auto", "true", "true"].get(it.num(8) as usize).copied().unwrap_or("\u{1}"));
        }
        p.flag_if("Header", it.num(26) == 0, "false");
        p.num_def("HeaderHeight", it.a(27), "1");
        p.flag_if("Footer", it.num(28) == 1, "true");
        p.num_def("FooterHeight", it.a(29), "1");
        p.en("HorizontalScrollBar", &["DontUse", "UseAlways"], it.num(30), 2);
        p.en("VerticalScrollBar", &["DontUse", "UseAlways"], it.num(31), 2);
        p.flag_if("HorizontalLinesBWA", it.num(32) == 0, "false");
        p.flag_if("VerticalLinesBWA", it.num(33) == 0, "false");
        p.flag_if("UseAlternationRowColorBWA", it.num(36) == 0, "false");
        p.flag_if("AutoInsertNewRow", it.num(37) == 1, "true");
        p.en("InitialListView", &["Beginning", "End"], it.num(38), 2);
        p.en("InitialTreeView", &["", "ExpandTopLevel", "ExpandAllLevels"], it.num(39), 0);
        p.en("Output", &["", "Enable", "Disable"], it.num(40), 0);
        p.flag_if("HorizontalStretch", it.num(41) == 0, "false");
        p.flag_if("VerticalStretch", it.num(42) == 0, "false");
        if let Some(s) = it.n(43).and_then(|r| self.data_path_from(it.n(11), r)) {
            // Недоступное поле динамического списка — с `~`.
            let attr = it.n(11).filter(|d| num_at(d, 0) == 1).and_then(|d| d.get(1)).map_or("", |a| atom_at(a, 0));
            let field = it.n(43).and_then(|r| r.get(1)).map_or("", |f| atom_at(f, 0));
            let s = match (self.lists.get(attr).filter(|_| !s.starts_with('~')), field, self.cols.get(&(attr, field))) {
                (Some(st), "10000000", _) => self.names.mark(st, "DefaultPicture", s, &self.ru_col(attr), self.roots.get(attr).map(Vec::as_slice)),
                (Some(st), _, Some(c)) => self.names.mark(st, c, s, &self.ru_col(attr), self.roots.get(attr).map(Vec::as_slice)),
                _ => s,
            };
            p.leaf("RowPictureDataPath", &s);
        }
        p.picture("RowsPicture", it.n(44), nm);
        p.color("BackColor", it.n(45), nm);
        p.color("TextColor", it.n(46), nm);
        p.color("BorderColor", it.n(47), nm);
        p.font("Font", it.n(48), nm);
        p.color("TitleTextColor", it.n(49), nm);
        p.font("TitleFont", it.n(50), nm);
        if let Some(s) = it.n(51).and_then(shortcut) {
            p.leaf("Shortcut", &s);
        }
        p.flag_if("EnableStartDrag", it.num(52) == 1, "true");
        p.flag_if("EnableDrag", it.num(53) == 1, "true");
        for (i, def) in [(15, "0"), (34, "0"), (35, "0")] {
            if it.a(i) != def {
                p.leaf("\u{1}таблица", &format!("\u{1}{i}:{}", it.a(i)));
            }
        }
        // Словарь свойств динамического списка и дерева: (номер, значение)...
        let nd = it.num(54).max(0) as usize;
        let keys: Vec<&str> = (0..nd).map(|j| it.a(55 + 2 * j)).collect();
        for j in 0..nd {
            let key = it.a(55 + 2 * j);
            let v = it.n(56 + 2 * j).unwrap_or(&EMPTY);
            let b = || bool_text(atom_at(v, 1) == "1");
            match key {
                "3" => p.en("ViewMode", &["All", "QuickAccess"], num_at(v, 2), -1),
                "4" => p.leaf("SettingsNamedItemDetailedRepresentation", b()),
                "5" => p.leaf("AutoRefresh", b()),
                "6" => p.leaf("AutoRefreshPeriod", atom_at(v, 1)),
                "7" => {
                    let mut w = p.w();
                    standard_period(&mut w, "Period", v);
                    p.push("Period", w);
                }
                "8" => p.en("ChoiceFoldersAndItems", &["Items", "Folders", "FoldersAndItems"], num_at(v, 2), -1),
                "9" => p.leaf("RestoreCurrentRow", b()),
                "10" | "13" => {
                    let tag = if key == "10" { "TopLevelParent" } else { "RowFilter" };
                    if v.get(0).and_then(Node::str) == Some("U") {
                        p.items.push((tag, format!("\r\n{}<{tag} xsi:nil=\"true\"/>", "\t".repeat(p.depth))));
                    } else {
                        p.leaf(tag, &unk("значение", v));
                    }
                }
                "11" => p.leaf("ShowRoot", b()),
                "12" => p.leaf("AllowRootChoice", b()),
                // UpdateOnDataChange: enums.dll FUN_1800472c0
                "14" => p.leaf("UpdateOnDataChange", ["Auto", "DontUpdate"].get(num_at(v, 2) as usize).copied().unwrap_or("\u{1}")),
                "16" => {
                    let id = atom_at(v, 1);
                    if id != "0" {
                        p.leaf("UserSettingsGroup", &self.item_name(id));
                    } else if !self.zero_items.is_empty() {
                        // 8.5: ключ 16 ставит только загрузка XML, и 0 — порожденная ею панель (единственный элемент с id 0;
                        // БухучетЗарплатыСотрудников.ФормаСписка — выгрузка платформы); несколько таких — не сверено
                        match self.zero_items.as_slice() {
                            [one] => p.leaf("UserSettingsGroup", one),
                            _ => p.leaf("UserSettingsGroup", "\u{1}элемент 0: несколько элементов с id 0"),
                        }
                    }
                }
                // Только у динамического списка (ключ 5); у дерева и таблицы значений не пишется.
                "20" if keys.contains(&"5") => p.leaf("AllowGettingCurrentRowURL", b()),
                "20" => {}
                // 21 — ComplexSettingsViewMode (перечисление 2eb62aaa): загрузка XML ставит умолчание 0 и в XML его не
                // пишет (imp/form/table.rs, 8.5: ИнтерактивноеИзменениеВыгрузки.СоставВыгрузки — выгрузка платформы).
                "21" if v.get(1).and_then(Node::atom) == Some("2eb62aaa-e6c1-48b6-a047-435354d5ae82") && atom_at(v, 2) == "0" => {}
                // 15 `{"U"}` и 19 `{"S",""}` в XML не попадают.
                "15" | "19" if matches!(v.get(0).and_then(Node::str), Some("U") | Some("S")) => {}
                // 0/1 — ограничения использования полей/параметров (XML в base64): пустые в XML не попадают.
                "0" | "1" if v.get(2).and_then(|b| b.get(0)).and_then(Node::atom).and_then(|a| a.strip_prefix("#base64:")).and_then(|b| crate::xml::base64(b).ok()).is_some_and(|x| {
                    let x = String::from_utf8_lossy(&x).into_owned();
                    let body = x.trim_start_matches('\u{feff}').split_once("?>").map_or(x.as_str(), |(_, r)| r).trim().to_string();
                    body.matches('<').count() == 1 && body.ends_with("/>")
                }) => {}
                _ => p.leaf("\u{1}словарь", &unk(key, v)),
            }
        }
        // Словарь динамического списка без ключа 20 — значение по умолчанию.
        if keys.contains(&"5") && !keys.contains(&"20") {
            p.leaf("AllowGettingCurrentRowURL", "true");
        }
        let mut k = 55 + 2 * nd;
        let ev = it.n(k).unwrap_or(&EMPTY);
        k += 2;
        let mut sub = Vec::new();
        for tag in ["ContextMenu", "AutoCommandBar"] {
            if it.num(k) == 1 {
                sub.push((tag, it.n(k + 1)));
                k += 1;
            }
            k += 1;
        }
        let (ch, t) = Self::pairs(it, k)?;
        p.tri("AutoMarkIncomplete", it.num(t));
        p.tri("AutoAddIncomplete", it.num(t + 1));
        p.flag_if("Visible", it.num(t + 2) == 0, "false");
        p.flag_if("MultipleChoice", it.num(t + 3) == 1, "true");
        p.unk_if("\u{1}t4", it.n(t + 4), "{\"Pattern\"}");
        p.unk_if("\u{1}t5", it.n(t + 5), "\"\"");
        p.unk_if("\u{1}t6", it.n(t + 6), "\"\"");
        p.tri("SkipOnInput", it.num(t + 7));
        p.en("SearchOnInput", &["Use", "DontUse"], it.num(t + 8), 2);
        p.en("ToolTipRepresentation", TOOLTIP_REPR, it.num(t + 9), 0);
        let mut k = t + 10;
        let mut et = None;
        if it.num(k) == 1 {
            et = it.n(k + 1);
            k += 1;
        }
        k += 1;
        p.en("SearchStringLocation", SEARCH_STRING_LOCATION, it.num(k), 0);
        p.en("ViewStatusLocation", &["", "None", "Top", "Bottom"], it.num(k + 1), 0);
        p.en("SearchControlLocation", &["", "None", "CommandBar"], it.num(k + 2), 0);
        // Команды — после SearchControlLocation: SearchHistory у таблицы не списка зависит от него. У таблицы без пути к
        // данным команд нет — и CommandSet нет (БП 3.0.107 МатериалыВыданныеСотрудникам: Интервалы; 2 из 2 в эталонах).
        if let Some(c) = it.n(55 + 2 * nd + 1).filter(|_| it.n(11).is_some_and(|d| num_at(d, 0) > 0)) {
            // Команды выбора — только в режиме выбора; у динамического списка без изменения состава строк
            // нет и команд изменения состава.
            let attr = it.n(11).and_then(|d| d.get(1)).map_or("", |f| atom_at(f, 0));
            let dl = self.attrs.get(attr).is_some_and(|a| a.1 == "DynamicList");
            // Основная таблица динамического списка: GUID и класс.
            let main = self.lists.get(attr).and_then(|st| st.list().get(2..)?.chunks(2).find(|c| c[0].str() == Some("MainTable"))?.get(1)).map_or("", |v| atom_at(v, 2));
            let main_class = self.names.full.get(main).map_or("", |f| f.split('.').next().unwrap_or(""));
            let (rows, choice) = (it.num(17) == 1, it.num(22) == 1);
            // Таблица коллекции значений: команд динамического списка (свои GUID) у нее нет — платформа их выбрасывает
            // (каркас: DynamicListStandardSettings и настройки списка у дерева значений), иерархии — нет у таблицы значений.
            let direct = it.n(11).is_some_and(|d| num_at(d, 0) == 1);
            let coll = self.attrs.get(attr).map_or("", |a| a.1.as_str());
            let coll = if direct { coll } else { "" };
            // 07c3d8a2 (ProcessLabelButton) загрузка XML исключает вместе с SearchEverywhere или GetURL, в XML ее нет
            // (imp/form/table.rs; 8.5: ИнтерфейсДокументовЭДО.АрхивЭлектронныхДокументов — выгрузка платформы)
            let label_pair = c.list().iter().skip(1).take(num_at(c, 0).max(0) as usize).filter_map(Node::atom).any(|g| matches!(std_item_command(g), Some("SearchEverywhere" | "GetURL")));
            let seen = |g: &str| {
                dl && label_pair && g == "07c3d8a2-e24d-4d52-b430-f5f7a131429c"
                    || coll == "v8:ValueTree" && DL_SETTINGS_COMMANDS.contains(&g)
                    || coll == "v8:ValueTable" && TREE_COMMANDS.contains(&g)
                    || coll == "DynamicList" && dl_lacks(g, main_class, self.names.older.is_some())
                    // SearchHistory у таблицы не списка без поиска (SearchControlLocation None) платформа выбрасывает
                    // (ФормаТранзитныхСтран, КорректировкаРеализации...: 2 из 2; у списков с None — остается, 6 из 6).
                    || !dl && it.num(k + 2) == 1 && g == "d96b0c03-b209-4d01-a3fc-17a14f873b64"
            };
            // Прочие такие сочетания не встречались — форма отвергается.
            let unseen = |g: &str| {
                matches!(coll, "v8:ValueTree" | "v8:ValueTable" | "v8:ValueListType") && (DL_SETTINGS_COMMANDS.contains(&g) || DL_ONLY_COMMANDS.contains(&g))
                    || coll == "v8:ValueListType" && TREE_COMMANDS.contains(&g)
            };
            let cnt = num_at(c, 0).max(0) as usize;
            if let Some(g) = c.list().iter().skip(1).take(cnt).filter_map(Node::atom).find(|g| !seen(g) && unseen(g)) {
                p.leaf("\u{1}команда", &format!("\u{1}команда списка у таблицы коллекции:{g}"));
            }
            self.command_set_item_by(&mut p, c, |g| seen(g), |n| match n {
                "Choose" => !choice,
                // При выделении одной строки команд множественного выделения нет.
                "SelectAll" | "ShowMultipleSelection" => it.num(24) == 0,                // shortcut: интервал дат снят только у справочников и регистров сведений — так в БП; прочие классы не проверены.
                "SetDateInterval" => self.no_date_interval(main),
                // У таблицы списка без основной таблицы их нет (ПодборОснованийСчетаФактуры; в эталоне ни у одной из 120 таких таблиц).
                "Post" | "UndoPosting" => {
                    self.names.no_posting.contains(main)
                        || matches!(main_class, "Catalog" | "InformationRegister")
                        || dl && it.n(11).is_some_and(|d| num_at(d, 0) == 1) && no_main(main)
                }
                "Copy" | "Create" | "CreateFolder" | "Delete" | "SetDeletionMark" | "MoveItem" if dl && !rows => true,
                // Только у таблицы самого списка: у таблицы его отбора (`Список.SettingsComposer...`) команды есть.
                n => dl && it.n(11).is_some_and(|d| num_at(d, 0) == 1) && no_main(main) && NO_MAIN_COMMANDS.contains(&n),
            });
        }
        k += 3;
        let mut adds = Vec::new();
        for _ in 0..3 {
            if it.num(k) == 1 {
                adds.extend(it.n(k + 1));
                k += 1;
            }
            k += 1;
        }
        p.en("RefreshRequest", &["", "PullFromTop", "PullFromBottom", "PullFromTopOrBottom"], it.num(k), 0);
        p.flag_if("AutoMaxWidth", it.num(k + 1) == 0, "false");
        p.num_nz("MaxWidth", it.a(k + 2));
        p.flag_if("AutoMaxHeight", it.num(k + 4) == 0, "false");
        p.num_nz("MaxHeight", it.a(k + 5));
        p.en("GroupHorizontalAlign", H_ALIGN, it.num(k + 6), 3);
        p.en("GroupVerticalAlign", V_ALIGN, it.num(k + 7), 3);
        p.en("HeightControlVariant", &["", "UseHeightInFormRows", "UseHeightInTableRows", "UseContentHeight"], it.num(k + 8), 0);
        p.flag_if("AutoMaxRowsCount", it.num(k + 9) == 0, "false");
        p.num_nz("MaxRowsCount", it.a(k + 10));
        p.en("CurrentRowUse", &["", "Choice", "SelectionPresentation", "SelectionPresentationAndChoice"], it.num(k + 11), 0);
        // mngcore 0x18043f200
        p.en("BehaviorOnHorizontalCompression", &["", "HideItemsByImportance", "MoveItemsByImportance"], it.num(k + 12), 0);
        p.importance(it.num(k + 13));
        // У старых версий таблицы поля нет — по умолчанию AsFile.
        p.flag_if("FileDragMode", matches!(it.num(k + 14), 0 | i64::MIN), "AsFile");
        for i in [k + 3, k + 15] {
            if !matches!(it.a(i), "0" | "") {
                p.leaf("\u{1}таблица", &format!("\u{1}t{}:{}", i - t, it.a(i)));
            }
        }
        for (tag, n) in sub {
            if let Some(n) = n {
                p.raw(tag, self.item(None, n, depth + 1, Some(tag))?);
            }
        }
        if let Some(e) = et {
            p.raw("ExtendedTooltip", self.item(None, e, depth + 1, Some("ExtendedTooltip"))?);
        }
        for a in adds {
            let s = self.addition(&It::new(a, 4), depth + 1)?;
            let tag = ["SearchStringAddition", "ViewStatusAddition", "SearchControlAddition"].get(It::new(a, 4).num(5) as usize).copied().unwrap_or("\u{1}дополнение");
            p.raw(tag, s);
        }
        let mut w = p.w();
        // События пользовательских настроек и OnGetDataAtServer есть только у таблицы динамического списка; у прочих
        // платформа пишет GUID (OnGetDataAtServer у таблицы значений — БП 3.0.107 УчетнаяЗапись).
        let attr = it.n(11).and_then(|d| d.get(1)).map_or("", |f| atom_at(f, 0));
        let dl = self.attrs.get(attr).is_some_and(|a| a.1 == "DynamicList");
        events_xml(&mut w, &[ev], &|g| match g {
            "c41e7b98-098c-433e-8ac3-56ec2a2c49e2" | "e91128e6-621d-4dc8-b12e-bd65aeb37e2d" | "97365900-eadf-4dfd-a9aa-fbb9ecabd079" if !dl => Some(g.to_string()),
            _ => item_event(g).map(String::from),
        })?;
        p.push("Events", w);
        let c = self.children(&ch, depth + 2)?;
        if !c.is_empty() {
            let ind = "\t".repeat(depth + 1);
            p.raw("ChildItems", format!("\r\n{ind}<ChildItems>{c}\r\n{ind}</ChildItems>"));
        }
        Ok(Self::element_named("Table", it.n(5).and_then(Node::str).unwrap_or(""), it.id(), depth, p, TABLE_ORDER))
    }

    /// Дополнение таблицы: 5 вид (0 строка поиска, 1 состояние просмотра, 2 управление поиском), 13 свойства вида,
    /// меню, подсказка, `{id таблицы,вид}` — источник.
    fn addition(&self, it: &It, depth: usize) -> Result<String, String> {
        let kind = it.num(5);
        let (tag, src) = match kind {
            0 => ("SearchStringAddition", "SearchStringRepresentation"),
            1 => ("ViewStatusAddition", "ViewStatusRepresentation"),
            2 => ("SearchControlAddition", "SearchControl"),
            _ => return Err(format!("дополнение вида {kind}")),
        };
        let nm = self.names;
        let mut p = P::new(depth + 1);
        p.adjustable("UserVisible", it.uv(), nm);
        p.local("Title", it.n(7).unwrap_or(&EMPTY));
        p.local("ToolTip", it.n(8).unwrap_or(&EMPTY));
        p.flag_if("Visible", it.num(9) == 0, "false");
        p.flag_if("Enabled", it.num(10) == 0, "false");
        p.en("ToolTipRepresentation", TOOLTIP_REPR, it.num(11), 0);
        let sub = it.n(13).unwrap_or(&EMPTY);
        let s = |i: usize| num_at(sub, i);
        // Свойства вида: ширина, растягивание, цвета, шрифт, события, AutoMaxWidth, MaxWidth.
        let (aw, rest) = match kind {
            0 => {
                p.num_nz("Width", atom_at(sub, 1));
                p.tri("HorizontalStretch", s(2));
                (8, &[(3, "{3,4,{0}}"), (4, "{3,4,{0}}"), (5, "{3,4,{0}}"), (6, "{7,3,0,1,100}"), (7, "{0,1,0}"), (10, "0")][..])
            }
            1 => {
                p.num_nz("Width", atom_at(sub, 1));
                p.tri("HorizontalStretch", s(2));
                p.en("HorizontalLocation", &["Left", "Center", "Right"], s(11), 3);
                (13, &[(3, "{3,4,{0}}"), (4, "{3,4,{0}}"), (5, "{3,4,{0}}"), (6, "{3,4,{0}}"), (7, "{3,4,{0}}"), (8, "{7,3,0,1,100}"), (9, "{7,3,0,1,100}"), (12, "{0,1,0}"), (15, "0")][..])
            }
            _ => {
                p.num_nz("Width", atom_at(sub, 1));
                p.tri("HorizontalStretch", s(10));
                (7, &[(2, "{3,4,{0}}"), (3, "{3,4,{0}}"), (4, "{3,4,{0}}"), (5, "{7,3,0,1,100}"), (6, "{0,1,0}"), (9, "0")][..])
            }
        };
        p.flag_if("AutoMaxWidth", s(aw) == 0, "false");
        p.num_nz("MaxWidth", atom_at(sub, aw + 1));
        for &(i, def) in rest {
            // Пустые события у старых версий — {0,0,0} или {0}.
            let old_empty = def == "{0,1,0}" && sub.get(i).is_some_and(|n| matches!(brace::write(n).unwrap_or_default().as_str(), "{0,0,0}" | "{0}"));
            if !old_empty {
                p.unk_if("\u{1}дополнение", sub.get(i), def);
            }
        }
        let mut k = 12;
        let mut cm = None;
        let mut et = None;
        if it.num(k) == 1 {
            k += 1;
        }
        k += 1;
        if it.num(k) == 1 {
            cm = it.n(k + 1);
            k += 1;
        }
        k += 1;
        if it.num(k) == 1 {
            et = it.n(k + 1);
            k += 1;
        }
        k += 1;
        let source = it.n(k + 1).unwrap_or(&EMPTY);
        // Источник — командная панель формы (-1): в эталоне такого AdditionSource нет ни разу.
        if atom_at(source, 0) != "-1" {
            let item = self.items.get(atom_at(source, 0)).copied().unwrap_or("\u{1}");
            let mut w = p.w();
            w.open("AdditionSource");
            w.leaf("Item", item);
            w.leaf("Type", src);
            w.close("AdditionSource");
            p.push("AdditionSource", w);
        }
        // За источником — дети `N,(guid,элемент)×N` (кнопка в управлении поиском, ВходящийЗапросФССДляРасчетаПособия).
        let (ch, t) = if it.num(k + 2) > 0 { Self::pairs(it, k + 2)? } else { (Vec::new(), k + 3) };
        p.en("GroupHorizontalAlign", H_ALIGN, it.num(t), 3);
        p.en("GroupVerticalAlign", V_ALIGN, it.num(t + 1), 3);
        p.importance(it.num(t + 2));
        if it.num(k + 2) < 0 || it.l.len() > it.off + t + 3 {
            p.leaf("\u{1}дополнение", &unk("хвост", &Node::List(it.l.to_vec())));
        }
        let c = self.children(&ch, depth + 2)?;
        if !c.is_empty() {
            let ind = "\t".repeat(depth + 1);
            p.raw("ChildItems", format!("\r\n{ind}<ChildItems>{c}\r\n{ind}</ChildItems>"));
        }
        if let Some(c) = cm {
            p.raw("ContextMenu", self.item(None, c, depth + 1, Some("ContextMenu"))?);
        }
        if let Some(e) = et {
            p.raw("ExtendedTooltip", self.item(None, e, depth + 1, Some("ExtendedTooltip"))?);
        }
        Ok(Self::element(tag, it, depth, p, ADDITION_ORDER))
    }
}

/// Стандартный период `{"#",2fdc88ec…,{вариант,начало,конец}}`.
fn standard_period(w: &mut W, tag: &str, v: &Node) {
    let x = v.get(2).unwrap_or(&EMPTY);
    let date = |i: usize| {
        let d = atom_at(x, i);
        if d.len() == 14 {
            format!("{}-{}-{}T{}:{}:{}", &d[0..4], &d[4..6], &d[6..8], &d[8..10], &d[10..12], &d[12..14])
        } else {
            format!("\u{1}{d}")
        }
    };
    w.open(tag);
    let variant = if atom_at(x, 0) == "0" { "Custom".to_string() } else { format!("\u{1}{}", atom_at(x, 0)) };
    w.line(&format!("<v8:variant xsi:type=\"v8:StandardPeriodVariant\">{variant}</v8:variant>"));
    w.leaf("v8:startDate", &date(1));
    w.leaf("v8:endDate", &date(2));
    w.close(tag);
}

const TABLE_ORDER: &[&str] = &[
    "Representation", "Visible", "UserVisible", "TitleLocation", "TitleHeight", "CommandBarLocation", "Autofill", "Enabled", "ReadOnly",
    "SkipOnInput", "DefaultItem", "ChangeRowSet", "ChangeRowOrder", "Width", "AutoMaxWidth", "MaxWidth", "MinWidth", "Height", "AutoMaxHeight",
    "MaxHeight", "HeightInTableRows", "HeightControlVariant", "AutoMaxRowsCount", "MaxRowsCount", "ChoiceMode", "MultipleChoice", "RowInputMode",
    "SelectionMode", "RowSelectionMode", "Header", "HeaderHeight", "Footer", "FooterHeight", "HorizontalScrollBar", "VerticalScrollBar",
    "HorizontalLinesBWA", "VerticalLinesBWA", "FixedLeft", "FixedRight", "UseAlternationRowColorBWA", "AutoInsertNewRow", "AutoAddIncomplete",
    "AutoMarkIncomplete", "MarkRequiredComplete", "SearchOnInput", "InitialListView", "InitialTreeView", "InitialRowActivation", "RowActionsShowType",
    "Output", "HorizontalStretch", "VerticalStretch", "EnableStartDrag", "EnableDrag", "FileDragMode", "DataPath", "RowPictureDataPath",
    "RowsPicture", "TextColor", "BackColor", "BorderColor", "Font", "Title", "TitleTextColor", "TitleFont", "Shortcut", "CommandSet", "ToolTip",
    "ToolTipRepresentation", "SearchStringLocation", "ViewStatusLocation", "SearchControlLocation", "GroupHorizontalAlign", "GroupVerticalAlign",
    "RefreshRequest", "CurrentRowUse", "BehaviorOnHorizontalCompression", "OnMainServerUnavalableBehavior", "HierarchyPanelLocation", "SaveColors",
    "IconSaveColorsKey", "ShapeSaveColorsKey", "ShowCommandBar", "MobileDeviceTableType", "AutoMaxCardHeight", "MaxCardHeight",
    "CardBehaviorOnVerticalCompression", "CellHyperlinksRepresentation", "ViewMode", "SettingsNamedItemDetailedRepresentation", "AutoRefresh",
    "AutoRefreshPeriod", "Period", "ChoiceFoldersAndItems", "RestoreCurrentRow", "TopLevelParent", "ShowRoot", "AllowRootChoice", "RowFilter",
    "UpdateOnDataChange", "UserSettingsGroup", "AllowGettingCurrentRowURL", "ComplexSettingsViewMode", "ContextMenu", "AutoCommandBar",
    "ExtendedTooltip", "SearchStringAddition", "ViewStatusAddition", "SearchControlAddition", "Events", "ChildItems",
];

// Порядок ViewStatusAddition (FUN_18072d2f0 + FUN_18072de20); порядки SearchStringAddition (FUN_18072daf0) и
// SearchControlAddition (FUN_18072e2e0) — его подпоследовательности.
const ADDITION_ORDER: &[&str] = &[
    "Visible", "UserVisible", "Enabled", "ToolTipRepresentation", "PlacementArea", "AdditionSource", "Title", "ToolTip", "GroupHorizontalAlign",
    "GroupVerticalAlign", "Width", "AutoMaxWidth", "MaxWidth", "MinWidth", "HorizontalStretch", "HorizontalLocation", "BackColor", "ButtonColor",
    "TextColor", "TitleTextColor", "BorderColor", "Font", "TitleFont", "Border", "ContextMenu", "AutoCommandBar", "ExtendedTooltip",
    "SearchStringAddition", "ViewStatusAddition", "SearchControlAddition", "Events", "ChildItems",
];

// ---------------------------------------------------------------- поля

const FIELD: &str = "77ffcc29-7f2d-4223-b22f-19666e7250ba";
const FORM_CHOICE_VALUE: &str = "0e704aa2-07bd-48b9-8223-a0212c4d5fc2";
const FIXED_ARRAY: &str = "4500381b-db30-4a10-9db4-990038032acf";

fn field_tag(kind: i64) -> Option<&'static str> {
    Some(match kind {
        1 => "LabelField",
        2 => "InputField",
        3 => "CheckBoxField",
        4 => "PictureField",
        5 => "RadioButtonField",
        6 => "SpreadSheetDocumentField",
        7 => "TextDocumentField",
        8 => "CalendarField",
        9 => "ProgressBarField",
        10 => "TrackBarField",
        11 => "ChartField",
        12 => "GanttChartField",
        14 => "GraphicalSchemaField",
        15 => "HTMLDocumentField",
        17 => "FormattedDocumentField",
        18 => "PeriodField",
        19 => "PlannerField",
        20 => "PDFDocumentField",
        _ => return None,
    })
}

const TITLE_LOCATION: &[&str] = &["None", "Auto", "Left", "Top", "Right", "Bottom"];

/// Значение `{"N",x}`, `{"S","…"}`, `{"B",0}`, `{"D","ГГГГММДДччммсс"}`, `{"#",тип,guid}` → (xsi:type, текст).
fn value(v: &Node, names: &Names) -> Option<(String, String)> {
    let a = |i| atom_at(v, i).to_string();
    Some(match v.get(0).and_then(Node::str)? {
        "N" => ("xs:decimal".into(), a(1)),
        "S" => ("xs:string".into(), v.get(1).and_then(Node::str).unwrap_or("").to_string()),
        "B" => ("xs:boolean".into(), bool_text(atom_at(v, 1) == "1").into()),
        "D" => {
            let d = a(1);
            if d.len() < 14 {
                return None;
            }
            ("xs:dateTime".into(), format!("{}-{}-{}T{}:{}:{}", &d[0..4], &d[4..6], &d[6..8], &d[8..10], &d[10..12], &d[12..14]))
        }
        // Вид счета: 0 Active, 1 Passive, 2 ActivePassive (Хозрасчетный.ФормаСчета).
        "#" if atom_at(v, 1) == "872f7198-7083-4e3e-b57e-a2a9802c769e" => {
            ("ent:AccountType".into(), ["Active", "Passive", "ActivePassive"].get(atom_at(v, 2).parse::<usize>().ok()?)?.to_string())
        }
        // Пустой список значений (ВходящийДокументСЭДОФСС.ФормаВыбораТребования); непустой не встречался.
        "#" if brace::write(v).ok()?.replace("\r\n", "") == r##"{"#",4772b3b4-f4a3-49c0-a1a5-8cb5961511a3,{6,1e512aab-1b41-4ef6-9375-f0137be9dd91,0,0,{0},{"Pattern"},0,-1}}"## => {
            ("xr:ValueList".into(), String::new())
        }
        "#" => {
            let g = atom_at(v, 2);
            ("xr:DesignTimeRef".into(), names.full.get(g)?.clone())
        }
        _ => return None,
    })
}

/// `<tag xsi:type="...">значение</tag>`; незнакомое значение — меткой.
fn value_xml(w: &mut W, tag: &str, v: &Node, names: &Names) {
    match value(v, names) {
        // Неопределено (в БП — у Value и app:value параметров выбора).
        None if brace::write(v).unwrap_or_default() == "{\"U\"}" => w.line(&format!("<{tag} xsi:nil=\"true\"/>")),
        Some((t, s)) if s.is_empty() => w.line(&format!("<{tag} xsi:type=\"{t}\"/>")),
        Some((t, s)) => w.line(&format!("<{tag} xsi:type=\"{t}\">{}</{tag}>", esc(&s))),
        None => w.line(&format!("<{tag}>{}</{tag}>", esc(&unk("значение", v)))),
    }
}

/// Значение списка выбора `{"#",0e704aa2-…,{0,1,значение,guid,guid,{представление}}}`.
fn choice_value(w: &mut W, tag: &str, v: &Node, names: &Names) {
    let inner = v.get(2).unwrap_or(&EMPTY);
    // Фиксированный массив `{"#",4500381b…,{N,значение...}}`.
    if atom_at(v, 1) == FIXED_ARRAY && v.get(0).and_then(Node::str) == Some("#") {
        let items = inner.list().get(1..).unwrap_or(&[]);
        if items.is_empty() {
            w.line(&format!("<{tag} xsi:type=\"v8:FixedArray\"/>"));
            return;
        }
        w.line(&format!("<{tag} xsi:type=\"v8:FixedArray\">"));
        w.depth += 1;
        for x in items {
            choice_value(w, "v8:Value", x, names);
        }
        w.close(tag);
        return;
    }
    if atom_at(v, 1) != FORM_CHOICE_VALUE {
        value_xml(w, tag, v, names);
        return;
    }
    w.line(&format!("<{tag} xsi:type=\"FormChoiceListDesTimeValue\">"));
    w.depth += 1;
    let pres = inner.get(5).unwrap_or(&EMPTY);
    // Только пустые варианты — как без вариантов (СписаниеКодовМаркировкиИСМП).
    if pres.list().iter().skip(2).take(num_at(pres, 1).max(0) as usize).any(|p| p.get(1).and_then(Node::str) != Some("")) {
        local(w, "Presentation", pres);
    } else {
        w.line("<Presentation/>");
    }
    // `{"U"}` с GUID значения — ссылка на предопределенное значение (перечисления).
    let id = atom_at(inner, 4);
    // Владелец по типу ссылки (`CatalogRef.X` → `Catalog.X`): нулевой GUID — пустая ссылка, иначе — предопределенный.
    let owner = names.types.get(atom_at(inner, 3)).and_then(|t| t.split_once("Ref.")).map(|(c, x)| format!("{c}.{x}"));
    let other = owner.and_then(|o| if id == ZERO_GUID { Some(format!("{o}.EmptyRef")) } else { names.predef.get(&format!("{o}.{id}")).cloned() });
    match (inner.get(2).and_then(|v| v.get(0)).and_then(Node::str), names.full.get(id).or(other.as_ref())) {
        // GUID значения объекта конфигурации важнее самого значения (строка у значения перечисления — НастройкаПечатиКонвертов).
        (_, Some(full)) if id != ZERO_GUID && names.full.contains_key(id) => w.line(&format!("<Value xsi:type=\"xr:DesignTimeRef\">{}</Value>", esc(full))),
        (Some("U"), Some(full)) => w.line(&format!("<Value xsi:type=\"xr:DesignTimeRef\">{}</Value>", esc(full))),
        // Тип не из конфигурации — `<тип>.<uuid>`, как DesignTimeRef описаний (common::design_time_ref; ЗУП ИндексацияЗаработка).
        // shortcut: тип из конфигурации с ненайденным значением не встречался — отказ; разобрать, когда встретится.
        (Some("U"), None) if is_guid(atom_at(inner, 3)) && atom_at(inner, 3) != ZERO_GUID && is_guid(id) && id != ZERO_GUID => {
            let t = atom_at(inner, 3);
            match names.types.contains_key(t) {
                false => w.line(&format!("<Value xsi:type=\"xr:DesignTimeRef\">{t}.{id}</Value>")),
                true => w.line(&format!("<Value>{}</Value>", esc(&unk("значение", inner)))),
            }
        }
        // Внутри бывает и фиксированный массив значений.
        _ => choice_value(w, "Value", inner.get(2).unwrap_or(&EMPTY), names),
    }
    w.close(tag);
}

impl P {
    /// Список выбора `{3,N,("представление",значение)...}`.
    fn choice_list(&mut self, n: Option<&Node>, names: &Names) {
        let Some(n) = n else { return };
        let cnt = num_at(n, 1).max(0) as usize;
        if cnt == 0 {
            return;
        }
        let mut w = self.w();
        w.open("ChoiceList");
        for k in 0..cnt {
            w.open("xr:Item");
            w.leaf("xr:Presentation", n.get(2 + 2 * k).and_then(Node::str).unwrap_or(""));
            w.leaf("xr:CheckState", "0");
            choice_value(&mut w, "xr:Value", n.get(3 + 2 * k).unwrap_or(&EMPTY), names);
            w.close("xr:Item");
        }
        w.close("ChoiceList");
        self.push("ChoiceList", w);
    }

    /// Параметры выбора `{0,N,("имя",значение)...}`.
    fn choice_parameters(&mut self, n: Option<&Node>, names: &Names) {
        let Some(n) = n else { return };
        let cnt = num_at(n, 1).max(0) as usize;
        if cnt == 0 {
            return;
        }
        let mut w = self.w();
        w.open("ChoiceParameters");
        for k in 0..cnt {
            let name = n.get(2 + 2 * k).and_then(Node::str).unwrap_or("");
            w.line(&format!("<app:item name=\"{}\">", esc_attr(name)));
            w.depth += 1;
            choice_value(&mut w, "app:value", n.get(3 + 2 * k).unwrap_or(&EMPTY), names);
            w.close("app:item");
        }
        w.close("ChoiceParameters");
        self.push("ChoiceParameters", w);
    }

    /// Число, если отличается от значения по умолчанию.
    fn num_def(&mut self, name: &'static str, v: &str, def: &str) {
        if !v.is_empty() && v != def {
            self.leaf(name, v);
        }
    }

    /// Неразобранные позиции `sub` со значением не как в БП — меткой `?поле`.
    fn unk_at(&mut self, sub: &Node, normal: &[(usize, &str)]) {
        for &(i, v) in normal {
            if let Some(n) = sub.get(i).filter(|n| brace::write(n).unwrap_or_default().replace("\r\n", "") != v) {
                self.leaf("\u{1}поле", &format!("{i}={}", unk("", n)));
            }
        }
    }

    fn unk_if(&mut self, name: &'static str, n: Option<&Node>, normal: &str) {
        if let Some(n) = n {
            let s = brace::write(n).unwrap_or_default().replace("\r\n", "");
            if s != normal {
                self.leaf(name, &unk("", n));
            }
        }
    }
}

impl Form<'_> {
    fn field(&self, it: &It, depth: usize) -> Result<String, String> {
        let kind = it.num(5);
        let tag = field_tag(kind).ok_or_else(|| format!("поле вида {kind}"))?;
        let nm = self.names;
        let mut p = P::new(depth + 1);
        p.en("TitleLocation", TITLE_LOCATION, it.num(7), 1);
        p.num_nz("TitleHeight", it.a(8));
        p.local("Title", it.n(9).unwrap_or(&EMPTY));
        p.local("ToolTip", it.n(10).unwrap_or(&EMPTY));
        if let Some(d) = it.n(11).and_then(|d| self.item_path(d)) {
            p.leaf("DataPath", &d);
        }
        if let Some(d) = it.n(12).and_then(|d| self.data_path(d)) {
            p.leaf("FooterDataPath", &d);
        }
        p.flag_if("Enabled", it.num(13) == 0, "false");
        p.flag_if("ReadOnly", it.num(14) == 1, "true");
        p.tri("SkipOnInput", it.num(15));
        p.flag_if("DefaultItem", it.num(16) == 1, "true");
        p.en("WarningOnEditRepresentation", &["Show", "DontShow"], it.num(17), 2);
        p.local("WarningOnEdit", it.n(18).unwrap_or(&EMPTY));
        p.local("FooterText", it.n(19).unwrap_or(&EMPTY));
        p.flag_if("ShowInHeader", it.num(20) == 0, "false");
        p.flag_if("ShowInFooter", it.num(21) == 0, "false");
        if it.num(22) == 1 {
            p.leaf("CellHyperlink", "true");
            p.leaf("CellHyperlinkRepresentation", "Show");
            p.leaf("CellHyperlinkDisplayVariant", "Always");
        }
        p.en("HorizontalAlign", H_ALIGN, it.num(23), 3);
        p.en("HeaderHorizontalAlign", &["", "Center", "Right", "Auto"], it.num(24), 0);
        p.en("FooterHorizontalAlign", H_ALIGN, it.num(25), 3);
        match it.num(26) {
            0 => p.leaf("EditMode", "Directly"),
            2 => {
                p.leaf("EditMode", "EnterOnInput");
                if !self.flag(it.id(), super::form_v59::UNSET_AUTO_EDIT) {
                    p.leaf("AutoEditMode", "true");
                }
            }
            _ => {}
        }
        p.en("VerticalAlign", V_ALIGN, it.num(27), 3);
        p.flag_if("AutoCellHeight", it.num(28) == 1, "true");
        p.picture("HeaderPicture", it.n(29), nm);
        p.picture("FooterPicture", it.n(30), nm);
        p.color("TitleTextColor", it.n(31), nm);
        p.font("TitleFont", it.n(32), nm);
        p.color("TitleBackColor", it.n(33), nm);
        // 34, 35 — цвета подвала (модель logform: FooterTextColor, FooterBackColor перед FooterFont)
        p.color("FooterTextColor", it.n(34), nm);
        p.color("FooterBackColor", it.n(35), nm);
        p.font("FooterFont", it.n(36), nm);
        if let Some(s) = it.n(37).and_then(shortcut) {
            p.leaf("Shortcut", &s);
        }
        p.adjustable("UserVisible", it.uv(), nm);
        let sub = it.n(39).ok_or("поле: нет свойств вида")?;
        let mut k = 41;
        let mut cm = None;
        if it.num(k) == 1 {
            cm = it.n(k + 1);
            k += 1;
        }
        k += 1;
        p.flag_if("Visible", it.num(k) == 0, "false");
        p.unk_if("\u{1}patternA", it.n(k + 1), "{\"Pattern\"}");
        p.unk_if("\u{1}patternB", it.n(k + 2), "{\"Pattern\"}");
        if let Some(c) = it.n(k + 5) {
            self.command_set_item(&mut p, c, |_| false);
        }
        p.en("FixingInTable", &["", "Left", "Right"], it.num(k + 6), 0);
        p.en("ToolTipRepresentation", TOOLTIP_REPR, it.num(k + 7), 0);
        k += 8;
        let mut et = None;
        if it.num(k) == 1 {
            et = it.n(k + 1);
            k += 1;
        }
        k += 1;
        p.en("GroupHorizontalAlign", H_ALIGN, it.num(k), 3);
        p.en("GroupVerticalAlign", V_ALIGN, it.num(k + 1), 3);
        if kind == 6 {
            p.en("PointerType", &["", "Regular"], it.num(k + 4), 0);
        }
        // У версий 34–35 (БП 3.0.107) — там же.
        if matches!(it.a(0), "34" | "35" | "36" | "37") {
            p.importance(it.num(55));
        }
        let s = |i: usize| num_at(sub, i);
        let sa = |i: usize| atom_at(sub, i);
        let sn = |i: usize| sub.get(i);
        // События вида поля идут после общих.
        let mut evs: Vec<&Node> = vec![it.n(40).unwrap_or(&EMPTY)];
        match kind {
            2 => {
                p.choice_list(sn(1), nm);
                p.num_nz("Width", sa(2));
                p.num_nz("Height", sa(3));
                p.tri("HorizontalStretch", s(4));
                p.tri("VerticalStretch", s(5));
                p.flag_if("Wrap", s(6) == 0, "false");
                p.tri("PasswordMode", s(7));
                p.tri("MultiLine", s(8));
                p.tri("ExtendedEdit", s(9));
                p.tri("MarkNegatives", s(10));
                p.tri("ChoiceListButton", s(11));
                p.tri("ChoiceButton", s(12));
                p.tri("ClearButton", s(13));
                p.tri("SpinButton", s(14));
                p.tri("OpenButton", s(15));
                if let Some(v) = sn(16).filter(|v| v.get(0).and_then(Node::str) != Some("U")) {
                    let mut w = p.w();
                    value_xml(&mut w, "MinValue", v, nm);
                    p.push("MinValue", w);
                }
                if let Some(v) = sn(17).filter(|v| v.get(0).and_then(Node::str) != Some("U")) {
                    let mut w = p.w();
                    value_xml(&mut w, "MaxValue", v, nm);
                    p.push("MaxValue", w);
                }
                if let Some(m) = sn(18).and_then(Node::str).filter(|m| !m.is_empty()) {
                    p.leaf("Mask", m);
                }
                p.flag_if("ListChoiceMode", s(19) == 1, "true");
                p.picture("ChoiceButtonPicture", sn(20), nm);
                p.num_nz("ChoiceListHeight", sa(21));
                p.num_nz("DropListWidth", sa(22));
                p.tri("QuickChoice", s(23));
                p.en("ChoiceFoldersAndItems", &["Items", "Folders", "FoldersAndItems"], s(24), 3);
                let cf = sa(25);
                if !cf.is_empty() && cf != "00000000-0000-0000-0000-000000000000" {
                    p.leaf("ChoiceForm", self.names.full.get(cf).map_or(cf, |s| s.as_str()));
                }
                self.choice_parameter_links(&mut p, sn(26));
                p.choice_parameters(sn(27), nm);
                p.tri("AutoChoiceIncomplete", s(28));
                p.local("Format", sn(29).unwrap_or(&EMPTY));
                p.local("EditFormat", sn(30).unwrap_or(&EMPTY));
                match s(31) {
                    0 | 1 => {
                        p.leaf("MarkRequiredComplete", bool_text(s(31) == 1));
                        p.leaf("AutoMarkIncomplete", bool_text(s(31) == 1));
                    }
                    _ => {}
                }
                p.flag_if("ChooseType", s(32) == 0, "false");
                p.flag_if("IncompleteChoiceMode", s(33) == 1, "OnActivate");
                if let Some(t) = sn(34).filter(|t| t.list().len() > 1) {
                    let mut w = p.w();
                    let mut inner = W { s: String::new(), depth: w.depth + 1 };
                    type_inner(&mut inner, nm, t);
                    w.line("<AvailableTypes>");
                    w.s.push_str(&inner.s);
                    w.line("</AvailableTypes>");
                    p.push("AvailableTypes", w);
                }
                p.flag_if("TypeDomainEnabled", s(35) == 0, "false");
                evs.extend(sn(36));
                p.color("TextColor", sn(37), nm);
                p.color("BackColor", sn(38), nm);
                p.color("BorderColor", sn(39), nm);
                p.font("Font", sn(40), nm);
                p.flag_if("TextEdit", s(41) == 0, "false");
                self.type_link(&mut p, sn(42));
                p.en("EditTextUpdate", &["", "DontUse", "OnValueChange", "Always"], s(43), 0);
                p.local("InputHint", sn(44).unwrap_or(&EMPTY));
                p.tri("CreateButton", s(45));
                p.en("ChoiceButtonRepresentation", &["", "ShowInDropList", "ShowInDropListAndInInputField", "ShowInInputField"], s(46), 0);
                p.tri("DropListButton", s(47));
                p.flag_if("ChoiceHistoryOnInput", s(48) == 1, "DontUse");
                p.flag_if("AutoMaxWidth", s(49) == 0, "false");
                p.num_nz("MaxWidth", sa(50));
                p.flag_if("AutoMaxHeight", s(52) == 0, "false");
                p.num_nz("MaxHeight", sa(53));
                p.en("HeightControlVariant", &["", "UseHeightInFormRows", "UseContentHeight"], s(54), 0);
                p.en("AutoShowClearButtonMode", &["", "", "FilledOnly"], s(55), 0);
                p.en("AutoShowOpenButtonMode", &["", "Always", "FilledOnly"], s(56), 0);
                p.en("AutoCorrectionOnTextInput", &["", "", "DontUse"], s(57), 0);
                p.en("SpellCheckingOnTextInput", &["", "", "DontUse"], s(58), 0);
                p.en("SpecialTextInputMode", &["", "", "", "", "Email", "PhoneNumber", "Digits"], s(60), 0);
                if let Some(m) = sn(62).filter(|m| brace::write(m).unwrap_or_default() != "{0}") {
                    self.multiple_values(&mut p, m, it.n(11));
                }
                p.flag_if("ExtendedEditMultipleValues", s(65) == 1, "true");
            }
            1 => {
                p.num_nz("Width", sa(1));
                p.num_nz("Height", sa(2));
                p.tri("HorizontalStretch", s(3));
                p.tri("VerticalStretch", s(4));
                p.flag_if("MarkNegatives", s(5) == 1, "true");
                p.local("Format", sn(6).unwrap_or(&EMPTY));
                p.flag_if("Hiperlink", s(7) == 1, "true");
                p.color("TextColor", sn(8), nm);
                p.color("BackColor", sn(9), nm);
                p.font("Font", sn(10), nm);
                p.tri("PasswordMode", s(11));
                evs.extend(sn(12));
                p.color("BorderColor", sn(13), nm);
                p.border(sn(14), 0);
                p.flag_if("AutoMaxWidth", s(15) == 0, "false");
                p.num_nz("MaxWidth", sa(16));
                p.flag_if("AutoMaxHeight", s(18) == 0, "false");
                p.num_nz("MaxHeight", sa(19));
            }
            3 => {
                p.flag_if("ThreeState", s(1) == 1, "true");
                p.local("EditFormat", sn(5).unwrap_or(&EMPTY));
                p.num_nz("ItemWidth", sa(9));
                p.num_nz("ItemHeight", sa(10));
                p.tri("EqualItemsWidth", s(11));
                // У свойств версии 10 вид флажка — поле 4 (ВводНачальныхОстатков, ПравилаОтправкиОтчетности...).
                p.en("CheckBoxType", &["", "CheckBox", "Tumbler", "Switcher"], if s(0) == 10 { s(4) } else { s(12) }, 0);
                p.num_nz("ItemTitleHeight", sa(8));
            }
            5 => {
                p.choice_list(sn(1), nm);
                p.num_nz("ColumnsCount", sa(2));
                p.color("TextColor", sn(3), nm);
                p.font("Font", sn(4), nm);
                p.num_nz("ItemHeight", sa(6));
                p.leaf("RadioButtonType", ["Auto", "RadioButtons", "Tumbler"].get(s(7) as usize).copied().unwrap_or("\u{1}"));
                p.num_nz("ItemTitleHeight", sa(9));
                p.num_nz("ItemWidth", sa(10));
                p.tri("EqualColumnsWidth", s(11));
            }
            4 => {
                p.num_nz("Width", sa(1));
                p.num_nz("Height", sa(2));
                p.flag_if("HorizontalStretch", s(3) == 0, "false");
                p.flag_if("VerticalStretch", s(4) == 0, "false");
                p.picture("ValuesPicture", sn(5), nm);
                p.en("PictureSize", PICTURE_SIZE, s(6), 0);
                p.flag_if("Zoomable", s(7) == 1, "true");
                p.flag_if("Hyperlink", s(8) == 1, "true");
                p.local("NonselectedPictureText", sn(9).unwrap_or(&EMPTY));
                p.color("TextColor", sn(10), nm);
                p.color("BorderColor", sn(11), nm);
                p.font("Font", sn(12), nm);
                p.border(sn(13), 1);
                p.flag_if("EnableDrag", s(15) == 1, "true");
                evs.extend(sn(16));
                p.flag_if("AutoMaxWidth", s(17) == 0, "false");
                p.num_nz("MaxWidth", sa(18));
                p.flag_if("AutoMaxHeight", s(20) == 0, "false");
                p.num_nz("MaxHeight", sa(21));
                // У старых версий поля 22 нет — по умолчанию AsFile.
                p.flag_if("FileDragMode", matches!(s(22), 0 | i64::MIN), "AsFile");
                // Каркас (8.5, свойства вида версии 12): поле 25 — PictureColor; у старых версий его нет — платформа пишет
                // цвет текста и PictureColor (СканированныеДокументы...ФормаЭлементаНовая, 6 форм БП).
                p.color("PictureColor", sn(25).or_else(|| sn(10).filter(|_| sub.get(25).is_none())), nm);
            }
            6 => {
                p.num_def("Width", sa(1), "50");
                p.num_def("Height", sa(2), "10");
                p.flag_if("HorizontalStretch", s(3) == 0, "false");
                p.flag_if("VerticalStretch", s(4) == 0, "false");
                p.flag_if("ShowGrid", s(5) == 1, "true");
                p.flag_if("ShowHeaders", s(6) == 1, "true");
                p.flag_if("Protection", s(10) == 1, "true");
                p.en("Output", &["", "Enable", "Disable"], s(12), 0);
                p.flag_if("Edit", s(13) == 1, "true");
                p.flag_if("ShowGroups", s(14) == 0, "false");
                p.color("BorderColor", sn(15), nm);
                p.flag_if("EnableStartDrag", s(16) == 0, "false");
                p.flag_if("EnableDrag", s(17) == 0, "false");
                evs.extend(sn(18));
                p.flag_if("ViewScalingMode", s(19) == 1, "Normal");
                p.flag_if("AutoMaxWidth", s(20) == 0, "false");
                p.num_nz("MaxWidth", sa(21));
                p.flag_if("AutoMaxHeight", s(23) == 0, "false");
                p.num_nz("MaxHeight", sa(24));
                p.flag_if("ShowCellNames", s(25) == 1, "true");
                p.flag_if("ShowRowAndColumnNames", s(26) == 1, "true");
                // Свойства версии 9 (25 полей) без 28/29: полосы — флажки 7/8, платформа пишет оба (4 из 4 полей БП).
                let (v, h) = if sub.get(28).is_some() { (s(28), s(29)) } else { (s(7), s(8)) };
                for (name, c) in [("VerticalScrollBar", v), ("HorizontalScrollBar", h)] {
                    match c {
                        0..=2 => p.tri(name, c),
                        _ => p.leaf(name, &format!("\u{1}{c}")),
                    }
                }
                p.en("SelectionShowMode", &["WhenActive", "", "DontShow", "WhenMultipleCellsSelected"], s(30), 1);
                // поле 31 (mngcore FUN_1802f8630 вид 15): умолчание 2, Show — 0 (обратное загрузке)
                p.en("DrawingSelectionShowMode", &["Show"], s(31), 2);
            }
            // Поля документов: размеры, растягивание (1 — по умолчанию), цвета, шрифт, события, макс. размеры.
            7 | 17 => {
                p.num_def("Width", sa(1), "50");
                p.num_def("Height", sa(2), "10");
                p.flag_if("HorizontalStretch", s(3) == 0, "false");
                p.flag_if("VerticalStretch", s(4) == 0, "false");
                p.unk_at(sub, &[(5, "0"), (6, "{3,4,{0}}"), (7, "{3,4,{0}}")]);
                p.color("BorderColor", sn(8), nm);
                p.font("Font", sn(9), nm);
                let k = if kind == 7 { 10 } else { 11 };
                evs.extend(sn(if kind == 7 { 15 } else { 10 }));
                p.flag_if("AutoMaxWidth", s(k) == 0, "false");
                p.num_nz("MaxWidth", sa(k + 1));
                p.unk_at(sub, &[(k + 2, "0")]);
                p.flag_if("AutoMaxHeight", s(k + 3) == 0, "false");
                p.num_nz("MaxHeight", sa(k + 4));
            }
            8 => {
                // Календарь: {6,Width,Height,?,?,?,ShowCurrentDate,?,даты,шрифт,цвет,?,?,события,ShowMonthsPanel,
                // WidthInMonths,HeightInMonths,...}; значения по умолчанию — из сверки (8 полей БП).
                p.num_def("Width", sa(1), "16");
                p.num_def("Height", sa(2), "9");
                // Поле 5 — SelectionMode: 2 Interval (ПериодКомандировки), прочие не встречались.
                match s(5) {
                    0 => {}
                    2 => p.leaf("SelectionMode", "Interval"),
                    _ => p.leaf("\u{1}поле", &unk("", sub)),
                }
                p.flag_if("ShowCurrentDate", s(6) == 0, "false");
                evs.extend(sn(14));
                p.flag_if("ShowMonthsPanel", s(15) == 1, "true");
                p.num_def("WidthInMonths", sa(16), "1");
                p.num_def("HeightInMonths", sa(17), "1");
                // Поля 3 и 19 встречались только оба 0 (КалендарьБухгалтера): AutoMaxWidth и HorizontalStretch false — какое
                // какое, не установлено; поодиночке и вместе со своими свойствами календаря (порядок не сверен) — отказ.
                match (s(3), s(19)) {
                    (1, 1) => {}
                    (0, 0) if s(5) == 0 && s(6) != 0 && s(15) != 1 && sa(16) == "1" && sa(17) == "1" => {
                        p.leaf("AutoMaxWidth", "false");
                        p.leaf("HorizontalStretch", "false");
                    }
                    _ => p.leaf("\u{1}поле", &unk("", sub)),
                }
                p.unk_at(sub, &[(4, "1"), (7, "1"), (8, "00010101000000"), (9, "00010101000000"), (10, "{7,3,0,1,100}"), (11, "{3,4,{0}}"), (12, "0"), (13, "0"), (20, "0"), (21, "0"), (22, "1"), (23, "0")]);
            }
            9 => {
                // Полоса индикатора (mngcore запись FUN_180310e40, Form.xml mngbase FUN_18071a450): {4,Width 32,Height 1,
                // HorizontalStretch 1,VerticalStretch 0,MinValue 0,MaxValue 100,Orientation 0,Representation 1,ShowPercent,
                // BorderColor,AutoMaxWidth…MaxHeight}
                p.num_def("Width", sa(1), "32");
                p.num_def("Height", sa(2), "1");
                p.flag_if("HorizontalStretch", s(3) == 0, "false");
                p.flag_if("VerticalStretch", s(4) == 1, "true");
                p.num_def("MinValue", sa(5), "0");
                p.num_def("MaxValue", sa(6), "100");
                if s(7) != 0 {
                    p.leaf("\u{1}поле", &format!("Orientation {}", s(7)));
                }
                if s(8) != 1 {
                    p.leaf("\u{1}поле", &format!("Representation {}", s(8)));
                }
                p.flag_if("ShowPercent", s(9) == 1, "true");
                p.color("BorderColor", sn(10), nm);
                p.flag_if("AutoMaxWidth", s(11) == 0, "false");
                p.num_nz("MaxWidth", sa(12));
                p.num_nz("MinWidth", sa(13));
                p.flag_if("AutoMaxHeight", s(14) == 0, "false");
                p.num_nz("MaxHeight", sa(15));
                if s(0) != 4 || sub.list().len() != 16 {
                    p.leaf("\u{1}поле", &unk("", sub));
                }
            }
            10 => {
                // Полоса регулирования (mngcore FUN_180311670, mngbase FUN_18071aa40): {2,Width 32,Height 2,HorizontalStretch 1,
                // VerticalStretch 0,MinValue 0,MaxValue 100,Step 1,Orientation 0,LargeStep 10,MarkingStep 5,MarkingAppearance 2,
                // BorderColor,AutoMaxWidth…MaxHeight}
                p.num_def("Width", sa(1), "32");
                p.num_def("Height", sa(2), "2");
                p.flag_if("HorizontalStretch", s(3) == 0, "false");
                p.flag_if("VerticalStretch", s(4) == 1, "true");
                p.num_def("MinValue", sa(5), "0");
                p.num_def("MaxValue", sa(6), "100");
                p.num_def("Step", sa(7), "1");
                if s(8) != 0 {
                    p.leaf("\u{1}поле", &format!("Orientation {}", s(8)));
                }
                p.num_def("LargeStep", sa(9), "10");
                p.num_def("MarkingStep", sa(10), "5");
                match s(11) {
                    2 => {}
                    1 => p.leaf("MarkingAppearance", "TopLeft"),
                    v => p.leaf("\u{1}поле", &format!("MarkingAppearance {v}")),
                }
                p.color("BorderColor", sn(12), nm);
                p.flag_if("AutoMaxWidth", s(13) == 0, "false");
                p.num_nz("MaxWidth", sa(14));
                p.num_nz("MinWidth", sa(15));
                p.flag_if("AutoMaxHeight", s(16) == 0, "false");
                p.num_nz("MaxHeight", sa(17));
                if s(0) != 2 || sub.list().len() != 18 {
                    p.leaf("\u{1}поле", &unk("", sub));
                }
            }
            11 => {
                // Диаграмма {1,Width,Height,HorizontalStretch,VerticalStretch,события,AutoMaxWidth,MaxWidth,?,AutoMaxHeight,
                // MaxHeight} (растягивание: 1 — умолчание, 0 false): умолчания — поле БП без свойств (ДиаграммаЗамеровВремени).
                p.num_def("Width", sa(1), "50");
                p.num_def("Height", sa(2), "10");
                for (i, name) in [(3, "HorizontalStretch"), (4, "VerticalStretch")] {
                    match s(i) {
                        1 => {}
                        0 => p.leaf(name, "false"),
                        _ => p.leaf("\u{1}поле", &unk("", sub)),
                    }
                }
                evs.extend(sn(5));
                p.flag_if("AutoMaxWidth", s(6) == 0, "false");
                p.num_nz("MaxWidth", sa(7));
                p.unk_at(sub, &[(0, "1"), (8, "0")]);
                p.flag_if("AutoMaxHeight", s(9) == 0, "false");
                p.num_nz("MaxHeight", sa(10));
                if sub.list().len() != 11 {
                    p.leaf("\u{1}поле", &unk("", sub));
                }
            }
            12 => {
                // Диаграмма Ганта (mngcore запись FUN_1803124c0, чтение FUN_1803125d0): {3,Width,Height,HorizontalStretch,
                // VerticalStretch,{события},AutoMaxWidth,MaxWidth,MinWidth,AutoMaxHeight,MaxHeight,TableLocation,
                // ValuesSelectionMode,IntervalsSelectionMode,ShowHorizontalLinesFlag,ShowVerticalLinesFlag}; в XML свойство
                // пишется, когда отличается от умолчания (mngbase FUN_18071b3b0). Своя таблица — предопределенный дочерний
                // элемент вида 6 (FUN_180710f40): в записи флаг k+5 и элемент k+6.
                p.num_def("Width", sa(1), "50");
                p.num_def("Height", sa(2), "10");
                for (i, name) in [(3, "HorizontalStretch"), (4, "VerticalStretch")] {
                    if s(i) == 0 {
                        p.leaf(name, "false");
                    }
                }
                evs.extend(sn(5));
                p.flag_if("AutoMaxWidth", s(6) == 0, "false");
                p.num_nz("MaxWidth", sa(7));
                p.num_nz("MinWidth", sa(8));
                p.flag_if("AutoMaxHeight", s(9) == 0, "false");
                p.num_nz("MaxHeight", sa(10));
                // перечисления mngcore 0x180789160, 0x1807891a0, 0x1807890e0
                p.en("TableLocation", &["Auto", "Left", "Right", "None"], s(11), 0);
                p.en("ValuesSelectionMode", &["Auto", "Multiple", "Single", "None"], s(12), 0);
                p.en("IntervalsSelectionMode", &["Auto", "Multiple", "Single", "None"], s(13), 0);
                p.en("ShowHorizontalLinesFlag", &["false", "true"], s(14), 2);
                p.en("ShowVerticalLinesFlag", &["false", "true"], s(15), 2);
                if s(0) != 3 || sub.list().len() != 16 {
                    p.leaf("\u{1}поле", &unk("", sub));
                }
                match it.n(k + 6).filter(|_| it.num(k + 5) == 1) {
                    Some(t) => p.raw("Table", self.table(&It::new(t, 4), depth + 1)?),
                    None => p.leaf("\u{1}поле", "таблица диаграммы Ганта"),
                }
            }
            14 => {
                // Графическая схема {3,Width,Height,?,?,цвет,события,...}: умолчания 50×10 (поле без Width/Height в БП);
                // прочие поля — как у 3 полей БП, у всех Edit false; иначе отказ.
                p.num_def("Width", sa(1), "50");
                p.num_def("Height", sa(2), "10");
                evs.extend(sn(6));
                p.unk_at(sub, &[(0, "3"), (3, "0"), (4, "0"), (5, "{3,4,{0}}"), (7, "1"), (8, "0"), (9, "0"), (10, "1"), (11, "0"), (12, "1"), (13, "1")]);
                p.leaf("Edit", "false");
                if sub.list().len() != 14 {
                    p.leaf("\u{1}поле", &unk("", sub));
                }
            }
            15 => {
                p.num_def("Width", sa(1), "50");
                p.num_def("Height", sa(2), "10");
                p.color("BorderColor", sn(3), nm);
                p.en("Output", &["", "Enable", "Disable"], s(4), 0);
                evs.extend(sn(5));
                p.flag_if("AutoMaxWidth", s(6) == 0, "false");
                p.num_nz("MaxWidth", sa(7));
                p.unk_at(sub, &[(8, "0")]);
                p.flag_if("AutoMaxHeight", s(9) == 0, "false");
                p.num_nz("MaxHeight", sa(10));
                p.flag_if("HorizontalStretch", s(11) == 0, "false");
                p.flag_if("VerticalStretch", s(12) == 0, "false");
            }
            // Поле PDF: в БП все свойства вида как ниже, кроме 13 ViewStatusLocation (1 None); состояние просмотра —
            // дополнение за флагом после GroupVerticalAlign (k + 4).
            20 => {
                // {1,Width 50,Height 10,BorderColor,Output,события,AutoMaxWidth…MaxHeight,HorizontalStretch,VerticalStretch,
                // ViewStatusLocation} (mngcore FUN_180314390, mngbase FUN_18071ce10)
                p.num_def("Width", sa(1), "50");
                p.num_def("Height", sa(2), "10");
                p.color("BorderColor", sn(3), nm);
                if s(4) != 0 {
                    p.leaf("\u{1}поле", &format!("Output {}", s(4)));
                }
                evs.extend(sn(5));
                p.flag_if("AutoMaxWidth", s(6) == 0, "false");
                p.num_nz("MaxWidth", sa(7));
                p.num_nz("MinWidth", sa(8));
                p.flag_if("AutoMaxHeight", s(9) == 0, "false");
                p.num_nz("MaxHeight", sa(10));
                p.flag_if("HorizontalStretch", s(11) == 0, "false");
                p.flag_if("VerticalStretch", s(12) == 0, "false");
                if s(0) != 1 {
                    p.leaf("\u{1}поле", &unk("", sub));
                }
                // 7274574 = 0x006F004E — «No» строки «None» (так 8.5 загрузила старый XML,
                // РаспознаваниеДокументовОтправкаФайлов.ОтправитьФайлы): платформа свойство не выгружает
                // shortcut: одно значение из одного образца; иные числа вне перечисления — отказ, как прежде
                if s(13) != 7274574 {
                    p.en("ViewStatusLocation", &["", "None"], s(13), 0);
                }
                if sub.list().len() != 14 || !matches!(it.num(k + 4), 0 | 1) {
                    p.leaf("\u{1}поле", &unk("", sub));
                }
                if let Some(a) = it.n(k + 5).filter(|_| it.num(k + 4) == 1) {
                    p.raw("ViewStatusAddition", self.addition(&It::new(a, 4), depth + 1)?);
                }
            }
            // Поле периода и планировщик: по одному полю в БП (ВыборСтандартногоПериодаНесколькоМесяцев, КонтактныеЛица.ФормаЛиды) —
            // свойства вида только такие, иначе отказ; у планировщика поле 7 — события.
            18 => {
                // Поле периода (mngcore FUN_180310730, mngbase FUN_18071a070): {2,Width,Height,Font,BorderColor,события,
                // HorizontalStretch,VerticalStretch,Border,AutoMaxWidth…MaxHeight}; Width/Height писатель сравнивает с 0
                p.num_nz("Width", sa(1));
                p.num_nz("Height", sa(2));
                p.font("Font", sn(3), nm);
                p.color("BorderColor", sn(4), nm);
                evs.extend(sn(5));
                p.flag_if("HorizontalStretch", s(6) == 0, "false");
                p.flag_if("VerticalStretch", s(7) == 0, "false");
                p.border(sn(8), 1);
                p.flag_if("AutoMaxWidth", s(9) == 0, "false");
                p.num_nz("MaxWidth", sa(10));
                p.num_nz("MinWidth", sa(11));
                p.flag_if("AutoMaxHeight", s(12) == 0, "false");
                p.num_nz("MaxHeight", sa(13));
                if s(0) != 2 || sub.list().len() != 14 {
                    p.leaf("\u{1}поле", &unk("", sub));
                }
            }
            19 => {
                // Планировщик (mngcore FUN_18030f290, mngbase FUN_18071c9c0): {1,Width 50,Height 10,HorizontalStretch 1,
                // VerticalStretch 1,EnableStartDrag,EnableDrag,события,TimeScaleItemHyperlink,DimensionItemHyperlink,
                // WrappedTimeScaleHeaderHyperlink}
                p.num_def("Width", sa(1), "50");
                p.num_def("Height", sa(2), "10");
                p.flag_if("HorizontalStretch", s(3) == 0, "false");
                p.flag_if("VerticalStretch", s(4) == 0, "false");
                p.flag_if("EnableStartDrag", s(5) == 1, "true");
                p.flag_if("EnableDrag", s(6) == 1, "true");
                evs.extend(sn(7));
                p.flag_if("TimeScaleItemHyperlink", s(8) == 1, "true");
                p.flag_if("DimensionItemHyperlink", s(9) == 1, "true");
                p.flag_if("WrappedTimeScaleHeaderHyperlink", s(10) == 1, "true");
                if s(0) != 1 || sub.list().len() != 11 {
                    p.leaf("\u{1}поле", &unk("", sub));
                }
            }
            _ => p.leaf("\u{1}поле", &unk("", sub)),
        }
        if let Some(c) = cm {
            p.raw("ContextMenu", self.item(None, c, depth + 1, None)?);
        }
        self.sub_items(&mut p, depth, et, &[])?;
        let mut w = p.w();
        events_xml(&mut w, &evs, &|g| item_event(g).map(String::from))?;
        p.push("Events", w);
        let order: &[&str] = match kind {
            1 => LABEL_FIELD_ORDER,
            2 => INPUT_FIELD_ORDER,
            3 => CHECKBOX_FIELD_ORDER,
            4 => PICTURE_FIELD_ORDER,
            5 => RADIO_FIELD_ORDER,
            6 => SPREADSHEET_FIELD_ORDER,
            7 => TEXT_DOCUMENT_FIELD_ORDER,
            8 => CALENDAR_FIELD_ORDER,
            9 => PROGRESS_BAR_FIELD_ORDER,
            10 => TRACK_BAR_FIELD_ORDER,
            11 => CHART_FIELD_ORDER,
            12 => GANTT_FIELD_ORDER,
            14 => GRAPHICAL_SCHEMA_FIELD_ORDER,
            15 => HTML_DOCUMENT_FIELD_ORDER,
            17 => FORMATTED_DOCUMENT_FIELD_ORDER,
            18 => PERIOD_FIELD_ORDER,
            19 => PLANNER_FIELD_ORDER,
            20 => PDF_DOCUMENT_FIELD_ORDER,
            _ => &[],
        };
        Ok(Self::element(tag, it, depth, p, order))
    }

    /// Множественные значения поля ввода `{1,2,AllowInputEmpty,0,ShowCheckBoxesInDropList (0 false, 2 авто),...,
    /// [9] путь значения, ..., [15] путь представления,...}`; пути — `{1,{колонка}}` реквизита-таблицы из DataPath поля.
    fn multiple_values(&self, p: &mut P, m: &Node, data: Option<&Node>) {
        p.unk_at(m, &[(0, "1"), (1, "2"), (3, "0"), (5, "{7,3,0,1,100}"), (6, "{3,4,{0}}"), (7, "{3,4,{0}}"), (8, "{4,0,{0},\"\",-1,-1,1,0,\"\"}"), (10, "\"\""), (11, "{\"Pattern\"}"), (12, "{0}"), (13, "\"\""), (14, "{\"Pattern\"}"), (16, "\"\""), (17, "{\"Pattern\"}"), (18, "0"), (19, "0")]);
        if m.list().len() != 20 {
            p.leaf("\u{1}multi", &unk("", m));
        }
        p.en("AllowInputEmptyMultipleValues", &["", "true"], num_at(m, 2), 0);
        match num_at(m, 4) {
            0 => p.leaf("ShowCheckBoxesInDropList", "false"),
            // BWA (mngcore 0x18078ae70): 1 true
            1 => p.leaf("ShowCheckBoxesInDropList", "true"),
            2 => {}
            c => p.leaf("ShowCheckBoxesInDropList", &format!("\u{1}{c}")),
        }
        // Колонка — реквизита формы из DataPath поля (путь из одной части).
        let attr = data.filter(|d| num_at(d, 0) == 1).and_then(|d| d.get(1)).map(|a| atom_at(a, 0));
        for (i, name) in [(9, "MultipleValueDataPath"), (15, "MultipleValuePresentDataPath")] {
            let Some(v) = m.get(i).filter(|v| brace::write(v).unwrap_or_default() != "{0}") else { continue };
            let col = attr.filter(|_| num_at(v, 0) == 1 && v.list().len() == 2).and_then(|a| self.cols.get(&(a, v.get(1).map_or("", |c| atom_at(c, 0)))));
            match (col, data.and_then(|d| self.data_path(d))) {
                (Some(c), Some(base)) => p.leaf(name, &format!("{base}.{c}")),
                _ => p.leaf(name, &unk("путь", v)),
            }
        }
    }

    /// Связь по типу `{3,K,часть пути×K,номер элемента}`; `{3,0,0}` — нет связи.
    fn type_link(&self, p: &mut P, n: Option<&Node>) {
        let Some(n) = n.filter(|n| num_at(n, 1) > 0) else { return };
        let k = num_at(n, 1) as usize;
        let Some(parts) = n.list().get(1..2 + k) else { return };
        let path = self.data_path(&Node::List(parts.to_vec())).unwrap_or_default();
        let mut w = p.w();
        w.open("TypeLink");
        w.leaf("xr:DataPath", &path);
        w.leaf("xr:LinkItem", atom_at(n, 2 + k));
        w.close("TypeLink");
        p.push("TypeLink", w);
    }

    /// Связи параметров выбора `{версия,N,("имя",K,часть пути×K,изменение)×N}`: 0 Clear, 1 DontChange.
    fn choice_parameter_links(&self, p: &mut P, n: Option<&Node>) {
        let Some(n) = n.filter(|n| num_at(n, 1) > 0) else { return };
        let l = n.list();
        let mut w = p.w();
        w.open("ChoiceParameterLinks");
        let mut i = 2;
        for _ in 0..num_at(n, 1) {
            let k = num_at(n, i + 1).max(0) as usize;
            let Some(parts) = l.get(i + 1..i + 2 + k) else { break };
            let path = self.data_path(&Node::List(parts.to_vec())).unwrap_or_default();
            w.open("xr:Link");
            w.leaf("xr:Name", l[i].str().unwrap_or(""));
            w.line(&format!("<xr:DataPath xsi:type=\"xs:string\">{}</xr:DataPath>", esc(&path)));
            w.leaf("xr:ValueChange", match num_at(n, i + 2 + k) {
                0 => "Clear",
                1 => "DontChange",
                _ => "\u{1}",
            });
            w.close("xr:Link");
            i += 3 + k;
        }
        w.close("ChoiceParameterLinks");
        p.push("ChoiceParameterLinks", w);
    }

    /// Исключенные команды поля `{N,guid...}` (у табличного и форматированного документа).
    fn command_set_item(&self, p: &mut P, ex: &Node, skip: impl Fn(&str) -> bool) {
        self.command_set_item_by(p, ex, |_| false, skip)
    }

    /// То же, `drop` — GUID команд, которых у элемента нет.
    fn command_set_item_by(&self, p: &mut P, ex: &Node, drop: impl Fn(&str) -> bool, skip: impl Fn(&str) -> bool) {
        let cnt = num_at(ex, 0).max(0) as usize;
        if cnt == 0 {
            return;
        }
        let guids = ex.list().iter().skip(1).take(cnt).filter_map(Node::atom).filter(|g| !drop(g));
        let mut names: Vec<String> = grouped(guids, |g| Some(std_item_command(g).map_or_else(|| format!("\u{1}{g}"), str::to_string)))
            .into_iter()
            .filter(|n| !skip(n))
            .collect();
        if names.is_empty() {
            return;
        }
        names.sort();
        let mut w = p.w();
        w.open("CommandSet");
        for n in names {
            w.leaf("ExcludedCommand", &n);
        }
        w.close("CommandSet");
        p.push("CommandSet", w);
    }
}

/// Сочетание клавиш `{0,код,модификаторы}`: 8 Ctrl, 16 Alt, 4 Shift.
fn shortcut(n: &Node) -> Option<String> {
    let key = num_at(n, 1);
    if key == 0 {
        return None;
    }
    let m = num_at(n, 2);
    let mut s = String::new();
    for (bit, name) in [(8, "Ctrl+"), (16, "Alt+"), (4, "Shift+")] {
        if m & bit != 0 {
            s += name;
        }
    }
    s += &match key {
        48..=57 | 65..=90 => (key as u8 as char).to_string(),
        112..=123 => format!("F{}", key - 111),
        27 => "Esc".into(),
        32 => "Space".into(),
        13 => "Enter".into(),
        46 => "Del".into(),
        45 => "Ins".into(),
        // ERP 2.5.22: Backspace и цифровой блок
        8 => "BackSpace".into(),
        96..=105 => format!("Num {}", key - 96),
        107 => "Num +".into(),
        109 => "Num -".into(),
        110 => "Num .".into(),
        _ => format!("\u{1}{key}"),
    };
    Some(s)
}

const INPUT_FIELD_ORDER: &[&str] = &[
    "DataPath", "Visible", "UserVisible", "DefaultItem", "Enabled", "ReadOnly", "SkipOnInput", "Title", "TitleTextColor", "TitleBackColor",
    "TitleFont", "TitleLocation", "TitleHeight", "ToolTip", "ToolTipRepresentation", "WarningOnEditRepresentation", "WarningOnEdit", "Shortcut",
    "CommandSet", "HorizontalAlign", "VerticalAlign", "GroupHorizontalAlign", "GroupVerticalAlign", "EditMode", "FixingInTable", "CellHyperlink",
    "AutoCellHeight", "ShowInHeader", "HeaderPicture", "HeaderHorizontalAlign", "ShowInFooter", "FooterDataPath", "FooterText", "FooterTextColor",
    "FooterBackColor", "FooterFont", "FooterPicture", "FooterHorizontalAlign", "OnMainServerUnavalableBehavior", "ShowInCard", "FixInCard",
    "MarkRequiredComplete", "CellMark", "AutoEditMode", "AppearanceInCard", "ShowTitleInCard", "WidthInCard", "AutoWidthInTable",
    "CellHyperlinkRepresentation", "CellHyperlinkDisplayVariant", "Width", "AutoMaxWidth", "MaxWidth", "MinWidth", "Height", "AutoMaxHeight",
    "MaxHeight", "HorizontalStretch", "VerticalStretch", "Wrap", "PasswordMode", "MultiLine", "ExtendedEdit", "MarkNegatives", "DropListButton",
    "ChoiceButton", "ChoiceButtonRepresentation", "ClearButton", "SpinButton", "OpenButton", "CreateButton", "Mask", "ListChoiceMode",
    "MultipleValuesFont", "MultipleValuesTextColor", "MultipleValuesBackColor", "AllowInputEmptyMultipleValues", "AllowMultipleValuesDuplicates",
    "ExtendedEditMultipleValues", "MultipleValuePictureSize", "MultipleValuePictureShape", "ShowCheckBoxesInDropList", "MultipleValuesHyperlink",
    "MultipleValuesPicture", "MultipleValueDataPath", "MultipleValuePictureDataPath", "MultipleValuePresentDataPath", "AutoChoiceIncomplete",
    "QuickChoice", "ChoiceFoldersAndItems", "Format", "EditFormat", "AutoMarkIncomplete", "ChooseType", "IncompleteChoiceMode", "TypeDomainEnabled",
    "TextEdit", "EditTextUpdate", "MinValue", "MaxValue", "ChoiceButtonPicture", "ChoiceForm", "ChoiceParameterLinks", "ChoiceParameters",
    "AvailableTypes", "ChoiceList", "ChoiceListButton", "ChoiceListHeight", "DropListWidth", "TextColor", "BackColor", "BorderColor", "Font",
    "TypeLink", "HeightControlVariant", "AutoShowClearButtonMode", "AutoShowOpenButtonMode", "AutoCorrectionOnTextInput", "SpellCheckingOnTextInput",
    "AutoCapitalizationOnTextInput", "SpecialTextInputMode", "AutofillHint", "OnScreenKeyboardReturnKeyText", "InputHint", "ChoiceHistoryOnInput",
    "DropListHint", "Picture", "ChoiceButtonTitle", "TimeChoiceMode", "TextSize", "ContextMenu", "AutoCommandBar", "ExtendedTooltip",
    "SearchStringAddition", "ViewStatusAddition", "SearchControlAddition", "Events", "ChildItems",
];

const TEXT_DOCUMENT_FIELD_ORDER: &[&str] = &[
    "DataPath", "Visible", "UserVisible", "DefaultItem", "Enabled", "ReadOnly", "SkipOnInput", "Title", "TitleTextColor", "TitleBackColor",
    "TitleFont", "TitleLocation", "TitleHeight", "ToolTip", "ToolTipRepresentation", "WarningOnEditRepresentation", "WarningOnEdit", "Shortcut",
    "CommandSet", "HorizontalAlign", "VerticalAlign", "GroupHorizontalAlign", "GroupVerticalAlign", "EditMode", "FixingInTable", "CellHyperlink",
    "AutoCellHeight", "ShowInHeader", "HeaderPicture", "HeaderHorizontalAlign", "ShowInFooter", "FooterDataPath", "FooterText", "FooterTextColor",
    "FooterBackColor", "FooterFont", "FooterPicture", "FooterHorizontalAlign", "OnMainServerUnavalableBehavior", "ShowInCard", "FixInCard",
    "MarkRequiredComplete", "CellMark", "AutoEditMode", "AppearanceInCard", "ShowTitleInCard", "WidthInCard", "AutoWidthInTable",
    "CellHyperlinkRepresentation", "CellHyperlinkDisplayVariant", "Width", "AutoMaxWidth", "MaxWidth", "MinWidth", "Height", "AutoMaxHeight",
    "MaxHeight", "HorizontalStretch", "VerticalStretch", "Output", "TextColor", "BackColor", "BorderColor", "Font", "ContextMenu", "AutoCommandBar",
    "ExtendedTooltip", "SearchStringAddition", "ViewStatusAddition", "SearchControlAddition", "Events", "ChildItems",
];

const FORMATTED_DOCUMENT_FIELD_ORDER: &[&str] = &[
    "DataPath", "Visible", "UserVisible", "DefaultItem", "Enabled", "ReadOnly", "SkipOnInput", "Title", "TitleTextColor", "TitleBackColor",
    "TitleFont", "TitleLocation", "TitleHeight", "ToolTip", "ToolTipRepresentation", "WarningOnEditRepresentation", "WarningOnEdit", "Shortcut",
    "CommandSet", "HorizontalAlign", "VerticalAlign", "GroupHorizontalAlign", "GroupVerticalAlign", "EditMode", "FixingInTable", "CellHyperlink",
    "AutoCellHeight", "ShowInHeader", "HeaderPicture", "HeaderHorizontalAlign", "ShowInFooter", "FooterDataPath", "FooterText", "FooterTextColor",
    "FooterBackColor", "FooterFont", "FooterPicture", "FooterHorizontalAlign", "OnMainServerUnavalableBehavior", "ShowInCard", "FixInCard",
    "MarkRequiredComplete", "CellMark", "AutoEditMode", "AppearanceInCard", "ShowTitleInCard", "WidthInCard", "AutoWidthInTable",
    "CellHyperlinkRepresentation", "CellHyperlinkDisplayVariant", "Width", "AutoMaxWidth", "MaxWidth", "MinWidth", "Height", "AutoMaxHeight",
    "MaxHeight", "HorizontalStretch", "VerticalStretch", "Output", "TextColor", "BackColor", "BorderColor", "Font", "ContextMenu", "AutoCommandBar",
    "ExtendedTooltip", "SearchStringAddition", "ViewStatusAddition", "SearchControlAddition", "Events", "ChildItems",
];

const PDF_DOCUMENT_FIELD_ORDER: &[&str] = &[
    "DataPath", "Visible", "UserVisible", "DefaultItem", "Enabled", "ReadOnly", "SkipOnInput", "Title", "TitleTextColor", "TitleBackColor",
    "TitleFont", "TitleLocation", "TitleHeight", "ToolTip", "ToolTipRepresentation", "WarningOnEditRepresentation", "WarningOnEdit", "Shortcut",
    "CommandSet", "HorizontalAlign", "VerticalAlign", "GroupHorizontalAlign", "GroupVerticalAlign", "EditMode", "FixingInTable", "CellHyperlink",
    "AutoCellHeight", "ShowInHeader", "HeaderPicture", "HeaderHorizontalAlign", "ShowInFooter", "FooterDataPath", "FooterText", "FooterTextColor",
    "FooterBackColor", "FooterFont", "FooterPicture", "FooterHorizontalAlign", "OnMainServerUnavalableBehavior", "ShowInCard", "FixInCard",
    "MarkRequiredComplete", "CellMark", "AutoEditMode", "AppearanceInCard", "ShowTitleInCard", "WidthInCard", "AutoWidthInTable",
    "CellHyperlinkRepresentation", "CellHyperlinkDisplayVariant", "Width", "AutoMaxWidth", "MaxWidth", "MinWidth", "Height", "AutoMaxHeight",
    "MaxHeight", "HorizontalStretch", "VerticalStretch", "Output", "BorderColor", "Scale", "CurrentPageNumber", "Orientation", "ViewStatusLocation",
    "ContextMenu", "AutoCommandBar", "ExtendedTooltip", "SearchStringAddition", "ViewStatusAddition", "SearchControlAddition", "Events", "ChildItems",
];

const PROGRESS_BAR_FIELD_ORDER: &[&str] = &[
    "DataPath", "Visible", "UserVisible", "DefaultItem", "Enabled", "ReadOnly", "SkipOnInput", "Title", "TitleTextColor", "TitleBackColor",
    "TitleFont", "TitleLocation", "TitleHeight", "ToolTip", "ToolTipRepresentation", "WarningOnEditRepresentation", "WarningOnEdit", "Shortcut",
    "CommandSet", "HorizontalAlign", "VerticalAlign", "GroupHorizontalAlign", "GroupVerticalAlign", "EditMode", "FixingInTable", "CellHyperlink",
    "AutoCellHeight", "ShowInHeader", "HeaderPicture", "HeaderHorizontalAlign", "ShowInFooter", "FooterDataPath", "FooterText", "FooterTextColor",
    "FooterBackColor", "FooterFont", "FooterPicture", "FooterHorizontalAlign", "OnMainServerUnavalableBehavior", "ShowInCard", "FixInCard",
    "MarkRequiredComplete", "CellMark", "AutoEditMode", "AppearanceInCard", "ShowTitleInCard", "WidthInCard", "AutoWidthInTable",
    "CellHyperlinkRepresentation", "CellHyperlinkDisplayVariant", "Width", "AutoMaxWidth", "MaxWidth", "MinWidth", "Height", "AutoMaxHeight",
    "MaxHeight", "HorizontalStretch", "VerticalStretch", "MinValue", "MaxValue", "Orientation", "Representation", "ShowPercent", "BorderColor",
    "ContextMenu", "AutoCommandBar", "ExtendedTooltip", "SearchStringAddition", "ViewStatusAddition", "SearchControlAddition", "Events", "ChildItems",
];

const TRACK_BAR_FIELD_ORDER: &[&str] = &[
    "DataPath", "Visible", "UserVisible", "DefaultItem", "Enabled", "ReadOnly", "SkipOnInput", "Title", "TitleTextColor", "TitleBackColor",
    "TitleFont", "TitleLocation", "TitleHeight", "ToolTip", "ToolTipRepresentation", "WarningOnEditRepresentation", "WarningOnEdit", "Shortcut",
    "CommandSet", "HorizontalAlign", "VerticalAlign", "GroupHorizontalAlign", "GroupVerticalAlign", "EditMode", "FixingInTable", "CellHyperlink",
    "AutoCellHeight", "ShowInHeader", "HeaderPicture", "HeaderHorizontalAlign", "ShowInFooter", "FooterDataPath", "FooterText", "FooterTextColor",
    "FooterBackColor", "FooterFont", "FooterPicture", "FooterHorizontalAlign", "OnMainServerUnavalableBehavior", "ShowInCard", "FixInCard",
    "MarkRequiredComplete", "CellMark", "AutoEditMode", "AppearanceInCard", "ShowTitleInCard", "WidthInCard", "AutoWidthInTable",
    "CellHyperlinkRepresentation", "CellHyperlinkDisplayVariant", "Width", "AutoMaxWidth", "MaxWidth", "MinWidth", "Height", "AutoMaxHeight",
    "MaxHeight", "HorizontalStretch", "VerticalStretch", "MinValue", "MaxValue", "Step", "LargeStep", "MarkingStep", "Orientation",
    "MarkingAppearance", "BorderColor", "ContextMenu", "AutoCommandBar", "ExtendedTooltip", "SearchStringAddition", "ViewStatusAddition",
    "SearchControlAddition", "Events", "ChildItems",
];

const CHART_FIELD_ORDER: &[&str] = &[
    "DataPath", "Visible", "UserVisible", "DefaultItem", "Enabled", "ReadOnly", "SkipOnInput", "Title", "TitleTextColor", "TitleBackColor",
    "TitleFont", "TitleLocation", "TitleHeight", "ToolTip", "ToolTipRepresentation", "WarningOnEditRepresentation", "WarningOnEdit", "Shortcut",
    "CommandSet", "HorizontalAlign", "VerticalAlign", "GroupHorizontalAlign", "GroupVerticalAlign", "EditMode", "FixingInTable", "CellHyperlink",
    "AutoCellHeight", "ShowInHeader", "HeaderPicture", "HeaderHorizontalAlign", "ShowInFooter", "FooterDataPath", "FooterText", "FooterTextColor",
    "FooterBackColor", "FooterFont", "FooterPicture", "FooterHorizontalAlign", "OnMainServerUnavalableBehavior", "ShowInCard", "FixInCard",
    "MarkRequiredComplete", "CellMark", "AutoEditMode", "AppearanceInCard", "ShowTitleInCard", "WidthInCard", "AutoWidthInTable",
    "CellHyperlinkRepresentation", "CellHyperlinkDisplayVariant", "Width", "AutoMaxWidth", "MaxWidth", "MinWidth", "Height", "AutoMaxHeight",
    "MaxHeight", "HorizontalStretch", "VerticalStretch", "ContextMenu", "AutoCommandBar", "ExtendedTooltip", "SearchStringAddition",
    "ViewStatusAddition", "SearchControlAddition", "Events", "ChildItems",
];

/// Как у диаграммы, свои свойства — в порядке модели logform (mngbase_root.res), затем предопределенные дочерние.
const GANTT_FIELD_ORDER: &[&str] = &[
    "DataPath", "Visible", "UserVisible", "DefaultItem", "Enabled", "ReadOnly", "SkipOnInput", "Title", "TitleTextColor", "TitleBackColor",
    "TitleFont", "TitleLocation", "TitleHeight", "ToolTip", "ToolTipRepresentation", "WarningOnEditRepresentation", "WarningOnEdit", "Shortcut",
    "CommandSet", "HorizontalAlign", "VerticalAlign", "GroupHorizontalAlign", "GroupVerticalAlign", "EditMode", "FixingInTable", "CellHyperlink",
    "AutoCellHeight", "ShowInHeader", "HeaderPicture", "HeaderHorizontalAlign", "ShowInFooter", "FooterDataPath", "FooterText", "FooterTextColor",
    "FooterBackColor", "FooterFont", "FooterPicture", "FooterHorizontalAlign", "OnMainServerUnavalableBehavior", "ShowInCard", "FixInCard",
    "MarkRequiredComplete", "CellMark", "AutoEditMode", "AppearanceInCard", "ShowTitleInCard", "WidthInCard", "AutoWidthInTable",
    "CellHyperlinkRepresentation", "CellHyperlinkDisplayVariant", "Width", "AutoMaxWidth", "MaxWidth", "MinWidth", "Height", "AutoMaxHeight",
    "MaxHeight", "HorizontalStretch", "VerticalStretch", "TableLocation", "ValuesSelectionMode", "IntervalsSelectionMode",
    "ShowHorizontalLinesFlag", "ShowVerticalLinesFlag", "ContextMenu", "AutoCommandBar", "ExtendedTooltip", "SearchStringAddition",
    "ViewStatusAddition", "SearchControlAddition", "Table", "Events", "ChildItems",
];

const GRAPHICAL_SCHEMA_FIELD_ORDER: &[&str] = &[
    "DataPath", "Visible", "UserVisible", "DefaultItem", "Enabled", "ReadOnly", "SkipOnInput", "Title", "TitleTextColor", "TitleBackColor",
    "TitleFont", "TitleLocation", "TitleHeight", "ToolTip", "ToolTipRepresentation", "WarningOnEditRepresentation", "WarningOnEdit", "Shortcut",
    "CommandSet", "HorizontalAlign", "VerticalAlign", "GroupHorizontalAlign", "GroupVerticalAlign", "EditMode", "FixingInTable", "CellHyperlink",
    "AutoCellHeight", "ShowInHeader", "HeaderPicture", "HeaderHorizontalAlign", "ShowInFooter", "FooterDataPath", "FooterText", "FooterTextColor",
    "FooterBackColor", "FooterFont", "FooterPicture", "FooterHorizontalAlign", "OnMainServerUnavalableBehavior", "ShowInCard", "FixInCard",
    "MarkRequiredComplete", "CellMark", "AutoEditMode", "AppearanceInCard", "ShowTitleInCard", "WidthInCard", "AutoWidthInTable",
    "CellHyperlinkRepresentation", "CellHyperlinkDisplayVariant", "Width", "AutoMaxWidth", "MaxWidth", "MinWidth", "Height", "AutoMaxHeight",
    "MaxHeight", "HorizontalStretch", "VerticalStretch", "Output", "Edit", "BorderColor", "ContextMenu", "AutoCommandBar", "ExtendedTooltip",
    "SearchStringAddition", "ViewStatusAddition", "SearchControlAddition", "Events", "ChildItems",
];

const CALENDAR_FIELD_ORDER: &[&str] = &[
    "DataPath", "Visible", "UserVisible", "DefaultItem", "Enabled", "ReadOnly", "SkipOnInput", "Title", "TitleTextColor", "TitleBackColor",
    "TitleFont", "TitleLocation", "TitleHeight", "ToolTip", "ToolTipRepresentation", "WarningOnEditRepresentation", "WarningOnEdit", "Shortcut",
    "CommandSet", "HorizontalAlign", "VerticalAlign", "GroupHorizontalAlign", "GroupVerticalAlign", "EditMode", "FixingInTable", "CellHyperlink",
    "AutoCellHeight", "ShowInHeader", "HeaderPicture", "HeaderHorizontalAlign", "ShowInFooter", "FooterDataPath", "FooterText", "FooterTextColor",
    "FooterBackColor", "FooterFont", "FooterPicture", "FooterHorizontalAlign", "OnMainServerUnavalableBehavior", "ShowInCard", "FixInCard",
    "MarkRequiredComplete", "CellMark", "AutoEditMode", "AppearanceInCard", "ShowTitleInCard", "WidthInCard", "AutoWidthInTable",
    "CellHyperlinkRepresentation", "CellHyperlinkDisplayVariant", "Width", "AutoMaxWidth", "MaxWidth", "MinWidth", "Height", "AutoMaxHeight",
    "MaxHeight", "HorizontalStretch", "VerticalStretch", "SelectionMode", "ShowCurrentDate", "CalendarNavigation", "BeginOfRepresentationPeriod",
    "EndOfRepresentationPeriod", "EnableStartDrag", "EnableDrag", "Font", "BorderColor", "Border", "ShowMonthsPanel", "WidthInMonths",
    "HeightInMonths", "ContextMenu", "AutoCommandBar", "ExtendedTooltip", "SearchStringAddition", "ViewStatusAddition", "SearchControlAddition",
    "Events", "ChildItems",
];

const PERIOD_FIELD_ORDER: &[&str] = &[
    "DataPath", "Visible", "UserVisible", "DefaultItem", "Enabled", "ReadOnly", "SkipOnInput", "Title", "TitleTextColor", "TitleBackColor",
    "TitleFont", "TitleLocation", "TitleHeight", "ToolTip", "ToolTipRepresentation", "WarningOnEditRepresentation", "WarningOnEdit", "Shortcut",
    "CommandSet", "HorizontalAlign", "VerticalAlign", "GroupHorizontalAlign", "GroupVerticalAlign", "EditMode", "FixingInTable", "CellHyperlink",
    "AutoCellHeight", "ShowInHeader", "HeaderPicture", "HeaderHorizontalAlign", "ShowInFooter", "FooterDataPath", "FooterText", "FooterTextColor",
    "FooterBackColor", "FooterFont", "FooterPicture", "FooterHorizontalAlign", "OnMainServerUnavalableBehavior", "ShowInCard", "FixInCard",
    "MarkRequiredComplete", "CellMark", "AutoEditMode", "AppearanceInCard", "ShowTitleInCard", "WidthInCard", "AutoWidthInTable",
    "CellHyperlinkRepresentation", "CellHyperlinkDisplayVariant", "Width", "AutoMaxWidth", "MaxWidth", "MinWidth", "Height", "AutoMaxHeight",
    "MaxHeight", "HorizontalStretch", "VerticalStretch", "Font", "BorderColor", "Border", "ContextMenu", "AutoCommandBar", "ExtendedTooltip",
    "SearchStringAddition", "ViewStatusAddition", "SearchControlAddition", "Events", "ChildItems",
];

const PLANNER_FIELD_ORDER: &[&str] = &[
    "DataPath", "Visible", "UserVisible", "DefaultItem", "Enabled", "ReadOnly", "SkipOnInput", "Title", "TitleTextColor", "TitleBackColor",
    "TitleFont", "TitleLocation", "TitleHeight", "ToolTip", "ToolTipRepresentation", "WarningOnEditRepresentation", "WarningOnEdit", "Shortcut",
    "CommandSet", "HorizontalAlign", "VerticalAlign", "GroupHorizontalAlign", "GroupVerticalAlign", "EditMode", "FixingInTable", "CellHyperlink",
    "AutoCellHeight", "ShowInHeader", "HeaderPicture", "HeaderHorizontalAlign", "ShowInFooter", "FooterDataPath", "FooterText", "FooterTextColor",
    "FooterBackColor", "FooterFont", "FooterPicture", "FooterHorizontalAlign", "OnMainServerUnavalableBehavior", "ShowInCard", "FixInCard",
    "MarkRequiredComplete", "CellMark", "AutoEditMode", "AppearanceInCard", "ShowTitleInCard", "WidthInCard", "AutoWidthInTable",
    "CellHyperlinkRepresentation", "CellHyperlinkDisplayVariant", "Width", "AutoMaxWidth", "MaxWidth", "MinWidth", "Height", "AutoMaxHeight",
    "MaxHeight", "HorizontalStretch", "VerticalStretch", "EnableStartDrag", "EnableDrag", "TimeScaleItemHyperlink", "DimensionItemHyperlink",
    "WrappedTimeScaleHeaderHyperlink", "ContextMenu", "AutoCommandBar", "ExtendedTooltip", "SearchStringAddition", "ViewStatusAddition",
    "SearchControlAddition", "Events", "ChildItems",
];

const HTML_DOCUMENT_FIELD_ORDER: &[&str] = &[
    "DataPath", "Visible", "UserVisible", "DefaultItem", "Enabled", "ReadOnly", "SkipOnInput", "Title", "TitleTextColor", "TitleBackColor",
    "TitleFont", "TitleLocation", "TitleHeight", "ToolTip", "ToolTipRepresentation", "WarningOnEditRepresentation", "WarningOnEdit", "Shortcut",
    "CommandSet", "HorizontalAlign", "VerticalAlign", "GroupHorizontalAlign", "GroupVerticalAlign", "EditMode", "FixingInTable", "CellHyperlink",
    "AutoCellHeight", "ShowInHeader", "HeaderPicture", "HeaderHorizontalAlign", "ShowInFooter", "FooterDataPath", "FooterText", "FooterTextColor",
    "FooterBackColor", "FooterFont", "FooterPicture", "FooterHorizontalAlign", "OnMainServerUnavalableBehavior", "ShowInCard", "FixInCard",
    "MarkRequiredComplete", "CellMark", "AutoEditMode", "AppearanceInCard", "ShowTitleInCard", "WidthInCard", "AutoWidthInTable",
    "CellHyperlinkRepresentation", "CellHyperlinkDisplayVariant", "Width", "AutoMaxWidth", "MaxWidth", "MinWidth", "Height", "AutoMaxHeight",
    "MaxHeight", "HorizontalStretch", "VerticalStretch", "Output", "BorderColor", "Border", "ContextMenu", "AutoCommandBar", "ExtendedTooltip",
    "SearchStringAddition", "ViewStatusAddition", "SearchControlAddition", "Events", "ChildItems",
];

const LABEL_FIELD_ORDER: &[&str] = &[
    "DataPath", "Visible", "UserVisible", "DefaultItem", "Enabled", "ReadOnly", "SkipOnInput", "Title", "TitleTextColor", "TitleBackColor",
    "TitleFont", "TitleLocation", "TitleHeight", "ToolTip", "ToolTipRepresentation", "WarningOnEditRepresentation", "WarningOnEdit", "Shortcut",
    "CommandSet", "HorizontalAlign", "VerticalAlign", "GroupHorizontalAlign", "GroupVerticalAlign", "EditMode", "FixingInTable", "CellHyperlink",
    "AutoCellHeight", "ShowInHeader", "HeaderPicture", "HeaderHorizontalAlign", "ShowInFooter", "FooterDataPath", "FooterText", "FooterTextColor",
    "FooterBackColor", "FooterFont", "FooterPicture", "FooterHorizontalAlign", "OnMainServerUnavalableBehavior", "ShowInCard", "FixInCard",
    "MarkRequiredComplete", "CellMark", "AutoEditMode", "AppearanceInCard", "ShowTitleInCard", "WidthInCard", "AutoWidthInTable",
    "CellHyperlinkRepresentation", "CellHyperlinkDisplayVariant", "Width", "AutoMaxWidth", "MaxWidth", "MinWidth", "Height", "AutoMaxHeight",
    "MaxHeight", "HorizontalStretch", "VerticalStretch", "MarkNegatives", "Format", "Hiperlink", "PasswordMode", "Border", "BorderColor", "TextColor",
    "BackColor", "Font", "UseCopy", "ContextMenu", "AutoCommandBar", "ExtendedTooltip", "SearchStringAddition", "ViewStatusAddition",
    "SearchControlAddition", "Events", "ChildItems",
];

const CHECKBOX_FIELD_ORDER: &[&str] = &[
    "DataPath", "Visible", "UserVisible", "DefaultItem", "Enabled", "ReadOnly", "SkipOnInput", "Title", "TitleTextColor", "TitleBackColor",
    "TitleFont", "TitleLocation", "TitleHeight", "ToolTip", "ToolTipRepresentation", "WarningOnEditRepresentation", "WarningOnEdit", "Shortcut",
    "CommandSet", "HorizontalAlign", "VerticalAlign", "GroupHorizontalAlign", "GroupVerticalAlign", "EditMode", "FixingInTable", "CellHyperlink",
    "AutoCellHeight", "ShowInHeader", "HeaderPicture", "HeaderHorizontalAlign", "ShowInFooter", "FooterDataPath", "FooterText", "FooterTextColor",
    "FooterBackColor", "FooterFont", "FooterPicture", "FooterHorizontalAlign", "OnMainServerUnavalableBehavior", "ShowInCard", "FixInCard",
    "MarkRequiredComplete", "CellMark", "AutoEditMode", "AppearanceInCard", "ShowTitleInCard", "WidthInCard", "AutoWidthInTable",
    "CellHyperlinkRepresentation", "CellHyperlinkDisplayVariant", "CheckBoxType", "ThreeState", "BorderColor", "BackColor", "TextColor", "Font",
    "TumblerRepresentation", "HorizontalStretch", "EditFormat", "ItemTitleHeight", "ItemWidth", "ItemHeight", "EqualItemsWidth", "ContextMenu",
    "AutoCommandBar", "ExtendedTooltip", "SearchStringAddition", "ViewStatusAddition", "SearchControlAddition", "Events", "ChildItems",
];

const RADIO_FIELD_ORDER: &[&str] = &[
    "DataPath", "Visible", "UserVisible", "DefaultItem", "Enabled", "ReadOnly", "SkipOnInput", "Title", "TitleTextColor", "TitleBackColor",
    "TitleFont", "TitleLocation", "TitleHeight", "ToolTip", "ToolTipRepresentation", "WarningOnEditRepresentation", "WarningOnEdit", "Shortcut",
    "CommandSet", "HorizontalAlign", "VerticalAlign", "GroupHorizontalAlign", "GroupVerticalAlign", "EditMode", "FixingInTable", "CellHyperlink",
    "AutoCellHeight", "ShowInHeader", "HeaderPicture", "HeaderHorizontalAlign", "ShowInFooter", "FooterDataPath", "FooterText", "FooterTextColor",
    "FooterBackColor", "FooterFont", "FooterPicture", "FooterHorizontalAlign", "OnMainServerUnavalableBehavior", "ShowInCard", "FixInCard",
    "MarkRequiredComplete", "CellMark", "AutoEditMode", "AppearanceInCard", "ShowTitleInCard", "WidthInCard", "AutoWidthInTable",
    "CellHyperlinkRepresentation", "CellHyperlinkDisplayVariant", "RadioButtonType", "TumblerRepresentation", "Orientation", "ItemWidth",
    "ItemHeight", "ItemTitleHeight", "ColumnsCount", "EqualColumnsWidth", "HorizontalStretch", "ChoiceList", "Font", "TextColor", "BackColor",
    "BorderColor", "ContextMenu", "AutoCommandBar", "ExtendedTooltip", "SearchStringAddition", "ViewStatusAddition", "SearchControlAddition",
    "Events", "ChildItems",
];

const PICTURE_FIELD_ORDER: &[&str] = &[
    "DataPath", "Visible", "UserVisible", "DefaultItem", "Enabled", "ReadOnly", "SkipOnInput", "Title", "TitleTextColor", "TitleBackColor",
    "TitleFont", "TitleLocation", "TitleHeight", "ToolTip", "ToolTipRepresentation", "WarningOnEditRepresentation", "WarningOnEdit", "Shortcut",
    "CommandSet", "HorizontalAlign", "VerticalAlign", "GroupHorizontalAlign", "GroupVerticalAlign", "EditMode", "FixingInTable", "CellHyperlink",
    "AutoCellHeight", "ShowInHeader", "HeaderPicture", "HeaderHorizontalAlign", "ShowInFooter", "FooterDataPath", "FooterText", "FooterTextColor",
    "FooterBackColor", "FooterFont", "FooterPicture", "FooterHorizontalAlign", "OnMainServerUnavalableBehavior", "ShowInCard", "FixInCard",
    "MarkRequiredComplete", "CellMark", "AutoEditMode", "AppearanceInCard", "ShowTitleInCard", "WidthInCard", "AutoWidthInTable",
    "CellHyperlinkRepresentation", "CellHyperlinkDisplayVariant", "Width", "AutoMaxWidth", "MaxWidth", "MinWidth", "Height", "AutoMaxHeight",
    "MaxHeight", "HorizontalStretch", "VerticalStretch", "PictureSize", "Zoomable", "ImageScale", "Hyperlink", "NonselectedPictureText",
    "EnableStartDrag", "EnableDrag", "ValuesPicture", "TextColor", "BorderColor", "Font", "Border", "FileDragMode", "BackgroundShowMode",
    "PictureColor", "ContextMenu", "AutoCommandBar", "ExtendedTooltip", "SearchStringAddition", "ViewStatusAddition", "SearchControlAddition",
    "Events", "ChildItems",
];

const SPREADSHEET_FIELD_ORDER: &[&str] = &[
    "DataPath", "Visible", "UserVisible", "DefaultItem", "Enabled", "ReadOnly", "SkipOnInput", "Title", "TitleTextColor", "TitleBackColor",
    "TitleFont", "TitleLocation", "TitleHeight", "ToolTip", "ToolTipRepresentation", "WarningOnEditRepresentation", "WarningOnEdit", "Shortcut",
    "CommandSet", "HorizontalAlign", "VerticalAlign", "GroupHorizontalAlign", "GroupVerticalAlign", "EditMode", "FixingInTable", "CellHyperlink",
    "AutoCellHeight", "ShowInHeader", "HeaderPicture", "HeaderHorizontalAlign", "ShowInFooter", "FooterDataPath", "FooterText", "FooterTextColor",
    "FooterBackColor", "FooterFont", "FooterPicture", "FooterHorizontalAlign", "OnMainServerUnavalableBehavior", "ShowInCard", "FixInCard",
    "MarkRequiredComplete", "CellMark", "AutoEditMode", "AppearanceInCard", "ShowTitleInCard", "WidthInCard", "AutoWidthInTable",
    "CellHyperlinkRepresentation", "CellHyperlinkDisplayVariant", "Width", "AutoMaxWidth", "MaxWidth", "MinWidth", "Height", "AutoMaxHeight",
    "MaxHeight", "HorizontalStretch", "VerticalStretch", "ShowGrid", "ShowHeaders", "VerticalScrollBar", "HorizontalScrollBar", "BlackAndWhiteView",
    "Protection", "SelectionShowMode", "DrawingSelectionShowMode", "PointerType", "Output", "Edit", "ShowGroups", "EnableStartDrag", "EnableDrag",
    "BorderColor", "ShowCellNames", "ShowRowAndColumnNames", "ViewScalingMode", "CellActionsButtonViewMode",
    "SpreadsheetDocumentMultipleSelectionPanelViewMode", "ContextMenu", "AutoCommandBar", "ExtendedTooltip", "SearchStringAddition",
    "ViewStatusAddition", "SearchControlAddition", "Events", "ChildItems",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated() {
        // Таблица «Список» с полем «Период»: сначала панель поля, потом порожденные элементы таблицы.
        let t = format!("{{0,1,{TABLE},{{53,{{1,{ELEM}}},0,\"Список\",1,{FIELD},{{34,{{4,{ELEM}}},0,\"Период\",{{8,{{5,{ELEM}}},0,\"ПериодКонтекстноеМеню\"}}}}}}}}");
        let n = brace::Doc::parse(&t).unwrap().roots.remove(0);
        let mut g = Vec::new();
        generated_items(&n, None, &mut g);
        assert_eq!(g.len(), 9);
        assert_eq!(g[0], "ПериодПанельДействийВыделенныхСтрок");
        assert_eq!(g[1], "СписокДействияСтроки");
        assert_eq!(g[8], "СписокПанельДействийВыделенныхСтрокУправлениеПоискомРасширеннаяПодсказка");
    }

    #[test]
    fn empty_form() {
        // Запись формы DataProcessors/ЭлектронноеВзаимодействие/Forms/ПустаяФорма (без модуля и настроек).
        let t = concat!(
            "{4,\r\n{49,0,0,0,0,1,0,0,00000000-0000-0000-0000-000000000000,1,\r\n{1,0},0,0,1,1,1,0,1,0,\r\n",
            "{1,3ccc650e-f631-4cae-8e33-3eaac610b5f9,\"ПриОткрытии\",1,0,3ccc650e-f631-4cae-8e33-3eaac610b5f9,0,1},\r\n{0},1,\r\n",
            "{22,\r\n{-1,02023637-7868-4a5f-8576-835a76e0c9ba},0,0,0,9,\"ФормаКоманднаяПанель\",\r\n{1,0},\r\n{1,0},0,1,0,0,0,2,2,\r\n",
            "{3,4,\r\n{0}\r\n},\r\n{7,3,0,1,100},\r\n{0,0,0},1,\r\n{0,0,1},0,1,0,0,0,3,3,0},0,\"\",\"\",1,\r\n",
            "{22,\r\n{0},0,0,0,7,\"Navigator\"},1,\"\",0,0,0,0,0,0,3,3,0,0,0,100,1,1,0,0,0,\r\n{49,0}\r\n},\"\",\r\n",
            "{4,0,0,0,\r\n{#base64:AA==}\r\n},\r\n{0,0},\r\n{0,0}}"
        );
        let x = String::from_utf8(form_xml(&Names::default(), t).unwrap().unwrap()).unwrap();
        let want = concat!(
            "\t<WindowOpeningMode>DontBlock</WindowOpeningMode>\r\n\t<Group>Vertical</Group>\r\n",
            "\t<AutoCommandBar name=\"ФормаКоманднаяПанель\" id=\"-1\"/>\r\n",
            "\t<Events>\r\n\t\t<Event name=\"OnOpen\">ПриОткрытии</Event>\r\n\t</Events>\r\n\t<Attributes/>\r\n</Form>"
        );
        assert!(x.ends_with(want), "{x}");
    }

    #[test]
    fn fonts_and_colors() {
        let n = |t: &str| brace::parse(t).unwrap();
        let names = Names::default();
        assert_eq!(font(&n("{7,3,0,1,100}"), &names), None);
        assert_eq!(
            font(&n("{7,2,62,{-31},90,700,0,0,0,1,100}"), &names).unwrap(),
            " ref=\"style:NormalTextFont\" height=\"9\" bold=\"true\" italic=\"false\" underline=\"false\" strikeout=\"false\" kind=\"StyleItem\""
        );
        assert_eq!(font(&n("{7,2,514,{-31},10,1,100}"), &names).unwrap(), " ref=\"style:NormalTextFont\" height=\"1\" kind=\"StyleItem\" scale=\"100\"");
        assert_eq!(font(&n("{7,3,4,700,1,100}"), &names).unwrap(), " bold=\"true\" kind=\"AutoFont\"");
        assert_eq!(color(&n("{3,0,{2018554}}"), &names).unwrap(), "#FACC1E");
        assert_eq!(color(&n("{3,3,{-23}}"), &names).unwrap(), "style:ToolTipBackColor");
        assert_eq!(color(&n("{3,4,{0}}"), &names), None);
    }

    #[test]
    fn unresolved_refs() {
        // Значение списка выбора с типом не из конфигурации — `<тип>.<uuid>`; тип из конфигурации без значения — отказ.
        let (t, id) = ("e8d415d9-ae35-4ab3-985a-9a9a075be81f", "386fd4ed-446b-452b-98db-a1b52695e18e");
        let v = brace::parse(&format!("{{\"#\",{FORM_CHOICE_VALUE},{{0,0,{{\"U\"}},{t},{id},{{1,0}}}}}}")).unwrap();
        let x = |names: &Names| {
            let mut w = W { s: String::new(), depth: 0 };
            choice_value(&mut w, "xr:Value", &v, names);
            w.s
        };
        let mut names = Names::default();
        assert!(x(&names).contains(&format!("<Value xsi:type=\"xr:DesignTimeRef\">{t}.{id}</Value>")), "{}", x(&names));
        names.types.insert(t.into(), "EnumRef.Х".into());
        assert!(x(&names).contains("<Value>\u{1}"), "{}", x(&names));
        // Расширенная подсказка поля встроена без вида перед собой — стандартных команд у нее нет; путь к текущим данным
        // таблицы 7 той же формы (без имени) — не элемент.
        let g = "00000000-0000-0000-0000-000000000001";
        let f = brace::parse(&format!("{{0,1,{FIELD},{{34,{{4,{ELEM}}},0,0,0,0,\"Поле\",1,{{12,{{5,{ELEM}}},0,0,0,0,\"Подсказка\"}},{{6,{{7,{ELEM}}},{{0,{g}}},{{0,{g}}},{{0,{g}}},{{0,{g}}},{{0,{g}}}}}}}}}")).unwrap();
        let (mut paths, mut nodes, mut plain) = (HashMap::new(), HashMap::new(), HashSet::new());
        collect_tables(&f, &mut paths, &mut nodes, &mut plain);
        assert!(plain.contains("5") && !plain.contains("4") && !plain.contains("7"), "{plain:?}");
    }

    #[test]
    fn unresolved_owner_command_and_prim_path() {
        let (c, a) = ("00000000-0000-0000-0000-0000000000c1", "00000000-0000-0000-0000-0000000000a1");
        let mut names = Names::default();
        names.full.insert(c.into(), "Catalog.Х".into());
        names.full.insert(a.into(), "Catalog.Х.Attribute.А".into());
        let form = |names: &Names, prim: bool| {
            let mut f = Form { names, w: W { s: String::new(), depth: 0 }, items: HashMap::new(), commands: HashMap::new(), attrs: HashMap::new(), cols: HashMap::new(), lists: HashMap::new(), roots: HashMap::new(), tables: HashMap::new(), excluded: HashSet::new(), scopes: [true; 7], table_nodes: HashMap::new(), plain: HashSet::new(), xcols: HashMap::new(), col_class: HashMap::new(), owners: HashMap::new(), prim: HashSet::new(), extent: HashMap::new(), generated: Vec::new(), zero_items: Vec::new(), v59_flags: HashMap::new() };
            f.attrs.insert("10", ("Флаг".into(), String::new(), String::new()));
            if prim {
                f.prim.insert("10");
            }
            (f.command(&brace::parse(&format!("{{4,{c}}}")).unwrap()), f.data_path(&brace::parse(&format!("{{2,{{10}},{{0,{a}}}}}")).unwrap()))
        };
        // Справочник без владельцев: OpenByValue нет — `4:guid`; путь {0,guid} от примитивного реквизита — сырой.
        assert_eq!(form(&names, true), (format!("4:{c}"), Some(format!("10/0:{a}"))));
        names.owned.insert(c.into());
        assert_eq!(form(&names, false), ("Catalog.Х.StandardCommand.OpenByValue".into(), Some("Флаг.А".into())));
    }

    #[test]
    fn dl_delete_by_class() {
        let (dl, coll) = ("ec576e13-1e76-4c33-98aa-a33204514227", "8d772f97-c0ef-47c0-9cb0-efea28c61341");
        assert!(dl_lacks(dl, "InformationRegister", false) && !dl_lacks(coll, "InformationRegister", false));
        assert!(!dl_lacks(dl, "Catalog", false) && dl_lacks(coll, "Catalog", false) && dl_lacks(coll, "", false));
        let all = "51c99108-107c-43e1-8918-e48835bf2495";
        assert!(!dl_lacks(all, "Catalog", false) && dl_lacks(all, "Catalog", true));
    }

    #[test]
    fn query_field_names() {
        let q = "ВЫБРАТЬ РАЗРЕШЕННЫЕ\r\n\tТ.Ссылка,\r\n\tВЫБОР КОГДА Т.А = \"x,y\" ТОГДА 1 КОНЕЦ КАК Флаг, // КАК Нет\r\n\tЕСТЬNULL(Т.Сумма, 0) КАК Сумма\r\nИЗ Справочник.Х КАК Т\r\nОБЪЕДИНИТЬ ВЫБРАТЬ 1, 2, 3";
        let f = query_fields(q);
        assert_eq!(f, ["ссылка", "флаг", "сумма"].into_iter().map(String::from).collect());
        // Пакет: временная таблица пропускается.
        assert!(query_fields("ВЫБРАТЬ А ПОМЕСТИТЬ ВТ ИЗ Т;\r\nВЫБРАТЬ ВТ.Б КАК В ИЗ ВТ").contains("в"));
        // Путь без псевдонима из трех частей — склейка частей после первой.
        let f = query_fields("ВЫБРАТЬ\r\n\tТ.Запись.Сервис,\r\n\tТ.Код\r\nИЗ Справочник.Х КАК Т");
        assert!(f.contains("записьсервис") && f.contains("код"), "{f:?}");
        // `{ВЫБРАТЬ ...}` компоновки — за выборкой.
        assert!(query_fields("ВЫБРАТЬ\r\n\tТ.А,\r\n\tТ.Способ\r\n{ВЫБРАТЬ\r\n\tБ}\r\nИЗ Т").contains("способ"));
        // Первая таблица источника — вне скобок подзапросов выборки.
        assert_eq!(first_source("ВЫБРАТЬ (ВЫБРАТЬ 1 ИЗ Документ.Д) КАК А ИЗ РегистрСведений.Р КАК Р"), Some("InformationRegister"));
        assert_eq!(first_source("ВЫБРАТЬ Т.А ИЗ РегистрНакопления.Р КАК Т"), Some(""));
    }

    #[test]
    fn broken_query() {
        let mut n = Names::default();
        n.objects.insert("catalog.х".into(), "g1".into());
        n.objects.insert("enum.в".into(), "g2".into());
        n.fields.insert("g2".into(), ["да".to_string()].into_iter().collect());
        n.fields.insert("g1".into(), ["сумма".to_string(), "в".to_string()].into_iter().collect());
        assert!(!n.query_broken("ВЫБРАТЬ Т.Ссылка ИЗ Справочник.Х КАК Т ГДЕ Т.В = ЗНАЧЕНИЕ(Перечисление.В.Да) // Справочник.Нет"));
        assert!(n.query_broken("ВЫБРАТЬ Т.Ссылка ИЗ Справочник.Х КАК Т ЛЕВОЕ СОЕДИНЕНИЕ РегистрСведений.Нет КАК Н ПО ИСТИНА"));
        assert!(n.query_broken("ВЫБРАТЬ ЗНАЧЕНИЕ(Перечисление.В.Нет) КАК А"));
        // Псевдоним, совпадающий с именем класса, — не таблица.
        assert!(!n.query_broken("ВЫБРАТЬ РегистрСведений.Сумма ИЗ Справочник.Х КАК РегистрСведений"));
        // Поле по псевдониму: у таблицы нет — недоступно; стандартное, виртуальной таблицы или подзапроса — нет.
        assert!(n.query_broken("ВЫБРАТЬ Т.Нет ИЗ Справочник.Х КАК Т"));
        assert!(!n.query_broken("ВЫБРАТЬ Т.Ссылка, Т.Сумма.Поле ИЗ Справочник.Х КАК Т"));
        assert!(!n.query_broken("ВЫБРАТЬ Т.СуммаОстаток, Т.СубконтоДт2 ИЗ Справочник.Х.Остатки(&Д, Сумма > 0) КАК Т"));
        assert!(!n.query_broken("ВЫБРАТЬ Т.Нет ИЗ Справочник.Х КАК Т ОБЪЕДИНИТЬ ВЫБРАТЬ Т.Нет ИЗ (ВЫБРАТЬ 1 КАК Нет) КАК Т"));
        // `Т.*` — все поля таблицы; источник с псевдонимом-классом (Алко*: `Справочник.Х КАК Справочник`).
        let q = "ВЫБРАТЬ Справочник.*, 1 КАК К ИЗ Справочник.Х КАК Справочник ГДЕ Справочник.В = 1";
        assert!(!n.query_broken(q) && n.star_field(q, "сумма") && !n.star_field(q, "нет"));
        // Подполе у поля только примитивных типов (в том числе через определяемый тип) — поля нет.
        n.prim_defined.insert("d1".into());
        n.field_refs.insert("g1".into(), [("сумма".to_string(), vec![]), ("в".to_string(), vec!["d1".to_string()])].into_iter().collect());
        assert!(n.query_broken("ВЫБРАТЬ Т.Сумма.Поле ИЗ Справочник.Х КАК Т"));
        assert!(n.query_broken("ВЫБРАТЬ Т.В ИЗ Справочник.Х КАК Т ГДЕ НЕ Т.В.Ссылка ЕСТЬ NULL"));
        assert!(!n.query_broken("ВЫБРАТЬ Т.Сумма ИЗ Справочник.Х КАК Т"));
    }

    #[test]
    fn list_extension_names() {
        let q = "ВЫБРАТЬ РАЗРЕШЕННЫЕ Т.Ссылка, Т.Проведен, Т.Сумма КАК Итог, Т.А.Б, ВЫБОР КОГДА Т.Х ТОГДА 1 КОНЕЦ КАК В // Т.Нет\nИЗ Документ.Д КАК Т";
        assert_eq!(query_items(q).unwrap(), ["Ref", "Posted", "Итог", "АБ", "В"]);
        assert!(query_items("ВЫБРАТЬ Т.* ИЗ Документ.Д КАК Т").is_none());
        assert!(query_items("ВЫБРАТЬ 1 + 2 ИЗ Документ.Д КАК Т").is_none());
        let st = brace::parse(r#"{0,3,"QueryText",{"S","ВЫБРАТЬ Т.Б КАК Бета, Т.А КАК альфа, Т.Ссылка ИЗ Т"},"ManualQuery",{"B",1},"FieldsMapItemId0",{"N",7},"FieldsMapItemName0",{"S","Ref"}}"#).unwrap();
        // Бета и альфа: место зависит от регистра — не разрешается.
        assert_eq!(list_extension(&Names::default(), &st), Some((7, vec![None, None])));
    }

    #[test]
    fn unavailable_by_root() {
        // Корень модели загрузки: путь вне корня — `~`, второй путь — только у записи FieldsMap со вторым именем.
        let st = brace::parse(r#"{0,6,"QueryText",{"S","ВЫБРАТЬ Т.Ссылка КАК Ссылка, Т.Владелец КАК Владелец ИЗ Справочник.Х КАК Т"},"ManualQuery",{"B",1},"FieldsMapItemId0",{"N",1},"FieldsMapItemName0",{"S","Ref"},"FieldsMapItemSecondaryName0",{"S","Ссылка"},"FieldsMapItemId1",{"N",2},"FieldsMapItemName1",{"S","Owner"}}"#).unwrap();
        let mut n = Names::default();
        n.objects.insert("catalog.х".into(), "g1".into());
        let root = ["Ссылка".to_string(), "Владелец".to_string()];
        let mark = |f: &str| n.mark(&st, f, format!("Список.{f}"), &|_| false, Some(&root));
        assert_eq!(mark("Ref"), "~Список.Ref~Список.Ссылка");
        assert_eq!(mark("Owner"), "~Список.Owner");
        assert_eq!(mark("Владелец"), "Список.Владелец");
    }

    #[test]
    fn grouped_commands() {
        let name = |g: &str| std_form_command2(g).map(str::to_string);
        let (a, b) = ("aa042316-63ba-4f10-8d39-3935474562d0", "e44f9b41-bf53-4837-b4d4-f0ff9cdf0feb");
        let help = "39bb0fe9-771d-4dd5-8a6e-2d16984523af";
        assert_eq!(grouped([b, help, a].into_iter(), name), ["Help", "LevelDown", "LevelUp"]);
        // Неполный набор не угадывается.
        assert_eq!(grouped([a, "00000000-0000-0000-0000-000000000001"].into_iter(), name), [format!("\u{1}{a}")]);
    }
}

