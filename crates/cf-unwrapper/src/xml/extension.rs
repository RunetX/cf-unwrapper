//! Семантика заимствованных объектов расширения. GUID свойств сверены с MdProperty
//! модели EDT и XML платформы 8.5.1.1529 (Контур.КЭДО 1.27.3).
use super::{
    common::{atom, head_of, ZERO},
    imp::dom::{self, El, Kid},
    w::W,
};
use crate::{
    brace::{self, Node},
    meta::Config,
};

pub fn is_extension(cfg: &Config) -> bool {
    cfg.index.contains_key("configinfo")
        && !cfg.index.contains_key("root")
        && cfg.root.kind == "Configuration"
}

// shortcut: только свойства, сверенные на CFE КЭДО; неизвестный GUID или
// состояние отвергаются, пока не восстановлена соответствующая семантика.
pub(crate) const PROPERTIES: &[(&str, &str, bool)] = &[
    ("d5963243-262e-4398-b4d7-fb16d06484f6", "Module", true),
    (
        "d22e852a-cf8a-4f77-8ccb-3548e7792bea",
        "ManagedApplicationModule",
        true,
    ),
    (
        "9b7bbbae-9771-46f2-9e4d-2489e0ffc702",
        "SessionModule",
        true,
    ),
    (
        "740eb5f6-e214-4e30-aefa-f15bab91c688",
        "HomePageWorkArea",
        true,
    ),
    (
        "7f676314-716a-4d54-8335-a71a3857b21c",
        "CommandInterface",
        true,
    ),
    ("c6690627-40cf-4741-8719-4e7feb832b84", "Content", false),
    ("7dbb2bc7-6bae-4b81-91cb-681317272a0b", "Global", false),
    (
        "74ce8a02-abd2-46a6-8544-8cfbb4e8c6e0",
        "ClientManagedApplication",
        false,
    ),
    ("6275a02e-96f0-4347-975a-2d661e6a0675", "Server", false),
    (
        "d12660a6-7298-4ae2-a332-b95a6459a280",
        "ExternalConnection",
        false,
    ),
    (
        "436af77a-e846-4084-818b-740a3378518e",
        "ClientOrdinaryApplication",
        false,
    ),
    ("c474bab9-d13a-4fbd-bfb0-9214d6dc2fde", "ServerCall", false),
    (
        "13134205-f60b-11d5-a3c7-0050bae0a776",
        "InformationRegisterPeriodicity",
        false,
    ),
    ("09c412e0-0f30-11d6-a3c7-0050bae0a776", "WriteMode", false),
    (
        "4b2a2bcf-a845-41ba-a03d-05b09a2c6c11",
        "LanguageCode",
        false,
    ),
];

pub(super) fn export(cfg: &Config, data: &[u8]) -> Result<Vec<u8>, String> {
    let text = std::str::from_utf8(data).map_err(|e| e.to_string())?;
    if !text.contains("<MetaDataObject ") {
        return Ok(data.to_vec());
    }
    let mut doc = dom::parse(text)?;
    let Some(obj) = doc.elems_mut().next() else {
        return Ok(data.to_vec());
    };
    let Some(id) = obj.attr("uuid") else {
        return Ok(data.to_vec());
    };
    let Some(record) = cfg.entry_text(id) else {
        return Ok(data.to_vec());
    };
    let root = brace::parse(&record)?;
    rewrite(obj, &root)?;
    let mut w = W::metadata();
    render(&mut w, obj)?;
    Ok(w.finish())
}

fn render(w: &mut W, e: &El) -> Result<(), String> {
    let attrs: Vec<_> = e
        .attrs
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();
    let kids: Vec<_> = e.elems().collect();
    if kids.is_empty() {
        w.text_attrs(&e.name, &attrs, &e.text()?);
    } else {
        w.open(&e.name, &attrs);
        for c in kids {
            render(w, c)?;
        }
        w.end();
    }
    Ok(())
}

fn rewrite(obj: &mut El, root: &Node) -> Result<(), String> {
    let h = if obj.name == "Configuration" {
        super::common::head(root)
    } else {
        obj.attr("uuid").and_then(|id| head_of(root, id))
    };
    if let Some(h) = h {
        if h.first().and_then(Node::atom) == Some("3") && h.get(5).and_then(Node::atom) == Some("1")
        {
            let count: usize = h
                .get(6)
                .and_then(Node::atom)
                .ok_or("нет числа состояний свойств")?
                .parse()
                .map_err(|_| "неверное число состояний свойств")?;
            let mut selected = vec!["Name", "Comment"];
            let mut states = Vec::new();
            for i in 0..count {
                let id = h
                    .get(7 + 2 * i)
                    .and_then(Node::atom)
                    .ok_or("нет GUID состояния свойства")?;
                let state = h
                    .get(8 + 2 * i)
                    .and_then(Node::atom)
                    .ok_or("нет состояния свойства")?;
                let (_, name, external) = PROPERTIES.iter().find(|(g, ..)| *g == id).ok_or(
                    format!("свойство расширения {id}: семантика не восстановлена"),
                )?;
                match (state, external) {
                    ("2", false) | ("3", false) => selected.push(name),
                    ("3", true) => states.push(*name),
                    _ => {
                        return Err(format!(
                            "свойство расширения {name}: неподдержанное состояние {state}"
                        ))
                    }
                }
            }
            let tail = 7 + 2 * count;
            if h.len() != tail + 2 {
                return Err("неподдержанный хвост заголовка заимствованного объекта".into());
            }
            let base = h
                .get(tail)
                .and_then(Node::atom)
                .ok_or("нет ссылки на заимствованный объект")?;
            if base != ZERO || h.get(tail + 1).and_then(Node::atom) != Some("0") {
                return Err("сопоставление заимствованного объекта по внутреннему идентификатору пока не поддержано".into());
            }
            if obj.name == "Configuration" {
                selected.extend([
                    "Synonym",
                    "NamePrefix",
                    "ConfigurationExtensionCompatibilityMode",
                    "ScriptVariant",
                    "Vendor",
                    "Version",
                    "Caption",
                    "ShortCaption",
                    "BriefInformation",
                    "DetailedInformation",
                    "Copyright",
                    "VendorInformationAddress",
                    "ConfigurationInformationAddress",
                ]);
            }
            if obj.name == "Document" {
                selected.push("RegisterRecords");
            }
            let is_config = obj.name == "Configuration";
            let props = obj
                .elems_mut()
                .find(|e| e.name == "Properties")
                .ok_or("нет Properties")?;
            props
                .kids
                .retain(|k| matches!(k,Kid::El(e) if selected.contains(&e.name.as_str())));
            props
                .kids
                .insert(0, Kid::El(El::leaf("ObjectBelonging", "Adopted")));
            if is_config {
                let p = root.at(&[3, 1, 1]).ok_or("нет свойств конфигурации")?;
                let purpose = match atom(p, &[44]) {
                    "0" => "Patch",
                    "1" => "Customization",
                    "2" => "AddOn",
                    v => return Err(format!("назначение расширения {v}")),
                };
                let keep = match atom(p, &[49]) {
                    "0" => "false",
                    "1" => "true",
                    v => return Err(format!("сопоставление объектов расширения {v}")),
                };
                let insert = props
                    .kids
                    .iter()
                    .position(|k| matches!(k,Kid::El(e) if e.name=="NamePrefix"))
                    .ok_or("нет NamePrefix")?;
                props.kids.insert(
                    insert,
                    Kid::El(El::leaf("ConfigurationExtensionPurpose", purpose)),
                );
                props.kids.insert(
                    insert + 1,
                    Kid::El(El::leaf(
                        "KeepMappingToExtendedConfigurationObjectsByIDs",
                        keep,
                    )),
                );
                for e in props.elems_mut() {
                    match e.name.as_str() {
                        "NamePrefix" => {
                            *e = El::leaf(
                                "NamePrefix",
                                p.get(42)
                                    .and_then(Node::str)
                                    .ok_or("нет префикса расширения")?,
                            )
                        }
                        "ConfigurationExtensionCompatibilityMode" => {
                            let v: u32 = atom(p, &[43])
                                .parse()
                                .map_err(|_| "неверный режим совместимости расширения")?;
                            *e = El::leaf(
                                "ConfigurationExtensionCompatibilityMode",
                                &format!("Version{}_{}_{}", v / 10000, v / 100 % 100, v % 100),
                            );
                        }
                        _ => {}
                    }
                }
            }
            {
                states.sort_by_key(|name| {
                    PROPERTIES
                        .iter()
                        .position(|(_, n, _)| n == name)
                        .unwrap_or(usize::MAX)
                });
                if !obj.elems().any(|e| e.name == "InternalInfo") {
                    obj.kids
                        .insert(0, Kid::El(El::new("InternalInfo", &[], vec![])));
                }
                let ii = obj.elems_mut().find(|e| e.name == "InternalInfo").unwrap();
                for name in states {
                    ii.kids.push(Kid::El(El::new(
                        "xr:PropertyState",
                        &[],
                        vec![
                            Kid::El(El::leaf("xr:Property", name)),
                            Kid::El(El::leaf("xr:State", "Extended")),
                        ],
                    )));
                }
            }
        }
    }
    if let Some(co) = obj.elems_mut().find(|e| e.name == "ChildObjects") {
        for child in co.elems_mut() {
            rewrite(child, root)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::xml::imp::tpl::{a, l, s};
    use crate::{
        container::{self, Entry, Width},
        meta::Config,
    };

    const ID: &str = "11111111-1111-4111-8111-111111111111";
    const OWN: &str = "22222222-2222-4222-8222-222222222222";

    fn configuration() -> Config {
        let mut p = vec![a("0"); 77];
        let h = l(vec![
            a("3"),
            l(vec![a("1"), a("0"), a(OWN)]),
            s("SyntheticExtension"),
            l(vec![a("0")]),
            s(""),
            a("1"),
            a("1"),
            a(PROPERTIES[1].0),
            a("3"),
            a(ZERO),
            a("0"),
        ]);
        p[0] = a("76");
        p[1] = l(vec![a("0"), h]);
        p[3] = a("1");
        for i in [4, 5, 6, 7, 8] {
            p[i] = l(vec![a("0")]);
        }
        for i in [14, 15, 42] {
            p[i] = s("");
        }
        p[26] = a("80316");
        p[42] = s("Synthetic_");
        p[43] = a("80316");
        p[44] = a("2");
        let root = l(vec![
            a("2"),
            l(vec![a(ID)]),
            a("1"),
            l(vec![
                a("9cd510cd-abfc-11d4-9434-004095e12fc7"),
                l(vec![a("1"), l(p), a("0")]),
            ]),
        ]);
        let rec = brace::write(&root).unwrap();
        let entries = [
            ("configinfo", format!("{{0}},\r\n{{2,{ID},}},\r\n{{0}}")),
            (ID, rec),
        ]
        .into_iter()
        .map(|(name, text)| Entry {
            name: name.into(),
            created: 0,
            modified: 0,
            data: container::deflate(text.as_bytes()),
        })
        .collect();
        Config::from_entries(entries, vec![(Width::W32, 2)]).unwrap()
    }

    #[test]
    fn configinfo_root_exports_extension_properties_and_module_state() {
        let cfg = configuration();
        assert!(is_extension(&cfg));
        assert_eq!(super::super::classes_tools::compat_mode(&cfg), Some(80316));
        let data = super::super::classes_tools::configuration(
            &cfg,
            &super::super::common::Index::new(&cfg),
        )
        .unwrap();
        let data = export(&cfg, &data).unwrap();
        let doc = dom::parse(std::str::from_utf8(&data).unwrap()).unwrap();
        let obj = doc.req("Configuration").unwrap();
        let p = obj.req("Properties").unwrap();
        assert_eq!(p.text_of("NamePrefix").unwrap(), "Synthetic_");
        assert_eq!(p.text_of("ConfigurationExtensionPurpose").unwrap(), "AddOn");
        assert_eq!(
            p.text_of("ConfigurationExtensionCompatibilityMode")
                .unwrap(),
            "Version8_3_16"
        );
        assert_eq!(p.text_of("ObjectBelonging").unwrap(), "Adopted");
        assert!(p.child("DefaultRunMode").is_none());
        assert_eq!(
            obj.req("InternalInfo")
                .unwrap()
                .req("xr:PropertyState")
                .unwrap()
                .text_of("xr:Property")
                .unwrap(),
            "ManagedApplicationModule"
        );
        assert!(!data.contains(&1));
    }

    #[test]
    fn adopted_property_filter_and_unknown_state_are_loss_averse() {
        let xml=format!("<CommonModule uuid=\"{OWN}\"><Properties><Name>Borrowed</Name><Synonym/><Comment/><Server>true</Server><Privileged>true</Privileged></Properties></CommonModule>");
        let mut obj = dom::parse(&xml).unwrap();
        let n=brace::parse(&format!("{{3,{{1,0,{OWN}}},\"Borrowed\",{{0}},\"\",1,1,6275a02e-96f0-4347-975a-2d661e6a0675,2,{ZERO},0}}")).unwrap();
        rewrite(&mut obj, &n).unwrap();
        let p = obj.req("Properties").unwrap();
        assert!(p.child("Server").is_some());
        assert!(p.child("Privileged").is_none());
        assert!(p.child("Synonym").is_none());
        let mut obj = dom::parse(&xml).unwrap();
        let unknown = brace::parse(&format!(
            "{{3,{{1,0,{OWN}}},\"Borrowed\",{{0}},\"\",1,1,{ID},3,{ZERO},0}}"
        ))
        .unwrap();
        assert!(rewrite(&mut obj, &unknown).unwrap_err().contains(ID));
    }

    #[test]
    fn extension_compatibility_reaches_the_form_importer() {
        let xml=format!("<MetaDataObject><Configuration uuid=\"{ID}\"><Properties><Name>SyntheticExtension</Name><ConfigurationExtensionPurpose>AddOn</ConfigurationExtensionPurpose><ConfigurationExtensionCompatibilityMode>Version8_3_16</ConfigurationExtensionCompatibilityMode></Properties><ChildObjects/></Configuration></MetaDataObject>");
        let doc = dom::parse(&xml).unwrap();
        let ix = super::super::imp::index::Index::build_from(&doc, &|_| None).unwrap();
        assert_eq!(ix.compat, Some(80316));
    }
}
