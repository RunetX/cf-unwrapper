//! Блок свойств реквизита `{27,{2,<заголовок>,<типы>},…}` (обратное `xml/attr.rs`) и значения `MinValue`, `FillValue`…
//! Свойства, которых у класса в XML нет (у константы — FillValue, FillFromFillingValue, CreateOnInput), платформа
//! пишет значением по умолчанию — оно взято из записей, где свойства нет.

use super::dom::El;
use super::index::Index;
use super::tpl::{a, crlf, l, s, Props};
use crate::brace::{is_guid, Node};
use crate::xml::common::ZERO;

const DESIGN_TIME_REF: &str = "5c14e26f-099b-4d37-84a6-b433d87400da";
const FIXED_ARRAY: &str = "4500381b-db30-4a10-9db4-990038032acf";

/// Блок `{27,…}` по свойствам `p` (заголовок — `p.header()`); `owner` — полное имя объекта для путей связей.
pub fn block(p: &Props, owner: &str) -> Result<Node, String> {
    let opt = |tag: &str, f: &dyn Fn(&El) -> Result<Node, String>, def: Node| -> Result<Node, String> {
        match p.el.child(tag) {
            Some(e) => f(e),
            None => Ok(def),
        }
    };
    let b = |tag: &str| opt(tag, &super::tpl::boolean, a("0"));
    let m = |tag: &str| opt(tag, &super::tpl::mlstring, l(vec![a("0")]));
    let t = |tag: &str, map: &[(&str, &str)], def: &str| opt(tag, &|e| super::tpl::token(e, map), a(def));
    let v = |tag: &str, def: Node| opt(tag, &|e| value(p.x, e), def);
    let u = || l(vec![s("U")]);
    Ok(l(vec![
        a("27"),
        l(vec![a("2"), p.header()?, p.types("Type")?]),
        b("PasswordMode")?,
        m("Format")?,
        m("ToolTip")?,
        b("MarkNegatives")?,
        opt("Mask", &|e| Ok(s(crlf(&e.text()?))), s(""))?,
        b("MultiLine")?,
        v("MinValue", u())?,
        v("MaxValue", u())?,
        t("ChoiceFoldersAndItems", &[("0", "Items"), ("1", "Folders"), ("2", "FoldersAndItems")], "0")?,
        opt("ChoiceForm", &|e| super::tpl::reference(p.x, e), a(ZERO))?,
        t("QuickChoice", &[("0", "DontUse"), ("1", "Use"), ("2", "Auto")], "2")?,
        t("FillChecking", &[("0", "DontCheck"), ("1", "ShowError")], "0")?,
        opt("ChoiceParameterLinks", &|e| links(p.x, owner, e), l(vec![a("5006"), a("0")]))?,
        opt("LinkByType", &|e| link_by_type(p.x, owner, e), l(vec![a("3"), a("0"), a("0")]))?,
        opt("ChoiceParameters", &|e| choice_parameters(p.x, e), l(vec![a("0"), a("0")]))?,
        b("ExtendedEdit")?,
        m("EditFormat")?,
        v("FillValue", l(vec![s("S"), s("")]))?,
        b("FillFromFillingValue")?,
        t("CreateOnInput", &[("0", "Auto"), ("1", "DontUse"), ("2", "Use")], "0")?,
        t("ChoiceHistoryOnInput", &[("0", "Auto"), ("1", "DontUse")], "0")?,
    ]))
}

/// Путь к полю (обратное `data_path` выгрузки): `0:<uuid>/0:<uuid>` — как есть; `<владелец>.StandardAttribute.<Имя>` —
/// `{-код}`; поле владельца — `{0,<uuid>}` по частям (`TabularSection.Т.Attribute.А` — табличная часть и реквизит).
pub fn path(x: &Index, owner: &str, dp: &str, at: &El) -> Result<Vec<Node>, String> {
    let err = || format!("строка {}: путь {dp:?}", at.line);
    // код стандартного реквизита без имени (у обработки платформа так пишет `-8`)
    if dp.starts_with('-') && super::tpl::int(dp).is_some() {
        return Ok(vec![l(vec![a(dp)])]);
    }
    if dp.starts_with("0:") {
        return dp.split('/').map(|p| p.strip_prefix("0:").filter(|g| is_guid(g)).map(|g| l(vec![a("0"), a(g)])).ok_or_else(err)).collect();
    }
    // константа из любого владельца — `{0,<uuid>}` (обратное `data_path` выгрузки)
    if dp.strip_prefix("Constant.").is_some_and(|c| !c.contains('.')) && !dp.starts_with(&format!("{owner}.")) {
        return Ok(vec![l(vec![a("0"), a(x.id(dp).ok_or_else(err)?)])]);
    }
    let rest = dp.strip_prefix(owner).and_then(|r| r.strip_prefix('.')).ok_or_else(err)?;
    if let Some(name) = rest.strip_prefix("StandardAttribute.") {
        let class = owner.split('.').next().unwrap_or_default();
        let names = crate::xml::standard::NAMES.iter().find(|(c, _)| *c == class).map(|(_, n)| *n).unwrap_or(&[]);
        let code = names.iter().find(|(_, n)| *n == name).map(|(c, _)| *c).ok_or_else(err)?;
        return Ok(vec![l(vec![a(code)])]);
    }
    let parts: Vec<&str> = rest.split('.').collect();
    if parts.len() % 2 != 0 {
        return Err(err());
    }
    let mut full = owner.to_string();
    let mut v = Vec::new();
    for c in parts.chunks(2) {
        full = format!("{full}.{}.{}", c[0], c[1]);
        v.push(l(vec![a("0"), a(x.id(&full).ok_or_else(err)?)]));
    }
    Ok(v)
}

/// Связи параметров выбора `{5006,N,"имя",K,<путь>…,<0 Clear | 1 DontChange>,…}`.
pub fn links(x: &Index, owner: &str, e: &El) -> Result<Node, String> {
    let mut v = vec![a("5006"), a("0")];
    let mut n = 0;
    for k in e.elems() {
        if k.name != "xr:Link" {
            return Err(format!("строка {}: <{}> в связях", k.line, k.name));
        }
        let dp = k.req("xr:DataPath")?;
        if dp.req_attr("xsi:type")? != "xs:string" {
            return Err(format!("строка {}: DataPath не строка", dp.line));
        }
        let p = path(x, owner, &dp.text()?, dp)?;
        v.push(s(k.text_of("xr:Name")?));
        v.push(a(p.len().to_string()));
        v.extend(p);
        v.push(super::tpl::token(k.req("xr:ValueChange")?, &[("0", "Clear"), ("1", "DontChange")])?);
        n += 1;
    }
    if n == 0 {
        e.text()?;
    }
    v[1] = a(n.to_string());
    Ok(l(v))
}

/// Связь по типу `{3,K,<путь>…,<элемент связи>}`; пусто — `{3,0,0}`.
pub fn link_by_type(x: &Index, owner: &str, e: &El) -> Result<Node, String> {
    if e.elems().next().is_none() {
        e.text()?;
        return Ok(l(vec![a("3"), a("0"), a("0")]));
    }
    let dp = e.req("xr:DataPath")?;
    let p = path(x, owner, &dp.text()?, dp)?;
    let item = e.text_of("xr:LinkItem")?;
    let mut v = vec![a("3"), a(p.len().to_string())];
    v.extend(p);
    v.push(a(super::tpl::int(&item).ok_or(format!("строка {}: LinkItem {item:?}", e.line))?));
    Ok(l(v))
}

/// Параметры выбора `{0,N,"имя",<значение>,…}`.
pub fn choice_parameters(x: &Index, e: &El) -> Result<Node, String> {
    let mut v = vec![a("0"), a("0")];
    let mut n = 0;
    for it in e.elems() {
        if it.name != "app:item" {
            return Err(format!("строка {}: <{}> в параметрах выбора", it.line, it.name));
        }
        v.push(s(it.req_attr("name")?));
        v.push(value(x, it.req("app:value")?)?);
        n += 1;
    }
    if n == 0 {
        e.text()?;
    }
    v[1] = a(n.to_string());
    Ok(l(v))
}

/// Значение `<X xsi:type="…">…</X>` / `<X xsi:nil="true"/>` → `{"U"}`, `{"S",…}`, `{"N",…}`, `{"B",…}`, `{"D",…}`,
/// ссылка `{"#",5c14e26f-…,{0,<TypeId>,<uuid>}}`, массив `{"#",4500381b-…,{N,<значения>}}`.
pub fn value(x: &Index, e: &El) -> Result<Node, String> {
    let at = || format!("строка {}: <{}>", e.line, e.name);
    if let Some(n) = e.attr("xsi:nil") {
        if n != "true" || e.attrs.len() != 1 || !e.text()?.is_empty() {
            return Err(format!("{}: xsi:nil={n}", at()));
        }
        return Ok(l(vec![s("U")]));
    }
    let ty = e.attr("xsi:type").ok_or(format!("{}: нет xsi:type", at()))?;
    if e.attrs.len() != 1 {
        return Err(format!("{}: лишние атрибуты", at()));
    }
    if ty == "v8:FixedArray" {
        let mut v = vec![a("0")];
        for c in e.elems() {
            if c.name != "v8:Value" {
                return Err(format!("строка {}: <{}> в массиве", c.line, c.name));
            }
            v.push(value(x, c)?);
        }
        v[0] = a((v.len() - 1).to_string());
        return Ok(l(vec![s("#"), a(FIXED_ARRAY), l(v)]));
    }
    let t = e.text()?;
    Ok(match ty {
        "xs:string" => l(vec![s("S"), s(crlf(&t))]),
        "xs:boolean" => l(vec![s("B"), a(match t.as_str() {
            "true" => "1",
            "false" => "0",
            _ => return Err(format!("{}: булево {t:?}", at())),
        })]),
        "xs:decimal" if is_number(&t) => l(vec![s("N"), a(t)]),
        "xs:dateTime" => {
            let d: String = t.chars().filter(|c| c.is_ascii_digit()).collect();
            let shape: String = t.chars().map(|c| if c.is_ascii_digit() { '9' } else { c }).collect();
            if shape != "9999-99-99T99:99:99" {
                return Err(format!("{}: дата {t:?}", at()));
            }
            l(vec![s("D"), a(d)])
        }
        // вид счета: в БП встречен только ActivePassive (2)
        "ent:AccountType" if t == "ActivePassive" => l(vec![s("#"), a("872f7198-7083-4e3e-b57e-a2a9802c769e"), a("2")]),
        // пустое описание типов
        "v8:TypeDescription" if t.is_empty() => l(vec![s("#"), a("f5c65050-3bbb-11d5-b988-0050bae0a95d"), l(vec![s("Pattern")])]),
        "xr:DesignTimeRef" => design_time_ref(x, &t).ok_or(format!("{}: незнакомая ссылка {t:?}", at()))?,
        _ => return Err(format!("{}: значение типа {ty} не загружается", at())),
    })
}

/// Число как пишет платформа: цифры, необязательные знак и дробная часть.
fn is_number(t: &str) -> bool {
    let d = t.strip_prefix('-').unwrap_or(t);
    let (i, f) = d.split_once('.').unwrap_or((d, ""));
    !i.is_empty() && i.bytes().all(|b| b.is_ascii_digit()) && f.bytes().all(|b| b.is_ascii_digit()) && (!d.contains('.') || !f.is_empty())
}

/// Ссылка периода разработки (обратное `common::value` выгрузки): `Catalog.Имя.EmptyRef`, `Enum.Имя.EnumValue.Значение`,
/// `Catalog.Имя.<предопределенный>`, `<TypeId>.<uuid>` (тип или значение не из конфигурации), пусто — оба нулевые.
pub(super) fn design_time_ref(x: &Index, t: &str) -> Option<Node> {
    let r = |ty: &str, id: &str| Some(l(vec![s("#"), a(DESIGN_TIME_REF), l(vec![a("0"), a(ty), a(id)])]));
    if t.is_empty() {
        return r(ZERO, ZERO);
    }
    if let Some((g1, g2)) = t.split_once('.').filter(|(g1, g2)| is_guid(g1) && is_guid(g2)) {
        return r(g1, g2);
    }
    let mut p = t.splitn(3, '.');
    let (class, name, rest) = (p.next()?, p.next()?, p.next()?);
    let ty = &x.types.get(&format!("{class}Ref.{name}"))?.0;
    let id = if rest == "EmptyRef" {
        ZERO.to_string()
    } else if rest.starts_with("EnumValue.") {
        x.id(t)?.to_string()
    } else {
        x.predefined.get(t)?.clone()
    };
    r(ty, &id)
}
