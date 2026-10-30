//! Права роли `Roles/<Имя>/Ext/Rights.xml` → запись `<id роли>.0` (обратное `xml/rights.rs`, устройство записи —
//! docs/research/xml-rights.md, правила загрузки — xml-import.md «Права»).
//!
//! Чего в XML нет, платформа выводит: снятое в XML право снимает зависимые (они пишутся явно), права интерфейса
//! конфигурации поздних версий (`CONFIG_TRUE`) без значения получают значение «для новых объектов». Зависимости и
//! порядок прав в записи сняты с загрузок платформы ([`tab::KINDS`]); не встречавшееся сочетание — отказ.

mod tab;

/// Права вида по порядку списка и строки зависимостей (`tab::KINDS`: строка F, символ R — `+` снятие F снимает R),
/// если известны целиком (без «?» — из кода платформы).
pub(crate) fn kind_deps(kind: &str) -> Option<(&'static [&'static str], &'static [&'static str])> {
    tab::KINDS.iter().find(|k| k.0 == kind).filter(|k| k.2.iter().all(|r| !r.contains('?'))).map(|k| (k.1, k.2))
}

use super::dom::El;
use super::index::Index;
use super::tpl::{a, crlf, l, s};
use super::Rec;
use crate::brace::{Doc, Node};
use crate::xml::rights::{std_key, CONFIG_TRUE, RIGHTS};

/// Виды, у которых признак поля в ключе объекта — 1 (умолчание прав — «для реквизитов»).
const FIELDS: &[&str] = &["Attribute", "StandardAttribute", "Dimension", "Resource", "AccountingFlag", "ExtDimensionAccountingFlag", "AddressingAttribute", "TabularSection", "StandardTabularSection"];

/// Виды вложенных объектов, на которые ссылается поле ограничения (краткое имя в XML).
const FIELD_KINDS: &[&str] = &["Attribute", "Dimension", "Resource", "AddressingAttribute", "AccountingFlag", "ExtDimensionAccountingFlag", "TabularSection"];

const ROOT_ATTRS: &[(&str, &str)] = &[
    ("xmlns", "http://v8.1c.ru/8.2/roles"),
    ("xmlns:xs", "http://www.w3.org/2001/XMLSchema"),
    ("xmlns:xsi", "http://www.w3.org/2001/XMLSchema-instance"),
    ("xsi:type", "Rights"),
    ("version", "2.21"),
];

/// Флаг роли в записи: истина — 1, ложь — 4294967295.
fn flag(e: &El) -> Result<bool, String> {
    match e.text()?.as_str() {
        "true" => Ok(true),
        "false" => Ok(false),
        v => Err(format!("строка {}: <{}> {v:?}", e.line, e.name)),
    }
}

struct Obj {
    id: String,
    key: Node,
    rights: Node,
}

pub fn record(doc: &El, x: &Index) -> Result<Rec, String> {
    let (root_id, conf) = (x.root.0.as_str(), x.root.1.as_str());
    if doc.name != "Rights" || doc.attrs.len() != ROOT_ATTRS.len() || ROOT_ATTRS.iter().any(|(k, v)| doc.attr(k) != Some(v)) {
        return Err("корень <Rights> не как у выгрузки 2.21".into());
    }
    let mut kids = doc.elems();
    let mut next = |name: &str| match kids.next() {
        Some(e) if e.name == name => Ok(e),
        other => Err(format!("ожидался <{name}>, а не {:?}", other.map(|e| &e.name))),
    };
    let set_new = flag(next("setForNewObjects")?)?;
    let set_attr = flag(next("setForAttributesByDefault")?)?;
    // поле 5 записи: 1 — true, 0 — false (выгрузка `rights::render`)
    let indep = flag(next("independentRightsOfChildObjects")?)?;
    let mut objs = Vec::new();
    let mut templates = vec![a("0")];
    for e in doc.elems().skip(3) {
        match e.name.as_str() {
            "object" if templates.len() == 1 => objs.push(object(e, x, root_id, conf, set_new, set_attr)?),
            "restrictionTemplate" => {
                let (n, c) = (e.text_of("name")?, e.text_of("condition")?);
                if e.elems().count() != 2 {
                    return Err(format!("строка {}: лишнее в <restrictionTemplate>", e.line));
                }
                templates.push(l(vec![s(crlf(&n)), s(crlf(&c))]));
            }
            _ => return Err(format!("строка {}: <{}> не на месте", e.line, e.name)),
        }
    }
    templates[0] = a((templates.len() - 1).to_string());
    // объекты — в порядке обхода boost::unordered_map платформы с хэшем Data1 GUID объекта, вставка — по XML
    let ordered = super::hashorder::boost_order(&objs.iter().collect::<Vec<_>>(), |o| super::hashorder::guid_hash(&o.id));
    let mut list = vec![a(ordered.len().to_string())];
    list.extend(ordered.into_iter().map(|o| l(vec![o.key.clone(), o.rights.clone()])));
    let yes = |b: bool| a(if b { "1" } else { "4294967295" });
    let rec = l(vec![a("10"), l(list), l(templates), yes(set_new), yes(set_attr), a(if indep { "1" } else { "0" }), a("4294967295")]);
    Ok(Rec::Raw(Doc::new(true, vec![rec]).write()?.into_bytes()))
}

/// Ключ `{1,<uuid>,<глубина>,{n,<коллекция>}…,<поле>}`, вид объекта (для таблицы прав) и uuid для порядка.
fn key(x: &Index, root_id: &str, conf: &str, name: &str) -> Result<(String, Node, &'static str, bool), String> {
    let parts: Vec<&str> = name.split('.').collect();
    let unknown = || format!("объект {name:?} не найден");
    let kind_of = |k: &str| tab::KINDS.iter().map(|t| t.0).find(|n| *n == k).ok_or(format!("{name}: права объектов вида {k} в загрузках не встречались"));
    if parts[0] == "Configuration" {
        if parts.len() != 2 || parts[1] != conf {
            return Err(unknown());
        }
        return Ok((root_id.to_string(), l(vec![a("1"), a(root_id), a("0"), a("0")]), kind_of("Configuration")?, false));
    }
    if let Some(i) = parts.iter().position(|p| *p == "StandardAttribute" || *p == "StandardTabularSection") {
        // стандартные реквизиты — только у объекта верхнего уровня (у табличных частей выгрузка их не пишет)
        if i != 2 {
            return Err(unknown());
        }
        let id = x.id(&parts[..2].join(".")).ok_or_else(unknown)?;
        let path = std_key(parts[0], &parts[2..].join(".")).ok_or_else(|| format!("{name}: стандартный реквизит не известен"))?;
        let mut k = vec![a("1"), a(id), a(path.len().to_string())];
        k.extend(path.iter().map(|(n, g)| l(vec![a(n.to_string()), a(*g)])));
        k.push(a("1"));
        return Ok((id.to_string(), l(k), kind_of(parts[parts.len() - 2])?, true));
    }
    let id = x.id(name).ok_or_else(unknown)?;
    let kind = if parts.len() == 2 { parts[0] } else { parts[parts.len() - 2] };
    let field = FIELDS.contains(&kind);
    Ok((id.to_string(), l(vec![a("1"), a(id), a("0"), a(if field { "1" } else { "0" })]), kind_of(kind)?, field))
}

fn object(e: &El, x: &Index, root_id: &str, conf: &str, set_new: bool, set_attr: bool) -> Result<Obj, String> {
    let at = |m: String| format!("строка {}: {m}", e.line);
    let name = e.text_of("name")?;
    let (id, key, kind, field) = key(x, root_id, conf, &name).map_err(at)?;
    let &(_, names, deps, ord) = tab::KINDS.iter().find(|t| t.0 == kind).unwrap();
    let pos = |r: &str| names.iter().position(|n| *n == r);
    // (номер права вида, значение, ограничения)
    let mut set: Vec<(usize, bool, Vec<&El>)> = Vec::new();
    for r in e.elems() {
        match r.name.as_str() {
            "name" => continue,
            "right" => {}
            _ => return Err(at(format!("<{}> в <object>", r.name))),
        }
        let rn = r.text_of("name")?;
        let i = pos(&rn).ok_or_else(|| at(format!("{name}: право {rn} у вида {kind} не встречено")))?;
        let v = flag(r.req("value")?)?;
        let rls: Vec<&El> = r.all("restrictionByCondition").collect();
        if r.elems().count() != 2 + rls.len() || set.iter().any(|(j, ..)| *j == i) {
            return Err(at(format!("{name}: право {rn} — лишнее или повтор")));
        }
        set.push((i, v, rls));
    }
    if set.is_empty() {
        return Err(at(format!("{name}: объект без прав")));
    }
    // (право, значение), которых в XML нет, а в записи они есть
    let mut added: Vec<(usize, bool)> = Vec::new();
    // права интерфейса поздних версий без значения — явно со значением «для новых объектов»
    if kind == "Configuration" {
        for n in CONFIG_TRUE {
            let i = pos(n).ok_or_else(|| at(format!("право {n} конфигурации не встречено")))?;
            if !set.iter().any(|(j, ..)| *j == i) {
                added.push((i, set_new));
            }
        }
    }
    let default = if field { set_attr } else { set_new };
    let given = |r: usize, added: &[(usize, bool)]| set.iter().map(|(j, v, _)| (*j, *v)).chain(added.iter().copied()).find(|(j, _)| *j == r).map(|(_, v)| v);
    // снятое право снимает всех потомков в графе прав класса и пишет их явно при любом умолчании
    // (backend.dll FUN_180eebfb0 «prolongDeny»: состояние 3 потомкам по карте детей дескриптора, рекурсивно);
    // право снимается, если его снимает хоть одно снятое; не известно ни про одно — отказ
    let falses: Vec<usize> = (0..names.len()).filter(|&f| given(f, &added) == Some(false)).collect();
    for r in 0..names.len() {
        if given(r, &added).is_some() || falses.is_empty() {
            continue;
        }
        let c: Vec<u8> = falses.iter().map(|&f| deps[f].as_bytes()[r]).collect();
        match (c.contains(&b'+'), c.contains(&b'?')) {
            (true, _) => added.push((r, false)),
            (false, false) => {}
            (false, true) => return Err(at(format!("{name}: снятые права и право {} — сочетание не встречено", names[r]))),
        }
    }
    let state = |r: usize| given(r, &added).unwrap_or(default);
    for f in (0..names.len()).filter(|&f| !state(f)) {
        if let Some(r) = (0..names.len()).find(|&r| state(r) && deps[f].as_bytes()[r] != b'-') {
            return Err(at(format!("{name}: {} без {} — сочетание не встречено", names[r], names[f])));
        }
    }
    // порядок записи: XML, добавленные — на свое место; неизвестный или нарушенный порядок — отказ
    let mut seq: Vec<usize> = set.iter().map(|(i, ..)| *i).collect();
    for p in seq.windows(2) {
        if ord[p[0]].as_bytes()[p[1]] != b'<' {
            return Err(at(format!("{name}: порядок прав {} и {} не как у платформы", names[p[0]], names[p[1]])));
        }
    }
    for &(r, _) in &added {
        if seq.iter().any(|&p| ord[p].as_bytes()[r] == b'?') {
            return Err(at(format!("{name}: место права {} в записи не известно", names[r])));
        }
        let at_pos = seq.iter().position(|&p| ord[p].as_bytes()[r] == b'>').unwrap_or(seq.len());
        seq.insert(at_pos, r);
    }
    let guid = |i: usize| RIGHTS.iter().find(|(_, n)| *n == names[i]).map(|(g, _)| *g).ok_or(format!("GUID права {}", names[i]));
    let mut pairs = Vec::new();
    let mut rls = Vec::new();
    for &i in &seq {
        let cond = set.iter().find(|(j, ..)| *j == i).map(|(_, _, c)| c.as_slice()).unwrap_or(&[]);
        let v = state(i);
        pairs.push(a(guid(i)?));
        pairs.push(a(if v { "1" } else { "-1" }));
        if !cond.is_empty() {
            rls.push((guid(i)?, restrictions(cond, x, &name)?));
        }
    }
    // ограничения — по GUID права
    rls.sort_by_key(|(g, _)| *g);
    let rls: Vec<Node> = rls.into_iter().map(|(g, r)| l(vec![a(g), r])).collect();
    let rights = if rls.is_empty() {
        let mut v = vec![a("0")];
        v.extend(pairs);
        l(v)
    } else {
        let mut v = vec![a("1"), a(seq.len().to_string())];
        v.extend(pairs);
        v.push(a(rls.len().to_string()));
        v.extend(rls);
        l(v)
    };
    Ok(Obj { id, key, rights })
}

/// Ограничения права `{N,{1,"условие",<число полей>,{{0},{0,<uuid поля>}}…}…}`: первым — ограничение без полей
/// (нет в XML — пустое `{1,"",0}`, выгрузка его не пишет), затем с полями в порядке XML.
fn restrictions(cond: &[&El], x: &Index, owner: &str) -> Result<Node, String> {
    let mut plain = None;
    let mut with_fields = Vec::new();
    for c in cond {
        let fields: Vec<String> = c.all("field").map(|f| f.text()).collect::<Result<_, _>>()?;
        let text = c.text_of("condition")?;
        if c.elems().count() != fields.len() + 1 {
            return Err(format!("строка {}: лишнее в <restrictionByCondition>", c.line));
        }
        let mut v = vec![a("1"), s(crlf(&text)), a(fields.len().to_string())];
        for f in &fields {
            let ids: Vec<&str> = FIELD_KINDS.iter().filter_map(|k| x.id(&format!("{owner}.{k}.{f}"))).collect();
            let [id] = ids[..] else { return Err(format!("строка {}: поле ограничения {f:?} у {owner} не найдено или неоднозначно", c.line)) };
            v.push(l(vec![l(vec![a("0")]), l(vec![a("0"), a(id)])]));
        }
        if fields.is_empty() {
            if plain.replace(l(v)).is_some() {
                return Err(format!("строка {}: два ограничения без полей", c.line));
            }
        } else {
            with_fields.push(l(v));
        }
    }
    let mut out = vec![a((1 + with_fields.len()).to_string()), plain.unwrap_or_else(|| l(vec![a("1"), s(""), a("0")]))];
    out.extend(with_fields);
    Ok(l(out))
}

#[cfg(test)]
mod tests {
    use super::*;

    const ROOT: &str = "<Rights xmlns=\"http://v8.1c.ru/8.2/roles\" xmlns:xs=\"http://www.w3.org/2001/XMLSchema\" xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\" xsi:type=\"Rights\" version=\"2.21\">";
    const READ: &str = "1c87578f-9e09-4ec0-a991-5629c87b1588";
    const VIEW: &str = "aa6448f2-be0f-42ea-ba26-1af7f52b5b65";
    const EDIT: &str = "b7bab52d-c1b1-4bd8-8276-02db08d42352";

    fn run(body: &str) -> Result<String, String> {
        let mut x = Index::default();
        x.root = ("33333333-3333-3333-3333-333333333333".into(), "Мини".into());
        x.full.insert("DocumentJournal.Ж".into(), "11111111-1111-1111-1111-111111111111".into());
        x.full.insert("InformationRegister.Р".into(), "22222222-2222-2222-2222-222222222222".into());
        x.full.insert("InformationRegister.Р.Dimension.Орг".into(), "44444444-4444-4444-4444-444444444444".into());
        let t = format!("{ROOT}<setForNewObjects>false</setForNewObjects><setForAttributesByDefault>true</setForAttributesByDefault><independentRightsOfChildObjects>false</independentRightsOfChildObjects>{body}</Rights>");
        let d = super::super::dom::parse(&t)?;
        let Rec::Raw(b) = record(&d, &x)? else { unreachable!() };
        assert!(d.unread().is_none());
        Ok(String::from_utf8(b).unwrap().replace("\r\n", ""))
    }

    #[test]
    fn dependents_restrictions_order() {
        let r = run(
            "<object><name>DocumentJournal.Ж</name><right><name>Read</name><value>false</value></right></object>\
             <object><name>InformationRegister.Р</name><right><name>Read</name><value>true</value><restrictionByCondition><field>Орг</field><condition>ГДЕ 1</condition></restrictionByCondition></right></object>\
             <object><name>InformationRegister.Р.Dimension.Орг</name><right><name>Edit</name><value>false</value></right></object>\
             <restrictionTemplate><name>Ш</name><condition>а\nб</condition></restrictionTemplate>",
        )
        .unwrap();
        // снятое Read журнала снимает View (пишется явно); ограничение с полем — после пустого без полей;
        // объекты — в порядке обхода хэш-таблицы (разные корзины — обратный порядок вставки)
        let want = format!(
            "\u{feff}{{10,{{3,{{{{1,44444444-4444-4444-4444-444444444444,0,1}},{{0,{EDIT},-1}}}},\
             {{{{1,22222222-2222-2222-2222-222222222222,0,0}},{{1,1,{READ},1,1,{{{READ},{{2,{{1,\"\",0}},{{1,\"ГДЕ 1\",1,{{{{0}},{{0,44444444-4444-4444-4444-444444444444}}}}}}}}}}}}}},\
             {{{{1,11111111-1111-1111-1111-111111111111,0,0}},{{0,{READ},-1,{VIEW},-1}}}}}},\
             {{1,{{\"Ш\",\"аб\"}}}},4294967295,1,0,4294967295}}"
        );
        assert_eq!(r, want);
        // право, которого у вида не было, и неизвестный объект — отказ
        assert!(run("<object><name>DocumentJournal.Ж</name><right><name>Posting</name><value>true</value></right></object>").is_err());
        assert!(run("<object><name>Catalog.Нет</name><right><name>Read</name><value>true</value></right></object>").is_err());
    }
}
