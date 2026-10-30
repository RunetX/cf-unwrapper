//! Разбор XML выгрузки в дерево с учетом прочитанного: загрузка обязана разобрать каждый элемент и атрибут,
//! [`El::unread`] находит первый, к которому никто не обращался, — такой файл не загружается (без потерь или отказ).
//!
//! Поддержано то, что пишет платформа: объявление `<?xml?>`, BOM, элементы, атрибуты, текст, сущности
//! `&amp; &lt; &gt; &quot; &apos; &#N; &#xN;`, CDATA. Комментарии, DOCTYPE, инструкции обработки — ошибка.
//! Текст — как в файле, без замены CRLF на LF: платформа пишет CRLF скобочной строки как LF, а `\r\r\n` — как CRLF,
//! и различие нужно сохранить.

use std::cell::Cell;

#[derive(Debug, Clone)]
pub struct El {
    pub name: String,
    pub attrs: Vec<(String, String)>,
    pub kids: Vec<Kid>,
    pub line: usize,
    used: Cell<bool>,
    used_attrs: Vec<Cell<bool>>,
}

#[derive(Debug, Clone)]
pub enum Kid {
    El(El),
    Text(String),
}

impl El {
    /// Новый элемент (правка дерева формы); без пробелов между детьми — загрузка их не читает.
    pub fn new(name: &str, attrs: &[(&str, &str)], kids: Vec<Kid>) -> El {
        let n = attrs.len();
        El { name: name.into(), attrs: attrs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect(), kids, line: 0, used: Cell::new(false), used_attrs: (0..n).map(|_| Cell::new(false)).collect() }
    }

    /// Элемент с текстом: `<name>text</name>`.
    pub fn leaf(name: &str, text: &str) -> El {
        El::new(name, &[], if text.is_empty() { Vec::new() } else { vec![Kid::Text(text.into())] })
    }

    /// Дочерние элементы для правки.
    pub fn elems_mut(&mut self) -> impl Iterator<Item = &mut El> {
        self.kids.iter_mut().filter_map(|k| match k {
            Kid::El(e) => Some(e),
            _ => None,
        })
    }

    /// Отметить элемент прочитанным (без детей).
    pub fn touch(&self) -> &El {
        self.used.set(true);
        self
    }

    /// Дочерние элементы по порядку (текст между ними — только пробелы, его проверяет [`El::unread`]).
    pub fn elems(&self) -> impl Iterator<Item = &El> {
        self.touch();
        self.kids.iter().filter_map(|k| match k {
            Kid::El(e) => Some(e),
            _ => None,
        })
    }

    /// Первый дочерний элемент с этим именем; отмечается прочитанным.
    pub fn child(&self, name: &str) -> Option<&El> {
        self.elems().find(|e| e.name == name).map(El::touch)
    }

    pub fn req(&self, name: &str) -> Result<&El, String> {
        self.child(name).ok_or_else(|| format!("строка {}: в <{}> нет <{name}>", self.line, self.name))
    }

    /// Все дочерние элементы с этим именем.
    pub fn all<'a>(&'a self, name: &'a str) -> impl Iterator<Item = &'a El> + 'a {
        self.elems().filter(move |e| e.name == name).map(El::touch)
    }

    pub fn attr(&self, name: &str) -> Option<&str> {
        let i = self.attrs.iter().position(|(k, _)| k == name)?;
        self.used_attrs[i].set(true);
        Some(&self.attrs[i].1)
    }

    pub fn req_attr(&self, name: &str) -> Result<&str, String> {
        self.attr(name).ok_or_else(|| format!("строка {}: у <{}> нет атрибута {name}", self.line, self.name))
    }

    /// Текст элемента без дочерних элементов (их наличие — ошибка); элемент отмечается прочитанным.
    pub fn text(&self) -> Result<String, String> {
        self.touch();
        let mut s = String::new();
        for k in &self.kids {
            match k {
                Kid::Text(t) => s.push_str(t),
                Kid::El(e) => return Err(format!("строка {}: в <{}> вместо текста элемент <{}>", e.line, self.name, e.name)),
            }
        }
        Ok(s)
    }

    /// Текст дочернего элемента; нет элемента — ошибка.
    pub fn text_of(&self, name: &str) -> Result<String, String> {
        self.req(name)?.text()
    }

    /// Первый непрочитанный элемент или атрибут (путь от этого элемента); `None` — прочитано все.
    pub fn unread(&self) -> Option<String> {
        if !self.used.get() {
            return Some(format!("<{}> (строка {})", self.name, self.line));
        }
        if let Some(i) = self.used_attrs.iter().position(|u| !u.get()) {
            return Some(format!("атрибут {} у <{}> (строка {})", self.attrs[i].0, self.name, self.line));
        }
        let has_el = self.kids.iter().any(|k| matches!(k, Kid::El(_)));
        for k in &self.kids {
            match k {
                Kid::El(e) => {
                    if let Some(u) = e.unread() {
                        return Some(format!("{}/{u}", self.name));
                    }
                }
                Kid::Text(t) if has_el && !t.chars().all(|c| matches!(c, ' ' | '\t' | '\r' | '\n')) => {
                    return Some(format!("текст {:?} в <{}> (строка {})", t.chars().take(40).collect::<String>(), self.name, self.line));
                }
                _ => {}
            }
        }
        None
    }

    /// Просмотр без отметки прочитанным (предварительный обход: что прочитано, решает потом сама загрузка элемента).
    pub fn peek_attr(&self, name: &str) -> Option<&str> {
        self.attrs.iter().find(|(k, _)| k == name).map(|(_, v)| v.as_str())
    }

    pub fn peek_elems(&self) -> impl Iterator<Item = &El> {
        self.kids.iter().filter_map(|k| match k {
            Kid::El(e) => Some(e),
            _ => None,
        })
    }

    pub fn peek_child(&self, name: &str) -> Option<&El> {
        self.peek_elems().find(|e| e.name == name)
    }

    /// Текст без отметки; есть дочерние элементы — `None`.
    pub fn peek_text(&self) -> Option<String> {
        let mut s = String::new();
        for k in &self.kids {
            match k {
                Kid::Text(t) => s.push_str(t),
                Kid::El(_) => return None,
            }
        }
        Some(s)
    }

    /// Есть ли на любой глубине элемент с этим именем (без отметки).
    pub fn peek_has(&self, name: &str) -> bool {
        self.peek_elems().any(|e| e.name == name || e.peek_has(name))
    }

    /// Документ обратно в текст, как пишет выгрузка: BOM, объявление, CRLF; пробелы между элементами — из дерева,
    /// экранируются `& < >` (в атрибутах и `"`). Для выгрузки платформы `parse` → `write` дает тот же файл байт в байт.
    pub fn write(&self) -> String {
        let mut o = String::from("\u{feff}<?xml version=\"1.0\" encoding=\"UTF-8\"?>\r\n");
        self.write_to(&mut o);
        o
    }

    fn write_to(&self, o: &mut String) {
        o.push('<');
        o.push_str(&self.name);
        for (k, v) in &self.attrs {
            o.push(' ');
            o.push_str(k);
            o.push_str("=\"");
            escape(v, true, o);
            o.push('"');
        }
        if self.kids.is_empty() {
            o.push_str("/>");
            return;
        }
        o.push('>');
        for k in &self.kids {
            match k {
                Kid::El(e) => e.write_to(o),
                Kid::Text(t) => escape(t, false, o),
            }
        }
        o.push_str("</");
        o.push_str(&self.name);
        o.push('>');
    }

    /// Отметить все поддерево прочитанным (содержимое разобрано целиком другим способом).
    pub fn touch_all(&self) {
        self.used.set(true);
        self.used_attrs.iter().for_each(|u| u.set(true));
        for k in &self.kids {
            if let Kid::El(e) = k {
                e.touch_all();
            }
        }
    }
}

/// Корневой элемент документа.
pub fn parse(src: &str) -> Result<El, String> {
    let s = src.strip_prefix('\u{feff}').unwrap_or(src);
    let mut p = P { s, i: 0, line: 1 };
    if p.rest().starts_with("<?xml") {
        let end = p.rest().find("?>").ok_or("незакрытое объявление XML")?;
        p.adv(end + 2);
    }
    p.ws();
    let root = p.element(0)?;
    p.ws();
    if p.i != p.s.len() {
        return Err(format!("строка {}: лишний текст после корневого элемента", p.line));
    }
    Ok(root)
}

const MAX_DEPTH: usize = 512;

struct P<'a> {
    s: &'a str,
    i: usize,
    line: usize,
}

impl P<'_> {
    fn rest(&self) -> &str {
        &self.s[self.i..]
    }
    fn adv(&mut self, n: usize) {
        self.line += self.s[self.i..self.i + n].matches('\n').count();
        self.i += n;
    }
    fn ws(&mut self) {
        let n = self.rest().len() - self.rest().trim_start_matches([' ', '\t', '\r', '\n']).len();
        self.adv(n);
    }
    fn err<T>(&self, m: &str) -> Result<T, String> {
        Err(format!("строка {}: {m}", self.line))
    }
    fn name(&mut self) -> Result<String, String> {
        let n = self.rest().find(|c: char| matches!(c, ' ' | '\t' | '\r' | '\n' | '/' | '>' | '=')).unwrap_or(self.rest().len());
        if n == 0 {
            return self.err("ожидалось имя");
        }
        let name = self.rest()[..n].to_string();
        self.adv(n);
        Ok(name)
    }

    fn element(&mut self, depth: usize) -> Result<El, String> {
        if depth > MAX_DEPTH {
            return self.err("слишком глубокая вложенность");
        }
        if !self.rest().starts_with('<') || self.rest().starts_with("<!") || self.rest().starts_with("<?") {
            return self.err("ожидался элемент (комментарии, DOCTYPE и инструкции не поддержаны)");
        }
        let line = self.line;
        self.adv(1);
        let name = self.name()?;
        let mut attrs = Vec::new();
        loop {
            self.ws();
            if self.rest().starts_with("/>") {
                self.adv(2);
                let n = attrs.len();
                return Ok(El { name, attrs, kids: Vec::new(), line, used: Cell::new(false), used_attrs: (0..n).map(|_| Cell::new(false)).collect() });
            }
            if self.rest().starts_with('>') {
                self.adv(1);
                break;
            }
            let k = self.name()?;
            self.ws();
            if !self.rest().starts_with('=') {
                return self.err("ожидался =");
            }
            self.adv(1);
            self.ws();
            let q = self.rest().chars().next().filter(|c| *c == '"' || *c == '\'').ok_or(format!("строка {}: значение атрибута без кавычек", self.line))?;
            self.adv(1);
            let end = self.rest().find(q).ok_or(format!("строка {}: незакрытый атрибут", self.line))?;
            let raw = &self.rest()[..end];
            if raw.contains('<') {
                return self.err("< в значении атрибута");
            }
            let v = unescape(raw).map_err(|e| format!("строка {}: {e}", self.line))?;
            self.adv(end + 1);
            if attrs.iter().any(|(a, _): &(String, String)| *a == k) {
                return self.err(&format!("повтор атрибута {k}"));
            }
            attrs.push((k, v));
        }
        let mut kids = Vec::new();
        let mut text = String::new();
        loop {
            if self.i >= self.s.len() {
                return self.err(&format!("незакрытый элемент <{name}>"));
            }
            if self.rest().starts_with("</") {
                self.adv(2);
                let end = self.name()?;
                self.ws();
                if end != name || !self.rest().starts_with('>') {
                    return self.err(&format!("ожидался </{name}>"));
                }
                self.adv(1);
                if !text.is_empty() {
                    kids.push(Kid::Text(std::mem::take(&mut text)));
                }
                let n = attrs.len();
                return Ok(El { name, attrs, kids, line, used: Cell::new(false), used_attrs: (0..n).map(|_| Cell::new(false)).collect() });
            }
            if self.rest().starts_with("<![CDATA[") {
                let end = self.rest().find("]]>").ok_or(format!("строка {}: незакрытый CDATA", self.line))?;
                text.push_str(&self.rest()[9..end]);
                self.adv(end + 3);
                continue;
            }
            if self.rest().starts_with('<') {
                if !text.is_empty() {
                    kids.push(Kid::Text(std::mem::take(&mut text)));
                }
                kids.push(Kid::El(self.element(depth + 1)?));
                continue;
            }
            let n = self.rest().find('<').unwrap_or(self.rest().len());
            let raw = self.rest()[..n].to_string();
            text.push_str(&unescape(&raw).map_err(|e| format!("строка {}: {e}", self.line))?);
            self.adv(n);
        }
    }
}

fn escape(s: &str, attr: bool, o: &mut String) {
    for c in s.chars() {
        match c {
            '&' => o.push_str("&amp;"),
            '<' => o.push_str("&lt;"),
            '>' => o.push_str("&gt;"),
            '"' if attr => o.push_str("&quot;"),
            c => o.push(c),
        }
    }
}

fn unescape(s: &str) -> Result<String, String> {
    if !s.contains('&') {
        return Ok(s.to_string());
    }
    let mut o = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(k) = rest.find('&') {
        o.push_str(&rest[..k]);
        let end = rest[k..].find(';').ok_or("незакрытая сущность")? + k;
        let ent = &rest[k + 1..end];
        let c = match ent {
            "amp" => '&',
            "lt" => '<',
            "gt" => '>',
            "quot" => '"',
            "apos" => '\'',
            _ => {
                let code = if let Some(h) = ent.strip_prefix("#x") {
                    u32::from_str_radix(h, 16).ok()
                } else if let Some(d) = ent.strip_prefix('#') {
                    d.parse().ok()
                } else {
                    None
                };
                code.and_then(char::from_u32).ok_or(format!("неизвестная сущность &{ent};"))?
            }
        };
        o.push(c);
        rest = &rest[end + 1..];
    }
    o.push_str(rest);
    Ok(o)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_tracks_reads() {
        let x = parse("\u{feff}<?xml version=\"1.0\" encoding=\"UTF-8\"?>\r\n<a x=\"1&amp;&quot;\">\r\n\t<b>т&lt;e&#13;\r\nст</b>\r\n\t<c/>\r\n\t<d><![CDATA[<z>]]></d>\r\n</a>").unwrap();
        assert_eq!(x.attr("x"), Some("1&\""));
        assert_eq!(x.text_of("b").unwrap(), "т<e\r\r\nст");
        assert_eq!(x.unread().as_deref(), Some("a/<c> (строка 5)"));
        assert_eq!(x.child("c").unwrap().text().unwrap(), "");
        assert!(x.unread().unwrap().starts_with("a/<d>"));
        assert_eq!(x.text_of("d").unwrap(), "<z>");
        assert_eq!(x.unread(), None);
        assert!(x.text().is_err());
    }

    #[test]
    fn unread_attr_and_text() {
        let x = parse("<a><b y=\"2\"/>junk</a>").unwrap();
        x.child("b");
        assert!(x.unread().unwrap().contains("атрибут y"));
        x.child("b").unwrap().attr("y");
        assert!(x.unread().unwrap().contains("junk"));
    }

    #[test]
    fn writes_back() {
        let src = "\u{feff}<?xml version=\"1.0\" encoding=\"UTF-8\"?>\r\n<a x=\"1&amp;&quot;\">\r\n\t<b>т&lt;e&gt;\nст</b>\r\n\t<c/>\r\n</a>";
        assert_eq!(parse(src).unwrap().write(), src);
    }

    #[test]
    fn errors() {
        for bad in ["<a>", "<a></b>", "<a x=1/>", "<!-- c --><a/>", "<a/><b/>", "<a>&foo;</a>", "<a x=\"1\" x=\"2\"/>"] {
            assert!(parse(bad).is_err(), "{bad}");
        }
    }
}
