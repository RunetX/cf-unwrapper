//! Описания вложенных объектов `<Владелец>/Forms/<Имя>.xml` и `Templates/<Имя>.xml` — из собственной записи
//! формы или макета (docs/research/xml-nested.md). Команды отдельного файла не имеют: они в описании владельца.

use super::classes::P;
use super::common::*;
use super::w::W;
use crate::brace::{is_guid, Node};

/// Тело формы `{13,<заголовок>,<IncludeHelpInContents>,<FormType>,{N,<назначения>...}}`; обертки вокруг
/// бывают разные (`{1,{13…},0}`, `{1,{0,{13…}},0}`, у отчетов и обработок `{1,{1,{0,{13…},{представление}}},0}`).
fn body(n: &Node) -> Option<&[Node]> {
    let l = n.list();
    let is_head = |h: &Node| {
        let id = h.get(1).map(Node::list).unwrap_or_default();
        h.get(2).and_then(Node::str).is_some() && id.len() == 3 && id[1].atom() == Some("0") && id[2].atom().is_some_and(is_guid)
    };
    if l.get(1).is_some_and(is_head) {
        return Some(l);
    }
    l.iter().find_map(body)
}

fn form_tok(w: &mut W, tag: &str, n: &Node, i: usize, map: &[(&str, &str)]) {
    match body(n).and_then(|b| b.get(i)) {
        Some(v) => token(w, tag, v, &[], map),
        None => w.text(tag, "\u{1}нет тела формы"),
    }
}

pub const FORM_INTERFACE: &[(&str, &str)] = &[("0", "Any"), ("1", "Taxi"), ("2", "Version85")];

const PURPOSES: &[(&str, &str)] = &[("1", "PlatformApplication"), ("2", "MobilePlatformApplication")];

/// `{N,{"#",1708fdaa-…,<назначение>},...}`; в старой записи (тело `{9,…}`) списка нет — в эталоне оба назначения.
fn use_purposes(w: &mut W, _: &Index, n: &Node) {
    let b = body(n).unwrap_or_default();
    let items: Vec<&str> = match b.get(4) {
        Some(l) => l.list().get(1..).unwrap_or_default().iter().map(|p| atom(p, &[2])).collect(),
        None => vec!["1", "2"],
    };
    if items.is_empty() {
        w.text("UsePurposes", "");
        return;
    }
    w.open("UsePurposes", &[]);
    for a in items {
        let v = PURPOSES.iter().find(|(k, _)| *k == a).map_or_else(|| format!("\u{1}{a}"), |(_, v)| v.to_string());
        w.text_attrs("v8:Value", &[("xsi:type", "app:ApplicationUsePurpose")], &v);
    }
    w.end();
}

/// Только у форм отчетов и обработок: корень[1] = `{1,{0,{13…},{представление}}}`.
fn extended_presentation(w: &mut W, _: &Index, n: &Node) {
    if atom(n, &[1, 0]) == "1" && n.at(&[1, 1, 2]).is_some_and(|p| matches!(p, Node::List(_))) {
        mlstring(w, "ExtendedPresentation", n.at(&[1, 1, 2]));
    }
}

pub const FORM: &[(&str, P)] = &[
    ("FormType", P::Custom(|w, _, n| form_tok(w, "FormType", n, 3, &[("0", "Ordinary"), ("1", "Managed")]))),
    ("IncludeHelpInContents", P::Custom(|w, _, n| form_tok(w, "IncludeHelpInContents", n, 2, &[("0", "false"), ("1", "true")]))),
    ("UsePurposes", P::Custom(use_purposes)),
    // как у общей формы — тело [5]; нет позиции (старая запись) — Any
    ("UseInInterfaceCompatibilityMode", P::Custom(|w, _, n| match body(n).and_then(|b| b.get(5)) {
        Some(v) => token(w, "UseInInterfaceCompatibilityMode", v, &[], FORM_INTERFACE),
        None => w.text("UseInInterfaceCompatibilityMode", "Any"),
    })),
    ("ExtendedPresentation", P::Custom(extended_presentation)),
];

/// Запись макета `{1,{2,<тип>,<заголовок>},0}`.
pub const TEMPLATE: &[(&str, P)] = &[("TemplateType", P::Tok(&[1, 1], super::classes_tools::TEMPLATE_TYPE))];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::brace;
    use std::collections::HashMap;

    fn xml(text: &str, table: &[(&str, P)]) -> String {
        let n = brace::Doc::parse(text).unwrap().roots.remove(0);
        let x = Index { full: HashMap::new(), types: HashMap::new() };
        let mut w = W::metadata();
        super::super::classes::props(&mut w, &x, &n, table);
        let s = String::from_utf8(w.finish()[3..].to_vec()).unwrap();
        let l: Vec<&str> = s.split("\r\n").collect();
        l[2..l.len() - 1].iter().map(|s| s.trim_start_matches('\t')).collect::<Vec<_>>().join("|")
    }

    const H: &str = "{3,{1,0,5a682c7f-10cd-42ba-855b-15b394158ce5},\"Ф\",{1,\"ru\",\"Ф\"},\"\",0,0,00000000-0000-0000-0000-000000000000,0}";
    const BOTH: &str = "<UsePurposes>|<v8:Value xsi:type=\"app:ApplicationUsePurpose\">PlatformApplication</v8:Value>|<v8:Value xsi:type=\"app:ApplicationUsePurpose\">MobilePlatformApplication</v8:Value>|</UsePurposes>";

    #[test]
    fn forms() {
        let p = "{2,{\"#\",1708fdaa-cbce-4289-b373-07a5a74bee91,1},{\"#\",1708fdaa-cbce-4289-b373-07a5a74bee91,2}}";
        assert_eq!(
            xml(&format!("{{1,{{0,{{13,{H},1,0,{p}}}}},0}}"), FORM),
            format!("<FormType>Ordinary</FormType>|<IncludeHelpInContents>true</IncludeHelpInContents>|{BOTH}|<UseInInterfaceCompatibilityMode>Any</UseInInterfaceCompatibilityMode>")
        );
        // обработка: представление в обертке; одно назначение
        let s = xml(&format!("{{1,{{1,{{0,{{13,{H},0,1,{{1,{{\"#\",1708fdaa-cbce-4289-b373-07a5a74bee91,1}}}}}},{{1,\"ru\",\"П\"}}}}}},0}}"), FORM);
        assert!(s.contains("<FormType>Managed</FormType>") && s.contains("ApplicationUsePurpose\">PlatformApplication</v8:Value>|</UsePurposes>"), "{s}");
        assert!(s.ends_with("<ExtendedPresentation>|<v8:item>|<v8:lang>ru</v8:lang>|<v8:content>П</v8:content>|</v8:item>|</ExtendedPresentation>"), "{s}");
        // раскладка как в записи БП: {1,{1,{0,{13…},{0}}},0}
        let s = xml(&format!("{{1,\r\n{{1,\r\n{{0,\r\n{{13,{H},0,1,{{0}}}},\r\n{{0}}\r\n}}\r\n}},0}}"), FORM);
        assert!(s.ends_with("<UsePurposes/>|<UseInInterfaceCompatibilityMode>Any</UseInInterfaceCompatibilityMode>|<ExtendedPresentation/>"), "{s}");
        // старая запись {9,…} без списка назначений
        let s = xml("{1,{0,{9,{0,{0,0,fa799592-d657-4696-b2bc-3f024ba111b2},\"Ф\",{1,\"ru\",\"Ф\"},\"\"},0,1}},0}", FORM);
        assert!(s.contains(BOTH) && s.contains("<IncludeHelpInContents>false"), "{s}");
    }

    #[test]
    fn templates() {
        let t = |k: &str| xml(&format!("{{1,{{2,{k},{H}}},0}}"), TEMPLATE);
        assert_eq!(t("6"), "<TemplateType>DataCompositionSchema</TemplateType>");
        assert_eq!(t("7"), "<TemplateType>DataCompositionAppearanceTemplate</TemplateType>");
        assert_eq!(t("10"), "<TemplateType>10</TemplateType>");
    }
}
