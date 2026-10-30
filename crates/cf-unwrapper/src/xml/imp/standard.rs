//! Стандартные реквизиты `<StandardAttributes>` (обратное `xml/standard.rs`):
//! `{1,N,{-код},510405d3-…,{14,25,<guid свойства>,<значение>,…},…}` — реквизиты в порядке XML, словарь — по GUID.
//! Словарь берется по умолчанию (как у загрузки платформы) и меняется по свойствам XML; свойства, GUID которых не
//! установлен (`UNRESOLVED` выгрузки), должны иметь значение по умолчанию — иначе отказ.

use super::attr::value;
use super::dom::El;
use super::index::Index;
use super::tpl::{a, boolean, l, mlstring, s};
use crate::brace::Node;
use crate::xml::standard::NAMES;

const STD_ATTR: &str = "510405d3-2a0c-4fea-960a-7fee59b32f9b";

/// Словарь свойств по умолчанию (запись загрузки платформы, версия 14, 25 свойств).
const DICT: &str = "{14,25,\
1183c14f-f814-49c6-9233-a3c26b3f64cf,{\"#\",9ad557b1-249e-48dc-824b-3e149ecf10a6,{3,0,0}},\
2723eb98-b4c1-498a-a6f3-70444757902f,{\"#\",98ea8e5a-b586-442b-b944-6e3447734aa7,0},\
2bbba66b-fabf-4863-8ba3-54b3c64c896e,{\"B\",0},\
2c8143d5-4248-4c43-8bfb-307c0be2e415,{\"B\",0},\
33c74a4d-561f-4bc0-9eaa-8d21c893c0a9,{\"#\",ad3615c5-aae6-4725-89be-91827523abd9,{ad3615c5-aae6-4725-89be-91827523abd9,0}},\
3b10624f-1e3d-495d-8093-25225efc5313,{\"#\",502b7765-f89c-4fd0-924f-0a28d3dc09b7,{502b7765-f89c-4fd0-924f-0a28d3dc09b7,0}},\
3eaf5a8b-06d6-47b0-ac7d-a9698247f499,{\"U\"},\
4690ff70-e3fa-4914-9127-6a9acc5fc949,{\"#\",87024738-fc2a-4436-ada1-df79d395c424,{0}},\
4de03908-56f4-4396-a61e-17253afca9ac,{\"B\",0},\
580c29e2-8af4-4258-882a-7cf8073e61c8,{\"#\",87024738-fc2a-4436-ada1-df79d395c424,{0}},\
6c4f7074-e7d4-48eb-b31b-132873666262,{\"#\",157fa490-4ce9-11d4-9415-008048da11f9,{1,00000000-0000-0000-0000-000000000000}},\
6e3a1131-37a3-4da5-8895-572d9d0c9db6,{\"#\",ace3fd07-11b2-477e-ab7f-36f0ea37c8dd,{ace3fd07-11b2-477e-ab7f-36f0ea37c8dd,2}},\
7ba608f2-e654-42a3-8885-334fe88ca910,{\"#\",12ca4003-ac70-450e-b897-37faf86bd313,0},\
88149a78-9448-4767-867b-0e650d165d2e,{\"#\",87024738-fc2a-4436-ada1-df79d395c424,{0}},\
90ae4b5d-e0fd-49ef-a008-d67c1e75038c,{\"B\",0},\
9288a8ed-b259-46d0-a8e3-70d87956ff2d,{\"#\",d46ea122-3201-4e5e-bed4-e669c6e463c8,{d46ea122-3201-4e5e-bed4-e669c6e463c8,1}},\
b02800e9-a8d1-42ab-9a12-f673e92be968,{\"B\",0},\
c65a541f-0b91-4f33-bc88-fbaaa57f9992,{\"U\"},\
cf4abea3-37b2-11d4-940f-008048da11f9,{\"#\",87024738-fc2a-4436-ada1-df79d395c424,{0}},\
cf4abea4-37b2-11d4-940f-008048da11f9,{\"S\",\"\"},\
d4232326-022b-421e-b6d3-88e418f74327,{\"#\",3b8e6bdd-d648-49d5-af2f-d46d84f87dd5,{3b8e6bdd-d648-49d5-af2f-d46d84f87dd5,1}},\
e3da683b-c54a-457a-a243-b9b4f9bf76dd,{\"#\",b76a58b9-2a56-4e46-bb31-8e04ad9f31ae,{5006,0}},\
e6b3f5f3-bdf3-4ad0-bc60-7323b3feb208,{\"U\"},\
f49e4ced-4033-4e6c-8755-9fbaaccd6078,{\"S\",\"\"},\
fcf503b8-1c06-454a-970c-06413e64aee5,{\"#\",f2eaae14-91a7-47b9-9d69-097877f41580,{0,0}}}";

/// Свойства, которые в БП не меняются ни разу (GUID не установлен): значение XML должно быть таким.
const FIXED: &[(&str, &str)] = &[
    ("xr:MultiLine", "false"),
    ("xr:ExtendedEdit", "false"),
    ("xr:PasswordMode", "false"),
    ("xr:MarkNegatives", "false"),
    ("xr:CreateOnInput", "Auto"),
    ("xr:QuickChoice", "Auto"),
    ("xr:ChoiceHistoryOnInput", "Auto"),
];

fn set(d: &mut Node, guid: &str, v: Node) -> Result<(), String> {
    let l = d.list_mut().ok_or("словарь")?;
    let i = l.iter().position(|n| n.atom() == Some(guid)).ok_or(format!("нет свойства {guid}"))?;
    l[i + 1] = v;
    Ok(())
}

/// `{"#",<тип>,{<тип>,<n>}}` — перечисление платформы.
fn enum_val(t: &str, n: &str) -> Node {
    l(vec![s("#"), a(t), l(vec![a(t), a(n)])])
}

fn wrapped_ml(e: &El) -> Result<Node, String> {
    Ok(l(vec![s("#"), a("87024738-fc2a-4436-ada1-df79d395c424"), mlstring(e)?]))
}

/// `<StandardAttributes>` → `{1,N,…}` для класса `class`.
pub fn attributes(x: &Index, class: &str, owner: &str, e: &El) -> Result<Node, String> {
    let names = NAMES.iter().find(|(c, _)| *c == class).map(|(_, t)| *t).ok_or(format!("стандартные реквизиты класса {class} не загружаются"))?;
    attributes_named(x, names, owner, e)
}

/// То же по своей таблице кодов (у табличных частей код LineNumber — по классу владельца).
pub fn attributes_named(x: &Index, names: &[(&str, &str)], owner: &str, e: &El) -> Result<Node, String> {
    let mut v = vec![a("1"), a("0")];
    for sa in e.elems() {
        if sa.name != "xr:StandardAttribute" {
            return Err(format!("строка {}: <{}> в StandardAttributes", sa.line, sa.name));
        }
        let name = sa.req_attr("name")?;
        // субконто регистра бухгалтерии: `{<N-1>,91162600-…}` (ExtDimensionN), `{<N-1>,b3b48b29-…}` (ExtDimensionTypeN)
        let ext = |pre: &str, g: &str| name.strip_prefix(pre).and_then(|n| n.parse::<usize>().ok()).filter(|n| *n >= 1).map(|n| l(vec![a((n - 1).to_string()), a(g)]));
        let code = match names.iter().find(|(_, n)| *n == name) {
            Some((c, _)) => l(vec![a(*c)]),
            None => ext("ExtDimensionType", "b3b48b29-d652-47ab-9d21-7e06768c31b5")
                .or_else(|| ext("ExtDimension", "91162600-3161-4326-89a0-4a7cecd5092a"))
                .ok_or(format!("строка {}: незнакомый стандартный реквизит {name}", sa.line))?,
        };
        v.push(code);
        v.push(a(STD_ATTR));
        v.push(dict(x, owner, sa)?);
    }
    v[1] = a(((v.len() - 2) / 3).to_string());
    Ok(l(v))
}

/// План обмена без ExchangeDate (-14) в XML (старые объекты): платформа добавляет его первым со свойствами по
/// умолчанию (21 из 26 планов БП).
pub fn exchange_date_first(n: &mut Node) -> Result<(), String> {
    let v = n.list_mut().ok_or("стандартные реквизиты")?;
    if v.iter().any(|c| c.list().first().and_then(Node::atom) == Some("-14")) {
        return Ok(());
    }
    let d = crate::brace::Doc::parse(DICT)?.roots.remove(0);
    v.splice(2..2, [l(vec![a("-14")]), a(STD_ATTR), d]);
    let k = (v.len() - 2) / 3;
    v[1] = a(k.to_string());
    Ok(())
}

fn dict(x: &Index, owner: &str, sa: &El) -> Result<Node, String> {
    let mut d = crate::brace::Doc::parse(DICT)?.roots.remove(0);
    let at = |e: &El| format!("строка {}: <{}>", e.line, e.name);
    for p in sa.elems() {
        p.touch();
        if let Some((_, want)) = FIXED.iter().find(|(t, _)| *t == p.name) {
            let got = if p.attr("xsi:nil").is_some() { "nil".to_string() } else { p.text()? };
            if got != *want {
                return Err(format!("{}: {got:?} — GUID свойства не установлен, загружается только {want:?}", at(p)));
            }
            continue;
        }
        match p.name.as_str() {
            // форма выбора: `{"#",157fa490-…,{1,<uuid формы>}}` (обратное выгрузке standard.rs)
            "xr:ChoiceForm" => {
                let t = p.text()?;
                if !t.is_empty() {
                    let id = x.id(&t).ok_or_else(|| format!("{}: нет формы {t}", at(p)))?;
                    set(&mut d, "6c4f7074-e7d4-48eb-b31b-132873666262", l(vec![s("#"), a("157fa490-4ce9-11d4-9415-008048da11f9"), l(vec![a("1"), a(id)])]))?;
                }
            }
            "xr:MinValue" | "xr:MaxValue" => {
                if p.attr("xsi:nil") != Some("true") {
                    return Err(format!("{}: GUID свойства не установлен, загружается только nil", at(p)));
                }
            }
            // связь по типу со стандартным реквизитом владельца: `{3,1,{<код>},<элемент>}`
            "xr:LinkByType" => {
                if p.elems().next().is_some() {
                    let dp = p.text_of("xr:DataPath")?;
                    let name = dp.strip_prefix(owner).and_then(|r| r.strip_prefix(".StandardAttribute.")).ok_or(format!("{}: путь {dp}", at(p)))?;
                    let class = owner.split('.').next().unwrap_or_default();
                    let names = NAMES.iter().find(|(c, _)| *c == class).map(|(_, n)| *n).unwrap_or(&[]);
                    let code = names.iter().find(|(_, n)| *n == name).map(|(c, _)| *c).ok_or(format!("{}: путь {dp}", at(p)))?;
                    let item = p.text_of("xr:LinkItem")?;
                    let item = super::tpl::int(&item).ok_or(format!("{}: LinkItem {item:?}", at(p)))?;
                    set(&mut d, "1183c14f-f814-49c6-9233-a3c26b3f64cf", l(vec![s("#"), a("9ad557b1-249e-48dc-824b-3e149ecf10a6"), l(vec![a("3"), a("1"), l(vec![a(code)]), a(item)])]))?;
                } else {
                    p.text()?;
                }
            }
            "xr:FillChecking" => set(&mut d, "2723eb98-b4c1-498a-a6f3-70444757902f", l(vec![s("#"), a("98ea8e5a-b586-442b-b944-6e3447734aa7"), super::tpl::token(p, &[("0", "DontCheck"), ("1", "ShowError")])?]))?,
            "xr:FillFromFillingValue" => set(&mut d, "2c8143d5-4248-4c43-8bfb-307c0be2e415", l(vec![s("B"), boolean(p)?]))?,
            "xr:ToolTip" => set(&mut d, "4690ff70-e3fa-4914-9127-6a9acc5fc949", wrapped_ml(p)?)?,
            "xr:Format" => set(&mut d, "580c29e2-8af4-4258-882a-7cf8073e61c8", wrapped_ml(p)?)?,
            "xr:EditFormat" => set(&mut d, "88149a78-9448-4767-867b-0e650d165d2e", wrapped_ml(p)?)?,
            "xr:Synonym" => set(&mut d, "cf4abea3-37b2-11d4-940f-008048da11f9", wrapped_ml(p)?)?,
            "xr:Comment" => set(&mut d, "cf4abea4-37b2-11d4-940f-008048da11f9", l(vec![s("S"), s(super::tpl::crlf(&p.text()?))]))?,
            "xr:Mask" => set(&mut d, "f49e4ced-4033-4e6c-8755-9fbaaccd6078", l(vec![s("S"), s(super::tpl::crlf(&p.text()?))]))?,
            "xr:DataHistory" => set(&mut d, "9288a8ed-b259-46d0-a8e3-70d87956ff2d", enum_val("d46ea122-3201-4e5e-bed4-e669c6e463c8", super::tpl::token(p, &[("0", "DontUse"), ("1", "Use")])?.atom().unwrap_or_default()))?,
            "xr:FullTextSearch" => set(&mut d, "d4232326-022b-421e-b6d3-88e418f74327", enum_val("3b8e6bdd-d648-49d5-af2f-d46d84f87dd5", super::tpl::token(p, &[("0", "DontUse"), ("1", "Use")])?.atom().unwrap_or_default()))?,
            // 3b10624f-…: у владельца справочника (Deny в XML) в записи 2, у остальных 0 (TransformValues)
            "xr:TypeReductionMode" => set(&mut d, "3b10624f-1e3d-495d-8093-25225efc5313", enum_val("502b7765-f89c-4fd0-924f-0a28d3dc09b7", super::tpl::token(p, &[("0", "TransformValues"), ("2", "Deny")])?.atom().unwrap_or_default()))?,
            "xr:FillValue" => set(&mut d, "e6b3f5f3-bdf3-4ad0-bc60-7323b3feb208", value(x, p)?)?,
            "xr:ChoiceParameterLinks" => set(&mut d, "e3da683b-c54a-457a-a243-b9b4f9bf76dd", l(vec![s("#"), a("b76a58b9-2a56-4e46-bb31-8e04ad9f31ae"), super::attr::links(x, owner, p)?]))?,
            "xr:ChoiceParameters" => set(&mut d, "fcf503b8-1c06-454a-970c-06413e64aee5", l(vec![s("#"), a("f2eaae14-91a7-47b9-9d69-097877f41580"), super::attr::choice_parameters(x, p)?]))?,
            _ => return Err(format!("{}: незнакомое свойство стандартного реквизита", at(p))),
        }
    }
    Ok(d)
}
