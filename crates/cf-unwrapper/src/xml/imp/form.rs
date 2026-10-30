//! Управляемая форма `Ext/Form.xml` + `Ext/Form/Module.bsl` → запись `<uuid формы>.0` (обратное `xml/form.rs`,
//! `xml/form_v59.rs`). Загрузка платформы пишет все формы версией 59 (узлы новых версий с дописанными полями).
//!
//! Раскладки узлов и значения полей, которых нет в XML, сняты со сверки XML ↔ записи загрузки платформы
//! (БП 3.0.171, 7 573 формы): поле — постоянное или функция одного свойства XML. Незнакомое — отказ.

use super::dom::El;
use super::index::Index;
use super::tpl::{a, crlf, l, s};
use super::Rec;
use crate::brace::{Doc, Node};
use std::collections::HashMap;

mod cmdset;
pub(crate) mod dlist;
mod dlist_settings;
mod fields;
mod items;
mod path;
pub(crate) mod platform;
mod table;

const ELEM: &str = "02023637-7868-4a5f-8576-835a76e0c9ba";
/// Место словаря динамического списка в узле реквизита до разбора всех путей формы.
const LIST_PENDING: &str = "\u{1}list";
const FORM_CMD: &str = "409b9a53-7f7e-4178-86c1-33176c7c7a7a";
const ZERO: &str = "00000000-0000-0000-0000-000000000000";
const TYPE_DESC: &str = "f5c65050-3bbb-11d5-b988-0050bae0a95d";

type R<T> = Result<T, String>;

fn err(e: &El, m: &str) -> String {
    format!("строка {}: <{}>: {m}", e.line, e.name)
}

fn an(v: impl ToString) -> Node {
    a(v.to_string())
}

fn txt(e: &El) -> R<String> {
    e.text()
}

fn boolean(e: &El) -> R<bool> {
    match e.text()?.as_str() {
        "true" => Ok(true),
        "false" => Ok(false),
        t => Err(err(e, &format!("не булево {t:?}"))),
    }
}

fn num(e: &El) -> R<String> {
    let t = e.text()?;
    super::tpl::int(&t).ok_or_else(|| err(e, &format!("не число {t:?}")))
}

fn token(e: &El, m: &[(&str, i64)]) -> R<Node> {
    let t = e.text()?;
    m.iter().find(|(n, _)| *n == t).map(|(_, v)| an(v)).ok_or_else(|| err(e, &format!("значение {t:?} не встречено")))
}

/// Значение перечисления — его номер в списке `t`.
fn token_of(e: &El, t: &[&str]) -> R<Node> {
    let v = e.text()?;
    t.iter().position(|n| *n == v).map(an).ok_or_else(|| err(e, &format!("значение {v:?} не встречено")))
}

/// Многоязычная строка формы `{1,N,{"<язык>","<текст>"}…}`; LF текста → CRLF записи.
fn ml(e: Option<&El>) -> R<Node> {
    let mut v = vec![a("1"), a("0")];
    if let Some(e) = e {
        for it in e.elems() {
            if it.name != "v8:item" {
                return Err(err(it, "ожидался <v8:item>"));
            }
            v.push(l(vec![s(it.text_of("v8:lang")?), s(crlf(&it.text_of("v8:content")?))]));
        }
        if v.len() == 2 {
            e.text()?;
        }
    }
    v[1] = an(v.len() - 2);
    Ok(l(v))
}

/// Текст форматированной строки без разметки — как `FormattedString::getPureData` (core85 @1800a0260: разбор
/// `create_fmt_string` FUN_1800960e0, сборка FUN_18009d320): `<<` и `>>` — `<` и `>`, одиночный `>` — буквально;
/// `<` открывает тег до первого `>` (без `>` — хвост отбрасывается); тег в текст не идет, кроме `img` — пробел;
/// `<tab>`, `<lineseparator>` и незнакомые теги — ничего. Платформа разметку не отвергает — `None` не бывает.
pub(crate) fn formatted_plain(t: &str) -> Option<String> {
    let mut out = String::new();
    let mut rest = t;
    while let Some(c) = rest.chars().next() {
        if let Some(r) = rest.strip_prefix("<<") {
            out.push('<');
            rest = r;
        } else if let Some(r) = rest.strip_prefix(">>") {
            out.push('>');
            rest = r;
        } else if c == '<' {
            let Some(end) = rest.find('>') else { break };
            let tag = &rest[1..end];
            if tag.split(' ').next() == Some("img") {
                out.push(' ');
            }
            rest = &rest[end + 1..];
        } else {
            out.push(c);
            rest = &rest[c.len_utf8()..];
        }
    }
    Some(out)
}

/// Заголовок: (текст для показа, `{1,<исходный>,<форматированный>}`); у форматированного показ — без разметки.
fn title_pair(e: Option<&El>) -> R<(Node, Node)> {
    let formatted = match e.and_then(|t| t.attr("formatted")) {
        None | Some("false") => false,
        Some("true") => true,
        Some(o) => return Err(err(e.unwrap(), &format!("formatted={o}"))),
    };
    let src = ml(e)?;
    if !formatted {
        return Ok((src.clone(), l(vec![a("1"), src, a("0")])));
    }
    let e = e.unwrap();
    let mut v = vec![a("1"), a("0")];
    for it in e.elems() {
        let t = it.text_of("v8:content")?;
        let p = formatted_plain(&t).ok_or_else(|| err(it, "незнакомая разметка форматированной строки"))?;
        v.push(l(vec![s(it.text_of("v8:lang")?), s(crlf(&p))]));
    }
    v[1] = an(v.len() - 2);
    Ok((l(v), l(vec![a("1"), src, a("1")])))
}

/// Настраиваемое булево `{0,{0,{"B",<общее>},N,(<роль>,{"B",<значение>})…}}`; нет элемента — общее true.
fn adjustable(x: &Index, e: Option<&El>) -> R<Node> {
    let Some(e) = e else { return Ok(l(vec![a("0"), l(vec![a("0"), l(vec![s("B"), a("1")]), a("0")])])) };
    let common = boolean(e.req("xr:Common")?)?;
    let mut v = vec![a("0"), l(vec![s("B"), a(if common { "1" } else { "0" })]), a("0")];
    let mut n = 0;
    for c in e.elems() {
        match c.name.as_str() {
            "xr:Common" => {}
            "xr:Value" => {
                let r = c.req_attr("name")?;
                let g = x.id(r).map(str::to_string).or_else(|| crate::brace::is_guid(r).then(|| r.to_string())).ok_or_else(|| err(c, &format!("нет роли {r}")))?;
                v.extend([a(g), l(vec![s("B"), a(if boolean(c)? { "1" } else { "0" })])]);
                n += 1;
            }
            o => return Err(err(c, &format!("<{o}> в настраиваемом булевом"))),
        }
    }
    v[2] = an(n);
    Ok(l(vec![a("0"), l(v)]))
}

/// Описание типов формы: как у описаний объектов (`tpl::type_desc`), плюс типы платформы, которые встречаются только
/// в формах.
fn type_desc(x: &Index, e: &El) -> R<Node> {
    super::tpl::type_desc(x, e)
}

/// Функциональные опции `{0,N,<uuid>…}`.
fn functional_options(x: &Index, e: Option<&El>) -> R<Node> {
    let mut v = vec![a("0"), a("0")];
    if let Some(e) = e {
        for it in e.elems() {
            if it.name != "Item" {
                return Err(err(it, "ожидался <Item>"));
            }
            let t = it.text()?;
            // пустой `<Item/>` — нулевой GUID (обратное functional_options выгрузки)
            if t.is_empty() {
                v.push(a(crate::xml::common::ZERO));
                continue;
            }
            let g = x.id(&t).map(str::to_string).or_else(|| crate::brace::is_guid(&t).then(|| t.clone())).ok_or_else(|| err(it, &format!("нет функциональной опции {t}")))?;
            v.push(a(g));
        }
    }
    v[1] = an(v.len() - 2);
    Ok(l(v))
}

/// Событие формы → GUID (обратное `form_event` выгрузки); у записи — по основному реквизиту.
fn form_event(name: &str, document: bool) -> Option<&'static str> {
    Some(match name {
        "AfterWrite" => "047d4d09-961c-4bdc-8519-eef10674c35b",
        "AfterWriteAtServer" => "213d1900-dcad-4616-9f20-3f077156a40f",
        "BeforeClose" => "52dbb775-1631-4fd5-8c55-1615b5881dac",
        "BeforeLoadDataFromSettingsAtServer" => "e773807c-0c0c-4689-a093-231ddcd6409f",
        "BeforeLoadUserSettingsAtServer" => "40925042-2517-455b-a600-d68282829334",
        "BeforeLoadVariantAtServer" => "1dd89674-8b50-4240-9899-e3426b79cb02",
        "BeforeWrite" if document => "8a5894c9-d2ff-4c1d-b433-89cc352bbfbc",
        "BeforeWrite" => "9cc34712-da5f-4faa-a653-343d2085fbe8",
        "BeforeWriteAtServer" if document => "8f42e083-be92-4102-b1f0-fa58452c1a63",
        "BeforeWriteAtServer" => "bf0ac0e1-bcbb-4dfe-8fc4-0b1923b461a6",
        "ChoiceProcessing" => "1d632984-de3c-4b4b-ad9f-d69682a10182",
        "ExternalEvent" => "5426e344-5740-4f23-99c1-99179a200dc5",
        "FillCheckProcessingAtServer" => "e73d6384-49d2-4885-a752-a674d6ff7742",
        "NavigationProcessing" => "93dfba16-26db-46f8-acb5-4f92f50c855f",
        "NewWriteProcessing" => "3b644f4f-055f-4808-bdc6-a50ce895e4d9",
        "NotificationProcessing" => "3699f6a3-9a2a-4c82-a775-6ff4824a08ca",
        "OnClose" => "ca21cd18-35b2-4281-b5c8-016ecc8da8ac",
        "OnCreateAtServer" => "9f2e5ddb-3492-4f5d-8f0d-416b8d1d5c5b",
        "OnLoadDataFromSettingsAtServer" => "79cea13e-f6fb-4483-905d-713326405771",
        "OnLoadUserSettingsAtServer" => "7b15b3db-1cd0-4e1d-a74b-2c972c9e2226",
        "OnLoadVariantAtServer" => "87ce636e-9de6-4e42-9395-f0f189d08397",
        "OnMainServerAvailabilityChange" => "d6b86f20-722b-4fe6-83fa-85c6aa4c1fe5",
        "OnOpen" => "3ccc650e-f631-4cae-8e33-3eaac610b5f9",
        "OnReadAtServer" => "390d5e4b-e732-4c88-8748-9e211a416984",
        "OnReopen" => "6b3175a5-c143-4179-a670-ef231dc0a688",
        "OnSaveDataInSettingsAtServer" => "1952a54f-35ad-4928-902f-df212ab38ca3",
        "OnSaveUserSettingsAtServer" => "961ee7c6-0327-422b-adcb-97a90c46753d",
        "OnSaveVariantAtServer" => "499bb7af-6262-4de4-819f-ef264d1a20ec",
        "OnUpdateUserSettingSetAtServer" => "d817bccf-504e-4133-a79a-dd16e3a4df73",
        "OnWriteAtServer" => "c1bc0d3e-d35e-4207-a06b-ece68ed25314",
        "URLGetProcessing" => "674956b3-e469-4fdc-acf5-24ebf88cf7ab",
        "URLListGetProcessing" => "44498116-1641-4bfa-ae33-86e53c205797",
        "URLProcessing" => "e0cd9bdf-88fa-428c-9f1f-86f7f73b11e2",
        // прочие — по таблице событий DLL, если имени отвечает один GUID
        _ => {
            let mut it = crate::xml::events_table::EVENTS.iter().filter(|(_, n, _)| *n == name);
            return match (it.next(), it.next()) {
                (Some((g, ..)), None) => Some(*g),
                _ => None,
            };
        }
    })
}

/// События `{N,(<guid>,"<обработчик>")…,1,0,(<guid>,0,1)…}`; имя события → GUID по `id`.
fn events(e: Option<&El>, id: &dyn Fn(&str) -> Option<String>) -> R<Node> {
    events_where(e, &|_| true, id)
}

/// События, имена которых проходят `take` (остальные пишет другая часть узла).
fn events_where(e: Option<&El>, take: &dyn Fn(&str) -> bool, id: &dyn Fn(&str) -> Option<String>) -> R<Node> {
    let mut list: Vec<(String, String)> = Vec::new();
    if let Some(e) = e {
        for ev in e.elems() {
            if ev.name != "Event" {
                return Err(err(ev, "ожидалось <Event>"));
            }
            let n = ev.req_attr("name")?;
            if !take(n) {
                continue;
            }
            let g = id(n).ok_or_else(|| err(ev, &format!("событие {n} не загружается")))?;
            if ev.attrs.len() != 1 {
                return Err(err(ev, "атрибуты события"));
            }
            list.push((g, ev.text()?));
        }
    }
    // платформа хранит события по возрастанию GUID (опыт: OnOpen, добавленный в конец Events, встает перед OnCreateAtServer)
    list.sort_by(|x, y| x.0.cmp(&y.0));
    let mut v = vec![an(list.len())];
    for (g, h) in &list {
        v.extend([a(g.clone()), s(h.clone())]);
    }
    v.extend([a("1"), a("0")]);
    for (g, _) in &list {
        v.extend([a(g.clone()), a("0"), a("1")]);
    }
    Ok(l(v))
}

/// Реквизит формы: (id, имя, класс типа основного: `DocumentObject`, `DynamicList`…).
struct Attr {
    class: String,
    id: String,
    /// типы `<v8:Type>` (как в XML)
    types: Vec<String>,
    /// свои колонки `<Columns><Column>`
    cols: Vec<Col>,
    /// дополнительные колонки `<AdditionalColumns table="<путь>">`: (путь таблицы, колонка)
    extra: Vec<(String, Col)>,
    /// основная таблица динамического списка (`<Settings><MainTable>`, и при своем запросе)
    main_table: Option<String>,
}

/// Колонка реквизита формы.
struct Col {
    name: String,
    id: String,
    types: Vec<String>,
}

fn types_of(e: Option<&El>) -> Vec<String> {
    let of = |k: &str| -> Vec<String> { e.map(|t| t.peek_elems().filter(|c| c.name == k).filter_map(El::peek_text).collect()).unwrap_or_default() };
    // только набор типов (`v8:TypeSet`, определяемый тип) — он: путь продолжается типами определяемого типа (path.rs)
    let v = of("v8:Type");
    if v.is_empty() { of("v8:TypeSet") } else { v }
}

fn columns(e: &El) -> R<Vec<Col>> {
    let attr = |c: &El, k: &str| c.peek_attr(k).map(str::to_string).ok_or_else(|| err(c, &format!("нет атрибута {k}")));
    e.peek_elems().filter(|c| c.name == "Column").map(|c| Ok(Col { name: attr(c, "name")?, id: attr(c, "id")?, types: types_of(c.peek_child("Type")) })).collect()
}

/// Элемент формы для ссылок команд `Form.Item.<имя>.StandardCommand.…`: id, вид (тег), таблица динамического списка.
struct Item {
    id: String,
    tag: String,
    dynamic_list: bool,
    data_path: String,
}

struct F<'a> {
    x: &'a Index,
    attrs: HashMap<String, Attr>,
    main: Option<String>,
    /// команды формы: имя → id
    cmds: HashMap<String, String>,
    items: HashMap<String, Item>,
    /// элементы, на команды которых ссылаются кнопки формы (`Form.Item.<имя>.…`)
    referenced: std::collections::HashSet<String>,
    /// файл встроенной картинки `Form/Items/<элемент>/<файл>` по пути `<элемент>/<файл>`
    read: &'a dyn Fn(&str) -> Result<Vec<u8>, String>,
    /// прочитанные файлы картинок (`<элемент>/<файл>`)
    used: std::cell::RefCell<Vec<String>>,
    /// динамические списки реквизитов (основная таблица): поля и FieldsMap
    lists: std::cell::RefCell<HashMap<String, dlist::List>>,
    /// номера полей «использовать всегда» динамических списков
    list_req: std::cell::RefCell<HashMap<String, Vec<i64>>>,
}

impl F<'_> {
    /// Картинка свойства `k` элемента `item`: ссылка или встроенная (файл `Form/Items/<элемент>/<Abs>`).
    fn pic(&self, item: &El, p: &items::Props, k: &str) -> R<Node> {
        let Some(e) = p.get(k) else { return Ok(items::picture_none()) };
        let name = item.peek_attr("name").ok_or_else(|| err(item, "нет атрибута name"))?;
        let (n, f) = super::tpl::picture_or_file(self.x, e, &|f| (self.read)(&format!("{name}/{f}")))?;
        if let Some(f) = f {
            self.used.borrow_mut().push(format!("{name}/{f}"));
        }
        Ok(n)
    }
}

/// Имена элементов из команд кнопок `Form.Item.<имя>.…` по всей форме.
fn referenced_items(doc: &El) -> std::collections::HashSet<String> {
    fn walk(e: &El, out: &mut std::collections::HashSet<String>) {
        for c in e.peek_elems() {
            if c.name == "CommandName" {
                if let Some(n) = c.peek_text().as_deref().and_then(|t| t.strip_prefix("Form.Item.")).and_then(|t| t.split_once('.')).map(|(n, _)| n.to_string()) {
                    out.insert(n);
                }
            }
            walk(c, out);
        }
    }
    let mut out = std::collections::HashSet::new();
    walk(doc, &mut out);
    out
}

/// Элементы с `id` и `name` на любой глубине (дочерние, командные панели, контекстные меню, подсказки).
fn collect_items(e: &El, attrs: &HashMap<String, Attr>, out: &mut HashMap<String, Item>) {
    for c in e.peek_elems() {
        if let (Some(id), Some(name)) = (c.peek_attr("id"), c.peek_attr("name")) {
            let data_path = c.peek_child("DataPath").and_then(El::peek_text).unwrap_or_default();
            let root = data_path.split('.').next().unwrap_or_default();
            let dynamic_list = c.name == "Table" && attrs.get(root).is_some_and(|a| a.class == "DynamicList");
            out.insert(name.to_string(), Item { id: id.to_string(), tag: c.name.clone(), dynamic_list, data_path });
        }
        collect_items(c, attrs, out);
    }
}

impl F<'_> {
    fn document(&self) -> bool {
        self.main.as_deref().and_then(|m| self.attrs.get(m)).is_some_and(|a| a.class == "DocumentObject")
    }

    /// Автоматическая командная панель (группа вида 9, версия 22).
    fn auto_command_bar(&self, e: &El) -> R<Node> {
        let id = e.req_attr("id")?;
        let name = e.req_attr("name")?;
        let mut halign = an(0);
        let mut autofill = true;
        let mut children = vec![a("0")];
        for c in e.elems() {
            match c.name.as_str() {
                "HorizontalAlign" => halign = token(c, platform::H_ALIGN)?,
                "Autofill" => autofill = boolean(c)?,
                "ChildItems" => children = self.children(Some(c))?,
                n => return Err(err(c, &format!("{n} у командной панели формы не загружается"))),
            }
        }
        let af = if autofill { "1" } else { "0" };
        // хвост группы: видимость, подсказка, …, выравнивание в группе, важность (выгрузка: поле k + 4, как у групп)
        let mut tail = vec![a("1"), a("0"), a("0"), a("0"), a("3"), a("3"), items::importance(e)?];
        children.append(&mut tail);
        let mut v = vec![
            a("22"),
            l(vec![a(id), a(ELEM)]),
            a("0"),
            a("0"),
            a("0"),
            a("9"),
            s(name),
            l(vec![a("1"), a("0")]),
            l(vec![a("1"), a("0")]),
            a("0"),
            a("1"),
            a("0"),
            a("0"),
            a("0"),
            a("2"),
            a("2"),
            l(vec![a("4"), a("4"), l(vec![a("0")]), a("4")]),
            l(vec![a("8"), a("3"), a("0"), a("1"), a("100")]),
            l(vec![a("0"), a("0"), a("0")]),
            a("1"),
            l(vec![a("1"), halign, a(af), a("0")]),
        ];
        v.append(&mut children);
        Ok(l(v))
    }

    /// Командный интерфейс формы, панель `panel` (NavigationPanel, CommandBar): `{0,N,<элемент>…}`, элемент
    /// `{3,<номер по порядку>,<команда>,<параметр — путь к данным>,<тип: 0 Auto, 1 Added>,{0[,<группа>]},<Index>,
    /// <видимость по умолчанию>,<видимость>}`. Номер — порядковый в панели (сверка БП: 833 + 833 панели).
    fn command_interface(&self, e: Option<&El>, panel: &str) -> R<Node> {
        let mut v = vec![a("0"), a("0")];
        let Some(e) = e else { return Ok(l(v)) };
        for c in e.elems() {
            if !matches!(c.name.as_str(), "NavigationPanel" | "CommandBar") {
                return Err(err(c, "в командном интерфейсе формы"));
            }
        }
        let Some(p) = e.child(panel) else { return Ok(l(v)) };
        for (i, it) in p.elems().enumerate() {
            if it.name != "Item" {
                return Err(err(it, "ожидался <Item>"));
            }
            let q = items::Props::new(it)?;
            let ce = q.get("Command").ok_or_else(|| err(it, "нет <Command>"))?;
            // команда одним числом `{N}` (выгрузка пишет ее как есть)
            let cmd = match ce.peek_text().as_deref().and_then(super::tpl::int) {
                Some(n) => {
                    ce.text()?;
                    l(vec![a(n)])
                }
                None => self.command_ref(ce)?,
            };
            let ty = token(q.get("Type").ok_or_else(|| err(it, "нет <Type>"))?, &[("Auto", 0), ("Added", 1)])?;
            let path = match q.get("Attribute") {
                Some(d) => self.data_path(d)?,
                None => l(vec![a("0")]),
            };
            let group = match q.get("CommandGroup") {
                None => l(vec![a("0")]),
                Some(g) => {
                    let t = g.text()?;
                    let id = match crate::xml::classes::STD_COMMAND_GROUPS.iter().find(|(_, n)| *n == t) {
                        Some((id, _)) => id.to_string(),
                        None if crate::brace::is_guid(&t) => t.clone(),
                        None => self.x.id(&t).ok_or_else(|| err(g, &format!("нет группы {t}")))?.to_string(),
                    };
                    l(vec![a("0"), a(id)])
                }
            };
            let default_visible = q.b("DefaultVisible", "1")?;
            if default_visible.atom() == Some("1") && q.get("Visible").is_some() {
                return Err(err(it, "Visible без DefaultVisible=false: не встречено"));
            }
            v.push(l(vec![a("3"), an(i), cmd, path, ty, group, q.num("Index", "0")?, default_visible, adjustable(self.x, q.get("Visible"))?]));
        }
        v[1] = an(v.len() - 2);
        Ok(l(v))
    }

    /// Состав командной панели мобильного устройства `{59,N,("",{"N",<id элемента>})…}` (сверка БП: 65 форм, у всех
    /// представление пустое и пометка 0).
    fn mobile_bar(&self, e: Option<&El>) -> R<Node> {
        let mut v = vec![a("59"), a("0")];
        if let Some(e) = e {
            for it in e.elems() {
                if it.name != "xr:Item" {
                    return Err(err(it, "ожидался <xr:Item>"));
                }
                let mut name = None;
                for c in it.elems() {
                    match c.name.as_str() {
                        "xr:Presentation" if c.text()?.is_empty() => {}
                        "xr:CheckState" if c.text()? == "0" => {}
                        "xr:Value" if c.attr("xsi:type") == Some("xs:string") => name = Some(c.text()?),
                        _ => return Err(err(c, "в составе командной панели мобильного устройства не встречено")),
                    }
                }
                let n = name.ok_or_else(|| err(it, "нет <xr:Value>"))?;
                // пустое имя — `{"N",0}` (выгрузка form.rs: 0 — пусто)
                // `<id>:02023637-…` — номер элемента, которого в форме нет (обратно выгрузке)
                let gone = n.split_once(':').filter(|(i, g)| *g == ELEM && crate::xml::imp::tpl::int(i).is_some()).map(|(i, _)| i.to_string());
                let id = match gone {
                    _ if n.is_empty() => "0".to_string(),
                    Some(i) => i,
                    None => self.items.get(&n).ok_or_else(|| err(it, &format!("нет элемента {n}")))?.id.clone(),
                };
                v.extend([s(""), l(vec![s("N"), a(id)])]);
            }
            v[1] = an((v.len() - 2) / 2);
        }
        Ok(l(v))
    }

    /// Реквизиты `{4,N,<реквизит>…,0,0,{#base64:<условное оформление>}}`.
    fn attributes(&self, e: Option<&El>) -> R<Node> {
        let mut v = vec![a("4"), a("0")];
        let mut n = 0;
        let mut ca = None;
        let mut pending = Vec::new();
        if let Some(e) = e {
            for at in e.elems() {
                match at.name.as_str() {
                    "Attribute" => {
                        if let Some(st) = at.peek_child("Settings").filter(|s| s.peek_attr("xsi:type") == Some("DynamicList")) {
                            pending.push((v.len(), at.peek_attr("name").unwrap_or_default().to_string(), st));
                        }
                        v.push(self.attribute(at)?);
                        n += 1;
                    }
                    "ConditionalAppearance" if ca.is_none() => ca = Some(at),
                    o => return Err(err(at, &format!("<{o}> в реквизитах"))),
                }
            }
            // FormAttributes (модель XDTO logform): Attribute и ConditionalAppearance; пустой — текст
            if n == 0 && ca.is_none() {
                e.text()?;
            }
        }
        v[1] = an(n);
        // дополнительные колонки `{0,<путь таблицы>,N,<колонка>…}` по всем реквизитам
        let mut extra = Vec::new();
        for at in e.into_iter().flat_map(|e| e.all("Attribute")) {
            for ac in at.child("Columns").into_iter().flat_map(|c| c.all("AdditionalColumns")) {
                let t = ac.req_attr("table")?;
                let mut x = vec![a("0"), self.path_text(t).map_err(|m| err(ac, &m))?, a("0")];
                let mut k = 0;
                for c in ac.elems() {
                    if c.name != "Column" {
                        return Err(err(c, "ожидалась <Column>"));
                    }
                    x.push(self.column(c)?);
                    k += 1;
                }
                if k == 0 {
                    ac.text()?;
                }
                x[2] = an(k);
                extra.push(l(x));
            }
        }
        v.push(an(extra.len()));
        v.extend(extra);
        match ca {
            None => {
                v.push(a("0"));
                v.push(l(vec![super::payload::base64_atom(EMPTY_SETTINGS.as_bytes())]));
            }
            Some(ca) => {
                let (fields, text) = self.conditional_appearance(ca)?;
                v.push(an(fields.len()));
                for f in fields {
                    v.extend(f);
                }
                v.push(l(vec![super::payload::base64_atom(text.as_bytes())]));
            }
        }
        // словари динамических списков: FieldsMap собран всеми путями формы
        for (i, name, st) in pending {
            let d = self.list_settings(&name, st)?;
            let at = v[i].list_mut().ok_or("реквизит")?;
            let k = at.len() - 2;
            at[k] = d;
        }
        Ok(l(v))
    }

    /// Условное оформление формы: поля (четверки `"<поле>","<поле>"|"",<путь>,<путь коллекции>|{0}` в порядке первого
    /// упоминания полем `dcscor:Field`; поле реквизита, которого у формы нет, — `"",{0},{0}`; сверка БП: 690 полей) и
    /// часть настроек компоновки (сериализатор платформы, как у настроек варианта схемы).
    fn conditional_appearance(&self, ca: &El) -> R<(Vec<[Node; 4]>, String)> {
        fn fields(e: &El, out: &mut Vec<String>) {
            for c in e.peek_elems() {
                if c.peek_attr("xsi:type") == Some("dcscor:Field") {
                    if let Some(t) = c.peek_text().filter(|t| !out.contains(t)) {
                        out.push(t);
                    }
                }
                fields(c, out);
            }
        }
        let mut names = Vec::new();
        fields(ca, &mut names);
        let mut quads = Vec::new();
        for n in names {
            // Поле, которого нет (ни реквизита формы, ни поля в метаданных): разбор пути (mngbase FUN_18075f8c0) дает 1 —
            // ошибка, вызывающие (FUN_180760600, FUN_1807650d0, FUN_180765690) пишут сообщение и оставляют путь пустым.
            let root = n.split('.').next().unwrap_or_default();
            let path = match self.path_text(&n) {
                _ if !self.attrs.contains_key(root) => None,
                Err(m) if m.ends_with("не найдено") => None,
                r => Some(r.map_err(|m| err(ca, &m))?),
            };
            let Some(path) = path else {
                quads.push([s(n), s(""), l(vec![a("0")]), l(vec![a("0")])]);
                continue;
            };
            let coll = match n.rsplit_once('.') {
                // количество строк и итог — поле самой коллекции, не строки: путь коллекции {0} (эталон и Клиент ЭДО
                // ЭлектронныйДокументИсходящийЭДО.ФормаПросмотра `Объект.СписокПодписантов.КоличествоСтрок`).
                // shortcut: по двум образцам, место решения в платформе не найдено — при расхождении искать в mngbase.
                Some((parent, _)) if self.is_collection(parent) && path.list().last().and_then(|p| p.get(0)).and_then(Node::atom).is_some_and(|c| c == "100000000" || c == "101000000") => l(vec![a("0")]),
                Some((parent, _)) if self.is_collection(parent) => self.path_text(parent).map_err(|m| err(ca, &m))?,
                _ => l(vec![a("0")]),
            };
            // второе имя — каноническое: русское имя стандартного реквизита (компоновка) — английским, если путь тот же
            let en: Vec<&str> = n.split('.').map(|p| match p {
                "КоличествоСтрок" => "RowsCount",
                "СчетДт" => "AccountDr",
                "СчетКт" => "AccountCr",
                p => dlist::std_en(p).unwrap_or(p),
            }).collect();
            let en = en.join(".");
            // проба английского пути не должна менять FieldsMap и номера (разрешение пути пишет их)
            let same = en != n && {
                let (lists, req) = (self.lists.borrow().clone(), self.list_req.borrow().clone());
                let same = self.path_text(&en).ok().as_ref() == Some(&path);
                (*self.lists.borrow_mut(), *self.list_req.borrow_mut()) = (lists, req);
                same
            };
            let canon = if same { en } else { n.clone() };
            quads.push([s(n), s(canon), path, coll]);
        }
        let mut src = String::from("<dcsset:settings");
        for (k, v) in ROOT_NS.iter().filter(|(k, _)| k.starts_with("xmlns")) {
            src += &format!(" {k}=\"{v}\"");
        }
        src += "><dcsset:conditionalAppearance>";
        for c in ca.elems() {
            xml_text(c, &mut src);
        }
        src += "</dcsset:conditionalAppearance></dcsset:settings>";
        ca.touch_all();
        Ok((quads, super::dcs::settings_part(&src, self.x).map_err(|m| err(ca, &m))?))
    }

    /// Поля реквизита `{0,N,<путь от реквизита>…}` (сохраняемые, «использовать всегда»): путь `{K,<часть>…}` без самого
    /// реквизита.
    fn save_fields(&self, attr: &str, e: Option<&El>) -> R<Node> {
        let mut v = vec![a("0"), a("0")];
        // Прочие поля настроек компоновщика `{2,{0},{код}}` выгрузка пишет только все 11 разом, по алфавиту имен (какой код
        // какому имени, не установлено) — обратное: все 11 имен `<реквизит>.Settings.…` → все 11 кодов.
        const REST: [&str; 11] = ["0", "5", "6", "10000", "10001", "10002", "10003", "10004", "10005", "10006", "10015"];
        const REST_NAMES: [&str; 11] = [
            "DataParameters", "HasConditionalAppearance", "HasFilter", "HasOrder", "HasOutputParameters", "HasSelection",
            "OutputParameters", "ReportStructure", "ReportStructurePicture", "Use", "UserFields",
        ];
        let rest: Vec<String> = REST_NAMES.iter().map(|n| format!("{attr}.Settings.{n}")).collect();
        let texts: Vec<String> = e.map(|e| e.peek_elems().filter_map(|f| f.peek_text()).collect()).unwrap_or_default();
        let all_rest = rest.iter().all(|r| texts.contains(r));
        if all_rest {
            v.extend(REST.iter().map(|c| l(vec![a("2"), l(vec![a("0")]), l(vec![a(*c)])])));
        }
        if let Some(e) = e {
            for f in e.elems() {
                if f.name != "Field" {
                    return Err(err(f, "ожидалось <Field>"));
                }
                let t = f.text()?;
                if all_rest && rest.contains(&t) {
                    continue;
                }
                // неразрешенный путь (`1/0:<uuid>`) — от id реквизита
                let id = self.attrs.get(attr).map(|a| a.id.as_str()).unwrap_or_default();
                let raw = t.contains('/') && t.split('/').next() == Some(id);
                if t != attr && !t.starts_with(&format!("{attr}.")) && !t.starts_with(&format!("{attr}[")) && !raw {
                    return Err(err(f, "сохраняемое поле не этого реквизита"));
                }
                let full = self.path_text(&t).map_err(|m| err(f, &m))?;
                let mut parts = full.list().get(2..).unwrap_or(&[]).to_vec();
                parts.insert(0, an(parts.len()));
                v.push(l(parts));
            }
        }
        // платформа пишет поля упорядоченными: по частям, числа — как числа ({2,…} раньше {11,…}), прочее — как текст
        v[2..].sort_by(path_cmp);
        v[1] = an(v.len() - 2);
        Ok(l(v))
    }

    /// Колонка реквизита `{5,<id>,0,"<имя>",<заголовок>,<тип>,<просмотр>,<редактирование>,<ФО>,<проверка заполнения>}`.
    fn column(&self, c: &El) -> R<Node> {
        let mut p: HashMap<&str, &El> = HashMap::new();
        for k in c.elems() {
            if p.insert(k.name.as_str(), k).is_some() {
                return Err(err(k, "повтор"));
            }
            if !matches!(k.name.as_str(), "Title" | "Type" | "View" | "Edit" | "FillCheck" | "FunctionalOptions") {
                return Err(err(k, &format!("{} у колонки пока не загружается", k.name)));
            }
        }
        let fill = match p.get("FillCheck") {
            Some(e) if e.text()? == "ShowError" => "1",
            Some(e) => return Err(err(e, "FillCheck не ShowError")),
            None => "0",
        };
        Ok(l(vec![
            a("5"),
            a(c.req_attr("id")?),
            a("0"),
            s(c.req_attr("name")?),
            ml(p.get("Title").copied())?,
            type_desc(self.x, p.get("Type").ok_or_else(|| err(c, "нет <Type>"))?)?,
            adjustable(self.x, p.get("View").copied())?,
            adjustable(self.x, p.get("Edit").copied())?,
            functional_options(self.x, p.get("FunctionalOptions").copied())?,
            a(fill),
        ]))
    }

    /// `{9,{<id>},0,"<имя>",<заголовок>,<тип>,<просмотр>,<редактирование>,{0,0},{0,0},<основной>,<сохраняемые данные>,
    /// <проверка заполнения>,<колонок>,<колонки>…,<настройки>,<ФО>}`.
    fn attribute(&self, at: &El) -> R<Node> {
        let id = at.req_attr("id")?;
        let name = at.req_attr("name")?;
        let mut p: HashMap<&str, &El> = HashMap::new();
        for c in at.elems() {
            if p.insert(c.name.as_str(), c).is_some() {
                return Err(err(c, "повтор"));
            }
        }
        for k in p.keys() {
            if !matches!(*k, "Title" | "Type" | "View" | "Edit" | "MainAttribute" | "SavedData" | "FillCheck" | "Settings" | "FunctionalOptions" | "Columns" | "Save" | "UseAlways") {
                return Err(err(p[k], &format!("{k} у реквизита пока не загружается")));
            }
        }
        let b = |k: &str| -> R<&str> { Ok(if p.get(k).map(|e| boolean(e)).transpose()? == Some(true) { "1" } else { "0" }) };
        let fill = match p.get("FillCheck") {
            Some(e) if e.text()? == "ShowError" => "1",
            Some(e) => return Err(err(e, "FillCheck не ShowError")),
            None => "0",
        };
        let ty = type_desc(self.x, p.get("Type").ok_or_else(|| err(at, "нет <Type>"))?)?;
        let settings = match p.get("Settings") {
            None => l(vec![a("0"), a("0")]),
            // словарь — после всех путей формы (`attributes`)
            Some(st) if st.attr("xsi:type") == Some("DynamicList") => a(LIST_PENDING),
            Some(st) if st.attr("xsi:type") == Some("v8:TypeDescription") => {
                let t = type_desc(self.x, st)?;
                l(vec![a("0"), a("1"), s("ElementType"), l(vec![s("#"), a(TYPE_DESC), t])])
            }
            Some(st) if st.attr("xsi:type") == Some("mxl:SpreadsheetDocument") && st.attr("xmlns:mxl") == Some("http://v8.1c.ru/8.2/data/spreadsheet") => {
                let mut inner = String::new();
                for c in st.elems() {
                    xml_text(c, &mut inner);
                }
                st.touch_all();
                let inner = inner.replace("<mxl:", "<").replace("</mxl:", "</").replace("\"mxl:", "\"");
                let doc = super::mxl::form_settings(&inner, self.x).map_err(|m| err(st, &m))?;
                l(vec![a("0"), a("1"), s("Moxel"), l(vec![s("#"), a("e603103e-a318-4edc-a014-b1c6cf94d49f"), doc])])
            }
            // диаграмма — как макет (обратно `form_settings::chart` выгрузки)
            Some(st) if st.attr("xsi:type").is_some_and(|t| t.ends_with(":Chart")) => {
                // 8 цветов версии 75 — копии классических (imp/chart.rs `body`)
                let c = super::chart::form_chart(self.x, st)?;
                l(vec![a("0"), a("1"), s("Chart"), l(vec![s("#"), a("3543ef08-3316-4f7e-9447-0cd0a1cbf1d5"), l(vec![a("11")]), c])])
            }
            // диаграмма Ганта — как в макете (обратно `form_settings::gantt` выгрузки)
            Some(st) if st.attr("xsi:type").is_some_and(|t| t.ends_with(":GanttChart")) => {
                let c = super::chart::gantt(self.x, st)?;
                l(vec![a("0"), a("1"), s("GanttChart"), l(vec![s("#"), a("3a6e63bf-16aa-42eb-b48c-2fff9670ad2f"), c])])
            }
            Some(st) if st.attr("xsi:type") == Some("pl:Planner") && st.attr("xmlns:pl") == Some("http://v8.1c.ru/8.3/data/planner") => {
                l(vec![a("0"), a("1"), s("Planner"), l(vec![s("#"), a("43dc7f37-5b1d-42a7-8f28-f545080d0255"), planner(st)?])])
            }
            Some(st) => return Err(err(st, "настройки реквизита этого вида пока не загружаются")),
        };
        let mut cols = Vec::new();
        if let Some(cs) = p.get("Columns") {
            for c in cs.elems() {
                match c.name.as_str() {
                    "Column" => cols.push(self.column(c)?),
                    "AdditionalColumns" => {}
                    o => return Err(err(c, &format!("<{o}> в колонках"))),
                }
            }
        }
        let mut v = vec![
            a("9"),
            l(vec![a(id)]),
            a("0"),
            s(name),
            ml(p.get("Title").copied())?,
            ty,
            adjustable(self.x, p.get("View").copied())?,
            adjustable(self.x, p.get("Edit").copied())?,
            match p.get("UseAlways") {
                Some(u) if self.attrs.get(name).is_some_and(|a| a.class == "DynamicList") => {
                    self.list_use_always(name, u)?;
                    l(vec![a("0"), a("0")])
                }
                u => self.save_fields(name, u.copied())?,
            },
            self.save_fields(name, p.get("Save").copied())?,
            a(b("MainAttribute")?),
            a(b("SavedData")?),
            a(fill),
            an(cols.len()),
        ];
        v.extend(cols);
        v.extend([settings, functional_options(self.x, p.get("FunctionalOptions").copied())?]);
        Ok(l(v))
    }

    /// Параметры `{0,N,{0,"<имя>",<тип>,<ключевой>}…}`.
    fn parameters(&self, e: Option<&El>) -> R<Node> {
        let mut v = vec![a("0"), a("0")];
        if let Some(e) = e {
            for pr in e.elems() {
                if pr.name != "Parameter" {
                    return Err(err(pr, "ожидался <Parameter>"));
                }
                let mut ty = None;
                let mut key = "0";
                for c in pr.elems() {
                    match c.name.as_str() {
                        "Type" => ty = Some(type_desc(self.x, c)?),
                        "KeyParameter" => key = if boolean(c)? { "1" } else { "0" },
                        o => return Err(err(c, &format!("{o} у параметра"))),
                    }
                }
                v.push(l(vec![a("0"), s(pr.req_attr("name")?), ty.ok_or_else(|| err(pr, "нет <Type>"))?, a(key)]));
            }
        }
        v[1] = an(v.len() - 2);
        Ok(l(v))
    }

    /// Команды `{0,N,{11,{<id>,409b9a53…},"<имя>",<заголовок>,<подсказка>,<использование>,<сочетание>,<картинка>,
    /// "<действие>",<отображение>,<изменяет данные>,0,<ФО>,<Use: 0, иначе 1>,0,1,0,0,<использование текущей строки>,0,0}…}`.
    fn commands(&self, e: Option<&El>) -> R<Node> {
        let mut v = vec![a("0"), a("0")];
        if let Some(e) = e {
            for c in e.elems() {
                if c.name != "Command" {
                    return Err(err(c, "ожидалась <Command>"));
                }
                let mut p: HashMap<&str, &El> = HashMap::new();
                for k in c.elems() {
                    if p.insert(k.name.as_str(), k).is_some() {
                        return Err(err(k, "повтор"));
                    }
                }
                for k in p.keys() {
                    if !matches!(*k, "Title" | "ToolTip" | "Use" | "Action" | "FunctionalOptions" | "Representation" | "ModifiesSavedData" | "CurrentRowUse" | "Picture" | "Shortcut" | "AssociatedTableElementId") {
                        return Err(err(p[k], &format!("{k} у команды пока не загружается")));
                    }
                }
                let repr = match p.get("Representation") {
                    Some(r) => token(r, &[("Text", 0), ("Picture", 1), ("TextPicture", 2)])?,
                    None => an(3),
                };
                let modifies = match p.get("ModifiesSavedData") {
                    Some(m) if boolean(m)? => "1",
                    Some(m) => return Err(err(m, "false не встречено")),
                    None => "0",
                };
                let cur = match p.get("CurrentRowUse") {
                    Some(r) => token(r, &[("Use", 0), ("DontUse", 1)])?,
                    None => an(2),
                };
                v.push(l(vec![
                    a("11"),
                    l(vec![a(c.req_attr("id")?), a(FORM_CMD)]),
                    s(c.req_attr("name")?),
                    ml(p.get("Title").copied())?,
                    ml(p.get("ToolTip").copied())?,
                    adjustable(self.x, p.get("Use").copied())?,
                    items::shortcut(p.get("Shortcut").copied())?,
                    match p.get("Picture") {
                        Some(pc) => super::tpl::picture(self.x, pc)?,
                        None => items::picture_none(),
                    },
                    s(p.get("Action").map(|a| a.text()).transpose()?.unwrap_or_default()),
                    repr,
                    a(modifies),
                    a("0"),
                    functional_options(self.x, p.get("FunctionalOptions").copied())?,
                    // старое поле использования текущей строки: 0 — Use, иначе 1 (8.5: Use — 1 из 1)
                    a(if cur.atom() == Some("0") { "0" } else { "1" }),
                    match p.get("AssociatedTableElementId") {
                        None => a("0"),
                        Some(t) => {
                            if t.attr("xsi:type") != Some("xs:string") {
                                return Err(err(t, "AssociatedTableElementId не строкой"));
                            }
                            // имя элемента; нет элемента — `<id>:02023637…` (обратное выгрузке)
                            item_id(self, t)?
                        }
                    },
                    a("1"),
                    a("0"),
                    a("0"),
                    cur,
                    a("0"),
                    a("0"),
                ]));
            }
        }
        v[1] = an(v.len() - 2);
        Ok(l(v))
    }
}

/// Элемент Form.xml обратно в текст XML (для сериализатора настроек компоновки): текст и значения атрибутов
/// экранируются так, как пишет XDTO.
/// Планировщик в настройках реквизита (обратно `form_settings::planner` выгрузки). shortcut: образец один
/// (КонтактныеЛица.ФормаЛиды БП, все значения по умолчанию) — запись образца в виде 8.5 (цвета `{4,4,{0},4}`), если XML
/// тот же; иначе отказ, пока не будет образцов с другими значениями.
fn planner(st: &El) -> R<Node> {
    const SAMPLE: &str = r##"{8,0,0,{4,4,{0},4},{4,4,{0},4},{4,4,{0},4},{4,4,{0},4},{8,3,0,1,100},00010101000000,00010101000000,1,0,0,{1,2,{"#","DLF=""DD"""},{"ru","DLF=""DD"""}},30,0,0,0,{3,2,1,{8,5,1,1,{4,0,{0},0,1,0,e5cabe59-d992-4d31-8086-3116931aff81,0},{4,4,{0},4},1,{1,1,{"ru","ДФ="}},{0,{1,0,0}},{4,4,{0},4},{4,4,{0},4},1},0,{4,4,{0},4},{4,4,{0},4},0},0,0,0,0,0,1,1,0,0,2,2,{3,0,{0},1,0,0,48312c09-257f-4b29-b280-284dd89efc1e},0}"##;
    // XML образца без отступов (выгрузка БП)
    const XML: &str = r##"<pl:borderColor>auto</pl:borderColor><pl:backColor>auto</pl:backColor><pl:textColor>auto</pl:textColor><pl:lineColor>auto</pl:lineColor><pl:font kind="AutoFont"/><pl:beginOfRepresentationPeriod>0001-01-01T00:00:00</pl:beginOfRepresentationPeriod><pl:endOfRepresentationPeriod>0001-01-01T00:00:00</pl:endOfRepresentationPeriod><pl:alignElementsOfTimeScale>true</pl:alignElementsOfTimeScale><pl:displayTimeScaleWrapHeaders>false</pl:displayTimeScaleWrapHeaders><pl:displayWrapHeaders>false</pl:displayWrapHeaders><pl:timeScaleWrapHeadersFormat><v8:item><v8:lang>#</v8:lang><v8:content>DLF="DD"</v8:content></v8:item><v8:item><v8:lang>ru</v8:lang><v8:content>DLF="DD"</v8:content></v8:item></pl:timeScaleWrapHeadersFormat><pl:periodicVariantUnit>Day</pl:periodicVariantUnit><pl:periodicVariantRepetition>0</pl:periodicVariantRepetition><pl:timeScaleWrapBeginIndent>0</pl:timeScaleWrapBeginIndent><pl:timeScaleWrapEndIndent>0</pl:timeScaleWrapEndIndent><pl:timeScale><placement xmlns="http://v8.1c.ru/8.2/data/chart">Left</placement><level xmlns="http://v8.1c.ru/8.2/data/chart"><measure>Second</measure><interval>1</interval><show>true</show><line width="1" gap="false"><v8ui:style xsi:type="v8ui:ChartLineType">None</v8ui:style></line><scaleColor>auto</scaleColor><dayFormatRule>MonthDay</dayFormatRule><format><v8:item><v8:lang>ru</v8:lang><v8:content>ДФ=</v8:content></v8:item></format><labels><ticks>0</ticks></labels><backColor>auto</backColor><textColor>auto</textColor><showPereodicalLabels>true</showPereodicalLabels></level><transparent xmlns="http://v8.1c.ru/8.2/data/chart">false</transparent><backColor xmlns="http://v8.1c.ru/8.2/data/chart">auto</backColor><textColor xmlns="http://v8.1c.ru/8.2/data/chart">auto</textColor><currentLevel xmlns="http://v8.1c.ru/8.2/data/chart">0</currentLevel></pl:timeScale><pl:displayCurrentDate>false</pl:displayCurrentDate><pl:itemsTimeRepresentation>DontDisplay</pl:itemsTimeRepresentation><pl:itemsBehaviorWhenSpaceInsufficient>CollapseItems</pl:itemsBehaviorWhenSpaceInsufficient><pl:autoMinColumnWidth>true</pl:autoMinColumnWidth><pl:autoMinRowHeight>true</pl:autoMinRowHeight><pl:minColumnWidth>0</pl:minColumnWidth><pl:minRowHeight>0</pl:minRowHeight><pl:fixDimensionsHeader>auto</pl:fixDimensionsHeader><pl:fixTimeScaleHeader>auto</pl:fixTimeScaleHeader><pl:border width="0"><v8ui:style xsi:type="v8ui:ControlBorderType">Single</v8ui:style></pl:border><pl:newItemsTextType>String</pl:newItemsTextType>"##;
    let node = crate::brace::parse(SAMPLE)?;
    let mut got = String::new();
    for c in st.elems() {
        xml_text(c, &mut got);
    }
    st.touch_all();
    let norm = |t: &str| t.lines().map(str::trim).collect::<String>();
    if norm(&got) != XML {
        return Err(err(st, "планировщик не как образец (КонтактныеЛица.ФормаЛиды БП): запись не выведена"));
    }
    Ok(node)
}

fn xml_text(e: &El, out: &mut String) {
    let esc = |t: &str, attr: bool| {
        let t = t.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;");
        if attr { t.replace('"', "&quot;") } else { t }
    };
    out.push('<');
    out.push_str(&e.name);
    for (k, v) in &e.attrs {
        out.push_str(&format!(" {k}=\"{}\"", esc(v, true)));
    }
    if e.kids.is_empty() {
        out.push_str("/>");
        return;
    }
    out.push('>');
    for k in &e.kids {
        match k {
            super::dom::Kid::El(c) => xml_text(c, out),
            super::dom::Kid::Text(t) => out.push_str(&esc(t, false)),
        }
    }
    out.push_str(&format!("</{}>", e.name));
}

/// Пустые настройки СКД условного оформления (часть настроек, как пишет сериализатор).
const EMPTY_SETTINGS: &str = "\u{feff}<?xml version=\"1.0\" encoding=\"UTF-8\"?>\r\n<Settings xmlns=\"http://v8.1c.ru/8.1/data-composition-system/settings\" xmlns:dcscor=\"http://v8.1c.ru/8.1/data-composition-system/core\" xmlns:pal=\"http://v8.1c.ru/8.1/data/ui/colors/palette\" xmlns:style=\"http://v8.1c.ru/8.1/data/ui/style\" xmlns:sys=\"http://v8.1c.ru/8.1/data/ui/fonts/system\" xmlns:v8=\"http://v8.1c.ru/8.1/data/core\" xmlns:v8ui=\"http://v8.1c.ru/8.1/data/ui\" xmlns:web=\"http://v8.1c.ru/8.1/data/ui/colors/web\" xmlns:win=\"http://v8.1c.ru/8.1/data/ui/colors/windows\" xmlns:xs=\"http://www.w3.org/2001/XMLSchema\" xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\"/>";

const ROOT_NS: &[(&str, &str)] = &[
    ("xmlns", "http://v8.1c.ru/8.3/xcf/logform"),
    ("xmlns:app", "http://v8.1c.ru/8.2/managed-application/core"),
    ("xmlns:cfg", "http://v8.1c.ru/8.1/data/enterprise/current-config"),
    ("xmlns:dcscor", "http://v8.1c.ru/8.1/data-composition-system/core"),
    ("xmlns:dcssch", "http://v8.1c.ru/8.1/data-composition-system/schema"),
    ("xmlns:dcsset", "http://v8.1c.ru/8.1/data-composition-system/settings"),
    ("xmlns:ent", "http://v8.1c.ru/8.1/data/enterprise"),
    ("xmlns:lf", "http://v8.1c.ru/8.2/managed-application/logform"),
    ("xmlns:pal", "http://v8.1c.ru/8.1/data/ui/colors/palette"),
    ("xmlns:style", "http://v8.1c.ru/8.1/data/ui/style"),
    ("xmlns:sys", "http://v8.1c.ru/8.1/data/ui/fonts/system"),
    ("xmlns:v8", "http://v8.1c.ru/8.1/data/core"),
    ("xmlns:v8ui", "http://v8.1c.ru/8.1/data/ui"),
    ("xmlns:web", "http://v8.1c.ru/8.1/data/ui/colors/web"),
    ("xmlns:win", "http://v8.1c.ru/8.1/data/ui/colors/windows"),
    ("xmlns:xr", "http://v8.1c.ru/8.3/xcf/readable"),
    ("xmlns:xs", "http://www.w3.org/2001/XMLSchema"),
    ("xmlns:xsi", "http://www.w3.org/2001/XMLSchema-instance"),
    ("version", "2.21"),
];

/// Есть ли в форме отложенная задача читателя, создающая менеджер расширений формы (см. ключ 1 в `form_dict`):
/// непустые события или исключенные команды любого элемента или формы, команда кнопки, источник команд, команда
/// элемента командного интерфейса формы.
fn manager_init(doc: &El) -> bool {
    fn walk(e: &El, cmd_if: bool) -> bool {
        e.peek_elems().any(|c| match c.name.as_str() {
            "Events" | "CommandSet" => c.peek_elems().next().is_some() || walk(c, cmd_if),
            "CommandName" | "CommandSource" => c.peek_text().is_some_and(|t| !t.is_empty()),
            "Command" if cmd_if => c.peek_text().is_some_and(|t| !t.is_empty()),
            "CommandInterface" => walk(c, true),
            _ => walk(c, cmd_if),
        })
    }
    walk(doc, false)
}

/// Классы основного реквизита форм объектов (словарь формы: ключи 24–26).
const OBJECT_CLASSES: &[&str] = &[
    "CatalogObject", "DocumentObject", "TaskObject", "ExchangePlanObject", "BusinessProcessObject", "ChartOfCharacteristicTypesObject",
    "ChartOfCalculationTypesObject", "ChartOfAccountsObject",
];

/// Словарь свойств формы по виду (поле 18): `N,(<ключ>,<значение>)…`. Сверка БП: у формы справочника и плана видов
/// характеристик — 0 (UseForFoldersAndItems), документа — 2, 3, 4 (AutoTime, UsePostingMode, RepostOnWrite), формы
/// объекта — 24–26 (постоянные) при модуле, исключенных командах или командном интерфейсе; формы без них и без событий,
/// команд, кнопок, условного оформления — без 24–26. Другое сочетание (данных нет), формы списка и отчета — отказ.
fn form_dict<'e>(f: &F, doc: &'e El, get: &dyn Fn(&str) -> Option<&'e El>, class: &str, module: Option<&[u8]>) -> R<Vec<Node>> {
    let hash = |g: &str, v: Node| l(vec![s("#"), a(g), v]);
    let mut d: Vec<(&str, Node)> = Vec::new();
    let req = |k: &str| get(k).ok_or_else(|| format!("нет <{k}> у формы {class}"));
    if let Some(e) = get("GroupList").filter(|_| !class.starts_with("DynamicList")) {
        return Err(err(e, "GroupList у формы не списка: не встречено"));
    }
    match class {
        "CatalogObject" | "ChartOfCharacteristicTypesObject" => {
            let e = req("UseForFoldersAndItems")?;
            d.push(("0", hash("59ef2b80-c86b-11d5-a3c1-0050bae0a776", token_of(e, crate::xml::form::FOLDERS_AND_ITEMS_USE)?)));
        }
        "DocumentObject" => {
            d.push(("2", hash("adeb08a0-415c-11d6-b9d1-0050bae0a95d", token_of(req("AutoTime")?, crate::xml::form::AUTO_TIME_MODE)?)));
            d.push(("3", hash("20d89b09-bd04-4304-a8c7-4d07fac6338a", token_of(req("UsePostingMode")?, crate::xml::form::POSTING_MODE_USE)?)));
            d.push(("4", l(vec![s("B"), a(if boolean(req("RepostOnWrite")?)? { "1" } else { "0" })])));
        }
        // Ключ 1 (группа пользовательских настроек списка) — слот расширения формы динамического списка: его копирует в
        // словарь формы mngbase FUN_1802c0e40 (→ mngcore FUN_180361f60) при инициализации менеджера расширений формы
        // FUN_1802e4070; менеджер создается первой отложенной задачей читателя, которая берет его функтором
        // (`std::function` задачи → backend FUN_180e18800): события FUN_18070ef90, исключенные команды FUN_1807107e0,
        // команда кнопки FUN_180761100, CommandSource FUN_1807617c0 (CommandBar 0x21, ButtonGroup 0x1f, Popup 0x20),
        // команда элемента командного интерфейса FUN_1807531f0. Без них ключ ставит только читатель GroupList
        // (FUN_18074f7d0). Трасса БП: 184 формы с ключом — все через эти задачи, 120 без ключа — ни одной.
        c if c.starts_with("DynamicList") => {
            let gl = get("GroupList");
            if gl.is_some() || manager_init(doc) {
                d.push(("1", l(vec![s("N"), gl.map(|e| item_id(f, e)).transpose()?.unwrap_or_else(|| a("0"))])));
            }
        }
        "ReportObject" => d = report_dict(f, get)?,
        // основной реквизит — компоновщик настроек (ИнтерфейсДокументовЭДО.НастройкаОтборовСписка — единственная такая
        // форма в БП, 8.5 и Клиенте ЭДО): 12, 16, 23 (группа пользовательских настроек), 28.
        // shortcut: один образец; без CustomSettingsFolder — отказ, пока не встретится
        "dcsset:SettingsComposer" => {
            let e = get("CustomSettingsFolder").ok_or("форма компоновщика настроек без CustomSettingsFolder: не встречено")?;
            d.extend([("12", l(vec![s("S"), s("")])), ("16", l(vec![s("B"), a("0")])), ("23", l(vec![s("N"), item_id(f, e)?])), ("28", l(vec![s("B"), a("0")]))]);
        }
        _ => {}
    }
    if OBJECT_CLASSES.contains(&class) {
        let has = |k: &str| get(k).is_some();
        let module = module.and_then(|b| std::str::from_utf8(b.strip_prefix(super::BOM).unwrap_or(b)).ok()).is_some_and(|t| !t.trim().is_empty());
        let deep = |n: &str| doc.peek_has(n);
        // кнопка без модуля — тоже 24–26 (опыт 28.10.2026: ibcmd import ЭДО с кнопкой в форме и в командной панели
        // у двух форм элемента без модуля)
        let a_rule = module || has("CommandSet") || has("CommandInterface") || deep("Button");
        // команды без модуля ключей не дают (опыт 28.10.2026: ibcmd import формы ЭДО ВидыЦен.ФормаЭлемента с командой —
        // словарь только с ключом 0); события и условное оформление без модуля не проверены — отказ
        let b_rule = a_rule || has("Events") || deep("ConditionalAppearance");
        if a_rule != b_rule {
            return Err("признак ключей 24–26 словаря формы не выведен для такого сочетания (события/команды без модуля)".into());
        }
        if a_rule {
            d.push(("24", l(vec![s("B"), a("0")])));
            d.push(("25", l(vec![s("U")])));
            d.push(("26", l(vec![s("B"), a("1")])));
        }
    }
    let mut v = vec![an(d.len())];
    for (k, n) in d {
        v.extend([a(k), n]);
    }
    Ok(v)
}

/// Ссылка на элемент формы: `<id>:02023637…` или имя элемента → id.
fn item_id(f: &F, e: &El) -> R<Node> {
    let t = e.text()?;
    // панель действий выделенных строк, которую платформа добавляет полю и таблице, — элемент без id (`{0}`): номер
    // элемента по имени (FUN_18075fdc0 → FUN_18074fba0) у нее 0
    let auto_panel = || t.strip_suffix("ПанельДействийВыделенныхСтрок").and_then(|n| f.items.get(n)).is_some_and(|it| it.tag.ends_with("Field") || it.tag == "Table");
    Ok(match t.split_once(':') {
        Some((id, ELEM)) if super::tpl::int(id).is_some() => a(id),
        _ if !f.items.contains_key(&t) && auto_panel() => a("0"),
        _ => a(f.items.get(&t).ok_or_else(|| err(e, &format!("нет элемента {t}")))?.id.clone()),
    })
}

/// Словарь формы отчета (сверка БП: 147 форм). Вид формы (7) Main/Variant — полный набор ключей 5–29: результат (5),
/// данные расшифровки (6), представление варианта (20) — ссылки на реквизиты формы, 12 — `Отчет.<имя отчета>`,
/// 23 — группа пользовательских настроек (id элемента), прочие постоянные; вид Settings — только 7, 21, 23, 27, 29.
/// Режимы ReportResultViewMode, ViewModeApplicationOnSetReportResult встречены только Auto (27, 29).
fn report_dict<'e>(f: &F, get: &dyn Fn(&str) -> Option<&'e El>) -> R<Vec<(&'static str, Node)>> {
    let hash = |g: &str, v: Node| l(vec![s("#"), a(g), v]);
    let attr_ref = |k: &str| -> R<Node> {
        let v = match get(k) {
            None => l(vec![a("0"), s("")]),
            Some(e) => {
                let n = e.text()?;
                let at = f.attrs.get(&n).ok_or_else(|| err(e, &format!("нет реквизита {n}")))?;
                l(vec![a("1"), l(vec![a(at.id.clone())]), s("")])
            }
        };
        Ok(hash("11cfd3e0-86f8-4480-aaa5-dc6a6ccac689", v))
    };
    // AutoShowState — обратное выгрузке (ключ 21; значения enums.dll FUN_180017e10)
    let auto_show = match get("AutoShowState") {
        None => a("0"),
        Some(e) => token(e, &[("Auto", 0), ("DontShow", 1), ("Show", 2), ("ShowOnComposition", 3)])?,
    };
    for k in ["ReportResultViewMode", "ViewModeApplicationOnSetReportResult"] {
        if let Some(e) = get(k) {
            token(e, &[("Auto", 0)])?;
        }
    }
    let kind = token(get("ReportFormType").ok_or("нет <ReportFormType> у формы отчета: не встречено")?, &[("Main", 0), ("Settings", 1), ("Variant", 2)])?;
    let settings = kind.atom() == Some("1");
    let main_type = f.main.as_deref().and_then(|m| f.attrs.get(m)).and_then(|a| a.types.first()).map(String::as_str);
    // Форма настроек «любого отчета» (основной реквизит `cfg:ReportObject`, общие ФормаНастроекОтчета и
    // ВспомогательнаяФормаНастроекОтчета) — полный набор ключей 5–29, как у основной; форма настроек самого отчета
    // (`cfg:ReportObject.<Имя>`) — 7, 21, 23, 27, 29. Сверено на БП, 8.5, Клиенте ЭДО (по 2 общие формы, все формы
    // настроек отчетов). shortcut: признак выведен по образцам, не из кода — при расхождении искать в mngbase.
    // отчет без основной схемы компоновки — тоже полный набор (форма настроек НастройкаСписка ERP)
    let no_schema = main_type.and_then(|t| t.strip_prefix("cfg:ReportObject.")).and_then(|n| f.x.objects.get(&format!("Report.{n}"))).is_some_and(|o| o.props.get("MainDataCompositionSchema").is_none_or(|v| v.is_empty()));
    let full = !settings || main_type == Some("cfg:ReportObject") || no_schema;
    let folder = match get("CustomSettingsFolder") {
        None if !full => return Err("форма настроек отчета без CustomSettingsFolder: не встречено".into()),
        None => a("0"),
        Some(e) => item_id(f, e)?,
    };
    let mut d = Vec::new();
    if full {
        // общая форма с типом `cfg:ReportObject` (любой отчет) — пустая строка
        let report = match f.main.as_deref().and_then(|m| f.attrs.get(m)).and_then(|a| a.types.first()).map(String::as_str) {
            Some("cfg:ReportObject") => String::new(),
            t => format!("Отчет.{}", t.and_then(|t| t.strip_prefix("cfg:ReportObject.")).ok_or("тип основного реквизита формы отчета")?),
        };
        // `urn:form:md:<вид>:<uuid конфигурации>`
        let conf = f.x.id("Configuration").ok_or("нет uuid конфигурации")?;
        let urn = |n: &str| l(vec![s("S"), s(format!("urn:form:md:{n}:{conf}"))]);
        let main = || l(vec![s("S"), s("Основной")]);
        d.extend([
            ("5", attr_ref("ReportResult")?),
            ("6", attr_ref("DetailsData")?),
            ("7", hash("acbc2eeb-2efb-48e4-b78a-661fd09fcf80", kind)),
            ("8", urn("14")),
            ("9", urn("13")),
            ("10", urn("16")),
            ("11", urn("15")),
            ("12", l(vec![s("S"), s(report)])),
            ("13", main()),
            ("14", l(vec![s("B"), a("0")])),
            ("15", l(vec![s("U")])),
            ("16", l(vec![s("B"), a("0")])),
            ("17", crate::brace::parse("{\"#\",4772b3b4-f4a3-49c0-a1a5-8cb5961511a3,{6,1e512aab-1b41-4ef6-9375-f0137be9dd91,0,0,{1,{1e512aab-1b41-4ef6-9375-f0137be9dd91,{\"Основной\",0,{\"S\",\"Основной\"},{4,0,{0},\"\",-1,-1,0,0,\"\"},0,0,\"\"}}},{\"Pattern\"},0,0}}")?),
            ("18", main()),
            ("19", l(vec![s("S"), s("")])),
            ("20", attr_ref("VariantAppearance")?),
        ]);
    } else {
        d.push(("7", hash("acbc2eeb-2efb-48e4-b78a-661fd09fcf80", kind)));
    }
    d.push(("21", hash("f26c3706-a6ca-45cb-869a-e6ad38cd5f78", auto_show)));
    if full {
        d.push(("22", l(vec![s("S"), s(ZERO)])));
    }
    d.extend([
        ("23", l(vec![s("N"), folder])),
        ("27", hash("b9311bea-b26b-4ae0-8b5d-7b64048fd2df", a("0"))),
        ("29", hash("874260df-7e23-4f02-9e10-5794914b5adf", a("0"))),
    ]);
    Ok(d)
}

/// Класс типа реквизита: `DocumentObject`, `CatalogObject`, `DynamicList`… (по имени первого типа).
fn type_class(e: Option<&El>) -> String {
    let Some(t) = e.and_then(|t| t.peek_child("v8:Type")) else { return String::new() };
    let v = t.peek_text().unwrap_or_default();
    let v = v.strip_prefix("cfg:").unwrap_or(&v);
    v.split('.').next().unwrap_or_default().to_string()
}

/// Запись формы и прочитанные файлы встроенных картинок (`<элемент>/<файл>` от `Form/Items`); `read` читает их там.
pub fn record(doc: &El, module: Option<&[u8]>, x: &Index, rel: &str, read: &dyn Fn(&str) -> Result<Vec<u8>, String>) -> R<(Rec, Vec<String>)> {
    if doc.name != "Form" {
        return Err("корень не <Form>".into());
    }
    // при совместимости 8.3.16 и ниже (схема полей списка не пишется) платформа не объявляет dcssch в корне; при новой
    // совместимости корень без dcssch — не платформенный XML (префикс в настройках списка был бы не связан): отказ
    let no_sch = doc.attr("xmlns:dcssch").is_none() && x.compat.is_some_and(|c| c < 0x139bf);
    for (k, v) in ROOT_NS.iter().filter(|(k, _)| !(no_sch && *k == "xmlns:dcssch")) {
        if doc.attr(k) != Some(v) {
            return Err(format!("<Form>: {k} не {v:?}"));
        }
    }
    if doc.attrs.len() != ROOT_NS.len() - no_sch as usize {
        return Err("<Form>: лишние атрибуты".into());
    }
    let mut p: HashMap<&str, &El> = HashMap::new();
    for c in doc.elems() {
        if p.insert(c.name.as_str(), c).is_some() {
            return Err(err(c, "повтор"));
        }
    }
    // реквизиты: классы типов (для событий и словаря формы)
    let mut attrs = HashMap::new();
    let mut main = None;
    // класс основного реквизита для стандартных команд; у динамического списка — с классом основной таблицы
    let mut main_key = String::new();
    if let Some(at) = p.get("Attributes") {
        for a in at.peek_elems().filter(|e| e.name == "Attribute") {
            let name = a.peek_attr("name").ok_or_else(|| err(a, "нет атрибута name"))?.to_string();
            if a.peek_child("MainAttribute").and_then(El::peek_text).as_deref() == Some("true") {
                main = Some(name.clone());
                main_key = type_class(a.peek_child("Type"));
                if main_key == "DynamicList" {
                    let mt = a.peek_child("Settings").and_then(|s| s.peek_child("MainTable")).and_then(El::peek_text).unwrap_or_default();
                    main_key = format!("DynamicList:{}", mt.split('.').next().unwrap_or_default());
                }
            }
            let (mut cols, mut extra) = (Vec::new(), Vec::new());
            if let Some(cs) = a.peek_child("Columns") {
                cols = columns(cs)?;
                for ac in cs.peek_elems().filter(|e| e.name == "AdditionalColumns") {
                    let t = ac.peek_attr("table").ok_or_else(|| err(ac, "нет атрибута table"))?;
                    extra.extend(columns(ac)?.into_iter().map(|c| (t.to_string(), c)));
                }
            }
            let id = a.peek_attr("id").ok_or_else(|| err(a, "нет атрибута id"))?.to_string();
            attrs.insert(name, Attr { class: type_class(a.peek_child("Type")), id, types: types_of(a.peek_child("Type")), cols, extra, main_table: a.peek_child("Settings").and_then(|s| s.peek_child("MainTable")).and_then(El::peek_text) });
        }
    }
    let mut cmds = HashMap::new();
    if let Some(cs) = p.get("Commands") {
        for c in cs.peek_elems().filter(|e| e.name == "Command") {
            let attr = |k: &str| c.peek_attr(k).map(str::to_string).ok_or_else(|| err(c, &format!("нет атрибута {k}")));
            cmds.insert(attr("name")?, attr("id")?);
        }
    }
    let mut items = HashMap::new();
    for k in ["AutoCommandBar", "ChildItems"] {
        if let Some(e) = p.get(k) {
            collect_items(e, &attrs, &mut items);
        }
    }
    let mut referenced = referenced_items(doc);
    let main_dl = main.as_deref().filter(|m| attrs.get(*m).is_some_and(|a| a.class == "DynamicList"));
    referenced.extend(table::main_list_table(doc, p.contains_key("CommandSet"), main_dl, &items, &referenced));
    // динамические списки с основной таблицей: поля корня (номера FieldsMap)
    let mut lists = HashMap::new();
    for a in p.get("Attributes").into_iter().flat_map(|at| at.peek_elems().filter(|e| e.name == "Attribute")) {
        let Some(st) = a.peek_child("Settings").filter(|s| s.peek_attr("xsi:type") == Some("DynamicList")) else { continue };
        let manual = st.peek_child("ManualQuery").and_then(El::peek_text).as_deref() == Some("true");
        if let (false, Some(mt)) = (manual, st.peek_child("MainTable").and_then(El::peek_text)) {
            let name = a.peek_attr("name").unwrap_or_default().to_string();
            lists.insert(name, dlist::List::new(x, &mt, st).map_err(|m| err(st, &m))?);
        }
        // свой запрос: поля корня — из текста запроса (dlist::List::from_query)
        if manual {
            let name = a.peek_attr("name").unwrap_or_default().to_string();
            lists.insert(name, dlist::List::from_query(x, st).map_err(|m| err(st, &m))?);
        }
    }
    let f = F { x, attrs, main, cmds, items, referenced, read, used: Default::default(), lists: std::cell::RefCell::new(lists), list_req: Default::default() };
    f.list_ca_only(doc)?;
    f.list_paths_in_load_order(doc);
    if let Ok(dump) = std::env::var("CFKIT_DP_DUMP") {
        f.dump_paths(doc, rel, &dump);
    }
    let get = |k: &str| p.get(k).copied();
    let b0 = |k: &str, yes: &str, no: &str| -> R<Node> { Ok(a(match get(k).map(boolean).transpose()? { Some(false) => no, Some(true) | None => yes })) };
    let tok = |k: &str, m: &[(&str, i64)], def: i64| -> R<Node> { get(k).map(|e| token(e, m)).unwrap_or_else(|| Ok(an(def))) };
    for k in p.keys() {
        if !matches!(
            *k,
            "WindowOpeningMode" | "Width" | "Height" | "EnterKeyBehavior" | "SaveDataInSettings" | "AutoSaveDataInSettings" | "AutoTitle" | "Title"
                | "Group" | "ChildItemsWidth" | "AutoFillCheck" | "Customizable" | "Enabled" | "CommandBarLocation" | "ShowCommandBar" | "AutoURL"
                | "VerticalScroll" | "ScalingMode" | "HorizontalSpacing" | "VerticalSpacing" | "HorizontalAlign" | "VerticalAlign" | "ChildrenAlign"
                | "Scale" | "ShowTitle" | "ShowCloseButton" | "ConversationsRepresentation" | "CollapseItemsByImportanceVariant" | "SaveWindowSettings"
                | "AutoCommandBar" | "Events" | "Attributes" | "Parameters" | "Commands" | "ChildItems" | "CommandSet" | "UseForFoldersAndItems"
                | "AutoTime" | "UsePostingMode" | "RepostOnWrite" | "ReportFormType" | "ReportResult" | "DetailsData" | "ReportResultViewMode"
                | "AutoShowState" | "ViewModeApplicationOnSetReportResult" | "CustomSettingsFolder" | "VariantAppearance" | "MobileDeviceCommandBarContent" | "CommandInterface"
                | "GroupList" | "SettingsStorage"
        ) {
            return Err(err(p[k], &format!("{k} у формы пока не загружается")));
        }
    }
    let wom = tok("WindowOpeningMode", &[("DontBlock", 0), ("LockOwner", 1), ("LockWholeInterface", 2)], 0)?;
    // WindowOpeningMode выгрузка пишет всегда (БП: 7 573 из 7 573); нет — «не задано» формы конструктора 8.5: второе
    // место режима — 3 (загрузка платформы, форма документа конструктора 27.10.2026)
    let wom_tail = if get("WindowOpeningMode").is_none() { a("3") } else { wom.clone() };
    let cbl_code = match get("CommandBarLocation").map(txt).transpose()?.as_deref() {
        None => 1,
        Some("None") => 0,
        Some("Top") => 2,
        Some("Bottom") => 3,
        Some(o) => return Err(format!("CommandBarLocation {o}")),
    };
    // ShowCommandBar пишется при None/Top/Bottom: false при None, true при Top/Bottom
    match (cbl_code, get("ShowCommandBar").map(boolean).transpose()?) {
        (1, None) | (0, Some(false)) | (2 | 3, Some(true)) => {}
        _ => return Err("ShowCommandBar не как у выгрузки".into()),
    }
    // Group выгрузка пишет всегда; нет — «не задано» формы конструктора 8.5 (хвост 7 = 4, `form_v59::UNSET_GROUP`):
    // у загрузки платформы тогда поле 11 корня и хвост 11 — 1, последнее поле группировки — 4 (эталон 27.10.2026)
    let unset_group = get("Group").is_none();
    let group = get("Group").map(txt).transpose()?.unwrap_or_else(|| "Vertical".into());
    let (g11, g18) = match group.as_str() {
        _ if unset_group => (1, 0),
        "Vertical" => (0, 0),
        "Horizontal" => (1, 1),
        "HorizontalIfPossible" => (1, 2),
        "AlwaysHorizontal" => (1, 3),
        o => return Err(format!("Group {o}")),
    };
    let tail11 = match group.as_str() {
        _ if unset_group => 1,
        "HorizontalIfPossible" => 2,
        "Vertical" => 0,
        _ => 1,
    };
    let vs = get("VerticalScroll").map(txt).transpose()?;
    let (vs2, vs12) = match vs.as_deref() {
        None => (0, 0),
        Some("useIfNecessary") => (2, 2),
        Some("useWithoutStretch") => (0, 3),
        Some(o) => return Err(format!("VerticalScroll {o}")),
    };
    // хранилище настроек — uuid объекта по полному имени, ненайденный — GUID как есть (обратное выгрузке: поле 8)
    let storage = match get("SettingsStorage") {
        Some(e) => {
            let t = e.text()?;
            a(x.id(&t).map(str::to_string).or_else(|| crate::brace::is_guid(&t).then(|| t.clone())).ok_or_else(|| err(e, &format!("нет хранилища настроек {t}")))?)
        }
        None => a(ZERO),
    };
    let width = get("Width").map(num).transpose()?.unwrap_or_else(|| "0".into());
    let height = get("Height").map(num).transpose()?.unwrap_or_else(|| "0".into());
    let show_title = get("ShowTitle").map(boolean).transpose()?.unwrap_or(true);
    let spacing = [("None", 1), ("Half", 2), ("Single", 3), ("OneAndHalf", 4), ("Double", 5)];
    let mut tree = vec![
        a("59"),
        a("0"),
        wom,
        a(width),
        a(height),
        tok("EnterKeyBehavior", &[("DefaultButton", 0)], 1)?,
        tok("SaveDataInSettings", &[("UseList", 1)], 0)?,
        tok("AutoSaveDataInSettings", &[("Use", 1)], 0)?,
        storage,
        b0("AutoTitle", "1", "0")?,
        ml(get("Title"))?,
        an(g11),
        tok("ChildItemsWidth", &[("Auto", 0), ("Equal", 1), ("LeftWide", 2), ("LeftWidest", 3), ("LeftNarrow", 4), ("LeftNarrowest", 5)], 0)?,
        b0("AutoFillCheck", "1", "0")?,
        b0("Customizable", "1", "0")?,
        b0("Enabled", "1", "0")?,
        a("0"),
        an(cbl_code),
    ];
    tree.extend(form_dict(&f, doc, &get, &main_key, module)?);
    let document = f.document();
    // имя события GUID — так выгрузка пишет событие, которого нет в ее таблице: как есть
    tree.push(events(get("Events"), &|n| form_event(n, document).map(str::to_string).or_else(|| crate::brace::is_guid(n).then(|| n.to_string())))?);
    tree.push(match get("CommandSet") {
        None => l(vec![a("0")]),
        Some(cs) => {
            let mut names = Vec::new();
            for c in cs.elems() {
                if c.name != "ExcludedCommand" {
                    return Err(err(c, "ожидалась <ExcludedCommand>"));
                }
                names.push(c.text()?);
            }
            cmdset::excluded(&main_key, &names).map_err(|m| err(cs, &m))?
        }
    });
    tree.push(a("1"));
    tree.push(f.auto_command_bar(get("AutoCommandBar").ok_or("нет <AutoCommandBar>")?)?);
    tree.extend(f.children(get("ChildItems"))?);
    tree.extend([s(""), s(""), a("0")]);
    let tail = vec![
        b0("AutoURL", "1", "0")?,
        s(""),
        an(vs2),
        tok("ScalingMode", &[("Normal", 1), ("Compact", 2)], 0)?,
        a("0"),
        a("0"),
        tok("HorizontalSpacing", &spacing, 0)?,
        tok("VerticalSpacing", &spacing, 0)?,
        tok("HorizontalAlign", platform::H_ALIGN, 3)?,
        tok("VerticalAlign", &[("Top", 0), ("Center", 1), ("Bottom", 2)], 3)?,
        tok("ChildrenAlign", &[("Auto", 0), ("None", 1), ("ItemsLeftTitlesLeft", 2), ("ItemsRightTitlesLeft", 3), ("ItemsLeftTitlesRight", 4), ("ItemsRightTitlesRight", 5), ("TitlesLeftDataAuto", 6)], 0)?,
        an(tail11),
        an(vs12),
        get("Scale").map(num).transpose()?.map_or_else(|| a("100"), a),
        a(if show_title { "1" } else { "0" }),
        b0("ShowCloseButton", "1", "0")?,
        tok("ConversationsRepresentation", &[("Show", 1), ("DontShow", 2)], 0)?,
        tok("CollapseItemsByImportanceVariant", &[("Auto", 0), ("Use", 1), ("DontUse", 2)], 0)?,
        an(g18),
        f.mobile_bar(get("MobileDeviceCommandBarContent"))?,
        b0("SaveWindowSettings", "1", "0")?,
        l(vec![a("1"), a("0")]),
        l(vec![a("4"), a("0"), l(vec![a("0")]), s(""), a("-1"), a("-1"), a("1"), a("0"), s("")]),
        a("0"),
        a("0"),
        wom_tail,
        a("0"),
        an(match cbl_code {
            0 => 0,
            1 => 2,
            _ => 1,
        }),
        an(if unset_group { 4 } else { g18 }),
        a("0"),
        a("0"),
        a(if show_title { "2" } else { "0" }),
        a("0"),
    ];
    tree.extend(tail);
    let text = match module {
        Some(b) => String::from_utf8(b.strip_prefix(super::BOM).ok_or("модуль формы без BOM")?.to_vec()).map_err(|_| "модуль формы не UTF-8")?,
        None => String::new(),
    };
    let root = l(vec![
        a("4"),
        l(tree),
        s(text),
        f.attributes(get("Attributes"))?,
        f.parameters(get("Parameters"))?,
        f.commands(get("Commands"))?,
        f.command_interface(get("CommandInterface"), "NavigationPanel")?,
        f.command_interface(get("CommandInterface"), "CommandBar")?,
        a("0"),
        a("0"),
    ]);
    Ok((Rec::Raw(Doc::new(true, vec![root]).write()?.into_bytes()), f.used.take()))
}

/// Порядок путей сохраняемых полей: поэлементно, числа — по значению.
fn path_cmp(x: &Node, y: &Node) -> std::cmp::Ordering {
    match (x, y) {
        (Node::List(p), Node::List(q)) => p.iter().zip(q).map(|(a, b)| path_cmp(a, b)).find(|o| o.is_ne()).unwrap_or(p.len().cmp(&q.len())),
        (Node::Atom(p), Node::Atom(q)) => match (p.parse::<i64>(), q.parse::<i64>()) {
            (Ok(m), Ok(n)) => m.cmp(&n),
            _ => p.cmp(q),
        },
        _ => crate::brace::write(x).unwrap_or_default().cmp(&crate::brace::write(y).unwrap_or_default()),
    }
}

#[cfg(test)]
mod tests {
    use super::formatted_plain;

    #[test]
    fn formatted_title_plain_text() {
        assert_eq!(formatted_plain("<b>Жирный</> и <link e1cib/x>ссылка</>").as_deref(), Some("Жирный и ссылка"));
        assert_eq!(formatted_plain("кнопка <img :eb47324b-85f9-4172-9315-bba8015d9970/> (окно)").as_deref(), Some("кнопка   (окно)"));
        assert_eq!(formatted_plain("<<Все>> <fontsize 14>а</>").as_deref(), Some("<Все> а"));
        assert_eq!(formatted_plain("<незнакомый>а</> б<tab>в").as_deref(), Some("а бв"));
        assert_eq!(formatted_plain("x > y <b").as_deref(), Some("x > y "));
    }

    #[test]
    fn saved_field_paths_numeric_order() {
        let p = |t: &str| crate::brace::parse(t).unwrap();
        let mut v = vec![p("{2,{0,b},{11,x}}"), p("{1,{11}}"), p("{2,{0,b},{2,x}}"), p("{1,{1}}"), p("{1,{-8}}")];
        v.sort_by(super::path_cmp);
        let w: Vec<String> = v.iter().map(|n| crate::brace::write(n).unwrap().replace(['\r', '\n'], "")).collect();
        assert_eq!(w, ["{1,{-8}}", "{1,{1}}", "{1,{11}}", "{2,{0,b},{2,x}}", "{2,{0,b},{11,x}}"].map(String::from));
    }
}
