//! Настройки списка динамического списка (`<ListSettings>`) → ключи словаря настроек `Filter`, `Order`, `Group…`,
//! `Appearance` и содержимое `DataParameters` (обратное `xml/form.rs` `dynamic_list`).
//!
//! Писатель словаря — mngbase FUN_1802b4660: после `AutoSaveUserSettings` по порядку отбор (`Filter`), порядок
//! (`Order`), группировка (`GroupSelectedSettingId`, `GroupSelectedSettingViewMode`, `Group`,
//! `GroupSelectedSettingPresentation`), условное оформление (`Appearance`); значение — объект настроек
//! `{"#",<тип>,{base64 XML}}`, XML — сериализатор XDTO (`dcs::value_part`). Ключ пишется, если у настройки есть
//! элементы или она не «по умолчанию» (FUN_1802b4540: идентификатор пользовательской настройки не пуст, режим
//! просмотра 0 — Normal, представление пусто). Ветка писателя с байтом +0x24 словаря полей (интерфейс
//! `DAT_180836a60`, FUN_1805f0fd0) — ключ только при элементах — не прослежена: в загрузке БП ключ без элементов есть
//! (`<ListSettings/>`), то есть байт снят.

use super::*;

const FILTER: &str = "f6841c6b-6c71-4c82-ae9e-d08b49db326c";
const ORDER: &str = "11743ff3-2db3-4cfc-9404-90ed8209437f";
const GROUP: &str = "e2e2f2e9-e309-4212-9c70-3ab32dd93b4d";
const APPEARANCE: &str = "93de27ad-a2d8-4b10-a82b-483c9b0648fe";
/// Многоязычная строка (представление пользовательской настройки).
const LOCAL_STRING: &str = "87024738-fc2a-4436-ada1-df79d395c424";

/// Разобранные настройки списка: ключи после `AutoSaveUserSettings` и XML параметров данных (`None` — пустые).
pub(super) struct ListSettings {
    pub keys: Vec<(String, Node)>,
    pub data_parameters: Option<String>,
}

/// Текст элемента-настройки `e` (его дети — элементы настройки) для `dcs::value_part`.
fn src(e: &El) -> String {
    e.touch_all();
    src_of(e.peek_elems())
}

fn src_of<'a>(items: impl Iterator<Item = &'a El>) -> String {
    let mut src = String::from("<dcsset:x");
    for (k, v) in ROOT_NS.iter().filter(|(k, _)| k.starts_with("xmlns")) {
        src += &format!(" {k}=\"{v}\"");
    }
    src += ">";
    for c in items {
        xml_text(c, &mut src);
    }
    src + "</dcsset:x>"
}

/// Группировка списка цепочкой (`StructureItemGroup` с `groupItems` и вложенной группировкой) → поля группировки по
/// порядку уровней. Платформа держит поля группировки списка цепочкой по одному полю на уровень: mngbase
/// FUN_18065ef10 переносит поля группы со второго в новые вложенные группы (`структура[0]`, тип DAT_1808889a0),
/// FUN_180660da0 / FUN_180662490 спускаются по `структура[0]` (+0xc0, QI DAT_180886910). shortcut: обратная сборка
/// (настройки XML → `Group` записи, геттер +0xd8 списка) не прочитана — принята только форма, которую пишет выгрузка
/// (`xml/form.rs`: на каждом вложенном уровне ровно одно поле, у уровня нет других детей); иное — отказ
fn group_chain(it: &El) -> R<Vec<&El>> {
    let mut items = Vec::new();
    let mut level = it;
    for depth in 0.. {
        if level.attr("xsi:type") != Some("dcsset:StructureItemGroup") {
            return Err(err(level, "группировка динамического списка не StructureItemGroup"));
        }
        let mut kids = level.elems();
        let (g, next) = match (kids.next(), kids.next(), kids.next()) {
            (Some(g), next, None) if g.name == "dcsset:groupItems" && next.is_none_or(|n| n.name == "dcsset:item") => (g, next),
            _ => return Err(err(level, "группировка динамического списка не только с groupItems")),
        };
        g.touch_all();
        let n = g.peek_elems().count();
        if n != 1 && (next.is_some() || depth > 0) {
            return Err(err(g, "группировка динамического списка цепочкой не по одному полю на уровень"));
        }
        items.extend(g.peek_elems());
        match next {
            Some(n) => level = n,
            None => break,
        }
    }
    Ok(items)
}

fn hash(g: &str, xml: &str) -> Node {
    l(vec![s("#"), a(g), l(vec![super::super::payload::base64_atom(xml.as_bytes())])])
}

/// Настройка «по умолчанию» (FUN_1802b4540): элементов нет, идентификатор не пуст, режим Normal, нет представления.
fn is_default(items: bool, id: &str, normal: bool, presentation: bool) -> bool {
    !items && !id.is_empty() && normal && !presentation
}

/// `<ListSettings>` → ключи словаря. Порядок элементов — как пишет выгрузка; иное — отказ.
pub(super) fn list_settings(x: &Index, ls: &El) -> R<ListSettings> {
    const ORDER_XML: &[&str] = &[
        "dcsset:filter",
        "dcsset:dataParameters",
        "dcsset:order",
        "dcsset:conditionalAppearance",
        "dcsset:item",
        "dcsset:itemsViewMode",
        "dcsset:itemsUserSettingID",
        "dcsset:itemsUserSettingPresentation",
    ];
    let mut at = 0;
    let mut got: Vec<Option<&El>> = vec![None; ORDER_XML.len()];
    for c in ls.elems() {
        let k = ORDER_XML.iter().position(|n| *n == c.name).filter(|&k| k >= at).ok_or_else(|| err(c, "состав настроек динамического списка пока не загружается"))?;
        got[k] = Some(c);
        at = k + 1;
    }
    let value = |e: &El, root: &str| dcs_value(x, e, root);
    let mut keys = Vec::new();
    plain(x, &mut keys, got[0], "Filter", FILTER, "Filter")?;
    plain(x, &mut keys, got[2], "Order", ORDER, "Order")?;
    let appearance = got[3];
    // группировка: `<dcsset:item xsi:type="dcsset:StructureItemGroup">` с `groupItems` (и цепочкой вложенных)
    let items = got[4].map(group_chain).transpose()?;
    let mode = match got[5].map(El::text).transpose()?.as_deref() {
        Some("Normal") => 0,
        None => 1,
        Some(_) => return Err(err(got[5].unwrap_or(ls), "режим группировки динамического списка пока не загружается")),
    };
    let id = got[6].map(El::text).transpose()?.unwrap_or_default();
    let has_items = items.as_ref().is_some_and(|v| !v.is_empty());
    // представление `<v8:item><v8:lang/><v8:content/></v8:item>…` → `{N,"язык","текст",…}` (обратно выгрузке)
    let mut pres = vec![a("0")];
    if let Some(p) = got[7] {
        if p.attr("xsi:type") != Some("v8:LocalStringType") {
            return Err(err(p, "представление группировки не LocalStringType"));
        }
        for it in p.elems() {
            if it.name != "v8:item" || it.elems().count() != 2 {
                return Err(err(it, "представление группировки"));
            }
            pres.extend([s(it.text_of("v8:lang")?), s(it.text_of("v8:content")?)]);
        }
        pres[0] = an((pres.len() as i64 - 1) / 2);
    }
    if !is_default(has_items, &id, mode == 0, got[7].is_some()) {
        keys.push(("GroupSelectedSettingId".into(), l(vec![s("S"), s(id)])));
        keys.push(("GroupSelectedSettingViewMode".into(), l(vec![s("N"), an(mode)])));
        let xml = match &items {
            Some(v) => super::super::dcs::value_part(&src_of(v.iter().copied()), x, "GroupItems").map_err(|m| err(got[4].unwrap_or(ls), &m))?,
            None => value_empty("GroupItems"),
        };
        keys.push(("Group".into(), hash(GROUP, &xml)));
        keys.push(("GroupSelectedSettingPresentation".into(), l(vec![s("#"), a(LOCAL_STRING), l(pres)])));
    }
    plain(x, &mut keys, appearance, "Appearance", APPEARANCE, "ConditionalAppearance")?;
    let data_parameters = got[1].map(|e| value(e, "DataParameterValues")).transpose()?;
    Ok(ListSettings { keys, data_parameters })
}

/// Отбор, порядок, оформление: нет элемента в XML — в записи пустая настройка (выгрузка ее не пишет); настройка
/// «по умолчанию» — ключа нет.
fn plain(x: &Index, keys: &mut Vec<(String, Node)>, e: Option<&El>, key: &str, g: &str, root: &str) -> R<()> {
    let xml = match e {
        None => value_empty(root),
        Some(e) if setting_is_default(e) => {
            e.touch_all();
            return Ok(());
        }
        Some(e) => dcs_value(x, e, root)?,
    };
    keys.push((key.to_string(), hash(g, &xml)));
    Ok(())
}

fn dcs_value(x: &Index, e: &El, root: &str) -> R<String> {
    super::super::dcs::value_part(&src(e), x, root).map_err(|m| err(e, &m))
}

fn value_empty(root: &str) -> String {
    format!("\u{feff}<?xml version=\"1.0\" encoding=\"UTF-8\"?>\r\n<{root} xmlns=\"http://v8.1c.ru/8.1/data-composition-system/settings\" xmlns:xs=\"http://www.w3.org/2001/XMLSchema\" xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\"/>")
}

/// Отбор, порядок, оформление «по умолчанию»: последние дети — `viewMode`, `userSettingID`, `userSettingPresentation`.
fn setting_is_default(e: &El) -> bool {
    let mut items = false;
    let (mut id, mut normal, mut presentation) = (String::new(), false, false);
    for c in e.peek_elems() {
        match c.name.as_str() {
            "dcsset:viewMode" => normal = c.peek_text().as_deref() == Some("Normal"),
            "dcsset:userSettingID" => id = c.peek_text().unwrap_or_default(),
            "dcsset:userSettingPresentation" => presentation = true,
            _ => items = true,
        }
    }
    is_default(items, &id, normal, presentation)
}

#[cfg(test)]
mod tests {
    #[test]
    fn default_setting() {
        assert!(super::is_default(false, "dfcece9d-5077-440b-b6b3-45a5cb4538eb", true, false));
        assert!(!super::is_default(false, "", true, false));
        assert!(!super::is_default(true, "x", true, false));
        assert!(!super::is_default(false, "x", false, false));
    }

    #[test]
    fn data_parameters_as_platform() {
        // Synthetic disabled data parameter.
        let x = "<ListSettings><dcsset:dataParameters><dcscor:item xsi:type=\"dcsset:SettingsParameterValue\"><dcscor:use>false</dcscor:use><dcscor:parameter>SyntheticParameter</dcscor:parameter></dcscor:item></dcsset:dataParameters></ListSettings>";
        let ls = super::super::super::dom::parse(x).unwrap();
        let r = super::list_settings(&Default::default(), &ls).unwrap();
        let want = "\u{feff}<?xml version=\"1.0\" encoding=\"UTF-8\"?>\r\n<DataParameterValues xmlns=\"http://v8.1c.ru/8.1/data-composition-system/settings\" xmlns:xs=\"http://www.w3.org/2001/XMLSchema\" xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\">\r\n\t<item xmlns=\"http://v8.1c.ru/8.1/data-composition-system/core\" xmlns:dcsset=\"http://v8.1c.ru/8.1/data-composition-system/settings\" xsi:type=\"dcsset:SettingsParameterValue\">\r\n\t\t<use>false</use>\r\n\t\t<parameter>SyntheticParameter</parameter>\r\n\t</item>\r\n</DataParameterValues>";
        assert_eq!(r.data_parameters.as_deref(), Some(want));
        // отбора, порядка, оформления в XML нет — пустые настройки; группировка без идентификатора — тоже ключи
        let keys: Vec<&str> = r.keys.iter().map(|(k, _)| k.as_str()).collect();
        assert_eq!(keys, ["Filter", "Order", "GroupSelectedSettingId", "GroupSelectedSettingViewMode", "Group", "GroupSelectedSettingPresentation", "Appearance"]);
    }

    #[test]
    fn group_chain() {
        let level = |f: &str, next: &str| format!("<dcsset:item xsi:type=\"dcsset:StructureItemGroup\"><dcsset:groupItems><dcsset:item xsi:type=\"dcsset:GroupItemField\"><dcsset:field>{f}</dcsset:field></dcsset:item></dcsset:groupItems>{next}</dcsset:item>");
        let x = level("А", &level("Б", ""));
        let e = super::super::super::dom::parse(&x).unwrap();
        let v: Vec<String> = super::group_chain(&e).unwrap().iter().map(|i| i.text_of("dcsset:field").unwrap()).collect();
        assert_eq!(v, ["А", "Б"]);
        // два поля на уровне цепочки — не форма выгрузки, отказ
        let two = "<dcsset:item xsi:type=\"dcsset:StructureItemGroup\"><dcsset:groupItems><dcsset:item/><dcsset:item/></dcsset:groupItems>";
        let e = super::super::super::dom::parse(&format!("{two}{}</dcsset:item>", level("Б", ""))).unwrap();
        assert!(super::group_chain(&e).is_err());
    }
}
