//! Файлы, которые лежат в записях почти как есть (обратное `xml/payload.rs`): картинки, двоичные и текстовые макеты,
//! пакеты XDTO, родительские конфигурации, подпись мобильного клиента, модули без исходного текста.

use super::dom::El;
use super::tpl::{a, l};
use super::{Rec, BOM};
use crate::brace::{Doc, Node};

/// `{#base64:…}`: строки по 64 знака через `\r\r\n` (первая — с префиксом `#base64:`), как пишет платформа.
/// После последней строки разделитель есть, если она полная и ее номер не 16k+1 (так у всех 7 869 таких значений
/// в записях загрузки БП: видимо, кодировщик пишет блоками по 16 строк).
pub fn base64_atom(data: &[u8]) -> Node {
    const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut s = Vec::with_capacity(data.len() * 4 / 3 + 4);
    for c in data.chunks(3) {
        let n = (c[0] as u32) << 16 | (*c.get(1).unwrap_or(&0) as u32) << 8 | *c.get(2).unwrap_or(&0) as u32;
        for k in 0..4 {
            s.push(if k <= c.len() { T[(n >> (18 - 6 * k) & 63) as usize] } else { b'=' });
        }
    }
    let mut o = String::from("#base64:");
    let n = s.chunks(64).count();
    for (i, c) in s.chunks(64).enumerate() {
        o.push_str(std::str::from_utf8(c).unwrap_or_default());
        if c.len() == 64 && (i + 1 < n || n % 16 != 1) {
            o.push_str("\r\r\n");
        }
    }
    a(o)
}

fn bytes(n: Node) -> Result<Vec<u8>, String> {
    Ok(Doc::new(true, vec![n]).write()?.into_bytes())
}

/// Двоичный макет и внешняя компонента: `{1,{#base64:…}}`.
pub fn binary(data: &[u8]) -> Result<Rec, String> {
    if data.is_empty() {
        // как платформа хранит пустые данные, не проверено
        return Err("пустой файл".into());
    }
    Ok(Rec::Raw(bytes(l(vec![a("1"), l(vec![base64_atom(data)])]))?))
}

/// Текстовый макет: файл как есть (BOM и CRLF).
pub fn text(data: &[u8]) -> Result<Rec, String> {
    if !data.starts_with(BOM) {
        return Err("текстовый макет без BOM".into());
    }
    Ok(Rec::Raw(data.to_vec()))
}

const EXT_NS: &[(&str, &str)] = &[
    ("xmlns", "http://v8.1c.ru/8.3/xcf/extrnprops"),
    ("xmlns:xr", "http://v8.1c.ru/8.3/xcf/readable"),
    ("xmlns:xs", "http://www.w3.org/2001/XMLSchema"),
    ("xmlns:xsi", "http://www.w3.org/2001/XMLSchema-instance"),
    ("version", "2.21"),
];

/// Корень файла внешних свойств (`ExtPicture`, …) с пространствами имен выгрузки 2.21.
pub fn check_ext_root(doc: &El, name: &str) -> Result<(), String> {
    check_ext_root_ns(doc, name, "xmlns:xr", "http://v8.1c.ru/8.3/xcf/readable")
}

/// То же, но на месте `xmlns:xr` — другое пространство (у расписания — `xmlns:ent`).
pub fn check_ext_root_ns(doc: &El, name: &str, key: &str, ns: &str) -> Result<(), String> {
    if doc.name != name {
        return Err(format!("корень <{}> вместо <{name}>", doc.name));
    }
    let want: Vec<(&str, &str)> = EXT_NS.iter().map(|&(k, v)| if k == "xmlns:xr" { (key, ns) } else { (k, v) }).collect();
    if doc.attrs.len() != want.len() || want.iter().any(|(k, v)| doc.attr(k) != Some(v)) {
        return Err(format!("пространства имен или версия <{name}> не как у выгрузки 2.21"));
    }
    Ok(())
}

/// `<role>.xml` (`ExtPicture`) и файл картинки рядом → `{1,{<прозрачность>,0,<x>,<y>},{{#base64:…}}}`.
/// Возвращает запись и имя файла картинки относительно `Ext`.
pub fn picture(doc: &El, role: &str, read: &dyn Fn(&str) -> Result<Vec<u8>, String>) -> Result<(Rec, String), String> {
    check_ext_root(doc, "ExtPicture")?;
    let p = doc.req("Picture")?;
    let abs = p.text_of("xr:Abs")?;
    if abs.is_empty() || abs.contains(['/', '\\', ':']) || abs == ".." {
        return Err(format!("xr:Abs: недопустимое имя {abs:?}"));
    }
    let (t, x, y) = match (p.text_of("xr:LoadTransparent")?.as_str(), p.child("xr:TransparentPixel")) {
        ("false", None) => ("0", "-1".to_string(), "-1".to_string()),
        ("true", Some(px)) => {
            let n = |k: &str| px.req_attr(k).and_then(|v| v.parse::<i64>().map(|n| n.to_string()).map_err(|_| format!("TransparentPixel {k}={v:?}")));
            ("1", n("x")?, n("y")?)
        }
        (v, px) => return Err(format!("LoadTransparent {v} и TransparentPixel {}: так платформа не выгружает", px.is_some())),
    };
    let rel = format!("{role}/{abs}");
    let data = read(&rel)?;
    if data.is_empty() {
        return Err(format!("{rel}: пустой файл"));
    }
    let n = l(vec![a("1"), l(vec![a(t), a("0"), a(x), a(y)]), l(vec![l(vec![base64_atom(&data)])])]);
    Ok((Rec::Raw(bytes(n)?), rel))
}

/// Встроенная картинка элемента формы `<Picture><xr:Abs>Свойство.расш</xr:Abs><xr:LoadTransparent/>[<xr:TransparentPixel/>]`
/// (файл `Form/Items/<элемент>/<Abs>`) → `{4,3,{0},"",<x>,<y>,<прозрачность>,{{#base64:<файл>}},0,""}`: байты файла
/// как есть (220 из 220 картинок форм БП совпали с записями загрузки). `read` читает файл по имени из `xr:Abs`;
/// в ответе — узел и это имя (файл загрузке формы надо отметить использованным). Для загрузки форм.
pub fn item_picture(p: &El, read: &dyn Fn(&str) -> Result<Vec<u8>, String>) -> Result<(Node, String), String> {
    let at = |m: String| format!("строка {}: <{}>: {m}", p.line, p.name);
    let abs = p.text_of("xr:Abs")?;
    let Some((_, ext)) = abs.rsplit_once('.').filter(|_| !abs.contains(['/', '\\', ':'])) else { return Err(at(format!("xr:Abs: недопустимое имя {abs:?}"))) };
    let t = super::tpl::boolean(p.req("xr:LoadTransparent")?)?;
    let (x, y) = match p.child("xr:TransparentPixel") {
        None => ("-1".to_string(), "-1".to_string()),
        Some(px) => {
            let n = |k: &str| px.req_attr(k).and_then(|v| v.parse::<i64>().map(|n| n.to_string()).map_err(|_| at(format!("TransparentPixel {k}={v:?}"))));
            (n("x")?, n("y")?)
        }
    };
    if p.elems().count() != 2 + usize::from(x != "-1" || y != "-1") {
        return Err(at("лишнее в картинке".into()));
    }
    let data = read(&abs)?;
    // расширение выгрузка берет по сигнатуре данных: другое — файл не от выгрузки
    if crate::xml::payload::picture_ext(&data) != Some(ext) {
        return Err(at(format!("{abs}: содержимое не {ext}")));
    }
    let n = l(vec![a("4"), a("3"), l(vec![a("0")]), super::tpl::s(""), a(x), a(y), t, l(vec![l(vec![base64_atom(&data)])]), a("0"), super::tpl::s("")]);
    Ok((n, abs))
}

/// Модуль без исходного текста (`<Роль>.bin`) и прочие записи-контейнеры: файл — сам вложенный контейнер.
pub fn container(data: &[u8]) -> Result<Rec, String> {
    let c = crate::container::read_at(data, 0).ok().filter(|_| crate::container::is_container(data)).ok_or("не контейнер 1С")?;
    Ok(Rec::Dir(c.entries.into_iter().map(|e| (e.name, e.data)).collect()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_lines() {
        let n = base64_atom(&[0u8; 100]);
        let t = n.atom().unwrap();
        let lines: Vec<&str> = t.split("\r\r\n").collect();
        assert_eq!(lines.iter().map(|s| s.len()).collect::<Vec<_>>(), [72, 64, 8]);
        assert_eq!(crate::xml::base64(t.strip_prefix("#base64:").unwrap()).unwrap(), vec![0u8; 100]);
        assert_eq!(base64_atom(b"ab").atom(), Some("#base64:YWI="));
        assert!(base64_atom(&[0u8; 96]).atom().unwrap().ends_with("AAAA\r\r\n"));
        assert!(base64_atom(&[0u8; 48]).atom().unwrap().ends_with("AAAA"));
        assert!(base64_atom(&[0u8; 48 * 17]).atom().unwrap().ends_with("AAAA"));
    }

    #[test]
    fn form_item_picture() {
        let x = "<Picture><xr:Abs>Picture.gif</xr:Abs><xr:LoadTransparent>true</xr:LoadTransparent><xr:TransparentPixel x=\"0\" y=\"30\"/></Picture>";
        let e = super::super::dom::parse(x).unwrap();
        let (n, f) = item_picture(&e, &|f| if f == "Picture.gif" { Ok(b"GIF89a".to_vec()) } else { Err(f.into()) }).unwrap();
        assert_eq!((crate::brace::write(&n).unwrap().replace("\r\n", ""), f.as_str()), ("{4,3,{0},\"\",0,30,1,{{#base64:R0lGODlh}},0,\"\"}".into(), "Picture.gif"));
        assert!(e.unread().is_none());
        // содержимое не того вида, что расширение, — отказ
        let e = super::super::dom::parse(&x.replace("Picture.gif", "Picture.png")).unwrap();
        assert!(item_picture(&e, &|_| Ok(b"GIF89a".to_vec())).is_err());
    }
}
