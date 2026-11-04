//! Табличный документ `Ext/Template.xml` (`<document>`) → запись макета `MOXCEL` (обратное `xml/mxl.rs`).
//!
//! Запись версии 12 (так пишет загрузка платформы). Форматы, шрифты, линии, цвета и прочие списки в записи
//! пронумерованы по первому использованию в порядке полей записи (так у 56 из 60 макетов выборки, остальные —
//! из-за колонтитулов, которые идут раньше строк), а не как в XML. Раскладка — docs/research/xml-mxl.md.

use super::dom::El;
use super::index::Index;
use super::tpl::{a, crlf, l, s};
use super::Rec;
use crate::brace::{Doc, Node};

const ZERO: &str = "00000000-0000-0000-0000-000000000000";
const CELL_LINE: &str = "f527dc88-1d39-40b3-bcbb-d98b690ead68";
const DRAWING_LINE: &str = "b7438842-27cc-42a3-846f-2250cd9c1bc3";
const DCS_DETAIL: &str = "3031edd8-c3df-47b2-98ca-47f628d4ec18";
const CHART: &str = "a8b97779-1a4b-4059-b09c-807f86d2a461";
const GANTT: &str = "e5fdc112-5c84-4a16-9728-72b85692b6e2";
const HEAD: &[u8] = b"MOXCEL\0\x08\0\x01\0\x0c\0";

const NS: &[(&str, &str)] = &[
    ("xmlns", "http://v8.1c.ru/8.2/data/spreadsheet"),
    ("xmlns:pal", "http://v8.1c.ru/8.1/data/ui/colors/palette"),
    ("xmlns:style", "http://v8.1c.ru/8.1/data/ui/style"),
    ("xmlns:v8", "http://v8.1c.ru/8.1/data/core"),
    ("xmlns:v8ui", "http://v8.1c.ru/8.1/data/ui"),
    ("xmlns:xs", "http://www.w3.org/2001/XMLSchema"),
    ("xmlns:xsi", "http://www.w3.org/2001/XMLSchema-instance"),
];

type R<T> = Result<T, String>;

fn err(e: &El, m: &str) -> String {
    format!("строка {}: <{}>: {m}", e.line, e.name)
}

fn num(e: &El) -> R<i64> {
    let t = e.text()?;
    t.parse().map_err(|_| err(e, &format!("не число {t:?}")))
}

fn idx(e: &El) -> R<usize> {
    let t = e.text()?;
    t.parse().map_err(|_| err(e, &format!("не номер {t:?}")))
}

fn an(v: i64) -> Node {
    a(v.to_string())
}

fn boolean(e: &El) -> R<bool> {
    match e.text()?.as_str() {
        "true" => Ok(true),
        "false" => Ok(false),
        t => Err(err(e, &format!("не булево {t:?}"))),
    }
}

/// Нумерация по первому использованию.
struct Order<K> {
    keys: Vec<K>,
}

impl<K: PartialEq> Order<K> {
    fn new() -> Self {
        Order { keys: Vec::new() }
    }
    fn id(&mut self, k: K) -> usize {
        match self.keys.iter().position(|x| *x == k) {
            Some(i) => i,
            None => {
                self.keys.push(k);
                self.keys.len() - 1
            }
        }
    }
}

/// Многоязычная строка табличного документа `{1,N,{"<язык>","<текст>"}…}`.
/// Многоязычная строка без разметки: каждый язык — `formatted_plain` (getPureData).
fn pure(src: &Node) -> Node {
    match src {
        Node::List(v) => Node::List(
            v.iter()
                .map(|it| match it {
                    Node::List(p) if p.len() == 2 => match p[1].str() {
                        Some(t) => l(vec![p[0].clone(), s(crate::xml::imp::form::formatted_plain(t).unwrap_or_default())]),
                        None => it.clone(),
                    },
                    _ => it.clone(),
                })
                .collect(),
        ),
        n => n.clone(),
    }
}

fn ml(e: &El) -> R<Node> {
    let mut v = vec![a("1"), a("0")];
    for it in e.elems() {
        if it.name != "v8:item" {
            return Err(err(it, "ожидался <v8:item>"));
        }
        let lang = it.req("v8:lang")?.text()?;
        v.push(l(vec![s(lang), s(crlf(&it.text_of("v8:content")?))]));
    }
    if v.len() == 2 {
        e.text()?;
    }
    v[1] = a((v.len() - 2).to_string());
    Ok(l(v))
}

struct B<'a> {
    x: &'a Index,
    xfmts: Vec<&'a El>,
    xfonts: Vec<&'a El>,
    xlines: Vec<&'a El>,
    fmts: Order<usize>,
    fonts: Order<usize>,
    lines: Order<usize>,
    /// Линии, использованные форматами рисунков (у них свой перечень видов).
    dlines: Vec<usize>,
    colors: Order<Node>,
    fstrings: Order<Node>,
    masks: Order<Node>,
    types: Order<Node>,
    controls: Order<Node>,
    /// Форматы рисунков: у них граница — drawingBorder.
    dfmts: Vec<usize>,
    /// Наибольший id рисунка (и примечания).
    max_id: i64,
}

impl<'a> B<'a> {
    /// Номер формата XML (с 1) → номер в записи (с 1); 0 — без формата.
    fn f(&mut self, e: &El) -> R<usize> {
        let i = idx(e)?;
        self.fi(i, e)
    }

    fn fi(&mut self, i: usize, e: &El) -> R<usize> {
        if i == 0 {
            return Ok(0);
        }
        if i > self.xfmts.len() {
            return Err(err(e, &format!("нет формата {i}")));
        }
        Ok(self.fmts.id(i) + 1)
    }

    fn color(&mut self, e: &El) -> R<Node> {
        let c = color_node(self.x, e)?;
        Ok(an(self.colors.id(c) as i64))
    }

    fn fstring(&mut self, e: &El, mask: bool) -> R<Node> {
        let v = ml(e)?;
        Ok(an(if mask { self.masks.id(v) } else { self.fstrings.id(v) } as i64))
    }

    /// Формат XML → `{маска,значения по возрастанию битов}` (отступы: левый 42, правый 44, верхний 43, нижний 45).
    fn format(&mut self, e: &El, drawing: bool) -> R<Node> {
        let mut vals: Vec<(u32, Node)> = Vec::new();
        let tok = |c: &El, m: &[(&str, i64)]| -> R<Node> {
            let t = c.text()?;
            m.iter().find(|(n, _)| *n == t).map(|(_, v)| an(*v)).ok_or_else(|| err(c, &format!("значение {t:?}")))
        };
        let b01 = |c: &El, inv: bool| -> R<Node> { Ok(a(if boolean(c)? != inv { "1" } else { "0" })) };
        let mut have = 0i64;
        let mut seen_have = 0;
        for c in e.elems() {
            if !c.attrs.is_empty() && !matches!(c.name.as_str(), "textColor" | "backColor" | "borderColor" | "patternColor") {
                return Err(err(c, "атрибуты"));
            }
            let (bit, v) = match c.name.as_str() {
                "font" => {
                    let i = idx(c)?;
                    if i >= self.xfonts.len() {
                        return Err(err(c, &format!("нет шрифта {i}")));
                    }
                    (0, an(self.fonts.id(i) as i64))
                }
                "border" | "leftBorder" | "topBorder" | "rightBorder" | "bottomBorder" | "drawingBorder" => {
                    let i = idx(c)?;
                    if i >= self.xlines.len() {
                        return Err(err(c, &format!("нет линии {i}")));
                    }
                    if (c.name == "drawingBorder") != drawing {
                        return Err(err(c, "граница не того вида"));
                    }
                    if drawing && !self.dlines.contains(&i) {
                        self.dlines.push(i);
                    }
                    let id = an(self.lines.id(i) as i64);
                    match c.name.as_str() {
                        "border" => {
                            for b in 1..=4 {
                                vals.push((b, id.clone()));
                            }
                            continue;
                        }
                        "leftBorder" | "drawingBorder" => (1, id),
                        "topBorder" => (2, id),
                        "rightBorder" => (3, id),
                        _ => (4, id),
                    }
                }
                n @ ("drawingHaveLeftBorder" | "drawingHaveTopBorder" | "drawingHaveRightBorder" | "drawingHaveBottomBorder") if drawing => {
                    let k = ["drawingHaveLeftBorder", "drawingHaveTopBorder", "drawingHaveRightBorder", "drawingHaveBottomBorder"].iter().position(|x| *x == n).unwrap_or(0);
                    if boolean(c)? {
                        have |= 1 << k;
                    }
                    seen_have += 1;
                    if seen_have == 4 {
                        vals.push((3, an(have)));
                    }
                    continue;
                }
                "print" if drawing => (4, b01(c, true)?),
                "borderColor" => (5, self.color(c)?),
                "textColor" => (10, self.color(c)?),
                "backColor" => (11, self.color(c)?),
                "patternColor" => (13, self.color(c)?),
                "height" => (6, an(num(c)?)),
                "width" => (7, an(num(c)?)),
                "horizontalAlignment" => (8, tok(c, &[("Left", 0), ("Right", 2), ("Justify", 4), ("Auto", 5), ("Center", 6)])?),
                "verticalAlignment" => (9, tok(c, &[("Top", 0), ("Bottom", 8), ("Center", 24)])?),
                "pattern" => {
                    let t = c.text()?;
                    let v = match t.as_str() {
                        "Solid" => 0,
                        "WithoutPattern" => 255,
                        p => p.strip_prefix("Pattern").and_then(|n| n.parse().ok()).filter(|n| (1..=20).contains(n)).ok_or_else(|| err(c, &format!("узор {p}")))?,
                    };
                    (12, an(v))
                }
                "textPlacement" => (14, tok(c, &[("Auto", 0), ("Cut", 1), ("Block", 2), ("Wrap", 3)])?),
                "fillType" => (15, tok(c, &[("Text", 0), ("Parameter", 1), ("Template", 2)])?),
                "protection" => (16, b01(c, true)?),
                "hidden" => (17, b01(c, false)?),
                "textOrientation" => (18, an(num(c)?)),
                "detailsUse" => (19, tok(c, &[("Cell", 0), ("Row", 1), ("WithoutProcessing", 2)])?),
                "bySelectedColumns" => (20, b01(c, false)?),
                "markNegatives" => (21, b01(c, false)?),
                "containsValue" => (22, b01(c, false)?),
                "valueType" => {
                    let t = super::tpl::type_desc(self.x, c)?;
                    (23, an(self.types.id(t) as i64))
                }
                "format" => (24, self.fstring(c, false)?),
                "controlType" => {
                    let g = c.text()?;
                    if !crate::brace::is_guid(&g) {
                        return Err(err(c, "не GUID"));
                    }
                    (25, an(self.controls.id(a(g)) as i64))
                }
                "hyperLink" => (26, b01(c, false)?),
                "quickDrag" => (27, b01(c, false)?),
                "autoMarkIncomplete" => (28, b01(c, false)?),
                "markIncomplete" => (29, b01(c, false)?),
                "indent" => (30, an(num(c)?)),
                "autoIndent" => (31, an(num(c)?)),
                "editFormat" => (32, self.fstring(c, false)?),
                "columnSizeChange" => (33, tok(c, &[("Normal", 0), ("QuickChange", 1)])?),
                "mask" => (34, self.fstring(c, true)?),
                "picIndex" => (35, an(num(c)?)),
                "picHorizontalAlignment" => (36, tok(c, &[("Left", 0), ("Right", 2), ("Justify", 4), ("Auto", 5), ("Center", 6)])?),
                "picVerticalAlignment" => (37, tok(c, &[("Top", 0), ("Bottom", 8), ("Center", 24)])?),
                "pictureSizeMode" => (38, tok(c, &[("RealSize", 0), ("Stretch", 1), ("Proportionally", 2), ("Tile", 3), ("AutoSize", 4), ("RealSizeIgnoreScale", 5), ("AutoSizeIgnoreScale", 6), ("ByFontSize", 7)])?),
                "textPosition" => (39, tok(c, &[("Left", 0), ("Right", 1), ("Top", 2), ("Bottom", 3), ("OnTop", 4), ("Auto", 5)])?),
                "autoWidthCalculation" => (40, b01(c, false)?),
                "widthWeightFactor" => (41, an(num(c)?)),
                "leftMargin" => (42, an(num(c)?)),
                "topMargin" => (43, an(num(c)?)),
                "rightMargin" => (44, an(num(c)?)),
                "bottomMargin" => (45, an(num(c)?)),
                n => return Err(err(c, &format!("свойство формата {n} не загружается"))),
            };
            if vals.iter().any(|(b, _)| *b == bit) {
                return Err(err(c, "свойство повторено"));
            }
            vals.push((bit, v));
        }
        if seen_have != 0 && seen_have != 4 {
            return Err(err(e, "не все drawingHave*Border"));
        }
        if vals.is_empty() {
            e.text()?;
        }
        vals.sort_by_key(|(b, _)| match b {
            43 => 44,
            44 => 43,
            b => *b,
        });
        let mask: u64 = vals.iter().map(|(b, _)| 1u64 << b).sum();
        let mut v = vec![a(mask.to_string())];
        v.extend(vals.into_iter().map(|(_, n)| n));
        Ok(l(v))
    }

    /// Ячейка `<c>` (внутренняя) → `{флаги,формат,…}`.
    fn cell(&mut self, c: &El, drawing: bool) -> R<Node> {
        let mut fmt = 0;
        let (mut text, mut value, mut detail, mut dvalue, mut control, mut picture) = (None, None, None, None, None, None);
        let mut note = None;
        let mut formatted: Option<Node> = None;
        for k in c.elems() {
            match k.name.as_str() {
                "f" | "formatIndex" => {
                    let i = idx(k)?;
                    fmt = self.fi(i, k)?;
                    if drawing && i != 0 && !self.dfmts.contains(&i) {
                        self.dfmts.push(i);
                    }
                }
                "tl" | "text" => text = Some(ml(k)?),
                // форматированная строка (moxel FUN_1803fee80): текст ячейки — без разметки (`FormattedString::getPureData`,
                // запись FUN_1802f6eb0), сама строка с разметкой — в хвосте `1,{1,строка,1}`
                "tfl" => {
                    let src = ml(k)?;
                    text = Some(pure(&src));
                    formatted = Some(src);
                }
                "parameter" => text = Some(l(vec![a("1"), a("1"), l(vec![s(""), s(crlf(&k.text()?))])])),
                "v" | "value" => value = Some(self.value(k)?),
                "detailParameter" => detail = Some(s(crlf(&k.text()?))),
                "d" | "detailValue" => dvalue = Some(self.dvalue(k)?),
                "r" => dvalue = Some(l(vec![s("#"), a(DCS_DETAIL), l(vec![an(num(k)?)])])),
                "control" => {
                    if k.attr("xsi:type") != Some("xs:base64Binary") {
                        return Err(err(k, "xsi:type"));
                    }
                    control = Some(a(format!("#base64:{}", k.text()?.replace("\r\n", "\r\r\n"))));
                }
                "pictureParameter" => picture = Some(s(k.text()?)),
                "note" => {
                    let (id, d) = self::drawing(self, k)?;
                    self.max_id = self.max_id.max(id);
                    let txt = ml(k.req("text")?)?;
                    note = Some((txt, d));
                }
                n => return Err(err(k, &format!("{n} в ячейке не загружается"))),
            }
        }
        let mut flags = 0;
        let mut v = vec![a("0"), an(fmt as i64)];
        if let Some(cn) = control {
            flags |= 1;
            v.extend([a("1"), l(vec![cn])]);
        }
        for (bit, x) in [(2, value), (4, dvalue), (8, detail), (64, picture), (16, text.clone())] {
            if let Some(x) = x {
                flags |= bit;
                v.push(x);
            }
        }
        // примечание: текст, 1, рисунок «Комментарий»
        if let Some((t, d)) = note {
            flags |= 32;
            v.extend([t, a("1"), d]);
        }
        match (&text, formatted) {
            (Some(_), Some(f)) => v.extend([a("1"), l(vec![a("1"), f, a("1")])]),
            (Some(_), None) => v.push(a("0")),
            _ => {}
        }
        v[0] = an(flags);
        Ok(l(v))
    }

    /// Значение расшифровки: примитив или Структура `{"#",4238019d-…,{n,{{"S",ключ},значение}…}}`, Массив
    /// `{"#",51e7a0d2-…,{n,значение…}}`, Соответствие
    /// `{"#",3d48feae-…,{n,{ключ,значение}…}}`, СписокЗначений `{"#",4772b3b4-…,{6,1e512aab-…,0,0,{n,{1e512aab-…,
    /// {представление,0,значение,{4,0,{0},"",-1,-1,0,0,""},id,0,""}}…},{"Pattern"},0,последний id}}` (обратное xvalue
    /// выгрузки). Пары соответствия загрузка вставляет в хэш-таблицу в порядке XML и пишет в порядке ее обхода
    /// (`hashorder::boost_order`, хэш строки — `h·37 + код UTF-16`); ключи не строки — только до 4 пар (обратный порядок).
    fn dvalue(&mut self, e: &El) -> R<Node> {
        const STRUCTURE: &str = "4238019d-7e49-4fc9-91db-b6b951d5cf8e";
        const MAP: &str = "3d48feae-a9c6-4c5a-a099-9eb6477630c6";
        const VALUE_LIST: &str = "4772b3b4-f4a3-49c0-a1a5-8cb5961511a3";
        const ITEM: &str = "1e512aab-1b41-4ef6-9375-f0137be9dd91";
        const ARRAY: &str = "51e7a0d2-530b-11d4-b98a-008048da3034";
        let wrap = |id: &str, b: Node| l(vec![s("#"), a(id), b]);
        match e.peek_attr("xsi:type") {
            Some("v8:Structure") => {
                e.attr("xsi:type");
                let props: Vec<&El> = e.elems().collect();
                let mut v = vec![an(props.len() as i64)];
                for p in props {
                    if p.name != "v8:Property" || p.elems().count() != 1 {
                        return Err(err(p, "свойство структуры"));
                    }
                    v.push(l(vec![l(vec![s("S"), s(p.req_attr("name")?)]), self.dvalue(p.req("v8:Value")?)?]));
                }
                if v.len() == 1 {
                    e.text()?;
                }
                Ok(wrap(STRUCTURE, l(v)))
            }
            // Массив `{n,значение*n}`: core85 0x1801ce470 (чтение Массива) — число, затем значения до конца списка.
            Some("v8:Array") => {
                e.attr("xsi:type");
                let mut v = vec![a("0")];
                for it in e.elems() {
                    if it.name != "v8:Value" {
                        return Err(err(it, "элемент массива"));
                    }
                    v.push(self.dvalue(it)?);
                }
                if v.len() == 1 {
                    e.text()?;
                }
                v[0] = an(v.len() as i64 - 1);
                Ok(wrap(ARRAY, l(v)))
            }
            Some("v8:Map") => {
                e.attr("xsi:type");
                let mut pairs = Vec::new();
                for p in e.elems() {
                    if p.name != "v8:pair" || p.elems().count() != 2 {
                        return Err(err(p, "пара соответствия"));
                    }
                    let k = p.req("v8:Key")?;
                    let key_text = (k.peek_attr("xsi:type") == Some("xs:string")).then(|| k.peek_text()).flatten();
                    pairs.push((key_text, l(vec![self.dvalue(k)?, self.dvalue(p.req("v8:Value")?)?])));
                }
                if pairs.is_empty() {
                    e.text()?;
                }
                let n = pairs.len();
                let ordered: Vec<Node> = if pairs.iter().all(|(k, _)| k.is_some()) {
                    let hash = |p: &(Option<String>, Node)| p.0.as_deref().unwrap_or("").encode_utf16().fold(0u32, |h, c| h.wrapping_mul(37).wrapping_add(c as u32)) as i32 as i64 as u64;
                    super::hashorder::boost_order(&pairs, hash).into_iter().map(|p| p.1).collect()
                } else if n <= 4 {
                    pairs.into_iter().rev().map(|p| p.1).collect()
                } else {
                    return Err(err(e, &format!("соответствие из {n} пар с нестроковыми ключами: порядок не выведен")));
                };
                let mut v = vec![an(n as i64)];
                v.extend(ordered);
                Ok(wrap(MAP, l(v)))
            }
            Some("v8:ValueListType") => {
                e.attr("xsi:type");
                let kids: Vec<&El> = e.elems().collect();
                let (Some(vt), Some(last)) = (kids.first().filter(|k| k.name == "v8:valueType"), kids.get(1).filter(|k| k.name == "v8:lastId")) else {
                    return Err(err(e, "список значений"));
                };
                vt.text().and_then(|t| if t.is_empty() { Ok(()) } else { Err(err(vt, "тип значений списка")) })?;
                if last.req_attr("xsi:type")? != "xs:decimal" {
                    return Err(err(last, "lastId"));
                }
                let mut items = vec![an(kids.len() as i64 - 2)];
                for it in &kids[2..] {
                    if it.name != "v8:item" || it.elems().count() != 3 {
                        return Err(err(it, "элемент списка значений"));
                    }
                    let val = self.dvalue(it.req("v8:value")?)?;
                    let pres = it.text_of("v8:presentation")?;
                    let id = it.req("v8:id")?;
                    if id.req_attr("xsi:type")? != "xs:decimal" || pres.is_empty() {
                        return Err(err(it, "элемент списка значений"));
                    }
                    let pic = l(vec![a("4"), a("0"), l(vec![a("0")]), s(""), a("-1"), a("-1"), a("0"), a("0"), s("")]);
                    items.push(l(vec![a(ITEM), l(vec![s(crlf(&pres)), a("0"), val, pic, an(num(id)?), a("0"), s("")])]));
                }
                if items.len() == 1 {
                    return Err(err(e, "пустой список значений не встречен"));
                }
                Ok(wrap(VALUE_LIST, l(vec![a("6"), a(ITEM), a("0"), a("0"), l(items), l(vec![s("Pattern")]), a("0"), an(num(last)?)])))
            }
            _ => self.value(e),
        }
    }

    /// Значение `<v xsi:type>`/`<d xsi:nil>`.
    fn value(&self, e: &El) -> R<Node> {
        if e.attr("xsi:nil") == Some("true") {
            e.text()?;
            return Ok(l(vec![s("U")]));
        }
        let t = e.req_attr("xsi:type")?;
        let v = e.text()?;
        Ok(match t {
            "xs:string" => l(vec![s("S"), s(crlf(&v))]),
            "xs:decimal" => l(vec![s("N"), a(v)]),
            "xs:boolean" => l(vec![s("B"), a(if boolean(e)? { "1" } else { "0" })]),
            "xs:dateTime" => {
                let d: String = v.chars().filter(|c| c.is_ascii_digit()).collect();
                if d.len() != 14 {
                    return Err(err(e, &format!("дата {v:?}")));
                }
                l(vec![s("D"), a(d)])
            }
            t => return Err(err(e, &format!("значение типа {t} не загружается"))),
        })
    }

    fn line(&self, k: usize) -> R<Node> {
        let e = self.xlines[k];
        let st = e.req("v8ui:style")?;
        let drawing = match st.req_attr("xsi:type")? {
            "v8ui:SpreadsheetDocumentCellLineType" => false,
            "v8ui:SpreadsheetDocumentDrawingLineType" => true,
            t => return Err(err(st, &format!("вид линии {t}"))),
        };
        let t = st.text()?;
        let style = match (t.as_str(), drawing) {
            ("Dashed", true) => 2,
            ("Dotted", true) => 3,
            ("DashDotted", true) => 4,
            ("DashDottedDotted", true) => 5,
            ("None", _) => 0,
            ("Solid", _) => 1,
            ("Dotted", false) => 2,
            ("Double", false) => 3,
            ("ThinDashed", false) => 4,
            ("ThickDashed", false) => 5,
            ("LargeDashed", false) => 6,
            _ => return Err(err(st, &format!("тип линии {t}"))),
        };
        let width = super::tpl::int(e.req_attr("width")?).ok_or_else(|| err(e, "width"))?;
        let gap = match e.req_attr("gap")? {
            "false" => "0",
            "true" => "1",
            g => return Err(err(e, &format!("gap {g}"))),
        };
        Ok(l(vec![a("4"), a("0"), l(vec![a("0")]), an(style), a(width), a("0"), a(if drawing { DRAWING_LINE } else { CELL_LINE }), a(gap)]))
    }

    fn font(&self, k: usize) -> R<Node> {
        font_node(self.x, self.xfonts[k], false)
    }
}


/// Шрифт: абсолютный `{8,0,575,высота×10,0,0,0,вес,курсив,подчеркнутый,зачеркнутый,0×5,"имя",1,масштаб,0}`,
/// Windows/стиля/авто `{8,вид,маска,[{ссылка}],[значения по маске],[имя],1,масштаб}` (маска: 1 имя, 2 высота,
/// 4 жирный, 8 курсив, 16 подчеркнутый, 32 зачеркнутый, 512 масштаб).
/// `sys_in_scope` — префикс sys объявлен предком (корнем формы), у элемента его может не быть.
pub(super) fn font_node(x: &Index, e: &El, sys_in_scope: bool) -> R<Node> {
    e.text()?;
    let bad = |m: &str| err(e, &format!("шрифт: {m}"));
    // высота в десятых пункта: "11.3" → 113 (так ее пишет выгрузка: дробная часть — одна ненулевая цифра)
    let height = |h: &str| -> R<String> {
        let v = match h.split_once('.') {
            None => h.parse::<u32>().ok().map(|i| i * 10),
            Some((i, f)) if f.len() == 1 && f != "0" => i.parse::<u32>().ok().zip(f.parse::<u32>().ok()).map(|(i, f)| i * 10 + f),
            _ => None,
        };
        v.map(|v| v.to_string()).ok_or_else(|| bad(&format!("высота {h}")))
    };
    let flag = |k: &str, yes: &'static str, no: &'static str| -> R<Option<Node>> {
        match e.attr(k) {
            Some("true") => Ok(Some(a(yes))),
            Some("false") => Ok(Some(a(no))),
            None => Ok(None),
            Some(v) => Err(bad(&format!("{k}={v}"))),
        }
    };
    let kind = e.req_attr("kind")?;
    if kind == "Absolute" {
        let req = |k: &str| e.attr(k).ok_or_else(|| bad(&format!("нет {k}")));
        let mut v = vec![a("8"), a("0"), a("575"), a(height(req("height")?)?), a("0"), a("0"), a("0")];
        v.push(flag("bold", "700", "400")?.ok_or_else(|| bad("нет bold"))?);
        for k in ["italic", "underline", "strikeout"] {
            v.push(flag(k, "1", "0")?.ok_or_else(|| bad(&format!("нет {k}")))?);
        }
        v.extend((0..5).map(|_| a("0")));
        v.extend([s(req("faceName")?), a("1"), a(super::tpl::int(req("scale")?).ok_or_else(|| bad("scale"))?), a("0")]);
        return Ok(l(v));
    }
    let (k, r) = match (kind, e.attr("ref")) {
        ("AutoFont", None) => ("3", None),
        ("WindowsFont", Some(r)) => {
            match e.attr("xmlns:sys") {
                Some("http://v8.1c.ru/8.1/data/ui/fonts/system") => {}
                None if sys_in_scope => {}
                _ => return Err(bad("xmlns:sys")),
            }
            let n = ["sys:DefaultGUIFont", "sys:OEMFixedFont", "sys:ANSIFixedFont", "sys:ANSIVariableFont", "sys:SystemFont"].iter().position(|x| *x == r).ok_or_else(|| bad(r))?;
            ("1", Some(l(vec![an(n as i64)])))
        }
        // элемент стиля без ссылки — его код как есть (обратное style_ref выгрузки форм: `{<код>}` → `ref="<код>"`)
        ("StyleItem", Some(r)) if super::tpl::int(r).is_some_and(|c| !c.starts_with('-')) => ("2", Some(l(vec![a(r)]))),
        ("StyleItem", Some(r)) => {
            let n = r.strip_prefix("style:").ok_or_else(|| bad(r))?;
            let std = [("TextFont", "-20"), ("SmallTextFont", "-30"), ("NormalTextFont", "-31"), ("LargeTextFont", "-32"), ("ExtraLargeTextFont", "-33"), ("TitleLevel1","-48"),("TitleLevel2","-59"),("TitleLevel3","-50"),("TitleLevel4","-51"),("TitleLevel5","-52"),("TextLevel1","-53"),("TextLevel2","-54"),("TextLevel3","-55"),("SubtitleLevel1","-56"),("SubtitleLevel2","-57")];
            match std.iter().find(|(x, _)| *x == n) {
                Some((_, c)) => ("2", Some(l(vec![a(*c)]))),
                None => ("2", Some(l(vec![a("0"), a(x.id(&format!("StyleItem.{n}")).ok_or_else(|| bad(r))?)]))),
            }
        }
        (k, r) => return Err(bad(&format!("kind={k} ref={r:?}"))),
    };
    let mut mask = 0;
    let mut f = Vec::new();
    if let Some(h) = e.attr("height") {
        mask |= 2;
        f.push(a(height(h)?));
    }
    for (bit, name, yes, no) in [(4, "bold", "700", "400"), (8, "italic", "1", "0"), (16, "underline", "1", "0"), (32, "strikeout", "1", "0")] {
        if let Some(v) = flag(name, yes, no)? {
            mask |= bit;
            f.push(v);
        }
    }
    if let Some(face) = e.attr("faceName") {
        mask |= 1;
        f.push(s(face));
    }
    let scale = match e.attr("scale") {
        Some(sc) => {
            mask |= 512;
            super::tpl::int(sc).ok_or_else(|| bad("scale"))?
        }
        None => "100".into(),
    };
    let mut v = vec![a("8"), a(k), an(mask)];
    v.extend(r);
    v.extend(f);
    v.extend([a("1"), a(scale)]);
    Ok(l(v))
}

pub(super) fn color_node(x: &Index, e: &El) -> R<Node> {
    let t = e.text()?;
    let bad = || err(e, &format!("цвет {t:?}"));
    let ns = |p: &str| e.attr(&format!("xmlns:{p}"));
    let c = if t == "auto" {
        l(vec![a("4"), a("4"), l(vec![a("0")]), a("4")])
    } else if let Some(h) = t.strip_prefix('#') {
        super::ui::color(&format!("#{h}")).ok_or_else(bad)?
    } else if let Some(n) = t.strip_prefix("style:") {
        match crate::xml::mxl::STYLE.iter().find(|(_, v)| *v == n) {
            Some((k, _)) => l(vec![a("4"), a("3"), l(vec![an(*k)]), a("3")]),
            None => {
                let g = x.id(&format!("StyleItem.{n}")).ok_or_else(bad)?;
                l(vec![a("4"), a("3"), l(vec![a("0"), a(g)]), a("3")])
            }
        }
    } else if let Some(n) = t.strip_prefix("win:") {
        let k = crate::xml::mxl::WINDOWS.iter().find(|(_, v)| *v == n).ok_or_else(bad)?.0;
        l(vec![a("4"), a("1"), l(vec![an(k)]), a("1")])
    } else if let Some(n) = t.strip_prefix("web:") {
        let k = crate::xml::mxl::WEB.iter().find(|(_, v)| *v == n).ok_or_else(bad)?.0;
        l(vec![a("4"), a("2"), l(vec![an(k)]), a("2")])
    } else if let Some(n) = t.strip_prefix("d3p1:") {
        match ns("d3p1") {
            Some("http://v8.1c.ru/8.1/data/ui/colors/windows") => {
                let k = crate::xml::mxl::WINDOWS.iter().find(|(_, v)| *v == n).ok_or_else(bad)?.0;
                l(vec![a("4"), a("1"), l(vec![an(k)]), a("1")])
            }
            Some("http://v8.1c.ru/8.1/data/ui/colors/web") => {
                let k = crate::xml::mxl::WEB.iter().find(|(_, v)| *v == n).ok_or_else(bad)?.0;
                l(vec![a("4"), a("2"), l(vec![an(k)]), a("2")])
            }
            _ => return Err(bad()),
        }
    } else {
        return Err(bad());
    };
    Ok(c)
}

/// Рисунок `<drawing>`/`<note>` → `{{ячейка},вид,к1,с1,смещ.к1,смещ.с1,к2,с2,смещ.к2,смещ.с2,id,[картинка,размер,]авто}`.
fn drawing(b: &mut B, e: &El) -> R<(i64, Node)> {
    let ty = e.text_of("drawingType")?;
    let kind = match ty.as_str() {
        "Line" => 1,
        "Rectangle" => 2,
        "Text" => 3,
        "Picture" => 5,
        "Comment" => 6,
        "Chart" | "GanttChart" => 10,
        t => return Err(err(e, &format!("рисунок {t} не загружается"))),
    };
    let mut object = None;
    let id = num(e.req("id")?)?;
    // ячейка рисунка: формат, текст/параметр, значение, расшифровка
    let mut cell_parts = Vec::new();
    let mut geo = std::collections::HashMap::new();
    let (mut auto, mut size, mut pic) = (None, None, None);
    for k in e.elems() {
        match k.name.as_str() {
            "drawingType" | "id" | "zOrder" => {}
            "formatIndex" | "text" | "parameter" | "value" | "detailParameter" | "detailValue" => cell_parts.push(k),
            "beginRow" | "beginRowOffset" | "endRow" | "endRowOffset" | "beginColumn" | "beginColumnOffset" | "endColumn" | "endColumnOffset" => {
                geo.insert(k.name.as_str(), num(k)?);
            }
            "autoSize" => auto = Some(boolean(k)?),
            "pictureSize" => size = Some(k.text()?),
            "pictureIndex" => pic = Some(num(k)?),
            "object" if kind == 10 => object = Some(k),
            n => return Err(err(k, &format!("{n} в рисунке не загружается"))),
        }
    }
    // ячейка из выбранных элементов: разбираем их сами в порядке XML
    let cell = {
        let mut fmt = 0;
        let (mut text, mut value, mut detail, mut dvalue) = (None, None, None, None);
        for k in cell_parts {
            match k.name.as_str() {
                "formatIndex" => {
                    let i = idx(k)?;
                    fmt = b.fi(i, k)?;
                    if i != 0 && !b.dfmts.contains(&i) {
                        b.dfmts.push(i);
                    }
                }
                "text" => text = Some(ml(k)?),
                "parameter" => text = Some(l(vec![a("1"), a("1"), l(vec![s(""), s(crlf(&k.text()?))])])),
                "value" => value = Some(b.value(k)?),
                "detailParameter" => detail = Some(s(crlf(&k.text()?))),
                _ => dvalue = Some(b.dvalue(k)?),
            }
        }
        let mut flags = 0;
        let mut v = vec![a("0"), an(fmt as i64)];
        for (bit, x) in [(2, value), (4, dvalue), (8, detail), (16, text.clone())] {
            if let Some(x) = x {
                flags |= bit;
                v.push(x);
            }
        }
        if text.is_some() {
            v.push(a("0"));
        }
        v[0] = an(flags);
        l(v)
    };
    let g = |k: &str| geo.get(k).copied().ok_or_else(|| err(e, &format!("нет {k}")));
    let mut v = vec![cell, an(kind)];
    for k in ["beginColumn", "beginRow", "beginColumnOffset", "beginRowOffset", "endColumn", "endRow", "endColumnOffset", "endRowOffset"] {
        v.push(an(g(k)?));
    }
    v.push(an(id));
    let size = size.ok_or_else(|| err(e, "нет pictureSize"))?;
    if kind == 5 {
        let p = pic.ok_or_else(|| err(e, "нет pictureIndex"))?;
        let sz = match size.as_str() {
            "RealSize" => 0,
            "Stretch" => 1,
            "Proportionally" => 2,
            "Tile" => 3,
            "AutoSize" => 4,
            x => return Err(err(e, &format!("размер картинки {x}"))),
        };
        v.extend([an(p), an(sz)]);
    } else if size != "Stretch" || pic.is_some() {
        return Err(err(e, "картинка у рисунка не-картинки"));
    }
    // диаграмма: `a8b97779-…,{{11},{диаграмма версии 75}}`
    if kind == 10 {
        let o = object.ok_or_else(|| err(e, "диаграмма без <object>"))?;
        if ty == "GanttChart" {
            v.extend([a(GANTT), l(vec![super::chart::gantt(b.x, o)?])]);
        } else {
            v.extend([a(CHART), l(vec![l(vec![a("11")]), super::chart::chart(b.x, o)?])]);
        }
    }
    v.push(a(if auto.ok_or_else(|| err(e, "нет autoSize"))? { "1" } else { "0" }));
    Ok((id, l(v)))
}

pub fn record(doc: &El, x: &Index) -> R<Rec> {
    if doc.name != "document" {
        return Err("корень не <document>".into());
    }
    for (k, v) in NS {
        if doc.attr(k) != Some(v) {
            return Err(format!("<document>: {k} не {v:?}"));
        }
    }
    if doc.attrs.len() != NS.len() {
        return Err("<document>: лишние атрибуты".into());
    }
    let mut by: std::collections::HashMap<&str, Vec<&El>> = std::collections::HashMap::new();
    let mut merges: Vec<(i64, &El)> = Vec::new();
    for e in doc.elems() {
        match e.name.as_str() {
            "merge" => merges.push((0, e)),
            "horizontalUnmerge" => merges.push((1, e)),
            "verticalUnmerge" => merges.push((2, e)),
            n @ ("languageSettings" | "columns" | "rowsItem" | "drawing" | "templateMode" | "stepDirection" | "defaultFormatIndex" | "height" | "vgLevels" | "vgRows" | "vg" | "hg" | "verticalPageBreak" | "namedItem" | "printSettings" | "printArea" | "repeatRows" | "line" | "font" | "format" | "picture" | "leftHeader" | "centerHeader" | "rightHeader" | "leftFooter" | "centerFooter" | "rightFooter" | "groupsBackColor" | "groupsColor" | "headersBackColor" | "headersColor") => by.entry(n).or_default().push(e),
            n => return Err(err(e, &format!("{n} в табличном документе не загружается"))),
        }
    }
    let get = |n: &str| by.get(n).cloned().unwrap_or_default();
    let one = |n: &str| -> R<Option<&El>> {
        let v = get(n);
        if v.len() > 1 {
            return Err(format!("<{n}> повторен"));
        }
        Ok(v.first().copied())
    };
    let mut b = B {
        x,
        xfmts: get("format"),
        xfonts: get("font"),
        xlines: get("line"),
        fmts: Order::new(),
        fonts: Order::new(),
        lines: Order::new(),
        dlines: Vec::new(),
        colors: Order::new(),
        fstrings: Order::new(),
        masks: Order::new(),
        types: Order::new(),
        controls: Order::new(),
        dfmts: Vec::new(),
        max_id: 0,
    };
    // линии — в порядке XML (а не по первому использованию, как форматы)
    for i in 0..b.xlines.len() {
        b.lines.id(i);
    }
    // режим макета: без <templateMode> — 0 (выгрузка не пишет 0, у реквизита формы; макеты — всегда 1)
    let template_mode = match one("templateMode")? {
        Some(t) => if boolean(t)? { "1" } else { "0" },
        None => "0",
    };
    // языки; без <languageSettings> — единственный язык «по умолчанию» `#` (выгрузка его не пишет; имя — строка
    // ресурса core85_ru.res «Язык по умолчанию»)
    let lang = match one("languageSettings")? {
        Some(ls) => {
            let mut lang = vec![s(ls.req("currentLanguage")?.text()?), s(ls.req("defaultLanguage")?.text()?), a("0"), a("0")];
            let infos: Vec<&El> = ls.all("languageInfo").collect();
            lang[3] = an(infos.len() as i64);
            for i in infos {
                lang.extend([s(i.text_of("id")?), s(i.text_of("code")?), s(i.text_of("description")?)]);
            }
            lang.push(a("0"));
            lang
        }
        None => vec![s("#"), s(""), a("0"), a("1"), s("#"), s("Язык по умолчанию"), s("Язык по умолчанию"), a("0")],
    };
    // колонтитулы: в записи — левый верхний, левый нижний, центральные, правые
    let mut hf = Vec::new();
    for n in ["leftHeader", "leftFooter", "centerHeader", "centerFooter", "rightHeader", "rightFooter"] {
        hf.push(match one(n)? {
            Some(e) => b.cell(e, false)?,
            None => l(vec![a("0"), a("0")]),
        });
    }
    // строки
    let cols_xml = get("columns");
    let set_ids: Vec<String> = cols_xml.iter().skip(1).map(|c| c.text_of("id")).collect::<R<_>>()?;
    let mut rows = Vec::new();
    let mut rowsets = Vec::new();
    for ri in get("rowsItem") {
        let i0 = num(ri.req("index")?)?;
        let i1 = ri.child("indexTo").map(num).transpose()?.unwrap_or(i0);
        let row = ri.req("row")?;
        let mut fmt = 0;
        let mut set = None;
        let mut cells = Vec::new();
        let mut next = 0i64;
        let mut empty = false;
        for k in row.elems() {
            match k.name.as_str() {
                "columnsID" => {
                    let id = k.text()?;
                    set = Some(set_ids.iter().position(|x| *x == id).ok_or_else(|| err(k, "нет набора колонок"))?);
                }
                "formatIndex" => fmt = b.f(k)?,
                "empty" => empty = boolean(k)?,
                "c" => {
                    let col = match k.child("i") {
                        Some(i) => num(i)?,
                        None => next,
                    };
                    next = col + 1;
                    let inner = k.req("c")?;
                    if k.elems().count() != 1 + k.child("i").is_some() as usize {
                        return Err(err(k, "состав ячейки"));
                    }
                    cells.push((col, b.cell(inner, false)?));
                }
                n => return Err(err(k, &format!("{n} в строке не загружается"))),
            }
        }
        if empty != cells.is_empty() || (i1 != i0 && !empty) {
            return Err(err(row, "empty не как у выгрузки"));
        }
        for r in i0..=i1 {
            let mut v = vec![an(r), an(fmt as i64), an(cells.len() as i64)];
            for (c, n) in &cells {
                v.extend([an(*c), n.clone()]);
            }
            rows.push(v);
            if let Some(s) = set {
                rowsets.push((r, s));
            }
        }
    }
    // наборы колонок
    let mut colsets = Vec::new();
    for (k, c) in cols_xml.iter().enumerate() {
        let id = match c.child("id") {
            Some(i) if k > 0 => i.text()?,
            Some(i) => return Err(err(i, "у первого набора колонок есть id")),
            None if k == 0 => ZERO.to_string(),
            None => return Err(err(c, "нет id")),
        };
        let fmt = match c.child("formatIndex") {
            Some(f) => b.f(f)?,
            None => 0,
        };
        let size = num(c.req("size")?)?;
        let mut v = vec![an(size), an(fmt as i64), a(id), a("0")];
        let mut n = 0;
        for ci in c.all("columnsItem") {
            let col = ci.req("column")?;
            let f = b.f(col.req("formatIndex")?)?;
            v.extend([an(num(ci.req("index")?)?), an(f as i64)]);
            n += 1;
        }
        v[3] = an(n);
        colsets.push(l(v));
    }
    if colsets.is_empty() {
        return Err("нет <columns>: не встречено".into());
    }
    // рисунки
    let mut drawings = Vec::new();
    let mut maxid = 0;
    for (k, d) in get("drawing").into_iter().enumerate() {
        if num(d.req("zOrder")?)? != k as i64 + 1 {
            return Err(err(d, "zOrder не по порядку"));
        }
        let (id, n) = drawing(&mut b, d)?;
        maxid = maxid.max(id);
        drawings.push(n);
    }
    // формат по умолчанию: копия формата XML
    let default = match one("defaultFormatIndex")? {
        Some(e) => {
            let i = idx(e)?;
            let f = *b.xfmts.get(i.wrapping_sub(1)).ok_or_else(|| err(e, "нет формата"))?;
            Some(f)
        }
        None => None,
    };
    // группировки строк (vg) и колонок (hg) — одного вида (moxel FUN_1803ffb40); `g` Begin — вид начала 1
    let groups = |tag: &str| -> R<Vec<Node>> {
        let mut gs = Vec::new();
        for g in get(tag) {
            let bb = num(g.req("b")?)?;
            let e = g.child("e").map(num).transpose()?.unwrap_or(bb);
            let t = match g.child("t") {
                Some(t) => ml(t)?,
                None => l(vec![a("1"), a("0")]),
            };
            let o = match g.child("o") {
                Some(o) if !boolean(o)? => 1,
                Some(o) => return Err(err(o, "o=true не встречено")),
                None => 0,
            };
            let begin = match g.child("g") {
                Some(x) if x.text()? == "Begin" => 1,
                Some(x) => return Err(err(x, "вид начала группировки не Begin")),
                None => 0,
            };
            gs.push((bb, e, t, o, begin));
        }
        // уровень — сколько предыдущих (в порядке XML) группировок содержат эту: так у всех 200 макетов выборки
        Ok(gs
            .iter()
            .enumerate()
            .map(|(k, (bb, e, t, o, begin))| {
                let level = gs[..k].iter().filter(|(b2, e2, ..)| b2 <= bb && e <= e2).count();
                l(vec![an(*bb), an(*e), an(level as i64), t.clone(), an(*o), an(*begin)])
            })
            .collect())
    };
    let vg = groups("vg")?;
    let hg = groups("hg")?;
    let levels = vg.iter().map(|g| g.get(2).and_then(Node::atom).and_then(|x| x.parse::<i64>().ok()).unwrap_or(0) + 1).max();
    if one("vgLevels")?.map(num).transpose()? != levels {
        return Err("vgLevels не равно числу уровней группировок".into());
    }
    let height = one("height")?.map(num).transpose()?.unwrap_or(0);
    if let Some(v) = one("vgRows")? {
        if num(v)? != height {
            return Err(err(v, "vgRows не равно height"));
        }
    }
    let vbreaks: Vec<i64> = get("verticalPageBreak").into_iter().map(|p| num(p.req("position")?)).collect::<R<_>>()?;
    // объединения: ячеек (0) и с набором колонок (2)
    let mut m0 = Vec::new();
    let mut m2 = Vec::new();
    let mut m1 = Vec::new();
    for (kind, m) in merges {
        let r = num(m.req("r")?)?;
        let c = num(m.req("c")?)?;
        let h = m.child("h").map(num).transpose()?.unwrap_or(0);
        let w = m.child("w").map(num).transpose()?.unwrap_or(0);
        let mut v = vec![an(c), an(r), an(c + w), an(r + h), an(kind)];
        let id = m.child("columnsID").map(El::text).transpose()?;
        match (r, c, id) {
            // колонки целиком (строка -1) — список с набором колонок (без набора — нулевой id)
            (-1, _, id) => {
                v.push(a(id.unwrap_or_else(|| ZERO.into())));
                m2.push(((r, c), l(v)));
            }
            (_, _, Some(id)) => return Err(err(m, &format!("объединение ячеек с набором колонок {id} не встречено"))),
            // строки целиком (колонка -1) — свой список (не встречено, по аналогии)
            (_, -1, None) => m1.push(((r, c), l(v))),
            _ => m0.push(((r, c), l(v))),
        }
    }
    m1.sort_by_key(|x| x.0);
    let m1: Vec<Node> = m1.into_iter().map(|x| x.1).collect();
    // в записи — по левой верхней ячейке (строка, колонка), объединения и разъединения вперемешку
    m0.sort_by_key(|x| x.0);
    m2.sort_by_key(|x| x.0 .1);
    let m0: Vec<Node> = m0.into_iter().map(|x| x.1).collect();
    let m2: Vec<Node> = m2.into_iter().map(|x| x.1).collect();
    // именованные области
    let mut named = vec![a("0")];
    let mut nn = 0;
    for n in get("namedItem") {
        let name = n.text_of("name")?;
        // в табличном документе формы тип с префиксом (`mxl:NamedItemCells`)
        let v = match n.req_attr("xsi:type")?.trim_start_matches("mxl:") {
            "NamedItemCells" => l(vec![a("1"), area(n.req("area")?)?, a("0")]),
            "NamedItemDrawing" => l(vec![a("2"), an(num(n.req("drawingID")?)?), a("0")]),
            t => return Err(err(n, &format!("вид области {t}"))),
        };
        named.extend([s(name), v]);
        nn += 1;
    }
    named[0] = an(nn);
    let print = print_settings(one("printSettings")?)?;
    let parea = match one("printArea")? {
        Some(p) => area(p)?,
        None => l(vec![a("0"), a("-1"), a("-1"), a("-1"), a("-1"), a(ZERO)]),
    };
    let (rr0, rr1) = match one("repeatRows")? {
        Some(r) => {
            if r.text_of("type")? != "Rows" || num(r.req("beginColumn")?)? != 0 || num(r.req("endColumn")?)? != 0 {
                return Err(err(r, "повторяемые строки не как у выгрузки"));
            }
            (num(r.req("beginRow")?)?, num(r.req("endRow")?)?)
        }
        None => (0, 0),
    };
    let step = match one("stepDirection")? {
        Some(s) if s.text()? == "WithoutMove" => 3,
        Some(s) => return Err(err(s, "направление шага")),
        None => 2,
    };
    // форматы по умолчанию, особые цвета, форматы списка: так нумеруются шрифты, линии, цвета
    let default = match default {
        Some(f) => b.format(f, false)?,
        None => l(vec![a("0")]),
    };
    // цвета группировок и заголовков; не заданы — цвета формы (фон, текст)
    let mut special = [0usize; 4];
    for (k, n) in ["groupsBackColor", "groupsColor", "headersBackColor", "headersColor"].iter().enumerate() {
        special[k] = match one(n)? {
            Some(e) => b.color(e)?.atom().and_then(|v| v.parse().ok()).unwrap_or(0),
            None => b.colors.id(l(vec![a("4"), a("3"), l(vec![a(if k % 2 == 0 { "-1" } else { "-3" })]), a("3")])),
        };
    }
    let mut formats = Vec::new();
    let mut k = 0;
    while k < b.fmts.keys.len() {
        let i = b.fmts.keys[k];
        let f = b.xfmts[i - 1];
        let dr = b.dfmts.contains(&i);
        formats.push(b.format(f, dr)?);
        k += 1;
    }
    // XML-форматы, не использованные в записи, платформа не пишет — но их содержимое должно быть разобрано
    for f in &b.xfmts {
        f.touch_all();
    }
    let lines: Vec<Node> = (0..b.lines.keys.len()).map(|k| b.line(b.lines.keys[k])).collect::<R<_>>()?;
    let fonts: Vec<Node> = (0..b.fonts.keys.len()).map(|k| b.font(b.fonts.keys[k])).collect::<R<_>>()?;
    for e in b.xlines.iter().chain(&b.xfonts) {
        e.touch_all();
    }
    // картинки
    let mut pictures = Vec::new();
    for (k, p) in get("picture").into_iter().enumerate() {
        if num(p.req("index")?)? != k as i64 {
            return Err(err(p, "index не по порядку"));
        }
        let pe = p.req("picture")?;
        let (tx, ty) = (pe.attr("tx").unwrap_or("-1").to_string(), pe.attr("ty").unwrap_or("-1").to_string());
        let t = match pe.attr("t") {
            Some("false") => "0",
            None => "1",
            Some(x) => return Err(err(pe, &format!("t={x}"))),
        };
        let data = pe.text()?;
        let n = if let Some(r) = pe.attr("ref") {
            // общая картинка конфигурации или стандартная картинка платформы (как в выгрузке)
            let name = r.strip_prefix("v8ui:").ok_or_else(|| err(pe, &format!("ref {r}")))?;
            const STD: &[(&str, &str)] = &[("4b54770b-d069-4c0e-9b17-5cc2a01134d9", "Information"), ("818ab7d0-4654-4542-bd5e-fd9d1352b5a1", "SaveFile"), ("85998f14-805b-4e2b-ba19-9d79b0464042", "AppearanceCheckIcon"), ("-6", "InputFieldCalculator"), ("-13", "Print")];
            let rf = match x.id(&format!("CommonPicture.{name}")) {
                Some(g) => l(vec![a("0"), a(g)]),
                None => match STD.iter().chain(crate::xml::ui::STD_PICTURES).chain(super::form::platform::STD_PICTURES).find(|(_, n)| *n == name) {
                    Some((g, _)) if g.starts_with('-') => l(vec![a(*g)]),
                    Some((g, _)) => l(vec![a("0"), a(*g)]),
                    None => return Err(err(pe, &format!("нет картинки {name}"))),
                },
            };
            if !data.is_empty() {
                return Err(err(pe, "картинка по ссылке с данными"));
            }
            l(vec![a("4"), a("1"), rf, s(""), a(tx), a(ty), a(t), a("0"), s("")])
        } else if data.is_empty() {
            l(vec![a("4"), a("0"), l(vec![a("0")]), s(""), a(tx), a(ty), a(t), a("0"), s("")])
        } else {
            l(vec![a("4"), a("3"), l(vec![a("0")]), s(""), a(tx), a(ty), a(t), l(vec![l(vec![a(format!("#base64:{}", data.replace("\r\n", "\r\r\n")))])]), a("0"), s("")])
        };
        pictures.push(n);
    }
    // запись
    let list = |v: Vec<Node>| -> Vec<Node> {
        let mut o = vec![an(v.len() as i64)];
        o.extend(v);
        o
    };
    let mut r = vec![a("8"), a("1"), a("12"), l(lang), default];
    let mut lv = vec![an(lines.len() as i64)];
    for ln in lines {
        lv.extend([a("1"), ln, a("0")]);
    }
    r.push(l(lv));
    r.push(a("0"));
    r.extend(hf);
    r.extend([a(template_mode), an(step), an(rows.len() as i64)]);
    for row in rows {
        r.extend(row);
    }
    let mut cs = colsets.into_iter();
    r.push(cs.next().unwrap_or_else(|| l(vec![])));
    r.push(an(height));
    r.extend(list(cs.collect()));
    r.push(an(rowsets.len() as i64));
    for (row, set) in rowsets {
        r.extend([an(row), an(set as i64)]);
    }
    r.push(an(maxid.max(b.max_id)));
    r.extend(list(drawings));
    r.push(an(vg.len() as i64)); // группировки строк: (группировка, -1)
    for g in vg {
        r.extend([g, a("-1")]);
    }
    r.push(an(hg.len() as i64)); // группировки колонок: (группировка, -1)
    for g in hg {
        r.extend([g, a("-1")]);
    }
    r.extend(list(vbreaks.into_iter().map(an).collect()));
    r.push(a("0")); // горизонтальные разрывы
    for m in [m0, m1, m2] {
        r.push(l(list(m)));
    }
    r.push(l(named));
    r.push(s(""));
    r.push(print);
    r.push(parea);
    r.extend([an(rr0), an(rr1), a("0"), a("0"), a("0"), a("0")]);
    r.extend(special.map(|k| an(k as i64)));
    r.extend(list(formats));
    r.extend(list(fonts));
    r.extend(list(b.fstrings.keys));
    r.extend(list(b.types.keys));
    r.extend(list(b.controls.keys));
    // цвета стиля отчета платформа хранит в таблице значением (RGB) и отдельной парой «номер — цвет стиля»
    const REPORT_RGB: &[(&str, &str)] = &[("-25", "12971252"), ("-26", "14217976"), ("-27", "15530491"), ("-28", "8765644")];
    let mut restyle = vec![a("0")];
    let mut colors = Vec::new();
    for (k, c) in b.colors.keys.into_iter().enumerate() {
        let code = c.get(2).and_then(|v| v.get(0)).and_then(Node::atom).map(str::to_string);
        match REPORT_RGB.iter().find(|(s, _)| c.get(1).and_then(Node::atom) == Some("3") && code.as_deref() == Some(s)) {
            Some((_, rgb)) => {
                colors.push(l(vec![a("4"), a("0"), l(vec![a(*rgb)]), a("0")]));
                restyle.extend([an(k as i64), c]);
            }
            None => colors.push(c),
        }
    }
    restyle[0] = an(((restyle.len() - 1) / 2) as i64);
    r.extend(list(colors));
    r.extend(list(pictures));
    r.extend([a("0"), a("0"), s(""), a("0")]);
    r.push(l("3,0,0,100,1,1,0,1,1,0,0,0,0,0,0,0,0,0,0,0,0,\"\",0,0,0,0,0,0,0".split(',').map(|x| if x == "\"\"" { s("") } else { a(x) }).collect()));
    r.push(l(restyle));
    r.extend(list(b.masks.keys));
    r.extend(["0", "0", "1", "0", "0", "0"].map(a));
    let text = Doc::new(true, vec![l(r)]).write()?;
    let mut out = HEAD.to_vec();
    out.extend_from_slice(text.as_bytes());
    Ok(Rec::Raw(out))
}

/// Область `<area>`/`<printArea>` → `{тип,к1,с1,к2,с2,id колонок}`.
/// Табличный документ в настройках реквизита формы (`<Settings xsi:type="mxl:SpreadsheetDocument">`, содержимое —
/// `inner` без префикса `mxl:`) → `{8,1,12,…}` записи (как у макета; обратное `form_settings::moxel` выгрузки).
pub fn form_settings(inner: &str, x: &Index) -> R<Node> {
    let ns: String = NS.iter().map(|(k, v)| format!(" {k}=\"{v}\"")).collect();
    let doc = super::dom::parse(&format!("<document{ns}>{inner}</document>"))?;
    let Rec::Raw(b) = record(&doc, x)? else { return Err("запись табличного документа".into()) };
    let text = String::from_utf8(b[HEAD.len()..].to_vec()).map_err(|e| e.to_string())?;
    crate::brace::parse(text.trim_start_matches('\u{feff}'))
}

fn area(e: &El) -> R<Node> {
    let ty = match e.text_of("type")?.as_str() {
        "Rows" => 1,
        "Columns" => 2,
        "Rectangle" => 3,
        t => return Err(err(e, &format!("тип области {t}"))),
    };
    let id = e.child("columnsID").map(El::text).transpose()?.unwrap_or_else(|| ZERO.into());
    Ok(l(vec![an(ty), an(num(e.req("beginColumn")?)?), an(num(e.req("beginRow")?)?), an(num(e.req("endColumn")?)?), an(num(e.req("endRow")?)?), a(id)]))
}

/// Параметры печати `{{0,N,(ключ,значение)*N}}` по возрастанию ключа; нет — шесть отступов по 1000.
fn print_settings(p: Option<&El>) -> R<Node> {
    let Some(p) = p else {
        let mut v = vec![a("0"), a("6")];
        for k in 6..=11 {
            v.extend([an(k), l(vec![s("N"), a("1000")])]);
        }
        return Ok(l(vec![l(v)]));
    };
    let mut kv: Vec<(i64, Node)> = Vec::new();
    for c in p.elems() {
        let n = |k: i64| -> R<(i64, Node)> { Ok((k, l(vec![s("N"), an(num(c)?)]))) };
        let b = |k: i64| -> R<(i64, Node)> { Ok((k, l(vec![s("N"), a(if boolean(c)? { "1" } else { "0" })]))) };
        let t = |k: i64, m: &[(&str, i64)]| -> R<(i64, Node)> {
            let x = c.text()?;
            let v = m.iter().find(|(n, _)| *n == x).ok_or_else(|| err(c, &format!("значение {x}")))?.1;
            Ok((k, l(vec![s("N"), an(v)])))
        };
        kv.push(match c.name.as_str() {
            "paper" => n(0)?,
            "pageSize" => (0, l(vec![s("S"), s(c.text()?)])),
            "pageOrientation" => t(1, &[("Portrait", 1), ("Landscape", 2)])?,
            "scale" => n(2)?,
            "collate" => b(3)?,
            "copies" => n(4)?,
            "perPage" => n(5)?,
            "topMargin" => n(6)?,
            "leftMargin" => n(7)?,
            "bottomMargin" => n(8)?,
            "rightMargin" => n(9)?,
            "headerSize" => n(10)?,
            "footerSize" => n(11)?,
            "fitToPage" => b(12)?,
            "blackAndWhite" => b(13)?,
            "printerName" => (14, l(vec![s("S"), s(c.text()?)])),
            "paperSource" => n(15)?,
            // размеры бывают дробными (60.5, ERP 2.5.22) — число как в тексте
            "pageWidth" | "pageHeight" => {
                let x = c.text()?;
                if x.parse::<f64>().is_err() {
                    return Err(err(c, &format!("размер страницы {x}")));
                }
                (if c.name == "pageWidth" { 16 } else { 17 }, l(vec![s("N"), a(&x)]))
            }
            "printAccuracy" => t(18, &[("Auto", 0), ("Accurate", 1)])?,
            "duplexType" => t(19, &[("None", 1), ("FlipPagesLeft", 2), ("UsePrinterSettings", 4)])?,
            "pagePlacementAlternation" => t(20, &[("Auto", 0)])?,
            "firstPageNumber" => n(21)?,
            x => return Err(err(c, &format!("параметр печати {x}"))),
        });
    }
    kv.sort_by_key(|(k, _)| *k);
    let mut v = vec![a("0"), an(kv.len() as i64)];
    for (k, n) in kv {
        v.extend([an(k), n]);
    }
    Ok(l(vec![l(v)]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::xml::imp::dom::parse;

    #[test]
    fn formats_numbered_by_record_order_and_groups_levels() {
        let ns: String = NS.iter().map(|(k, v)| format!(" {k}=\"{v}\"")).collect();
        let x = format!(
            "<document{ns}><languageSettings><currentLanguage>ru</currentLanguage><defaultLanguage>ru</defaultLanguage><languageInfo><id>ru</id><code>Русский</code><description>Русский</description></languageInfo></languageSettings>\
             <columns><size>2</size><columnsItem><index>0</index><column><formatIndex>1</formatIndex></column></columnsItem></columns>\
             <rowsItem><index>0</index><row><formatIndex>2</formatIndex><c><c><f>1</f><tl><v8:item><v8:lang>ru</v8:lang><v8:content>А\nБ</v8:content></v8:item></tl></c></c></row></rowsItem>\
             <templateMode>true</templateMode><defaultFormatIndex>1</defaultFormatIndex><height>3</height><vgLevels>2</vgLevels><vgRows>3</vgRows>\
             <vg><b>0</b><e>2</e></vg><vg><b>1</b></vg><merge><r>1</r><c>1</c><w>1</w></merge><verticalUnmerge><r>0</r><c>2</c><w>1</w></verticalUnmerge>\
             <font faceName=\"Arial\" height=\"8.5\" bold=\"true\" italic=\"false\" underline=\"false\" strikeout=\"false\" kind=\"Absolute\" scale=\"100\"/>\
             <format><font>0</font><width>72</width></format><format><height>44</height><horizontalAlignment>Center</horizontalAlignment></format></document>"
        );
        let d = parse(&x).unwrap();
        let Rec::Raw(b) = record(&d, &Index::default()).unwrap() else { panic!() };
        assert!(d.unread().is_none());
        let t = String::from_utf8(b[HEAD.len()..].to_vec()).unwrap().replace("\r\n", "");
        // формат строки (XML 2) — первый в записи, формат колонки и ячейки (XML 1) — второй
        // (переводы строк выше убраны и из текста: LF ячейки стал CRLF)
        assert!(t.contains("1,2,1,0,1,1,0,{16,2,{1,1,{\"ru\",\"АБ\"}},0},{2,0,00000000-0000-0000-0000-000000000000,1,0,2},3,"), "{t}");
        assert!(t.contains("2,{320,44,6},{129,0,72},1,{8,0,575,85,0,0,0,700,"), "{t}");
        assert!(t.contains("2,{0,2,0,{1,0},0,0},-1,{1,1,1,{1,0},0,0},-1,"), "{t}");
        assert!(t.contains("{2,{2,0,3,0,2},{1,1,2,1,0}},{0},{0}"), "{t}");
    }

    #[test]
    fn detail_map_order_and_header_formatted_text() {
        let ns: String = NS.iter().map(|(k, v)| format!(r#" {k}="{v}""#)).collect();
        let pair = |k: &str| format!(r#"<v8:pair><v8:Key xsi:type="xs:string">{k}</v8:Key><v8:Value xsi:nil="true"/></v8:pair>"#);
        // XML — порядок обхода; запись — порядок обхода таблицы, в которую пары вставлены в порядке XML
        // (УчетнаяПолитика.РегистрыБУ: ИмяОбъекта и СчетКт в одной корзине; у исходной записи 1Cv8.cf они наоборот)
        let keys = ["СчетДт", "СчетПредставление", "Отбор", "ИмяОбъекта", "СчетКт", "Счет", "КорСчет"];
        let map: String = keys.iter().map(|k| pair(k)).collect();
        let x = format!(
            r#"<document{ns}><languageSettings><currentLanguage>ru</currentLanguage><defaultLanguage>ru</defaultLanguage><languageInfo><id>ru</id><code>Русский</code><description>Русский</description></languageInfo></languageSettings><columns><size>1</size></columns><rowsItem><index>0</index><row><c><c><d xsi:type="v8:Map">{map}</d></c></c></row></rowsItem><leftHeader><f>0</f><tfl><v8:item><v8:lang>ru</v8:lang><v8:content>Стр</v8:content></v8:item></tfl></leftHeader><templateMode>true</templateMode><height>1</height></document>"#
        );
        let d = parse(&x).unwrap();
        let Rec::Raw(b) = record(&d, &Index::default()).unwrap() else { panic!() };
        assert!(d.unread().is_none());
        let t = String::from_utf8(b[HEAD.len()..].to_vec()).unwrap().replace("\r\n", "");
        let rec: String = ["КорСчет", "Счет", "СчетКт", "ИмяОбъекта", "Отбор", "СчетПредставление", "СчетДт"].iter().map(|k| format!(r#",{{{{"S","{k}"}},{{"U"}}}}"#)).collect();
        assert!(t.contains(&format!(r##"{{"#",3d48feae-a9c6-4c5a-a099-9eb6477630c6,{{7{rec}}}}}"##)), "{t}");
        assert!(t.contains(r#"{16,0,{1,1,{"ru","Стр"}},1,{1,{1,1,{"ru","Стр"}},1}}"#), "{t}");
    }

    #[test]
    fn detail_array() {
        let ns: String = NS.iter().map(|(k, v)| format!(r#" {k}="{v}""#)).collect();
        let arr = r#"<d xsi:type="v8:Array"><v8:Value xsi:type="v8:Structure"><v8:Property name="А"><v8:Value xsi:type="xs:decimal">2</v8:Value></v8:Property></v8:Value><v8:Value xsi:nil="true"/></d>"#;
        let x = format!(
            r#"<document{ns}><languageSettings><currentLanguage>ru</currentLanguage><defaultLanguage>ru</defaultLanguage><languageInfo><id>ru</id><code>Русский</code><description>Русский</description></languageInfo></languageSettings><columns><size>1</size></columns><rowsItem><index>0</index><row><c><c>{arr}</c></c></row></rowsItem><templateMode>true</templateMode><height>1</height></document>"#
        );
        let d = parse(&x).unwrap();
        let Rec::Raw(b) = record(&d, &Index::default()).unwrap() else { panic!() };
        assert!(d.unread().is_none());
        let t = String::from_utf8(b[HEAD.len()..].to_vec()).unwrap().replace("\r\n", "");
        assert!(t.contains(r##"{"#",51e7a0d2-530b-11d4-b98a-008048da3034,{2,{"#",4238019d-7e49-4fc9-91db-b6b951d5cf8e,{1,{{"S","А"},{"N",2}}}},{"U"}}}"##), "{t}");
    }
}
