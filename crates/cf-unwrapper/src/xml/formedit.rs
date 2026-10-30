//! Модель редактора управляемой формы: дерево ее Form.xml (как выгрузка), модуль и картинки элементов. Правка — дерево,
//! сохранение — загрузка этого XML в запись `<id>.0` (`imp::form::record`), как у Конфигуратора после загрузки файлов;
//! незнакомое загрузке — отказ, `.cf` не меняется. Модуль формы — в той же записи; его правит редактор модуля
//! (`Config::set_module_text`), сохранение формы берет текущий текст из `.cf`.

use super::edit::{self, Change};
use super::imp::index::Index;
use super::imp::dom::{El, Kid};
use super::imp::{dom, Rec};
use std::collections::HashSet;
use crate::meta::Config;

pub struct FormDoc {
    /// uuid формы (запись описания), полное имя `Catalog.А.Form.Ф`
    pub id: String,
    pub full: String,
    pub dom: dom::El,
    /// Form.xml при открытии: дерево без правок не сохраняется (поля записи, которых нет в XML, остаются как были)
    orig: String,
    /// картинки элементов: путь от Form/Items (`Кнопка/Picture.png`), данные
    pictures: Vec<(String, Vec<u8>)>,
}

/// Форма по uuid; не управляемая форма или выгрузка ее отвергает — ошибка с причиной (такую форму окно показывает
/// только для чтения).
pub fn open(cfg: &Config, fnames: &super::form::Names, id: &str) -> Result<FormDoc, String> {
    let (owner, form) = owner_of(cfg, id).ok_or(format!("нет формы {id}"))?;
    let entry = format!("{id}.0");
    let text = cfg.entry_text(&entry).ok_or(format!("нет записи {entry}"))?;
    if cfg.index.get(&entry).is_some_and(|&i| crate::container::is_container(&crate::container::content(&cfg.entries[i]))) {
        return Err("форма обычного приложения".into());
    }
    let xml = super::form::form_xml(fnames, &text)?.ok_or("форма без элементов")?;
    let xml = String::from_utf8(xml).map_err(|_| "Form.xml не UTF-8")?;
    let pictures = super::payload::form_pictures(&text).into_iter().map(|(p, d)| (p.trim_start_matches("Form/Items/").to_string(), d)).collect();
    Ok(FormDoc { id: id.to_string(), full: if owner == "CommonForm" { format!("CommonForm.{form}") } else { format!("{owner}.Form.{form}") }, dom: dom::parse(&xml)?, orig: xml, pictures })
}

/// Индекс загрузки по всей конфигурации (поля чужих таблиц, перечисления, измерения регистров…), как у загрузки
/// каталога выгрузки; строится выгрузкой описаний в память. После правки метаданных — построить заново.
pub fn index(cfg: &Config) -> Result<Index, String> {
    let files = super::index_files(cfg)?;
    // внешняя обработка: индекс по `<Имя>.xml` и `<Имя>/…` (Configuration.xml у нее нет)
    if cfg.root.kind == "ExternalDataProcessor" {
        let read = |rel: &str| files.get(rel.trim_start_matches("./")).cloned();
        return Ok(Index::build_external_from(&read, "ExternalDataProcessor", &cfg.root.header.name));
    }
    let conf = files.get("Configuration.xml").ok_or("Configuration.xml не выгружается")?;
    let conf = dom::parse(std::str::from_utf8(conf).map_err(|_| "Configuration.xml не UTF-8")?)?;
    Index::build_from(&conf, &|rel| files.get(rel).cloned())
}

/// Записать форму в конфигурацию; дерево без правок или запись не изменилась — `None`.
pub fn save(cfg: &mut Config, x: &Index, doc: &FormDoc) -> Result<Option<Change>, String> {
    if doc.dom.write() == doc.orig {
        return Ok(None);
    }
    let rec = record(cfg, x, doc)?;
    edit::store(cfg, &format!("{}.0", doc.id), &rec)
}

/// Запись формы из дерева — загрузка его XML с текущим модулем формы.
pub fn record(cfg: &Config, x: &Index, doc: &FormDoc) -> Result<Vec<u8>, String> {
    // из текста, а не из `doc.dom`: загрузка отмечает прочитанное в дереве
    let el = dom::parse(&doc.dom.write())?;
    let module = cfg.module_text(&format!("{}.0", doc.id)).filter(|m| !m.is_empty()).map(|m| [super::w::BOM, m.as_bytes()].concat());
    let pics = |f: &str| doc.pictures.iter().find(|(p, _)| p == f).map(|(_, d)| d.clone()).ok_or(format!("Form/Items/{f}: нет картинки"));
    Ok(match super::imp::form::record(&el, module.as_deref(), x, &format!("{}/Form.xml", doc.full), &pics)?.0 {
        Rec::Raw(b) => b,
        Rec::Dir(v) => crate::container::write(&v.into_iter().map(|(name, data)| crate::container::Entry { name, created: 0, modified: 0, data }).collect::<Vec<_>>()),
    })
}

/// Владелец формы `Класс.Имя` и имя формы.
fn owner_of(cfg: &Config, id: &str) -> Option<(String, String)> {
    // внешняя обработка: формы — у корня (ExternalDataProcessor.<Имя>.Form.<Форма>)
    if cfg.root.kind == "ExternalDataProcessor" {
        let f = cfg.root.children.iter().find(|c| c.kind == "Form" && c.header.id == id)?;
        return Some((format!("ExternalDataProcessor.{}", cfg.root.header.name), f.header.name.clone()));
    }
    cfg.groups.iter().find_map(|g| {
        g.items.iter().find_map(|it| {
            if g.class == "CommonForm" {
                return (it.header.id == id).then(|| ("CommonForm".to_string(), it.header.name.clone()));
            }
            let f = it.children.iter().find(|c| c.kind == "Form" && c.header.id == id)?;
            Some((format!("{}.{}", g.class, it.header.name), f.header.name.clone()))
        })
    })
}

/// Узлы формы, которые не элементы: id и имена в них свои.
const COLLECTIONS: &[&str] = &["Attributes", "Commands", "Parameters", "CommandInterface", "Events", "CommandSet"];

/// Наибольший id элементов формы (с контекстными меню и подсказками); новые — следующие, как у Конфигуратора
/// (эталон 28.10.2026: группа 51, ее подсказка 52, поле 53–55).
// shortcut: у Конфигуратора после поля из перетаскивания один номер пропущен (56) — не воспроизводится.
fn max_id(e: &El) -> i64 {
    let own = e.peek_attr("id").and_then(|i| i.parse().ok()).unwrap_or(0);
    e.peek_elems().filter(|c| !COLLECTIONS.contains(&c.name.as_str())).map(max_id).fold(own, i64::max)
}

fn item_names(e: &El, out: &mut HashSet<String>) {
    if let Some(n) = e.peek_attr("name") {
        out.insert(n.to_string());
    }
    e.peek_elems().filter(|c| !COLLECTIONS.contains(&c.name.as_str())).for_each(|c| item_names(c, out));
}

/// Первое свободное имя `<основа>N` (N с 1) или само имя, если свободно и `bare`; число в конце занятого имени
/// заменяется (поле реквизита «Реквизит1» при занятом — «Реквизит2», эталон Конфигуратора 28.10.2026).
fn free_name(taken: &HashSet<String>, stem: &str, bare: bool) -> String {
    if bare && !taken.contains(stem) {
        return stem.to_string();
    }
    let base = stem.trim_end_matches(|c: char| c.is_ascii_digit());
    (1..).map(|n| format!("{base}{n}")).find(|n| !taken.contains(n)).unwrap()
}

/// Виды элементов, которые добавляет окно: (вид, тег, основа имени, есть контекстное меню). Вид — тег, кроме
/// «UsualGroupNone» — обычная группа без отображения.
pub const NEW_ITEMS: &[(&str, &str, &str, bool)] = &[
    ("InputField", "InputField", "Поле", true),
    ("CheckBoxField", "CheckBoxField", "Поле", true),
    ("LabelField", "LabelField", "Поле", true),
    ("UsualGroup", "UsualGroup", "Группа", false),
    ("UsualGroupNone", "UsualGroup", "Группа", false),
    ("Pages", "Pages", "Группа", false),
    ("Page", "Page", "Группа", false),
    ("CommandBar", "CommandBar", "Группа", false),
    ("Popup", "Popup", "Группа", false),
    ("ButtonGroup", "ButtonGroup", "Группа", false),
    ("Button", "Button", "Кнопка", false),
    ("Table", "Table", "Таблица", true),
    ("LabelDecoration", "LabelDecoration", "Декорация", true),
    ("PictureDecoration", "PictureDecoration", "Декорация", true),
];

/// Элемент, в который добавляют внутрь.
pub fn is_container(tag: &str) -> bool {
    matches!(tag, "UsualGroup" | "Pages" | "Page" | "ColumnGroup" | "ButtonGroup" | "CommandBar" | "Popup" | "Table")
}

/// Командная панель и ее группы: кнопка в них — CommandBarButton, подменю и группа кнопок — только в них.
fn is_bar(tag: &str) -> bool {
    matches!(tag, "CommandBar" | "AutoCommandBar" | "ButtonGroup" | "Popup" | "ContextMenu")
}

/// Тег элемента, в ChildItems которого лежит элемент `id`.
fn parent_tag(e: &El, id: &str) -> Option<String> {
    e.peek_elems().filter(|c| !COLLECTIONS.contains(&c.name.as_str())).find_map(|c| {
        if c.name == "ChildItems" && c.peek_elems().any(|k| k.peek_attr("id") == Some(id)) {
            return Some(e.name.clone());
        }
        parent_tag(c, id)
    })
}

/// Таблица без пути к данным, как добавляет Конфигуратор (эталон 28.10.2026): ShowCommandBar, меню +1, командная
/// панель +2, подсказка +3, дополнения поиска +4, просмотра +7, управления поиском +10.
fn table_kids(name: &str, id: i64) -> Vec<Kid> {
    let el = |tag: &str, suffix: &str, n: i64| Kid::El(El::new(tag, &[("name", &format!("{name}{suffix}")), ("id", &n.to_string())], Vec::new()));
    let addition = |tag: &str, suffix: &str, kind: &str, n: i64| {
        let src = El::new("AdditionSource", &[], vec![Kid::El(El::leaf("Item", name)), Kid::El(El::leaf("Type", kind))]);
        let full = format!("{name}{suffix}");
        Kid::El(El::new(tag, &[("name", &full), ("id", &n.to_string())], vec![Kid::El(src), el("ContextMenu", &format!("{suffix}КонтекстноеМеню"), n + 1), el("ExtendedTooltip", &format!("{suffix}РасширеннаяПодсказка"), n + 2)]))
    };
    vec![
        Kid::El(El::leaf("ShowCommandBar", "auto")),
        el("ContextMenu", "КонтекстноеМеню", id + 1),
        el("AutoCommandBar", "КоманднаяПанель", id + 2),
        el("ExtendedTooltip", "РасширеннаяПодсказка", id + 3),
        addition("SearchStringAddition", "СтрокаПоиска", "SearchStringRepresentation", id + 4),
        addition("ViewStatusAddition", "СостояниеПросмотра", "ViewStatusRepresentation", id + 7),
        addition("SearchControlAddition", "УправлениеПоиском", "SearchControl", id + 10),
    ]
}

/// Новый элемент вида `kind` (`NEW_ITEMS`): в группу `at` (в конец) или после элемента `at`, без `at` — в конец
/// формы; `data_path` — поле реквизита (имя — имя реквизита, занятое — с номером). Возвращает id нового элемента.
/// Состав — как у Конфигуратора (эталон 28.10.2026, src/xml/new/designer/c1-item-elements.xml): кнопка — Type
/// (UsualButton, в командной панели CommandBarButton) и CommandName 0; группа без отображения — Representation None,
/// ShowTitle false. shortcut: номера — после наибольшего в форме; у Конфигуратора свой счетчик: в форме с наибольшим
/// 40 первый новый — 51 (два опыта на c1-item), поле занимает 4 номера, таблица 21, кнопка и группа 2 — правило
/// начала не выведено (нужен код Конфигуратора), платформе важна только уникальность.
pub fn add_item(root: &mut El, at: Option<&str>, kind: &str, data_path: Option<&str>) -> Result<String, String> {
    let &(_, tag, stem, menu) = NEW_ITEMS.iter().find(|n| n.0 == kind).ok_or(format!("добавление {kind} не сделано"))?;
    let mut taken = HashSet::new();
    item_names(root, &mut taken);
    let name = match data_path {
        Some(p) => free_name(&taken, p.rsplit('.').next().unwrap_or(p), true),
        None => free_name(&taken, stem, false),
    };
    let id = max_id(root) + 1;
    let mut kids = Vec::new();
    if let Some(p) = data_path {
        kids.push(Kid::El(El::leaf("DataPath", p)));
    }
    match kind {
        "InputField" => kids.push(Kid::El(El::leaf("ExtendedEditMultipleValues", "true"))),
        "UsualGroupNone" => kids.extend([Kid::El(El::leaf("Representation", "None")), Kid::El(El::leaf("ShowTitle", "false"))]),
        // тип уточняется после вставки — по родителю
        "Button" => kids.extend([Kid::El(El::leaf("Type", "UsualButton")), Kid::El(El::leaf("CommandName", "0"))]),
        _ => {}
    }
    if kind == "Table" {
        kids = table_kids(&name, id);
    } else {
        let mut n = id;
        if menu {
            n += 1;
            kids.push(Kid::El(El::new("ContextMenu", &[("name", &format!("{name}КонтекстноеМеню")), ("id", &n.to_string())], Vec::new())));
        }
        n += 1;
        kids.push(Kid::El(El::new("ExtendedTooltip", &[("name", &format!("{name}РасширеннаяПодсказка")), ("id", &n.to_string())], Vec::new())));
    }
    let new = El::new(tag, &[("name", &name), ("id", &id.to_string())], kids);
    let mut tree = root.clone();
    match at {
        None => child_items(&mut tree).kids.push(Kid::El(new)),
        Some(t) => {
            if insert(&mut tree, t, new).is_err() {
                return Err(format!("нет элемента id={t}"));
            }
        }
    }
    let sid = id.to_string();
    let bar = parent_tag(&tree, &sid).is_some_and(|p| is_bar(&p));
    if matches!(kind, "Popup" | "ButtonGroup") && !bar {
        return Err("подменю и группа кнопок — только в командной панели".into());
    }
    if kind == "Button" && bar {
        if let Some(t) = item_mut(&mut tree, Some(&sid)).and_then(|b| b.elems_mut().find(|c| c.name == "Type")) {
            t.kids = vec![Kid::Text("CommandBarButton".into())];
        }
    }
    *root = tree;
    name_command_bar(root);
    Ok(sid)
}

/// ChildItems элемента (создается, если нет).
fn child_items(e: &mut El) -> &mut El {
    if e.peek_child("ChildItems").is_none() {
        e.kids.push(Kid::El(El::new("ChildItems", &[], Vec::new())));
    }
    e.elems_mut().find(|c| c.name == "ChildItems").unwrap()
}

/// Вставка `new` в элемент `target` (контейнер — в конец) или после него; не найден — `new` обратно.
fn insert(e: &mut El, target: &str, new: El) -> Result<(), El> {
    let mut new = new;
    if e.peek_attr("id") == Some(target) && is_container(&e.name) {
        child_items(e).kids.push(Kid::El(new));
        return Ok(());
    }
    if e.name == "ChildItems" {
        if let Some(i) = e.kids.iter().position(|k| matches!(k, Kid::El(c) if c.peek_attr("id") == Some(target) && !is_container(&c.name))) {
            e.kids.insert(i + 1, Kid::El(new));
            return Ok(());
        }
    }
    for c in e.elems_mut().filter(|c| !COLLECTIONS.contains(&c.name.as_str())) {
        match insert(c, target, new) {
            Ok(()) => return Ok(()),
            Err(back) => new = back,
        }
    }
    Err(new)
}

/// Удалить элемент `id` со всем, что в нем.
pub fn delete_item(root: &mut El, id: &str) -> Result<(), String> {
    fn walk(e: &mut El, id: &str) -> bool {
        if e.name == "ChildItems" {
            if let Some(i) = e.kids.iter().position(|k| matches!(k, Kid::El(c) if c.peek_attr("id") == Some(id))) {
                e.kids.remove(i);
                return true;
            }
        }
        e.elems_mut().filter(|c| !COLLECTIONS.contains(&c.name.as_str())).any(|c| walk(c, id))
    }
    if !walk(root, id) {
        return Err(format!("элемент id={id} не удаляется (не в составе формы)"));
    }
    name_command_bar(root);
    Ok(())
}

/// Элемент `id` выше (`up`) или ниже соседа в своей группе; крайний — без изменений (`false`).
pub fn move_item(root: &mut El, id: &str, up: bool) -> bool {
    fn walk(e: &mut El, id: &str, up: bool) -> Option<bool> {
        if e.name == "ChildItems" {
            let els: Vec<usize> = e.kids.iter().enumerate().filter(|(_, k)| matches!(k, Kid::El(_))).map(|(i, _)| i).collect();
            if let Some(p) = els.iter().position(|&i| matches!(&e.kids[i], Kid::El(c) if c.peek_attr("id") == Some(id))) {
                let q = if up { p.checked_sub(1) } else { Some(p + 1).filter(|&q| q < els.len()) };
                return Some(q.map(|q| e.kids.swap(els[p], els[q])).is_some());
            }
        }
        e.elems_mut().filter(|c| !COLLECTIONS.contains(&c.name.as_str())).find_map(|c| walk(c, id, up))
    }
    let moved = walk(root, id, up).unwrap_or(false);
    if moved {
        name_command_bar(root);
    }
    moved
}

/// Командная панель формы после правки состава получает имя «ФормаКоманднаяПанель» (у нетронутой формы оно пустое;
/// эталон Конфигуратора 28.10.2026).
// shortcut: имя ставится при правке состава; у Конфигуратора — при сохранении измененной формы (опыт один).
fn name_command_bar(root: &mut El) {
    if let Some(cb) = root.elems_mut().find(|c| c.name == "AutoCommandBar") {
        if let Some(a) = cb.attrs.iter_mut().find(|(k, v)| k == "name" && v.is_empty()) {
            a.1 = "ФормаКоманднаяПанель".into();
        }
    }
}

fn collection<'a>(root: &'a mut El, name: &str) -> &'a mut El {
    if root.peek_child(name).is_none() {
        root.kids.push(Kid::El(El::new(name, &[], Vec::new())));
    }
    root.elems_mut().find(|c| c.name == name).unwrap()
}

fn next_in(root: &El, coll: &str, kind: &str, stem: &str) -> (String, String) {
    let items: Vec<&El> = root.peek_child(coll).into_iter().flat_map(|c| c.peek_elems()).filter(|e| e.name == kind).collect();
    let taken = items.iter().filter_map(|e| e.peek_attr("name")).map(str::to_string).collect();
    let id = items.iter().filter_map(|e| e.peek_attr("id")?.parse::<i64>().ok()).max().unwrap_or(0) + 1;
    (free_name(&taken, stem, false), id.to_string())
}

/// Новый реквизит формы «РеквизитN»: Строка неограниченной длины (эталон Конфигуратора 28.10.2026). Возвращает имя.
pub fn add_attribute(root: &mut El) -> String {
    let (name, id) = next_in(root, "Attributes", "Attribute", "Реквизит");
    let q = El::new("v8:StringQualifiers", &[], vec![Kid::El(El::leaf("v8:Length", "0")), Kid::El(El::leaf("v8:AllowedLength", "Variable"))]);
    let ty = El::new("Type", &[], vec![Kid::El(El::leaf("v8:Type", "xs:string")), Kid::El(q)]);
    collection(root, "Attributes").kids.push(Kid::El(El::new("Attribute", &[("name", &name), ("id", &id)], vec![Kid::El(ty)])));
    name
}

/// Удалить реквизит формы; основной и тот, на который ссылаются элементы (`…DataPath`), — отказ.
pub fn delete_attribute(root: &mut El, name: &str) -> Result<(), String> {
    fn used(e: &El, name: &str) -> bool {
        (e.name.ends_with("DataPath") && e.peek_text().is_some_and(|t| t == name || t.starts_with(&format!("{name}."))))
            || e.peek_elems().filter(|c| c.name != "Attributes").any(|c| used(c, name))
    }
    if used(root, name) {
        return Err(format!("реквизит «{name}» используется элементами формы"));
    }
    let attrs = collection(root, "Attributes");
    let i = attrs.kids.iter().position(|k| matches!(k, Kid::El(a) if a.peek_attr("name") == Some(name))).ok_or(format!("нет реквизита {name}"))?;
    if let Kid::El(a) = &attrs.kids[i] {
        if a.peek_child("MainAttribute").and_then(El::peek_text).as_deref() == Some("true") {
            return Err("основной реквизит формы не удаляется".into());
        }
    }
    attrs.kids.remove(i);
    Ok(())
}

/// Новая команда формы «КомандаN» (без действия — обработчик создается отдельно). Возвращает имя.
pub fn add_command(root: &mut El) -> String {
    let (name, id) = next_in(root, "Commands", "Command", "Команда");
    collection(root, "Commands").kids.push(Kid::El(El::new("Command", &[("name", &name), ("id", &id)], Vec::new())));
    name
}

/// Удалить команду формы; команду кнопки (`CommandName` Form.Command.<имя>) — отказ.
pub fn delete_command(root: &mut El, name: &str) -> Result<(), String> {
    fn used(e: &El, cmd: &str) -> bool {
        (e.name == "CommandName" && e.peek_text().as_deref() == Some(cmd)) || e.peek_elems().filter(|c| c.name != "Commands").any(|c| used(c, cmd))
    }
    if used(root, &format!("Form.Command.{name}")) {
        return Err(format!("команда «{name}» используется кнопками формы"));
    }
    let cmds = collection(root, "Commands");
    let i = cmds.kids.iter().position(|k| matches!(k, Kid::El(a) if a.peek_attr("name") == Some(name))).ok_or(format!("нет команды {name}"))?;
    cmds.kids.remove(i);
    Ok(())
}

/// Поле для реквизита формы (перетаскивание реквизита в дерево элементов): флажок у булева, иначе поле ввода.
pub fn field_for(root: &mut El, at: Option<&str>, attr: &str) -> Result<String, String> {
    let a = root.peek_child("Attributes").into_iter().flat_map(|c| c.peek_elems()).find(|a| a.peek_attr("name") == Some(attr)).ok_or(format!("нет реквизита {attr}"))?;
    let types: Vec<String> = a.peek_child("Type").into_iter().flat_map(|t| t.peek_elems()).filter(|v| v.name == "v8:Type").filter_map(El::peek_text).collect();
    let tag = if types == ["xs:boolean"] { "CheckBoxField" } else { "InputField" };
    add_item(root, at, tag, Some(attr))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn form() -> El {
        dom::parse(include_str!("new/designer/c1-item.xml")).unwrap()
    }

    #[test]
    fn add_group_field_and_attribute_like_designer() {
        let mut f = form();
        let top = max_id(&f);
        let g = add_item(&mut f, None, "UsualGroup", None).unwrap();
        assert_eq!(g, (top + 1).to_string());
        let p = add_item(&mut f, Some(&g), "InputField", None).unwrap();
        assert_eq!(p, (top + 3).to_string());
        let a = add_attribute(&mut f);
        assert_eq!(a, "Реквизит1");
        let x = field_for(&mut f, None, &a).unwrap();
        assert_eq!(x, (top + 6).to_string());
        let w = f.write();
        assert!(w.contains(&format!("<UsualGroup name=\"Группа1\" id=\"{g}\"><ExtendedTooltip name=\"Группа1РасширеннаяПодсказка\" id=\"{}\"/><ChildItems><InputField name=\"Поле1\"", top + 2)), "{w}");
        assert!(w.contains("<InputField name=\"Реквизит1\" id=\"") && w.contains("<DataPath>Реквизит1</DataPath>"));
        assert!(w.contains("<AutoCommandBar name=\"ФормаКоманднаяПанель\" id=\"-1\"/>"));
        assert!(delete_attribute(&mut f, "Реквизит1").is_err());
        delete_item(&mut f, &x).unwrap();
        delete_attribute(&mut f, "Реквизит1").unwrap();
        assert!(delete_attribute(&mut f, "Объект").is_err());
        assert!(move_item(&mut f, &g, true));
        assert!(!move_item(&mut f, "1", true));
        assert_eq!(add_command(&mut f), "Команда1");
    }

    /// Элементы из окна «Тип элемента» в корень и в командную панель — как Конфигуратор 8.5 (c1-item-elements.xml).
    #[test]
    fn add_elements_like_designer() {
        let mut f = form();
        for k in ["UsualGroupNone", "CommandBar", "Button", "Table", "PictureDecoration"] {
            add_item(&mut f, None, k, None).unwrap();
        }
        let bar = (max_id(&form()) + 3).to_string();
        for k in ["Button", "Popup", "ButtonGroup"] {
            add_item(&mut f, Some(&bar), k, None).unwrap();
        }
        assert!(add_item(&mut f, None, "Popup", None).is_err());
        // номера у Конфигуратора свои (shortcut в add_item), отступы новых узлов дает запись формы — сверка без них
        fn strip(e: &mut El) {
            e.attrs.retain(|a| a.0 != "id");
            e.kids.retain(|k| !matches!(k, Kid::Text(t) if t.trim().is_empty()));
            e.elems_mut().for_each(strip);
        }
        let mut want = dom::parse(include_str!("new/designer/c1-item-elements.xml")).unwrap();
        strip(&mut f);
        strip(&mut want);
        assert_eq!(f.write(), want.write());
    }
}

/// События вида элемента (тег XML; `Form` — сама форма): (имя в XML, русское имя, параметры обработчика).
pub fn events_of(tag: &str) -> &'static [(&'static str, &'static str, &'static str)] {
    super::form_props::EVENTS.iter().find(|(t, _)| *t == tag).map_or(&[], |(_, e)| e)
}

/// Элемент формы по id (`None` — сама форма).
fn item_mut<'a>(root: &'a mut El, id: Option<&str>) -> Option<&'a mut El> {
    fn walk<'a>(e: &'a mut El, id: &str) -> Option<&'a mut El> {
        if e.peek_attr("id") == Some(id) && !COLLECTIONS.contains(&e.name.as_str()) {
            return Some(e);
        }
        e.elems_mut().filter(|c| !COLLECTIONS.contains(&c.name.as_str())).find_map(|c| walk(c, id))
    }
    match id {
        None => Some(root),
        Some(id) => walk(root, id),
    }
}

/// Обработчик события `event` элемента `id` (`None` — формы): имя процедуры или нет.
pub fn handler(root: &El, id: Option<&str>, event: &str) -> Option<String> {
    fn walk<'a>(e: &'a El, id: &str) -> Option<&'a El> {
        if e.peek_attr("id") == Some(id) && !COLLECTIONS.contains(&e.name.as_str()) {
            return Some(e);
        }
        e.peek_elems().filter(|c| !COLLECTIONS.contains(&c.name.as_str())).find_map(|c| walk(c, id))
    }
    let e = match id {
        None => root,
        Some(id) => walk(root, id)?,
    };
    e.peek_child("Events")?.peek_elems().find(|v| v.peek_attr("name") == Some(event)).and_then(El::peek_text)
}

/// Назначить обработчик события (`None` — снять; процедура в модуле остается, как у Конфигуратора).
pub fn set_event(root: &mut El, id: Option<&str>, event: &str, proc: Option<&str>) -> Result<(), String> {
    let e = item_mut(root, id).ok_or("нет элемента")?;
    if e.peek_child("Events").is_none() {
        e.kids.push(Kid::El(El::new("Events", &[], Vec::new())));
    }
    let evs = e.elems_mut().find(|c| c.name == "Events").unwrap();
    evs.kids.retain(|k| !matches!(k, Kid::El(v) if v.peek_attr("name") == Some(event)));
    if let Some(p) = proc {
        evs.kids.push(Kid::El(El::new("Event", &[("name", event)], vec![Kid::Text(p.into())])));
    }
    if evs.kids.is_empty() {
        e.kids.retain(|k| !matches!(k, Kid::El(c) if c.name == "Events"));
    }
    Ok(())
}

/// Имя обработчика по умолчанию, как у Конфигуратора: событие формы — русское имя события («ПриОткрытии»), элемента —
/// имя элемента и событие («Поле1ПриИзменении»).
pub fn default_handler(root: &El, id: Option<&str>, event: &str) -> Option<String> {
    let tag = match id {
        None => "Form".to_string(),
        Some(i) => item_tag(root, i)?,
    };
    let &(_, ru, _) = events_of(&tag).iter().find(|e| e.0 == event)?;
    Some(match id {
        None => ru.to_string(),
        Some(i) => format!("{}{ru}", item_name(root, i)?),
    })
}

fn find_item<'a>(e: &'a El, id: &str) -> Option<&'a El> {
    if e.peek_attr("id") == Some(id) && !COLLECTIONS.contains(&e.name.as_str()) {
        return Some(e);
    }
    e.peek_elems().filter(|c| !COLLECTIONS.contains(&c.name.as_str())).find_map(|c| find_item(c, id))
}

/// Тег элемента формы по id.
pub fn item_tag(root: &El, id: &str) -> Option<String> {
    find_item(root, id).map(|e| e.name.clone())
}

fn item_name(root: &El, id: &str) -> Option<String> {
    find_item(root, id).and_then(|e| e.peek_attr("name")).map(str::to_string)
}

/// Есть ли в модуле процедура или функция с этим именем.
fn has_proc(module: &str, name: &str) -> bool {
    let low = name.to_lowercase();
    module.lines().any(|l| {
        let t = l.trim_start().to_lowercase();
        ["процедура ", "функция ", "procedure ", "function "].iter().any(|k| t.strip_prefix(k).is_some_and(|r| r.trim_start().starts_with(&low) && r.trim_start()[low.len()..].trim_start().starts_with('(')))
    })
}

/// Блок заготовки, как дописывает Конфигуратор 8.5 (эталон 28.10.2026): пустая строка, директива, процедура,
/// комментарий «Вставить содержимое обработчика.», КонецПроцедуры; CRLF, табуляция.
fn block(directive: &str, name: &str, params: &str, body: &str) -> String {
    format!("\r\n&{directive}\r\nПроцедура {name}({params})\r\n\t{body}\r\nКонецПроцедуры\r\n")
}

const STUB: &str = "// Вставить содержимое обработчика.";

/// Модуль с заготовкой обработчика события (если процедуры еще нет): директива — «НаСервере» у событий «…НаСервере»,
/// иначе «НаКлиенте»; у события элемента первый параметр — Элемент.
pub fn event_stub(module: &str, tag: &str, event: &str, proc: &str) -> Option<String> {
    if has_proc(module, proc) {
        return None;
    }
    let &(_, ru, params) = events_of(tag).iter().find(|e| e.0 == event)?;
    let params = if tag != "Form" && !params.starts_with("Элемент") {
        if params.is_empty() { "Элемент".to_string() } else { format!("Элемент, {params}") }
    } else {
        params.to_string()
    };
    let directive = if ru.ends_with("НаСервере") { "НаСервере" } else { "НаКлиенте" };
    Some(format!("{module}{}", block(directive, proc, &params, STUB)))
}

/// Как создать обработчик команды (окно «Создание обработчика команды», снимок 17).
#[derive(Clone, Copy, PartialEq)]
pub enum CommandHandler {
    Client,
    ClientAndServerNoContext,
    ClientAndServer,
}

/// Модуль с обработчиком команды `name` (если процедуры нет): на клиенте; или серверная процедура `<имя>НаСервере`
/// (без контекста) и вызов ее с клиента — серверная выше клиентской (эталон 28.10.2026).
pub fn command_stub(module: &str, name: &str, how: CommandHandler) -> Option<String> {
    if has_proc(module, name) {
        return None;
    }
    let server = format!("{name}НаСервере");
    Some(match how {
        CommandHandler::Client => format!("{module}{}", block("НаКлиенте", name, "Команда", STUB)),
        _ => {
            let dir = if how == CommandHandler::ClientAndServer { "НаСервере" } else { "НаСервереБезКонтекста" };
            format!("{module}{}{}", block(dir, &server, "", STUB), block("НаКлиенте", name, "Команда", &format!("{server}();")))
        }
    })
}

/// События формы для «Процедур и функций» модуля формы (txtedui EnumProc, эталон 29.10.2026 на форме внешней
/// обработки — 28 строк): таблица `form_props` без событий записи объекта (`object` — основной реквизит —
/// записываемый объект) и ВыборЗначения, и события 8.5, которых в синтакс-помощнике нет (оба без параметров).
pub fn module_events(object: bool) -> Vec<(&'static str, &'static str, &'static str)> {
    const WRITE: &[&str] = &["AfterWrite", "AfterWriteAtServer", "BeforeWrite", "BeforeWriteAtServer", "OnReadAtServer", "OnWriteAtServer"];
    const V85: &[(&str, &str, &str)] = &[("OnClientApplicationSuspend", "ПриЗасыпанииКлиентскогоПриложения", ""), ("OnClientApplicationResume", "ПриПробужденииКлиентскогоПриложения", "")];
    // shortcut: ВыборЗначения — у какого вида формы, не снято; в списке нет
    events_of("Form").iter().copied().filter(|e| e.0 != "ValueChoice" && (object || !WRITE.contains(&e.0))).chain(V85.iter().copied()).collect()
}

/// Модуль с заготовкой обработчика события формы из «Процедур и функций» (эталон 29.10.2026,
/// локальный синтетический эксперимент): событие «…НаСервере» — на сервере; остальные — на клиенте или с серверной
/// процедурой `<имя>НаСервере` выше клиентской, как у обработчика команды. Комментарий — «//Вставить содержимое
/// обработчика» (без пробела и точки, в отличие от заготовки из палитры).
pub fn module_event_stub(module: &str, ru: &str, params: &str, how: CommandHandler) -> String {
    const PROCS_STUB: &str = "//Вставить содержимое обработчика";
    if ru.ends_with("НаСервере") {
        return format!("{module}{}", block("НаСервере", ru, params, PROCS_STUB));
    }
    let server = format!("{ru}НаСервере");
    match how {
        CommandHandler::Client => format!("{module}{}", block("НаКлиенте", ru, params, PROCS_STUB)),
        _ => {
            let dir = if how == CommandHandler::ClientAndServer { "НаСервере" } else { "НаСервереБезКонтекста" };
            format!("{module}{}{}", block(dir, &server, "", PROCS_STUB), block("НаКлиенте", ru, params, &format!("{server}();")))
        }
    }
}

/// Действие команды формы (`<Action>`): процедура-обработчик.
pub fn set_action(root: &mut El, cmd: &str, proc: &str) -> Result<(), String> {
    let c = root.elems_mut().find(|c| c.name == "Commands").and_then(|cs| cs.elems_mut().find(|c| c.peek_attr("name") == Some(cmd))).ok_or(format!("нет команды {cmd}"))?;
    c.kids.retain(|k| !matches!(k, Kid::El(a) if a.name == "Action"));
    c.kids.push(Kid::El(El::leaf("Action", proc)));
    Ok(())
}

#[cfg(test)]
mod event_tests {
    use super::*;

    #[test]
    fn stubs_like_designer() {
        // эталон — модуль ФормаЭлемента1 из Конфигуратора 8.5 (команда — клиент и сервер, ПриСозданииНаСервере, ПриОткрытии)
        let m = command_stub("", "Команда1", CommandHandler::ClientAndServer).unwrap();
        let m = event_stub(&m, "Form", "OnCreateAtServer", "ПриСозданииНаСервере").unwrap();
        let m = event_stub(&m, "Form", "OnOpen", "ПриОткрытии").unwrap();
        let want = "\r\n&НаСервере\r\nПроцедура Команда1НаСервере()\r\n\t// Вставить содержимое обработчика.\r\nКонецПроцедуры\r\n\r\n&НаКлиенте\r\nПроцедура Команда1(Команда)\r\n\tКоманда1НаСервере();\r\nКонецПроцедуры\r\n\r\n&НаСервере\r\nПроцедура ПриСозданииНаСервере(Отказ, СтандартнаяОбработка)\r\n\t// Вставить содержимое обработчика.\r\nКонецПроцедуры\r\n\r\n&НаКлиенте\r\nПроцедура ПриОткрытии(Отказ)\r\n\t// Вставить содержимое обработчика.\r\nКонецПроцедуры\r\n";
        assert_eq!(m, want);
        assert!(event_stub(&m, "Form", "OnOpen", "ПриОткрытии").is_none());
        let f = event_stub("", "InputField", "OnChange", "Поле1ПриИзменении").unwrap();
        assert!(f.contains("&НаКлиенте\r\nПроцедура Поле1ПриИзменении(Элемент)\r\n"), "{f}");
        let mut r = dom::parse(include_str!("new/designer/c1-item.xml")).unwrap();
        assert_eq!(default_handler(&r, Some("1"), "OnChange").as_deref(), Some("КодПриИзменении"));
        set_event(&mut r, None, "OnOpen", Some("ПриОткрытии")).unwrap();
        set_event(&mut r, Some("1"), "OnChange", Some("КодПриИзменении")).unwrap();
        assert_eq!(handler(&r, Some("1"), "OnChange").as_deref(), Some("КодПриИзменении"));
        assert_eq!(handler(&r, None, "OnOpen").as_deref(), Some("ПриОткрытии"));
        set_event(&mut r, Some("1"), "OnChange", None).unwrap();
        assert!(handler(&r, Some("1"), "OnChange").is_none() && !r.write().contains("<InputField name=\"Код\" id=\"1\"><Events"));
    }
}
