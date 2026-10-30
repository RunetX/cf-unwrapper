//! Элементы формы: группы, декорации, кнопки, поля… (узлы версии 59, раскладки сняты со сверки XML ↔ записи).

use super::*;

pub(super) const GROUP: &str = "cd5394d0-7dda-4b56-8927-93ccbe967a01";
pub(super) const DECORATION: &str = "3d3cb80c-508b-41fa-8a18-680cdf5f1712";
const BUTTON: &str = "a9f3b1ac-f51b-431e-b102-55a69acdecad";
const BORDER: &str = "48312c09-257f-4b29-b280-284dd89efc1e";

pub(super) fn color_auto() -> Node {
    l(vec![a("4"), a("4"), l(vec![a("0")]), a("4")])
}

pub(super) fn font_auto() -> Node {
    l(vec![a("8"), a("3"), a("0"), a("1"), a("100")])
}

pub(super) fn picture_none() -> Node {
    l(vec![a("4"), a("0"), l(vec![a("0")]), s(""), a("-1"), a("-1"), a("1"), a("0"), s("")])
}

/// Свойства элемента: имя → элемент XML (повтор — ошибка). Непрочитанное свойство отвергнет проверка `unread`.
pub(super) struct Props<'e> {
    m: HashMap<&'e str, &'e El>,
}

impl<'e> Props<'e> {
    pub(super) fn new(e: &'e El) -> R<Props<'e>> {
        let mut m = HashMap::new();
        for c in e.elems() {
            if m.insert(c.name.as_str(), c).is_some() {
                return Err(err(c, "повтор"));
            }
        }
        Ok(Props { m })
    }
    pub(super) fn get(&self, k: &str) -> Option<&'e El> {
        self.m.get(k).copied()
    }
    /// Булево: true → 1, false → 0, нет — `def`.
    pub(super) fn b(&self, k: &str, def: &str) -> R<Node> {
        Ok(a(match self.get(k).map(boolean).transpose()? {
            Some(true) => "1",
            Some(false) => "0",
            None => def,
        }))
    }
    /// 0/1/2 = false/true/авто.
    pub(super) fn tri(&self, k: &str) -> R<Node> {
        self.b(k, "2")
    }
    pub(super) fn num(&self, k: &str, def: &str) -> R<Node> {
        Ok(a(self.get(k).map(num).transpose()?.unwrap_or_else(|| def.to_string())))
    }
    pub(super) fn tok(&self, k: &str, m: &[(&str, i64)], def: i64) -> R<Node> {
        self.get(k).map(|e| token(e, m)).unwrap_or_else(|| Ok(an(def)))
    }
    pub(super) fn ml(&self, k: &str) -> R<Node> {
        ml(self.get(k))
    }
    pub(super) fn color(&self, x: &Index, k: &str) -> R<Node> {
        match self.get(k) {
            Some(e) => crate::xml::imp::mxl::color_node(x, e),
            None => Ok(color_auto()),
        }
    }
    pub(super) fn font(&self, x: &Index, k: &str) -> R<Node> {
        match self.get(k) {
            Some(e) => crate::xml::imp::mxl::font_node(x, e, true),
            None => Ok(font_auto()),
        }
    }
}

pub(super) const H_ALIGN: &[(&str, i64)] = &[("Left", 0), ("Center", 1), ("Right", 2), ("Auto", 3)];
pub(super) const V_ALIGN: &[(&str, i64)] = &[("Top", 0), ("Center", 1), ("Bottom", 2)];
pub(super) const TOOLTIP_REPR: &[(&str, i64)] = &[("Auto", 0), ("None", 1), ("Balloon", 2), ("Button", 3), ("ShowAuto", 4), ("ShowTop", 5), ("ShowLeft", 6), ("ShowBottom", 7), ("ShowRight", 8)];
pub(super) const SPACING: &[(&str, i64)] = &[("Auto", 0), ("None", 1), ("Half", 2), ("Single", 3), ("OneAndHalf", 4), ("Double", 5)];
pub(super) const CHILD_WIDTH: &[(&str, i64)] = &[("Auto", 0), ("Equal", 1), ("LeftWide", 2), ("LeftWidest", 3), ("LeftNarrow", 4), ("LeftNarrowest", 5)];
pub(super) const CHILDREN_ALIGN: &[(&str, i64)] = &[("Auto", 0), ("None", 1), ("ItemsLeftTitlesLeft", 2), ("ItemsRightTitlesLeft", 3), ("ItemsLeftTitlesRight", 4), ("ItemsRightTitlesRight", 5), ("TitlesLeftDataAuto", 6)];
const IMPORTANCE: &[(&str, i64)] = &[("Auto", 0), ("VeryHigh", 1), ("High", 2), ("Usual", 3), ("Low", 4), ("VeryLow", 5)];

/// Событие элемента → GUID по виду элемента (сверка БП: у каждой пары один GUID); GUID в XML — как есть.
pub(super) fn item_event(tag: &str, name: &str) -> Option<String> {
    if crate::brace::is_guid(name) {
        return Some(name.to_string());
    }
    const T: &[(&str, &str, &str)] = &[
        ("CalendarField", "OnActivateDate", "3793cac5-9f9a-4b7c-adda-386e5cccf794"),
        ("CalendarField", "OnChange", "fe115cc8-9e33-4684-a166-bd5136fe7a9f"),
        ("CalendarField", "OnPeriodOutput", "1490ede6-6f33-4c6d-b971-53b2541331ea"),
        ("CalendarField", "Selection", "2feb1ee9-b750-4352-bb4c-67ba1c608dc6"),
        ("CheckBoxField", "OnChange", "fe115cc8-9e33-4684-a166-bd5136fe7a9f"),
        ("ExtendedTooltip", "Click", "11707a99-4eb9-4373-bc8c-84891483a034"),
        ("ExtendedTooltip", "URLProcessing", "d710ea07-5c96-4c43-ab6e-e138d3653780"),
        ("FormattedDocumentField", "OnChange", "fe115cc8-9e33-4684-a166-bd5136fe7a9f"),
        ("GraphicalSchemaField", "Selection", "3c3da18f-fc18-4f77-8c2d-96c25bec40a5"),
        ("HTMLDocumentField", "DocumentComplete", "53325f0c-b112-4c44-ab12-5d1ee0b1f07b"),
        ("HTMLDocumentField", "OnClick", "da8dfb86-c5d1-4e35-a8a4-01b167a60ad3"),
        ("InputField", "AutoComplete", "178a97c4-0ffe-4fcc-93e6-505369939da5"),
        ("InputField", "ChoiceProcessing", "f72043b8-2d79-414e-bc4e-3972fe9dbca1"),
        ("InputField", "Clearing", "b50dc41b-c15a-4ebe-a17f-d01e51c47de6"),
        ("InputField", "Creating", "aeba313d-c467-44b3-b4a2-956340932c8f"),
        ("InputField", "EditTextChange", "14256303-d2b7-4a58-bfab-e77493d10a59"),
        ("InputField", "OnChange", "fe115cc8-9e33-4684-a166-bd5136fe7a9f"),
        // события множественных значений поля ввода: mngcore.dll FUN_1802afc00 (GUID → обработчик, имена en/ru)
        ("InputField", "MultipleValueOpening", "e6e68c5f-dc83-43c1-b6ca-438be00b77c1"),
        ("InputField", "MultipleValueURLProcessing", "26b8cded-3de1-4118-90ea-08e3cde193ec"),
        ("InputField", "MultipleValuesAdd", "9d59f117-9183-4044-be98-829c9cd48754"),
        ("InputField", "MultipleValuesDelete", "49ede602-af78-4a50-b821-ec81f6778f2d"),
        ("InputField", "Opening", "ac5a9c5a-5f1d-4fc5-b88c-a187038c16d1"),
        ("InputField", "StartChoice", "1960479b-4d89-4eba-8b39-0aa802020558"),
        ("InputField", "StartListChoice", "b3b65989-73ac-4db3-b6cb-398cb41a062f"),
        ("InputField", "TextEditEnd", "c331eb1b-d32b-4533-844c-1276600b64e3"),
        ("InputField", "Tuning", "70636369-514c-4662-977e-1c3976c9756c"),
        ("LabelDecoration", "Click", "11707a99-4eb9-4373-bc8c-84891483a034"),
        ("LabelDecoration", "URLProcessing", "d710ea07-5c96-4c43-ab6e-e138d3653780"),
        ("LabelField", "Click", "eba5f295-c611-4dd9-84b5-22911ad60c53"),
        ("LabelField", "OnChange", "fe115cc8-9e33-4684-a166-bd5136fe7a9f"),
        ("LabelField", "URLProcessing", "509eca20-d6e4-4fef-a0f8-3a6b44c64178"),
        ("Pages", "OnCurrentPageChange", "526c501f-ed3f-4db4-8731-fd0324707501"),
        ("PictureDecoration", "Click", "9874537f-454c-40ae-83e9-3b9cefbc6d08"),
        ("PictureDecoration", "Drag", "8ad48496-8d0b-4f6c-ae48-99d95227884b"),
        ("PictureDecoration", "DragCheck", "0d644ff6-443b-4390-86fa-7f9105e42711"),
        ("PictureField", "Click", "996b8c30-7a89-4973-8d56-2c9ce2976695"),
        ("PictureField", "Drag", "8ad48496-8d0b-4f6c-ae48-99d95227884b"),
        ("PictureField", "DragCheck", "0d644ff6-443b-4390-86fa-7f9105e42711"),
        ("PictureField", "OnChange", "fe115cc8-9e33-4684-a166-bd5136fe7a9f"),
        ("PlannerField", "BeforeCreate", "2c8ad9e0-53f0-406d-9c8b-460f6cb6fd74"),
        ("PlannerField", "BeforeDelete", "6f758c30-b414-4949-a236-584bccb0102a"),
        ("PlannerField", "BeforeStartEdit", "82f5f464-5aa5-4efc-9813-51432c74ff8e"),
        ("PlannerField", "BeforeStartQuickEdit", "b56fa930-855b-4e67-a87c-8ea917d1dfab"),
        ("PlannerField", "CommandGenerateProcessing", "b34abb58-ee89-4119-945c-6432e9ef04a9"),
        ("PlannerField", "OnCurrentRepresentationPeriodChange", "5e6c8466-44d0-4fd0-8d87-ac749c16ff60"),
        ("PlannerField", "OnEditEnd", "68fc3ed6-a517-4843-a7a4-473cf4e27209"),
        ("PlannerField", "Selection", "b6aaed5c-8a5a-4a03-bd5b-1f14d0f099ec"),
        ("RadioButtonField", "OnChange", "fe115cc8-9e33-4684-a166-bd5136fe7a9f"),
        ("SpreadSheetDocumentField", "AdditionalDetailProcessing", "0b8dc702-d001-4637-a215-9f35613e096c"),
        ("SpreadSheetDocumentField", "BeforePrint", "61455593-0982-4415-bc2e-2e8722a7abd0"),
        ("SpreadSheetDocumentField", "BeforeWrite", "b7646583-04d3-4905-8f04-8985914bd1b7"),
        ("SpreadSheetDocumentField", "DetailProcessing", "2988b2a5-c887-4928-94ae-5d0c9c31e999"),
        // mngcore.dll FUN_1802b6210 (события поля диаграммы)
        ("ChartField", "DetailProcessing", "650da4af-3233-4ce0-a1ae-23f87a226eee"),
        ("ChartField", "Selection", "515cd17b-dd4c-4181-bbbf-8676467acf49"),
        ("GanttChartField", "DetailProcessing", "8724b8d4-140d-4357-8ac9-46e29ba7b168"),
        ("SpreadSheetDocumentField", "Drag", "8ad48496-8d0b-4f6c-ae48-99d95227884b"),
        ("SpreadSheetDocumentField", "DragCheck", "0d644ff6-443b-4390-86fa-7f9105e42711"),
        ("SpreadSheetDocumentField", "OnActivate", "2042ec93-3108-4190-b767-ec6c10dd9ff4"),
        // OnChange — одно событие полей в mngbase.dll (GUID fe115cc8… только в FUN_1802330c0)
        ("SpreadSheetDocumentField", "OnChange", "fe115cc8-9e33-4684-a166-bd5136fe7a9f"),
        ("SpreadSheetDocumentField", "OnChangeAreaContent", "411a4578-276c-4f4a-b56a-b3b01181c997"),
        ("SpreadSheetDocumentField", "Selection", "22287505-97d8-4258-a318-209e2493f7eb"),
        // mngcore.dll FUN_1802b2b90 (события поля табличного документа)
        ("SpreadSheetDocumentField", "URLProcessing", "06d41ccc-4e8a-46f8-aeff-b3303cf753d2"),
        ("Table", "AfterDeleteRow", "de65638d-a806-4a76-bc10-f62bbc86e0e7"),
        ("Table", "BeforeAddRow", "2391e7b8-7235-45d7-ab7e-6ff3dc086396"),
        ("Table", "BeforeCollapse", "a7a9dc42-29b6-4c5b-8980-6d0b87149bdd"),
        ("Table", "BeforeDeleteRow", "2ccfdec5-583d-4eca-8319-e55de492665a"),
        ("Table", "BeforeEditEnd", "4d88756d-bad4-4fde-92e1-c1f1402ac6b2"),
        ("Table", "BeforeExpand", "7c39b7bc-db0f-4410-9d98-8e5b7896995e"),
        ("Table", "BeforeLoadUserSettingsAtServer", "c41e7b98-098c-433e-8ac3-56ec2a2c49e2"),
        ("Table", "BeforeRowChange", "ab930362-ff94-4dcb-ad16-188805d23e3c"),
        ("Table", "ChoiceProcessing", "8bfdb5eb-62dc-4851-8a2c-e983526356bf"),
        ("Table", "Drag", "8ad48496-8d0b-4f6c-ae48-99d95227884b"),
        ("Table", "DragCheck", "0d644ff6-443b-4390-86fa-7f9105e42711"),
        ("Table", "DragEnd", "cb286ab3-3a1c-40d2-a232-6e64f624ccec"),
        ("Table", "DragStart", "6d4d6747-a823-4f61-ab31-a426572f2c6c"),
        ("Table", "NewWriteProcessing", "ce67decf-16b8-4d61-b347-4e6a063580dc"),
        ("Table", "OnActivateCell", "f228b12f-d892-4925-b338-695617357b32"),
        ("Table", "OnActivateField", "6e973761-8683-47fa-a609-4e230950294d"),
        ("Table", "OnActivateRow", "60edb81d-887b-478e-94ee-7fef2b13393d"),
        ("Table", "OnChange", "fe115cc8-9e33-4684-a166-bd5136fe7a9f"),
        ("Table", "OnCurrentParentChange", "2971b9a9-1724-4f34-aaa4-f3db584c3ca0"),
        ("Table", "OnEditEnd", "01d80ddd-dce5-4db3-beb5-f63c97cb05b9"),
        ("Table", "OnGetDataAtServer", "97365900-eadf-4dfd-a9aa-fbb9ecabd079"),
        ("Table", "OnLoadUserSettingsAtServer", "336b3ee5-d67f-4651-b098-e2c53f8317e2"),
        ("Table", "OnSaveUserSettingsAtServer", "a73dae96-734d-42e4-8ae7-b70249ecd233"),
        ("Table", "OnStartEdit", "b3c10170-c5ff-4cba-b537-679e1c872b45"),
        ("Table", "OnUpdateUserSettingSetAtServer", "e91128e6-621d-4dc8-b12e-bd65aeb37e2d"),
        ("Table", "RefreshRequestProcessing", "ff33c4d6-a0db-4906-992e-37b3f44cd97a"),
        ("Table", "Selection", "1282f000-23b6-4887-87f4-9e8e79db3d32"),
        ("Table", "ValueChoice", "0d8cf5b0-55eb-4d1e-960a-22c160210945"),
        // mngcore.dll FUN_1802ba8a0 (события таблицы)
        ("Table", "URLGetProcessing", "674956b3-e469-4fdc-acf5-24ebf88cf7ab"),
        ("Table", "URLListGetProcessing", "44498116-1641-4bfa-ae33-86e53c205797"),
        ("TextDocumentField", "OnChange", "fe115cc8-9e33-4684-a166-bd5136fe7a9f"),
        ("TrackBarField", "OnChange", "fe115cc8-9e33-4684-a166-bd5136fe7a9f"),
    ];
    T.iter().find(|(t, n, _)| *t == tag && *n == name).map(|(.., g)| g.to_string()).or_else(|| {
        use crate::xml::events_table::EVENTS;
        // прочие — по таблице событий DLL: в наборах событий, где лежат уже известные события этого вида элемента,
        // иначе — если имени отвечает один GUID
        let sets: Vec<&str> = T.iter().filter(|(t, ..)| *t == tag).flat_map(|(.., g)| EVENTS.iter().filter(move |(id, ..)| id == g).map(|(.., st)| *st)).collect();
        let pick = |it: &mut dyn Iterator<Item = &(&str, &str, &str)>| match (it.next(), it.next()) {
            (Some((g, ..)), None) => Some(g.to_string()),
            (Some((g, ..)), Some((h, ..))) if g == h => Some(g.to_string()),
            _ => None,
        };
        pick(&mut EVENTS.iter().filter(|(_, n, st)| *n == name && sets.contains(st))).or_else(|| pick(&mut EVENTS.iter().filter(|(_, n, _)| *n == name)))
    })
}

/// Стандартная команда формы → GUID; у части команд GUID зависит от класса основного реквизита (сверка БП: список
/// и объект различаются), поэтому принимаются только встреченные классы; `*` — команда любой формы.
fn form_command(name: &str, class: &str) -> Option<&'static str> {
    const T: &[(&str, &[&str], &str)] = &[
        ("Cancel", &["*"], "679b62d9-ff72-4329-bf3a-c0c32b311dd2"),
        ("Close", &["*"], "3772996b-41f4-4c47-a5a8-ea397db424ae"),
        ("CustomizeForm", &["*"], "198ea630-fda2-4cda-8a23-f999f4c67ee6"),
        ("Help", &["*"], "39bb0fe9-771d-4dd5-8a6e-2d16984523af"),
        ("OK", &["*"], "f3613d5c-20c6-46e5-b4d5-7d712ece1296"),
        ("RestoreValues", &["*"], "71e0226e-ebb2-4e33-8745-0a94a01bbf15"),
        ("SaveValues", &["*"], "239f0103-8de9-4fdf-b485-eb5531da7e51"),
        ("Yes", &["*"], "5d41082e-9619-42ec-b96f-98b082b3a2f0"),
        ("CancelSearch", &["DynamicList"], "96e0bc70-f8ff-4732-8119-060923203629"),
        ("Change", &["DynamicList"], "6886601d-276c-4d3f-af0a-05c586025608"),
        ("ChangeHistory", &["DynamicList"], "c9abb6b0-eafd-4505-8312-9a7b6888cbf3"),
        ("Choose", &["DynamicList"], "8e2b82cf-d1ea-46b2-afdf-a8d64e66ea2b"),
        ("Copy", &["DynamicList"], "342c531d-dc73-458a-8ac4-6a746916a33b"),
        ("Copy", &["CatalogObject"], "68baa1bc-edd1-4d9b-ad80-1d53fb8a7988"),
        ("Create", &["DynamicList"], "4f834c38-add1-45e4-a9f3-cefe3efac5c9"),
        ("CreateFolder", &["DynamicList"], "d8772fd1-a3bf-417d-8334-c49968dbb45e"),
        ("Delete", &["CatalogObject"], "c32d43de-b820-49d0-bf7a-d70829f48f40"),
        ("DynamicListStandardSettings", &["DynamicList"], "d603a249-6eb3-4e38-bb2d-a8a86a8ab156"),
        ("Find", &["DynamicList"], "bdefa701-6685-453e-a02a-3683d0cc16d3"),
        ("FindByCurrentValue", &["DynamicList"], "b520ca45-d8db-4982-b128-bb42a6afd911"),
        ("GetURL", &["DynamicList"], "9885f4b6-d830-435f-a0e3-6b70ffe0f85c"),
        ("HierarchicalList", &["DynamicList"], "ffc5e8d5-40a7-4893-a590-49bd588f9466"),
        ("List", &["DynamicList"], "a2b927a1-35af-43e3-af73-4af22ac2c0fa"),
        ("ListSettings", &["DynamicList"], "1c00edb8-a826-4855-9bde-94dbc5f620e5"),
        ("LoadDynamicListSettings", &["DynamicList"], "952c2984-9955-415a-8235-5c710aabe732"),
        ("MoveItem", &["DynamicList"], "39c6a2fb-45cc-41b1-853f-967fb68aa1df"),
        ("OutputList", &["DynamicList"], "9758d344-4b1d-4dc9-80bd-81060bc18b2a"),
        ("Refresh", &["DynamicList"], "fd8f031f-c168-4e1b-8b0c-15eb3057e688"),
        ("Reread", &["CatalogObject"], "1f317795-c420-4a30-b594-c492abc55f7a"),
        ("SaveDynamicListSettings", &["DynamicList"], "d5c3842d-7252-4370-9174-756a6cc553e5"),
        ("SetDeletionMark", &["DynamicList"], "2e86453d-8958-4c9a-a1b4-b15215eedc2e"),
        ("SetDeletionMark", &["CatalogObject"], "827b541d-30c1-4f06-aecf-92aa496a0835"),
        ("ShowInList", &["CatalogObject"], "3a17e914-ec6a-4280-b4df-78914f40522b"),
        ("Start", &["BusinessProcessObject"], "8d7bcd38-1bbb-4dc1-a9ad-cc9d5966ca8e"),
        ("StartAndClose", &["BusinessProcessObject"], "e6a9041f-4d43-4f06-8e17-e95753531565"),
        ("Tree", &["DynamicList"], "0b83270d-7f95-4cdd-93c3-342d7991fed5"),
        // Write, WriteAndClose, PostAndClose документа и записи регистра сведений — GUID те же, что у исключенных команд
        ("Write", &["BusinessProcessObject", "CatalogObject", "TaskObject", "DocumentObject", "InformationRegisterRecordManager"], "fe558fde-99b3-45d0-a060-9fc2905309f6"),
        ("WriteAndClose", &["CatalogObject", "InformationRegisterRecordManager", "DocumentObject", "ChartOfCharacteristicTypesObject", "InformationRegisterRecordSet"], "32df4349-2607-4c2b-a4b9-bca4a1a28bd7"),
        // сверка кнопок БП (форма — класс основного реквизита)
        ("ChangeHistory", &["DocumentObject"], "174e58ce-82ad-4787-b956-9367937f7971"),
        ("Copy", &["DocumentObject"], "68baa1bc-edd1-4d9b-ad80-1d53fb8a7988"),
        ("Delete", &["InformationRegisterRecordManager"], "c32d43de-b820-49d0-bf7a-d70829f48f40"),
        ("Post", &["DocumentObject"], "3b8cedbc-8e74-4017-b901-d14b09f32f7a"),
        ("Reread", &["DocumentObject", "InformationRegisterRecordManager", "InformationRegisterRecordSet"], "1f317795-c420-4a30-b594-c492abc55f7a"),
        ("SetDeletionMark", &["DocumentObject", "TaskObject"], "827b541d-30c1-4f06-aecf-92aa496a0835"),
        ("ShowInList", &["DocumentObject"], "3a17e914-ec6a-4280-b4df-78914f40522b"),
        ("UndoPosting", &["DocumentObject"], "389ef1f1-97ce-4326-adf5-886b2dead75c"),
        ("Write", &["ChartOfCharacteristicTypesObject"], "fe558fde-99b3-45d0-a060-9fc2905309f6"),
        ("CancelEdit", &["ReportObject", "dcsset:SettingsComposer"], "8149a06a-dbf3-4d4d-a275-5385a4196fc7"),
        ("EndEdit", &["ReportObject"], "74c1abd6-b274-4654-baf0-7b8418b792ea"),
        ("LoadReportSettings", &["ReportObject"], "b0c9afb6-320c-4e36-be21-8f6d48116415"),
        ("LoadVariant", &["ReportObject"], "b08b7a35-583a-4756-b814-0436ff9139c0"),
        ("NewWindow", &["ReportObject"], "03df6ee5-883c-4cc6-b319-d886d1a9b2c8"),
        ("ReportSettings", &["ReportObject"], "0fb774df-ec1c-4e23-9ed1-e089974f74bf"),
        ("SaveReportSettings", &["ReportObject"], "7910bb04-ddcc-4e5d-89f0-104c6ad0f187"),
        ("SaveVariant", &["ReportObject"], "9bffcf73-7b1d-4a8d-bf23-5e051af3ee29"),
        ("Ignore", &["*"], "d7e9e72c-8fa7-430c-a3e9-aeadfd57dfc7"),
        ("No", &["*"], "06ee6a21-061e-47f8-81c5-92ae8b8f3b5d"),
        ("PostAndClose", &["DocumentObject"], "87317f86-057f-477e-9045-2da4e4980199"),
    ];
    T.iter().find(|(n, cs, _)| *n == name && cs.iter().any(|c| *c == "*" || *c == class)).map(|(.., g)| *g)
}

/// Стандартная команда элемента → GUID по виду элемента (у таблицы — динамический список или нет; `None` — любая).
fn item_command(tag: &str, dynamic_list: bool, name: &str) -> Option<&'static str> {
    const T: &[(&str, Option<bool>, &str, &str)] = &[
        ("FormattedDocumentField", None, "AlignCenter", "ab0ebc39-68ee-4034-b2f4-43eee55bd651"),
        ("FormattedDocumentField", None, "AlignJustify", "56ae90b6-588f-406e-919c-cc5cc7f86297"),
        ("FormattedDocumentField", None, "AlignLeft", "87ecfbdd-8e2b-4ba2-a315-0897020f382f"),
        ("FormattedDocumentField", None, "AlignRight", "e428af27-c4f7-4577-b80e-95a79f94322d"),
        ("FormattedDocumentField", None, "BackColor", "17724105-6e59-4d52-8a42-cf0fb4838037"),
        ("FormattedDocumentField", None, "Bold", "f20eefc2-f819-4ab1-be67-87b3ca2e26e6"),
        ("FormattedDocumentField", None, "BulletedList", "a0033f06-56f7-4855-b901-7ac66fe1bb99"),
        ("FormattedDocumentField", None, "Char", "871100d5-049d-4b22-a46a-fabf54bd64c3"),
        ("FormattedDocumentField", None, "CopyToClipboard", "7a294bdc-b86b-4b73-abc4-df9c811f61ef"),
        ("FormattedDocumentField", None, "CutToClipboard", "83670388-2e45-439e-9968-587eca6c7f8d"),
        ("FormattedDocumentField", None, "DecreaseFontSize", "ec647dcc-2be7-486c-9046-d8b371f9909e"),
        ("FormattedDocumentField", None, "DecreaseIndent", "39f6b9f1-7aa1-4a03-a01b-e127d51bc228"),
        ("FormattedDocumentField", None, "Font", "a8f6b59e-b712-4d3e-a974-55a3be4eb295"),
        ("FormattedDocumentField", None, "Hyperlink", "6d83186a-5838-40a5-95e7-8990193adf0a"),
        ("FormattedDocumentField", None, "IncreaseFontSize", "0bdc43a3-79f0-48d6-bce4-1142542e1a59"),
        ("FormattedDocumentField", None, "IncreaseIndent", "d0a4d953-115b-4059-a6cb-6e67f903a4f3"),
        ("FormattedDocumentField", None, "Italic", "a8631f01-318a-4da2-80a9-9075c7524463"),
        ("FormattedDocumentField", None, "LineSpacing", "408f351e-0536-46be-8916-a891db9bfbe6"),
        ("FormattedDocumentField", None, "NumberedList", "2b5d0007-b74c-4786-a904-37e64bda8414"),
        ("FormattedDocumentField", None, "PasteFromClipboard", "905692d2-c3e7-4433-8f10-8d2ce35f652b"),
        ("FormattedDocumentField", None, "Picture", "9d8a3915-de52-4227-91cd-2fce22e09972"),
        ("FormattedDocumentField", None, "Preview", "4ca32834-6f9f-4dfb-89ce-6db36931c89b"),
        ("FormattedDocumentField", None, "Print", "a8483976-8b13-416a-9680-133b306dc6b0"),
        ("FormattedDocumentField", None, "Redo", "6f1ea963-0807-4de8-b544-b5666f500b05"),
        ("FormattedDocumentField", None, "SaveAs", "5a331cec-bf93-4af5-8f51-80fd7118db47"),
        ("FormattedDocumentField", None, "SelectAll", "b67f202a-dcf8-41f3-bda8-1ff9bed5f2ef"),
        ("FormattedDocumentField", None, "Strikeout", "db1cd9b3-bdf4-43f5-abd6-c2e4bd85d709"),
        ("FormattedDocumentField", None, "TextColor", "71007f7d-1995-44aa-9125-9926e70a35b5"),
        ("FormattedDocumentField", None, "Underline", "85bd789b-0047-46f9-9b2e-845907fc1b1d"),
        ("FormattedDocumentField", None, "Undo", "f5814962-2bef-43dd-b633-a193d4b0970e"),
        ("SpreadSheetDocumentField", None, "PageSetup", "41c1bb40-1027-4fd1-a19e-17976100e64b"),
        ("SpreadSheetDocumentField", None, "Preview", "5aa38159-2001-42ae-8451-f8cabe0762c3"),
        ("SpreadSheetDocumentField", None, "Print", "d673d512-f71a-48a6-ae5d-527a64ffd813"),
        ("GraphicalSchemaField", None, "PageSetup", "01db2225-b62d-4112-a4b6-d39d627bf79f"),
        ("GraphicalSchemaField", None, "Preview", "1d13f9a3-402a-46cb-9c68-1709356840f2"),
        ("GraphicalSchemaField", None, "Print", "e2d6f793-b786-4640-a91b-8d77f73860f1"),
        ("GraphicalSchemaField", None, "PrintImmediately", "7b53bef7-3811-4375-810a-672bf817ace4"),
        ("PDFDocumentField", None, "GoBack", "32a87619-85ce-495a-a195-2719f5e9c71e"),
        ("PDFDocumentField", None, "GoForward", "85a1b7ee-e94d-4783-b519-4af123f58596"),
        ("PDFDocumentField", None, "GoToBegin", "a4e92b1d-5e86-4a86-b276-00e71ecf3fd4"),
        ("PDFDocumentField", None, "GoToEnd", "3e80231d-e169-456e-8373-0b9d4c15c8ab"),
        ("PDFDocumentField", None, "RotateClockwise", "30f0b852-3284-4ae4-838b-f89b78388bdc"),
        ("PDFDocumentField", None, "RotateCounterclockwise", "e28575de-9fe8-4a8e-a285-13250de33d49"),
        ("PDFDocumentField", None, "ScaleDown", "1e5ebd8b-32ee-4af9-b85c-3a0417000660"),
        ("PDFDocumentField", None, "ScaleUp", "d9117be6-f436-40fd-9670-8d71b99b2477"),
        ("SpreadSheetDocumentField", None, "AlignCenter", "ab0ebc39-68ee-4034-b2f4-43eee55bd651"),
        ("SpreadSheetDocumentField", None, "AlignJustify", "56ae90b6-588f-406e-919c-cc5cc7f86297"),
        ("SpreadSheetDocumentField", None, "AlignLeft", "87ecfbdd-8e2b-4ba2-a315-0897020f382f"),
        ("SpreadSheetDocumentField", None, "AlignRight", "e428af27-c4f7-4577-b80e-95a79f94322d"),
        ("SpreadSheetDocumentField", None, "BackColor", "17724105-6e59-4d52-8a42-cf0fb4838037"),
        ("SpreadSheetDocumentField", None, "Bold", "f20eefc2-f819-4ab1-be67-87b3ca2e26e6"),
        ("SpreadSheetDocumentField", None, "BorderAll", "4efdcc95-9f24-4652-a5ed-febfaa51f135"),
        ("SpreadSheetDocumentField", None, "BorderBottom", "5ff37d88-ba18-428c-bf49-9ccde4c46268"),
        ("SpreadSheetDocumentField", None, "BorderColor", "c1c95e8a-37b4-477c-9df1-7bc0fdbc1bd3"),
        ("SpreadSheetDocumentField", None, "BorderInside", "c4713141-07d1-411f-8804-172d4b8c4f01"),
        ("SpreadSheetDocumentField", None, "BorderLeft", "635136cc-3a47-43d1-8893-d7d536bf37c4"),
        ("SpreadSheetDocumentField", None, "BorderNone", "1177792e-7da2-4755-81a9-997f2d0e3dce"),
        ("SpreadSheetDocumentField", None, "BorderOutline", "05183b65-aa1f-401c-ac34-d99a6ff4ff60"),
        ("SpreadSheetDocumentField", None, "BorderRight", "e05dddd1-40ad-44a2-8ae7-0a551f0b1809"),
        ("SpreadSheetDocumentField", None, "BorderTop", "81be7ad1-a93f-4936-92d0-d59c10213f28"),
        ("SpreadSheetDocumentField", None, "CollapseAllGroups", "ff5c34f8-b172-4ef2-91d3-48283a66a725"),
        ("SpreadSheetDocumentField", None, "Edit", "5c52ec51-6000-4190-b8c8-bd6201a271f5"),
        ("SpreadSheetDocumentField", None, "ExpandAllGroups", "12acffde-8389-4e5e-bd86-ff248262d84a"),
        ("SpreadSheetDocumentField", None, "Find", "0cf34151-92d3-42fd-954f-5938433908a4"),
        ("SpreadSheetDocumentField", None, "FindNext", "29fc1bfc-7bf8-4849-9b4b-3e42d12ebcbb"),
        ("SpreadSheetDocumentField", None, "Font", "05e95a55-947e-4dde-a657-16ec11750a2f"),
        ("SpreadSheetDocumentField", None, "Italic", "a8631f01-318a-4da2-80a9-9075c7524463"),
        ("SpreadSheetDocumentField", None, "Merge", "f4df676f-eefb-48bd-9117-83ee6c207cb5"),
        ("SpreadSheetDocumentField", None, "PrintImmediately", "0e355e57-d603-4ac6-998b-c522c43d3668"),
        ("SpreadSheetDocumentField", None, "Properties", "be8800c3-8ccf-444a-bbf0-8f3078ff0ded"),
        ("SpreadSheetDocumentField", None, "RemoveName", "adc3d2d0-4d84-4038-a453-ab5d693a60bd"),
        ("SpreadSheetDocumentField", None, "Save", "d8e20c4d-3519-49aa-80e5-d6d66fee741a"),
        ("SpreadSheetDocumentField", None, "SaveAs", "999b786e-0534-4fca-8b62-f706d5336798"),
        ("SpreadSheetDocumentField", None, "SetName", "5ce7de77-4796-41a0-beb3-7dc420053639"),
        ("SpreadSheetDocumentField", None, "ShowGrid", "83c3121e-60cc-4b47-a296-0c1976f0d766"),
        ("SpreadSheetDocumentField", None, "ShowHeaders", "4a9ec98e-c814-47b6-911f-694effc10fc5"),
        ("SpreadSheetDocumentField", None, "TextColor", "71007f7d-1995-44aa-9125-9926e70a35b5"),
        ("SpreadSheetDocumentField", None, "ThickBorderBottom", "65ee8b9b-8a0a-4de8-9a71-3a7a8c43c4ba"),
        ("SpreadSheetDocumentField", None, "ThickBorderOutline", "08872d94-c57a-470f-a050-0d4d81765df2"),
        ("SpreadSheetDocumentField", None, "ThickBorderTop", "a01654df-d7f1-4ec5-8b03-258d953de2e7"),
        ("SpreadSheetDocumentField", None, "Underline", "85bd789b-0047-46f9-9b2e-845907fc1b1d"),
        ("Table", Some(false), "Add", "b0016a68-ec64-4e6d-b905-c71fd62efc4c"),
        ("Table", None, "CancelSearch", "44ad3ec9-f3c2-4913-9224-5f9fb6418743"),
        ("Table", None, "Change", "b41f5bbc-ba5d-4888-8cd1-db246a371418"),
        ("Table", Some(false), "CheckAll", "18248aa8-e621-4e19-a611-54fb8923644c"),
        ("Table", None, "Choose", "8969c93a-23e5-4bef-941d-aaef315858d2"),
        ("Table", None, "Copy", "0ae4bea5-23be-42a7-b69e-97b11b29c453"),
        ("Table", None, "CopyToClipboard", "88078230-1f6b-415f-99e4-ad2ff73810cf"),
        ("Table", Some(true), "Create", "0f8d6d98-2f8b-405a-b8b3-0538e9d95da5"),
        ("Table", Some(true), "CreateFolder", "d82ca05c-2966-4d77-9a39-a1eea087bfa7"),
        ("Table", None, "Delete", "8d772f97-c0ef-47c0-9cb0-efea28c61341"),
        ("Table", Some(true), "DynamicListStandardSettings", "33b7b9cd-6979-4435-8c58-d9bc8250edec"),
        ("Table", Some(false), "EndEdit", "9ef79140-3de6-436a-8dda-610bb963f5db"),
        ("Table", None, "Find", "c0519548-2a9a-44de-a25e-faf01e089d4d"),
        ("Table", None, "FindByCurrentValue", "714d44cc-63da-4431-b33a-428e398d2a08"),
        ("Table", Some(true), "LevelDown", "dc118d99-b351-4e30-9310-e864f2e53ec0"),
        ("Table", Some(true), "LevelUp", "0e9b637d-cf6e-4330-8a8f-cd44842e34bb"),
        ("Table", Some(true), "ListSettings", "14559f7c-853c-42a4-9ea1-01546107747b"),
        ("Table", Some(true), "LoadDynamicListSettings", "182a793b-22a5-4625-b316-6a5be7f88078"),
        ("Table", Some(false), "MoveDown", "fa51b106-eae6-44c7-8054-76cbb3100603"),
        ("Table", Some(true), "MoveItem", "27bd521a-51c6-4fe7-846d-a98f988774b5"),
        ("Table", Some(false), "MoveUp", "37740564-9e86-44a0-bea9-3f485a5a3f91"),
        ("Table", Some(false), "OutputList", "49602716-fea6-497f-8047-726404038857"),
        ("Table", Some(true), "OutputList", "825c1c15-ef8f-47ab-b002-e6b84b3e5b10"),
        ("Table", Some(true), "Refresh", "403bc6e6-b98e-4181-9f43-9c75cbbf82cf"),
        ("Table", Some(true), "SaveDynamicListSettings", "95b4bc12-2ece-4d7a-b3e2-6f9293620a06"),
        ("Table", Some(false), "SearchEverywhere", "7b683784-b474-441a-ba63-3d757bd0ffd4"),
        ("Table", Some(false), "SelectAll", "51c99108-107c-43e1-8918-e48835bf2495"),
        ("Table", Some(true), "SetDeletionMark", "a2f737a8-0114-4e86-a214-45e5c213fa65"),
        ("Table", Some(false), "ShowMultipleSelection", "e7216412-03ac-4a81-99c2-1d7c28e88e31"),
        ("Table", Some(false), "SortListAsc", "2bbe4e12-06d2-409b-a972-eea585125d83"),
        ("Table", Some(false), "SortListDesc", "58b2a785-23f6-4b0e-a324-9a1323285595"),
        ("Table", Some(false), "UncheckAll", "5048cc44-702b-44e3-8445-9af75c02724d"),
        ("Table", Some(false), "UserSettingItemProperties", "1f1e900a-8488-4159-81be-9704eb96906d"),
    ];
    T.iter().find(|(t, dl, n, _)| *t == tag && *n == name && dl.is_none_or(|d| d == dynamic_list)).map(|(.., g)| *g)
}

/// Сочетание клавиш `{0,<код>,<модификаторы>}` (обратное `shortcut` выгрузки).
pub(super) fn shortcut(e: Option<&El>) -> R<Node> {
    let Some(e) = e else { return Ok(l(vec![a("0"), a("0"), a("0")])) };
    let t = e.text()?;
    let mut rest = t.as_str();
    let mut m = 0;
    loop {
        if let Some(r) = rest.strip_prefix("Ctrl+") {
            m |= 8;
            rest = r;
        } else if let Some(r) = rest.strip_prefix("Alt+") {
            m |= 16;
            rest = r;
        } else if let Some(r) = rest.strip_prefix("Shift+") {
            m |= 4;
            rest = r;
        } else {
            break;
        }
    }
    let key: i64 = match rest {
        "Esc" => 27,
        "Space" => 32,
        "Enter" => 13,
        "Del" => 46,
        "Ins" => 45,
        "BackSpace" => 8,
        "Num +" => 107,
        "Num -" => 109,
        "Num ." => 110,
        k if k.strip_prefix("Num ").is_some_and(|d| d.len() == 1 && d.as_bytes()[0].is_ascii_digit()) => 96 + (k.as_bytes()[4] - b'0') as i64,
        k if k.len() == 1 && k.chars().all(|c| c.is_ascii_digit() || c.is_ascii_uppercase()) => k.chars().next().unwrap_or('0') as i64,
        k if k.starts_with('F') => k[1..].parse::<i64>().ok().filter(|n| (1..=12).contains(n)).map(|n| n + 111).ok_or_else(|| err(e, &format!("клавиша {k}")))?,
        k => return Err(err(e, &format!("клавиша {k}"))),
    };
    Ok(l(vec![a("0"), an(key), an(m)]))
}

/// Рамка элемента `{3,0,{0},<стиль>,<толщина>,0,48312c09-…}`; нет — стиль `def`, толщина 1.
pub(super) fn border(e: Option<&El>, def: i64) -> R<Node> {
    let (style, width) = match e {
        None => (def, "1".to_string()),
        Some(b) => {
            let w = crate::xml::imp::tpl::int(b.req_attr("width")?).ok_or_else(|| err(b, "width"))?;
            let st = b.req("v8ui:style")?;
            if st.req_attr("xsi:type")? != "v8ui:ControlBorderType" {
                return Err(err(st, "xsi:type"));
            }
            let s = match st.text()?.as_str() {
                "WithoutBorder" => 0,
                "Single" => 1,
                "Embossed" => 2,
                "Indented" => 3,
                "Underline" => 4,
                "Overline" => 7,
                "Double" => 200,
                o => return Err(err(st, &format!("рамка {o}"))),
            };
            (s, w)
        }
    };
    Ok(l(vec![a("3"), a("0"), l(vec![a("0")]), an(style), a(width), a("0"), a(BORDER)]))
}

/// Важность отображения (атрибут DisplayImportance): нет — 0.
pub(super) fn importance(e: &El) -> R<Node> {
    match e.attr("DisplayImportance") {
        None => Ok(a("0")),
        Some(v) => IMPORTANCE.iter().find(|(n, _)| *n == v).map(|(_, c)| an(c)).ok_or_else(|| err(e, &format!("DisplayImportance {v}"))),
    }
}

impl F<'_> {
    /// Дети `<ChildItems>`: число и пары (вид, элемент).
    pub(super) fn children(&self, e: Option<&El>) -> R<Vec<Node>> {
        let mut v = vec![a("0")];
        let mut n = 0;
        if let Some(e) = e {
            for c in e.elems() {
                let (kind, node) = self.item(c)?;
                v.extend([a(kind), node]);
                n += 1;
            }
            if n == 0 {
                e.text()?;
            }
        }
        v[0] = an(n);
        Ok(v)
    }

    fn item(&self, e: &El) -> R<(&'static str, Node)> {
        match e.name.as_str() {
            "LabelDecoration" => Ok((DECORATION, self.decoration(e, 0)?)),
            "PictureDecoration" => Ok((DECORATION, self.decoration(e, 1)?)),
            "UsualGroup" => Ok((GROUP, self.usual_group(e)?)),
            "Button" => Ok((BUTTON, self.button(e)?)),
            "InputField" => Ok((super::fields::FIELD, self.field(e, 2, "InputField")?)),
            "CheckBoxField" => Ok((super::fields::FIELD, self.field(e, 3, "CheckBoxField")?)),
            "LabelField" => Ok((super::fields::FIELD, self.field(e, 1, "LabelField")?)),
            "RadioButtonField" => Ok((super::fields::FIELD, self.field(e, 5, "RadioButtonField")?)),
            "SpreadSheetDocumentField" => Ok((super::fields::FIELD, self.field(e, 6, "SpreadSheetDocumentField")?)),
            "HTMLDocumentField" => Ok((super::fields::FIELD, self.field(e, 15, "HTMLDocumentField")?)),
            "PictureField" => Ok((super::fields::FIELD, self.field(e, 4, "PictureField")?)),
            "ProgressBarField" => Ok((super::fields::FIELD, self.field(e, 9, "ProgressBarField")?)),
            "TextDocumentField" => Ok((super::fields::FIELD, self.field(e, 7, "TextDocumentField")?)),
            "CalendarField" => Ok((super::fields::FIELD, self.field(e, 8, "CalendarField")?)),
            "TrackBarField" => Ok((super::fields::FIELD, self.field(e, 10, "TrackBarField")?)),
            "GraphicalSchemaField" => Ok((super::fields::FIELD, self.field(e, 14, "GraphicalSchemaField")?)),
            "FormattedDocumentField" => Ok((super::fields::FIELD, self.field(e, 17, "FormattedDocumentField")?)),
            "PDFDocumentField" => Ok((super::fields::FIELD, self.field(e, 20, "PDFDocumentField")?)),
            "ChartField" => Ok((super::fields::FIELD, self.field(e, 11, "ChartField")?)),
            "GanttChartField" => Ok((super::fields::FIELD, self.field(e, 12, "GanttChartField")?)),
            "PeriodField" => Ok((super::fields::FIELD, self.field(e, 18, "PeriodField")?)),
            "PlannerField" => Ok((super::fields::FIELD, self.field(e, 19, "PlannerField")?)),
            "Pages" => Ok((GROUP, self.pages(e)?)),
            "ButtonGroup" => Ok((GROUP, self.button_group(e)?)),
            "CommandBar" => Ok((GROUP, self.command_bar(e)?)),
            "Popup" => Ok((GROUP, self.popup(e)?)),
            "Page" => Ok((GROUP, self.page(e)?)),
            "ColumnGroup" => Ok((GROUP, self.column_group(e)?)),
            "Table" => Ok((super::table::TABLE, self.table(e)?)),
            "SearchStringAddition" | "ViewStatusAddition" | "SearchControlAddition" => Ok((super::table::ADDITION, self.addition(e)?)),
            n => Err(err(e, &format!("элемент {n} пока не загружается"))),
        }
    }

    /// Поля 0–6: версия, `{<id>,02023637…}`, 0, 0, флаг видимости по ролям (+ значение), вид, имя.
    fn head(&self, ver: &str, e: &El, p: &Props, kind: i64) -> R<Vec<Node>> {
        let mut v = vec![a(ver), l(vec![a(e.req_attr("id")?), a(ELEM)]), a("0"), a("0")];
        match p.get("UserVisible") {
            Some(u) => v.extend([a("1"), adjustable(self.x, Some(u))?]),
            None => v.push(a("0")),
        }
        v.extend([an(kind), s(e.req_attr("name")?)]);
        Ok(v)
    }

    /// Расширенная подсказка (декорация-надпись без контекстного меню).
    pub(super) fn extended_tooltip(&self, e: &El) -> R<Node> {
        self.decoration(e, -1)
    }

    /// Декорация (версия 12): надпись (вид 0); `kind` -1 — расширенная подсказка (вид 0 без контекстного меню и своей
    /// подсказки).
    fn decoration(&self, e: &El, kind: i64) -> R<Node> {
        let p = Props::new(e)?;
        let tag = match kind {
            -1 => "ExtendedTooltip",
            1 => "PictureDecoration",
            _ => "LabelDecoration",
        };
        let mut v = self.head("12", e, &p, kind.max(0))?;
        let (title, title_src) = title_pair(p.get("Title"))?;
        // у картинки TextColor выгрузка пишет равным PictureColor (поле 14 — одно на оба)
        if kind == 1 && p.get("TextColor").map(txt).transpose()? != p.get("PictureColor").map(txt).transpose()? {
            return Err(err(e, "TextColor не равен PictureColor"));
        }
        v.extend([
            title,
            p.ml("ToolTip")?,
            p.b("Enabled", "1")?,
            p.num("Width", "0")?,
            p.num("Height", "0")?,
            p.tri("HorizontalStretch")?,
            p.tri("VerticalStretch")?,
            p.color(self.x, "TextColor")?,
            p.font(self.x, "Font")?,
            shortcut(p.get("Shortcut"))?,
            a("1"),
        ]);
        let ev = events(p.get("Events"), &|n| item_event(tag, n))?;
        if kind == 1 {
            v.push(l(vec![
                a("6"),
                self.pic(e, &p, "Picture")?,
                p.b("Hyperlink", "0")?,
                p.tok("PictureSize", super::platform::PICTURE_SIZE, 0)?,
                p.b("Zoomable", "0")?,
                p.ml("NonselectedPictureText")?,
                p.color(self.x, "BorderColor")?,
                border(p.get("Border"), 0)?,
                p.b("EnableStartDrag", "0")?,
                p.b("EnableDrag", "0")?,
                ev,
                p.tok("FileDragMode", &[("AsFile", 0)], 1)?,
                p.num("ImageScale", "100")?,
                a("0"),
                p.color(self.x, "PictureColor")?,
            ]));
        } else {
            v.push(l(vec![
                a("5"),
                p.b("Hyperlink", "0")?,
                p.tok("HorizontalAlign", H_ALIGN, 0)?,
                p.tok("VerticalAlign", V_ALIGN, 3)?,
                p.num("TitleHeight", "0")?,
                ev,
                p.color(self.x, "BackColor")?,
                p.color(self.x, "BorderColor")?,
                border(p.get("Border"), 0)?,
            ]));
        }
        if kind >= 0 {
            v.extend([a("1"), self.context_menu(p.get("ContextMenu").ok_or_else(|| err(e, "нет <ContextMenu>"))?)?]);
        } else {
            v.push(a("0"));
        }
        v.extend([p.b("Visible", "1")?, p.tri("SkipOnInput")?, title_src, p.tok("ToolTipRepresentation", TOOLTIP_REPR, 0)?]);
        if kind >= 0 {
            v.extend([a("1"), self.extended_tooltip(p.get("ExtendedTooltip").ok_or_else(|| err(e, "нет <ExtendedTooltip>"))?)?]);
        } else {
            v.push(a("0"));
        }
        v.extend([
            p.b("AutoMaxWidth", "1")?,
            p.num("MaxWidth", "0")?,
            a("0"),
            p.b("AutoMaxHeight", "1")?,
            p.num("MaxHeight", "0")?,
            p.tok("GroupHorizontalAlign", H_ALIGN, 3)?,
            p.tok("GroupVerticalAlign", V_ALIGN, 3)?,
            importance(e)?,
            a("0"),
        ]);
        Ok(l(v))
    }

    /// Контекстное меню (группа вида 8, версия 22).
    pub(super) fn context_menu(&self, e: &El) -> R<Node> {
        let p = Props::new(e)?;
        let mut v = self.group_head(e, &p, 8)?;
        v.push(l(vec![a("1"), p.b("Autofill", "1")?]));
        v.extend(self.children(p.get("ChildItems"))?);
        v.extend([a("1"), a("0"), a("0"), a("0"), a("3"), a("3"), a("0")]);
        Ok(l(v))
    }

    /// Поля 0–19 группы.
    pub(super) fn group_head(&self, e: &El, p: &Props, kind: i64) -> R<Vec<Node>> {
        let mut v = self.head("22", e, p, kind)?;
        v.extend([
            p.ml("Title")?,
            p.ml("ToolTip")?,
            p.b("EnableContentChange", "0")?,
            p.b("Enabled", "1")?,
            p.b("ReadOnly", "0")?,
            p.num("Width", "0")?,
            p.num("Height", "0")?,
            p.tri("HorizontalStretch")?,
            p.tri("VerticalStretch")?,
            p.color(self.x, "TitleTextColor")?,
            p.font(self.x, "TitleFont")?,
            shortcut(p.get("Shortcut"))?,
            a("1"),
        ]);
        Ok(v)
    }

    /// Хвост группы после детей: видимость, отображение подсказки, расширенная подсказка, выравнивание в группе, важность.
    pub(super) fn group_tail(&self, e: &El, p: &Props) -> R<Vec<Node>> {
        Ok(vec![
            p.b("Visible", "1")?,
            p.tok("ToolTipRepresentation", TOOLTIP_REPR, 0)?,
            a("1"),
            self.extended_tooltip(p.get("ExtendedTooltip").ok_or_else(|| err(e, "нет <ExtendedTooltip>"))?)?,
            a("0"),
            p.tok("GroupHorizontalAlign", H_ALIGN, 3)?,
            p.tok("GroupVerticalAlign", V_ALIGN, 3)?,
            importance(e)?,
        ])
    }

    /// Кнопка (версия 34).
    fn button(&self, e: &El) -> R<Node> {
        let p = Props::new(e)?;
        let ty = p.get("Type").map(txt).transpose()?;
        let (t4, t46) = match ty.as_deref() {
            Some("CommandBarButton") => (0, 0),
            Some("UsualButton") => (1, 1),
            Some("Hyperlink") => (2, 2),
            Some("CommandBarHyperlink") => (0, 3),
            o => return Err(err(e, &format!("Type {o:?}"))),
        };
        let (loc15, loc49) = match p.get("LocationInCommandBar").map(txt).transpose()?.as_deref() {
            None => (2, 0),
            Some("InAdditionalSubmenu") => (0, 1),
            Some("InCommandBar") => (1, 2),
            Some("InCommandBarAndInAdditionalSubmenu") => (1, 3),
            Some(o) => return Err(err(e, &format!("LocationInCommandBar {o}"))),
        };
        let data_path = match p.get("DataPath") {
            Some(d) => self.data_path(d)?,
            None => l(vec![a("0")]),
        };
        // параметр команды — обратное выгрузке (form.rs, кнопка): описание типов `{"#",f5c65050…,<типы>}`, ссылка на
        // объект метаданных `{"#",fc01b5df…,<uuid>}`; нет — `{"U"}`
        let param = match p.get("Parameter") {
            None => l(vec![s("U")]),
            Some(d) => match d.req_attr("xsi:type")? {
                "v8:TypeDescription" => l(vec![s("#"), a("f5c65050-3bbb-11d5-b988-0050bae0a95d"), super::type_desc(self.x, d)?]),
                "xr:MDObjectRef" => {
                    let t = d.text()?;
                    let id = self.x.id(&t).ok_or_else(|| err(d, &format!("нет объекта {t}")))?;
                    l(vec![s("#"), a("fc01b5df-97fe-449b-83d4-218a090e681e"), a(id)])
                }
                o => return Err(err(d, &format!("параметр команды кнопки {o} пока не загружается"))),
            },
        };
        let mut v = vec![a("34"), l(vec![a(e.req_attr("id")?), a(ELEM)]), a("0")];
        match p.get("UserVisible") {
            Some(u) => v.extend([a("1"), adjustable(self.x, Some(u))?]),
            None => v.push(a("0")),
        }
        v.extend([
            an(t4),
            s(e.req_attr("name")?),
            p.ml("Title")?,
            p.b("Enabled", "1")?,
            self.command_ref(p.get("CommandName").ok_or_else(|| err(e, "нет <CommandName>"))?)?,
            data_path,
            p.tok("Representation", &[("Text", 0), ("Picture", 1), ("PictureAndText", 2)], 3)?,
            // ButtonImportance Main — кнопка по умолчанию и без DefaultButton: выгрузка платформы пишет их только парой,
            // а запись с Main и DefaultButton false выгружает обоими (опыт 28.10.2026, 3 формы БП)
            match (p.b("DefaultButton", "0")?, p.get("ButtonImportance").map(txt).transpose()?.as_deref()) {
                (_, Some("Main")) => a("1"),
                (d, _) => d,
            },
            a("0"),
            p.b("DefaultItem", "0")?,
            a("2"),
            an(loc15),
            p.num("Width", "0")?,
            p.num("Height", "0")?,
            p.num("TitleHeight", "0")?,
            p.color(self.x, "BackColor")?,
            p.color(self.x, "TextColor")?,
            p.color(self.x, "BorderColor")?,
            p.font(self.x, "Font")?,
            shortcut(p.get("Shortcut"))?,
            p.b("Check", "0")?,
            self.pic(e, &p, "Picture")?,
            p.b("Visible", "1")?,
            l(vec![s("Pattern")]),
            s(""),
            p.tri("SkipOnInput")?,
            p.tok("ToolTipRepresentation", TOOLTIP_REPR, 0)?,
            a("1"),
            self.extended_tooltip(p.get("ExtendedTooltip").ok_or_else(|| err(e, "нет <ExtendedTooltip>"))?)?,
            param,
            p.b("AutoMaxWidth", "1")?,
            p.num("MaxWidth", "0")?,
            a("0"),
            p.b("AutoMaxHeight", "1")?,
            p.num("MaxHeight", "0")?,
            p.b("HorizontalStretch", "0")?,
            p.b("VerticalStretch", "0")?,
            p.tok("GroupHorizontalAlign", H_ALIGN, 3)?,
            p.tok("GroupVerticalAlign", V_ALIGN, 3)?,
            p.tok("RepresentationInContextMenu", &[("None", 0), ("AdditionalInContextMenu", 1), ("OnlyInContextMenu", 2)], 3)?,
            // номер — порядок перечисления модели XDTO (mngcore_root.res, logform ButtonShape: Auto, Usual, Oval;
            // ButtonShapeRepresentation: Auto, Always, WhenActive, None), как и таблицы выгрузки (form.rs)
            p.tok("Shape", &[("Usual", 1), ("Oval", 2)], 0)?,
            p.tok("ShapeRepresentation", &[("Always", 1), ("WhenActive", 2), ("None", 3)], 0)?,
            an(t46),
            p.tok("PictureLocation", super::platform::BUTTON_PICTURE_LOCATION, 0)?,
            importance(e)?,
            an(loc49),
            a("1"),
            a("0"),
            a("0"),
            picture_none(),
            a("0"),
            a("0"),
            a("0"),
            p.tok("ButtonImportance", &[("Main", 0), ("Supplementary", 2)], 1)?,
            s(""),
        ]);
        Ok(l(v))
    }

    /// Команда кнопки `{<N>,<guid>}`: команда формы — `{<id>,409b9a53…}`; стандартная команда формы или элемента —
    /// GUID по таблице сверки (у элемента N — его id); команда объекта — uuid команды; стандартная команда объекта —
    /// `{<код>,<uuid объекта>}`; `N:<guid>` — как есть; `0` — `{0}`.
    pub(super) fn command_ref(&self, e: &El) -> R<Node> {
        let t = e.text()?;
        let pair = |n: &str, g: &str| Ok(l(vec![a(n), a(g)]));
        if t == "0" {
            return Ok(l(vec![a("0")]));
        }
        if let Some((n, g)) = t.split_once(':') {
            if crate::xml::imp::tpl::int(n).is_some() && crate::brace::is_guid(g) {
                return pair(n, g);
            }
        }
        if let Some(n) = t.strip_prefix("Form.Command.") {
            let id = self.cmds.get(n).ok_or_else(|| err(e, &format!("нет команды формы {n}")))?;
            return pair(id, FORM_CMD);
        }
        if let Some(n) = t.strip_prefix("Form.StandardCommand.") {
            let class = self.main.as_deref().and_then(|m| self.attrs.get(m)).map(|a| a.class.as_str()).unwrap_or("");
            // у динамического списка поставщик команд — с видом основной таблицы (Delete: platform::form_command)
            let pclass = match self.main.as_deref().and_then(|m| self.lists.borrow().get(m).map(|l| l.main.clone())) {
                Some(mt) if class == "DynamicList" => format!("DynamicList:{}", mt.split('.').next().unwrap_or_default()),
                _ => class.to_string(),
            };
            let g = form_command(n, class).or_else(|| super::platform::form_command(&pclass, n)).ok_or_else(|| err(e, &format!("стандартная команда формы {n} (основной реквизит {class:?}) не встречена")))?;
            return pair("0", g);
        }
        if let Some(rest) = t.strip_prefix("Form.Item.") {
            let (item, n) = rest.split_once(".StandardCommand.").ok_or_else(|| err(e, "команда элемента"))?;
            let it = self.items.get(item).ok_or_else(|| err(e, &format!("нет элемента {item}")))?;
            // у пары AddAutoOrderItem/Expand соответствие GUID имени не установлено — кнопкой не встречены
            let sc = it.tag == "Table" && self.settings_table(Some(it.data_path.as_str()).filter(|d| !d.is_empty())) && !matches!(n, "AddAutoOrderItem" | "Expand");
            // Delete кнопки таблицы списка со своим запросом — по составу команд таблицы списка (query::settings::list_delete)
            if let Some(l) = self.lists.borrow().get(it.data_path.as_str()).filter(|l| it.tag == "Table" && n == "Delete" && l.custom) {
                let g = crate::query::settings::list_delete(self.x, &l.main).ok_or_else(|| err(e, "команда Delete кнопки таблицы списка со своим запросом: состав команд таблицы при этой основной таблице не выведен"))?;
                return pair(&it.id, g);
            }
            let g = sc.then(|| super::table::table_command(n, false, true)).flatten().or_else(|| item_command(&it.tag, it.dynamic_list, n))
                .or_else(|| if it.tag == "Table" { super::platform::table_command(n) } else { super::platform::field_command(&it.tag, n) })
                .ok_or_else(|| err(e, &format!("стандартная команда {n} элемента {} не встречена", it.tag)))?;
            return pair(&it.id, g);
        }
        if let Some((obj, n)) = t.split_once(".StandardCommand.") {
            let class = obj.split('.').next().unwrap_or_default();
            // коды — по таблице выгрузки (`form::STD_COMMANDS`)
            let known = crate::xml::form::STD_COMMANDS.iter().find(|(c, _, s)| *c == class && *s == n).map(|(_, k, _)| k.to_string());
            let code = match (class, n) {
                _ if known.is_some() => known.as_deref().unwrap_or_default(),
                // открыть по значению ведущего измерения: 3 + его номер среди ведущих (как читает выгрузка)
                ("InformationRegister", d) if d.starts_with("OpenByValue.") => {
                    let pos = self.x.masters.get(obj).and_then(|m| m.iter().position(|x| Some(x.as_str()) == d.strip_prefix("OpenByValue.")));
                    let pos = pos.ok_or_else(|| err(e, &format!("{d}: нет такого ведущего измерения у {obj}")))?;
                    return pair(&(pos + 3).to_string(), self.x.id(obj).ok_or_else(|| err(e, &format!("нет объекта {obj}")))?);
                }
                _ => return Err(err(e, &format!("стандартная команда {class}.{n} не встречена"))),
            };
            let g = self.x.id(obj).ok_or_else(|| err(e, &format!("нет объекта {obj}")))?;
            return pair(code, g);
        }
        let g = self.x.id(&t).ok_or_else(|| err(e, &format!("нет команды {t}")))?;
        if !(t.starts_with("CommonCommand.") || t.contains(".Command.")) {
            return Err(err(e, &format!("команда {t}")));
        }
        pair("0", g)
    }

    /// Источник команд группы: нет — `{0}`, `Form` — `{0,02023637…}`, `Item.<имя>` — `{<id>,02023637…}`,
    /// `FormCommandPanelGlobalCommands` — `{0,2ef6d6fa…}`.
    fn command_source(&self, p: &Props) -> R<Node> {
        let Some(e) = p.get("CommandSource") else { return Ok(l(vec![a("0")])) };
        let t = txt(e)?;
        Ok(match t.as_str() {
            "Form" => l(vec![a("0"), a(ELEM)]),
            "FormCommandPanelGlobalCommands" => l(vec![a("0"), a("2ef6d6fa-847a-485e-8684-d37a3ab5efb8")]),
            o => match o.strip_prefix("Item.").and_then(|n| self.items.get(n)) {
                Some(it) => l(vec![a(it.id.clone()), a(ELEM)]),
                // элемента нет — выгрузка пишет `<id>:02023637…` (обратное `command_source` выгрузки)
                None => match o.split_once(':') {
                    Some((n, g)) if g == ELEM && n.parse::<u64>().is_ok() => l(vec![a(n), a(ELEM)]),
                    _ => return Err(err(e, &format!("источник команд {o}"))),
                },
            },
        })
    }

    /// Группа кнопок (вид 6): `{2,<источник команд>,2,<отображение>}`.
    fn button_group(&self, e: &El) -> R<Node> {
        let p = Props::new(e)?;
        let mut v = self.group_head(e, &p, 6)?;
        v.push(l(vec![a("2"), self.command_source(&p)?, a("2"), p.tok("Representation", &[("Usual", 1), ("Compact", 2)], 0)?]));
        v.extend(self.children(p.get("ChildItems"))?);
        v.extend(self.group_tail(e, &p)?);
        Ok(l(v))
    }

    /// Командная панель (вид 0): `{2,<расположение>,<источник команд>,0}`.
    fn command_bar(&self, e: &El) -> R<Node> {
        let p = Props::new(e)?;
        let mut v = self.group_head(e, &p, 0)?;
        v.push(l(vec![a("2"), p.tok("HorizontalLocation", &[("Center", 1), ("Right", 2), ("Auto", 3)], 0)?, self.command_source(&p)?, a("0")]));
        v.extend(self.children(p.get("ChildItems"))?);
        v.extend(self.group_tail(e, &p)?);
        Ok(l(v))
    }

    /// Подменю (вид 1): свойства вида — узел версии 8.
    fn popup(&self, e: &El) -> R<Node> {
        let p = Props::new(e)?;
        let mut v = self.group_head(e, &p, 1)?;
        v.push(l(vec![
            a("8"),
            self.pic(e, &p, "Picture")?,
            self.command_source(&p)?,
            a("2"),
            p.tok("Representation", &[("Text", 0), ("Picture", 1), ("PictureAndText", 2)], 3)?,
            p.tok("Shape", &[("Usual", 1), ("Oval", 2)], 0)?,
            p.tok("ShapeRepresentation", &[("Always", 1), ("WhenActive", 2), ("None", 3)], 0)?,
            p.color(self.x, "BackColor")?,
            p.color(self.x, "BorderColor")?,
            a("1"),
        ]));
        v.extend(self.children(p.get("ChildItems"))?);
        v.extend(self.group_tail(e, &p)?);
        Ok(l(v))
    }

    /// Страницы (группа вида 3): свойства вида `{4,<отображение>,<события>,<использование текущей строки>,0,<отображение>}`.
    fn pages(&self, e: &El) -> R<Node> {
        let p = Props::new(e)?;
        let mut v = self.group_head(e, &p, 3)?;
        const REPR: &[(&str, i64)] = &[("None", 0), ("TabsOnTop", 1), ("TabsOnBottom", 2), ("TabsOnLeftHorizontal", 3), ("TabsOnRightHorizontal", 4), ("Swipe", 5), ("Auto", 6)];
        let r5 = p.tok("PagesRepresentation", REPR, 6)?;
        v.push(l(vec![
            a("4"),
            p.tok("PagesRepresentation", REPR, 1)?,
            events(p.get("Events"), &|n| item_event("Pages", n))?,
            p.tok("CurrentRowUse", &[("Use", 0), ("DontUse", 1)], 2)?,
            a("0"),
            r5,
        ]));
        v.extend(self.children(p.get("ChildItems"))?);
        v.extend(self.group_tail(e, &p)?);
        Ok(l(v))
    }

    /// Страница (группа вида 4): свойства вида — узел версии 22.
    fn page(&self, e: &El) -> R<Node> {
        let p = Props::new(e)?;
        let mut v = self.group_head(e, &p, 4)?;
        let group = p.get("Group").map(txt).transpose()?.unwrap_or_else(|| "Vertical".into());
        // нет Group — «не задано» страницы формы конструктора 8.5, как у корня формы (`form.rs`, unset_group)
        let unset = p.get("Group").is_none();
        let (g2, g16, g17) = match group.as_str() {
            _ if unset => (1, 1, 0),
            "Vertical" => (0, 0, 0),
            "Horizontal" => (1, 1, 1),
            "HorizontalIfPossible" => (1, 2, 2),
            "AlwaysHorizontal" => (1, 1, 3),
            o => return Err(err(e, &format!("Group {o}"))),
        };
        // нет — «не задано» страницы формы конструктора 8.5: прежнее место 0, хвост 2 (загрузка платформы, 27.10.2026)
        let (scroll, scroll_tail) = match p.get("ScrollOnCompress") {
            Some(s) => {
                let v = a(if boolean(s)? { "1" } else { "0" });
                (v.clone(), v)
            }
            None => (a("0"), a("2")),
        };
        let title_path = match p.get("TitleDataPath") {
            Some(d) => self.data_path(d)?,
            None => l(vec![a("0")]),
        };
        v.push(l(vec![
            a("22"),
            self.pic(e, &p, "Picture")?,
            an(g2),
            p.tok("ChildItemsWidth", CHILD_WIDTH, 0)?,
            title_path,
            p.ml("Format")?,
            p.b("ShowTitle", "1")?,
            l(vec![s("Pattern")]),
            s(""),
            p.color(self.x, "BackColor")?,
            p.tok("HorizontalSpacing", SPACING, 0)?,
            p.tok("VerticalSpacing", SPACING, 0)?,
            p.tok("HorizontalAlign", super::platform::H_ALIGN, 3)?,
            p.tok("VerticalAlign", V_ALIGN, 3)?,
            p.tok("ChildrenAlign", CHILDREN_ALIGN, 0)?,
            scroll,
            an(g16),
            an(g17),
            color_auto(),
            font_auto(),
            an(if unset { 4 } else { g17 }),
            scroll_tail,
            a("0"),
            a(if p.get("ShowTitle").map(boolean).transpose()? == Some(false) { "0" } else { "2" }),
        ]));
        v.extend(self.children(p.get("ChildItems"))?);
        v.extend(self.group_tail(e, &p)?);
        Ok(l(v))
    }

    /// Обычная группа (вид 5): свойства вида — узел версии 38.
    fn usual_group(&self, e: &El) -> R<Node> {
        let p = Props::new(e)?;
        let mut v = self.group_head(e, &p, 5)?;
        let group = p.get("Group").map(txt).transpose()?.unwrap_or_else(|| "Vertical".into());
        // нет Group и Representation — «не задано» обычной группы Конфигуратора 8.5: прежние места (1, 1, 2) и 2,
        // хвост 4 (`form_v59::UNSET_GROUP`, `UNSET_REPRESENTATION`; эталон 28.10.2026)
        let unset = p.get("Group").is_none();
        let (g1, g22, g27) = match group.as_str() {
            _ if unset => (1, 1, 2),
            "Vertical" => (0, 0, 0),
            "Horizontal" => (1, 1, 1),
            "HorizontalIfPossible" => (1, 2, 2),
            "AlwaysHorizontal" => (1, 1, 3),
            o => return Err(err(e, &format!("Group {o}"))),
        };
        let behavior = p.get("Behavior").map(txt).transpose()?;
        let (b10, b24, b28) = match behavior.as_deref() {
            None => (1, 0, 3),
            Some("Usual") => (0, 0, 0),
            Some("Collapsible") => (1, 1, 1),
            Some("PopUp") => (1, 2, 2),
            Some(o) => return Err(err(e, &format!("Behavior {o}"))),
        };
        let (repr, repr_tail) = match p.get("Representation") {
            Some(r) => {
                let t = token(r, &[("None", 0), ("StrongSeparation", 1), ("WeakSeparation", 2), ("NormalSeparation", 3)])?;
                (t.clone(), t)
            }
            None => (a("2"), a("4")),
        };
        let show_title = p.get("ShowTitle").map(boolean).transpose()?.unwrap_or(true);
        // путь заголовка — поле 5 свойств вида (выгрузка form.rs, вид 5), как у страницы
        let title_path = match p.get("TitleDataPath") {
            Some(d) => self.data_path(d)?,
            None => l(vec![a("0")]),
        };
        let sub = vec![
            a("38"),
            an(g1),
            p.tok("ChildItemsWidth", CHILD_WIDTH, 0)?,
            repr,
            a(if show_title { "1" } else { "0" }),
            title_path,
            p.ml("Format")?,
            l(vec![s("Pattern")]),
            s(""),
            p.color(self.x, "BackColor")?,
            an(b10),
            p.tok("ControlRepresentation", &[("Button", 1)], 0)?,
            p.b("Collapsed", "0")?,
            p.b("ShowLeftMargin", "1")?,
            p.ml("CollapsedRepresentationTitle")?,
            p.tok("HorizontalSpacing", SPACING, 0)?,
            p.tok("VerticalSpacing", SPACING, 0)?,
            p.tok("HorizontalAlign", super::platform::H_ALIGN, 3)?,
            p.tok("VerticalAlign", V_ALIGN, 3)?,
            p.tok("ThroughAlign", &[("Use", 0), ("DontUse", 1)], 2)?,
            p.tok("ChildrenAlign", CHILDREN_ALIGN, 0)?,
            p.b("United", "1")?,
            an(g22),
            // поле 23 — HiddenStateTitleBackColor: mngcore FUN_180351380 пишет цвет +0x15c (геттер +0x160 интерфейса группы)
            p.color(self.x, "HiddenStateTitleBackColor")?,
            an(b24),
            // выгрузка form.rs, вид 5: поле 25 — CurrentRowUse ("", DontUse), умолчание 2
            p.tok("CurrentRowUse", &[("DontUse", 1)], 2)?,
            a("0"),
            an(g27),
            an(b28),
            a("0"),
            a("0"),
            a("0"),
            picture_none(),
            l(vec![a("0"), a("1"), a("0")]),
            a("0"),
            repr_tail,
            an(if unset { 4 } else { g27 }),
            a("0"),
            a("2"),
            a("0"),
            a("0"),
            a(if show_title { "2" } else { "0" }),
        ];
        v.push(l(sub));
        v.extend(self.children(p.get("ChildItems"))?);
        v.extend(self.group_tail(e, &p)?);
        Ok(l(v))
    }
}
