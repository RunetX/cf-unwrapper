//! Понижение формата выгрузки: файл 2.21 (8.5) → старая версия по таблице правил `formats/<версия>.txt`.
//! Таблица — данные, а не код: файл `formats/<версия>.txt` рядом с cfkit.exe перекрывает встроенный.
//! Таблицы — цепочкой: каждая описывает шаг от соседней новой версии (`from 2.20`), 2.21 → 2.20 → 2.17.
//! Правила работают над строками готового XML (выгрузка пишет элемент на строку); значение, которое старый формат
//! не выражает, — ошибка файла, не догадка.

/// Встроенные таблицы: версия формата → текст правил.
const BUILTIN: &[(&str, &str)] = &[
    ("2.20", include_str!("../../formats/2.20.txt")),
    ("2.17", include_str!("../../formats/2.17.txt")),
    ("compat-8.3.16", include_str!("../../formats/compat-8.3.16.txt")),
];

pub struct Format {
    pub version: String,
    /// Версия, от которой шаг этой таблицы; не 2.21 — сначала понижение до нее (`base`).
    from: String,
    base: Option<Box<Format>>,
    rules: Vec<Rule>,
}

struct Rule {
    tag: String,
    /// Родитель элемента: варианты через `|`, каждый — суффикс пути (`EnumValue/Properties`); пусто — любой.
    parents: Vec<Vec<String>>,
    kind: Kind,
}

enum Kind {
    /// Элемента в старом формате нет: перечисленные значения опускаются, прочие — ошибка; без списка — любое.
    Absent(Vec<String>),
    /// Элемента в формате нет, если у него есть ребенок с таким значением: блок опускается целиком.
    AbsentWhere(String, String),
    /// Старый формат не пишет значение по умолчанию; прочие значения остаются.
    Default(String),
    /// Значение переименовано: `a` (новое) → `b` (старое); `*:x` — с любым префиксом пространства имен.
    Value(String, String),
    /// Элемент переименован.
    Rename(String),
    /// Старый формат пишет элемент, которого у новой версии нет: вставить перед первым из `before` (или в конец),
    /// если у родителя нет ни самого элемента, ни одного из `unless`.
    Insert { value: String, unless: Vec<String>, before: Vec<String> },
    /// Атрибут убрать у элемента (`*` — у любого); для `xmlns:p` префикс не должен встречаться в файле.
    AttrDrop(String),
    /// Строка открытия элемента содержит подстроку: в формате это не выразить по тексту 2.21 — ошибка файла.
    Refuse(String),
}

impl Format {
    /// Таблица версии: `formats/<версия>.txt` рядом с исполняемым файлом, иначе встроенная.
    pub fn load(version: &str) -> Result<Format, String> {
        let local = std::env::current_exe().ok().and_then(|e| Some(e.parent()?.join("formats").join(format!("{version}.txt"))));
        let text = match local.as_ref().and_then(|p| std::fs::read_to_string(p).ok()) {
            Some(t) => t,
            None => BUILTIN.iter().find(|(v, _)| *v == version).map(|(_, t)| t.to_string()).ok_or_else(|| {
                let known: Vec<_> = BUILTIN.iter().map(|(v, _)| *v).collect();
                format!("формат {version} не описан (есть: 2.21, {})", known.join(", "))
            })?,
        };
        let mut f = Self::parse(&text)?;
        if f.from != "2.21" {
            if f.from == f.version {
                return Err(format!("формат {version}: from указывает на себя"));
            }
            f.base = Some(Box::new(Format::load(&f.from)?));
        }
        Ok(f)
    }

    pub fn parse(text: &str) -> Result<Format, String> {
        let mut f = Format { version: String::new(), from: "2.21".into(), base: None, rules: Vec::new() };
        for (n, line) in text.lines().enumerate() {
            let line = line.split(" #").next().unwrap_or("").trim();
            if line.starts_with('#') {
                continue;
            }
            if line.is_empty() {
                continue;
            }
            let err = |m: &str| format!("формат, строка {}: {m}: {line}", n + 1);
            let t: Vec<&str> = line.split_whitespace().collect();
            if t[0] == "version" || t[0] == "from" {
                let v = t.get(1).ok_or_else(|| err("нет версии"))?.to_string();
                *(if t[0] == "version" { &mut f.version } else { &mut f.from }) = v;
                continue;
            }
            let tag = t.get(1).ok_or_else(|| err("нет элемента"))?.to_string();
            let mut i = 2;
            let mut parents = Vec::new();
            if t.get(i) == Some(&"in") {
                parents = t.get(i + 1).ok_or_else(|| err("нет родителя"))?.split('|').map(|p| p.split('/').map(String::from).collect()).collect();
                i += 2;
            }
            let rest = &t[i..];
            // `""` — пустое значение (`<a/>`)
            let list = |s: &str| s.split('|').map(|v| if v == "\"\"" { String::new() } else { v.to_string() }).collect::<Vec<_>>();
            let kind = match (t[0], rest) {
                ("absent", []) => Kind::Absent(Vec::new()),
                ("absent", ["=", v]) => Kind::Absent(list(v)),
                ("absent", ["where", c, "=", v]) => Kind::AbsentWhere(c.to_string(), v.to_string()),
                ("default", ["=", v]) => Kind::Default(v.to_string()),
                ("value", ["=", a, ">", b]) => Kind::Value(a.to_string(), b.to_string()),
                ("rename", [">", b]) => Kind::Rename(b.to_string()),
                ("refuse", ["has", v]) => Kind::Refuse(v.to_string()),
                ("attr-drop", [a]) => Kind::AttrDrop(a.to_string()),
                ("insert", ["=", v, more @ ..]) => {
                    let (mut unless, mut before) = (Vec::new(), Vec::new());
                    let mut m = more;
                    while let [k, v, tail @ ..] = m {
                        match *k {
                            "unless" => unless = list(v),
                            "before" => before = list(v),
                            _ => return Err(err("ожидалось unless или before")),
                        }
                        m = tail;
                    }
                    if !m.is_empty() {
                        return Err(err("лишнее в конце"));
                    }
                    Kind::Insert { value: v.to_string(), unless, before }
                }
                _ => return Err(err("непонятное правило")),
            };
            if matches!(kind, Kind::Insert { .. }) && parents.is_empty() {
                return Err(err("вставке нужен родитель (in)"));
            }
            f.rules.push(Rule { tag, parents, kind });
        }
        if f.version.is_empty() {
            return Err("формат: нет строки version".into());
        }
        Ok(f)
    }

    /// Файл выгрузки 2.21 → этот формат; не XML (модули, картинки, справка) — как есть.
    pub fn down(&self, path: &std::path::Path, data: &[u8]) -> Result<Vec<u8>, String> {
        if path.extension().is_none_or(|e| e != "xml") {
            return Ok(data.to_vec());
        }
        match &self.base {
            Some(b) => self.step(&b.down(path, data)?),
            None => self.step(data),
        }
    }

    /// Шаг `from` → `version` по своим правилам.
    fn step(&self, data: &[u8]) -> Result<Vec<u8>, String> {
        let text = std::str::from_utf8(data).map_err(|_| "не UTF-8".to_string())?;
        let lines: Vec<&str> = text.split('\n').collect();
        let eol = if text.contains("\r\n") { "\r" } else { "" };
        let doc = Doc::scan(&lines);
        let mut out: Vec<Option<String>> = lines.iter().map(|l| Some(l.to_string())).collect();
        let mut inserts: Vec<(usize, String)> = Vec::new();
        for (i, e) in doc.el.iter().enumerate() {
            let Some(e) = e else { continue };
            // заголовок: version корня
            if Some(i) == doc.root {
                if let Some(l) = &mut out[i] {
                    *l = l.replace(&format!(" version=\"{}\"", self.from), &format!(" version=\"{}\"", self.version));
                }
            }
            for r in &self.rules {
                if let Kind::AttrDrop(a) = &r.kind {
                    if r.tag == "*" || r.tag == e.tag {
                        if let Some(l) = &mut out[i] {
                            drop_attr(l, a);
                        }
                    }
                    continue;
                }
                if let Kind::Insert { value, unless, before } = &r.kind {
                    // правило вставки относится к родителю: `i` — открытие родителя с детьми
                    if e.close.is_none() || !r.parents.iter().any(|p| doc.is(i, p)) {
                        continue;
                    }
                    let kids: Vec<usize> = doc.children(i);
                    if kids.iter().any(|&k| doc.tag(k) == r.tag || unless.iter().any(|u| doc.tag(k) == u)) {
                        continue;
                    }
                    let at = kids.iter().copied().find(|&k| before.iter().any(|b| doc.tag(k) == b)).or(e.close).unwrap();
                    let indent: String = lines[i].chars().take_while(|c| *c == '\t').collect();
                    inserts.push((at, format!("{indent}\t<{0}>{value}</{0}>{eol}", r.tag)));
                    continue;
                }
                if e.tag != r.tag || !r.parents.is_empty() && !r.parents.iter().any(|p| doc.under(i, p)) {
                    continue;
                }
                let Some(l) = &mut out[i] else { continue };
                match &r.kind {
                    Kind::Absent(vals) => match (&e.value, e.close) {
                        (Some(v), _) if vals.is_empty() || vals.contains(v) => out[i] = None,
                        (Some(v), _) => return Err(format!("<{}>{v}: в формате {} такого свойства нет", e.tag, self.version)),
                        (None, Some(c)) if vals.is_empty() => (i..=c).for_each(|k| out[k] = None),
                        (None, _) => return Err(format!("<{}> из нескольких строк: в формате {} такого свойства нет", e.tag, self.version)),
                    },
                    Kind::AbsentWhere(c, v) => {
                        if let Some(end) = e.close.filter(|_| doc.children(i).iter().any(|&k| doc.tag(k) == c && doc.value(k) == Some(v.as_str()))) {
                            (i..=end).for_each(|k| out[k] = None);
                        }
                    }
                    Kind::Refuse(v) if l.contains(v.as_str()) => return Err(format!("<{}> с {v}: в формате {} по тексту 2.21 не выразить", e.tag, self.version)),
                    Kind::Default(d) if e.value.as_deref() == Some(d.as_str()) => out[i] = None,
                    Kind::Value(a, b) => {
                        if let Some(v) = &e.value {
                            let new = match a.strip_prefix("*:") {
                                Some(a) => v.split_once(':').filter(|(_, x)| x == &a).map(|(p, _)| format!("{p}:{}", b.trim_start_matches("*:"))),
                                None => (v == a).then(|| b.clone()),
                            };
                            if let Some(new) = new {
                                *l = l.replacen(&format!(">{v}</"), &format!(">{new}</"), 1);
                            }
                        }
                    }
                    Kind::Rename(b) => {
                        *l = l.replacen(&format!("<{}", e.tag), &format!("<{b}"), 1).replacen(&format!("</{}>", e.tag), &format!("</{b}>"), 1);
                        if let Some(c) = e.close.filter(|&c| c != i) {
                            if let Some(cl) = &mut out[c] {
                                *cl = cl.replacen(&format!("</{}>", e.tag), &format!("</{b}>"), 1);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        inserts.sort_by_key(|(at, _)| *at);
        let mut res = String::with_capacity(text.len());
        let mut ins = inserts.into_iter().peekable();
        for (i, l) in out.into_iter().enumerate() {
            while let Some((_, s)) = ins.next_if(|(at, _)| *at == i) {
                res.push_str(&s);
                res.push('\n');
            }
            if let Some(l) = l {
                res.push_str(&l);
                if i + 1 < lines.len() {
                    res.push('\n');
                }
            }
        }
        for r in &self.rules {
            if let Kind::AttrDrop(a) = &r.kind {
                if let Some(p) = a.strip_prefix("xmlns:") {
                    if res.contains(&format!(">{p}:")) || res.contains(&format!("\"{p}:")) {
                        return Err(format!("префикс {p}: используется, а в формате {} его объявления нет", self.version));
                    }
                }
            }
        }
        Ok(res.into_bytes())
    }
}

/// Убрать ` a="…"` из строки.
fn drop_attr(l: &mut String, a: &str) {
    if let Some(s) = l.find(&format!(" {a}=\"")) {
        if let Some(e) = l[s + a.len() + 3..].find('"') {
            l.replace_range(s..s + a.len() + 3 + e + 1, "");
        }
    }
}

/// Элемент, открытый в начале строки.
struct El {
    tag: String,
    /// Строка с открытием родителя.
    parent: Option<usize>,
    /// Значение элемента в одну строку (`<a>v</a>`, `<a/>` — пустое); None — элемент с детьми.
    value: Option<String>,
    /// Строка закрытия элемента с детьми.
    close: Option<usize>,
}

struct Doc {
    el: Vec<Option<El>>,
    root: Option<usize>,
}

impl Doc {
    fn scan(lines: &[&str]) -> Doc {
        let mut el: Vec<Option<El>> = (0..lines.len()).map(|_| None).collect();
        let mut stack: Vec<usize> = Vec::new();
        let mut root = None;
        let line = |i: usize| lines[i].trim_start_matches(['\t', ' ', '\u{feff}']).trim_end_matches('\r');
        let mut i = 0;
        while i < lines.len() {
            let s = line(i);
            // элемент, открытый в начале строки
            if let Some(rest) = s.strip_prefix('<').filter(|r| !r.starts_with(['/', '?', '!'])) {
                let tag: String = rest.chars().take_while(|c| !c.is_whitespace() && *c != '>' && *c != '/').collect();
                let close_tag = format!("</{tag}>");
                // конец открывающего тега — первый `>` вне кавычек
                let mut q = false;
                let gt = s.char_indices().find(|&(_, c)| {
                    q ^= c == '"';
                    c == '>' && !q
                });
                let value = if s.ends_with("/>") && !s.contains("</") {
                    Some(String::new())
                } else if s.ends_with(&close_tag) {
                    gt.map(|(g, _)| s[g + 1..s.len() - close_tag.len()].to_string())
                } else {
                    None
                };
                root.get_or_insert(i);
                let parent = stack.last().copied();
                // текст в несколько строк (`<v8:content>…⏎…</v8:content>`): элемент без детей до строки с закрытием
                if value.is_none() && gt.is_some_and(|(g, _)| g + 1 < s.len()) {
                    let end = (i + 1..lines.len()).find(|&k| line(k).ends_with(&close_tag)).unwrap_or(lines.len() - 1);
                    el[i] = Some(El { tag, parent, value: None, close: Some(end) });
                    i = end + 1;
                    continue;
                }
                el[i] = Some(El { tag, parent, value: value.clone(), close: None });
                if value.is_none() {
                    stack.push(i);
                }
                i += 1;
                continue;
            }
            if s.starts_with("</") {
                if let Some(o) = stack.pop() {
                    if let Some(e) = &mut el[o] {
                        e.close = Some(i);
                    }
                }
            }
            i += 1;
        }
        Doc { el, root }
    }

    fn tag(&self, i: usize) -> &str {
        self.el[i].as_ref().map_or("", |e| e.tag.as_str())
    }

    fn value(&self, i: usize) -> Option<&str> {
        self.el[i].as_ref().and_then(|e| e.value.as_deref())
    }

    fn children(&self, i: usize) -> Vec<usize> {
        let end = self.el[i].as_ref().and_then(|e| e.close).unwrap_or(i);
        (i + 1..end).filter(|&k| self.el[k].as_ref().is_some_and(|e| e.parent == Some(i))).collect()
    }

    /// Путь элемента `i` (вместе с ним самим) кончается на `path`.
    fn is(&self, i: usize, path: &[String]) -> bool {
        let mut cur = Some(i);
        for want in path.iter().rev() {
            match cur.and_then(|c| self.el[c].as_ref()) {
                Some(e) if e.tag == *want => cur = e.parent,
                _ => return false,
            }
        }
        true
    }

    /// Родитель элемента `i` — `path` (суффикс пути).
    fn under(&self, i: usize, path: &[String]) -> bool {
        self.el[i].as_ref().and_then(|e| e.parent).is_some_and(|p| self.is(p, path))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rules() {
        let f = Format::parse(
            "version 2.17\nattr-drop * xmlns:pal\nabsent AutoEditMode = true\ndefault Group in Form = Vertical\nvalue WindowOpeningMode = LockOwner > LockOwnerWindow\n\
             value v8:TypeSet = *:AnyIBRef > *:AnyRef\nrename HorizontalLinesBWA > HorizontalLines\ninsert CheckBoxType in CheckBoxField = Auto unless ThreeState before ContextMenu\n",
        )
        .unwrap();
        let src = "\u{feff}<?xml version=\"1.0\"?>\r\n<Form xmlns:pal=\"p\" version=\"2.21\">\r\n\t<WindowOpeningMode>LockOwner</WindowOpeningMode>\r\n\t<Group>Vertical</Group>\r\n\
                   \t<CheckBoxField name=\"a\">\r\n\t\t<AutoEditMode>true</AutoEditMode>\r\n\t\t<ContextMenu/>\r\n\t</CheckBoxField>\r\n\
                   \t<CheckBoxField name=\"b\">\r\n\t\t<ThreeState>true</ThreeState>\r\n\t</CheckBoxField>\r\n\
                   \t<Table>\r\n\t\t<HorizontalLinesBWA>false</HorizontalLinesBWA>\r\n\t\t<Group>Vertical</Group>\r\n\t\t<v8:TypeSet xmlns:d5p1=\"x\">d5p1:AnyIBRef</v8:TypeSet>\r\n\t</Table>\r\n</Form>";
        let want = "\u{feff}<?xml version=\"1.0\"?>\r\n<Form version=\"2.17\">\r\n\t<WindowOpeningMode>LockOwnerWindow</WindowOpeningMode>\r\n\
                    \t<CheckBoxField name=\"a\">\r\n\t\t<CheckBoxType>Auto</CheckBoxType>\r\n\t\t<ContextMenu/>\r\n\t</CheckBoxField>\r\n\
                    \t<CheckBoxField name=\"b\">\r\n\t\t<ThreeState>true</ThreeState>\r\n\t</CheckBoxField>\r\n\
                    \t<Table>\r\n\t\t<HorizontalLines>false</HorizontalLines>\r\n\t\t<Group>Vertical</Group>\r\n\t\t<v8:TypeSet xmlns:d5p1=\"x\">d5p1:AnyRef</v8:TypeSet>\r\n\t</Table>\r\n</Form>";
        let got = String::from_utf8(f.down(std::path::Path::new("a/Form.xml"), src.as_bytes()).unwrap()).unwrap();
        assert_eq!(got, want);
        // значение, которого старый формат не выражает, — ошибка
        assert!(f.down(std::path::Path::new("a.xml"), b"<a>\n\t<AutoEditMode>false</AutoEditMode>\n</a>").is_err());
        // текст в несколько строк не сбивает вложенность: вставка — в свой CheckBoxField, перед ContextMenu
        let src = "<Form>\n\t<CheckBoxField>\n\t\t<ToolTip>\n\t\t\t<v8:content>x\ny</v8:content>\n\t\t</ToolTip>\n\t\t<ContextMenu/>\n\t</CheckBoxField>\n</Form>";
        let want = "<Form>\n\t<CheckBoxField>\n\t\t<ToolTip>\n\t\t\t<v8:content>x\ny</v8:content>\n\t\t</ToolTip>\n\t\t<CheckBoxType>Auto</CheckBoxType>\n\t\t<ContextMenu/>\n\t</CheckBoxField>\n</Form>";
        assert_eq!(String::from_utf8(f.down(std::path::Path::new("a.xml"), src.as_bytes()).unwrap()).unwrap(), want);
        // блок с ребенком-значением
        let g = Format::parse("version 2.17
absent app:functionality where app:functionality = TextToSpeech
").unwrap();
        let src = "<a>
	<app:functionality>
		<app:functionality>Speech</app:functionality>
	</app:functionality>
	<app:functionality>
		<app:functionality>TextToSpeech</app:functionality>
	</app:functionality>
</a>";
        let want = "<a>
	<app:functionality>
		<app:functionality>Speech</app:functionality>
	</app:functionality>
</a>";
        assert_eq!(String::from_utf8(g.down(std::path::Path::new("a.xml"), src.as_bytes()).unwrap()).unwrap(), want);
        // refuse: картинка СКД в «читаемом» виде — ошибка, прежний вид проходит
        let r = Format::parse("version 2.20\nrefuse dcscor:value has xcf/readable\n").unwrap();
        assert!(r.down(std::path::Path::new("a.xml"), b"<a>\n\t<dcscor:value xmlns:d8p1=\"http://v8.1c.ru/8.3/xcf/readable\" xsi:type=\"d8p1:Picture\">\n\t</dcscor:value>\n</a>").is_err());
        assert!(r.down(std::path::Path::new("a.xml"), b"<a>\n\t<dcscor:value xsi:type=\"v8ui:Picture\" ref=\"v8ui:X\"/>\n</a>").is_ok());
        // префикс в ходу — объявление убирать нельзя
        assert!(f.down(std::path::Path::new("a.xml"), b"<a xmlns:pal=\"p\">\n\t<c>pal:x</c>\n</a>").is_err());
    }
}
