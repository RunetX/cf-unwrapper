//! Шаблоны записей и значения свойств: запись класса пишется скобками с местами `$вид:Свойство`, значения
//! берутся из `<Properties>` описания. Виды: `b` — булево (0/1), `t` — перечисление по [`TOKENS`], `m` — многоязычная
//! строка, `s` — строка, `r` — ссылка по полному имени (uuid), `T` — описание типов; `$H` — заголовок объекта,
//! `$id` — uuid. Остальные места — свои для класса (замыкание `custom`); `$*имя` вставляет несколько элементов.

use super::dom::El;
use super::index::Index;
use crate::brace::{Doc, Node};
use crate::xml::common::{ANY_TYPES, NS_TYPES, PLATFORM_TYPES, ZERO};

pub fn a(s: impl Into<String>) -> Node {
    Node::Atom(s.into())
}
pub fn s(v: impl Into<String>) -> Node {
    Node::Str(v.into())
}
pub fn l(v: Vec<Node>) -> Node {
    Node::List(v)
}

/// Перечисления XML ↔ числа записи: (свойство, [(число, значение XML)]).
pub const TOKENS: &[(&str, &[(&str, &str)])] = &[
    ("ReturnValuesReuse", &[("0", "DontUse"), ("1", "DuringRequest"), ("2", "DuringSession")]),
    ("TemplateType", crate::xml::classes_tools::TEMPLATE_TYPE),
    ("Representation", crate::xml::classes::REPRESENTATION),
    ("DataLockControlMode", &[("0", "Automatic"), ("1", "Managed"), ("2", "AutomaticAndManaged")]),
    ("DataHistory", &[("0", "DontUse"), ("1", "Use")]),
    ("ChoiceMode", &[("0", "FromForm"), ("1", "QuickChoice"), ("2", "BothWays")]),
    ("ChoiceHistoryOnInput", &[("0", "Auto"), ("1", "DontUse")]),
    ("FormType", &[("0", "Ordinary"), ("1", "Managed")]),
    // свойства конфигурации (коды — как в выгрузке, xml/classes_tools.rs CONFIGURATION)
    ("ScriptVariant", &[("0", "English"), ("1", "Russian")]),
    ("DefaultRunMode", &[("0", "OrdinaryApplication"), ("1", "ManagedApplication"), ("2", "Auto")]),
    ("ObjectAutonumerationMode", &[("0", "AutoFree"), ("1", "NotAutoFree")]),
    ("ModalityUseMode", &[("0", "Use"), ("1", "UseWithWarnings"), ("2", "DontUse")]),
    ("SynchronousPlatformExtensionAndAddInCallUseMode", &[("0", "Use"), ("1", "UseWithWarnings"), ("2", "DontUse")]),
    ("MainClientApplicationWindowMode", &[("0", "Normal"), ("1", "Workplace"), ("2", "FullscreenWorkplace"), ("3", "Kiosk"), ("4", "EmbeddedWorkplace")]),
    ("ClientApplicationTheme", &[("0", "Auto"), ("1", "Light"), ("2", "Dark")]),
    ("MainClientApplicationWindowInterfaceVariant", &[("0", "NavigationLeft"), ("1", "NavigationTop")]),
    ("ClientApplicationWindowsOpenVariant", &[("0", "OpenDataInTabs"), ("1", "OpenDataInDialogs")]),
    ("Version85InterfaceMigrationMode", &[("0", "Use"), ("1", "DontUse")]),
    ("DatabaseTablespacesUseMode", &[("0", "DontUse")]),
    ("Indexing", &[("0", "DontIndex"), ("1", "Index"), ("2", "IndexWithAdditionalOrder")]),
    ("FullTextSearch", &[("0", "DontUse"), ("1", "Use")]),
    ("DataSeparation", &[("0", "Separate"), ("1", "DontUse")]),
    ("AutoUse", &[("0", "Use"), ("1", "DontUse")]),
    ("SeparatedDataUse", &[("0", "Independently"), ("1", "IndependentlyAndSimultaneously")]),
    ("UsersSeparation", &[("0", "DontUse"), ("1", "Separate")]),
    ("AuthenticationSeparation", &[("0", "DontUse"), ("1", "Separate")]),
    ("ConfigurationExtensionsSeparation", &[("0", "DontUse"), ("1", "Separate")]),
    // тип элемента стиля
    ("Type", &[("0", "Color"), ("1", "Font"), ("2", "Border")]),
    ("UseInInterfaceCompatibilityMode", crate::xml::classes_nested::FORM_INTERFACE),
    ("Category", &[("1", "NavigationPanel"), ("2", "FormNavigationPanel"), ("4", "ActionsPanel"), ("8", "FormCommandBar")]),
];

pub type Custom<'a> = dyn FnMut(&str) -> Result<Vec<Node>, String> + 'a;

/// Значения свойств объекта.
pub struct Props<'a> {
    pub el: &'a El,
    pub x: &'a Index,
    pub id: &'a str,
}

impl Props<'_> {
    pub fn get(&self, tag: &str) -> Result<&El, String> {
        self.el.req(tag)
    }

    pub fn text(&self, tag: &str) -> Result<String, String> {
        self.get(tag)?.text()
    }

    pub fn bool(&self, tag: &str) -> Result<Node, String> {
        boolean(self.get(tag)?)
    }

    pub fn tok(&self, tag: &str) -> Result<Node, String> {
        let map = TOKENS.iter().find(|(t, _)| *t == tag).map(|(_, m)| *m).ok_or(format!("нет таблицы значений {tag}"))?;
        token(self.get(tag)?, map)
    }

    pub fn ml(&self, tag: &str) -> Result<Node, String> {
        mlstring(self.get(tag)?)
    }

    pub fn string(&self, tag: &str) -> Result<Node, String> {
        Ok(s(crlf(&self.text(tag)?)))
    }

    pub fn reference(&self, tag: &str) -> Result<Node, String> {
        reference(self.x, self.get(tag)?)
    }

    pub fn types(&self, tag: &str) -> Result<Node, String> {
        type_desc(self.x, self.get(tag)?)
    }

    /// Заголовок `{3,{1,0,<uuid>},"Имя",{синоним},"комментарий",0,0,<нулевой>,0}` (у всех записей БП и 8.5 одинаков).
    pub fn header(&self) -> Result<Node, String> {
        Ok(l(vec![a("3"), l(vec![a("1"), a("0"), a(self.id)]), s(self.text("Name")?), self.ml("Synonym")?, self.string("Comment")?, a("0"), a("0"), a(ZERO), a("0")]))
    }

    /// Стандартное место шаблона.
    pub fn place(&self, p: &str) -> Option<Result<Node, String>> {
        if p == "H" {
            return Some(self.header());
        }
        if p == "id" {
            return Some(Ok(a(self.id)));
        }
        let (kind, tag) = p.split_once(':')?;
        Some(match kind {
            "b" => self.bool(tag),
            "t" => self.tok(tag),
            "m" => self.ml(tag),
            "s" => self.string(tag),
            "r" => self.reference(tag),
            "T" => self.types(tag),
            "n" => self.text(tag).and_then(|t| int(&t).map(a).ok_or(format!("<{tag}>: не целое {t:?}"))),
            "p" => self.get(tag).and_then(|e| picture(self.x, e)),
            _ => return None,
        })
    }

    /// Запись по шаблону: стандартные места, остальные — `custom`.
    pub fn fill(&self, tpl: &str, custom: &mut Custom) -> Result<Node, String> {
        fill(tpl, &mut |p| match self.place(p) {
            Some(r) => r.map(|n| vec![n]),
            None => custom(p),
        })
    }
}

/// Шаблон в скобках → узел; места `$…` заменяются результатом `f` (`$*…` — несколькими элементами).
pub fn fill(tpl: &str, f: &mut Custom) -> Result<Node, String> {
    let t: String = tpl.split_whitespace().collect();
    let mut d = Doc::parse(&t).map_err(|e| format!("шаблон: {e}"))?;
    let mut root = d.roots.remove(0);
    subst(&mut root, f)?;
    Ok(root)
}

fn subst(n: &mut Node, f: &mut Custom) -> Result<(), String> {
    let Node::List(v) = n else { return Ok(()) };
    let mut out = Vec::with_capacity(v.len());
    for mut c in std::mem::take(v) {
        match &c {
            Node::Atom(p) if p.starts_with("$*") => out.extend(f(&p[2..])?),
            Node::Atom(p) if p.starts_with('$') => {
                let r = f(&p[1..])?;
                if r.len() != 1 {
                    return Err(format!("шаблон: место {p} дало {} элементов", r.len()));
                }
                out.extend(r);
            }
            _ => {
                subst(&mut c, f)?;
                out.push(c);
            }
        }
    }
    *v = out;
    Ok(())
}

/// Перевод строки в тексте XML (LF) → CRLF скобочной строки; CRLF файла — `\r\r\n` записи (обратное `w::escape`).
pub fn crlf(t: &str) -> String {
    t.replace('\n', "\r\n")
}

pub fn boolean(e: &El) -> Result<Node, String> {
    match e.text()?.as_str() {
        "true" => Ok(a("1")),
        "false" => Ok(a("0")),
        v => Err(format!("строка {}: <{}> не булево: {v:?}", e.line, e.name)),
    }
}

pub fn token(e: &El, map: &[(&str, &str)]) -> Result<Node, String> {
    let v = e.text()?;
    map.iter().find(|(_, x)| *x == v).map(|(n, _)| a(*n)).ok_or(format!("строка {}: <{}>: незнакомое значение {v:?}", e.line, e.name))
}

/// `<v8:item><v8:lang>ru</v8:lang><v8:content>текст</v8:content></v8:item>…` → `{N,"ru","текст",…}`; пусто — `{0}`.
pub fn mlstring(e: &El) -> Result<Node, String> {
    let mut v = vec![a("0")];
    for it in e.elems() {
        if it.name != "v8:item" {
            return Err(format!("строка {}: в <{}> элемент <{}>", it.line, e.name, it.name));
        }
        v.push(s(it.text_of("v8:lang")?));
        v.push(s(crlf(&it.text_of("v8:content")?)));
    }
    v[0] = a(((v.len() - 1) / 2).to_string());
    Ok(l(v))
}

/// Целое число как пишет платформа (без ведущих нулей и плюса).
pub fn int(t: &str) -> Option<String> {
    let d = t.strip_prefix('-').unwrap_or(t);
    let ok = !d.is_empty() && d.bytes().all(|b| b.is_ascii_digit()) && (d == "0" || !d.starts_with('0')) && t != "-0";
    ok.then(|| t.to_string())
}

const MD_REF: &str = "157fa490-4ce9-11d4-9415-008048da11f9";

/// Ссылка на объект метаданных `{"#",157fa490-…,{1,<uuid>}}` по полному имени.
pub fn md_ref(x: &Index, e: &El) -> Result<Node, String> {
    let id = reference(x, e)?;
    Ok(l(vec![s("#"), a(MD_REF), l(vec![a("1"), id])]))
}

/// Список ссылок `{0,N,<элемент>…}`: элементы `item` по порядку XML.
pub fn md_list(x: &Index, e: &El, item: &str, wrap: &dyn Fn(Node) -> Node) -> Result<Node, String> {
    let mut v = vec![a("0"), a("0")];
    for it in e.elems() {
        if it.name != item {
            return Err(format!("строка {}: в <{}> элемент <{}>", it.line, e.name, it.name));
        }
        if let Some(t) = it.attr("xsi:type") {
            if t != "xr:MDObjectRef" {
                return Err(format!("строка {}: xsi:type {t}", it.line));
            }
        }
        v.push(wrap(md_ref(x, it)?));
    }
    v[1] = a((v.len() - 2).to_string());
    Ok(l(v))
}

/// Картинка `<Picture><xr:Ref>…</xr:Ref><xr:LoadTransparent>…</xr:LoadTransparent>[<xr:TransparentPixel x y/>]</Picture>`
/// → `{4,1,{0,<uuid>}|{-N}|{0},"",<x>,<y>,<прозрачность>,0,""}`; пустая — `{4,0,{0},"",-1,-1,1,0,""}`.
pub fn picture(x: &Index, e: &El) -> Result<Node, String> {
    /// Стандартные картинки сверх таблицы выгрузки (`ui::STD_PICTURES`): сверка картинок кнопок, страниц и полей форм БП.
    const MORE_STD_PICTURES: &[(&str, &str)] = &[
        ("20175a6d-7510-46a8-9077-75a5d0d97802", "AccountingRegister"), ("70f51581-87b6-41cb-a21b-c9dcdcc7fa93", "AccumulationRegister"),
        ("9ef73565-2250-4a35-9fb3-470bd19ca9ca", "AppearanceCrossIcon"), ("97c5a6d5-47ed-43f9-8c8c-10e9903c23d2", "Calendar"),
        ("069b8324-7c51-4c73-a6a8-c06d4fc383b5", "Catalog"), ("2f130057-bb2a-4e22-bba5-e108fac26940", "ChooseValue"),
        ("4ab0e87f-7d9b-4aa8-ac4b-680a78522da8", "CreateFolder"), ("977e831a-0e73-4d60-af51-091a6fa8612e", "CreateListItem"),
        ("984b0a3e-daa2-4a9e-b75c-1f230a6e592a", "DataCompositionConditionalAppearance"), ("a075c3ef-bc4e-4c96-bdad-2245ec09c28e", "DataCompositionDataParameters"),
        ("d90a7482-9a1d-4d3d-ae96-6db440214d96", "DataCompositionFilter"), ("ad8cb448-a6bb-43b4-886a-7d6a8367eef2", "DataCompositionGroupFields"),
        ("efda7350-6cd7-4416-b188-f5ca9baf66c2", "DataCompositionOrder"), ("ee7c4a5b-2d9b-4087-ae3e-947792085f09", "DataCompositionOutputParameters"),
        ("2c732bfa-f734-48bc-a18b-7554db8a3888", "DataCompositionSelection"), ("affb1617-24bc-4170-9c84-0902cc3ef206", "DataCompositionSettingsWizard"),
        ("b68eb29c-2372-46e1-b84e-13843899ccf6", "DataCompositionUserFields"), ("894afc03-9904-465d-b671-f555ffb9b21c", "Document"),
        ("2ef82795-06fe-4365-bd0c-44b486264620", "FilterCriterion"), ("b7c81c62-d6ad-4eae-9cea-0e203182db67", "FormHelp"),
        ("-5", "InputFieldCalendar"), ("-1", "InputFieldSelect"), ("18492a87-2fe4-44af-b218-304897fed020", "MarkToDelete"),
        ("fc4f29e0-d168-4fe0-8e64-e982fabf2595", "Refresh"), ("818ab7d0-4654-4542-bd5e-fd9d1352b5a1", "SaveFile"),
        ("1970a480-9b38-405e-9d9e-8209f3fad5f1", "ScheduledJob"), ("6206a729-16e5-4e32-b53c-122de4e30c8d", "ScheduledJobs"),
        ("9c96aa25-d656-4b3d-ab3e-81d9718da238", "ShowInList"), ("26518e18-e364-475a-8026-e41134658b2a", "SpreadsheetInsertPageBreak"),
        ("3bdc16c8-6a96-4467-9442-a8e4804b3fa2", "SyncContents"), ("894cf65b-4109-4533-a1d7-c87b1fcc80a3", "Write"),
        ("e6fc55a0-3d58-4b15-bdd3-717453929598", "WriteAndClose"),
            ("7a9cd2fd-6372-4342-9a9e-3ebbd754fd83", "AppearanceCheckBox"),
        ("85998f14-805b-4e2b-ba19-9d79b0464042", "AppearanceCheckIcon"),
        ("2721abfb-fbff-4a3a-98ac-b7c9eb29cd85", "AppearanceCircleEmpty"),
        ("c1a61df2-f280-49d0-a8b3-7e5fc6f56ff7", "AppearanceCircleRed"),
        ("b2202798-23e0-4165-9982-24878f432488", "AppearanceCross"),
        ("b39aa431-a32f-4447-984a-45606474c82d", "AppearanceExclamationMarkIcon"),
        ("87d032df-0956-47e9-bead-4e15330f1983", "AppearanceUpArrowGreen"),
        ("0c1f7756-6143-4903-a94c-8f22c85e44de", "Attribute"),
        ("-10", "CheckAll"),
        ("-200", "Clear"),
        ("35bc8caa-f7ce-4158-87da-d9bf785afa39", "DataSearch"),
        ("60643198-e4b2-4c39-9de1-53cca3fff382", "DeleteDirectly"),
        ("5289d9a4-b012-4d54-9bce-50473fe29b57", "DialogExclamation"),
        ("8bdf1079-8fad-4d21-ad7f-4b2e4ecdce3d", "DialogInformation"),
        ("ef27ae9e-7040-4374-b93c-0d276de2ea23", "DialogQuestion"),
        ("83db1f8a-41bd-4016-bdb2-a28e3a8d6dcc", "DialogStop"),
        ("544fdbe8-5956-4512-bc62-93b4c022d291", "ExchangePlan"),
        ("ffab30f1-da11-44b5-b34c-24da22badcf4", "Find"),
        ("31bf709f-3b50-4137-9b51-ebc7fb802a7c", "GotoExternalURL"),
        ("c283cd1c-3187-451d-8ef2-7df55daeef06", "History"),
        ("4b54770b-d069-4c0e-9b17-5cc2a01134d9", "Information"),
        ("-2", "InputFieldClear"),
        ("01ec9d9a-7497-4d88-b93f-066c633a4866", "InputOnBasis"),
        ("549a2c45-4fce-493f-94ee-9a3a4f426551", "ListViewMode"),
        ("78da2c47-172f-4d57-ab52-a06e40548136", "Message"),
        ("928075d1-b90b-416c-b0b2-c3104cf084aa", "Notifications"),
        ("64837726-d2a2-4682-a788-737423e80013", "Picture"),
        ("20ebc47b-f4d9-439c-acd3-fdc624fbac2a", "Post"),
        ("be23a908-fe1b-44df-be94-d0f6e8353abe", "SendMessage"),
        ("03665ff1-3a05-41d1-96d3-04bda2d8ede3", "SpreadsheetInsertComment"),
        ("75a40cc4-c719-4c3f-91ea-fc5787bc34ca", "UserWithAuthentication"),
            ("3c904ff7-1195-4a7c-9a38-7b1f6ca49cce", "Back"),
        ("dcd23a32-5c7c-43f2-9021-80d98128556f", "CheckSyntax"),
        ("479470e0-ea0f-4266-8549-e2b1e8c06534", "ClearFilter"),
        ("448d6f55-d885-496c-870d-d1bd78374745", "CloneListItem"),
        ("f6532868-30b9-44ab-803c-78f0f0b06b02", "CloneObject"),
        ("1377931c-5744-4948-bade-cb35117b5f63", "Close"),
        ("6511326b-20c3-4bf8-8503-c2c2c9072c6c", "CustomizeForm"),
        ("c78c9f3d-e92c-4f38-bb72-d8bd7fa5dbe3", "Dimension"),
        ("b1406535-6cc2-4410-95ea-753556e8460f", "FilterByCurrentValue"),
        ("fc34a694-e99b-4d1c-a526-63f5571bdb09", "Form"),
        ("3d4ad3b1-17de-4cf1-a2e4-0c2c83a5b5c2", "ListViewModeHierarchicalList"),
        ("c757209d-a87f-4410-b1a3-76000178f1f0", "ListViewModeList"),
        ("-4", "MoveDown"),
        ("-8", "MoveLeft"),
        ("-3", "MoveUp"),
        ("785362cb-3756-48ed-87d2-292ded17054a", "OpenFile"),
        ("c2e2d966-5b7f-4699-903b-28a6f50d5471", "OutputList"),
        ("0abdab67-5c90-4296-8168-239d22024d11", "PrintImmediately"),
        ("7df3febb-2640-41b7-ad8b-7a23b7ad4aec", "QueryWizardCreateTempTableDropQuery"),
        ("942e0303-a3ec-4fe8-887c-5aea8516d424", "ReportSettings"),
        ("8f29e0e2-d5e6-41e8-a34d-9a0288156322", "Reread"),
        ("b5a0aaba-3a83-4a71-b6f9-24aae1574681", "SaveReportSettings"),
        ("caf2e58b-ca3d-4b63-82c9-f21f1c9bc9eb", "Setting"),
        ("-11", "UncheckAll"),
    ];
    let tail = |flag: &str, r: Node, px: (String, String), t: &str| l(vec![a("4"), a(flag), r, s(""), a(px.0), a(px.1), a(t), a("0"), s("")]);
    if e.elems().next().is_none() && e.text()?.is_empty() {
        return Ok(tail("0", l(vec![a("0")]), ("-1".into(), "-1".into()), "1"));
    }
    let name = e.text_of("xr:Ref")?;
    let t = boolean(e.req("xr:LoadTransparent")?)?;
    let px = match e.child("xr:TransparentPixel") {
        None => ("-1".to_string(), "-1".to_string()),
        Some(p) => {
            let n = |k: &str| p.req_attr(k).ok().and_then(int).ok_or(format!("строка {}: TransparentPixel {k}", p.line));
            (n("x")?, n("y")?)
        }
    };
    let r = if name == "0" {
        l(vec![a("0")])
    } else if let Some(std) = name.strip_prefix("StdPicture.") {
        let id = crate::xml::ui::STD_PICTURES
            .iter()
            .chain(MORE_STD_PICTURES)
            .chain(crate::xml::form::FORM_STD_PICTURES)
            .chain(super::form::platform::STD_PICTURES)
            .find(|(_, n)| *n == std)
            .map(|(g, _)| *g)
            .ok_or(format!("строка {}: незнакомая стандартная картинка {std}", e.line))?;
        if id.starts_with('-') {
            l(vec![a(id)])
        } else {
            l(vec![a("0"), a(id)])
        }
    } else if let Some(g) = name.strip_prefix("0:").filter(|g| crate::brace::is_guid(g)) {
        // картинка не из конфигурации и не стандартная (выгрузка `picture`: NOT_STD_PICTURES → `0:<uuid>`)
        l(vec![a("0"), a(g)])
    } else {
        l(vec![a("0"), a(x.id(&name).ok_or(format!("строка {}: нет картинки {name}", e.line))?)])
    };
    Ok(tail("1", r, px, t.atom().unwrap_or_default()))
}

/// Картинка элемента формы: ссылка (`xr:Ref`, [`picture`]) или встроенная (`xr:Abs` — файл
/// `Form/Items/<элемент>/<Abs>`, [`super::payload::item_picture`]; `read` читает его по имени). Второе в ответе — имя
/// прочитанного файла (его надо отметить загруженным).
pub fn picture_or_file(x: &Index, e: &El, read: &dyn Fn(&str) -> Result<Vec<u8>, String>) -> Result<(Node, Option<String>), String> {
    if e.peek_child("xr:Abs").is_some() {
        let (n, f) = super::payload::item_picture(e, read)?;
        return Ok((n, Some(f)));
    }
    picture(x, e).map(|n| (n, None))
}

/// Полное имя объекта метаданных → uuid; пусто — нулевой; uuid (объекта нет в конфигурации — так его пишет
/// выгрузка) — как есть.
pub fn reference(x: &Index, e: &El) -> Result<Node, String> {
    let t = e.text()?;
    if t.is_empty() {
        return Ok(a(ZERO));
    }
    if crate::brace::is_guid(&t) {
        return Ok(a(t));
    }
    x.id(&t).map(a).ok_or(format!("строка {}: <{}>: нет объекта {t}", e.line, e.name))
}

/// GUID простых типов (core85: регистрация типов FUN_1801b3410 — kNullTypeString af40a278…, kBooleanTypeString
/// 5d4125ad…, kNumericTypeString b0be78f2…, kStringTypeString 9b6abf8b…, kDateTypeString aae38c48…): по ним простой
/// тип встает среди `{"#",<guid>}` при сортировке описания типов.
const PRIMITIVE_GUIDS: &[(&str, &str)] = &[
    ("L", "af40a278-63bc-478e-91a8-19e0d16b10b5"),
    ("B", "5d4125ad-f6e7-4313-be32-f71d0ab60915"),
    ("N", "b0be78f2-0ee6-4d31-a3bb-77dd32ba5bec"),
    ("S", "9b6abf8b-0173-48e5-b0a0-83b21fcf63c5"),
    ("D", "aae38c48-a877-411c-a6d3-fbaa1f83c4bd"),
];

/// Ключ сортировки элемента описания типов: GUID типа (`{"#",<guid>}` или простого).
fn sort_key(n: &Node) -> Result<&str, String> {
    let k = n.get(0).and_then(Node::str).unwrap_or_default();
    if k == "#" {
        return Ok(n.get(1).and_then(Node::atom).unwrap_or_default());
    }
    PRIMITIVE_GUIDS.iter().find(|p| p.0 == k).map(|p| p.1).ok_or(format!("GUID простого типа {k} не известен"))
}

fn before(x: &Node, y: &Node) -> Result<bool, String> {
    Ok(sort_key(x)? < sort_key(y)?)
}

/// Описание типов `<Type><v8:Type>…</v8:Type>…<v8:…Qualifiers/></Type>` → `{"Pattern",…}`; типы — в порядке GUID.
pub fn type_desc(x: &Index, e: &El) -> Result<Node, String> {
    let at = |el: &El| format!("строка {}: <{}>", el.line, el.name);
    let (mut num, mut st, mut date) = (None, None, None);
    for q in e.elems() {
        match q.name.as_str() {
            "v8:NumberQualifiers" => num = Some(q),
            "v8:StringQualifiers" => st = Some(q),
            "v8:DateQualifiers" => date = Some(q),
            _ => {}
        }
    }
    let mut items: Vec<Node> = Vec::new();
    for t in e.elems().filter(|q| !q.name.ends_with("Qualifiers")) {
        let v = t.text()?;
        let item = match (t.name.as_str(), v.as_str()) {
            ("v8:Type", "xs:boolean") => l(vec![s("B")]),
            ("v8:Type", "v8:Null") => l(vec![s("L")]),
            ("v8:Type", "xs:string") => {
                let q = st.ok_or(format!("{}: нет StringQualifiers", at(e)))?;
                let len = q.text_of("v8:Length")?;
                let var = match q.text_of("v8:AllowedLength")?.as_str() {
                    "Variable" => "1",
                    "Fixed" => "0",
                    o => return Err(format!("{}: AllowedLength {o}", at(q))),
                };
                match (len.as_str(), var) {
                    ("0", "1") => l(vec![s("S")]),
                    ("0", _) => return Err(format!("{}: строка нулевой фиксированной длины не встречена", at(q))),
                    _ => l(vec![s("S"), a(num_text(&len, q)?), a(var)]),
                }
            }
            ("v8:Type", "xs:decimal") => {
                let q = num.ok_or(format!("{}: нет NumberQualifiers", at(e)))?;
                let (d, f) = (num_text(&q.text_of("v8:Digits")?, q)?, num_text(&q.text_of("v8:FractionDigits")?, q)?);
                let sign = match q.text_of("v8:AllowedSign")?.as_str() {
                    "Any" => "0",
                    "Nonnegative" => "1",
                    o => return Err(format!("{}: AllowedSign {o}", at(q))),
                };
                if (d.as_str(), f.as_str(), sign) == ("0", "0", "0") {
                    l(vec![s("N")])
                } else {
                    l(vec![s("N"), a(d), a(f), a(sign)])
                }
            }
            ("v8:Type", "xs:dateTime") => {
                let q = date.ok_or(format!("{}: нет DateQualifiers", at(e)))?;
                match q.text_of("v8:DateFractions")?.as_str() {
                    "DateTime" => l(vec![s("D")]),
                    "Date" => l(vec![s("D"), s("D")]),
                    "Time" => l(vec![s("D"), s("T")]),
                    o => return Err(format!("{}: DateFractions {o}", at(q))),
                }
            }
            (el, v) => l(vec![s("#"), a(type_guid(x, t, el, v)?)]),
        };
        items.push(item);
    }
    // квалификаторы без своего типа: так платформа не выгружает
    for (q, t) in [(num, "xs:decimal"), (st, "xs:string"), (date, "xs:dateTime")] {
        if let Some(q) = q {
            q.touch_all();
            if !e.elems().any(|c| c.name == "v8:Type" && c.text().ok().as_deref() == Some(t)) {
                return Err(format!("{}: квалификаторы без типа {t}", at(q)));
            }
        }
    }
    // вставками: сравнение бывает неопределенным (ошибка)
    let mut sorted: Vec<Node> = Vec::with_capacity(items.len());
    for it in items {
        let mut i = sorted.len();
        while i > 0 && before(&it, &sorted[i - 1])? {
            i -= 1;
        }
        sorted.insert(i, it);
    }
    let mut v = vec![s("Pattern")];
    v.extend(sorted);
    Ok(l(v))
}

fn num_text(t: &str, q: &El) -> Result<String, String> {
    if !t.is_empty() && t.bytes().all(|b| b.is_ascii_digit()) && (t == "0" || !t.starts_with('0')) {
        Ok(t.to_string())
    } else {
        Err(format!("строка {}: <{}>: не число {t:?}", q.line, q.name))
    }
}

/// Имя набора «любая ссылка» для таблицы типов: при совместимости < 0x139c3 тип XDTO зарегистрирован как `AnyRef`
/// (mngbase FUN_18048d960), иначе `AnyIBRef`.
pub(super) fn any_ib_ref<'v>(x: &Index, v: &'v str) -> &'v str {
    if v == "cfg:AnyRef" && x.compat.is_some_and(|c| c < 0x139c3) { "cfg:AnyIBRef" } else { v }
}

/// GUID ссылочного типа или типа платформы по имени из XML.
fn type_guid(x: &Index, t: &El, el: &str, v: &str) -> Result<String, String> {
    let err = || format!("строка {}: <{el}>{v}: незнакомый тип", t.line);
    let v = any_ib_ref(x, v);
    // тип без имени — по GUID (выгрузка `mxl::value_type`)
    if el == "v8:TypeId" {
        return Some(v).filter(|g| crate::brace::is_guid(g)).map(str::to_string).ok_or_else(err);
    }
    // тип со своим пространством имен: `<v8:Type xmlns:d4p1="…">d4p1:Chart</v8:Type>`
    if let Some((p, name)) = v.split_once(':').filter(|(p, _)| t.attrs.iter().any(|(k, _)| *k == format!("xmlns:{p}"))) {
        let ns = t.attr(&format!("xmlns:{p}")).unwrap_or_default();
        // так XDTO пишет ссылочный тип вне описания объекта (предопределенные ПВХ)
        if ns == "http://v8.1c.ru/8.1/data/enterprise/current-config" && el == "v8:Type" {
            return x.types.get(name).map(|(t, _)| t.clone()).ok_or_else(err);
        }
        // типы документов реквизитов форм (GUID — по таблице выгрузки форм)
        const FORM_NS_TYPES: &[(&str, &str, &str)] = &[
            ("e603103e-a318-4edc-a014-b1c6cf94d49f", "http://v8.1c.ru/8.2/data/spreadsheet", "SpreadsheetDocument"),
            ("ebf766b1-f32c-11d3-9851-008048da1252", "http://v8.1c.ru/8.1/data/txtedt", "TextDocument"),
            ("151f8778-e2d0-496a-9f02-d9ffd93b57ec", "http://v8.1c.ru/8.2/data/formatted-document", "FormattedDocument"),
            ("48510817-200c-48c2-9973-06cf90840514", "http://v8.1c.ru/8.3/data/pdf", "PDFDocument"),
            ("3543ef08-3316-4f7e-9447-0cd0a1cbf1d5", "http://v8.1c.ru/8.2/data/chart", "Chart"),
            ("4af83795-fc2a-48cd-9bea-ce665789a62c", "http://v8.1c.ru/8.2/data/graphscheme", "FlowchartContextType"),
            ("43dc7f37-5b1d-42a7-8f28-f545080d0255", "http://v8.1c.ru/8.3/data/planner", "Planner"),
            ("95de81b0-81c3-4936-9dbb-6400e5c90378", "http://v8.1c.ru/8.2/data/geo", "GeographicalSchema"),
            ("3a6e63bf-16aa-42eb-b48c-2fff9670ad2f", "http://v8.1c.ru/8.2/data/chart", "GanttChart"),
        ];
        if el == "v8:Type" {
            if let Some((g, ..)) = FORM_NS_TYPES.iter().find(|(_, n, nm)| *n == ns && *nm == name) {
                return Ok(g.to_string());
            }
            // прочие — по таблице XDTO, снятой с DLL (как у выгрузки `form::platform_type`)
            if let Some((g, ..)) = crate::xml::xdto_table::XDTO.iter().find(|(_, t, n)| *n == ns && t.rsplit(':').next() == Some(name)) {
                return Ok(g.to_string());
            }
        }
        return NS_TYPES.iter().find(|(_, n, _, nm)| *n == ns && *nm == name).map(|(g, ..)| g.to_string()).ok_or_else(err);
    }
    if let Some((g, ..)) = PLATFORM_TYPES.iter().find(|(_, e, val)| *e == el && *val == v) {
        return Ok(g.to_string());
    }
    // типы платформы реквизитов форм (префиксы объявлены корнем; GUID — по таблице выгрузки форм)
    const FORM_TYPES: &[(&str, &str)] = &[
        ("140b5ff4-37b1-4df5-b5ec-a0bfd2b94f8f", "v8ui:FormattedString"),
        ("9cd510c7-abfc-11d4-9434-004095e12fc7", "v8ui:Color"),
        ("9cd510c8-abfc-11d4-9434-004095e12fc7", "v8ui:Font"),
        ("e6f51714-91cb-4dce-94fe-90ae3e3e1ad1", "v8ui:Picture"),
        ("f5c65050-3bbb-11d5-b988-0050bae0a95d", "v8:TypeDescription"),
        ("cab0d12b-3c88-4993-8edc-8c3827cadc7d", "dcsset:SettingsComposer"),
        ("52616226-8ccf-4d1d-a3da-827eeb4f9cf9", "v8ui:VerticalAlign"),
        ("7a0135c7-5b77-4de2-839e-25c5e63a2142", "v8ui:SizeChangeMode"),
        ("f6841c6b-6c71-4c82-ae9e-d08b49db326c", "dcsset:Filter"),
        ("98ea8e5a-b586-442b-b944-6e3447734aa7", "v8:FillChecking"),
        ("872f7198-7083-4e3e-b57e-a2a9802c769e", "ent:AccountType"),
        ("741ae838-6e42-4ac0-b6a4-17e5604b0669", "ent:AccountingRecordType"),
        ("43f9c095-40e8-441a-8fad-20a45798c71b", "v8ui:HorizontalAlign"),
        ("dcbf2698-3c1f-4a22-997f-48070ae9bd64", "dcsset:DataCompositionComparisonType"),
        ("a090004e-b706-453f-aa10-090a77b53757", "dcsset:DataCompositionFieldPlacement"),
        ("913e8016-6e90-47a0-b2a0-4513f4edad61", "dcscor:Field"),
        ("af4a19b5-da3d-406f-be0c-81143e400452", "dcscor:DataCompositionSortDirection"),
        // колонки ФормаНастроекОтчета (как в `form::platform_type` выгрузки)
        ("0e0850cf-0634-414e-85ba-9a88a8bd44c4", "dcscor:DataCompositionGroupType"),
        ("c6a52555-d20f-452c-bfc2-1b53e9a56063", "dcscor:DataCompositionPeriodAdditionType"),
        // любой отчет (основной реквизит общей формы отчета)
        ("1dd6fdb9-553d-40d4-b2d1-c7fc31f497bb", "cfg:ReportObject"),
        // тип «Динамический список» (GUID — в mngcore.dll/mngbase.dll; выгрузка: `form::platform_type`)
        ("65abad24-838b-4987-8b35-ed9e2bd4d9c8", "cfg:DynamicList"),
    ];
    if let Some((g, _)) = FORM_TYPES.iter().find(|(_, val)| el == "v8:Type" && *val == v) {
        return Ok(g.to_string());
    }
    if let Some((g, ..)) = crate::xml::xdto_table::XDTO.iter().find(|(_, t, _)| el == "v8:Type" && *t == v && !v.starts_with("d5p1:")) {
        return Ok(g.to_string());
    }
    if let Some((g, ..)) = ANY_TYPES.iter().find(|(_, e, val)| *e == el && format!("cfg:{val}") == v) {
        return Ok(g.to_string());
    }
    let name = v.strip_prefix("cfg:").ok_or_else(err)?;
    let set = name.starts_with("DefinedType.") || name.starts_with("Characteristic.");
    if set != (el == "v8:TypeSet") {
        return Err(err());
    }
    x.types.get(name).map(|(t, _)| t.clone()).ok_or_else(err)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::xml::imp::dom::parse;

    #[test]
    fn template_and_props() {
        let d = parse("<P><Name>Имя</Name><Synonym><v8:item><v8:lang>ru</v8:lang><v8:content>а\nб</v8:content></v8:item></Synonym><Comment/><G>true</G><R>DuringSession</R></P>").unwrap();
        let x = Index::default();
        let p = Props { el: &d, x: &x, id: "u" };
        let n = p.fill("{1,{12, $H, $b:G, $t:ReturnValuesReuse, $*two},0}", &mut |q| Ok(if q == "two" { vec![a("7"), a("8")] } else { vec![] })).unwrap_err();
        assert!(n.contains("нет <ReturnValuesReuse>"), "{n}");
        let n = p.fill("{1,{12, $H, $b:G, $*two},0}", &mut |q| Ok(if q == "two" { vec![a("7"), a("8")] } else { vec![] })).unwrap();
        assert_eq!(crate::brace::write(&n).unwrap(), "{1,\r\n{12,\r\n{3,\r\n{1,0,u},\"Имя\",\r\n{1,\"ru\",\"а\r\nб\"},\"\",0,0,00000000-0000-0000-0000-000000000000,0},1,7,8},0}");
    }

    #[test]
    fn types_sorted_with_primitives() {
        let mut x = Index::default();
        x.types.insert("CatalogRef.A".into(), ("ffffffff-0000-0000-0000-000000000000".into(), String::new()));
        x.types.insert("CatalogRef.B".into(), ("00000000-0000-0000-0000-000000000001".into(), String::new()));
        let d = parse("<Type><v8:Type>cfg:CatalogRef.A</v8:Type><v8:Type>xs:string</v8:Type><v8:Type>cfg:CatalogRef.B</v8:Type><v8:Type>xs:boolean</v8:Type><v8:StringQualifiers><v8:Length>10</v8:Length><v8:AllowedLength>Variable</v8:AllowedLength></v8:StringQualifiers></Type>").unwrap();
        let n = type_desc(&x, &d).unwrap();
        assert_eq!(crate::brace::write(&n).unwrap().replace("\r\n", ""), "{\"Pattern\",{\"#\",00000000-0000-0000-0000-000000000001},{\"B\"},{\"S\",10,1},{\"#\",ffffffff-0000-0000-0000-000000000000}}");
        assert_eq!(d.unread(), None);
        x.types.insert("CatalogRef.C".into(), ("9c000000-0000-0000-0000-000000000000".into(), String::new()));
        let d = parse("<Type><v8:Type>cfg:CatalogRef.C</v8:Type><v8:Type>xs:string</v8:Type><v8:StringQualifiers><v8:Length>0</v8:Length><v8:AllowedLength>Variable</v8:AllowedLength></v8:StringQualifiers></Type>").unwrap();
        // GUID строки 9b6abf8b… (core85) — раньше 9c…
        let n = type_desc(&x, &d).unwrap();
        assert_eq!(crate::brace::write(&n).unwrap().replace("\r\n", ""), "{\"Pattern\",{\"S\"},{\"#\",9c000000-0000-0000-0000-000000000000}}");
        let d = parse("<Type><v8:Type>v8:Null</v8:Type><v8:Type>xs:decimal</v8:Type><v8:Type>xs:dateTime</v8:Type><v8:NumberQualifiers><v8:Digits>0</v8:Digits><v8:FractionDigits>0</v8:FractionDigits><v8:AllowedSign>Any</v8:AllowedSign></v8:NumberQualifiers><v8:DateQualifiers><v8:DateFractions>DateTime</v8:DateFractions></v8:DateQualifiers></Type>").unwrap();
        let n = type_desc(&x, &d).unwrap();
        assert_eq!(crate::brace::write(&n).unwrap().replace("\r\n", ""), "{\"Pattern\",{\"D\"},{\"L\"},{\"N\"}}");
    }
}
