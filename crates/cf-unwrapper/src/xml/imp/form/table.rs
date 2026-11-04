//! Таблица формы (узел версии 73), ее дополнения (строка поиска, состояние просмотра, управление поиском) и группа
//! колонок. Раскладки сняты со сверки XML ↔ записи загрузки БП (5 170 таблиц, 2 488 групп колонок): поле — постоянное
//! или функция одного свойства XML; узлы, которые платформа добавляет сама (панель действий выделенных строк, действия
//! строки и дополнения панели), — от имени и id таблицы.
//!
//! Словарь таблицы `N,(<ключ>,<значение>)…`: 13 — отбор строк (`<RowFilter xsi:nil="true"/>` → `{"U"}`), 19 `{"S",""}` —
//! в XML его нет. Ключ 19 платформа пишет, когда при загрузке обращается к расширению таблицы до конца разбора формы:
//! у таблицы есть события или исключенные команды, или кнопка формы ссылается на ее команду (`Form.Item.<таблица>.…`).
//! У набора записей регистра и таблицы без пути к данным его нет никогда. Сверка: 2 585 из 2 585 таблиц коллекций с
//! ключом 19 и 381 из 382 без (исключение — таблица без пути к данным); 8.5 — так же.
//! По коду: словарь — слоты расширения таблицы (mngcore FUN_180397f80 пишет map модели как есть); расширение коллекции
//! объявляет один слот 0x13 со значением "" (mngbase FUN_1803b1ae0, FUN_1803b1af0), форма копирует его в модель при
//! прикреплении расширения (mngbase FUN_1802c1180 → FUN_1802c1660). Когда прикрепляет при загрузке — правило сверки выше.

use super::items::*;
use super::*;

pub(super) const TABLE: &str = "143c00f7-a42d-4cd7-9189-88e4467dc768";
/// Дополнение таблицы, вынесенное в другую группу формы.
pub(super) const ADDITION: &str = "c5259a1d-518a-4afd-b98d-0176027e4feb";

/// Стандартная команда таблицы → GUID (сверка исключенных команд таблиц БП: у всех имен один GUID на вид таблицы).
/// `dl` — таблица динамического списка, `sc` — таблица настроек компоновщика.
pub(super) fn table_command(name: &str, dl: bool, sc: bool) -> Option<&'static str> {
    Some(match name {
        "Add" if !dl => "b0016a68-ec64-4e6d-b905-c71fd62efc4c",
        "AddFilterItem" if sc => "fca750bc-4fb6-40e2-ae0f-e818939a32e7",
        "AddFilterItemGroup" if sc => "a5fdef31-bbf0-4a9d-98aa-fd5fd8f1344a",
        "AddOrderItem" if sc => "62ff963c-9426-43af-bb23-1d2ef3a9a0c1",
        // AddAutoOrderItem и Expand — только парой (см. исключенные команды таблицы)
        "AddAutoOrderItem" if sc => "48e12019-0fd6-46eb-aab6-2acba716a623",
        "Expand" if sc => "fc120c02-7f39-469b-b357-b2dd8d4b0765",
        "CancelSearch" => "44ad3ec9-f3c2-4913-9224-5f9fb6418743",
        "Change" => "b41f5bbc-ba5d-4888-8cd1-db246a371418",
        "ChangeHistory" if dl => "11761e12-cf32-4826-a175-b23213e3b229",
        "CheckAll" if !dl => "18248aa8-e621-4e19-a611-54fb8923644c",
        "Choose" if sc => "d77e5787-b130-4355-8f8f-01ecec82f843",
        "Choose" => "8969c93a-23e5-4bef-941d-aaef315858d2",
        "ChooseAll" if sc => "15664824-eedc-4a92-9f6b-c89a2dead157",
        "Copy" => "0ae4bea5-23be-42a7-b69e-97b11b29c453",
        "CopyToClipboard" => "88078230-1f6b-415f-99e4-ad2ff73810cf",
        "Create" if dl => "0f8d6d98-2f8b-405a-b8b3-0538e9d95da5",
        "CreateByParameter" if dl => "b59f3c87-e213-4947-abae-9dbaffaef147",
        "CreateFolder" if dl => "d82ca05c-2966-4d77-9a39-a1eea087bfa7",
        "Delete" if !dl => "8d772f97-c0ef-47c0-9cb0-efea28c61341",
        "Detailed" if sc => "e6900951-1a42-4397-bf00-cabb2cd7ad6d",
        "DynamicListStandardSettings" if dl => "33b7b9cd-6979-4435-8c58-d9bc8250edec",
        "EndEdit" if !dl => "9ef79140-3de6-436a-8dda-610bb963f5db",
        "Find" => "c0519548-2a9a-44de-a25e-faf01e089d4d",
        "FindByCurrentValue" => "714d44cc-63da-4431-b33a-428e398d2a08",
        "GroupFilterItems" if sc => "4a817da0-5797-4e16-906f-02fb869e1873",
        "HierarchicalList" => "01833a5a-6553-4c49-b445-095018107bb5",
        "LevelDown" if dl => "dc118d99-b351-4e30-9310-e864f2e53ec0",
        "LevelUp" if dl => "0e9b637d-cf6e-4330-8a8f-cd44842e34bb",
        "List" => "0d0249a4-2b2f-4fc0-a66f-b36f9494b3cc",
        "ListSettings" if dl => "14559f7c-853c-42a4-9ea1-01546107747b",
        "LoadDynamicListSettings" if dl => "182a793b-22a5-4625-b316-6a5be7f88078",
        "MoveDown" if !dl => "fa51b106-eae6-44c7-8054-76cbb3100603",
        "MoveItem" if dl => "27bd521a-51c6-4fe7-846d-a98f988774b5",
        "MoveUp" if !dl => "37740564-9e86-44a0-bea9-3f485a5a3f91",
        "OutputList" if dl => "825c1c15-ef8f-47ab-b002-e6b84b3e5b10",
        "OutputList" => "49602716-fea6-497f-8047-726404038857",
        "Pickup" if !dl => "59b4387d-f5be-4658-901f-bd3068217469",
        "Post" if dl => "e3dd8850-fc3c-41b1-bbb3-7c66af082608",
        "Refresh" if dl => "403bc6e6-b98e-4181-9f43-9c75cbbf82cf",
        "SaveDynamicListSettings" if dl => "95b4bc12-2ece-4d7a-b3e2-6f9293620a06",
        "SearchEverywhere" => "7b683784-b474-441a-ba63-3d757bd0ffd4",
        "SearchHistory" => "d96b0c03-b209-4d01-a3fc-17a14f873b64",
        "SelectAll" if !dl => "51c99108-107c-43e1-8918-e48835bf2495",
        "SetDateInterval" if dl => "daa306cd-a78a-4e74-a14c-739daba624cb",
        "SetDeletionMark" if dl => "a2f737a8-0114-4e86-a214-45e5c213fa65",
        "SetPresentation" if sc => "7d4db5ed-0981-4020-b3b8-886b7165ba05",
        "ShowMultipleSelection" => "e7216412-03ac-4a81-99c2-1d7c28e88e31",
        "ShowRowRearrangement" if !dl => "8af6ebff-cd02-4bfe-a984-44a292623708",
        "SortListAsc" if !dl => "2bbe4e12-06d2-409b-a972-eea585125d83",
        "SortListDesc" if !dl => "58b2a785-23f6-4b0e-a324-9a1323285595",
        "Tree" => "05468165-f954-45a5-84f2-6641c51f9f23",
        "UncheckAll" if !dl => "5048cc44-702b-44e3-8445-9af75c02724d",
        "UndoPosting" if dl => "04ac7211-e74f-4776-9749-35a9282b1d52",
        "Ungroup" if sc => "82b88a24-2856-484a-afd9-55a15bdf9785",
        "UseFieldAsValue" if sc => "d7e55d2e-bfea-4d80-b4ad-a1bb31ec2147",
        "UserSettingItemProperties" if sc => "1f1e900a-8488-4159-81be-9704eb96906d",
        _ => return None,
    })
}

/// Вид таблицы по пути к данным: от него зависят словарь и команды.
#[derive(PartialEq, Clone, Copy)]
enum Kind {
    /// без пути к данным
    NoPath,
    /// коллекция значений, табличная часть
    Coll,
    /// набор записей регистра
    RecordSet,
    /// динамический список
    List,
    /// настройки компоновщика
    Settings,
}

/// Узел элемента без id: `{<id>,02023637…}` → `{0}` на любой глубине (узлы, которые платформа создает сама).
/// Таблица основного реквизита — динамического списка, к расширению которой обращается сборка списка стандартных команд
/// при загрузке. Отложенные задачи разрешения имен команд — исключенные команды элемента (mngbase FUN_1807107e0 →
/// FUN_1802e63f0) и команда кнопки / элемента командного интерфейса (FUN_180760ea0 из читателей FUN_180730850 свойство
/// 0x10 и FUN_1807393f0 свойство 0 → задача FUN_180762090 → FUN_180761100 → FUN_1802e6a30: `Form.StandardCommand.X` —
/// команды формы, `Form.Item.<элемент>.….X` — команды элемента) — перебирают команды элемента (FUN_1802e61f0 /
/// FUN_1802e63f0 → FUN_1802fc810 → FUN_1802fcdd0). У расширения динамического списка счет команд FUN_18042ff10 создает
/// поставщик команд (FUN_180430410 → FUN_1804302f0 → FUN_1806c9410), тот спрашивает таблицу у менеджера расширений
/// (482fffe4 +0x18, FUN_1802e9e90): реквизит владельца менеджера, первая таблица с путем к нему (форма +0x200) —
/// прикрепляет ее расширение (FUN_1802e88f0 → FUN_1802c1660), затем вторую — элемент с id из свойства 1 словаря формы
/// (`owner+0x20()` — форма, +0x328/+0x330(1), FUN_1802ea240), то есть `GroupList` (трасса БП: ВидыОтправляемыхДокументов —
/// Список и Дерево). Владелец менеджера — форма: +0x38 — id основного реквизита (вход form+0x200 в трассе — его путь).
/// При нескольких таблицах основного реквизита порядок формы +0x200 (mngcore, модель +0x30) не выведен — пусто; `GroupList`
/// не таблица — пусто (прикрепится ли, не выведено).
pub(super) fn main_list_table(doc: &El, form_cs: bool, main: Option<&str>, items: &HashMap<String, Item>, referenced: &std::collections::HashSet<String>) -> Vec<String> {
    fn walk<'e>(e: &'e El, cmd: &mut bool, tables: &mut Vec<&'e El>) {
        for c in e.peek_elems() {
            if matches!(c.name.as_str(), "CommandName" | "Command") && c.peek_text().is_some_and(|t| t.starts_with("Form.StandardCommand.")) {
                *cmd = true;
            }
            if c.name == "Table" {
                tables.push(c);
            }
            walk(c, cmd, tables);
        }
    }
    let Some(main) = main else { return vec![] };
    let (mut std_cmd, mut tables) = (false, Vec::new());
    walk(doc, &mut std_cmd, &mut tables);
    // перебор команд расширения динамического списка: команды формы или команды таблицы списка с исключенными
    // командами / ссылкой кнопки
    let list_cmds = items.iter().any(|(n, it)| it.dynamic_list && (referenced.contains(n) || tables.iter().any(|t| t.peek_attr("name") == Some(n.as_str()) && t.peek_child("CommandSet").is_some())));
    if !(form_cs || std_cmd || list_cmds) {
        return vec![];
    }
    let own: Vec<_> = tables.iter().filter(|t| t.peek_child("DataPath").and_then(El::peek_text).as_deref() == Some(main)).collect();
    let [t] = own[..] else { return vec![] };
    let mut v: Vec<String> = t.peek_attr("name").map(str::to_string).into_iter().collect();
    if let Some(g) = doc.peek_child("GroupList").and_then(El::peek_text) {
        if !tables.iter().any(|t| t.peek_attr("name") == Some(g.as_str())) {
            return vec![];
        }
        v.push(g);
    }
    v
}

fn no_ids(n: &mut Node) {
    if let Node::List(v) = n {
        if v.len() == 2 && v[1].atom() == Some(ELEM) && v[0].atom().is_some() {
            *n = l(vec![a("0")]);
            return;
        }
        v.iter_mut().for_each(no_ids);
    }
}

impl F<'_> {
    pub(super) fn settings_table(&self, dp: Option<&str>) -> bool {
        self.table_kind(dp) == Kind::Settings
    }

    /// GUID команды Delete таблицы динамического списка реквизита `attr` — та из двух, что есть в составе команд таблицы
    /// (mngbase FUN_180611300, слот +0x58 поставщика команд таблицы списка, vtable 0x18088cf20; ветви взаимоисключающие):
    /// ec576e13 (вместе с SetDeletionMark) — основная таблица журнал документов (класс 4612bd75), объект с пометкой
    /// удаления (интерфейс ede4bd22, +0x40 — реквизит DeletionMark, ср. FUN_18063bb50) или задача; 8d772f97 — регистр
    /// сведений независимый и не по позиции регистратора (FUN_18060d500: класс 13134201, +0x248 == 0, +0x238 != 6).
    /// Основная таблица — из настроек и при своем запросе (FUN_18060cfa0 берет ее по id). shortcut: прочие регистры
    /// (интерфейс 2b98afb7, +0x28) и список без основной таблицы (в составе нет Delete) — отказ, пока не разобраны
    fn list_delete(&self, attr: &str) -> Option<&'static str> {
        let mt = self.attrs.get(attr)?.main_table.as_ref()?;
        match mt.split('.').next()? {
            "DocumentJournal" | "Catalog" | "Document" | "ChartOfCharacteristicTypes" | "ChartOfAccounts" | "ChartOfCalculationTypes" | "ExchangePlan"
            | "BusinessProcess" | "Task" => Some("ec576e13-1e76-4c33-98aa-a33204514227"),
            "InformationRegister" => {
                let p = &self.x.objects.get(mt)?.props;
                let p = |k: &str| p.get(k).map(String::as_str);
                (p("WriteMode") == Some("Independent") && p("InformationRegisterPeriodicity") != Some("RecorderPosition"))
                    .then_some("8d772f97-c0ef-47c0-9cb0-efea28c61341")
            }
            _ => None,
        }
    }

    fn table_kind(&self, dp: Option<&str>) -> Kind {
        let Some(dp) = dp else { return Kind::NoPath };
        // путь через текущие данные таблицы компоновщика: mngbase FUN_1802c8a30 подставляет путь той таблицы, тип
        // части — поле компоновщика (dcscore), расширение — по нему (`settings_dict`)
        if let Some((t, _)) = dp.strip_prefix("Items.").and_then(|r| r.split_once(".CurrentData.")) {
            if self.items.get(t).is_some_and(|it| it.tag == "Table" && self.settings_table(Some(&it.data_path))) {
                return Kind::Settings;
            }
        }
        let root = dp.split('.').next().unwrap_or_default();
        let class = self.attrs.get(root).map(|a| a.class.as_str()).unwrap_or_default();
        // отбор и порядок самого динамического списка (`Список.Filter`, −2 / −1 пути) — настройки компоновки
        if dp.contains("SettingsComposer") || class.contains("SettingsComposer") || (class == "DynamicList" && matches!(dp.strip_prefix(root), Some(".Filter" | ".Order"))) {
            Kind::Settings
        } else if class == "DynamicList" && dp == root {
            Kind::List
        } else if class.ends_with("RecordSet") && dp == root {
            Kind::RecordSet
        } else {
            Kind::Coll
        }
    }

    /// Путь `{K,<часть>…}` от пути таблицы (картинка строк).
    fn rel_path(&self, table: &str, e: &El) -> R<Node> {
        let t = e.text()?;
        if !t.trim_start_matches('~').starts_with(&format!("{table}.")) {
            return Err(err(e, "путь картинки строк не от пути таблицы"));
        }
        let full = self.path_text(&t).map_err(|m| err(e, &m))?;
        let base = self.path_text(table).map_err(|m| err(e, &m))?;
        let k = base.list().len() - 1;
        let mut v = vec![an(full.list().len() - 1 - k)];
        v.extend(full.list()[1 + k..].iter().cloned());
        Ok(l(v))
    }

    /// Таблица (версия 73).
    pub(super) fn table(&self, e: &El) -> R<Node> {
        let p = Props::new(e)?;
        let id = e.req_attr("id")?;
        let name = e.req_attr("name")?;
        let dp = p.get("DataPath").map(txt).transpose()?;
        let kind = self.table_kind(dp.as_deref());
        // нет RowSelectionMode — «не задано» формы конструктора 8.5: хвост 17 = 2 (`form_v59::UNSET_ROW_SELECTION`)
        let (rsm, rsm_tail) = match p.get("RowSelectionMode") {
            Some(r) => {
                let t = token(r, &[("Cell", 0), ("Row", 1)])?;
                (t.clone(), t)
            }
            None => (a("0"), a("2")),
        };
        // ShowCommandBar пишет только форма конструктора 8.5 (хвост 11 = 1, `form_v59::SHOW_COMMAND_BAR`); значение —
        // по CommandBarLocation: None — false, по умолчанию — auto, Top/Bottom — true
        let scb = match p.get("ShowCommandBar").map(txt).transpose()? {
            None => false,
            Some(v) => {
                let cbl = p.get("CommandBarLocation").map(txt).transpose()?;
                let want = match cbl.as_deref() {
                    Some("None") => "false",
                    None | Some("Auto") => "auto",
                    _ => "true",
                };
                if v != want {
                    return Err(err(e, "ShowCommandBar не как у выгрузки"));
                }
                true
            }
        };
        let data_path = match p.get("DataPath") {
            Some(d) => self.data_path(d)?,
            None => l(vec![a("0")]),
        };
        let row_pic = match (p.get("RowPictureDataPath"), dp.as_deref()) {
            (None, _) => l(vec![a("0")]),
            (Some(r), Some(t)) => self.rel_path(t, r)?,
            (Some(r), None) => return Err(err(r, "картинка строк без пути таблицы")),
        };
        let mut v = vec![a("73"), l(vec![a(id), a(ELEM)]), a("0"), p.tok("Representation", &[("List", 0), ("Tree", 2)], 1)?];
        match p.get("UserVisible") {
            Some(u) => v.extend([a("1"), adjustable(self.x, Some(u))?]),
            None => v.push(a("0")),
        }
        v.extend([
            s(name),
            p.tok("TitleLocation", &[("None", 0), ("Auto", 1), ("Left", 2), ("Top", 3), ("Right", 4), ("Bottom", 5)], 0)?,
            p.num("TitleHeight", "0")?,
            p.tok("CommandBarLocation", &[("None", 0), ("Top", 2), ("Bottom", 3)], 1)?,
            p.ml("Title")?,
            p.ml("ToolTip")?,
            data_path,
            p.b("Autofill", "0")?,
            p.b("Enabled", "1")?,
            p.b("ReadOnly", "0")?,
            a("0"),
            p.b("DefaultItem", "0")?,
            p.b("ChangeRowSet", "1")?,
            p.b("ChangeRowOrder", "1")?,
            p.num("Width", "0")?,
            p.num("Height", "0")?,
            p.num("HeightInTableRows", "0")?,
            p.b("ChoiceMode", "0")?,
            p.tok("RowInputMode", &[("EndOfWindow", 1), ("AfterCurrentRow", 2), ("BeforeCurrentRow", 3)], 0)?,
            p.tok("SelectionMode", &[("SingleRow", 0), ("MultiRow", 1)], 1)?,
            rsm,
            p.b("Header", "1")?,
            p.num("HeaderHeight", "1")?,
            p.b("Footer", "0")?,
            p.num("FooterHeight", "1")?,
            p.tok("HorizontalScrollBar", &[("DontUse", 0), ("UseAlways", 1)], 2)?,
            p.tok("VerticalScrollBar", &[("DontUse", 0), ("UseAlways", 1)], 2)?,
            p.b("HorizontalLinesBWA", "1")?,
            a("0"),
            a("0"),
            a("0"),
            p.b("UseAlternationRowColorBWA", "1")?,
            p.b("AutoInsertNewRow", "0")?,
            p.tok("InitialListView", &[("Beginning", 0), ("End", 1)], 2)?,
            p.tok("InitialTreeView", &[("ExpandTopLevel", 1), ("ExpandAllLevels", 2)], 0)?,
            p.tok("Output", &[("Enable", 1), ("Disable", 2)], 0)?,
            p.b("HorizontalStretch", "1")?,
            p.b("VerticalStretch", "1")?,
            row_pic,
            self.pic(e, &p, "RowsPicture")?,
            p.color(self.x, "BackColor")?,
            p.color(self.x, "TextColor")?,
            p.color(self.x, "BorderColor")?,
            p.font(self.x, "Font")?,
            p.color(self.x, "TitleTextColor")?,
            p.font(self.x, "TitleFont")?,
            shortcut(p.get("Shortcut"))?,
            p.b("EnableStartDrag", "0")?,
            p.b("EnableDrag", "0")?,
        ]);
        // словарь
        let events_el = p.get("Events");
        let cs = p.get("CommandSet");
        let touched = events_el.is_some_and(|x| x.peek_elems().next().is_some()) || cs.is_some() || self.referenced.contains(name);
        // свойство 0x13 (`{"S",""}`) объявляют расширения табличной части, таблицы значений, дерева и списка значений
        // (mngbase: классы universalCollection B/C, universalTree, valueList); набору записей регистра такое расширение не
        // подходит — ключа нет (движения документа `Объект.RegisterRecords.<регистр>`)
        let record_set = dp.as_deref().is_some_and(|d| d.split('.').nth(1) == Some("RegisterRecords"));
        let full = matches!(kind, Kind::Coll) && touched && !record_set;
        let mut dict: Vec<(&str, Node)> = Vec::new();
        if kind == Kind::Settings {
            dict = self.settings_dict(e, &p, dp.as_deref().unwrap_or_default(), touched)?;
        }
        if kind == Kind::List {
            // порядок нумерации полей списка — очередь отложенных путей читателя (dlist.rs, list_paths_in_load_order)
            dict = self.list_dict(e, &p, touched)?;
        }
        if let Some(rf) = p.get("RowFilter").filter(|_| kind != Kind::List) {
            if rf.attr("xsi:nil") != Some("true") {
                return Err(err(rf, "отбор строк не пустой: не встречено"));
            }
            rf.text()?;
            dict.push(("13", l(vec![s("U")])));
        }
        if full {
            dict.push(("19", l(vec![s("S"), s("")])));
        }
        // ComplexSettingsViewMode (свойство XDTO 0x69): FUN_180734ba0 его не разбирает (таблица переходов — ветвь по
        // умолчанию 0x180736413), FUN_1807577e0 пропускает элемент (FUN_18075ed40) — в записи его нет
        if let Some(c) = p.get("ComplexSettingsViewMode") {
            c.text()?;
        }
        v.push(an(dict.len()));
        for (k, n) in dict {
            v.extend([a(k), n]);
        }
        v.push(events(events_el, &|n| item_event("Table", n))?);
        v.push(match cs {
            None => l(vec![a("0")]),
            Some(cs) => {
                let mut gs = Vec::new();
                for c in cs.elems() {
                    if c.name != "ExcludedCommand" {
                        return Err(err(c, "ожидалась <ExcludedCommand>"));
                    }
                    let n = c.text()?;
                    let g = match n.as_str() {
                        "Delete" if kind == Kind::List => self.list_delete(dp.as_deref().unwrap_or_default()),
                        _ => table_command(&n, kind == Kind::List, kind == Kind::Settings).or_else(|| super::platform::table_command(&n)),
                    };
                    gs.push(g.ok_or_else(|| err(c, &format!("исключенная команда таблицы {n} не встречена")))?);
                    // 07c3d8a2 (ProcessLabelButton, последняя в списке команд таблицы FUN_18045ab80) имени не пишет: разрешение
                    // исключенных mngbase FUN_1802e63f0 сверяет с ними имя предыдущей команды (общий буфер описания local_178)
                    // shortcut: предыдущая команда — по сверке Клиента ЭДО (без основной таблицы SearchEverywhere —
                    // ИнтерфейсДокументовЭДО.АрхивЭлектронныхДокументов, с ней GetURL — СервисДоставки.ОтражениеДоставки);
                    // порядок команд таблицы списка (FUN_180611300 с FUN_180455490) не выведен
                    let main = self.attrs.get(dp.as_deref().unwrap_or_default()).is_some_and(|a| a.main_table.is_some());
                    if kind == Kind::List && n == if main { "GetURL" } else { "SearchEverywhere" } {
                        gs.push("07c3d8a2-e24d-4d52-b430-f5f7a131429c");
                    }
                }
                // пара исключается только вместе: какой GUID какое имя — не установлено (БП и 8.5)
                const PAIR: [&str; 2] = ["48e12019-0fd6-46eb-aab6-2acba716a623", "fc120c02-7f39-469b-b357-b2dd8d4b0765"];
                if PAIR.iter().filter(|g| gs.contains(g)).count() == 1 {
                    return Err(err(cs, "AddAutoOrderItem и Expand исключаются только вместе"));
                }
                gs.sort_unstable();
                if gs.windows(2).any(|w| w[0] == w[1]) {
                    return Err(err(cs, "повтор исключенной команды"));
                }
                let mut n = vec![an(gs.len())];
                n.extend(gs.into_iter().map(a));
                l(n)
            }
        });
        v.extend([a("1"), self.context_menu(p.get("ContextMenu").ok_or_else(|| err(e, "нет <ContextMenu>"))?)?]);
        v.extend([a("1"), self.auto_command_bar(p.get("AutoCommandBar").ok_or_else(|| err(e, "нет <AutoCommandBar>"))?)?]);
        v.extend(self.children(p.get("ChildItems"))?);
        v.extend([
            p.tri("AutoMarkIncomplete")?,
            p.tri("AutoAddIncomplete")?,
            p.b("Visible", "1")?,
            p.b("MultipleChoice", "0")?,
            l(vec![s("Pattern")]),
            s(""),
            s(""),
            p.tri("SkipOnInput")?,
            p.tok("SearchOnInput", &[("Use", 0), ("DontUse", 1)], 2)?,
            p.tok("ToolTipRepresentation", TOOLTIP_REPR, 0)?,
            a("1"),
            self.extended_tooltip(p.get("ExtendedTooltip").ok_or_else(|| err(e, "нет <ExtendedTooltip>"))?)?,
            p.tok("SearchStringLocation", &[("Auto", 0), ("None", 1), ("CommandBar", 2), ("Top", 3), ("Bottom", 4), ("FormCaption", 5), ("PullFromTop", 6)], 0)?,
            p.tok("ViewStatusLocation", &[("None", 1), ("Top", 2), ("Bottom", 3)], 0)?,
            p.tok("SearchControlLocation", &[("None", 1), ("CommandBar", 2)], 0)?,
        ]);
        // дополнения на месте (флаг 1 и узел) или вынесенные в другую группу (флаг 0)
        for k in ["SearchStringAddition", "ViewStatusAddition", "SearchControlAddition"] {
            match p.get(k) {
                Some(x) => v.extend([a("1"), self.addition(x)?]),
                None => v.push(a("0")),
            }
        }
        v.extend([
            p.tok("RefreshRequest", &[("PullFromTop", 1), ("PullFromBottom", 2), ("PullFromTopOrBottom", 3)], 0)?,
            p.b("AutoMaxWidth", "1")?,
            p.num("MaxWidth", "0")?,
            a("0"),
            p.b("AutoMaxHeight", "1")?,
            p.num("MaxHeight", "0")?,
            p.tok("GroupHorizontalAlign", H_ALIGN, 3)?,
            p.tok("GroupVerticalAlign", V_ALIGN, 3)?,
            p.tok("HeightControlVariant", &[("UseHeightInFormRows", 1), ("UseHeightInTableRows", 2), ("UseContentHeight", 3)], 0)?,
            p.b("AutoMaxRowsCount", "1")?,
            p.num("MaxRowsCount", "0")?,
            p.tok("CurrentRowUse", &[("Choice", 1), ("SelectionPresentation", 2), ("SelectionPresentationAndChoice", 3)], 0)?,
            p.tok("BehaviorOnHorizontalCompression", &[("Auto", 0), ("HideItemsByImportance", 1), ("MoveItemsByImportance", 2)], 0)?,
            importance(e)?,
            p.tok("FileDragMode", &[("AsFile", 0)], 1)?,
            a("0"),
            p.tri("HorizontalLinesBWA")?,
            p.tri("VerticalLinesBWA")?,
            p.tok("HierarchyPanelLocation",&[("Auto",0),("None",1)],0)?,
            a("0"),
            s(""),
            s(""),
            p.tok("InitialRowActivation",&[("Auto",0),("Activate",1),("NoActivate",2)],0)?,
            a("1"),
            panel(name, "ПанельДействийВыделенныхСтрок", "10"),
            a("0"),
            p.tri("UseAlternationRowColorBWA")?,
            a(if scb { "1" } else { "0" }),
            p.tok("CommandBarLocation", &[("None", 0), ("Top", 1), ("Bottom", 1)], 2)?,
            a("1"),
            panel(name, "ДействияСтроки", "11"),
            a("2"),
            a("0"),
            rsm_tail,
            a("1"),
            a("0"),
            a("0"),
            a("0"),
            a("0"),
            a("0"),
            a("1"),
            self.panel_addition(name, id, 0)?,
            a("1"),
            self.panel_addition(name, id, 2)?,
        ]);
        Ok(l(v))
    }

    /// Словарь таблицы динамического списка. Читатель Form.xml (mngbase FUN_1807577e0 — элементы `<Table>` по номеру
    /// свойства XDTO-типа logform:Table, свойства таблицы — FUN_180734ba0) ставит каждое свойство списка из XML прямо в
    /// словарь модели («установить» +0x5f0, FUN_18074fb80) под ключом FUN_180731700: ViewMode 3,
    /// SettingsNamedItemDetailedRepresentation 4, AutoRefresh 5 … AllowRootChoice 12, RowFilter 13, UpdateOnDataChange 14,
    /// AllowGettingCurrentRowURL 20; UserSettingsGroup — 16 (id элемента, отложенно, FUN_18075fdc0). ComplexSettingsViewMode
    /// (0x69) FUN_180731700 отображает в 21, но FUN_180734ba0 его не разбирает — у списка с ним остается отказ ниже.
    /// Расширение cfg:DynamicList (vtable 0x180850210) при этом не создается. Если при загрузке к расширению обращаются
    /// (`touched`, как у коллекций), FUN_1802c1660 проходит ключи 0–21: необъявленные расширением (FUN_18043e230: 5–12, 14,
    /// 15, 16, 19, 20) удаляет (кроме `{"U"}` — «есть» FUN_180396140 его не видит), объявленным без значения ставит
    /// умолчание FUN_18043e250.
    fn list_dict(&self, e: &El, p: &Props, touched: bool) -> R<Vec<(&'static str, Node)>> {
        let hash = |g: &str, v: Node| l(vec![s("#"), a(g), v]);
        let bn = |b: bool| l(vec![s("B"), a(if b { "1" } else { "0" })]);
        let nil = |x: &El| -> R<Node> {
            if x.attr("xsi:nil") != Some("true") {
                return Err(err(x, "не пустое значение: не встречено"));
            }
            x.text()?;
            Ok(l(vec![s("U")]))
        };
        let period = |x: &El| -> R<Node> {
            let ve = x.req("v8:variant")?;
            let v = ve.text()?;
            if v != "Custom" || ve.attr("xsi:type") != Some("v8:StandardPeriodVariant") {
                return Err(err(x, &format!("вариант периода {v}: не встречен")));
            }
            let date = |k: &str| -> R<Node> {
                let t = x.text_of(k)?;
                let d: String = t.chars().filter(char::is_ascii_digit).collect();
                if d.len() != 14 {
                    return Err(err(x, &format!("дата {t:?}")));
                }
                Ok(a(d))
            };
            Ok(hash("2fdc88ec-7c9b-43cd-8ba5-873f043bdd88", l(vec![a("0"), date("v8:startDate")?, date("v8:endDate")?])))
        };
        let mut d: std::collections::BTreeMap<u8, Node> = Default::default();
        for (k, name) in [
            (3, "ViewMode"),
            (4, "SettingsNamedItemDetailedRepresentation"),
            (5, "AutoRefresh"),
            (6, "AutoRefreshPeriod"),
            (7, "Period"),
            (8, "ChoiceFoldersAndItems"),
            (9, "RestoreCurrentRow"),
            (10, "TopLevelParent"),
            (11, "ShowRoot"),
            (12, "AllowRootChoice"),
            (13, "RowFilter"),
            (14, "UpdateOnDataChange"),
            (16, "UserSettingsGroup"),
            (20, "AllowGettingCurrentRowURL"),
            (21, "ComplexSettingsViewMode"),
        ] {
            let Some(x) = p.get(name) else { continue };
            let v = match k {
                3 => hash("c04ead79-749a-4981-915e-6fcb144f44e4", token(x, &[("All", 0), ("QuickAccess", 1)])?),
                4 | 5 | 9 | 11 | 12 | 20 => bn(boolean(x)?),
                6 => l(vec![s("N"), a(num(x)?)]),
                7 => period(x)?,
                8 => hash("59ef2b80-c86b-11d5-a3c1-0050bae0a776", token(x, &[("Items", 0), ("Folders", 1), ("FoldersAndItems", 2)])?),
                10 | 13 => nil(x)?,
                14 => hash("eac7bfa0-10b4-4369-996c-d258871ad519", token(x, &[("Auto", 0), ("DontUpdate", 1)])?),
                16 => l(vec![s("N"), super::item_id(self, x)?]),
                _ => return Err(err(x, "не встречено")),
            };
            d.insert(k, v);
        }
        const KEYS: [&str; 22] = ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12", "13", "14", "15", "16", "17", "18", "19", "20", "21"];
        if !touched {
            // Без событий, исключенных команд и ссылок кнопок расширение таблицы при загрузке не прикрепляется (трасса БП:
            // 394 из 398 таких таблиц — ни одного вызова FUN_1802c1660; входы прикрепления — xml-import.md, «Кто
            // прикрепляет расширение элемента при загрузке») — в словаре только свойства, которые ставит читатель.
            // Ключ 1 словаря формы динамического списка от этого не зависит (`form_dict`: менеджер расширений формы).
            return Ok(d.into_iter().map(|(k, v)| (KEYS[k as usize], v)).collect());
        }
        ext_sync(&mut d).map_err(|k| err(e, &format!("нет свойства динамического списка (ключ {k}): не встречено")))?;
        Ok(d.into_iter().map(|(k, v)| (KEYS[k as usize], v)).collect())
    }

    /// Словарь таблицы настроек компоновщика — по последней части пути (сверка БП: 260 таблиц): отбор — режим
    /// просмотра (3) и подробное представление (4), условное оформление — 4, доступные поля (при обращении к расширению) и параметры — пустые
    /// ограничения использования (0, 1), порядок и структура — 19 при обращении к расширению (как у коллекций).
    /// Таблица через текущие данные таблицы компоновщика (`Items.<таблица>.CurrentData.Item…`) — то же по типу части
    /// (вид `Settings` в `table_kind`). Классы расширений mngbase (проверка типа +0x50 → слоты +0xb0) — ниже по ветвям.
    fn settings_dict(&self, e: &El, p: &Props, dp: &str, touched: bool) -> R<Vec<(&'static str, Node)>> {
        let hash = |g: &str, v: Node| l(vec![s("#"), a(g), v]);
        let b4 = || -> R<Node> {
            let v = boolean(p.get("SettingsNamedItemDetailedRepresentation").ok_or_else(|| err(e, "нет SettingsNamedItemDetailedRepresentation: не встречено"))?)?;
            Ok(l(vec![s("B"), a(if v { "1" } else { "0" })]))
        };
        let restrictions = |root: &str| {
            let x = format!("\u{feff}<?xml version=\"1.0\" encoding=\"UTF-8\"?>\r\n<{root} xmlns=\"http://v8.1c.ru/8.1/data-composition-system/settings\" xmlns:xs=\"http://www.w3.org/2001/XMLSchema\" xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\"/>");
            l(vec![crate::xml::imp::payload::base64_atom(x.as_bytes())])
        };
        let last = dp.rsplit('.').next().unwrap_or_default();
        Ok(match last {
            "Filter" | "ItemFilter" => {
                let vm = token(p.get("ViewMode").ok_or_else(|| err(e, "нет ViewMode: не встречено"))?, &[("All", 0), ("QuickAccess", 1)])?;
                vec![("3", hash("c04ead79-749a-4981-915e-6fcb144f44e4", vm)), ("4", b4()?)]
            }
            "ConditionalAppearance" | "ItemConditionalAppearance" => vec![("4", b4()?)],
            // ограничения — только при обращении к расширению (события, команды): без них словаря нет
            "FilterAvailableFields" | "OrderAvailableFields" | "SelectionAvailableFields" | "GroupFieldsAvailableFields" if !touched => vec![],
            // доступные поля (9d9f5b23 DataCompositionAvailableFields: dcscore — типы свойств GroupFieldsAvailableFields
            // FUN_180164a90 и OrderAvailableFields; проверка mngbase FUN_1803bc640) — слот 0 (FUN_180047ed0), умолчание
            // FUN_1803bd180
            "FilterAvailableFields" | "OrderAvailableFields" | "SelectionAvailableFields" | "GroupFieldsAvailableFields" => {
                vec![("0", hash("54ea12c9-d306-418f-93c0-586cb80d41a5", restrictions("AvailableFieldUseRestrictions")))]
            }
            "Appearance" => vec![("1", hash("f33178f9-6060-48d3-bcd4-3f8682618c3f", restrictions("AvailableParameterUseRestrictions")))],
            // Расширение по типу пути (FUN_1802c1180: фабрики 3d7fb435, проверка типа +0x50), FUN_1802c1660 оставляет
            // только объявленные слоты (+0xb0), объявленным без значения — умолчание (+0xb8). Типы частей `Item…` строки
            // структуры настроек — dcscore (свойства элемента, 0x1801d6190…0x1801d67dd), переход через текущие данные
            // таблицы — путь таблицы (FUN_1802c8a30). Значения параметров (6217eee1 ItemDataParameters, 3300f132
            // ItemOutputParameters; проверка FUN_1803b3fc0, как у Appearance) — слот 1 (FUN_1803b6630), умолчание
            // FUN_1803b6640.
            // параметры данных настроек (`Settings.DataParameters`) — того же типа 6217eee1 (ЗначенияПараметровДанных), что
            // ItemDataParameters. shortcut: тип свойства Settings — по справке платформы, таблица свойств dcscore не
            // прочитана; сверено на УпрощеннаяНастройкаСхемыКомпоновкиДанных ERP
            "DataParameters" | "ItemDataParameters" | "ItemOutputParameters" if touched => {
                vec![("1", hash("f33178f9-6060-48d3-bcd4-3f8682618c3f", restrictions("AvailableParameterUseRestrictions")))]
            }
            // базовое расширение (слот 0x13 = "", FUN_1803b1ae0, FUN_1803b1af0): порядок (11743ff3, FUN_1803cc250),
            // настройки (3c72ff56, FUN_1803e2930), поля группировки (e2e2f2e9, FUN_1803c76b0), выбранные поля (a549d202,
            // FUN_1803cfd40), пользовательские поля (58f85874, FUN_1803d7bc0), поля условного оформления (95a67d64,
            // FUN_1803b97f0)
            "Order" | "Settings" | "ItemOrder" | "ItemGroupFields" | "ItemSelection" | "ItemUserFields" if touched => vec![("19", l(vec![s("S"), s("")]))],
            "Fields" if touched && dp.contains("ConditionalAppearance") => vec![("19", l(vec![s("S"), s("")]))],
            // пользовательские настройки (f099bb57, проверка FUN_1803d9530) — слоты 3 и 0x15 (FUN_1803db320), умолчания
            // FUN_1803db330: перечисления c04ead79 и 2eb62aaa, значение 0. ViewMode из XML проходит проверку
            // FUN_1803db400 (тип c04ead79); ComplexSettingsViewMode читатель не ставит (см. `table`) — всегда умолчание.
            "UserSettings" if touched => {
                let vm = match p.get("ViewMode") {
                    Some(x) => token(x, &[("All", 0), ("QuickAccess", 1)])?,
                    None => a("0"),
                };
                vec![("3", hash("c04ead79-749a-4981-915e-6fcb144f44e4", vm)), ("21", hash("2eb62aaa-e6c1-48b6-a047-435354d5ae82", a("0")))]
            }
            // без прикрепления расширения (как у таблицы списка: FUN_1802c1660 при загрузке — только пути A–D) в словаре
            // только свойства, которые ставит читатель (FUN_180734ba0 → ключ FUN_180731700: ViewMode 3, …Representation 4)
            _ if !touched => {
                let mut v = Vec::new();
                if let Some(vm) = p.get("ViewMode") {
                    v.push(("3", hash("c04ead79-749a-4981-915e-6fcb144f44e4", token(vm, &[("All", 0), ("QuickAccess", 1)])?)));
                }
                if p.get("SettingsNamedItemDetailedRepresentation").is_some() {
                    v.push(("4", b4()?));
                }
                v
            }
            _ => return Err(err(e, &format!("таблица настроек компоновщика {last} (с обращением к расширению) не встречена"))),
        })
    }

    /// Дополнение таблицы (версия 6): строка поиска (вид 0), состояние просмотра (1), управление поиском (2).
    pub(super) fn addition(&self, e: &El) -> R<Node> {
        let p = Props::new(e)?;
        let (kind, src_type) = match e.name.as_str() {
            "SearchStringAddition" => (0, "SearchStringRepresentation"),
            "ViewStatusAddition" => (1, "ViewStatusRepresentation"),
            "SearchControlAddition" => (2, "SearchControl"),
            o => return Err(err(e, &format!("дополнение {o}"))),
        };
        // без <AdditionSource> — источник командная панель формы: `{-1,-1}` (обратно выгрузке)
        let src_id = match p.get("AdditionSource") {
            None => None,
            Some(src) => {
                let item = src.text_of("Item")?;
                if src.text_of("Type")? != src_type {
                    return Err(err(src, "вид источника дополнения не как у дополнения"));
                }
                for c in src.elems() {
                    if !matches!(c.name.as_str(), "Item" | "Type") {
                        return Err(err(c, "в источнике дополнения"));
                    }
                }
                Some(self.items.get(&item).ok_or_else(|| err(src, &format!("нет элемента {item}")))?.id.clone())
            }
        };
        let sub = match kind {
            0 => vec![
                a("1"),
                p.num("Width", "0")?,
                p.tri("HorizontalStretch")?,
                color_auto(),
                color_auto(),
                color_auto(),
                font_auto(),
                events(p.get("Events"), &|_| None)?,
                p.b("AutoMaxWidth", "1")?,
                p.num("MaxWidth", "0")?,
                a("0"),
            ],
            1 => vec![
                a("1"),
                p.num("Width", "0")?,
                p.tri("HorizontalStretch")?,
                color_auto(),
                color_auto(),
                color_auto(),
                color_auto(),
                color_auto(),
                font_auto(),
                font_auto(),
                border(None, 0)?,
                p.tok("HorizontalLocation", &[("Left", 0), ("Center", 1), ("Right", 2)], 3)?,
                events(p.get("Events"), &|_| None)?,
                p.b("AutoMaxWidth", "1")?,
                p.num("MaxWidth", "0")?,
                a("0"),
            ],
            _ => vec![
                a("1"),
                p.num("Width", "0")?,
                color_auto(),
                color_auto(),
                color_auto(),
                font_auto(),
                events(p.get("Events"), &|_| None)?,
                p.b("AutoMaxWidth", "1")?,
                p.num("MaxWidth", "0")?,
                a("0"),
                p.tri("HorizontalStretch")?,
            ],
        };
        let mut v = vec![a("6"), l(vec![a(e.req_attr("id")?), a(ELEM)]), a("0"), a("0")];
        match p.get("UserVisible") {
            Some(u) => v.extend([a("1"), adjustable(self.x, Some(u))?]),
            None => v.push(a("0")),
        }
        v.extend([
            an(kind),
            s(e.req_attr("name")?),
            p.ml("Title")?,
            p.ml("ToolTip")?,
            p.b("Visible", "1")?,
            p.b("Enabled", "1")?,
            p.tok("ToolTipRepresentation", TOOLTIP_REPR, 0)?,
            a("1"),
            l(sub),
            a("1"),
            self.context_menu(p.get("ContextMenu").ok_or_else(|| err(e, "нет <ContextMenu>"))?)?,
            a("1"),
            self.extended_tooltip(p.get("ExtendedTooltip").ok_or_else(|| err(e, "нет <ExtendedTooltip>"))?)?,
            a("2"),
            src_id.map_or_else(|| l(vec![a("-1"), a("-1")]), |i| l(vec![a(i), an(kind)])),
        ]);
        v.extend(self.children(p.get("ChildItems"))?);
        v.extend([p.tok("GroupHorizontalAlign", H_ALIGN, 3)?, p.tok("GroupVerticalAlign", V_ALIGN, 3)?, importance(e)?, s("")]);
        Ok(l(v))
    }

    /// Дополнение панели действий выделенных строк: как пустое дополнение таблицы, без id, имя — от имени таблицы;
    /// в конце — имя дополнения таблицы того же вида по умолчанию (`<таблица>СтрокаПоиска`, даже если свое названо иначе).
    fn panel_addition(&self, table: &str, id: &str, kind: i64) -> R<Node> {
        let (tag, ty, suffix) = match kind {
            0 => ("SearchStringAddition", "SearchStringRepresentation", "СтрокаПоиска"),
            _ => ("SearchControlAddition", "SearchControl", "УправлениеПоиском"),
        };
        let n = format!("{table}{}{suffix}", "ПанельДействийВыделенныхСтрок");
        let x = format!(
            "<{tag} name=\"{n}\" id=\"0\"><AdditionSource><Item>{table}</Item><Type>{ty}</Type></AdditionSource><ContextMenu name=\"{n}КонтекстноеМеню\" id=\"0\"/><ExtendedTooltip name=\"{n}РасширеннаяПодсказка\" id=\"0\"/></{tag}>"
        );
        let el = crate::xml::imp::dom::parse(&x)?;
        let mut node = self.addition(&el)?;
        no_ids(&mut node);
        let v = node.list_mut().ok_or("дополнение панели")?;
        // источник — сама таблица
        let src = v.len() - 6;
        v[src] = l(vec![a(id), an(kind)]);
        *v.last_mut().ok_or("дополнение панели")? = s(format!("{table}{suffix}"));
        Ok(node)
    }

    /// Группа колонок (группа вида 2): свойства вида — узел из 16 полей.
    pub(super) fn column_group(&self, e: &El) -> R<Node> {
        let p = Props::new(e)?;
        let mut v = self.group_head(e, &p, 2)?;
        let show_title = p.get("ShowTitle").map(boolean).transpose()?.unwrap_or(true);
        v.push(l(vec![
            a("5"),
            p.tok("Group", &[("Horizontal", 0), ("InCell", 2)], 1)?,
            a(if show_title { "1" } else { "0" }),
            p.b("ShowInHeader", "0")?,
            p.tok("HeaderHorizontalAlign", H_ALIGN, 3)?,
            self.pic(e, &p, "HeaderPicture")?,
            p.color(self.x, "TitleBackColor")?,
            match p.get("HeaderDataPath") {
                Some(d) => self.data_path(d)?,
                None => l(vec![a("0")]),
            },
            l(vec![s("Pattern")]),
            s(""),
            p.ml("HeaderFormat")?,
            p.tok("FixingInTable", super::platform::FIXING_IN_TABLE, 0)?,
            a("1"),
            a("2"),
            a("2"),
            p.tri("ShowTitle")?,
        ]));
        v.extend(self.children(p.get("ChildItems"))?);
        v.extend(self.group_tail(e, &p)?);
        Ok(l(v))
    }
}

/// Панель, которую платформа добавляет таблице (группа без id): `ПанельДействийВыделенныхСтрок` (вид 10),
/// `ДействияСтроки` (вид 11).
fn panel(table: &str, suffix: &str, kind: &str) -> Node {
    l(vec![
        a("22"),
        l(vec![a("0")]),
        a("1"),
        a("0"),
        a("0"),
        a(kind),
        s(format!("{table}{suffix}")),
        l(vec![a("1"), a("0")]),
        l(vec![a("1"), a("0")]),
        a("0"),
        a("1"),
        a("0"),
        a("0"),
        a("0"),
        a("2"),
        a("2"),
        color_auto(),
        font_auto(),
        l(vec![a("0"), a("0"), a("0")]),
        a("1"),
        l(vec![a("0"), a("1"), a("0"), a("1")]),
        a("0"),
        a("1"),
        a("0"),
        a("0"),
        a("0"),
        a("3"),
        a("3"),
        a("0"),
    ])
}

/// Словарь модели таблицы после прикрепления расширения cfg:DynamicList (mngbase FUN_1802c1660 по ключам 0–21):
/// необъявленные расширением (FUN_18043e230) удаляются, если «есть» (FUN_180396140: `{"U"}` не видит), объявленным без
/// значения — умолчание FUN_18043e250. Ошибка — ключ, умолчание которого в записи не сверено (7, 8, 14).
fn ext_sync(d: &mut std::collections::BTreeMap<u8, Node>) -> Result<(), u8> {
    const DECLARED: [u8; 13] = [5, 6, 7, 8, 9, 10, 11, 12, 14, 15, 16, 19, 20];
    let bn = |b: bool| l(vec![s("B"), a(if b { "1" } else { "0" })]);
    d.retain(|k, v| DECLARED.contains(k) || v.get(0).and_then(Node::str) == Some("U"));
    for k in DECLARED {
        if d.contains_key(&k) {
            continue;
        }
        let v = match k {
            5 | 9 | 12 => bn(false),
            6 => l(vec![s("N"), a("60")]),
            10 | 15 => l(vec![s("U")]),
            11 | 20 => bn(true),
            16 => l(vec![s("N"), a("0")]),
            19 => l(vec![s("S"), s("")]),
            _ => return Err(k),
        };
        d.insert(k, v);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_dropped_at_any_depth() {
        let mut n = l(vec![a("6"), l(vec![a("5"), a(ELEM)]), l(vec![a("22"), l(vec![a("7"), a(ELEM)])]), l(vec![a("5"), a("0")])]);
        no_ids(&mut n);
        assert_eq!(n, l(vec![a("6"), l(vec![a("0")]), l(vec![a("22"), l(vec![a("0")])]), l(vec![a("5"), a("0")])]));
        assert_eq!(table_command("Delete", false, false), Some("8d772f97-c0ef-47c0-9cb0-efea28c61341"));
        assert_eq!(table_command("Delete", true, false), None);
    }

    #[test]
    fn dynamic_list_extension_sync() {
        let mut d: std::collections::BTreeMap<u8, Node> = Default::default();
        d.insert(3, l(vec![s("#"), a("c04ead79-749a-4981-915e-6fcb144f44e4"), a("0")]));
        d.insert(13, l(vec![s("U")]));
        for k in [7, 8, 14] {
            d.insert(k, a("x"));
        }
        d.insert(6, l(vec![s("N"), a("30")]));
        ext_sync(&mut d).unwrap();
        let keys: Vec<u8> = d.keys().copied().collect();
        assert_eq!(keys, [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 19, 20]);
        assert_eq!(d[&6], l(vec![s("N"), a("30")]));
        assert_eq!(d[&19], l(vec![s("S"), s("")]));
        d.remove(&7);
        assert_eq!(ext_sync(&mut d), Err(7));
    }

    #[test]
    fn settings_data_parameters_as_item() {
        // параметры данных настроек — словарь как у ItemDataParameters; без обращения к расширению — пустой
        let x = Index::default();
        let read = |_: &str| -> Result<Vec<u8>, String> { Err(String::new()) };
        let f = F { x: &x, attrs: Default::default(), main: None, cmds: Default::default(), items: Default::default(), referenced: Default::default(), read: &read, used: Default::default(), lists: Default::default(), list_req: Default::default() };
        let e = crate::xml::imp::dom::parse("<Table/>").unwrap();
        let p = Props::new(&e).unwrap();
        let dp = f.settings_dict(&e, &p, "К.Settings.DataParameters", true).unwrap();
        assert_eq!(dp, f.settings_dict(&e, &p, "К.Settings.Structure.ItemDataParameters", true).unwrap());
        assert_eq!(dp[0].0, "1");
        assert!(f.settings_dict(&e, &p, "К.Settings.DataParameters", false).unwrap().is_empty());
    }
}
