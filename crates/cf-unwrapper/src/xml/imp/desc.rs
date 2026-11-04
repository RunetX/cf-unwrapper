//! Описания объектов `<Класс>/<Имя>.xml` → запись `<uuid>`: шаблон класса ([`super::tpl`]) и значения свойств.
//! Шаблоны — обратные таблицам выгрузки (`classes*.rs`), позиции те же.

use super::dom::El;
use super::index::Index;
use super::tpl::{a, l, md_list, s, Props};
use crate::brace::{Doc, Node};
use crate::xml::w::W;

const FORM_BODY: &str = "{14,$H,$b:IncludeHelpInContents,$t:FormType,$purposes,$t:UseInInterfaceCompatibilityMode}";

/// Шаблон записи описания класса (`owner` — класс владельца вложенной формы); `None` — загрузки еще нет.
fn template(class: &str, owner: Option<&str>) -> Option<String> {
    if class == "Form" {
        // обертка тела формы — по классу владельца (сверено по всем формам БП и 8.5)
        let (pre, post) = match owner? {
            "AccountingRegister" | "BusinessProcess" | "ChartOfAccounts" | "ChartOfCharacteristicTypes" | "Task" => ("{1,", ",0}"),
            // shortcut: критерий отбора — по записям загрузки ERP (ЗадачиПоЭкземпляруБюджета) и БП 3.0.107 (ФайлыВТоме),
            // сериализатор описания формы по классу владельца не прочитан — пересмотреть при первом расхождении
            "AccumulationRegister" | "Catalog" | "ChartOfCalculationTypes" | "Document" | "DocumentJournal" | "Enum" | "ExchangePlan" | "FilterCriterion" | "InformationRegister" | "SettingsStorage" => ("{1,{0,", "},0}"),
            "DataProcessor" | "Report" | "ExternalDataProcessor" => ("{1,{1,{0,", ",$m:ExtendedPresentation}},0}"),
            _ => return None,
        };
        return Some(format!("{pre}{FORM_BODY}{post}"));
    }
    Some(match class {
        "Template" => "{1,{2,$t:TemplateType,$H},0}",
        "StyleItem" => "{1,{3,$t:Type,$style,$H},0}",
        "CommonCommand" => "{1,{2,{1,{2,$id,078a6af8-d22c-4248-9c33-7e90075a3d2c},$ccbody}},0}",
        "CommonAttribute" => "{1,{5,$attr,$ca_content,$t:Indexing,$t:FullTextSearch,$t:DataSeparation,$t:AutoUse,{1,$r:DataSeparationValue},{1,$r:DataSeparationUse},{1,$r:ConditionalSeparation},$t:UsersSeparation,$t:AuthenticationSeparation,$t:SeparatedDataUse,$t:ConfigurationExtensionsSeparation,$t:DataHistory},0}",
        "SettingsStorage" => "{1,{2,{0,$H},$gt:Manager,$gv:Manager,$r:DefaultLoadForm,$r:DefaultSaveForm,$r:AuxiliaryLoadForm,$r:AuxiliarySaveForm},2,$coll:3daea016-69b7-4ed4-9453-127911372fe6:Template,$coll:b8533c0c-2342-4db3-91a2-c2b08cbf6b23:Form}",
        "CommonForm" => "{1,{4,{14,$H,$b:IncludeHelpInContents,$t:FormType,$purposes,$t:UseInInterfaceCompatibilityMode},$m:ExtendedPresentation,$m:Explanation,$b:UseStandardCommands},0}",
        "CommonModule" => "{1,{12,$H,$b:ClientOrdinaryApplication,$b:Server,$b:ExternalConnection,$b:Privileged,$b:Global,$b:ClientManagedApplication,$t:ReturnValuesReuse,$b:ServerCall},0}",
        "Role" => "{1,{6,$H,1,0,0},0}",
        "SessionParameter" => "{1,{1,{2,$H,$T:Type}},0}",
        "DefinedType" => "{1,{0,$gt:DefinedType,$gv:DefinedType,$H,$T:Type},0}",
        "Language" => "{1,{0,$H,$s:LanguageCode},0}",
        "Style" => "{1,{3,$H},0}",
        "XDTOPackage" => "{1,{1,$H,$s:Namespace},0}",
        "CommonPicture" => "{1,{4,$H,$b:AvailabilityForChoice,$b:AvailabilityForAppearance},0}",
        "CommonTemplate" => "{1,{4,$H,$t:TemplateType},0}",
        "EventSubscription" => "{1,{1,$H,$T:Source,$event,$*handler:Handler},0}",
        "ScheduledJob" => "{1,{2,$H,$s:Key,$s:Description,$b:Use,$b:Predefined,$*handler:MethodName,$n:RestartCountOnFailure,$n:RestartIntervalOnFailure},0}",
        "FunctionalOption" => "{1,{2,$H,$r:Location,$fo_content,$b:PrivilegedGetMode},0}",
        "FunctionalOptionsParameter" => "{1,{0,$H,$fop_use},0}",
        "CommandGroup" => "{1,{3,$p:Picture,$t:Category,$t:Representation,$m:ToolTip,{0},$H},0}",
        "Constant" => "{1,{16,$attr,$gt:Manager,$gv:Manager,$gt:ValueManager,$gv:ValueManager,$t:DataLockControlMode,$b:UseStandardCommands,$m:ExtendedPresentation,$m:Explanation,$r:DefaultForm,0,$t:DataHistory,$gt:ValueKey,$gv:ValueKey,$b:UpdateDataHistoryImmediatelyAfterWrite,$b:ExecuteAfterWriteDataHistoryVersionProcessing},0}",
        "Enum" => "{1,{20,$gt:Ref,$gv:Ref,$gt:Manager,$gv:Manager,{0,$H},$b:UseStandardCommands,$gt:List,$gv:List,$r:DefaultListForm,$r:DefaultChoiceForm,$t:ChoiceMode,$b:QuickChoice,$r:AuxiliaryListForm,$r:AuxiliaryChoiceForm,$m:ListPresentation,$m:ExtendedListPresentation,$m:Explanation,$std,$chars,$t:ChoiceHistoryOnInput},4,$coll:33f2e54b-37ce-4a7a-a569-b648d7aa4634:Form,$coll:3daea016-69b7-4ed4-9453-127911372fe6:Template,$cmds:6d8d73a7-ba29-401d-9032-3872ec2d6433,$enum_values}",
        c => return super::refs::template(c).map(str::to_string),
    }
    .to_string())
}

/// Запись описания; ошибка — файл отвергается.
/// `full` — полное имя, если оно не `<Класс>.<Имя>` (вложенная подсистема).
pub fn record(doc: &El, class: &str, owner: Option<&str>, full: Option<&str>, x: &Index) -> Result<Option<Vec<u8>>, String> {
    if doc.peek_child(class).is_some_and(super::extension::adopted) {
        return super::extension::record(doc,class,x).map(Some);
    }
    let Some(tpl) = template(class, owner) else { return Ok(None) };
    check_root(doc)?;
    let obj = doc.req(class)?;
    let uuid = obj.req_attr("uuid")?;
    // у корня .epf заголовок — по ObjectId объекта (ContainedObject), uuid описания — имя записи
    let contained = match class {
        "ExternalDataProcessor" => Some(contained_object(obj, crate::meta::EXTERNAL_DATA_PROCESSOR)?),
        _ => None,
    };
    let id = contained.as_deref().unwrap_or(uuid);
    let props = obj.req("Properties")?;
    let p = Props { el: props, x, id };
    let full = match full {
        Some(f) => f.to_string(),
        None => format!("{class}.{}", props.text_of("Name")?),
    };
    let o = Obj { class, full: &full, el: obj, gen: obj.child("InternalInfo") };
    let n = p.fill(&tpl, &mut |q| if q == "uuid" { Ok(vec![a(uuid)]) } else { custom(q, &p, &o) })?;
    // пустой ChildObjects — так у объектов без вложенных; непустой без загрузки останется непрочитанным
    if let Some(co) = obj.child("ChildObjects") {
        co.text().ok();
    }
    let text = Doc::new(true, vec![n]).write()?;
    if let Some(u) = doc.unread() {
        return Err(format!("не разобрано: {u}"));
    }
    Ok(Some(text.into_bytes()))
}

/// ObjectId из `<InternalInfo><xr:ContainedObject>` с данным ClassId.
pub fn contained_object(obj: &El, class_id: &str) -> Result<String, String> {
    let c = obj.req("InternalInfo")?.all("xr:ContainedObject").find(|c| c.peek_child("xr:ClassId").and_then(El::peek_text).as_deref() == Some(class_id));
    let c = c.ok_or(format!("нет ContainedObject {class_id}"))?;
    c.text_of("xr:ClassId")?;
    c.text_of("xr:ObjectId")
}

/// Объект, описание которого загружается.
pub struct Obj<'a> {
    pub class: &'a str,
    /// `Enum.Имя`
    pub full: &'a str,
    pub el: &'a El,
    pub gen: Option<&'a El>,
}

impl Obj<'_> {
    /// Вложенные элементы вида `kind` в ChildObjects.
    pub fn children<'k>(&'k self, kind: &'k str) -> Vec<&'k El> {
        self.el.child("ChildObjects").map(|co| co.all(kind).collect()).unwrap_or_default()
    }

    /// Коллекция `{<guid>,N,<uuid>…}` форм, макетов: имена из ChildObjects → uuid по индексу.
    pub fn named(&self, x: &Index, guid: &str, kind: &str) -> Result<Node, String> {
        let mut v = vec![a(guid), a("0")];
        for e in self.children(kind) {
            let n = e.text()?;
            let f = format!("{}.{kind}.{n}", self.full);
            v.push(a(x.id(&f).ok_or(format!("нет {f}"))?));
        }
        v[1] = a((v.len() - 2).to_string());
        Ok(l(v))
    }
}

/// События подписок: в записи — английское и русское имя через `_` (собраны по БП 3.0.171 и каркасу 8.5).
const EVENTS: &[&str] = &[
    "BeforeWrite_ПередЗаписью",
    "OnWrite_ПриЗаписи",
    "BeforeDelete_ПередУдалением",
    "FormGetProcessing_ОбработкаПолученияФормы",
    "FillCheckProcessing_ОбработкаПроверкиЗаполнения",
    "Filling_ОбработкаЗаполнения",
    "OnSetNewNumber_ПриУстановкеНовогоНомера",
    "Posting_ОбработкаПроведения",
    "OnCopy_ПриКопировании",
    "OnSetNewCode_ПриУстановкеНовогоКода",
    "UndoPosting_ОбработкаУдаленияПроведения",
    "PresentationFieldsGetProcessing_ОбработкаПолученияПолейПредставления",
    "PresentationGetProcessing_ОбработкаПолученияПредставления",
    "OnSendDataToSlave_ПриОтправкеДанныхПодчиненному",
    "OnSendNodeDataToSlave_ПриОтправкеДанныхУзлаПодчиненному",
    "OnSendDataToMaster_ПриОтправкеДанныхГлавному",
    "OnReceiveDataFromMaster_ПриПолученииДанныхОтГлавного",
    "OnReceiveDataFromSlave_ПриПолученииДанныхОтПодчиненного",
    // пара имен из DLL (accnt.dll и др.: ChoiceDataGetProcessing / ОбработкаПолученияДанныхВыбора)
    "ChoiceDataGetProcessing_ОбработкаПолученияДанныхВыбора",
];

const FO_ITEM: &str = "3ea29ea5-66f6-4e3b-8595-d8940db766a2";

fn custom(q: &str, p: &Props, o: &Obj) -> Result<Vec<Node>, String> {
    if let Some(r) = super::refs::custom(q, p, o) {
        return r;
    }
    match q {
        "event" => {
            let e = p.text("Event")?;
            // пустое событие — обе части пустые
            if e.is_empty() {
                return Ok(vec![s("_")]);
            }
            let full = EVENTS.iter().find(|v| v.split('_').next() == Some(e.as_str())).ok_or(format!("<Event>: незнакомое событие {e}"))?;
            return Ok(vec![s(*full)]);
        }
        "fo_content" => return Ok(vec![md_list(p.x, p.get("Content")?, "xr:Object", &|r| l(vec![s("#"), a(FO_ITEM), l(vec![a("0"), r])]))?]),
        "fop_use" => return Ok(vec![md_list(p.x, p.get("Use")?, "xr:Item", &|r| r)?]),
        "attr" => return Ok(vec![super::attr::block(p, o.full)?]),
        // состав общего реквизита `{3,N,<объект>,{2,<1 Use|2 DontUse>,<условное разделение>},…}`
        "ca_content" => {
            let mut v = vec![a("3"), a("0")];
            for it in p.get("Content")?.elems() {
                if it.name != "xr:Item" {
                    return Err(format!("строка {}: <{}> в Content", it.line, it.name));
                }
                v.push(super::tpl::reference(p.x, it.req("xr:Metadata")?)?);
                let u = super::tpl::token(it.req("xr:Use")?, &[("1", "Use"), ("2", "DontUse")])?;
                v.push(l(vec![a("2"), u, super::tpl::reference(p.x, it.req("xr:ConditionalSeparation")?)?]));
            }
            v[1] = a(((v.len() - 2) / 2).to_string());
            return Ok(vec![l(v)]);
        }
        "style" => return Ok(vec![super::ui::style_value(p.get("Value")?)?]),
        "ccbody" => return Ok(vec![super::refs::command_body(p, p.bool("IncludeHelpInContents")?)?]),
        // назначения `{N,{"#",1708fdaa-…,<1|2>}…}`
        "purposes" => {
            let mut v = vec![a("0")];
            for e in p.get("UsePurposes")?.elems() {
                if e.name != "v8:Value" || e.req_attr("xsi:type")? != "app:ApplicationUsePurpose" {
                    return Err(format!("строка {}: UsePurposes: <{}>", e.line, e.name));
                }
                v.push(l(vec![s("#"), a("1708fdaa-cbce-4289-b373-07a5a74bee91"), super::tpl::token(e, &[("1", "PlatformApplication"), ("2", "MobilePlatformApplication")])?]));
            }
            v[0] = a((v.len() - 1).to_string());
            return Ok(vec![l(v)]);
        }
        // стандартные реквизиты `{1,{1,N,…}}`; нет элемента — `{0}`
        "std" => {
            return Ok(vec![match p.el.child("StandardAttributes") {
                Some(e) => {
                    let mut n = super::standard::attributes(p.x, o.class, o.full, e)?;
                    // ExchangeDate — стандартный реквизит плана обмена при совместимости ≥ 0x139c2 (backend FUN_180517d70:
                    // 7 реквизитов до 0x139b1, 8 до 0x139c2, иначе 9; девятый — ExchangeDate, FUN_180517ec0)
                    if o.class == "ExchangePlan" && p.x.compat.is_none_or(|c| c >= 0x139c2) {
                        super::standard::exchange_date_first(&mut n)?;
                    }
                    l(vec![a("1"), n])
                }
                None => l(vec![a("0")]),
            }])
        }
        // характеристики: пока только пустые
        "chars" => {
            let e = p.get("Characteristics")?;
            if e.elems().next().is_some() || !e.text()?.is_empty() {
                return Err("Characteristics: непустые не загружаются".into());
            }
            return Ok(vec![l(vec![a("0"), l(vec![a("0")])])]);
        }
        "enum_values" => {
            let mut v = vec![a("bee0a08c-07eb-40c0-8544-5c364c171465"), a("0")];
            for e in o.children("EnumValue") {
                let id = e.req_attr("uuid")?;
                let vp = Props { el: e.req("Properties")?, x: p.x, id };
                let color=match vp.text("Color")?.as_str() {"auto"=>l(vec![a("4"),a("4"),l(vec![a("0")]),a("4")]),c=>super::ui::color(c).ok_or(format!("EnumValue: неизвестный цвет {c}"))?};
                v.push(l(vec![l(vec![a("1"), vp.header()?, color]), a("0")]));
            }
            v[1] = a((v.len() - 2).to_string());
            return Ok(vec![l(v)]);
        }
        _ => {}
    }
    // `CommonModule.Имя.Процедура` → uuid модуля и имя процедуры
    if let Some(tag) = q.strip_prefix("handler:") {
        let h = p.text(tag)?;
        if h.is_empty() {
            return Ok(vec![a("00000000-0000-0000-0000-000000000000"), s("")]);
        }
        let (m, proc_) = h.rsplit_once('.').filter(|(m, _)| m.starts_with("CommonModule.")).ok_or(format!("<{tag}>: не процедура общего модуля: {h:?}"))?;
        let id = p.x.id(m).ok_or(format!("<{tag}>: нет модуля {m}"))?;
        return Ok(vec![a(id), s(proc_)]);
    }
    // `coll:<guid>:<вид>` — формы, макеты по именам
    if let Some((guid, kind)) = q.strip_prefix("coll:").and_then(|r| r.split_once(':')) {
        return Ok(vec![o.named(p.x, guid, kind)?]);
    }
    if let Some((k, cat)) = q.split_once(':').filter(|(k, _)| *k == "gt" || *k == "gv") {
        let g = o.gen.ok_or("нет InternalInfo")?.all("xr:GeneratedType").find(|g| g.attr("category") == Some(cat)).ok_or(format!("нет GeneratedType {cat}"))?;
        g.req_attr("name")?;
        let tag = if k == "gt" { "xr:TypeId" } else { "xr:ValueId" };
        let v = g.text_of(tag)?;
        if !crate::brace::is_guid(&v) {
            return Err(format!("{tag}: не GUID {v:?}"));
        }
        // TypeId и ValueId читаются по очереди: второй отметит оставшийся элемент
        return Ok(vec![a(v)]);
    }
    Err(format!("шаблон: неизвестное место ${q}"))
}

/// `<MetaDataObject>` с пространствами имен и версией как у выгрузки платформы: иначе префиксы в тексте
/// (`cfg:`, `v8:`) значили бы другое.
pub fn check_root(doc: &El) -> Result<(), String> {
    thread_local! {
        static STD: Vec<(String, String)> = crate::xml::imp::dom::parse(std::str::from_utf8(&W::metadata().finish()[3..]).unwrap_or_default()).map(|e| e.attrs).unwrap_or_default();
    }
    if doc.name != "MetaDataObject" {
        return Err(format!("корень <{}> вместо <MetaDataObject>", doc.name));
    }
    STD.with(|std| {
        if doc.attrs.len() != std.len() || std.iter().any(|(k, v)| doc.attr(k) != Some(v)) {
            return Err("пространства имен или версия <MetaDataObject> не как у выгрузки 2.21".to_string());
        }
        Ok(())
    })
}
