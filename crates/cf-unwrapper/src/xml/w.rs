//! Запись XML в раскладке платформы: BOM, строки через CRLF (после последней — ничего), отступ табами,
//! пустой элемент `<X/>`, переводы строк внутри текста — LF. Экранируются только `&`, `<`, `>` (в атрибутах и `"`).

pub const BOM: &[u8] = &[0xef, 0xbb, 0xbf];

const NAMESPACES: &str = "xmlns=\"http://v8.1c.ru/8.3/MDClasses\" xmlns:app=\"http://v8.1c.ru/8.2/managed-application/core\" xmlns:cfg=\"http://v8.1c.ru/8.1/data/enterprise/current-config\" xmlns:cmi=\"http://v8.1c.ru/8.2/managed-application/cmi\" xmlns:ent=\"http://v8.1c.ru/8.1/data/enterprise\" xmlns:lf=\"http://v8.1c.ru/8.2/managed-application/logform\" xmlns:pal=\"http://v8.1c.ru/8.1/data/ui/colors/palette\" xmlns:style=\"http://v8.1c.ru/8.1/data/ui/style\" xmlns:sys=\"http://v8.1c.ru/8.1/data/ui/fonts/system\" xmlns:v8=\"http://v8.1c.ru/8.1/data/core\" xmlns:v8ui=\"http://v8.1c.ru/8.1/data/ui\" xmlns:web=\"http://v8.1c.ru/8.1/data/ui/colors/web\" xmlns:win=\"http://v8.1c.ru/8.1/data/ui/colors/windows\" xmlns:xen=\"http://v8.1c.ru/8.3/xcf/enums\" xmlns:xpr=\"http://v8.1c.ru/8.3/xcf/predef\" xmlns:xr=\"http://v8.1c.ru/8.3/xcf/readable\" xmlns:xs=\"http://www.w3.org/2001/XMLSchema\" xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\"";

pub struct W {
    out: String,
    open: Vec<String>,
}

impl W {
    /// Описание объекта: `<MetaDataObject …>` с полным списком пространств имен.
    pub fn metadata() -> W {
        let mut w = W { out: "<?xml version=\"1.0\" encoding=\"UTF-8\"?>".into(), open: Vec::new() };
        w.line(&format!("<MetaDataObject {NAMESPACES} version=\"2.21\">"));
        w.open.push("MetaDataObject".into());
        w
    }

    /// Документ с корнем `<tag …>` (строка корня — целиком в `head`).
    pub fn root(tag: &str, head: &str) -> W {
        let mut w = W { out: "<?xml version=\"1.0\" encoding=\"UTF-8\"?>".into(), open: Vec::new() };
        w.line(head);
        w.open.push(tag.into());
        w
    }

    fn line(&mut self, s: &str) {
        if !self.out.is_empty() {
            self.out.push_str("\r\n");
        }
        for _ in 0..self.open.len() {
            self.out.push('\t');
        }
        self.out.push_str(s);
    }

    /// `<tag a="v">`; закрывается [`W::end`].
    pub fn open(&mut self, tag: &str, attrs: &[(&str, &str)]) {
        let s = format!("<{tag}{}>", attrs_text(attrs));
        self.line(&s);
        self.open.push(tag.into());
    }

    pub fn end(&mut self) {
        let tag = self.open.pop().expect("лишний end");
        self.line(&format!("</{tag}>"));
    }

    /// `<tag>текст</tag>`, пустой — `<tag/>`.
    pub fn text(&mut self, tag: &str, text: &str) {
        self.text_attrs(tag, &[], text);
    }

    pub fn text_attrs(&mut self, tag: &str, attrs: &[(&str, &str)], text: &str) {
        let a = attrs_text(attrs);
        let s = if text.is_empty() { format!("<{tag}{a}/>") } else { format!("<{tag}{a}>{}</{tag}>", escape(text)) };
        self.line(&s);
    }

    pub fn bool(&mut self, tag: &str, v: bool) {
        self.text(tag, if v { "true" } else { "false" });
    }

    /// `<tag xsi:nil="true"/>`
    pub fn nil(&mut self, tag: &str) {
        self.line(&format!("<{tag} xsi:nil=\"true\"/>"));
    }

    /// Число открытых элементов (глубина следующей строки).
    pub fn depth(&self) -> usize {
        self.open.len()
    }

    /// Закрывает открытые элементы; файл — BOM и текст без перевода строки в конце.
    pub fn finish(mut self) -> Vec<u8> {
        while !self.open.is_empty() {
            self.end();
        }
        [BOM, self.out.as_bytes()].concat()
    }
}

/// Префикс пространства имен, объявленного в `<MetaDataObject>`.
pub fn prefix(ns: &str) -> Option<&'static str> {
    NAMESPACES.split(' ').find_map(|d| {
        let (p, v) = d.strip_prefix("xmlns:")?.split_once('=')?;
        (v.trim_matches('"') == ns).then_some(p)
    })
}

/// Пространство имен префикса из `<MetaDataObject>` (обратное [`prefix`]).
pub fn namespace(p: &str) -> Option<&'static str> {
    NAMESPACES.split(' ').find_map(|d| {
        let (k, v) = d.strip_prefix("xmlns:")?.split_once('=')?;
        (k == p).then(|| v.trim_matches('"'))
    })
}

fn attrs_text(attrs: &[(&str, &str)]) -> String {
    attrs.iter().map(|(k, v)| format!(" {k}=\"{}\"", escape(v).replace('"', "&quot;"))).collect()
}

/// Текст элемента: CRLF из скобочной строки становится LF.
pub fn escape(s: &str) -> String {
    let mut o = String::with_capacity(s.len());
    for c in s.replace("\r\n", "\n").chars() {
        match c {
            '&' => o.push_str("&amp;"),
            '<' => o.push_str("&lt;"),
            '>' => o.push_str("&gt;"),
            c => o.push(c),
        }
    }
    o
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout() {
        let mut w = W::metadata();
        w.open("CommonModule", &[("uuid", "u")]);
        w.open("Properties", &[]);
        w.text("Name", "А<&>\"");
        w.text("Comment", "");
        w.text("Text", "a\r\nb");
        w.nil("MinValue");
        w.bool("Global", false);
        let b = w.finish();
        assert!(b.starts_with(BOM));
        let s = String::from_utf8(b[3..].to_vec()).unwrap();
        let lines: Vec<&str> = s.split("\r\n").collect();
        assert_eq!(lines[0], "<?xml version=\"1.0\" encoding=\"UTF-8\"?>");
        assert!(lines[1].starts_with("<MetaDataObject xmlns=") && lines[1].ends_with(" version=\"2.21\">"));
        assert_eq!(
            &lines[2..],
            ["\t<CommonModule uuid=\"u\">", "\t\t<Properties>", "\t\t\t<Name>А&lt;&amp;&gt;\"</Name>", "\t\t\t<Comment/>", "\t\t\t<Text>a\nb</Text>", "\t\t\t<MinValue xsi:nil=\"true\"/>", "\t\t\t<Global>false</Global>", "\t\t</Properties>", "\t</CommonModule>", "</MetaDataObject>"]
        );
    }
}
