//! Настройки СКД внутри записи формы: XML со своими префиксами (`d5p1:`, пространство по умолчанию) →
//! фрагмент Form.xml с префиксами платформы (`dcsset:`, `v8:`...) и отступами по месту.

/// Элемент XML без потерь: имя как в источнике, атрибуты, дети.
#[derive(Debug, Clone)]
pub struct El {
    pub name: String,
    pub attrs: Vec<(String, String)>,
    pub kids: Vec<Kid>,
}

#[derive(Debug, Clone)]
pub enum Kid {
    El(El),
    Text(String),
}

impl El {
    pub fn local(&self) -> &str {
        self.name.rsplit(':').next().unwrap_or(&self.name)
    }
    pub fn elements(&self) -> impl Iterator<Item = &El> {
        self.kids.iter().filter_map(|k| if let Kid::El(e) = k { Some(e) } else { None })
    }
    pub fn attr(&self, name: &str) -> Option<&str> {
        self.attrs.iter().find(|(k, _)| k == name).map(|(_, v)| v.as_str())
    }
}

/// Разбор документа (BOM, `<?xml?>` и комментарии пропускаются); текст остается экранированным, как в источнике.
pub fn parse(x: &str) -> Result<El, String> {
    let mut p = Parser { s: x.trim_start_matches('\u{feff}'), i: 0, depth: 0 };
    p.misc();
    let e = p.element()?;
    p.misc();
    if p.i < p.s.len() {
        return Err(format!("xml: лишнее после корня на {}", p.i));
    }
    Ok(e)
}

/// Предел вложенности, как в скобочном формате: испорченная запись не должна переполнить стек.
const MAX_DEPTH: usize = 256;

struct Parser<'a> {
    s: &'a str,
    i: usize,
    depth: usize,
}

impl Parser<'_> {
    fn rest(&self) -> &str {
        &self.s[self.i..]
    }
    fn skip_ws(&mut self) {
        let r = self.rest();
        self.i += r.len() - r.trim_start().len();
    }
    /// Пробелы, объявление, комментарии.
    fn misc(&mut self) {
        loop {
            self.skip_ws();
            let end = if self.rest().starts_with("<?") {
                "?>"
            } else if self.rest().starts_with("<!--") {
                "-->"
            } else {
                return;
            };
            match self.rest().find(end) {
                Some(k) => self.i += k + end.len(),
                None => self.i = self.s.len(),
            }
        }
    }
    fn name(&mut self) -> String {
        let r = self.rest();
        let n = r.find(|c: char| c.is_whitespace() || matches!(c, '>' | '/' | '=')).unwrap_or(r.len());
        let name = r[..n].to_string();
        self.i += n;
        name
    }
    fn element(&mut self) -> Result<El, String> {
        if !self.rest().starts_with('<') {
            return Err(format!("xml: ждали элемент на {}", self.i));
        }
        self.i += 1;
        let name = self.name();
        let mut attrs = Vec::new();
        loop {
            self.skip_ws();
            let r = self.rest();
            if r.starts_with("/>") {
                self.i += 2;
                return Ok(El { name, attrs, kids: Vec::new() });
            }
            if r.starts_with('>') {
                self.i += 1;
                break;
            }
            let k = self.name();
            if k.is_empty() || !self.rest().starts_with('=') {
                return Err(format!("xml: плохой атрибут на {}", self.i));
            }
            self.i += 1;
            let q = self.rest().chars().next().filter(|c| *c == '"' || *c == '\'').ok_or("xml: нет кавычки")?;
            self.i += 1;
            let end = self.rest().find(q).ok_or("xml: нет закрывающей кавычки")?;
            attrs.push((k, self.rest()[..end].to_string()));
            self.i += end + 1;
        }
        let mut kids = Vec::new();
        loop {
            let r = self.rest();
            if r.starts_with("</") {
                self.i += 2;
                let close = self.name();
                if close != name {
                    return Err(format!("xml: </{close}> вместо </{name}>"));
                }
                self.skip_ws();
                if !self.rest().starts_with('>') {
                    return Err("xml: нет > у закрывающего тега".into());
                }
                self.i += 1;
                return Ok(El { name, attrs, kids });
            }
            if r.starts_with("<!--") {
                self.misc();
                continue;
            }
            if r.starts_with('<') {
                if self.depth >= MAX_DEPTH {
                    return Err(format!("xml: вложенность больше {MAX_DEPTH}"));
                }
                self.depth += 1;
                let k = self.element();
                self.depth -= 1;
                kids.push(Kid::El(k?));
                continue;
            }
            let n = r.find('<').ok_or_else(|| format!("xml: не закрыт <{name}>"))?;
            kids.push(Kid::Text(r[..n].to_string()));
            self.i += n;
        }
    }
}

/// Пространства имен с префиксами из заголовка Form.xml; пространство формы — без префикса.
const NS: &[(&str, &str)] = &[
    ("http://v8.1c.ru/8.3/xcf/logform", ""),
    ("http://v8.1c.ru/8.2/managed-application/core", "app"),
    ("http://v8.1c.ru/8.1/data/enterprise/current-config", "cfg"),
    ("http://v8.1c.ru/8.1/data-composition-system/core", "dcscor"),
    ("http://v8.1c.ru/8.1/data-composition-system/schema", "dcssch"),
    ("http://v8.1c.ru/8.1/data-composition-system/settings", "dcsset"),
    ("http://v8.1c.ru/8.1/data/enterprise", "ent"),
    ("http://v8.1c.ru/8.2/managed-application/logform", "lf"),
    ("http://v8.1c.ru/8.1/data/ui/colors/palette", "pal"),
    ("http://v8.1c.ru/8.1/data/ui/style", "style"),
    ("http://v8.1c.ru/8.1/data/ui/fonts/system", "sys"),
    ("http://v8.1c.ru/8.1/data/core", "v8"),
    ("http://v8.1c.ru/8.1/data/ui", "v8ui"),
    ("http://v8.1c.ru/8.1/data/ui/colors/web", "web"),
    ("http://v8.1c.ru/8.1/data/ui/colors/windows", "win"),
    ("http://v8.1c.ru/8.3/xcf/readable", "xr"),
    ("http://www.w3.org/2001/XMLSchema", "xs"),
    ("http://www.w3.org/2001/XMLSchema-instance", "xsi"),
];

const V8: &str = "http://v8.1c.ru/8.1/data/core";
const V8UI: &str = "http://v8.1c.ru/8.1/data/ui";

/// Имя по GUID: элемент стиля → `style:Имя`, тип конфигурации → `cfg:CatalogRef.Имя`.
pub type Style<'a> = &'a dyn Fn(&str) -> Option<String>;

/// Область видимости: объявления источника (префикс → URI) и выведенные свои (URI → префикс).
#[derive(Clone, Default)]
struct Scope {
    src: Vec<(String, String)>,
    out: Vec<(String, String)>,
    /// Внутри оформления (`appearance`): имена параметров платформа пишет по-русски.
    appearance: bool,
}

impl Scope {
    fn uri(&self, prefix: &str) -> Option<&str> {
        self.src.iter().rev().find(|(p, _)| p == prefix).map(|(_, u)| u.as_str())
    }
}

/// Корень с детьми на глубине `depth` (число табуляций); `rename` — имя корня в выводе (в пространстве корня).
pub fn write_root(out: &mut String, e: &El, rename: Option<&str>, depth: usize, style: Style) {
    write_in(out, None, e, rename, depth, style);
}

/// То же для ребенка `e` элемента `parent` (объявления пространств берутся у родителя).
pub fn write_in(out: &mut String, parent: Option<&El>, e: &El, rename: Option<&str>, depth: usize, style: Style) {
    let mut sc = Scope::default();
    if let Some(p) = parent {
        declare(&mut sc, p);
    }
    let mut e = e.clone();
    if let Some(r) = rename {
        e.name = match e.name.split_once(':') {
            Some((p, _)) => format!("{p}:{r}"),
            None => r.to_string(),
        };
    }
    write(out, &e, depth, &sc, style);
}

fn declare(sc: &mut Scope, e: &El) {
    for (k, v) in &e.attrs {
        if k == "xmlns" {
            sc.src.push((String::new(), v.clone()));
        } else if let Some(p) = k.strip_prefix("xmlns:") {
            sc.src.push((p.to_string(), v.clone()));
        }
    }
}

/// Имя `p:local` источника → имя вывода; незнакомое пространство объявляется на элементе как `d<глубина>p<N>`.
fn qname(sc: &mut Scope, decl: &mut Vec<(String, String)>, q: &str, depth: usize, default_ns: bool) -> String {
    let (p, local) = q.split_once(':').unwrap_or(("", q));
    let uri = if p.is_empty() && !default_ns { Some("") } else { sc.uri(p) };
    let Some(uri) = uri.map(str::to_string) else { return format!("\u{1}ns:{q}") };
    if uri.is_empty() {
        return local.to_string();
    }
    if let Some((_, pre)) = NS.iter().find(|(u, _)| *u == uri) {
        return if pre.is_empty() { local.to_string() } else { format!("{pre}:{local}") };
    }
    if let Some((_, pre)) = sc.out.iter().rev().find(|(u, _)| *u == uri) {
        return format!("{pre}:{local}");
    }
    let pre = format!("d{}p{}", depth + 1, decl.len() + 1);
    decl.push((pre.clone(), uri.clone()));
    sc.out.push((uri, pre.clone()));
    format!("{pre}:{local}")
}

fn write(out: &mut String, e: &El, depth: usize, parent: &Scope, style: Style) {
    let mut sc = parent.clone();
    declare(&mut sc, e);
    if e.name.rsplit(':').next() == Some("appearance") {
        sc.appearance = true;
    }
    let mut decl = Vec::new();
    // Лист без префикса со своим `xmlns="uri"` незнакомого пространства платформа так и пишет — без префикса
    // (`<expression xmlns="…/common">` в orderExpression вычисляемого поля, ОперацииСПодключаемымОборудованием).
    let own_default = e.attrs.iter().find(|(k, _)| k == "xmlns").map(|(_, u)| u.as_str()).filter(|u| !e.name.contains(':') && e.elements().next().is_none() && !NS.iter().any(|(n, _)| n == u));
    let name = match own_default {
        Some(_) => e.name.clone(),
        None => qname(&mut sc, &mut decl, &e.name, depth, true),
    };
    let mut attrs = String::new();
    let mut xtype = None;
    // Шрифт стиля `ref="0:guid"` платформа пишет именем элемента стиля, как цвет (МЧД003.ФормаПросмотра, 3 формы БП).
    let font = e.attrs.iter().any(|(k, v)| k.ends_with(":type") && v.rsplit_once(':').is_some_and(|(p, l)| l == "Font" && sc.uri(p) == Some(V8UI)));
    for (k, v) in &e.attrs {
        if k == "xmlns" || k.starts_with("xmlns:") {
            continue;
        }
        let kn = if k.contains(':') { qname(&mut sc, &mut decl, k, depth, true) } else { k.clone() };
        let val = if kn == "xsi:type" {
            let t = qname(&mut sc, &mut decl, v, depth, true);
            xtype = Some(t.clone());
            t
        } else if font && kn == "ref" && v.starts_with("0:") {
            color(v, style)
        } else {
            v.clone()
        };
        attrs += &format!(" {kn}=\"{val}\"");
    }
    let mut elems: Vec<&El> = e.elements().collect();
    // Пустой вариант многоязычной строки платформа не пишет (БП 3.0.107 ВводДанныхДляРасчетаСреднегоЗаработкаОбщий:
    // `<dcscor:value xsi:type="v8:LocalStringType"/>`), как и у схем компоновки (`dcs`).
    if xtype.as_deref() == Some("v8:LocalStringType") {
        let blank = |c: &El| c.kids.iter().all(|k| matches!(k, Kid::Text(t) if t.is_empty()));
        elems.retain(|i| !(i.local() == "item" && i.elements().any(|c| c.local() == "content" && blank(c))));
    }
    let mut text: String = e.kids.iter().filter_map(|k| if let Kid::Text(t) = k { Some(t.as_str()) } else { None }).collect();
    // без вариантов — пустой элемент, отступы источника не в счет
    if elems.is_empty() && xtype.as_deref() == Some("v8:LocalStringType") && text.trim().is_empty() {
        text.clear();
    }
    let mut body = None;
    let mut name = name;
    if elems.is_empty() && !text.is_empty() {
        let t = text.replace("\r\n", "\n");
        let (p, local) = e.name.split_once(':').unwrap_or(("", &e.name));
        // Значение-тип: элемент v8:Type или xsi:type="v8:Type" (`<dcssch:value xsi:type="v8:Type">d3p2:Undefined`).
        let is_type = (local == "Type" && sc.uri(p) == Some(V8)) || xtype.as_deref() == Some("v8:Type");
        // Тип по GUID платформа выгружает именем; определяемый тип (набор) — остается GUID.
        let type_id = (local == "TypeId" && sc.uri(p) == Some(V8))
            .then(|| style(&t))
            .flatten()
            .filter(|n| n.starts_with("cfg:") && !n.starts_with("cfg:DefinedType."));
        // «Любая ссылка» класса по GUID — набор типов (`cfg:DocumentRef`; ЗадачаИсполнителя.ЗадачиПоПредметуБП).
        let any_ref = (local == "TypeId" && sc.uri(p) == Some(V8) && type_id.is_none())
            .then(|| super::common::platform_type(&t).filter(|(el, _)| *el == "v8:TypeSet").map(|x| x.1))
            .flatten();
        body = Some(if let Some(n) = type_id {
            // характеристика плана видов характеристик — набор типов (ВнутреннееПотребление.ФормаВыбора ERP)
            name = name.replace("TypeId", if n.starts_with("cfg:Characteristic.") { "TypeSet" } else { "Type" });
            n
        } else if let Some(n) = any_ref {
            name = name.replace("TypeId", "TypeSet");
            n.to_string()
        } else if is_type {
            qname(&mut sc, &mut decl, &t, depth, true)
        } else if sc.appearance && local == "parameter" && t.is_ascii() {
            // Параметр оформления латиницей (ЗаявлениеАбонентаСпецоператораСвязи: `TextColor`) — по-русски; в эталоне
            // латинских нет ни одного (636 ЦветТекста), прочие латинские не встречались — отказ.
            match t.as_str() {
                "TextColor" => "ЦветТекста".to_string(),
                _ => format!("\u{1}параметр оформления:{t}"),
            }
        } else if xtype.as_deref() == Some("v8ui:Color") {
            // Префикс источника (`d5p2:` = пространство стилей) → префикс платформы.
            let t = match t.split_once(':').and_then(|(p, l)| Some((NS.iter().find(|(u, _)| Some(*u) == sc.uri(p))?.1, l))) {
                Some((pre, l)) => format!("{pre}:{l}"),
                None => t,
            };
            color(&t, style)
        } else {
            t
        });
    }
    let decls: String = own_default.map(|u| format!(" xmlns=\"{u}\"")).into_iter().chain(decl.iter().map(|(p, u)| format!(" xmlns:{p}=\"{u}\""))).collect();
    out.push_str("\r\n");
    out.push_str(&"\t".repeat(depth));
    match body {
        Some(t) => out.push_str(&format!("<{name}{decls}{attrs}>{t}</{name}>")),
        None if elems.is_empty() => out.push_str(&format!("<{name}{decls}{attrs}/>")),
        None => {
            out.push_str(&format!("<{name}{decls}{attrs}>"));
            if !text.trim().is_empty() {
                out.push_str("\u{1}смешанный текст");
            }
            for k in elems {
                write(out, k, depth + 1, &sc, style);
            }
            out.push_str("\r\n");
            out.push_str(&"\t".repeat(depth));
            out.push_str(&format!("</{name}>"));
        }
    }
}

/// Цвет СКД: `0:<guid>` — элемент стиля; `#RRGGBB`, `auto`, `web:`, `win:`, `style:` — как есть.
fn color(t: &str, style: Style) -> String {
    match t.split_once(':') {
        Some(("0", g)) => match style(g) {
            Some(s) if s.starts_with("style:") => s,
            // Элемента стиля нет в конфигурации — как в записи (Банки каркаса: `0:guid` у ЦветТекста, 3 формы).
            None if crate::brace::is_guid(g) => t.to_string(),
            _ => format!("\u{1}style:{g}"),
        },
        Some(("web" | "win" | "style", _)) => t.to_string(),
        None if t == "auto" || t.starts_with('#') => t.to_string(),
        _ => format!("\u{1}цвет:{t}"),
    }
}

/// Состояние динамического списка: `41 C1`, куски `9A <len8>` / `9B <len16 LE>`, конец `20 20`.
pub fn server_state(b: &[u8]) -> Result<String, String> {
    if !b.starts_with(&[0x41, 0xc1]) {
        return Err("ServerState: нет заголовка 41 C1".into());
    }
    let mut i = 2;
    let mut out = Vec::new();
    while i < b.len() {
        let (n, h) = match b[i] {
            0x9a => (*b.get(i + 1).ok_or("ServerState: обрыв")? as usize, 2),
            0x9b => (u16::from_le_bytes([*b.get(i + 1).ok_or("ServerState: обрыв")?, *b.get(i + 2).ok_or("ServerState: обрыв")?]) as usize, 3),
            0x20 if b[i..] == [0x20, 0x20] => break,
            c => return Err(format!("ServerState: метка {c:02x} на {i}")),
        };
        i += h;
        out.extend_from_slice(b.get(i..i + n).ok_or("ServerState: кусок за концом")?);
        i += n;
    }
    String::from_utf8(out).map_err(|e| format!("ServerState: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn depth_limited() {
        let deep = |n: usize| format!("{}{}", "<a>".repeat(n), "</a>".repeat(n));
        assert!(parse(&deep(MAX_DEPTH)).is_ok());
        assert!(parse(&deep(MAX_DEPTH + 2)).is_err());
    }

    #[test]
    fn prefixes_and_indent() {
        let x = "\u{feff}<?xml version=\"1.0\"?>\r\n<Filter xmlns=\"http://v8.1c.ru/8.1/data-composition-system/settings\" xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\">\r\n\t<item xsi:type=\"FilterItemComparison\">\r\n\t\t<left xmlns:dcscor=\"http://v8.1c.ru/8.1/data-composition-system/core\" xsi:type=\"dcscor:Field\">А</left>\r\n\t\t<right xmlns:d5p1=\"http://v8.1c.ru/8.1/data/ui\" xsi:type=\"d5p1:Color\">0:g1</right>\r\n\t\t<right xmlns:d5p1=\"http://v8.1c.ru/8.1/data/ui\" xmlns:d5p2=\"http://v8.1c.ru/8.1/data/ui/style\" xsi:type=\"d5p1:Color\">d5p2:Ф</right>\r\n\t\t<v xmlns:d5p1=\"http://x\" xsi:type=\"d5p1:T\"/>\r\n\t</item>\r\n</Filter>";
        let e = parse(x).unwrap();
        let mut s = String::new();
        let st = |g: &str| (g == "g1").then(|| "style:Ц".to_string());
        write_root(&mut s, &e, Some("filter"), 1, &st);
        assert_eq!(
            s,
            "\r\n\t<dcsset:filter>\r\n\t\t<dcsset:item xsi:type=\"dcsset:FilterItemComparison\">\r\n\t\t\t<dcsset:left xsi:type=\"dcscor:Field\">А</dcsset:left>\r\n\t\t\t<dcsset:right xsi:type=\"v8ui:Color\">style:Ц</dcsset:right>\r\n\t\t\t<dcsset:right xsi:type=\"v8ui:Color\">style:Ф</dcsset:right>\r\n\t\t\t<dcsset:v xmlns:d4p1=\"http://x\" xsi:type=\"d4p1:T\"/>\r\n\t\t</dcsset:item>\r\n\t</dcsset:filter>"
        );
    }

    #[test]
    fn server_state_chunks() {
        let mut b = vec![0x41, 0xc1, 0x9a, 2, b'<', b'a'];
        b.extend([0x9b, 2, 0, b'/', b'>', 0x20, 0x20]);
        assert_eq!(server_state(&b).unwrap(), "<a/>");
        assert!(server_state(&[0x41, 0xc1, 0x77]).is_err());
    }
}
