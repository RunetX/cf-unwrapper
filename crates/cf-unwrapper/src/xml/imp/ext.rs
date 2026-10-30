//! Вспомогательные файлы `Ext/*.xml` (обратное `xml/ext.rs`): расписание регламентного задания, состав плана обмена.

use super::dom::El;
use super::payload::check_ext_root_ns;
use super::tpl::{a, l, s};
use super::Rec;
use crate::brace::{Doc, Node};

/// `Schedule.xml` → `{начало,конец,время начала,время конца,время завершения,интервал,повтор,пауза,N,дни недели…,
/// неделя месяца,день месяца,N,месяцы…,период недель,период дней,N,{детальные}…}`. В XML у дат нет времени,
/// у времени — даты: в записи они нулевые (`00010101`, `000000`).
pub fn schedule(doc: &El) -> Result<Rec, String> {
    check_ext_root_ns(doc, "JobSchedule", "xmlns:ent", "http://v8.1c.ru/8.1/data/enterprise")?;
    let n = schedule_el(doc.req("Schedule")?)?;
    Ok(Rec::Raw(Doc::new(true, vec![n]).write()?.into_bytes()))
}

/// `Content.xml` → `{2,N,<uuid>,<авторегистрация 0 Deny|1 Allow>,…,0}`; порядок — обхода хэш-таблицы платформы
/// по Data1 uuid, вставка — в порядке XML ([`super::hashorder`]).
pub fn content(doc: &El, x: &super::index::Index) -> Result<Rec, String> {
    super::payload::check_ext_root(doc, "ExchangePlanContent")?;
    let mut items = Vec::new();
    for it in doc.elems() {
        if it.name != "Item" {
            return Err(format!("строка {}: <{}> в составе", it.line, it.name));
        }
        let id = super::tpl::reference(x, it.req("Metadata")?)?;
        let auto = super::tpl::token(it.req("AutoRecord")?, &[("0", "Deny"), ("1", "Allow")])?;
        items.push((id.atom().unwrap_or_default().to_string(), auto));
    }
    if items.is_empty() {
        doc.text()?;
    }
    let ordered = super::hashorder::msvc_order(&items, |(g, _)| super::hashorder::guid_hash(g));
    let mut v = vec![a("2"), a(ordered.len().to_string())];
    for (g, auto) in ordered {
        v.push(a(g));
        v.push(auto);
    }
    v.push(a("0"));
    Ok(Rec::Raw(Doc::new(true, vec![l(v)]).write()?.into_bytes()))
}

/// Пространства имен корня `Style.xml` (как у выгрузки, `xml/ui.rs` STYLE_HEAD).
const STYLE_NS: &[(&str, &str)] = &[
    ("xmlns", "http://v8.1c.ru/8.3/xcf/extrnprops"),
    ("xmlns:pal", "http://v8.1c.ru/8.1/data/ui/colors/palette"),
    ("xmlns:style", "http://v8.1c.ru/8.1/data/ui/style"),
    ("xmlns:sys", "http://v8.1c.ru/8.1/data/ui/fonts/system"),
    ("xmlns:v8ui", "http://v8.1c.ru/8.1/data/ui"),
    ("xmlns:web", "http://v8.1c.ru/8.1/data/ui/colors/web"),
    ("xmlns:win", "http://v8.1c.ru/8.1/data/ui/colors/windows"),
    ("xmlns:xr", "http://v8.1c.ru/8.3/xcf/readable"),
    ("xmlns:xs", "http://www.w3.org/2001/XMLSchema"),
    ("xmlns:xsi", "http://www.w3.org/2001/XMLSchema-instance"),
    ("version", "2.21"),
];

/// `Style.xml` стиля (обратное `xml/ui.rs` style_xml) → `{2,N,{<ключ>,<вид>,<значение>}…,{<цветов палитры>,…}}`
/// (backend.dll 0x180d4cc20): ключ `{-номер}` — элемент стиля платформы (`mxl::STYLE`), `{0,<uuid>}` — элемент стиля
/// конфигурации; вид 0 — цвет `{4,…}`, 1 — шрифт `{8,…}`, 2 — рамка-ссылка `{3,1,{-номер},0,0,0}`.
/// Элементы — в порядке обхода `std::unordered_map` стиля (core85 `BaseStyle`: 8 корзин, вставка 0x18001b2f0 —
/// [`super::hashorder::msvc_order`]); загрузка XML (backend 0x1806c3640, применение 0x1806c3a00) вставляет их в
/// порядке файла. Хэш ключа (core85 0x18001a1f0): `(i32)(номер + 5·Data1)` у ключа с uuid, у ключа без uuid — номер;
/// корзина — от него со знаком в 64 бита. Цвета палитры (Item с именем цвета палитры) не встречены — отказ.
pub fn style(doc: &El, x: &super::index::Index) -> Result<Rec, String> {
    if doc.name != "Style" || doc.attrs.len() != STYLE_NS.len() || STYLE_NS.iter().any(|(k, v)| doc.attr(k) != Some(v)) {
        return Err(format!("корень <{}>: не <Style> с пространствами имен выгрузки 2.21", doc.name));
    }
    let platform = |n: &str| crate::xml::mxl::STYLE.iter().find(|(_, s)| *s == n).map(|(k, _)| *k);
    let mut names = std::collections::HashSet::new();
    let mut items: Vec<(Node, u64)> = Vec::new();
    for it in doc.elems() {
        let err = |m: &str| format!("строка {}: элемент стиля: {m}", it.line);
        if it.name != "Item" {
            return Err(err(&format!("<{}>", it.name)));
        }
        let name = it.req_attr("name")?;
        if !names.insert(name) {
            return Err(err(&format!("{name} повторяется")));
        }
        let (key, hash) = match (platform(name), x.id(name).filter(|_| name.starts_with("StyleItem."))) {
            (Some(n), _) => (l(vec![a(n.to_string())]), n as u64),
            (None, Some(id)) => (l(vec![a("0"), a(id)]), (super::hashorder::guid_hash(id) as u32).wrapping_mul(5) as i32 as i64 as u64),
            _ => return Err(err(&format!("{name}: не элемент стиля платформы и не элемент стиля конфигурации"))),
        };
        let mut vals = it.elems();
        let (Some(v), None) = (vals.next(), vals.next()) else { return Err(err(&format!("{name}: не одно значение"))) };
        let (kind, val) = match v.name.as_str() {
            "Color" => {
                let t = v.text()?;
                ("0", super::ui::color(&t).ok_or_else(|| err(&format!("цвет {t:?}")))?)
            }
            "Font" => {
                v.text()?;
                ("1", super::ui::font(v)?)
            }
            "Border" => {
                v.text()?;
                let r = v.req_attr("ref")?;
                let n = r.strip_prefix("style:").and_then(platform).ok_or_else(|| err(&format!("рамка {r:?}")))?;
                ("2", l(vec![a("3"), a("1"), l(vec![a(n.to_string())]), a("0"), a("0"), a("0")]))
            }
            t => return Err(err(&format!("{name}: значение <{t}>"))),
        };
        items.push((l(vec![key, a(kind), val]), hash));
    }
    let ordered = super::hashorder::msvc_order(&items, |(_, h)| *h);
    let mut v = vec![a("2"), a(ordered.len().to_string())];
    v.extend(ordered.into_iter().map(|(n, _)| n));
    v.push(l(vec![a("0")]));
    Ok(Rec::Raw(Doc::new(true, vec![l(v)]).write()?.into_bytes()))
}

/// Команда командного интерфейса (обратное `command` выгрузки): `<полное имя команды>` → `{0,<uuid>}`,
/// `<объект>.StandardCommand.OpenList|Create|Open` → `{0|1|100,<uuid объекта>}` (Open у отчета и обработки — 0),
/// `<вид>:<uuid>` → `{<вид>,<uuid>}`, `0` → `{0}`.
fn ci_command(x: &super::index::Index, t: &str, at: &El) -> Result<Node, String> {
    let err = || format!("строка {}: команда {t:?}", at.line);
    if t == "0" {
        return Ok(l(vec![a("0")]));
    }
    if let Some((k, g)) = t.split_once(':').filter(|(k, g)| super::tpl::int(k).is_some() && crate::brace::is_guid(g)) {
        return Ok(l(vec![a(k), a(g)]));
    }
    if let Some((obj, std)) = t.split_once(".StandardCommand.") {
        let class = obj.split('.').next().unwrap_or_default();
        // номера по классам — `form::STD_COMMANDS`
        let kind = crate::xml::form::STD_COMMANDS.iter().find(|(c, _, s)| *c == class && *s == std).map(|(_, k, _)| k.to_string()).ok_or_else(err)?;
        return Ok(l(vec![a(kind), a(x.id(obj).ok_or_else(err)?)]));
    }
    Ok(l(vec![a("0"), a(x.id(t).ok_or_else(err)?)]))
}

/// Группа: стандартная или CommandGroup.
fn ci_group(x: &super::index::Index, e: &El) -> Result<Node, String> {
    let t = e.text()?;
    if let Some((g, _)) = crate::xml::classes::STD_COMMAND_GROUPS.iter().find(|(_, n)| *n == t) {
        return Ok(a(*g));
    }
    if crate::brace::is_guid(&t) {
        return Ok(a(t));
    }
    x.id(&t).map(a).ok_or(format!("строка {}: нет группы {t}", e.line))
}

/// Видимость `<Visibility><xr:Common>…</xr:Common><xr:Value name="Role.…">…</xr:Value>…` →
/// `{0,{0,{"B",<общая>},N,<роль>,{"B",<значение>},…}}`.
pub fn visibility(x: &super::index::Index, e: &El) -> Result<Node, String> {
    let b = |el: &El| -> Result<Node, String> { Ok(l(vec![s("B"), super::tpl::boolean(el)?])) };
    let mut v = vec![a("0"), b(e.req("xr:Common")?)?, a("0")];
    let mut n = 0;
    for r in e.all("xr:Value") {
        let name = r.req_attr("name")?;
        let id = if crate::brace::is_guid(name) { name.to_string() } else { x.id(name).ok_or(format!("строка {}: нет роли {name}", r.line))?.to_string() };
        v.push(a(id));
        v.push(b(r)?);
        n += 1;
    }
    v[2] = a(n.to_string());
    Ok(l(vec![a("0"), l(v)]))
}

/// `CommandInterface.xml` → `{7,<видимость>,<размещение>,<порядок команд>,<порядок подсистем>,<порядок групп>,
/// <видимость подсистем>}`; часть — `0` или `1,N,<элементы>` подряд в списке. Порядок — как в XML.
pub fn command_interface(doc: &El, x: &super::index::Index) -> Result<Rec, String> {
    super::payload::check_ext_root(doc, "CommandInterface")?;
    let mut v = vec![a("7")];
    let part = |v: &mut Vec<Node>, items: Vec<Vec<Node>>| {
        if items.is_empty() {
            v.push(a("0"));
        } else {
            v.push(a("1"));
            v.push(a(items.len().to_string()));
            v.extend(items.into_iter().flatten());
        }
    };
    let mut seen = Vec::new();
    for e in doc.elems() {
        seen.push(e.name.clone());
    }
    let order = ["SubsystemsVisibility", "CommandsVisibility", "CommandsPlacement", "CommandsOrder", "SubsystemsOrder", "GroupsOrder"];
    if seen.iter().any(|n| !order.contains(&n.as_str())) {
        return Err(format!("CommandInterface: незнакомый раздел ({seen:?})"));
    }
    let sect = |name: &str| doc.child(name);
    let mut items = Vec::new();
    if let Some(s0) = sect("CommandsVisibility") {
        for c in s0.all("Command") {
            items.push(vec![ci_command(x, c.req_attr("name")?, c)?, visibility(x, c.req("Visibility")?)?]);
        }
    }
    part(&mut v, std::mem::take(&mut items));
    if let Some(s1) = sect("CommandsPlacement") {
        for c in s1.all("Command") {
            let place = super::tpl::token(c.req("Placement")?, &[("0", "Auto"), ("1", "Manual")])?;
            items.push(vec![ci_command(x, c.req_attr("name")?, c)?, ci_group(x, c.req("CommandGroup")?)?, place]);
        }
    }
    part(&mut v, std::mem::take(&mut items));
    if let Some(s2) = sect("CommandsOrder") {
        for c in s2.all("Command") {
            items.push(vec![ci_group(x, c.req("CommandGroup")?)?, ci_command(x, c.req_attr("name")?, c)?]);
        }
    }
    part(&mut v, std::mem::take(&mut items));
    if let Some(s3) = sect("SubsystemsOrder") {
        for c in s3.all("Subsystem") {
            items.push(vec![super::tpl::reference(x, c)?]);
        }
    }
    part(&mut v, std::mem::take(&mut items));
    if let Some(s4) = sect("GroupsOrder") {
        for c in s4.all("Group") {
            // группу с нулевым GUID (так выгрузка пишет пустую ссылку) загрузка платформы отбрасывает
            let g = ci_group(x, c)?;
            if g.atom() != Some(crate::xml::common::ZERO) {
                items.push(vec![g]);
            }
        }
    }
    part(&mut v, std::mem::take(&mut items));
    // видимость подсистем (корень конфигурации): подсистема — полное имя или GUID (нет в конфигурации), видимость
    if let Some(s5) = sect("SubsystemsVisibility") {
        for c in s5.all("Subsystem") {
            let name = c.req_attr("name")?;
            let id = if crate::brace::is_guid(name) { name.to_string() } else { x.id(name).ok_or(format!("строка {}: нет подсистемы {name}", c.line))?.to_string() };
            items.push(vec![a(id), visibility(x, c.req("Visibility")?)?]);
        }
    }
    part(&mut v, items);
    Ok(Rec::Raw(Doc::new(true, vec![l(v)]).write()?.into_bytes()))
}

/// `HomePageWorkArea.xml` → `{1,<шаблон>,N,<левая колонка или Column>…,M,<правая>…,2}`, элемент —
/// `{0,{0,<форма>},<высота>,<видимость>}`; последнее — MACommandInterfaceDisplays, без элемента None = 2 (выгрузка `ext::home_page`).
pub fn home_page(doc: &El, x: &super::index::Index) -> Result<Rec, String> {
    super::payload::check_ext_root(doc, "HomePageWorkArea")?;
    let t = super::tpl::token(doc.req("WorkingAreaTemplate")?, crate::xml::ext::WORKING_AREA_TEMPLATES)?;
    let mut v = vec![a("1"), t.clone()];
    let cols = if t.atom() == Some("0") { ["Column", ""] } else { ["LeftColumn", "RightColumn"] };
    for col in cols {
        let items: Vec<&El> = doc.child(col).map(|c| c.all("Item").collect()).unwrap_or_default();
        v.push(a(items.len().to_string()));
        for it in items {
            let form = super::tpl::reference(x, it.req("Form")?)?;
            let h = it.text_of("Height")?;
            let h = super::tpl::int(&h).ok_or(format!("строка {}: Height {h:?}", it.line))?;
            v.push(l(vec![a("0"), l(vec![a("0"), form]), a(h), visibility(x, it.req("Visibility")?)?]));
        }
    }
    v.push(a("2"));
    Ok(Rec::Raw(Doc::new(true, vec![l(v)]).write()?.into_bytes()))
}

const LAYOUT_GROUP: &str = "f1d2a809-46ac-46ab-8b37-b2135ac5b323";
const LAYOUT_PANEL: &str = "8b252cb5-d895-4c40-8ccb-581ce795fb80";

/// Элемент расположения: `<group id>` → (f1d2a809-…, `{0,<id>,0,{0,K,<дочерние>}}`), каждый дочерний — в своей
/// обертке `<group>`; `<panel id>` → (8b252cb5-…, `{0,<id>,<uuid>,2,<высота>}`).
fn layout_item(e: &El) -> Result<(Node, Node), String> {
    let id = e.req_attr("id")?;
    match e.name.as_str() {
        "group" => {
            let mut inner = vec![a("0"), a("0")];
            let mut k = 0;
            for w in e.elems() {
                let kids: Vec<&El> = w.elems().collect();
                if w.name != "group" || w.attrs.len() != 0 || kids.len() != 1 {
                    return Err(format!("строка {}: элемент группы не как у выгрузки", w.line));
                }
                let (t, b) = layout_item(kids[0])?;
                inner.push(t);
                inner.push(b);
                k += 1;
            }
            inner[1] = a(k.to_string());
            Ok((a(LAYOUT_GROUP), l(vec![a("0"), a(id), a("0"), l(inner)])))
        }
        "panel" => {
            let h = e.child("height").map(|h| h.text()).transpose()?.unwrap_or_else(|| "0".into());
            Ok((a(LAYOUT_PANEL), l(vec![a("0"), a(id), a(e.text_of("uuid")?), a("2"), a(super::tpl::int(&h).ok_or("height")?)])))
        }
        n => Err(format!("строка {}: <{n}> в расположении", e.line)),
    }
}

/// `ClientApplicationInterface.xml` → `{1,{0,1,<верх>},{0,2,<низ>},{0,3,<лево>},{0,4,<право>},2,{<панель>,0},…,0}`
/// (стороны 2 и 4 — как у выгрузки, по образцу не проверены).
pub fn client_interface(doc: &El) -> Result<Rec, String> {
    if doc.name != "ClientApplicationInterface" {
        return Err("корень не <ClientApplicationInterface>".into());
    }
    for (k, val) in [("xmlns", "http://v8.1c.ru/8.2/managed-application/core"), ("xmlns:xs", "http://www.w3.org/2001/XMLSchema"), ("xmlns:xsi", "http://www.w3.org/2001/XMLSchema-instance"), ("xsi:type", "InterfaceLayouter")] {
        if doc.attr(k) != Some(val) {
            return Err(format!("<ClientApplicationInterface>: {k} не как у выгрузки"));
        }
    }
    if doc.attrs.len() != 4 {
        return Err("<ClientApplicationInterface>: лишние атрибуты".into());
    }
    let mut v = vec![a("1")];
    for (i, side) in ["top", "bottom", "left", "right"].iter().enumerate() {
        // элемент стороны — свой `<side>` с одним элементом внутри (выгрузка mngcore FUN_180041c20)
        let mut body = vec![a("0"), a("0")];
        let mut k = 0;
        for s in doc.all(side) {
            let [it] = s.elems().collect::<Vec<_>>()[..] else { return Err(format!("строка {}: <{side}> не с одним элементом", s.line)) };
            let (t, b) = layout_item(it)?;
            body.push(t);
            body.push(b);
            k += 1;
        }
        body[1] = a(k.to_string());
        v.push(l(vec![a("0"), a((i + 1).to_string()), l(body)]));
    }
    // стандартные панели (mngcore 0x1809ceb00) — `2,{id,0}`; прочие — свои панели `1,{id,1,"имя"}` (чтение FUN_18059f000)
    const STD_PANELS: &[&str] = &[
        "b553047f-c9aa-4157-978d-448ecad24248",
        "13322b22-3960-4d68-93a6-fe2dd7f28ca3",
        "c933ac92-92cd-459d-81cc-e0c8a83ced99",
        "cbab57f2-a0f3-4f0a-89ea-4cb19570ab75",
        "b2735bd3-d822-4430-ba59-c9e869693b24",
    ];
    for p in doc.all("panelDef") {
        let id = p.req_attr("id")?;
        if STD_PANELS.contains(&id) {
            v.push(a("2"));
            v.push(l(vec![a(id), a("0")]));
        } else {
            v.push(a("1"));
            v.push(l(vec![a(id), a("1"), s(p.attr("name").unwrap_or(""))]));
        }
    }
    v.push(a("0"));
    Ok(Rec::Raw(Doc::new(true, vec![l(v)]).write()?.into_bytes()))
}

fn schedule_el(e: &El) -> Result<Node, String> {
    let at = |k: &str| e.req_attr(k).map(str::to_string);
    let date = |k: &str| -> Result<Node, String> {
        let v = at(k)?;
        let d: String = v.split('-').collect();
        if v.len() != 10 || d.len() != 8 || !d.bytes().all(|c| c.is_ascii_digit()) {
            return Err(format!("строка {}: {k}={v:?}", e.line));
        }
        Ok(a(format!("{d}000000")))
    };
    let time = |k: &str| -> Result<Node, String> {
        let v = at(k)?;
        let t: String = v.split(':').collect();
        if v.len() != 8 || t.len() != 6 || !t.bytes().all(|c| c.is_ascii_digit()) {
            return Err(format!("строка {}: {k}={v:?}", e.line));
        }
        Ok(a(format!("00010101{t}")))
    };
    let num = |k: &str| -> Result<Node, String> { at(k).and_then(|v| super::tpl::int(&v).map(a).ok_or(format!("строка {}: {k}={v:?}", e.line))) };
    let list = |tag: &str, v: &mut Vec<Node>| -> Result<(), String> {
        let t = e.text_of(&format!("ent:{tag}"))?;
        let items: Vec<&str> = t.split(' ').filter(|x| !x.is_empty()).collect();
        v.push(a(items.len().to_string()));
        for x in items {
            v.push(a(super::tpl::int(x).ok_or(format!("строка {}: {tag}: {x:?}", e.line))?));
        }
        Ok(())
    };
    let mut v = vec![date("BeginDate")?, date("EndDate")?, time("BeginTime")?, time("EndTime")?, time("CompletionTime")?];
    v.extend([num("CompletionInterval")?, num("RepeatPeriodInDay")?, num("RepeatPause")?]);
    list("WeekDays", &mut v)?;
    v.extend([num("WeekDayInMonth")?, num("DayInMonth")?]);
    list("Months", &mut v)?;
    v.extend([num("WeeksPeriod")?, num("DaysRepeatPeriod")?]);
    let det: Vec<&El> = e.all("ent:DetailedDailySchedules").collect();
    v.push(a(det.len().to_string()));
    for d in det {
        v.push(schedule_el(d)?);
    }
    Ok(l(v))
}

/// Состав автономной конфигурации (обратное `payload::standalone` выгрузки): `{2,N,uuid×N,M,uuid×M,K,(uuid,приоритет)×K,
/// план обмена,ExchangeOnChangeData,ExchangePeriod,TransactionCount,InactiveNodesCleanupTimeout,0}`; элементы — по
/// порядку хэш-множества GUID.
pub fn standalone(doc: &El, x: &super::index::Index) -> Result<Rec, String> {
    if doc.name != "StandaloneContent" || doc.attr("xmlns") != Some("http://v8.1c.ru/8.3/xcf/extrnprops") {
        return Err("корень не <StandaloneContent>".into());
    }
    for k in ["version", "xmlns:xs", "xmlns:xsi"] {
        doc.req_attr(k)?;
    }
    let (mut used, mut unused, mut prio) = (Vec::new(), Vec::new(), Vec::new());
    let mut ex = None;
    for e in doc.elems() {
        let id = |e: &El| -> Result<String, String> {
            let m = e.text_of("Metadata")?;
            x.id(&m).map(str::to_string).ok_or_else(|| format!("строка {}: нет объекта {m}", e.line))
        };
        match e.name.as_str() {
            "UsedItem" => used.push(id(e)?),
            "UnusedItem" => unused.push(id(e)?),
            "PriorityItem" => {
                let p = match e.text_of("Priority")?.as_str() {
                    "Default" => "0",
                    "MainServer" => "1",
                    "LocalServer" => "2",
                    o => return Err(format!("строка {}: приоритет {o}", e.line)),
                };
                prio.push((id(e)?, p));
            }
            "DataExchangeSettings" => {
                let b = match e.text_of("ExchangeOnChangeData")?.as_str() {
                    "true" => "1",
                    "false" => "0",
                    o => return Err(format!("строка {}: ExchangeOnChangeData {o}", e.line)),
                };
                ex = Some([b.to_string(), e.text_of("ExchangePeriod")?, e.text_of("TransactionCount")?, e.text_of("InactiveNodesCleanupTimeout")?]);
            }
            n => return Err(format!("строка {}: <{n}> в составе автономной конфигурации", e.line)),
        }
    }
    let ex = ex.ok_or("нет DataExchangeSettings")?;
    // порядок — обход хэш-таблицы платформы (MSVC) с хэшем Data1 GUID, вставка — по XML (как состав плана обмена)
    let h = |g: &String| super::hashorder::guid_hash(g);
    let used = super::hashorder::msvc_order(&used, h);
    let unused = super::hashorder::msvc_order(&unused, h);
    let prio = super::hashorder::msvc_order(&prio, |p: &(String, &str)| super::hashorder::guid_hash(&p.0));
    let mut v = vec![a("2"), a(used.len().to_string())];
    v.extend(used.into_iter().map(a));
    v.push(a(unused.len().to_string()));
    v.extend(unused.into_iter().map(a));
    v.push(a(prio.len().to_string()));
    for (id, p) in prio {
        v.extend([a(id), a(p)]);
    }
    v.push(a("00000000-0000-0000-0000-000000000000"));
    v.extend(ex.into_iter().map(a));
    v.push(a("0"));
    Ok(Rec::Raw(Doc::new(true, vec![l(v)]).write()?.into_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn style_order() {
        // корзины из 8: -1 → 7, -3 → 5, 415ee031·5 = 46da60f5 → 5 (перед -3), -20 → 4, -18 → 6
        let mut x = super::super::index::Index::default();
        x.full.insert("StyleItem.А".into(), "415ee031-6029-48b0-8d82-c1720f628932".into());
        let ns: String = STYLE_NS.iter().map(|(k, v)| format!(" {k}=\"{v}\"")).collect();
        let src = format!(
            "<Style{ns}><Item name=\"FormBackColor\"><Color>web:Cream</Color></Item><Item name=\"FormTextColor\"><Color>style:FormTextColor</Color></Item>\
             <Item name=\"StyleItem.А\"><Color>style:FormBackColor</Color></Item><Item name=\"TextFont\"><Font ref=\"style:TextFont\" kind=\"StyleItem\"/></Item>\
             <Item name=\"ControlBorder\"><Border ref=\"style:ControlBorder\"/></Item></Style>"
        );
        let d = super::super::dom::parse(&src).unwrap();
        let Rec::Raw(b) = style(&d, &x).unwrap() else { panic!() };
        assert!(d.unread().is_none());
        let t: String = String::from_utf8(b).unwrap().chars().filter(|c| !matches!(c, '\u{feff}' | '\r' | '\n')).collect();
        assert_eq!(
            t,
            "{2,5,{{-1},0,{4,2,{20},2}},{{0,415ee031-6029-48b0-8d82-c1720f628932},0,{4,3,{-1},3}},{{-3},0,{4,3,{-3},3}},\
             {{-20},1,{8,2,0,{-20},1,100}},{{-18},2,{3,1,{-18},0,0,0}},{0}}"
        );
    }

    #[test]
    fn client_interface_two_items_on_side() {
        // ЗУП: две группы сверху — два `<top>` в XML и обратно одна сторона из двух элементов
        let rec = "{1,{0,1,{0,2,f1d2a809-46ac-46ab-8b37-b2135ac5b323,{0,g1,0,{0,1,8b252cb5-d895-4c40-8ccb-581ce795fb80,{0,p1,u1,2,0}}},\
                   f1d2a809-46ac-46ab-8b37-b2135ac5b323,{0,g2,0,{0,1,8b252cb5-d895-4c40-8ccb-581ce795fb80,{0,p2,u2,2,1}}}}},\
                   {0,2,{0,0}},{0,3,{0,0}},{0,4,{0,0}},2,{b553047f-c9aa-4157-978d-448ecad24248,0},0}";
        let xml = crate::xml::ext::client_interface(&crate::brace::parse(rec).unwrap()).unwrap();
        let t = String::from_utf8(xml[3..].to_vec()).unwrap();
        assert_eq!(t.matches("<top>").count(), 2, "{t}");
        let d = super::super::dom::parse(&t).unwrap();
        let Rec::Raw(b) = client_interface(&d).unwrap() else { panic!() };
        assert!(d.unread().is_none());
        let got: String = String::from_utf8(b).unwrap().chars().filter(|c| !matches!(c, '\u{feff}' | '\r' | '\n')).collect();
        assert_eq!(got, rec);
    }
}
