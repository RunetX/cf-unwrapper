//! `Configuration.xml` → запись конфигурации `<uuid из <Configuration uuid>>` (обратное `configuration` в
//! `xml/classes_tools.rs`): семь частей-контейнеров (ClassId и ObjectId — из InternalInfo), свойства в части
//! `9cd510cd-…` и коллекции объектов по классам: коллекции по GUID класса, объекты — в порядке ChildObjects.
//! Последний элемент `{{1,"",""},{<число>}}` платформа пишет заново при каждой загрузке (у двух загрузок одного XML
//! разный) — пишем 0, сверка его не сравнивает.

use super::desc::check_root;
use super::dom::El;
use super::index::Index;
use super::tpl::{a, l, md_list, s, token, Props};
use crate::brace::{Doc, Node};
use std::collections::BTreeSet;
use crate::xml::common::ZERO;

/// Части записи: (ClassId, вид заголовка, GUID коллекций по классам).
const PARTS: &[(&str, &str, &[&str])] = &[
    (
        "9cd510cd-abfc-11d4-9434-004095e12fc7",
        "main",
        &[
            "09736b02-9cac-4e3f-b4f7-d3e9576ab948", "0c89c792-16c3-11d5-b96b-0050bae0a95d", "0fe48980-252d-11d6-a3c7-0050bae0a776", "0fffc09c-8f4c-47cc-b41c-8d5c5a221d79",
            "102f6202-43fa-40b0-8898-acd3876daacb", "11bdaf85-d5ad-4d91-bb24-aa0eee139052", "15794563-ccec-41f6-a83c-ec5f7b9a5bc1", "24c43748-c938-45d0-8d14-01424a72b11e",
            "30d554db-541e-4f62-8970-a1c6dcfeb2bc", "37f2fa9a-b276-11d4-9435-004095e12fc7", "39bddf6a-0c3c-452b-921c-d99cfa1c2f1b", "3e5404af-6ef8-4c73-ad11-91bd2dfac4c8",
            "3e7bfcc0-067d-11d6-a3c7-0050bae0a776", "46b4cd97-fd13-4eaa-aba2-3bddd7699218", "4e828da6-0f44-4b5b-b1c0-a2b3cfe7bdcc", "58848766-36ea-4076-8800-e91eb49590d7",
            "6e6dc072-b7ac-41e7-8f88-278d25b6da2a", "7dcd43d9-aca5-4926-b549-1842e6a4e8cf", "857c4a91-e5f4-4fac-86ec-787626f1c108", "8657032e-7740-4e1d-a3ba-5dd6e8afb78f",
            "9cd510ce-abfc-11d4-9434-004095e12fc7", "a7641777-7813-45c6-96ef-9d51587a6ac6", "af547940-3268-434f-a3e7-e47d6d2638c3", "c045099e-13b9-4fb6-9d50-fca00202971e",
            "cc9df798-7c94-4616-97d2-7aa0b7bc515e", "d26096fb-7a5d-4df9-af63-47d04771fa9b",
        ],
    ),
    (
        "9fcd25a0-4822-11d4-9414-008048da11f9",
        "zero",
        &[
            "0195e80c-b157-11d4-9435-004095e12fc7", "061d872a-5787-460e-95ac-ed74ea3a3e84", "07ee8426-87f1-11d5-b99c-0050bae0a95d", "13134201-f60b-11d5-a3c7-0050bae0a776",
            "1c57eabe-7349-44b3-b1de-ebfeab67b47d", "2f1a5187-fb0e-4b05-9489-dc5dd6412348", "36a8e346-9aaa-4af9-bdbd-83be3c177977", "4612bd75-71b7-4a5c-8cc5-2b0b65f9fa0d",
            "631b75a0-29e2-11d6-a3c7-0050bae0a776", "82a1b659-b220-4d94-a9bd-14d757b95a48", "b64d9a40-1642-11d6-a3c7-0050bae0a776", "bc587f20-35d9-11d6-a3c7-0050bae0a776",
            "bf845118-327b-4682-b5c6-285d2a0eb296", "cf4abea6-37b2-11d4-940f-008048da11f9", "f6a80749-5ad7-400b-8519-39dc5dff2542",
        ],
    ),
    ("e3687481-0a87-462c-a166-9f34594f9bba", "zero-id", &["238e7e88-3c5f-48b2-8a3b-81ebbecb20ed", "2deed9b8-0056-4ffe-a473-c20a6c32a0bc"]),
    ("9de14907-ec23-4a07-96f0-85521cb6b53b", "id", &["30b100d6-b29f-47ac-aec7-cb8ca8a54767", "f2de87a8-64e5-45eb-a22d-b3aedab050e7"]),
    ("51f2d5d8-ea4d-4064-8892-82951750031e", "zero-id", &["3e63355c-1378-4953-be9b-1deb5fb6bec5", "fcd3404e-1523-48ce-9bc0-ecdb822684a1"]),
    ("e68182ea-4237-4383-967f-90c1e3370bc7", "id", &["5274d9fc-9c3a-4a71-8f5e-a0db8ab23de5"]),
    ("fb282519-d103-4dd3-bc12-cb271d631dfc", "id", &["bf3420b0-f6f9-41a0-b83a-fe9d4ab0b65d"]),
];

/// Свойства (часть `9cd510cd-…`, `{76,…}`); места `$…` — как у описаний объектов. Позиции и постоянные — по записям
/// загрузки БП и 8.5 (в обеих одинаковы); где выгрузка не различает позиции группы (`group` выгрузки), загружается
/// только пустое значение.
const PROPS: &str = "{76,{0,$H},$empty_s:NamePrefix,$t:ScriptVariant,$m:BriefInformation,$m:DetailedInformation,$m:Copyright,\
$m:VendorInformationAddress,$m:ConfigurationInformationAddress,$zero:DefaultInterface,$r:DefaultLanguage,$zero:DefaultStyle,$first_role,\
$b:IncludeHelpInContents,$s:Vendor,$s:Version,$s:UpdateCatalogAddress,$t:DataLockControlMode,$empty_l:AdditionalFullTextSearchDictionaries,\
$t:ObjectAutonumerationMode,$empty_l:Content,$t:DefaultRunMode,$r:CommonSettingsStorage,$r:ReportsUserSettingsStorage,$r:ReportsVariantsStorage,\
$r:FormDataSettingsStorage,$compat,$empty_l:StandaloneConfigurationRestrictionRoles,$b:UseManagedFormInOrdinaryApplication,\
$b:UseOrdinaryFormInManagedApplication,$r:DefaultReportForm,$r:DefaultReportVariantForm,$r:DefaultReportSettingsForm,$purposes,\
$zero:DefaultReportAppearanceTemplate,$zero:DefaultDynamicListSettingsForm,$t:ModalityUseMode,$zero:DefaultSearchForm,$ifc,\
$mdl:DefaultRoles,$perm,$t:SynchronousPlatformExtensionAndAddInCallUseMode,\"\",$extcompat,1,$t:MainClientApplicationWindowMode,\
$zero:DefaultDataHistoryChangeHistoryForm,$zero:DefaultDataHistoryVersionDataForm,$zero:DefaultDataHistoryVersionDifferencesForm,1,\
$zero:DynamicListsUserSettingsStorage,$perm2,$empty_l:MobileApplicationURLs,$mobile,{0},$zero:URLExternalDataStorage,\"\",\
$t:DatabaseTablespacesUseMode,1,$share,1,0,$ifc,$t:ClientApplicationTheme,{0},{0},$t:MainClientApplicationWindowInterfaceVariant,\
$t:ClientApplicationWindowsOpenVariant,$t:Version85InterfaceMigrationMode,$zero:DefaultCollaborationSystemUsersChoiceForm,\
$zero:DefaultConstantsForm,$empty_r:AuxiliaryReportForm,$empty_r:AuxiliaryReportVariantForm,$empty_r:AuxiliaryReportSettingsForm,\
$empty_r:AuxiliaryDynamicListSettingsForm,$empty_r:AuxiliaryDataHistoryChangeHistoryForm,$empty_r:AuxiliaryDataHistoryVersionDataForm}";

/// Списки разрешений мобильного приложения (позиции [40] и [51]): в XML их нет (RequiredMobileApplicationPermissions
/// пусто) — загрузка платформы выводит их из UsedMobileApplicationFunctionalities. Функция перевода в коде не найдена
/// (перечисления — mngcore FUN_180486410 разрешения, FUN_180486e10 функциональности), поэтому таблица снята оракулом:
/// `ibcmd config import` пустого каркаса с каждой функциональностью по одной (26.10, 8.5.1.1529); модель «разрешения —
/// объединение по включенным функциональностям» сверена на 46 наборах (по одной, сочетания 13–15, три случайных,
/// БП, 8.5, Клиент ЭДО) — обе позиции байт в байт.
/// Разрешения каждой функциональности, по порядку `MOBILE`.
const PERM_OF: [&[u8]; 38] = [
    &[29], &[1], &[10, 35, 36], &[7, 30], &[7, 31], &[2], &[3], &[5, 35], &[6, 35], &[8], &[12], &[9], &[13, 16], &[14, 18],
    &[14, 17], &[15, 19], &[15, 20], &[15, 21], &[22], &[23], &[24], &[25], &[26], &[11], &[27], &[28], &[], &[22], &[11], &[32],
    &[22, 23, 33], &[34], &[22], &[23], &[35, 36], &[37], &[38], &[26],
];
/// [51]: разрешения, которые пишутся всегда; 13–15 — только включенные.
const PERM_ALWAYS: &[u8] = &[1, 2, 3, 5, 6, 7, 8, 9, 10, 11, 12, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38];
/// [40] — порядок обхода хэш-таблицы (0 — разрешение 38 «U», в старом перечислении без значения): без 13–15 — `PERM_ORDER0`,
/// с любым из них — `PERM_ORDER` без невключенных из 13–15 (у 29+ элементов 7 и 20 меняются местами).
const PERM_ORDER0: &[u8] = &[28, 0, 25, 30, 24, 22, 21, 19, 18, 17, 12, 27, 10, 31, 26, 9, 8, 7, 20, 29, 16, 6, 5, 3, 2, 11, 23, 1];
const PERM_ORDER: &[u8] = &[28, 0, 25, 30, 24, 22, 21, 19, 18, 17, 13, 12, 27, 10, 31, 26, 9, 14, 8, 20, 7, 29, 16, 6, 15, 5, 3, 2, 11, 23, 1];

/// Позиции [40] и [51] по включенным функциональностям (`used[i]` — по порядку `MOBILE`).
fn permissions(used: &[bool]) -> Result<(Node, Node), String> {
    let on: BTreeSet<u8> = used.iter().zip(PERM_OF).filter(|(u, _)| **u).flat_map(|(_, p)| p.iter().copied()).collect();
    let b = |x: bool| u8::from(x);
    let extra: Vec<u8> = [13, 14, 15].into_iter().filter(|i| on.contains(i)).collect();
    let ids: BTreeSet<u8> = PERM_ALWAYS.iter().copied().chain(extra.iter().copied()).collect();
    let p51: Vec<String> = ids.iter().map(|i| format!("{{{i},{},0}}", b(on.contains(i)))).collect();
    let order: Vec<u8> = if extra.is_empty() { PERM_ORDER0.to_vec() } else { PERM_ORDER.iter().copied().filter(|i| !(13..=15).contains(i) || on.contains(i)).collect() };
    let p40: Vec<String> = order
        .iter()
        .map(|&i| if i == 0 { format!("{{{{\"U\"}},{{\"B\",{}}}}}", b(on.contains(&38))) } else { format!("{{{{\"#\",e4c53f94-e5f7-4a34-8c10-218bd811cae1,{i}}},{{\"B\",{}}}}}", b(on.contains(&i))) })
        .collect();
    let n = |s: String| Doc::parse(&s).map(|mut d| d.roots.remove(0));
    Ok((n(format!("{{{},{}}}", p40.len(), p40.join(",")))?, n(format!("{{2,{},{}}}", p51.len(), p51.join(",")))?))
}

/// Свойства, у которых нет своей позиции в записи (или она не установлена): загружаются только пустыми.
const EMPTY_ONLY: &[&str] = &["AuxiliaryDataHistoryVersionDifferencesForm", "AuxiliaryCollaborationSystemUsersChoiceForm", "Caption", "ShortCaption"];

pub const MOBILE: &[&str] = &[
    "Biometrics", "Location", "BackgroundLocation", "BluetoothPrinters", "WiFiPrinters", "Contacts", "Calendars", "PushNotifications",
    "LocalNotifications", "InAppPurchases", "PersonalComputerFileExchange", "Ads", "NumberDialing", "CallProcessing", "CallLog", "AutoSendSMS",
    "ReceiveSMS", "SMSLog", "Camera", "Microphone", "MusicLibrary", "PictureAndVideoLibraries", "AudioPlaybackAndVibration",
    "BackgroundAudioPlaybackAndVibration", "InstallPackages", "OSBackup", "ApplicationUsageStatistics", "BarcodeScanning", "BackgroundAudioRecording",
    "AllFilesAccess", "Videoconferences", "NFC", "DocumentScanning", "SpeechToText", "Geofences", "IncomingShareRequests",
    "AllIncomingShareRequestsTypesProcessing", "TextToSpeech",
];

pub fn record(doc: &El, x: &Index) -> Result<Vec<u8>, String> {
    check_root(doc)?;
    let c = doc.req("Configuration")?;
    let root_id = c.req_attr("uuid")?;
    let ii = c.req("InternalInfo")?;
    let mut oids = Vec::new();
    for co in ii.elems() {
        if co.name == "xr:PropertyState" && super::extension::adopted(c) {continue;}
        if co.name != "xr:ContainedObject" {
            return Err(format!("строка {}: <{}> в InternalInfo", co.line, co.name));
        }
        oids.push((co.text_of("xr:ClassId")?, co.text_of("xr:ObjectId")?));
    }
    // у новой базы 8.5 (`ibcmd infobase create`) последней части (сервисы интеграции) нет — запись пишется без нее
    if oids.len() + 1 < PARTS.len() || oids.len() > PARTS.len() || oids.iter().zip(PARTS).any(|((c, _), (pc, ..))| c != pc) {
        return Err("InternalInfo: части конфигурации не как в записях БП".into());
    }
    let props = c.req("Properties")?;
    let p = Props { el: props, x, id: &oids[0].1 };
    props.text_of("Name")?;
    let extension=super::extension::adopted(c);
    let pn = if extension {super::extension::configuration_props(c,&oids[0].1,x)?} else {p.fill(PROPS, &mut |q| custom(q, &p))?};
    for t in EMPTY_ONLY.iter().filter(|_|!extension) {
        let e = p.get(t)?;
        if e.elems().next().is_some() || !e.text()?.is_empty() {
            return Err(format!("<{t}>: непустое значение не загружается — позиция в записи не установлена"));
        }
    }
    // коллекции объектов по классам — из ChildObjects
    let co = c.req("ChildObjects")?;
    let mut by_class: std::collections::HashMap<&str, Vec<Node>> = std::collections::HashMap::new();
    for e in co.elems() {
        let name = e.text()?;
        let guid = crate::meta::CLASSES.iter().find(|c| c.1 == e.name).map(|c| c.0).ok_or(format!("строка {}: незнакомый класс <{}>", e.line, e.name))?;
        let f = format!("{}.{name}", e.name);
        by_class.entry(guid).or_default().push(a(x.id(&f).ok_or(format!("строка {}: нет описания {f}", e.line))?));
    }
    let coll = |g: &str, by: &mut std::collections::HashMap<&str, Vec<Node>>| {
        let items = by.remove(g).unwrap_or_default();
        let mut v = vec![a(g), a(items.len().to_string())];
        v.extend(items);
        l(v)
    };
    let mut parts = Vec::new();
    for (k, (class, kind, guids)) in PARTS.iter().enumerate().take(oids.len()) {
        let oid = &oids[k].1;
        let mut v: Vec<Node> = match *kind {
            "main" => vec![a("1"), pn.clone(), a(guids.len().to_string())],
            "zero" => vec![a("6"), l(vec![a("1"), l(vec![l(vec![a("1"), a("0"), a(oid)]), a(ZERO)]), a(guids.len().to_string())])],
            "zero-id" => vec![a("1"), l(vec![a("0"), l(vec![a("1"), a("0"), a(oid)])]), a(guids.len().to_string())],
            _ => vec![a("1"), l(vec![l(vec![a("1"), a("0"), a(oid)])]), a(guids.len().to_string())],
        };
        let target = if *kind == "zero" { v[1].list_mut().unwrap() } else { &mut v };
        for g in guids.iter() {
            target.push(coll(g, &mut by_class));
        }
        parts.push(l(vec![a(*class), l(v)]));
    }
    if let Some(g) = by_class.keys().next() {
        return Err(format!("класс {g}: нет коллекции в записи конфигурации"));
    }
    let mut root = vec![a("2"), l(vec![a(root_id)]), a(oids.len().to_string())];
    root.extend(parts);
    root.push(l(vec![l(vec![a("1"), s(""), s("")]), l(vec![a("0")])]));
    if let Some(u) = doc.unread() {
        return Err(format!("не разобрано: {u}"));
    }
    Ok(Doc::new(true, vec![l(root)]).write()?.into_bytes())
}

const INTERFACE_MODE: &[(&str, &str)] = &[
    ("0", "Version8_2"),
    ("1", "Version8_2EnableTaxi"),
    ("2", "TaxiEnableVersion8_2"),
    ("3", "Taxi"),
    ("4", "TaxiEnableVersion8_5"),
    ("5", "Version8_5"),
    ("6", "Version8_5EnableTaxi"),
];

fn version(e: &El) -> Result<Node, String> {
    let t = e.text()?;
    let v: Vec<u32> = t.strip_prefix("Version").ok_or(format!("<{}>: {t}", e.name))?.split('_').map(|p| p.parse().map_err(|_| format!("<{}>: {t}", e.name))).collect::<Result<_, _>>()?;
    match v[..] {
        [a1, b, c] if b < 100 && c < 100 => a1.checked_mul(10000).and_then(|n| n.checked_add(b * 100 + c)).map(|n| a(n.to_string())).ok_or(format!("<{}>: {t}", e.name)),
        _ => Err(format!("<{}>: {t}", e.name)),
    }
}

fn custom(q: &str, p: &Props) -> Result<Vec<Node>, String> {
    let (k, tag) = q.split_once(':').unwrap_or((q, ""));
    let empty = |tag: &str, n: Node| -> Result<Vec<Node>, String> {
        let e = p.get(tag)?;
        if e.elems().next().is_some() || !e.text()?.is_empty() {
            return Err(format!("<{tag}>: непустое значение не загружается — позиция в записи не установлена"));
        }
        Ok(vec![n])
    };
    Ok(match k {
        "empty_s" => empty(tag, s(""))?,
        "empty_l" => empty(tag, l(vec![a("0"), a("0")]))?,
        "empty_r" | "zero" => empty(tag, a(ZERO))?,
        "first_role" => {
            let e = p.get("DefaultRoles")?;
            let first = e.elems().next().map(|r| super::tpl::reference(p.x, r)).transpose()?;
            vec![first.unwrap_or(a(ZERO))]
        }
        "compat" => vec![version(p.get("CompatibilityMode")?)?],
        "extcompat" => vec![version(p.get("ConfigurationExtensionCompatibilityMode")?)?],
        "ifc" => vec![token(p.get("InterfaceCompatibilityMode")?, INTERFACE_MODE)?],
        "purposes" => {
            let mut v = vec![a("0")];
            for e in p.get("UsePurposes")?.elems() {
                if e.name != "v8:Value" || e.req_attr("xsi:type")? != "app:ApplicationUsePurpose" {
                    return Err(format!("строка {}: UsePurposes: <{}>", e.line, e.name));
                }
                v.push(l(vec![s("#"), a("1708fdaa-cbce-4289-b373-07a5a74bee91"), token(e, &[("1", "PlatformApplication"), ("2", "MobilePlatformApplication")])?]));
            }
            v[0] = a((v.len() - 1).to_string());
            vec![l(v)]
        }
        "mdl" => vec![md_list(p.x, p.get(tag)?, "xr:Item", &|r| r)?],
        "perm" | "perm2" => {
            let used: Vec<bool> = p.get("UsedMobileApplicationFunctionalities")?.elems().map(|it| it.req("app:use").and_then(super::tpl::boolean).map(|b| b.atom() == Some("1"))).collect::<Result<_, _>>()?;
            if used.len() != MOBILE.len() {
                return Err("UsedMobileApplicationFunctionalities: не полный список".into());
            }
            let (perm, perm2) = permissions(&used)?;
            if k == "perm" { empty("RequiredMobileApplicationPermissions", perm)? } else { vec![perm2] }
        }
        "mobile" => {
            let e = p.get("UsedMobileApplicationFunctionalities")?;
            let mut v = vec![a("2"), a("0")];
            let items: Vec<&El> = e.elems().collect();
            if items.len() != MOBILE.len() {
                return Err("UsedMobileApplicationFunctionalities: не полный список".into());
            }
            for (i, (it, name)) in items.iter().zip(MOBILE).enumerate() {
                if it.text_of("app:functionality")? != *name {
                    return Err(format!("строка {}: функциональность не по порядку платформы", it.line));
                }
                let code = if i < 28 { i } else { i + 4 };
                v.push(l(vec![a(code.to_string()), super::tpl::boolean(it.req("app:use")?)?]));
            }
            v[1] = a(MOBILE.len().to_string());
            v.push(a("0"));
            vec![l(v)]
        }
        "share" => {
            let mut v = vec![a("0")];
            for it in p.get("AllowedIncomingShareRequestTypes")?.elems() {
                if it.name != "v8:Value" || it.req_attr("xsi:type")? != "app:AllowedIncomingShareRequestType" {
                    return Err(format!("строка {}: AllowedIncomingShareRequestTypes: <{}>", it.line, it.name));
                }
                let pv = it.req("app:processingVariant")?;
                let body = l(vec![
                    a("0"),
                    s(it.text_of("app:mime")?),
                    s(it.text_of("app:uti")?),
                    s(it.text_of("app:ext")?),
                    super::attr::value(p.x, pv).and_then(|n| n.get(1).cloned().ok_or("число".to_string()))?,
                    super::tpl::boolean(it.req("app:isCustom")?)?,
                ]);
                v.push(l(vec![s("#"), a("f251d17e-94e0-4f9b-974e-d642cf9cb6e4"), body]));
            }
            v[0] = a((v.len() - 1).to_string());
            vec![l(v)]
        }
        _ => {
            return Err(format!("шаблон: неизвестное место ${q}"));
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn permissions_synthetic() {
        // Synthetic enabled flags: Biometrics and PictureAndVideoLibraries.
        let mut used = vec![false; 38];
        used[0] = true;
        used[21] = true;
        let (p40, p51) = permissions(&used).unwrap();
        let s = |n: &Node| crate::brace::write(n).unwrap().replace(['\r', '\n'], "");
        assert!(s(&p51).starts_with("{2,34,{1,0,0},{2,0,0},{3,0,0},{5,0,0},{6,0,0},{7,0,0},{8,0,0}"), "{}", s(&p51));
        assert!(s(&p40).starts_with("{28,{{\"#\",e4c53f94-e5f7-4a34-8c10-218bd811cae1,28},{\"B\",0}},{{\"U\"},{\"B\",0}},{{\"#\",e4c53f94-e5f7-4a34-8c10-218bd811cae1,25},{\"B\",1}}"), "{}", s(&p40));
        // 13 (NumberDialing): 29 элементов, 13 перед 12, 20 перед 7
        let mut u = vec![false; 38];
        u[12] = true;
        let (p40, _) = permissions(&u).unwrap();
        assert!(s(&p40).starts_with("{29,"));
        assert!(s(&p40).contains(",8},{\"B\",0}},{{\"#\",e4c53f94-e5f7-4a34-8c10-218bd811cae1,20}"));
    }
}
