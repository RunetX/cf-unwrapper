//! Диаграмма рисунка табличного документа (`<object xsi:type="dNp1:Chart">`) → узел диаграммы версии 75 — так ее
//! пишет загрузка 8.5 при любой исходной версии (обратное `xml/chart.rs` `body7x`, раскладка — docs/research/xml-mxl.md
//! «Диаграмма»). Поля, которых XML не выражает, — значения загрузки платформы (сверено на 26 диаграммах БП); свойство
//! XML, которое выгрузка пишет постоянным, должно им и быть — иначе отказ.

use super::dom::El;
use super::index::Index;
use super::mxl::{color_node, font_node};
use super::tpl::{a, crlf, l, s};
use crate::brace::Node;

type R<T> = Result<T, String>;

const CHART_NS: &str = "http://v8.1c.ru/8.2/data/chart";
const LINE: &str = "e5cabe59-d992-4d31-8086-3116931aff81";
const BORDER: &str = "48312c09-257f-4b29-b280-284dd89efc1e";
const ZERO: &str = "00000000-0000-0000-0000-000000000000";

/// Тип диаграммы — как у выгрузки (chart.dll FUN_1800a8870).
use crate::xml::chart::{CHART_TYPE, GANTT_VALUE_TEXT, LABEL_TYPE, PALETTE};
const MARKER: &[(i64, &str)] = &[(0, "None"), (1, "Rect"), (2, "Circle"), (3, "Rhomb"), (4, "Auto")];

fn err(e: &El, m: &str) -> String {
    format!("строка {}: <{}>: диаграмма: {m}", e.line, e.name)
}

fn auto() -> Node {
    l(vec![a("4"), a("4"), l(vec![a("0")]), a("4")])
}

fn font_auto() -> Node {
    l(vec![a("8"), a("3"), a("0"), a("1"), a("100")])
}

/// Число XML → число записи. shortcut: значения переносятся как в XML (так у всех встреченных), числа с порядком —
/// по месту, где платформа пишет их так (см. [`sci`]).
fn dec(e: &El, t: &str) -> R<String> {
    let ok = !t.is_empty() && t.trim_start_matches('-').bytes().all(|b| b.is_ascii_digit() || b == b'.') && t.matches('.').count() <= 1;
    if ok { Ok(t.to_string()) } else { Err(err(e, &format!("число {t:?}"))) }
}

/// Число 1С в записи (`5e2`, `1.6e2`, `1e-1`, `5.48`): целое с нулями на конце — мантисса с порядком; дробное,
/// у которого после нулей одна значащая цифра (0.1, 0.03), — тоже с порядком; остальное — как в XML.
fn sci(e: &El, t: &str) -> R<String> {
    let t = dec(e, t)?;
    let (neg, u) = t.strip_prefix('-').map_or((false, t.as_str()), |r| (true, r));
    let (ip, fp) = u.split_once('.').unwrap_or((u, ""));
    let fp = fp.trim_end_matches('0');
    let ip = ip.trim_start_matches('0');
    let sign = if neg { "-" } else { "" };
    if ip.is_empty() && fp.is_empty() {
        return Ok("0".into());
    }
    if fp.is_empty() {
        let digits = ip.trim_end_matches('0');
        let zeros = ip.len() - digits.len();
        if zeros == 0 {
            return Ok(format!("{sign}{ip}"));
        }
        let m = if digits.len() > 1 { format!("{}.{}", &digits[..1], &digits[1..]) } else { digits.to_string() };
        return Ok(format!("{sign}{m}e{}", ip.len() - 1));
    }
    if ip.is_empty() {
        let lead = fp.len() - fp.trim_start_matches('0').len();
        let rest = &fp[lead..];
        if rest.len() == 1 {
            return Ok(format!("{sign}{rest}e-{}", lead + 1));
        }
    }
    Ok(t)
}

/// Дочерние элементы `<object>` по порядку схемы; каждый — `<p:имя>`.
struct Cur<'a> {
    kids: Vec<&'a El>,
    i: usize,
    p: String,
    x: &'a Index,
    at: &'a El,
}

impl<'a> Cur<'a> {
    fn peek(&self, tag: &str) -> bool {
        self.kids.get(self.i).is_some_and(|e| e.name == format!("{}:{tag}", self.p))
    }
    fn take(&mut self, tag: &str) -> R<&'a El> {
        match self.kids.get(self.i) {
            Some(e) if e.name == format!("{}:{tag}", self.p) => {
                self.i += 1;
                Ok(e)
            }
            Some(e) => Err(err(e, &format!("ждали {tag}"))),
            None => Err(err(self.at, &format!("нет {tag}"))),
        }
    }
    fn opt(&mut self, tag: &str) -> Option<&'a El> {
        if self.peek(tag) {
            self.take(tag).ok()
        } else {
            None
        }
    }
    fn text(&mut self, tag: &str) -> R<String> {
        self.take(tag)?.text()
    }
    fn b(&mut self, tag: &str) -> R<Node> {
        let e = self.take(tag)?;
        bool_of(e)
    }
    fn i(&mut self, tag: &str) -> R<Node> {
        let e = self.take(tag)?;
        int_of(e)
    }
    fn en(&mut self, tag: &str, vals: &[(i64, &str)]) -> R<Node> {
        let e = self.take(tag)?;
        en_of(e, vals)
    }
    fn ml(&mut self, tag: &str) -> R<Node> {
        let e = self.take(tag)?;
        ml_of(e)
    }
    fn color(&mut self, tag: &str) -> R<Node> {
        let e = self.take(tag)?;
        color_node(self.x, e)
    }
    fn font(&mut self, tag: &str) -> R<Node> {
        let e = self.take(tag)?;
        font_node(self.x, e, false)
    }
    fn line(&mut self, tag: &str) -> R<Node> {
        let e = self.take(tag)?;
        line_of(e)
    }
    fn border(&mut self, tag: &str) -> R<Node> {
        let e = self.take(tag)?;
        border_of(e)
    }
    /// Свойство, которое выгрузка пишет постоянным: в XML должно быть оно же.
    fn fixed(&mut self, tag: &str, v: &str) -> R<()> {
        let e = self.take(tag)?;
        let t = e.text()?;
        if t != v {
            return Err(err(e, &format!("{tag} = {t:?} (загружается только {v:?})")));
        }
        Ok(())
    }
}

fn bool_of(e: &El) -> R<Node> {
    match e.text()?.as_str() {
        "true" => Ok(a("1")),
        "false" => Ok(a("0")),
        t => Err(err(e, &format!("не булево {t:?}"))),
    }
}

fn int_of(e: &El) -> R<Node> {
    let t = e.text()?;
    t.parse::<i64>().map(|v| a(v.to_string())).map_err(|_| err(e, &format!("не целое {t:?}")))
}

fn en_of(e: &El, vals: &[(i64, &str)]) -> R<Node> {
    let t = e.text()?;
    vals.iter().find(|(_, v)| *v == t).map(|(k, _)| a(k.to_string())).ok_or_else(|| err(e, &format!("значение {t:?} не встречено")))
}

/// Многоязычная строка `<v8:item><v8:lang/><v8:content/></v8:item>…` → `{1,N,{"язык","текст"}…}`.
fn ml_of(e: &El) -> R<Node> {
    let items: Vec<&El> = e.elems().collect();
    if items.is_empty() {
        e.text()?;
    }
    let mut v = vec![a("1"), a(items.len().to_string())];
    for it in items {
        if it.name != "v8:item" || it.elems().count() != 2 {
            return Err(err(it, "многоязычная строка"));
        }
        v.push(l(vec![s(crlf(&it.text_of("v8:lang")?)), s(crlf(&it.text_of("v8:content")?))]));
    }
    Ok(l(v))
}

fn style_of(e: &El, ty: &str, vals: &[(i64, &str)]) -> R<Node> {
    let st = e.req("v8ui:style")?;
    if st.req_attr("xsi:type")? != ty || e.elems().count() != 1 {
        return Err(err(e, "вид линии"));
    }
    en_of(st, vals)
}

/// Линия `<p:x width gap="false"><v8ui:style xsi:type="v8ui:ChartLineType">` → `{4,0,{0},вид,толщина,0,e5cabe59-…,0}`.
fn line_of(e: &El) -> R<Node> {
    let w = e.req_attr("width")?;
    if e.req_attr("gap")? != "false" || w.parse::<u32>().is_err() {
        return Err(err(e, "линия"));
    }
    let k = style_of(e, "v8ui:ChartLineType", &[(0, "None"), (1, "Solid"), (2, "Dotted")])?;
    Ok(l(vec![a("4"), a("0"), l(vec![a("0")]), k, a(w), a("0"), a(LINE), a("0")]))
}

/// Рамка `<p:x width><v8ui:style xsi:type="v8ui:ControlBorderType">` → `{3,0,{0},вид,толщина,0,48312c09-…}`.
fn border_of(e: &El) -> R<Node> {
    let w = e.req_attr("width")?;
    if w.parse::<u32>().is_err() {
        return Err(err(e, "рамка"));
    }
    let k = style_of(e, "v8ui:ControlBorderType", &[(0, "WithoutBorder"), (1, "Single")])?;
    Ok(l(vec![a("3"), a("0"), l(vec![a("0")]), k, a(w), a("0"), a(BORDER)]))
}

/// Палитра «Авто» загрузки (chart.dll 8.5.1.1529, таблица 0x18040a980, функция цвета по номеру 0x18017ef10:
/// `таблица[номер % 30]`): цвет серии с цветом auto в блоке записи. В оформлении серии остается auto.
const AUTO_PALETTE: [u32; 30] = [
    16762956, 7964671, 49407, 7258719, 15899038, 6736876, 16737735, 13160817, 5046193, 15512678, 15492775, 15235583, 9465259, 7051715, 13222065, 11837108, 5074335, 4239354, 7581872,
    7633580, 10138817, 15060084, 10128322, 9017497, 7641966, 13214095, 8354939, 11378304, 5793218, 4116952,
];

/// Палитра «Авто» диаграммы реквизита формы (chart.dll 8.5.1.1529, таблица 0x18040aaa0; сверка БП ПроверкаКонтрагента:
/// 4 серии и сводная).
const FORM_PALETTE: [u32; 30] = [
    16757805, 7105526, 2866687, 5090622, 15564667, 1938929, 13595561, 11845712, 6604689, 15713425, 14828674, 13074392, 7425168, 4881326, 11971223, 8874375, 3034242, 6329570,
    5412502, 5462417, 8166315, 14342770, 8154029, 6912635, 5471565, 11961456, 5921370, 9601375, 3687597, 2277577,
];

/// Серия или точка: (блок записи, id, цвет, маркер). `n` — номер серии: цвет auto и маркер Auto в блоке
/// заменяются цветом палитры (номер как 64-битное без знака % 30) и маркером (номер % 3 со знаком: 0 Rhomb, 1 Rect,
/// 2 Circle, иначе Rect); у точки — ее номер, у сводной серии — -1 (цвет 15-й, маркер Rect). Сверено: 2 серии и
/// 3 точки макетов, сводная диаграммы Ганта; у форм палитра другая (таблица 0x18040aaa0: серии и сводная
/// формы ПроверкаКонтрагента.Форма дают тот же круг маркеров).
fn series(c: &mut Cur, tag: &str, point: bool, n: i64, pal: &[u32]) -> R<(Vec<Node>, Node, Node, Node)> {
    let e = c.take(tag)?;
    let mut k = Cur { kids: e.elems().collect(), i: 0, p: c.p.clone(), x: c.x, at: e };
    let id = k.i("id")?;
    let color = k.color("color")?;
    let line = k.line("line")?;
    let marker = k.en("marker", MARKER)?;
    let text = k.ml("text")?;
    let changed = k.b("strIsChanged")?;
    let expand = k.b("isExpand")?;
    let ind = k.b("isIndicator")?;
    let prio = k.b("colorPriority")?;
    if k.i != k.kids.len() {
        return Err(err(k.kids[k.i], "лишнее в серии"));
    }
    let u = || l(vec![s("U")]);
    let (mut bcolor, mut bmarker) = (color.clone(), marker.clone());
    if bcolor == self::auto() {
        let v = pal[(n as u64 % pal.len() as u64) as usize];
        bcolor = l(vec![a("4"), a("0"), l(vec![a(v.to_string())]), a("0")]);
    }
    if bmarker.atom() == Some("4") {
        bmarker = a(match n % 3 {
            0 => "3",
            2 => "2",
            _ => "1",
        });
    }
    let block = if point {
        vec![text, changed, id.clone(), bcolor, line, bmarker, expand, ind, u(), u(), prio]
    } else {
        vec![bcolor, line, bmarker, text, changed, expand, ind, id.clone(), u(), u(), prio]
    };
    Ok((block, id, color, marker))
}

/// Шкала (точек, значений, серий) `{2,заголовок скрыт,0,2,{1,0},{область},линии сетки,0,есть линия[,линия],авто,
/// шрифт,цвет подписей,2,формат,0,авто,0,0,0,0,0,в диаграмме}`; нет элемента — по умолчанию.
fn scale(c: &mut Cur, tag: &str) -> R<Node> {
    let area = l(vec![a("1"), a("4"), a("0.5"), a("0.5"), font_auto(), auto(), auto(), a("1"), l(vec![a("3"), a("0"), l(vec![a("0")]), a("0"), a("1"), a("0"), a(BORDER)]), auto(), a("4"), a("2"), a("0")]);
    let (mut grid, mut line, mut label, mut fmt, mut in_chart) = (false, None, auto(), l(vec![a("1"), a("0")]), false);
    // поле 12 — ориентация подписей (ChartLabelsOrientation, chart.dll FUN_1800ab5e0): 2 Auto — умолчание
    let mut orient = 2;
    // поле 7 — положение подписей (ChartScaleLabelLocation, chart.dll 0x1800c3570): 0 Auto — умолчание
    let mut loc = 0;
    // поля 1–4: показ заголовка (0 Auto, 1 DontShow, 2 Show), источник текста (0 Auto, 1 UseText, 2 AutoText), место
    // (0 WithAxis, 1 PlotArea, 2 SpecialArea — умолчание), текст; 10 — шрифт подписей, 17 — угол, 19 — строк подписи
    let (mut show, mut source, mut placement, mut title, mut font, mut angle, mut rows) = (0, 0, 2, l(vec![a("1"), a("0")]), font_auto(), 0i64, 0i64);
    if let Some(e) = c.opt(tag) {
        let mut k = Cur { kids: e.elems().collect(), i: 0, p: c.p.clone(), x: c.x, at: e };
        let pick = |k: &mut Cur, tag: &str, names: &[&str]| -> R<Option<usize>> {
            match k.opt(tag) {
                Some(o) => {
                    let t = o.text()?;
                    names.iter().position(|x| *x == t).map(Some).ok_or_else(|| err(o, &format!("{tag} {t}")))
                }
                None => Ok(None),
            }
        };
        if let Some(i) = pick(&mut k, "showTitle", &["Auto", "DontShow", "Show"])? {
            show = i;
        }
        if let Some(i) = pick(&mut k, "titleTextSource", &["Auto", "UseText", "AutoText"])? {
            source = i;
        }
        if k.peek("titleText") {
            title = k.ml("titleText")?;
        }
        let ta = k.take("titleArea")?;
        let mut t = Cur { kids: ta.elems().collect(), i: 0, p: c.p.clone(), x: c.x, at: ta };
        let f = t.font("font")?;
        let (tc, bc) = (t.color("textColor")?, t.color("backColor")?);
        let b = t.border("border")?;
        let bcol = t.color("borderColor")?;
        let want_b = l(vec![a("3"), a("0"), l(vec![a("0")]), a("0"), a("1"), a("0"), a(BORDER)]);
        if f != font_auto() || tc != auto() || bc != auto() || b != want_b || bcol != auto() || t.i != t.kids.len() {
            return Err(err(ta, "область заголовка шкалы не по умолчанию"));
        }
        if let Some(i) = pick(&mut k, "titlePlacement", &["WithAxis", "PlotArea", "SpecialArea"])? {
            placement = i;
        }
        if k.peek("gridLinesShowMode") {
            k.fixed("gridLinesShowMode", "Show")?;
            grid = true;
        }
        if let Some(o) = k.opt("scaleLabelLocation") {
            let t = o.text()?;
            loc = ["Auto", "None", "Inside", "Outside"].iter().position(|x| *x == t).ok_or_else(|| err(o, &format!("scaleLabelLocation {t}")))?;
        }
        if k.peek("gridLine") {
            line = Some(k.line("gridLine")?);
        }
        if k.peek("labelColor") {
            label = k.color("labelColor")?;
        }
        if k.peek("labelFont") {
            font = k.font("labelFont")?;
        }
        if let Some(o) = k.opt("labelOrientation") {
            let t = o.text()?;
            orient = ["Horizontal", "Vertical", "Auto", "CustomAngle"].iter().position(|x| *x == t).ok_or_else(|| err(o, &format!("labelOrientation {t}")))?;
        }
        let int = |k: &mut Cur, tag: &str| -> R<i64> {
            let o = k.take(tag)?;
            let t = o.text()?;
            t.parse().map_err(|_| err(o, &format!("{tag} {t}")))
        };
        if k.peek("maxLabelRows") {
            rows = int(&mut k, "maxLabelRows")?;
        }
        if k.peek("labelAngle") {
            angle = int(&mut k, "labelAngle")?;
        }
        if k.peek("labelFormat") {
            fmt = k.ml("labelFormat")?;
        }
        if k.peek("showInChart") {
            k.fixed("showInChart", "DontShow")?;
            in_chart = true;
        }
        if k.i != k.kids.len() {
            return Err(err(k.kids[k.i], "лишнее в шкале"));
        }
    }
    let mut v = vec![a("2"), a(show.to_string()), a(source.to_string()), a(placement.to_string()), title, area, a(if grid { "0" } else { "2" }), a(loc.to_string())];
    match line {
        Some(n) => v.extend([a("1"), n]),
        None => v.push(a("0")),
    }
    v.extend([auto(), font, label, a(orient.to_string()), fmt, a("0"), auto(), a("0"), a(angle.to_string()), a("0"), a(rows.to_string()), a("0"), a(if in_chart { "2" } else { "0" })]);
    Ok(l(v))
}

/// Полосы шкалы индикатора → `{1,N,{3,начало,конец,цвет,{1,0},{1,0},"",0,"",0,начало,конец}…,0,0}`.
fn bands(c: &mut Cur) -> R<Node> {
    let e = c.take("gaugeQualityBands")?;
    if e.req_attr("useTextStr")? != "false" || e.req_attr("useTooltipStr")? != "false" {
        return Err(err(e, "подписи полос"));
    }
    let items: Vec<&El> = e.elems().collect();
    if items.is_empty() {
        e.text()?;
    }
    let mut v = vec![a("1"), a(items.len().to_string())];
    for it in items {
        let mut k = Cur { kids: it.elems().collect(), i: 0, p: "v8ui".into(), x: c.x, at: it };
        let b = k.take("begin")?;
        let b = dec(b, &b.text()?)?;
        let en = k.take("end")?;
        let en = dec(en, &en.text()?)?;
        let col = k.color("backColor")?;
        k.fixed("text", "")?;
        k.fixed("tooltip", "")?;
        if it.name != "v8ui:item" || k.i != k.kids.len() {
            return Err(err(it, "полоса шкалы"));
        }
        let ml0 = || l(vec![a("1"), a("0")]);
        v.push(l(vec![a("3"), a(b.clone()), a(en.clone()), col, ml0(), ml0(), s(""), a("0"), s(""), a("0"), a(b), a(en)]));
    }
    v.extend([a("0"), a("0")]);
    Ok(l(v))
}

/// Доля области → рамка, округленная до точки холста `size`: (double, десятичное). Десятичное — 27 знаков после
/// запятой с округлением; double — из него, как переводит платформа: мантисса старшими 64 битами и младшими 32,
/// деленная на 1e27 (так у всех 7 встреченных долей, у прямого `p/q` 19/360 — на единицу младшего разряда больше).
fn frame(e: &El, x: &str, size: u32) -> R<(String, String)> {
    let px = x.parse::<f64>().map_err(|_| err(e, &format!("доля {x}")))? * size as f64;
    if px.fract() == 0.5 || !(0.0..=size as f64).contains(&px) {
        return Err(err(e, &format!("доля {x}: округление до точки не установлено")));
    }
    Ok(frame_px(px.round() as u32, size))
}

/// Доля `p / size` точек: (двоичное с плавающей точкой, десятичное 27 знаков) — как в записи диаграммы.
fn frame_px(p: u32, size: u32) -> (String, String) {
    let p = p as u128;
    let m = (p * 10u128.pow(27) * 2 / size as u128 + 1) / 2;
    if m == 0 || m == 10u128.pow(27) {
        return ((m / 10u128.pow(27)).to_string(), (m / 10u128.pow(27)).to_string());
    }
    let dec = format!("0.{m:027}").trim_end_matches('0').to_string();
    let dbl = numeric_f64(m);
    let s = format!("{dbl:.15e}");
    let (mant, ex) = s.split_once('e').unwrap();
    let mant = mant.trim_end_matches('0').trim_end_matches('.');
    (format!("{mant}e{ex}"), dec)
}

/// Доля `m / 10^27` в double, как `core::Numeric::operator double` (core85.dll 0x180103200): разряды по 10^9
/// (нулевые с краев отброшены), три старших — Горнером с округлением на каждом шаге, затем деление на `1e9^(e − lo)`.
fn numeric_f64(m: u128) -> f64 {
    let mut limbs: Vec<u32> = (0..3).map(|i| (m / 1_000_000_000u128.pow(i) % 1_000_000_000) as u32).collect();
    let mut e = 3i32;
    while limbs.len() > 1 && limbs[0] == 0 {
        limbs.remove(0);
        e -= 1;
    }
    while limbs.len() > 1 && limbs[limbs.len() - 1] == 0 {
        limbs.pop();
    }
    let n = limbs.len();
    let mut v = limbs[n - 1] as f64;
    for k in (n.saturating_sub(3)..n - 1).rev() {
        v = limbs[k] as f64 + v * 1e9;
    }
    let lo = n.saturating_sub(3) as i32;
    v / 1e9f64.powi(e - lo)
}

/// Область `<p:x><left/><right/><top/><bottom/>` → (left, top, right, bottom).
fn area(c: &mut Cur, tag: &str) -> R<[String; 4]> {
    let e = c.take(tag)?;
    let mut k = Cur { kids: e.elems().collect(), i: 0, p: c.p.clone(), x: c.x, at: e };
    let mut g = |t: &str| -> R<String> {
        let x = k.take(t)?;
        dec(x, &x.text()?)
    };
    let (lft, r, t, b) = (g("left")?, g("right")?, g("top")?, g("bottom")?);
    if k.i != k.kids.len() {
        return Err(err(e, "лишнее в области"));
    }
    Ok([lft, t, r, b])
}

/// `<object xsi:type="dNp1:Chart">` → узел `{75,…}`.
pub fn chart(x: &Index, obj: &El) -> R<Node> {
    chart_with(x, obj, false)
}

/// Диаграмма — настройки реквизита формы: своя палитра «Авто».
pub fn form_chart(x: &Index, obj: &El) -> R<Node> {
    chart_with(x, obj, true)
}

/// Таблицы видов палитры (chart.dll FUN_18017ef10: цвет — `таблица[номер % длина]`, номер со знаком как u64).
const PALETTE8: [u32; 8] = [7504122, 14772545, 9234160, 5737262, 15658671, 14053591, 4163021, 12632256];
const PALETTE32: [u32; 32] = [
    3484368, 123390, 1690649, 15053337, 13375769, 16718284, 1644953, 1690879, 1677593, 15040281, 10033433, 15014297, 1644927, 1684223, 1670937, 15027225, 6691097, 11671935, 1644902, 4271093,
    2818047, 1697049, 15073049, 16718105, 16718309, 1684223, 1664537, 10033433, 4987161, 11671935, 1644876, 1657881,
];
const PASTEL: [u32; 10] = [10874353, 16759258, 16774791, 14596607, 12451715, 11448063, 10274047, 9894911, 8323018, 12450203];
const BRIGHT: [u32; 10] = [16763648, 54783, 6881511, 29695, 65456, 255, 16711907, 3134081, 16748377, 227];
const WARM: [u32; 18] = [2319090, 1811706, 6978545, 389887, 5694911, 250623, 5935095, 4239612, 9741050, 6813183, 9433288, 6216959, 1527231, 1213124, 1320635, 114381, 4372353, 435912];
const COLD: [u32; 15] = [10783576, 9680202, 12091776, 14658151, 14205307, 8020224, 7046454, 9854062, 10181396, 12493824, 14072203, 11980191, 13674418, 14924384, 15455911];
const RICH: [u32; 30] = [
    16433458, 6187007, 1745151, 4250636, 16555422, 8440575, 16735432, 3388679, 7596287, 16758387, 16723641, 14908927, 10751183, 1147087, 13092807, 8553090, 549281, 5092863, 6874430, 13374975,
    16546172, 16761539, 10066329, 13409055, 2592516, 11579568, 10571008, 10384912, 3358463, 12106751,
];
const RICH_FORM: [u32; 30] = [
    13409055, 3358463, 1147087, 3388679, 16546172, 5092863, 16723641, 2592516, 3727103, 16750392, 13573523, 14174975, 8324769, 549281, 11579568, 7237230, 213872, 1745151, 4250636, 10751183,
    16405848, 16555422, 8553090, 10384912, 1730049, 10066329, 7353856, 7229445, 2042063, 9212671,
];

/// Таблица цветов вида палитры `kind` (`paletteKind`); `form` — флаг FUN_18017ef10 (`line_palette` у реквизита формы).
/// Auto — SoftAdaptive (совместимость ≥ 0x139b7, стиль без флагов 0xc0); градиентные и своя палитра — не выведены.
fn palette(kind: &str, form: bool) -> Option<&'static [u32]> {
    Some(match kind {
        "Palette8" => &PALETTE8,
        "Palette32" => &PALETTE32,
        "Pastel" => &PASTEL,
        "Bright" => &BRIGHT,
        "Warm" => &WARM,
        "Cold" => &COLD,
        "Soft" => &AUTO_PALETTE,
        "Rich" if form => &RICH_FORM,
        "Rich" => &RICH,
        "Auto" | "SoftAdaptive" if form => &FORM_PALETTE,
        "Auto" | "SoftAdaptive" => &AUTO_PALETTE,
        _ => return None,
    })
}

/// Флаг палитры серии для FUN_18017ef10 — chart.dll FUN_18002c400(диаграмма, номер серии; сводная — −1), его зовет
/// сериализатор записи FUN_1800399a0 (0x180039d01) перед цветом сводной: «линейные» типы (Line, Step, Stock,
/// OpenHighLowClose, StackedLine, RadarLine, RadarStackedLine, Scatter, Bubble) — истина; гистограммы и линейки
/// (Column…StackedBar3D) — признак серии +0x148 == 1; прочие — ложь. Сверено опытом под Frida (реквизит формы: Pie —
/// 0, Line — 1). shortcut: +0x148 серии с XML не сопоставлен, у гистограмм принят 0 (Column3D форм ERP: сводная —
/// палитра «Авто»); при расхождении у гистограммы с цветом серии auto — разобрать
fn line_palette(chart_type: &str) -> bool {
    matches!(chart_type, "Line" | "Step" | "Stock" | "OpenHighLowClose" | "StackedLine" | "RadarLine" | "RadarStackedLine" | "Scatter" | "Bubble")
}

fn chart_with(x: &Index, obj: &El, form: bool) -> R<Node> {
    let ty = obj.req_attr("xsi:type")?;
    let p = ty.strip_suffix(":Chart").ok_or_else(|| err(obj, "не Chart"))?.to_string();
    if obj.attr(&format!("xmlns:{p}")) != Some(CHART_NS) || obj.attrs.len() != 2 {
        return Err(err(obj, "пространство имен диаграммы"));
    }
    body(x, obj, p, form)
}

/// Свойства диаграммы — дети `obj` с префиксом `p` (у диаграммы Ганта — элемент `<p:chart>`); `form` — реквизит формы.
fn body(x: &Index, obj: &El, p: String, form: bool) -> R<Node> {
    // вид палитры — в XML после серий, а цвет серии с auto берется из него
    let kind = obj.elems().find(|e| e.name == format!("{p}:paletteKind")).map(|e| e.text()).transpose()?.unwrap_or_else(|| "Auto".into());
    let line = form && {
        let ty = obj.elems().find(|e| e.name == format!("{p}:chartType")).map(|e| e.text()).transpose()?;
        line_palette(&ty.ok_or_else(|| err(obj, "диаграмма реквизита формы без chartType: флаг палитры не выведен"))?)
    };
    let pal = palette(&kind, line).ok_or_else(|| err(obj, &format!("палитра {kind}: цвета не выведены")))?;
    let mut c = Cur { kids: obj.elems().collect(), i: 0, p, x, at: obj };
    let x_labels;
    let mut v = vec![a("75"), c.i("seriesCurId")?, c.i("pointsCurId")?, c.b("isSeriesDesign")?];
    let ns: usize = c.text("realSeriesCount")?.parse().map_err(|_| err(obj, "realSeriesCount"))?;
    v.push(a(ns.to_string()));
    let mut ids = Vec::new();
    let mut looks = Vec::new();
    for k in 0..ns {
        let (b, id, col, m) = series(&mut c, "realSeriesData", false, k as i64, pal)?;
        v.extend(b);
        ids.push(id);
        looks.push((col, m));
    }
    // сводная — тот же флаг (FUN_18002c400 с номером −1)
    let (b, ex_id, col, m) = series(&mut c, "realExSeriesData", false, -1, pal)?;
    v.extend(b);
    looks.push((col, m));
    v.push(c.b("isPointsDesign")?);
    let np: usize = c.text("realPointCount")?.parse().map_err(|_| err(obj, "realPointCount"))?;
    v.push(a(np.to_string()));
    let mut point_colors = Vec::new();
    for k in 0..np {
        let (b, _, col, _) = series(&mut c, "realPointData", true, k as i64, pal)?;
        v.extend(b);
        point_colors.push(col);
    }
    // свойства r0…r99
    v.extend([c.i("curSeries")?, c.i("curPoint")?]);
    let kind = c.en("chartType", CHART_TYPE)?;
    v.extend([kind.clone(), c.en("circleLabelType", LABEL_TYPE)?]);
    v.push(s(crlf(&c.text("labelsDelimiter")?)));
    v.extend([c.en("labelsLocation", &[(0, "Edge"), (4, "Auto")])?, c.ml("lbFormat")?, c.ml("lbpFormat")?, c.color("labelsColor")?]);
    let labels = [c.font("labelsFont")?, c.b("transparentLabelsBkg")?, c.color("labelsBkgColor")?, c.border("labelsBorder")?, c.color("labelsBorderColor")?];
    c.fixed("circleExpandMode", "None")?;
    c.fixed("chart3Dcrd", "SouthWest")?;
    v.extend([a("0"), a("0"), c.ml("title")?]);
    let (show_title, show_legend) = (c.b("isShowTitle")?, c.b("isShowLegend")?);
    v.extend([show_title.clone(), show_legend.clone()]);
    for t in ["ttl", "lg", "ch"] {
        v.push(c.border(&format!("{t}Border"))?);
        v.push(c.color(&format!("{t}BorderColor"))?);
    }
    v.extend([c.b("transparent")?, c.color("bkgColor")?]);
    for t in ["Ttl", "Leg", "Ch"] {
        v.push(c.b(&format!("isTrnsp{t}"))?);
        v.push(c.color(&format!("{}Color", t.to_lowercase()))?);
    }
    for t in ["ttl", "leg", "ch"] {
        v.push(c.color(&format!("{t}TxtColor"))?);
    }
    for t in ["ttl", "leg", "ch"] {
        v.push(c.font(&format!("{t}Font"))?);
    }
    for t in ["isShowScale", "isShowScaleVL", "isShowSeriesScale", "isShowPointsScale", "isShowValuesScale"] {
        v.push(c.b(t)?);
    }
    v.extend([c.ml("vsFormat")?, {
        // ориентация подписей шкалы точек — хвост T1 (ChartLabelsOrientation, chart.dll FUN_1800ab5e0); r40 — старый флажок
        let t = c.text("xLabelsOrientation")?;
        x_labels = ["Horizontal", "Vertical", "Auto", "CustomAngle"].iter().position(|o| *o == t).ok_or_else(|| err(obj, &format!("xLabelsOrientation {t}")))?;
        a("0")
    }, c.line("scaleLine")?, c.color("scaleColor")?]);
    v.extend([c.b("isAutoSeriesName")?, c.b("isAutoPointName")?, c.en("maxMode", &[(0, "NotDefined")])?, c.i("maxSeries")?, c.i("maxSeriesPrc")?]);
    v.push(c.en("spaceMode", &[(1, "Half")])?);
    let bv = c.take("baseVal")?;
    v.push(a(dec(bv, &bv.text()?)?));
    v.extend([c.b("isOutline")?, c.i("realPiePoint")?, c.i("realStockSeries")?]);
    for t in ["isLight", "isGradient", "isTransposition", "hideBaseVal", "dataTable", "dtVerLines", "dtHorLines"] {
        v.push(c.b(t)?);
    }
    v.extend([c.en("dtHAlign", &[(2, "Right")])?, c.ml("dtFormat")?, c.b("dtKeys")?]);
    // вид палитры — хвост T0 (chart.dll FUN_1800399a0), номер — позиция в PALETTE
    let pk = c.text("paletteKind")?;
    let palette = PALETTE.iter().position(|p| *p == pk).ok_or_else(|| format!("диаграмма: paletteKind {pk:?}"))?;
    let animation = c.en("animation", &[(0, "Auto"), (1, "Use"), (2, "DontUse")])?;
    let rebuild = c.i("rebuildTime")?;
    // r63, r64 — isTransposed и autoTransposition (поля 0x118a, 0x1189)
    v.extend([c.b("isTransposed")?, c.b("autoTransposition")?, c.b("legendScrollEnable")?, c.color("surfaceColor")?, c.en("radarScaleType", &[(0, "Circle")])?]);
    v.extend([c.en("gaugeValuesPresentation", &[(0, "Needle")])?, bands(&mut c)?]);
    v.extend([c.i("beginGaugeAngle")?, c.i("endGaugeAngle")?, c.i("gaugeThickness")?, c.en("gaugeLabelsLocation", &[(1, "InsideScale")])?]);
    v.extend([c.b("gaugeLabelsArcDirection")?, c.i("gaugeBushThickness")?, c.color("gaugeBushColor")?, c.b("autoMaxValue")?]);
    let mx = c.take("userMaxValue")?;
    v.push(a(sci(mx, &mx.text()?)?));
    v.push(c.b("autoMinValue")?);
    let mn = c.take("userMinValue")?;
    v.push(a(sci(mn, &mn.text()?)?));
    let init = c.b("elementsIsInit")?.atom() == Some("1");
    v.push(a(if init { "1" } else { "0" }));
    // 12 чисел (рамки областей) — после расположения областей, оно ниже по XML
    let tw1 = v.len();
    v.extend((0..12).map(|_| a("0")));
    let inits = [c.b("titleIsInit")?, c.b("legendIsInit")?, c.b("chartIsInit")?];
    let frames = (inits[0].atom() == Some("1"), inits[1].atom() == Some("1"));
    let areas = [area(&mut c, "elementsChart")?, area(&mut c, "elementsLegend")?, area(&mut c, "elementsTitle")?];
    v.extend([a("0"), c.color("borderColor")?, c.border("border")?]);
    c.fixed("dataSourceDescription", "")?;
    c.fixed("isDataSourceMode", "false")?;
    v.extend([s(""), a("0"), c.b("isRandomizedNewValues")?]);
    // данные: на каждую пару серия × точка — значение, расшифровка, подсказка
    if let Some(items) = c.opt("realDataItems") {
        let its: Vec<&El> = items.elems().collect();
        if its.len() != ns * np {
            return Err(err(items, "число значений не серии × точки"));
        }
        for it in its {
            let mut k = Cur { kids: it.elems().collect(), i: 0, p: c.p.clone(), x: c.x, at: it };
            let vd = k.take("valData")?;
            let val = match vd.req_attr("xsi:type")? {
                "xs:decimal" => l(vec![s("N"), a(dec(vd, &vd.text()?)?)]),
                "xs:string" => l(vec![s("S"), s(crlf(&vd.text()?))]),
                t => return Err(err(vd, &format!("значение {t}"))),
            };
            let vi = k.take("valInfo")?;
            if vi.req_attr("xsi:nil")? != "true" {
                return Err(err(vi, "расшифровка значения"));
            }
            vi.text()?;
            k.fixed("toolTip", "")?;
            if it.name != format!("{}:item", c.p) || k.i != k.kids.len() {
                return Err(err(it, "значение"));
            }
            v.extend([val, l(vec![s("U")]), s("")]);
        }
    } else if ns * np > 0 {
        return Err(err(obj, "нет realDataItems"));
    }
    // хвост
    v.extend([a(palette.to_string()), a(x_labels.to_string())]);
    v.extend(labels);
    v.extend(inits);
    let spline = match c.opt("splineMode") {
        Some(e) if e.text()? == "SmoothCurve" => "1",
        Some(e) => return Err(err(e, "splineMode")),
        None => "0",
    };
    v.extend([a(spline), a("0"), c.i("splineStrain")?]);
    c.fixed("translucencePercent", "0")?;
    for t in ["funnelNeckHeightPercent", "funnelNeckWidthPercent", "funnelGapSumPercent"] {
        let e = c.take(t)?;
        v.push(a(percent(e)?));
    }
    v.extend([c.line("multiStageLinkLine")?, c.color("multiStageLinkColor")?, a("2"), a("255"), animation, rebuild, a(ZERO)]);
    // id серий без повторов (у серии бывает id 0, как у сводной)
    let mut uniq: Vec<&Node> = Vec::new();
    for id in std::iter::once(&ex_id).chain(ids.iter()) {
        if !uniq.contains(&id) {
            uniq.push(id);
        }
    }
    v.push(a(uniq.len().to_string()));
    for id in uniq {
        v.push(l(vec![a("0"), id.clone(), a("0")]));
    }
    v.extend((0..=ns).map(|_| l(vec![a("0"), a("0")])));
    v.push(a("0"));
    v.push(values_axis(&mut c)?);
    let lim0 = || l(vec![a("0"), a("0"), l(vec![a("0"), a("1"), a("0"), a("1"), a("0")]), a("0"), a("0")]);
    v.push(lim0());
    v.extend(["0", "0", "2", "-2", "1", "10", "1", "20", "0", "0"].into_iter().map(a));
    c.fixed("pointsAxis", "")?;
    for t in ["pointsScale", "valuesScale", "seriesScale"] {
        v.push(scale(&mut c, t)?);
    }
    v.extend([a("0"), a("0"), auto(), auto(), a("0")]);
    v.extend(point_colors.into_iter().map(|col| l(vec![col])));
    // оформление: серии, затем сводная
    let (ex_look, s_looks) = looks.split_last().unwrap();
    for (col, m) in s_looks.iter().chain(std::iter::once(ex_look)) {
        v.push(l(vec![col.clone(), m.clone(), a("0"), a("0"), a("0"), s(""), l(vec![a("1"), a("0")]), l(vec![a("1"), a("0")]), l(vec![a("1"), a("0")]), a("0")]));
    }
    let tw2 = v.len();
    v.extend((0..12).map(|_| a("0")));
    let legend = match c.opt("legendPlacement").map(|e| e.text()).transpose()?.as_deref() {
        None => "0",
        Some("Left") => "1",
        Some("Right") => "2",
        Some("Top") => "3",
        Some("Bottom") => "4",
        Some("UseCoordinates") => "5",
        Some("None") => "6",
        Some(t) => return Err(err(obj, &format!("legendPlacement {t}"))),
    };
    let plot = match c.opt("plotAreaPlacement").map(|e| e.text()).transpose()?.as_deref() {
        None => "0",
        Some("UseCoordinates") => "1",
        Some(t) => return Err(err(obj, &format!("plotAreaPlacement {t}"))),
    };
    // ChartTitleAreaPlacement — chart.dll 0x1800c3be0
    const TITLE: [&str; 9] = ["Auto", "UseCoordinates", "LeftTop", "Top", "RightTop", "LeftBottom", "Bottom", "RightBottom", "None"];
    let title = match c.opt("titleAreaPlacement").map(|e| e.text()).transpose()?.as_deref() {
        None => "0".to_string(),
        Some(t) => TITLE.iter().position(|x| *x == t).map(|i| i.to_string()).ok_or_else(|| err(obj, &format!("titleAreaPlacement {t}")))?,
    };
    // рамки пишет сериализатор FUN_1800399a0 раскладкой FUN_180054650 → FUN_1800a1e80 на холсте 480×360: заголовок не
    // в координатах (место ≠ 1) с пустым текстом (vtbl+0x178, длина 0) — место 8, прямоугольник пуст. При загрузке текст
    // пуст: заголовок «12313» у края Top дал пустую рамку (ERP МобильноеПриложениеНачальнаяСтраница), как и легенда —
    // клетками без текста. shortcut: пустота текста при загрузке выведена по записи, а не по коду — при заголовке с
    // рамкой у края в эталоне пересмотреть
    let title_box = if title == "1" { "1" } else { "8" };
    // Рамки областей (12 чисел дважды: область построения, легенда, заголовок): у загрузки — 0,0,0,0, затем
    // 0,0,1,1 у легенды и заголовка при legendIsInit и titleIsInit (27 диаграмм БП), без них — нули (1). Рамки —
    // раскладка областей на холсте 480×360: при UseCoordinates всех трех — области XML, округленные до точки
    // (диаграмма Ганта АнализЖурналаРегистрации); иначе раскладку легенды платформа считает по тексту — отказ.
    // Реквизит формы: диаграмма с сериями и легендой у края — платформа при загрузке раскладывает легенду на холсте
    // 480×360 клетками на серию (линейная 32×19, гистограмма 19×19; сводная не в счет; от текста серий, размеров поля
    // и областей XML не зависит), по центру края, нечетная точка — вниз; область построения отступает от того края на
    // клетку (опыты ibcmd config import на 1–4 сериях, все четыре края: ПроверкаКонтрагента БП). shortcut: прочие виды
    // (у круговой раскладка своя), заголовок и область построения с координатами — отказ
    let edge = show_legend.atom() == Some("1") && ns > 0 && matches!(legend, "1" | "2" | "3" | "4");
    match frames {
        (true, true) if form && edge => {
            let cell = match kind.atom() {
                Some("0") => 32,
                Some("6") => 19,
                _ => 0,
            };
            if cell == 0 || show_title.atom() != Some("0") || plot != "0" || title_box != "8" {
                return Err(err(obj, "легенда диаграммы реквизита формы: раскладка выведена только для линейной и гистограммы без заголовка"));
            }
            let (w, h) = (cell * ns as u32, 19 * ns as u32);
            if w > 480 || h > 360 {
                return Err(err(obj, "легенда больше холста: раскладка не выведена"));
            }
            let (l, t) = ((480 - w) / 2, (360 - h) / 2);
            // (слева, сверху, справа, снизу) — отступы в точках от краев холста
            let (plot_px, legend_px): ([u32; 4], [u32; 4]) = match legend {
                "1" => ([cell, 0, 0, 0], [0, t, 480 - cell, 360 - h - t]),
                "2" => ([0, 0, cell, 0], [480 - cell, t, 0, 360 - h - t]),
                "3" => ([0, 19, 0, 0], [l, 0, 480 - w - l, 341]),
                _ => ([0, 0, 0, 19], [l, 341, 480 - w - l, 0]),
            };
            for (k, px) in plot_px.iter().chain(&legend_px).enumerate() {
                let (n, d) = frame_px(*px, if k % 2 == 0 { 480 } else { 360 });
                v[tw1 + k] = a(n);
                v[tw2 + k] = a(d);
            }
            for (k, n) in ["0", "0", "1", "1"].into_iter().enumerate() {
                v[tw1 + 8 + k] = a(n);
                v[tw2 + 8 + k] = a(n);
            }
        }
        // легенда None и заголовок без координат — их рамка пуста, 0,0,1,1 (5 отчетов-диаграмм ERP,
        // МобильноеПриложениеНачальнаяСтраница), области с координатами — из XML (FUN_1800a17d0)
        (true, true) if plot == "1" && matches!(legend, "5" | "6") => {
            for (k, x) in areas.iter().flatten().enumerate() {
                let (n, d) = match (legend, title_box, k) {
                    ("6", _, 4..=7) | (_, "8", 8..=11) => (["0", "0", "1", "1"][k % 4].to_string(), ["0", "0", "1", "1"][k % 4].to_string()),
                    _ => frame(obj, x, if k % 2 == 0 { 480 } else { 360 })?,
                };
                v[tw1 + k] = a(n);
                v[tw2 + k] = a(d);
            }
        }
        (true, true) => {
            if legend == "5" || title == "1" {
                return Err(err(obj, "рамки областей при расположении легенды или заголовка UseCoordinates (не всех трех областей) не выводятся"));
            }
            for at in [tw1, tw2] {
                for (k, n) in ["0", "0", "0", "0", "0", "0", "1", "1", "0", "0", "1", "1"].into_iter().enumerate() {
                    v[at + k] = a(n);
                }
            }
        }
        // без titleIsInit и legendIsInit, но область построения и легенда UseCoordinates — рамки из XML, заголовок None —
        // 0,0,1,1 (СравнительныйАнализМенеджеров ERP). shortcut: по одному макету, прочие сочетания — нули
        (false, false) if plot == "1" && legend == "5" && title == "8" => {
            for (k, x) in areas.iter().flatten().enumerate() {
                let (n, d) = match k {
                    8..=11 => (["0", "0", "1", "1"][k - 8].to_string(), ["0", "0", "1", "1"][k - 8].to_string()),
                    _ => frame(obj, x, if k % 2 == 0 { 480 } else { 360 })?,
                };
                v[tw1 + k] = a(n);
                v[tw2 + k] = a(d);
            }
        }
        (false, false) => {}
        _ => return Err(err(obj, "titleIsInit и legendIsInit различаются — рамки областей не выводятся")),
    }
    v.extend([a(plot), a(legend), a(title)]);
    for ar in areas {
        v.extend(ar.into_iter().map(a));
    }
    v.extend((0..4).map(|_| l(vec![a("0"), a("0")])));
    // T79 — описание палитры (`colorPaletteDescription`, в конце XML), T80 — палитра полос отметок
    let pal_at = v.len();
    for _ in 0..2 {
        v.push(l(vec![a("0"), a("14"), auto(), auto(), a("0"), a("0")]));
    }
    v.extend([a("0"), a("0")]);
    let hints = if c.peek("valuesToolTipShowMode") {
        c.fixed("valuesToolTipShowMode", "ShowOnHover")?;
        c.fixed("pointsDropLinesShowMode", "DontShow")?;
        c.fixed("valuesDropLinesShowMode", "DontShow")?;
        ["0", "2", "0", "2", "2"]
    } else {
        ["0"; 5]
    };
    v.push(l(hints.into_iter().map(a).collect()));
    v.extend([l(vec![a("0"), a("0"), a("0"), a("0")]), a("0")]);
    for _ in 0..ns * np {
        v.push(l(vec![l(vec![a("1"), l(vec![a("1"), a("0")]), a("0")]), a("0")]));
    }
    v.extend([a(""), a("60")]);
    // шкала по умолчанию, пределы, 7 нулей версии 74 и 8 цветов версии 75 (у загрузки — авто)
    let mut dummy = Cur { kids: Vec::new(), i: 0, p: String::new(), x, at: obj };
    v.push(scale(&mut dummy, "-")?);
    v.push(lim0());
    v.extend((0..7).map(|_| a("0")));
    // 8 цветов версии 75 — копии классических (chart.dll запись FUN_1800399a0, чтение FUN_18003b930): цвет, равный
    // умолчанию свойства, копируется как auto. Умолчания: стиль −1 (фон, заголовок, легенда), белый (область
    // построения), стиль −3 (тексты и подписи)
    let style = |k: &str| l(vec![a("4"), a("3"), l(vec![a(k)]), a("3")]);
    let white = l(vec![a("4"), a("0"), l(vec![a("16777215")]), a("0")]);
    for (t, d) in [("bkgColor", style("-1")), ("chColor", white), ("ttlColor", style("-1")), ("legColor", style("-1")), ("ttlTxtColor", style("-3")), ("legTxtColor", style("-3")), ("chTxtColor", style("-3")), ("labelsColor", style("-3"))] {
        let e = obj.elems().find(|e| e.name.rsplit(':').next() == Some(t)).ok_or_else(|| err(obj, &format!("нет {t}")))?;
        let c = color_node(x, e)?;
        v.push(if c == d { auto() } else { c });
    }
    // описание палитры (chart.dll FUN_1801dd9d0): только вид палитры — цвета градиента и своя палитра не сверены
    if let Some(d) = c.opt("colorPaletteDescription") {
        let k = match d.elems().collect::<Vec<_>>().as_slice() {
            [e] if e.name == "d4p1:colorPalette" || e.name.ends_with(":colorPalette") || e.name == "colorPalette" => {
                let t = e.text()?;
                PALETTE.iter().position(|p| *p == t).ok_or_else(|| err(e, &format!("палитра {t:?}")))?
            }
            _ => return Err(err(d, "описание палитры: состав не сверен")),
        };
        v[pal_at] = l(vec![a("0"), a(k.to_string()), auto(), auto(), a("0"), a("0")]);
    }
    if c.i != c.kids.len() {
        return Err(err(c.kids[c.i], "свойство не загружается"));
    }
    Ok(l(v))
}

const TIME_UNIT: &[(i64, &str)] = &[(5, "Second"), (10, "Minute"), (20, "Hour"), (30, "Day"), (50, "Month")];

/// Дата `ГГГГ-ММ-ДДTчч:мм:сс` → `ГГГГММДДччммсс`.
fn date(e: &El) -> R<Node> {
    let t = e.text()?;
    let d: String = t.chars().filter(|c| c.is_ascii_digit()).collect();
    let shape = t.len() == 19 && t.as_bytes()[4] == b'-' && t.as_bytes()[7] == b'-' && t.as_bytes()[10] == b'T' && t.as_bytes()[13] == b':' && t.as_bytes()[16] == b':';
    if !shape || d.len() != 14 {
        return Err(err(e, &format!("дата {t:?}")));
    }
    Ok(a(d))
}

/// Шкала времени `{3,положение,N,уровень×N,прозрачность,цвет фона,цвет текста,текущий уровень}`, уровень —
/// `{8,единица,интервал,показ,линия,цвет шкалы,правило дня,формат,{0,{1,0,0}},цвет фона,цвет текста,подписи}`.
fn time_scale(c: &mut Cur) -> R<Node> {
    let e = c.take("timeScale")?;
    let mut k = Cur { kids: e.elems().collect(), i: 0, p: c.p.clone(), x: c.x, at: e };
    let mut v = vec![a("3"), k.en("placement", &[(0, "Top"), (2, "Left")])?];
    let mut levels = Vec::new();
    while k.peek("level") {
        let lv = k.take("level")?;
        let mut m = Cur { kids: lv.elems().collect(), i: 0, p: c.p.clone(), x: c.x, at: lv };
        let mut f = vec![a("8"), m.en("measure", TIME_UNIT)?, m.i("interval")?, m.b("show")?, m.line("line")?, m.color("scaleColor")?];
        f.push(m.en("dayFormatRule", &[(1, "MonthDay"), (2, "WeekDay"), (3, "MonthDayWeekDay")])?);
        f.push(m.ml("format")?);
        let lb = m.take("labels")?;
        let mut t = Cur { kids: lb.elems().collect(), i: 0, p: c.p.clone(), x: c.x, at: lb };
        t.fixed("ticks", "0")?;
        if t.i != t.kids.len() {
            return Err(err(lb, "подписи шкалы"));
        }
        f.push(l(vec![a("0"), l(vec![a("1"), a("0"), a("0")])]));
        f.extend([m.color("backColor")?, m.color("textColor")?, m.b("showPereodicalLabels")?]);
        if m.i != m.kids.len() {
            return Err(err(lv, "лишнее в уровне шкалы"));
        }
        levels.push(l(f));
    }
    if levels.is_empty() {
        return Err(err(e, "шкала без уровней"));
    }
    v.push(a(levels.len().to_string()));
    v.extend(levels);
    v.extend([k.b("transparent")?, k.color("backColor")?, k.color("textColor")?, k.i("currentLevel")?]);
    if k.i != k.kids.len() {
        return Err(err(e, "лишнее в шкале времени"));
    }
    Ok(l(v))
}

/// Точки или серии диаграммы Ганта — пустой элемент с baseData (так у обеих диаграмм БП; иное — отказ).
fn gantt_items(c: &mut Cur, tag: &str, points: bool) -> R<Node> {
    let e = c.take(tag)?;
    let mut k = Cur { kids: e.elems().collect(), i: 0, p: c.p.clone(), x: c.x, at: e };
    k.fixed("testMode", "false")?;
    let val = k.take("value")?;
    let mut vv = Cur { kids: val.elems().collect(), i: 0, p: c.p.clone(), x: c.x, at: val };
    for t in ["itemKey", "key", "parentKey", "leftKey", "rightKey", "extKey"] {
        vv.fixed(t, "0")?;
    }
    vv.fixed("title", "")?;
    vv.fixed("cacheKey", "0")?;
    let bd = vv.text("baseData")?;
    if bd.is_empty() || !bd.bytes().all(|b| b.is_ascii_digit()) {
        return Err(err(val, "baseData"));
    }
    if points {
        if vv.font("font")? != font_auto() {
            return Err(err(val, "шрифт точек не по умолчанию"));
        }
        vv.fixed("picture", "")?;
    }
    let cc = k.take("contentCacheItem")?;
    let mut m = Cur { kids: cc.elems().collect(), i: 0, p: c.p.clone(), x: c.x, at: cc };
    m.fixed("mainColor", "#000000")?;
    m.fixed("secondColor", "#000000")?;
    let hatch = if points {
        m.fixed("backColor", "auto")?;
        m.fixed("textColor", "auto")?;
        None
    } else {
        Some(m.color("hatchBetweenIntervalsColor")?)
    };
    let auto_text = k.b("autoText")?;
    let reverse = k.b("useValuesReverseBehavior")?;
    if k.i != k.kids.len() || vv.i != vv.kids.len() || m.i != m.kids.len() {
        return Err(err(e, "лишнее в точках или сериях"));
    }
    let key = l(vec![a("8"), a("0"), a("0"), a("0"), a("0"), a("0"), l(vec![s("U")]), l(vec![a("1"), a("0")]), l(vec![s("U")]), a("0"), a(bd)]);
    let black = || l(vec![a("4"), a("0"), l(vec![a("0")]), a("0")]);
    Ok(if points {
        let pic = l(vec![a("4"), a("0"), l(vec![a("0")]), s(""), a("-1"), a("-1"), a("1"), a("0"), s("")]);
        l(vec![a("1"), l(vec![a("3"), a("0"), a("1"), a("0"), l(vec![a("2"), key, pic, font_auto()]), l(vec![a("0"), a("1"), l(vec![a("0"), l(vec![a("0"), black(), black()]), auto(), auto()])]), auto_text, reverse])])
    } else {
        l(vec![a("0"), l(vec![a("3"), a("0"), a("1"), a("0"), l(vec![a("3"), key]), l(vec![a("0"), a("1"), l(vec![a("0"), l(vec![a("0"), black(), black()]), hatch.unwrap_or_else(black)])]), auto_text, reverse])])
    })
}

/// `<object xsi:type="dNp1:GanttChart">` → `{19,{0,{11},{диаграмма}},точки,серии,0,0,drawEmpty,шкала времени,…}`
/// (версия 19 — так пишет загрузка 8.5; раскладка — xml-mxl.md «Диаграмма Ганта»).
pub fn gantt(x: &Index, obj: &El) -> R<Node> {
    let ty = obj.req_attr("xsi:type")?;
    let p = ty.strip_suffix(":GanttChart").ok_or_else(|| err(obj, "не GanttChart"))?.to_string();
    if obj.attr(&format!("xmlns:{p}")) != Some(CHART_NS) || obj.attrs.len() != 2 {
        return Err(err(obj, "пространство имен диаграммы Ганта"));
    }
    let mut c = Cur { kids: obj.elems().collect(), i: 0, p: p.clone(), x, at: obj };
    let inner = c.take("chart")?;
    let mut v = vec![a("19"), l(vec![a("0"), l(vec![a("11")]), body(x, inner, p, false)?])];
    v.push(gantt_items(&mut c, "points", true)?);
    v.push(gantt_items(&mut c, "series", false)?);
    v.extend([a("0"), a("0"), c.b("drawEmpty")?, time_scale(&mut c)?]);
    v.extend([c.en("keepScaleVariant", &[(2, "AllData")])?, c.en("fixedVariantMeasure", TIME_UNIT)?, c.i("fixedVariantInterval")?, c.b("autoFullInterval")?]);
    for t in ["fullIntervalBegin", "fullIntervalEnd", "visualBegin"] {
        let e = c.take(t)?;
        v.push(date(e)?);
    }
    v.extend([c.en("intervalDrawType", &[(0, "Flat"), (3, "Gradient")])?, c.i("noneVariantChars")?, c.en("noneVariantMeasure", TIME_UNIT)?]);
    v.extend([c.en("verticalStretch", &[(0, "None")])?, c.b("verticalScrollEnable")?, c.en("showValueText", GANTT_VALUE_TEXT)?, c.ml("extTitle")?, c.color("outboundColor")?]);
    // фоновые интервалы: пустые (`ticks` 0)
    let bi = c.take("backIntervals")?;
    let mut k = Cur { kids: bi.elems().collect(), i: 0, p: c.p.clone(), x, at: bi };
    let coll = k.take("collection")?;
    let mut t = Cur { kids: coll.elems().collect(), i: 0, p: c.p.clone(), x, at: coll };
    t.fixed("ticks", "0")?;
    k.fixed("ticks", "0")?;
    if k.i != k.kids.len() || t.i != t.kids.len() {
        return Err(err(bi, "фоновые интервалы"));
    }
    v.push(l(vec![a("3"), l(vec![a("0"), l(vec![a("1"), a("0"), a("0")]), a("0")]), l(vec![a("0"), a("0")])]));
    v.extend([a("0"), c.color("linksColor")?, c.line("linksLine")?, l(vec![a("0"), a("0"), a("0")])]);
    v.push(c.en("showPointsText", &[(0, "Auto"), (1, "Show")])?);
    // showData и intervalTextRepresentation — Auto (0); между ними 1 и textPlacement (chart.dll FUN_1800de410)
    c.fixed("showData", "Auto")?;
    let text = c.en("textPlacement", &[(0, "Auto"), (1, "Cut"), (2, "Wrap")])?;
    c.fixed("intervalTextRepresentation", "Auto")?;
    v.extend([a("0"), a("1"), text, a("0")]);
    if c.i != c.kids.len() {
        return Err(err(c.kids[c.i], "свойство диаграммы Ганта не загружается"));
    }
    Ok(l(v))
}

/// Процент XML (10) → доля записи (1e-1).
fn percent(e: &El) -> R<String> {
    let t = e.text()?;
    let v: u32 = t.parse().map_err(|_| err(e, &format!("процент {t:?}")))?;
    let frac = if v == 0 { "0".to_string() } else if v % 100 == 0 { (v / 100).to_string() } else { format!("0.{:02}", v) };
    sci(e, &frac)
}

/// Ось значений `{0,базовое,{0,1,мин,1,макс},метод макс,метод мин}` (2 — UseValueWithLimitations).
fn values_axis(c: &mut Cur) -> R<Node> {
    let e = c.take("valuesAxis")?;
    let mut k = Cur { kids: e.elems().collect(), i: 0, p: c.p.clone(), x: c.x, at: e };
    let num = |k: &mut Cur, t: &str, typed: bool| -> R<String> {
        match k.opt(t) {
            Some(x) => {
                if typed && x.req_attr("xsi:type")? != "xs:decimal" {
                    return Err(err(x, "тип предела"));
                }
                dec(x, &x.text()?)
            }
            None => Ok("0".into()),
        }
    };
    let bv = num(&mut k, "baseValue", false)?;
    let min = num(&mut k, "minValue", true)?;
    let max = num(&mut k, "maxValue", true)?;
    let method = |k: &mut Cur, t: &str| -> R<&'static str> {
        match k.opt(t) {
            Some(x) if x.text()? == "UseValueWithLimitations" => Ok("2"),
            Some(x) => Err(err(x, "метод предела")),
            None => Ok("0"),
        }
    };
    let min_m = method(&mut k, "minValueDetectionMethod")?;
    let max_m = method(&mut k, "maxValueDetectionMethod")?;
    if k.i != k.kids.len() {
        return Err(err(e, "лишнее в оси значений"));
    }
    if k.kids.is_empty() {
        e.text()?;
    }
    Ok(l(vec![a("0"), a(bv), l(vec![a("0"), a("1"), a(min), a("1"), a(max)]), a(max_m), a(min_m)]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numeric_double() {
        // 338/360: на ulp ниже точного округления — как у платформы (ЗаказНаПроизводство ERP)
        assert_eq!(frame_px(338, 360).0, "9.388888888888888e-1");
        assert_eq!(frame_px(60, 360).0, "1.666666666666667e-1");
        assert_eq!(frame_px(300, 360).0, "8.333333333333334e-1");
        assert_eq!(frame_px(22, 360).0, "6.111111111111111e-2");
    }

    #[test]
    fn numbers() {
        let e = super::super::dom::parse("<x/>").unwrap();
        for (x, r) in [("500", "5e2"), ("160", "1.6e2"), ("301", "301"), ("-1000", "-1e3"), ("0.1", "1e-1"), ("0.03", "3e-2"), ("5.48", "5.48"), ("0", "0"), ("5", "5")] {
            assert_eq!(sci(&e, x).unwrap(), r, "{x}");
        }
    }

    /// Рамки из записей загрузки БП (диаграмма Ганта и диаграмма формы ПроверкаКонтрагента).
    #[test]
    fn frames() {
        let e = super::super::dom::parse("<x/>").unwrap();
        for (x, size, n, d) in [
            ("0.1675", 480, "1.666666666666667e-1", "0.166666666666666666666666667"),
            ("0.8325", 480, "8.333333333333334e-1", "0.833333333333333333333333333"),
            ("0.0539419087136928", 360, "5.277777777777777e-2", "0.052777777777777777777777778"),
            ("0.00196463654223969", 480, "2.083333333333333e-3", "0.002083333333333333333333333"),
            ("0.945558739255014", 360, "9.444444444444444e-1", "0.944444444444444444444444444"),
            ("0.36666", 480, "3.666666666666666e-1", "0.366666666666666666666666667"),
            ("0.94722", 360, "9.472222222222222e-1", "0.947222222222222222222222222"),
            ("0", 480, "0", "0"),
            ("1", 360, "1", "1"),
        ] {
            assert_eq!(frame(&e, x, size).unwrap(), (n.to_string(), d.to_string()), "{x}");
        }
    }

    #[test]
    fn legend_frame_px() {
        // рамка легенды внизу ПроверкаКонтрагента БП: 19, 132 и 341 точка из 360
        assert_eq!(frame_px(19, 360), ("5.277777777777777e-2".to_string(), "0.052777777777777777777777778".to_string()));
        assert_eq!(frame_px(341, 360), ("9.472222222222222e-1".to_string(), "0.947222222222222222222222222".to_string()));
        assert_eq!(frame_px(0, 480), ("0".to_string(), "0".to_string()));
    }
}
