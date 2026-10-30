//! Скобочный формат 1С: `{1,"строка",{вложенный},guid}`. Кавычки внутри строки удваиваются,
//! символы вне BMP пишутся суррогатами `"\db40"\dc32`.
//!
//! Раскладка у 1С одна (проверено на всех скобочных записях БП 3.0.171 и ERP 2.6.1): перед каждым вложенным `{`
//! стоит `\r\n`, перед `}` — `\r\n`, если последний элемент списка — список; других пробелов нет.
//! Поэтому [`write`] восстанавливает текст 1С байт в байт, а правленые узлы пишутся как у 1С.
//! Текст в другой раскладке (лишние пробелы, суррогаты в верхнем регистре) читается, но пишется уже в раскладке 1С.

#[derive(Clone, Debug, PartialEq)]
pub enum Node {
    List(Vec<Node>),
    Str(String),
    Atom(String),
}

impl Node {
    pub fn list(&self) -> &[Node] {
        match self {
            Node::List(v) => v,
            _ => &[],
        }
    }
    pub fn get(&self, i: usize) -> Option<&Node> {
        self.list().get(i)
    }
    pub fn atom(&self) -> Option<&str> {
        match self {
            Node::Atom(s) => Some(s),
            _ => None,
        }
    }
    pub fn str(&self) -> Option<&str> {
        match self {
            Node::Str(s) => Some(s),
            _ => None,
        }
    }
    pub fn list_mut(&mut self) -> Option<&mut Vec<Node>> {
        match self {
            Node::List(v) => Some(v),
            _ => None,
        }
    }
    /// Узел по пути индексов; пустой путь — сам узел.
    pub fn at(&self, path: &[usize]) -> Option<&Node> {
        path.iter().try_fold(self, |n, &i| n.get(i))
    }
    pub fn at_mut(&mut self, path: &[usize]) -> Option<&mut Node> {
        path.iter().try_fold(self, |n, &i| n.list_mut()?.get_mut(i))
    }
    /// Заменяет узел по пути, возвращает прежний.
    pub fn set(&mut self, path: &[usize], n: Node) -> Result<Node, String> {
        let at = self.at_mut(path).ok_or_else(|| format!("нет элемента {path:?}"))?;
        Ok(std::mem::replace(at, n))
    }
    /// Вставляет узел так, чтобы он оказался по пути `path` (последний индекс может быть равен длине списка).
    pub fn insert(&mut self, path: &[usize], n: Node) -> Result<(), String> {
        let (v, i) = self.parent(path)?;
        if i > v.len() {
            return Err(format!("нет позиции {path:?}"));
        }
        v.insert(i, n);
        Ok(())
    }
    pub fn remove(&mut self, path: &[usize]) -> Result<Node, String> {
        let (v, i) = self.parent(path)?;
        if i >= v.len() {
            return Err(format!("нет элемента {path:?}"));
        }
        Ok(v.remove(i))
    }
    fn parent(&mut self, path: &[usize]) -> Result<(&mut Vec<Node>, usize), String> {
        let (&i, up) = path.split_last().ok_or("пустой путь")?;
        let v = self.at_mut(up).and_then(Node::list_mut).ok_or_else(|| format!("нет списка {up:?}"))?;
        Ok((v, i))
    }
}

/// Первый список текста. Пустые элементы (`{2,x,}`) отбрасываются — для записи без потерь есть [`Doc`].
pub fn parse(text: &str) -> Result<Node, String> {
    let mut i = text.find('{').ok_or("нет открывающей скобки")?;
    node(text, &mut i, false, 0)
}

/// Запись: BOM и корневые списки через `,\r\n` (у `configinfo` корней несколько).
#[derive(Clone, Debug, PartialEq)]
pub struct Doc {
    pub bom: bool,
    pub roots: Vec<Node>,
    /// Пробелы после последнего корня (у вложенных `info` — `\r\n`).
    tail: String,
}

/// Глубже 1С не вкладывает; без предела рекурсия на битом файле переполняет стек.
const MAX_DEPTH: usize = 256;

impl Doc {
    pub fn new(bom: bool, roots: Vec<Node>) -> Doc {
        Doc { bom, roots, tail: String::new() }
    }

    pub fn parse(text: &str) -> Result<Doc, String> {
        let (bom, t) = match text.strip_prefix('\u{feff}') {
            Some(t) => (true, t),
            None => (false, text),
        };
        if !t.starts_with('{') {
            return Err("нет открывающей скобки".into());
        }
        let mut d = Doc::new(bom, Vec::new());
        let mut i = 0;
        loop {
            d.roots.push(node(t, &mut i, true, 0)?);
            if t[i..].starts_with(",\r\n{") {
                i += 3;
            } else if t[i..].trim_start_matches([' ', '\r', '\n', '\t']).is_empty() {
                d.tail = t[i..].to_string();
                return Ok(d);
            } else {
                return Err(format!("лишний текст на позиции {i}"));
            }
        }
    }

    pub fn write(&self) -> Result<String, String> {
        let mut o = String::new();
        if self.bom {
            o.push('\u{feff}');
        }
        for (k, r) in self.roots.iter().enumerate() {
            if !matches!(r, Node::List(_)) {
                return Err(format!("корень {k} — не список"));
            }
            if k > 0 {
                o.push_str(",\r\n");
            }
            put(r, &mut o, 0)?;
        }
        o.push_str(&self.tail);
        Ok(o)
    }
}

/// Текст узла в раскладке 1С. Ошибка — если узел нельзя записать так, чтобы он читался обратно тем же.
pub fn write(n: &Node) -> Result<String, String> {
    let mut o = String::new();
    put(n, &mut o, 0)?;
    Ok(o)
}

fn put(n: &Node, o: &mut String, depth: usize) -> Result<(), String> {
    match n {
        Node::List(_) if depth >= MAX_DEPTH => return Err("слишком глубокая вложенность".into()),
        Node::List(v) if matches!(v[..], [Node::Atom(ref a)] if a.is_empty()) => {
            return Err("список из одного пустого элемента записать нельзя".into())
        }
        Node::List(v) => {
            o.push('{');
            for (k, x) in v.iter().enumerate() {
                if k > 0 {
                    o.push(',');
                }
                if let Node::List(_) = x {
                    o.push_str("\r\n");
                }
                put(x, o, depth + 1)?;
            }
            if let Some(Node::List(_)) = v.last() {
                o.push_str("\r\n");
            }
            o.push('}');
        }
        Node::Str(s) => {
            o.push('"');
            for c in s.chars() {
                match c {
                    '"' => o.push_str("\"\""),
                    c if c > '\u{ffff}' => {
                        let mut u = [0; 2];
                        for x in c.encode_utf16(&mut u) {
                            o.push_str(&format!("\"\\{x:x}"));
                        }
                    }
                    c => o.push(c),
                }
            }
            o.push('"');
        }
        Node::Atom(a) if a.contains([',', '{', '}', '"']) || a.starts_with([' ', '\r', '\n', '\t']) => {
            return Err(format!("недопустимый атом {a:?}"))
        }
        Node::Atom(a) => o.push_str(a),
    }
    Ok(())
}

/// `keep`: сохранять пустые элементы и атомы как есть (иначе — отбрасывать пустые и обрезать пробелы).
fn node(t: &str, i: &mut usize, keep: bool, depth: usize) -> Result<Node, String> {
    if depth >= MAX_DEPTH {
        return Err(format!("слишком глубокая вложенность на позиции {i}"));
    }
    let b = t.as_bytes();
    *i += 1; // '{'
    let mut items = Vec::new();
    // ждём элемент: после '{' и ','
    let mut want = true;
    loop {
        while *i < b.len() && matches!(b[*i], b' ' | b'\r' | b'\n' | b'\t') {
            *i += 1;
        }
        let c = b.get(*i).copied();
        if !want && !matches!(c, Some(b',' | b'}') | None) {
            return Err(format!("нет запятой на позиции {i}"));
        }
        match c {
            None => return Err("незакрытая скобка".into()),
            Some(b'}') => {
                *i += 1;
                if keep && want && !items.is_empty() {
                    items.push(Node::Atom(String::new()));
                }
                return Ok(Node::List(items));
            }
            Some(b',') => {
                if keep && want {
                    items.push(Node::Atom(String::new()));
                }
                *i += 1;
                want = true;
                continue;
            }
            Some(b'{') => items.push(node(t, i, keep, depth + 1)?),
            Some(b'"') => {
                let mut s = String::new();
                *i += 1;
                let mut hi = None;
                loop {
                    let k = t[*i..].find('"').ok_or("незакрытая строка")? + *i;
                    if hi.is_some() && k > *i {
                        return Err(format!("одиночный суррогат на позиции {i}"));
                    }
                    s.push_str(&t[*i..k]);
                    *i = k + 1;
                    // символ вне BMP 1С пишет суррогатами: "\d83d"\de00
                    let unit = (b.get(*i) == Some(&b'\\')).then(|| t.get(*i + 1..*i + 5)).flatten().and_then(|h| u16::from_str_radix(h, 16).ok());
                    match (unit, hi) {
                        (Some(u @ 0xd800..=0xdbff), None) => hi = Some(u),
                        (Some(u @ 0xdc00..=0xdfff), Some(h)) => {
                            hi = None;
                            s.extend(char::decode_utf16([h, u]).flatten());
                        }
                        (Some(_), _) => return Err(format!("неверный суррогат на позиции {i}")),
                        (None, Some(_)) => return Err(format!("одиночный суррогат на позиции {i}")),
                        (None, None) if b.get(*i) == Some(&b'"') => {
                            s.push('"');
                            *i += 1;
                            continue;
                        }
                        (None, None) => break,
                    }
                    *i += 5;
                }
                items.push(Node::Str(s));
            }
            Some(_) => {
                let j = *i;
                *i = t[j..].find([',', '}']).map_or(t.len(), |k| j + k);
                let a = &t[j..*i];
                items.push(Node::Atom(if keep { a } else { a.trim() }.to_string()));
            }
        }
        want = false;
    }
}

pub fn is_guid(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() == 36
        && b.iter().enumerate().all(|(i, &c)| if matches!(i, 8 | 13 | 18 | 23) { c == b'-' } else { c.is_ascii_hexdigit() })
}

/// Текст записи: UTF-8 с BOM или без.
pub fn text(bytes: &[u8]) -> String {
    let b = bytes.strip_prefix(&[0xef, 0xbb, 0xbf]).unwrap_or(bytes);
    String::from_utf8_lossy(b).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn same(t: &str) {
        assert_eq!(Doc::parse(t).unwrap().write().unwrap(), t);
    }

    #[test]
    fn parses_nested_and_quotes() {
        let n = parse("\u{feff}{1,\r\n{3,{1,0,0aa19482-2d94-47dc-bcaf-5e3f2c360e13},\"Имя \"\"в кавычках\"\"\",{1,\"ru\",\"Синоним\"}},{#base64:AAA=}}").unwrap();
        let obj = n.get(1).unwrap();
        assert_eq!(obj.get(1).unwrap().get(2).unwrap().atom(), Some("0aa19482-2d94-47dc-bcaf-5e3f2c360e13"));
        assert_eq!(obj.get(2).unwrap().str(), Some("Имя \"в кавычках\""));
        assert_eq!(n.get(2).unwrap().get(0).unwrap().atom(), Some("#base64:AAA="));
    }

    #[test]
    fn errors_on_unclosed() {
        assert!(parse("{1,{2,").is_err());
        assert!(parse("{\"abc").is_err());
        assert!(Doc::parse("{1,\"a\"\"").is_err());
    }

    #[test]
    fn rejects_json() {
        assert!(Doc::parse("{\r\n\t\"Макет\": \"x\"\r\n}").is_err());
    }

    #[test]
    fn guid_check() {
        assert!(is_guid("cf4abea6-37b2-11d4-940f-008048da11f9"));
        assert!(!is_guid("cf4abea6-37b2-11d4-940f-008048da11fz"));
    }

    #[test]
    fn roundtrip_strings_and_atoms() {
        same("{\"\",\"a\"\"b\"\"\",\"строка\r\nвторая\",-9,0.5,\r\n{#base64:AAEC\r\nAwQ=}\r\n}");
        let t = "{\"Moment\"\\db40\"\\dc32 of\"\"\"\\db40\"\\dc30\"}";
        same(t);
        assert_eq!(parse(t).unwrap().get(0).and_then(Node::str), Some("Moment\u{e0032} of\"\u{e0030}"));
        assert!(Doc::parse("{\"a\"\\db40x\"}").is_err());
        assert!(Doc::parse("{\"a\"\\dc32\"}").is_err());
        same("{2,c5ddcba5-99c0-4ca3-bb3f-d04b743320bd,}");
        same("{,}");
        same("{}");
        let d = Doc::parse("{\"a\"\"b\",,}").unwrap();
        assert_eq!(d.roots[0], Node::List(vec![Node::Str("a\"b".into()), Node::Atom("".into()), Node::Atom("".into())]));
        assert_eq!(parse("{2,x,}").unwrap(), Node::List(vec![Node::Atom("2".into()), Node::Atom("x".into())]));
    }

    #[test]
    fn roundtrip_nested_bom_and_tail() {
        same("\u{feff}{1,\r\n{0,\r\n{0}\r\n},1,\r\n{\r\n{2,\r\n{\"S\",\"\"}\r\n},0}\r\n}");
        same("{\r\n{216,0,\r\n{2,0}\r\n}\r\n}\r\n");
        let t = "{\r\n{1,2}\r\n},\r\n{2,c5ddcba5-99c0-4ca3-bb3f-d04b743320bd,},\r\n{52,\"x\"}";
        let d = Doc::parse(t).unwrap();
        assert_eq!(d.roots.len(), 3);
        assert_eq!(d.write().unwrap(), t);
        assert_eq!(Doc::new(false, d.roots.clone()).write().unwrap(), t);
    }

    #[test]
    fn edits_are_canonical() {
        let mut d = Doc::parse("\u{feff}{1,\r\n{0,\"a\"},5}").unwrap();
        let r = &mut d.roots[0];
        assert_eq!(r.set(&[1, 1], Node::Str("б\"в".into())).unwrap(), Node::Str("a".into()));
        r.insert(&[3], Node::List(vec![Node::Atom("7".into())])).unwrap();
        r.insert(&[1, 0], Node::List(vec![])).unwrap();
        assert_eq!(d.write().unwrap(), "\u{feff}{1,\r\n{\r\n{},0,\"б\"\"в\"},5,\r\n{7}\r\n}");
        let r = &mut d.roots[0];
        assert_eq!(r.remove(&[3]).unwrap(), Node::List(vec![Node::Atom("7".into())]));
        assert_eq!(r.at(&[1, 2]).and_then(Node::str), Some("б\"в"));
        assert!(r.remove(&[9]).is_err());
        assert!(r.insert(&[0, 0], Node::Atom("x".into())).is_err());
        assert!(r.set(&[], Node::Atom("x".into())).is_ok());
        assert_eq!(write(&d.roots[0]).unwrap(), "x");
    }

    fn l(v: &[&str]) -> Node {
        Node::List(v.iter().map(|a| Node::Atom(a.to_string())).collect())
    }

    #[test]
    fn root_edits_keep_separators() {
        let mut d = Doc::parse("{1},\r\n{2}").unwrap();
        d.roots.remove(0);
        assert_eq!(d.write().unwrap(), "{2}");
        let mut d = Doc::parse("{1},\r\n{2}\r\n").unwrap();
        d.roots.push(l(&["3"]));
        assert_eq!(d.write().unwrap(), "{1},\r\n{2},\r\n{3}\r\n");
        d.roots.push(Node::Atom("4".into()));
        assert!(d.write().is_err());
        assert!(Doc::parse("hello").is_err());
        assert!(Doc::parse("{1}garbage{2}").is_err());
        assert!(Doc::parse("\r\n{1}").is_err());
    }

    #[test]
    fn rejects_unwritable_atoms() {
        for a in ["a,b", "a}", "{a", "a\"", " 1", "\r\n1"] {
            assert!(write(&l(&["1", a])).is_err(), "{a:?}");
        }
        assert!(write(&l(&[""])).is_err());
        assert_eq!(write(&l(&["", ""])).unwrap(), "{,}");
    }

    #[test]
    fn deep_nesting_is_error() {
        // 1 МБ — стек главного потока Windows, где разбирает окно
        let ok = std::thread::Builder::new()
            .stack_size(1 << 20)
            .spawn(|| {
                let n = |k: usize| "{".repeat(k) + &"}".repeat(k);
                let deep = (0..MAX_DEPTH).fold(Node::List(vec![]), |n, _| Node::List(vec![n]));
                Doc::parse(&n(MAX_DEPTH)).is_ok()
                    && parse(&n(MAX_DEPTH + 1)).is_err()
                    && Doc::parse(&n(100_000)).is_err()
                    && write(&deep).is_err()
            })
            .unwrap()
            .join()
            .unwrap();
        assert!(ok);
    }
}
