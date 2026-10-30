//! Таблицы из кода платформы 8.5.1.1529 (статический разбор DLL, Ghidra; см. docs/research/xml-import.md,
//! «Таблицы из кода платформы»): имена стандартных команд → GUID по поставщикам команд и значения перечислений
//! XML управляемой формы. Выборки сверки БП (`cmdset.rs`, `items.rs`, `table.rs`) — подмножество этих таблиц
//! (тест `samples_are_subsets`); имя, которому у поставщика соответствуют разные GUID, здесь не решается.

/// GUID стандартной команды формы `name` у формы с основным реквизитом класса `class` (как в `cmdset.rs`:
/// `DocumentObject`, `DynamicList:<вид основной таблицы>`…): команды самой формы, затем поставщика основного
/// реквизита. `None` — нет у поставщика или имя у него не однозначно.
pub(super) fn form_command(class: &str, name: &str) -> Option<&'static str> {
    if let Some(g) = find(FORM, name) {
        return Some(g);
    }
    if let Some(table) = class.strip_prefix("DynamicList") {
        // Delete динамического списка — два GUID, каждый — команда таблицы списка (mngbase FUN_180612330): 1cc781aa ↔
        // 8d772f97, 3dd3bd8a ↔ ec576e13; в составе команд таблицы (FUN_180611300) только одна из них: ec576e13 — у
        // ссылочных видов, журнала и задачи, 8d772f97 — у регистра сведений (FUN_18060d500: независимый и не по позиции
        // регистратора; иначе — интерфейс 2b98afb7 +0x28, не разобран). shortcut: здесь нет свойств регистра —
        // регистр сведений принят независимым (как `table::list_delete` для команды таблицы); подчиненный регистр
        // сведений со списком формы и исключенной Delete — передать сюда свойства и отказать
        if name == "Delete" {
            return match table.trim_start_matches(':') {
                "InformationRegister" => Some("1cc781aa-f32b-4dc7-996a-6c38c3deda5c"),
                "Catalog" | "Document" | "DocumentJournal" | "ChartOfCharacteristicTypes" | "ChartOfAccounts" | "ChartOfCalculationTypes" | "ExchangePlan"
                | "BusinessProcess" | "Task" => Some("3dd3bd8a-ac1e-44d6-ac83-e7802642a5e2"),
                _ => None,
            };
        }
        return unique(LIST, name);
    }
    match class {
        c if c.ends_with("ReportObject") => find(REPORT, name),
        "dcsset:SettingsComposer" => find(COMPOSER, name),
        c if c.ends_with("DataProcessorObject") => None,
        c if c.ends_with("Object") || c.contains("RecordManager") || c.contains("RecordSet") || c == "ConstantsSet" => find(OBJECT, name),
        _ => None,
    }
}

/// GUID стандартной команды таблицы (поставщик команд элемента «Таблица»); имена с двумя GUID — `None`.
pub(super) fn table_command(name: &str) -> Option<&'static str> {
    unique(TABLE, name)
}

/// GUID стандартной команды поля табличного документа или поля документа (форматированного, PDF, календаря);
/// имена с несколькими GUID — `None`.
pub(super) fn field_command(tag: &str, name: &str) -> Option<&'static str> {
    match tag {
        "SpreadSheetDocumentField" => find(SPREADSHEET, name).or_else(|| unique(DOCUMENT, name)),
        // разделы цепочки по видам поля — по строкам описаний IDS_FIELDCMD_FORMATTEDDOC*, IDS_FIELDCMD_PLANNERFIELD*,
        // IDS_FIELDCMD_PDFDOCUMENTFIELD* (mngbase.dll FUN_1803eca50): форматированный документ — до Today, PDF — с ScaleUp
        "FormattedDocumentField" => find(&DOCUMENT[..DOC_PLANNER], name),
        "PDFDocumentField" => find(&DOCUMENT[DOC_PDF..], name),
        "CalendarField" => unique(DOCUMENT, name),
        _ => None,
    }
}

// Значения перечислений XML формы: таблицы (значение, токен) mngcore.dll, каждую читает свой объект перечисления
// (фабрика с границами таблицы, например FUN_1803a54c0 → [0x18078a850, 0x18078a890)). Связь «элемент XML → таблица»
// в коде не прослежена: таблица выбрана как единственная, согласная со всеми значениями выборки сверки у этого
// элемента; значение записи = значению перечисления (так у всех значений выборок).

/// Горизонтальное положение: mngcore 0x18078a890 (и копия 0x18075d540). Выборки: HorizontalAlign, HeaderHorizontalAlign.
pub(super) const H_ALIGN: &[(&str, i64)] = &[("Left", 0), ("Center", 1), ("Right", 2), ("Auto", 3)];
/// Фиксация в таблице: mngcore 0x18078a060.
pub(super) const FIXING_IN_TABLE: &[(&str, i64)] = &[("None", 0), ("Left", 1), ("Right", 2)];
/// Положение картинки кнопки: mngcore 0x180789160 (выборка Right=2 согласна и с 0x18078a060 None/Left/Right — Left=1
/// у обеих; у 0x1807400b0 Left=0, но там Left — значение по умолчанию, которое выгрузка не пишет).
pub(super) const BUTTON_PICTURE_LOCATION: &[(&str, i64)] = &[("Auto", 0), ("Left", 1), ("Right", 2), ("None", 3)];
/// Размер картинки: mngcore 0x18078a380 (и копия 0x18075d790).
#[rustfmt::skip]
pub(super) const PICTURE_SIZE: &[(&str, i64)] = &[
    ("RealSize", 0), ("Stretch", 1), ("Proportionally", 2), ("Tile", 3), ("AutoSize", 4), ("RealSizeIgnoreScale", 5),
    ("AutoSizeIgnoreScale", 6), ("ByFontSize", 7),
];
/// Специальный режим ввода текста: mngcore 0x180789360.
#[rustfmt::skip]
pub(super) const SPECIAL_TEXT_INPUT_MODE: &[(&str, i64)] = &[
    ("Auto", 0), ("None", 1), ("DigitsAndPunctuation", 2), ("URL", 3), ("Email", 4), ("PhoneNumber", 5), ("Digits", 6),
];

fn find(t: &[(&str, &'static str)], name: &str) -> Option<&'static str> {
    t.iter().find(|(n, _)| *n == name).map(|(_, g)| *g)
}

fn unique(t: &[(&str, &'static str)], name: &str) -> Option<&'static str> {
    let mut it = t.iter().filter(|(n, _)| *n == name).map(|(_, g)| *g);
    it.next().filter(|_| it.next().is_none())
}

/// Команды самой формы (у всех форм): mngbase.dll FUN_1802fb3c0 — описание команды по GUID (цепочка сравнений GUID → `core::term_string(<пара имен en/ru>)`).
#[rustfmt::skip]
const FORM: &[(&str, &str)] = &[
    ("Close", "3772996b-41f4-4c47-a5a8-ea397db424ae"),
    ("Help", "39bb0fe9-771d-4dd5-8a6e-2d16984523af"),
    ("CustomizeForm", "198ea630-fda2-4cda-8a23-f999f4c67ee6"),
    ("OK", "f3613d5c-20c6-46e5-b4d5-7d712ece1296"),
    ("Cancel", "679b62d9-ff72-4329-bf3a-c0c32b311dd2"),
    ("Yes", "5d41082e-9619-42ec-b96f-98b082b3a2f0"),
    ("No", "06ee6a21-061e-47f8-81c5-92ae8b8f3b5d"),
    ("Retry", "5174ad3f-0569-42fd-8adf-011d8206db6c"),
    ("Abort", "3f01ed62-97f8-465b-b4f7-6517ac2bc994"),
    ("Ignore", "d7e9e72c-8fa7-430c-a3e9-aeadfd57dfc7"),
    ("SaveValues", "239f0103-8de9-4fdf-b485-eb5531da7e51"),
    ("RestoreValues", "71e0226e-ebb2-4e33-8745-0a94a01bbf15"),
    ("OpenFromMainServer", "573e81b7-57eb-45f0-ba4d-ada7c2537a2d"),
    ("OpenFromStandaloneServer", "0ea1a92b-3477-44dd-b152-ea7d411f1c5d"),
];

/// Команды элемента «Таблица» (обычная, динамического списка, настроек компоновщика): mngbase.dll FUN_18045caa0 (описание команды по GUID); Delete, Choose, OutputList, AddGroup, Ungroup — по два GUID.
#[rustfmt::skip]
const TABLE: &[(&str, &str)] = &[
    ("CopyToClipboard", "88078230-1f6b-415f-99e4-ad2ff73810cf"),
    ("SearchEverywhere", "7b683784-b474-441a-ba63-3d757bd0ffd4"),
    ("Add", "b0016a68-ec64-4e6d-b905-c71fd62efc4c"),
    ("CheckAll", "18248aa8-e621-4e19-a611-54fb8923644c"),
    ("UncheckAll", "5048cc44-702b-44e3-8445-9af75c02724d"),
    ("Create", "0f8d6d98-2f8b-405a-b8b3-0538e9d95da5"),
    ("CreateByParameter", "b59f3c87-e213-4947-abae-9dbaffaef147"),
    ("CreateFolder", "d82ca05c-2966-4d77-9a39-a1eea087bfa7"),
    ("Copy", "0ae4bea5-23be-42a7-b69e-97b11b29c453"),
    ("Delete", "ec576e13-1e76-4c33-98aa-a33204514227"),
    ("Delete", "8d772f97-c0ef-47c0-9cb0-efea28c61341"),
    ("SetDeletionMark", "a2f737a8-0114-4e86-a214-45e5c213fa65"),
    ("Post", "e3dd8850-fc3c-41b1-bbb3-7c66af082608"),
    ("UndoPosting", "04ac7211-e74f-4776-9749-35a9282b1d52"),
    ("SetDateInterval", "daa306cd-a78a-4e74-a14c-739daba624cb"),
    ("Change", "b41f5bbc-ba5d-4888-8cd1-db246a371418"),
    ("MoveUp", "37740564-9e86-44a0-bea9-3f485a5a3f91"),
    ("MoveDown", "fa51b106-eae6-44c7-8054-76cbb3100603"),
    ("SortListAsc", "2bbe4e12-06d2-409b-a972-eea585125d83"),
    ("SortListDesc", "58b2a785-23f6-4b0e-a324-9a1323285595"),
    ("Pickup", "59b4387d-f5be-4658-901f-bd3068217469"),
    ("Refresh", "403bc6e6-b98e-4181-9f43-9c75cbbf82cf"),
    ("Filter", "1590e15e-ac25-42d2-9e38-6022ccba48ec"),
    ("EndEdit", "9ef79140-3de6-436a-8dda-610bb963f5db"),
    ("Choose", "8969c93a-23e5-4bef-941d-aaef315858d2"),
    ("EditList", "ef1f648a-27f6-46f1-bd7b-9f8c00775ba7"),
    ("LevelUp", "0e9b637d-cf6e-4330-8a8f-cd44842e34bb"),
    ("LevelDown", "dc118d99-b351-4e30-9310-e864f2e53ec0"),
    ("OutputList", "49602716-fea6-497f-8047-726404038857"),
    ("MoveItem", "27bd521a-51c6-4fe7-846d-a98f988774b5"),
    ("Find", "c0519548-2a9a-44de-a25e-faf01e089d4d"),
    ("CancelSearch", "44ad3ec9-f3c2-4913-9224-5f9fb6418743"),
    ("FindByCurrentValue", "714d44cc-63da-4431-b33a-428e398d2a08"),
    ("OutputList", "825c1c15-ef8f-47ab-b002-e6b84b3e5b10"),
    ("ListSettings", "14559f7c-853c-42a4-9ea1-01546107747b"),
    ("List", "0d0249a4-2b2f-4fc0-a66f-b36f9494b3cc"),
    ("HierarchicalList", "01833a5a-6553-4c49-b445-095018107bb5"),
    ("Tree", "05468165-f954-45a5-84f2-6641c51f9f23"),
    ("AddMultiple", "6db04d66-8367-4b9d-a368-16742c09f654"),
    ("AddGroup", "7b70c79a-199e-4e87-a7eb-29dea9a5ad69"),
    ("AddTable", "46647493-f1bf-4cd6-9110-6bfab80b62de"),
    ("AddChart", "a10f1c0b-73ec-448f-b6d2-be0c86e95712"),
    ("AddNestedSchema", "e809ae75-11b6-480d-bc87-caf93b28236d"),
    ("AddSelectedField", "481fc962-f834-43a5-92c0-c937fb2d33e7"),
    ("AddSelectedFieldGroup", "29258972-8ac5-4a5a-8586-742fcf406695"),
    ("AddAutoSelectedField", "eaab4e19-897e-464c-9674-262fddb1dce9"),
    ("AddFilterItem", "fca750bc-4fb6-40e2-ae0f-e818939a32e7"),
    ("AddFilterItemGroup", "a5fdef31-bbf0-4a9d-98aa-fd5fd8f1344a"),
    ("AddUserFieldCase", "895cef27-7ab6-4605-b05a-8f672eb3e1d2"),
    ("AddUserFieldExpression", "0e0b5073-d049-4255-aa02-03eefea68bc0"),
    ("AddOrderItem", "62ff963c-9426-43af-bb23-1d2ef3a9a0c1"),
    ("AddAutoOrderItem", "48e12019-0fd6-46eb-aab6-2acba716a623"),
    ("AddGroupField", "2b51fffd-a2a6-40b6-9f1a-fbee79cd4df7"),
    ("AddAutoGroupField", "3870086b-23c3-472c-b54d-85789649d01d"),
    ("AddGroup", "1528a1e1-be7c-433e-8c6e-788e4a9e87b9"),
    ("Group", "33ff70c9-5df3-4907-9611-7649411f9180"),
    ("GroupFilterItems", "4a817da0-5797-4e16-906f-02fb869e1873"),
    ("GroupSelectedFields", "385b7651-6417-4e04-b035-37815425224b"),
    ("Ungroup", "23802256-7145-47c7-b379-8d60ca1b1262"),
    ("Ungroup", "82b88a24-2856-484a-afd9-55a15bdf9785"),
    ("UngroupSelectedFields", "b659b014-d383-4dc0-87fa-312d1e4c4697"),
    ("StandardSettings", "3bd8cc97-31ca-4fad-acf4-cc8f4d648a95"),
    ("LoadSettings", "358196aa-1061-458a-8fba-e9cd11081205"),
    ("SaveSettings", "49a25ff2-06bc-4547-a119-a428f60bdfbf"),
    ("Expand", "fc120c02-7f39-469b-b357-b2dd8d4b0765"),
    ("SetTitle", "c71001c4-02f6-4036-89a3-8f54f9435176"),
    ("SetPresentation", "7d4db5ed-0981-4020-b3b8-886b7165ba05"),
    ("Detailed", "e6900951-1a42-4397-bf00-cabb2cd7ad6d"),
    ("Choose", "d77e5787-b130-4355-8f8f-01ecec82f843"),
    ("ChooseAll", "15664824-eedc-4a92-9f6b-c89a2dead157"),
    ("UserSettingItemProperties", "1f1e900a-8488-4159-81be-9704eb96906d"),
    ("UserSettings", "329bb47c-392f-4779-a1af-347d06bb624b"),
    ("SelectAll", "51c99108-107c-43e1-8918-e48835bf2495"),
    ("UseFieldAsValue", "d7e55d2e-bfea-4d80-b4ad-a1bb31ec2147"),
    ("EditGroupFields", "b916f3dc-cb7d-4034-8552-f906f3def462"),
    ("TurnUseOn", "eeec121e-c48c-42fa-8590-4c02d875ddb6"),
    ("TurnUseOff", "d45eb10a-73c2-4cb4-9502-1892842aa22a"),
    ("WriteChanges", "47e5fcad-79e8-401f-a58d-a1ce409b5399"),
    ("ReadChanges", "6577d007-771a-48b6-a2ec-66b7c926ceef"),
    ("CreateInitialImage", "e86afc8e-47c8-4cfd-8c92-2e10b2f67f53"),
    ("SwitchActivity", "2ccaf251-ee77-440f-a6e5-16f88825c6c0"),
    ("ShowMultipleSelection", "e7216412-03ac-4a81-99c2-1d7c28e88e31"),
    ("ShowRowRearrangement", "8af6ebff-cd02-4bfe-a984-44a292623708"),
    ("SearchHistory", "d96b0c03-b209-4d01-a3fc-17a14f873b64"),
    ("SaveDynamicListSettings", "95b4bc12-2ece-4d7a-b3e2-6f9293620a06"),
    ("LoadDynamicListSettings", "182a793b-22a5-4625-b316-6a5be7f88078"),
    ("DynamicListStandardSettings", "33b7b9cd-6979-4435-8c58-d9bc8250edec"),
    ("ChangeHistory", "11761e12-cf32-4826-a175-b23213e3b229"),
    ("GetURL", "0e36114c-5b59-4005-9426-374a6c067e4a"),
    ("ClearTableMarksAppearance", "daf40cdf-c477-48c5-9627-57d5450e1f3d"),
    ("DeselectAll", "03022d03-1986-4dc4-93c2-03d995332357"),
];

/// Команды поля табличного документа: mngbase.dll FUN_1803f1500.
#[rustfmt::skip]
const SPREADSHEET: &[(&str, &str)] = &[
    ("Print", "d673d512-f71a-48a6-ae5d-527a64ffd813"),
    ("FixTable", "b34f83ab-1cd7-41b1-89bd-dd0804a47b26"),
    ("Preview", "5aa38159-2001-42ae-8451-f8cabe0762c3"),
    ("PrintImmediately", "0e355e57-d603-4ac6-998b-c522c43d3668"),
    ("PageSetup", "41c1bb40-1027-4fd1-a19e-17976100e64b"),
    ("Find", "0cf34151-92d3-42fd-954f-5938433908a4"),
    ("CopyToClipboard", "1ba33890-92e9-42a3-95bd-a5c783f46d55"),
    ("SearchEverywhere", "ff533ae0-46a9-4e1d-aa3a-6dffa27e076b"),
    ("PasteFromClipboard", "edf14e37-e755-4d1c-970c-48ed776e3a0e"),
    ("Save", "d8e20c4d-3519-49aa-80e5-d6d66fee741a"),
    ("SaveAs", "999b786e-0534-4fca-8b62-f706d5336798"),
    ("SelectAll", "cca9b248-5f35-477f-a49d-95da3b7becad"),
    ("FindNext", "29fc1bfc-7bf8-4849-9b4b-3e42d12ebcbb"),
    ("FindPrevious", "474180f3-c5bd-49bf-bfd4-fddb03918295"),
    ("ShowGroups", "b55ad06a-ee91-4435-a747-6f51884772d9"),
    ("CollapseAllGroups", "ff5c34f8-b172-4ef2-91d3-48283a66a725"),
    ("ExpandAllGroups", "12acffde-8389-4e5e-bd86-ff248262d84a"),
    ("Edit", "5c52ec51-6000-4190-b8c8-bd6201a271f5"),
    ("Font", "05e95a55-947e-4dde-a657-16ec11750a2f"),
    ("BorderLeft", "635136cc-3a47-43d1-8893-d7d536bf37c4"),
    ("BorderTop", "81be7ad1-a93f-4936-92d0-d59c10213f28"),
    ("BorderRight", "e05dddd1-40ad-44a2-8ae7-0a551f0b1809"),
    ("BorderBottom", "5ff37d88-ba18-428c-bf49-9ccde4c46268"),
    ("BorderAll", "4efdcc95-9f24-4652-a5ed-febfaa51f135"),
    ("BorderOutline", "05183b65-aa1f-401c-ac34-d99a6ff4ff60"),
    ("BorderInside", "c4713141-07d1-411f-8804-172d4b8c4f01"),
    ("ThickBorderOutline", "08872d94-c57a-470f-a050-0d4d81765df2"),
    ("ThickBorderTop", "a01654df-d7f1-4ec5-8b03-258d953de2e7"),
    ("ThickBorderBottom", "65ee8b9b-8a0a-4de8-9a71-3a7a8c43c4ba"),
    ("BorderNone", "1177792e-7da2-4755-81a9-997f2d0e3dce"),
    ("BorderColor", "c1c95e8a-37b4-477c-9df1-7bc0fdbc1bd3"),
    ("Merge", "f4df676f-eefb-48bd-9117-83ee6c207cb5"),
    ("InsertRowsTop", "4ecc8cf1-2a26-446f-9fb6-93db4ffee068"),
    ("InsertRowsBottom", "1d6dbce7-a813-437b-89b8-450319ed13bd"),
    ("DeleteRows", "7b63d2c2-5b6b-472e-8f6c-7438d952be73"),
    ("InsertColumnsLeft", "468dca2b-17be-4657-bae8-64b94fcf6187"),
    ("InsertColumnsRight", "0a2d962b-5178-4fce-983b-19068b919f41"),
    ("DeleteColumns", "202efa3a-32e6-482c-8bc9-1a596bdd57f4"),
    ("ClearAll", "59e67a77-8141-42cf-b062-7cb92e210b6d"),
    ("ClearContent", "7eae9c22-db31-4f27-a56a-b4dd62d21a2c"),
    ("Names", "feb51db7-bc1f-4b9f-a6e6-db24d5f812ab"),
    ("ShowRowAndColumnNames", "08fdfb5b-192a-41a9-b57a-9781cd3ef7b6"),
    ("ShowCellNames", "0c66c888-7512-402c-941d-96bec0e5749a"),
    ("SetName", "5ce7de77-4796-41a0-beb3-7dc420053639"),
    ("RemoveName", "adc3d2d0-4d84-4038-a453-ab5d693a60bd"),
    ("ShowGrid", "83c3121e-60cc-4b47-a296-0c1976f0d766"),
    ("ShowHeaders", "4a9ec98e-c814-47b6-911f-694effc10fc5"),
    ("ShowComments", "95dbc17e-d11e-4008-b9a9-24d5f5b1d061"),
    ("BlackAndWhiteView", "9e525e9b-99ed-4d89-9f02-2bf449ba65e6"),
    ("PageViewMode", "1c7e6bb5-54ac-4ebf-8823-e92b3cf629da"),
    ("CombineToGroup", "4402cb7a-f68e-44cd-9478-52a695b18a25"),
    ("RemoveFromGroup", "69333d9f-28d1-446b-bd9a-cf8f85cf1704"),
    ("HeaderFooter", "2da58c85-ae4d-403f-b0e2-c50027a5467f"),
    ("SetPrintArea", "6728e5c7-8f67-4b0d-bd6f-90b728218fe3"),
    ("RemovePrintArea", "41f3fbde-476a-4984-bd12-b32e990af811"),
    ("RepeatOnEachPage", "c50fd6b2-51a1-47e0-8cd3-84b16823287c"),
    ("RemoveRepeatOnEachPage", "0e8c7cb4-f146-4208-af36-b3f8c7d71b66"),
    ("InsertPageBreak", "952af05e-0771-4c26-adb6-a3418a262e4a"),
    ("RemovePageBreak", "3a7ef674-f589-4734-9b22-954ea64dc79f"),
    ("InsertComment", "ed6630f2-c296-43dd-b408-d370513fcebc"),
    ("DeleteComment", "531b9a9b-e4b3-4fe2-9562-1ff678c6d73d"),
    ("PreviousComment", "e1ae173a-22c3-4909-a72c-5454b64c6446"),
    ("NextComment", "3e15759b-551a-46c4-8d24-8d6df22a1a64"),
    ("GoToCell", "25d773e7-9961-49fc-a9c8-527079090143"),
    ("InsertRows", "e5c3a5a6-695d-41bf-9c88-4367fd2a2a6e"),
    ("SplitCell", "02bdf755-7bc7-4e41-a199-f65cef10e6bc"),
    ("Hide", "b573b54a-ce87-4078-bd21-4f06709157c6"),
    ("Show", "1b680da5-a5ca-4ea7-8db9-df079de39b61"),
    ("RowHeight", "17b9f6bb-74b3-439d-b719-eb236b2fe001"),
    ("ColumnWidth", "97407339-2c9f-400b-bd5b-3d97b6d00c21"),
    ("Picture", "a97ea34e-7af2-412c-aa9d-b3393b1914ac"),
    ("Text", "80455469-5f1c-4817-a992-756dfee9138f"),
    ("Rectangle", "852f0fba-4338-4c43-a2da-851fcffd07bb"),
    ("Line", "c9b9e671-7c9b-44b5-97e9-dd1ee51a1bfd"),
    ("Ellipse", "93d90e38-02a4-42f8-828a-2798f51c4500"),
    ("Group", "e406e2a0-f06b-4402-b8c3-9017c95df44c"),
    ("Ungroup", "88a56d46-abff-4925-91a2-6592a4664912"),
    ("AlignDrawingLeft", "ee0aab77-fd5f-4594-9c5e-e989a953642f"),
    ("AlignDrawingCenter", "f2b6b156-d929-4be2-af5b-9c9b792524bb"),
    ("AlignDrawingRight", "80a0b41c-24df-40e4-8269-683fb557214d"),
    ("AlignDrawingTop", "f9395bfa-9301-4cec-8c1e-e2b62fb3abd6"),
    ("AlignDrawingMiddle", "719daaab-c2d0-473d-b373-faf18ebe7d9d"),
    ("AlignDrawingBottom", "9f71febd-8c22-4471-8410-31f455bb3c57"),
    ("EqualDrawingWidth", "5ccf1fce-3fab-4fb6-ac04-a9b2cf689cee"),
    ("EqualDrawingHeight", "f5773ab5-4036-49ca-8286-7a4ea2c354d7"),
    ("EqualDrawingSize", "fd523437-4160-4a52-a70b-9166c7eebcf0"),
    ("DistributeDrawingsVertically", "7f3f496d-506c-4239-98fb-58e1ea6ba54a"),
    ("DistributeDrawingsHorizontally", "60abcc40-dc62-4d03-833b-7b8ab8232d2c"),
    ("BringDrawingForward", "49a22a23-d2cf-4f84-97ae-66f94f863145"),
    ("SendDrawingBackward", "7e79f8d3-6cab-49d5-aac0-43f5056ed958"),
    ("BringDrawingToFront", "b383fa5a-2324-4e7e-a166-aabb5d64aea3"),
    ("SendDrawingToBack", "14bd1c58-da9d-41db-a515-75f8b39fdc52"),
    ("FormattedString", "96a49a89-ccb4-4b16-8e6e-1b0e678f67a1"),
];

/// Начало раздела планировщика (Today) и поля PDF (ScaleUp) в [`DOCUMENT`].
const DOC_PLANNER: usize = 32;
const DOC_PDF: usize = 39;

/// Команды полей документов (форматирование текста, PDF, календарь; общая часть, в mngbase.dll шесть копий: FUN_1803eca50, FUN_1803fb150, FUN_18040a410, FUN_18040f2b0, FUN_180414a20, FUN_18041e6e0); Print, Preview — по нескольку GUID.
#[rustfmt::skip]
const DOCUMENT: &[(&str, &str)] = &[
    // первое сравнение функции разбором не снято: GUID из реестра стандартных команд mngcore.dll FUN_18037afe0, имя — сверка
    ("Bold", "f20eefc2-f819-4ab1-be67-87b3ca2e26e6"),
    ("Italic", "a8631f01-318a-4da2-80a9-9075c7524463"),
    ("Underline", "85bd789b-0047-46f9-9b2e-845907fc1b1d"),
    ("Strikeout", "db1cd9b3-bdf4-43f5-abd6-c2e4bd85d709"),
    ("IncreaseIndent", "d0a4d953-115b-4059-a6cb-6e67f903a4f3"),
    ("DecreaseIndent", "39f6b9f1-7aa1-4a03-a01b-e127d51bc228"),
    ("BulletedList", "a0033f06-56f7-4855-b901-7ac66fe1bb99"),
    ("NumberedList", "2b5d0007-b74c-4786-a904-37e64bda8414"),
    ("Hyperlink", "6d83186a-5838-40a5-95e7-8990193adf0a"),
    ("Picture", "9d8a3915-de52-4227-91cd-2fce22e09972"),
    ("Char", "871100d5-049d-4b22-a46a-fabf54bd64c3"),
    ("AlignLeft", "87ecfbdd-8e2b-4ba2-a315-0897020f382f"),
    ("AlignCenter", "ab0ebc39-68ee-4034-b2f4-43eee55bd651"),
    ("AlignRight", "e428af27-c4f7-4577-b80e-95a79f94322d"),
    ("AlignJustify", "56ae90b6-588f-406e-919c-cc5cc7f86297"),
    ("Properties", "be8800c3-8ccf-444a-bbf0-8f3078ff0ded"),
    ("LineSpacing", "408f351e-0536-46be-8916-a891db9bfbe6"),
    ("Font", "a8f6b59e-b712-4d3e-a974-55a3be4eb295"),
    ("IncreaseFontSize", "0bdc43a3-79f0-48d6-bce4-1142542e1a59"),
    ("DecreaseFontSize", "ec647dcc-2be7-486c-9046-d8b371f9909e"),
    ("BackColor", "17724105-6e59-4d52-8a42-cf0fb4838037"),
    ("TextColor", "71007f7d-1995-44aa-9125-9926e70a35b5"),
    ("Undo", "f5814962-2bef-43dd-b633-a193d4b0970e"),
    ("Redo", "6f1ea963-0807-4de8-b544-b5666f500b05"),
    ("CopyToClipboard", "7a294bdc-b86b-4b73-abc4-df9c811f61ef"),
    ("SearchEverywhere", "6e2f7ea0-a346-4c78-96d9-a0f512000910"),
    ("PasteFromClipboard", "905692d2-c3e7-4433-8f10-8d2ce35f652b"),
    ("CutToClipboard", "83670388-2e45-439e-9968-587eca6c7f8d"),
    ("Print", "a8483976-8b13-416a-9680-133b306dc6b0"),
    ("Preview", "4ca32834-6f9f-4dfb-89ce-6db36931c89b"),
    ("SelectAll", "b67f202a-dcf8-41f3-bda8-1ff9bed5f2ef"),
    ("SaveAs", "5a331cec-bf93-4af5-8f51-80fd7118db47"),
    ("Today", "8480b02c-2c1d-4f46-9b35-0d9144c066b6"),
    ("Forward", "1a2d66b8-f9e7-4aa8-bb03-dd83d3259299"),
    ("Back", "8e2632b7-adee-4525-9254-7c4d971944de"),
    ("Begin", "4bc313f4-a302-4147-b095-a590f2049253"),
    ("End", "6d89d147-5c33-41c1-b23e-282f3f6d7545"),
    ("Print", "52712954-ebf3-4b33-bdec-905ec26c72db"),
    ("Preview", "2c75e90f-36f0-48c8-913d-0d92afdb4b93"),
    ("ScaleUp", "d9117be6-f436-40fd-9670-8d71b99b2477"),
    ("ScaleDown", "1e5ebd8b-32ee-4af9-b85c-3a0417000660"),
    ("Scale", "b070babd-edd9-4915-9538-3c2d25607304"),
    ("GoToBegin", "a4e92b1d-5e86-4a86-b276-00e71ecf3fd4"),
    ("GoBack", "32a87619-85ce-495a-a195-2719f5e9c71e"),
    ("GoForward", "85a1b7ee-e94d-4783-b519-4af123f58596"),
    ("GoToEnd", "3e80231d-e169-456e-8373-0b9d4c15c8ab"),
    ("GoToPage", "4d567f43-e428-4aea-9b3f-a228567bba4d"),
    ("RotateClockwise", "30f0b852-3284-4ae4-838b-f89b78388bdc"),
    ("RotateCounterclockwise", "e28575de-9fe8-4a8e-a285-13250de33d49"),
    ("VerifySignatures", "73a51a95-670d-4aad-94d1-6da2f5df8222"),
    ("SignatureProperties", "f04aeb5d-87f9-42c2-836c-1cce8a6c4bf3"),
    ("Print", "67916de8-b666-4d6e-b899-29433f416ac8"),
    ("ConvertToGrayScale", "f42d80c5-9a19-4be3-9f32-91fa5100caeb"),
];

/// Команды объекта основного реквизита (запись, проведение, задачи, бизнес-процессы, планы обмена): объединение поставщиков basic.dll FUN_18017f150, FUN_18033a9d0, FUN_18033ed80, FUN_180435150, FUN_1804a01d0, FUN_1805a3a50; backend.dll FUN_18057bd60, FUN_180581200; accnt.dll FUN_18026ce30, FUN_1802e3e20, FUN_1802e85e0; bp.dll FUN_1800be3e0, FUN_1800c36c0, FUN_18013f810; calc.dll FUN_180126660, FUN_18012b0e0, FUN_1802943a0 — имена однозначны (ExecuteAndClose задачи и WriteAndClose — один GUID).
#[rustfmt::skip]
const OBJECT: &[(&str, &str)] = &[
    ("Activate", "6f959e83-23ec-4991-901d-575d7ea98868"),
    ("ChangeHistory", "174e58ce-82ad-4787-b956-9367937f7971"),
    ("Copy", "68baa1bc-edd1-4d9b-ad80-1d53fb8a7988"),
    ("CreateInitialImage", "d82e191e-f052-40ee-8691-00cac5b34629"),
    ("Delete", "c32d43de-b820-49d0-bf7a-d70829f48f40"),
    ("Execute", "2cacadf7-8fb3-4ec6-ae2b-0ca3fd311c9e"),
    ("ExecuteAndClose", "32df4349-2607-4c2b-a4b9-bca4a1a28bd7"),
    ("FindInList", "212b009b-8b40-486d-b2ee-aec9c65c6aa9"),
    ("Post", "3b8cedbc-8e74-4017-b901-d14b09f32f7a"),
    ("PostAndClose", "87317f86-057f-477e-9045-2da4e4980199"),
    ("ReadChanges", "3328a951-c3c8-4f22-b99e-814f7cea6b82"),
    ("Reread", "1f317795-c420-4a30-b594-c492abc55f7a"),
    ("SetDeletionMark", "827b541d-30c1-4f06-aecf-92aa496a0835"),
    ("ShowInList", "3a17e914-ec6a-4280-b4df-78914f40522b"),
    ("Start", "8d7bcd38-1bbb-4dc1-a9ad-cc9d5966ca8e"),
    ("StartAndClose", "e6a9041f-4d43-4f06-8e17-e95753531565"),
    ("SwitchActivity", "f4613f71-5449-48ed-aea5-de005b272a1d"),
    ("UndoPosting", "389ef1f1-97ce-4326-adf5-886b2dead75c"),
    ("Write", "fe558fde-99b3-45d0-a060-9fc2905309f6"),
    ("WriteAndClose", "32df4349-2607-4c2b-a4b9-bca4a1a28bd7"),
    ("WriteChanges", "8b81add7-25af-4df7-a69c-144e3e3e4c8e"),
];

/// Команды отчета: basic.dll FUN_1808516b0. CancelEdit — в режиме совместимости до 8.3.4 имя Cancel (порог 0x139b0, как у mngbase.dll FUN_1803e09c0).
#[rustfmt::skip]
const REPORT: &[(&str, &str)] = &[
    ("Generate", "b5e6da6b-cec4-450c-876a-6a5f0837f6cc"),
    ("ReportSettings", "0fb774df-ec1c-4e23-9ed1-e089974f74bf"),
    ("NewWindow", "03df6ee5-883c-4cc6-b319-d886d1a9b2c8"),
    ("CancelEdit", "8149a06a-dbf3-4d4d-a275-5385a4196fc7"),
    ("EndEdit", "74c1abd6-b274-4654-baf0-7b8418b792ea"),
    ("SaveReportSettings", "7910bb04-ddcc-4e5d-89f0-104c6ad0f187"),
    ("LoadReportSettings", "b0c9afb6-320c-4e36-be21-8f6d48116415"),
    ("StandardSettings", "c8f1bd8c-b4d1-46d5-97b3-929b5606b6c3"),
    ("ChangeSettingsStructure", "3ea8bf45-5f33-4545-a3bb-29f80666b627"),
    ("ChangeVariant", "fb9d7977-258a-440a-9b59-0a650c86f6a2"),
    ("SaveVariant", "9bffcf73-7b1d-4a8d-bf23-5e051af3ee29"),
    ("LoadVariant", "b08b7a35-583a-4756-b814-0436ff9139c0"),
    ("Print", "a11fe36e-0b45-4c07-80b3-2346b660a51e"),
    ("Preview", "1ba5803f-cfd6-4fd4-b31f-8f98015de04d"),
    ("Save", "a6d73055-3730-42e7-8934-3145ee987141"),
    ("CompactViewMode", "45bea91e-d9e5-4756-b5e6-375ab84b6903"),
    ("ClearChartAppearance", "77fe7401-0b78-44e2-be82-9a5b2b760070"),
];

/// Команды формы динамического списка: GUID → номер mngbase.dll FUN_180613180 (обратно FUN_180612fd0), номер → GUID
/// команды таблицы FUN_180612330, имя — команды таблицы (FUN_18045caa0); номера 33, 34 (ChangeHistory, GetURL) — без
/// команды таблицы в FUN_180612330, имена по сверке БП. Delete — два GUID (см. `form_command`).
#[rustfmt::skip]
const LIST: &[(&str, &str)] = &[
    ("Choose", "8e2b82cf-d1ea-46b2-afdf-a8d64e66ea2b"),
    ("EditList", "a781782f-46c4-49e7-8ad0-16a17e179c58"),
    ("Create", "4f834c38-add1-45e4-a9f3-cefe3efac5c9"),
    ("CreateByParameter", "0ce53bd5-a3c5-43e0-b051-54c835a87be5"),
    ("CreateFolder", "d8772fd1-a3bf-417d-8334-c49968dbb45e"),
    ("Copy", "342c531d-dc73-458a-8ac4-6a746916a33b"),
    ("Change", "6886601d-276c-4d3f-af0a-05c586025608"),
    ("SetDeletionMark", "2e86453d-8958-4c9a-a1b4-b15215eedc2e"),
    ("Delete", "1cc781aa-f32b-4dc7-996a-6c38c3deda5c"),
    ("Delete", "3dd3bd8a-ac1e-44d6-ac83-e7802642a5e2"),
    ("Refresh", "fd8f031f-c168-4e1b-8b0c-15eb3057e688"),
    ("HierarchicalList", "ffc5e8d5-40a7-4893-a590-49bd588f9466"),
    ("List", "a2b927a1-35af-43e3-af73-4af22ac2c0fa"),
    ("Tree", "0b83270d-7f95-4cdd-93c3-342d7991fed5"),
    ("MoveItem", "39c6a2fb-45cc-41b1-853f-967fb68aa1df"),
    ("LevelDown", "aa042316-63ba-4f10-8d39-3935474562d0"),
    ("LevelUp", "e44f9b41-bf53-4837-b4d4-f0ff9cdf0feb"),
    ("SetDateInterval", "eb880cb2-a91f-4ad6-afb7-f0e6d7a1b111"),
    ("Post", "4c569466-1af5-4fc1-9b63-7bf6493097bf"),
    ("UndoPosting", "441362c1-0c86-4f73-bf50-6e1048a2db73"),
    ("FindByCurrentValue", "b520ca45-d8db-4982-b128-bb42a6afd911"),
    ("Find", "bdefa701-6685-453e-a02a-3683d0cc16d3"),
    ("CancelSearch", "96e0bc70-f8ff-4732-8119-060923203629"),
    ("ListSettings", "1c00edb8-a826-4855-9bde-94dbc5f620e5"),
    ("LoadDynamicListSettings", "952c2984-9955-415a-8235-5c710aabe732"),
    ("SaveDynamicListSettings", "d5c3842d-7252-4370-9174-756a6cc553e5"),
    ("DynamicListStandardSettings", "d603a249-6eb3-4e38-bb2d-a8a86a8ab156"),
    ("OutputList", "9758d344-4b1d-4dc9-80bd-81060bc18b2a"),
    ("WriteChanges", "a29c4f3a-3b41-480a-a31e-5f9f73aa3216"),
    ("ReadChanges", "e7ae2a27-60a2-44ae-ab1d-f307d11c85bf"),
    ("CreateInitialImage", "62778a6d-6114-471c-93f7-e1ccd54bd266"),
    ("ShowMultipleSelection", "9fea4ba9-7d33-47d4-a271-cb54df4a9b74"),
    ("ShowRowRearrangement", "2993e218-2504-4c0d-975a-e9e01d11a36d"),
    ("ChangeHistory", "c9abb6b0-eafd-4505-8312-9a7b6888cbf3"),
    ("GetURL", "9885f4b6-d830-435f-a0e3-6b70ffe0f85c"),
];

/// Команды основного реквизита «Компоновщик настроек»: номер → GUID mngbase.dll 0x1803e0980, имена — FUN_1803e09c0
/// (CancelEdit: в режиме совместимости до 8.3.4 — Cancel), Generate — по имени у отчета (basic.dll FUN_1808516b0).
#[rustfmt::skip]
const COMPOSER: &[(&str, &str)] = &[
    ("Generate", "b5e6da6b-cec4-450c-876a-6a5f0837f6cc"),
    ("ChangeSettingsStructure", "3ea8bf45-5f33-4545-a3bb-29f80666b627"),
    ("StandardSettings", "c8f1bd8c-b4d1-46d5-97b3-929b5606b6c3"),
    ("CancelEdit", "8149a06a-dbf3-4d4d-a275-5385a4196fc7"),
    ("EndEdit", "74c1abd6-b274-4654-baf0-7b8418b792ea"),
];

/// Стандартные картинки: GUID или отрицательный номер → имя. Регистрация в реестре картинок (метод +0x68:
/// `CompositeID`, имена en/ru, описание `IDS_PICTURE_*`, `v8res://…zip`); сняты все вызовы в функциях, ссылающихся на
/// строки `IDS_PICTURE_` (и регистрация ecscore.dll). Картинки без такого описания (оформление Appearance*, взаимодействие, внешние источники
/// данных…) — не здесь, их дают выборки (`ui::STD_PICTURES` и др.).
#[rustfmt::skip]
pub(crate) const STD_PICTURES: &[(&str, &str)] = &[
    // core85.dll FUN_1802b1e80
    ("2a0c2238-cb59-4473-ada6-352b60f3c0a9", "AddListItem"), ("977e831a-0e73-4d60-af51-091a6fa8612e", "CreateListItem"),
    ("448d6f55-d885-496c-870d-d1bd78374745", "CloneListItem"),
    ("167a160b-fa48-4337-87ab-7e0fe95c4b5a", "DeleteListItem"), ("08a45a70-c221-4339-b3b1-9f11cb22147d", "Delete"),
    ("da0c4924-973c-4ef0-9dcf-f1fc3307e5e2", "ChangeListItem"), ("97b2cc97-d5c6-45fb-9824-9d6d73db21fe", "Change"),
    ("fafe4c1f-c265-4220-a0e1-8f82af26b72e", "SortList"), ("91022b99-b610-48ad-954e-a297848081ce", "SortListAsc"),
    ("1fa32fdb-a180-418f-a6eb-db7516b7a30b", "SortListDesc"), ("fc4f29e0-d168-4fe0-8e64-e982fabf2595", "Refresh"),
    ("3bdc16c8-6a96-4467-9442-a8e4804b3fa2", "SyncContents"),
    ("b1406535-6cc2-4410-95ea-753556e8460f", "FilterByCurrentValue"),
    ("479470e0-ea0f-4266-8549-e2b1e8c06534", "ClearFilter"), ("3c904ff7-1195-4a7c-9a38-7b1f6ca49cce", "Back"),
    ("f874b0cc-db1d-4577-8c77-d4ba206eb05d", "Forward"), ("e6fc55a0-3d58-4b15-bdd3-717453929598", "WriteAndClose"),
    ("1377931c-5744-4948-bade-cb35117b5f63", "Close"), ("894cf65b-4109-4533-a1d7-c87b1fcc80a3", "Write"),
    ("cb34c423-3d6a-4202-a809-3b3f45fb14ab", "LevelUp"), ("01743054-d102-4e7c-bf15-5ed7fd84441b", "LevelDown"),
    ("ed0bec43-4633-416c-8c08-0384ca444e32", "EndEdit"), ("7168f070-087c-448e-ae3a-6740f424076a", "ChooseFromList"),
    ("2f130057-bb2a-4e22-bba5-e108fac26940", "ChooseValue"), ("f6532868-30b9-44ab-803c-78f0f0b06b02", "CloneObject"),
    ("584b470d-ba34-4b25-9620-8de4066ffeaa", "Previous"), ("9cf611dc-2370-4357-910d-a2b49c7a1ec6", "Next"),
    ("8f29e0e2-d5e6-41e8-a34d-9a0288156322", "Reread"), ("31b93f03-0ba2-4631-a171-0d3a3d2ecc48", "ListSettings"),
    ("f04794cb-c198-4172-86c3-649386013c85", "CustomizeList"),
    ("38a5a658-ec2b-4ddf-8088-c6d2169d3181", "ChooseTopLevel"), ("c2e2d966-5b7f-4699-903b-28a6f50d5471", "OutputList"),
    ("c7cdd3c0-3879-436a-b145-5e2615e9b3e1", "FindInList"), ("9c96aa25-d656-4b3d-ab3e-81d9718da238", "ShowInList"),
    ("e8a49985-fef7-45a9-b6bb-ddd2b9028172", "DataHistory"), ("c283cd1c-3187-451d-8ef2-7df55daeef06", "History"),
    ("ffab30f1-da11-44b5-b34c-24da22badcf4", "Find"), ("05612131-3e11-49c0-9592-07e6d9318ef7", "FindNext"),
    ("e3b38083-0191-4a10-8f5b-51571f2419b4", "FindPrevious"), ("6cb69e7f-fe19-4f64-bfb5-1a4fad6c2ef9", "Replace"),
    ("785362cb-3756-48ed-87d2-292ded17054a", "OpenFile"), ("818ab7d0-4654-4542-bd5e-fd9d1352b5a1", "SaveFile"),
    ("0abdab67-5c90-4296-8168-239d22024d11", "PrintImmediately"), ("b7c81c62-d6ad-4eae-9cea-0e203182db67", "FormHelp"),
    ("b4c7ab2c-bcda-4468-a28f-5fee93838c4e", "Properties"), ("9e808d29-787b-4825-863a-13c6844ce91d", "CancelSearch"),
    ("27ee3053-952c-49e5-8261-9215098e0e9c", "CollapseAll"), ("fb7e9fb5-110b-41cb-adc6-753969ae1c81", "ExpandAll"),
    ("1a4342a5-fa06-4556-8a85-e8738fc25821", "GetURL"), ("3b2a508b-f36e-4e0b-9dc0-70b2b56276a9", "GotoURL"),
    ("1001ae3e-9289-4303-9699-3c0c17e20e61", "AddToFavorites"),
    ("c757209d-a87f-4410-b1a3-76000178f1f0", "ListViewModeList"),
    ("3d4ad3b1-17de-4cf1-a2e4-0c2c83a5b5c2", "ListViewModeHierarchicalList"),
    ("da9ac044-0ff7-4bcf-a441-3187bd1d951f", "ListViewModeTree"),
    ("6511326b-20c3-4bf8-8503-c2c2c9072c6c", "CustomizeForm"), ("4781692b-60eb-47e4-acfb-2caf0f49e977", "Char"),
    ("fc6a06a8-1308-4385-b1b2-9d302d2054ed", "SearchControl"),
    ("6a248caf-0a7b-46ad-a595-74890ea202f7", "Conversations"),
    ("928075d1-b90b-416c-b0b2-c3104cf084aa", "Notifications"),
    ("5289d9a4-b012-4d54-9bce-50473fe29b57", "DialogExclamation"),
    ("8bdf1079-8fad-4d21-ad7f-4b2e4ecdce3d", "DialogInformation"),
    ("ef27ae9e-7040-4374-b93c-0d276de2ea23", "DialogQuestion"), ("83db1f8a-41bd-4016-bdb2-a28e3a8d6dcc", "DialogStop"),
    ("b9cb1339-815d-4e11-84d3-aebfa5a58459", "Zoom"), ("835db646-1531-494b-b7c1-3239b0080bcb", "Parameters"),
    ("4b54770b-d069-4c0e-9b17-5cc2a01134d9", "Information"),
    ("dfcd2d21-24ea-4b27-ab9a-6bf754577536", "FunctionMenuCommand"),
    ("c38cc4cf-111d-4bc8-8dcb-4464e2ddfb25", "Favorites"), ("caf2e58b-ca3d-4b63-82c9-f21f1c9bc9eb", "Setting"),
    ("a722bc14-4edb-4eed-84b9-5d9b2b443e04", "CollaborationSystemUser"),
    ("35bc8caa-f7ce-4158-87da-d9bf785afa39", "DataSearch"), ("31bf709f-3b50-4137-9b51-ebc7fb802a7c", "GotoExternalURL"),
    ("97c5a6d5-47ed-43f9-8c8c-10e9903c23d2", "Calendar"), ("fada8a16-8b14-4151-87a9-775099f37832", "Calculator"),
    ("6ecee038-9722-4d80-bb91-7ee7046ec4c7", "Help"), ("b4ea245b-70ce-4777-a321-c7eb09839643", "OpenFromMainServer"),
    ("3bf7bd14-2232-4f53-b3b3-59a8c683f9b0", "OpenFromStandaloneServer"),
    ("97f87955-b88a-4225-a0d8-03af981ecd86", "ShowPassword"), ("0e2da390-5c04-46a2-a74b-1b7e13a40f2b", "HidePassword"),
    ("-1", "InputFieldSelect"), ("-100", "Select"), ("-2", "InputFieldClear"), ("-200", "Clear"), ("-3", "MoveUp"),
    ("-4", "MoveDown"), ("-5", "InputFieldCalendar"), ("-6", "InputFieldCalculator"), ("-7", "InputFieldOpen"),
    ("-700", "Magnifier"), ("-8", "MoveLeft"), ("-9", "MoveRight"), ("-10", "CheckAll"), ("-11", "UncheckAll"),
    ("-12", "GrayedAll"), ("-13", "Print"), ("-14", "InputFieldChooseType"), ("-1400", "ChooseType"),
    ("-15", "ZoomOut"), ("-16", "ZoomIn"),
    // basic.dll FUN_18030abd0
    ("20ebc47b-f4d9-439c-acd3-fdc624fbac2a", "Post"), ("8ca4ea33-603d-4992-8a41-c7924b5bd40b", "UndoPosting"),
    ("23f940bf-7381-4c2b-85a1-e541ed428042", "SaveValues"), ("a7707ed1-39b0-418f-974d-4d500d27a9c6", "RestoreValues"),
    ("6e3687cf-a8d1-446a-833a-bfaf38516353", "SwitchActivity"),
    ("a6cbfd77-fcf0-40f4-a8de-ee0d3e580fe6", "DataProcessor"), ("db817ee1-fd28-4e7f-bb4a-53686b2b153c", "Report"),
    ("069b8324-7c51-4c73-a6a8-c06d4fc383b5", "Catalog"), ("894afc03-9904-465d-b671-f555ffb9b21c", "Document"),
    ("e51185a4-d915-45b8-b201-1c46cc2d8104", "DocumentJournal"), ("e93f538e-dfaf-4a91-a9b6-c053555bcf60", "Constant"),
    ("2bfafd1e-6d07-4609-8010-6c1c605cefd2", "ChartOfCharacteristicTypes"),
    ("58174855-39be-462e-8723-cb2d95182146", "SetDateInterval"),
    ("4eb938cb-7d69-4f3d-ac17-1f20f1212d49", "FindByNumber"), ("0bac63da-5b4e-48af-b593-7c5d29663e83", "FilterByType"),
    ("55ef0776-5ee4-4daf-9a9b-70d63643ab8d", "SetTime"), ("ea3383df-7098-42f2-ba46-2fc891f799cb", "NewFolder"),
    ("4ab0e87f-7d9b-4aa8-ac4b-680a78522da8", "CreateFolder"), ("38bbcebe-e456-461b-8457-07c9a72344a3", "FindInTree"),
    ("37e91e77-93ce-4c3b-8d30-a9d8cfd3d3b0", "MoveItem"), ("a119150f-6c0c-4a94-97b0-5f08d7ebd6f5", "HierarchicalView"),
    ("683fc04f-e189-49b7-92a9-f835c36f5fb2", "ViewByOwner"), ("80d1a756-13a8-4281-86cc-eaaf84da4cf9", "Enum"),
    ("5b87ad1b-d8cc-43c1-b5c4-dc43613c518c", "InformationRegister"),
    ("70f51581-87b6-41cb-a21b-c9dcdcc7fa93", "AccumulationRegister"),
    ("d971ec77-85e4-4e8d-a20a-8b130eedeb29", "CatalogObject"),
    ("2209b937-6f41-4153-b1ac-ef7e170ad3a7", "DocumentObject"),
    ("45921467-71f2-4176-97e8-7df1d31d9e87", "ChartOfCharacteristicTypesObject"),
    ("4128e6b6-e4ef-48d4-a523-99dc6b613054", "InformationRegisterRecord"),
    ("0ce78048-0196-4f80-a781-9829cdb7f43e", "GenerateReport"),
    ("942e0303-a3ec-4fe8-887c-5aea8516d424", "ReportSettings"), ("eb47324b-85f9-4172-9315-bba8015d9970", "NewWindow"),
    ("b5a0aaba-3a83-4a71-b6f9-24aae1574681", "SaveReportSettings"),
    ("283ecabd-aaed-41d1-ad46-6cca91c29120", "LoadReportSettings"),
    ("549a2c45-4fce-493f-94ee-9a3a4f426551", "ListViewMode"),
    // backend.dll FUN_1801f1500
    ("6cbf8f9a-3d2f-427b-bfce-5e2bc7a8589d", "DeleteListItemDirectly"),
    ("60643198-e4b2-4c39-9de1-53cca3fff382", "DeleteDirectly"),
    ("73af51dd-6cda-48be-a093-5a7161c60c77", "FilterAndSort"),
    ("4bf588d5-b8d8-47fd-8f41-eb9b668981ce", "SetListItemDeletionMark"),
    ("18492a87-2fe4-44af-b218-304897fed020", "MarkToDelete"), ("8729a534-9f88-47b0-8d6b-ec213689580d", "FilterHistory"),
    ("01ec9d9a-7497-4d88-b93f-066c633a4866", "InputOnBasis"), ("544fdbe8-5956-4512-bc62-93b4c022d291", "ExchangePlan"),
    ("1f046bc2-d6c5-46a3-a459-b2c0508f86fb", "QueryWizard"), ("dcd23a32-5c7c-43f2-9021-80d98128556f", "CheckSyntax"),
    ("021c20a0-071b-4a60-8e44-12487adde0c8", "EditInDialog"), ("6ff3ddbd-56e3-4ddf-a5bf-048c1e2dfb2f", "User"),
    ("f16ab927-3ac6-4cdd-bcf1-d8acf005a255", "UserWithoutNecessaryProperties"),
    ("75a40cc4-c719-4c3f-91ea-fc5787bc34ca", "UserWithAuthentication"),
    ("4d2570b5-205f-413c-b4cc-b2097f61684f", "CreateInitialImage"),
    ("f62488ee-f90c-47f7-929d-f42ec11a1e63", "WriteChanges"), ("83c8f18d-8701-41f3-bef4-53f88adbb868", "ReadChanges"),
    ("743cc773-12e6-4ee6-9c57-5f5800bac427", "ExchangePlanObject"),
    ("2ef82795-06fe-4365-bd0c-44b486264620", "FilterCriterion"), ("a594c8a1-7218-420a-860f-7b493c5e65c4", "Sort"),
    ("1970a480-9b38-405e-9d9e-8209f3fad5f1", "ScheduledJob"), ("6206a729-16e5-4e32-b53c-122de4e30c8d", "ScheduledJobs"),
    ("6b909f65-95a4-4697-8ca0-c8f331227b9a", "SettingsStorage"), ("fc34a694-e99b-4d1c-a526-63f5571bdb09", "Form"),
    ("47f01799-7968-4f44-9acc-fe1bdde8beb2", "ActiveUsers"), ("723765ab-0b92-4745-a621-1ba0f77c92c9", "EventLog"),
    ("4fddea39-5129-4b4c-83fe-4e443cd61940", "EventLogByUser"), ("a064544f-6037-48ca-b19f-8ad63e43af23", "ShowData"),
    ("270de5f0-f2df-4845-9fde-30b1ec486217", "QueryWizardShowChangesTables"),
    ("c8a269ff-5b6d-4f42-9fa6-369d7b492aa7", "Rename"),
    ("18bca3d7-a7a5-41df-a180-4dff9c217f43", "QueryWizardCreateNestedQuery"),
    ("7604cff7-5cc6-4f88-8d16-504f01b92a3c", "QueryWizardCreateTempTableDescription"),
    ("a9481ba4-dc85-4112-9c50-f9f340a61298", "QueryWizardReplaceTable"),
    ("fe740df0-d828-4241-a12f-7414e12302e8", "QueryWizardTableParameters"),
    ("7df3febb-2640-41b7-ad8b-7a23b7ad4aec", "QueryWizardCreateTempTableDropQuery"),
    ("0c1f7756-6143-4903-a94c-8f22c85e44de", "Attribute"), ("c78c9f3d-e92c-4f38-bb72-d8bd7fa5dbe3", "Dimension"),
    ("d6eefec0-792a-4720-8933-e2a57f9e312c", "Resource"), ("52b637e5-f95f-4c70-9a72-2a4b5a9df449", "NestedTable"),
    ("6b322d7c-3fa2-437d-b311-fd70c4c7ed46", "QueryWizardTempTableDescription"),
    ("49369551-39dc-4adb-8795-5ab619c6c2bd", "QueryWizardTempTablesGroup"),
    ("64ca52ee-f1a3-468f-8055-311935077515", "QueryWizardTempTable"),
    ("c5501b77-0070-46e3-8ebd-f04a61eb9a73", "QueryWizardNestedQuery"),
    ("f695666a-bad9-49f6-ab7c-5198d7ea4739", "CustomExpression"), ("7562cef7-0e57-4f63-a754-b61128a4f3ae", "GoForward"),
    ("892196a9-c94f-4e50-8224-3c0cea4ea6b8", "GoBack"), ("85cc7dd0-44fc-41aa-967f-f52f202ee2e6", "GoToBegin"),
    ("6ca1af75-2aa5-48a6-820d-efa0ccb8f4f1", "Today"), ("14b24498-e49c-4713-be64-75101d0abfb9", "GoToEnd"),
    ("8da615a0-65a5-4511-a8d4-00dc156478fb", "AppearanceUpTriangleGreen"),
    ("e0ad5543-df7f-423c-8f59-8caf200cca0a", "AppearanceDashYellow"),
    ("7a2a6f5c-7677-45e1-a2d3-ce2444d99a63", "AppearanceDownTriangleRed"),
    ("87d032df-0956-47e9-bead-4e15330f1983", "AppearanceUpArrowGreen"),
    ("0a688f53-2f13-4833-b340-c604cb1d3eb1", "AppearanceUpInclineArrowGreen"),
    ("09547c21-3f15-4a6d-ba87-e5f289dc1a6d", "AppearanceUpInclineArrowYellow"),
    ("697e00c5-4a1d-4367-90e8-ddb734069ad5", "AppearanceRightArrowYellow"),
    ("ce1c3d6b-7825-4cac-a58c-58bac18cefc9", "AppearanceDownInclineArrowYellow"),
    ("0880808c-5146-42b2-87a3-e5a9693ad256", "AppearanceDownArrowRed"),
    ("80a31eac-8df6-49c0-ba84-b53c4631d09f", "AppearanceDownInclineArrowRed"),
    ("a5d108f7-17d6-42dc-a789-45d71fb50ecd", "AppearanceUpArrowGray"),
    ("92e24ce1-3917-4ee4-bbde-adce48b6c96b", "AppearanceUpInclineArrowGray"),
    ("20b82e97-5fcc-4c68-8e0d-d01060847520", "AppearanceRightArrowGray"),
    ("a30ab2ef-6076-457d-9293-44edc7c6767e", "AppearanceDownInclineArrowGray"),
    ("e3b29b1d-4694-4f56-8d55-922f83afed7a", "AppearanceDownArrowGray"),
    ("71cbcb5c-f3f0-4ffd-a4d0-19b802b5ed6b", "AppearanceCircleGreen"),
    ("8d7e5026-9c1c-4542-bec0-2b729c84e139", "AppearanceCircleYellow"),
    ("c1a61df2-f280-49d0-a8b3-7e5fc6f56ff7", "AppearanceCircleRed"),
    ("cf10e497-9779-44a7-82d8-811b88193215", "AppearanceCircleBlack"),
    ("85998f14-805b-4e2b-ba19-9d79b0464042", "AppearanceCheckIcon"),
    ("9ef73565-2250-4a35-9fb3-470bd19ca9ca", "AppearanceCrossIcon"),
    ("7a9cd2fd-6372-4342-9a9e-3ebbd754fd83", "AppearanceCheckBox"),
    ("b2202798-23e0-4165-9982-24878f432488", "AppearanceCross"),
    ("bbd37b6b-2742-48f3-9efa-d7c245f125b0", "AppearanceFlagGreen"),
    ("16e4a14a-f38b-453d-908a-41fb9479c3f6", "AppearanceFlagYellow"),
    ("b0dd988f-2d9f-4364-b1f4-a4d5f45ffb78", "AppearanceFlagRed"),
    ("3689585c-a3e2-45d0-a302-caeb31b78835", "AppearanceStarFilled"),
    ("26abd244-8dd5-417e-aed0-247a573394ca", "AppearanceStarHalfFilled"),
    ("f8d11b3a-a72b-4ee3-ad44-d401472e4b7c", "AppearanceStarEmpty"),
    ("788667db-61c9-45f3-9c4f-5f660ecdf3e1", "AppearanceCircleFilled"),
    ("08a03482-420e-4faa-af9d-e0138c53bfff", "AppearanceCircleThreeFourthFilled"),
    ("dc9b46a9-905b-4baa-9e53-4d824e291a5b", "AppearanceCircleTwoFourthFilled"),
    ("fc058833-e57f-4f93-ba7a-803992a65c3e", "AppearanceCircleOneFourthFilled"),
    ("2721abfb-fbff-4a3a-98ac-b7c9eb29cd85", "AppearanceCircleEmpty"),
    ("ba592483-bc90-4e26-ba4d-2126359c6529", "AppearanceBoxesFilled"),
    ("8c18ce73-0840-4f64-b76c-89d2356b1a41", "AppearanceBoxesThreeFilled"),
    ("f164fa2a-0a20-48df-9816-e2d6cbe0aee9", "AppearanceBoxesTwoFilled"),
    ("7026a8e6-f973-4fab-88f3-1047734d787f", "AppearanceBoxesOneFilled"),
    ("879ca050-8650-4135-b799-f3ef9e00a65a", "AppearanceBoxesEmpty"),
    ("34c981c1-f0bd-43e0-a0cf-c2a9cd648b91", "SortedDesc"), ("e4c49dfb-ef8e-4f4a-9d01-9536ba61f944", "SortedAsc"),
    // accnt.dll FUN_180122ce0
    ("ccb3d8f7-6da2-4c65-aba6-17b2ffbba78c", "ChartOfAccounts"),
    ("20175a6d-7510-46a8-9077-75a5d0d97802", "AccountingRegister"), ("196622f7-0941-435b-992b-722f3082adf4", "Debit"),
    ("55bc1099-a7df-4d0a-b332-a45f0473b368", "Credit"), ("ed067d76-b144-4d00-bb36-d1833dd1350c", "DebitCredit"),
    ("6d4277ee-b29b-4790-8c3c-c4f868311c96", "ChartOfAccountsObject"),
    // bp.dll FUN_18022e5d0
    ("509c4a7f-6406-4388-bb8c-bc81fb5131aa", "BusinessProcess"), ("37cf7cc0-abad-4385-b597-6fd2d8dc085a", "Task"),
    ("9fecbaff-2a05-4da6-9ef1-807e754b928d", "BusinessProcessStart"),
    ("003024ed-fa25-42ac-9f53-f5014e383801", "ExecuteTask"), ("093dd4ed-e03c-4fc6-a95a-01f51379cccf", "ActivateTask"),
    ("a24cff7f-a1a5-4403-af82-a7b31852cde9", "BusinessProcessObject"),
    ("b9e2dae2-b205-4442-aee5-802cd82f5199", "TaskObject"), ("e96de06b-fa83-48cf-b033-190a249855c9", "GraphicalSchema"),
    // calc.dll FUN_180105f50
    ("80837aab-ad67-4811-9d83-88344439c721", "CalculationType"),
    ("f3b8f300-5a54-4eea-8136-5798413a479c", "CalculationRegister"),
    ("b58ea40e-819b-4d07-bd9f-5fbf42e01f55", "ChartOfCalculationTypes"),
    ("64400154-ca4c-4a05-8028-83434fda5bef", "ChartOfCalculationTypesObject"),
    // dcscore.dll FUN_180242040
    ("6c2759b1-2b63-4fa5-8d13-75786c7e1e89", "DataCompositionNewTable"),
    ("8ac19694-383a-457a-b050-0a3ee937f5f3", "DataCompositionNewNestedScheme"),
    ("732f9dd3-5baf-47ff-af7b-edfe16dac2a1", "DataCompositionNewChart"),
    ("77180b5e-8faa-4712-a788-e9f8903e3419", "DataCompositionNewGroup"),
    ("affb1617-24bc-4170-9c84-0902cc3ef206", "DataCompositionSettingsWizard"),
    ("251aaa98-0127-44c3-a163-6f5ab4367ee2", "DataCompositionStandardSettings"),
    ("d90a7482-9a1d-4d3d-ae96-6db440214d96", "DataCompositionFilter"),
    ("efda7350-6cd7-4416-b188-f5ca9baf66c2", "DataCompositionOrder"),
    ("2c732bfa-f734-48bc-a18b-7554db8a3888", "DataCompositionSelection"),
    ("984b0a3e-daa2-4a9e-b75c-1f230a6e592a", "DataCompositionConditionalAppearance"),
    ("ee7c4a5b-2d9b-4087-ae3e-947792085f09", "DataCompositionOutputParameters"),
    ("ad8cb448-a6bb-43b4-886a-7d6a8367eef2", "DataCompositionGroupFields"),
    ("a075c3ef-bc4e-4c96-bdad-2245ec09c28e", "DataCompositionDataParameters"),
    ("b68eb29c-2372-46e1-b84e-13843899ccf6", "DataCompositionUserFields"),
    ("a0b879df-db58-4cba-b76e-9bbc4391f3d9", "DataCompositionSelectionDisabled"),
    ("d35bd799-1cc3-44d1-8ae3-09755a09d44b", "DataCompositionFilterDisabled"),
    ("c8923bc1-6ea9-456b-829f-3f979dc98420", "DataCompositionOrderDisabled"),
    ("afbaed82-4b8e-4970-8ac6-97a3bff796d3", "DataCompositionGroupFieldsDisabled"),
    ("5ff5bf50-1f3c-46b6-b4ef-a4973dd610aa", "DataCompositionConditionalAppearanceDisabled"),
    ("1522a72e-11d1-41e3-ba0e-e0780c4ba543", "DataCompositionOutputParametersDisabled"),
    // moxel.dll FUN_1802ef890
    ("f3c1376a-d2ee-46c4-9e44-aa2f7dae31c4", "Chart"), ("3646acb9-a91e-4163-bbe1-2006041cd65d", "SpreadsheetShowGrid"),
    ("46598f81-5f95-4485-9b33-bfe4fd1276d0", "SpreadsheetShowHeaders"),
    ("2846af8d-af84-47e3-82b9-01b01f960426", "SpreadsheetReadOnly"),
    ("5182f57f-e834-4d11-9c9f-4aedc002b6e9", "FixTable"),
    ("03665ff1-3a05-41d1-96d3-04bda2d8ede3", "SpreadsheetInsertComment"),
    ("7c75b1df-1fdc-471f-aae6-6b7870318cd4", "SpreadsheetDeleteComment"),
    ("26518e18-e364-475a-8026-e41134658b2a", "SpreadsheetInsertPageBreak"),
    ("aa96f4bb-cf28-4dad-bc42-5ed53de95c0c", "SpreadsheetDeletePageBreak"),
    ("bc61d1e7-2d40-4e93-8630-cce840bdcf99", "SpreadsheetShowComments"),
    ("702a9e16-0bb6-4efb-af11-10faf1e6ee87", "SpreadsheetShowGroups"),
    ("fa67cb81-8d56-4534-90bd-b62fb0dbf5f0", "GanttChart"), ("79bf535f-2256-4ebb-bb8d-c26f3dd1a3de", "PivotChart"),
    ("4bf9fbb5-53c5-4b09-bab2-d69bbfab945b", "Dendrogram"),
    ("a9152be7-62cf-4523-be34-a23f018f497e", "GeographicalSchema"), ("64837726-d2a2-4682-a788-737423e80013", "Picture"),
    ("a636287d-8d6f-4ef3-94bb-a8f986ab52bb", "FormattedString"),
    // html.dll FUN_180009dd0
    ("1cd7b762-ec6a-4e92-ac9a-1832be228ec3", "Stop"),
    // ecscore.dll FUN_18006d6e0 (описания IDS_<ИМЯ>_PICTURE_DESCR, не IDS_PICTURE_*)
    ("9c7f3c8c-a4b4-4a0a-84c8-d60d9feeca16", "NewConversation"), ("f6e88116-03d8-4400-9d88-791895d7031a", "Attach"),
    ("d66b6f73-53b8-49b9-8efc-33c54aa06e3f", "Notify"), ("517ed1af-55eb-42da-a1d4-a9960ef7c1f3", "DontNotify"),
    ("a55455d6-be06-4da3-9a4f-62fbefd1aded", "DontDisturb"), ("be23a908-fe1b-44df-be94-d0f6e8353abe", "SendMessage"),
    ("78da2c47-172f-4d57-ab52-a06e40548136", "Message"), ("2a7e58e2-a6c5-4387-a459-5249c441947a", "GroupConversation"),
    ("05b8ccd1-f644-4e2e-87de-9683410c7038", "CollaborationSystemIntegrationUser"),
    ("e7f605f9-53d4-4e07-b56a-4715dc3a3e50", "StartVideoconference"),
    ("d08abfb8-44b6-4b0f-bad2-e235c6a718c3", "MessageHistory"),
    ("66c18df5-d7a6-465e-92b5-bcc6c1032504", "CollaborationSystemExternalUser"),
];

/// Стандартная картинка по GUID или отрицательному номеру (`"-N"`).
pub(crate) fn std_picture(id: &str) -> Option<&'static str> {
    STD_PICTURES.iter().find(|(g, _)| *g == id).map(|(_, n)| *n)
}

/// GUID или отрицательный номер стандартной картинки по имени.
pub(crate) fn std_picture_id(name: &str) -> Option<&'static str> {
    STD_PICTURES.iter().find(|(_, n)| *n == name).map(|(g, _)| *g)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Выборка сверки команд таблиц (`table::table_command`) — подмножество команд таблицы платформы.
    #[test]
    fn samples_are_subsets() {
        for (n, _) in TABLE {
            for (dl, sc) in [(false, false), (true, false), (false, true)] {
                if let Some(g) = super::super::table::table_command(n, dl, sc) {
                    assert!(TABLE.contains(&(n, g)), "{n}: {g}");
                }
            }
        }
        assert_eq!(form_command("DynamicList:InformationRegister", "Delete"), Some("1cc781aa-f32b-4dc7-996a-6c38c3deda5c"));
        assert_eq!(form_command("DynamicList:Catalog", "ChangeHistory"), Some("c9abb6b0-eafd-4505-8312-9a7b6888cbf3"));
        assert_eq!(form_command("ReportObject", "ChangeSettingsStructure"), Some("3ea8bf45-5f33-4545-a3bb-29f80666b627"));
        assert_eq!(form_command("DataProcessorObject", "OpenFromMainServer"), Some("573e81b7-57eb-45f0-ba4d-ada7c2537a2d"));
        assert_eq!(form_command("DataProcessorObject", "Write"), None);
        assert_eq!(table_command("Delete"), None);
        assert_eq!((DOCUMENT[DOC_PLANNER - 1].0, DOCUMENT[DOC_PLANNER].0, DOCUMENT[DOC_PDF].0), ("SaveAs", "Today", "ScaleUp"));
        assert_eq!(field_command("FormattedDocumentField", "Preview"), Some("4ca32834-6f9f-4dfb-89ce-6db36931c89b"));
        assert_eq!(field_command("PDFDocumentField", "Print"), Some("67916de8-b666-4d6e-b899-29433f416ac8"));
    }

    /// Выборки стандартных картинок согласны с кодом платформы (имена в таблице однозначны).
    #[test]
    fn pictures_agree_with_samples() {
        for (i, (g, n)) in STD_PICTURES.iter().enumerate() {
            assert!(STD_PICTURES[i + 1..].iter().all(|(_, m)| m != n), "{n}");
            for (sg, sn) in crate::xml::ui::STD_PICTURES.iter().chain(crate::xml::form::FORM_STD_PICTURES) {
                assert!(sn != n || sg == g, "{n}: {sg} / {g}");
            }
        }
    }
}
