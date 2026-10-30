//! Справка и HTML-макет (обратное `html` в `xml/payload.rs`): `<base>.xml` (`<Help>` со списком страниц),
//! `<base>/<язык>.html`, `<base>/_files/<имя>` → `{5,N,"<язык>",{#base64:<страница>},…,K,"<имя>",1,{#base64:…},…}`.
//!
//! Страница — файл с CRLF вместо LF; ссылки — как хранит загрузка платформы (сверено по всем 3 389 страницам справки
//! и 128 HTML-макетам БП): `"<Объект>/Help` → `"../id<uuid>/038b5c85-…`, `"CommonPicture.Имя"` и `"StdPicture.Имя"` →
//! `"../../mdpicture/id<uuid>/00000000-…"` (или `idn-<номер>`), `_files/` → `<guid>_files/` (у справки — 038b5c85-…,
//! у макета — 8eb4fad1-…).

use super::dom::El;
use super::index::Index;
use super::payload::base64_atom;
use super::tpl::{a, l, s};
use super::Rec;
use crate::brace::Doc;
use crate::xml::payload::{STD_NUMBERED, STD_PICTURES};

pub const HELP_FILES: &str = "038b5c85-fb1c-4082-9c4c-e69f8928bf3a";
pub const TEMPLATE_FILES: &str = "8eb4fad1-1fa6-403e-970f-2c12dbb43e23";
const HELP_PROP: &str = "038b5c85-fb1c-4082-9c4c-e69f8928bf3a";
const NO_GUID: &str = "00000000-0000-0000-0000-000000000000";

/// Запись и прочитанные файлы (относительно каталога `<base>`).
pub fn record(doc: &El, x: &Index, files_guid: &str, dir: &std::path::Path) -> Result<(Rec, Vec<String>), String> {
    // у Help.xml нет xmlns:xr
    if doc.name != "Help" || doc.attrs.len() != 4 {
        return Err("корень не <Help> или пространства имен не как у выгрузки 2.21".into());
    }
    for (k, v) in [("xmlns", "http://v8.1c.ru/8.3/xcf/extrnprops"), ("xmlns:xs", "http://www.w3.org/2001/XMLSchema"), ("xmlns:xsi", "http://www.w3.org/2001/XMLSchema-instance"), ("version", "2.21")] {
        if doc.attr(k) != Some(v) {
            return Err("пространства имен или версия <Help> не как у выгрузки 2.21".into());
        }
    }
    let mut used = Vec::new();
    let mut v = vec![a("5"), a("0")];
    for p in doc.elems() {
        if p.name != "Page" {
            return Err(format!("строка {}: <{}> в <Help>", p.line, p.name));
        }
        let lang = p.text()?;
        if !crate::xml::safe_name(&lang) {
            return Err(format!("недопустимый код языка {lang:?}"));
        }
        let f = format!("{lang}.html");
        let b = std::fs::read(dir.join(&f)).map_err(|e| format!("{f}: {e}"))?;
        let t = String::from_utf8(b).map_err(|_| format!("{f}: не UTF-8"))?;
        if !t.starts_with('\u{feff}') {
            return Err(format!("{f}: страница без BOM"));
        }
        let page = links(&super::tpl::crlf(&t), x, files_guid)?;
        v.push(s(lang));
        v.push(l(vec![base64_atom(page.as_bytes())]));
        used.push(f);
    }
    if used.is_empty() {
        return Err("справка без страниц (платформа такую не выгружает)".into());
    }
    v[1] = a(used.len().to_string());
    // вложения — в порядке имен файлов
    let mut att: Vec<String> = match std::fs::read_dir(dir.join("_files")) {
        Ok(rd) => rd.filter_map(|e| e.ok()).filter(|e| e.path().is_file()).filter_map(|e| e.file_name().into_string().ok()).collect(),
        Err(_) => Vec::new(),
    };
    att.sort_by_key(|n| n.to_uppercase());
    v.push(a(att.len().to_string()));
    for n in att {
        let b = std::fs::read(dir.join("_files").join(&n)).map_err(|e| format!("_files/{n}: {e}"))?;
        v.push(s(n.clone()));
        v.push(a("1"));
        v.push(l(vec![base64_atom(&b)]));
        used.push(format!("_files/{n}"));
    }
    Ok((Rec::Raw(Doc::new(true, vec![l(v)]).write()?.into_bytes()), used))
}

/// Ссылки страницы как в записи загрузки платформы.
fn links(page: &str, x: &Index, files_guid: &str) -> Result<String, String> {
    let mut out = String::with_capacity(page.len() + 256);
    // содержимое <style> и <script> разборщик HTML платформы читает текстом: ссылки в нем не переводятся
    // (незакрытый <style> в справке трех отчетов ERP)
    let low = page.to_ascii_lowercase();
    let mut at = 0;
    while let Some(o) = ["<style", "<script"].iter().filter_map(|t| low[at..].find(t).map(|i| (at + i, &t[1..]))).min() {
        let close = low[o.0..].find(&format!("</{}", o.1)).map_or(page.len(), |i| o.0 + i);
        quoted(&page[at..o.0], x, &mut out)?;
        out.push_str(&page[o.0..close]);
        at = close;
    }
    quoted(&page[at..], x, &mut out)?;
    files(&out, files_guid)
}

/// Ссылки в кавычках: справка объекта, стандартные и общие картинки.
fn quoted(page: &str, x: &Index, out: &mut String) -> Result<(), String> {
    let mut rest = page;
    while let Some(q) = rest.find('"') {
        out.push_str(&rest[..=q]);
        rest = &rest[q + 1..];
        let end = rest.find(['"', '#', '\'', ' ', '>', '<']).unwrap_or(rest.len());
        let word = &rest[..end];
        if let Some(obj) = word.strip_suffix("/Help") {
            if let Some(id) = x.id(obj) {
                out.push_str(&format!("../id{id}/{HELP_PROP}"));
                rest = &rest[end..];
                continue;
            }
        }
        if let Some(name) = word.strip_prefix("StdPicture.").filter(|_| rest[end..].starts_with('"')) {
            if let Some(g) = STD_PICTURES.iter().find(|(_, n)| *n == name).map(|(g, _)| *g).or_else(|| super::form::platform::std_picture_id(name)).filter(|g| !g.starts_with('-')) {
                out.push_str(&format!("../../mdpicture/id{g}/{NO_GUID}"));
                rest = &rest[end..];
                continue;
            }
            if let Some(num) = STD_NUMBERED.iter().find(|(_, n)| *n == name).map(|(k, _)| *k).or_else(|| super::form::platform::std_picture_id(name).and_then(|g| g.strip_prefix('-'))) {
                out.push_str(&format!("../../mdpicture/idn-{num}"));
                rest = &rest[end..];
                continue;
            }
            return Err(format!("незнакомая стандартная картинка {name}"));
        }
        if word.starts_with("CommonPicture.") && rest[end..].starts_with('"') {
            let id = x.id(word).ok_or(format!("нет картинки {word}"))?;
            out.push_str(&format!("../../mdpicture/id{id}/{NO_GUID}"));
            rest = &rest[end..];
            continue;
        }
    }
    out.push_str(rest);
    Ok(())
}

/// `_files/` без GUID перед ним — с GUID вложений.
fn files(out: &str, files_guid: &str) -> Result<String, String> {
    let mut res = String::with_capacity(out.len() + 64);
    let mut r = out;
    while let Some(p) = r.find("_files/") {
        let head = &r[..p];
        res.push_str(head);
        let has = head.len() >= 36 && head.is_char_boundary(head.len() - 36) && crate::brace::is_guid(&head[head.len() - 36..]);
        if !has {
            res.push_str(files_guid);
        }
        res.push_str("_files/");
        r = &r[p + 7..];
    }
    res.push_str(r);
    Ok(res)
}
