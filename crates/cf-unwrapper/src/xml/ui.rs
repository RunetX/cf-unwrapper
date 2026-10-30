//! Цвета, шрифты, рамки: `{"#",<тип>,...}` → значения v8ui (элементы стиля, позже — формы).

use super::common::raw;
use super::w::W;
use crate::brace::Node;

const COLOR: &str = "9cd510c7-abfc-11d4-9434-004095e12fc7";
const FONT: &str = "9cd510c8-abfc-11d4-9434-004095e12fc7";
const BORDER: &str = "4d10ca00-111a-4d43-9c96-92cd773716de";

fn find<'a>(t: &[(&str, &'a str)], k: &str) -> Option<&'a str> {
    t.iter().find(|(a, _)| *a == k).map(|(_, v)| *v)
}

/// Цвет `{3,<вид>,{<значение>}}`: 0 — RGB (число BGR), 1 — Windows, 2 — web, 3 — стиль.
pub fn color(c: &Node) -> String {
    let v = c.at(&[2, 0]).and_then(Node::atom).unwrap_or_default();
    let named = |prefix: &str, t: &[(i64, &'static str)]| {
        v.parse().ok().and_then(|k| super::mxl::color_name(t, k)).map_or_else(|| format!("{prefix}:\u{1}{v}"), |n| format!("{prefix}:{n}"))
    };
    match c.get(1).and_then(Node::atom) {
        Some("0") => match v.parse::<u32>() {
            Ok(bgr) => format!("#{:02X}{:02X}{:02X}", bgr & 0xff, bgr >> 8 & 0xff, bgr >> 16 & 0xff),
            Err(_) => raw(Some(c)),
        },
        Some("1") => named("win", super::mxl::WINDOWS),
        Some("2") => named("web", super::mxl::WEB),
        Some("3") => named("style", super::mxl::STYLE),
        _ => raw(Some(c)),
    }
}

/// Атрибуты шрифта `{7,<вид>,<маска>,...}`: вид 0 — абсолютный (все поля LOGFONT), 1 — Windows, 2 — стиль.
/// У ссылочных шрифтов маска говорит, какие поля записаны: 2 — размер, 4/8/16/32 — жирный/курсив/подчеркнутый/зачеркнутый, 512 — масштаб.
pub fn font_attrs(f: &Node) -> Vec<(&'static str, String)> {
    let l = f.list();
    let a = |i: usize| l.get(i).and_then(Node::atom).unwrap_or_default();
    let flag = |s: &str| if s == "0" { "false" } else { "true" }.to_string();
    let height = |s: &str| s.parse::<u32>().map_or_else(|_| format!("\u{1}{s}"), |h| (h / 10).to_string());
    let bold = |s: &str| if s == "700" { "true".to_string() } else if s == "400" { "false".to_string() } else { format!("\u{1}{s}") };
    let mut out = Vec::new();
    if a(1) == "0" {
        // {7,0,маска,высота,ширина,наклон,поворот,вес,курсив,подчеркнутый,зачеркнутый,…,"гарнитура",1,масштаб}
        out.push(("faceName", l.get(16).and_then(Node::str).unwrap_or_default().to_string()));
        out.push(("height", height(a(3))));
        out.push(("bold", bold(a(7))));
        out.push(("italic", flag(a(8))));
        out.push(("underline", flag(a(9))));
        out.push(("strikeout", flag(a(10))));
        out.push(("kind", "Absolute".into()));
        out.push(("scale", a(18).into()));
        return out;
    }
    let r = l.get(3).and_then(|n| n.get(0)).and_then(Node::atom).unwrap_or_default();
    let (kind, reference) = match a(1) {
        "1" if r == "0" => ("WindowsFont", "sys:DefaultGUIFont".to_string()),
        "2" => ("StyleItem", r.parse().ok().and_then(|k| super::mxl::color_name(super::mxl::STYLE, k)).map_or_else(|| format!("style:\u{1}{r}"), |n| format!("style:{n}"))),
        _ => ("\u{1}", raw(Some(f))),
    };
    out.push(("ref", reference));
    let mask: u32 = a(2).parse().unwrap_or(0);
    let mut i = 4;
    let mut next = || {
        i += 1;
        a(i - 1).to_string()
    };
    if mask & 2 != 0 {
        out.push(("height", height(&next())));
    }
    for (bit, name) in [(4, "bold"), (8, "italic"), (16, "underline"), (32, "strikeout")] {
        if mask & bit != 0 {
            let v = next();
            out.push((name, if bit == 4 { bold(&v) } else { flag(&v) }));
        }
    }
    out.push(("kind", kind.into()));
    // дальше «1» и масштаб
    next();
    let scale = next();
    if mask & 512 != 0 {
        out.push(("scale", scale));
    }
    out
}

/// Значение `{"#",<тип>,...}` элемента стиля: цвет, шрифт или рамка.
pub fn style_value(w: &mut W, tag: &str, v: &Node) {
    match v.get(1).and_then(Node::atom) {
        Some(COLOR) => match v.get(3) {
            Some(c) => w.text_attrs(tag, &[("xsi:type", "v8ui:Color")], &color(c)),
            None => w.text(tag, &raw(Some(v))),
        },
        Some(FONT) => {
            let attrs = v.get(3).map(font_attrs).unwrap_or_default();
            let mut a: Vec<(&str, &str)> = vec![("xsi:type", "v8ui:Font")];
            a.extend(attrs.iter().map(|(k, v)| (*k, v.as_str())));
            w.text_attrs(tag, &a, "");
        }
        // {"#",4d10ca00-…,1,{3,0,{0},<вид>,<толщина>,0,<guid>},0}, как рамка диаграммы (chart.rs); ERP 2.5.22
        // РамкаНетРамки: 0,1 → WithoutBorder, width 1
        Some(BORDER) => {
            let b = v.get(3);
            let width = b.and_then(|b| b.get(4)).and_then(Node::atom).unwrap_or_default();
            let style = match b.and_then(|b| b.get(3)).and_then(Node::atom) {
                Some("0") => "WithoutBorder".to_string(),
                Some("1") => "Single".to_string(),
                s => format!("\u{1}{}", s.unwrap_or_default()),
            };
            w.open(tag, &[("xsi:type", "v8ui:Border"), ("width", width)]);
            w.text_attrs("v8ui:style", &[("xsi:type", "v8ui:ControlBorderType")], &style);
            w.end();
        }
        _ => w.text(tag, &raw(Some(v))),
    }
}

const STYLE_HEAD: &str = "<Style xmlns=\"http://v8.1c.ru/8.3/xcf/extrnprops\" xmlns:pal=\"http://v8.1c.ru/8.1/data/ui/colors/palette\" xmlns:style=\"http://v8.1c.ru/8.1/data/ui/style\" xmlns:sys=\"http://v8.1c.ru/8.1/data/ui/fonts/system\" xmlns:v8ui=\"http://v8.1c.ru/8.1/data/ui\" xmlns:web=\"http://v8.1c.ru/8.1/data/ui/colors/web\" xmlns:win=\"http://v8.1c.ru/8.1/data/ui/colors/windows\" xmlns:xr=\"http://v8.1c.ru/8.3/xcf/readable\" xmlns:xs=\"http://www.w3.org/2001/XMLSchema\" xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\" version=\"2.21\">";

/// Ext/Style.xml стиля из записи `{1,N,{ключ,вид,значение}…}`: ключ `{-номер}` — элемент стиля платформы, `{0,uuid}` —
/// элемент стиля конфигурации; вид 0 — цвет `{3,…}`, 1 — шрифт `{7,…}`, 2 — рамка `{3,1,{-номер},0,0,0}`.
/// Порядок — не записи: сначала элементы платформы в ее порядке (`mxl::STYLE`), затем элементы конфигурации в порядке
/// конфигурации (backend.dll 0x1806c3070 обходит полный список описаний и пишет присутствующие; корень — 0x1806c2730).
pub fn style_xml(cfg: &super::Config, text: &str) -> Result<Vec<u8>, String> {
    let doc = crate::brace::Doc::parse(text).map_err(|e| e.to_string())?;
    let root = doc.roots.first().ok_or("пустая запись")?;
    let l = root.list();
    if l.first().and_then(Node::atom) != Some("1") || l.get(1).and_then(Node::atom).and_then(|n| n.parse::<usize>().ok()) != Some(l.len() - 2) {
        return Err(format!("заголовок записи {}", raw(Some(root)).chars().take(40).collect::<String>()));
    }
    let mut platform = std::collections::HashMap::new();
    let mut own = std::collections::HashMap::new();
    for it in &l[2..] {
        let key = it.get(0).ok_or("элемент без ключа")?;
        match (key.get(0).and_then(Node::atom), key.get(1).and_then(Node::atom)) {
            (Some(n), None) => platform.insert(n.parse::<i64>().map_err(|_| format!("ключ {n}"))?, it),
            (Some("0"), Some(id)) => own.insert(id.to_string(), it),
            _ => return Err(format!("ключ {}", raw(Some(key)))),
        };
    }
    let mut w = W::root("Style", STYLE_HEAD);
    let mut put = |name: String, it: &Node| -> Result<(), String> {
        let v = it.get(2).ok_or("элемент без значения")?;
        w.open("Item", &[("name", &name)]);
        match it.get(1).and_then(Node::atom) {
            Some("0") => w.text("Color", &color(v)),
            Some("1") => {
                let attrs = font_attrs(v);
                w.text_attrs("Font", &attrs.iter().map(|(k, v)| (*k, v.as_str())).collect::<Vec<_>>(), "");
            }
            // ссылка на рамку стиля; прочие рамки не встречались — отказ
            Some("2") => match v.list() {
                [k, one, r, z1, z2, z3] if k.atom() == Some("3") && one.atom() == Some("1") && [z1, z2, z3].iter().all(|z| z.atom() == Some("0")) => {
                    let n = r.get(0).and_then(Node::atom).and_then(|n| n.parse().ok()).and_then(|n| super::mxl::color_name(super::mxl::STYLE, n)).ok_or_else(|| format!("рамка {}", raw(Some(v))))?;
                    w.text_attrs("Border", &[("ref", &format!("style:{n}"))], "");
                }
                _ => return Err(format!("рамка {}", raw(Some(v)))),
            },
            k => return Err(format!("вид значения {k:?}")),
        }
        w.end();
        Ok(())
    };
    for (n, name) in super::mxl::STYLE {
        if let Some(it) = platform.remove(n) {
            put(name.to_string(), it)?;
        }
    }
    if let Some((n, _)) = platform.iter().next() {
        return Err(format!("элемент стиля платформы {n} не в таблице"));
    }
    for g in cfg.groups.iter().filter(|g| g.class == "StyleItem") {
        for si in &g.items {
            if let Some(it) = own.remove(&si.header.id) {
                put(format!("StyleItem.{}", si.header.name), it)?;
            }
        }
    }
    if let Some(id) = own.keys().next() {
        return Err(format!("элемент стиля {id} не в конфигурации"));
    }
    Ok(w.finish())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shortcuts() {
        let k = |t: &str| shortcut(Some(&crate::brace::parse(t).unwrap()));
        assert_eq!((k("{0,122,0}"), k("{0,70,24}"), k("{0,83,12}"), k("{0,0,0}")), ("F11".into(), "Ctrl+Alt+F".into(), "Ctrl+Shift+S".into(), String::new()));
        assert!(k("{0,83,20}").starts_with('\u{1}') && k("{0,49,8}").starts_with('\u{1}'));
    }

    fn node(t: &str) -> Node {
        crate::brace::Doc::parse(t).unwrap().roots.remove(0)
    }

    #[test]
    fn colors() {
        assert_eq!(color(&node("{3,0,{4625920}}")), "#009646");
        assert_eq!(color(&node("{3,2,{26}}")), "web:DarkGray");
        assert_eq!(color(&node("{3,1,{5}}")), "win:WindowBackground");
        assert_eq!(color(&node("{3,3,{-3}}")), "style:FormTextColor");
        assert_eq!(color(&node("{3,2,{999}}")), "web:\u{1}999");
    }

    #[test]
    fn fonts() {
        let s = |t: &str| font_attrs(&node(t)).iter().map(|(k, v)| format!("{k}={v}")).collect::<Vec<_>>().join(" ");
        assert_eq!(s("{7,2,62,{-31},100,700,0,0,0,1,100}"), "ref=style:NormalTextFont height=10 bold=true italic=false underline=false strikeout=false kind=StyleItem");
        assert_eq!(s("{7,2,572,{-31},400,1,0,0,1,90}"), "ref=style:NormalTextFont bold=false italic=true underline=false strikeout=false kind=StyleItem scale=90");
        assert_eq!(s("{7,1,0,{0},1,100}"), "ref=sys:DefaultGUIFont kind=WindowsFont");
        assert_eq!(s("{7,2,2,{-31},70,1,100}"), "ref=style:NormalTextFont height=7 kind=StyleItem");
        assert_eq!(
            s("{7,0,575,140,0,0,0,400,0,0,0,0,0,0,0,0,\"Arial\",1,100}"),
            "faceName=Arial height=14 bold=false italic=false underline=false strikeout=false kind=Absolute scale=100"
        );
    }
}

/// Стандартные картинки: в скобках uuid или отрицательный номер. shortcut: только встреченные в БП 3.0.171.
pub const STD_PICTURES: &[(&str, &str)] = &[
    ("-13", "Print"), ("-9", "MoveRight"), ("-7", "InputFieldOpen"),
    ("5b87ad1b-d8cc-43c1-b5c4-dc43613c518c", "InformationRegister"), ("283ecabd-aaed-41d1-ad46-6cca91c29120", "LoadReportSettings"),
    ("37cf7cc0-abad-4385-b597-6fd2d8dc085a", "Task"), ("e8a49985-fef7-45a9-b6bb-ddd2b9028172", "DataHistory"),
    ("a24cff7f-a1a5-4403-af82-a7b31852cde9", "BusinessProcessObject"), ("eb47324b-85f9-4172-9315-bba8015d9970", "NewWindow"),
    ("6ff3ddbd-56e3-4ddf-a5bf-048c1e2dfb2f", "User"), ("723765ab-0b92-4745-a621-1ba0f77c92c9", "EventLog"),
    ("4d2570b5-205f-413c-b4cc-b2097f61684f", "CreateInitialImage"), ("08a45a70-c221-4339-b3b1-9f11cb22147d", "Delete"),
    ("83c8f18d-8701-41f3-bef4-53f88adbb868", "ReadChanges"), ("58174855-39be-462e-8723-cb2d95182146", "SetDateInterval"),
    ("aa96f4bb-cf28-4dad-bc42-5ed53de95c0c", "SpreadsheetDeletePageBreak"), ("4bf9fbb5-53c5-4b09-bab2-d69bbfab945b", "Dendrogram"),
    // команды справочников и документов (GenerateReport, ShowData — из УТ 11.5.27)
    ("97b2cc97-d5c6-45fb-9824-9d6d73db21fe", "Change"), ("e51185a4-d915-45b8-b201-1c46cc2d8104", "DocumentJournal"),
    ("003024ed-fa25-42ac-9f53-f5014e383801", "ExecuteTask"), ("db817ee1-fd28-4e7f-bb4a-53686b2b153c", "Report"),
    ("a7707ed1-39b0-418f-974d-4d500d27a9c6", "RestoreValues"), ("1cd7b762-ec6a-4e92-ac9a-1832be228ec3", "Stop"),
    ("9fecbaff-2a05-4da6-9ef1-807e754b928d", "BusinessProcessStart"), ("0ce78048-0196-4f80-a781-9829cdb7f43e", "GenerateReport"),
    ("a064544f-6037-48ca-b19f-8ad63e43af23", "ShowData"),
];

/// Картинка `{4,<есть>,{0,<uuid>}|{-N}|{0},"",<x>,<y>,<прозрачность>,0,""}`: x,y — прозрачный пиксель (-1 — нет).
pub fn picture(w: &mut W, tag: &str, x: &super::common::Index, p: Option<&Node>) {
    let Some(p) = p.filter(|p| p.get(1).and_then(Node::atom) == Some("1")) else {
        w.text(tag, "");
        return;
    };
    let a = |i: usize| p.get(i).and_then(Node::atom).unwrap_or_default();
    let r = p.get(2).map(Node::list).unwrap_or_default();
    let id = match r {
        [Node::Atom(z), Node::Atom(id)] if z == "0" => id.as_str(),
        [Node::Atom(n)] => n.as_str(),
        _ => "\u{1}",
    };
    let name = match find(STD_PICTURES, id).or_else(|| crate::xml::imp::form::platform::std_picture(id)) {
        Some(n) => format!("StdPicture.{n}"),
        // {0} без uuid платформа выгружает как «0»
        None if id == "0" => "0".into(),
        None if id.starts_with('-') => format!("StdPicture.\u{1}{id}"),
        // не общая картинка и не известная стандартная: uuid вместо имени в эталоне не встречен
        None => Some(x.name(id)).filter(|n| n != id).unwrap_or_else(|| format!("\u{1}{id}")),
    };
    w.open(tag, &[]);
    w.text("xr:Ref", &name);
    w.text("xr:LoadTransparent", if a(6) == "1" { "true" } else { "false" });
    if a(4) != "-1" {
        w.text_attrs("xr:TransparentPixel", &[("x", a(4)), ("y", a(5))], "");
    }
    w.end();
}

/// Сочетание клавиш `{0,<код клавиши>,<модификаторы>}`: F1–F12 и буквы, Ctrl, Ctrl+Shift, Ctrl+Alt.
pub fn shortcut(s: Option<&Node>) -> String {
    let a = |i: usize| s.and_then(|s| s.get(i)).and_then(Node::atom).unwrap_or_default();
    // `{0,<клавиша>,<модификаторы: 4 Shift, 8 Ctrl, 16 Alt>}`; только встреченные в БП сочетания, прочее — меткой.
    let key = match a(1).parse::<u32>() {
        Ok(k @ 112..=123) => Some(format!("F{}", k - 111)),
        Ok(k @ 65..=90) => char::from_u32(k).map(String::from),
        _ => None,
    };
    let mods = match a(2) {
        "" | "0" => Some(""),
        "8" => Some("Ctrl+"),
        "12" => Some("Ctrl+Shift+"),
        "24" => Some("Ctrl+Alt+"),
        _ => None,
    };
    match (a(0), a(1), key, mods) {
        ("0", "0", ..) if matches!(a(2), "" | "0") => String::new(),
        ("0", _, Some(k), Some(m)) => format!("{m}{k}"),
        _ => s.map(|s| raw(Some(s))).unwrap_or_default(),
    }
}
