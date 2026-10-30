//! Блок свойств реквизита `{27,{2,<заголовок>,<описание типов>},...}` — общий у констант, общих реквизитов,
//! реквизитов справочников и документов. Позиции сверены по 30 тыс. реквизитов БП 3.0.171.

use super::common::*;
use super::w::W;
use crate::brace::Node;

/// Свойство `tag` из блока `b`; неизвестное свойство или значение — `?…`, чтобы сверка показала.
pub fn prop(w: &mut W, x: &Index, b: &Node, tag: &str) {
    let tok = |w: &mut W, i: usize, map: &[(&str, &str)]| token(w, tag, b, &[i], map);
    match tag {
        "Type" => match b.at(&[1, 2]) {
            Some(p) => type_desc(w, tag, x, p),
            None => w.text(tag, "\u{1}нет описания типа"),
        },
        "PasswordMode" => bool_at(w, tag, b, &[2]),
        "Format" => mlstring(w, tag, b.get(3)),
        "ToolTip" => mlstring(w, tag, b.get(4)),
        "MarkNegatives" => bool_at(w, tag, b, &[5]),
        "Mask" => w.text(tag, string(b, &[6])),
        "MultiLine" => bool_at(w, tag, b, &[7]),
        "MinValue" => value(w, x, tag, b.get(8)),
        "MaxValue" => value(w, x, tag, b.get(9)),
        "ChoiceFoldersAndItems" => tok(w, 10, &[("0", "Items"), ("1", "Folders"), ("2", "FoldersAndItems")]),
        "ChoiceForm" => reference(w, tag, x, b, &[11]),
        "QuickChoice" => tok(w, 12, &[("0", "DontUse"), ("1", "Use"), ("2", "Auto")]),
        "FillChecking" => tok(w, 13, &[("0", "DontCheck"), ("1", "ShowError")]),
        "ChoiceParameterLinks" => links(w, x, tag, &owner(x, b), b.get(14)),
        "LinkByType" => link_by_type(w, x, tag, &owner(x, b), b.get(15)),
        "ChoiceParameters" => choice_parameters(w, x, tag, b.get(16)),
        "ExtendedEdit" => bool_at(w, tag, b, &[17]),
        "EditFormat" => mlstring(w, tag, b.get(18)),
        "FillValue" => match (b.get(19).and_then(|v| v.get(0)).and_then(Node::str), b.at(&[1, 2])) {
            // значение не из простых типов реквизита (пустая строка у булева признака учета) платформа пишет как nil;
            // при ссылочных и определяемых типах значение пишется как есть
            (Some(k @ ("S" | "N" | "B" | "D")), Some(t)) if {
                let kinds: Vec<&str> = t.list().iter().skip(1).filter_map(|p| p.get(0).and_then(Node::str)).collect();
                !kinds.is_empty() && kinds.iter().all(|c| matches!(*c, "S" | "N" | "B" | "D")) && !kinds.contains(&k)
            } => w.nil(tag),
            _ => value(w, x, tag, b.get(19)),
        },
        "FillFromFillingValue" => bool_at(w, tag, b, &[20]),
        // в блоке старого формата {25,...} позиций 21–22 нет — Auto
        "CreateOnInput" => token_or0(w, tag, b, &[21], &[("0", "Auto"), ("1", "DontUse"), ("2", "Use")]),
        "ChoiceHistoryOnInput" => token_or0(w, tag, b, &[22], &[("0", "Auto"), ("1", "DontUse")]),
        _ => w.text(tag, "\u{1}свойство реквизита"),
    }
}

/// Полное имя объекта-владельца реквизита (`Document.Имя`) по uuid из заголовка блока.
fn owner(x: &Index, b: &Node) -> String {
    let full = x.name(b.at(&[1, 1, 1, 2]).and_then(Node::atom).unwrap_or_default());
    full.splitn(3, '.').take(2).collect::<Vec<_>>().join(".")
}

/// Путь к полю `{0,<uuid>}`… в объекте `owner`: последнее поле пути. Поле чужого объекта (ссылка осталась от
/// копирования) платформа пишет как `0:<uuid>/0:<uuid>` по всему пути; `{-код}` — стандартный реквизит владельца.
fn data_path(x: &Index, owner: &str, path: &[Node]) -> String {
    let Some(last) = path.last() else { return "\u{1}нет пути".into() };
    match last.list() {
        [Node::Atom(z), Node::Atom(id)] if z == "0" => match x.full.get(id) {
            Some(n) if n.starts_with(&format!("{owner}.")) => n.clone(),
            // константа — по имени из любого владельца (ERP 2.5.22: ВидЦеныПлановойСтоимостиМатериаловРабот)
            Some(n) if n.strip_prefix("Constant.").is_some_and(|c| !c.contains('.')) => n.clone(),
            _ => path.iter().map(|p| format!("{}:{}", atom(p, &[0]), atom(p, &[1]))).collect::<Vec<_>>().join("/"),
        },
        [Node::Atom(c)] if c.starts_with('-') => {
            let class = owner.split('.').next().unwrap_or_default();
            let names = super::standard::NAMES.iter().find(|(k, _)| *k == class).map(|(_, t)| *t).unwrap_or(&[]);
            match names.iter().find(|(k, _)| k == c) {
                Some((_, nm)) => format!("{owner}.StandardAttribute.{nm}"),
                None => format!("{owner}.StandardAttribute.\u{1}{c}"),
            }
        }
        _ => raw(Some(last)),
    }
}

/// Связи параметров выбора `{5004|5006,N,"имя",K,<K полей пути>,<0 Clear | 1 DontChange>,...}`.
pub fn links(w: &mut W, x: &Index, tag: &str, owner: &str, n: Option<&Node>) {
    let l = n.map(Node::list).unwrap_or_default();
    let count: usize = l.get(1).and_then(Node::atom).and_then(|c| c.parse().ok()).unwrap_or(0);
    if count == 0 {
        w.text(tag, if l.get(1).and_then(Node::atom) == Some("0") { "" } else { "\u{1}нет связей" });
        return;
    }
    w.open(tag, &[]);
    let mut i = 2;
    for _ in 0..count {
        let k: usize = l.get(i + 1).and_then(Node::atom).and_then(|c| c.parse().ok()).unwrap_or(0);
        let (Some(name), Some(path), Some(change)) = (l.get(i).and_then(Node::str), l.get(i + 2..i + 2 + k), l.get(i + 2 + k).and_then(Node::atom)) else {
            w.text("xr:Link", &raw(n));
            break;
        };
        w.open("xr:Link", &[]);
        w.text("xr:Name", name);
        let mut dp = data_path(x, owner, path);
        // у обработки и отчета стандартных реквизитов нет (связь скопирована у справочника): платформа пишет код как есть
        // (1 связь БП, `-8`); код номера строки табличной части и связь по типу не встречены — там остается метка
        let line_number = |c: &str| super::standard::NAMES.iter().any(|(k, t)| *k == "TabularSection" && t.iter().any(|(n, _)| *n == c));
        if let Some((_, c)) = dp.split_once(".StandardAttribute.\u{1}").filter(|(_, c)| !line_number(c)) {
            if owner.starts_with("DataProcessor.") || owner.starts_with("Report.") {
                dp = c.to_string();
            }
        }
        w.text_attrs("xr:DataPath", &[("xsi:type", "xs:string")], &dp);
        w.text("xr:ValueChange", &match change {
            "0" => "Clear".into(),
            "1" => "DontChange".into(),
            c => format!("{}{c}", super::UNK),
        });
        w.end();
        i += 3 + k;
    }
    w.end();
}

/// Связь по типу `{3,K,<K полей пути>,<элемент связи>}`; `{3,0,0}` — нет.
fn link_by_type(w: &mut W, x: &Index, tag: &str, owner: &str, n: Option<&Node>) {
    let l = n.map(Node::list).unwrap_or_default();
    let k: usize = l.get(1).and_then(Node::atom).and_then(|c| c.parse().ok()).unwrap_or(0);
    match (k, l.get(2..2 + k), l.get(2 + k).and_then(Node::atom)) {
        (0, _, _) if l.get(1).and_then(Node::atom) == Some("0") => w.text(tag, ""),
        (1.., Some(path), Some(item)) => {
            w.open(tag, &[]);
            w.text("xr:DataPath", &data_path(x, owner, path));
            w.text("xr:LinkItem", item);
            w.end();
        }
        _ => w.text(tag, &raw(n)),
    }
}

/// Параметры выбора `{0,N,"имя",<значение>,...}` → `<app:item name><app:value …>`.
pub fn choice_parameters(w: &mut W, x: &Index, tag: &str, n: Option<&Node>) {
    let l = n.map(Node::list).unwrap_or_default();
    let count: usize = l.get(1).and_then(Node::atom).and_then(|c| c.parse().ok()).unwrap_or(0);
    if count == 0 || l.len() != 2 + 2 * count {
        let v = if l.get(1).and_then(Node::atom) == Some("0") { String::new() } else { raw(n) };
        w.text(tag, &v);
        return;
    }
    w.open(tag, &[]);
    for c in l[2..].chunks(2) {
        w.open("app:item", &[("name", c[0].str().unwrap_or("\u{1}"))]);
        value(w, x, "app:value", c.get(1));
        w.end();
    }
    w.end();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn xml(f: impl FnOnce(&mut W)) -> String {
        let mut w = W::metadata();
        f(&mut w);
        let s = String::from_utf8(w.finish()[3..].to_vec()).unwrap();
        let l: Vec<&str> = s.split("\r\n").collect();
        l[2..l.len() - 1].iter().map(|s| s.trim_start_matches('\t')).collect::<Vec<_>>().join("|")
    }

    fn node(t: &str) -> Node {
        crate::brace::Doc::parse(t).unwrap().roots.remove(0)
    }

    const ID: &str = "11111111-1111-1111-1111-111111111111";

    fn index() -> Index {
        let mut x = Index { full: HashMap::new(), types: HashMap::new() };
        x.full.insert(ID.into(), "Document.Д.Attribute.Р".into());
        x.full.insert("22222222-2222-2222-2222-222222222222".into(), "Document.Д.Attribute.Организация".into());
        x.full.insert("33333333-3333-3333-3333-333333333333".into(), "Document.Чужой.Attribute.Организация".into());
        x
    }

    #[test]
    fn old_block_without_create_on_input() {
        // блок {25,...} из 21 элемента: позиций 21–22 нет
        let b = node(&format!("{{25,{{2,{{1,{{0,0,{ID}}},\"Р\",{{0}},\"\",0,0}},{{\"Pattern\",{{\"B\"}}}}}},0,{{0}},{{0}},0,\"\",0,{{\"U\"}},{{\"U\"}},0,00000000-0000-0000-0000-000000000000,2,0,{{5004,0}},{{3,0,0}},{{0,0}},0,{{0}},{{\"B\",0}},0}}"));
        let x = index();
        assert_eq!(xml(|w| prop(w, &x, &b, "CreateOnInput")), "<CreateOnInput>Auto</CreateOnInput>");
        assert_eq!(xml(|w| prop(w, &x, &b, "ChoiceHistoryOnInput")), "<ChoiceHistoryOnInput>Auto</ChoiceHistoryOnInput>");
        assert_eq!(xml(|w| prop(w, &x, &b, "FillValue")), "<FillValue xsi:type=\"xs:boolean\">false</FillValue>");
        assert_eq!(xml(|w| prop(w, &x, &b, "ChoiceParameterLinks")), "<ChoiceParameterLinks/>");
    }

    #[test]
    fn fill_value_outside_type_is_nil() {
        let b = |pattern: &str| node(&format!("{{27,{{2,{{3,{{1,0,{ID}}},\"Р\",{{0}},\"\",0}},{{\"Pattern\",{pattern}}}}},0,{{0}},{{0}},0,\"\",0,{{\"U\"}},{{\"U\"}},0,00000000-0000-0000-0000-000000000000,2,0,{{5006,0}},{{3,0,0}},{{0,0}},0,{{0}},{{\"S\",\"\"}},0,0,0}}"));
        let x = index();
        // признак учета: тип булево, а хранится пустая строка
        assert_eq!(xml(|w| prop(w, &x, &b("{\"B\"}"), "FillValue")), "<FillValue xsi:nil=\"true\"/>");
        assert_eq!(xml(|w| prop(w, &x, &b("{\"S\",10,1}"), "FillValue")), "<FillValue xsi:type=\"xs:string\"/>");
        // у ссылочного (определяемого) типа значение пишется как есть
        assert_eq!(xml(|w| prop(w, &x, &b(&format!("{{\"#\",{ID}}}")), "FillValue")), "<FillValue xsi:type=\"xs:string\"/>");
    }

    #[test]
    fn links_and_parameters() {
        let x = index();
        // своя ссылка, чужая (осталась от копирования) — 0:<uuid>, стандартный реквизит
        let l = node("{5006,3,\"Отбор.Организация\",1,{0,22222222-2222-2222-2222-222222222222},1,\"Отбор.Владелец\",1,{0,33333333-3333-3333-3333-333333333333},0,\"Отбор.Дата\",1,{-3},0}");
        assert_eq!(
            xml(|w| links(w, &x, "L", "Document.Д", Some(&l))),
            "<L>|<xr:Link>|<xr:Name>Отбор.Организация</xr:Name>|<xr:DataPath xsi:type=\"xs:string\">Document.Д.Attribute.Организация</xr:DataPath>|<xr:ValueChange>DontChange</xr:ValueChange>|</xr:Link>\
|<xr:Link>|<xr:Name>Отбор.Владелец</xr:Name>|<xr:DataPath xsi:type=\"xs:string\">0:33333333-3333-3333-3333-333333333333</xr:DataPath>|<xr:ValueChange>Clear</xr:ValueChange>|</xr:Link>\
|<xr:Link>|<xr:Name>Отбор.Дата</xr:Name>|<xr:DataPath xsi:type=\"xs:string\">Document.Д.StandardAttribute.Date</xr:DataPath>|<xr:ValueChange>Clear</xr:ValueChange>|</xr:Link>|</L>"
        );
        // у обработки стандартных реквизитов нет — код как есть; у документа незнакомый код — метка
        let l = node("{5006,1,\"Отбор.Владелец\",1,{-8},0}");
        assert!(xml(|w| links(w, &x, "L", "DataProcessor.О", Some(&l))).contains("<xr:DataPath xsi:type=\"xs:string\">-8</xr:DataPath>"));
        assert!(xml(|w| links(w, &x, "L", "Document.Д", Some(&l))).contains("StandardAttribute.\u{1}-8"));
        let p = node("{0,1,\"Отбор.Активен\",{\"B\",1}}");
        assert_eq!(xml(|w| choice_parameters(w, &x, "P", Some(&p))), "<P>|<app:item name=\"Отбор.Активен\">|<app:value xsi:type=\"xs:boolean\">true</app:value>|</app:item>|</P>");
        let t = node("{3,1,{0,22222222-2222-2222-2222-222222222222},1}");
        assert_eq!(xml(|w| link_by_type(w, &x, "T", "Document.Д", Some(&t))), "<T>|<xr:DataPath>Document.Д.Attribute.Организация</xr:DataPath>|<xr:LinkItem>1</xr:LinkItem>|</T>");
    }
}
