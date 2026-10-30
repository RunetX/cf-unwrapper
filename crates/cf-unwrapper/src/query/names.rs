//! Имя поля выборки без псевдонима — как его дает компиляция запроса компоновки (entext.dll, объект CLSID
//! `53eef593…`, +0x70 FUN_180384b00 → FUN_180345a60 → FUN_18034f420 → FUN_18037ce90). FUN_18037ce90: у поля без
//! псевдонима имя на языке 0 (+0x728, английское) и 1 (+0x740, русское) дает метод +0xf8 класса выражения; ложь — имя
//! «Field»/«Поле» (таблица 0x1805dec60) и номер: счетчик с 1 на часть запроса, растет только на таких полях (и на
//! таких же вложенных полях `Т.ТЧ.(…)`). Классы выражений, которые строит разбор (FUN_180336000…FUN_18034a000,
//! фабрика по GUID FUN_18044eb30), и их +0xf8:
//! - параметр `&Имя` (лексер FUN_1803dcbf0: `&` → вид 6, текст — буквы, цифры, `_` после `&`; разбор FUN_180341820:
//!   вид 6 → класс vtable 0x1805f1b10) — FUN_1804a16b0: имя = текст параметра на обоих языках;
//! - поле `Т.Поле` (vtable 0x1805f9380, FUN_18046a130 → FUN_1802e1120): имя поля на языке (стандартное — английское /
//!   русское); путь через точку — склейка имен звеньев, первое звено — при условии +0x28 поля (не разобрано);
//! - `Т.ТЧ.(…)` (vtable 0x1805fae00 — его GUID FUN_18037ce90 проверяет для вложенных полей; FUN_1804714f0 →
//!   FUN_1802e1120 без поля → FUN_1801b4db0): имя таблицы на языке — у табличной части объекта ее имя;
//! - агрегаты (токены 0x3fb СУММА, 0x3fc КОЛИЧЕСТВО, 0x3fd МИНИМУМ, 0x3fe МАКСИМУМ, 0x3ff СРЕДНЕЕ; таблица ключевых слов
//!   ~0x1805e6800; конструкторы FUN_18033dc80, FUN_18033c490, FUN_18033e200, FUN_18033e4c0, FUN_18033df40 — vtable
//!   0x1805f4108, 0x1805f3eb0/0x1805f4360, 0x1805f3060, 0x1805f2e08, 0x1805f45b8) — FUN_180477040: +0xf8 аргумента
//!   (+0x30); у `КОЛИЧЕСТВО(*)` аргумента нет — ложь;
//! - `ПРЕДСТАВЛЕНИЕ`/`ПРЕДСТАВЛЕНИЕССЫЛКИ` (0x441/0x442, FUN_18033ea20, vtable 0x1805f1860) — FUN_1804bde40: имя
//!   аргумента + «Presentation»/«Представление»; из корня набора такое поле пропадает (где — не найдено) — не выводится;
//! - все прочие классы разбора (константы FUN_18045fae0/FUN_180340dc0 → 0x1805f5940, ВЫБОР 0x1805f2018, ВЫРАЗИТЬ
//!   FUN_18049b090 → 0x1805f8f20, ЕСТЬNULL 0x1805f1dc0, функции и операции, `(…).Поле` FUN_180342360 → 0x1805f9cc8,
//!   унарный минус 0x1805f53c8, ТИП 0x1805f8680, ТИПЗНАЧЕНИЯ 0x1805f88f0 …) — FUN_180014540: ложь. Скобки класса не
//!   дают (FUN_18033c260 → FUN_180342360 возвращает выражение как есть).

use super::parse::{Batch, Expr, Select, Source, Table};
use super::resolve::{class_en, eq, Ty};
use crate::xml::imp::index::Index;

/// Имя поля выборки без псевдонима.
#[derive(Debug, PartialEq)]
pub enum Unnamed<'a> {
    /// Одно имя на обоих языках.
    Fixed(&'a str),
    /// `Источник.Поле` / `Поле` (и путь дальше): имя по метаданным поля.
    Field(&'a [String]),
    /// `Т.ТЧ.(…)`: имя табличной части по метаданным.
    Table(&'a [String]),
    /// +0xf8 ложь: «Field»/«Поле» + номер по порядку таких полей части запроса.
    Numbered,
    /// Не выведено (представление, `*`, параметр компоновки).
    No,
}

pub(crate) const AGGREGATES: &[&str] = &["СУММА", "SUM", "КОЛИЧЕСТВО", "COUNT", "МИНИМУМ", "MIN", "МАКСИМУМ", "MAX", "СРЕДНЕЕ", "AVG"];
const PRESENTATION: &[&str] = &["ПРЕДСТАВЛЕНИЕ", "PRESENTATION", "ПРЕДСТАВЛЕНИЕССЫЛКИ", "REFPRESENTATION"];

pub fn unnamed(e: &Expr) -> Unnamed<'_> {
    match e {
        Expr::Param(n) => Unnamed::Fixed(n),
        Expr::Path(p) => Unnamed::Field(p),
        Expr::Nested(p, _) => Unnamed::Table(p),
        Expr::Call { name, args, .. } if AGGREGATES.contains(&name.as_str()) => match args.as_slice() {
            [Expr::Star(p)] if p.is_empty() => Unnamed::Numbered,
            [x] => unnamed(x),
            _ => Unnamed::No,
        },
        Expr::Call { name, .. } if PRESENTATION.contains(&name.as_str()) => Unnamed::No,
        Expr::Star(_) | Expr::Dcs(_) | Expr::Tuple(_) | Expr::Sub(_) => Unnamed::No,
        _ => Unnamed::Numbered,
    }
}

/// `СТРОКА(…)` / `STRING(…)` в выражении (`t` — тип аргумента). entext: слово без числа в скобках разбор первичного
/// выражения FUN_180341820 отдает поиску функции по имени FUN_18033b7b0 (трасса qp-names-9: FUN_1803414c0, литерал
/// типа для лексемы 0x41a, на таком тексте не вызывается): при `(` функция берется из реестра (служба
/// `7c02c03d-…` FUN_1803b8a00 → класс FUN_1803c4920, интерфейс vtable 0x1805db560, +0x18 FUN_1803ef050 — поиск по
/// имени; заполняет FUN_1803ee580 через FUN_1803eeed0 по именам +0x18(i)). STRING/СТРОКА — класс vtable 0x1805e63a8:
/// аргументов от +0x38 до +0x40 (FUN_18001dc00: 1 и 1), проверка аргумента +0x48 FUN_18043dd30 — тип аргумента
/// (+0x60) целиком из {Строка, Число, Булево, Дата, d6e85ef8-… (тип entext)} (FUN_18045e0d0: ни одного типа вне
/// набора), а не УникальныйИдентификатор, Null, ссылка; результат +0x28 FUN_18043da10 — Строка. Узел вызова — класс
/// vtable 0x1805f2268, его +0xf8 — FUN_180014540 (ложь: без псевдонима «Field»/«Поле» + номер, как `unnamed`).
/// `СТРОКА(<число>)` — тот же итог и при литерале типа FUN_1803414c0 (класс 0x1805fa870, +0xf8 тоже ложь). Тип,
/// который тут не различается (параметр, NULL, НЕОПРЕДЕЛЕНО, ВЫБОР без ИНАЧЕ — Null), — отказ.
pub fn string_type(args: &[Expr], distinct: bool, t: Option<&[Ty]>) -> Result<(), String> {
    match (args, t) {
        ([a], Some(t)) if !distinct && !t.is_empty() && t.iter().all(|t| matches!(t, Ty::Prim | Ty::Date)) && !matches!(a, Expr::Const("NULL" | "UNDEFINED") | Expr::Case { other: None, .. }) => Ok(()),
        _ => Err("СТРОКА(…): аргумент не из простых типов Строка/Число/Булево/Дата (entext FUN_18043dd30) или его тип не выведен".into()),
    }
}

/// Источник с псевдонимом `a` среди источников выборки (с соединениями и скобками); у таблицы без псевдонима —
/// последнее имя таблицы (как в `resolve`).
fn source<'a>(from: &'a [Source], a: &str) -> Option<&'a Source> {
    from.iter().find_map(|s| {
        let own = match &s.table {
            Table::Name { path, .. } => path.last(),
            _ => None,
        };
        if s.alias.as_ref().or(own).is_some_and(|x| eq(x, a)) {
            return Some(s);
        }
        let g = match &s.table {
            Table::Group(g) => source(std::slice::from_ref(g.as_ref()), a),
            _ => None,
        };
        g.or_else(|| s.joins.iter().find_map(|j| source(std::slice::from_ref(&j.source), a)))
    })
}

/// Выборка производного источника: первая часть вложенного запроса или запроса пакета `ПОМЕСТИТЬ` (последнего до
/// запроса `qi`) и номер запроса, временные таблицы которого ей видны.
fn inner<'a>(b: &'a Batch, qi: usize, s: &'a Source) -> Option<(&'a Select, usize)> {
    match &s.table {
        Table::Sub(q) => Some((q.parts.first()?, qi)),
        Table::Name { path, params: None } if path.len() == 1 => {
            let (i, q) = b.queries[..qi].iter().enumerate().rev().find(|(_, q)| q.parts.first().and_then(|p| p.into.as_deref()).is_some_and(|t| eq(t, &path[0])))?;
            Some((q.parts.first()?, i))
        }
        _ => None,
    }
}

/// Имена (язык 0, язык 1) поля `Источник.Поле` без псевдонима, если источник — вложенный запрос или временная
/// таблица (`None` — источник не такой). entext: поля производного источника создает FUN_180373e40 по полям выборки
/// (вектор +0x168 части): объект класса vtable 0x1805dcfa8, FUN_1803e1ea0 (вызов 0x1803757eb) пишет +0x68 ← имя поля
/// выборки +0x728 (язык 0) и +0x80 ← +0x740 (язык 1); его +0x18 FUN_1803e2100 отдает +0x68 / +0x80 по языку, а
/// FUN_1802e1120 при пустом — язык 0. Имена поля выборки дает FUN_18037ce90: с псевдонимом +0x728 = псевдоним, +0x740
/// не заполняется (значит, на обоих языках псевдоним); без псевдонима — +0xf8 выражения на языках 0 и 1 (`unnamed`).
/// У того же поля по трассе бывает и класс vtable 0x1805d0980 с одним именем (+0x18 FUN_1802c5840: язык 0 — +0x38,
/// иначе пусто), какой из них видит компоновка — не выведено; поэтому поле берется только с одинаковым именем на
/// обоих языках (псевдоним, параметр, реквизит таблицы по метаданным), иначе отказ.
pub fn derived(x: &Index, b: &Batch, s: &Select, p: &[String], std: &[(&str, &str)]) -> Result<Option<(String, String)>, String> {
    let qi = b.queries.len().saturating_sub(1);
    let Some((sel, i)) = source(&s.from, &p[0]).and_then(|src| inner(b, qi, src)) else { return Ok(None) };
    let (n0, n1) = field_names(x, b, i, sel, &p[1], std)?;
    if n0 != n1 {
        return Err(format!("поле {} производного источника без псевдонима: имена на языках разные ({n0}/{n1}), класс поля (entext 0x1805dcfa8 или 0x1805d0980) не выведен", p.join(".")));
    }
    Ok(Some((n0, n1)))
}

/// Имена поля `n` выборки производного источника; поле с невыведенным именем в выборке — отказ (оно может совпасть).
fn field_names(x: &Index, b: &Batch, qi: usize, s: &Select, n: &str, std: &[(&str, &str)]) -> Result<(String, String), String> {
    let mut hit = None;
    for f in &s.fields {
        let pair = match &f.alias {
            Some(a) => Some((a.clone(), a.clone())),
            None => match unnamed(&f.expr) {
                Unnamed::Fixed(v) => Some((v.to_string(), v.to_string())),
                Unnamed::Field(q) if q.len() == 2 => field_pair(x, b, qi, s, q, std)?,
                _ => None,
            },
        };
        let Some((a, c)) = pair else { return Err(format!("поле {n} производного источника: у его выборки есть поле без псевдонима, имя которого не выведено")) };
        if eq(&a, n) || eq(&c, n) {
            if hit.is_some() {
                return Err(format!("поле {n} производного источника неоднозначно"));
            }
            hit = Some((a, c));
        }
    }
    hit.ok_or_else(|| format!("поле {n} производного источника не найдено"))
}

/// Имена поля выборки `Т.Поле` без псевдонима (entext FUN_1802e1120 без подпути: имя поля на языке): реквизит
/// объекта или табличной части — имя по метаданным на обоих языках; поле производного источника — его имена;
/// стандартное и прочее — `None` (не выведено).
fn field_pair(x: &Index, b: &Batch, qi: usize, s: &Select, q: &[String], std: &[(&str, &str)]) -> Result<Option<(String, String)>, String> {
    let Some(src) = source(&s.from, &q[0]) else { return Ok(None) };
    if let Some((sel, i)) = inner(b, qi, src) {
        return field_names(x, b, i, sel, &q[1], std).map(Some);
    }
    if std.iter().any(|(e, r)| eq(e, &q[1]) || eq(r, &q[1])) {
        return Ok(None);
    }
    let Table::Name { path, params: None } = &src.table else { return Ok(None) };
    if path.len() < 2 {
        return Ok(None);
    }
    let Some(obj) = class_en(&path[0]).and_then(|c| x.objects.keys().find(|k| eq(k, &format!("{c}.{}", path[1])))) else { return Ok(None) };
    let meta = |prefix: String| x.full.keys().find_map(|k| k.strip_prefix(&prefix).filter(|r| eq(r, &q[1])).map(str::to_string));
    Ok(match path.len() {
        2 => x.objects[obj].kids.iter().find(|(k, v)| matches!(k.as_str(), "Attribute" | "Dimension" | "Resource") && eq(v, &q[1])).map(|(_, v)| (v.clone(), v.clone())),
        3 => x.objects[obj].kids.iter().find(|(k, v)| k == "TabularSection" && eq(v, &path[2])).and_then(|(_, t)| meta(format!("{obj}.TabularSection.{t}.Attribute."))).map(|v| (v.clone(), v)),
        _ => None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classes() {
        let f = |q: &str| {
            let b = super::super::parse(&format!("ВЫБРАТЬ {q} ИЗ Справочник.Банки КАК Б")).unwrap();
            let e = b.queries[0].parts[0].fields[0].expr.clone();
            format!("{:?}", unnamed(&e))
        };
        assert_eq!(f("&П"), "Fixed(\"П\")");
        assert_eq!(f("МАКСИМУМ(Б.Код)"), "Field([\"Б\", \"Код\"])");
        assert_eq!(f("(Б.Код)"), "Field([\"Б\", \"Код\"])");
        assert_eq!(f("КОЛИЧЕСТВО(*)"), "Numbered");
        assert_eq!(f("КОЛИЧЕСТВО(РАЗЛИЧНЫЕ &П)"), "Fixed(\"П\")");
        for q in ["1", "\"а\"", "ИСТИНА", "ВЫБОР КОГДА ИСТИНА ТОГДА 1 КОНЕЦ", "ЕСТЬNULL(Б.Код, 0)", "Б.Код + 1", "ПОДСТРОКА(Б.Код, 1, 2)", "СУММА(Б.Код * 2)"] {
            assert_eq!(f(q), "Numbered", "{q}");
        }
        assert_eq!(f("СТРОКА(100)"), "Numbered");
        let p = [Ty::Prim];
        assert!(string_type(&[Expr::Num("100".into())], false, Some(&p)).is_ok());
        assert!(string_type(&[Expr::Path(vec!["Б".into(), "Код".into()])], false, Some(&p)).is_ok());
        assert!(string_type(&[Expr::Path(vec!["Б".into(), "Код".into()])], false, Some(&[Ty::Uuid])).is_err());
        assert!(string_type(&[Expr::Const("NULL")], false, Some(&p)).is_err());
        assert!(string_type(&[Expr::Param("П".into())], false, None).is_err());
        assert_eq!(f("ПРЕДСТАВЛЕНИЕ(Б.Код)"), "No");
        assert_eq!(f("Б.*"), "No");
    }

    #[test]
    fn derived_fields() {
        let mut x = Index::default();
        let mut o = crate::xml::imp::index::Obj::default();
        o.kids = vec![("Attribute".into(), "Дата".into())];
        x.objects.insert("InformationRegister.ЭП".into(), o);
        let std = [("Period", "Период")];
        let d = |q: &str, p: &str| {
            let b = super::super::parse(q).unwrap();
            let p: Vec<String> = p.split('.').map(String::from).collect();
            derived(&x, &b, &b.queries.last().unwrap().parts[0], &p, &std)
        };
        let vt = "ВЫБРАТЬ Р.дата, Р.Комм КАК К, &П ПОМЕСТИТЬ ВТ ИЗ РегистрСведений.ЭП КАК Р; ВЫБРАТЬ 1 ИЗ ВТ КАК Т2";
        assert_eq!(d(vt, "Т2.Дата"), Ok(Some(("Дата".into(), "Дата".into()))));
        assert_eq!(d(vt, "Т2.к"), Ok(Some(("К".into(), "К".into()))));
        assert_eq!(d(vt, "Т2.П"), Ok(Some(("П".into(), "П".into()))));
        let sub = "ВЫБРАТЬ 1 ИЗ (ВЫБРАТЬ Т.Дата КАК Ссылка ИЗ (ВЫБРАТЬ Р.Дата ИЗ РегистрСведений.ЭП КАК Р) КАК Т) КАК Т";
        assert_eq!(d(sub, "Т.Ссылка"), Ok(Some(("Ссылка".into(), "Ссылка".into()))));
        assert_eq!(d(sub, "Р.Дата"), Ok(None));
        // стандартное поле, выражение без псевдонима — имена на языках разные или не выведены: отказ
        assert!(d("ВЫБРАТЬ Р.Период ПОМЕСТИТЬ ВТ ИЗ РегистрСведений.ЭП КАК Р; ВЫБРАТЬ 1 ИЗ ВТ", "ВТ.Период").is_err());
        assert!(d("ВЫБРАТЬ 1 ИЗ (ВЫБРАТЬ Р.Дата КАК Д, 2 ИЗ РегистрСведений.ЭП КАК Р) КАК Т", "Т.Д").is_err());
    }
}
