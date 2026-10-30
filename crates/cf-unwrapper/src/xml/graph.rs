//! Графическая схема (`Ext/Template.xml` с корнем `<GraphicalSchema>`) из скобочной записи макета.
//! Элементы — декорации и линии (обратное imp/flowchart.rs), таблицы значений — scheme.dll; не встреченные значения
//! пишутся как `?<сырое>` — файл тогда заведомо не совпадет, и это видно в xml-diff. Подробности — docs/research/xml-dcs.md.

use crate::brace::Node;
use std::collections::HashMap;

const ROOT: &str = "<GraphicalSchema xmlns=\"http://v8.1c.ru/8.3/xcf/scheme\" xmlns:pal=\"http://v8.1c.ru/8.1/data/ui/colors/palette\" xmlns:sch=\"http://v8.1c.ru/8.2/data/graphscheme\" xmlns:style=\"http://v8.1c.ru/8.1/data/ui/style\" xmlns:v8=\"http://v8.1c.ru/8.1/data/core\" xmlns:v8ui=\"http://v8.1c.ru/8.1/data/ui\" xmlns:web=\"http://v8.1c.ru/8.1/data/ui/colors/web\" xmlns:win=\"http://v8.1c.ru/8.1/data/ui/colors/windows\" xmlns:xs=\"http://www.w3.org/2001/XMLSchema\" xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\" version=\"2.21\">";

/// Вид линии `{4,0,{0},<вид>,<толщина>,<разрыв>,e45c0cd8-…,0}`.
const LINE_KIND: &str = "e45c0cd8-a878-4bcb-8e1a-af934481e1cc";

/// Цвета стиля платформы `{3,3,{-N}}`.
/// Цвета и шрифты стиля — общая таблица элементов стиля платформы (`mxl::STYLE`).
fn style_name(v: &str) -> Option<&'static str> {
    super::mxl::color_name(super::mxl::STYLE, v.parse().ok()?)
}

/// Template.xml; `None`, если запись не графическая схема или в ней элемент другого вида. Картинки элементов пишет
/// `payload::schema_pictures`. `{5,{<заголовок>},N,(<вид>,<элемент>)…,<счетчик id>}`; вид — scheme.dll 0x1800c6ec0
/// (0 Decoration, 1 Connector, 2 EntryPoint, …): в макетах встречены декорации и линии (ERP 2.5.22 ВыполнениеМаршрутныхЛистов).
pub fn schema(n: &Node) -> Option<Vec<u8>> {
    let l = n.list();
    let count: usize = atom(l.get(2)).parse().ok()?;
    if l.first()?.atom()? != "5" || count == 0 || l.len() != 4 + 2 * count || l[1].list().len() != 1 {
        return None;
    }
    let items: Vec<(&str, &Node)> = (0..count).map(|j| (atom(l.get(3 + 2 * j)), &l[4 + 2 * j])).collect();
    // id элемента → имя: на них ссылаются линии
    let names: HashMap<&str, &str> = items.iter().filter_map(|(_, it)| Some((atom(it.get(0)?.get(1)), it.get(0)?.get(3)?.str()?))).collect();
    let mut out = String::from("\u{feff}<?xml version=\"1.0\" encoding=\"UTF-8\"?>\r\n");
    out += ROOT;
    out += "\r\n";
    out += &header(l[1].get(0)?)?;
    out += "\t<Items>\r\n";
    for (kind, it) in items {
        match kind {
            "0" => decoration(it, &mut out)?,
            "1" => connector(it, &names, &mut out)?,
            _ => return None,
        }
    }
    out += "\t</Items>\r\n</GraphicalSchema>";
    Some(out.into_bytes())
}

/// Заголовок `{1,<фон>,<сетка>,<шаг>,<шаг>,<вид сетки>,K,(<код>,{"N",<значение>})…}` (обратное imp/flowchart.rs `record`).
fn header(h: &Node) -> Option<String> {
    let v = |i: usize| atom(h.get(i));
    let k: usize = v(6).parse().ok()?;
    if v(0) != "1" || h.list().len() != 7 + 2 * k {
        return None;
    }
    let b = |x: &str| pick(x, &[("0", "false"), ("1", "true")]);
    // GraphicalSchemaGridDrawMode: scheme.dll FUN_180016d40
    let grid = pick(v(5), &[("0", "None"), ("1", "Dots"), ("2", "Chess"), ("3", "Lines")]);
    // шаги сетки бывают и не 20 (ERP 2.5.22 УчетНДС, ВыполнениеМаршрутныхЛистов: 5).
    // shortcut: какой из шагов горизонтальный, по равным значениям не различить — порядок как в XML; пересмотреть на схеме с разными шагами
    let mut o = format!("\t<BackColor>{}</BackColor>\r\n\t<GridEnabled>{}</GridEnabled>\r\n\t<DrawGridMode>{grid}</DrawGridMode>\r\n\t<GridHorizontalStep>{}</GridHorizontalStep>\r\n\t<GridVerticalStep>{}</GridVerticalStep>\r\n\t<PrintParameters>\r\n", color(h.get(1)), b(v(2)), v(3), v(4));
    let mut margins: Vec<String> = Vec::new();
    for j in 0..k {
        let p = h.get(8 + 2 * j)?;
        let x = if p.list().len() == 2 && p.get(0).and_then(Node::str) == Some("N") { atom(p.get(1)).to_string() } else { format!("\u{1}{}", raw(Some(p))) };
        // коды настроек печати читает scheme.dll FUN_18004dde0 (6–9 — числа, 13 — булево, 16 — FitPageMode, FUN_180019610).
        // shortcut: какое поле у кода 6–9, взято по порядку в XML (во всех схемах поля равны 10); пересмотреть по именам свойств FUN_18004dde0.
        // Пока не выведено — неравные поля помечаются (ниже), чтобы перестановка не прошла молча
        let (tag, x) = match v(7 + 2 * j) {
            "6" => ("TopMargin".to_string(), x),
            "7" => ("LeftMargin".into(), x),
            "8" => ("BottomMargin".into(), x),
            "9" => ("RightMargin".into(), x),
            "13" => ("BlackAndWhite".into(), b(&x)),
            "16" => ("FitPageMode".into(), pick(&x, &[("0", "Auto"), ("1", "PageWidth"), ("2", "Proportionally")])),
            c => (format!("\u{1}{c}"), x),
        };
        if tag.ends_with("Margin") {
            margins.push(x.clone());
        }
        o += &format!("\t\t<{tag}>{x}</{tag}>\r\n");
    }
    if margins.windows(2).any(|w| w[0] != w[1]) {
        o += "\u{1}неравные поля печати: соответствие кодов 6–9 полям не выведено";
    }
    o += "\t</PrintParameters>\r\n";
    Some(o)
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

/// Значение атрибута: еще и кавычка.
fn esc_attr(s: &str) -> String {
    esc(s).replace('"', "&quot;")
}

fn raw(n: Option<&Node>) -> String {
    n.and_then(|x| crate::brace::write(x).ok()).map(|s| s.replace("\r\n", "")).unwrap_or_default()
}

fn atom(n: Option<&Node>) -> &str {
    n.and_then(Node::atom).unwrap_or("")
}

/// `?<сырое>` для значения, которого нет в таблице.
fn pick<'a>(v: &'a str, table: &[(&str, &'a str)]) -> String {
    table.iter().find(|(k, _)| *k == v).map_or_else(|| format!("\u{1}{v}"), |(_, x)| x.to_string())
}

fn color(n: Option<&Node>) -> String {
    let (kind, v) = (n.and_then(|c| c.get(1)).and_then(Node::atom), n.and_then(|c| c.get(2)).and_then(|x| x.get(0)).and_then(Node::atom));
    match (kind, v) {
        (Some("4"), _) => "auto".into(),
        (Some("3"), Some(v)) => style_name(v).map_or_else(|| format!("\u{1}{}", raw(n)), |s| format!("style:{s}")),
        // Абсолютный цвет хранится числом 0xBBGGRR.
        (Some("0"), Some(v)) => v.parse::<u32>().map_or_else(|_| format!("\u{1}{}", raw(n)), |c| format!("#{:02X}{:02X}{:02X}", c & 0xff, c >> 8 & 0xff, c >> 16 & 0xff)),
        // цвета Windows и web (ERP 2.5.22 ВыполнениеМаршрутныхЛистов: web:Black, web:DarkGray)
        (Some("1" | "2"), Some(_)) => n.map_or_else(String::new, super::ui::color),
        _ => format!("\u{1}{}", raw(n)),
    }
}

/// `{<версия 6|7>,<вид>,<маска>,[{ссылка}],[высота*10],[жирность],[курсив],[подчеркнутый],[зачеркнутый],["шрифт"],1,[масштаб]}`.
pub(crate) fn font(n: &Node) -> String {
    let l = n.list();
    let bad = || format!(" kind=\"\u{1}{}\"", esc_attr(&raw(Some(n))));
    let (Some(ver), Some(kind), Some(mask)) = (l.first().and_then(Node::atom), l.get(1).and_then(Node::atom), l.get(2).and_then(Node::atom).and_then(|m| m.parse::<u32>().ok())) else { return bad() };
    let mut i = 3;
    let mut a = String::new();
    let kind = match kind {
        "1" => {
            // Шрифт окна {0} — системный DefaultGUIFont.
            if raw(l.get(i)) != "{0}" {
                return bad();
            }
            i += 1;
            a += " xmlns:sys=\"http://v8.1c.ru/8.1/data/ui/fonts/system\" ref=\"sys:DefaultGUIFont\"";
            "WindowsFont"
        }
        "2" => {
            let r = l.get(i).and_then(|x| x.get(0)).and_then(Node::atom).unwrap_or("");
            i += 1;
            a += &format!(" ref=\"style:{}\"", style_name(r).map_or_else(|| format!("\u{1}{r}"), str::to_string));
            "StyleItem"
        }
        "3" => "AutoFont",
        _ => return bad(),
    };
    let mut next = || {
        let v = l.get(i);
        i += 1;
        v
    };
    let (mut height, mut flags) = (String::new(), String::new());
    if mask & 2 != 0 {
        let h = atom(next());
        height = match h.parse::<u32>() {
            Ok(h) if h % 10 == 0 => format!(" height=\"{}\"", h / 10),
            _ => format!(" height=\"\u{1}{h}\""),
        };
    }
    for (bit, name, yes) in [(4, "bold", "700"), (8, "italic", "1"), (16, "underline", "1"), (32, "strikeout", "1")] {
        if mask & bit != 0 {
            let v = atom(next());
            let s = if v == yes { "true" } else if v == "0" || (bit == 4 && v == "400") { "false" } else { "\u{1}" };
            flags += &format!(" {name}=\"{s}{}\"", if s == "\u{1}" { v } else { "" });
        }
    }
    let face = if mask & 1 != 0 { format!(" faceName=\"{}\"", esc_attr(next().and_then(Node::str).unwrap_or("\u{1}"))) } else { String::new() };
    // Хвост `1[,100]`: другие значения не встречались.
    let one = atom(next());
    let scale = if ver == "7" { atom(next()) } else { "100" };
    let tail_ok = one == "1" && scale == "100" && i == l.len();
    if mask & !63 != 0 || !tail_ok {
        return bad();
    }
    format!("{a}{face}{height}{flags} kind=\"{kind}\"")
}

/// Общее начало свойств декорации и линии (обратное imp/flowchart.rs `common`): заголовок
/// `{4,<id>,{заголовок},"Имя",<порядок обхода>}` и оформление
/// `{7,<фон>,<линия>,<текст>,<шрифт>,<подсказка>,<гор.>,<верт.>,<картинка>,<Z>,<гиперссылка>,<прозрачность>,<группа?>,0}`.
fn common(tag: &str, hdr: &Node, s: &[Node], out: &mut String) -> Option<()> {
    if atom(hdr.get(0)) != "4" {
        return None;
    }
    let (id, name, tab) = (atom(hdr.get(1)), hdr.get(3)?.str()?, atom(hdr.get(4)));
    let v = |i: usize| atom(s.get(i));
    // номер группы во всех схемах 0, его место в записи не установлено (загрузка другой номер тоже отвергает)
    let group = if v(12) == "0" && s.get(13).is_none_or(|x| x.atom() == Some("0")) { "0".to_string() } else { format!("\u{1}{}", v(12)) };
    let b = |x: &str| pick(x, &[("0", "false"), ("1", "true")]);
    *out += &format!("\t\t<{tag} id=\"{id}\">\r\n\t\t\t<Properties>\r\n\t\t\t\t<Name>{}</Name>\r\n", esc(name));
    ml("Title", hdr.get(2)?, out);
    ml("ToolTip", s.get(5)?, out);
    *out += &format!("\t\t\t\t<TabOrder>{tab}</TabOrder>\r\n");
    *out += &format!("\t\t\t\t<BackColor>{}</BackColor>\r\n\t\t\t\t<TextColor>{}</TextColor>\r\n\t\t\t\t<LineColor>{}</LineColor>\r\n", color(s.get(1)), color(s.get(3)), color(s.get(2)));
    *out += &format!("\t\t\t\t<GroupNumber>{group}</GroupNumber>\r\n\t\t\t\t<ZOrder>{}</ZOrder>\r\n", v(9));
    *out += &format!("\t\t\t\t<Hyperlink>{}</Hyperlink>\r\n\t\t\t\t<Transparent>{}</Transparent>\r\n", b(v(10)), b(v(11)));
    *out += &format!("\t\t\t\t<Font{}/>\r\n", font(s.get(4)?));
    *out += &format!("\t\t\t\t<HorizontalAlign>{}</HorizontalAlign>\r\n", pick(v(6), &[("0", "Left"), ("1", "Center"), ("2", "Right")]));
    // VerticalAlign: mngcore 0x18075d580
    *out += &format!("\t\t\t\t<VerticalAlign>{}</VerticalAlign>\r\n", pick(v(7), &[("0", "Top"), ("1", "Center"), ("2", "Bottom")]));
    // GraphicalSchemaItemPictureLocation: scheme.dll FUN_180017150
    *out += &format!("\t\t\t\t<PictureLocation>{}</PictureLocation>\r\n", pick(v(8), &[("1", "Left"), ("2", "Top"), ("3", "Right"), ("4", "Bottom"), ("5", "Center")]));
    Some(())
}

/// Многоязычная строка `{1,N,{"<язык>","<текст>"}…}`.
fn ml(tag: &str, n: &Node, out: &mut String) {
    let l = n.list();
    if atom(l.get(1)) == "0" {
        *out += &format!("\t\t\t\t<{tag}/>\r\n");
        return;
    }
    *out += &format!("\t\t\t\t<{tag}>\r\n");
    for pair in l.iter().skip(2) {
        let (lang, text) = (pair.get(0).and_then(Node::str).unwrap_or("\u{1}"), pair.get(1).and_then(Node::str).unwrap_or("\u{1}"));
        *out += &format!("\t\t\t\t\t<v8:item>\r\n\t\t\t\t\t\t<v8:lang>{}</v8:lang>\r\n\t\t\t\t\t\t<v8:content>{}</v8:content>\r\n\t\t\t\t\t</v8:item>\r\n", esc(lang), esc(&text.replace("\r\n", "\n")));
    }
    *out += &format!("\t\t\t\t</{tag}>\r\n");
}

/// Декорация `{<заголовок>,2,{{<оформление>,6,л,в,п,н,<размер картинки>,{картинка},<прозрачность>,<фигура>,<отражение>,<угол>}}}`.
fn decoration(it: &Node, out: &mut String) -> Option<()> {
    if atom(it.get(1)) != "2" {
        return None;
    }
    let p = it.get(2)?.get(0)?.list();
    let s = p.first()?.list();
    let t = |i: usize| atom(p.get(i));
    // после картинки — повтор прозрачности оформления (так во всех схемах)
    let ok = t(1) == "6" && p.len() == 12 && atom(s.get(11)) == t(8);
    common("Decoration", it.get(0)?, s, out)?;
    *out += &format!("\t\t\t\t<Location top=\"{}\" left=\"{}\" bottom=\"{}\" right=\"{}\"/>\r\n", t(3), t(2), t(5), t(4));
    // Картинка `{<версия>,<?>,{0},"",-1,-1,<прозрачность>,{{#base64:...}},...}` или без данных.
    let pic = p.get(7)?.list();
    match pic.get(7).and_then(|x| x.get(0)).and_then(|x| x.get(0)).and_then(Node::atom).and_then(|a| a.strip_prefix("#base64:")) {
        Some(b64) => {
            let data = super::base64(b64).ok()?;
            let ext = super::payload::picture_ext(&data).map_or_else(|| "\u{1}".to_string(), str::to_string);
            let lt = pick(atom(pic.get(6)), &[("0", "false")]);
            *out += &format!("\t\t\t\t<Picture>\r\n\t\t\t\t\t<Abs xmlns=\"http://v8.1c.ru/8.3/xcf/readable\">Picture.{ext}</Abs>\r\n\t\t\t\t\t<LoadTransparent xmlns=\"http://v8.1c.ru/8.3/xcf/readable\">{lt}</LoadTransparent>\r\n\t\t\t\t</Picture>\r\n");
        }
        None if atom(pic.get(6)) == "1" => *out += "\t\t\t\t<Picture/>\r\n",
        None => *out += &format!("\t\t\t\t<Picture>\u{1}{}</Picture>\r\n", esc(&raw(p.get(7)))),
    }
    *out += &format!("\t\t\t\t<PictureSize>{}</PictureSize>\r\n", pick(t(6), &[("0", "RealSize"), ("2", "Proportionally"), ("4", "AutoSize")]));
    // GraphicalSchemaShapes: scheme.dll FUN_180017360
    const SHAPES: &[(&str, &str)] = &[("0", "None"), ("1", "File"), ("2", "Folder"), ("3", "RightArrow"), ("4", "LeftArrow"), ("5", "UpArrow"), ("6", "DownArrow"), ("7", "LeftRightArrow"), ("8", "UpDownArrow"), ("9", "VerticalBrackets"), ("10", "Document"), ("11", "Block"), ("12", "HorizontalBrackets"), ("13", "Ellipse")];
    let shape = if ok { pick(t(9), SHAPES) } else { format!("\u{1}{}", raw(Some(it))) };
    *out += &format!("\t\t\t\t<Shape>{shape}</Shape>\r\n\t\t\t\t<FlipMode>{}</FlipMode>\r\n\t\t\t\t<Angle xsi:type=\"xs:decimal\">{}</Angle>\r\n", t(10), t(11));
    *out += "\t\t\t</Properties>\r\n\t\t</Decoration>\r\n";
    Some(())
}

/// Линия `{<заголовок>,3,<откуда>,<из порта «истина»>,<куда>,<декоративная>,{{<оформление>,6,N,<точки>…,<линия>,<текст>,
/// <порт откуда>,<порт куда>,M,(<x1>,<y1>,<x2>,<y2>,<номер отрезка>)…,<стрелка начала>,<стрелка конца>}}}` (обратное imp/flowchart.rs).
fn connector(it: &Node, names: &HashMap<&str, &str>, out: &mut String) -> Option<()> {
    let p = it.get(6)?.get(0)?.list();
    let t = |i: usize| atom(p.get(i));
    let np: usize = t(2).parse().ok()?;
    let i = 3 + 2 * np;
    let nm: usize = t(i + 4).parse().ok()?;
    // в макете нет условий — линия не выходит из порта «истина»
    if atom(it.get(1)) != "3" || t(1) != "6" || atom(it.get(3)) != "0" || p.len() != i + 7 + 5 * nm {
        return None;
    }
    common("ConnectionLine", it.get(0)?, p.first()?.list(), out)?;
    *out += "\t\t\t\t<PivotPoints>\r\n";
    for j in 0..np {
        *out += &format!("\t\t\t\t\t<Point x=\"{}\" y=\"{}\"/>\r\n", t(3 + 2 * j), t(4 + 2 * j));
    }
    *out += "\t\t\t\t</PivotPoints>\r\n";
    if nm > 0 {
        *out += "\t\t\t\t<ManualyMovedSegments>\r\n";
        for k in (i + 5..i + 5 + 5 * nm).step_by(5) {
            *out += &format!("\t\t\t\t\t<Segment index=\"{}\">\r\n\t\t\t\t\t\t<Start x=\"{}\" y=\"{}\"/>\r\n\t\t\t\t\t\t<End x=\"{}\" y=\"{}\"/>\r\n\t\t\t\t\t</Segment>\r\n", t(k + 4), t(k), t(k + 1), t(k + 2), t(k + 3));
        }
        *out += "\t\t\t\t</ManualyMovedSegments>\r\n";
    }
    let end = |id: &str, port: &str| {
        let item = match (id, names.get(id)) {
            ("-1", _) => "\t\t\t\t\t\t<Item/>\r\n".to_string(),
            (_, Some(n)) => format!("\t\t\t\t\t\t<Item>{}</Item>\r\n", esc(n)),
            (id, None) => format!("\t\t\t\t\t\t<Item>\u{1}{id}</Item>\r\n"),
        };
        format!("{item}\t\t\t\t\t\t<PortIndex>{port}</PortIndex>\r\n")
    };
    *out += &format!("\t\t\t\t<Connect>\r\n\t\t\t\t\t<From>\r\n{}\t\t\t\t\t</From>\r\n\t\t\t\t\t<To>\r\n{}\t\t\t\t\t</To>\r\n\t\t\t\t</Connect>\r\n", end(atom(it.get(2)), t(i + 2)), end(atom(it.get(4)), t(i + 3)));
    // ConnectorLineType: scheme.dll FUN_180016f00
    let ln = p.get(i)?.list();
    let lv = |k: usize| atom(ln.get(k));
    let style = if ln.len() == 8 && lv(1) == "0" && raw(ln.get(2)) == "{0}" && lv(6) == LINE_KIND && lv(7) == "0" {
        pick(lv(3), &[("0", "None"), ("1", "Solid"), ("2", "Dashed"), ("3", "Dotted"), ("4", "DashDotted"), ("5", "DashDottedDotted")])
    } else {
        format!("\u{1}{}", raw(p.get(i)))
    };
    let b = |x: &str| pick(x, &[("0", "false"), ("1", "true")]);
    *out += &format!("\t\t\t\t<Line width=\"{}\" gap=\"{}\">\r\n\t\t\t\t\t<v8ui:style xsi:type=\"sch:ConnectorLineType\">{style}</v8ui:style>\r\n\t\t\t\t</Line>\r\n", lv(4), b(lv(5)));
    // ConnectorTextLocation: scheme.dll FUN_180019a60; ArrowStyle: FUN_180018e90
    let arrow = |x: &str| pick(x, &[("0", "None"), ("1", "Filled"), ("2", "Blank")]);
    let j = i + 5 + 5 * nm;
    *out += &format!("\t\t\t\t<DecorativeLine>{}</DecorativeLine>\r\n\t\t\t\t<TextLocation>{}</TextLocation>\r\n", b(atom(it.get(5))), pick(t(i + 1), &[("0", "FirstSegment"), ("1", "Middle")]));
    *out += &format!("\t\t\t\t<BeginArrow>{}</BeginArrow>\r\n\t\t\t\t<EndArrow>{}</EndArrow>\r\n\t\t\t</Properties>\r\n\t\t</ConnectionLine>\r\n", arrow(t(j)), arrow(t(j + 1)));
    Some(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fonts_and_colors() {
        let f = |s: &str| font(&crate::brace::parse(s).unwrap());
        assert_eq!(f("{7,3,0,1,100}"), " kind=\"AutoFont\"");
        assert_eq!(f("{7,2,62,{-31},80,700,0,0,0,1,100}"), " ref=\"style:NormalTextFont\" height=\"8\" bold=\"true\" italic=\"false\" underline=\"false\" strikeout=\"false\" kind=\"StyleItem\"");
        assert_eq!(
            f("{6,1,63,{0},200,700,0,0,0,\"Arial\",1}"),
            " xmlns:sys=\"http://v8.1c.ru/8.1/data/ui/fonts/system\" ref=\"sys:DefaultGUIFont\" faceName=\"Arial\" height=\"20\" bold=\"true\" italic=\"false\" underline=\"false\" strikeout=\"false\" kind=\"WindowsFont\""
        );
        assert!(f("{7,3,0,1,90}").contains('\u{1}'));
        assert!(f("{6,1,63,{0},200,700,0,0,0,\"A\"\"<\",1}").contains(" faceName=\"A&quot;&lt;\""));
        assert!(f("{6,9,\"x\"}").starts_with(" kind=\"\u{1}{6,9,&quot;x&quot;}"));
        let c = |s: &str| color(Some(&crate::brace::parse(s).unwrap()));
        assert_eq!(c("{3,0,{6723891}}"), "#339966");
        assert_eq!(c("{3,3,{-22}}"), "style:BorderColor");
        assert_eq!(c("{3,4,{-14}}"), "auto");
        assert!(c("{3,3,{-99}}").starts_with('\u{1}'));
        assert_eq!(c("{3,2,{8}}"), "web:Black");
    }

    /// Synthetic decoration and connector with independent coordinates and IDs.
    #[test]
    fn decoration_and_connector() {
        let v = "{7,{3,4,{0}},{3,3,{-22}},{3,3,{-3}},{7,3,0,1,100},{1,0},1,0,1,1,0,0,0,0}";
        let d = format!("{{{{4,306,{{1,0}},\"Ф\",158}},2,{{{{{v},6,215,40,345,220,4,{{4,0,{{0}},\"\",-1,-1,1,0,\"\"}},0,11,0,0}}}}}}");
        let ln = format!("{{{{4,307,{{1,0}},\"Л\",159}},3,306,0,-1,1,{{{{{v},6,2,90,80,215,80,{{4,0,{{0}},2,1,0,{LINE_KIND},0}},0,3,1,1,365,80,365,300,1,0,1}}}}}}");
        let t = format!("{{5,{{{{1,{{3,3,{{-1}}}},1,5,5,0,2,6,{{\"N\",10}},16,{{\"N\",0}}}}}},2,0,{d},1,{ln},308}}");
        let x = String::from_utf8(schema(&crate::brace::parse(&t).unwrap()).unwrap()).unwrap();
        assert!(!x.contains('\u{1}'), "{x}");
        assert!(x.contains("\t<GridEnabled>true</GridEnabled>\r\n\t<DrawGridMode>None</DrawGridMode>\r\n\t<GridHorizontalStep>5</GridHorizontalStep>"), "{x}");
        assert!(x.contains("<VerticalAlign>Top</VerticalAlign>") && x.contains("<Shape>Block</Shape>"), "{x}");
        assert!(x.contains("<Segment index=\"1\">\r\n\t\t\t\t\t\t<Start x=\"365\" y=\"80\"/>\r\n\t\t\t\t\t\t<End x=\"365\" y=\"300\"/>"), "{x}");
        assert!(x.contains("<From>\r\n\t\t\t\t\t\t<Item>Ф</Item>\r\n\t\t\t\t\t\t<PortIndex>3</PortIndex>\r\n\t\t\t\t\t</From>\r\n\t\t\t\t\t<To>\r\n\t\t\t\t\t\t<Item/>"), "{x}");
        assert!(x.contains(">Dashed<") && x.contains("<BeginArrow>None</BeginArrow>\r\n\t\t\t\t<EndArrow>Filled</EndArrow>\r\n\t\t\t</Properties>\r\n\t\t</ConnectionLine>"), "{x}");
        // элемент другого вида (старт) — незнакомое устройство
        assert!(schema(&crate::brace::parse(&t.replacen(",1,{{4,307", ",2,{{4,307", 1)).unwrap()).is_none());
    }
}
