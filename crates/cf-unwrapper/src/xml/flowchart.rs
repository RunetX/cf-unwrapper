//! Карта маршрута бизнес-процесса `Ext/Flowchart.xml` (запись `<id>.7`). Устройство — docs/research/xml-ext.md.
//!
//! Значения, которых нет в эталоне БП 3.0.171 (цвета, шрифты, виды фигур и т. п.), пишутся как есть
//! в скобочном виде с меткой `UNK`: такой файл не выгружается (отказ с причиной), а не угадывается.

use super::ext::esc;
use super::UNK;
use crate::brace::{self, Node};
use std::collections::HashMap;

const HEAD: &str = "<GraphicalSchema xmlns=\"http://v8.1c.ru/8.3/xcf/scheme\" xmlns:pal=\"http://v8.1c.ru/8.1/data/ui/colors/palette\" xmlns:sch=\"http://v8.1c.ru/8.2/data/graphscheme\" xmlns:style=\"http://v8.1c.ru/8.1/data/ui/style\" xmlns:v8=\"http://v8.1c.ru/8.1/data/core\" xmlns:v8ui=\"http://v8.1c.ru/8.1/data/ui\" xmlns:web=\"http://v8.1c.ru/8.1/data/ui/colors/web\" xmlns:win=\"http://v8.1c.ru/8.1/data/ui/colors/windows\" xmlns:xs=\"http://www.w3.org/2001/XMLSchema\" xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\" version=\"2.21\">\r\n";

fn raw(n: &Node) -> String {
    brace::write(n).unwrap_or_default().replace("\r\n", "")
}

/// Незнакомое значение — с меткой.
fn unk(v: &str) -> String {
    format!("{UNK}{v}")
}

fn at<'a>(n: &'a Node, i: usize) -> Result<&'a Node, String> {
    n.get(i).ok_or_else(|| format!("карта маршрута: нет элемента {i}"))
}

fn a<'a>(n: &'a Node, i: usize) -> Result<&'a str, String> {
    at(n, i)?.atom().ok_or_else(|| format!("карта маршрута: ожидалось число в позиции {i}"))
}

fn s<'a>(n: &'a Node, i: usize) -> Result<&'a str, String> {
    at(n, i)?.str().ok_or_else(|| format!("карта маршрута: ожидалась строка в позиции {i}"))
}

fn bool01(v: &str) -> String {
    match v {
        "0" => "false".into(),
        "1" => "true".into(),
        v => unk(v),
    }
}

/// Цвет `{3,<вид>,{<значение>}}`: 4 — авто, 3 — цвет стиля, 0 — абсолютный.
fn color(n: &Node) -> String {
    let v = n.get(2).and_then(|x| x.get(0)).and_then(Node::atom);
    match (n.get(1).and_then(Node::atom), v) {
        (Some("4"), Some("0")) => "auto".into(),
        (Some("0"), Some("0")) => "#000000".into(),
        (Some("3"), Some("-3")) => "style:BorderColor".into(),
        (Some("3"), Some("-10")) => "style:FieldBackColor".into(),
        (Some("3"), Some("-22")) => "style:FormTextColor".into(),
        _ => unk(&raw(n)),
    }
}

fn font(n: &Node) -> String {
    match raw(n).as_str() {
        "{7,1,0,{0},1,100}" => "<Font xmlns:sys=\"http://v8.1c.ru/8.1/data/ui/fonts/system\" ref=\"sys:DefaultGUIFont\" kind=\"WindowsFont\"/>".into(),
        "{7,3,0,1,100}" => "<Font kind=\"AutoFont\"/>".into(),
        r => format!("<Font>{UNK}{r}</Font>"),
    }
}

/// Многоязычная строка `{1,N,{"ru","текст"},...}`.
fn ml(tag: &str, n: &Node, t: &str, o: &mut String) -> Result<(), String> {
    let k: usize = a(n, 1)?.parse().map_err(|_| "карта маршрута: строка")?;
    if k == 0 {
        o.push_str(&format!("{t}<{tag}/>\r\n"));
        return Ok(());
    }
    o.push_str(&format!("{t}<{tag}>\r\n"));
    for j in 0..k {
        let it = at(n, 2 + j)?;
        o.push_str(&format!("{t}\t<v8:item>\r\n{t}\t\t<v8:lang>{}</v8:lang>\r\n{t}\t\t<v8:content>{}</v8:content>\r\n{t}\t</v8:item>\r\n", s(it, 0)?, esc(s(it, 1)?)));
    }
    o.push_str(&format!("{t}</{tag}>\r\n"));
    Ok(())
}

/// Линия рамки/соединения `{4,0,{0},<вид>,<толщина>,<разрыв>,<uuid>,0}`.
fn line(tag: &str, n: &Node, t: &str, o: &mut String) -> Result<(), String> {
    let style = match a(n, 3)? {
        "1" => "Solid".into(),
        "2" => "Dashed".into(),
        v => unk(v),
    };
    o.push_str(&format!("{t}<{tag} width=\"{}\" gap=\"{}\">\r\n{t}\t<v8ui:style xsi:type=\"sch:ConnectorLineType\">{style}</v8ui:style>\r\n{t}</{tag}>\r\n", a(n, 4)?, bool01(a(n, 5)?)));
    Ok(())
}

/// Общие свойства: заголовок `{4,<id>,<заголовок>,"Имя",<порядок обхода>}` и оформление
/// `{7,<фон>,<текст>,<линия>,<шрифт>,<подсказка>,<гор.>,<верт.>,<картинка>,<Z>,<группа>,<прозрачность>,<гиперссылка>,0}`.
fn common(h: &Node, v: &Node, t: &str, o: &mut String) -> Result<(), String> {
    o.push_str(&format!("{t}<Name>{}</Name>\r\n", esc(s(h, 3)?)));
    ml("Title", at(h, 2)?, t, o)?;
    ml("ToolTip", at(v, 5)?, t, o)?;
    let halign = match a(v, 6)? {
        "0" => "Left".into(),
        "1" => "Center".into(),
        "2" => "Right".into(),
        x => unk(x),
    };
    // VerticalAlign: mngcore 0x18075d580
    let valign = match a(v, 7)? {
        "0" => "Top".into(),
        "1" => "Center".into(),
        "2" => "Bottom".into(),
        x => unk(x),
    };
    let ploc = if a(v, 8)? == "1" { "Left".into() } else { unk(a(v, 8)?) };
    o.push_str(&format!(
        "{t}<TabOrder>{}</TabOrder>\r\n{t}<BackColor>{}</BackColor>\r\n{t}<TextColor>{}</TextColor>\r\n{t}<LineColor>{}</LineColor>\r\n{t}<GroupNumber>{}</GroupNumber>\r\n{t}<ZOrder>{}</ZOrder>\r\n{t}<Hyperlink>{}</Hyperlink>\r\n{t}<Transparent>{}</Transparent>\r\n{t}{}\r\n{t}<HorizontalAlign>{halign}</HorizontalAlign>\r\n{t}<VerticalAlign>{valign}</VerticalAlign>\r\n{t}<PictureLocation>{ploc}</PictureLocation>\r\n",
        a(h, 4)?, color(at(v, 1)?), color(at(v, 2)?), color(at(v, 3)?), a(v, 10)?, a(v, 9)?, bool01(a(v, 12)?), bool01(a(v, 11)?), font(at(v, 4)?)
    ));
    Ok(())
}

fn location(n: &Node, i: usize, t: &str, o: &mut String) -> Result<(), String> {
    // left, top, right, bottom
    o.push_str(&format!("{t}<Location top=\"{}\" left=\"{}\" bottom=\"{}\" right=\"{}\"/>\r\n", a(n, i + 1)?, a(n, i)?, a(n, i + 3)?, a(n, i + 2)?));
    Ok(())
}

fn picture(n: &Node, t: &str, o: &mut String) {
    match raw(n).as_str() {
        "{4,0,{0},\"\",-1,-1,1,0,\"\"}" => o.push_str(&format!("{t}<Picture/>\r\n{t}<PictureSize>AutoSize</PictureSize>\r\n")),
        r => o.push_str(&format!("{t}<Picture>{UNK}{}</Picture>\r\n", esc(r))),
    }
}

/// События `{N,{<номер>,"обработчик"},...}`: в XML — все события вида по порядку номеров.
fn events(n: &Node, names: &[&str], o: &mut String) -> Result<(), String> {
    let mut h: HashMap<usize, &str> = HashMap::new();
    for e in n.list().iter().skip(1) {
        h.insert(a(e, 0)?.parse().map_err(|_| "карта маршрута: номер события")?, s(e, 1)?);
    }
    o.push_str("\t\t\t<Events>\r\n");
    for (k, name) in names.iter().enumerate() {
        match h.get(&k) {
            Some(x) => o.push_str(&format!("\t\t\t\t<Event name=\"{name}\">{}</Event>\r\n", esc(x))),
            None => o.push_str(&format!("\t\t\t\t<Event name=\"{name}\"/>\r\n")),
        }
    }
    o.push_str("\t\t\t</Events>\r\n");
    Ok(())
}

/// `{5,{{1,<фон>,<сетка>,<шаг по гор.>,<шаг по верт.>,<вид сетки>,N,<номер>,{"N",<значение>},...}},N,<вид>,{элемент},...,<следующий id>}`;
/// `addressing` — полное имя реквизита адресации по uuid и его порядок у задачи.
/// `value` — ссылка времени разработки в значении реквизита адресации (`Catalog.X.Имя`).
pub fn flowchart(n: &Node, addressing: &dyn Fn(&str) -> (String, usize), value: &dyn Fn(&Node) -> Option<String>) -> Result<String, String> {
    // карта, сохраненная 8.5: цвета и шрифты в прежние версии
    let mut v85 = n.clone();
    super::form_v59::styles(&mut v85)?;
    let n = &v85;
    let p = at(at(n, 1)?, 0)?;
    // GraphicalSchemaGridDrawMode: scheme.dll FUN_180016d40
    let grid = match a(p, 5)? {
        "0" => "None".into(),
        "1" => "Dots".into(),
        "2" => "Chess".into(),
        "3" => "Lines".into(),
        x => unk(x),
    };
    let mut o = format!(
        "{HEAD}\t<BackColor>{}</BackColor>\r\n\t<GridEnabled>{}</GridEnabled>\r\n\t<DrawGridMode>{grid}</DrawGridMode>\r\n\t<GridHorizontalStep>{}</GridHorizontalStep>\r\n\t<GridVerticalStep>{}</GridVerticalStep>\r\n\t<PrintParameters>\r\n",
        color(at(p, 1)?), bool01(a(p, 2)?), a(p, 3)?, a(p, 4)?
    );
    let k: usize = a(p, 6)?.parse().map_err(|_| "карта маршрута: параметры печати")?;
    for j in 0..k {
        let v = at(p, 8 + 2 * j)?.get(1).and_then(Node::atom).unwrap_or("\u{1}");
        let (tag, v) = match a(p, 7 + 2 * j)? {
            "6" => ("TopMargin".into(), v.into()),
            "7" => ("LeftMargin".into(), v.into()),
            "8" => ("BottomMargin".into(), v.into()),
            "9" => ("RightMargin".into(), v.into()),
            "13" => ("BlackAndWhite".into(), bool01(v)),
            "16" => ("FitPageMode".into(), if v == "0" { "Auto".into() } else { unk(v) }),
            x => (unk(x), v.to_string()),
        };
        o.push_str(&format!("\t\t<{tag}>{v}</{tag}>\r\n"));
    }
    o.push_str("\t</PrintParameters>\r\n\t<Items>\r\n");
    let count: usize = a(n, 2)?.parse().map_err(|_| "карта маршрута: число элементов")?;
    let items: Vec<(&str, &Node)> = (0..count).map(|j| Ok((a(n, 3 + 2 * j)?, at(n, 4 + 2 * j)?))).collect::<Result<_, String>>()?;
    // id элемента → имя: на них ссылаются линии.
    let head = |ty: &str, it: &Node| -> Result<Node, String> { Ok(if matches!(ty, "0" | "1") { at(it, 0)?.clone() } else { at(at(it, 0)?, 0)?.clone() }) };
    let mut names = HashMap::new();
    for (ty, it) in &items {
        let h = head(ty, it)?;
        names.insert(a(&h, 1)?.to_string(), s(&h, 3)?.to_string());
    }
    let t = "\t\t\t\t";
    for (ty, it) in items {
        let h = head(ty, it)?;
        let id = a(&h, 1)?;
        let tag = match ty {
            "0" => "Decoration",
            "1" => "ConnectionLine",
            "2" => "Start",
            "3" => "Completion",
            "4" => "Condition",
            "5" => "Activity",
            // ERP 2.5.22: СогласованиеЗакупки, ЗаявкаСотрудникаСправкаСМестаРаботы
            "7" => "Split",
            "8" => "Join",
            "9" => "Processing",
            x => {
                o.push_str(&format!("\t\t<Unknown kind=\"{x}\">{UNK}{}</Unknown>\r\n", esc(&raw(it))));
                continue;
            }
        };
        match ty {
            "0" => {
                // {<заголовок>,2,{{<оформление>,6,<место>,4,<картинка>,1,<фигура>,<отражение>,<угол>}}}
                let b = at(at(it, 2)?, 0)?;
                o.push_str(&format!("\t\t<{tag} id=\"{id}\">\r\n\t\t\t<Properties>\r\n"));
                common(&h, at(b, 0)?, t, &mut o)?;
                location(b, 2, t, &mut o)?;
                picture(at(b, 7)?, t, &mut o);
                // фигура: scheme.dll 0x180017360
                const SHAPES: &[&str] = &["None", "File", "Folder", "RightArrow", "LeftArrow", "UpArrow", "DownArrow", "LeftRightArrow", "UpDownArrow", "VerticalBrackets", "Document", "Block", "HorizontalBrackets", "Ellipse"];
                let shape = a(b, 9)?.parse::<usize>().ok().and_then(|k| SHAPES.get(k)).map_or_else(|| unk(a(b, 9).unwrap_or_default()), |s| s.to_string());
                o.push_str(&format!("{t}<Shape>{shape}</Shape>\r\n{t}<FlipMode>{}</FlipMode>\r\n{t}<Angle xsi:type=\"xs:decimal\">{}</Angle>\r\n\t\t\t</Properties>\r\n", a(b, 10)?, a(b, 11)?));
            }
            "1" => {
                // {<заголовок>,3,<откуда>,?,<куда>,<декоративная>,{{<оформление>,6,N,<точки>...,<линия>,<текст>,<порт откуда>,<порт куда>,M,<отрезки>...,<начало>,<конец>}}}
                let b = at(at(it, 6)?, 0)?;
                o.push_str(&format!("\t\t<{tag} id=\"{id}\">\r\n\t\t\t<Properties>\r\n"));
                common(&h, at(b, 0)?, t, &mut o)?;
                let np: usize = a(b, 2)?.parse().map_err(|_| "карта маршрута: точки")?;
                o.push_str(&format!("{t}<PivotPoints>\r\n"));
                for j in 0..np {
                    o.push_str(&format!("{t}\t<Point x=\"{}\" y=\"{}\"/>\r\n", a(b, 3 + 2 * j)?, a(b, 4 + 2 * j)?));
                }
                o.push_str(&format!("{t}</PivotPoints>\r\n"));
                let mut i = 3 + 2 * np;
                let style = at(b, i)?;
                let text_loc = match a(b, i + 1)? {
                    "0" => "FirstSegment".into(),
                    x => unk(x),
                };
                let (pf, pt) = (a(b, i + 2)?, a(b, i + 3)?);
                let nm: usize = a(b, i + 4)?.parse().map_err(|_| "карта маршрута: отрезки")?;
                i += 5;
                if nm > 0 {
                    o.push_str(&format!("{t}<ManualyMovedSegments>\r\n"));
                    for _ in 0..nm {
                        o.push_str(&format!("{t}\t<Segment index=\"{}\">\r\n{t}\t\t<Start x=\"{}\" y=\"{}\"/>\r\n{t}\t\t<End x=\"{}\" y=\"{}\"/>\r\n{t}\t</Segment>\r\n", a(b, i + 4)?, a(b, i)?, a(b, i + 1)?, a(b, i + 2)?, a(b, i + 3)?));
                        i += 5;
                    }
                    o.push_str(&format!("{t}</ManualyMovedSegments>\r\n"));
                }
                let end = |x: &str| -> Result<String, String> { Ok(names.get(x).map_or(format!("{t}\t\t<Item/>\r\n"), |n| format!("{t}\t\t<Item>{}</Item>\r\n", esc(n)))) };
                o.push_str(&format!("{t}<Connect>\r\n{t}\t<From>\r\n{}{t}\t\t<PortIndex>{pf}</PortIndex>\r\n{t}\t</From>\r\n{t}\t<To>\r\n{}{t}\t\t<PortIndex>{pt}</PortIndex>\r\n{t}\t</To>\r\n{t}</Connect>\r\n", end(a(it, 2)?)?, end(a(it, 4)?)?));
                line("Line", style, t, &mut o)?;
                let arrow = |x: &str| match x {
                    "0" => "None".to_string(),
                    "1" => "Filled".into(),
                    "2" => "Blank".into(),
                    x => unk(x),
                };
                o.push_str(&format!("{t}<DecorativeLine>{}</DecorativeLine>\r\n{t}<TextLocation>{text_loc}</TextLocation>\r\n{t}<BeginArrow>{}</BeginArrow>\r\n{t}<EndArrow>{}</EndArrow>\r\n\t\t\t</Properties>\r\n", bool01(a(it, 5)?), arrow(a(b, i)?), arrow(a(b, i + 1)?)));
            }
            _ => {
                // {{<заголовок>,4,<uuid>,0},<версия>,{{{<оформление>,5,<место>,N,<точки>...,4,<картинка>,<рамка>},<доп.>...}},...}
                let uuid = a(at(it, 0)?, 2)?;
                let wrap = at(at(it, 2)?, 0)?;
                let b = at(wrap, 0)?;
                o.push_str(&format!("\t\t<{tag} id=\"{id}\" uuid=\"{uuid}\">\r\n\t\t\t<Properties>\r\n"));
                common(&h, at(b, 0)?, t, &mut o)?;
                location(b, 2, t, &mut o)?;
                let np: usize = a(b, 6)?.parse().map_err(|_| "карта маршрута: точки фигуры")?;
                let i = 7 + 2 * np;
                line("Border", at(b, i + 2)?, t, &mut o)?;
                picture(at(b, i + 1)?, t, &mut o);
                let ev: &[&str] = match ty {
                    "2" => &["BeforeStart"],
                    "3" => &["OnComplete"],
                    "4" => &["ConditionCheck"],
                    "9" => &["Processing"],
                    "7" | "8" => &[],
                    _ => &["InteractiveActivationProcessing", "BeforeCreateTasks", "OnCreateTask", "OnExecute", "CheckExecutionProcessing", "BeforeExecute", "BeforeExecuteInteractively"],
                };
                match ty {
                    "4" => o.push_str(&format!("{t}<TruePortIndex>{}</TruePortIndex>\r\n{t}<FalsePortIndex>{}</FalsePortIndex>\r\n", a(wrap, 2)?, a(wrap, 3)?)),
                    "5" => {
                        o.push_str(&format!("{t}<TaskDescription>{}</TaskDescription>\r\n{t}<Explanation>{}</Explanation>\r\n{t}<Group>{}</Group>\r\n", esc(s(it, 7)?), esc(s(it, 3)?), bool01(a(it, 4)?)));
                        let attrs = at(it, 6)?;
                        o.push_str(&format!("{t}<AddressingAttributes>\r\n"));
                        // В записи — по uuid, в XML — в порядке реквизитов адресации задачи.
                        let mut list: Vec<((String, usize), &Node)> = attrs.list().iter().skip(1).map(|x| Ok((addressing(a(x, 0)?), at(x, 1)?))).collect::<Result<_, String>>()?;
                        list.sort_by_key(|x| x.0 .1);
                        for ((name, _), val) in list {
                            // префикс readable объявлен на самом Value: d<уровень>p1 по глубине (ERP 2.5.22: d7p1)
                            let p = format!("d{}p1", t.len() + 3);
                            let v = match (raw(val).as_str(), value(val)) {
                                ("{\"U\"}", _) => "<Value xsi:nil=\"true\"/>".to_string(),
                                (_, Some(v)) => format!("<Value xmlns:{p}=\"http://v8.1c.ru/8.3/xcf/readable\" xsi:type=\"{p}:DesignTimeRef\">{}</Value>", esc(&v)),
                                (r, None) => format!("<Value>{UNK}{}</Value>", esc(r)),
                            };
                            o.push_str(&format!("{t}\t<AddressingAttribute ref=\"{}\">\r\n{t}\t\t{v}\r\n{t}\t</AddressingAttribute>\r\n", name));
                        }
                        o.push_str(&format!("{t}</AddressingAttributes>\r\n"));
                    }
                    _ => {}
                }
                o.push_str("\t\t\t</Properties>\r\n");
                // у разделения и слияния событий нет — нет и их части в записи
                if !ev.is_empty() {
                    events(at(it, if ty == "5" { 5 } else { 3 })?, ev, &mut o)?;
                }
            }
        }
        o.push_str(&format!("\t\t</{tag}>\r\n"));
    }
    o.push_str("\t</Items>\r\n</GraphicalSchema>");
    Ok(o)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_and_line() {
        let v = "{7,{3,4,{0}},{3,3,{-22}},{3,3,{-3}},{7,1,0,{0},1,100},{1,0},1,1,1,0,0,0,0,0}";
        let start = format!("{{{{{{4,1,{{1,0}},\"Старт\",1}},4,0e506886-d155-46df-9ace-ce806b7c0b6b,0}},2,{{{{{{{v},5,220,20,260,60,5,220,20,259,20,259,49,240,59,220,49,4,{{4,0,{{0}},\"\",-1,-1,1,0,\"\"}},{{4,0,{{0}},1,1,0,e45c0cd8-a878-4bcb-8e1a-af934481e1cc,0}}}},1}}}},{{1,{{0,\"ПриСтарте\"}}}}}}");
        let ln = format!("{{{{4,2,{{1,0}},\"Линия1\",2}},3,1,0,-1,1,{{{{{v},6,2,240,60,240,100,{{4,0,{{0}},2,1,0,e45c0cd8-a878-4bcb-8e1a-af934481e1cc,0}},0,4,0,1,240,60,240,100,1,0,2}}}}}}");
        let t = format!("{{5,{{{{1,{{3,3,{{-10}}}},1,20,20,3,2,6,{{\"N\",10}},16,{{\"N\",0}}}}}},2,2,{start},1,{ln},3}}");
        let x = flowchart(&brace::parse(&t).unwrap(), &|id| (id.to_string(), 0), &|_| None).unwrap();
        assert!(x.contains("\t\t<TopMargin>10</TopMargin>\r\n\t\t<FitPageMode>Auto</FitPageMode>\r\n"), "{x}");
        assert!(x.contains("<Start id=\"1\" uuid=\"0e506886-d155-46df-9ace-ce806b7c0b6b\">\r\n\t\t\t<Properties>\r\n\t\t\t\t<Name>Старт</Name>\r\n\t\t\t\t<Title/>\r\n\t\t\t\t<ToolTip/>\r\n\t\t\t\t<TabOrder>1</TabOrder>\r\n\t\t\t\t<BackColor>auto</BackColor>"), "{x}");
        assert!(x.contains("<Location top=\"20\" left=\"220\" bottom=\"60\" right=\"260\"/>"), "{x}");
        assert!(x.contains("<Event name=\"BeforeStart\">ПриСтарте</Event>"), "{x}");
        assert!(x.contains("<Segment index=\"1\">\r\n\t\t\t\t\t\t<Start x=\"240\" y=\"60\"/>"), "{x}");
        assert!(x.contains("<From>\r\n\t\t\t\t\t\t<Item>Старт</Item>\r\n\t\t\t\t\t\t<PortIndex>4</PortIndex>"), "{x}");
        assert!(x.contains("<To>\r\n\t\t\t\t\t\t<Item/>\r\n"), "{x}");
        assert!(x.contains(">Dashed<") && x.contains("<DecorativeLine>true</DecorativeLine>") && x.contains("<EndArrow>Blank</EndArrow>"), "{x}");
    }
}
