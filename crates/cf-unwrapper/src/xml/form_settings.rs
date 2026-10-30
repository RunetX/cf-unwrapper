//! Настройки реквизита формы, которые несут значение платформенного типа: табличный документ (Moxel).
//! Подмодуль `form` (видит его закрытые типы); раскладка — docs/research/xml-form.md, раздел «Проход r8-f2».

use super::{Names, W};
use crate::brace::Node;
use crate::xml::mxl;

const MXL_NS: &str = "http://v8.1c.ru/8.2/data/spreadsheet";

/// Табличный документ `{"#",e603103e-...,{8,1,версия,...}}` → `<Settings xsi:type="mxl:SpreadsheetDocument">` с тем же
/// содержимым, что у Template.xml макета, но с префиксом `mxl:`. Err — причина отказа.
pub(super) fn moxel(w: &mut W, names: &Names, v: &Node) -> Result<(), String> {
    if v.get(0).and_then(Node::str) != Some("#") || v.get(1).and_then(Node::atom) != Some("e603103e-a318-4edc-a014-b1c6cf94d49f") || v.list().len() != 3 {
        return Err("не табличный документ".into());
    }
    let lookup = |class: &str, id: &str| match class {
        "Type" => names.types.get(id).cloned(),
        _ => names.full.get(id).filter(|f| f.split('.').next() == Some(class)).map(|f| f.rsplit('.').next().unwrap_or(f).to_string()),
    };
    let body = mxl::body(&v.list()[2], &lookup)?;
    w.line(&format!("<Settings xmlns:mxl=\"{MXL_NS}\" xsi:type=\"mxl:SpreadsheetDocument\">"));
    w.s.push_str(&prefixed(&body, w.depth));
    w.line("</Settings>");
    Ok(())
}

/// Планировщик `{"#",43dc7f37-...,{8,...}}` → `<Settings xmlns:pl="..." xsi:type="pl:Planner">` (`chart::planner`).
pub(super) fn planner(w: &mut W, v: &Node) -> Result<(), String> {
    if v.get(0).and_then(Node::str) != Some("#") || v.get(1).and_then(Node::atom) != Some("43dc7f37-5b1d-42a7-8f28-f545080d0255") || v.list().len() != 3 {
        return Err("не планировщик".into());
    }
    let lines = crate::xml::chart::planner(v.list()[2].list())?;
    w.line("<Settings xmlns:pl=\"http://v8.1c.ru/8.3/data/planner\" xsi:type=\"pl:Planner\">");
    push_lines(w, lines);
    w.line("</Settings>");
    Ok(())
}

/// Диаграмма `{"#",3543ef08-...,{11},{версия,...}}` → `<Settings xmlns:dNp1="…/chart" xsi:type="dNp1:Chart">` (`chart::body`;
/// префикс — по глубине, как у рисунка табличного документа).
pub(super) fn chart(w: &mut W, v: &Node) -> Result<(), String> {
    let l = v.list();
    if l.len() != 4 || l[0].str() != Some("#") || l[1].atom() != Some("3543ef08-3316-4f7e-9447-0cd0a1cbf1d5") || crate::brace::write(&l[2]).ok().as_deref() != Some("{11}") {
        return Err("не диаграмма".into());
    }
    let p = format!("d{}p1", w.depth + 1);
    let lines = crate::xml::chart::body(l[3].list(), &p)?;
    w.line(&format!("<Settings xmlns:{p}=\"http://v8.1c.ru/8.2/data/chart\" xsi:type=\"{p}:Chart\">"));
    push_lines(w, lines);
    w.line("</Settings>");
    Ok(())
}

/// Диаграмма Ганта `{"#",3a6e63bf-...,{версия,...}}` → `<Settings xmlns:dNp1="…/chart" xsi:type="dNp1:GanttChart">`
/// (`chart::gantt`, как рисунок табличного документа).
pub(super) fn gantt(w: &mut W, v: &Node) -> Result<(), String> {
    let l = v.list();
    if l.len() != 3 || l[0].str() != Some("#") || l[1].atom() != Some("3a6e63bf-16aa-42eb-b48c-2fff9670ad2f") {
        return Err("не диаграмма Ганта".into());
    }
    let p = format!("d{}p1", w.depth + 1);
    let lines = crate::xml::chart::gantt(&l[2..], &p)?;
    w.line(&format!("<Settings xmlns:{p}=\"http://v8.1c.ru/8.2/data/chart\" xsi:type=\"{p}:GanttChart\">"));
    push_lines(w, lines);
    w.line("</Settings>");
    Ok(())
}

/// Строки с отступа 0 → в `w` на отступе `depth + 1`.
fn push_lines(w: &mut W, lines: Vec<String>) {
    for l in lines {
        w.s.push_str("\r\n");
        w.s.extend(std::iter::repeat_n('\t', w.depth + 1));
        w.s.push_str(&l);
    }
}

/// Пустые настройки списка таблицы значений с состоянием сервера: те же 5 ключей, что `empty` (в XML не пишутся), и
/// `"ServerState",{"S",base64 UniversalListServerOnlyState},"DataParameters",{"U"}` — платформа их тоже не пишет
/// (ЕдиныйНалоговыйСчетЛичныйКабинет: 3 таблицы, Settings в эталоне нет).
pub(super) fn server_state_list(n: &Node, empty: &str) -> bool {
    let l = n.list();
    if l.len() != 16 || l.get(1).and_then(Node::atom) != Some("7") {
        return false;
    }
    let mut head = l[..12].to_vec();
    head[1] = Node::Atom("5".into());
    let state = l[12].str() == Some("ServerState") && l[13].get(0).and_then(Node::str) == Some("S") && l[13].list().len() == 2;
    let params = l[14].str() == Some("DataParameters") && matches!(l[15].list(), [u] if u.str() == Some("U"));
    state && params && crate::brace::write(&Node::List(head)).is_ok_and(|s| s.replace("\r\n", "") == empty)
}

/// Тело `<document>` (отступ 1, CRLF после каждого элемента) → строки с `\r\n` впереди, как пишет `W`, на отступе
/// `depth + 1`; имена элементов без префикса — с `mxl:`. Продолжение многострочного текста не сдвигается: текст
/// экранирован, поэтому строка разметки — та, перед которой строка кончилась на `>`.
fn prefixed(body: &str, depth: usize) -> String {
    let mut o = String::with_capacity(body.len() * 2);
    let mut markup = true;
    for line in body.strip_suffix("\r\n").unwrap_or(body).split("\r\n") {
        o.push_str("\r\n");
        if markup {
            o.extend(std::iter::repeat_n('\t', depth));
        }
        let mut rest = line;
        while let Some(i) = rest.find('<') {
            let (head, tail) = rest.split_at(i + 1);
            o.push_str(head);
            let tail = match tail.strip_prefix('/') {
                Some(t) => {
                    o.push('/');
                    t
                }
                None => tail,
            };
            let name = tail.split([' ', '>', '/']).next().unwrap_or("");
            if !name.contains(':') {
                o.push_str("mxl:");
            }
            rest = tail;
        }
        o.push_str(rest);
        markup = line.ends_with('>');
    }
    // типы своего пространства имен тоже с префиксом (`xsi:type="mxl:NamedItemCells"`)
    o.replace("xsi:type=\"NamedItem", "xsi:type=\"mxl:NamedItem")
}

#[cfg(test)]
mod tests {
    #[test]
    fn prefixed() {
        let b = "\t<a>\r\n\t\t<b xsi:type=\"x\">1</b>\r\n\t\t<v8:c/>\r\n\t\t<d>x\r\ny</d>\r\n\t</a>\r\n";
        assert_eq!(super::prefixed(b, 3), "\r\n\t\t\t\t<mxl:a>\r\n\t\t\t\t\t<mxl:b xsi:type=\"x\">1</mxl:b>\r\n\t\t\t\t\t<v8:c/>\r\n\t\t\t\t\t<mxl:d>x\r\ny</mxl:d>\r\n\t\t\t\t</mxl:a>");
    }
}
