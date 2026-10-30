//! Части словаря настроек динамического списка из элементов `<Settings xsi:type="DynamicList">`, которые не задают
//! ни текст запроса, ни поля корня: ключ (`KeyType`, `KeyField`), флаги и состояние сервера (`ServerState`).
//!
//! Читатель XML — mngbase FUN_18073d190: элемент за элементом FUN_18073ee50 по номеру свойства модели logform ставит
//! свойство объекта настроек (те же слоты, что у чтения словаря FUN_1802b1a50); писатель словаря — FUN_1802b4660.
//! - 2 AutoFillAvailableFields — слот +0x270 (запись +0x278), 11 GetInvisibleFieldPresentations — +0x258 (запись
//!   +0x238); умолчание — истина (выгрузка пишет только «ложь»);
//! - KeyType — перечисление mngcore FUN_1806445f0: Auto 0, FieldValue 1, RowKey 2, RowNumber 3 (запись +0x90 — `N`);
//! - 9 KeyField — строка в конец вектора ключевых полей (+0xa0/+0x98); запись — `KeyFields`, список `{"S",имя}`;
//! - 4 Field, 5 CalculatedField, 6 Parameter — объект XDTO в конец коллекции (интерфейс 1808370f0, слоты 3, 5, 7).
//!   `ServerState` — XML этих коллекций (FUN_180702cc0): корень `UniversalListServerOnlyState` без пространства,
//!   по порядку все поля (`Field`, строка 0x180a9c4a0), вычисляемые (`ExpressionField`, 0x180a9c4b8), параметры
//!   (`Parameter`, 0x180a9c4d0) — значения сериализатором XDTO с `xsi:type`.

use crate::xml::imp::dom::El;

type R<T> = Result<T, String>;

const KEY_TYPES: &[(&str, i64)] = &[("Auto", 0), ("FieldValue", 1), ("RowKey", 2), ("RowNumber", 3)];

/// Элементы настроек, которые разбирает этот модуль.
pub const ELEMENTS: &[&str] = &["KeyType", "KeyField", "AutoFillAvailableFields", "GetInvisibleFieldPresentations", "Field", "CalculatedField", "Parameter"];

pub struct Parts {
    pub key_type: i64,
    pub key_fields: Vec<String>,
    pub auto_fill: bool,
    pub invisible: bool,
    /// источник XML состояния сервера для сериализатора (`dcs::server_state_part`): корень без пространства, его
    /// дети — поля и параметры в записи
    pub state: String,
}

fn boolean(e: &El) -> R<bool> {
    match e.text()?.as_str() {
        "true" => Ok(true),
        "false" => Ok(false),
        t => Err(format!("строка {}: <{}>: не булево {t:?}", e.line, e.name)),
    }
}

/// `els` — элементы настроек из `ELEMENTS` по порядку документа; `text` — элемент обратно в текст XML (как
/// `form::xml_text`), `ns` — объявления пространств корня Form.xml (`xmlns:…`).
pub fn parts(els: &[&El], text: fn(&El, &mut String), ns: &[(&str, &str)]) -> R<Parts> {
    let mut p = Parts { key_type: 0, key_fields: Vec::new(), auto_fill: true, invisible: true, state: String::new() };
    let mut fields = String::new();
    let mut calc = String::new();
    let mut params = String::new();
    for e in els {
        match e.name.as_str() {
            "KeyType" => {
                let t = e.text()?;
                p.key_type = KEY_TYPES.iter().find(|(n, _)| *n == t).map(|(_, v)| *v).ok_or_else(|| format!("строка {}: KeyType {t:?} не встречен", e.line))?;
            }
            "KeyField" => p.key_fields.push(e.text()?),
            "AutoFillAvailableFields" => p.auto_fill = boolean(e)?,
            "GetInvisibleFieldPresentations" => p.invisible = boolean(e)?,
            "Field" => value(e, "Field", None, text, &mut fields)?,
            "CalculatedField" => value(e, "ExpressionField", Some("dcssch:CalculatedField"), text, &mut calc)?,
            "Parameter" => value(e, "Parameter", Some("dcssch:Parameter"), text, &mut params)?,
            o => return Err(format!("строка {}: {o} не из настроек ключа и состояния списка", e.line)),
        }
    }
    let mut s = String::from("<x xmlns=\"\"");
    for (k, v) in ns.iter().filter(|(k, _)| k.starts_with("xmlns:")) {
        s += &format!(" {k}=\"{v}\"");
    }
    p.state = s + ">" + &fields + &calc + &params + "</x>";
    Ok(p)
}

/// Значение коллекции — элементом `name` без пространства; тип, которого в Form.xml нет (у параметра и вычисляемого
/// поля он задан моделью), — `xsi:type`.
fn value(e: &El, name: &str, ty: Option<&str>, text: fn(&El, &mut String), out: &mut String) -> R<()> {
    let has = e.attr("xsi:type").is_some();
    if has == ty.is_some() {
        return Err(format!("строка {}: <{}> с xsi:type {:?}: не встречено", e.line, e.name, e.peek_attr("xsi:type")));
    }
    if e.attrs.len() != has as usize {
        return Err(format!("строка {}: у <{}> атрибуты кроме xsi:type", e.line, e.name));
    }
    *out += &format!("<{name} xsi:type=\"{}\">", e.peek_attr("xsi:type").or(ty).unwrap_or_default());
    for c in e.peek_elems() {
        text(c, out);
    }
    e.touch_all();
    *out += &format!("</{name}>");
    Ok(())
}

/// Вычисляемые поля настроек списка (`<CalculatedField>` у `st`), которые попадают в корень дерева полей, — по порядку
/// корня; `has` — есть ли поле набора по пути выражения (`А`, `А.Б`), `cmp` — сравнение имен без учета регистра
/// (`core::compare_nocase`).
/// Схема списка получает их из ULSchemeInfoImpl +0x48 (mngbase FUN_18067c6b0); корень режима 9 (dcs FUN_1800ae8d0)
/// начинается с них (FUN_1800aefe0: карта composer+0x1b0[режим], ключ — dataPath, порядок FUN_18003aed0), поля
/// наборов идут следом без имен, уже взятых вычисляемыми (FUN_1800aebd0). Карту строит FUN_1800a9840: поле не входит,
/// если его отбрасывает ограничение использования (FUN_180062dd0, в режиме 9 — слот +0x18) или если компоновщик вида
/// 2/4 (у списка — 4, mngbase FUN_1802b7570) находит в выражении поле, которого нет среди полей набора режима
/// (FUN_1800bffc0 → FUN_1800b15d0: итог не 1); поле выражения из корня входит и в отбор FUN_1800bfcb0 (байт +0x10,
/// FUN_1800b7430 при p6 = 1). В корень — только имя без точки (FUN_1800ab150: прямой потомок пути).
pub fn calc_root(st: &El, has: &dyn Fn(&str) -> R<bool>, cmp: fn(&str, &str) -> std::cmp::Ordering) -> R<Vec<String>> {
    let mut v: Vec<String> = Vec::new();
    for c in st.peek_elems().filter(|e| e.name == "CalculatedField") {
        let g = |k: &str| c.peek_child(k).and_then(El::peek_text).unwrap_or_default();
        let p = g("dcssch:dataPath");
        let why = |m: &str| Err(format!("строка {}: вычисляемое поле {p}: {m}", c.line));
        if p.is_empty() || p.contains('.') {
            return why("путь пуст или через точку — узел дерева полей (dcs FUN_1800b29c0) не выведен");
        }
        // shortcut: слот +0x18 ограничения (FUN_180062dd0) по чтению XDTO не сопоставлен с `field` — такое поле отказ
        if c.peek_child("dcssch:useRestriction").and_then(|u| u.peek_child("dcssch:field")).and_then(El::peek_text).as_deref() == Some("true") {
            return why("ограничение использования «поле»: слот ограничения не сопоставлен");
        }
        for r in expression_fields(&g("dcssch:expression")).map_err(|m| format!("строка {}: вычисляемое поле {p}: {m}", c.line))? {
            if !has(&r).map_err(|m| format!("строка {}: вычисляемое поле {p}: поле выражения {r}: {m}", c.line))? {
                return why(&format!("поле выражения {r} не из полей набора корня — отбор (dcs FUN_1800bffc0) не выведен"));
            }
        }
        if v.iter().any(|n| cmp(n, &p).is_eq()) {
            return why("два вычисляемых поля с одним именем без учета регистра");
        }
        v.push(p);
    }
    v.sort_by(|a, b| cmp(a, b));
    Ok(v)
}

/// Поля, на которые ссылается выражение компоновки вычисляемого поля, по порядку: пути (`А`, `А.Б`) среди литералов
/// (строка, число, Неопределено, NULL, Истина/Ложь), скобок и арифметики `+ - * /`. shortcut: прочие выражения (функции,
/// сравнения, ВЫБОР) — отказ: узлы полей разбора (dcs FUN_18034e100, FUN_18034d120 +0x58) не повторены.
fn expression_fields(e: &str) -> R<Vec<String>> {
    let lit = ["Неопределено", "Undefined", "NULL", "Истина", "Ложь", "True", "False"];
    let bad = || Err(format!("выражение {:?}: поля выражения не выведены", e.trim()));
    let mut out = Vec::new();
    let mut cs = e.chars().peekable();
    while let Some(&c) = cs.peek() {
        if c.is_whitespace() || "+-*/()".contains(c) {
            cs.next();
        } else if c == '"' {
            cs.next();
            // строка: кавычка внутри удвоена
            loop {
                match cs.next() {
                    Some('"') if cs.peek() == Some(&'"') => {
                        cs.next();
                    }
                    Some('"') => break,
                    Some(_) => {}
                    None => return bad(),
                }
            }
        } else if c.is_ascii_digit() {
            let mut n = String::new();
            while let Some(&d) = cs.peek().filter(|d| d.is_ascii_digit() || **d == '.') {
                n.push(d);
                cs.next();
            }
            if n.matches('.').count() > 1 {
                return bad();
            }
        } else if c.is_alphabetic() || c == '_' {
            let mut n = String::new();
            while let Some(&d) = cs.peek().filter(|d| d.is_alphanumeric() || **d == '_' || **d == '.') {
                n.push(d);
                cs.next();
            }
            // вызов функции, путь с пустой частью — не выведены
            if n.split('.').any(str::is_empty) || cs.clone().find(|c| !c.is_whitespace()) == Some('(') {
                return bad();
            }
            if !lit.iter().any(|l| l.to_lowercase() == n.to_lowercase()) {
                out.push(n);
            }
        } else {
            return bad();
        }
    }
    Ok(out)
}

/// Типы реквизита `key` (`<v8:Type>`) и типы определяемых типов из его `<v8:TypeSet>cfg:DefinedType.X` — состав
/// описания типов `DefinedType.X` (определяемый тип — набор типов своего описания); наборы другого вида (`cfg:CatalogRef`,
/// характеристики) не раскрываются, как и раньше. `None` — типов нет.
pub fn types_of<'a>(x: &'a crate::xml::imp::index::Index, key: &str) -> Option<Vec<&'a str>> {
    let mut v: Vec<&str> = x.attr_types.get(key)?.iter().map(String::as_str).collect();
    for s in x.type_sets.get(key).into_iter().flatten() {
        if let Some(d) = s.strip_prefix("cfg:DefinedType.") {
            v.extend(x.attr_types.get(&format!("DefinedType.{d}")).into_iter().flatten().map(String::as_str));
        }
    }
    (!v.is_empty()).then_some(v)
}

/// GUID команды Delete таблицы динамического списка с основной таблицей `main` (и при своем запросе: объект основной
/// таблицы — FUN_18060cfa0 по id из настроек): в составе команд таблицы списка (mngbase FUN_180611300) из двух GUID
/// всегда один — ec576e13 у журнала документов, объекта с пометкой удаления (ede4bd22 +0x40, ссылочные виды) и задачи,
/// 8d772f97 у регистра сведений независимого не по позиции регистратора (FUN_18060d500). Кнопка `Form.Item.<таблица>.
/// StandardCommand.Delete`: FUN_1802e6a30 → FUN_1802e61f0 — первая команда с этим именем в списке команд элемента
/// (`(this−8)+0x20(элемент)`: +0x30 счет, +0x38 GUID, +0x28 имя; список — FUN_1802fc810/FUN_1802fcdd0), тот же список,
/// что перебирает разрешение исключенных команд FUN_1802e63f0, — правило form/table.rs `list_delete`.
/// shortcut: прочие регистры (2b98afb7 +0x28) — `None`.
pub fn list_delete(x: &crate::xml::imp::index::Index, main: &str) -> Option<&'static str> {
    match main.split('.').next()? {
        "DocumentJournal" | "Catalog" | "Document" | "ChartOfCharacteristicTypes" | "ChartOfAccounts" | "ChartOfCalculationTypes" | "ExchangePlan" | "BusinessProcess"
        | "Task" => Some("ec576e13-1e76-4c33-98aa-a33204514227"),
        "InformationRegister" => {
            let p = &x.objects.get(main)?.props;
            let p = |k: &str| p.get(k).map(String::as_str);
            (p("WriteMode") == Some("Independent") && p("InformationRegisterPeriodicity") != Some("RecorderPosition")).then_some("8d772f97-c0ef-47c0-9cb0-efea28c61341")
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(e: &El, out: &mut String) {
        out.push_str(&format!("<{}>{}</{}>", e.name, e.peek_text().unwrap_or_default(), e.name));
    }

    #[test]
    fn parts_of_settings() {
        let x = crate::xml::imp::dom::parse("<S><KeyType>RowNumber</KeyType><KeyField>А</KeyField><KeyField>Б</KeyField><GetInvisibleFieldPresentations>false</GetInvisibleFieldPresentations><Parameter><dcssch:name>П</dcssch:name></Parameter><Field xsi:type=\"dcssch:DataSetFieldField\"><dcssch:field>Ф</dcssch:field></Field></S>").unwrap();
        let els: Vec<&El> = x.peek_elems().collect();
        let p = parts(&els, text, &[("xmlns", "lf"), ("xmlns:dcssch", "sch")]).unwrap();
        assert_eq!((p.key_type, p.key_fields.join(","), p.auto_fill, p.invisible), (3, "А,Б".to_string(), true, false));
        // поля раньше параметров; у параметра тип — из модели
        assert_eq!(p.state, "<x xmlns=\"\" xmlns:dcssch=\"sch\"><Field xsi:type=\"dcssch:DataSetFieldField\"><dcssch:field>Ф</dcssch:field></Field><Parameter xsi:type=\"dcssch:Parameter\"><dcssch:name>П</dcssch:name></Parameter></x>");
        let bad = crate::xml::imp::dom::parse("<S><KeyType>Ключ</KeyType></S>").unwrap();
        assert!(parts(&bad.peek_elems().collect::<Vec<_>>(), text, &[]).is_err());
    }

    #[test]
    fn calc_fields_of_root() {
        let cf = |p: &str, e: &str| format!("<CalculatedField><dcssch:dataPath>{p}</dcssch:dataPath><dcssch:expression>{e}</dcssch:expression></CalculatedField>");
        let cmp = |a: &str, b: &str| a.to_lowercase().cmp(&b.to_lowercase());
        let st = crate::xml::imp::dom::parse(&format!("<S>{}{}{}</S>", cf("Я", "\"\""), cf("Б", "Дата"), cf("А", "Неопределено"))).unwrap();
        let date = |n: &str| Ok(n == "Дата" || n == "Сумма" || n == "Дата.Год");
        assert_eq!(calc_root(&st, &date, cmp).unwrap(), ["А", "Б", "Я"]);
        // поле выражения не из набора, выражение-функция, путь через точку — отказ
        assert!(calc_root(&st, &|n: &str| Ok(n == "Номер"), cmp).is_err());
        for (p, e) in [("В", "Формат(Дата)"), ("Г.Д", "1"), ("Е", "Дата > 1")] {
            assert!(calc_root(&crate::xml::imp::dom::parse(&format!("<S>{}</S>", cf(p, e))).unwrap(), &date, cmp).is_err());
        }
        // арифметика полей и путь через точку
        assert_eq!(expression_fields("Дата.Год + (Сумма - 1.5) * \"а\"\"б\" / Истина").unwrap(), ["Дата.Год", "Сумма"]);
        assert_eq!(calc_root(&crate::xml::imp::dom::parse(&format!("<S>{}</S>", cf("Ж", "Сумма + Дата.Год"))).unwrap(), &date, cmp).unwrap(), ["Ж"]);
        // вычисляемое поле — в состоянии сервера между полями и параметрами, элементом ExpressionField
        let els: Vec<&El> = st.peek_elems().collect();
        let p = parts(&els[..1], text, &[]).unwrap();
        assert!(p.state.contains("<ExpressionField xsi:type=\"dcssch:CalculatedField\"><dcssch:dataPath>Я</dcssch:dataPath>"), "{}", p.state);
    }
}
