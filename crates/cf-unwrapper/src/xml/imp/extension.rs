//! Import of the supported adopted-object layouts; defaults come from a synthetic
//! platform-created extension, not from production metadata.
use super::{
    dom::El,
    index::Index,
    tpl::{self, a, l, s, Props},
};
use crate::{
    brace::{Doc, Node},
    xml::{classes::P, common::ZERO, extension::PROPERTIES},
};
#[path = "extension_defaults.rs"]
mod defaults;

pub fn adopted(obj: &El) -> bool {
    obj.peek_child("Properties")
        .and_then(|p| p.peek_child("ObjectBelonging"))
        .and_then(El::peek_text)
        .as_deref()
        == Some("Adopted")
}

fn states(obj: &El, configuration: bool) -> Result<Vec<(String, String)>, String> {
    let mut out = Vec::new();
    if let Some(ii) = obj.child("InternalInfo") {
        for st in ii.all("xr:PropertyState") {
            let name = st.text_of("xr:Property")?;
            if st.text_of("xr:State")? != "Extended" {
                return Err(format!(
                    "состояние расширяемого свойства {name}: поддержано только Extended"
                ));
            }
            let (id, _, _) = PROPERTIES
                .iter()
                .find(|(_, n, ext)| *n == name && *ext)
                .ok_or(format!("расширяемое свойство {name} не поддержано"))?;
            if out.iter().any(|(g, _)| g == id) {
                return Err(format!("повторное состояние свойства {name}"));
            }
            out.push((id.to_string(), "3".into()));
        }
    }
    if !configuration {
        for e in obj.req("Properties")?.elems() {
            if matches!(e.name.as_str(), "ObjectBelonging" | "Name" | "Comment") {
                continue;
            }
            if e.name == "RegisterRecords"
                && obj.name == "Document"
                && e.elems().next().is_none()
                && e.text()?.is_empty()
            {
                continue;
            }
            let (id, _, external) = PROPERTIES
                .iter()
                .find(|(_, name, _)| *name == e.name)
                .ok_or(format!("заимствованное свойство {} не поддержано", e.name))?;
            if *external {
                return Err(format!("{} должно быть в PropertyState", e.name));
            }
            out.push((
                id.to_string(),
                if e.name == "Content" { "3" } else { "2" }.into(),
            ));
        }
    }
    if obj.name == "Subsystem"
        && obj.req("Properties")?.peek_child("Content").is_some()
        && !out
            .iter()
            .any(|(g, _)| g == "7f676314-716a-4d54-8335-a71a3857b21c")
    {
        out.push(("7f676314-716a-4d54-8335-a71a3857b21c".into(), "3".into()));
    }
    Ok(out)
}

fn header(obj: &El, id: &str, x: &Index, configuration: bool) -> Result<Node, String> {
    let props = obj.req("Properties")?;
    if props.text_of("ObjectBelonging")? != "Adopted" {
        return Err("ожидается ObjectBelonging Adopted".into());
    }
    let p = Props { el: props, x, id };
    let states = states(obj, configuration)?;
    let synonym = match props.child("Synonym") {
        Some(e) => tpl::mlstring(e)?,
        None => l(vec![a("0")]),
    };
    let mut h = vec![
        a("3"),
        l(vec![a("1"), a("0"), a(id)]),
        s(p.text("Name")?),
        synonym,
        p.string("Comment")?,
        a("1"),
        a(states.len().to_string()),
    ];
    for (guid, state) in states {
        h.extend([a(guid), a(state)]);
    }
    h.extend([a(ZERO), a("0")]);
    Ok(l(h))
}

fn generated(obj: &El, q: &str) -> Result<Node, String> {
    let (kind, category) = q.split_once(':').ok_or(format!("место шаблона {q}"))?;
    let ii = obj.req("InternalInfo")?;
    let g = ii
        .all("xr:GeneratedType")
        .find(|g| g.attr("category") == Some(category))
        .ok_or(format!("нет GeneratedType {category}"))?;
    g.req_attr("name")?;
    let value = g.text_of(if kind == "gt" {
        "xr:TypeId"
    } else {
        "xr:ValueId"
    })?;
    if !crate::brace::is_guid(&value) {
        return Err(format!("GeneratedType {category}: не GUID"));
    }
    Ok(a(value))
}

pub fn record(doc: &El, class: &str, x: &Index) -> Result<Vec<u8>, String> {
    super::desc::check_root(doc)?;
    let obj = doc.req(class)?;
    let id = obj.req_attr("uuid")?;
    let template = match class {
        "Catalog" => defaults::CATALOG,
        "Document" => defaults::DOCUMENT,
        "CommonModule" => defaults::COMMON_MODULE,
        "InformationRegister" => defaults::INFORMATION_REGISTER,
        "Language" => defaults::LANGUAGE,
        "Subsystem" => defaults::SUBSYSTEM,
        _ => {
            return Err(format!(
                "загрузка заимствованного объекта {class} пока не поддержана"
            ))
        }
    };
    let head = header(obj, id, x, false)?;
    let mut root = tpl::fill(template, &mut |q| {
        Ok(vec![if q == "H" {
            head.clone()
        } else if q.starts_with("gt:") || q.starts_with("gv:") {
            generated(obj, q)?
        } else {
            return Err(format!("место шаблона {q}"));
        }])
    })?;
    let table = crate::xml::classes::table(class)
        .or_else(|| crate::xml::classes_refs::table(class))
        .or_else(|| crate::xml::classes_regs::tables(class).map(|t| t.0))
        .or_else(|| crate::xml::classes_tools::descriptor_table(class))
        .ok_or("нет таблицы свойств")?;
    for e in obj.req("Properties")?.elems() {
        if matches!(
            e.name.as_str(),
            "ObjectBelonging" | "Name" | "Synonym" | "Comment"
        ) {
            continue;
        }
        if class == "Document" && e.name == "RegisterRecords" {
            e.text()?;
            continue;
        }
        let (_, prop) = table
            .iter()
            .find(|(name, _)| *name == e.name)
            .ok_or(format!("неизвестное свойство {}", e.name))?;
        let (path, value) = match prop {
            P::Bool(path)
            | P::Tok(path, _)
            | P::TokOr(path, _, _)
            | P::Ref(path)
            | P::Ml(path)
            | P::Str(path)
            | P::Type(path) => (*path, super::refs::p_value(prop, e, x)?),
            P::Custom(_) if class == "Subsystem" && e.name == "Content" => {
                (&[1, 7][..], tpl::md_list(x, e, "xr:Item", &|n| n)?)
            }
            _ => {
                return Err(format!(
                    "заимствованное свойство {} пока не загружается",
                    e.name
                ))
            }
        };
        *root
            .at_mut(path)
            .ok_or(format!("нет позиции свойства {}", e.name))? = value;
    }
    if let Some(children) = obj.child("ChildObjects") {
        children.text()?;
    }
    if let Some(u) = doc.unread() {
        return Err(format!("не разобрано: {u}"));
    }
    Ok(Doc::new(true, vec![root]).write()?.into_bytes())
}

pub fn configuration_props(obj: &El, id: &str, x: &Index) -> Result<Node, String> {
    let head = header(obj, id, x, true)?;
    let props = obj.req("Properties")?;
    let p = Props { el: props, x, id };
    let compat = super::index::compat_of(&p.text("ConfigurationExtensionCompatibilityMode")?)
        .ok_or("неверный режим совместимости расширения")?;
    // shortcut: hidden defaults were verified only for the 8.3.16 layout.
    if compat != 80316 {
        return Err(format!(
            "расширение: скрытые значения режима {compat} пока не проверены"
        ));
    }
    let purpose = match p.text("ConfigurationExtensionPurpose")?.as_str() {
        "Patch" => "0",
        "Customization" => "1",
        "AddOn" => "2",
        v => return Err(format!("назначение расширения {v}")),
    };
    let mapping = p.bool("KeepMappingToExtendedConfigurationObjectsByIDs")?;
    for tag in ["Caption", "ShortCaption"] {
        if !p.text(tag)?.is_empty() {
            return Err(format!("расширение: непустое {tag} пока не загружается"));
        }
    }
    let language = obj
        .req("ChildObjects")?
        .all("Language")
        .next()
        .map(El::text)
        .transpose()?
        .map(|n| {
            x.id(&format!("Language.{n}"))
                .map(str::to_string)
                .ok_or("нет языка расширения")
        })
        .transpose()?
        .unwrap_or_else(|| ZERO.into());
    tpl::fill(defaults::CONFIGURATION_PROPS, &mut |q| {
        Ok(vec![match q {
            "H" => head.clone(),
            "extension_compat" => a(compat.to_string()),
            "extension_purpose" => a(purpose),
            "mapping" => mapping.clone(),
            "default_language" => a(&language),
            _ => p.place(q).ok_or(format!("место шаблона {q}"))??,
        }])
    })
}

pub fn cfe(conf: &El, imp: &super::Import) -> Result<Vec<u8>, String> {
    use crate::container::{self, Entry};
    if !imp.refused.is_empty() {
        return Err("расширение содержит незагруженные файлы".into());
    }
    let obj = conf.req("Configuration")?;
    let root = obj.req_attr("uuid")?;
    if !imp.recs.contains_key(root) {
        return Err("нет корневой записи расширения".into());
    }
    let compat = super::index::compat_of(
        &obj.req("Properties")?
            .text_of("ConfigurationExtensionCompatibilityMode")?,
    )
    .ok_or("неверная совместимость расширения")?;
    let now = super::ticks();
    let mut entries = Vec::new();
    let mut hashes = vec![a(imp.recs.len().to_string())];
    for (name, (_, record)) in &imp.recs {
        if name == "configinfo" {
            return Err("повторная служебная запись configinfo".into());
        }
        let raw = match record {
            super::Rec::Raw(bytes) => bytes.clone(),
            super::Rec::Dir(files) => container::write(
                &files
                    .iter()
                    .map(|(name, data)| Entry {
                        name: name.clone(),
                        created: now,
                        modified: now,
                        data: data.clone(),
                    })
                    .collect::<Vec<_>>(),
            ),
        };
        let data = container::deflate(&raw);
        // configinfo hashes the compressed entry bytes, not the XML/brace text.
        let digest = super::payload::base64_atom(&crate::xml::common::sha1(&data));
        let hash = digest
            .atom()
            .and_then(|s| s.strip_prefix("#base64:"))
            .ok_or("не сформирован SHA-1")?;
        hashes.extend([s(name), a(hash)]);
        entries.push(Entry {
            name: name.clone(),
            created: now,
            modified: now,
            data,
        });
    }
    let version = l(vec![
        a("0"),
        l(vec![
            a("216"),
            a("0"),
            l(vec![a(compat.to_string()), a("0")]),
        ]),
    ]);
    let root = l(vec![a("2"), a(root), a("")]);
    let ci = Doc::new(true, vec![version, root, l(hashes)]).write()?;
    entries.push(Entry {
        name: "configinfo".into(),
        created: now,
        modified: now,
        data: container::deflate(ci.as_bytes()),
    });
    entries.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(container::write(&entries))
}
