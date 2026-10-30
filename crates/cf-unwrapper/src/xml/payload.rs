//! Файлы выгрузки, которые лежат в .cf почти как есть: картинки, макеты (текст, двоичные данные, внешние компоненты,
//! HTML), пакеты XDTO, WSDL, родительские конфигурации, подпись мобильного клиента, модули без исходного текста,
//! формы обычного приложения. Устройство записей — docs/research/xml-payload.md.

use crate::brace::{self, is_guid, Node};
use crate::container;
use crate::meta::{Config, Item};
use crate::xml::imp::dom;
use std::collections::BTreeSet;

const BOM: &[u8] = &[0xef, 0xbb, 0xbf];

/// Файлы для записи `entry` с ролью `role`: (путь относительно `Ext`, данные).
/// `None` — не наш формат или незнакомое устройство (запись остается в отчете как невыгруженная).
/// В `rejected` — пропущенные части записи (страница справки с недопустимым кодом языка).
pub fn files(cfg: &Config, index: &super::common::Index, old: bool, it: &Item, entry: &str, role: &str, rejected: &mut Vec<String>) -> Option<Vec<(String, Vec<u8>)>> {
    match role {
        "Picture" | "Splash" | "MainSectionPicture" | "Logo" => picture(&brace::text(&raw(cfg, entry)?), role),
        "Template" => {
            let n = rejected.len();
            let r = template(cfg, index, old, &cfg.entry_text(&it.header.id)?, raw(cfg, entry)?, rejected);
            if r.is_none() && rejected.len() == n {
                rejected.push(format!("макет вида {:?}: незнакомое устройство записи — не выгружено", cfg.entry_text(&it.header.id).as_deref().and_then(template_kind)));
            }
            r
        }
        "Style" => match super::ui::style_xml(cfg, &brace::text(&raw(cfg, entry)?)) {
            Ok(x) => Some(vec![("Style.xml".into(), x)]),
            Err(e) => {
                rejected.push(format!("стиль: {e}"));
                None
            }
        },
        "StandaloneConfigurationContent" => match standalone(index, &brace::text(&raw(cfg, entry)?)) {
            Ok(x) => Some(vec![("StandaloneConfigurationContent.bin".into(), x)]),
            Err(e) => {
                rejected.push(format!("состав автономной конфигурации: {e}"));
                None
            }
        },
        "Package" | "ParentConfigurations" | "MobileClientSignature" => Some(vec![(format!("{role}.bin"), raw(cfg, entry)?)]),
        "WSDefinition" => wsdl(&raw(cfg, entry)?, rejected),
        "Help" => html(&cfg.entry_text(entry)?, "Help", &|id| md_path(cfg, id), rejected),
        // Модуль без текста (только «image» — поставлен без исходников) и форма обычного приложения
        // выгружаются вложенным контейнером целиком.
        "Form" => {
            let b = raw(cfg, entry)?;
            container::is_container(&b).then(|| vec![("Form.bin".into(), b)])
        }
        r if r.ends_with("Module") => {
            let b = raw(cfg, entry)?;
            let c = container::read_at(&b, 0).ok().filter(|_| container::is_container(&b))?;
            (c.entries.iter().any(|e| e.name == "image") && !c.entries.iter().any(|e| e.name == "text")).then(|| vec![(format!("{r}.bin"), b)])
        }
        _ => None,
    }
}

/// Файлы, которые пишутся вдобавок к основному файлу записи (его выгружает Exporter): картинки элементов
/// управляемой формы рядом с Form/Module.bsl. У обычной формы (контейнер) картинки внутри Form.bin.
pub fn extra(cfg: &Config, entry: &str, role: &str) -> Vec<(String, Vec<u8>)> {
    if role != "Form" {
        return Vec::new();
    }
    match raw(cfg, entry) {
        Some(b) if !container::is_container(&b) => form_pictures(&brace::text(&b)),
        _ => Vec::new(),
    }
}

/// Состав автономной конфигурации (свойство 4ee1cca2…, backend.dll XML FUN_1806c14a0, корень FUN_1806c2620): запись
/// `{2,N,uuid×N (Used),M,uuid×M (Unused),K,(uuid,приоритет)×K,план обмена,ExchangeOnChangeData,ExchangePeriod,
/// TransactionCount,InactiveNodesCleanupTimeout,L,uuid×L (NoCodeCorrection)}`. Элементы — только найденные объекты, по
/// возрастанию uuid (упорядоченное множество GUID); приоритет 0 Default, 1 MainServer, 2 LocalServer.
fn standalone(index: &super::common::Index, text: &str) -> Result<Vec<u8>, String> {
    let doc = brace::Doc::parse(text).map_err(|e| e.to_string())?;
    let l = doc.roots.first().ok_or("пустая запись")?.list();
    let at = |i: usize| l.get(i).and_then(brace::Node::atom).ok_or_else(|| format!("поле {i}"));
    if at(0)? != "2" {
        return Err(format!("версия {}", at(0)?));
    }
    let mut i = 1;
    let mut list = |pairs: bool| -> Result<Vec<(String, String)>, String> {
        let n: usize = at(i)?.parse().map_err(|_| format!("число в поле {i}"))?;
        i += 1;
        let mut v = Vec::new();
        for _ in 0..n {
            let id = at(i)?.to_string();
            i += 1;
            let p = if pairs {
                i += 1;
                at(i - 1)?.to_string()
            } else {
                String::new()
            };
            v.push((id, p));
        }
        v.sort();
        Ok(v)
    };
    let used = list(false)?;
    let unused = list(false)?;
    let prio = list(true)?;
    let mut w = super::w::W::root("StandaloneContent", "<StandaloneContent xmlns=\"http://v8.1c.ru/8.3/xcf/extrnprops\" xmlns:xs=\"http://www.w3.org/2001/XMLSchema\" xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\" version=\"2.21\">");
    for (tag, items) in [("UsedItem", &used), ("UnusedItem", &unused)] {
        for (id, _) in items {
            if let Some(f) = index.full.get(id) {
                w.open(tag, &[]);
                w.text("Metadata", f);
                w.end();
            }
        }
    }
    for (id, p) in &prio {
        if let Some(f) = index.full.get(id) {
            w.open("PriorityItem", &[]);
            w.text("Metadata", f);
            w.text("Priority", match p.as_str() {
                "0" => "Default",
                "1" => "MainServer",
                "2" => "LocalServer",
                o => return Err(format!("приоритет {o}")),
            });
            w.end();
        }
    }
    let plan = at(i)?;
    if plan != "00000000-0000-0000-0000-000000000000" {
        return Err("план обмена: не сверено".into());
    }
    w.open("DataExchangeSettings", &[]);
    w.text("ExchangeOnChangeData", if at(i + 1)? == "1" { "true" } else { "false" });
    w.text("ExchangePeriod", at(i + 2)?);
    w.text("TransactionCount", at(i + 3)?);
    w.text("InactiveNodesCleanupTimeout", at(i + 4)?);
    if at(i + 5)? != "0" {
        return Err("NoCodeCorrection: не сверено".into());
    }
    w.end();
    Ok(w.finish())
}

fn raw(cfg: &Config, entry: &str) -> Option<Vec<u8>> {
    cfg.index.get(entry).map(|&i| container::content(&cfg.entries[i]))
}

/// `{1,{<прозрачность 0|1>,0,<x>,<y>},{{#base64:...}}}` → `<role>.xml` и `<role>/Picture.<расширение>`.
pub fn picture(text: &str, role: &str) -> Option<Vec<(String, Vec<u8>)>> {
    let n = brace::parse(text).ok()?;
    let p = n.get(1)?;
    let a = |i| p.get(i).and_then(Node::atom);
    if n.get(0)?.atom()? != "1" || a(1)? != "0" {
        return None;
    }
    let data = super::base64(n.get(2)?.get(0)?.get(0)?.atom()?.strip_prefix("#base64:")?).ok()?;
    let ext = picture_ext(&data)?;
    let transparent = match a(0)? {
        "0" => String::new(),
        "1" => format!("\t\t<xr:TransparentPixel x=\"{}\" y=\"{}\"/>\r\n", a(2)?, a(3)?),
        _ => return None,
    };
    let xml = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\r\n<ExtPicture xmlns=\"http://v8.1c.ru/8.3/xcf/extrnprops\" xmlns:xr=\"http://v8.1c.ru/8.3/xcf/readable\" xmlns:xs=\"http://www.w3.org/2001/XMLSchema\" xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\" version=\"2.21\">\r\n\t<Picture>\r\n\t\t<xr:Abs>Picture.{ext}</xr:Abs>\r\n\t\t<xr:LoadTransparent>{}</xr:LoadTransparent>\r\n{transparent}\t</Picture>\r\n</ExtPicture>",
        !transparent.is_empty()
    );
    Some(vec![(format!("{role}.xml"), [BOM, xml.as_bytes()].concat()), (format!("{role}/Picture.{ext}"), data)])
}

/// Расширение файла картинки по сигнатуре — так его называет платформа.
pub(super) fn picture_ext(d: &[u8]) -> Option<&'static str> {
    let head = d.strip_prefix(BOM).unwrap_or(d);
    Some(match d {
        [0x89, b'P', b'N', b'G', ..] => "png",
        [b'P', b'K', 3, 4, ..] => "zip",
        [b'G', b'I', b'F', b'8', ..] => "gif",
        [0xff, 0xd8, 0xff, ..] => "jpg",
        [b'B', b'M', ..] => "bmp",
        [0, 0, 1, 0, ..] => "ico",
        _ if head.starts_with(b"<svg") || head.starts_with(b"<?xml") && String::from_utf8_lossy(&head[..head.len().min(1024)]).contains("<svg") => "svg",
        _ => return None,
    })
}

/// Тип макета из описания: `{1,{2,<тип>,{заголовок}},0}` или `{1,{4,{заголовок},<тип>},0}`.
/// 0 табличный документ, 1 двоичные данные, 3 HTML, 4 текст, 6 схема компоновки, 7 макет оформления СКД,
/// 8 графическая схема, 9 внешняя компонента.
fn template_kind(desc: &str) -> Option<u32> {
    let n = brace::parse(desc).ok()?;
    let l = n.get(1)?;
    match l.get(0)?.atom()? {
        "2" => l.get(1)?.atom()?.parse().ok(),
        "4" => l.get(2)?.atom()?.parse().ok(),
        _ => None,
    }
}

fn template(cfg: &Config, index: &super::common::Index, old: bool, desc: &str, data: Vec<u8>, rejected: &mut Vec<String>) -> Option<Vec<(String, Vec<u8>)>> {
    match template_kind(desc)? {
        0 => match super::mxl::template_data(cfg, &data)? {
            Ok(x) => Some(vec![("Template.xml".into(), x)]),
            Err(e) => {
                rejected.push(format!("табличный документ: {e}"));
                None
            }
        },
        // Текст хранится как есть, с BOM и CRLF.
        4 => Some(vec![("Template.txt".into(), data)]),
        1 | 9 => {
            let n = brace::parse(&brace::text(&data)).ok()?;
            let b64 = n.get(1)?.get(0)?.atom()?.strip_prefix("#base64:")?;
            Some(vec![("Template.bin".into(), super::base64(b64).ok()?)])
        }
        3 => html(&brace::text(&data), "Template", &|id| md_path(cfg, id), rejected),
        // Схема компоновки: ошибка разбора — в отчет, запись остается невыгруженной.
        6 => super::dcs::schema_files(&data, index, old).map_err(|e| rejected.push(e)).ok(),
        7 => super::dcs::appearance(&data).map(|x| vec![("Template.xml".into(), x)]),
        8 => {
            let mut n = brace::parse(&brace::text(&data)).ok()?;
            // схема, сохраненная 8.5: цвета и шрифты в прежние версии
            if let Err(e) = super::form_v59::styles(&mut n) {
                rejected.push(format!("графическая схема: {e}"));
                return None;
            }
            let mut out = Vec::new();
            match super::graph::schema(&n) {
                Some(x) => out.push(("Template.xml".to_string(), x)),
                None => rejected.push("графическая схема: незнакомое устройство, Template.xml не выгружен".into()),
            }
            schema_pictures(&n, &mut None, &mut out);
            Some(out)
        }
        _ => None,
    }
}

/// Имя файла из записи: без разделителей пути, чтобы не выйти за каталог выгрузки.
fn safe(name: &str) -> Option<&str> {
    (!name.is_empty() && name != "." && name != ".." && !name.contains(['/', '\\', ':'])).then_some(name)
}

/// Картинка-значение `{<версия>,3,{0},"",<x>,<y>,<прозрачность>,{{#base64:...}},...}` (у версии 4 еще `0,""`): данные.
pub(super) fn picture_value(n: &Node) -> Option<Vec<u8>> {
    let l = n.list();
    if l.len() < 8 || l[1].atom()? != "3" || l[2].list() != [Node::Atom("0".into())] || l[3].str()? != "" {
        return None;
    }
    super::base64(l[7].get(0)?.get(0)?.atom()?.strip_prefix("#base64:")?).ok()
}

/// Картинка элемента `<dir>/Items/<элемент>/<свойство>.<расширение>`; незнакомую сигнатуру не пишем.
fn push_picture(out: &mut Vec<(String, Vec<u8>)>, dir: &str, item: &str, prop: &str, data: Vec<u8>) {
    if let (Some(item), Some(ext)) = (safe(item), picture_ext(&data)) {
        out.push((format!("{dir}/Items/{item}/{prop}.{ext}"), data));
    }
}

/// Графическая схема: картинка элемента — Template/Items/<имя>/Picture.<расширение>, где имя — из ближайшего
/// предшествующего узла `{<n>,<n>,{1,0},"<имя>",...}` (так у всех 72 картинок БП 3.0.171).
fn schema_pictures(n: &Node, name: &mut Option<String>, out: &mut Vec<(String, Vec<u8>)>) {
    let l = n.list();
    if l.len() > 3 && l[2].list() == [Node::Atom("1".into()), Node::Atom("0".into())] {
        if let Some(s) = l[3].str() {
            *name = Some(s.to_string());
        }
    }
    match picture_value(n) {
        Some(data) => {
            if let Some(item) = name.as_deref() {
                push_picture(out, "Template", item, "Picture", data);
            }
        }
        None => l.iter().for_each(|c| schema_pictures(c, name, out)),
    }
}

/// Запись элемента формы `{<версия>,{<n>,02023637-...},...,"<имя>",...}`; перед ней в списке — GUID типа элемента.
const FORM_ITEM: &str = "02023637-7868-4a5f-8576-835a76e0c9ba";

/// (тип элемента, путь от имени элемента до картинки, свойство): все 220 картинок форм БП 3.0.171.
/// Первый индекс пути — смещение от позиции имени, она у разных версий записи разная.
const FORM_PICTURES: &[(&str, &[usize], &str)] = &[
    ("77ffcc29-7f2d-4223-b22f-19666e7250ba", &[33, 5], "ValuesPicture"), // поле
    ("77ffcc29-7f2d-4223-b22f-19666e7250ba", &[23], "HeaderPicture"),
    ("3d3cb80c-508b-41fa-8a18-680cdf5f1712", &[12, 1], "Picture"), // декорация
    ("a9f3b1ac-f51b-431e-b102-55a69acdecad", &[20], "Picture"), // кнопка
    ("143c00f7-a42d-4cd7-9189-88e4467dc768", &[39], "RowsPicture"), // таблица
    ("cd5394d0-7dda-4b56-8927-93ccbe967a01", &[14, 1], "Picture"), // группа
];

/// Картинки элементов управляемой формы → Form/Items/<элемент>/<свойство>.<расширение>.
/// Картинка в незнакомом месте не выгружается и видна в xml-diff как отсутствующая.
pub fn form_pictures(text: &str) -> Vec<(String, Vec<u8>)> {
    let mut out = Vec::new();
    if text.contains("#base64:") {
        if let Ok(n) = brace::parse(text) {
            form_walk(&n, None, &mut Vec::new(), &mut out);
        }
    }
    out
}

/// `item` — (тип, имя, позиция имени) ближайшей объемлющей записи элемента, `path` — путь от нее.
fn form_walk(n: &Node, item: Option<(&str, &str, usize)>, path: &mut Vec<usize>, out: &mut Vec<(String, Vec<u8>)>) {
    if let Some(data) = picture_value(n) {
        let Some((kind, name, at)) = item else { return };
        let Some(first) = path.first().and_then(|p| p.checked_sub(at)) else { return };
        let rel: Vec<usize> = std::iter::once(first).chain(path[1..].iter().copied()).collect();
        if let Some(&(_, _, prop)) = FORM_PICTURES.iter().find(|(k, p, _)| *k == kind && *p == rel.as_slice()) {
            push_picture(out, "Form", name, prop, data);
        }
        return;
    }
    let l = n.list();
    for (i, c) in l.iter().enumerate() {
        let rec = i
            .checked_sub(1)
            .and_then(|k| l[k].atom())
            .filter(|_| c.get(1).and_then(|x| x.get(1)).and_then(Node::atom) == Some(FORM_ITEM))
            .and_then(|kind| c.list().iter().enumerate().find_map(|(at, x)| x.str().map(|name| (kind, name, at))));
        match rec {
            Some(r) => form_walk(c, Some(r), &mut Vec::new(), out),
            None => {
                path.push(i);
                form_walk(c, item, path, out);
                path.pop();
            }
        }
    }
}

/// HTML-макет и справка `{5,<страниц>,"ru",{#base64:...},...,<вложений>,"имя",1,{#base64:...},...}` →
/// <base>.xml, <base>/<язык>.html (LF вместо CRLF) и <base>/_files/<имя>. Справку без страниц платформа не пишет.
/// `resolve` — путь объекта конфигурации по GUID для ссылок (см. [`links`]). Страница с кодом языка, негодным
/// для имени файла, пропускается (и в <base>.xml тоже) и попадает в `rejected`, остальные выгружаются.
fn html(text: &str, base: &str, resolve: &dyn Fn(&str) -> Option<String>, rejected: &mut Vec<String>) -> Option<Vec<(String, Vec<u8>)>> {
    let n = brace::parse(text).ok()?;
    let l = n.list();
    let b64 = |i: usize| super::base64(l.get(i)?.get(0)?.atom()?.strip_prefix("#base64:")?).ok();
    if l.first()?.atom()? != "5" {
        return None;
    }
    let pages: usize = l.get(1)?.atom()?.parse().ok()?;
    if pages == 0 {
        return Some(Vec::new());
    }
    let mut xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\r\n<Help xmlns=\"http://v8.1c.ru/8.3/xcf/extrnprops\" xmlns:xs=\"http://www.w3.org/2001/XMLSchema\" xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\" version=\"2.21\">\r\n");
    let mut out = Vec::new();
    for k in 0..pages {
        let lang = l.get(2 + 2 * k)?.str()?;
        if !super::safe_name(lang) {
            rejected.push(format!("страница {base} с недопустимым кодом языка {lang:?} пропущена"));
            continue;
        }
        xml.push_str(&format!("\t<Page>{lang}</Page>\r\n"));
        let Ok(page) = String::from_utf8(b64(3 + 2 * k)?) else {
            rejected.push(format!("{base}: страница {lang} не в UTF-8 — не выгружено"));
            return Some(Vec::new());
        };
        let mut page = page.replace("\r\n", "\n");
        // Страницу без этого DOCTYPE (старая разметка `<HTML><HEAD>`) платформа переписывает через DOM (`old_markup`).
        if !page.trim_start_matches('\u{feff}').starts_with(DOCTYPE) {
            match old_markup(&page) {
                Ok(p) => page = p,
                Err(why) => {
                    rejected.push(format!("{base}: страница {lang} в старой разметке без DOCTYPE ({why}), платформа ее переписывает — не выгружено"));
                    return Some(Vec::new());
                }
            }
        }
        // Объявления `xmlns:` у `<html>` платформа пишет по алфавиту префиксов (12 таких страниц БП уже по алфавиту, одну
        // переставила); с другими атрибутами или объявлением без префикса не по порядку — не воспроизводим.
        let Some(page) = html_ns_sort(&page) else {
            rejected.push(format!("{base}: страница {lang}: пространства имен у <html> не по алфавиту, платформа их переставляет — не выгружено"));
            return Some(Vec::new());
        };
        // Ссылки платформа переписывает в атрибутах разобранной страницы: внутри комментария `<!-- -->` они остаются
        // как есть (1 страница БП, `Catalogs/ПодразделенияОрганизаций`: `<A href="../id…">` в комментарии).
        let page = outside_comments(&page, &strip_files_guid);
        let linked = outside_comments(&page, &|s| links(s, resolve));
        // Ссылки в тегах верхнего регистра (`<A href>`) вне комментария не встречены — правило не установлено.
        if linked != page && { let u = uncommented(&page); u.contains("<A ") || u.contains("<IMG ") } {
            rejected.push(format!("{base}: страница {lang} со ссылками в тегах верхнего регистра — не выгружено"));
            return Some(Vec::new());
        }
        out.push((format!("{base}/{lang}.html"), linked.into_bytes()));
    }
    xml.push_str("</Help>");
    out.insert(0, (format!("{base}.xml"), [BOM, xml.as_bytes()].concat()));
    let mut i = 2 + 2 * pages;
    let count: usize = l.get(i)?.atom()?.parse().ok()?;
    i += 1;
    for _ in 0..count {
        let name = safe(l.get(i)?.str()?)?;
        if l.get(i + 1)?.atom()? != "1" {
            return None;
        }
        out.push((format!("{base}/_files/{name}"), b64(i + 2)?));
        i += 3;
    }
    (i == l.len()).then_some(out)
}

const DOCTYPE: &str = "<!DOCTYPE html PUBLIC \"-//W3C//DTD HTML 4.0 Transitional//EN\">";

/// Страница старой разметки (`<HTML><HEAD>…`, без DOCTYPE) так, как ее пишет платформа после разбора в DOM:
/// DOCTYPE, имена тегов в нижнем регистре, значения атрибутов в кавычках, закрывающий тег у каждого элемента
/// (`</meta>`), `&nbsp;` — символом, неоткрытый закрывающий тег (`</FONT>`) отброшен, `<LI>`/`<P>` закрывают
/// незакрытый предыдущий; пробельный текст выброшен в html, head, в начале ul/menu и после ul.
/// Правила — по 7 из 7 таких страниц БП 3.0.171; все, чего в них нет (другие теги, сущности, комментарии,
/// пробельный текст в другом месте, неявное закрытие других тегов), — Err с причиной.
fn old_markup(page: &str) -> Result<String, String> {
    const TAGS: &[&str] = &["html", "head", "meta", "link", "body", "h1", "p", "menu", "ul", "li", "div", "strong", "font", "span"];
    let body = page.strip_prefix('\u{feff}').ok_or("нет BOM")?;
    let mut out = format!("\u{feff}{DOCTYPE}");
    // (тег, последний ребенок: "" — нет, "#" — текст, иначе тег)
    let mut stack: Vec<(String, String)> = Vec::new();
    let mut rest = body;
    let mut done = false;
    while !rest.is_empty() {
        if done {
            return Err("текст после </html>".into());
        }
        if let Some(t) = rest.strip_prefix("</") {
            let n = t.find('>').ok_or("незакрытый тег")?;
            let name = t[..n].trim_end().to_ascii_lowercase();
            rest = &t[n + 1..];
            if stack.last().map(|s| s.0.as_str()) == Some(name.as_str()) {
                stack.pop();
                out.push_str(&format!("</{name}>"));
                done = stack.is_empty();
            } else if stack.iter().any(|s| s.0 == name) {
                return Err(format!("неявное закрытие внутри </{name}>"));
            }
            continue;
        }
        if let Some(t) = rest.strip_prefix('<') {
            let n = t.find('>').ok_or("незакрытый тег")?;
            let (inner, after) = (&t[..n], &t[n + 1..]);
            rest = after;
            let name_len = inner.find(|c: char| !c.is_ascii_alphanumeric()).unwrap_or(inner.len());
            let name = inner[..name_len].to_ascii_lowercase();
            if !TAGS.contains(&name.as_str()) {
                return Err(format!("тег <{}>", &inner[..name_len]));
            }
            let mut attrs = String::new();
            let mut a = &inner[name_len..];
            loop {
                let t = a.trim_start_matches(' ');
                if t.is_empty() {
                    break;
                }
                if t.len() == a.len() {
                    return Err(format!("атрибуты <{name}>"));
                }
                let k = t.find('=').ok_or(format!("атрибут без значения у <{name}>"))?;
                let key = &t[..k];
                if key.is_empty() || !key.bytes().all(|b| b.is_ascii_lowercase() || b == b'-') {
                    return Err(format!("имя атрибута {key:?}"));
                }
                let v = &t[k + 1..];
                let (val, tail, quoted) = match v.strip_prefix('"') {
                    Some(q) => {
                        let e = q.find('"').ok_or("незакрытые кавычки")?;
                        (&q[..e], &q[e + 1..], true)
                    }
                    None => {
                        let e = v.find(' ').unwrap_or(v.len());
                        (&v[..e], &v[e..], false)
                    }
                };
                // `'` в значении в двойных кавычках остается как есть (БП 3.0.107: `FONT-FAMILY: 'Times New Roman'`).
                if val.contains(['&', '<', '>', '"', '\n', '\t']) || (!quoted && val.contains('\'')) {
                    return Err(format!("значение атрибута {key}"));
                }
                attrs.push_str(&format!(" {key}=\"{val}\""));
                a = tail;
            }
            let parent = stack.last().map(|s| s.0.as_str()).unwrap_or("");
            // незакрытый предыдущий <li>/<p> закрывается следующим таким же
            if (name == "li" || name == "p") && parent == name {
                stack.pop();
                out.push_str(&format!("</{name}>"));
            }
            let parent = stack.last().map(|s| s.0.as_str()).unwrap_or("");
            let ok = match name.as_str() {
                "html" => stack.is_empty() && out.len() == 3 + DOCTYPE.len(),
                "head" | "body" => parent == "html",
                "meta" | "link" => parent == "head",
                "li" => parent == "ul" || parent == "menu",
                "h1" | "ul" | "menu" => parent == "body",
                // и внутри `<span>` прямо в `<body>` (БП 3.0.107 СчетаСОсобымПорядкомПереоценкиБУ)
                "p" => parent == "body" || (parent == "span" && stack.len() >= 2 && stack[stack.len() - 2].0 == "body"),
                "div" => parent == "li",
                "span" => matches!(parent, "p" | "h1" | "div" | "li" | "strong" | "font" | "span" | "body"),
                // strong, font
                _ => matches!(parent, "p" | "h1" | "div" | "li" | "strong" | "font" | "span"),
            };
            if !ok {
                return Err(format!("<{name}> внутри <{parent}>"));
            }
            if let Some(top) = stack.last_mut() {
                top.1 = name.clone();
            }
            out.push_str(&format!("<{name}{attrs}>"));
            if name == "meta" || name == "link" {
                out.push_str(&format!("</{name}>"));
            } else {
                stack.push((name, String::new()));
            }
            continue;
        }
        let n = rest.find('<').unwrap_or(rest.len());
        let text = &rest[..n];
        rest = &rest[n..];
        let Some(top) = stack.last_mut() else { return Err("текст вне <html>".into()) };
        if text.chars().all(|c| c == ' ' || c == '\n') {
            let keep = match (top.0.as_str(), top.1.as_str()) {
                ("html" | "head", _) | ("ul" | "menu", "") | ("body", "ul") => false,
                ("body", "" | "h1" | "p") | ("li", "" | "div") => true,
                // `<span>` прямо в `<body>` (СчетаСОсобымПорядкомПереоценкиБУ): в его начале и после него — остается
                ("span", "") | ("body", "span") => true,
                (p, prev) => return Err(format!("пробельный текст в <{p}> после {prev:?}")),
            };
            if keep {
                out.push_str(text);
                top.1 = "#".into();
            }
            continue;
        }
        if matches!(top.0.as_str(), "html" | "head" | "ul" | "menu") || text.contains(['>', '\t']) || text.replace("&nbsp;", "").contains('&') {
            return Err(format!("текст в <{}>", top.0));
        }
        out.push_str(&text.replace("&nbsp;", "\u{a0}"));
        top.1 = "#".into();
    }
    if !done {
        return Err("нет </html>".into());
    }
    Ok(out)
}

/// Префиксы `xmlns:<p>` в теге `<html ...>` идут по алфавиту, объявление без префикса — после них.
/// Не по алфавиту — переставляются, если у `<html>` нет ничего, кроме ` xmlns:<p>="…"` (так у единственной такой
/// страницы БП, `v`,`o` → `o`,`v`); иначе `None`.
fn html_ns_sort(page: &str) -> Option<String> {
    let Some((i, j)) = page.find("<html").and_then(|i| page[i..].find('>').map(|j| (i, i + j))) else { return Some(page.to_string()) };
    let tag = &page[i..j];
    let decl: Vec<&str> = tag.split(" xmlns").skip(1).map(|d| d.strip_prefix(':').map_or("\u{10ffff}", |p| p.split('=').next().unwrap_or(""))).collect();
    if decl.windows(2).all(|w| w[0] <= w[1]) {
        return Some(page.to_string());
    }
    let mut parts: Vec<(&str, &str)> = Vec::new();
    let mut rest = &tag["<html".len()..];
    while !rest.is_empty() {
        let p = rest.strip_prefix(" xmlns:")?;
        let (prefix, v) = p.split_once("=\"").filter(|(x, _)| !x.is_empty() && x.bytes().all(|b| b.is_ascii_alphanumeric()))?;
        let e = v.find('"')?;
        let len = " xmlns:".len() + prefix.len() + 2 + e + 1;
        parts.push((prefix, &rest[..len]));
        rest = &rest[len..];
    }
    parts.sort();
    Some(format!("{}<html{}{}", &page[..i], parts.iter().map(|p| p.1).collect::<String>(), &page[j..]))
}

/// Путь объекта как в ссылках выгрузки: `Catalog.Имя`, `Catalog.Имя.Form.Имя`, `Subsystem.А.Subsystem.Б`.
fn md_path(cfg: &Config, id: &str) -> Option<String> {
    fn find(it: &Item, id: &str) -> Option<String> {
        let me = || format!("{}.{}", it.kind, it.header.name);
        if it.header.id == id {
            return Some(me());
        }
        it.children.iter().find_map(|c| find(c, id)).map(|p| format!("{}.{p}", me()))
    }
    cfg.groups.iter().flat_map(|g| &g.items).find_map(|it| find(it, id))
}

const HELP_PROP: &str = "038b5c85-fb1c-4082-9c4c-e69f8928bf3a";
const NO_GUID: &str = "00000000-0000-0000-0000-000000000000";

/// Стандартные картинки платформы в ссылках HTML: только встреченные в БП 3.0.171 и в схемах компоновки (42), остальные остаются GUID.
pub(super) const STD_PICTURES: &[(&str, &str)] = &[
    ("7a9cd2fd-6372-4342-9a9e-3ebbd754fd83", "AppearanceCheckBox"),
    ("b2202798-23e0-4165-9982-24878f432488", "AppearanceCross"),
    ("0c1f7756-6143-4903-a94c-8f22c85e44de", "Attribute"),
    ("3c904ff7-1195-4a7c-9a38-7b1f6ca49cce", "Back"),
    ("509c4a7f-6406-4388-bb8c-bc81fb5131aa", "BusinessProcess"),
    ("97c5a6d5-47ed-43f9-8c8c-10e9903c23d2", "Calendar"),
    ("97b2cc97-d5c6-45fb-9824-9d6d73db21fe", "Change"),
    ("27ee3053-952c-49e5-8261-9215098e0e9c", "CollapseAll"),
    ("4ab0e87f-7d9b-4aa8-ac4b-680a78522da8", "CreateFolder"),
    ("4d2570b5-205f-413c-b4cc-b2097f61684f", "CreateInitialImage"),
    ("977e831a-0e73-4d60-af51-091a6fa8612e", "CreateListItem"),
    ("ee7c4a5b-2d9b-4087-ae3e-947792085f09", "DataCompositionOutputParameters"),
    ("08a45a70-c221-4339-b3b1-9f11cb22147d", "Delete"),
    ("5289d9a4-b012-4d54-9bce-50473fe29b57", "DialogExclamation"),
    ("894afc03-9904-465d-b671-f555ffb9b21c", "Document"),
    ("544fdbe8-5956-4512-bc62-93b4c022d291", "ExchangePlan"),
    ("003024ed-fa25-42ac-9f53-f5014e383801", "ExecuteTask"),
    ("fb7e9fb5-110b-41cb-adc6-753969ae1c81", "ExpandAll"),
    ("2ef82795-06fe-4365-bd0c-44b486264620", "FilterCriterion"),
    ("1a4342a5-fa06-4556-8a85-e8738fc25821", "GetURL"),
    ("4b54770b-d069-4c0e-9b17-5cc2a01134d9", "Information"),
    ("3d4ad3b1-17de-4cf1-a2e4-0c2c83a5b5c2", "ListViewModeHierarchicalList"),
    ("283ecabd-aaed-41d1-ad46-6cca91c29120", "LoadReportSettings"),
    ("18492a87-2fe4-44af-b218-304897fed020", "MarkToDelete"),
    ("785362cb-3756-48ed-87d2-292ded17054a", "OpenFile"),
    ("64837726-d2a2-4682-a788-737423e80013", "Picture"),
    ("b4c7ab2c-bcda-4468-a28f-5fee93838c4e", "Properties"),
    ("7df3febb-2640-41b7-ad8b-7a23b7ad4aec", "QueryWizardCreateTempTableDropQuery"),
    ("83c8f18d-8701-41f3-bef4-53f88adbb868", "ReadChanges"),
    ("fc4f29e0-d168-4fe0-8e64-e982fabf2595", "Refresh"),
    ("818ab7d0-4654-4542-bd5e-fd9d1352b5a1", "SaveFile"),
    ("b5a0aaba-3a83-4a71-b6f9-24aae1574681", "SaveReportSettings"),
    ("1970a480-9b38-405e-9d9e-8209f3fad5f1", "ScheduledJob"),
    ("58174855-39be-462e-8723-cb2d95182146", "SetDateInterval"),
    ("a064544f-6037-48ca-b19f-8ad63e43af23", "ShowData"),
    ("03665ff1-3a05-41d1-96d3-04bda2d8ede3", "SpreadsheetInsertComment"),
    ("2846af8d-af84-47e3-82b9-01b01f960426", "SpreadsheetReadOnly"),
    ("1cd7b762-ec6a-4e92-ac9a-1832be228ec3", "Stop"),
    ("3bdc16c8-6a96-4467-9442-a8e4804b3fa2", "SyncContents"),
    ("37cf7cc0-abad-4385-b597-6fd2d8dc085a", "Task"),
    ("6ff3ddbd-56e3-4ddf-a5bf-048c1e2dfb2f", "User"),
    ("f62488ee-f90c-47f7-929d-f42ec11a1e63", "WriteChanges"),
];

/// Стандартные картинки по номеру `idn-<номер>`: встреченные в БП 3.0.171.
pub(super) const STD_NUMBERED: &[(&str, &str)] = &[
    ("1", "InputFieldSelect"),
    ("2", "InputFieldClear"),
    ("3", "MoveUp"),
    ("4", "MoveDown"),
    ("5", "InputFieldCalendar"),
    ("7", "InputFieldOpen"),
    ("8", "MoveLeft"),
    ("9", "MoveRight"),
    ("10", "CheckAll"),
    ("11", "UncheckAll"),
    ("13", "Print"),
];

/// Ссылки в HTML: `"../id<объект>/038b5c85-...` (справка объекта) → `"Catalog.Имя/Help`;
/// `"../../mdpicture/id<картинка>/00000000-...` (в макетах `dp`, `mp`) → `"CommonPicture.Имя` или `"StdPicture.Имя`.
/// Ссылку на объект, которого нет в конфигурации, платформа оставляет как есть.
fn links(page: &str, resolve: &dyn Fn(&str) -> Option<String>) -> String {
    let mut out = String::with_capacity(page.len());
    let mut rest = page;
    while let Some(p) = rest.find("id") {
        let (head, tail) = rest.split_at(p);
        match link(head, tail, resolve) {
            Some((cut, s, used)) => {
                out.push_str(&head[..cut]);
                out.push_str(&s);
                rest = &tail[used..];
            }
            None => {
                out.push_str(head);
                out.push_str("id");
                rest = &tail[2..];
            }
        }
    }
    out.push_str(rest);
    out
}

/// `tail` начинается с `id`: (сколько оставить от `head`, замена, сколько байт `tail` она заменяет).
fn link(head: &str, tail: &str, resolve: &dyn Fn(&str) -> Option<String>) -> Option<(usize, String, usize)> {
    let picture = || ["\"../../mdpicture/", "\"../../dp/", "\"../../mp/"].into_iter().find(|x| head.ends_with(x)).map(|x| head.len() - x.len() + 1);
    // Стандартная картинка по номеру: `"../../mdpicture/idn-1"`.
    if let Some(num) = tail.strip_prefix("idn-") {
        let d = num.bytes().take_while(u8::is_ascii_digit).count();
        let name = match STD_NUMBERED.iter().find(|(n, _)| num.get(..d) == Some(*n)) {
            Some(p) => p.1,
            None => crate::xml::imp::form::platform::std_picture(&format!("-{}", num.get(..d)?))?,
        };
        (num.as_bytes().get(d) == Some(&b'"')).then_some(())?;
        return Some((picture()?, format!("StdPicture.{name}"), 4 + d));
    }
    let (obj, prop) = (tail.get(2..38)?, tail.get(39..75)?);
    if !is_guid(obj) || tail.as_bytes()[38] != b'/' || !is_guid(prop) {
        return None;
    }
    if prop == HELP_PROP && head.ends_with("\"../") {
        return Some((head.len() - 3, format!("{}/Help", resolve(obj)?), 75));
    }
    if prop != NO_GUID {
        return None;
    }
    let cut = picture()?;
    let name = match resolve(obj) {
        Some(m) if m.starts_with("CommonPicture.") => m,
        // незнакомая стандартная картинка: имени не угадать — метка (страница не выгружается)
        _ => match STD_PICTURES.iter().find(|(g, _)| *g == obj).map(|p| p.1).or_else(|| crate::xml::imp::form::platform::std_picture(obj)) {
            Some(n) => format!("StdPicture.{n}"),
            // картинки нет ни в конфигурации, ни среди стандартных (GUID нет в файлах платформы): ссылка как есть
            None if super::form::NOT_STD_PICTURES.contains(&obj) => return None,
            None => format!("{}{obj}", super::UNK),
        },
    };
    Some((cut, name, 75))
}

/// Платформа пишет ссылки на вложения `<guid>_files/x.png` как `_files/x.png`.
fn strip_files_guid(page: &str) -> String {
    let mut out = String::with_capacity(page.len());
    let mut rest = page;
    while let Some(p) = rest.find("_files/") {
        let head = &rest[..p];
        let cut = if head.len() >= 36 && head.is_char_boundary(head.len() - 36) && is_guid(&head[head.len() - 36..]) { head.len() - 36 } else { head.len() };
        out.push_str(&head[..cut]);
        out.push_str("_files/");
        rest = &rest[p + 7..];
    }
    out.push_str(rest);
    out
}

/// `f` — к частям страницы вне комментариев `<!-- ... -->`; комментарии (и незакрытый до конца) — как есть.
fn outside_comments(page: &str, f: &dyn Fn(&str) -> String) -> String {
    let mut out = String::with_capacity(page.len());
    let mut rest = page;
    while let Some(p) = rest.find("<!--") {
        out.push_str(&f(&rest[..p]));
        let end = rest[p + 4..].find("-->").map_or(rest.len(), |e| p + 4 + e + 3);
        out.push_str(&rest[p..end]);
        rest = &rest[end..];
    }
    out.push_str(&f(rest));
    out
}

/// Страница без комментариев.
fn uncommented(page: &str) -> String {
    let mut out = String::new();
    let mut rest = page;
    while let Some(p) = rest.find("<!--") {
        out.push_str(&rest[..p]);
        rest = rest[p + 4..].find("-->").map_or("", |e| &rest[p + 4 + e + 3..]);
    }
    out.push_str(rest);
    out
}

/// Вложенный контейнер `0.wsdl`, `1.xsd`, ...: схемы — как есть, `0.wsdl` → WSDefinition.xml (`wsdl_definition`).
fn wsdl(b: &[u8], rejected: &mut Vec<String>) -> Option<Vec<(String, Vec<u8>)>> {
    if !container::is_container(b) {
        return None;
    }
    let c = container::read_at(b, 0).ok()?;
    let xsds: Vec<&[u8]> = c.entries.iter().filter(|e| e.name.ends_with(".xsd")).map(|e| e.data.as_slice()).collect();
    let mut out = Vec::new();
    for e in &c.entries {
        match e.name.as_str() {
            "0.wsdl" => match std::str::from_utf8(&e.data).map_err(|_| "не UTF-8".to_string()).and_then(|t| wsdl_definition(t, &xsds)) {
                Ok(x) => out.push(("WSDefinition.xml".into(), [BOM, x.as_bytes()].concat())),
                Err(why) => rejected.push(format!("WSDefinition.xml: платформа заново сериализует WSDL, а здесь {why} — не восстановлено")),
            },
            n if n.ends_with(".xsd") => out.push((safe(n)?.to_string(), e.data.clone())),
            _ => return None,
        }
    }
    Some(out)
}

/// WSDL так, как его пишет платформа (1 WS-ссылка БП 3.0.171; записанный WSDL — уже в ее разметке): префиксы
/// `xmlns:xsdN` корня перенумерованы по алфавиту URI, а во вложенной `<xsd:schema>` объявлены прежние привязки этих
/// префиксов (в конец объявлений, они там по алфавиту префиксов). Корень — ws.dll FUN_18005f0e0: привязки "" (wsdl),
/// tns (при непустом targetNamespace), soapbind, soap12bind, xsd, wsp, wsu, insN (импорты WSDL) и xsdN — пространства
/// схем (targetNamespace, импорты) в std::set с порядком core::compare (FUN_180066650), префикс "xsd" и номер с 1;
/// схему пишет DOM-узлом FUN_180061ce0. Корень сверяется с этим составом. Корень уже по алфавиту URI — перенумерация
/// тождественна, текст как есть (что прочее при записи неподвижно — shortcut ниже). Все, что сверх этих случаев (при
/// перенумерации префикс xsdN используется в тексте, схем не одна, привязка части префиксов не меняется; импорт WSDL;
/// xsdN вне `<types>`; другая разметка), — Err с причиной. `xsds` — внешние схемы контейнера (1.xsd, …).
fn wsdl_definition(text: &str, xsds: &[&[u8]]) -> Result<String, String> {
    let decl = |l: &str, indent: &str| -> Option<(String, String)> {
        let (p, u) = l.strip_prefix(indent)?.strip_prefix("xmlns:")?.split_once("=\"")?;
        Some((p.to_string(), u.strip_suffix('"')?.to_string()))
    };
    let mut lines: Vec<String> = text.split("\r\n").map(str::to_string).collect();
    if lines.first().map(String::as_str) != Some("<?xml version=\"1.0\" encoding=\"UTF-8\"?>") || !lines.get(1).is_some_and(|l| l.starts_with("<definitions xmlns=\"")) {
        return Err("незнакомый заголовок".into());
    }
    let is_xsd_n = |p: &str| p.strip_prefix("xsd").is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()));
    let root: Vec<usize> = (2..lines.len()).take_while(|&i| decl(&lines[i], "\t\t").is_some()).filter(|&i| decl(&lines[i], "\t\t").is_some_and(|d| is_xsd_n(&d.0))).collect();
    let old: Vec<(String, String)> = root.iter().filter_map(|&i| decl(&lines[i], "\t\t")).collect();
    if old.is_empty() || old.iter().enumerate().any(|(k, d)| d.0 != format!("xsd{}", k + 1)) || root.windows(2).any(|w| w[1] != w[0] + 1) {
        return Err("нет префиксов xsd1…xsdN подряд".into());
    }
    let xsd_uses = |s: &str| s.match_indices("xsd").filter(|(i, _)| s.as_bytes().get(i + 3).is_some_and(u8::is_ascii_digit)).count();
    let uses = xsd_uses(text);
    // Корень — тот, что пишет FUN_18005f0e0: XML-писатель выводит привязки по алфавиту префиксов, затем name и
    // targetNamespace (оба — только непустые).
    let doc = dom::parse(text)?;
    let fixed = [
        ("xmlns", "http://schemas.xmlsoap.org/wsdl/"),
        ("xmlns:soap12bind", "http://schemas.xmlsoap.org/wsdl/soap12/"),
        ("xmlns:soapbind", "http://schemas.xmlsoap.org/wsdl/soap/"),
        ("xmlns:tns", doc.peek_attr("targetNamespace").unwrap_or("")),
        ("xmlns:wsp", "http://schemas.xmlsoap.org/ws/2004/09/policy"),
        ("xmlns:wsu", "http://docs.oasis-open.org/wss/2004/01/oasis-200401-wss-wssecurity-utility-1.0.xsd"),
        ("xmlns:xsd", "http://www.w3.org/2001/XMLSchema"),
    ];
    let want: Vec<(String, String)> = fixed.iter().filter(|f| !f.1.is_empty()).map(|(p, u)| (p.to_string(), u.to_string())).chain(old.iter().map(|(p, u)| (format!("xmlns:{p}"), u.clone()))).collect();
    let rest: Vec<&str> = doc.attrs.iter().skip(want.len()).map(|a| if a.1.is_empty() { "" } else { a.0.as_str() }).collect();
    if doc.attrs.get(..want.len()) != Some(&want[..]) || !matches!(rest[..], [] | ["name"] | ["targetNamespace"] | ["name", "targetNamespace"]) {
        return Err("корень не тот, что пишет платформа (wsdl, soap12bind, soapbind, tns, wsp, wsu, xsd, xsdN)".into());
    }
    if doc.peek_child("import").is_some() {
        return Err("импорт WSDL (префиксы insN)".into());
    }
    // Множество xsdN (там же): targetNamespace схем <types> и namespace их импортов (узлы вида 0x1a); третий
    // источник — список интерфейса DAT_1800a7a00, у каждого слот +0xa8, тот же, что targetNamespace схемы.
    // shortcut: третий источник не опознан и считается внешними схемами (1.xsd, …); пространство, которое они
    // добавляют сверх <types>, — Err; пересмотреть на образце, где внешняя схема добавляет пространство.
    let mut ns = BTreeSet::new();
    for s in doc.peek_child("types").into_iter().flat_map(|t| t.peek_elems()) {
        schema_ns(s, &mut ns)?;
    }
    let mut ext = BTreeSet::new();
    for x in xsds {
        schema_ns(&dom::parse(std::str::from_utf8(x).map_err(|_| "внешняя схема не UTF-8")?)?, &mut ext)?;
    }
    let mut uris: Vec<&str> = old.iter().map(|d| d.1.as_str()).collect();
    uris.sort();
    if !uris.iter().copied().eq(ns.iter().map(String::as_str)) || !ext.is_subset(&ns) {
        return Err("пространства xsdN корня — не пространства схем".into());
    }
    // привязка корня, повторенная во вложенном элементе, при записи пропала бы (shortcut ниже) — такой текст не
    // разметка платформы
    fn repeats(e: &dom::El, root: &[(String, String)]) -> bool {
        e.peek_elems().any(|c| c.attrs.iter().any(|a| a.0.starts_with("xmlns") && root.contains(a)) || repeats(c, root))
    }
    if repeats(&doc, &want) {
        return Err("вложенный элемент повторяет привязку корня".into());
    }
    // Уже в разметке платформы (WSDL из загрузки XML 8.5 — так его записывает и выгружает платформа: WSСборОтчетностиРосстата
    // БП, загруженной 8.5 из XML): корень по алфавиту URI, схема сама объявляет xsd1…xsdN — повторная разметка ничего
    // не меняет, текст как есть.
    let local: Vec<(String, String)> = lines.iter().filter_map(|l| decl(l, "\t\t\t\t")).filter(|d| is_xsd_n(&d.0)).collect();
    let sorted = old.windows(2).all(|w| w[0].1 < w[1].1);
    if !local.is_empty() {
        let same = local.len() == old.len() && local.iter().enumerate().all(|(k, d)| d.0 == format!("xsd{}", k + 1));
        if sorted && same && uses == 2 * old.len() && text.matches("<xsd:schema ").count() == 1 {
            return Ok(text.to_string());
        }
        return Err("схема объявляет префиксы xsdN, а корень не по алфавиту URI".into());
    }
    // Корень уже по алфавиту URI: перенумерация тождественна, привязки корня не меняются — текст как есть
    // (UpdateFilesApiImplService ERP 2.5.22: `type="xsd1:…"` в `<xs:schema>`). Схему ws.dll читает из потока с
    // унаследованным контекстом (FUN_180033230 → FUN_1800373e0), а пишет DOM-узлом: xml2.dll FUN_1800f8810 отдает
    // каждый атрибут xmlns в ЗаписатьСоответствиеПространстваИмен (+0x70).
    // shortcut: что запись пропускает привязку, уже действующую в контексте (унаследованные soapbind, xsd корня не
    // повторены в схеме WSСборОтчетностиРосстата БП 3.0.171, xsd1 — в UpdateFilesApiImplService), выведено по этим
    // двум образцам, а не из кода записи xml2.dll; пересмотреть, если корень по алфавиту, а эталон разойдется.
    // shortcut: что прочее (элементы вне корня и <types>, их разметка, QName сообщений и привязок с tns) при записи
    // неподвижно, выведено по тем же двум образцам — писатель сообщений, привязок и политик не разобран; пересмотреть,
    // если корень по алфавиту, а эталон разойдется.
    if sorted {
        // xsdN вне <types> — отказ: QName сообщений и привязок пишет сам писатель WSDL, а какой префикс он берет для
        // URI, общего у tns и xsdN, не выведено (в UpdateFilesApiImplService — tns)
        let (a, z) = (text.find("\r\n\t<types>").unwrap_or(0), text.find("\r\n\t</types>").unwrap_or(0));
        if uses != old.len() + xsd_uses(&text[a..z.max(a)]) {
            return Err("префикс xsdN вне <types>".into());
        }
        return Ok(text.to_string());
    }
    if uses != old.len() || text.matches("<xsd:schema ").count() != 1 {
        return Err("префиксы xsdN в тексте или схем не одна".into());
    }
    if old.iter().zip(&uris).any(|(d, u)| d.1 == *u) {
        return Err("привязка части префиксов xsdN не меняется".into());
    }
    for (k, (&i, u)) in root.iter().zip(&uris).enumerate() {
        lines[i] = format!("\t\txmlns:xsd{}=\"{u}\"", k + 1);
    }
    let s = lines.iter().position(|l| l.starts_with("\t\t<xsd:schema xmlns:")).ok_or("нет <xsd:schema> в types")?;
    let mut prefixes = vec![lines[s]["\t\t<xsd:schema ".len()..].strip_prefix("xmlns:").and_then(|x| x.split_once('=')).ok_or("разметка <xsd:schema>")?.0.to_string()];
    let mut end = s + 1;
    while let Some((p, _)) = lines.get(end).and_then(|l| decl(l, "\t\t\t\t")) {
        prefixes.push(p);
        end += 1;
    }
    if prefixes.windows(2).any(|w| w[0] >= w[1]) || prefixes.last().is_some_and(|p| p.as_str() >= "xsd1") {
        return Err("объявления <xsd:schema> не по алфавиту".into());
    }
    let added: Vec<String> = old.iter().map(|(p, u)| format!("\t\t\t\txmlns:{p}=\"{u}\"")).collect();
    lines.splice(end..end, added);
    Ok(lines.join("\r\n"))
}

/// Пространства схемы для xsdN корня WSDL (ws.dll FUN_18005f0e0): непустой targetNamespace и namespace импортов.
/// Включение и переопределение (импорты подключенной схемы) не разобраны — Err.
fn schema_ns(s: &dom::El, out: &mut BTreeSet<String>) -> Result<(), String> {
    if s.name.rsplit(':').next() != Some("schema") {
        return Err(format!("<{}> вместо схемы", s.name));
    }
    out.extend(s.peek_attr("targetNamespace").filter(|t| !t.is_empty()).map(str::to_string));
    for e in s.peek_elems() {
        match e.name.rsplit(':').next() {
            Some("import") => {
                out.insert(e.peek_attr("namespace").ok_or("импорт схемы без namespace")?.to_string());
            }
            Some("include" | "redefine") => return Err("включение схемы".into()),
            _ => {}
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn picture_transparent_pixel() {
        let f = picture("{1,\r\n{1,0,10,7},\r\n{\r\n{#base64:iVBORw0K\r\nGgo=}\r\n}\r\n}", "Picture").unwrap();
        assert_eq!(f[1], ("Picture/Picture.png".to_string(), b"\x89PNG\r\n\x1a\n".to_vec()));
        let xml = String::from_utf8(f[0].1.clone()).unwrap();
        assert!(xml.contains("<xr:LoadTransparent>true</xr:LoadTransparent>\r\n\t\t<xr:TransparentPixel x=\"10\" y=\"7\"/>\r\n\t</Picture>"), "{xml}");
        assert!(picture("{1,{0,1,-1,-1},{{#base64:iVBORw0KGgo=}}}", "Picture").is_none());
    }

    #[test]
    fn picture_kinds() {
        assert_eq!(picture_ext(b"PK\x03\x04"), Some("zip"));
        assert_eq!(picture_ext(b"\xef\xbb\xbf<svg x"), Some("svg"));
        assert_eq!(picture_ext(b"<?xml version=\"1.0\"?><svg/>"), Some("svg"));
        assert_eq!(picture_ext(b"<?xml version=\"1.0\"?><html/>"), None);
    }

    #[test]
    fn template_kinds() {
        assert_eq!(template_kind("{1,{2,4,{2,{1,0,a9059390-f267-49b1-875b-1ecdb98d0153},\"Т\",{1,\"ru\",\"Т\"},\"\",0,0,00000000-0000-0000-0000-000000000000}},0}"), Some(4));
        assert_eq!(template_kind("{1,{4,{3,{1,0,a9059390-f267-49b1-875b-1ecdb98d0153},\"Т\",{1,\"ru\",\"Т\"},\"\",0,0,00000000-0000-0000-0000-000000000000,0},9},0}"), Some(9));
    }

    #[test]
    fn html_with_files() {
        // <!DOCTYPE html PUBLIC "-//W3C//DTD HTML 4.0 Transitional//EN"><img src="8eb4fad1-1fa6-403e-970f-2c12dbb43e23_files/a.png">\r\n
        let page = "PCFET0NUWVBFIGh0bWwgUFVCTElDICItLy9XM0MvL0RURCBIVE1MIDQuMCBUcmFuc2l0aW9uYWwvL0VOIj48aW1nIHNyYz0iOGViNGZhZDEtMWZhNi00MDNlLTk3MGYtMmMxMmRiYjQzZTIzX2ZpbGVzL2EucG5nIj4NCg==";
        let f = html(&format!("{{5,1,\"ru\",\r\n{{#base64:{page}}},1,\"a.png\",1,\r\n{{#base64:iVBORw0KGgo=}}\r\n}}"), "Template", &|_| None, &mut Vec::new()).unwrap();
        let names: Vec<&str> = f.iter().map(|x| x.0.as_str()).collect();
        assert_eq!(names, ["Template.xml", "Template/ru.html", "Template/_files/a.png"]);
        assert_eq!(f[1].1, b"<!DOCTYPE html PUBLIC \"-//W3C//DTD HTML 4.0 Transitional//EN\"><img src=\"_files/a.png\">\n");
        // недопустимый код языка пропускает только свою страницу; pt-BR — обычный код
        let mut rejected = Vec::new();
        let f = html(&format!("{{5,2,\"../x\",{{#base64:{page}}},\"pt-BR\",{{#base64:{page}}},0}}"), "Help", &|_| None, &mut rejected).unwrap();
        let names: Vec<&str> = f.iter().map(|x| x.0.as_str()).collect();
        assert_eq!(names, ["Help.xml", "Help/pt-BR.html"]);
        let xml = String::from_utf8_lossy(&f[0].1);
        assert!(xml.contains("<Page>pt-BR</Page>") && !xml.contains("../x"));
        assert_eq!(rejected.len(), 1);
        // старая разметка без DOCTYPE и ссылка в теге верхнего регистра — отказ от всей справки
        let resolve = |_: &str| Some("Catalog.К".to_string());
        for p in ["PEhUTUw+PEEgaHJlZj0iLi4vaWQ0NGNkNWU2My04OTBmLTQzMWQtODRkNC1iY2UwMDg5ODEzNWEvMDM4YjVjODUtZmIxYy00MDgyLTljNGMtZTY5Zjg5MjhiZjNhIj4=", "PCFET0NUWVBFIGh0bWwgUFVCTElDICItLy9XM0MvL0RURCBIVE1MIDQuMCBUcmFuc2l0aW9uYWwvL0VOIj48QSBocmVmPSIuLi9pZDQ0Y2Q1ZTYzLTg5MGYtNDMxZC04NGQ0LWJjZTAwODk4MTM1YS8wMzhiNWM4NS1mYjFjLTQwODItOWM0Yy1lNjlmODkyOGJmM2EiPg=="] {
            let mut rejected = Vec::new();
            assert!(html(&format!("{{5,1,\"ru\",{{#base64:{p}}},0}}"), "Help", &resolve, &mut rejected).unwrap().is_empty());
            assert_eq!(rejected.len(), 1);
        }
    }

    #[test]
    fn old_markup_rewrite() {
        let src = "\u{feff}<HTML><HEAD>\n<META http-equiv=Content-Type content=\"text/html; charset=utf-8\"><LINK href=\"v8help://s\" type=text/css rel=stylesheet>\n<META content=\"MSHTML 6\" name=GENERATOR></HEAD>\n<BODY>\n<H1 class=\"\">А&nbsp;Б</H1>\n<P class=Usual>В <FONT face=Symbol size=2>·</FONT> Г\n<P class=Usual>Д</FONT></P>\n<UL>\n<LI>\n<DIV class=\"\"><STRONG>Е</STRONG>;</DIV>\n<LI>\n<DIV class=\"\">Ж</DIV></LI></UL>\n<P>З</P></BODY></HTML>";
        assert_eq!(
            old_markup(src).unwrap(),
            format!("\u{feff}{DOCTYPE}<html><head><meta http-equiv=\"Content-Type\" content=\"text/html; charset=utf-8\"></meta><link href=\"v8help://s\" type=\"text/css\" rel=\"stylesheet\"></link><meta content=\"MSHTML 6\" name=\"GENERATOR\"></meta></head><body>\n<h1 class=\"\">А\u{a0}Б</h1>\n<p class=\"Usual\">В <font face=\"Symbol\" size=\"2\">·</font> Г\n</p><p class=\"Usual\">Д</p>\n<ul><li>\n<div class=\"\"><strong>Е</strong>;</div>\n</li><li>\n<div class=\"\">Ж</div></li></ul><p>З</p></body></html>")
        );
        // незнакомое — отказ: другой тег, сущность, неявное закрытие, нет BOM
        for bad in ["<HTML><BODY><BR></BODY></HTML>", "<HTML><BODY><P>&lt;</P></BODY></HTML>", "<HTML><BODY><P>а</BODY></HTML>"] {
            assert!(old_markup(&format!("\u{feff}{bad}")).is_err(), "{bad}");
        }
        assert!(old_markup("<HTML></HTML>").is_err());
    }

    #[test]
    fn wsdl_prefixes_renumbered() {
        let head = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\r\n<definitions xmlns=\"http://schemas.xmlsoap.org/wsdl/\"\r\n\t\txmlns:soap12bind=\"http://schemas.xmlsoap.org/wsdl/soap12/\"\r\n\t\txmlns:soapbind=\"http://schemas.xmlsoap.org/wsdl/soap/\"\r\n\t\txmlns:wsp=\"http://schemas.xmlsoap.org/ws/2004/09/policy\"\r\n\t\txmlns:wsu=\"http://docs.oasis-open.org/wss/2004/01/oasis-200401-wss-wssecurity-utility-1.0.xsd\"\r\n\t\txmlns:xsd=\"http://www.w3.org/2001/XMLSchema\"\r\n";
        let body = "\t\tname=\"S\">\r\n\t<types>\r\n\t\t<xsd:schema xmlns:ns1=\"u:a\"\r\n\t\t\t\txmlns:tns=\"u:b\"\r\n\t\t\t\ttargetNamespace=\"u:b\">\r\n\t\t\t<xsd:import namespace=\"u:a\"/>\r\n\t\t</xsd:schema>\r\n\t</types>\r\n</definitions>";
        let src = format!("{head}\t\txmlns:xsd1=\"u:b\"\r\n\t\txmlns:xsd2=\"u:a\"\r\n{body}");
        let w = |t: &str| wsdl_definition(t, &[]);
        assert_eq!(
            w(&src).unwrap(),
            format!("{head}\t\txmlns:xsd1=\"u:a\"\r\n\t\txmlns:xsd2=\"u:b\"\r\n{}", body.replace("\"u:b\"\r\n\t\t\t\ttargetNamespace", "\"u:b\"\r\n\t\t\t\txmlns:xsd1=\"u:b\"\r\n\t\t\t\txmlns:xsd2=\"u:a\"\r\n\t\t\t\ttargetNamespace"))
        );
        // корень уже по алфавиту — как есть, и с xsdN в <xsd:schema>; xsdN вне <types> — отказ
        let sorted = format!("{head}\t\txmlns:xsd1=\"u:a\"\r\n\t\txmlns:xsd2=\"u:b\"\r\n{body}");
        assert_eq!(w(&sorted).unwrap(), sorted);
        let typed = sorted.replace("\"u:a\"/>\r\n", "\"u:a\"/>\r\n\t\t\t<xsd:element name=\"E\"\r\n\t\t\t\t\ttype=\"xsd1:T\"/>\r\n");
        assert_eq!(w(&typed).unwrap(), typed);
        assert!(w(&typed.replace("\t</types>\r\n", "\t</types>\r\n\t<message name=\"M\">\r\n\t\t<part name=\"p\"\r\n\t\t\t\telement=\"xsd1:E\"/>\r\n\t</message>\r\n")).is_err());
        assert!(w(&sorted.replace("name=\"S\"", "name=\"xsd1:S\"")).is_err());
        // повтор привязки корня во вложенном элементе — в строке тега и глубже — отказ
        assert!(w(&sorted.replace("<xsd:schema xmlns:ns1", "<xsd:schema xmlns:xsd=\"http://www.w3.org/2001/XMLSchema\"\r\n\t\t\t\txmlns:ns1")).is_err());
        assert!(w(&sorted.replace("<xsd:import ", "<xsd:import xmlns:soapbind=\"http://schemas.xmlsoap.org/wsdl/soap/\" ")).is_err());
        // корень не тот, что пишет платформа: нет wsp, лишняя привязка, tns без targetNamespace; импорт WSDL — отказ
        assert!(w(&sorted.replace("\t\txmlns:wsp=\"http://schemas.xmlsoap.org/ws/2004/09/policy\"\r\n", "")).is_err());
        assert!(w(&sorted.replace("\t\txmlns:wsu=", "\t\txmlns:wsa=\"a\"\r\n\t\txmlns:wsu=")).is_err());
        let tns = sorted.replace("\t\txmlns:wsp=", "\t\txmlns:tns=\"u:t\"\r\n\t\txmlns:wsp=");
        assert!(w(&tns).is_err());
        let tns = tns.replace("name=\"S\"", "name=\"S\"\r\n\t\ttargetNamespace=\"u:t\"");
        assert_eq!(w(&tns).unwrap(), tns);
        // tns корня совпадает с tns схемы (JAX-WS) — повтор привязки
        assert!(w(&tns.replace("u:t", "u:b")).is_err());
        assert!(w(&sorted.replace("\t<types>", "\t<import namespace=\"u:i\"\r\n\t\t\tlocation=\"l\"/>\r\n\t<types>")).is_err());
        // пространства xsdN — не пространства схем: нет импорта; внешняя схема добавляет пространство
        assert!(w(&sorted.replace("\t\t\t<xsd:import namespace=\"u:a\"/>\r\n", "")).is_err());
        assert!(wsdl_definition(&sorted, &[b"<xs:schema xmlns:xs=\"x\"\r\n\t\ttargetNamespace=\"u:c\"/>".as_slice()]).is_err());
        assert!(wsdl_definition(&sorted, &[b"<xs:schema xmlns:xs=\"x\"\r\n\t\ttargetNamespace=\"u:a\"/>".as_slice()]).is_ok());
        // при перенумерации xsdN в тексте — отказ; часть привязок на месте — отказ
        assert!(w(&src.replace("name=\"S\"", "name=\"xsd1:S\"")).is_err());
        let part = src.replace("\t\txmlns:xsd2=\"u:a\"\r\n", "\t\txmlns:xsd2=\"u:a\"\r\n\t\txmlns:xsd3=\"u:c\"\r\n").replace("\"u:a\"/>\r\n", "\"u:a\"/>\r\n\t\t\t<xsd:import namespace=\"u:c\"/>\r\n");
        assert_eq!(w(&part).unwrap_err(), "привязка части префиксов xsdN не меняется");
        // разметка платформы (WSDL из загрузки XML 8.5) — неподвижная точка
        let done = w(&src).unwrap();
        assert_eq!(w(&done).unwrap(), done);
    }

    #[test]
    fn links_in_comments_kept() {
        let g = "44cd5e63-890f-431d-84d4-bce00898135a";
        let resolve = |_: &str| Some("Catalog.К".to_string());
        let page = format!("<a href=\"../id{g}/{HELP_PROP}\">x</a><!--<A href=\"../id{g}/{HELP_PROP}\">--><a href=\"../id{g}/{HELP_PROP}\">");
        assert_eq!(
            outside_comments(&page, &|s| links(s, &resolve)),
            format!("<a href=\"Catalog.К/Help\">x</a><!--<A href=\"../id{g}/{HELP_PROP}\">--><a href=\"Catalog.К/Help\">")
        );
        assert_eq!(uncommented(&page), format!("<a href=\"../id{g}/{HELP_PROP}\">x</a><a href=\"../id{g}/{HELP_PROP}\">"));
    }

    #[test]
    fn html_namespaces_order() {
        let same = |p: &str| html_ns_sort(p).as_deref() == Some(p);
        assert!(same("<html xmlns:o=\"a\" xmlns:v=\"b\" xmlns:w=\"c\" xmlns=\"d\"><p>"));
        assert!(same("<p>без html"));
        assert_eq!(html_ns_sort("<!D><html xmlns:v=\"b\" xmlns:st1=\"c\" xmlns:o=\"a\"><head>").as_deref(), Some("<!D><html xmlns:o=\"a\" xmlns:st1=\"c\" xmlns:v=\"b\"><head>"));
        // объявление без префикса или другой атрибут не по порядку — отказ
        assert_eq!(html_ns_sort("<html xmlns=\"d\" xmlns:o=\"a\">"), None);
        assert_eq!(html_ns_sort("<html xmlns:v=\"b\" lang=\"ru\" xmlns:o=\"a\">"), None);
    }

    #[test]
    fn html_links() {
        let g = "44cd5e63-890f-431d-84d4-bce00898135a";
        let resolve = |id: &str| (id == "44cd5e63-890f-431d-84d4-bce00898135a").then(|| "Report.Р".to_string());
        let page = format!(
            "<a href=\"../id{g}/{HELP_PROP}#Я\">x</a><a href=\"../id{NO_GUID}/{HELP_PROP}\">x</a><img src=\"../../mdpicture/id4d2570b5-205f-413c-b4cc-b2097f61684f/{NO_GUID}\"><img src=\"../../mdpicture/idn-13\">id"
        );
        assert_eq!(
            links(&page, &resolve),
            format!("<a href=\"Report.Р/Help#Я\">x</a><a href=\"../id{NO_GUID}/{HELP_PROP}\">x</a><img src=\"StdPicture.CreateInitialImage\"><img src=\"StdPicture.Print\">id")
        );
    }

    #[test]
    fn form_item_pictures() {
        // кнопка: имя на позиции 5, картинка через 20; вторая картинка — в незнакомом месте
        let pic = "{4,3,{0},\"\",5,3,1,{{#base64:R0lGODlh}},0,\"\"}";
        let t = format!(
            "{{1,{{a9f3b1ac-f51b-431e-b102-55a69acdecad,{{31,{{7,02023637-7868-4a5f-8576-835a76e0c9ba}},0,0,1,\"Кн\",{}{pic},{pic}}}}}}}",
            "0,".repeat(19)
        );
        let t = t.as_str();
        assert_eq!(form_pictures(t), [("Form/Items/Кн/Picture.gif".to_string(), b"GIF89a".to_vec())]);
    }

    #[test]
    fn schema_item_pictures() {
        let t = "{5,{{4,1,{1,0},\"Фон\",1},{6,0},{3,3,{0},\"\",-1,-1,0,{{#base64:R0lGODlh}}}},{{4,1,{1,0},\"Пусто\",1}}}";
        let mut out = Vec::new();
        schema_pictures(&brace::parse(t).unwrap(), &mut None, &mut out);
        assert_eq!(out, [("Template/Items/Фон/Picture.gif".to_string(), b"GIF89a".to_vec())]);
    }
}
