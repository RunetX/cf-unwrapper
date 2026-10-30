//! Исключенные команды формы `<CommandSet>` → `{N,<guid>…}` (по возрастанию GUID). GUID зависит от класса основного
//! реквизита (у динамического списка — и от класса основной таблицы); сняты со сверки БП (2 942 формы): имя → GUID
//! исключением по формам; имена, которые встречаются только вместе (Abort, Ignore, Retry…), — группой: вместе их GUID
//! известны, порознь не нужны. Группа не целиком, незнакомое имя или класс — отказ.

use super::*;

/// (класс, имена, GUID): имена группы исключаются только вместе.
#[rustfmt::skip]
const T: &[(&str, &[&str], &[&str])] = &[
    ("", &["Abort", "Ignore", "Retry"], &["3f01ed62-97f8-465b-b4f7-6517ac2bc994", "5174ad3f-0569-42fd-8adf-011d8206db6c", "d7e9e72c-8fa7-430c-a3e9-aeadfd57dfc7"]),
    ("", &["Cancel"], &["679b62d9-ff72-4329-bf3a-c0c32b311dd2"]),
    ("", &["Close"], &["3772996b-41f4-4c47-a5a8-ea397db424ae"]),
    ("", &["CustomizeForm"], &["198ea630-fda2-4cda-8a23-f999f4c67ee6"]),
    ("", &["Help"], &["39bb0fe9-771d-4dd5-8a6e-2d16984523af"]),
    ("", &["No"], &["06ee6a21-061e-47f8-81c5-92ae8b8f3b5d"]),
    ("", &["OK"], &["f3613d5c-20c6-46e5-b4d5-7d712ece1296"]),
    ("", &["OpenFromMainServer", "OpenFromStandaloneServer"], &["0ea1a92b-3477-44dd-b152-ea7d411f1c5d", "573e81b7-57eb-45f0-ba4d-ada7c2537a2d"]),
    ("", &["RestoreValues"], &["71e0226e-ebb2-4e33-8745-0a94a01bbf15"]),
    ("", &["SaveValues"], &["239f0103-8de9-4fdf-b485-eb5531da7e51"]),
    ("", &["Yes"], &["5d41082e-9619-42ec-b96f-98b082b3a2f0"]),
    ("BusinessProcessObject", &["Activate"], &["6f959e83-23ec-4991-901d-575d7ea98868"]),
    ("BusinessProcessObject", &["Copy", "Start", "StartAndClose"], &["68baa1bc-edd1-4d9b-ad80-1d53fb8a7988", "8d7bcd38-1bbb-4dc1-a9ad-cc9d5966ca8e", "e6a9041f-4d43-4f06-8e17-e95753531565"]),
    ("CatalogObject", &["Abort", "Ignore", "No", "Retry"], &["06ee6a21-061e-47f8-81c5-92ae8b8f3b5d", "3f01ed62-97f8-465b-b4f7-6517ac2bc994", "5174ad3f-0569-42fd-8adf-011d8206db6c", "d7e9e72c-8fa7-430c-a3e9-aeadfd57dfc7"]),
    ("CatalogObject", &["Cancel"], &["679b62d9-ff72-4329-bf3a-c0c32b311dd2"]),
    // как у документа и записи регистра сведений (команда объекта)
    ("CatalogObject", &["ChangeHistory"], &["174e58ce-82ad-4787-b956-9367937f7971"]),
    ("CatalogObject", &["Close"], &["3772996b-41f4-4c47-a5a8-ea397db424ae"]),
    ("CatalogObject", &["Copy"], &["68baa1bc-edd1-4d9b-ad80-1d53fb8a7988"]),
    ("CatalogObject", &["CustomizeForm"], &["198ea630-fda2-4cda-8a23-f999f4c67ee6"]),
    ("CatalogObject", &["Delete"], &["c32d43de-b820-49d0-bf7a-d70829f48f40"]),
    ("CatalogObject", &["Help"], &["39bb0fe9-771d-4dd5-8a6e-2d16984523af"]),
    ("CatalogObject", &["OK"], &["f3613d5c-20c6-46e5-b4d5-7d712ece1296"]),
    ("CatalogObject", &["Reread"], &["1f317795-c420-4a30-b594-c492abc55f7a"]),
    ("CatalogObject", &["RestoreValues"], &["71e0226e-ebb2-4e33-8745-0a94a01bbf15"]),
    ("CatalogObject", &["SaveValues"], &["239f0103-8de9-4fdf-b485-eb5531da7e51"]),
    ("CatalogObject", &["SetDeletionMark"], &["827b541d-30c1-4f06-aecf-92aa496a0835"]),
    ("CatalogObject", &["ShowInList"], &["3a17e914-ec6a-4280-b4df-78914f40522b"]),
    ("CatalogObject", &["Write"], &["fe558fde-99b3-45d0-a060-9fc2905309f6"]),
    ("CatalogObject", &["WriteAndClose"], &["32df4349-2607-4c2b-a4b9-bca4a1a28bd7"]),
    ("CatalogObject", &["Yes"], &["5d41082e-9619-42ec-b96f-98b082b3a2f0"]),
    ("ChartOfCharacteristicTypesObject", &["SetDeletionMark"], &["827b541d-30c1-4f06-aecf-92aa496a0835"]),
    ("ConstantsSet", &["Abort", "Ignore", "No", "Retry", "Write"], &["06ee6a21-061e-47f8-81c5-92ae8b8f3b5d", "3f01ed62-97f8-465b-b4f7-6517ac2bc994", "5174ad3f-0569-42fd-8adf-011d8206db6c", "d7e9e72c-8fa7-430c-a3e9-aeadfd57dfc7", "fe558fde-99b3-45d0-a060-9fc2905309f6"]),
    ("ConstantsSet", &["Cancel"], &["679b62d9-ff72-4329-bf3a-c0c32b311dd2"]),
    ("ConstantsSet", &["Close"], &["3772996b-41f4-4c47-a5a8-ea397db424ae"]),
    ("ConstantsSet", &["OK"], &["f3613d5c-20c6-46e5-b4d5-7d712ece1296"]),
    ("ConstantsSet", &["RestoreValues"], &["71e0226e-ebb2-4e33-8745-0a94a01bbf15"]),
    ("ConstantsSet", &["SaveValues"], &["239f0103-8de9-4fdf-b485-eb5531da7e51"]),
    ("ConstantsSet", &["WriteAndClose"], &["32df4349-2607-4c2b-a4b9-bca4a1a28bd7"]),
    ("ConstantsSet", &["Yes"], &["5d41082e-9619-42ec-b96f-98b082b3a2f0"]),
    ("DataProcessorObject", &["Abort", "No", "Retry"], &["06ee6a21-061e-47f8-81c5-92ae8b8f3b5d", "3f01ed62-97f8-465b-b4f7-6517ac2bc994", "5174ad3f-0569-42fd-8adf-011d8206db6c"]),
    ("DataProcessorObject", &["Cancel"], &["679b62d9-ff72-4329-bf3a-c0c32b311dd2"]),
    ("DataProcessorObject", &["Close"], &["3772996b-41f4-4c47-a5a8-ea397db424ae"]),
    ("DataProcessorObject", &["CustomizeForm"], &["198ea630-fda2-4cda-8a23-f999f4c67ee6"]),
    ("DataProcessorObject", &["Help"], &["39bb0fe9-771d-4dd5-8a6e-2d16984523af"]),
    ("DataProcessorObject", &["OK"], &["f3613d5c-20c6-46e5-b4d5-7d712ece1296"]),
    ("DataProcessorObject", &["RestoreValues"], &["71e0226e-ebb2-4e33-8745-0a94a01bbf15"]),
    ("DataProcessorObject", &["SaveValues"], &["239f0103-8de9-4fdf-b485-eb5531da7e51"]),
    ("DataProcessorObject", &["Yes"], &["5d41082e-9619-42ec-b96f-98b082b3a2f0"]),
    ("DocumentObject", &["Abort", "Ignore", "No", "Retry"], &["06ee6a21-061e-47f8-81c5-92ae8b8f3b5d", "3f01ed62-97f8-465b-b4f7-6517ac2bc994", "5174ad3f-0569-42fd-8adf-011d8206db6c", "d7e9e72c-8fa7-430c-a3e9-aeadfd57dfc7"]),
    ("DocumentObject", &["Cancel"], &["679b62d9-ff72-4329-bf3a-c0c32b311dd2"]),
    ("DocumentObject", &["ChangeHistory"], &["174e58ce-82ad-4787-b956-9367937f7971"]),
    ("DocumentObject", &["Close"], &["3772996b-41f4-4c47-a5a8-ea397db424ae"]),
    ("DocumentObject", &["Copy"], &["68baa1bc-edd1-4d9b-ad80-1d53fb8a7988"]),
    ("DocumentObject", &["Delete"], &["c32d43de-b820-49d0-bf7a-d70829f48f40"]),
    ("DocumentObject", &["Help"], &["39bb0fe9-771d-4dd5-8a6e-2d16984523af"]),
    ("DocumentObject", &["OK"], &["f3613d5c-20c6-46e5-b4d5-7d712ece1296"]),
    ("DocumentObject", &["Post"], &["3b8cedbc-8e74-4017-b901-d14b09f32f7a"]),
    ("DocumentObject", &["PostAndClose"], &["87317f86-057f-477e-9045-2da4e4980199"]),
    ("DocumentObject", &["Reread"], &["1f317795-c420-4a30-b594-c492abc55f7a"]),
    ("DocumentObject", &["RestoreValues"], &["71e0226e-ebb2-4e33-8745-0a94a01bbf15"]),
    ("DocumentObject", &["SaveValues"], &["239f0103-8de9-4fdf-b485-eb5531da7e51"]),
    ("DocumentObject", &["SetDeletionMark"], &["827b541d-30c1-4f06-aecf-92aa496a0835"]),
    ("DocumentObject", &["ShowInList"], &["3a17e914-ec6a-4280-b4df-78914f40522b"]),
    ("DocumentObject", &["UndoPosting"], &["389ef1f1-97ce-4326-adf5-886b2dead75c"]),
    ("DocumentObject", &["Write"], &["fe558fde-99b3-45d0-a060-9fc2905309f6"]),
    ("DocumentObject", &["WriteAndClose"], &["32df4349-2607-4c2b-a4b9-bca4a1a28bd7"]),
    ("DocumentObject", &["Yes"], &["5d41082e-9619-42ec-b96f-98b082b3a2f0"]),
    ("DynamicList:", &["Abort", "DynamicListStandardSettings", "Ignore", "ListSettings", "LoadDynamicListSettings", "No", "Retry", "SaveDynamicListSettings"], &["06ee6a21-061e-47f8-81c5-92ae8b8f3b5d", "1c00edb8-a826-4855-9bde-94dbc5f620e5", "3f01ed62-97f8-465b-b4f7-6517ac2bc994", "5174ad3f-0569-42fd-8adf-011d8206db6c", "952c2984-9955-415a-8235-5c710aabe732", "d5c3842d-7252-4370-9174-756a6cc553e5", "d603a249-6eb3-4e38-bb2d-a8a86a8ab156", "d7e9e72c-8fa7-430c-a3e9-aeadfd57dfc7"]),
    ("DynamicList:", &["Cancel"], &["679b62d9-ff72-4329-bf3a-c0c32b311dd2"]),
    ("DynamicList:", &["CancelSearch"], &["96e0bc70-f8ff-4732-8119-060923203629"]),
    ("DynamicList:", &["Close"], &["3772996b-41f4-4c47-a5a8-ea397db424ae"]),
    ("DynamicList:", &["Find"], &["bdefa701-6685-453e-a02a-3683d0cc16d3"]),
    ("DynamicList:", &["OK"], &["f3613d5c-20c6-46e5-b4d5-7d712ece1296"]),
    ("DynamicList:", &["Refresh"], &["fd8f031f-c168-4e1b-8b0c-15eb3057e688"]),
    ("DynamicList:", &["RestoreValues"], &["71e0226e-ebb2-4e33-8745-0a94a01bbf15"]),
    ("DynamicList:", &["SaveValues"], &["239f0103-8de9-4fdf-b485-eb5531da7e51"]),
    ("DynamicList:", &["Yes"], &["5d41082e-9619-42ec-b96f-98b082b3a2f0"]),
    ("DynamicList:AccumulationRegister", &["CancelSearch"], &["96e0bc70-f8ff-4732-8119-060923203629"]),
    ("DynamicList:BusinessProcess", &["Copy", "Create"], &["342c531d-dc73-458a-8ac4-6a746916a33b", "4f834c38-add1-45e4-a9f3-cefe3efac5c9"]),
    ("DynamicList:Catalog", &["Abort", "Ignore", "No", "Retry"], &["06ee6a21-061e-47f8-81c5-92ae8b8f3b5d", "3f01ed62-97f8-465b-b4f7-6517ac2bc994", "5174ad3f-0569-42fd-8adf-011d8206db6c", "d7e9e72c-8fa7-430c-a3e9-aeadfd57dfc7"]),
    ("DynamicList:Catalog", &["Cancel"], &["679b62d9-ff72-4329-bf3a-c0c32b311dd2"]),
    ("DynamicList:Catalog", &["CancelSearch"], &["96e0bc70-f8ff-4732-8119-060923203629"]),
    ("DynamicList:Catalog", &["Change"], &["6886601d-276c-4d3f-af0a-05c586025608"]),
    ("DynamicList:Catalog", &["Choose"], &["8e2b82cf-d1ea-46b2-afdf-a8d64e66ea2b"]),
    ("DynamicList:Catalog", &["Close"], &["3772996b-41f4-4c47-a5a8-ea397db424ae"]),
    ("DynamicList:Catalog", &["Copy"], &["342c531d-dc73-458a-8ac4-6a746916a33b"]),
    ("DynamicList:Catalog", &["Create"], &["4f834c38-add1-45e4-a9f3-cefe3efac5c9"]),
    ("DynamicList:Catalog", &["CreateFolder"], &["d8772fd1-a3bf-417d-8334-c49968dbb45e"]),
    ("DynamicList:Catalog", &["CustomizeForm"], &["198ea630-fda2-4cda-8a23-f999f4c67ee6"]),
    ("DynamicList:Catalog", &["Delete"], &["3dd3bd8a-ac1e-44d6-ac83-e7802642a5e2"]),
    ("DynamicList:Catalog", &["Find"], &["bdefa701-6685-453e-a02a-3683d0cc16d3"]),
    ("DynamicList:Catalog", &["FindByCurrentValue"], &["b520ca45-d8db-4982-b128-bb42a6afd911"]),
    ("DynamicList:Catalog", &["Help"], &["39bb0fe9-771d-4dd5-8a6e-2d16984523af"]),
    ("DynamicList:Catalog", &["HierarchicalList", "Tree"], &["0b83270d-7f95-4cdd-93c3-342d7991fed5", "ffc5e8d5-40a7-4893-a590-49bd588f9466"]),
    ("DynamicList:Catalog", &["LevelDown", "LevelUp"], &["aa042316-63ba-4f10-8d39-3935474562d0", "e44f9b41-bf53-4837-b4d4-f0ff9cdf0feb"]),
    ("DynamicList:Catalog", &["List"], &["a2b927a1-35af-43e3-af73-4af22ac2c0fa"]),
    ("DynamicList:Catalog", &["ListSettings"], &["1c00edb8-a826-4855-9bde-94dbc5f620e5"]),
    ("DynamicList:Catalog", &["MoveItem"], &["39c6a2fb-45cc-41b1-853f-967fb68aa1df"]),
    ("DynamicList:Catalog", &["OK"], &["f3613d5c-20c6-46e5-b4d5-7d712ece1296"]),
    ("DynamicList:Catalog", &["OutputList"], &["9758d344-4b1d-4dc9-80bd-81060bc18b2a"]),
    ("DynamicList:Catalog", &["Refresh"], &["fd8f031f-c168-4e1b-8b0c-15eb3057e688"]),
    ("DynamicList:Catalog", &["RestoreValues"], &["71e0226e-ebb2-4e33-8745-0a94a01bbf15"]),
    ("DynamicList:Catalog", &["SaveValues"], &["239f0103-8de9-4fdf-b485-eb5531da7e51"]),
    ("DynamicList:Catalog", &["SetDeletionMark"], &["2e86453d-8958-4c9a-a1b4-b15215eedc2e"]),
    ("DynamicList:Catalog", &["Yes"], &["5d41082e-9619-42ec-b96f-98b082b3a2f0"]),
    ("DynamicList:ChartOfAccounts", &["MoveItem"], &["39c6a2fb-45cc-41b1-853f-967fb68aa1df"]),
    ("DynamicList:ChartOfCharacteristicTypes", &["Copy", "Delete", "SetDeletionMark"], &["2e86453d-8958-4c9a-a1b4-b15215eedc2e", "342c531d-dc73-458a-8ac4-6a746916a33b", "3dd3bd8a-ac1e-44d6-ac83-e7802642a5e2"]),
    ("DynamicList:Document", &["Abort", "Ignore", "No", "Retry"], &["06ee6a21-061e-47f8-81c5-92ae8b8f3b5d", "3f01ed62-97f8-465b-b4f7-6517ac2bc994", "5174ad3f-0569-42fd-8adf-011d8206db6c", "d7e9e72c-8fa7-430c-a3e9-aeadfd57dfc7"]),
    ("DynamicList:Document", &["Cancel"], &["679b62d9-ff72-4329-bf3a-c0c32b311dd2"]),
    ("DynamicList:Document", &["CancelSearch", "Find"], &["96e0bc70-f8ff-4732-8119-060923203629", "bdefa701-6685-453e-a02a-3683d0cc16d3"]),
    ("DynamicList:Document", &["Change"], &["6886601d-276c-4d3f-af0a-05c586025608"]),
    ("DynamicList:Document", &["Close"], &["3772996b-41f4-4c47-a5a8-ea397db424ae"]),
    ("DynamicList:Document", &["Copy"], &["342c531d-dc73-458a-8ac4-6a746916a33b"]),
    ("DynamicList:Document", &["Create"], &["4f834c38-add1-45e4-a9f3-cefe3efac5c9"]),
    ("DynamicList:Document", &["CustomizeForm"], &["198ea630-fda2-4cda-8a23-f999f4c67ee6"]),
    ("DynamicList:Document", &["Delete"], &["3dd3bd8a-ac1e-44d6-ac83-e7802642a5e2"]),
    ("DynamicList:Document", &["DynamicListStandardSettings", "LoadDynamicListSettings", "SaveDynamicListSettings"], &["952c2984-9955-415a-8235-5c710aabe732", "d5c3842d-7252-4370-9174-756a6cc553e5", "d603a249-6eb3-4e38-bb2d-a8a86a8ab156"]),
    ("DynamicList:Document", &["Help"], &["39bb0fe9-771d-4dd5-8a6e-2d16984523af"]),
    ("DynamicList:Document", &["ListSettings"], &["1c00edb8-a826-4855-9bde-94dbc5f620e5"]),
    ("DynamicList:Document", &["OK"], &["f3613d5c-20c6-46e5-b4d5-7d712ece1296"]),
    ("DynamicList:Document", &["OutputList"], &["9758d344-4b1d-4dc9-80bd-81060bc18b2a"]),
    ("DynamicList:Document", &["Post", "UndoPosting"], &["441362c1-0c86-4f73-bf50-6e1048a2db73", "4c569466-1af5-4fc1-9b63-7bf6493097bf"]),
    ("DynamicList:Document", &["Refresh"], &["fd8f031f-c168-4e1b-8b0c-15eb3057e688"]),
    ("DynamicList:Document", &["RestoreValues"], &["71e0226e-ebb2-4e33-8745-0a94a01bbf15"]),
    ("DynamicList:Document", &["SaveValues"], &["239f0103-8de9-4fdf-b485-eb5531da7e51"]),
    ("DynamicList:Document", &["SetDeletionMark"], &["2e86453d-8958-4c9a-a1b4-b15215eedc2e"]),
    ("DynamicList:Document", &["Yes"], &["5d41082e-9619-42ec-b96f-98b082b3a2f0"]),
    ("DynamicList:DocumentJournal", &["Copy"], &["342c531d-dc73-458a-8ac4-6a746916a33b"]),
    ("DynamicList:DocumentJournal", &["Create"], &["4f834c38-add1-45e4-a9f3-cefe3efac5c9"]),
    ("DynamicList:DocumentJournal", &["CreateByParameter"], &["0ce53bd5-a3c5-43e0-b051-54c835a87be5"]),
    ("DynamicList:DocumentJournal", &["Delete", "SetDeletionMark"], &["2e86453d-8958-4c9a-a1b4-b15215eedc2e", "3dd3bd8a-ac1e-44d6-ac83-e7802642a5e2"]),
    ("DynamicList:DocumentJournal", &["Refresh"], &["fd8f031f-c168-4e1b-8b0c-15eb3057e688"]),
    ("DynamicList:InformationRegister", &["Abort", "Ignore", "No", "Retry"], &["06ee6a21-061e-47f8-81c5-92ae8b8f3b5d", "3f01ed62-97f8-465b-b4f7-6517ac2bc994", "5174ad3f-0569-42fd-8adf-011d8206db6c", "d7e9e72c-8fa7-430c-a3e9-aeadfd57dfc7"]),
    ("DynamicList:InformationRegister", &["Cancel"], &["679b62d9-ff72-4329-bf3a-c0c32b311dd2"]),
    ("DynamicList:InformationRegister", &["CancelSearch", "Find"], &["96e0bc70-f8ff-4732-8119-060923203629", "bdefa701-6685-453e-a02a-3683d0cc16d3"]),
    ("DynamicList:InformationRegister", &["Change"], &["6886601d-276c-4d3f-af0a-05c586025608"]),
    ("DynamicList:InformationRegister", &["ChangeHistory"], &["c9abb6b0-eafd-4505-8312-9a7b6888cbf3"]),
    ("DynamicList:InformationRegister", &["Close"], &["3772996b-41f4-4c47-a5a8-ea397db424ae"]),
    ("DynamicList:InformationRegister", &["Copy"], &["342c531d-dc73-458a-8ac4-6a746916a33b"]),
    ("DynamicList:InformationRegister", &["Create"], &["4f834c38-add1-45e4-a9f3-cefe3efac5c9"]),
    ("DynamicList:InformationRegister", &["CustomizeForm"], &["198ea630-fda2-4cda-8a23-f999f4c67ee6"]),
    ("DynamicList:InformationRegister", &["Delete"], &["1cc781aa-f32b-4dc7-996a-6c38c3deda5c"]),
    ("DynamicList:InformationRegister", &["Help"], &["39bb0fe9-771d-4dd5-8a6e-2d16984523af"]),
    ("DynamicList:InformationRegister", &["OK"], &["f3613d5c-20c6-46e5-b4d5-7d712ece1296"]),
    ("DynamicList:InformationRegister", &["Refresh"], &["fd8f031f-c168-4e1b-8b0c-15eb3057e688"]),
    ("DynamicList:InformationRegister", &["RestoreValues"], &["71e0226e-ebb2-4e33-8745-0a94a01bbf15"]),
    ("DynamicList:InformationRegister", &["SaveValues"], &["239f0103-8de9-4fdf-b485-eb5531da7e51"]),
    ("DynamicList:InformationRegister", &["ShowMultipleSelection"], &["9fea4ba9-7d33-47d4-a271-cb54df4a9b74"]),
    ("DynamicList:InformationRegister", &["Yes"], &["5d41082e-9619-42ec-b96f-98b082b3a2f0"]),
    ("DynamicList:Task", &["Delete", "SetDeletionMark"], &["2e86453d-8958-4c9a-a1b4-b15215eedc2e", "3dd3bd8a-ac1e-44d6-ac83-e7802642a5e2"]),
    ("ExchangePlanObject", &["CreateInitialImage", "ReadChanges", "WriteChanges"], &["3328a951-c3c8-4f22-b99e-814f7cea6b82", "8b81add7-25af-4df7-a69c-144e3e3e4c8e", "d82e191e-f052-40ee-8691-00cac5b34629"]),
    ("InformationRegisterRecordManager", &["Abort", "No", "Retry"], &["06ee6a21-061e-47f8-81c5-92ae8b8f3b5d", "3f01ed62-97f8-465b-b4f7-6517ac2bc994", "5174ad3f-0569-42fd-8adf-011d8206db6c"]),
    ("InformationRegisterRecordManager", &["Cancel"], &["679b62d9-ff72-4329-bf3a-c0c32b311dd2"]),
    ("InformationRegisterRecordManager", &["ChangeHistory"], &["174e58ce-82ad-4787-b956-9367937f7971"]),
    ("InformationRegisterRecordManager", &["Close"], &["3772996b-41f4-4c47-a5a8-ea397db424ae"]),
    ("InformationRegisterRecordManager", &["Copy"], &["68baa1bc-edd1-4d9b-ad80-1d53fb8a7988"]),
    ("InformationRegisterRecordManager", &["CustomizeForm"], &["198ea630-fda2-4cda-8a23-f999f4c67ee6"]),
    ("InformationRegisterRecordManager", &["Delete"], &["c32d43de-b820-49d0-bf7a-d70829f48f40"]),
    ("InformationRegisterRecordManager", &["Help"], &["39bb0fe9-771d-4dd5-8a6e-2d16984523af"]),
    ("InformationRegisterRecordManager", &["OK"], &["f3613d5c-20c6-46e5-b4d5-7d712ece1296"]),
    ("InformationRegisterRecordManager", &["Reread"], &["1f317795-c420-4a30-b594-c492abc55f7a"]),
    ("InformationRegisterRecordManager", &["RestoreValues"], &["71e0226e-ebb2-4e33-8745-0a94a01bbf15"]),
    ("InformationRegisterRecordManager", &["SaveValues"], &["239f0103-8de9-4fdf-b485-eb5531da7e51"]),
    ("InformationRegisterRecordManager", &["Write"], &["fe558fde-99b3-45d0-a060-9fc2905309f6"]),
    ("InformationRegisterRecordManager", &["WriteAndClose"], &["32df4349-2607-4c2b-a4b9-bca4a1a28bd7"]),
    ("InformationRegisterRecordManager", &["Yes"], &["5d41082e-9619-42ec-b96f-98b082b3a2f0"]),
    ("InformationRegisterRecordSet", &["Abort", "Ignore", "No", "Retry"], &["06ee6a21-061e-47f8-81c5-92ae8b8f3b5d", "3f01ed62-97f8-465b-b4f7-6517ac2bc994", "5174ad3f-0569-42fd-8adf-011d8206db6c", "d7e9e72c-8fa7-430c-a3e9-aeadfd57dfc7"]),
    ("InformationRegisterRecordSet", &["Cancel"], &["679b62d9-ff72-4329-bf3a-c0c32b311dd2"]),
    ("InformationRegisterRecordSet", &["CustomizeForm"], &["198ea630-fda2-4cda-8a23-f999f4c67ee6"]),
    ("InformationRegisterRecordSet", &["OK"], &["f3613d5c-20c6-46e5-b4d5-7d712ece1296"]),
    ("InformationRegisterRecordSet", &["Reread", "Write"], &["1f317795-c420-4a30-b594-c492abc55f7a", "fe558fde-99b3-45d0-a060-9fc2905309f6"]),
    ("InformationRegisterRecordSet", &["RestoreValues"], &["71e0226e-ebb2-4e33-8745-0a94a01bbf15"]),
    ("InformationRegisterRecordSet", &["SaveValues"], &["239f0103-8de9-4fdf-b485-eb5531da7e51"]),
    ("InformationRegisterRecordSet", &["SwitchActivity"], &["f4613f71-5449-48ed-aea5-de005b272a1d"]),
    ("InformationRegisterRecordSet", &["Yes"], &["5d41082e-9619-42ec-b96f-98b082b3a2f0"]),
    ("ReportObject", &["Cancel"], &["679b62d9-ff72-4329-bf3a-c0c32b311dd2"]),
    ("ReportObject", &["ChangeVariant"], &["fb9d7977-258a-440a-9b59-0a650c86f6a2"]),
    ("ReportObject", &["Close"], &["3772996b-41f4-4c47-a5a8-ea397db424ae"]),
    ("ReportObject", &["CustomizeForm"], &["198ea630-fda2-4cda-8a23-f999f4c67ee6"]),
    ("ReportObject", &["EndEdit"], &["74c1abd6-b274-4654-baf0-7b8418b792ea"]),
    ("ReportObject", &["Generate"], &["b5e6da6b-cec4-450c-876a-6a5f0837f6cc"]),
    ("ReportObject", &["Help"], &["39bb0fe9-771d-4dd5-8a6e-2d16984523af"]),
    // порознь — по командам кнопок (сверка БП)
    ("ReportObject", &["CancelEdit"], &["8149a06a-dbf3-4d4d-a275-5385a4196fc7"]),
    ("ReportObject", &["LoadReportSettings"], &["b0c9afb6-320c-4e36-be21-8f6d48116415"]),
    ("ReportObject", &["SaveReportSettings"], &["7910bb04-ddcc-4e5d-89f0-104c6ad0f187"]),
    ("ReportObject", &["LoadVariant"], &["b08b7a35-583a-4756-b814-0436ff9139c0"]),
    ("ReportObject", &["SaveVariant"], &["9bffcf73-7b1d-4a8d-bf23-5e051af3ee29"]),
    ("ReportObject", &["NewWindow"], &["03df6ee5-883c-4cc6-b319-d886d1a9b2c8"]),
    ("ReportObject", &["OK"], &["f3613d5c-20c6-46e5-b4d5-7d712ece1296"]),
    ("ReportObject", &["ReportSettings"], &["0fb774df-ec1c-4e23-9ed1-e089974f74bf"]),
    ("ReportObject", &["RestoreValues"], &["71e0226e-ebb2-4e33-8745-0a94a01bbf15"]),
    ("ReportObject", &["SaveValues"], &["239f0103-8de9-4fdf-b485-eb5531da7e51"]),
    ("ReportObject", &["Yes"], &["5d41082e-9619-42ec-b96f-98b082b3a2f0"]),
    // как у справочника, документа и записи регистра (команда объекта)
    ("TaskObject", &["Delete"], &["c32d43de-b820-49d0-bf7a-d70829f48f40"]),
    // по команде кнопки (сверка БП)
    ("TaskObject", &["SetDeletionMark"], &["827b541d-30c1-4f06-aecf-92aa496a0835"]),
    ("TaskObject", &["Activate", "Copy", "Execute", "ExecuteAndClose"], &["2cacadf7-8fb3-4ec6-ae2b-0ca3fd311c9e", "32df4349-2607-4c2b-a4b9-bca4a1a28bd7", "68baa1bc-edd1-4d9b-ad80-1d53fb8a7988", "6f959e83-23ec-4991-901d-575d7ea98868"]),
    ("dcsset:SettingsComposer", &["EndEdit"], &["74c1abd6-b274-4654-baf0-7b8418b792ea"]),
    ("xs:string", &["Abort", "Ignore", "No", "Retry"], &["06ee6a21-061e-47f8-81c5-92ae8b8f3b5d", "3f01ed62-97f8-465b-b4f7-6517ac2bc994", "5174ad3f-0569-42fd-8adf-011d8206db6c", "d7e9e72c-8fa7-430c-a3e9-aeadfd57dfc7"]),
    ("xs:string", &["Cancel"], &["679b62d9-ff72-4329-bf3a-c0c32b311dd2"]),
    ("xs:string", &["Close"], &["3772996b-41f4-4c47-a5a8-ea397db424ae"]),
    ("xs:string", &["CustomizeForm"], &["198ea630-fda2-4cda-8a23-f999f4c67ee6"]),
    ("xs:string", &["Help"], &["39bb0fe9-771d-4dd5-8a6e-2d16984523af"]),
    ("xs:string", &["OK"], &["f3613d5c-20c6-46e5-b4d5-7d712ece1296"]),
    ("xs:string", &["RestoreValues"], &["71e0226e-ebb2-4e33-8745-0a94a01bbf15"]),
    ("xs:string", &["SaveValues"], &["239f0103-8de9-4fdf-b485-eb5531da7e51"]),
    ("xs:string", &["Yes"], &["5d41082e-9619-42ec-b96f-98b082b3a2f0"]),
    // досняты по всем формам БП (в том числе не загружаемым пока)
    ("DynamicList:Catalog", &["ChangeHistory"], &["c9abb6b0-eafd-4505-8312-9a7b6888cbf3"]),
    ("DynamicList:Catalog", &["DynamicListStandardSettings", "LoadDynamicListSettings", "SaveDynamicListSettings"], &["952c2984-9955-415a-8235-5c710aabe732", "d5c3842d-7252-4370-9174-756a6cc553e5", "d603a249-6eb3-4e38-bb2d-a8a86a8ab156"]),
    ("DynamicList:Catalog", &["GetURL"], &["9885f4b6-d830-435f-a0e3-6b70ffe0f85c"]),
    ("DynamicList:Catalog", &["ShowMultipleSelection"], &["9fea4ba9-7d33-47d4-a271-cb54df4a9b74"]),
    ("DynamicList:Document", &["Choose"], &["8e2b82cf-d1ea-46b2-afdf-a8d64e66ea2b"]),
    ("DynamicList:Document", &["FindByCurrentValue"], &["b520ca45-d8db-4982-b128-bb42a6afd911"]),
    ("DynamicList:Document", &["SetDateInterval"], &["eb880cb2-a91f-4ad6-afb7-f0e6d7a1b111"]),
    ("DynamicList:Document", &["ShowMultipleSelection"], &["9fea4ba9-7d33-47d4-a271-cb54df4a9b74"]),
    ("DynamicList:InformationRegister", &["DynamicListStandardSettings", "LoadDynamicListSettings", "SaveDynamicListSettings"], &["952c2984-9955-415a-8235-5c710aabe732", "d5c3842d-7252-4370-9174-756a6cc553e5", "d603a249-6eb3-4e38-bb2d-a8a86a8ab156"]),
    ("DynamicList:InformationRegister", &["FindByCurrentValue"], &["b520ca45-d8db-4982-b128-bb42a6afd911"]),
    ("DynamicList:InformationRegister", &["GetURL"], &["9885f4b6-d830-435f-a0e3-6b70ffe0f85c"]),
    ("DynamicList:InformationRegister", &["ListSettings", "OutputList"], &["1c00edb8-a826-4855-9bde-94dbc5f620e5", "9758d344-4b1d-4dc9-80bd-81060bc18b2a"]),
    ("ExchangePlanObject", &["Copy", "Delete", "SetDeletionMark"], &["68baa1bc-edd1-4d9b-ad80-1d53fb8a7988", "827b541d-30c1-4f06-aecf-92aa496a0835", "c32d43de-b820-49d0-bf7a-d70829f48f40"]),
    ("ReportObject", &["ChangeSettingsStructure"], &["3ea8bf45-5f33-4545-a3bb-29f80666b627"]),
    ("ReportObject", &["CompactViewMode"], &["45bea91e-d9e5-4756-b5e6-375ab84b6903"]),
    ("ReportObject", &["Print", "Save"], &["a11fe36e-0b45-4c07-80b3-2346b660a51e", "a6d73055-3730-42e7-8934-3145ee987141"]),
    ("ReportObject", &["StandardSettings"], &["c8f1bd8c-b4d1-46d5-97b3-929b5606b6c3"]),
    ("TaskObject", &["Write"], &["fe558fde-99b3-45d0-a060-9fc2905309f6"]),
];

/// Команды диалога и общие команды формы: у всех классов один GUID (во всех группах таблицы `T` — те же).
const COMMON: &[(&str, &str)] = &[
    // Abort и Retry в сверке встречались только вместе; порознь — по коду платформы (`platform::FORM`)
    ("Abort", "3f01ed62-97f8-465b-b4f7-6517ac2bc994"),
    ("Cancel", "679b62d9-ff72-4329-bf3a-c0c32b311dd2"),
    ("Close", "3772996b-41f4-4c47-a5a8-ea397db424ae"),
    ("CustomizeForm", "198ea630-fda2-4cda-8a23-f999f4c67ee6"),
    ("Help", "39bb0fe9-771d-4dd5-8a6e-2d16984523af"),
    ("Ignore", "d7e9e72c-8fa7-430c-a3e9-aeadfd57dfc7"),
    ("No", "06ee6a21-061e-47f8-81c5-92ae8b8f3b5d"),
    ("OK", "f3613d5c-20c6-46e5-b4d5-7d712ece1296"),
    // по формам, где исключена одна из пары (документ, отчет)
    ("OpenFromMainServer", "573e81b7-57eb-45f0-ba4d-ada7c2537a2d"),
    ("OpenFromStandaloneServer", "0ea1a92b-3477-44dd-b152-ea7d411f1c5d"),
    ("RestoreValues", "71e0226e-ebb2-4e33-8745-0a94a01bbf15"),
    ("Retry", "5174ad3f-0569-42fd-8adf-011d8206db6c"),
    ("SaveValues", "239f0103-8de9-4fdf-b485-eb5531da7e51"),
    ("Yes", "5d41082e-9619-42ec-b96f-98b082b3a2f0"),
];

/// Исключенные команды `names` формы с основным реквизитом класса `class`.
pub(super) fn excluded(class: &str, names: &[String]) -> Result<Node, String> {
    // сначала таблицы кода платформы, остальное — по выборке сверки
    let mut gs: Vec<&str> = names.iter().filter_map(|n| super::platform::form_command(class, n)).collect();
    let names: Vec<&String> = names.iter().filter(|n| super::platform::form_command(class, n).is_none()).collect();
    let mut used = vec![false; names.len()];
    for (i, n) in names.iter().enumerate() {
        if let Some((_, g)) = COMMON.iter().find(|(c, _)| c == n) {
            gs.push(g);
            used[i] = true;
        }
    }
    let common = |n: &str| COMMON.iter().any(|(c, g)| *c == n || *g == n);
    for (_, group, guids) in T.iter().filter(|(c, ..)| *c == class) {
        let group: Vec<&str> = group.iter().copied().filter(|n| !common(n)).collect();
        let guids: Vec<&str> = guids.iter().copied().filter(|g| !common(g)).collect();
        let hit: Vec<usize> = names.iter().enumerate().filter(|(_, n)| group.contains(&n.as_str())).map(|(i, _)| i).collect();
        if hit.is_empty() {
            continue;
        }
        if hit.len() != group.len() {
            return Err(format!("команды {group:?} исключаются только вместе (класс {class:?})"));
        }
        for i in hit {
            used[i] = true;
        }
        gs.extend(guids);
    }
    if let Some(i) = used.iter().position(|u| !u) {
        return Err(format!("исключенная команда {} у формы с основным реквизитом {class:?} не встречена", names[i]));
    }
    gs.sort_unstable();
    let mut v = vec![an(gs.len())];
    v.extend(gs.into_iter().map(a));
    Ok(l(v))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(n: Node) -> String {
        Doc::new(true, vec![n]).write().unwrap().replace(['\u{feff}', '\r', '\n'], "")
    }

    #[test]
    fn groups_only_whole_and_sorted_by_guid() {
        let names = |v: &[&str]| v.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        assert_eq!(text(excluded("CatalogObject", &names(&["Copy"])).unwrap()), "{1,68baa1bc-edd1-4d9b-ad80-1d53fb8a7988}");
        assert!(text(excluded("", &names(&["Retry", "Abort", "Ignore"])).unwrap()).starts_with("{3,3f01ed62"));
        assert!(text(excluded("DataProcessorObject", &names(&["Ignore"])).unwrap()).starts_with("{1,d7e9e72c"));
        assert!(excluded("НетТакого", &names(&["Copy"])).is_err());
    }

    /// Выборка сверки — подмножество таблиц кода платформы: каждое имя группы известно платформе, GUID группы те же.
    #[test]
    fn sample_is_subset_of_platform() {
        for (class, group, guids) in T {
            let mut p: Vec<&str> = group.iter().map(|n| super::super::platform::form_command(class, n).unwrap_or_else(|| panic!("{class}: {n}"))).collect();
            p.sort_unstable();
            let mut g = guids.to_vec();
            g.sort_unstable();
            assert_eq!(p, g, "{class}: {group:?}");
        }
        for (n, g) in COMMON {
            assert_eq!(super::super::platform::form_command("", n), Some(*g), "{n}");
        }
    }
}
