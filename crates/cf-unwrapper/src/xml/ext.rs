//! Вспомогательные файлы Ext/*.xml: расписание регламентного задания, состав плана обмена,
//! командный интерфейс подсистем и конфигурации, рабочая область начальной страницы.
//! Устройство записей — docs/research/xml-ext.md.

use super::{Exporter, BOM};
use crate::brace::{self, Node};
use crate::meta::{Config, Item};
use std::collections::HashMap;
use std::path::Path;

const XMLNS: &str = "xmlns=\"http://v8.1c.ru/8.3/xcf/extrnprops\" xmlns:xr=\"http://v8.1c.ru/8.3/xcf/readable\" xmlns:xs=\"http://www.w3.org/2001/XMLSchema\" xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\" version=\"2.21\"";

/// uuid объекта или вложенного элемента → (полное имя `Catalog.Валюты.Command.Подбор`, вид, порядковый номер
/// в дереве: соседние элементы нумеруются в порядке описания владельца).
pub struct Names(HashMap<String, (String, &'static str, usize)>);

impl Names {
    pub fn new(cfg: &Config) -> Names {
        fn add(m: &mut HashMap<String, (String, &'static str, usize)>, it: &Item, full: String) {
            for ch in &it.children {
                add(m, ch, format!("{full}.{}.{}", ch.kind, ch.header.name));
            }
            let k = m.len();
            m.entry(it.header.id.clone()).or_insert((full, it.kind, k));
        }
        let mut m = HashMap::new();
        for g in &cfg.groups {
            for it in &g.items {
                add(&mut m, it, format!("{}.{}", g.class, it.header.name));
            }
        }
        Names(m)
    }

    pub fn get(&self, id: &str) -> Option<&(String, &'static str, usize)> {
        self.0.get(id)
    }

    fn name_or_id<'a>(&'a self, id: &'a str) -> &'a str {
        self.get(id).map_or(id, |(n, ..)| n.as_str())
    }
}

fn atom(n: &Node, i: usize) -> Result<&str, String> {
    n.get(i).and_then(Node::atom).ok_or_else(|| format!("нет числа в позиции {i}"))
}

fn num(n: &Node, i: usize) -> Result<usize, String> {
    atom(n, i)?.parse().map_err(|_| format!("не число в позиции {i}"))
}

fn doc(body: &str) -> Vec<u8> {
    [BOM, "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\r\n".as_bytes(), body.as_bytes()].concat()
}

fn unk(v: &str) -> String {
    format!("{}{v}", super::UNK)
}

/// Текст элемента XML; переводы строк платформа пишет как LF, хотя хранит CRLF.
pub fn esc(s: &str) -> String {
    s.replace("\r\n", "\n").replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

impl Exporter<'_> {
    /// Роли Ext/*.xml этого модуля; false — формат записи не распознан.
    pub(super) fn ext_file(&mut self, entry: &str, role: &str, ext: &Path) -> Result<bool, String> {
        self.ext_file_in(entry, role, ext).map_err(|e| format!("{entry} ({role}): {e}"))
    }

    fn ext_file_in(&mut self, entry: &str, role: &str, ext: &Path) -> Result<bool, String> {
        let Some(t) = self.cfg.entry_text(entry) else { return Ok(false) };
        let n = brace::parse(&t)?;
        let names = self.names.get_or_init(|| Names::new(self.cfg));
        let (file, data) = match role {
            "Schedule" => ("Schedule.xml", Some(schedule(&n)?)),
            "Content" => ("Content.xml", Some(content(&n, names)?)),
            "CommandInterface" => ("CommandInterface.xml", command_interface(self.cfg, &n, names)?),
            "MainSectionCommandInterface" => ("MainSectionCommandInterface.xml", command_interface(self.cfg, &n, names)?),
            "HomePageWorkArea" => ("HomePageWorkArea.xml", Some(home_page(&n, names)?)),
            "ClientApplicationInterface" => ("ClientApplicationInterface.xml", Some(client_interface(&n)?)),
            "Flowchart" => ("Flowchart.xml", Some(doc(&super::flowchart::flowchart(&n, &|id| names.get(id).map_or_else(|| (unk(id), usize::MAX), |x| (x.0.clone(), x.2)), &|v| super::common::design_time_ref(self.index, v))?))),
            "Predefined" if entry.ends_with(".9") => {
                let flag = |g: &str| names.get(g).map(|(n, _, k)| (n.clone(), *k));
                // Вид субконто — предопределенный ПВХ: ключ индекса `<ПВХ>.<id>`. Один id у двух ПВХ (ERP 2.5.22: ВидыСубконто
                // Хозрасчетные и Международные) — берется ПВХ видов субконто этого плана счетов (ExtDimensionTypes, n[1][19]);
                // иначе отказ.
                let own_cct = entry.split('.').next().and_then(|o| self.cfg.entry_text(o)).and_then(|t| brace::parse(&t).ok())
                    .and_then(|o| o.at(&[1, 19]).and_then(Node::atom).and_then(|g| self.index.full.get(g)).cloned());
                let kind = |id: &str| {
                    let m: Vec<_> = self.index.full.iter().filter(|(k, _)| k.starts_with("ChartOfCharacteristicTypes.") && k.ends_with(id) && k.len() > id.len() && k.as_bytes()[k.len() - id.len() - 1] == b'.').collect();
                    match m.as_slice() {
                        [(_, v)] => Some((*v).clone()),
                        _ => m.iter().filter(|(k, _)| own_cct.as_ref().is_some_and(|c| k.starts_with(&format!("{c}.")))).map(|(_, v)| (*v).clone()).next(),
                    }
                };
                ("Predefined.xml", super::predefined::accounts(&n, &flag, &kind)?.map(|b| doc(&b)))
            }
            "Predefined" if entry.ends_with(".7") =>("Predefined.xml", super::predefined::characteristic_types(&n, &self.index.types)?.map(|b| doc(&b))),
            "Predefined" if entry.ends_with(".2") =>("Predefined.xml", super::predefined::calc_types(&n)?.map(|b| doc(&b))),
            // Справочники (.1c) и планы видов расчета (.2); у планов видов характеристик и счетов нужен тип значения.
            "Predefined" if entry.ends_with(".1c") => {
                let obj = self.cfg.entry_text(entry.trim_end_matches(".1c")).map(|t| brace::parse(&t)).transpose()?;
                let code_len = obj.as_ref().and_then(|o| o.get(1)?.get(17)?.atom()?.parse().ok()).unwrap_or(0);
                ("Predefined.xml", super::predefined::catalog(&n, code_len)?.map(|b| doc(&b)))
            }
            _ => return Ok(false),
        };
        if let Some(d) = data {
            self.write(&ext.join(file), &d)?;
        }
        Ok(true)
    }
}

/// `{начало,конец,время начала,время конца,время завершения,интервал,повтор,пауза,N,дни недели...,
/// неделя месяца,день месяца,N,месяцы...,период недель,период дней,N,{детальные расписания}...}`.
pub fn schedule(n: &Node) -> Result<Vec<u8>, String> {
    let mut o = format!("<JobSchedule {}>\r\n", XMLNS.replace(" xmlns:xr=\"http://v8.1c.ru/8.3/xcf/readable\"", " xmlns:ent=\"http://v8.1c.ru/8.1/data/enterprise\""));
    schedule_el(n, "Schedule", 1, &mut o)?;
    o.push_str("</JobSchedule>");
    Ok(doc(&o))
}

fn schedule_el(n: &Node, tag: &str, depth: usize, o: &mut String) -> Result<(), String> {
    let tab = "\t".repeat(depth);
    let dt = |i: usize| -> Result<(String, String), String> {
        let a = atom(n, i)?;
        if a.len() != 14 || !a.bytes().all(|c| c.is_ascii_digit()) {
            return Err(format!("расписание: дата {a:?}"));
        }
        Ok((format!("{}-{}-{}", &a[0..4], &a[4..6], &a[6..8]), format!("{}:{}:{}", &a[8..10], &a[10..12], &a[12..14])))
    };
    let mut i = 8;
    let list = |i: &mut usize| -> Result<String, String> {
        let k = num(n, *i)?;
        let v: Result<Vec<&str>, String> = (1..=k).map(|j| atom(n, *i + j)).collect();
        *i += k + 1;
        Ok(v?.join(" "))
    };
    let week_days = list(&mut i)?;
    let (wdim, dim) = (atom(n, i)?, atom(n, i + 1)?);
    i += 2;
    let months = list(&mut i)?;
    o.push_str(&format!(
        "{tab}<{tag} BeginDate=\"{}\" EndDate=\"{}\" BeginTime=\"{}\" EndTime=\"{}\" CompletionTime=\"{}\" CompletionInterval=\"{}\" RepeatPeriodInDay=\"{}\" RepeatPause=\"{}\" WeekDayInMonth=\"{wdim}\" DayInMonth=\"{dim}\" WeeksPeriod=\"{}\" DaysRepeatPeriod=\"{}\">\r\n",
        dt(0)?.0, dt(1)?.0, dt(2)?.1, dt(3)?.1, dt(4)?.1, atom(n, 5)?, atom(n, 6)?, atom(n, 7)?, atom(n, i)?, atom(n, i + 1)?
    ));
    let el = |name: &str, v: &str| if v.is_empty() { format!("{tab}\t<ent:{name}/>\r\n") } else { format!("{tab}\t<ent:{name}>{v}</ent:{name}>\r\n") };
    o.push_str(&el("WeekDays", &week_days));
    o.push_str(&el("Months", &months));
    let details = num(n, i + 2)?;
    for k in 0..details {
        let d = n.get(i + 3 + k).ok_or("расписание: нет детального расписания")?;
        schedule_el(d, "ent:DetailedDailySchedules", depth + 1, o)?;
    }
    o.push_str(&format!("{tab}</{tag}>\r\n"));
    Ok(())
}

/// `{2,N,<uuid>,<авторегистрация>,...,0}` (у старых `{1,N,...}`); платформа сортирует по uuid.
pub fn content(n: &Node, names: &Names) -> Result<Vec<u8>, String> {
    let k = num(n, 1)?;
    let mut items = Vec::with_capacity(k);
    for j in 0..k {
        items.push((atom(n, 2 + 2 * j)?, atom(n, 3 + 2 * j)?));
    }
    items.sort();
    let mut o = format!("<ExchangePlanContent {XMLNS}");
    if items.is_empty() {
        o.push_str("/>");
        return Ok(doc(&o));
    }
    o.push_str(">\r\n");
    for (id, auto) in items {
        let auto = match auto {
            "0" => "Deny".into(),
            "1" => "Allow".into(),
            a => unk(a),
        };
        // удаленный объект в составе в эталоне не встречен — метка, а не uuid
        let md = names.get(id).map_or_else(|| unk(id), |n| n.0.clone());
        o.push_str(&format!("\t<Item>\r\n\t\t<Metadata>{md}</Metadata>\r\n\t\t<AutoRecord>{auto}</AutoRecord>\r\n\t</Item>\r\n"));
    }
    o.push_str("</ExchangePlanContent>");
    Ok(doc(&o))
}

/// Стандартные группы командного интерфейса.
fn group(id: &str, names: &Names) -> String {
    let std = match id {
        "77ea1b8f-dd79-4717-9dba-5628e7f348cf" => "NavigationPanelOrdinary",
        "1af6d528-0b86-4fba-ab95-bd7475db03ba" => "NavigationPanelImportant",
        "bc80566a-86a5-4e87-acd4-872239385a2e" => "NavigationPanelSeeAlso",
        "aabb34e1-98c1-4bd0-bf7f-243f95437b44" => "ActionsPanelTools",
        "4f499c31-050b-47c5-aa84-d0366c0a0da8" => "ActionsPanelCreate",
        "5b360bff-01a1-49b6-93d2-26e7e8e3a038" => "ActionsPanelReports",
        _ => return names.name_or_id(id).to_string(),
    };
    std.into()
}

/// Позиция флага «Использовать стандартные команды» в описании объекта (`n[1][i]`).
fn standard_commands_pos(class: &str) -> Option<usize> {
    Some(match class {
        "Catalog" => 31,
        "Document" => 23,
        "InformationRegister" => 21,
        "AccumulationRegister" => 16,
        "CalculationRegister" => 24,
        "ChartOfCharacteristicTypes" => 14,
        "ExchangePlan" => 13,
        "Report" | "Constant" => 7,
        "DataProcessor" | "DocumentJournal" => 5,
        "CommonForm" => 4,
        _ => return None,
    })
}

/// Команда `{вид,<uuid>}`: команда объекта или общая команда, иначе стандартная команда объекта
/// (вид 0 — открыть список/форму, 1 — создать, 2 — создать группу, 100 — открыть). Нераспознанное — `вид:uuid`, как у платформы
/// для объектов без стандартных команд и для удаленных.
fn command(cfg: &Config, r: &Node, names: &Names) -> Result<String, String> {
    // Пустая ссылка `{0}` выгружается как `0`.
    let kind = atom(r, 0)?;
    let Some(id) = r.get(1).and_then(Node::atom) else { return Ok(kind.into()) };
    let raw = format!("{kind}:{id}");
    let Some((full, class, _)) = names.get(id) else { return Ok(raw) };
    if matches!(*class, "Command" | "CommonCommand") {
        return Ok(if kind == "0" { full.clone() } else { raw });
    }
    // номера команд по классам — `form::STD_COMMANDS` (функции команд классов в DLL)
    let Some(std) = kind.parse().ok().and_then(|n| super::form::object_std_command(class, n)) else { return Ok(unk(&raw)) };
    let obj = cfg.entry_text(id).and_then(|t| brace::parse(&t).ok());
    if let Some(pos) = standard_commands_pos(class) {
        let on = obj.as_ref().and_then(|o| o.get(1).and_then(|p| p.get(pos)).and_then(Node::atom).map(|a| a == "1"));
        if on == Some(false) {
            return Ok(raw);
        }
    }
    // Справочник без владельцев (Owners — n[1][12]) команды OpenByValue не имеет: basic FUN_1801a0ae0. Описание не
    // прочитано — как без владельцев, так же, как `form::Names::owned`.
    if *class == "Catalog" && std == "OpenByValue" && !obj.as_ref().and_then(|o| o.at(&[1, 12])).is_some_and(|o| o.list().len() > 2) {
        return Ok(raw);
    }
    Ok(format!("{full}.StandardCommand.{std}"))
}

/// Видимость `{0,{0,{"B",общая},N,<роль>,{"B",значение},...}}` (отступ — как у `<Visibility>` в файлах командного интерфейса).
fn visibility(v: &Node, names: &Names, o: &mut String) -> Result<(), String> {
    let v = v.get(1).ok_or("видимость: нет значения")?;
    let b = |n: Option<&Node>| -> Result<&str, String> {
        match n.and_then(|x| x.get(1)).and_then(Node::atom) {
            Some("0") => Ok("false"),
            Some("1") => Ok("true"),
            x => Err(format!("видимость: {x:?}")),
        }
    };
    o.push_str(&format!("\t\t\t<Visibility>\r\n\t\t\t\t<xr:Common>{}</xr:Common>\r\n", b(v.get(1))?));
    for k in 0..num(v, 2)? {
        let role = atom(v, 3 + 2 * k)?;
        o.push_str(&format!("\t\t\t\t<xr:Value name=\"{}\">{}</xr:Value>\r\n", names.name_or_id(role), b(v.get(4 + 2 * k))?));
    }
    o.push_str("\t\t\t</Visibility>\r\n");
    Ok(())
}

/// `{7,<видимость>,<размещение>,<порядок команд>,<порядок подсистем>,<порядок групп>,0}`,
/// каждая часть — `0` или `1,N,элементы...`. Пустой интерфейс не выгружается.
pub fn command_interface(cfg: &Config, n: &Node, names: &Names) -> Result<Option<Vec<u8>>, String> {
    if atom(n, 0)? != "7" {
        return Err(format!("командный интерфейс: версия {:?}", n.get(0)));
    }
    let mut parts: Vec<Vec<&[Node]>> = Vec::new();
    let mut i = 1;
    // Шестая часть — видимость подсистем (подсистема, видимость); у записей без нее (БП 3.0.171) ее нет вовсе.
    for (k, width) in [2, 3, 2, 1, 1, 2].into_iter().enumerate() {
        let mut items = Vec::new();
        if k == 5 && i == n.list().len() {
            parts.push(items);
            continue;
        }
        if atom(n, i)? == "1" {
            let k = num(n, i + 1)?;
            i += 2;
            let l = n.list();
            for _ in 0..k {
                items.push(l.get(i..i + width).ok_or("командный интерфейс: обрыв")?);
                i += width;
            }
        } else {
            i += 1;
        }
        parts.push(items);
    }
    if parts.iter().all(Vec::is_empty) {
        return Ok(None);
    }
    if i != n.list().len() {
        return Err(format!("командный интерфейс: хвост записи с поля {i}"));
    }
    let mut o = format!("<CommandInterface {XMLNS}>\r\n");
    let cmd = |r: &Node| command(cfg, r, names);
    if !parts[5].is_empty() {
        o.push_str("\t<SubsystemsVisibility>\r\n");
        for it in &parts[5] {
            // Подсистема, которой нет в конфигурации, — по GUID (БП 3.0.107).
            o.push_str(&format!("\t\t<Subsystem name=\"{}\">\r\n", names.name_or_id(it[0].atom().unwrap_or("\u{1}"))));
            visibility(&it[1], names, &mut o)?;
            o.push_str("\t\t</Subsystem>\r\n");
        }
        o.push_str("\t</SubsystemsVisibility>\r\n");
    }
    if !parts[0].is_empty() {
        o.push_str("\t<CommandsVisibility>\r\n");
        for it in &parts[0] {
            o.push_str(&format!("\t\t<Command name=\"{}\">\r\n", cmd(&it[0])?));
            visibility(&it[1], names, &mut o)?;
            o.push_str("\t\t</Command>\r\n");
        }
        o.push_str("\t</CommandsVisibility>\r\n");
    }
    if !parts[1].is_empty() {
        o.push_str("\t<CommandsPlacement>\r\n");
        for it in &parts[1] {
            let place = match it[2].atom() {
                Some("0") => "Auto",
                Some("1") => "Manual",
                p => p.unwrap_or("\u{1}"),
            };
            let g = group(it[1].atom().unwrap_or("\u{1}"), names);
            o.push_str(&format!("\t\t<Command name=\"{}\">\r\n\t\t\t<CommandGroup>{g}</CommandGroup>\r\n\t\t\t<Placement>{place}</Placement>\r\n\t\t</Command>\r\n", cmd(&it[0])?));
        }
        o.push_str("\t</CommandsPlacement>\r\n");
    }
    if !parts[2].is_empty() {
        o.push_str("\t<CommandsOrder>\r\n");
        for it in &parts[2] {
            let g = group(it[0].atom().unwrap_or("\u{1}"), names);
            o.push_str(&format!("\t\t<Command name=\"{}\">\r\n\t\t\t<CommandGroup>{g}</CommandGroup>\r\n\t\t</Command>\r\n", cmd(&it[1])?));
        }
        o.push_str("\t</CommandsOrder>\r\n");
    }
    if !parts[3].is_empty() {
        o.push_str("\t<SubsystemsOrder>\r\n");
        for it in &parts[3] {
            match it[0].atom().unwrap_or("\u{1}") {
                // Пустая ссылка (нулевой uuid) — пустой элемент.
                "00000000-0000-0000-0000-000000000000" => o.push_str("\t\t<Subsystem/>\r\n"),
                s => o.push_str(&format!("\t\t<Subsystem>{}</Subsystem>\r\n", names.name_or_id(s))),
            }
        }
        o.push_str("\t</SubsystemsOrder>\r\n");
    }
    if !parts[4].is_empty() {
        o.push_str("\t<GroupsOrder>\r\n");
        for it in &parts[4] {
            o.push_str(&format!("\t\t<Group>{}</Group>\r\n", group(it[0].atom().unwrap_or("\u{1}"), names)));
        }
        o.push_str("\t</GroupsOrder>\r\n");
    }
    o.push_str("</CommandInterface>");
    Ok(Some(doc(&o)))
}

/// Шаблон рабочей области: номер — таблица backend.dll 0x18145b188 (конвертер FUN_1806a5a00).
pub(crate) const WORKING_AREA_TEMPLATES: &[(&str, &str)] = &[("0", "OneColumn"), ("1", "TwoColumnsEqualWidth"), ("2", "TwoColumnsVariableWidth")];

/// `{1,<шаблон>,N,<элементы левой колонки>...,M,<элементы правой>...,<MACommandInterfaceDisplays>}`,
/// элемент — `{0,{0,<форма>},<высота>,<видимость>}`. Свойства — тип HomePageWorkArea XDTO-модели backend.dll
/// (WorkingAreaTemplate, Column, LeftColumn, RightColumn, MACommandInterfaceDisplays: Top 0, Bottom 1, None 2);
/// у шаблона OneColumn одна колонка `Column`. MACommandInterfaceDisplays = None в XML не пишется.
pub fn home_page(n: &Node, names: &Names) -> Result<Vec<u8>, String> {
    let t = atom(n, 1)?;
    let template = WORKING_AREA_TEMPLATES.iter().find(|(k, _)| *k == t).map_or_else(|| unk(t), |(_, v)| v.to_string());
    let mut o = format!("<HomePageWorkArea {XMLNS}>\r\n\t<WorkingAreaTemplate>{template}</WorkingAreaTemplate>\r\n");
    let mut i = 2;
    let cols = if t == "0" { ["Column", "\u{1}RightColumn"] } else { ["LeftColumn", "RightColumn"] };
    for col in cols {
        let k = num(n, i)?;
        i += 1;
        if k == 0 {
            continue;
        }
        o.push_str(&format!("\t<{col}>\r\n"));
        for _ in 0..k {
            let it = n.get(i).ok_or("начальная страница: обрыв")?;
            i += 1;
            let form = it.get(1).and_then(|f| f.get(1)).and_then(Node::atom).ok_or("начальная страница: нет формы")?;
            o.push_str(&format!("\t\t<Item>\r\n\t\t\t<Form>{}</Form>\r\n\t\t\t<Height>{}</Height>\r\n", names.name_or_id(form), atom(it, 2)?));
            visibility(it.get(3).ok_or("начальная страница: нет видимости")?, names, &mut o)?;
            o.push_str("\t\t</Item>\r\n");
        }
        o.push_str(&format!("\t</{col}>\r\n"));
    }
    if atom(n, i)? != "2" {
        o.push_str(&format!("\t<MACommandInterfaceDisplays>{}</MACommandInterfaceDisplays>\r\n", unk(atom(n, i)?)));
    }
    o.push_str("</HomePageWorkArea>");
    Ok(doc(&o))
}

/// Расположение панелей клиентского приложения (mngcore чтение FUN_18059f000, XML FUN_180041cd0):
/// `{версия,{0,1,<верх>},{0,2,<низ>},{0,3,<лево>},{0,4,<право>},(признак,{панель})…,0}`; в XML стороны — top, left, right, bottom.
pub fn client_interface(n: &Node) -> Result<Vec<u8>, String> {
    let mut o = String::from("<ClientApplicationInterface xmlns=\"http://v8.1c.ru/8.2/managed-application/core\" xmlns:xs=\"http://www.w3.org/2001/XMLSchema\" xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\" xsi:type=\"InterfaceLayouter\">\r\n");
    let mut i = 5;
    for (k, side) in [(1, "top"), (3, "left"), (4, "right"), (2, "bottom")] {
        let s = n.get(k).ok_or("интерфейс клиента: нет стороны")?;
        let body = s.get(2).ok_or("интерфейс клиента: нет стороны")?;
        // каждый элемент стороны — свой `<side>` (mngcore FUN_180041c20)
        for j in 0..num(body, 1)? {
            let (ty, b) = (atom(body, 2 + 2 * j)?, body.get(3 + 2 * j).ok_or("интерфейс клиента: обрыв")?);
            o.push_str(&format!("\t<{side}>\r\n"));
            layout_group(&Node::List(vec![Node::Atom("0".into()), Node::Atom("1".into()), Node::Atom(ty.into()), b.clone()]), 2, &mut o)?;
            o.push_str(&format!("\t</{side}>\r\n"));
        }
    }
    // Описания панелей (FUN_1800422c0): признак 1 — своя панель, 2 — стандартная; `{uuid,флаги,[&1 имя],[&2 spr]}`;
    // name — только у своей и непустое, spr — только у панели разделов b553047f… и не 1 (умолчание)
    while matches!(atom(n, i)?, "1" | "2") {
        let own = atom(n, i)? == "1";
        let b = n.get(i + 1).ok_or("интерфейс клиента: нет панели")?;
        let p = atom(b, 0)?;
        let flags = num(b, 1)?;
        let mut k = 2;
        let mut attrs = format!(" id=\"{p}\"");
        if flags & 1 != 0 {
            let name = b.get(k).and_then(Node::str).ok_or("интерфейс клиента: имя панели")?;
            if own && !name.is_empty() {
                attrs += &format!(" name=\"{}\"", name.replace('&', "&amp;").replace('<', "&lt;").replace('"', "&quot;"));
            }
            k += 1;
        }
        if flags & 2 != 0 && p == "b553047f-c9aa-4157-978d-448ecad24248" && num(b, k)? != 1 {
            return Err(format!("интерфейс клиента: spr {} не сверен", atom(b, k)?));
        }
        o.push_str(&format!("\t<panelDef{attrs}/>\r\n"));
        i += 2;
    }
    if n.list().get(i..).is_none_or(|t| t.iter().map(Node::atom).ne([Some("0")])) {
        return Err(format!("интерфейс клиента: незнакомый хвост записи с позиции {i}"));
    }
    o.push_str("</ClientApplicationInterface>");
    Ok(doc(&o))
}

/// Группа `{0,N,<тип>,{...}}`: f1d2a809… — группа `{0,<id>,0,{дочерние}}`, 8b252cb5… — панель `{0,<id>,<панель>,2,<высота>}`.
fn layout_group(g: &Node, depth: usize, o: &mut String) -> Result<(), String> {
    let tab = "\t".repeat(depth);
    let k = num(g, 1)?;
    for j in 0..k {
        let (ty, body) = (atom(g, 2 + 2 * j)?, g.get(3 + 2 * j).ok_or("интерфейс клиента: обрыв")?);
        let id = atom(body, 1)?;
        match ty {
            "f1d2a809-46ac-46ab-8b37-b2135ac5b323" => {
                o.push_str(&format!("{tab}<group id=\"{id}\">\r\n"));
                let inner = body.get(3).ok_or("интерфейс клиента: нет состава группы")?;
                for m in 0..num(inner, 1)? {
                    let (t2, b2) = (atom(inner, 2 + 2 * m)?, inner.get(3 + 2 * m).ok_or("интерфейс клиента: обрыв")?);
                    o.push_str(&format!("{tab}\t<group>\r\n"));
                    layout_group(&Node::List(vec![Node::Atom("0".into()), Node::Atom("1".into()), Node::Atom(t2.into()), b2.clone()]), depth + 2, o)?;
                    o.push_str(&format!("{tab}\t</group>\r\n"));
                }
                o.push_str(&format!("{tab}</group>\r\n"));
            }
            "8b252cb5-d895-4c40-8ccb-581ce795fb80" => {
                o.push_str(&format!("{tab}<panel id=\"{id}\">\r\n{tab}\t<uuid>{}</uuid>\r\n", atom(body, 2)?));
                if atom(body, 4)? != "0" {
                    o.push_str(&format!("{tab}\t<height>{}</height>\r\n", atom(body, 4)?));
                }
                o.push_str(&format!("{tab}</panel>\r\n"));
            }
            t => return Err(format!("интерфейс клиента: элемент {t}")),
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(b: Vec<u8>) -> String {
        String::from_utf8(b[3..].to_vec()).unwrap()
    }

    #[test]
    fn schedule_with_details() {
        let n = brace::parse("{00010101000000,00010101000000,00010101010000,00010101000000,00010101000000,0,0,0,2,6,7,0,5,0,1,1,1,\r\n{00010101000000,00010101000000,00010101140000,00010101000000,00010101000000,0,0,0,0,0,0,0,1,0,0}\r\n}").unwrap();
        let x = s(schedule(&n).unwrap());
        assert!(x.contains("<Schedule BeginDate=\"0001-01-01\" EndDate=\"0001-01-01\" BeginTime=\"01:00:00\" EndTime=\"00:00:00\" CompletionTime=\"00:00:00\" CompletionInterval=\"0\" RepeatPeriodInDay=\"0\" RepeatPause=\"0\" WeekDayInMonth=\"0\" DayInMonth=\"5\" WeeksPeriod=\"1\" DaysRepeatPeriod=\"1\">\r\n\t\t<ent:WeekDays>6 7</ent:WeekDays>\r\n\t\t<ent:Months/>\r\n\t\t<ent:DetailedDailySchedules BeginDate"), "{x}");
        assert!(x.contains("BeginTime=\"14:00:00\""), "{x}");
        assert!(x.ends_with("\t\t</ent:DetailedDailySchedules>\r\n\t</Schedule>\r\n</JobSchedule>"), "{x}");
    }

    #[test]
    fn client_interface_tail() {
        let n = |tail: &str| brace::parse(&format!("{{1,{{0,1,{{0,0}}}},{{0,2,{{0,0}}}},{{0,3,{{0,0}}}},{{0,4,{{0,0}}}},2,{{p,0}},{tail}}}")).unwrap();
        assert!(s(client_interface(&n("0")).unwrap()).contains("\t<panelDef id=\"p\"/>\r\n</ClientApplicationInterface>"));
        // незнакомый хвост вместо завершающего 0 — отказ, а не молча обрезанный файл
        assert!(client_interface(&n("3,{q,0},0")).is_err());
    }

    #[test]
    fn content_sorted_by_uuid() {
        let names = Names(HashMap::from([("bb".to_string(), ("Catalog.Б".to_string(), "Catalog", 0))]));
        let x = s(content(&brace::parse("{2,2,bb,1,aa,0,0}").unwrap(), &names).unwrap());
        assert!(x.contains("<Metadata>\u{1}aa</Metadata>\r\n\t\t<AutoRecord>Deny</AutoRecord>\r\n\t</Item>\r\n\t<Item>\r\n\t\t<Metadata>Catalog.Б</Metadata>\r\n\t\t<AutoRecord>Allow</AutoRecord>"), "{x}");
        assert!(s(content(&brace::parse("{1,0}").unwrap(), &names).unwrap()).ends_with("version=\"2.21\"/>"));
    }
}
