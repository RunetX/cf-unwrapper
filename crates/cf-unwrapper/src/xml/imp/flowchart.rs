//! Графическая схема: карта маршрута бизнес-процесса `Ext/Flowchart.xml` (запись `<id>.7`) и макет
//! `Ext/Template.xml` с корнем `<GraphicalSchema>` (запись `<id макета>.0`); обратное `xml/flowchart.rs`, `xml/graph.rs`.
//!
//! `{5,{{1,<фон>,<сетка>,<шаг>,<шаг>,<вид сетки>,K,(<код>,{"N",<v>})…}},N,(<вид>,<элемент>)…,<счетчик>}`.
//! Точек фигур в XML нет — платформа строит их по размеру; формулы сняты с карт БП и проверены только на
//! встреченных там размерах, другие размеры — отказ.

use super::dom::El;
use super::index::Index;
use super::tpl::{a, boolean, crlf, int, l, s};
use super::Rec;
use crate::brace::{Doc, Node};
use std::collections::HashMap;

const NS: &[(&str, &str)] = &[
    ("xmlns", "http://v8.1c.ru/8.3/xcf/scheme"),
    ("xmlns:pal", "http://v8.1c.ru/8.1/data/ui/colors/palette"),
    ("xmlns:sch", "http://v8.1c.ru/8.2/data/graphscheme"),
    ("xmlns:style", "http://v8.1c.ru/8.1/data/ui/style"),
    ("xmlns:v8", "http://v8.1c.ru/8.1/data/core"),
    ("xmlns:v8ui", "http://v8.1c.ru/8.1/data/ui"),
    ("xmlns:web", "http://v8.1c.ru/8.1/data/ui/colors/web"),
    ("xmlns:win", "http://v8.1c.ru/8.1/data/ui/colors/windows"),
    ("xmlns:xs", "http://www.w3.org/2001/XMLSchema"),
    ("xmlns:xsi", "http://www.w3.org/2001/XMLSchema-instance"),
    ("version", "2.21"),
];

const LINE_KIND: &str = "e45c0cd8-a878-4bcb-8e1a-af934481e1cc";
const READABLE: &str = "http://v8.1c.ru/8.3/xcf/readable";

/// Цвета стиля в схеме (`xml/graph.rs`: проверены на 42 схемах БП).

fn err(e: &El, m: &str) -> String {
    format!("строка {}: <{}>: {m}", e.line, e.name)
}

fn num(e: &El) -> Result<String, String> {
    let t = e.text()?;
    int(&t).ok_or_else(|| err(e, &format!("не число {t:?}")))
}

fn num_of(e: &El, tag: &str) -> Result<String, String> {
    num(e.req(tag)?)
}

fn num_attr(e: &El, k: &str) -> Result<i64, String> {
    let v = e.req_attr(k)?;
    v.parse().map_err(|_| err(e, &format!("{k}={v:?}")))
}

fn tok(e: &El, map: &[(&str, &str)]) -> Result<Node, String> {
    super::tpl::token(e, map)
}

fn color(e: &El) -> Result<Node, String> {
    let t = e.text()?;
    if t == "auto" {
        return Ok(l(vec![a("4"), a("4"), l(vec![a("0")]), a("4")]));
    }
    if let Some(n) = t.strip_prefix("style:") {
        let k = crate::xml::mxl::STYLE.iter().find(|(_, v)| *v == n).ok_or_else(|| err(e, &format!("цвет {t:?}")))?.0.to_string();
        return Ok(l(vec![a("4"), a("3"), l(vec![a(k)]), a("3")]));
    }
    super::ui::color(&t).ok_or_else(|| err(e, &format!("цвет {t:?}")))
}

/// Шрифт `{8,<вид 1 окна|2 стиля|3 авто>,<маска>,[{<ссылка>}],[высота×10],[жирный 700|400],[курсив],[подчеркнутый],
/// [зачеркнутый],["<имя>"],1,<масштаб>}`; маска: 1 имя, 2 высота, 4 жирный, 8 курсив, 16 подчеркнутый, 32 зачеркнутый.
fn font(e: &El) -> Result<Node, String> {
    e.text()?;
    let bad = |m: &str| err(e, &format!("шрифт: {m}"));
    let (kind, r) = match (e.req_attr("kind")?, e.attr("ref")) {
        ("AutoFont", None) => ("3", None),
        ("WindowsFont", Some("sys:DefaultGUIFont")) => {
            if e.attr("xmlns:sys") != Some("http://v8.1c.ru/8.1/data/ui/fonts/system") {
                return Err(bad("xmlns:sys"));
            }
            ("1", Some("0".to_string()))
        }
        ("StyleItem", Some(r)) => {
            let n = r.strip_prefix("style:").ok_or_else(|| bad(r))?;
            ("2", Some(crate::xml::mxl::STYLE.iter().find(|(_, v)| *v == n).ok_or_else(|| bad(r))?.0.to_string()))
        }
        (k, r) => return Err(bad(&format!("kind={k} ref={r:?}"))),
    };
    let mut mask = 0;
    let mut f = Vec::new();
    if let Some(h) = e.attr("height") {
        mask |= 2;
        let h: u32 = h.parse().map_err(|_| bad(h))?;
        f.push(a((h * 10).to_string()));
    }
    for (bit, k, yes, no) in [(4, "bold", "700", "400"), (8, "italic", "1", "0"), (16, "underline", "1", "0"), (32, "strikeout", "1", "0")] {
        match e.attr(k) {
            Some("true") => f.push(a(yes)),
            Some("false") => f.push(a(no)),
            None => continue,
            Some(v) => return Err(bad(&format!("{k}={v}"))),
        }
        mask |= bit;
    }
    if let Some(face) = e.attr("faceName") {
        mask |= 1;
        f.push(s(face));
    }
    if e.attr("scale").is_some() {
        return Err(bad("масштаб в схеме не встречен"));
    }
    let mut v = vec![a("8"), a(kind), a(mask.to_string())];
    if let Some(r) = r {
        v.push(l(vec![a(r)]));
    }
    v.extend(f);
    v.extend([a("1"), a("100")]);
    Ok(l(v))
}

/// Многоязычная строка схемы: `{1,N,{"<язык>","<текст>"}…}`.
fn ml(e: &El) -> Result<Node, String> {
    let mut v = vec![a("1"), a("0")];
    for it in e.elems() {
        if it.name != "v8:item" {
            return Err(err(it, "ожидался <v8:item>"));
        }
        v.push(l(vec![s(it.text_of("v8:lang")?), s(crlf(&it.text_of("v8:content")?))]));
    }
    if v.len() == 2 {
        e.text()?;
    }
    v[1] = a((v.len() - 2).to_string());
    Ok(l(v))
}

/// Рамка или линия `{4,0,{0},<вид 1 Solid|2 Dashed>,<толщина>,<разрыв>,e45c0cd8-…,0}`.
fn line(e: &El) -> Result<Node, String> {
    let st = e.req("v8ui:style")?;
    if st.req_attr("xsi:type")? != "sch:ConnectorLineType" {
        return Err(err(st, "xsi:type"));
    }
    let kind = tok(st, &[("1", "Solid"), ("2", "Dashed")])?;
    let gap = match e.req_attr("gap")? {
        "false" => "0",
        "true" => "1",
        g => return Err(err(e, &format!("gap={g:?}"))),
    };
    Ok(l(vec![a("4"), a("0"), l(vec![a("0")]), kind, a(num_attr(e, "width")?.to_string()), a(gap), a(LINE_KIND), a("0")]))
}

/// Файл картинки элемента по имени элемента и имени файла (`<Abs>`).
pub type Pictures<'a> = dyn Fn(&str, &str) -> Result<Vec<u8>, String> + 'a;

/// Размер картинки и картинка: пустая `{4,0,{0},"",-1,-1,1,0,""}`, с данными
/// `{4,3,{0},"",-1,-1,0,{{#base64:…}},0,""}` (LoadTransparent только false: другое не встречено).
fn picture(p: &El, name: &str, pics: &Pictures, used: &mut Vec<String>) -> Result<(Node, Node), String> {
    let size = tok(p.req("PictureSize")?, &[("0", "RealSize"), ("2", "Proportionally"), ("4", "AutoSize")])?;
    let pic = p.req("Picture")?;
    if !pic.attrs.is_empty() {
        return Err(err(pic, "атрибуты картинки"));
    }
    let Some(abs) = pic.child("Abs") else {
        pic.text()?;
        return Ok((size, l(vec![a("4"), a("0"), l(vec![a("0")]), s(""), a("-1"), a("-1"), a("1"), a("0"), s("")])));
    };
    let lt = pic.req("LoadTransparent")?;
    for e in [abs, lt] {
        if e.attr("xmlns") != Some(READABLE) || e.attrs.len() != 1 {
            return Err(err(e, "xmlns"));
        }
    }
    if pic.elems().count() != 2 || lt.text()? != "false" {
        return Err(err(pic, "картинка не как в схемах БП"));
    }
    let file = abs.text()?;
    let data = pics(name, &file)?;
    used.push(format!("{name}/{file}"));
    let b64 = super::payload::base64_atom(&data);
    Ok((size, l(vec![a("4"), a("3"), l(vec![a("0")]), s(""), a("-1"), a("-1"), a("0"), l(vec![l(vec![b64])]), a("0"), s("")])))
}

/// Заголовок `{4,<id>,<заголовок>,"<имя>",<порядок обхода>}` и оформление
/// `{7,<фон>,<линия>,<текст>,<шрифт>,<подсказка>,<гор.>,<верт.>,<картинка>,<Z>,<гиперссылка>,<прозрачность>,<группа?>,0}`
/// (группа везде 0 — место не установлено, другое значение — отказ).
fn common(id: &str, p: &El) -> Result<(Node, Node), String> {
    let h = l(vec![a("4"), a(id), ml(p.req("Title")?)?, s(p.text_of("Name")?), a(num_of(p, "TabOrder")?)]);
    if num_of(p, "GroupNumber")? != "0" {
        return Err(err(p, "номер группы не 0 не загружается"));
    }
    let b = |t: &str| -> Result<Node, String> { boolean(p.req(t)?) };
    let v = l(vec![
        a("7"),
        color(p.req("BackColor")?)?,
        color(p.req("LineColor")?)?,
        color(p.req("TextColor")?)?,
        font(p.req("Font")?)?,
        ml(p.req("ToolTip")?)?,
        tok(p.req("HorizontalAlign")?, &[("0", "Left"), ("1", "Center"), ("2", "Right")])?,
        tok(p.req("VerticalAlign")?, &[("0", "Top"), ("1", "Center"), ("2", "Bottom")])?,
        tok(p.req("PictureLocation")?, &[("1", "Left"), ("5", "Center")])?,
        a(num_of(p, "ZOrder")?),
        b("Hyperlink")?,
        b("Transparent")?,
        a("0"),
        a("0"),
    ]);
    Ok((h, v))
}

/// Левый, верхний, правый, нижний края.
fn location(p: &El) -> Result<[i64; 4], String> {
    let e = p.req("Location")?;
    e.text()?;
    Ok([num_attr(e, "left")?, num_attr(e, "top")?, num_attr(e, "right")?, num_attr(e, "bottom")?])
}

/// Точки контура фигуры по ее виду и размеру (как строит платформа; проверено на размерах из карт БП).
fn outline(kind: &str, [l0, t, r, b]: [i64; 4], e: &El) -> Result<Vec<(i64, i64)>, String> {
    // scheme.dll FUN_18004b760 (переход по виду элемента 2..10): скос — trunc(x · tan 30° · 0.5) в double, деления
    // целочисленные; у старта и завершения скос от ширины (больше высоты — h/2), у условия — от высоты (больше w/2 — w/4)
    let (w, h) = (r - l0, b - t);
    let slant = |x: i64| (x as f64 * 0.577350270614116 * 0.5) as i64;
    Ok(match kind {
        "Start" | "Completion" => {
            let d = slant(w);
            let s = if d <= h { d } else { h / 2 };
            if kind == "Start" {
                vec![(l0, t), (r - 1, t), (r - 1, b - s), (l0 + w / 2, b - 1), (l0, b - s)]
            } else {
                vec![(l0 + w / 2, t), (r - 1, t + s), (r - 1, b - 1), (l0, b - 1), (l0, t + s)]
            }
        }
        "Condition" => {
            let d = slant(h);
            let s = if d <= w / 2 { d } else { w / 4 };
            vec![(l0, t + h / 2), (l0 + s, t), (r - s - 1, t), (r - 1, t + h / 2), (r - s - 1, b - 1), (l0 + s, b - 1)]
        }
        "Activity" | "Processing" => vec![(l0, t), (r - 1, t), (r - 1, b - 1), (l0, b - 1)],
        "Split" => {
            let m = (w - 1) / 2;
            vec![(l0, t), (l0 + 2 * m, t), (l0 + m, b - 1)]
        }
        "Join" => {
            let m = (w - 1) / 2;
            vec![(l0, b - 1), (l0 + 2 * m, b - 1), (l0 + m, t)]
        }
        _ => return Err(err(e, &format!("контур фигуры {kind} не выведен"))),
    })
}

/// События `{N,{<номер>,"<обработчик>"}…}`: в XML все события вида по порядку, пустые пропускаются.
fn events(e: &El, names: &[&str]) -> Result<Node, String> {
    let got: Vec<&El> = e.elems().collect();
    if got.len() != names.len() {
        return Err(err(e, "состав событий"));
    }
    let mut v = vec![a("0")];
    for (k, (ev, name)) in got.iter().zip(names).enumerate() {
        if ev.name != "Event" || ev.req_attr("name")? != *name || ev.attrs.len() != 1 {
            return Err(err(ev, &format!("ожидалось событие {name}")));
        }
        let t = ev.text()?;
        if !t.is_empty() {
            v.push(l(vec![a(k.to_string()), s(t)]));
        }
    }
    v[0] = a((v.len() - 1).to_string());
    Ok(l(v))
}

fn pairs(v: &mut Vec<Node>, pts: &[(i64, i64)]) {
    v.push(a(pts.len().to_string()));
    for (x, y) in pts {
        v.extend([a(x.to_string()), a(y.to_string())]);
    }
}

/// Счетчик в конце схемы — последний выданный id: платформа создает элементы по порядку XML, каждому выдает
/// следующий свободный номер (больше прежнего и не занятый id уже загруженных элементов), затем ставит id из XML.
/// 51 из 51 схемы БП и 8.5 (id в XML с разрывами и не по порядку). Повтор id — отказ.
fn counter(ids: &[i64]) -> Result<i64, String> {
    let mut used = std::collections::HashSet::new();
    let mut c = 0;
    for &id in ids {
        c += 1;
        while used.contains(&c) {
            c += 1;
        }
        if !used.insert(id) {
            return Err(format!("id {id} у двух элементов схемы"));
        }
    }
    Ok(c)
}

/// Схема → запись; `pics` — файлы картинок элементов, в ответе — использованные (`<элемент>/<файл>`).
pub fn record(doc: &El, x: &Index, pics: &Pictures) -> Result<(Rec, Vec<String>), String> {
    if doc.name != "GraphicalSchema" {
        return Err("корень не <GraphicalSchema>".into());
    }
    for (k, v) in NS {
        if doc.attr(k) != Some(v) {
            return Err(format!("<GraphicalSchema>: {k} не {v:?}"));
        }
    }
    if doc.attrs.len() != NS.len() {
        return Err("<GraphicalSchema>: лишние атрибуты".into());
    }
    let mut used = Vec::new();
    let mut head = vec![a("1"), color(doc.req("BackColor")?)?, boolean(doc.req("GridEnabled")?)?, a(num_of(doc, "GridHorizontalStep")?), a(num_of(doc, "GridVerticalStep")?), tok(doc.req("DrawGridMode")?, &[("0", "None"), ("1", "Dots"), ("2", "Chess"), ("3", "Lines")])?];
    let pp: Vec<&El> = doc.req("PrintParameters")?.elems().collect();
    head.push(a(pp.len().to_string()));
    for p in pp {
        let (code, v) = match p.name.as_str() {
            "TopMargin" => ("6", a(num(p)?)),
            "LeftMargin" => ("7", a(num(p)?)),
            "BottomMargin" => ("8", a(num(p)?)),
            "RightMargin" => ("9", a(num(p)?)),
            "BlackAndWhite" => ("13", boolean(p)?),
            "FitPageMode" => ("16", tok(p, &[("0", "Auto")])?),
            _ => return Err(err(p, "параметр печати не загружается")),
        };
        head.extend([a(code), l(vec![s("N"), v])]);
    }
    let items: Vec<&El> = match doc.child("Items") {
        Some(i) => i.elems().collect(),
        None => return Err("схема без <Items> не встречена".into()),
    };
    // имя → (id, порт «истина» условия): на них ссылаются линии
    let mut names: HashMap<String, (String, Option<String>)> = HashMap::new();
    for it in &items {
        let p = it.req("Properties")?;
        let tp = if it.name == "Condition" { Some(num_of(p, "TruePortIndex")?) } else { None };
        names.insert(p.text_of("Name")?, (it.req_attr("id")?.to_string(), tp));
    }
    let mut ids = Vec::new();
    let mut out = vec![a("5"), l(vec![l(head)]), a(items.len().to_string())];
    for it in items {
        let id = it.req_attr("id")?;
        ids.push(id.parse::<i64>().map_err(|_| err(it, "id"))?);
        let p = it.req("Properties")?;
        let name = p.text_of("Name")?;
        let (h, v) = common(id, p)?;
        let (kind, node) = match it.name.as_str() {
            "Decoration" => {
                if it.attrs.len() != 1 {
                    return Err(err(it, "атрибуты"));
                }
                let [l0, t, r, b] = location(p)?;
                // фигура декорации: scheme.dll 0x180017360
                let shape = tok(p.req("Shape")?, &[("0", "None"), ("1", "File"), ("2", "Folder"), ("3", "RightArrow"), ("4", "LeftArrow"), ("5", "UpArrow"), ("6", "DownArrow"), ("7", "LeftRightArrow"), ("8", "UpDownArrow"), ("9", "VerticalBrackets"), ("10", "Document"), ("11", "Block"), ("12", "HorizontalBrackets"), ("13", "Ellipse")])?;
                let flip = num_of(p, "FlipMode")?;
                let angle = p.req("Angle")?;
                if angle.req_attr("xsi:type")? != "xs:decimal" {
                    return Err(err(angle, "xsi:type"));
                }
                // после картинки — повтор прозрачности (так во всех схемах БП)
                let transparent = boolean(p.req("Transparent")?)?;
                let (size, pic) = picture(p, &name, pics, &mut used)?;
                let body = vec![v, a("6"), a(l0.to_string()), a(t.to_string()), a(r.to_string()), a(b.to_string()), size, pic, transparent, shape, a(flip), a(num(angle)?)];
                ("0", l(vec![h, a("2"), l(vec![l(body)])]))
            }
            "ConnectionLine" => {
                if it.attrs.len() != 1 {
                    return Err(err(it, "атрибуты"));
                }
                let pts: Vec<(i64, i64)> = p
                    .req("PivotPoints")?
                    .elems()
                    .map(|pt| {
                        if pt.name != "Point" {
                            return Err(err(pt, "ожидалась <Point>"));
                        }
                        pt.text()?;
                        Ok((num_attr(pt, "x")?, num_attr(pt, "y")?))
                    })
                    .collect::<Result<_, String>>()?;
                let mut body = vec![v, a("6")];
                pairs(&mut body, &pts);
                body.push(line(p.req("Line")?)?);
                body.push(tok(p.req("TextLocation")?, &[("0", "FirstSegment")])?);
                let c = p.req("Connect")?;
                let end = |tag: &str| -> Result<(String, String, Option<String>), String> {
                    let e = c.req(tag)?;
                    let item = e.req("Item")?;
                    let port = num_of(e, "PortIndex")?;
                    let n = item.text()?;
                    if n.is_empty() {
                        return Ok(("-1".into(), port, None));
                    }
                    let (id, tp) = names.get(&n).ok_or_else(|| err(item, &format!("нет элемента {n:?}")))?;
                    Ok((id.clone(), port, tp.clone()))
                };
                let ((from, pf, tp), (to, pt, _)) = (end("From")?, end("To")?);
                // 1 — линия выходит из порта «истина» условия
                let branch = if tp.as_deref() == Some(pf.as_str()) { "1" } else { "0" };
                body.extend([a(pf), a(pt)]);
                let segs: Vec<&El> = p.child("ManualyMovedSegments").map(|m| m.elems().collect()).unwrap_or_default();
                body.push(a(segs.len().to_string()));
                for sg in segs {
                    if sg.name != "Segment" {
                        return Err(err(sg, "ожидался <Segment>"));
                    }
                    let (st, en) = (sg.req("Start")?, sg.req("End")?);
                    st.text()?;
                    en.text()?;
                    body.extend([num_attr(st, "x")?, num_attr(st, "y")?, num_attr(en, "x")?, num_attr(en, "y")?, num_attr(sg, "index")?].map(|n| a(n.to_string())));
                }
                let arrow = [("0", "None"), ("1", "Filled"), ("2", "Blank")];
                body.extend([tok(p.req("BeginArrow")?, &arrow)?, tok(p.req("EndArrow")?, &arrow)?]);
                ("1", l(vec![h, a("3"), a(from), a(branch), a(to), boolean(p.req("DecorativeLine")?)?, l(vec![l(body)])]))
            }
            k @ ("Start" | "Completion" | "Condition" | "Activity" | "Processing" | "Split" | "Join") => {
                let uuid = it.req_attr("uuid")?;
                if it.attrs.len() != 2 {
                    return Err(err(it, "атрибуты"));
                }
                let loc = location(p)?;
                let mut b = vec![v, a("5")];
                b.extend(loc.iter().map(|n| a(n.to_string())));
                pairs(&mut b, &outline(k, loc, p)?);
                let (size, pic) = picture(p, &name, pics, &mut used)?;
                b.extend([size, pic, line(p.req("Border")?)?]);
                let hdr = l(vec![h, a("4"), a(uuid), a("0")]);
                // у разделения и слияния событий нет: запись без списка событий (ERP СогласованиеЗакупки)
                let ev = if matches!(k, "Split" | "Join") { it } else { it.req("Events")? };
                match k {
                    "Split" => ("7", l(vec![hdr, a("1"), l(vec![l(vec![l(b), a("1")])])])),
                    "Join" => ("8", l(vec![hdr, a("1"), l(vec![l(vec![l(b), a("1")])])])),
                    "Start" => ("2", l(vec![hdr, a("2"), l(vec![l(vec![l(b), a("1")])]), events(ev, &["BeforeStart"])?])),
                    "Completion" => ("3", l(vec![hdr, a("2"), l(vec![l(vec![l(b), a("1")])]), events(ev, &["OnComplete"])?])),
                    // обработка (вид 9): как старт, версия 0 (ERP СогласованиеПродажи)
                    "Processing" => ("9", l(vec![hdr, a("0"), l(vec![l(vec![l(b), a("1")])]), events(ev, &["Processing"])?])),
                    "Condition" => {
                        let wrap = l(vec![l(b), a("3"), a(num_of(p, "TruePortIndex")?), a(num_of(p, "FalsePortIndex")?)]);
                        ("4", l(vec![hdr, a("1"), l(vec![wrap]), events(ev, &["ConditionCheck"])?]))
                    }
                    _ => {
                        let mut attrs: Vec<(String, Node)> = Vec::new();
                        for aa in p.req("AddressingAttributes")?.elems() {
                            let r = aa.req_attr("ref")?;
                            let g = x.id(r).ok_or_else(|| err(aa, &format!("нет реквизита {r}")))?;
                            let val = aa.req("Value")?;
                            // пусто — `{"U"}`; ссылка времени разработки — как у реквизитов (обратное common::design_time_ref)
                            let v = if val.attr("xsi:nil") == Some("true") && val.attrs.len() == 1 {
                                val.text()?;
                                l(vec![s("U")])
                            } else if let Some(pre) = val.attr("xsi:type").and_then(|t| t.strip_suffix(":DesignTimeRef")).filter(|_| val.attrs.len() == 2) {
                                if val.req_attr(&format!("xmlns:{pre}"))? != "http://v8.1c.ru/8.3/xcf/readable" {
                                    return Err(err(val, "пространство имен ссылки"));
                                }
                                let t = val.text()?;
                                super::attr::design_time_ref(x, &t).ok_or_else(|| err(val, &format!("незнакомая ссылка {t:?}")))?
                            } else {
                                return Err(err(val, "значение реквизита адресации не загружается"));
                            };
                            attrs.push((g.to_string(), v));
                        }
                        // в записи — по uuid (так у всех 11 точек действия БП)
                        attrs.sort_by(|p, q| p.0.cmp(&q.0));
                        let mut av = vec![a(attrs.len().to_string())];
                        av.extend(attrs.into_iter().map(|(g, v)| l(vec![a(g), v])));
                        let names = ["InteractiveActivationProcessing", "BeforeCreateTasks", "OnCreateTask", "OnExecute", "CheckExecutionProcessing", "BeforeExecute", "BeforeExecuteInteractively"];
                        let wrap = l(vec![l(b), a("3"), a("2"), a("0")]);
                        ("5", l(vec![hdr, a("8"), l(vec![wrap]), s(p.text_of("Explanation")?), boolean(p.req("Group")?)?, events(ev, &names)?, l(av), s(p.text_of("TaskDescription")?)]))
                    }
                }
            }
            n => return Err(err(it, &format!("элемент схемы {n} не загружается"))),
        };
        out.extend([a(kind), node]);
    }
    out.push(a(counter(&ids)?.to_string()));
    Ok((Rec::Raw(Doc::new(true, vec![l(out)]).write()?.into_bytes()), used))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn outlines_of_checked_sizes_only() {
        let e = super::super::dom::parse("<P/>").unwrap();
        assert_eq!(outline("Start", [220, 20, 260, 60], &e).unwrap(), vec![(220, 20), (259, 20), (259, 49), (240, 59), (220, 49)]);
        assert_eq!(outline("Condition", [0, 0, 120, 60], &e).unwrap(), vec![(0, 30), (17, 0), (102, 0), (119, 30), (102, 59), (17, 59)]);
        assert_eq!(outline("Condition", [0, 0, 160, 40], &e).unwrap()[1], (11, 0));
        assert_eq!(outline("Start", [0, 0, 150, 60], &e).unwrap()[2], (149, 17));
    }

    #[test]
    fn counter_cases() {
        assert_eq!(counter(&[2, 3, 4, 13]), Ok(5));
        assert_eq!(counter(&[1, 2, 74, 81, 82]), Ok(5));
        // БП: Задание (37) и Расходы82 (79) — номера, занятые id из XML, пропускаются
        assert_eq!(counter(&[1, 2, 3, 4, 5, 6, 11, 12, 13, 20, 21, 22, 23, 24, 25, 26, 27, 29, 30, 34, 35, 36, 74]), Ok(37));
        let r82: Vec<i64> = [1, 17].into_iter().chain(31..=74).chain(80..=84).collect();
        assert_eq!(counter(&r82), Ok(79));
        assert!(counter(&[1, 1]).is_err());
    }
}
