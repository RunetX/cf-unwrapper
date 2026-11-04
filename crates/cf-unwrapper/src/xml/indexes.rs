//! AdditionalIndexes: version-one index descriptions and single metadata-field paths.
//! Other path shapes are rejected until their semantics have been recovered.
use super::{
    common::{Index, ZERO},
    imp::{
        dom::El,
        index::Index as ImportIndex,
        tpl::{a, l, s},
    },
    w::W,
};
use crate::brace::{self, Node};
const INDEX: &str = "4b3b32e1-14f6-4ce8-b4c4-1bc85a74237e";
const FIELD: &str = "07c5e7a4-56de-47f1-9895-724a499e8a8c";
const NS: &[(&str, &str)] = &[
    ("xmlns", "http://v8.1c.ru/8.3/xcf/extrnprops"),
    ("xmlns:v8", "http://v8.1c.ru/8.1/data/core"),
    ("xmlns:xr", "http://v8.1c.ru/8.3/xcf/readable"),
    ("xmlns:xs", "http://www.w3.org/2001/XMLSchema"),
    ("xmlns:xsi", "http://www.w3.org/2001/XMLSchema-instance"),
    ("version", "2.21"),
];

fn counted(n: &Node) -> Result<&[Node], String> {
    let v = n.list();
    let count = v
        .first()
        .and_then(Node::atom)
        .and_then(|n| n.parse::<usize>().ok())
        .ok_or("нет числа элементов индекса")?;
    if count != v.len().saturating_sub(1) {
        return Err("неверное число элементов индекса".into());
    }
    Ok(&v[1..])
}
fn reference(n: &Node) -> Result<&str, String> {
    match n.list() {
        [Node::Atom(z), Node::Atom(id)] if z == "0" && brace::is_guid(id) => Ok(id),
        _ => Err("индекс: путь к метаданным не из одного GUID".into()),
    }
}
fn field(n: &Node) -> Result<&str, String> {
    match n.list() {
        [Node::Str(mark), Node::Atom(kind), body] if mark == "#" && kind == FIELD => {
            match body.list() {
                [Node::Atom(version), path] if version == "1" => reference(path),
                _ => Err("индекс: неизвестный путь поля".into()),
            }
        }
        _ => Err("индекс: неизвестное описание поля".into()),
    }
}
pub(super) fn export(n: &Node, x: &Index) -> Result<Vec<u8>, String> {
    let list = match n.list() {
        [Node::Atom(v), list] if v == "1" => counted(list)?,
        _ => return Err("неизвестная версия дополнительных индексов".into()),
    };
    let head = format!(
        "<AdditionalIndexes {}>",
        NS.iter()
            .map(|(k, v)| format!("{k}=\"{v}\""))
            .collect::<Vec<_>>()
            .join(" ")
    );
    let mut w = W::root("AdditionalIndexes", &head);
    for it in list {
        let body = match it.list() {
            [Node::Str(mark), Node::Atom(kind), body] if mark == "#" && kind == INDEX => body,
            _ => return Err("неизвестный тип дополнительного индекса".into()),
        };
        let (id, fields, additional, name, table) = match body.list() {
            [Node::Atom(v), Node::Atom(id), fields, additional, Node::Str(name), table, Node::Atom(enabled), Node::Atom(zero)]
                if v == "1" && enabled == "1" && zero == ZERO && brace::is_guid(id) =>
            {
                (id, fields, additional, name, table)
            }
            _ => return Err("неизвестная структура дополнительного индекса".into()),
        };
        let table = x
            .full
            .get(reference(table)?)
            .ok_or("нет таблицы дополнительного индекса")?;
        w.open("AdditionalIndex", &[("id", id)]);
        w.text("Name", name);
        w.text("Table", table);
        for (tag, n) in [("IndexedFields", fields), ("AdditionalFields", additional)] {
            let fields = counted(n)?;
            if fields.is_empty() {
                w.text(tag, "");
                continue;
            }
            w.open(tag, &[]);
            for item in fields {
                let full = x
                    .full
                    .get(field(item)?)
                    .ok_or("нет поля дополнительного индекса")?;
                let tail = full
                    .strip_prefix(&format!("{table}."))
                    .ok_or("индекс с полем чужой таблицы")?;
                let (kind, name) = tail.split_once('.').ok_or("неизвестное поле индекса")?;
                if !matches!(kind, "Attribute" | "Dimension" | "Resource") || name.contains('.') {
                    return Err("индекс: поддержаны только собственные поля таблицы".into());
                }
                w.text("Field", name);
            }
            w.end();
        }
        w.end();
    }
    Ok(w.finish())
}

pub(crate) fn import(doc: &El, x: &ImportIndex) -> Result<Vec<u8>, String> {
    if doc.name != "AdditionalIndexes"
        || doc.attrs.len() != NS.len()
        || NS.iter().any(|(k, v)| doc.attr(k) != Some(*v))
    {
        return Err("AdditionalIndexes: неверный корень, пространства имён или версия".into());
    }
    let mut indexes = vec![a("0")];
    for item in doc.elems() {
        if item.name != "AdditionalIndex" {
            return Err(format!("неизвестный элемент индекса {}", item.name));
        }
        let id = item.req_attr("id")?;
        if !brace::is_guid(id) {
            return Err("индекс: id не GUID".into());
        }
        let name = item.text_of("Name")?;
        let table = item.text_of("Table")?;
        let table_id = x.id(&table).ok_or("нет таблицы дополнительного индекса")?;
        let fields = |tag: &str| -> Result<Node, String> {
            let mut out = vec![a("0")];
            for f in item.req(tag)?.elems() {
                if f.name != "Field" {
                    return Err("индекс: неизвестный элемент списка полей".into());
                }
                let name = f.text()?;
                let candidates: Vec<_> = ["Attribute", "Dimension", "Resource"]
                    .iter()
                    .filter_map(|kind| x.id(&format!("{table}.{kind}.{name}")))
                    .collect();
                if candidates.len() != 1 {
                    return Err(format!("индекс: поле {name} отсутствует или неоднозначно"));
                }
                out.push(l(vec![
                    s("#"),
                    a(FIELD),
                    l(vec![a("1"), l(vec![a("0"), a(candidates[0])])]),
                ]));
            }
            out[0] = a((out.len() - 1).to_string());
            Ok(l(out))
        };
        let body = l(vec![
            a("1"),
            a(id),
            fields("IndexedFields")?,
            fields("AdditionalFields")?,
            s(name),
            l(vec![a("0"), a(table_id)]),
            a("1"),
            a(ZERO),
        ]);
        indexes.push(l(vec![s("#"), a(INDEX), body]));
    }
    indexes[0] = a((indexes.len() - 1).to_string());
    if let Some(u) = doc.unread() {
        return Err(format!("не разобрано: {u}"));
    }
    brace::Doc::new(true, vec![l(vec![a("1"), l(indexes)])])
        .write()
        .map(String::into_bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    const TABLE_ID: &str = "00000000-0000-4000-8000-000000000001";
    const FIELD_ID: &str = "00000000-0000-4000-8000-000000000002";
    const INDEX_ID: &str = "00000000-0000-4000-8000-000000000003";

    fn document(field: &str, extra: &str) -> El {
        super::super::imp::dom::parse(&format!(
            "<AdditionalIndexes {}><AdditionalIndex id=\"{INDEX_ID}\"><Name>Lookup</Name><Table>InformationRegister.Sample</Table><IndexedFields><Field>{field}</Field></IndexedFields><AdditionalFields/>{extra}</AdditionalIndex></AdditionalIndexes>",
            NS.iter().map(|(k,v)|format!("{k}=\"{v}\"")).collect::<Vec<_>>().join(" ")
        )).unwrap()
    }
    fn index() -> ImportIndex {
        let mut x = ImportIndex::default();
        x.full
            .insert("InformationRegister.Sample".into(), TABLE_ID.into());
        x.full.insert(
            "InformationRegister.Sample.Dimension.Key".into(),
            FIELD_ID.into(),
        );
        x
    }
    #[test]
    fn additional_index_roundtrip_and_reference_validation() {
        let x = index();
        let bytes = import(&document("Key", ""), &x).unwrap();
        let node = brace::Doc::parse(std::str::from_utf8(&bytes).unwrap()).unwrap();
        let mut names = Index {
            full: x
                .full
                .iter()
                .map(|(n, id)| (id.clone(), n.clone()))
                .collect(),
            types: Default::default(),
        };
        let xml = export(&node.roots[0], &names).unwrap();
        let doc = super::super::imp::dom::parse(std::str::from_utf8(&xml).unwrap()).unwrap();
        assert_eq!(import(&doc, &x).unwrap(), bytes);
        names.full.insert(
            FIELD_ID.into(),
            "InformationRegister.Other.Dimension.Key".into(),
        );
        assert!(export(&node.roots[0], &names)
            .unwrap_err()
            .contains("чужой таблицы"));
    }
    #[test]
    fn unknown_fields_and_properties_are_refused() {
        assert!(import(&document("Missing", ""), &index())
            .unwrap_err()
            .contains("отсутствует"));
        assert!(
            import(&document("Key", "<Unknown>true</Unknown>"), &index())
                .unwrap_err()
                .contains("не разобрано")
        );
    }
}
