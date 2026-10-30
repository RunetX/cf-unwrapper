//! Диаграмма (`Chart`, 8.2/data/chart): рисунок табличного документа вида 10 и настройки реквизита формы.
//! Запись `{версия,seriesCurId,pointsCurId,isSeriesDesign,N,серия×N,сводная серия,isPointsDesign,M,точка×M,...}`.
//! Сверено для версий 54 (13 диаграмм БП: ФинансовыйАнализ — Оценки, Коэффициенты, Рентабельность), 73 и 74 (макеты
//! ДосьеКонтрагента и ПроверкаКонтрагента.ФинансовыйАнализ, формы ПроверкаКонтрагента.Форма и ТестИПилиООО);
//! поля, которых XML не выражает или которые в образцах не менялись, должны совпасть со встреченным значением —
//! иначе отказ. Раскладка — docs/research/xml-mxl.md, разделы «Диаграмма» и «Проход r9». Здесь же планировщик и
//! шкала времени.

use crate::brace::{self, Node};

type R<T> = Result<T, String>;

fn short(n: &Node) -> String {
    brace::write(n).unwrap_or_default().replace("\r\n", "")
}

fn atom(n: Option<&Node>) -> R<&str> {
    n.and_then(Node::atom).ok_or_else(|| format!("диаграмма: ждали число, а не {}", n.map(short).unwrap_or_default()))
}

fn int(n: Option<&Node>) -> R<i64> {
    atom(n)?.parse().map_err(|_| format!("диаграмма: ждали целое, а не {}", n.map(short).unwrap_or_default()))
}

fn esc(s: &str) -> String {
    s.replace("\r\n", "\n").replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

/// Число записи (`5e2`, `-3e2`, `8.318042813455651e-1`) → десятичная запись XML; у числа с порядком больше 15
/// значащих цифр — округление до 15 (так пишет платформа: 0.831804281345565), десятичная запись — как есть.
pub fn dec(a: &str) -> Option<String> {
    let (neg, a) = a.strip_prefix('-').map_or((false, a), |r| (true, r));
    let (m, e) = a.split_once(['e', 'E']).map_or((a, 0), |(m, e)| (m, e.parse::<i32>().unwrap_or(i32::MAX)));
    if e == i32::MAX || m.is_empty() {
        return None;
    }
    let (ip, fp) = m.split_once('.').unwrap_or((m, ""));
    if !(ip.chars().chain(fp.chars()).all(|c| c.is_ascii_digit())) {
        return None;
    }
    let mut digits: Vec<u8> = ip.bytes().chain(fp.bytes()).map(|b| b - b'0').collect();
    let mut point = ip.len() as i32 + e; // позиция запятой от начала digits
    // Ведущие нули не значащие.
    while digits.len() > 1 && digits[0] == 0 {
        digits.remove(0);
        point -= 1;
    }
    // Число с порядком — double: больше 15 значащих цифр округляется; десятичная запись (поля 8.3, ПроверкаКонтрагента.
    // Форма: 0.14968152866242038) переносится как есть.
    let double = e != 0 || a.contains(['e', 'E']);
    if double && digits.len() > 17 {
        return None;
    }
    if double && digits.len() > 15 {
        let up = digits[15] >= 5;
        digits.truncate(15);
        if up {
            let mut i = 15;
            loop {
                if i == 0 {
                    digits.insert(0, 1);
                    point += 1;
                    break;
                }
                i -= 1;
                if digits[i] == 9 {
                    digits[i] = 0;
                } else {
                    digits[i] += 1;
                    break;
                }
            }
        }
    }
    while digits.len() > 1 && *digits.last().unwrap() == 0 && (digits.len() as i32) > point {
        digits.pop();
    }
    let s: String = digits.iter().map(|d| char::from(b'0' + d)).collect();
    let mut out = if point <= 0 {
        format!("0.{}{s}", "0".repeat((-point) as usize))
    } else if point as usize >= s.len() {
        format!("{s}{}", "0".repeat(point as usize - s.len()))
    } else {
        format!("{}.{}", &s[..point as usize], &s[point as usize..])
    };
    if out == "0" || out.chars().all(|c| c == '0' || c == '.') {
        out = "0".into();
    } else if neg {
        out.insert(0, '-');
    }
    Some(out)
}

struct X<'a> {
    p: &'a str,
    lines: Vec<String>,
    depth: usize,
}

impl X<'_> {
    fn raw(&mut self, l: &str) {
        self.lines.push(format!("{}{l}", "\t".repeat(self.depth)));
    }
    /// Имя элемента с префиксом; пустой префикс — без него (пространство имен по умолчанию).
    fn t(&self, tag: &str) -> String {
        if self.p.is_empty() { tag.to_string() } else { format!("{}:{tag}", self.p) }
    }
    fn leaf(&mut self, tag: &str, v: &str) {
        let t = self.t(tag);
        if v.is_empty() {
            self.raw(&format!("<{t}/>"));
        } else {
            self.raw(&format!("<{t}>{v}</{t}>"));
        }
    }
    fn open(&mut self, tag: &str) {
        let t = self.t(tag);
        self.raw(&format!("<{t}>"));
        self.depth += 1;
    }
    fn close(&mut self, tag: &str) {
        self.depth -= 1;
        let t = self.t(tag);
        self.raw(&format!("</{t}>"));
    }
    fn b(&mut self, tag: &str, n: Option<&Node>) -> R<()> {
        let v = match atom(n)? {
            "0" => "false",
            "1" => "true",
            _ => return Err(format!("диаграмма: {tag}: {}", n.map(short).unwrap_or_default())),
        };
        self.leaf(tag, v);
        Ok(())
    }
    fn i(&mut self, tag: &str, n: Option<&Node>) -> R<()> {
        let v = int(n)?.to_string();
        self.leaf(tag, &v);
        Ok(())
    }
    fn d(&mut self, tag: &str, n: Option<&Node>) -> R<()> {
        let v = dec(atom(n)?).ok_or_else(|| format!("диаграмма: {tag}: число {}", n.map(short).unwrap_or_default()))?;
        self.leaf(tag, &v);
        Ok(())
    }
    /// Перечисление: код записи → имя; незнакомый код — отказ.
    fn en(&mut self, tag: &str, n: Option<&Node>, vals: &[(i64, &str)]) -> R<()> {
        let c = int(n)?;
        let v = vals.iter().find(|v| v.0 == c).ok_or_else(|| format!("диаграмма: {tag} {c}"))?.1;
        self.leaf(tag, v);
        Ok(())
    }
    /// Многоязычная строка `{1,N,{"язык","текст"}...}`; `{1,0}` — пустой элемент.
    fn ml(&mut self, tag: &str, n: Option<&Node>) -> R<()> {
        let l = n.map(Node::list).unwrap_or(&[]);
        let bad = || format!("диаграмма: {tag}: {}", n.map(short).unwrap_or_default());
        if l.first().and_then(Node::atom) != Some("1") || int(l.get(1))? as usize + 2 != l.len() {
            return Err(bad());
        }
        if l.len() == 2 {
            self.leaf(tag, "");
            return Ok(());
        }
        self.open(tag);
        for it in &l[2..] {
            let (Some(lang), Some(text)) = (it.get(0).and_then(Node::str), it.get(1).and_then(Node::str)) else { return Err(bad()) };
            if lang.is_empty() || text.is_empty() {
                return Err(bad());
            }
            self.raw("<v8:item>");
            self.raw(&format!("\t<v8:lang>{}</v8:lang>", esc(lang)));
            self.raw(&format!("\t<v8:content>{}</v8:content>", esc(text)));
            self.raw("</v8:item>");
        }
        self.close(tag);
        Ok(())
    }
    fn color(&mut self, tag: &str, n: Option<&Node>) -> R<()> {
        let v = color(n)?;
        self.leaf(tag, &v);
        Ok(())
    }
    /// Линия `{4,0,{0},тип,толщина,0,e5cabe59-…,0}` (вид — ChartLineType: 0 None, 1 Solid, 2 Dotted).
    fn line(&mut self, tag: &str, n: Option<&Node>) -> R<()> {
        let s = n.map(short).unwrap_or_default();
        let l = n.map(Node::list).unwrap_or(&[]);
        let style = match (l.len(), l.get(3).and_then(Node::atom)) {
            (8, Some("0")) => "None",
            (8, Some("1")) => "Solid",
            (8, Some("2")) => "Dotted",
            _ => "",
        };
        let w = l.get(4).and_then(Node::atom).filter(|w| w.parse::<u32>().is_ok());
        let ok = !style.is_empty() && short(&Node::List(l[..3].to_vec())) == "{4,0,{0}}" && l[5].atom() == Some("0") && l[6].atom() == Some("e5cabe59-d992-4d31-8086-3116931aff81") && l[7].atom() == Some("0");
        let (true, Some(w)) = (ok, w) else { return Err(format!("диаграмма: {tag}: линия {s}")) };
        let t = self.t(tag);
        self.raw(&format!("<{t} width=\"{w}\" gap=\"false\">"));
        self.raw(&format!("\t<v8ui:style xsi:type=\"v8ui:ChartLineType\">{style}</v8ui:style>"));
        self.raw(&format!("</{t}>"));
        Ok(())
    }
    /// Рамка `{3,0,{0},вид,толщина,0|3,48312c09-…|нулевой}` (ControlBorderType: 0 WithoutBorder, 1 Single); поле 5 и
    /// GUID на XML не влияют (встречены 0 и 3, оба GUID).
    fn border(&mut self, tag: &str, n: Option<&Node>) -> R<()> {
        let s = n.map(short).unwrap_or_default();
        let l = n.map(Node::list).unwrap_or(&[]);
        let style = match l.get(3).and_then(Node::atom) {
            Some("0") => "WithoutBorder",
            Some("1") => "Single",
            _ => "",
        };
        let w = l.get(4).and_then(Node::atom).filter(|w| w.parse::<u32>().is_ok());
        let ok = l.len() == 7
            && short(&Node::List(l[..3].to_vec())) == "{3,0,{0}}"
            && matches!(l[5].atom(), Some("0" | "3"))
            && matches!(l[6].atom(), Some("48312c09-257f-4b29-b280-284dd89efc1e" | "00000000-0000-0000-0000-000000000000"));
        let (true, false, Some(w)) = (ok, style.is_empty(), w) else { return Err(format!("диаграмма: {tag}: рамка {s}")) };
        let t = self.t(tag);
        self.raw(&format!("<{t} width=\"{w}\">"));
        self.raw(&format!("\t<v8ui:style xsi:type=\"v8ui:ControlBorderType\">{style}</v8ui:style>"));
        self.raw(&format!("</{t}>"));
        Ok(())
    }
    /// Шрифт: `{7,3,0,1,100}` — авто, `{7,2,0,{-N},1,100}` — шрифт стиля, `{7,0,575,высота×10,0,0,0,вес,курсив,
    /// подчеркнутый,зачеркнутый,0,0,0,0,0,"гарнитура",1,масштаб}` — абсолютный (как у шрифта элемента формы).
    fn font(&mut self, tag: &str, n: Option<&Node>) -> R<()> {
        let s = n.map(short).unwrap_or_default();
        let l = n.map(Node::list).unwrap_or(&[]);
        let bad = || format!("диаграмма: {tag}: шрифт {s}");
        let a = match (l.len(), l.get(1).and_then(Node::atom)) {
            _ if s == "{7,3,0,1,100}" => "kind=\"AutoFont\"".to_string(),
            (6, Some("2")) if l[2].atom() == Some("0") && short(&Node::List(l[4..].to_vec())) == "{1,100}" => {
                let code = l[3].get(0).and_then(Node::atom).and_then(|c| c.parse().ok()).ok_or_else(bad)?;
                format!("ref=\"style:{}\" kind=\"StyleItem\"", super::form::std_style_font(code).ok_or_else(bad)?)
            }
            // шрифт стиля с переопределенными частями (маска: высота, жирность…) — как в графической схеме
            (_, Some("2")) => Some(super::graph::font(n.ok_or_else(bad)?)).filter(|a| !a.contains('\u{1}')).ok_or_else(bad)?.trim_start().to_string(),
            (19, Some("0")) if l[2].atom() == Some("575") && short(&Node::List(l[4..7].to_vec())) == "{0,0,0}" && short(&Node::List(l[11..16].to_vec())) == "{0,0,0,0,0}" && l[17].atom() == Some("1") => {
                let n = |k: usize| l[k].atom().and_then(|v| v.parse::<i64>().ok()).ok_or_else(bad);
                let f = |k: usize| match n(k)? {
                    0 => Ok("false"),
                    1 => Ok("true"),
                    _ => Err(bad()),
                };
                let bold = match n(7)? {
                    400 => "false",
                    700 => "true",
                    _ => return Err(bad()),
                };
                format!(
                    "faceName=\"{}\" height=\"{}\" bold=\"{bold}\" italic=\"{}\" underline=\"{}\" strikeout=\"{}\" kind=\"Absolute\" scale=\"{}\"",
                    l[16].str().ok_or_else(bad)?.replace('&', "&amp;").replace('"', "&quot;").replace('<', "&lt;"),
                    Some(n(3)?).filter(|h| h % 10 == 0).ok_or_else(bad)? / 10,
                    f(8)?,
                    f(9)?,
                    f(10)?,
                    n(18)?
                )
            }
            _ => return Err(bad()),
        };
        let t = self.t(tag);
        self.raw(&format!("<{t} {a}/>"));
        Ok(())
    }
}

/// Цвет `{3,4,{0}}` — auto, `{3,0,{BGR}}` — #RRGGBB, `{3,3,{-N}}` — стандартный цвет стиля.
fn color(n: Option<&Node>) -> R<String> {
    let bad = || format!("диаграмма: цвет {}", n.map(short).unwrap_or_default());
    let n = n.ok_or_else(bad)?;
    let l = n.list();
    if l.len() != 3 || l[0].atom() != Some("3") {
        return Err(bad());
    }
    let v = int(l[2].get(0))?;
    Ok(match (l[1].atom(), l[2].list().len()) {
        (Some("4"), 1) if v == 0 => "auto".into(),
        (Some("0"), 1) => format!("#{:02X}{:02X}{:02X}", v & 0xff, (v >> 8) & 0xff, (v >> 16) & 0xff),
        (Some("3"), 1) => format!("style:{}", super::form::std_style_color(v).ok_or_else(bad)?),
        _ => return Err(bad()),
    })
}

/// Единица шкалы времени (диаграмма Ганта, планировщик); другие коды не встречались.
const TIME_UNIT: &[(i64, &str)] = &[(5, "Second"), (10, "Minute"), (20, "Hour"), (30, "Day"), (50, "Month")];

/// Дата записи `ГГГГММДДччммсс` → `ГГГГ-ММ-ДДTчч:мм:сс`.
fn date(n: Option<&Node>) -> R<String> {
    let a = atom(n)?;
    if a.len() != 14 || !a.bytes().all(|b| b.is_ascii_digit()) {
        return Err(format!("дата {a}"));
    }
    Ok(format!("{}-{}-{}T{}:{}:{}", &a[..4], &a[4..6], &a[6..8], &a[8..10], &a[10..12], &a[12..14]))
}

/// Шкала времени `{3,положение,N,уровень×N,прозрачность,цвет фона,цвет текста,текущий уровень}`, уровень —
/// `{8,единица,интервал,показ,линия,цвет шкалы,правило дня,формат,{0,{1,0,0}},цвет фона,цвет текста,периодические подписи}`.
/// Сверено: диаграммы Ганта БП (2) и планировщик (1). `ns` — у элементов шкалы свое пространство имен по умолчанию
/// (планировщик: `<placement xmlns="…/chart">`, внутри уровня без префикса), иначе — префикс `x.p`.
fn time_scale(x: &mut X, n: Option<&Node>, ns: bool) -> R<()> {
    let l = n.map(Node::list).unwrap_or(&[]);
    let bad = || format!("шкала времени {}", n.map(short).unwrap_or_default());
    let cnt = int(l.get(2)).map_err(|_| bad())?.max(0) as usize;
    if l.first().and_then(Node::atom) != Some("3") || l.len() != cnt + 7 || cnt == 0 {
        return Err(bad());
    }
    const CHART_NS: &str = " xmlns=\"http://v8.1c.ru/8.2/data/chart\"";
    let p = x.p;
    let ns_attr = if ns { CHART_NS } else { "" };
    // Элемент шкалы: при `ns` — без префикса и с атрибутом пространства имен.
    let top = |x: &mut X, tag: &str, v: &str| {
        if ns {
            x.raw(&format!("<{tag}{CHART_NS}>{v}</{tag}>"));
        } else {
            x.leaf(tag, v);
        }
    };
    x.open("timeScale");
    let placement = match int(l.get(1))? {
        0 => "Top",
        2 => "Left",
        _ => return Err(bad()),
    };
    top(x, "placement", placement);
    for lv in &l[3..3 + cnt] {
        let f = lv.list();
        if f.len() != 12 || f[0].atom() != Some("8") || short(&f[8]) != "{0,{1,0,0}}" {
            return Err(bad());
        }
        if ns {
            x.raw(&format!("<level{ns_attr}>"));
            x.depth += 1;
            x.p = "";
        } else {
            x.open("level");
        }
        x.en("measure", f.get(1), TIME_UNIT)?;
        x.i("interval", f.get(2))?;
        x.b("show", f.get(3))?;
        x.line("line", f.get(4))?;
        x.color("scaleColor", f.get(5))?;
        x.en("dayFormatRule", f.get(6), &[(1, "MonthDay"), (2, "WeekDay"), (3, "MonthDayWeekDay")])?;
        x.ml("format", f.get(7))?;
        x.open("labels");
        x.leaf("ticks", "0");
        x.close("labels");
        x.color("backColor", f.get(9))?;
        x.color("textColor", f.get(10))?;
        x.b("showPereodicalLabels", f.get(11))?;
        x.close("level");
        x.p = p;
    }
    let b = match atom(l.get(cnt + 3))? {
        "0" => "false",
        "1" => "true",
        _ => return Err(bad()),
    };
    top(x, "transparent", b);
    top(x, "backColor", &color(l.get(cnt + 4))?);
    top(x, "textColor", &color(l.get(cnt + 5))?);
    top(x, "currentLevel", &int(l.get(cnt + 6))?.to_string());
    x.close("timeScale");
    Ok(())
}

/// Планировщик `{8,0,0,цвета рамки, фона, текста, линий,шрифт,начало и конец периода отображения,3 флажка,формат
/// заголовков переноса шкалы,единица и повтор периодического варианта,отступы переноса,шкала времени,11 полей,рамка,
/// тип текста новых элементов}`. Образец один (КонтактныеЛица.ФормаЛиды, все значения по умолчанию): поля, которые
/// в нем не менялись, сверяются с ним — иначе отказ. Строки элементов `<Settings xsi:type="pl:Planner">` с отступа 0.
pub fn planner(c: &[Node]) -> R<Vec<String>> {
    let mut x = X { p: "pl", lines: Vec::new(), depth: 0 };
    let fixed = |k: usize, v: &str| -> R<()> {
        match c.get(k).map(short) {
            Some(s) if s == v => Ok(()),
            s => Err(format!("планировщик: поле {k} = {} не сверено", s.unwrap_or_default())),
        }
    };
    if c.len() != 32 {
        return Err(format!("планировщик: {} полей не сверено", c.len()));
    }
    for (k, v) in [(0, "8"), (1, "0"), (2, "0")] {
        fixed(k, v)?;
    }
    for (k, tag) in [(3, "borderColor"), (4, "backColor"), (5, "textColor"), (6, "lineColor")] {
        x.color(tag, c.get(k))?;
    }
    x.font("font", c.get(7))?;
    x.leaf("beginOfRepresentationPeriod", &date(c.get(8))?);
    x.leaf("endOfRepresentationPeriod", &date(c.get(9))?);
    x.b("alignElementsOfTimeScale", c.get(10))?;
    x.b("displayTimeScaleWrapHeaders", c.get(11))?;
    x.b("displayWrapHeaders", c.get(12))?;
    x.ml("timeScaleWrapHeadersFormat", c.get(13))?;
    x.en("periodicVariantUnit", c.get(14), TIME_UNIT)?;
    x.i("periodicVariantRepetition", c.get(15))?;
    x.i("timeScaleWrapBeginIndent", c.get(16))?;
    x.i("timeScaleWrapEndIndent", c.get(17))?;
    time_scale(&mut x, c.get(18), true)?;
    // Поля 19–29 в образце: 0,0,0,0,0,1,1,0,0,2,2 — какие из 19–23 отвечают за три свойства ниже, не различить.
    for (k, v) in [(19, "0"), (20, "0"), (21, "0"), (22, "0"), (23, "0"), (24, "1"), (25, "1"), (26, "0"), (27, "0"), (28, "2"), (29, "2"), (31, "0")] {
        fixed(k, v)?;
    }
    x.leaf("displayCurrentDate", "false");
    x.leaf("itemsTimeRepresentation", "DontDisplay");
    x.leaf("itemsBehaviorWhenSpaceInsufficient", "CollapseItems");
    x.leaf("autoMinColumnWidth", "true");
    x.leaf("autoMinRowHeight", "true");
    x.leaf("minColumnWidth", "0");
    x.leaf("minRowHeight", "0");
    x.leaf("fixDimensionsHeader", "auto");
    x.leaf("fixTimeScaleHeader", "auto");
    x.border("border", c.get(30))?;
    x.leaf("newItemsTextType", "String");
    Ok(x.lines)
}

/// Тип диаграммы: chart.dll FUN_1800a8870 (регистрация значений ChartType).
/// Текст значений диаграммы Ганта (GanttChartValueTextRepresentation): chart.dll FUN_1800af670.
pub(crate) const GANTT_VALUE_TEXT: &[(i64, &str)] = &[(0, "None"), (1, "Right")];

/// Вид подписей к диаграмме (ChartLabelType): таблица chart.dll 0x1800a96e0.
pub(crate) const LABEL_TYPE: &[(i64, &str)] = &[(0, "None"), (1, "Series"), (2, "Value"), (3, "Percent"), (4, "SeriesPercent"), (5, "SeriesValue"), (6, "SeriesValuePercent"), (7, "ValuePercent"), (8, "Point"), (9, "PointPercent"), (10, "PointValue"), (11, "PointValuePercent"), (12, "SeriesPoint"), (13, "SeriesPointPercent"), (14, "SeriesPointValue"), (15, "SeriesPointValuePercent"), (16, "ValueSize"), (17, "SeriesSize"), (18, "SeriesValueSize"), (19, "SeriesPointSize"), (20, "SeriesPointValueSize"), (21, "PointSize"), (22, "PointValueSize")];

/// Вид палитры (ChartColorPalette, регистрация chart.dll FUN_1800c2840): номер — позиция.
pub(crate) const PALETTE: &[&str] = &["Palette8", "Palette32", "Pastel", "Bright", "Soft", "Warm", "Cold", "Blue", "Orange", "Green", "Yellow", "Gray", "Custom", "Gradient", "Auto", "SoftAdaptive", "Rich"];

pub(crate) const CHART_TYPE: &[(i64, &str)] = &[(0, "Line"), (1, "Step"), (2, "Area"), (3, "StackedArea"), (4, "Column"), (5, "StackedColumn"), (6, "Column3D"), (7, "StackedColumn3D"), (8, "Bar"), (9, "StackedBar"), (10, "Bar3D"), (11, "StackedBar3D"), (12, "Pie"), (13, "Pie3D"), (14, "Stock"), (15, "OpenHighLowClose"), (16, "BarGraph"), (17, "CeilGraph"), (18, "TapeGraph"), (19, "PyramidGraph"), (20, "Waterfall"), (21, "WireframeSurface"), (22, "Surface"), (23, "ConvexSurface"), (24, "ConcaveSurface"), (25, "ShadedSurface"), (26, "StackedLine"), (27, "NormalizedArea"), (28, "NormalizedColumn"), (29, "NormalizedColumn3D"), (30, "NormalizedBar"), (31, "NormalizedBar3D"), (32, "Honeycomb"), (33, "RadarLine"), (34, "RadarArea"), (35, "RadarStackedLine"), (36, "RadarStackedArea"), (37, "RadarNormalizedArea"), (38, "Gauge"), (39, "Funnel"), (40, "Funnel3D"), (41, "NormalizedFunnel"), (42, "NormalizedFunnel3D"), (43, "Scatter"), (44, "Bubble"), (45, "Donut"), (46, "Donut3D")];
const MARKER: &[(i64, &str)] = &[(0, "None"), (1, "Rect"), (2, "Circle"), (3, "Rhomb"), (4, "Auto")];

/// Серия или точка: поля записи по порядку XML; `{"U"},{"U"}` — не пишутся (других значений не встречалось).
/// `look` — цвет и маркер из списков оформления (версии 73, 74): XML берет их оттуда, а не из блока.
fn series(x: &mut X, tag: &str, b: &[Node], point: bool, look: (Option<&Node>, Option<&Node>), shift: i64) -> R<()> {
    // Серия: цвет, линия, маркер, текст, strIsChanged, isExpand, isIndicator, id, U, U, colorPriority.
    // Точка: текст, strIsChanged, id, цвет, линия, маркер, isExpand, isIndicator, U, U, colorPriority.
    let [c, l, m, t, s, e, ind, id] = if point { [3, 4, 5, 0, 1, 6, 7, 2] } else { [0, 1, 2, 3, 4, 5, 6, 7] };
    if b.len() != 11 || short(&b[8]) != "{\"U\"}" || short(&b[9]) != "{\"U\"}" {
        return Err(format!("диаграмма: {tag}"));
    }
    x.open(tag);
    x.leaf("id", &(int(b.get(id))? + shift).to_string());
    x.color("color", look.0.or(b.get(c)))?;
    x.line("line", b.get(l))?;
    x.en("marker", look.1.or(b.get(m)), MARKER)?;
    x.ml("text", b.get(t))?;
    x.b("strIsChanged", b.get(s))?;
    x.b("isExpand", b.get(e))?;
    x.b("isIndicator", b.get(ind))?;
    x.b("colorPriority", b.get(10))?;
    x.close(tag);
    Ok(())
}

/// Строки элементов `<объект xsi:type="p:Chart">` с отступа 0; `p` — префикс пространства имен диаграммы.
/// Версия 54 — ниже по номерам полей; 73 и 74 — [`body7x`].
pub fn body(c: &[Node], p: &str) -> R<Vec<String>> {
    body_with(c, p, &[], &|_, _| Ok(None))
}

/// То же для рисунка табличного документа: `sources` — источники данных диаграмм документа, `cell` — текст ячейки.
pub fn body_with(c: &[Node], p: &str, sources: &[Node], cell: &Cells) -> R<Vec<String>> {
    match c.first().and_then(Node::atom) {
        Some("54") if sources.is_empty() => {}
        Some(v @ ("71" | "73" | "74" | "75")) => return body7x(c, p, v.parse().unwrap_or(0), sources, cell),
        _ => return Err(format!("диаграмма: версия {} не сверена", c.first().map(short).unwrap_or_default())),
    }
    let mut x = X { p, lines: Vec::new(), depth: 0 };
    x.i("seriesCurId", c.get(1))?;
    x.i("pointsCurId", c.get(2))?;
    x.b("isSeriesDesign", c.get(3))?;
    let ns = int(c.get(4))?.max(0) as usize;
    x.leaf("realSeriesCount", &ns.to_string());
    let block = |k: usize| c.get(k..k + 11).ok_or_else(|| "диаграмма: короткая запись".to_string());
    for k in 0..ns {
        series(&mut x, "realSeriesData", block(5 + 11 * k)?, false, (None, None), 0)?;
    }
    let q = 5 + 11 * ns;
    series(&mut x, "realExSeriesData", block(q)?, false, (None, None), 0)?;
    x.b("isPointsDesign", c.get(q + 11))?;
    let np = int(c.get(q + 12))?.max(0) as usize;
    x.leaf("realPointCount", &np.to_string());
    for k in 0..np {
        series(&mut x, "realPointData", block(q + 13 + 11 * k)?, true, (None, None), 0)?;
    }
    let base = q + 13 + 11 * np;
    let r = |k: usize| c.get(base + k);
    // Поля, которые в XML не попадают или в образцах не менялись: (номер от base, значение).
    const FIXED54: &[(usize, &str)] = &[
        (9, "0"), (10, "0"), (42, "{3,0,{11119017}}"), (65, "0"), (94, "1"), (95, "{0}"), (98, "\"\""),
        (104, "1"), (105, "2"), (106, "{7,3,0,1,100}"), (107, "1"), (108, "{3,4,{0}}"),
        (109, "{3,0,{0},1,1,0,48312c09-257f-4b29-b280-284dd89efc1e}"), (110, "{3,4,{0}}"), (111, "1"), (112, "1"), (113, "1"),
        (114, "0"), (115, "0"), (122, "2"), (123, "255"), (124, "0"), (125, "1128906422"),
    ];

    // В образцах одна серия и одна точка — одно значение (3 поля); другое число не сверено.
    if (ns, np) != (1, 1) {
        return Err(format!("диаграмма: {ns} серий и {np} точек не сверено"));
    }
    let items = 1;
    for &(k, v) in FIXED54 {
        if r(k).map(short).as_deref() != Some(v) {
            return Err(format!("диаграмма: поле {k} = {} (не сверено, полей {})", r(k).map(short).unwrap_or_default(), c.len() - base));
        }
    }
    if c.len() != base + 127 || !c[base + 126].atom().is_some_and(brace::is_guid) {
        return Err("диаграмма: длина записи".into());
    }
    x.i("curSeries", r(0))?;
    x.i("curPoint", r(1))?;
    x.en("chartType", r(2), CHART_TYPE)?;
    x.en("circleLabelType", r(3), LABEL_TYPE)?;
    let delim = r(4).and_then(Node::str).ok_or("диаграмма: labelsDelimiter")?;
    x.leaf("labelsDelimiter", &esc(delim));
    x.en("labelsLocation", r(5), &[(0, "Edge")])?;
    x.ml("lbFormat", r(6))?;
    x.ml("lbpFormat", r(7))?;
    x.color("labelsColor", r(8))?;
    // Свойств подписей ниже в версии 54 нет — значения по умолчанию.
    x.raw(&format!("<{p}:labelsFont kind=\"AutoFont\"/>"));
    x.leaf("transparentLabelsBkg", "true");
    x.leaf("labelsBkgColor", "auto");
    x.raw(&format!("<{p}:labelsBorder width=\"1\">"));
    x.raw("\t<v8ui:style xsi:type=\"v8ui:ControlBorderType\">Single</v8ui:style>");
    x.raw(&format!("</{p}:labelsBorder>"));
    x.leaf("labelsBorderColor", "auto");
    x.leaf("circleExpandMode", "None");
    x.leaf("chart3Dcrd", "SouthWest");
    x.ml("title", r(11))?;
    x.b("isShowTitle", r(12))?;
    x.b("isShowLegend", r(13))?;
    for (k, tag) in [(14, "ttl"), (16, "lg"), (18, "ch")] {
        x.border(&format!("{tag}Border"), r(k))?;
        x.color(&format!("{tag}BorderColor"), r(k + 1))?;
    }
    x.b("transparent", r(20))?;
    x.color("bkgColor", r(21))?;
    for (k, tag) in [(22, "Ttl"), (24, "Leg"), (26, "Ch")] {
        x.b(&format!("isTrnsp{tag}"), r(k))?;
        x.color(&format!("{}Color", tag.to_lowercase()), r(k + 1))?;
    }
    for (k, tag) in [(28, "ttl"), (29, "leg"), (30, "ch")] {
        x.color(&format!("{tag}TxtColor"), r(k))?;
    }
    for (k, tag) in [(31, "ttl"), (32, "leg"), (33, "ch")] {
        x.font(&format!("{tag}Font"), r(k))?;
    }
    for (k, tag) in [(34, "isShowScale"), (35, "isShowScaleVL"), (36, "isShowSeriesScale"), (37, "isShowPointsScale"), (38, "isShowValuesScale")] {
        x.b(tag, r(k))?;
    }
    x.ml("vsFormat", r(39))?;
    x.en("xLabelsOrientation", r(40), &[(0, "Auto")])?;
    x.line("scaleLine", r(41))?;
    // Цвет шкалы версии 54 платформа не переносит: в образцах {3,0,{11119017}} → auto.
    x.leaf("scaleColor", "auto");
    x.b("isAutoSeriesName", r(43))?;
    x.b("isAutoPointName", r(44))?;
    x.en("maxMode", r(45), &[(0, "NotDefined")])?;
    x.i("maxSeries", r(46))?;
    x.i("maxSeriesPrc", r(47))?;
    x.en("spaceMode", r(48), &[(1, "Half")])?;
    x.d("baseVal", r(49))?;
    x.b("isOutline", r(50))?;
    x.i("realPiePoint", r(51))?;
    x.i("realStockSeries", r(52))?;
    for (k, tag) in [(53, "isLight"), (54, "isGradient"), (55, "isTransposition"), (56, "hideBaseVal"), (57, "dataTable"), (58, "dtVerLines"), (59, "dtHorLines")] {
        x.b(tag, r(k))?;
    }
    x.en("dtHAlign", r(60), &[(2, "Right")])?;
    x.ml("dtFormat", r(61))?;
    x.b("dtKeys", r(62))?;
    x.en("paletteKind", r(63), &[(0, "Auto")])?;
    x.en("animation", r(64), &[(0, "Auto")])?;
    // У версии 54 в образцах поле 125 = 1128906422, а rebuildTime — 8906422 (остаток от 10^7); других не встречалось.
    x.leaf("rebuildTime", "8906422");
    x.leaf("isTransposed", "false");
    x.leaf("autoTransposition", "false");
    x.leaf("legendScrollEnable", "false");
    x.color("surfaceColor", r(66))?;
    x.en("radarScaleType", r(67), &[(0, "Circle")])?;
    x.en("gaugeValuesPresentation", r(68), &[(0, "Needle")])?;
    bands(&mut x, r(69))?;
    x.i("beginGaugeAngle", r(70))?;
    x.i("endGaugeAngle", r(71))?;
    x.i("gaugeThickness", r(72))?;
    x.en("gaugeLabelsLocation", r(73), &[(1, "InsideScale")])?;
    x.b("gaugeLabelsArcDirection", r(74))?;
    x.i("gaugeBushThickness", r(75))?;
    x.color("gaugeBushColor", r(76))?;
    x.b("autoMaxValue", r(77))?;
    x.d("userMaxValue", r(78))?;
    x.b("autoMinValue", r(79))?;
    x.d("userMinValue", r(80))?;
    x.b("elementsIsInit", r(81))?;
    for tag in ["titleIsInit", "legendIsInit", "chartIsInit"] {
        x.leaf(tag, "true");
    }
    // Области: в записи left, top, right, bottom; в XML left, right, top, bottom.
    for (k, tag) in [(82, "elementsChart"), (86, "elementsLegend"), (90, "elementsTitle")] {
        x.open(tag);
        for (j, side) in [(0, "left"), (2, "right"), (1, "top"), (3, "bottom")] {
            x.d(side, r(k + j))?;
        }
        x.close(tag);
    }
    x.color("borderColor", r(96))?;
    x.border("border", r(97))?;
    x.leaf("dataSourceDescription", "");
    x.b("isDataSourceMode", r(99))?;
    x.b("isRandomizedNewValues", r(100))?;
    x.open("realDataItems");
    for k in 0..items {
        let v = r(101 + 3 * k).map(Node::list).unwrap_or(&[]);
        let (Some("N"), Some(val)) = (v.first().and_then(Node::str), v.get(1).and_then(Node::atom)) else {
            return Err(format!("диаграмма: значение {}", r(101 + 3 * k).map(short).unwrap_or_default()));
        };
        if r(102 + 3 * k).map(short).as_deref() != Some("{\"U\"}") || r(103 + 3 * k).and_then(Node::str) != Some("") {
            return Err("диаграмма: расшифровка или подсказка значения не сверены".into());
        }
        x.open("item");
        let val = dec(val).ok_or("диаграмма: значение")?;
        x.raw(&format!("<{p}:valData xsi:type=\"xs:decimal\">{val}</{p}:valData>"));
        x.raw(&format!("<{p}:valInfo xsi:nil=\"true\"/>"));
        x.leaf("toolTip", "");
        x.close("item");
    }
    x.close("realDataItems");
    x.i("splineStrain", r(116))?;
    x.leaf("translucencePercent", "0");
    for (k, tag) in [(117, "funnelNeckHeightPercent"), (118, "funnelNeckWidthPercent"), (119, "funnelGapSumPercent")] {
        let v = dec(atom(r(k))?).and_then(|v| dec(&format!("{v}e2"))).ok_or_else(|| format!("диаграмма: {tag}"))?;
        x.leaf(tag, &v);
    }
    x.line("multiStageLinkLine", r(120))?;
    x.color("multiStageLinkColor", r(121))?;
    values_axis(&mut x, c, base)?;
    x.leaf("pointsAxis", "");
    x.open("valuesScale");
    x.open("titleArea");
    x.raw(&format!("<{p}:font kind=\"AutoFont\"/>"));
    x.leaf("textColor", "auto");
    x.leaf("backColor", "auto");
    x.raw(&format!("<{p}:border width=\"1\">"));
    x.raw("\t<v8ui:style xsi:type=\"v8ui:ControlBorderType\">WithoutBorder</v8ui:style>");
    x.raw(&format!("</{p}:border>"));
    x.leaf("borderColor", "auto");
    x.close("titleArea");
    x.ml("labelFormat", c.get(base + 39))?;
    x.close("valuesScale");
    // Без легенды и заголовка (isShowLegend/isShowTitle false) — None; другие не встречались.
    for (k, tag) in [(13, "legendPlacement"), (0, "plotAreaPlacement"), (12, "titleAreaPlacement")] {
        let v = match (k, c.get(base + k).and_then(Node::atom)) {
            (0, _) => "UseCoordinates",
            (_, Some("0")) => "None",
            _ => return Err(format!("диаграмма: {tag} при показе не сверен")),
        };
        x.leaf(tag, v);
    }
    Ok(x.lines)
}

/// Текст ячейки табличного документа (строка, колонка с 0): None — пусто.
pub type Cells<'a> = dyn Fn(usize, usize) -> R<Option<String>> + 'a;

/// Источник данных диаграммы: единственный в документе `{3,к1,с1,к2,с2,нулевой GUID}` с той же областью, что
/// описание `R<с1+1>C<к1+1>:R<с2+1>C<к2+1>`, и серий/точек столько, сколько строк/колонок за первой → (с1, к1).
fn source(desc: &str, sources: &[Node], ns: usize, np: usize) -> R<(usize, usize)> {
    let bad = || format!("диаграмма: источник данных {desc} не сверен");
    let [s] = sources else { return Err(bad()) };
    let l = s.list();
    let n = |k: usize| l.get(k).and_then(Node::atom).and_then(|a| a.parse::<usize>().ok()).ok_or_else(bad);
    if l.len() != 6 || l[0].atom() != Some("3") || l[5].atom() != Some("00000000-0000-0000-0000-000000000000") {
        return Err(bad());
    }
    let (c1, r1, c2, r2) = (n(1)?, n(2)?, n(3)?, n(4)?);
    if desc != format!("R{}C{}:R{}C{}", r1 + 1, c1 + 1, r2 + 1, c2 + 1) || r2 < r1 || c2 < c1 || r2 - r1 != ns || c2 - c1 != np {
        return Err(bad());
    }
    Ok((r1, c1))
}

/// Чтение записи по порядку: поле, сверка с ожидаемым значением.
struct Seq<'a> {
    c: &'a [Node],
    i: usize,
}

impl<'a> Seq<'a> {
    fn next(&mut self) -> R<&'a Node> {
        let n = self.c.get(self.i).ok_or("диаграмма: короткая запись")?;
        self.i += 1;
        Ok(n)
    }
    fn take(&mut self, k: usize) -> R<&'a [Node]> {
        let s = self.c.get(self.i..self.i + k).ok_or("диаграмма: короткая запись")?;
        self.i += k;
        Ok(s)
    }
    /// Поле, которое XML не выражает (или в образцах не менялось): должно совпасть.
    fn fixed(&mut self, v: &str) -> R<()> {
        let i = self.i;
        let n = self.next()?;
        if short(n) != v {
            return Err(format!("диаграмма: поле {i} = {} (ждали {v}, не сверено)", short(n)));
        }
        Ok(())
    }
    /// Число, которого XML не выражает.
    fn number(&mut self) -> R<()> {
        let n = self.next()?;
        let ok = n.atom().is_some_and(|a| !a.is_empty() && a.bytes().all(|b| b.is_ascii_digit() || b"-.e".contains(&b)));
        if ok { Ok(()) } else { Err(format!("диаграмма: ждали число, а не {}", short(n))) }
    }
}

/// Шкала диаграммы (точек, значений, серий) версий 73, 74: `{2,показ заголовка,источник текста,место заголовка,текст,
/// {область заголовка},линии сетки,положение подписей,есть линия[,линия],{3,4,{0}},шрифт,цвет подписей,ориентация,
/// формат подписей,0,{3,4,{0}},0,угол,0,строк подписи,0,в диаграмме}`. Пишется, только если что-то не по умолчанию.
/// Область заголовка — всегда как в образцах (рамка без линии толщины 1).
fn scale(x: &mut X, tag: &str, n: &Node) -> R<()> {
    let l = n.list();
    let bad = || format!("диаграмма: шкала {tag} {}", short(n));
    let line = l.get(8).and_then(Node::atom) == Some("1");
    let o = usize::from(line);
    // Версия шкалы 1 (диаграмма 71, БП 3.0.107) — без последнего поля «в диаграмме».
    let v1 = l.first().and_then(Node::atom) == Some("1");
    if l.len() != 22 + o - usize::from(v1) {
        return Err(bad());
    }
    let area = short(&l[5]);
    let area_ok = ["48312c09-257f-4b29-b280-284dd89efc1e", "00000000-0000-0000-0000-000000000000"]
        .iter()
        .any(|g| area == format!("{{1,4,0.5,0.5,{{7,3,0,1,100}},{{3,4,{{0}}}},{{3,4,{{0}}}},1,{{3,0,{{0}},0,1,0,{g}}},{{3,4,{{0}}}},4,2,0}}"));
    let at = |k: usize| short(&l[k + o]);
    let fixed = (l[0].atom() == Some("2") || v1) && area_ok
        && at(9) == "{3,4,{0}}" && at(14) == "0" && at(15) == "{3,4,{0}}"
        && [16, 18, 20].iter().all(|&k| at(k) == "0");
    if !fixed {
        return Err(bad());
    }
    // поле 1 — показ заголовка (chart.dll 0x1800ca1d0): 0 Auto — умолчание, 1 DontShow, 2 Show
    let show = match l[1].atom() {
        Some("0") => None,
        Some("1") => Some("DontShow"),
        Some("2") => Some("Show"),
        _ => return Err(bad()),
    };
    // поле 2 — источник текста заголовка (0x1800ca620): 0 Auto — умолчание, 1 UseText, 2 AutoText
    let source = match l[2].atom() {
        Some("0") => None,
        Some("1") => Some("UseText"),
        Some("2") => Some("AutoText"),
        _ => return Err(bad()),
    };
    // поле 3 — место заголовка (0x1800c9d80): 0 WithAxis, 1 PlotArea, 2 SpecialArea — умолчание
    let placement = match l[3].atom() {
        Some("0") => Some("WithAxis"),
        Some("1") => Some("PlotArea"),
        Some("2") => None,
        _ => return Err(bad()),
    };
    let title = &l[4];
    let font = &l[10 + o];
    let num = |k: usize| at(k).parse::<i64>().map_err(|_| bad());
    let (angle, rows) = (num(17)?, num(19)?);
    let grid = match l[6].atom() {
        Some("2") => false,
        Some("0") => true,
        _ => return Err(bad()),
    };
    // поле 7 — положение подписей (ChartScaleLabelLocation, chart.dll 0x1800c3570): 0 Auto — умолчание
    let loc = match l[7].atom() {
        Some("0") => None,
        Some("1") => Some("None"),
        Some("2") => Some("Inside"),
        Some("3") => Some("Outside"),
        _ => return Err(bad()),
    };
    let label = color(Some(&l[11 + o]))?;
    // поле 12 — ориентация подписей (ChartLabelsOrientation, chart.dll FUN_1800ab5e0): 2 Auto — умолчание
    let orient = match at(12).as_str() {
        "0" => Some("Horizontal"),
        "1" => Some("Vertical"),
        "2" => None,
        "3" => Some("CustomAngle"),
        _ => return Err(bad()),
    };
    let fmt = &l[13 + o];
    let in_chart = match l.get(21 + o).and_then(Node::atom) {
        None if v1 => false,
        Some("0") => false,
        Some("2") => true,
        _ => return Err(bad()),
    };
    if show.is_none() && source.is_none() && placement.is_none() && short(title) == "{1,0}" && !grid && loc.is_none() && !line && label == "auto"
        && short(font) == "{7,3,0,1,100}" && orient.is_none() && rows == 0 && angle == 0 && short(fmt) == "{1,0}" && !in_chart
    {
        return Ok(());
    }
    let p = x.p;
    x.open(tag);
    // shortcut: порядок новых элементов — по образцам ERP (МобильноеПриложениеНачальнаяСтраница) и БП (СравнениеПродаж);
    // взаимное место titlePlacement/gridLinesShowMode/scaleLabelLocation и labelColor/labelFont, labelAngle/labelFormat не
    // встречалось — сверить при новом образце
    if let Some(v) = show {
        x.leaf("showTitle", v);
    }
    if let Some(v) = source {
        x.leaf("titleTextSource", v);
    }
    if short(title) != "{1,0}" {
        x.ml("titleText", Some(title))?;
    }
    x.open("titleArea");
    x.raw(&format!("<{p}:font kind=\"AutoFont\"/>"));
    x.leaf("textColor", "auto");
    x.leaf("backColor", "auto");
    x.raw(&format!("<{p}:border width=\"1\">"));
    x.raw("\t<v8ui:style xsi:type=\"v8ui:ControlBorderType\">WithoutBorder</v8ui:style>");
    x.raw(&format!("</{p}:border>"));
    x.leaf("borderColor", "auto");
    x.close("titleArea");
    if let Some(v) = placement {
        x.leaf("titlePlacement", v);
    }
    if grid {
        x.leaf("gridLinesShowMode", "Show");
    }
    if let Some(v) = loc {
        x.leaf("scaleLabelLocation", v);
    }
    if line {
        x.line("gridLine", l.get(9))?;
    }
    if label != "auto" {
        x.leaf("labelColor", &label);
    }
    if short(font) != "{7,3,0,1,100}" {
        x.font("labelFont", Some(font))?;
    }
    if let Some(v) = orient {
        x.leaf("labelOrientation", v);
    }
    if rows != 0 {
        x.leaf("maxLabelRows", &rows.to_string());
    }
    if angle != 0 {
        x.leaf("labelAngle", &angle.to_string());
    }
    if short(fmt) != "{1,0}" {
        x.ml("labelFormat", Some(fmt))?;
    }
    if in_chart {
        x.leaf("showInChart", "DontShow");
    }
    x.close(tag);
    Ok(())
}

/// Диаграмма версий 73 и 74 — поля по порядку записи (docs/research/xml-mxl.md, «Диаграмма», проход r9): заголовок
/// (серии, сводная серия, точки), 100 полей свойств, данные (значение, расшифровка, подсказка — на каждую пару
/// серия × точка), затем свойства подписей, флажки инициализации, параметры воронки, время перестроения, список
/// id серий, оси, шкалы, списки оформления точек и серий, расположение областей, области, режимы подсказок, у 74 —
/// еще 6 нулей, у 75 — и 8 цветов. Поля, которых XML не выражает или которые в образцах не менялись, сверяются со встреченными.
fn body7x(c: &[Node], p: &str, ver: u32, sources: &[Node], cell: &Cells) -> R<Vec<String>> {
    let mut x = X { p, lines: Vec::new(), depth: 0 };
    let ns = int(c.get(4))?.max(0) as usize;
    let q = 5 + 11 * ns;
    let np = int(c.get(q + 12))?.max(0) as usize;
    let base = q + 13 + 11 * np;
    let block = |k: usize| c.get(k..k + 11).ok_or_else(|| "диаграмма: короткая запись".to_string());
    // Поле 94 — число элементов `{0}` за ним (у диаграммы внутри диаграммы Ганта 71 — один, БП 3.0.107
    // ПродолжительностьРаботыРегламентныхЗаданий; XML их не выражает); поля дальше сдвинуты на это число.
    let extra = c.get(base + 94).and_then(Node::atom).and_then(|a| a.parse::<usize>().ok()).ok_or("диаграмма: поле 94")?;
    if c.get(base + 95..base + 95 + extra).is_none_or(|l| l.iter().any(|n| short(n) != "{0}")) {
        return Err("диаграмма: элементы поля 94 не сверены".into());
    }
    let r = |k: usize| c.get(base + k + if k > 94 { extra } else { 0 });
    // Источник данных (поле 98 = 1, поле 97 — область `R1C1:R76C6`): платформа выгружает диаграмму без него — значения
    // строками из ячеек (серия — строка, точка — колонка за первыми), id серий и точек — новые (id + число серий/точек).
    let src = match (r(98).and_then(Node::atom), r(97).and_then(Node::str)) {
        (Some("0"), Some("")) if sources.is_empty() => None,
        (Some("1"), Some(desc)) => Some(source(desc, sources, ns, np)?),
        _ => return Err("диаграмма: источник данных не сверен".into()),
    };
    // Хвост после свойств и данных (у диаграммы с источником нет isRandomizedNewValues и данных).
    let mut s = Seq { c, i: extra + if src.is_some() { base + 99 } else { base + 100 + 3 * ns * np } };
    // Хвост (chart.dll запись FUN_1800399a0, чтение FUN_18003b930): T0 — вид палитры (int объекта палитры; у версий < 58
    // единица — Auto), значения ChartColorPalette — регистрация FUN_1800c2840.
    let palette = match int(Some(s.next()?))? {
        1 if ver < 58 => "Auto",
        v => PALETTE.get(v as usize).copied().ok_or_else(|| format!("диаграмма: вид палитры {v}"))?,
    };
    // T1 — ориентация подписей шкалы точек (поле шкалы точек +0x120; r(40) — ее старый флажок)
    let x_labels = match int(Some(s.next()?))? {
        0 => "Horizontal",
        1 => "Vertical",
        2 => "Auto",
        3 => "CustomAngle",
        v => return Err(format!("диаграмма: xLabelsOrientation {v}")),
    };
    let labels = s.take(5)?;
    let init = s.take(3)?;
    let spline = match s.next()?.atom() {
        Some("0") => None,
        Some("1") => Some("SmoothCurve"),
        v => return Err(format!("диаграмма: splineMode {v:?}")),
    };
    s.fixed("0")?;
    let funnel = s.take(6)?;
    s.fixed("2")?;
    s.fixed("255")?;
    // анимация: регистрация FUN_1800cd9b0
    let animation = match s.next()?.atom() {
        Some("0") => "Auto",
        Some("1") => "Use",
        Some("2") => "DontUse",
        v => return Err(format!("диаграмма: animation {v:?}")),
    };
    let rebuild = s.next()?;
    s.fixed("00000000-0000-0000-0000-000000000000")?;
    // Id сводной серии и серий: `{0,id,0}`; затем `{0,0}` на каждую.
    let ids: Vec<String> = std::iter::once(q + 7).chain((0..ns).map(|k| 5 + 11 * k + 7)).map(|k| c.get(k).map(short).unwrap_or_default()).collect();
    // список id — без повторов (ERP: id серии 0, как у сводной); `{0,0}` — на каждую серию
    let mut uniq: Vec<&String> = Vec::new();
    for id in &ids {
        if !uniq.contains(&id) {
            uniq.push(id);
        }
    }
    // Список бывает и пустым (ТестИПилиООО — круговая); в XML его нет.
    if s.c.get(s.i).and_then(Node::atom) == Some("0") {
        s.fixed("0")?;
    } else {
        s.fixed(&uniq.len().to_string())?;
        for id in &uniq {
            s.fixed(&format!("{{0,{id},0}}"))?;
        }
    }
    for _ in &ids {
        s.fixed("{0,0}")?;
    }
    s.fixed("0")?;
    let values_axis = s.next()?;
    s.fixed("{0,0,{0,1,0,1,0},0,0}")?;
    for v in ["0", "0", "2", "-2", "1", "10", "1", "20", "0", "0"] {
        s.fixed(v)?;
    }
    let scales = s.take(3)?;
    for v in ["0", "0", "{3,4,{0}}", "{3,4,{0}}", "0"] {
        s.fixed(v)?;
    }
    let point_colors = s.take(np)?;
    let looks = s.take(ns + 1)?;
    for _ in 0..12 {
        s.number()?;
    }
    let place = s.take(3)?;
    let areas = s.take(12)?;
    // линии и полосы отметок (T75–78) пусты; T79 — описание палитры (FUN_18017fd70: {0,вид,нач.цвет,кон.цвет,макс.цветов,
    // N,цвета×N}), T80 — палитра полос отметок
    for v in ["{0,0}", "{0,0}", "{0,0}", "{0,0}"] {
        s.fixed(v)?;
    }
    let pal = s.next()?;
    for v in ["{0,14,{3,4,{0}},{3,4,{0}},0,0}", "0", "0"] {
        s.fixed(v)?;
    }
    let hints = s.next()?;
    s.fixed("{0,0,0,0}")?;
    s.fixed("0")?;
    // На каждую точку (у диаграммы с источником их нет); у записи из загрузки XML (8.5, `imp/chart.rs`) — на каждое
    // значение, серий × точек.
    let per = s.c[s.i..].iter().take_while(|n| short(n) == "{{1,{1,0},0},0}").count();
    let old = if src.is_some() { 0 } else { np };
    for _ in 0..if per == ns * np && per != old { ns * np } else { old } {
        s.fixed("{{1,{1,0},0},0}")?;
    }
    let empty = s.next()?;
    if !empty.atom().is_some_and(str::is_empty) {
        return Err(format!("диаграмма: пустое поле не сверено: {} (поле {})", short(empty), s.i - 1));
    }
    s.fixed("60")?;
    // У 71 запись на этом кончается.
    if ver > 71 {
        s.fixed(SCALE)?;
        s.fixed("{0,0,{0,1,0,1,0},0,0}")?;
        for _ in 0..if ver >= 74 { 7 } else { 1 } {
            s.fixed("0")?;
        }
    }
    // Версия 75 (8.5, каркас; цвета уже понижены `form_v59`) дописывает 8 цветов: в образцах 4 FieldBackColor и 4 auto,
    // выгрузка та же, что у 74.
    // После загрузки из XML (платформа, `imp/chart.rs`) все 8 — auto: выгрузка БП, загруженной из XML, та же.
    if ver == 75 {
        let auto = s.c.get(s.i..s.i + 4).is_some_and(|f| f.iter().all(|n| short(n) == "{3,4,{0}}"));
        for v in ["{3,3,{-10}}", "{3,3,{-10}}", "{3,3,{-10}}", "{3,3,{-10}}", "{3,4,{0}}", "{3,4,{0}}", "{3,4,{0}}", "{3,4,{0}}"] {
            s.fixed(if auto { "{3,4,{0}}" } else { v })?;
        }
    }
    if s.i != c.len() {
        return Err(format!("диаграмма: длина записи {} (разобрано {})", c.len(), s.i));
    }
    // Оформление: цвет и маркер серии `{цвет,маркер,0,0,0,"",{1,0},{1,0},{1,0},0}` (у 71 — только `{цвет,маркер}`),
    // цвет точки `{цвет}`.
    let look = |k: usize| -> R<(Option<&Node>, Option<&Node>)> {
        let l = looks[k].list();
        if ver == 71 && l.len() == 2 {
            return Ok((l.first(), l.get(1)));
        }
        if l.len() != 10 || short(&Node::List(l[2..].to_vec())) != "{0,0,0,\"\",{1,0},{1,0},{1,0},0}" {
            return Err(format!("диаграмма: оформление серии {}", short(&looks[k])));
        }
        Ok((l.first(), l.get(1)))
    };
    // С источником: id серий (точек) идут подряд до seriesCurId (pointsCurId), имена — из ячеек области; в XML id
    // сдвинуты на число серий (точек).
    let (shift_s, shift_p) = match src {
        None => (0, 0),
        Some((r1, c1)) => {
            for (k, pt, n, cur, name) in (0..ns).map(|k| (k, false, ns, 1, cell(r1 + 1 + k, c1))).chain((0..np).map(|k| (k, true, np, 2, cell(r1, c1 + 1 + k)))) {
                let b = if pt { block(q + 13 + 11 * k)? } else { block(5 + 11 * k)? };
                let (id, text) = if pt { (&b[2], &b[0]) } else { (&b[7], &b[3]) };
                let want = name?.map(|t| format!("{{1,1,{{\"#\",\"{}\"}}}}", t.replace('"', "\"\"")));
                // У 71 номера в записи свои (серии с 2), в XML — те же новые номера.
                if (ver != 71 && int(Some(id))? != int(c.get(cur))? - n as i64 + 1 + k as i64) || want.as_deref() != Some(short(text).as_str()) {
                    return Err(format!("диаграмма: источник данных, {} {k} не сверена", if pt { "точка" } else { "серия" }));
                }
            }
            (ns as i64, np as i64)
        }
    };
    x.leaf("seriesCurId", &(int(c.get(1))? + shift_s).to_string());
    x.leaf("pointsCurId", &(int(c.get(2))? + shift_p).to_string());
    x.b("isSeriesDesign", c.get(3))?;
    x.leaf("realSeriesCount", &ns.to_string());
    // С источником id в XML — новые: прежний cur + 1 + k (у 73–75 это id + число серий, у 71 номера записи свои).
    let renum = |b: &[Node], pt: bool, k: usize| -> R<i64> {
        let (cur, shift) = if pt { (2, shift_p) } else { (1, shift_s) };
        Ok(if shift == 0 { 0 } else { int(c.get(cur))? + 1 + k as i64 - int(b.get(if pt { 2 } else { 7 }))? })
    };
    for k in 0..ns {
        let b = block(5 + 11 * k)?;
        series(&mut x, "realSeriesData", b, false, look(k)?, renum(b, false, k)?)?;
    }
    series(&mut x, "realExSeriesData", block(q)?, false, look(ns)?, 0)?;
    x.b("isPointsDesign", c.get(q + 11))?;
    x.leaf("realPointCount", &np.to_string());
    for k in 0..np {
        let pc = point_colors[k].list();
        if pc.len() != 1 {
            return Err(format!("диаграмма: цвет точки {}", short(&point_colors[k])));
        }
        let b = block(q + 13 + 11 * k)?;
        series(&mut x, "realPointData", b, true, (pc.first(), None), renum(b, true, k)?)?;
    }
    for (k, v) in [(9, "0"), (10, "0"), (64, "0")] {
        if r(k).map(short).as_deref() != Some(v) {
            return Err(format!("диаграмма: поле {k} = {} (не сверено)", r(k).map(short).unwrap_or_default()));
        }
    }
    for k in 82..94 {
        r(k).and_then(Node::atom).and_then(dec).ok_or("диаграмма: области")?;
    }
    x.i("curSeries", r(0))?;
    x.i("curPoint", r(1))?;
    x.en("chartType", r(2), CHART_TYPE)?;
    x.en("circleLabelType", r(3), LABEL_TYPE)?;
    let delim = r(4).and_then(Node::str).ok_or("диаграмма: labelsDelimiter")?;
    x.leaf("labelsDelimiter", &esc(delim));
    x.en("labelsLocation", r(5), &[(0, "Edge"), (4, "Auto")])?;
    x.ml("lbFormat", r(6))?;
    x.ml("lbpFormat", r(7))?;
    x.color("labelsColor", r(8))?;
    x.font("labelsFont", labels.first())?;
    x.b("transparentLabelsBkg", labels.get(1))?;
    x.color("labelsBkgColor", labels.get(2))?;
    x.border("labelsBorder", labels.get(3))?;
    x.color("labelsBorderColor", labels.get(4))?;
    x.leaf("circleExpandMode", "None");
    x.leaf("chart3Dcrd", "SouthWest");
    x.ml("title", r(11))?;
    x.b("isShowTitle", r(12))?;
    x.b("isShowLegend", r(13))?;
    for (k, tag) in [(14, "ttl"), (16, "lg"), (18, "ch")] {
        x.border(&format!("{tag}Border"), r(k))?;
        x.color(&format!("{tag}BorderColor"), r(k + 1))?;
    }
    x.b("transparent", r(20))?;
    x.color("bkgColor", r(21))?;
    for (k, tag) in [(22, "Ttl"), (24, "Leg"), (26, "Ch")] {
        x.b(&format!("isTrnsp{tag}"), r(k))?;
        x.color(&format!("{}Color", tag.to_lowercase()), r(k + 1))?;
    }
    for (k, tag) in [(28, "ttl"), (29, "leg"), (30, "ch")] {
        x.color(&format!("{tag}TxtColor"), r(k))?;
    }
    for (k, tag) in [(31, "ttl"), (32, "leg"), (33, "ch")] {
        x.font(&format!("{tag}Font"), r(k))?;
    }
    for (k, tag) in [(34, "isShowScale"), (35, "isShowScaleVL"), (36, "isShowSeriesScale"), (37, "isShowPointsScale"), (38, "isShowValuesScale")] {
        x.b(tag, r(k))?;
    }
    x.ml("vsFormat", r(39))?;
    x.leaf("xLabelsOrientation", x_labels);
    x.line("scaleLine", r(41))?;
    x.color("scaleColor", r(42))?;
    x.b("isAutoSeriesName", r(43))?;
    x.b("isAutoPointName", r(44))?;
    x.en("maxMode", r(45), &[(0, "NotDefined")])?;
    x.i("maxSeries", r(46))?;
    x.i("maxSeriesPrc", r(47))?;
    x.en("spaceMode", r(48), &[(1, "Half")])?;
    x.d("baseVal", r(49))?;
    x.b("isOutline", r(50))?;
    x.i("realPiePoint", r(51))?;
    x.i("realStockSeries", r(52))?;
    for (k, tag) in [(53, "isLight"), (54, "isGradient"), (55, "isTransposition"), (56, "hideBaseVal"), (57, "dataTable"), (58, "dtVerLines"), (59, "dtHorLines")] {
        x.b(tag, r(k))?;
    }
    x.en("dtHAlign", r(60), &[(2, "Right")])?;
    x.ml("dtFormat", r(61))?;
    x.b("dtKeys", r(62))?;
    x.leaf("paletteKind", palette);
    x.leaf("animation", animation);
    x.i("rebuildTime", Some(rebuild))?;
    // r(63), r(64) — isTransposed и autoTransposition (поля 0x118a, 0x1189)
    x.b("isTransposed", r(63))?;
    x.b("autoTransposition", r(64))?;
    x.b("legendScrollEnable", r(65))?;
    x.color("surfaceColor", r(66))?;
    x.en("radarScaleType", r(67), &[(0, "Circle")])?;
    x.en("gaugeValuesPresentation", r(68), &[(0, "Needle")])?;
    bands(&mut x, r(69))?;
    x.i("beginGaugeAngle", r(70))?;
    x.i("endGaugeAngle", r(71))?;
    x.i("gaugeThickness", r(72))?;
    x.en("gaugeLabelsLocation", r(73), &[(1, "InsideScale")])?;
    x.b("gaugeLabelsArcDirection", r(74))?;
    x.i("gaugeBushThickness", r(75))?;
    x.color("gaugeBushColor", r(76))?;
    x.b("autoMaxValue", r(77))?;
    x.d("userMaxValue", r(78))?;
    x.b("autoMinValue", r(79))?;
    x.d("userMinValue", r(80))?;
    x.b("elementsIsInit", r(81))?;
    for (k, tag) in ["titleIsInit", "legendIsInit", "chartIsInit"].into_iter().enumerate() {
        x.b(tag, init.get(k))?;
    }
    // Области: в записи left, top, right, bottom; в XML left, right, top, bottom.
    for (k, tag) in [(0, "elementsChart"), (4, "elementsLegend"), (8, "elementsTitle")] {
        x.open(tag);
        for (j, side) in [(0, "left"), (2, "right"), (1, "top"), (3, "bottom")] {
            x.d(side, areas.get(k + j))?;
        }
        x.close(tag);
    }
    x.color("borderColor", r(95))?;
    x.border("border", r(96))?;
    x.leaf("dataSourceDescription", "");
    x.leaf("isDataSourceMode", "false");
    if let Some((r1, c1)) = src {
        x.leaf("isRandomizedNewValues", "false");
        x.open("realDataItems");
        for k in 0..ns * np {
            x.open("item");
            match cell(r1 + 1 + k / np, c1 + 1 + k % np)?.filter(|t| !t.is_empty()) {
                Some(t) => x.raw(&format!("<{p}:valData xsi:type=\"xs:string\">{}</{p}:valData>", esc(&t))),
                None => x.raw(&format!("<{p}:valData xsi:type=\"xs:string\"/>")),
            }
            x.raw(&format!("<{p}:valInfo xsi:nil=\"true\"/>"));
            x.leaf("toolTip", "");
            x.close("item");
        }
        x.close("realDataItems");
    } else {
        x.b("isRandomizedNewValues", r(99))?;
    }
    if src.is_none() && ns * np > 0 {
        x.open("realDataItems");
        for k in 0..ns * np {
            let v = c.get(base + 100 + 3 * k).map(Node::list).unwrap_or(&[]);
            let bad = || format!("диаграмма: значение {}", c.get(base + 100 + 3 * k).map(short).unwrap_or_default());
            if c.get(base + 101 + 3 * k).map(short).as_deref() != Some("{\"U\"}") || c.get(base + 102 + 3 * k).and_then(Node::str) != Some("") {
                return Err("диаграмма: расшифровка или подсказка значения не сверены".into());
            }
            x.open("item");
            match (v.first().and_then(Node::str), v.get(1)) {
                (Some("N"), Some(Node::Atom(val))) => {
                    let val = dec(val).ok_or("диаграмма: значение")?;
                    x.raw(&format!("<{p}:valData xsi:type=\"xs:decimal\">{val}</{p}:valData>"));
                }
                // строка: диаграмма с источником данных, загруженная из XML (8.5) — значения ячеек в самой записи
                (Some("S"), Some(Node::Str(t))) if t.is_empty() => x.raw(&format!("<{p}:valData xsi:type=\"xs:string\"/>")),
                (Some("S"), Some(Node::Str(t))) => x.raw(&format!("<{p}:valData xsi:type=\"xs:string\">{}</{p}:valData>", esc(t))),
                _ => return Err(bad()),
            }
            x.raw(&format!("<{p}:valInfo xsi:nil=\"true\"/>"));
            x.leaf("toolTip", "");
            x.close("item");
        }
        x.close("realDataItems");
    }
    if let Some(m) = spline {
        x.leaf("splineMode", m);
    }
    x.i("splineStrain", funnel.first())?;
    x.leaf("translucencePercent", "0");
    for (k, tag) in [(1, "funnelNeckHeightPercent"), (2, "funnelNeckWidthPercent"), (3, "funnelGapSumPercent")] {
        let v = dec(atom(funnel.get(k))?).and_then(|v| dec(&format!("{v}e2"))).ok_or_else(|| format!("диаграмма: {tag}"))?;
        x.leaf(tag, &v);
    }
    x.line("multiStageLinkLine", funnel.get(4))?;
    x.color("multiStageLinkColor", funnel.get(5))?;
    axis(&mut x, Some(values_axis))?;
    x.leaf("pointsAxis", "");
    for (k, tag) in [(0, "pointsScale"), (1, "valuesScale"), (2, "seriesScale")] {
        scale(&mut x, tag, &scales[k])?;
    }
    // Расположение легенды, области построения, заголовка: 0 (Auto) — не пишется.
    // ChartLegendPlacement — chart.dll 0x1800c3940, ChartTitleAreaPlacement — 0x1800c3be0
    const LEGEND: [&str; 7] = ["Auto", "Left", "Right", "Top", "Bottom", "UseCoordinates", "None"];
    const TITLE: [&str; 9] = ["Auto", "UseCoordinates", "LeftTop", "Top", "RightTop", "LeftBottom", "Bottom", "RightBottom", "None"];
    let code = |n: &Node, names: &'static [&'static str]| n.atom().and_then(|v| v.parse::<usize>().ok()).and_then(|i| names.get(i)).map(|v| (*v != "Auto").then_some(*v));
    let legend = code(&place[1], &LEGEND).ok_or_else(|| format!("диаграмма: legendPlacement {}", short(&place[1])))?;
    let plot = match place[0].atom() {
        Some("0") => None,
        Some("1") => Some("UseCoordinates"),
        _ => return Err(format!("диаграмма: plotAreaPlacement {}", short(&place[0]))),
    };
    let title = code(&place[2], &TITLE).ok_or_else(|| format!("диаграмма: titleAreaPlacement {}", short(&place[2])))?;
    for (tag, v) in [("legendPlacement", legend), ("plotAreaPlacement", plot), ("titleAreaPlacement", title)] {
        if let Some(v) = v {
            x.leaf(tag, v);
        }
    }
    // Режимы подсказки значений и линий проекции: встречены только все по умолчанию и один набор.
    match short(hints).as_str() {
        "{0,0,0,0,0}" => {}
        "{0,2,0,2,2}" => {
            x.leaf("valuesToolTipShowMode", "ShowOnHover");
            x.leaf("pointsDropLinesShowMode", "DontShow");
            x.leaf("valuesDropLinesShowMode", "DontShow");
        }
        h => return Err(format!("диаграмма: режимы подсказки {h} не сверены")),
    }
    // colorPaletteDescription (FUN_1801dd9d0): colorPalette — если вид не Auto; цвета градиента, их число и своя палитра
    // встречены только по умолчанию — иначе отказ
    match pal.list() {
        [z, k, a, b, m, n] if z.atom() == Some("0") && short(a) == "{3,4,{0}}" && short(b) == "{3,4,{0}}" && m.atom() == Some("0") && n.atom() == Some("0") => {
            let k = int(Some(k))?;
            if k != 14 {
                x.open("colorPaletteDescription");
                x.leaf("colorPalette", PALETTE.get(k as usize).ok_or_else(|| format!("диаграмма: палитра {k}"))?);
                x.close("colorPaletteDescription");
            }
        }
        _ => return Err(format!("диаграмма: описание палитры {} не сверено", short(pal))),
    }
    Ok(x.lines)
}

/// Диаграмма Ганта (рисунок вида 10, e5fdc112-…): `{{версия,{0,{11},{диаграмма}},точки,серии,0,0,drawEmpty,шкала
/// времени,keepScaleVariant,fixedVariantMeasure,fixedVariantInterval,autoFullInterval,начало и конец полного интервала,
/// visualBegin,intervalDrawType,noneVariantChars,noneVariantMeasure,0,0,0,extTitle,outboundColor,{фоновые интервалы},0,
/// linksColor,linksLine[,{0,0,0},1,0,1]}}` — версии 17 и 18 (2 макета БП). Точки и серии — пустые элементы (ключи 0)
/// с baseData; поля, которые в образцах не менялись, сверяются. Строки элементов `<object xsi:type="p:GanttChart">`.
pub fn gantt(o: &[Node], p: &str) -> R<Vec<String>> {
    let g = match o {
        [g] => g.list(),
        _ => return Err("диаграмма Ганта: запись".into()),
    };
    let v18 = match (g.first().and_then(Node::atom), g.len()) {
        (Some("17"), 27) => false,
        (Some("18"), 31) | (Some("19"), 33) => true,
        _ => return Err(format!("диаграмма Ганта: версия {} ({} полей) не сверена", g.first().map(short).unwrap_or_default(), g.len())),
    };
    let c = g[1].list();
    if c.len() != 3 || c[0].atom() != Some("0") || short(&c[1]) != "{11}" {
        return Err("диаграмма Ганта: диаграмма".into());
    }
    let mut x = X { p, lines: Vec::new(), depth: 0 };
    x.open("chart");
    let depth = x.depth;
    for l in body(c[2].list(), p)? {
        x.lines.push(format!("{}{l}", "\t".repeat(depth)));
    }
    x.close("chart");
    // Точки и серии: `{1|0,{3,0,1,0,{вид,{8,0,0,0,0,0,{"U"},{1,0},{"U"},0,baseData}[,картинка,шрифт]},{оформление},1,0}}`.
    // `_` — цвет штриховки серий; хвост `,A,R}}` — autoText и useValuesReverseBehavior
    let base_data = |n: &Node, head: &str, mid: &str| -> R<(String, String, &'static str, &'static str)> {
        let s = short(n);
        let bad = || format!("диаграмма Ганта: {}", s);
        let r = s.strip_prefix(head).ok_or_else(bad)?;
        let k = r.find(|c: char| !c.is_ascii_digit()).filter(|&k| k > 0).ok_or_else(bad)?;
        let (b, r) = r.split_at(k);
        let (pre, post) = mid.split_once('_').unwrap_or((mid, ""));
        let r = r.strip_prefix(pre).ok_or_else(bad)?;
        let (hatch, r) = if mid.contains('_') {
            ["{3,0,{0}}", "{3,4,{0}}"].into_iter().find_map(|h| r.strip_prefix(h).map(|r| (h, r))).ok_or_else(bad)?
        } else {
            ("", r)
        };
        let r = r.strip_prefix(post).ok_or_else(bad)?;
        let flags = |v: &str| match v {
            "1" => Some("true"),
            "0" => Some("false"),
            _ => None,
        };
        let (at, rv) = r.strip_suffix("}}").and_then(|r| r.split_once(',')).and_then(|(a, b)| Some((flags(a)?, flags(b)?))).ok_or_else(bad)?;
        Ok((b.to_string(), if hatch == "{3,4,{0}}" { "auto".into() } else { "#000000".into() }, at, rv))
    };
    const KEY: &str = "{8,0,0,0,0,0,{\"U\"},{1,0},{\"U\"},0,";
    let pb = base_data(&g[2], &format!("{{1,{{3,0,1,0,{{2,{KEY}"), "},{4,0,{0},\"\",-1,-1,1,0,\"\"},{7,3,0,1,100}},{0,1,{0,{0,{3,0,{0}},{3,0,{0}}},{3,4,{0}},{3,4,{0}}}},")?;
    let sb = base_data(&g[3], &format!("{{0,{{3,0,1,0,{{3,{KEY}"), "}},{0,1,{0,{0,{3,0,{0}},{3,0,{0}}},_}},")?;
    for (tag, (b, hatch, auto_text, reverse), points) in [("points", pb, true), ("series", sb, false)] {
        x.open(tag);
        x.leaf("testMode", "false");
        x.open("value");
        for k in ["itemKey", "key", "parentKey", "leftKey", "rightKey", "extKey"] {
            x.leaf(k, "0");
        }
        x.leaf("title", "");
        x.leaf("cacheKey", "0");
        x.leaf("baseData", &b);
        if points {
            x.raw(&format!("<{p}:font kind=\"AutoFont\"/>"));
            x.leaf("picture", "");
        }
        x.close("value");
        x.open("contentCacheItem");
        x.leaf("mainColor", "#000000");
        x.leaf("secondColor", "#000000");
        if points {
            x.leaf("backColor", "auto");
            x.leaf("textColor", "auto");
        } else {
            x.leaf("hatchBetweenIntervalsColor", &hatch);
        }
        x.close("contentCacheItem");
        x.leaf("autoText", auto_text);
        x.leaf("useValuesReverseBehavior", reverse);
        x.close(tag);
    }
    let fixed = |k: usize, v: &str| -> R<()> {
        if short(&g[k]) != v {
            return Err(format!("диаграмма Ганта: поле {k} = {} не сверено", short(&g[k])));
        }
        Ok(())
    };
    fixed(4, "0")?;
    fixed(5, "0")?;
    x.b("drawEmpty", g.get(6))?;
    time_scale(&mut x, g.get(7), false)?;
    x.en("keepScaleVariant", g.get(8), &[(2, "AllData")])?;
    x.en("fixedVariantMeasure", g.get(9), TIME_UNIT)?;
    x.i("fixedVariantInterval", g.get(10))?;
    x.b("autoFullInterval", g.get(11))?;
    x.leaf("fullIntervalBegin", &date(g.get(12))?);
    x.leaf("fullIntervalEnd", &date(g.get(13))?);
    x.leaf("visualBegin", &date(g.get(14))?);
    x.en("intervalDrawType", g.get(15), &[(0, "Flat"), (3, "Gradient")])?;
    x.i("noneVariantChars", g.get(16))?;
    x.en("noneVariantMeasure", g.get(17), TIME_UNIT)?;
    x.en("verticalStretch", g.get(18), &[(0, "None")])?;
    x.b("verticalScrollEnable", g.get(19))?;
    x.en("showValueText", g.get(20), GANTT_VALUE_TEXT)?;
    x.ml("extTitle", g.get(21))?;
    x.color("outboundColor", g.get(22))?;
    fixed(23, "{3,{0,{1,0,0},0},{0,0}}")?;
    x.open("backIntervals");
    x.open("collection");
    x.leaf("ticks", "0");
    x.close("collection");
    x.leaf("ticks", "0");
    x.close("backIntervals");
    fixed(24, "0")?;
    x.color("linksColor", g.get(25))?;
    x.line("linksLine", g.get(26))?;
    // Версия 18 дописывает `{0,0,0},<28>,0,<30>` — в образце showPointsText Show; у 17 все четыре свойства Auto.
    // Версия 19 — еще `0,0` (Клиент ЭДО, ДлительностьОтложенногоОбновления: XML как у 18).
    // Поле 28 — showPointsText: 1 Show, 0 Auto (8.5, АнализЖурналаРегистрации — выгрузка платформы).
    // Поле 31 (версия 19) — textPlacement (GanttChartTextPlacementType, chart.dll FUN_1800de410).
    // Поле 30 — булево: чтение FUN_18021ce90 его отбрасывает, запись FUN_18021c750 пишет true — в XML его нет (ЗУП: 0).
    let tail = if g.len() == 33 { "{{0,0,0},_,0,_,_,0}" } else { "{{0,0,0},_,0,_}" };
    let dropped = match g.get(30).and_then(Node::atom) {
        Some(b @ ("0" | "1")) => b,
        _ => "",
    };
    let (show_points, text) = if v18 {
        let (show, k) = match g[28].atom() {
            Some("1") => ("Show", "1"),
            Some("0") => ("Auto", "0"),
            _ => ("", ""),
        };
        let (text, t) = match g.get(31).and_then(Node::atom) {
            Some("1") => ("Cut", "1"),
            Some("2") => ("Wrap", "2"),
            _ => ("Auto", "0"),
        };
        if short(&Node::List(g[27..].to_vec())) != tail.replacen('_', k, 1).replacen('_', dropped, 1).replacen('_', t, 1) {
            return Err(format!("диаграмма Ганта: хвост {} не сверен", short(&Node::List(g[27..].to_vec()))));
        }
        (show, text)
    } else {
        ("Auto", "Auto")
    };
    x.leaf("showPointsText", show_points);
    x.leaf("showData", "Auto");
    x.leaf("textPlacement", text);
    x.leaf("intervalTextRepresentation", "Auto");
    Ok(x.lines)
}

/// Полосы шкалы `{1,N,{2,begin,end,цвет,{текст},{подсказка},"",0,"",0}×N,0,0}`.
fn bands(x: &mut X, n: Option<&Node>) -> R<()> {
    let l = n.map(Node::list).unwrap_or(&[]);
    let bad = || format!("диаграмма: полосы шкалы {}", n.map(short).unwrap_or_default());
    let cnt = int(l.get(1)).map_err(|_| bad())? as usize;
    if l.first().and_then(Node::atom) != Some("1") || l.len() != cnt + 4 || short(&l[cnt + 2]) != "0" || short(&l[cnt + 3]) != "0" {
        return Err(bad());
    }
    let p = x.p;
    if cnt == 0 {
        x.raw(&format!("<{p}:gaugeQualityBands useTextStr=\"false\" useTooltipStr=\"false\"/>"));
        return Ok(());
    }
    x.raw(&format!("<{p}:gaugeQualityBands useTextStr=\"false\" useTooltipStr=\"false\">"));
    x.depth += 1;
    for it in &l[2..cnt + 2] {
        let f = it.list();
        // Версия 3 (у диаграммы 73) повторяет начало и конец в полях 10 и 11.
        let v3 = f.len() == 12 && f[0].atom() == Some("3") && short(&f[10]) == short(&f[1]) && short(&f[11]) == short(&f[2]);
        if !(v3 || f.len() == 10 && f[0].atom() == Some("2")) || short(&f[4]) != "{1,0}" || short(&f[5]) != "{1,0}" || short(&f[6]) != "\"\"" || short(&f[7]) != "0" || short(&f[8]) != "\"\"" || short(&f[9]) != "0" {
            return Err(bad());
        }
        let num = |k: usize| f[k].atom().and_then(dec).ok_or_else(bad);
        x.raw("<v8ui:item>");
        x.raw(&format!("\t<v8ui:begin>{}</v8ui:begin>", num(1)?));
        x.raw(&format!("\t<v8ui:end>{}</v8ui:end>", num(2)?));
        x.raw(&format!("\t<v8ui:backColor>{}</v8ui:backColor>", color(Some(&f[3]))?));
        x.raw("\t<v8ui:text/>");
        x.raw("\t<v8ui:tooltip/>");
        x.raw("</v8ui:item>");
    }
    x.depth -= 1;
    x.raw(&format!("</{p}:gaugeQualityBands>"));
    Ok(())
}

/// Шкала (узел версии 73), с которой сверяются шкалы: формат подписей пустой.
const SCALE: &str = "{2,0,0,2,{1,0},{1,4,0.5,0.5,{7,3,0,1,100},{3,4,{0}},{3,4,{0}},1,{3,0,{0},0,1,0,48312c09-257f-4b29-b280-284dd89efc1e},{3,4,{0}},4,2,0},2,0,0,{3,4,{0}},{7,3,0,1,100},{3,4,{0}},2,{1,0},0,{3,4,{0}},0,0,0,0,0,0}";

/// Ось значений версии 73 `{0,baseValue,{0,1,min,1,max},maxMethod,minMethod}`: 2 — UseValueWithLimitations, 0 — не пишется.
fn axis(x: &mut X, n: Option<&Node>) -> R<()> {
    let l = n.map(Node::list).unwrap_or(&[]);
    let bad = || format!("диаграмма: ось значений {}", n.map(short).unwrap_or_default());
    let lim = l.get(2).map(Node::list).unwrap_or(&[]);
    if l.len() != 5 || l[0].atom() != Some("0") || lim.len() != 5 || lim[0].atom() != Some("0") || lim[1].atom() != Some("1") || lim[3].atom() != Some("1") {
        return Err(bad());
    }
    let num = |n: &Node| n.atom().and_then(dec).ok_or_else(bad);
    let (bv, min, max) = (num(&l[1])?, num(&lim[2])?, num(&lim[4])?);
    let method = |n: &Node| match n.atom() {
        Some("0") => Ok(false),
        Some("2") => Ok(true),
        _ => Err(bad()),
    };
    let (max_m, min_m) = (method(&l[3])?, method(&l[4])?);
    let p = x.p;
    if bv == "0" && min == "0" && max == "0" && !max_m && !min_m {
        x.leaf("valuesAxis", "");
        return Ok(());
    }
    x.open("valuesAxis");
    if bv != "0" {
        x.leaf("baseValue", &bv);
    }
    if min != "0" {
        x.raw(&format!("<{p}:minValue xsi:type=\"xs:decimal\">{min}</{p}:minValue>"));
    }
    if max != "0" {
        x.raw(&format!("<{p}:maxValue xsi:type=\"xs:decimal\">{max}</{p}:maxValue>"));
    }
    if min_m {
        x.leaf("minValueDetectionMethod", "UseValueWithLimitations");
    }
    if max_m {
        x.leaf("maxValueDetectionMethod", "UseValueWithLimitations");
    }
    x.close("valuesAxis");
    Ok(())
}

/// Ось значений версии 54 — из базового значения и пределов (baseVal 49, autoMax 77, max 78, autoMin 79, min 80).
fn values_axis(x: &mut X, c: &[Node], base: usize) -> R<()> {
    let a = |k: usize| atom(c.get(base + k));
    let (bv, auto_max, max, auto_min, min) = (dec(a(49)?), a(77)?, dec(a(78)?), a(79)?, dec(a(80)?));
    let (Some(bv), Some(max), Some(min)) = (bv, max, min) else { return Err("диаграмма: ось значений".into()) };
    // Встречены: автомаксимум выключен, максимум не 0; минимум 0 при автоминимуме.
    if auto_max != "0" || max == "0" || (auto_min == "1" && min != "0") || !matches!(auto_min, "0" | "1") {
        return Err("диаграмма: ось значений не сверена".into());
    }
    let p = x.p;
    x.open("valuesAxis");
    if bv != "0" {
        x.leaf("baseValue", &bv);
    }
    if min != "0" {
        x.raw(&format!("<{p}:minValue xsi:type=\"xs:decimal\">{min}</{p}:minValue>"));
    }
    x.raw(&format!("<{p}:maxValue xsi:type=\"xs:decimal\">{max}</{p}:maxValue>"));
    if auto_min == "0" {
        x.leaf("minValueDetectionMethod", "UseValueWithLimitations");
    }
    x.leaf("maxValueDetectionMethod", "UseValueWithLimitations");
    x.close("valuesAxis");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::dec;

    #[test]
    fn decimals() {
        for (a, b) in [("5e2", "500"), ("1.6e2", "160"), ("-3e2", "-300"), ("0", "0"), ("95", "95"), ("5.48", "5.48"), ("8.318042813455651e-1", "0.831804281345565"), ("1.266666666666667e-1", "0.126666666666667"), ("8.733333333333331e-1", "0.873333333333333"), ("1e-1", "0.1"), ("3e-2", "0.03"), ("-99999", "-99999"), ("0.14968152866242038", "0.14968152866242038")] {
            assert_eq!(dec(a).as_deref(), Some(b), "{a}");
        }
    }
}
