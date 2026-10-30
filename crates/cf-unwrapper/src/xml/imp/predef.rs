//! Предопределенные элементы `Ext/Predefined.xml` (обратное `xml/predefined.rs`): справочник (`<id>.1c`),
//! план видов характеристик (`<id>.7`), план счетов (`<id>.9`), план видов расчета (`<id>.2`).
//!
//! Таблица `{2,N,(<позиция>,<номер колонки>)*N,{1,1,<корень>},-1,<последний номер>}`, строка
//! `{2,<номер>,<значений>,<значения>…,0|1,{1,K,<строки>…}}`; номера — по порядку обхода дерева (корень 0).

use super::dom::El;
use super::index::Index;
use super::tpl::{a, l, s};
use super::Rec;
use crate::brace::{Doc, Node};
use crate::xml::common::ZERO;

const REF: &str = "ae135932-4f94-44df-92c1-c91f15a92848";
const TYPE_DESC: &str = "f5c65050-3bbb-11d5-b988-0050bae0a95d";

const NS: &[(&str, &str)] = &[
    ("xmlns", "http://v8.1c.ru/8.3/xcf/predef"),
    ("xmlns:v8", "http://v8.1c.ru/8.1/data/core"),
    ("xmlns:xr", "http://v8.1c.ru/8.3/xcf/readable"),
    ("xmlns:xs", "http://www.w3.org/2001/XMLSchema"),
    ("xmlns:xsi", "http://www.w3.org/2001/XMLSchema-instance"),
    ("version", "2.21"),
];

fn check_root(doc: &El, ty: &str) -> Result<(), String> {
    if doc.name != "PredefinedData" {
        return Err("корень не <PredefinedData>".into());
    }
    for (k, v) in NS.iter().copied().chain([("xsi:type", ty)]) {
        if doc.attr(k) != Some(v) {
            return Err(format!("<PredefinedData>: {k} не {v:?}"));
        }
    }
    if doc.attrs.len() != NS.len() + 1 {
        return Err("<PredefinedData>: лишние атрибуты".into());
    }
    Ok(())
}

fn rf(id: &str) -> Node {
    l(vec![s("#"), a(REF), l(vec![a("1"), a(id)])])
}

fn col(i: usize, pattern: Node) -> Node {
    l(vec![a(i.to_string()), s(""), l(vec![s("Pattern"), pattern]), s(""), a("0")])
}

/// Код и наименование владельца: тип кода, длина, допустимая длина, длина наименования.
struct Owner {
    number: bool,
    code_len: usize,
    fixed: bool,
    desc_len: usize,
}

impl Owner {
    fn read(p: &El, has_type: bool) -> Result<Owner, String> {
        let n = |t: &str| -> Result<usize, String> { p.text_of(t)?.parse().map_err(|_| format!("владелец: {t} не число")) };
        let number = has_type
            && match p.text_of("CodeType")?.as_str() {
                "String" => false,
                "Number" => true,
                o => return Err(format!("владелец: CodeType {o}")),
            };
        let fixed = match p.text_of("CodeAllowedLength")?.as_str() {
            "Variable" => false,
            "Fixed" => true,
            o => return Err(format!("владелец: CodeAllowedLength {o}")),
        };
        let code_len = n("CodeLength")?;
        // при нулевой длине кода допустимая длина не действует (у 11 справочников БП — Fixed)
        Ok(Owner { number, code_len, fixed: fixed && code_len > 0, desc_len: n("DescriptionLength")? })
    }

    fn code_col(&self) -> Result<Node, String> {
        Ok(match (self.number, self.code_len) {
            (true, n) => l(vec![s("N"), a(n.to_string()), a("0"), a("1")]),
            (false, 0) => l(vec![s("S")]),
            (false, n) => l(vec![s("S"), a(n.to_string()), a(if self.fixed { "0" } else { "1" })]),
        })
    }

    fn desc_col(&self) -> Node {
        if self.desc_len == 0 {
            l(vec![s("S")])
        } else {
            l(vec![s("S"), a(self.desc_len.to_string()), a("1")])
        }
    }

    /// `<Code>`: число — с `xsi:type="xs:decimal"`, строка — как в XML (длину фиксированного кода платформа
    /// при загрузке, возможно, дополняет — не проверено, поэтому отказ).
    fn code(&self, e: &El) -> Result<Node, String> {
        let t = e.text()?;
        if self.number {
            if e.attr("xsi:type") != Some("xs:decimal") || e.attrs.len() != 1 {
                return Err(format!("строка {}: числовой код без xsi:type=\"xs:decimal\"", e.line));
            }
            let v = super::tpl::int(&t).ok_or(format!("строка {}: код {t:?}", e.line))?;
            return Ok(l(vec![s("N"), a(v)]));
        }
        if !e.attrs.is_empty() {
            return Err(format!("строка {}: атрибуты строкового кода", e.line));
        }
        if self.fixed && t.chars().count() != self.code_len {
            return Err(format!("строка {}: код {t:?} короче фиксированной длины {}", e.line, self.code_len));
        }
        // при нулевой длине кода поля Code у таблицы нет (как корень списка, dlist `root_fields`): код из XML не
        // хранится. shortcut: по одному элементу (ГруппыПользователей БП 3.0.107), запись кода загрузчиком
        // предопределенных (basic FUN_1808dca50, через vtable) не прослежена — смотреть при числовом коде
        if self.code_len == 0 {
            return Ok(l(vec![s("S"), s("")]));
        }
        Ok(l(vec![s("S"), s(t)]))
    }
}

/// Строка таблицы: `{2,<номер>,N,<значения>…,0}` или `…,1,{1,K,<дочерние>…}}`.
fn row(num: usize, vals: Vec<Node>, kids: Option<Vec<Node>>) -> Node {
    let mut v = vec![a("2"), a(num.to_string()), a(vals.len().to_string())];
    v.extend(vals);
    match kids {
        Some(k) => {
            v.push(a("1"));
            let mut c = vec![a("1"), a(k.len().to_string())];
            c.extend(k);
            v.push(l(c));
        }
        None => v.push(a("0")),
    }
    l(v)
}

/// Элементы `<Item>` уровня с номерами по порядку обхода; `vals` — значения строки элемента.
fn items(parent: &El, n: &mut usize, vals: &mut dyn FnMut(&El) -> Result<Vec<Node>, String>) -> Result<Vec<Node>, String> {
    let mut out = Vec::new();
    for it in parent.elems() {
        if it.name != "Item" {
            return Err(format!("строка {}: <{}> вместо <Item>", it.line, it.name));
        }
        *n += 1;
        let me = *n;
        let v = vals(it)?;
        let kids = match it.child("ChildItems") {
            Some(c) => {
                let k = items(c, n, vals)?;
                if k.is_empty() {
                    return Err(format!("строка {}: пустой <ChildItems> не встречен", c.line));
                }
                Some(k)
            }
            None => None,
        };
        out.push(row(me, v, kids));
    }
    Ok(out)
}

fn ordered(it: &El, names: &[&str]) -> Result<(), String> {
    let got: Vec<&str> = it.elems().map(|e| e.name.as_str()).collect();
    let want: Vec<&str> = names.iter().copied().filter(|n| *n != "ChildItems" || got.contains(&"ChildItems")).collect();
    if got != want {
        return Err(format!("строка {}: состав элемента {got:?}", it.line));
    }
    Ok(())
}

fn table(cols: Vec<Node>, pairs: Vec<(usize, usize)>, root: Node, last: usize) -> Node {
    let mut c = vec![a(cols.len().to_string())];
    c.extend(cols);
    let mut t = vec![a("2"), a(pairs.len().to_string())];
    for (p, k) in pairs {
        t.push(a(p.to_string()));
        t.push(a(k.to_string()));
    }
    t.extend([l(vec![a("1"), a("1"), root]), a("-1"), a(last.to_string())]);
    l(vec![l(c), l(t)])
}

fn bytes(n: Node) -> Result<Rec, String> {
    Ok(Rec::Raw(Doc::new(true, vec![n]).write()?.into_bytes()))
}

/// Справочник: `{0,{1,<колонки 0–6>,<таблица>}}`; колонки — ссылка, группа, родитель, имя, код, наименование, `{"N"}`.
/// У корня значения обрезаны после последней колонки нулевой длины (код, наименование): так у всех 98 справочников БП.
pub fn catalog(doc: &El, owner: &El) -> Result<Rec, String> {
    check_root(doc, "CatalogPredefinedItems")?;
    let o = Owner::read(owner.req("Properties")?, true)?;
    let mut n = 0;
    let kids = items(doc, &mut n, &mut |it| {
        ordered(it, &["Name", "Code", "Description", "IsFolder", "ChildItems"])?;
        Ok(vec![
            rf(it.req_attr("id")?),
            l(vec![s("B"), super::tpl::boolean(it.req("IsFolder")?)?]),
            rf(ZERO),
            l(vec![s("S"), s(it.text_of("Name")?)]),
            o.code(it.req("Code")?)?,
            l(vec![s("S"), s(it.text_of("Description")?)]),
            l(vec![s("N"), a("0")]),
        ])
    })?;
    if kids.is_empty() {
        return Err("нет элементов: как платформа хранит пустой список, не проверено".into());
    }
    let mut rv = vec![rf(ZERO), l(vec![s("B"), a("1")]), rf(ZERO), l(vec![s("S"), s("Элементы")])];
    if o.code_len == 0 || o.desc_len == 0 {
        rv.push(if o.number { l(vec![s("N"), a("0")]) } else { l(vec![s("S"), s("")]) });
    }
    if o.desc_len == 0 {
        rv.push(l(vec![s("S"), s("")]));
    }
    let cols = vec![col(0, l(vec![s("#"), a(REF)])), col(1, l(vec![s("B")])), col(2, l(vec![s("#"), a(REF)])), col(3, l(vec![s("S")])), col(4, o.code_col()?), col(5, o.desc_col()), col(6, l(vec![s("N")]))];
    let t = table(cols, (0..7).map(|i| (i, i)).collect(), row(0, rv, Some(kids)), n);
    bytes(l(vec![a("0"), l(vec![a("1"), t.list()[0].clone(), t.list()[1].clone()])]))
}

/// План видов характеристик: `{1,{1,<колонки 1–7>,<таблица>}}`; колонки — ссылка, группа, имя, код, наименование,
/// тип значения, `{"N"}`. Корень «Характеристики»: код — пробелы на всю фиксированную длину или пустой.
pub fn characteristic_types(doc: &El, owner: &El, x: &Index) -> Result<Rec, String> {
    check_root(doc, "PlanOfCharacteristicKindPredefinedItems")?;
    let o = Owner::read(owner.req("Properties")?, false)?;
    let mut n = 0;
    let kids = items(doc, &mut n, &mut |it| {
        ordered(it, &["Name", "Code", "Description", "Type", "IsFolder", "ChildItems"])?;
        Ok(vec![
            rf(it.req_attr("id")?),
            l(vec![s("B"), super::tpl::boolean(it.req("IsFolder")?)?]),
            l(vec![s("S"), s(it.text_of("Name")?)]),
            o.code(it.req("Code")?)?,
            l(vec![s("S"), s(it.text_of("Description")?)]),
            l(vec![s("#"), a(TYPE_DESC), super::tpl::type_desc(x, it.req("Type")?)?]),
            l(vec![s("N"), a("0")]),
        ])
    })?;
    if kids.is_empty() {
        return Err("нет элементов: как платформа хранит пустой список, не проверено".into());
    }
    let code = if o.fixed { " ".repeat(o.code_len) } else { String::new() };
    let rv = vec![rf(ZERO), l(vec![s("B"), a("1")]), l(vec![s("S"), s("Характеристики")]), l(vec![s("S"), s(code)]), l(vec![s("S"), s("")]), l(vec![s("#"), a(TYPE_DESC), l(vec![s("Pattern")])])];
    let cols = vec![col(1, l(vec![s("#"), a(REF)])), col(2, l(vec![s("B")])), col(3, l(vec![s("S")])), col(4, o.code_col()?), col(5, o.desc_col()), col(6, l(vec![s("#"), a(TYPE_DESC)])), col(7, l(vec![s("N")]))];
    let t = table(cols, (0..7).map(|i| (i, i + 1)).collect(), row(0, rv, Some(kids)), n);
    bytes(l(vec![a("1"), l(vec![a("1"), t.list()[0].clone(), t.list()[1].clone()])]))
}

const SUBTABLE: &str = "acf6192e-81ca-46ef-93a6-5a6968b78663";

/// Вложенная таблица `{"#",acf6192e-…,{9,{N,<колонки>},{2,N,<пары>,{1,K,<строки>},a,b},{0,0}}}`: у непустой
/// a = -1, b — последний номер; у пустой a = N-1, b = -1 (так у всех таблиц БП: 5 колонок — 4, без колонок — -1).
fn subtable(cols: Vec<Node>, rows: Vec<Node>) -> Node {
    let (nc, k) = (cols.len(), rows.len());
    let mut c = vec![a(nc.to_string())];
    c.extend(cols);
    let mut t = vec![a("2"), a(nc.to_string())];
    for i in 0..nc {
        t.extend([a(i.to_string()), a(i.to_string())]);
    }
    let mut r = vec![a("1"), a(k.to_string())];
    r.extend(rows);
    t.push(l(r));
    if k == 0 {
        t.extend([a((nc as i64 - 1).to_string()), a("-1")]);
    } else {
        t.extend([a("-1"), a((k - 1).to_string())]);
    }
    l(vec![s("#"), a(SUBTABLE), l(vec![a("9"), l(c), l(t), l(vec![a("0"), a("0")])])])
}

fn named_col(i: usize, name: &str, pattern: Node) -> Node {
    l(vec![a(i.to_string()), s(name), l(vec![s("Pattern"), pattern]), s(""), a("0")])
}

/// `<AccountingFlags><Flag ref>…`: ровно признаки владельца в порядке описания → `{"B",…}`.
fn flags(e: &El, want: &[String]) -> Result<Vec<Node>, String> {
    let got: Vec<&El> = e.elems().collect();
    if got.len() != want.len() {
        return Err(format!("строка {}: признаков учета {} вместо {}", e.line, got.len(), want.len()));
    }
    if got.is_empty() {
        e.text()?;
    }
    got.iter()
        .zip(want)
        .map(|(f, w)| {
            if f.name != "Flag" || f.req_attr("ref")? != w || f.attrs.len() != 1 {
                return Err(format!("строка {}: признак учета не {w}", f.line));
            }
            Ok(l(vec![s("B"), super::tpl::boolean(f)?]))
        })
        .collect()
}

/// План счетов: `{2,{1,<колонки>,<таблица>}}`. Колонки 0 ссылка, 1 имя, 2 код, 3 наименование, 4 вид счета,
/// 5 забалансовый, 6 виды субконто, признаки учета (номера 7…, имя — uuid признака), 10000 порядок, 20000 `{"N"}`;
/// в строке значения 0–6, порядок, `{"N",0}`, признаки. Корень «Счета» — 7 значений.
pub fn accounts(doc: &El, owner: &El, x: &Index) -> Result<Rec, String> {
    check_root(doc, "ChartOfAccountsPredefinedItems")?;
    let p = owner.req("Properties")?;
    let full = format!("ChartOfAccounts.{}", p.text_of("Name")?);
    let n = |t: &str| -> Result<usize, String> { p.text_of(t)?.parse().map_err(|_| format!("владелец: {t} не число")) };
    let (code_len, desc_len, order_len) = (n("CodeLength")?, n("DescriptionLength")?, n("OrderLength")?);
    let str_col = |len: usize| if len == 0 { l(vec![s("S")]) } else { l(vec![s("S"), a(len.to_string()), a("1")]) };
    let co = owner.req("ChildObjects")?;
    let kids_of = |k: &str| -> Result<Vec<(String, String)>, String> { co.all(k).map(|f| Ok((format!("{full}.{k}.{}", f.req("Properties")?.text_of("Name")?), f.req_attr("uuid")?.to_string()))).collect() };
    let (af, ef) = (kids_of("AccountingFlag")?, kids_of("ExtDimensionAccountingFlag")?);
    let af_names: Vec<String> = af.iter().map(|f| f.0.clone()).collect();
    let ef_names: Vec<String> = ef.iter().map(|f| f.0.clone()).collect();
    let mut ecols = vec![col(0, l(vec![s("#"), a(REF)])), col(1, l(vec![s("B")]))];
    ecols.extend(ef.iter().enumerate().map(|(i, f)| named_col(2 + i, &f.1, l(vec![s("B")]))));
    let ext_dims = |e: Option<&El>| -> Result<Node, String> {
        let mut rows = Vec::new();
        if let Some(e) = e {
            for (i, d) in e.elems().enumerate() {
                if d.name != "ExtDimensionType" || d.attrs.len() != 1 {
                    return Err(format!("строка {}: <{}> в видах субконто", d.line, d.name));
                }
                ordered(d, &["Turnover", "AccountingFlags"])?;
                let kn = d.req_attr("name")?;
                let kid = x.predefined.get(kn).ok_or(format!("строка {}: вид субконто {kn} не найден", d.line))?;
                let mut v = vec![rf(kid), l(vec![s("B"), super::tpl::boolean(d.req("Turnover")?)?])];
                v.extend(flags(d.req("AccountingFlags")?, &ef_names)?);
                rows.push(row(i, v, None));
            }
            if rows.is_empty() {
                e.text()?;
            }
        }
        Ok(subtable(ecols.clone(), rows))
    };
    let mut cnt = 0;
    let kids = items(doc, &mut cnt, &mut |it| {
        ordered(it, &["Name", "Code", "Description", "AccountType", "OffBalance", "Order", "AccountingFlags", "ExtDimensionTypes", "ChildItems"])?;
        let kind = super::tpl::token(it.req("AccountType")?, &[("0", "Active"), ("1", "Passive"), ("2", "ActivePassive")])?;
        let code = it.req("Code")?;
        if !code.attrs.is_empty() {
            return Err(format!("строка {}: атрибуты кода", code.line));
        }
        let mut v = vec![
            rf(it.req_attr("id")?),
            l(vec![s("S"), s(it.text_of("Name")?)]),
            l(vec![s("S"), s(code.text()?)]),
            l(vec![s("S"), s(it.text_of("Description")?)]),
            l(vec![s("N"), kind]),
            l(vec![s("B"), super::tpl::boolean(it.req("OffBalance")?)?]),
            ext_dims(Some(it.req("ExtDimensionTypes")?))?,
            l(vec![s("S"), s(it.text_of("Order")?)]),
            l(vec![s("N"), a("0")]),
        ];
        v.extend(flags(it.req("AccountingFlags")?, &af_names)?);
        Ok(v)
    })?;
    if kids.is_empty() {
        return Err("нет элементов: как платформа хранит пустой список, не проверено".into());
    }
    let rv = vec![rf(ZERO), l(vec![s("S"), s("Счета")]), l(vec![s("S"), s("")]), l(vec![s("S"), s("")]), l(vec![s("N"), a("0")]), l(vec![s("B"), a("0")]), ext_dims(None)?];
    let mut cols = vec![
        col(0, l(vec![s("#"), a(REF)])),
        col(1, l(vec![s("S")])),
        col(2, str_col(code_len)),
        col(3, str_col(desc_len)),
        col(4, l(vec![s("N")])),
        col(5, l(vec![s("B")])),
        l(vec![a("6"), s(""), l(vec![s("Pattern")]), s(""), a("0")]),
    ];
    let mut pairs: Vec<(usize, usize)> = (0..7).map(|i| (i, i)).collect();
    for (i, f) in af.iter().enumerate() {
        cols.push(named_col(7 + i, &f.1, l(vec![s("B")])));
        pairs.push((9 + i, 7 + i));
    }
    cols.extend([col(10000, str_col(order_len)), col(20000, l(vec![s("N")]))]);
    pairs.extend([(7, 10000), (8, 20000)]);
    let t = table(cols, pairs, row(0, rv, Some(kids)), cnt);
    bytes(l(vec![a("2"), l(vec![a("1"), t.list()[0].clone(), t.list()[1].clone()])]))
}

/// План видов расчета: `{9,{9,<колонки 1–9>},{2,9,<пары>,{1,K,<строки>},-1,<последний номер>},{0,0}}` без корня,
/// номера с 0. Колонки ссылка, имя, код, наименование, период действия базовый, три вложенные таблицы видов
/// расчета (в XML их нет — пустые), `{"N"}`.
pub fn calc_types(doc: &El, owner: &El) -> Result<Rec, String> {
    check_root(doc, "CalculationTypePredefinedItems")?;
    let o = Owner::read(owner.req("Properties")?, true)?;
    let mut rows = Vec::new();
    for (i, it) in doc.elems().enumerate() {
        if it.name != "Item" {
            return Err(format!("строка {}: <{}> вместо <Item>", it.line, it.name));
        }
        ordered(it, &["Name", "Code", "Description", "ActionPeriodIsBase"])?;
        let mut v = vec![
            rf(it.req_attr("id")?),
            l(vec![s("S"), s(it.text_of("Name")?)]),
            o.code(it.req("Code")?)?,
            l(vec![s("S"), s(it.text_of("Description")?)]),
            l(vec![s("B"), super::tpl::boolean(it.req("ActionPeriodIsBase")?)?]),
        ];
        v.extend((0..3).map(|_| subtable(vec![], vec![])));
        v.push(l(vec![s("N"), a("0")]));
        rows.push(row(i, v, None));
    }
    if rows.is_empty() {
        return Err("нет элементов: как платформа хранит пустой список, не проверено".into());
    }
    let sub = l(vec![s("#"), a(SUBTABLE)]);
    let cols = vec![col(1, l(vec![s("#"), a(REF)])), col(2, l(vec![s("S")])), col(3, o.code_col()?), col(4, o.desc_col()), col(5, l(vec![s("B")])), col(6, sub.clone()), col(7, sub.clone()), col(8, sub), col(9, l(vec![s("N")]))];
    let k = rows.len();
    let mut c = vec![a("9")];
    c.extend(cols);
    let mut t = vec![a("2"), a("9")];
    for i in 0..9 {
        t.extend([a(i.to_string()), a((i + 1).to_string())]);
    }
    let mut r = vec![a("1"), a(k.to_string())];
    r.extend(rows);
    t.extend([l(r), a("-1"), a((k - 1).to_string())]);
    bytes(l(vec![a("9"), l(c), l(t), l(vec![a("0"), a("0")])]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::xml::imp::dom::parse;

    #[test]
    fn catalog_tree_numbering_and_root() {
        let head = NS.iter().map(|(k, v)| format!(" {k}=\"{v}\"")).collect::<String>();
        let it = |id: &str, name: &str, kids: &str| format!("<Item id=\"{id}\"><Name>{name}</Name><Code/><Description>Д</Description><IsFolder>{}</IsFolder>{kids}</Item>", !kids.is_empty());
        let x = format!("<PredefinedData{head} xsi:type=\"CatalogPredefinedItems\">{}{}</PredefinedData>", it("a", "А", &format!("<ChildItems>{}</ChildItems>", it("b", "Б", ""))), it("c", "В", ""));
        let owner = parse("<Catalog><Properties><CodeType>String</CodeType><CodeLength>0</CodeLength><CodeAllowedLength>Fixed</CodeAllowedLength><DescriptionLength>25</DescriptionLength></Properties></Catalog>").unwrap();
        let Rec::Raw(b) = catalog(&parse(&x).unwrap(), &owner).unwrap() else { panic!() };
        let t = String::from_utf8(b).unwrap().replace("\r\n", "");
        // корень с пустым кодом (длина кода 0), номера по обходу: А 1, Б 2, В 3; последний номер 3
        assert!(t.contains("{\"S\",\"Элементы\"},{\"S\",\"\"},1,{1,2,{2,1,7,"), "{t}");
        assert!(t.contains("{2,2,7,") && t.contains("{2,3,7,") && t.ends_with("},-1,3}}}"), "{t}");
        assert!(t.contains("{4,\"\",{\"Pattern\",{\"S\"}},\"\",0},{5,\"\",{\"Pattern\",{\"S\",25,1}}"), "{t}");
    }
}
