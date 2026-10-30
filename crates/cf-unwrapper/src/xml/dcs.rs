//! Макеты, которые выгружаются в `Ext/Template.xml` не табличным документом: схема компоновки данных,
//! макет оформления компоновки (виды 6 и 7 из `payload::template`; графическая схема — `graph.rs`, HTML — `payload::html`).
//! Ссылки на объекты конфигурации по GUID разрешаются через `common::Index`.
//! Устройство записей и правила пересборки — docs/research/xml-dcs.md.
//!
//! Запись схемы: `u32 0, u32 n`, n+1 длин `u64`, затем части: схема (`<SchemaFile>`), n-1 вариантов настроек
//! (`<Settings>`) и хвост (`<SchemaFile>` с макетами и общим списком оформлений, на который ссылаются `appIndex`).
//! Платформа читает части в объекты и пишет одним документом; здесь то же делается на уровне XML-дерева.

use super::common::{self, Index};
use super::payload::{STD_NUMBERED, STD_PICTURES};
use super::BOM;

const XSI: &str = "http://www.w3.org/2001/XMLSchema-instance";
const SCH: &str = "http://v8.1c.ru/8.1/data-composition-system/schema";
const SET: &str = "http://v8.1c.ru/8.1/data-composition-system/settings";
const AT: &str = "http://v8.1c.ru/8.1/data-composition-system/area-template";
const V8: &str = "http://v8.1c.ru/8.1/data/core";
const V8UI: &str = "http://v8.1c.ru/8.1/data/ui";
const STYLE: &str = "http://v8.1c.ru/8.1/data/ui/style";
const CFG: &str = "http://v8.1c.ru/8.1/data/enterprise/current-config";
const READABLE: &str = "http://v8.1c.ru/8.3/xcf/readable";

/// Пространства корня `<DataCompositionSchema>` в порядке объявления.
const ROOT: &[(&str, &str)] = &[
    (SCH, ""),
    ("http://v8.1c.ru/8.1/data-composition-system/common", "dcscom"),
    ("http://v8.1c.ru/8.1/data-composition-system/core", "dcscor"),
    (SET, "dcsset"),
    (V8, "v8"),
    (V8UI, "v8ui"),
    ("http://www.w3.org/2001/XMLSchema", "xs"),
    (XSI, "xsi"),
];
/// Объявляются на каждом `<dcsset:settings>` варианта.
const SETTINGS: &[(&str, &str)] = &[
    ("http://v8.1c.ru/8.1/data/ui/colors/palette", "pal"),
    (STYLE, "style"),
    ("http://v8.1c.ru/8.1/data/ui/fonts/system", "sys"),
    ("http://v8.1c.ru/8.1/data/ui/colors/web", "web"),
    ("http://v8.1c.ru/8.1/data/ui/colors/windows", "win"),
];
/// Пространства, которые в хранимой части стоят по умолчанию (`xmlns="..."`), а в выгрузке получают префикс.
const LOCAL: &[(&str, &str)] = &[(AT, "dcsat"), ("http://v8.1c.ru/8.2/data/spreadsheet", "mxl"), ("http://v8.1c.ru/8.1/data/ui/fonts/system", "sys")];
/// Картинки платформы, встреченные в схемах только по имени (`ref="v8ui:Имя"`; БП 3.0.171 и УТ 11.5.27), без GUID.
const STD_NAMED: &[&str] = &["AppearanceCircleGreen", "AppearanceCircleRed", "AppearanceCircleYellow", "EventLog", "GrayedAll"];

/// Файлы схемы компоновки (макет вида 6): Template.xml и Template/value.svg.
/// `old` — формат до 2.21: картинки значений остаются как хранятся (`v8ui:Picture ref=…`), «читаемый» вид — с 8.5.
pub(super) fn schema_files(b: &[u8], index: &Index, old: bool) -> Result<Vec<(String, Vec<u8>)>, String> {
    let parts = split(b).ok_or("схема компоновки: незнакомое устройство записи")?;
    let (xml, svg) = schema(&parts, index, old)?;
    let mut out = vec![("Template.xml".to_string(), xml)];
    out.extend(svg.map(|s| (format!("Template/value.{}", super::payload::picture_ext(&s).unwrap_or("svg")), s)));
    Ok(out)
}

/// Корень макета оформления компоновки в выгрузке.
const APPEARANCE_ROOT: &str = "<AppearanceTemplate xmlns=\"http://v8.1c.ru/8.1/data-composition-system/appearance-template\" xmlns:dcscor=\"http://v8.1c.ru/8.1/data-composition-system/core\" xmlns:pal=\"http://v8.1c.ru/8.1/data/ui/colors/palette\" xmlns:style=\"http://v8.1c.ru/8.1/data/ui/style\" xmlns:sys=\"http://v8.1c.ru/8.1/data/ui/fonts/system\" xmlns:v8ui=\"http://v8.1c.ru/8.1/data/ui\" xmlns:web=\"http://v8.1c.ru/8.1/data/ui/colors/web\" xmlns:win=\"http://v8.1c.ru/8.1/data/ui/colors/windows\" xmlns:xs=\"http://www.w3.org/2001/XMLSchema\" xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\">";

/// Макет оформления хранится готовым XML; платформа заменяет только объявления корня.
/// shortcut: остальное копируется как есть (в БП всего 3 макета, все сошлись); при расхождениях — пересобирать деревом, как схему.
pub(super) fn appearance(b: &[u8]) -> Option<Vec<u8>> {
    let t = std::str::from_utf8(b).ok()?;
    let at = t.find("<AppearanceTemplate").filter(|&i| i < 100)?;
    let end = at + t[at..].find('>')? + 1;
    Some([t[..at].as_bytes(), APPEARANCE_ROOT.as_bytes(), t[end..].as_bytes()].concat())
}

/// Части записи схемы; `None`, если это не схема.
fn split(b: &[u8]) -> Option<Vec<&[u8]>> {
    let u32_at = |i: usize| b.get(i..i + 4).map(|s| u32::from_le_bytes(s.try_into().unwrap()));
    if u32_at(0)? != 0 {
        return None;
    }
    let n = u32_at(4)? as usize;
    if n == 0 || n > 10_000 {
        return None;
    }
    let mut pos = 8 + 8 * (n + 1);
    let mut out = Vec::with_capacity(n + 2);
    for k in 0..=n {
        let l = u64::from_le_bytes(b.get(8 + 8 * k..16 + 8 * k)?.try_into().unwrap()) as usize;
        out.push(b.get(pos..pos.checked_add(l)?)?);
        pos += l;
    }
    out.push(&b[pos..]);
    out[0].windows(11).take(200).any(|w| w == b"<SchemaFile").then_some(out)
}

/// Имя объекта класса `class` (`StyleItem.`, `CommonPicture.`) по GUID.
fn named<'a>(x: &'a Index, id: &str, class: &str) -> Option<&'a str> {
    x.full.get(id)?.strip_prefix(class)
}

/// `TypeId` → (`Type` или `TypeSet`, имя типа): порожденный тип объекта конфигурации или набор платформы.
fn type_ref(x: &Index, id: &str) -> Option<(&'static str, Q)> {
    let q = |ns: &str, local: &str| Q { ns: ns.into(), local: local.into(), prefix: String::new() };
    if let Some(t) = x.types.get(id) {
        // Определяемый тип платформа в схеме оставляет TypeId (32 схемы БП 3.0.171).
        if t.starts_with("DefinedType.") {
            return None;
        }
        return Some((if t.starts_with("Characteristic.") { "TypeSet" } else { "Type" }, q(CFG, t)));
    }
    let (el, v) = common::platform_type(id)?;
    let (p, local) = v.split_once(':')?;
    Some((el.strip_prefix("v8:")?, q(if p == "cfg" { CFG } else { V8 }, local)))
}

// ---- XML-дерево без потери текста: значения и тексты хранятся как в исходнике (с сущностями) ----

/// Имя с разрешенным пространством; `prefix` — как было записано в источнике.
#[derive(Clone, Debug, PartialEq)]
struct Q {
    ns: String,
    local: String,
    prefix: String,
}

#[derive(Clone, Debug)]
enum Val {
    Raw(String),
    Q(Q),
}

#[derive(Clone, Debug)]
struct Attr {
    name: Q,
    val: Val,
}

/// Типы описания `<Type>`/`<TypeId>` — в порядке GUID, как выгружает платформа: 8.5 хранит схему с простым типом
/// первым (`xs:string`, затем `TypeId`), а выгружает его среди ссылок по GUID простого типа (core85, как
/// `imp/tpl.rs` `PRIMITIVE_GUIDS`; Макет ИзменениеРолейПрофилей корпуса 8.5). Незнакомый тип или набор — как есть.
fn sort_types(kids: &mut [El], is_set: &dyn Fn(&str) -> bool) {
    const PRIMITIVE: &[(&str, &str)] = &[
        ("null", "af40a278-63bc-478e-91a8-19e0d16b10b5"),
        ("boolean", "5d4125ad-f6e7-4313-be32-f71d0ab60915"),
        ("decimal", "b0be78f2-0ee6-4d31-a3bb-77dd32ba5bec"),
        ("string", "9b6abf8b-0173-48e5-b0a0-83b21fcf63c5"),
        ("dateTime", "aae38c48-a877-411c-a6d3-fbaa1f83c4bd"),
    ];
    let at: Vec<usize> = (0..kids.len()).filter(|&i| kids[i].ns == V8 && matches!(kids[i].name.as_str(), "Type" | "TypeId" | "TypeSet")).collect();
    if at.len() < 2 {
        return;
    }
    let key = |e: &El| -> Option<String> {
        let t = e.text.as_deref().unwrap_or("");
        match e.name.as_str() {
            // набор (`TypeId` набора платформы) — после типов, как и `TypeSet`
            "TypeId" if crate::brace::is_guid(t) && is_set(t) => Some("~".into()),
            "TypeId" if crate::brace::is_guid(t) => Some(t.to_string()),
            "Type" => {
                let local = e.qtext.as_ref().map_or_else(|| t.rsplit(':').next().unwrap_or(""), |q| q.local.as_str());
                let local = if local == "Null" { "null" } else { local };
                PRIMITIVE.iter().find(|p| p.0 == local).map(|p| p.1.to_string())
            }
            // shortcut: «любая ссылка» — последней (3 схемы корпуса 8.5, там она `TypeId` набора); GUID набора не выведен
            "TypeSet" if e.qtext.as_ref().is_some_and(|q| matches!(q.local.as_str(), "AnyRef" | "AnyIBRef")) => Some("~".into()),
            _ => None,
        }
    };
    let Some(keys) = at.iter().map(|&i| key(&kids[i])).collect::<Option<Vec<String>>>() else { return };
    let mut order: Vec<usize> = (0..at.len()).collect();
    order.sort_by(|&a, &b| keys[a].cmp(&keys[b]));
    let moved: Vec<El> = order.iter().map(|&k| std::mem::take(&mut kids[at[k]])).collect();
    for (&i, e) in at.iter().zip(moved) {
        kids[i] = e;
    }
}

#[derive(Clone, Debug, Default)]
struct El {
    ns: String,
    prefix: String,
    name: String,
    attrs: Vec<Attr>,
    kids: Vec<El>,
    text: Option<String>,
    /// Текст — QName (`xs:string`, `style:Цвет`): префикс при записи пересчитывается.
    qtext: Option<Q>,
}

impl El {
    fn new(ns: &str, prefix: &str, name: &str) -> El {
        El { ns: ns.into(), prefix: prefix.into(), name: name.into(), ..El::default() }
    }
    fn is_empty(&self) -> bool {
        self.kids.is_empty() && self.text.is_none() && self.attrs.is_empty()
    }
    fn attr(&self, ns: &str, name: &str) -> Option<&Val> {
        self.attrs.iter().find(|a| a.name.ns == ns && a.name.local == name).map(|a| &a.val)
    }
    fn xsi_type(&self) -> Option<&Q> {
        match self.attr(XSI, "type")? {
            Val::Q(q) => Some(q),
            Val::Raw(_) => None,
        }
    }
}

/// `префикс:имя` → (префикс, имя); как в XML: префикс начинается с буквы или `_`, без пробелов и второго `:`.
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

type NsMap = Vec<(String, String)>;

fn lookup<'a>(m: &'a NsMap, p: &str) -> Option<&'a str> {
    m.iter().rev().find(|(k, _)| k == p).map(|(_, v)| v.as_str())
}

/// QName из значения: только с объявленным префиксом (кроме `xml`), без префикса — если `default`.
fn qname(t: &str, m: &NsMap, default: bool) -> Option<Q> {
    let (p, l) = split_qname(t)?;
    match p {
        None => {
            let ns = lookup(m, "").filter(|n| default && !n.is_empty())?;
            Some(Q { ns: ns.into(), local: t.into(), prefix: String::new() })
        }
        Some("xml") => None,
        Some(p) => Some(Q { ns: lookup(m, p)?.into(), local: l.into(), prefix: p.into() }),
    }
}

/// Предел вложенности элементов, как у скобочного формата: дальше рекурсия записи рискует переполнить стек.
const MAX_DEPTH: usize = 256;

fn parse(text: &str) -> Result<El, String> {
    let s = text.strip_prefix('\u{feff}').unwrap_or(text);
    let mut stack: Vec<(NsMap, El)> = Vec::new();
    let mut root: Option<El> = None;
    let base: NsMap = vec![("xml".into(), "http://www.w3.org/XML/1998/namespace".into()), (String::new(), String::new())];
    let mut i = 0;
    while i < s.len() {
        let rest = &s[i..];
        if !rest.starts_with('<') {
            let end = rest.find('<').unwrap_or(rest.len());
            if let Some((_, e)) = stack.last_mut() {
                e.text = Some(rest[..end].to_string());
            }
            i += end;
            continue;
        }
        if rest.starts_with("<?") || rest.starts_with("<!--") {
            let close = if rest.starts_with("<?") { "?>" } else { "-->" };
            i += rest.find(close).ok_or("xml: незакрытая инструкция")? + close.len();
            continue;
        }
        let end = tag_end(rest).ok_or("xml: незакрытый тег")?;
        let tag = &rest[1..end];
        i += end + 1;
        if let Some(name) = tag.strip_prefix('/') {
            let (m, mut el) = stack.pop().ok_or("xml: лишний закрывающий тег")?;
            if name.trim() != qual(&el.prefix, &el.name) {
                return Err(format!("xml: </{name}> вместо </{}>", qual(&el.prefix, &el.name)));
            }
            if !el.kids.is_empty() {
                el.text = None;
            } else if let Some(t) = &el.text {
                el.qtext = qname(t, &m, el.ns == V8 && (el.name == "Type" || el.name == "TypeSet"));
            }
            close(el, &mut stack, &mut root);
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
            let v0 = attrs_src[eq + 1..].trim_start();
            let v0 = v0.strip_prefix('"').ok_or("xml: значение атрибута без кавычек")?;
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
        let mut el = El::new(lookup(&m, p).unwrap_or(""), p, l);
        for (a, v) in raw {
            let (ap, al) = a.split_once(':').unwrap_or(("", a));
            let ans = if ap.is_empty() { "" } else { lookup(&m, ap).unwrap_or("") };
            let name = Q { ns: ans.into(), local: al.into(), prefix: ap.into() };
            let val = if ans == XSI && al == "type" {
                let (vp, vl) = v.split_once(':').unwrap_or(("", v));
                Val::Q(Q { ns: lookup(&m, vp).unwrap_or("").into(), local: vl.into(), prefix: vp.into() })
            } else {
                qname(v, &m, false).map_or_else(|| Val::Raw(v.into()), Val::Q)
            };
            el.attrs.push(Attr { name, val });
        }
        if empty {
            close(el, &mut stack, &mut root);
        } else {
            if stack.len() >= MAX_DEPTH {
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

fn close(el: El, stack: &mut [(NsMap, El)], root: &mut Option<El>) {
    match stack.last_mut() {
        Some((_, p)) => p.kids.push(el),
        None => {
            if root.is_none() {
                *root = Some(el);
            }
        }
    }
}

fn qual(p: &str, l: &str) -> String {
    if p.is_empty() {
        l.into()
    } else {
        format!("{p}:{l}")
    }
}

/// `dNpK` — префикс, который сериализатор платформы порождает сам; при записи он пересчитывается.
fn generated(p: &str) -> bool {
    let Some(r) = p.strip_prefix('d') else { return false };
    let Some((a, b)) = r.split_once('p') else { return false };
    !a.is_empty() && !b.is_empty() && a.bytes().all(|c| c.is_ascii_digit()) && b.bytes().all(|c| c.is_ascii_digit())
}

struct Writer {
    out: String,
}

impl Writer {
    /// `scope` — пространство → префикс; `par` — имя родителя.
    fn el(&mut self, e: &El, scope: &[(String, String)], depth: usize, par: &str) {
        let mut scope = scope.to_vec();
        let mut decl: Vec<(String, String)> = Vec::new();
        // Настройки варианта и вложенной схемы объявляют пространства оформления, даже если они не нужны.
        let extra = if e.name == "settings" && (par == "settingsVariant" || par == "nestedSchema") { SETTINGS } else { &[] };
        for (ns, p) in extra {
            scope.push((ns.to_string(), p.to_string()));
            decl.push((p.to_string(), ns.to_string()));
        }
        let mut pfx = |ns: &str, stored: &str, scope: &mut Vec<(String, String)>| -> String {
            if let Some((_, p)) = scope.iter().rev().find(|(n, _)| n == ns) {
                return p.clone();
            }
            let p = if !stored.is_empty() && !generated(stored) {
                stored.to_string()
            } else if let Some((_, p)) = LOCAL.iter().find(|(n, _)| *n == ns) {
                p.to_string()
            } else {
                format!("d{}p{}", depth + 1, decl.len() + 1)
            };
            scope.push((ns.into(), p.clone()));
            decl.push((p.clone(), ns.into()));
            p
        };
        let p = pfx(&e.ns, &e.prefix, &mut scope);
        let mut attrs = Vec::new();
        for a in &e.attrs {
            let ap = if a.name.ns.is_empty() { String::new() } else { pfx(&a.name.ns, &a.name.prefix, &mut scope) };
            let v = match &a.val {
                Val::Raw(v) => v.clone(),
                Val::Q(q) if q.ns.is_empty() => q.local.clone(),
                Val::Q(q) => qual(&pfx(&q.ns, &q.prefix, &mut scope), &q.local),
            };
            attrs.push(format!("{}=\"{v}\"", qual(&ap, &a.name.local)));
        }
        // Шрифт без масштаба платформа пишет с масштабом по умолчанию.
        if e.xsi_type().is_some_and(|q| q.local == "Font") && matches!(e.attr("", "kind"), Some(Val::Raw(k)) if k == "Absolute") && e.attr("", "scale").is_none() {
            attrs.push("scale=\"100\"".into());
        }
        let text = match &e.qtext {
            Some(q) => Some(qual(&pfx(&q.ns, &q.prefix, &mut scope), &q.local)),
            None => e.text.clone(),
        };
        let name = qual(&p, &e.name);
        let ind = "\t".repeat(depth);
        let mut head = format!("{ind}<{name}");
        for (dp, ns) in &decl {
            head += &format!(" xmlns{}=\"{ns}\"", if dp.is_empty() { String::new() } else { format!(":{dp}") });
        }
        for a in &attrs {
            head += " ";
            head += a;
        }
        self.out += &head;
        if !e.kids.is_empty() {
            self.out += ">\r\n";
            for k in &e.kids {
                self.el(k, &scope, depth + 1, &e.name);
            }
            self.out += &format!("{ind}</{name}>\r\n");
        } else if let Some(t) = text {
            self.out += &format!(">{t}</{name}>\r\n");
        } else {
            self.out += "/>\r\n";
        }
    }
}

// ---- Пересборка схемы ----

/// Template.xml и встроенная SVG-картинка: все такие картинки платформа пишет в один `value.svg`, остается последняя.
fn schema(parts: &[&[u8]], refs: &Index, old: bool) -> Result<(Vec<u8>, Option<Vec<u8>>), String> {
    let text = |b: &[u8]| String::from_utf8(b.to_vec()).map_err(|_| "схема: не UTF-8".to_string());
    let mut sch = parse(&text(parts[0])?)?;
    let mut tail = parse(&text(parts[parts.len() - 1])?)?;
    let mut sets = parts[1..parts.len() - 1].iter().map(|p| parse(&text(p)?)).collect::<Result<Vec<_>, _>>()?;
    if sch.kids.is_empty() || tail.kids.is_empty() {
        return Err("схема: нет dataCompositionSchema".into());
    }
    let mut dcs = sch.kids.swap_remove(0);
    let apps: Vec<El> = tail.kids.iter().filter(|k| k.name == "appearance").cloned().collect();
    let mut tk = std::mem::take(&mut tail.kids[0].kids);
    let fx = Fix { apps: &apps, refs, old, svg: Default::default() };
    for t in &mut tk {
        fx.run(t, "")?;
    }
    fx.run(&mut dcs, "")?;
    for s in &mut sets {
        fx.run(s, "")?;
    }
    // Макеты из хвоста — перед вариантами настроек.
    let mut kids = dcs.kids;
    let at = kids.iter().position(|k| k.name == "settingsVariant").unwrap_or(kids.len());
    kids.splice(at..at, tk);
    let mut sets = sets.into_iter();
    let mut w = Writer { out: String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\r\n<DataCompositionSchema") };
    for (ns, p) in ROOT {
        w.out += &format!(" xmlns{}=\"{ns}\"", if p.is_empty() { String::new() } else { format!(":{p}") });
    }
    w.out += ">\r\n";
    let scope: Vec<(String, String)> = ROOT.iter().map(|(n, p)| (n.to_string(), p.to_string())).collect();
    for k in &kids {
        if k.name != "settingsVariant" {
            w.el(k, &scope, 1, "DataCompositionSchema");
            continue;
        }
        let s = sets.next().ok_or("схема: вариантов больше, чем частей настроек")?;
        let mut st = El::new(SET, "", "settings");
        st.kids = s.kids;
        w.out += "\t<settingsVariant>\r\n";
        for c in &k.kids {
            w.el(c, &scope, 2, "settingsVariant");
        }
        w.el(&st, &scope, 2, "settingsVariant");
        w.out += "\t</settingsVariant>\r\n";
    }
    w.out += "</DataCompositionSchema>";
    Ok(([BOM, w.out.as_bytes()].concat(), fx.svg.into_inner()))
}

struct Fix<'a> {
    apps: &'a [El],
    refs: &'a Index,
    old: bool,
    svg: std::cell::RefCell<Option<Vec<u8>>>,
}

impl Fix<'_> {
    /// `par` — имя родителя `e`: пустое оформление сохраняется только у элементов условного оформления.
    fn run(&self, e: &mut El, par: &str) -> Result<(), String> {
        sort_types(&mut e.kids, &|g| type_ref(self.refs, g).is_some_and(|(tag, _)| tag == "TypeSet"));
        let mut nk = Vec::with_capacity(e.kids.len());
        for mut k in std::mem::take(&mut e.kids) {
            // Оформление ячейки хранится индексом в общем списке хвоста.
            if k.name == "appIndex" && k.ns == AT {
                let i: usize = k.text.as_deref().and_then(|t| t.parse().ok()).ok_or("схема: appIndex не число")?;
                let a = self.apps.get(i).ok_or("схема: appIndex вне списка")?;
                if !a.kids.is_empty() {
                    let mut n = El::new(AT, "dcsat", "appearance");
                    n.kids = a.kids.clone();
                    self.run(&mut n, &e.name)?;
                    nk.push(n);
                }
                continue;
            }
            self.run(&mut k, &e.name)?;
            if k.is_empty() && (k.name == "outputParameters" || k.name == "appearance" && par != "conditionalAppearance") {
                continue;
            }
            nk.push(k);
        }
        e.kids = nk;
        self.value(e);
        Ok(())
    }

    /// Ссылки на объекты конфигурации хранятся по GUID, выгружаются по имени.
    fn value(&self, e: &mut El) {
        if e.ns == V8 && e.name == "TypeId" {
            if let Some((tag, q)) = e.text.as_deref().and_then(|t| type_ref(self.refs, t)) {
                e.name = tag.to_string();
                e.qtext = Some(q);
                e.text = Some(String::new());
            }
            return;
        }
        // Старое имя набора «любая ссылка».
        if let (true, Some(q)) = (e.ns == V8 && e.name == "TypeSet", &mut e.qtext) {
            if q.ns == CFG && q.local == "AnyRef" {
                q.local = "AnyIBRef".into();
            }
        }
        // Пустой вариант многоязычной строки не выгружается.
        if e.xsi_type().is_some_and(|q| q.ns == V8 && q.local == "LocalStringType") {
            e.kids.retain(|k| !(k.name == "item" && k.kids.iter().any(|c| c.name == "content" && c.is_empty())));
        }
        let Some(t) = e.xsi_type().filter(|q| q.ns == V8UI).map(|q| q.local.clone()) else { return };
        let style = |g: &str| named(self.refs, g.strip_prefix("0:")?, "StyleItem.").map(|n| Q { ns: STYLE.into(), local: n.to_string(), prefix: String::new() });
        match t.as_str() {
            "Color" => {
                if let Some(q) = e.text.as_deref().and_then(style) {
                    e.qtext = Some(q);
                }
            }
            "Font" => {
                for a in &mut e.attrs {
                    if let (true, Val::Raw(v)) = (a.name.ns.is_empty() && a.name.local == "ref", &a.val) {
                        // У шрифта элемент стиля конфигурации пишется с префиксом style, у цвета — с порожденным dNpK.
                        if let Some(q) = style(v) {
                            a.val = Val::Q(Q { prefix: "style".into(), ..q });
                        }
                    }
                }
            }
            "Picture" if !self.old => self.picture(e),
            _ => {}
        }
    }

    /// Картинка выгружается в «читаемом» виде: `<Ref>CommonPicture.Имя</Ref>` или `<Abs>value.svg</Abs>`.
    fn picture(&self, e: &mut El) {
        let src = match (e.attr("", "ref"), &e.text) {
            (None, None) => None,
            (Some(Val::Raw(raw)), _) => {
                let g = raw.strip_prefix("0:").unwrap_or(raw);
                match (named(self.refs, g, "CommonPicture."), STD_PICTURES.iter().find(|(id, _)| *id == g).map(|(_, n)| *n).or_else(|| crate::xml::imp::form::platform::std_picture(g))) {
                    (Some(n), _) => Some(("Ref", format!("CommonPicture.{n}"))),
                    (None, Some(n)) => Some(("Ref", format!("StdPicture.{n}"))),
                    // Ссылки нет ни в конфигурации, ни у платформы: она пишет ее как есть (корпус 8.5: АнализПравДоступа,
                    // ПраваРолей, СхемаРаспределенияЗатрат — `<Ref>0:guid</Ref>`).
                    _ if raw.starts_with("0:") && crate::brace::is_guid(g) => Some(("Ref", raw.clone())),
                    _ => return,
                }
            }
            // По имени: общая картинка конфигурации или известная картинка платформы; незнакомое имя остается как есть.
            (Some(Val::Q(q)), _) if q.ns == V8UI => {
                let common = self.refs.full.values().any(|n| n.strip_prefix("CommonPicture.") == Some(q.local.as_str()));
                let std = STD_PICTURES.iter().chain(STD_NUMBERED).any(|(_, n)| *n == q.local)
                    || STD_NAMED.contains(&q.local.as_str())
                    || crate::xml::imp::form::platform::std_picture_id(&q.local).is_some();
                match (common, std) {
                    (true, _) => Some(("Ref", format!("CommonPicture.{}", q.local))),
                    (false, true) => Some(("Ref", format!("StdPicture.{}", q.local))),
                    _ => return,
                }
            }
            // встроенная картинка — файл `value.<расширение по сигнатуре>` (SVG, PNG… — `payload::picture_ext`)
            (None, Some(t)) if super::base64(t).ok().as_deref().and_then(super::payload::picture_ext).is_some() => {
                let d = super::base64(t).unwrap_or_default();
                let ext = super::payload::picture_ext(&d).unwrap_or("svg");
                *self.svg.borrow_mut() = Some(d);
                Some(("Abs", format!("value.{ext}")))
            }
            (None, Some(t)) => {
                e.text = Some(format!("{}{t}", super::UNK));
                return;
            }
            _ => return,
        };
        let mut n = El::new(&e.ns, &e.prefix, &e.name);
        n.attrs.push(Attr { name: Q { ns: XSI.into(), local: "type".into(), prefix: "xsi".into() }, val: Val::Q(Q { ns: READABLE.into(), local: "Picture".into(), prefix: String::new() }) });
        if let Some((tag, v)) = src {
            let mut a = El::new(READABLE, "", tag);
            a.text = Some(v);
            let mut t = El::new(READABLE, "", "LoadTransparent");
            t.text = Some("true".into());
            n.kids = vec![a, t];
        }
        *e = n;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn refs() -> Index {
        Index { full: Default::default(), types: Default::default() }
    }

    fn entry(parts: &[&str]) -> Vec<u8> {
        let mut b = vec![0, 0, 0, 0];
        b.extend((parts.len() as u32 - 2).to_le_bytes());
        for p in &parts[..parts.len() - 1] {
            b.extend((p.len() as u64).to_le_bytes());
        }
        for p in parts {
            b.extend(p.as_bytes());
        }
        b
    }

    const EMPTY_TAIL: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\r\n<SchemaFile xmlns=\"\" xmlns:xs=\"http://www.w3.org/2001/XMLSchema\" xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\">\r\n\t<dataCompositionSchema xmlns=\"http://v8.1c.ru/8.1/data-composition-system/schema\"/>\r\n</SchemaFile>";

    #[test]
    fn schema_merges_parts() {
        let sch = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\r\n<SchemaFile xmlns=\"\" xmlns:xs=\"http://www.w3.org/2001/XMLSchema\" xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\">\r\n\t<dataCompositionSchema xmlns=\"http://v8.1c.ru/8.1/data-composition-system/schema\">\r\n\t\t<dataSet xsi:type=\"DataSetObject\">\r\n\t\t\t<field xsi:type=\"DataSetFieldField\">\r\n\t\t\t\t<valueType>\r\n\t\t\t\t\t<Type xmlns=\"http://v8.1c.ru/8.1/data/core\">xs:string</Type>\r\n\t\t\t\t</valueType>\r\n\t\t\t\t<appearance/>\r\n\t\t\t</field>\r\n\t\t</dataSet>\r\n\t\t<settingsVariant>\r\n\t\t\t<name xmlns=\"http://v8.1c.ru/8.1/data-composition-system/settings\">Осн</name>\r\n\t\t</settingsVariant>\r\n\t</dataCompositionSchema>\r\n</SchemaFile>";
        let set = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\r\n<Settings xmlns=\"http://v8.1c.ru/8.1/data-composition-system/settings\"><outputParameters/><filter/></Settings>";
        let b = entry(&[sch, set, EMPTY_TAIL]);
        let p = split(&b).unwrap();
        assert_eq!(p.len(), 3);
        let out = String::from_utf8(schema(&p, &refs(), false).unwrap().0).unwrap();
        let want = "\u{feff}<?xml version=\"1.0\" encoding=\"UTF-8\"?>\r\n<DataCompositionSchema xmlns=\"http://v8.1c.ru/8.1/data-composition-system/schema\" xmlns:dcscom=\"http://v8.1c.ru/8.1/data-composition-system/common\" xmlns:dcscor=\"http://v8.1c.ru/8.1/data-composition-system/core\" xmlns:dcsset=\"http://v8.1c.ru/8.1/data-composition-system/settings\" xmlns:v8=\"http://v8.1c.ru/8.1/data/core\" xmlns:v8ui=\"http://v8.1c.ru/8.1/data/ui\" xmlns:xs=\"http://www.w3.org/2001/XMLSchema\" xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\">\r\n\t<dataSet xsi:type=\"DataSetObject\">\r\n\t\t<field xsi:type=\"DataSetFieldField\">\r\n\t\t\t<valueType>\r\n\t\t\t\t<v8:Type>xs:string</v8:Type>\r\n\t\t\t</valueType>\r\n\t\t</field>\r\n\t</dataSet>\r\n\t<settingsVariant>\r\n\t\t<dcsset:name>Осн</dcsset:name>\r\n\t\t<dcsset:settings xmlns:pal=\"http://v8.1c.ru/8.1/data/ui/colors/palette\" xmlns:style=\"http://v8.1c.ru/8.1/data/ui/style\" xmlns:sys=\"http://v8.1c.ru/8.1/data/ui/fonts/system\" xmlns:web=\"http://v8.1c.ru/8.1/data/ui/colors/web\" xmlns:win=\"http://v8.1c.ru/8.1/data/ui/colors/windows\">\r\n\t\t\t<dcsset:filter/>\r\n\t\t</dcsset:settings>\r\n\t</settingsVariant>\r\n</DataCompositionSchema>";
        assert_eq!(out, want);
    }

    #[test]
    fn app_index_inlined_and_prefixes_generated() {
        let sch = EMPTY_TAIL.replace("schema\"/>", "schema\">\r\n\t\t<x xsi:type=\"Font\" kind=\"Absolute\"/>\r\n\t</dataCompositionSchema>");
        let tail = "<?xml version=\"1.0\"?><SchemaFile xmlns=\"\" xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\"><dataCompositionSchema xmlns=\"http://v8.1c.ru/8.1/data-composition-system/schema\"><template><item xmlns=\"http://v8.1c.ru/8.1/data-composition-system/area-template\"><tableCell><appIndex>1</appIndex></tableCell><tableCell><appIndex>0</appIndex></tableCell></item></template></dataCompositionSchema>\
            <appearance xmlns=\"http://v8.1c.ru/8.1/data-composition-system/area-template\" xsi:type=\"TableCellAppearance\"/>\
            <appearance xmlns=\"http://v8.1c.ru/8.1/data-composition-system/area-template\" xsi:type=\"TableCellAppearance\"><item xmlns=\"http://v8.1c.ru/8.1/data-composition-system/core\"><value xmlns:d4p1=\"http://v8.1c.ru/8.1/data/ui/style\" xmlns:q=\"urn:q\" xsi:type=\"q:T\">d4p1:Цвет</value></item></appearance></SchemaFile>";
        let b = entry(&[&sch, "<Settings xmlns=\"http://v8.1c.ru/8.1/data-composition-system/settings\"/>", tail]);
        let out = String::from_utf8(schema(&split(&b).unwrap(), &refs(), false).unwrap().0).unwrap();
        let body = &out[out.find(">\r\n\t").unwrap() + 3..];
        assert_eq!(
            body,
            "\t<x xsi:type=\"Font\" kind=\"Absolute\" scale=\"100\"/>\r\n\t<template>\r\n\t\t<dcsat:item xmlns:dcsat=\"http://v8.1c.ru/8.1/data-composition-system/area-template\">\r\n\t\t\t<dcsat:tableCell>\r\n\t\t\t\t<dcsat:appearance>\r\n\t\t\t\t\t<dcscor:item>\r\n\t\t\t\t\t\t<dcscor:value xmlns:q=\"urn:q\" xmlns:d7p2=\"http://v8.1c.ru/8.1/data/ui/style\" xsi:type=\"q:T\">d7p2:Цвет</dcscor:value>\r\n\t\t\t\t\t</dcscor:item>\r\n\t\t\t\t</dcsat:appearance>\r\n\t\t\t</dcsat:tableCell>\r\n\t\t\t<dcsat:tableCell/>\r\n\t\t</dcsat:item>\r\n\t</template>\r\n</DataCompositionSchema>"
        );
    }

    #[test]
    fn type_refs() {
        let mut x = refs();
        for (g, t) in [("t1", "DefinedType.Д"), ("t2", "CatalogRef.С"), ("t3", "Characteristic.Х")] {
            x.types.insert(g.into(), t.into());
        }
        let r = |id: &str| type_ref(&x, id).map(|(el, q)| format!("{el} {} {}", q.ns.rsplit('/').next().unwrap(), q.local));
        assert_eq!(r("t1"), None);
        assert_eq!(r("t2").as_deref(), Some("Type current-config CatalogRef.С"));
        assert_eq!(r("t3").as_deref(), Some("TypeSet current-config Characteristic.Х"));
        assert_eq!(r("280f5f0e-9c8a-49cc-bf6d-4d296cc17a63").as_deref(), Some("TypeSet current-config AnyIBRef"));
        assert_eq!(r("fc01b5df-97fe-449b-83d4-218a090e681e").as_deref(), Some("Type core UUID"));
        assert_eq!(r("00000000-0000-0000-0000-000000000001"), None);
    }

    #[test]
    fn depth_limited() {
        let n = |d: usize| format!("{}{}", "<a>".repeat(d), "</a>".repeat(d));
        assert!(parse(&n(MAX_DEPTH)).is_ok());
        assert!(parse(&n(MAX_DEPTH + 1)).is_err());
    }

    #[test]
    fn picture_by_name() {
        let mut x = refs();
        x.full.insert("g".into(), "CommonPicture.Знак".into());
        let pic = |name: &str| {
            let sch = EMPTY_TAIL.replace("schema\"/>", &format!("schema\"><p xmlns:v8ui=\"http://v8.1c.ru/8.1/data/ui\" xsi:type=\"v8ui:Picture\" ref=\"v8ui:{name}\"/></dataCompositionSchema>"));
            let b = entry(&[&sch, "<Settings xmlns=\"http://v8.1c.ru/8.1/data-composition-system/settings\"/>", EMPTY_TAIL]);
            String::from_utf8(schema(&split(&b).unwrap(), &x, false).unwrap().0).unwrap()
        };
        assert!(pic("Знак").contains("<d2p1:Ref>CommonPicture.Знак</d2p1:Ref>"), "{}", pic("Знак"));
        assert!(pic("Print").contains("<d2p1:Ref>StdPicture.Print</d2p1:Ref>"));
        assert!(pic("EventLog").contains("<d2p1:Ref>StdPicture.EventLog</d2p1:Ref>"));
        assert!(pic("Неведомая").contains("<p xsi:type=\"v8ui:Picture\" ref=\"v8ui:Неведомая\"/>"), "{}", pic("Неведомая"));
        // встроенная PNG — файлом value.png; данные без известной сигнатуры — с меткой незнакомого значения
        let one = |t: &str| {
            let sch = EMPTY_TAIL.replace("schema\"/>", &format!("schema\"><p xmlns:v8ui=\"http://v8.1c.ru/8.1/data/ui\" xsi:type=\"v8ui:Picture\">{t}</p></dataCompositionSchema>"));
            let b = entry(&[&sch, "<Settings xmlns=\"http://v8.1c.ru/8.1/data-composition-system/settings\"/>", EMPTY_TAIL]);
            let (x, pic) = schema(&split(&b).unwrap(), &x, false).unwrap();
            (String::from_utf8(x).unwrap(), pic)
        };
        let (s, pic) = one("iVBORw0KGgo=");
        assert!(s.contains("<d2p1:Abs>value.png</d2p1:Abs>") && pic.is_some(), "{s}");
        assert!(one("AAAAAA==").0.contains(">\u{1}AAAAAA==<"));
    }

    #[test]
    fn style_items() {
        let mut x = refs();
        x.full.insert("g1".into(), "StyleItem.Шрифт".into());
        x.full.insert("g2".into(), "StyleItem.Цвет".into());
        let ui = "xmlns:v8ui=\"http://v8.1c.ru/8.1/data/ui\"";
        let sch = EMPTY_TAIL.replace("schema\"/>", &format!("schema\"><f {ui} xsi:type=\"v8ui:Font\" ref=\"0:g1\" kind=\"StyleItem\"/><c {ui} xsi:type=\"v8ui:Color\">0:g2</c></dataCompositionSchema>"));
        let b = entry(&[&sch, "<Settings xmlns=\"http://v8.1c.ru/8.1/data-composition-system/settings\"/>", EMPTY_TAIL]);
        let out = String::from_utf8(schema(&split(&b).unwrap(), &x, false).unwrap().0).unwrap();
        assert!(out.contains("	<f xmlns:style=\"http://v8.1c.ru/8.1/data/ui/style\" xsi:type=\"v8ui:Font\" ref=\"style:Шрифт\" kind=\"StyleItem\"/>"), "{out}");
        assert!(out.contains("	<c xmlns:d2p1=\"http://v8.1c.ru/8.1/data/ui/style\" xsi:type=\"v8ui:Color\">d2p1:Цвет</c>"), "{out}");
    }

    #[test]
    fn qnames() {
        assert_eq!(split_qname("xs:string"), Some((Some("xs"), "string")));
        assert_eq!(split_qname("0:abc"), None);
        assert_eq!(split_qname("a b"), None);
        assert!(generated("d10p2") && !generated("dcsat") && !generated("dp1"));
    }
}
