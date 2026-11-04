//! Схема компоновки данных `Ext/Template.xml` (`<DataCompositionSchema>`) → запись макета (обратное `xml/dcs.rs`).
//!
//! Запись: `u32 0`, `u32 n` (число вариантов настроек), n+1 длин `u64`, части: схема (`<SchemaFile>` без макетов и
//! настроек вариантов), настройки каждого варианта (`<Settings>`), хвост (`<SchemaFile>` с макетами и общим списком
//! оформлений ячеек, на который ссылаются `appIndex`). Части пишет сериализатор XDTO платформы: пространство элемента
//! не по умолчанию — `xmlns="…"` на нем (или префикс, если он уже объявлен), QName значений — префикс из области
//! видимости или объявленный тут же: у пространств компоновки — свои имена (`dcscor`, `dcsset`, `dcsat`, `mxl`),
//! у прочих — `d<глубина>p<номер>`.

use super::index::Index;
use super::Rec;
use crate::xml::common::PLATFORM_TYPES;

const XSI: &str = "http://www.w3.org/2001/XMLSchema-instance";
const XS: &str = "http://www.w3.org/2001/XMLSchema";
const SCH: &str = "http://v8.1c.ru/8.1/data-composition-system/schema";
const SET: &str = "http://v8.1c.ru/8.1/data-composition-system/settings";
const AT: &str = "http://v8.1c.ru/8.1/data-composition-system/area-template";
const V8: &str = "http://v8.1c.ru/8.1/data/core";
const V8UI: &str = "http://v8.1c.ru/8.1/data/ui";
const STYLE: &str = "http://v8.1c.ru/8.1/data/ui/style";
const READABLE: &str = "http://v8.1c.ru/8.3/xcf/readable";
const CFG: &str = "http://v8.1c.ru/8.1/data/enterprise/current-config";
const BOM: &str = "\u{feff}";

/// Корень части настроек `<Settings>` в записи.
const SETTINGS_ROOT: &[(&str, &str)] = &[
    ("", SET),
    ("dcscor", "http://v8.1c.ru/8.1/data-composition-system/core"),
    ("pal", "http://v8.1c.ru/8.1/data/ui/colors/palette"),
    ("style", "http://v8.1c.ru/8.1/data/ui/style"),
    ("sys", "http://v8.1c.ru/8.1/data/ui/fonts/system"),
    ("v8", V8),
    ("v8ui", "http://v8.1c.ru/8.1/data/ui"),
    ("web", "http://v8.1c.ru/8.1/data/ui/colors/web"),
    ("win", "http://v8.1c.ru/8.1/data/ui/colors/windows"),
    ("xs", XS),
    ("xsi", XSI),
];
/// Свои имена префиксов сериализатора для пространств компоновки (схема — dcs.dll SCOM_Main, 0x1804e0a07: пакет
/// регистрирует пространство схемы с префиксом `dcssch`).
const KNOWN: &[(&str, &str)] = &[
    ("http://v8.1c.ru/8.1/data-composition-system/core", "dcscor"),
    (SET, "dcsset"),
    (AT, "dcsat"),
    (SCH, "dcssch"),
    ("http://v8.1c.ru/8.2/data/spreadsheet", "mxl"),
];
/// Еще свои имена — у значений атрибутов (`ref` шрифта).
const KNOWN_ATTR: &[(&str, &str)] = &[("http://v8.1c.ru/8.1/data/ui/fonts/system", "sys"), (STYLE, "style")];
/// Элементы схемы, которые хранятся в хвосте.
const TAIL: &[&str] = &["template", "groupTemplate", "groupHeaderTemplate", "totalFieldsTemplate", "fieldTemplate"];

type R<T> = Result<T, String>;

#[derive(Clone, Debug, PartialEq)]
struct Q {
    ns: String,
    local: String,
}

#[derive(Clone, Debug)]
enum Val {
    Raw(String),
    Q(Q),
}

#[derive(Clone, Debug)]
struct Attr {
    ns: String,
    local: String,
    val: Val,
}

#[derive(Clone, Debug, Default)]
struct El {
    ns: String,
    name: String,
    attrs: Vec<Attr>,
    kids: Vec<El>,
    /// Текст как в файле (с сущностями).
    text: Option<String>,
    /// Текст — QName.
    qtext: Option<Q>,
    line: usize,
    /// Настройки вложенной схемы: объявляют постоянный набор пространств.
    fixed: bool,
}

impl El {
    fn new(ns: &str, name: &str) -> El {
        El { ns: ns.into(), name: name.into(), ..El::default() }
    }
}

type NsMap = Vec<(String, String)>;

fn lookup<'a>(m: &'a NsMap, p: &str) -> Option<&'a str> {
    m.iter().rev().find(|(k, _)| k == p).map(|(_, v)| v.as_str())
}

fn split_qname(t: &str) -> Option<(Option<&str>, &str)> {
    if t.is_empty() || t.chars().any(char::is_whitespace) {
        return None;
    }
    match t.split_once(':') {
        None => Some((None, t)),
        Some((p, l)) => {
            let ok = p.starts_with(|c: char| c.is_ascii_alphabetic() || c == '_') && p.chars().all(|c| c.is_alphanumeric() || "_.-".contains(c));
            (ok && !l.is_empty() && !l.contains(':')).then_some((Some(p), l))
        }
    }
}

/// QName значения: префикс обязан быть объявлен; без префикса — только если `default`.
fn qname(t: &str, m: &NsMap, default: bool) -> Option<Q> {
    let (p, l) = split_qname(t)?;
    match p {
        None => {
            let ns = lookup(m, "").filter(|n| default && !n.is_empty())?;
            Some(Q { ns: ns.into(), local: t.into() })
        }
        Some("xml") => None,
        Some(p) => Some(Q { ns: lookup(m, p)?.into(), local: l.into() }),
    }
}

fn tag_end(s: &str) -> Option<usize> {
    let mut quoted = false;
    for (i, c) in s.char_indices() {
        match c {
            '"' => quoted = !quoted,
            '>' if !quoted => return Some(i),
            _ => {}
        }
    }
    None
}

/// Разбор XML без потери текста; объявления пространств разрешаются и в записи не хранятся.
fn parse(text: &str) -> R<El> {
    let s = text.strip_prefix('\u{feff}').unwrap_or(text);
    let mut stack: Vec<(NsMap, El)> = Vec::new();
    let mut root: Option<El> = None;
    let base: NsMap = vec![("xml".into(), "http://www.w3.org/XML/1998/namespace".into()), (String::new(), String::new())];
    let mut i = 0;
    let mut line = 1;
    while i < s.len() {
        let rest = &s[i..];
        if !rest.starts_with('<') {
            let end = rest.find('<').unwrap_or(rest.len());
            line += rest[..end].matches('\n').count();
            if let Some((_, e)) = stack.last_mut() {
                e.text = Some(rest[..end].to_string());
            } else if !rest[..end].trim().is_empty() {
                return Err("xml: текст вне корня".into());
            }
            i += end;
            continue;
        }
        if rest.starts_with("<?") {
            i += rest.find("?>").ok_or("xml: незакрытая инструкция")? + 2;
            continue;
        }
        if rest.starts_with("<!") {
            return Err(format!("строка {line}: комментарии и DOCTYPE не загружаются"));
        }
        let end = tag_end(rest).ok_or("xml: незакрытый тег")?;
        let tag = &rest[1..end];
        line += tag.matches('\n').count();
        i += end + 1;
        if tag.starts_with('/') {
            let (m, mut el) = stack.pop().ok_or("xml: лишний закрывающий тег")?;
            if !el.kids.is_empty() {
                if el.text.as_deref().is_some_and(|t| !t.chars().all(|c| matches!(c, ' ' | '\t' | '\r' | '\n'))) {
                    return Err(format!("строка {}: <{}>: текст вместе с элементами", el.line, el.name));
                }
                el.text = None;
            } else if let Some(t) = &el.text {
                el.qtext = qname(t, &m, el.ns == V8 && (el.name == "Type" || el.name == "TypeSet"));
            }
            match stack.last_mut() {
                Some((_, p)) => p.kids.push(el),
                None => root = Some(el),
            }
            continue;
        }
        let (tag, empty) = match tag.strip_suffix('/') {
            Some(t) => (t, true),
            None => (tag, false),
        };
        let (qn, mut attrs_src) = tag.split_once(|c: char| c.is_whitespace()).unwrap_or((tag, ""));
        let mut m = stack.last().map(|(m, _)| m.clone()).unwrap_or_else(|| base.clone());
        let mut raw = Vec::new();
        loop {
            attrs_src = attrs_src.trim_start();
            if attrs_src.is_empty() {
                break;
            }
            let eq = attrs_src.find('=').ok_or("xml: атрибут без значения")?;
            let a = attrs_src[..eq].trim();
            let v0 = attrs_src[eq + 1..].trim_start().strip_prefix('"').ok_or("xml: значение атрибута без кавычек")?;
            let ve = v0.find('"').ok_or("xml: незакрытое значение атрибута")?;
            let v = &v0[..ve];
            attrs_src = &v0[ve + 1..];
            if a == "xmlns" {
                m.push((String::new(), v.into()));
            } else if let Some(p) = a.strip_prefix("xmlns:") {
                m.push((p.into(), v.into()));
            } else {
                raw.push((a, v));
            }
        }
        let (p, l) = qn.split_once(':').unwrap_or(("", qn));
        let ns = lookup(&m, p).ok_or_else(|| format!("строка {line}: префикс {p} не объявлен"))?;
        let mut el = El::new(ns, l);
        el.line = line;
        for (a, v) in raw {
            let (ap, al) = a.split_once(':').unwrap_or(("", a));
            let ans = if ap.is_empty() { "" } else { lookup(&m, ap).ok_or_else(|| format!("строка {line}: префикс {ap} не объявлен"))? };
            let val = if ans == XSI && al == "type" {
                Val::Q(qname(v, &m, true).ok_or_else(|| format!("строка {line}: xsi:type {v}"))?)
            } else {
                qname(v, &m, false).map_or_else(|| Val::Raw(v.into()), Val::Q)
            };
            el.attrs.push(Attr { ns: ans.into(), local: al.into(), val });
        }
        if empty {
            match stack.last_mut() {
                Some((_, p)) => p.kids.push(el),
                None => root = Some(el),
            }
        } else {
            if stack.len() >= 256 {
                return Err("xml: слишком глубокая вложенность".into());
            }
            stack.push((m, el));
        }
    }
    if !stack.is_empty() {
        return Err("xml: незакрытые элементы".into());
    }
    root.ok_or_else(|| "xml: нет корня".into())
}

/// Запись в стиле XDTO; `scope` — (префикс, пространство), "" — по умолчанию.
struct Writer {
    out: String,
}

impl Writer {
    fn el(&mut self, e: &El, scope: &NsMap, depth: usize) {
        let mut scope = scope.clone();
        let mut decl: Vec<(String, String)> = Vec::new();
        let name = if lookup(&scope, "") == Some(e.ns.as_str()) {
            e.name.clone()
        } else if let Some((p, _)) = scope.iter().rev().find(|(p, n)| !p.is_empty() && *n == e.ns && lookup(&scope, p) == Some(n.as_str())) {
            format!("{p}:{}", e.name)
        } else {
            decl.push((String::new(), e.ns.clone()));
            scope.push((String::new(), e.ns.clone()));
            e.name.clone()
        };
        if e.fixed {
            for (p, ns) in SETTINGS_ROOT.iter().filter(|(p, _)| !p.is_empty() && *p != "xs" && *p != "xsi") {
                decl.push((p.to_string(), ns.to_string()));
                scope.push((p.to_string(), ns.to_string()));
            }
        }
        let q = |q: &Q, scope: &mut NsMap, decl: &mut Vec<(String, String)>, attr: bool| -> String {
            if lookup(scope, "") == Some(q.ns.as_str()) {
                return q.local.clone();
            }
            if let Some((p, _)) = scope.iter().rev().find(|(p, n)| !p.is_empty() && *n == q.ns && lookup(scope, p) == Some(n.as_str())) {
                return format!("{p}:{}", q.local);
            }
            let k = decl.iter().filter(|(p, _)| !p.is_empty()).count() + 1;
            let known = KNOWN.iter().chain(if attr { KNOWN_ATTR } else { &[] }).find(|(n, _)| *n == q.ns);
            let p = known.map_or_else(|| format!("d{depth}p{k}"), |(_, p)| p.to_string());
            decl.push((p.clone(), q.ns.clone()));
            scope.push((p.clone(), q.ns.clone()));
            format!("{p}:{}", q.local)
        };
        let mut attrs = Vec::new();
        for a in &e.attrs {
            let an = if a.ns.is_empty() {
                a.local.clone()
            } else {
                q(&Q { ns: a.ns.clone(), local: a.local.clone() }, &mut scope, &mut decl, false)
            };
            let v = match &a.val {
                Val::Raw(v) => v.clone(),
                Val::Q(x) => q(x, &mut scope, &mut decl, !(a.ns == XSI && a.local == "type")),
            };
            attrs.push(format!("{an}=\"{v}\""));
        }
        // шрифт — элемент стиля (`ref="0:<uuid>"`, kind="StyleItem"): префикс `style` платформа объявляет на самом элементе,
        // если его нет выше (эталон: настройки динамического списка, БП и 8.5 МашиночитаемыеДоверенности)
        let style_font = e.attrs.iter().any(|a| a.ns.is_empty() && a.local == "kind" && matches!(&a.val, Val::Raw(v) if v == "StyleItem"));
        if style_font && !scope.iter().any(|(p, n)| n == STYLE && lookup(&scope, p) == Some(n.as_str())) {
            decl.push(("style".into(), STYLE.into()));
            scope.push(("style".into(), STYLE.into()));
        }
        let text = match &e.qtext {
            Some(x) => Some(q(x, &mut scope, &mut decl, false)),
            None => e.text.clone(),
        };
        let ind = "\t".repeat(depth - 1);
        self.out += &format!("{ind}<{name}");
        for (p, ns) in &decl {
            if p.is_empty() {
                self.out += &format!(" xmlns=\"{ns}\"");
            } else {
                self.out += &format!(" xmlns:{p}=\"{ns}\"");
            }
        }
        for a in &attrs {
            self.out += " ";
            self.out += a;
        }
        if !e.kids.is_empty() {
            self.out += ">\r\n";
            for k in &e.kids {
                self.el(k, &scope, depth + 1);
            }
            self.out += &format!("{ind}</{name}>\r\n");
        } else if let Some(t) = text {
            self.out += &format!(">{t}</{name}>\r\n");
        } else {
            self.out += "/>\r\n";
        }
    }
}

/// Обратные преобразования выгрузки: имена типов → TypeId.
struct Fix<'a> {
    x: &'a Index,
    apps: Vec<El>,
    keys: Vec<String>,
    /// `Template/value.svg`: встроенная картинка (все встроенные картинки выгрузка пишет в этот один файл).
    svg: Option<&'a [u8]>,
    svg_used: std::cell::Cell<bool>,
}

impl Fix<'_> {
    fn run(&mut self, e: &mut El, in_tail: bool) -> R<()> {
        let mut nk = Vec::with_capacity(e.kids.len());
        let cell = e.ns == AT && e.name == "tableCell";
        let nested = e.name == "nestedSchema" || e.name == "settingsVariant";
        for mut k in std::mem::take(&mut e.kids) {
            // оформление ячейки макета — индексом в общем списке хвоста
            if in_tail && cell && k.ns == AT && k.name == "appearance" {
                let mut a = El::new(AT, "appearance");
                a.attrs.push(Attr { ns: XSI.into(), local: "type".into(), val: Val::Q(Q { ns: AT.into(), local: "TableCellAppearance".into() }) });
                a.kids = std::mem::take(&mut k.kids);
                let svg_before = self.svg_used.get();
                self.svg_used.set(false);
                for c in &mut a.kids {
                    self.run(c, in_tail)?;
                }
                // встроенная картинка равна только себе: такое оформление не сливается с другими
                let own = self.svg_used.get();
                self.svg_used.set(svg_before || own);
                // одинаковые оформления платформа хранит один раз (номер — первого такого)
                let key = if own {
                    format!("\u{1}{}", self.apps.len())
                } else {
                    let mut w = Writer { out: String::new() };
                    let scope: NsMap = vec![(String::new(), String::new()), ("xs".into(), XS.into()), ("xsi".into(), XSI.into())];
                    w.el(&a, &scope, 2);
                    w.out
                };
                let n = match self.keys.iter().position(|k| *k == key) {
                    Some(n) => n,
                    None => {
                        self.keys.push(key);
                        self.apps.push(a);
                        self.apps.len() - 1
                    }
                };
                let mut i = El::new(AT, "appIndex");
                i.text = Some(n.to_string());
                nk.push(i);
                continue;
            }
            k.fixed = nested && k.name == "settings";
            self.run(&mut k, in_tail)?;
            nk.push(k);
        }
        e.kids = nk;
        self.value(e)
    }

    fn value(&self, e: &mut El) -> R<()> {
        // в описании типов платформа хранит сначала типы по имени (как в XML), затем TypeId по возрастанию
        let is_t = |k: &El| k.ns == V8 && (k.name == "Type" || k.name == "TypeId");
        if e.kids.iter().any(|k| k.ns == V8 && k.name == "TypeId") {
            let pos: Vec<usize> = (0..e.kids.len()).filter(|&i| is_t(&e.kids[i])).collect();
            let mut ts: Vec<El> = pos.iter().map(|&i| e.kids[i].clone()).collect();
            ts.sort_by(|a, b| match (a.name == "TypeId", b.name == "TypeId") {
                (true, true) => a.text.cmp(&b.text),
                (x, y) => x.cmp(&y),
            });
            for (i, t) in pos.into_iter().zip(ts) {
                e.kids[i] = t;
            }
        }
        // шрифт — элемент стиля конфигурации: в записи `ref="0:<uuid>"`
        let is_font = e.attrs.iter().any(|a| a.ns == XSI && a.local == "type" && matches!(&a.val, Val::Q(q) if q.ns == V8UI && q.local == "Font"));
        if is_font {
            for a in &mut e.attrs {
                if let (true, Val::Q(q)) = (a.ns.is_empty() && a.local == "ref", &a.val) {
                    if q.ns == STYLE {
                        if let Some(g) = self.x.id(&format!("StyleItem.{}", q.local)) {
                            a.val = Val::Raw(format!("0:{g}"));
                        }
                    }
                }
            }
        }
        // картинка: в выгрузке «читаемая» (`<Ref>CommonPicture.Имя</Ref>`), в записи — `ref="v8ui:Имя"`
        let readable = e.attrs.iter().position(|a| a.ns == XSI && a.local == "type" && matches!(&a.val, Val::Q(q) if q.ns == READABLE && q.local == "Picture"));
        if let Some(ti) = readable {
            e.attrs[ti].val = Val::Q(Q { ns: V8UI.into(), local: "Picture".into() });
            let kids = std::mem::take(&mut e.kids);
            let opaque=kids.iter().any(|e|e.ns==READABLE && e.name=="LoadTransparent" && e.text.as_deref()==Some("false"));
            match kids.as_slice() {
                [] => {}
                [r, lt] if r.ns == READABLE && r.name == "Ref" && lt.ns == READABLE && lt.name == "LoadTransparent" && matches!(lt.text.as_deref(),Some("true"|"false")) => {
                    let t = r.text.clone().unwrap_or_default();
                    // ссылка по uuid (картинки нет в конфигурации, 8.5: ПраваРолей и др.) — в записи как есть
                    let val = if t.strip_prefix("0:").is_some_and(crate::brace::is_guid) {
                        Val::Raw(t)
                    } else {
                        let name = t.strip_prefix("CommonPicture.").or_else(|| t.strip_prefix("StdPicture.")).ok_or_else(|| format!("строка {}: картинка {t}", r.line))?;
                        Val::Q(Q { ns: V8UI.into(), local: name.into() })
                    };
                    e.attrs.push(Attr { ns: String::new(), local: "ref".into(), val });
                }
                [r, lt] if r.ns == READABLE && r.name == "Abs" && r.text.as_deref().is_some_and(|t| t.starts_with("value.")) && lt.ns == READABLE && lt.name == "LoadTransparent" && matches!(lt.text.as_deref(),Some("true"|"false")) => {
                    let svg = self.svg.ok_or_else(|| format!("строка {}: нет Template/value.svg", r.line))?;
                    // в записи — base64 строками по 64 знака через CRLF
                    let b = super::payload::base64_atom(svg);
                    let raw = b.atom().unwrap_or_default().trim_start_matches("#base64:").replace("\r\r\n", "");
                    let lines: Vec<&str> = raw.as_bytes().chunks(64).map(|c| std::str::from_utf8(c).unwrap_or_default()).collect();
                    e.text = Some(lines.join("\r\n"));
                    e.qtext = None;
                    self.svg_used.set(true);
                }
                _ => return Err(format!("строка {}: картинка не загружается", e.line)),
            }
            if opaque {e.attrs.push(Attr{ns:String::new(),local:"t".into(),val:Val::Raw("false".into())});}
        }
        // цвет — элемент стиля конфигурации: в записи `0:<uuid>`
        let is_color = e.attrs.iter().any(|a| a.ns == XSI && a.local == "type" && matches!(&a.val, Val::Q(q) if q.ns == V8UI && q.local == "Color"));
        if is_color {
            if let Some(q) = e.qtext.clone().filter(|q| q.ns == STYLE) {
                if let Some(g) = self.x.id(&format!("StyleItem.{}", q.local)) {
                    e.qtext = None;
                    e.text = Some(format!("0:{g}"));
                }
            }
        }
        if e.ns == V8 && (e.name == "Type" || e.name == "TypeSet") {
            let q = e.qtext.clone().ok_or_else(|| format!("строка {}: тип не QName", e.line))?;
            if q.ns == CFG {
                let id = match self.x.types.get(&q.local) {
                    Some((t, _)) if (e.name == "TypeSet") == q.local.starts_with("Characteristic.") => t.clone(),
                    Some(_) => return Err(format!("строка {}: {} {}", e.line, e.name, q.local)),
                    None => {
                        let el = format!("v8:{}", e.name);
                        let v = format!("cfg:{}", q.local);
                        let v = super::tpl::any_ib_ref(self.x, &v);
                        PLATFORM_TYPES.iter().find(|(_, a, b)| *a == el && *b == v).map(|(g, ..)| g.to_string()).ok_or_else(|| format!("строка {}: незнакомый тип {}", e.line, q.local))?
                    }
                };
                e.name = "TypeId".into();
                e.qtext = None;
                e.text = Some(id);
            } else if e.name == "TypeSet" {
                return Err(format!("строка {}: набор типов {}", e.line, q.local));
            }
        }
        Ok(())
    }
}

/// Template.xml схемы компоновки → запись; `svg` — содержимое `Template/value.svg`, если есть.
pub fn record(src: &str, x: &Index, svg: Option<&[u8]>) -> R<(Rec, bool)> {
    let doc = parse(src)?;
    if doc.ns != SCH || doc.name != "DataCompositionSchema" {
        return Err("корень не <DataCompositionSchema>".into());
    }
    let mut fix = Fix { x, apps: Vec::new(), keys: Vec::new(), svg, svg_used: Default::default() };
    let mut main = Vec::new();
    let mut tail = Vec::new();
    let mut sets = Vec::new();
    for mut k in doc.kids {
        if k.ns != SCH {
            return Err(format!("строка {}: <{}> не из пространства схемы", k.line, k.name));
        }
        if TAIL.contains(&k.name.as_str()) {
            fix.run(&mut k, true)?;
            tail.push(k);
            continue;
        }
        if k.name == "settingsVariant" {
            let pos = k.kids.iter().position(|c| c.ns == SET && c.name == "settings").ok_or_else(|| format!("строка {}: вариант без настроек", k.line))?;
            let mut st = k.kids.remove(pos);
            if pos != k.kids.len() {
                return Err(format!("строка {}: настройки варианта не последние", st.line));
            }
            fix.run(&mut st, false)?;
            sets.push(st);
        }
        fix.run(&mut k, false)?;
        main.push(k);
    }
    let file = |kids: Vec<El>, apps: Vec<El>| -> String {
        let mut w = Writer { out: format!("{BOM}<?xml version=\"1.0\" encoding=\"UTF-8\"?>\r\n<SchemaFile xmlns=\"\" xmlns:xs=\"{XS}\" xmlns:xsi=\"{XSI}\">\r\n") };
        let scope: NsMap = vec![(String::new(), String::new()), ("xs".into(), XS.into()), ("xsi".into(), XSI.into())];
        let mut d = El::new(SCH, "dataCompositionSchema");
        d.kids = kids;
        w.el(&d, &scope, 2);
        for a in &apps {
            w.el(a, &scope, 2);
        }
        w.out += "</SchemaFile>";
        w.out
    };
    let mut parts = vec![file(main, vec![])];
    for st in sets {
        let mut w = Writer { out: format!("{BOM}<?xml version=\"1.0\" encoding=\"UTF-8\"?>\r\n<Settings") };
        for (p, ns) in SETTINGS_ROOT {
            w.out += &if p.is_empty() { format!(" xmlns=\"{ns}\"") } else { format!(" xmlns:{p}=\"{ns}\"") };
        }
        let scope: NsMap = SETTINGS_ROOT.iter().map(|(p, n)| (p.to_string(), n.to_string())).collect();
        if st.kids.is_empty() {
            w.out += "/>";
        } else {
            w.out += ">\r\n";
            for k in &st.kids {
                w.el(k, &scope, 2);
            }
            w.out += "</Settings>";
        }
        parts.push(w.out);
    }
    let apps = std::mem::take(&mut fix.apps);
    parts.push(file(tail, apps));
    let n = parts.len() - 2;
    let mut out = Vec::new();
    out.extend(0u32.to_le_bytes());
    out.extend((n as u32).to_le_bytes());
    for p in &parts[..parts.len() - 1] {
        out.extend((p.len() as u64).to_le_bytes());
    }
    for p in &parts {
        out.extend(p.as_bytes());
    }
    Ok((Rec::Raw(out), fix.svg_used.get()))
}

/// Настройки компоновки `src` (корень — любой элемент, его дети — элементы настроек: условное оформление формы…) →
/// часть `<Settings>` так, как ее пишет сериализатор платформы (как у настроек варианта схемы).
pub fn settings_part(src: &str, x: &Index) -> R<String> {
    part(src, x, "Settings", SETTINGS_ROOT)
}

/// Значение-объект настроек (`src` — как у `settings_part`) так, как его пишет сериализатор XDTO значением в записи
/// (`{"#",<тип>,{base64}}`): корень `root` в пространстве настроек, объявлены только `xs`, `xsi`.
pub fn value_part(src: &str, x: &Index, root: &str) -> R<String> {
    part(src, x, root, &[("", SET), ("xs", XS), ("xsi", XSI)])
}

/// Состояние сервера динамического списка (mngbase FUN_180702cc0): корень `UniversalListServerOnlyState` без
/// пространства, объявлены `xs`, `xsi`; `src` — корень, дети которого — значения коллекций.
pub fn server_state_part(src: &str, x: &Index) -> R<String> {
    part(src, x, "UniversalListServerOnlyState", &[("", ""), ("xs", XS), ("xsi", XSI)])
}

fn part(src: &str, x: &Index, root: &str, decl: &[(&str, &str)]) -> R<String> {
    let mut st = parse(src)?;
    let mut fix = Fix { x, apps: Vec::new(), keys: Vec::new(), svg: None, svg_used: Default::default() };
    fix.run(&mut st, false)?;
    let mut w = Writer { out: format!("{BOM}<?xml version=\"1.0\" encoding=\"UTF-8\"?>\r\n<{root}") };
    for (p, ns) in decl {
        w.out += &if p.is_empty() { format!(" xmlns=\"{ns}\"") } else { format!(" xmlns:{p}=\"{ns}\"") };
    }
    let scope: NsMap = decl.iter().map(|(p, n)| (p.to_string(), n.to_string())).collect();
    if st.kids.is_empty() {
        w.out += "/>";
    } else {
        w.out += ">\r\n";
        for k in &st.kids {
            w.el(k, &scope, 2);
        }
        w.out += &format!("</{root}>");
    }
    Ok(w.out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parts_prefixes_and_shared_appearance() {
        let app = "<dcsat:appearance><dcscor:item><dcscor:parameter>Ш</dcscor:parameter><dcscor:value xsi:type=\"xs:decimal\">1</dcscor:value></dcscor:item></dcsat:appearance>";
        let x = format!(
            "\u{feff}<?xml version=\"1.0\" encoding=\"UTF-8\"?>\r\n<DataCompositionSchema xmlns=\"{SCH}\" xmlns:dcscor=\"http://v8.1c.ru/8.1/data-composition-system/core\" xmlns:dcsset=\"{SET}\" xmlns:v8=\"{V8}\" xmlns:xs=\"{XS}\" xmlns:xsi=\"{XSI}\">\r\n\
             \t<parameter><name>П</name><title xsi:type=\"v8:LocalStringType\"><v8:item><v8:lang>ru</v8:lang><v8:content>п</v8:content></v8:item></title><valueType><v8:Type>xs:string</v8:Type></valueType></parameter>\r\n\
             \t<template><name>М</name><template xmlns:dcsat=\"{AT}\" xsi:type=\"dcsat:AreaTemplate\"><dcsat:item xsi:type=\"dcsat:TableRow\"><dcsat:tableCell>{app}</dcsat:tableCell><dcsat:tableCell>{app}</dcsat:tableCell></dcsat:item></template></template>\r\n\
             \t<settingsVariant><dcsset:name>О</dcsset:name><dcsset:settings xmlns:pal=\"p\"/></settingsVariant>\r\n</DataCompositionSchema>"
        );
        let (Rec::Raw(b), _) = record(&x, &Index::default(), None).unwrap() else { panic!() };
        assert_eq!(&b[..8], &[0, 0, 0, 0, 1, 0, 0, 0]);
        let t = String::from_utf8_lossy(&b[24..]).into_owned();
        assert!(t.contains("<title xmlns:d4p1=\"http://v8.1c.ru/8.1/data/core\" xsi:type=\"d4p1:LocalStringType\">\r\n\t\t\t\t<d4p1:item>"), "{t}");
        assert!(t.contains("<Type xmlns=\"http://v8.1c.ru/8.1/data/core\">xs:string</Type>"), "{t}");
        assert!(t.contains("<name xmlns=\"http://v8.1c.ru/8.1/data-composition-system/settings\">О</name>"), "{t}");
        assert!(t.contains("<Settings xmlns=\"http://v8.1c.ru/8.1/data-composition-system/settings\" xmlns:dcscor="), "{t}");
        // одно оформление на две ячейки
        assert_eq!(t.matches("<dcsat:appIndex>0</dcsat:appIndex>").count(), 2, "{t}");
        assert_eq!(t.matches("xsi:type=\"TableCellAppearance\"").count(), 1, "{t}");
    }

    #[test]
    fn server_state_prefixes() {
        // запись загрузки БП (ПодборДокументовДляЭДО): пространство схемы — префикс dcssch, прочие — d<глубина>p<номер>
        let src = format!("<x xmlns=\"\" xmlns:dcssch=\"{SCH}\" xmlns:v8=\"{V8}\" xmlns:xsi=\"{XSI}\"><Field xsi:type=\"dcssch:DataSetFieldField\"><dcssch:dataPath>А</dcssch:dataPath><dcssch:title xsi:type=\"v8:LocalStringType\"><v8:item><v8:lang>ru</v8:lang><v8:content>а</v8:content></v8:item></dcssch:title></Field></x>");
        let t = server_state_part(&src, &Index::default()).unwrap();
        let want = format!("{BOM}<?xml version=\"1.0\" encoding=\"UTF-8\"?>\r\n<UniversalListServerOnlyState xmlns=\"\" xmlns:xs=\"{XS}\" xmlns:xsi=\"{XSI}\">\r\n\t<Field xmlns:dcssch=\"{SCH}\" xsi:type=\"dcssch:DataSetFieldField\">\r\n\t\t<dcssch:dataPath>А</dcssch:dataPath>\r\n\t\t<dcssch:title xmlns:d3p1=\"{V8}\" xsi:type=\"d3p1:LocalStringType\">\r\n\t\t\t<d3p1:item>\r\n\t\t\t\t<d3p1:lang>ru</d3p1:lang>\r\n\t\t\t\t<d3p1:content>а</d3p1:content>\r\n\t\t\t</d3p1:item>\r\n\t\t</dcssch:title>\r\n\t</Field>\r\n</UniversalListServerOnlyState>");
        assert_eq!(t, want);
    }
}
