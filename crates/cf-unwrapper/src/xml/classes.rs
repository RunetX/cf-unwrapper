//! Описания объектов `<Класс>/<Имя>.xml`: свойства по позициям в скобочной записи объекта.
//! Позиции и значения получены сверкой скобок с выгрузкой платформы по всем объектам БП 3.0.171
//! (таблица — docs/research/xml-export.md).

use super::common::*;
use super::w::W;
use crate::brace::Node;
use crate::meta::{Config, Item};

/// Откуда берется свойство. Пути — от корня записи объекта.
pub enum P {
    /// Во всем эталоне одно значение, позиция в скобках не найдена.
    Const(&'static str),
    Bool(&'static [usize]),
    Tok(&'static [usize], &'static [(&'static str, &'static str)]),
    /// Как Tok, но позиции нет (запись старой версии) — это значение.
    TokOr(&'static [usize], &'static [(&'static str, &'static str)], &'static str),
    /// uuid → полное имя (`CommonForm.Имя`)
    Ref(&'static [usize]),
    Ml(&'static [usize]),
    Str(&'static [usize]),
    Type(&'static [usize]),
    /// Свойство реквизита из блока `{27,...}` по этому пути (имя свойства — тег), см. [`super::attr`].
    Attr(&'static [usize]),
    Custom(fn(&mut W, &Index, &Node)),
}

pub fn props(w: &mut W, x: &Index, root: &Node, table: &[(&str, P)]) {
    for (tag, p) in table {
        match p {
            P::Const(v) => w.text(tag, v),
            // позиции нет в записях старого формата — значение по умолчанию (0)
            P::Bool(path) => token_or0(w, tag, root, path, &[("0", "false"), ("1", "true")]),
            P::Tok(path, map) => token_or0(w, tag, root, path, map),
            P::TokOr(path, map, default) => match root.at(path) {
                Some(_) => token(w, tag, root, path, map),
                None => w.text(tag, default),
            },
            P::Ref(path) => reference(w, tag, x, root, path),
            P::Ml(path) => mlstring(w, tag, root.at(path)),
            P::Str(path) => w.text(tag, string(root, path)),
            P::Type(path) => match root.at(path) {
                Some(p) => type_desc(w, tag, x, p),
                None => w.text(tag, "\u{1}нет описания типа"),
            },
            P::Attr(path) => match root.at(path) {
                Some(b) => super::attr::prop(w, x, b, tag),
                None => w.text(tag, "\u{1}нет блока реквизита"),
            },
            P::Custom(f) => f(w, x, root),
        }
    }
}

const COMMON_MODULE: &[(&str, P)] = &[
    ("Global", P::Bool(&[1, 6])),
    ("ClientManagedApplication", P::Bool(&[1, 7])),
    ("Server", P::Bool(&[1, 3])),
    ("ExternalConnection", P::Bool(&[1, 4])),
    ("ClientOrdinaryApplication", P::Bool(&[1, 2])),
    ("ServerCall", P::Bool(&[1, 9])),
    // в эталоне всегда false; по порядку флагов — позиция 5
    ("Privileged", P::Bool(&[1, 5])),
    ("ReturnValuesReuse", P::Tok(&[1, 8], &[("0", "DontUse"), ("1", "DuringRequest"), ("2", "DuringSession")])),
];

const CONSTANT: &[(&str, P)] = &[
    ("Type", P::Type(&[1, 1, 1, 2])),
    ("UseStandardCommands", P::Bool(&[1, 7])),
    ("DefaultForm", P::Ref(&[1, 10])),
    ("ExtendedPresentation", P::Ml(&[1, 8])),
    ("Explanation", P::Ml(&[1, 9])),
    ("PasswordMode", P::Attr(&[1, 1])),
    ("Format", P::Attr(&[1, 1])),
    ("EditFormat", P::Attr(&[1, 1])),
    ("ToolTip", P::Attr(&[1, 1])),
    ("MarkNegatives", P::Attr(&[1, 1])),
    ("Mask", P::Attr(&[1, 1])),
    ("MultiLine", P::Attr(&[1, 1])),
    ("ExtendedEdit", P::Attr(&[1, 1])),
    ("MinValue", P::Attr(&[1, 1])),
    ("MaxValue", P::Attr(&[1, 1])),
    ("FillChecking", P::Attr(&[1, 1])),
    ("ChoiceFoldersAndItems", P::Attr(&[1, 1])),
    ("ChoiceParameterLinks", P::Attr(&[1, 1])),
    ("ChoiceParameters", P::Attr(&[1, 1])),
    ("QuickChoice", P::Attr(&[1, 1])),
    ("ChoiceForm", P::Attr(&[1, 1])),
    ("LinkByType", P::Attr(&[1, 1])),
    ("ChoiceHistoryOnInput", P::Attr(&[1, 1])),
    ("DataLockControlMode", P::Tok(&[1, 6], &[("0", "Automatic"), ("1", "Managed"), ("2", "AutomaticAndManaged")])),
    ("DataHistory", P::Tok(&[1, 12], &[("0", "DontUse"), ("1", "Use")])),
    // в эталоне всегда 0; позиций нет у констант старого формата
    ("UpdateDataHistoryImmediatelyAfterWrite", P::Bool(&[1, 15])),
    ("ExecuteAfterWriteDataHistoryVersionProcessing", P::Bool(&[1, 16])),
];

const COMMON_ATTRIBUTE: &[(&str, P)] = &[
    ("Type", P::Type(&[1, 1, 1, 2])),
    ("PasswordMode", P::Attr(&[1, 1])),
    ("Format", P::Attr(&[1, 1])),
    ("EditFormat", P::Attr(&[1, 1])),
    ("ToolTip", P::Attr(&[1, 1])),
    ("MarkNegatives", P::Attr(&[1, 1])),
    ("Mask", P::Attr(&[1, 1])),
    ("MultiLine", P::Attr(&[1, 1])),
    ("ExtendedEdit", P::Attr(&[1, 1])),
    ("MinValue", P::Attr(&[1, 1])),
    ("MaxValue", P::Attr(&[1, 1])),
    ("FillFromFillingValue", P::Attr(&[1, 1])),
    ("FillValue", P::Attr(&[1, 1])),
    ("FillChecking", P::Attr(&[1, 1])),
    ("ChoiceFoldersAndItems", P::Attr(&[1, 1])),
    ("ChoiceParameterLinks", P::Attr(&[1, 1])),
    ("ChoiceParameters", P::Attr(&[1, 1])),
    ("QuickChoice", P::Attr(&[1, 1])),
    ("CreateOnInput", P::Attr(&[1, 1])),
    ("ChoiceForm", P::Attr(&[1, 1])),
    ("LinkByType", P::Attr(&[1, 1])),
    ("ChoiceHistoryOnInput", P::Attr(&[1, 1])),
    ("Content", P::Custom(ca_content)),
    ("AutoUse", P::Tok(&[1, 6], &[("0", "Use"), ("1", "DontUse")])),
    ("DataSeparation", P::Tok(&[1, 5], SEPARATE)),
    ("SeparatedDataUse", P::Tok(&[1, 12], &[("0", "Independently"), ("1", "IndependentlyAndSimultaneously")])),
    ("DataSeparationValue", P::Ref(&[1, 7, 1])),
    ("DataSeparationUse", P::Ref(&[1, 8, 1])),
    ("ConditionalSeparation", P::Ref(&[1, 9, 1])),
    // в эталоне три флага ниже всегда равны друг другу — какой где, не проверено
    ("UsersSeparation", P::Tok(&[1, 10], SEPARATE_INV)),
    ("AuthenticationSeparation", P::Tok(&[1, 11], SEPARATE_INV)),
    ("ConfigurationExtensionsSeparation", P::Tok(&[1, 13], SEPARATE_INV)),
    ("Indexing", P::Tok(&[1, 3], INDEXING)),
    ("FullTextSearch", P::Tok(&[1, 4], &[("0", "DontUse"), ("1", "Use")])),
    ("DataHistory", P::Tok(&[1, 14], &[("0", "DontUse"), ("1", "Use")])),
];

const SEPARATE: &[(&str, &str)] = &[("0", "Separate"), ("1", "DontUse")];
const SEPARATE_INV: &[(&str, &str)] = &[("0", "DontUse"), ("1", "Separate")];
const INDEXING: &[(&str, &str)] = &[("0", "DontIndex"), ("1", "Index"), ("2", "IndexWithAdditionalOrder")];

/// Состав общего реквизита `{3,N,<uuid объекта>,{2,<использование>,<uuid условного разделения>},...}`.
fn ca_content(w: &mut W, x: &Index, n: &Node) {
    let l = n.at(&[1, 2]).map(Node::list).unwrap_or_default();
    let items = l.get(2..).unwrap_or_default();
    if items.is_empty() {
        w.text("Content", "");
        return;
    }
    w.open("Content", &[]);
    for c in items.chunks(2) {
        let u = c.get(1);
        w.open("xr:Item", &[]);
        w.text("xr:Metadata", &x.name(c[0].atom().unwrap_or_default()));
        let use_ = u.and_then(|u| u.get(1)).and_then(Node::atom).unwrap_or_default();
        // 0 в эталоне не встречается (вероятно Auto) — с пометкой, чтобы сверка показала
        let use_ = match use_ {
            "1" => "Use".to_string(),
            "2" => "DontUse".to_string(),
            s => format!("\u{1}{s}"),
        };
        w.text("xr:Use", &use_);
        w.text("xr:ConditionalSeparation", &x.name(u.and_then(|u| u.get(2)).and_then(Node::atom).unwrap_or_default()));
        w.end();
    }
    w.end();
}

const SESSION_PARAMETER: &[(&str, P)] = &[("Type", P::Type(&[1, 1, 2]))];

const DEFINED_TYPE: &[(&str, P)] = &[("Type", P::Type(&[1, 4]))];

const LANGUAGE: &[(&str, P)] = &[("LanguageCode", P::Str(&[1, 2]))];
/// Стиль: в описании только имя, синоним и комментарий; значения — Ext/Style.xml.
const STYLE: &[(&str, P)] = &[];

const ROLE: &[(&str, P)] = &[];

const ENUM: &[(&str, P)] = &[
    ("UseStandardCommands", P::Bool(&[1, 6])),
    ("StandardAttributes", P::Custom(enum_standard)),
    // в эталоне всегда {0,{0}}
    ("Characteristics", P::Custom(|w, x, n| super::classes_refs::characteristics(w, x, n.at(&[1, 19])))),
    ("QuickChoice", P::Bool(&[1, 12])),
    ("ChoiceMode", P::Tok(&[1, 11], &[("0", "FromForm"), ("1", "QuickChoice"), ("2", "BothWays")])),
    ("DefaultListForm", P::Ref(&[1, 9])),
    ("DefaultChoiceForm", P::Ref(&[1, 10])),
    ("AuxiliaryListForm", P::Ref(&[1, 13])),
    ("AuxiliaryChoiceForm", P::Ref(&[1, 14])),
    ("ListPresentation", P::Ml(&[1, 15])),
    ("ExtendedListPresentation", P::Ml(&[1, 16])),
    ("Explanation", P::Ml(&[1, 17])),
    ("ChoiceHistoryOnInput", P::Tok(&[1, 20], &[("0", "Auto"), ("1", "DontUse")])),
];

const SETTINGS_STORAGE: &[(&str, P)] = &[
    ("DefaultSaveForm", P::Ref(&[1, 5])),
    ("DefaultLoadForm", P::Ref(&[1, 4])),
    // дополнительные формы в эталоне пусты; позиции — по аналогии с основными
    ("AuxiliarySaveForm", P::Ref(&[1, 7])),
    ("AuxiliaryLoadForm", P::Ref(&[1, 6])),
];

const FUNCTIONAL_OPTION: &[(&str, P)] = &[
    ("Location", P::Ref(&[1, 2])),
    ("PrivilegedGetMode", P::Bool(&[1, 4])),
    ("Content", P::Custom(fo_content)),
];

const EVENT_SUBSCRIPTION: &[(&str, P)] = &[
    ("Source", P::Type(&[1, 2])),
    ("Event", P::Custom(es_event)),
    ("Handler", P::Custom(es_handler)),
];

/// `"OnWrite_ПриЗаписи"` → `OnWrite`.
fn es_event(w: &mut W, _: &Index, n: &Node) {
    let s = string(n, &[1, 3]);
    w.text("Event", s.split('_').next().unwrap_or_default());
}

/// uuid общего модуля и имя процедуры → `CommonModule.Имя.Процедура`.
fn es_handler(w: &mut W, x: &Index, n: &Node) {
    let m = x.name(atom(n, &[1, 4]));
    let p = string(n, &[1, 5]);
    w.text("Handler", &if m.is_empty() { String::new() } else { format!("{m}.{p}") });
}

/// Состав функциональной опции `{0,N,{"#",3ea29ea5-…,{0,<ссылка>}},...}` → `<xr:Object>`.
/// Ссылку на объект, которого нет в конфигурации, писатель FuncOptionContentType (backend FUN_180e83bb0) пропускает:
/// имя берет FUN_180e3b8b0 с флагом 0, и FUN_180e3e7f0 без имени не подставляет uuid (в отличие от MDListType, см. [`md_refs`]).
pub(super) fn fo_content(w: &mut W, x: &Index, n: &Node) {
    let items = n.at(&[1, 3]).map(Node::list).unwrap_or_default().get(2..).unwrap_or_default();
    // Index::name не нашел объект — вернул сам uuid
    let names: Vec<String> = items
        .iter()
        .filter_map(|it| {
            let id = md_ref(it).unwrap_or_default();
            let name = x.name(id);
            (id.is_empty() || name != id).then_some(name)
        })
        .collect();
    if names.is_empty() {
        w.text("Content", "");
        return;
    }
    w.open("Content", &[]);
    for name in names {
        w.text("xr:Object", &name);
    }
    w.end();
}

const FUNCTIONAL_OPTIONS_PARAMETER: &[(&str, P)] = &[("Use", P::Custom(fop_use))];

/// `{0,N,<ссылка>...}` → `<xr:Item xsi:type="xr:MDObjectRef">`.
fn fop_use(w: &mut W, x: &Index, n: &Node) {
    md_refs(w, "Use", "xr:Item", &[("xsi:type", "xr:MDObjectRef")], x, n.at(&[1, 2]));
}

/// Список `{0,N,элементы...}`, в каждом элементе ссылка на объект метаданных.
/// Ссылку на объект, которого нет в конфигурации, писатель MDListType (backend FUN_180e81330) пишет uuid:
/// FUN_180e3e7f0 подставляет его по флагу сериализатора vtbl+200 — в выгрузке он поднят (состав подсистем эталонов БП, ЭДО).
pub fn md_refs(w: &mut W, tag: &str, item: &str, attrs: &[(&str, &str)], x: &Index, n: Option<&Node>) {
    let items = n.map(Node::list).unwrap_or_default().get(2..).unwrap_or_default();
    if items.is_empty() {
        w.text(tag, "");
        return;
    }
    w.open(tag, &[]);
    for it in items {
        w.text_attrs(item, attrs, &x.name(md_ref(it).unwrap_or_default()));
    }
    w.end();
}

/// Описание объекта класса; `None` — класс еще не восстановлен.
/// Таблица свойств класса из этого файла; `None` — класс из другого семейства (палитра свойств — xml/edit.rs).
pub fn table(class: &str) -> Option<&'static [(&'static str, P)]> {
    Some(match class {
        "CommonModule" => COMMON_MODULE,
        "Constant" => CONSTANT,
        "SessionParameter" => SESSION_PARAMETER,
        "DefinedType" => DEFINED_TYPE,
        "Language" => LANGUAGE,
        "Style" => STYLE,
        "Role" => ROLE,
        "Enum" => ENUM,
        "SettingsStorage" => SETTINGS_STORAGE,
        "FunctionalOption" => FUNCTIONAL_OPTION,
        "FunctionalOptionsParameter" => FUNCTIONAL_OPTIONS_PARAMETER,
        "EventSubscription" => EVENT_SUBSCRIPTION,
        "StyleItem" => STYLE_ITEM,
        "CommandGroup" => COMMAND_GROUP,
        "CommonAttribute" => COMMON_ATTRIBUTE,
        "CommonCommand" => COMMON_COMMAND,
        "Form" => super::classes_nested::FORM,
        "Template" => super::classes_nested::TEMPLATE,
        _ => return None,
    })
}

pub fn descriptor(cfg: &Config, x: &Index, class: &str, it: &Item) -> Option<Vec<u8>> {
    let table = match (table(class), class) {
        (Some(t), _) => t,
        (None, "InformationRegister" | "AccumulationRegister" | "AccountingRegister" | "CalculationRegister" | "DocumentJournal" | "Sequence" | "DocumentNumerator") => {
            return super::classes_regs::descriptor(cfg, x, class, it)
        }
        (None, c) => match super::classes_refs::table(c) {
            Some(t) => t,
            None => return super::classes_tools::descriptor(cfg, x, class, it),
        },
    };
    let mut root = object_brace(cfg, it)?;
    if class == "Catalog" {
        super::classes_refs::upgrade_old_catalog(&mut root);
    }
    let mut w = W::metadata();
    w.open(class, &[("uuid", &it.header.id)]);
    internal_info(&mut w, class, &it.header.id, &it.header.name, &root);
    w.open("Properties", &[]);
    if let Some(h) = head(&root) {
        name_synonym_comment(&mut w, h);
    }
    props(&mut w, x, &root, table);
    w.end();
    child_objects(&mut w, x, class, it, &root);
    Some(w.finish())
}

/// Значения перечисления — полностью; формы, макеты, команды — именами.
fn child_objects(w: &mut W, x: &Index, class: &str, it: &Item, root: &Node) {
    if super::classes_refs::child_objects(w, x, class, it, root) {
        return;
    }
    if !matches!(class, "Enum" | "SettingsStorage") {
        return;
    }
    let nested = ["Form", "Template", "Command"];
    let own: Vec<&Item> = it.children.iter().filter(|c| !nested.contains(&c.kind)).collect();
    let named: Vec<&Item> = nested.iter().flat_map(|k| it.children.iter().filter(move |c| c.kind == *k)).collect();
    if own.is_empty() && named.is_empty() {
        w.text("ChildObjects", "");
        return;
    }
    w.open("ChildObjects", &[]);
    for c in own {
        w.open(c.kind, &[("uuid", &c.header.id)]);
        w.open("Properties", &[]);
        if let Some(h) = head_of(root, &c.header.id) {
            name_synonym_comment(w, h);
        }
        if c.kind == "EnumValue" {
            let color=super::classes_tools::element(root,&c.header.id).and_then(|e|e.at(&[0,2]));
            w.text("Color",&match color {Some(c) if c.get(1).and_then(Node::atom)!=Some("4")=>super::ui::color(c),_=>"auto".into()});
        }
        w.end();
        w.end();
    }
    for c in named {
        match c.kind {
            "Command" => super::classes_tools::command(w, x, c, root),
            k => w.text(k, &c.header.name),
        }
    }
    w.end();
}

/// Стандартные реквизиты перечисления — в (1,18): `{0}` — нет элемента, `{1,{1,N,...}}` — есть.
fn enum_standard(w: &mut W, x: &Index, n: &Node) {
    if let Some(sa) = n.at(&[1, 18, 1]) {
        let owner = x.name(head(n).and_then(|h| h[1].get(2)).and_then(Node::atom).unwrap_or_default());
        super::standard::standard_attributes(w, x, "Enum", &owner, sa);
    }
}

const STYLE_ITEM: &[(&str, P)] = &[("Type", P::Tok(&[1, 1], &[("0", "Color"), ("1", "Font"), ("2", "Border")])), ("Value", P::Custom(style_item_value))];

fn style_item_value(w: &mut W, _: &Index, n: &Node) {
    match n.at(&[1, 2]) {
        Some(v) => super::ui::style_value(w, "Value", v),
        None => w.text("Value", "\u{1}нет значения"),
    }
}

const COMMAND_GROUP: &[(&str, P)] = &[
    ("Representation", P::Tok(&[1, 3], REPRESENTATION)),
    ("ToolTip", P::Ml(&[1, 4])),
    ("Picture", P::Custom(|w, x, n| super::ui::picture(w, "Picture", x, n.at(&[1, 1])))),
    // флаги, а не коды EDT: 2 FormNavigationPanel — по каркасу
    ("Category", P::Tok(&[1, 2], &[("1", "NavigationPanel"), ("2", "FormNavigationPanel"), ("4", "ActionsPanel"), ("8", "FormCommandBar")])),
];

/// Поведение при недоступности основного сервера (Common.xcore OnMainServerUnavalableBehavior).
pub const ON_MAIN_SERVER_UNAVAILABLE: &[(&str, &str)] = &[("0", "Auto"), ("1", "MakeDisable"), ("2", "DontChangeBehavior")];

/// 0 Text — по командам обработок УТ 11.5.27 (в БП не встречается).
pub const REPRESENTATION: &[(&str, &str)] = &[("0", "Text"), ("1", "Picture"), ("2", "PictureAndText"), ("3", "Auto")];

/// Стандартные группы команд: uuid → имя.
pub const STD_COMMAND_GROUPS: &[(&str, &str)] = &[
    ("77ea1b8f-dd79-4717-9dba-5628e7f348cf", "NavigationPanelOrdinary"),
    ("1af6d528-0b86-4fba-ab95-bd7475db03ba", "NavigationPanelImportant"),
    ("bc80566a-86a5-4e87-acd4-872239385a2e", "NavigationPanelSeeAlso"),
    ("aabb34e1-98c1-4bd0-bf7f-243f95437b44", "ActionsPanelTools"),
    ("4f499c31-050b-47c5-aa84-d0366c0a0da8", "ActionsPanelCreate"),
    ("5b360bff-01a1-49b6-93d2-26e7e8e3a038", "ActionsPanelReports"),
    ("eacad741-96b9-4b3a-bf79-dde9ecead1a1", "FormNavigationPanelGoTo"),
    ("8ab1540c-0bfa-4fa6-a1e1-5d5069efc7d8", "FormNavigationPanelSeeAlso"),
    ("dc11a6be-de1f-4b64-a7a5-9b17bf4ec9f2", "FormNavigationPanelImportant"),
    ("cb50f5c0-8013-4262-93a2-f0db379d6b6b", "FormCommandBarImportant"),
    ("dc2ade0f-383e-4c78-85f2-c0dabc0e2dc0", "FormCommandBarCreateBasedOn"),
];

/// Общая команда: свойства в `{8,...}` по пути (1,1,2).
const COMMON_COMMAND: &[(&str, P)] = &[
    ("Group", P::Custom(cc_group)),
    ("Representation", P::Tok(&[1, 1, 2, 2], REPRESENTATION)),
    ("ToolTip", P::Ml(&[1, 1, 2, 3])),
    ("Picture", P::Custom(|w, x, n| super::ui::picture(w, "Picture", x, n.at(&[1, 1, 2, 1])))),
    ("Shortcut", P::Custom(|w, _, n| w.text("Shortcut", &super::ui::shortcut(n.at(&[1, 1, 2, 5]))))),
    ("IncludeHelpInContents", P::Bool(&[1, 1, 2, 6])),
    ("CommandParameterType", P::Type(&[1, 1, 2, 8])),
    ("ParameterUseMode", P::Tok(&[1, 1, 2, 11], &[("0", "Single"), ("1", "Multiple")])),
    ("ModifiesData", P::Bool(&[1, 1, 2, 10])),
    ("OnMainServerUnavalableBehavior", P::Tok(&[1, 1, 2, 12], ON_MAIN_SERVER_UNAVAILABLE)),
];

fn cc_group(w: &mut W, x: &Index, n: &Node) {
    let id = atom(n, &[1, 1, 2, 7, 1]);
    let name = STD_COMMAND_GROUPS.iter().find(|(g, _)| *g == id).map_or_else(|| x.name(id), |(_, nm)| nm.to_string());
    w.text("Group", &name);
}
