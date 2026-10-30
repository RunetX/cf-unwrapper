//! Лексемы языка запросов: имена (русские и английские), числа, строки `"…"` (кавычка удваивается), параметры `&Имя`,
//! знаки. Комментарий `//` до конца строки пропускается.

#[derive(Clone, Debug, PartialEq)]
pub enum Tok {
    Name(String),
    Num(String),
    Str(String),
    Param(String),
    /// Знак: `. , ( ) ; { } + - * / = < > <= >= <>`.
    P(&'static str),
}

const PUNCT: &[&str] = &["<=", ">=", "<>", ".", ",", "(", ")", ";", "{", "}", "+", "-", "*", "/", "=", "<", ">"];

pub fn tokens(t: &str) -> Result<Vec<Tok>, String> {
    let c: Vec<char> = t.chars().collect();
    let word = |ch: char| ch.is_alphanumeric() || ch == '_';
    let mut out = Vec::new();
    let mut i = 0;
    while i < c.len() {
        let ch = c[i];
        if ch.is_whitespace() {
            i += 1;
        } else if ch == '/' && c.get(i + 1) == Some(&'/') {
            while i < c.len() && c[i] != '\n' {
                i += 1;
            }
        } else if ch == '"' {
            let mut s = String::new();
            i += 1;
            loop {
                match c.get(i) {
                    None => return Err("строка без закрывающей кавычки".into()),
                    Some('"') if c.get(i + 1) == Some(&'"') => {
                        s.push('"');
                        i += 2;
                    }
                    Some('"') => {
                        i += 1;
                        break;
                    }
                    Some(&x) => {
                        s.push(x);
                        i += 1;
                    }
                }
            }
            out.push(Tok::Str(s));
        } else if ch == '&' {
            let s: String = c[i + 1..].iter().take_while(|&&x| word(x)).collect();
            if s.is_empty() {
                return Err("параметр без имени".into());
            }
            i += 1 + s.chars().count();
            out.push(Tok::Param(s));
        } else if ch.is_ascii_digit() {
            let mut s: String = c[i..].iter().take_while(|x| x.is_ascii_digit()).collect();
            i += s.len();
            if c.get(i) == Some(&'.') && c.get(i + 1).is_some_and(|x| x.is_ascii_digit()) {
                let f: String = c[i + 1..].iter().take_while(|x| x.is_ascii_digit()).collect();
                i += 1 + f.len();
                s = format!("{s}.{f}");
            }
            out.push(Tok::Num(s));
        } else if word(ch) {
            let s: String = c[i..].iter().take_while(|&&x| word(x)).collect();
            i += s.chars().count();
            out.push(Tok::Name(s));
        } else {
            let rest: String = c[i..c.len().min(i + 2)].iter().collect();
            let p = PUNCT.iter().find(|p| rest.starts_with(**p)).ok_or_else(|| format!("знак «{ch}» не поддержан"))?;
            i += p.len();
            out.push(Tok::P(p));
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lexemes() {
        let t = tokens("ВЫБРАТЬ Т.Поле КАК \"а\"\"б\", &П1 <> 1.5 // коммент\n{ГДЕ}").unwrap();
        let n = |s: &str| Tok::Name(s.into());
        assert_eq!(
            t,
            vec![n("ВЫБРАТЬ"), n("Т"), Tok::P("."), n("Поле"), n("КАК"), Tok::Str("а\"б".into()), Tok::P(","), Tok::Param("П1".into()), Tok::P("<>"), Tok::Num("1.5".into()), Tok::P("{"), n("ГДЕ"), Tok::P("}")]
        );
    }
}
