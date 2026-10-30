//! Предопределенные элементы `Ext/Predefined.xml`: справочника (запись `<id>.1c`) и плана видов расчета (`<id>.2`).
//!
//! `{0,{1,{N,<описания колонок>...},{2,N,<пары номеров>...,{1,1,<корень>},-1,<последний номер>}}}`,
//! строка — `{2,<номер>,N,<значения колонок>...,0|1,{1,M,<строки>...}}`.
//! Колонки справочника: 0 ссылка `{"#",<тип>,{1,<uuid>}}`, 1 группа `{"B",0|1}`, 2 родитель, 3 имя, 4 код
//! (`{"S",..}` или `{"N",..}`), 5 наименование.
//! План видов расчета: `{8,{N,<колонки>...},{2,N,<пары>...,{1,M,<строки>...},-1,<последний номер>}}` без иерархии;
//! колонки 0 ссылка, 1 имя, 2 код, 3 наименование, 4 период действия базовый, 5–7 вложенные таблицы видов расчета.

use super::ext::esc;
use crate::brace::Node;

fn head(ty: &str) -> String {
    format!("<PredefinedData xmlns=\"http://v8.1c.ru/8.3/xcf/predef\" xmlns:v8=\"http://v8.1c.ru/8.1/data/core\" xmlns:xr=\"http://v8.1c.ru/8.3/xcf/readable\" xmlns:xs=\"http://www.w3.org/2001/XMLSchema\" xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\" xsi:type=\"{ty}\" version=\"2.21\">\r\n")
}

/// Строки таблицы `{2,N,<пары>...,{1,M,<строки>...},-1,<последний номер>}`.
fn rows(t: &Node) -> Option<&[Node]> {
    t.list().iter().rev().find(|x| matches!(x, Node::List(_))).and_then(|c| c.list().get(2..))
}

/// Текст тела XML без заголовка `<?xml`; `None` — предопределенных нет (файл не выгружается).
/// `code_len` — длина кода справочника: хранимый код длиннее нее платформа обрезает.
pub fn catalog(n: &Node, code_len: usize) -> Result<Option<String>, String> {
    let rows = n.get(1).and_then(|t| t.get(2)).ok_or("предопределенные: нет таблицы")?;
    let root = self::rows(rows).and_then(|r| r.first()).ok_or("предопределенные: нет корня")?;
    let Some(kids) = children(root)? else { return Ok(None) };
    let mut o = head("CatalogPredefinedItems");
    for r in kids {
        row(r, 1, code_len, &mut o)?;
    }
    o.push_str("</PredefinedData>");
    Ok(Some(o))
}

fn children(r: &Node) -> Result<Option<&[Node]>, String> {
    let nc: usize = r.get(2).and_then(Node::atom).and_then(|a| a.parse().ok()).ok_or("предопределенные: нет числа колонок")?;
    match r.get(3 + nc).and_then(Node::atom) {
        Some("1") => Ok(r.get(4 + nc).and_then(|c| c.list().get(2..)).filter(|k| !k.is_empty())),
        _ => Ok(None),
    }
}

fn row(r: &Node, depth: usize, code_len: usize, o: &mut String) -> Result<(), String> {
    let t = "\t".repeat(depth);
    let v = |i: usize| r.get(3 + i).ok_or_else(|| format!("предопределенные: нет колонки {i}"));
    let s = |i: usize| -> Result<&str, String> { v(i)?.get(1).and_then(|x| x.str().or(x.atom())).ok_or_else(|| format!("предопределенные: колонка {i}")) };
    let id = v(0)?.get(2).and_then(|x| x.get(1)).and_then(Node::atom).ok_or("предопределенные: нет ссылки")?;
    let folder = s(1)? == "1";
    o.push_str(&format!("{t}<Item id=\"{id}\">\r\n{t}\t<Name>{}</Name>\r\n", esc(s(3)?)));
    code_desc(v(4)?, s(5)?, code_len, &t, o)?;
    o.push_str(&format!("{t}\t<IsFolder>{folder}</IsFolder>\r\n"));
    if let Some(kids) = children(r)? {
        o.push_str(&format!("{t}\t<ChildItems>\r\n"));
        for k in kids {
            row(k, depth + 2, code_len, o)?;
        }
        o.push_str(&format!("{t}\t</ChildItems>\r\n"));
    }
    o.push_str(&format!("{t}</Item>\r\n"));
    Ok(())
}

/// Код `{"S",..}`/`{"N",..}` и наименование.
fn code_desc(code: &Node, d: &str, code_len: usize, t: &str, o: &mut String) -> Result<(), String> {
    let c = code.get(1).and_then(|x| x.str().or(x.atom())).ok_or("предопределенные: код")?;
    if code.get(0).and_then(Node::str) == Some("N") {
        o.push_str(&format!("{t}\t<Code xsi:type=\"xs:decimal\">{c}</Code>\r\n"));
    } else {
        let c: String = if code_len > 0 { c.chars().take(code_len).collect() } else { c.into() };
        o.push_str(&if c.is_empty() { format!("{t}\t<Code/>\r\n") } else { format!("{t}\t<Code>{}</Code>\r\n", esc(&c)) });
    }
    o.push_str(&if d.is_empty() { format!("{t}\t<Description/>\r\n") } else { format!("{t}\t<Description>{}</Description>\r\n", esc(d)) });
    Ok(())
}

/// Предопределенные плана видов характеристик (запись `<id>.7`): `{1,{1,{N,<колонки>...},<таблица>}}`, иерархия как
/// у справочника; колонки 0 ссылка, 1 группа, 2 имя, 3 код, 4 наименование, 5 тип значения
/// `{"#",f5c65050-…,{"Pattern",...}}`, дальше `{"N",0}` (что это — неизвестно; другое значение не выгружаем).
/// `types` — TypeId → `CatalogRef.Имя`.
pub fn characteristic_types(n: &Node, types: &std::collections::HashMap<String, String>) -> Result<Option<String>, String> {
    let t = n.get(1).ok_or("предопределенные: нет таблицы")?;
    // Длина кода — из описания колонки 3 `{4,"",{"Pattern",{"S",<длина>,..}},"",0}`.
    let code_len = t.get(1).and_then(|c| c.get(4)).and_then(|c| c.get(2)).and_then(|p| p.get(1)).and_then(|s| s.get(1)).and_then(Node::atom).and_then(|a| a.parse().ok()).unwrap_or(0);
    let root = t.get(2).and_then(rows).and_then(|r| r.first()).ok_or("предопределенные: нет корня")?;
    let Some(kids) = children(root)? else { return Ok(None) };
    let mut o = head("PlanOfCharacteristicKindPredefinedItems");
    for r in kids {
        cct_row(r, 1, code_len, types, &mut o)?;
    }
    o.push_str("</PredefinedData>");
    Ok(Some(o))
}

fn cct_row(r: &Node, depth: usize, code_len: usize, types: &std::collections::HashMap<String, String>, o: &mut String) -> Result<(), String> {
    let t = "\t".repeat(depth);
    let v = |i: usize| r.get(3 + i).ok_or_else(|| format!("предопределенные: нет колонки {i}"));
    let s = |i: usize| -> Result<&str, String> { v(i)?.get(1).and_then(|x| x.str().or(x.atom())).ok_or_else(|| format!("предопределенные: колонка {i}")) };
    let nc: usize = r.get(2).and_then(Node::atom).and_then(|a| a.parse().ok()).ok_or("предопределенные: нет числа колонок")?;
    for i in 6..nc {
        if crate::brace::write(v(i)?)? != "{\"N\",0}" {
            return Err(format!("предопределенные: колонка {i} = {}", crate::brace::write(v(i)?)?));
        }
    }
    let id = v(0)?.get(2).and_then(|x| x.get(1)).and_then(Node::atom).ok_or("предопределенные: нет ссылки")?;
    o.push_str(&format!("{t}<Item id=\"{id}\">\r\n{t}\t<Name>{}</Name>\r\n", esc(s(2)?)));
    if code_len > 0 && s(3)?.chars().count() > code_len {
        return Err(format!("предопределенные: код {:?} длиннее {code_len}", s(3)?));
    }
    code_desc(v(3)?, s(4)?, 0, &t, o)?;
    value_type(v(5)?, depth + 2, types, o)?;
    o.push_str(&format!("{t}\t<IsFolder>{}</IsFolder>\r\n", s(1)? == "1"));
    if let Some(kids) = children(r)? {
        o.push_str(&format!("{t}\t<ChildItems>\r\n"));
        for k in kids {
            cct_row(k, depth + 2, code_len, types, o)?;
        }
        o.push_str(&format!("{t}\t</ChildItems>\r\n"));
    }
    o.push_str(&format!("{t}</Item>\r\n"));
    Ok(())
}

const TYPE_DESC: &str = "f5c65050-3bbb-11d5-b988-0050bae0a95d";

/// `<Type>` как сериализует XDTO: ссылочный тип — с объявлением `xmlns:d<уровень>p1` на самом элементе
/// (уровень — глубина `<v8:Type>` с 1 у корня), затем квалификаторы числа, строки, даты.
/// Незнакомый тип — ошибка: угадывать нельзя.
fn value_type(n: &Node, tabs: usize, types: &std::collections::HashMap<String, String>, o: &mut String) -> Result<(), String> {
    let raw = crate::brace::write(n)?;
    let bad = || format!("предопределенные: тип {raw}");
    let (Some(TYPE_DESC), Some(p)) = (n.get(1).and_then(Node::atom), n.get(2)) else { return Err(bad()) };
    if n.get(0).and_then(Node::str) != Some("#") || n.list().len() != 3 || p.get(0).and_then(Node::str) != Some("Pattern") {
        return Err(bad());
    }
    let items = &p.list()[1..];
    let t = "\t".repeat(tabs - 1);
    if items.is_empty() {
        o.push_str(&format!("{t}<Type/>\r\n"));
        return Ok(());
    }
    let ti = "\t".repeat(tabs);
    let (mut num, mut st, mut date) = (None, None, None);
    o.push_str(&format!("{t}<Type>\r\n"));
    for it in items {
        let l = it.list();
        let a = |i: usize| l.get(i).and_then(|x| x.atom().or(x.str()));
        let ty = match (a(0), l.len()) {
            (Some("B"), 1) => "xs:boolean".to_string(),
            (Some("S"), 1 | 3) => {
                st = Some((a(1).unwrap_or("0"), if a(2) == Some("0") { "Fixed" } else { "Variable" }));
                "xs:string".into()
            }
            (Some("N"), 4) => {
                num = Some((a(1).unwrap_or(""), a(2).unwrap_or(""), if a(3) == Some("1") { "Nonnegative" } else { "Any" }));
                "xs:decimal".into()
            }
            (Some("D"), 1 | 2) => {
                date = Some(match a(1) {
                    Some("D") => "Date",
                    Some("T") => "Time",
                    None => "DateTime",
                    _ => return Err(bad()),
                });
                "xs:dateTime".into()
            }
            (Some("#"), 2) => {
                let name = a(1).and_then(|g| types.get(g)).filter(|t| t.contains("Ref.")).ok_or_else(bad)?;
                o.push_str(&format!("{ti}<v8:Type xmlns:d{0}p1=\"http://v8.1c.ru/8.1/data/enterprise/current-config\">d{0}p1:{name}</v8:Type>\r\n", tabs + 1));
                continue;
            }
            _ => return Err(bad()),
        };
        o.push_str(&format!("{ti}<v8:Type>{ty}</v8:Type>\r\n"));
    }
    if let Some((d, f, s)) = num {
        o.push_str(&format!("{ti}<v8:NumberQualifiers>\r\n{ti}\t<v8:Digits>{d}</v8:Digits>\r\n{ti}\t<v8:FractionDigits>{f}</v8:FractionDigits>\r\n{ti}\t<v8:AllowedSign>{s}</v8:AllowedSign>\r\n{ti}</v8:NumberQualifiers>\r\n"));
    }
    if let Some((len, al)) = st {
        o.push_str(&format!("{ti}<v8:StringQualifiers>\r\n{ti}\t<v8:Length>{len}</v8:Length>\r\n{ti}\t<v8:AllowedLength>{al}</v8:AllowedLength>\r\n{ti}</v8:StringQualifiers>\r\n"));
    }
    if let Some(f) = date {
        o.push_str(&format!("{ti}<v8:DateQualifiers>\r\n{ti}\t<v8:DateFractions>{f}</v8:DateFractions>\r\n{ti}</v8:DateQualifiers>\r\n"));
    }
    o.push_str(&format!("{t}</Type>\r\n"));
    Ok(())
}

/// Колонки таблицы: номер колонки → (позиция значения в строке, имя колонки — GUID признака учета или "").
/// Описания `{N,{<номер>,"<имя>",{"Pattern",...},"",0}...}`, в таблице `{2,N,(<позиция>,<номер>)*N,...}`.
fn columns<'a>(defs: &'a Node, table: &Node) -> Result<Vec<(&'a str, &'a str, usize)>, String> {
    let bad = || "предопределенные: колонки".to_string();
    let n: usize = defs.get(0).and_then(Node::atom).and_then(|a| a.parse().ok()).ok_or_else(bad)?;
    (0..n).map(|i| {
        let d = defs.get(1 + i).ok_or_else(bad)?;
        let (id, name) = (d.get(0).and_then(Node::atom).ok_or_else(bad)?, d.get(1).and_then(Node::str).ok_or_else(bad)?);
        let pos = (0..n).find(|j| table.get(3 + 2 * j).and_then(Node::atom) == Some(id)).and_then(|j| table.get(2 + 2 * j)?.atom()?.parse().ok()).ok_or_else(bad)?;
        Ok((id, name, pos))
    }).collect()
}

/// Предопределенные счета (запись `<id>.9`): `{2,{1,{N,<колонки>...},<таблица>}}`. Колонки по номерам: 0 ссылка, 1 имя,
/// 2 код, 3 наименование, 4 вид счета (0 Active, 1 Passive, 2 ActivePassive), 5 забалансовый, 6 виды субконто
/// (вложенная таблица: 0 вид — предопределенный ПВХ, 1 только обороты, дальше признаки учета субконто),
/// 10000 порядок, 20000 — `{"N",0}` (что это — неизвестно; другое значение не выгружаем); колонки с GUID в имени —
/// признаки учета. `flag(guid)` — (полное имя признака, порядковый номер в описании), `kind(id)` — полное имя вида субконто.
pub fn accounts(n: &Node, flag: &dyn Fn(&str) -> Option<(String, usize)>, kind: &dyn Fn(&str) -> Option<String>) -> Result<Option<String>, String> {
    let t = n.get(1).ok_or("предопределенные: нет таблицы")?;
    let (defs, table) = (t.get(1).ok_or("предопределенные: нет колонок")?, t.get(2).ok_or("предопределенные: нет таблицы")?);
    let root = rows(table).and_then(|r| r.first()).ok_or("предопределенные: нет корня")?;
    let Some(kids) = children(root)? else { return Ok(None) };
    let cols = columns(defs, table)?;
    let mut o = head("ChartOfAccountsPredefinedItems");
    for r in kids {
        account(r, 1, &cols, flag, kind, &mut o)?;
    }
    o.push_str("</PredefinedData>");
    Ok(Some(o))
}

/// Признаки учета в порядке колонок; порядок колонок должен совпадать с порядком в описании объекта.
fn flags<'a>(cols: &[(&str, &'a str, usize)], flag: &dyn Fn(&str) -> Option<(String, usize)>) -> Result<Vec<(String, usize)>, String> {
    let f: Vec<(String, usize, usize)> = cols.iter().filter(|c| !c.1.is_empty()).map(|c| flag(c.1).map(|(n, k)| (n, k, c.2)).ok_or_else(|| format!("предопределенные: признак учета {}", c.1))).collect::<Result<_, _>>()?;
    if f.windows(2).any(|w| w[0].1 > w[1].1) {
        return Err("предопределенные: порядок признаков учета в таблице не как в описании".into());
    }
    Ok(f.into_iter().map(|(n, _, p)| (n, p)).collect())
}

fn account(r: &Node, depth: usize, cols: &[(&str, &str, usize)], flag: &dyn Fn(&str) -> Option<(String, usize)>, kind: &dyn Fn(&str) -> Option<String>, o: &mut String) -> Result<(), String> {
    let t = "\t".repeat(depth);
    let at = |c: &str| cols.iter().find(|x| x.0 == c).and_then(|x| r.get(3 + x.2)).ok_or_else(|| format!("предопределенные: нет колонки {c}"));
    let s = |c: &str| -> Result<&str, String> { at(c)?.get(1).and_then(|x| x.str().or(x.atom())).ok_or_else(|| format!("предопределенные: колонка {c}")) };
    let b = |v: &Node| match crate::brace::write(v) {
        Ok(x) if x == "{\"B\",0}" => Ok(false),
        Ok(x) if x == "{\"B\",1}" => Ok(true),
        x => Err(format!("предопределенные: не булево {x:?}")),
    };
    let known = ["0", "1", "2", "3", "4", "5", "6", "10000", "20000"];
    if let Some(c) = cols.iter().find(|c| c.1.is_empty() && !known.contains(&c.0)) {
        return Err(format!("предопределенные: незнакомая колонка {}", c.0));
    }
    // у старых записей колонки 20000 нет (БП 3.0.107)
    if let Some(v) = cols.iter().any(|x| x.0 == "20000").then(|| at("20000")).transpose()? {
        if crate::brace::write(v)? != "{\"N\",0}" {
            return Err(format!("предопределенные: колонка 20000 = {}", crate::brace::write(v)?));
        }
    }
    let id = at("0")?.get(2).and_then(|x| x.get(1)).and_then(Node::atom).ok_or("предопределенные: нет ссылки")?;
    o.push_str(&format!("{t}<Item id=\"{id}\">\r\n{t}\t<Name>{}</Name>\r\n", esc(s("1")?)));
    code_desc(at("2")?, s("3")?, 0, &t, o)?;
    let ty = match s("4")? {
        "0" => "Active",
        "1" => "Passive",
        "2" => "ActivePassive",
        x => return Err(format!("предопределенные: вид счета {x}")),
    };
    o.push_str(&format!("{t}\t<AccountType>{ty}</AccountType>\r\n{t}\t<OffBalance>{}</OffBalance>\r\n", b(at("5")?)?));
    let ord = s("10000")?;
    o.push_str(&if ord.is_empty() { format!("{t}\t<Order/>\r\n") } else { format!("{t}\t<Order>{}</Order>\r\n", esc(ord)) });
    o.push_str(&format!("{t}\t<AccountingFlags>\r\n"));
    for (name, p) in flags(cols, flag)? {
        o.push_str(&format!("{t}\t\t<Flag ref=\"{name}\">{}</Flag>\r\n", b(r.get(3 + p).ok_or("предопределенные: нет признака")?)?));
    }
    o.push_str(&format!("{t}\t</AccountingFlags>\r\n"));
    // Виды субконто `{"#",acf6192e-…,{9,{N,<колонки>},<таблица>,{0,0}}}`.
    let ed = at("6")?;
    let inner = ed.get(2).filter(|_| ed.get(1).and_then(Node::atom) == Some(SUBTABLE) && ed.list().len() == 3).ok_or("предопределенные: виды субконто")?;
    if inner.list().len() != 4 || crate::brace::write(&inner.list()[3])? != "{0,0}" {
        return Err("предопределенные: виды субконто".into());
    }
    let (edefs, etable) = (inner.get(1).ok_or("предопределенные: виды субконто")?, inner.get(2).ok_or("предопределенные: виды субконто")?);
    let ecols = columns(edefs, etable)?;
    if let Some(c) = ecols.iter().find(|c| c.1.is_empty() && !["0", "1"].contains(&c.0)) {
        return Err(format!("предопределенные: незнакомая колонка субконто {}", c.0));
    }
    let erows = rows(etable).ok_or("предопределенные: виды субконто")?;
    if erows.is_empty() {
        o.push_str(&format!("{t}\t<ExtDimensionTypes/>\r\n"));
    } else {
        let eflags = flags(&ecols, flag)?;
        let pos = |c: &str| ecols.iter().find(|x| x.0 == c).map(|x| x.2).ok_or("предопределенные: колонка субконто");
        o.push_str(&format!("{t}\t<ExtDimensionTypes>\r\n"));
        for er in erows {
            let v = |p: usize| er.get(3 + p).ok_or("предопределенные: строка субконто");
            // `{2,<номер>,<колонок>,...,0}`: без вложенных строк
            if er.list().last().and_then(Node::atom) != Some("0") {
                return Err("предопределенные: строка субконто".into());
            }
            let kid = v(pos("0")?)?.get(2).and_then(|x| x.get(1)).and_then(Node::atom).ok_or("предопределенные: вид субконто")?;
            let kname = kind(kid).ok_or_else(|| format!("предопределенные: вид субконто {kid}"))?;
            o.push_str(&format!("{t}\t\t<ExtDimensionType name=\"{}\">\r\n{t}\t\t\t<Turnover>{}</Turnover>\r\n{t}\t\t\t<AccountingFlags>\r\n", esc(&kname), b(v(pos("1")?)?)?));
            for (name, p) in &eflags {
                o.push_str(&format!("{t}\t\t\t\t<Flag ref=\"{name}\">{}</Flag>\r\n", b(v(*p)?)?));
            }
            o.push_str(&format!("{t}\t\t\t</AccountingFlags>\r\n{t}\t\t</ExtDimensionType>\r\n"));
        }
        o.push_str(&format!("{t}\t</ExtDimensionTypes>\r\n"));
    }
    if let Some(kids) = children(r)? {
        o.push_str(&format!("{t}\t<ChildItems>\r\n"));
        for k in kids {
            account(k, depth + 2, cols, flag, kind, o)?;
        }
        o.push_str(&format!("{t}\t</ChildItems>\r\n"));
    }
    o.push_str(&format!("{t}</Item>\r\n"));
    Ok(())
}

const SUBTABLE: &str = "acf6192e-81ca-46ef-93a6-5a6968b78663";

/// Предопределенные виды расчета. Непустые вложенные таблицы (базовые, ведущие, вытесняющие) в эталоне
/// не встречались — пишутся как есть, чтобы расхождение было видно.
pub fn calc_types(n: &Node) -> Result<Option<String>, String> {
    let items = n.get(2).and_then(rows).ok_or("предопределенные: нет таблицы")?;
    if items.is_empty() {
        return Ok(None);
    }
    let mut o = head("CalculationTypePredefinedItems");
    for r in items {
        let v = |i: usize| r.get(3 + i).ok_or_else(|| format!("предопределенные: нет колонки {i}"));
        let s = |i: usize| -> Result<&str, String> { v(i)?.get(1).and_then(|x| x.str().or(x.atom())).ok_or_else(|| format!("предопределенные: колонка {i}")) };
        let id = v(0)?.get(2).and_then(|x| x.get(1)).and_then(Node::atom).ok_or("предопределенные: нет ссылки")?;
        o.push_str(&format!("\t<Item id=\"{id}\">\r\n\t\t<Name>{}</Name>\r\n", esc(s(1)?)));
        code_desc(v(2)?, s(3)?, 0, "\t", &mut o)?;
        o.push_str(&format!("\t\t<ActionPeriodIsBase>{}</ActionPeriodIsBase>\r\n", s(4)? == "1"));
        for i in 5..8 {
            let t = v(i)?.get(2).and_then(|x| x.get(2));
            if !t.and_then(rows).is_some_and(<[Node]>::is_empty) {
                o.push_str(&format!("\t\t<Unknown>{}{}</Unknown>\r\n", super::UNK, esc(&crate::brace::write(v(i)?)?)));
            }
        }
        o.push_str("\t</Item>\r\n");
    }
    o.push_str("</PredefinedData>");
    Ok(Some(o))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::brace;

    #[test]
    fn nested_items_and_code_cut() {
        let r = |n: u32, id: &str, folder: u8, name: &str, code: &str, kids: &str| {
            format!("{{2,{n},6,{{\"#\",ae135932-4f94-44df-92c1-c91f15a92848,{{1,{id}}}}},{{\"B\",{folder}}},{{\"#\",ae135932-4f94-44df-92c1-c91f15a92848,{{1,00000000-0000-0000-0000-000000000000}}}},{{\"S\",\"{name}\"}},{{\"S\",\"{code}\"}},{{\"S\",\"\"}},{kids}}}")
        };
        let leaf = r(2, "22222222-2222-2222-2222-222222222222", 0, "Б", "0000000001", "0");
        let grp = r(1, "11111111-1111-1111-1111-111111111111", 1, "А<", "", &format!("1,{{1,1,{leaf}}}"));
        let root = r(0, "00000000-0000-0000-0000-000000000000", 1, "Элементы", "", &format!("1,{{1,1,{grp}}}"));
        let t = format!("{{0,{{1,{{6}},{{2,6,0,0,{{1,1,{root}}},-1,2}}}}}}");
        let x = catalog(&brace::parse(&t).unwrap(), 9).unwrap().unwrap();
        assert!(x.contains("\t<Item id=\"11111111-1111-1111-1111-111111111111\">\r\n\t\t<Name>А&lt;</Name>\r\n\t\t<Code/>\r\n\t\t<Description/>\r\n\t\t<IsFolder>true</IsFolder>\r\n\t\t<ChildItems>\r\n\t\t\t<Item id=\"2222"), "{x}");
        assert!(x.contains("\t\t\t\t<Code>000000000</Code>\r\n"), "{x}");
        let empty = format!("{{0,{{1,{{6}},{{2,6,0,0,{{1,1,{}}},-1,0}}}}}}", r(0, "00000000-0000-0000-0000-000000000000", 1, "Элементы", "", "0"));
        assert_eq!(catalog(&brace::parse(&empty).unwrap(), 9).unwrap(), None);
    }

    #[test]
    fn characteristic_type_item() {
        let p = "ae135932-4f94-44df-92c1-c91f15a92848";
        let td = "f5c65050-3bbb-11d5-b988-0050bae0a95d";
        let leaf = format!("{{2,6,7,{{\"#\",{p},{{1,942d99dc-3fa8-478d-8e93-6d1c26536c6c}}}},{{\"B\",0}},{{\"S\",\"Х\"}},{{\"S\",\"\"}},{{\"S\",\"Д\"}},{{\"#\",{td},{{\"Pattern\",{{\"#\",da33c2df-80df-4be5-9156-04dbaf8af8f1}},{{\"N\",2,0,0}}}}}},{{\"N\",0}},0}}");
        let root = format!("{{2,0,6,{{\"#\",{p},{{1,00000000-0000-0000-0000-000000000000}}}},{{\"B\",1}},{{\"S\",\"Характеристики\"}},{{\"S\",\"\"}},{{\"S\",\"\"}},{{\"#\",{td},{{\"Pattern\"}}}},1,{{1,1,{leaf}}}}}");
        let t = format!("{{1,{{1,{{7,{{1}},{{2}},{{3}},{{4,\"\",{{\"Pattern\",{{\"S\",5,0}}}},\"\",0}}}},{{2,7,0,{{1,1,{root}}},-1,6}}}}}}");
        let types = [("da33c2df-80df-4be5-9156-04dbaf8af8f1".to_string(), "CatalogRef.Р".to_string())].into();
        let x = characteristic_types(&brace::parse(&t).unwrap(), &types).unwrap().unwrap();
        assert!(x.contains("\t\t<Description>Д</Description>\r\n\t\t<Type>\r\n\t\t\t<v8:Type xmlns:d4p1=\"http://v8.1c.ru/8.1/data/enterprise/current-config\">d4p1:CatalogRef.Р</v8:Type>\r\n\t\t\t<v8:Type>xs:decimal</v8:Type>\r\n\t\t\t<v8:NumberQualifiers>\r\n\t\t\t\t<v8:Digits>2</v8:Digits>\r\n\t\t\t\t<v8:FractionDigits>0</v8:FractionDigits>\r\n\t\t\t\t<v8:AllowedSign>Any</v8:AllowedSign>\r\n\t\t\t</v8:NumberQualifiers>\r\n\t\t</Type>\r\n\t\t<IsFolder>false</IsFolder>\r\n\t</Item>\r\n</PredefinedData>"), "{x}");
        // незнакомый тип и непустая дополнительная колонка — отказ
        assert!(characteristic_types(&brace::parse(&t).unwrap(), &Default::default()).is_err());
        assert!(characteristic_types(&brace::parse(&t.replace("{\"N\",0},0}", "{\"N\",1},0}")).unwrap(), &types).is_err());
    }

    #[test]
    fn account_item() {
        let p = "{\"#\",ae135932-4f94-44df-92c1-c91f15a92848,{1,";
        let def = |i: &str, n: &str| format!("{{{i},\"{n}\",{{\"Pattern\"}},\"\",0}}");
        let sub = |rows: &str| format!("{{\"#\",acf6192e-81ca-46ef-93a6-5a6968b78663,{{9,{{3,{},{},{}}},{{2,3,0,0,1,1,2,2,{{1,{rows}}},-1,0}},{{0,0}}}}}}", def("0", ""), def("1", ""), def("2", "e"));
        let acc = |id: &str, name: &str, kind: &str, subs: &str, fl: u8, tail: &str| format!("{{2,1,10,{p}{id}}}}},{{\"S\",\"{name}\"}},{{\"S\",\"01\"}},{{\"S\",\"\"}},{{\"N\",{kind}}},{{\"B\",0}},{},{{\"S\",\" 01\"}},{{\"B\",{fl}}},{{\"N\",0}},{tail}}}", sub(subs));
        let leaf = acc("11111111-1111-1111-1111-111111111111", "ОС", "1", &format!("1,{{2,0,3,{p}22222222-2222-2222-2222-222222222222}}}},{{\"B\",1}},{{\"B\",0}},0}}"), 1, "0");
        let root = acc("00000000-0000-0000-0000-000000000000", "Счета", "0", "0", 0, &format!("1,{{1,1,{leaf}}}"));
        let defs = ["0", "1", "2", "3", "4", "5", "6"].map(|i| def(i, "")).join(",") + "," + &def("10000", "") + "," + &def("9", "f") + "," + &def("20000", "");
        let t = format!("{{2,{{1,{{10,{defs}}},{{2,10,0,0,1,1,2,2,3,3,4,4,5,5,6,6,7,10000,8,9,9,20000,{{1,1,{root}}},-1,1}}}}}}");
        let flag = |g: &str| match g {
            "f" => Some(("ChartOfAccounts.Х.AccountingFlag.Валютный".to_string(), 1)),
            "e" => Some(("ChartOfAccounts.Х.ExtDimensionAccountingFlag.Суммовой".to_string(), 2)),
            _ => None,
        };
        let kind = |id: &str| (id == "22222222-2222-2222-2222-222222222222").then(|| "ChartOfCharacteristicTypes.В.ОС".to_string());
        let x = accounts(&brace::parse(&t).unwrap(), &flag, &kind).unwrap().unwrap();
        assert!(x.contains("\t\t<Name>ОС</Name>\r\n\t\t<Code>01</Code>\r\n\t\t<Description/>\r\n\t\t<AccountType>Passive</AccountType>\r\n\t\t<OffBalance>false</OffBalance>\r\n\t\t<Order> 01</Order>\r\n\t\t<AccountingFlags>\r\n\t\t\t<Flag ref=\"ChartOfAccounts.Х.AccountingFlag.Валютный\">true</Flag>\r\n\t\t</AccountingFlags>\r\n\t\t<ExtDimensionTypes>\r\n\t\t\t<ExtDimensionType name=\"ChartOfCharacteristicTypes.В.ОС\">\r\n\t\t\t\t<Turnover>true</Turnover>\r\n\t\t\t\t<AccountingFlags>\r\n\t\t\t\t\t<Flag ref=\"ChartOfAccounts.Х.ExtDimensionAccountingFlag.Суммовой\">false</Flag>\r\n"), "{x}");
        assert!(accounts(&brace::parse(&t).unwrap(), &flag, &|_| None).is_err());
    }

    #[test]
    fn calc_type_item() {
        let sub = "{\"#\",acf6192e-81ca-46ef-93a6-5a6968b78663,{8,{1,{1,\"\",{\"Pattern\"},\"\",0}},{2,1,0,1,{1,0},-1,139}}}";
        let t = format!("{{8,{{8}},{{2,8,0,1,{{1,1,{{2,0,8,{{\"#\",ae135932-4f94-44df-92c1-c91f15a92848,{{1,35c7a5dd-d8e9-4aa0-8378-2a7e501d846a}}}},{{\"S\",\"Оклад\"}},{{\"S\",\"00001\"}},{{\"S\",\"\"}},{{\"B\",0}},{sub},{sub},{sub},0}}}},-1,33}}}}");
        let x = calc_types(&brace::parse(&t).unwrap()).unwrap().unwrap();
        assert!(x.contains("<Item id=\"35c7a5dd-d8e9-4aa0-8378-2a7e501d846a\">\r\n\t\t<Name>Оклад</Name>\r\n\t\t<Code>00001</Code>\r\n\t\t<Description/>\r\n\t\t<ActionPeriodIsBase>false</ActionPeriodIsBase>\r\n\t</Item>\r\n</PredefinedData>"), "{x}");
    }
}
