//! Цвета, шрифты, рамки (обратное `xml/ui.rs`) в записи загрузки платформы: цвет `{4,<вид>,{<v>},<вид>}`,
//! шрифт `{8,…}` (у абсолютного в конце `0`), рамка `{3,0,{0},<толщина>,<вид>,0,48312c09-…}`.

use super::dom::El;
use super::tpl::{a, l, s};
use crate::brace::Node;
use crate::xml::mxl::{STYLE as STYLE_COLORS, WEB, WINDOWS};

const COLOR: &str = "9cd510c7-abfc-11d4-9434-004095e12fc7";
const FONT: &str = "9cd510c8-abfc-11d4-9434-004095e12fc7";
const BORDER: &str = "4d10ca00-111a-4d43-9c96-92cd773716de";

fn back_num(t: &[(i64, &str)], name: &str) -> Option<i64> {
    t.iter().find(|(_, n)| *n == name).map(|(k, _)| *k)
}

/// Цвет `#RRGGBB`, `web:…`, `win:…`, `style:…` → `{4,<вид>,{<значение>},<вид>}`.
pub fn color(t: &str) -> Option<Node> {
    let (kind, v) = if let Some(h) = t.strip_prefix('#') {
        if h.len() != 6 || !h.bytes().all(|b| b.is_ascii_hexdigit() && !b.is_ascii_lowercase()) {
            return None;
        }
        let rgb = u32::from_str_radix(h, 16).ok()?;
        let bgr = (rgb & 0xff) << 16 | (rgb & 0xff00) | rgb >> 16;
        ("0", bgr.to_string())
    } else if let Some(n) = t.strip_prefix("web:") {
        ("2", back_num(WEB, n)?.to_string())
    } else if let Some(n) = t.strip_prefix("win:") {
        ("1", back_num(WINDOWS, n)?.to_string())
    } else if let Some(n) = t.strip_prefix("style:") {
        ("3", back_num(STYLE_COLORS, n)?.to_string())
    } else {
        return None;
    };
    Some(l(vec![a("4"), a(kind), l(vec![a(v)]), a(kind)]))
}

/// Шрифт из атрибутов `<… xsi:type="v8ui:Font" …>` (атрибуты, кроме xsi:type).
pub fn font(e: &El) -> Result<Node, String> {
    let at = |k: &str| e.attr(k);
    let err = |m: &str| format!("строка {}: шрифт: {m}", e.line);
    let flag = |k: &str| -> Result<Node, String> {
        match at(k) {
            Some("true") => Ok(a("1")),
            Some("false") => Ok(a("0")),
            v => Err(err(&format!("{k}={v:?}"))),
        }
    };
    let num = |v: &str| super::tpl::int(v).ok_or(err(v));
    let height = |v: &str| -> Result<String, String> { Ok((num(v)?.parse::<u32>().map_err(|_| err(v))? * 10).to_string()) };
    let bold = |v: &str| -> Result<Node, String> {
        match v {
            "true" => Ok(a("700")),
            "false" => Ok(a("400")),
            _ => Err(err(v)),
        }
    };
    match at("kind") {
        Some("Absolute") => {
            let face = at("faceName").ok_or(err("нет faceName"))?;
            let mut v = vec![a("8"), a("0"), a("575"), a(height(at("height").ok_or(err("нет height"))?)?), a("0"), a("0"), a("0")];
            v.push(bold(at("bold").unwrap_or_default())?);
            v.extend([flag("italic")?, flag("underline")?, flag("strikeout")?]);
            v.extend((0..5).map(|_| a("0")));
            v.extend([s(face), a("1"), a(num(at("scale").ok_or(err("нет scale"))?)?), a("0")]);
            Ok(l(v))
        }
        Some(kind @ ("StyleItem" | "WindowsFont")) => {
            let (k, r) = match (kind, at("ref")) {
                ("WindowsFont", Some("sys:DefaultGUIFont")) => ("1", "0".to_string()),
                ("StyleItem", Some(r)) => ("2", back_num(STYLE_COLORS, r.strip_prefix("style:").unwrap_or("?")).ok_or(err(r))?.to_string()),
                (_, r) => return Err(err(&format!("ref {r:?}"))),
            };
            let mut mask = 0;
            let mut fields = Vec::new();
            if let Some(h) = at("height") {
                mask |= 2;
                fields.push(a(height(h)?));
            }
            if let Some(b) = at("bold") {
                mask |= 4;
                fields.push(bold(b)?);
            }
            for (bit, k) in [(8, "italic"), (16, "underline"), (32, "strikeout")] {
                if at(k).is_some() {
                    mask |= bit;
                    fields.push(flag(k)?);
                }
            }
            let scale = match at("scale") {
                Some(sc) => {
                    mask |= 512;
                    num(sc)?
                }
                None => "100".into(),
            };
            let mut v = vec![a("8"), a(k), a(mask.to_string()), l(vec![a(r)])];
            v.extend(fields);
            v.extend([a("1"), a(scale)]);
            Ok(l(v))
        }
        k => Err(err(&format!("kind={k:?}"))),
    }
}

/// Значение элемента стиля `<Value xsi:type="v8ui:Color|Font|Border">` → `{"#",<тип>,<n>,{…},0}`.
pub fn style_value(e: &El) -> Result<Node, String> {
    let ty = e.req_attr("xsi:type")?;
    match ty {
        "v8ui:Color" => {
            let t = e.text()?;
            let c = color(&t).ok_or(format!("строка {}: цвет {t:?}", e.line))?;
            Ok(l(vec![s("#"), a(COLOR), a("2"), c]))
        }
        "v8ui:Font" => {
            e.text()?;
            Ok(l(vec![s("#"), a(FONT), a("1"), font(e)?, a("0")]))
        }
        "v8ui:Border" => {
            let w = super::tpl::int(e.req_attr("width")?).ok_or("рамка: толщина")?;
            let st = e.req("v8ui:style")?;
            // {3,0,{0},<вид>,<толщина>,…} — как у выгрузки (xml/ui.rs)
            let (ty, v) = (st.req_attr("xsi:type")?, st.text()?);
            let kind = match (ty.as_ref(), v.as_ref()) {
                ("v8ui:ControlBorderType", "WithoutBorder") => "0",
                ("v8ui:ControlBorderType", "Single") => "1",
                (_, v) => return Err(format!("строка {}: вид рамки {v} не загружается", st.line)),
            };
            Ok(l(vec![s("#"), a(BORDER), a("1"), l(vec![a("3"), a("0"), l(vec![a("0")]), a(kind), a(w), a("0"), a("48312c09-257f-4b29-b280-284dd89efc1e")]), a("0")]))
        }
        t => Err(format!("строка {}: значение стиля {t}", e.line)),
    }
}
