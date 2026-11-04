//! Модель метаданных поверх контейнера: дерево объектов, имена, модули.
//!
//! Таблицы GUID классов и ролей файлов получены голосованием: GUID объектов из .cf
//! шаблона БП 3.0.171 сопоставлены с XML-выгрузками (ConfigDumpInfo.xml) тех же объектов.

use crate::brace::{self, is_guid, Node};
use crate::container::{self, Container, Entry};
use std::collections::HashMap;

/// Класс корня внешней обработки (`ClassId` в `InternalInfo` выгрузки `ExternalDataProcessor`).
pub const EXTERNAL_DATA_PROCESSOR: &str = "c3831ec8-d8d5-4f93-8a22-f9bfae07327f";

/// (GUID класса, имя в XML-выгрузке, группа в дереве, один объект)
pub const CLASSES: &[(&str, &str, &str, &str)] = &[
    ("37f2fa9a-b276-11d4-9435-004095e12fc7", "Subsystem", "Подсистемы", "Подсистема"),
    ("0fe48980-252d-11d6-a3c7-0050bae0a776", "CommonModule", "Общие модули", "Общий модуль"),
    ("24c43748-c938-45d0-8d14-01424a72b11e", "SessionParameter", "Параметры сеанса", "Параметр сеанса"),
    ("09736b02-9cac-4e3f-b4f7-d3e9576ab948", "Role", "Роли", "Роль"),
    ("15794563-ccec-41f6-a83c-ec5f7b9a5bc1", "CommonAttribute", "Общие реквизиты", "Общий реквизит"),
    ("857c4a91-e5f4-4fac-86ec-787626f1c108", "ExchangePlan", "Планы обмена", "План обмена"),
    ("3e7bfcc0-067d-11d6-a3c7-0050bae0a776", "FilterCriterion", "Критерии отбора", "Критерий отбора"),
    ("4e828da6-0f44-4b5b-b1c0-a2b3cfe7bdcc", "EventSubscription", "Подписки на события", "Подписка на событие"),
    ("11bdaf85-d5ad-4d91-bb24-aa0eee139052", "ScheduledJob", "Регламентные задания", "Регламентное задание"),
    ("af547940-3268-434f-a3e7-e47d6d2638c3", "FunctionalOption", "Функциональные опции", "Функциональная опция"),
    ("30d554db-541e-4f62-8970-a1c6dcfeb2bc", "FunctionalOptionsParameter", "Параметры функциональных опций", "Параметр функциональных опций"),
    ("c045099e-13b9-4fb6-9d50-fca00202971e", "DefinedType", "Определяемые типы", "Определяемый тип"),
    ("46b4cd97-fd13-4eaa-aba2-3bddd7699218", "SettingsStorage", "Хранилища настроек", "Хранилище настроек"),
    ("2f1a5187-fb0e-4b05-9489-dc5dd6412348", "CommonCommand", "Общие команды", "Общая команда"),
    ("1c57eabe-7349-44b3-b1de-ebfeab67b47d", "CommandGroup", "Группы команд", "Группа команд"),
    ("07ee8426-87f1-11d5-b99c-0050bae0a95d", "CommonForm", "Общие формы", "Общая форма"),
    ("0c89c792-16c3-11d5-b96b-0050bae0a95d", "CommonTemplate", "Общие макеты", "Общий макет"),
    ("7dcd43d9-aca5-4926-b549-1842e6a4e8cf", "CommonPicture", "Общие картинки", "Общая картинка"),
    ("cc9df798-7c94-4616-97d2-7aa0b7bc515e", "XDTOPackage", "XDTO-пакеты", "XDTO-пакет"),
    ("8657032e-7740-4e1d-a3ba-5dd6e8afb78f", "WebService", "Web-сервисы", "Web-сервис"),
    ("0fffc09c-8f4c-47cc-b41c-8d5c5a221d79", "HTTPService", "HTTP-сервисы", "HTTP-сервис"),
    ("d26096fb-7a5d-4df9-af63-47d04771fa9b", "WSReference", "WS-ссылки", "WS-ссылка"),
    ("58848766-36ea-4076-8800-e91eb49590d7", "StyleItem", "Элементы стиля", "Элемент стиля"),
    // нет в БП; ERP 2.5.22: Основной
    ("3e5404af-6ef8-4c73-ad11-91bd2dfac4c8", "Style", "Стили", "Стиль"),
    ("9cd510ce-abfc-11d4-9434-004095e12fc7", "Language", "Языки", "Язык"),
    ("0195e80c-b157-11d4-9435-004095e12fc7", "Constant", "Константы", "Константа"),
    ("cf4abea6-37b2-11d4-940f-008048da11f9", "Catalog", "Справочники", "Справочник"),
    ("061d872a-5787-460e-95ac-ed74ea3a3e84", "Document", "Документы", "Документ"),
    ("36a8e346-9aaa-4af9-bdbd-83be3c177977", "DocumentNumerator", "Нумераторы документов", "Нумератор"),
    ("bc587f20-35d9-11d6-a3c7-0050bae0a776", "Sequence", "Последовательности", "Последовательность"),
    ("4612bd75-71b7-4a5c-8cc5-2b0b65f9fa0d", "DocumentJournal", "Журналы документов", "Журнал документов"),
    ("f6a80749-5ad7-400b-8519-39dc5dff2542", "Enum", "Перечисления", "Перечисление"),
    ("631b75a0-29e2-11d6-a3c7-0050bae0a776", "Report", "Отчеты", "Отчет"),
    ("bf845118-327b-4682-b5c6-285d2a0eb296", "DataProcessor", "Обработки", "Обработка"),
    ("82a1b659-b220-4d94-a9bd-14d757b95a48", "ChartOfCharacteristicTypes", "Планы видов характеристик", "План видов характеристик"),
    ("238e7e88-3c5f-48b2-8a3b-81ebbecb20ed", "ChartOfAccounts", "Планы счетов", "План счетов"),
    ("30b100d6-b29f-47ac-aec7-cb8ca8a54767", "ChartOfCalculationTypes", "Планы видов расчета", "План видов расчета"),
    ("13134201-f60b-11d5-a3c7-0050bae0a776", "InformationRegister", "Регистры сведений", "Регистр сведений"),
    ("b64d9a40-1642-11d6-a3c7-0050bae0a776", "AccumulationRegister", "Регистры накопления", "Регистр накопления"),
    ("2deed9b8-0056-4ffe-a473-c20a6c32a0bc", "AccountingRegister", "Регистры бухгалтерии", "Регистр бухгалтерии"),
    // нет в БП; GUID — по части 9de14907 корня ERP 2.5.22 (вместе с планами видов расчета)
    ("f2de87a8-64e5-45eb-a22d-b3aedab050e7", "CalculationRegister", "Регистры расчета", "Регистр расчета"),
    ("fcd3404e-1523-48ce-9bc0-ecdb822684a1", "BusinessProcess", "Бизнес-процессы", "Бизнес-процесс"),
    ("3e63355c-1378-4953-be9b-1deb5fb6bec5", "Task", "Задачи", "Задача"),
    // нет в БП 3.0.171; GUID — по корню УТ 11.5.27 и ее выгрузке
    ("6e6dc072-b7ac-41e7-8f88-278d25b6da2a", "Bot", "Боты", "Бот"),
    ("bf3420b0-f6f9-41a0-b83a-fe9d4ab0b65d", "IntegrationService", "Сервисы интеграции", "Сервис интеграции"),
];

/// GUID вложенных коллекций → вид элемента (из голосования по XML-выгрузкам).
pub const SUBCLASSES: &[(&str, &str)] = &[
    ("888744e1-b616-11d4-9436-004095e12fc7", "Attribute"),
    ("cf4abea7-37b2-11d4-940f-008048da11f9", "Attribute"),
    ("45e46cbc-3e24-4165-8b7b-cc98a6f80211", "Attribute"),
    ("a2207540-1400-11d6-a3c7-0050bae0a776", "Attribute"),
    ("5d24a9d1-098e-11d6-b9b8-0050bae0a95d", "Attribute"),
    ("7e7123e0-29e2-11d6-a3c7-0050bae0a776", "Attribute"),
    ("ec6bb5e5-b7a8-4d75-bec9-658107a699cf", "Attribute"),
    ("c339c860-29e2-11d6-a3c7-0050bae0a776", "Attribute"),
    ("87c988de-ecbf-413b-87b0-b9516df05e28", "Attribute"),
    ("b64d9a42-1642-11d6-a3c7-0050bae0a776", "Attribute"),
    ("1a1b4fea-e093-470d-94ff-1d2f16cda2ab", "Attribute"),
    ("31182525-9346-4595-81f8-6f91a72ebe06", "Attribute"),
    ("0dc22ad2-476a-4794-afae-cfa7ed251752", "Attribute"),
    ("8ddfb495-c5fc-46b9-bdc5-bcf58341bff0", "Attribute"),
    ("6e65cbf5-daa8-4d8d-bef8-59723f4e5777", "Attribute"),
    ("9d28ee33-9c7e-4a1b-8f13-50aa9b36607b", "Attribute"),
    ("13134203-f60b-11d5-a3c7-0050bae0a776", "Dimension"),
    // регистр расчета (ERP 2.5.22: Начисления, Удержания)
    ("b12fc850-8210-43c8-ae05-89567e698fbb", "Dimension"),
    ("702b33ad-843e-41aa-8064-112cd38cc92c", "Resource"),
    ("1b304502-2216-440b-960f-60decd04bb5d", "Attribute"),
    ("b64d9a43-1642-11d6-a3c7-0050bae0a776", "Dimension"),
    ("437488c0-35e2-11d6-a3c7-0050bae0a776", "Dimension"),
    ("35b63b9d-0adf-4625-a047-10ae874c19a3", "Dimension"),
    ("13134202-f60b-11d5-a3c7-0050bae0a776", "Resource"),
    ("b64d9a41-1642-11d6-a3c7-0050bae0a776", "Resource"),
    ("63405499-7491-4ce3-ac72-43433cbe4112", "Resource"),
    ("932159f9-95b2-4e76-a8dd-8849fe5c5ded", "TabularSection"),
    ("2bcef0d1-0981-11d6-b9b8-0050bae0a95d", "TabularSection"),
    ("21c53e09-8950-4b5e-a6a0-1054f1bbc274", "TabularSection"),
    ("b077d780-29e2-11d6-a3c7-0050bae0a776", "TabularSection"),
    ("52293f4b-f98c-43ea-a80f-41047ae7ab58", "TabularSection"),
    ("a3fe6537-d787-40f7-8a06-419d2f0c1cfd", "TabularSection"),
    ("54e36536-7863-42fd-bea3-c5edd3122fdc", "TabularSection"),
    ("054aa8cf-faa6-4634-aef4-1087ca0d88fc", "TabularSection"),
    // табличные части плана счетов и задачи — с каркаса
    ("4c7fec95-d1bd-4508-8a01-f1db090d9af8", "TabularSection"),
    ("ee865d4b-a458-48a0-b38f-5a26898feeb0", "TabularSection"),
    ("bee0a08c-07eb-40c0-8544-5c364c171465", "EnumValue"),
    ("13134204-f60b-11d5-a3c7-0050bae0a776", "Form"),
    ("d5b0e5ed-256d-401c-9c36-f630cafd8a62", "Form"),
    ("fb880e93-47d7-4127-9357-a20e69c17545", "Form"),
    ("87c509ab-3d38-4d67-b379-aca796298578", "Form"),
    ("fdf816d2-1ead-11d5-b975-0050bae0a95d", "Form"),
    ("a3b368c0-29e2-11d6-a3c7-0050bae0a776", "Form"),
    ("b8533c0c-2342-4db3-91a2-c2b08cbf6b23", "Form"),
    ("ec81ad10-ca07-11d5-b9a5-0050bae0a95d", "Form"),
    ("3f7a8120-b71a-4265-98bf-4d9bc09b7719", "Form"),
    ("b64d9a44-1642-11d6-a3c7-0050bae0a776", "Form"),
    ("eb2b78a8-40a6-4b7e-b1b3-6ca9966cbc94", "Form"),
    ("5372e285-03db-4f8c-8565-fe56f1aea40e", "Form"),
    ("3f58cbfb-4172-4e54-be49-561a579bb38b", "Form"),
    ("33f2e54b-37ce-4a7a-a569-b648d7aa4634", "Form"),
    ("d3b5d6eb-4ea2-4610-a3e2-624d4e815934", "Form"),
    ("a7f8f92a-7a4b-484b-937e-42d242e64144", "Form"),
    // формы критерия отбора (БП 3.0.107 ФайлыВТоме)
    ("00867c40-06b1-11d6-a3c7-0050bae0a776", "Form"),
    ("4fe87c89-9ad4-43f6-9fdb-9dc83b3879c6", "Command"),
    ("e7ff38c0-ec3c-47a0-ae90-20c73ca72246", "Command"),
    ("45556acb-826a-4f73-898a-6025fc9536e1", "Command"),
    ("b44ba719-945c-445c-8aab-1088fa4df16e", "Command"),
    ("b544fc6a-2ba3-4885-8fb2-cb289fb6d65e", "Command"),
    ("99f328af-a77f-4572-a2d8-80ed20c81890", "Command"),
    ("a49a35ce-120a-4c80-8eea-b0618479cd70", "Command"),
    ("0df30176-6865-4787-9fc8-609eb144174f", "Command"),
    ("f27c2152-a2c9-4c30-adb1-130f5eb2590f", "Command"),
    // команды бизнес-процесса и плана обмена (в БП 3.0.171 их нет, найдены в УТ 11.5.27)
    ("7a3e533c-f232-40d5-a932-6a311d2480bf", "Command"),
    ("d5207c64-11d5-4d46-bba2-55b7b07ff4eb", "Command"),
    // найдены на каркасе (tools/skel/nest.py): команды ПВХ, ПВР, перечисления, критерия отбора, регистра бухгалтерии
    ("95b5e1d4-abfa-4a16-818d-a5b07b7d3f73", "Command"),
    ("2e90c75b-2f0c-4899-a7d4-5426eaefc96e", "Command"),
    ("6d8d73a7-ba29-401d-9032-3872ec2d6433", "Command"),
    ("23fa3b84-220a-40e9-8331-e588bed87f7d", "Command"),
    ("7162da60-f7fe-4d78-ad5d-e31700f9af18", "Command"),
    ("3daea016-69b7-4ed4-9453-127911372fe6", "Template"),
    ("37f2fa9a-b276-11d4-9435-004095e12fc7", "Subsystem"),
    ("ec6896c2-9b28-42d8-9140-48491146b8ea", "URLTemplate"),
    ("21c96ea8-c8fc-424a-a0b4-e1ffb2fa1a73", "Method"),
    ("36186084-c23a-43bd-876c-a3a8ba1a9622", "Operation"),
    ("b78a00b2-2260-4ef5-a70c-17889cfee695", "Parameter"),
    ("5aee69df-0513-4c6c-9815-103102471712", "Column"),
    ("78bd1243-c4df-46c3-8138-e147465cb9a4", "AccountingFlag"),
    ("c70ca527-5042-4cad-a315-dcb4007e32a3", "ExtDimensionAccountingFlag"),
    ("e97c0570-251c-4566-b0f1-10686820f143", "AddressingAttribute"),
    // класс канала — дочерний класс сервиса интеграции (integ.dll FUN_1800f2470, коллекция записи — FUN_1800e0a60)
    ("acb7e81f-0637-4ebd-88ff-954ba075ae51", "IntegrationServiceChannel"),
];

/// Порядок и русские подписи вложенных групп, как в дереве Конфигуратора.
pub const SUBKINDS: &[(&str, &str)] = &[
    ("Dimension", "Измерения"),
    ("Resource", "Ресурсы"),
    ("Attribute", "Реквизиты"),
    ("AddressingAttribute", "Реквизиты адресации"),
    ("AccountingFlag", "Признаки учета"),
    ("ExtDimensionAccountingFlag", "Признаки учета субконто"),
    ("TabularSection", "Табличные части"),
    ("EnumValue", "Значения"),
    ("Column", "Графы"),
    ("URLTemplate", "Шаблоны URL"),
    ("Method", "Методы"),
    ("Operation", "Операции"),
    ("Parameter", "Параметры"),
    ("IntegrationServiceChannel", "Каналы"),
    ("Subsystem", "Подсистемы"),
    ("Form", "Формы"),
    ("Command", "Команды"),
    ("Template", "Макеты"),
];

/// Суффикс файла объекта → роль (собственные файлы объекта; у формы .0 — форма, у команды .2 — модуль).
pub fn own_role(class: &str, suffix: &str) -> Option<&'static str> {
    let t: &[(&str, &str)] = match class {
        // shortcut: .3 — справка (Ext/Help.xml) по записи ЗУП `{5,…}` того же вида, что у объектов, а не по таблице свойств
        // платформы, пересмотреть, если у корня встретится .3 другого вида
        "Configuration" => &[("0", "OrdinaryApplicationModule"), ("1", "Logo"), ("2", "Splash"), ("3", "Help"), ("4", "ParentConfigurations"), ("5", "ExternalConnectionModule"), ("6", "ManagedApplicationModule"), ("7", "SessionModule"), ("8", "HomePageWorkArea"), ("9", "MainSectionCommandInterface"), ("10", "MobileClientSignature"), ("a", "CommandInterface"), ("b", "ClientApplicationInterface"), ("c", "MainSectionPicture"), ("f", "StandaloneConfigurationContent")],
        // у сервиса интеграции файл 0 — свойство-модуль d5963243 (integ.dll FUN_1800f2470)
        "CommonModule" | "HTTPService" | "WebService" | "IntegrationService" => &[("0", "Module")],
        "CommonCommand" => &[("1", "Help"), ("2", "CommandModule")],
        "CommonForm" | "Form" => &[("0", "Form"), ("1", "Help")],
        "Command" => &[("2", "CommandModule")],
        "CommonTemplate" | "Template" => &[("0", "Template")],
        "CommonPicture" => &[("0", "Picture")],
        "Subsystem" => &[("0", "Help"), ("1", "CommandInterface")],
        "Catalog" => &[("0", "ObjectModule"), ("1", "Help"), ("3", "ManagerModule"), ("1c", "Predefined")],
        "Document" => &[("0", "ObjectModule"), ("1", "Help"), ("2", "ManagerModule")],
        "DataProcessor" | "Report" | "ExternalDataProcessor" => &[("0", "ObjectModule"), ("1", "Help"), ("2", "ManagerModule")],
        "Constant" => &[("0", "ValueManagerModule"), ("1", "ManagerModule")],
        "Enum" | "FilterCriterion" => &[("0", "ManagerModule")],
        "DocumentJournal" => &[("0", "Help"), ("1", "ManagerModule")],
        "InformationRegister" => &[("0", "Help"), ("1", "RecordSetModule"), ("2", "ManagerModule"), ("3", "AdditionalIndexes")],
        "AccumulationRegister" | "CalculationRegister" => &[("0", "Help"), ("1", "RecordSetModule"), ("2", "ManagerModule")],
        "AccountingRegister" => &[("5", "Help"), ("6", "RecordSetModule"), ("7", "ManagerModule")],
        "Sequence" => &[("0", "RecordSetModule")],
        "ExchangePlan" => &[("0", "Help"), ("1", "Content"), ("2", "ObjectModule"), ("3", "ManagerModule")],
        "ChartOfAccounts" => &[("5", "Help"), ("9", "Predefined"), ("14", "ObjectModule"), ("15", "ManagerModule")],
        "ChartOfCharacteristicTypes" => &[("5", "Help"), ("7", "Predefined"), ("15", "ObjectModule"), ("16", "ManagerModule")],
        "ChartOfCalculationTypes" => &[("0", "ObjectModule"), ("1", "Help"), ("2", "Predefined"), ("3", "ManagerModule")],
        "BusinessProcess" => &[("5", "Help"), ("6", "ObjectModule"), ("7", "Flowchart"), ("8", "ManagerModule")],
        "Task" => &[("5", "Help"), ("6", "ObjectModule"), ("7", "ManagerModule")],
        "SettingsStorage" => &[("8", "ManagerModule")],
        "Role" => &[("0", "Rights")],
        "Style" => &[("0", "Style")],
        "ScheduledJob" => &[("0", "Schedule")],
        "XDTOPackage" => &[("0", "Package")],
        "WSReference" => &[("0", "WSDefinition")],
        _ => &[],
    };
    t.iter().find(|(s, _)| *s == suffix).map(|(_, r)| *r)
}

pub fn role_ru(role: &str) -> &str {
    match role {
        "Module" => "Модуль",
        "ObjectModule" => "Модуль объекта",
        "ManagerModule" => "Модуль менеджера",
        "RecordSetModule" => "Модуль набора записей",
        "ValueManagerModule" => "Модуль менеджера значения",
        "CommandModule" => "Модуль команды",
        "Form" => "Форма",
        "Help" => "Справочная информация",
        "Template" => "Макет",
        "Picture" => "Картинка",
        "Rights" => "Права",
        "ManagedApplicationModule" => "Модуль управляемого приложения",
        "OrdinaryApplicationModule" => "Модуль обычного приложения",
        "SessionModule" => "Модуль сеанса",
        "ExternalConnectionModule" => "Модуль внешнего соединения",
        "CommandInterface" => "Командный интерфейс",
        "Predefined" => "Предопределенные",
        "Content" => "Состав",
        "Flowchart" => "Карта маршрута",
        "Schedule" => "Расписание",
        "Package" => "Пакет",
        "Splash" => "Заставка",
        "Logo" => "Логотип",
        "ParentConfigurations" => "Родительские конфигурации",
        "HomePageWorkArea" => "Рабочая область начальной страницы",
        "MainSectionCommandInterface" => "Командный интерфейс основного раздела",
        "ClientApplicationInterface" => "Интерфейс клиентского приложения",
        "MainSectionPicture" => "Картинка основного раздела",
        "AdditionalIndexes" => "Дополнительные индексы",
        "WSDefinition" => "WSDL",
        "MobileClientSignature" => "Подпись мобильного клиента",
        "Data" => "Данные",
        r => r,
    }
}

/// Русское название вида объекта или вложенного элемента.
pub fn kind_ru(kind: &str) -> &str {
    if kind == "Configuration" {
        return "Конфигурация";
    }
    if let Some(c) = CLASSES.iter().find(|c| c.1 == kind) {
        return c.3;
    }
    match kind {
        "Attribute" => "Реквизит",
        "Dimension" => "Измерение",
        "Resource" => "Ресурс",
        "TabularSection" => "Табличная часть",
        "EnumValue" => "Значение перечисления",
        "Form" => "Форма",
        "Command" => "Команда",
        "Template" => "Макет",
        "Column" => "Графа",
        "URLTemplate" => "Шаблон URL",
        "Method" => "Метод",
        "Operation" => "Операция",
        "Parameter" => "Параметр",
        "IntegrationServiceChannel" => "Канал сервиса интеграции",
        "AccountingFlag" => "Признак учета",
        "ExtDimensionAccountingFlag" => "Признак учета субконто",
        "AddressingAttribute" => "Реквизит адресации",
        k => k,
    }
}

pub fn is_module_role(role: &str) -> bool {
    role.ends_with("Module") || role == "Form"
}

#[derive(Clone, Debug, Default)]
pub struct Header {
    pub id: String,
    pub name: String,
    pub synonym: String,
    pub comment: String,
}

#[derive(Clone, Debug)]
pub struct Item {
    pub kind: &'static str,
    pub header: Header,
    /// (имя записи контейнера, роль)
    pub files: Vec<(String, &'static str)>,
    pub children: Vec<Item>,
}

#[derive(Clone, Debug)]
pub struct Group {
    pub class: &'static str,
    pub title: &'static str,
    pub items: Vec<Item>,
}

pub struct Config {
    pub entries: Vec<Entry>,
    pub index: HashMap<String, usize>,
    pub root: Item,
    pub groups: Vec<Group>,
    pub version: String,
    /// Части файла (у крупных .cf их две) — для отчета `info`.
    pub parts: Vec<(container::Width, usize)>,
    /// Остальные части файла и номер основной: при сохранении пишутся как были.
    pub other_parts: Vec<(usize, Vec<Entry>)>,
    pub main_part: usize,
    /// Имя записи корневого объекта (из `root` или `configinfo`); роли ссылаются на конфигурацию по нему.
    pub root_id: String,
}

/// Первый узел вида `{N,{1,0,<guid>},"Имя",{синоним},"комментарий",...}` в обходе.
pub fn header(n: &Node) -> Option<Header> {
    let l = header_node(n)?;
    let synonym = l.get(3).map(synonym).unwrap_or_default();
    let comment = l.get(4).and_then(Node::str).unwrap_or_default().to_string();
    Some(Header { id: l[1].get(2)?.atom()?.into(), name: l[2].str()?.into(), synonym, comment })
}

/// Сам узел заголовка (для XML-выгрузки: синоним нужен по всем языкам).
pub fn header_node(n: &Node) -> Option<&[Node]> {
    let l = n.list();
    if let (Some(Node::List(idl)), Some(Node::Str(_))) = (l.get(1), l.get(2)) {
        if idl.len() == 3 && idl[1].atom() == Some("0") && idl[2].atom().is_some_and(is_guid) {
            return Some(l);
        }
    }
    l.iter().find_map(header_node)
}

fn synonym(n: &Node) -> String {
    let l = n.list();
    let pairs: Vec<(&str, &str)> = l.iter().skip(1).collect::<Vec<_>>().chunks(2).filter_map(|c| Some((c.first()?.str()?, c.get(1)?.str()?))).collect();
    pairs.iter().find(|(lang, _)| *lang == "ru").or(pairs.first()).map(|(_, t)| t.to_string()).unwrap_or_default()
}

/// Коллекции `{<guid класса>,<count>,элементы...}`.
fn collections<'a>(n: &'a Node, out: &mut Vec<(&'a str, &'a [Node])>) {
    let l = n.list();
    if l.len() >= 2 {
        if let (Some(c), Some(cnt)) = (l[0].atom(), l[1].atom()) {
            if is_guid(c) && cnt.parse::<usize>().ok() == Some(l.len() - 2) {
                out.push((c, &l[2..]));
            }
        }
    }
    for c in l {
        collections(c, out);
    }
}

impl Config {
    pub fn open(bytes: &[u8]) -> Result<Config, String> {
        let parts = container::read_all(bytes).map_err(|e| e.to_string())?;
        let info: Vec<_> = parts.iter().map(|p: &Container| (p.width, p.entries.len())).collect();
        // Основная конфигурация — часть с наибольшим числом записей.
        let main_part = (0..parts.len()).max_by_key(|&i| parts[i].entries.len()).unwrap();
        let mut main = Vec::new();
        let mut other_parts = Vec::new();
        for (i, p) in parts.into_iter().enumerate() {
            if i == main_part {
                main = p.entries;
            } else {
                other_parts.push((i, p.entries));
            }
        }
        let mut cfg = Config::from_entries(main, info)?;
        cfg.other_parts = other_parts;
        cfg.main_part = main_part;
        Ok(cfg)
    }

    pub fn from_entries(entries: Vec<Entry>, parts: Vec<(container::Width, usize)>) -> Result<Config, String> {
        let index: HashMap<String, usize> = entries.iter().enumerate().map(|(i, e)| (e.name.clone(), i)).collect();
        let text_of = |name: &str| index.get(name).map(|&i| brace::text(&container::content(&entries[i])));
        let (root_id, version) = if let Some(r) = text_of("root") {
            let id = r.split(',').nth(1).unwrap_or_default().trim().to_string();
            (id, text_of("version").unwrap_or_default())
        } else if let Some(ci) = text_of("configinfo") {
            // configinfo = {версия},{2,<root>,},{файлы с SHA-1}
            let all = brace::Doc::parse(&ci)?;
            let id = all.roots.iter().find_map(|x| x.get(1).and_then(Node::atom).filter(|a| is_guid(a))).unwrap_or_default().to_string();
            (id, String::new())
        } else {
            return Err("не найдена запись root/configinfo — это не файл конфигурации".into());
        };
        let root_text = text_of(&root_id).ok_or(format!("нет корневого объекта {root_id}"))?;
        let root_node = brace::parse(&root_text)?;
        // Внешняя обработка: корень `{1,{<uuid>},1,{<класс>,{тело как у DataProcessor}}}`.
        let kind = match root_node.get(3).and_then(|n| n.get(0)).and_then(Node::atom) {
            Some(EXTERNAL_DATA_PROCESSOR) => "ExternalDataProcessor",
            _ => "Configuration",
        };
        let mut cfg = Config {
            root: Item { kind, header: header(&root_node).unwrap_or_default(), files: Vec::new(), children: Vec::new() },
            entries,
            index,
            groups: Vec::new(),
            version: version.split_whitespace().collect::<Vec<_>>().join(""),
            parts,
            other_parts: Vec::new(),
            main_part: 0,
            root_id: root_id.clone(),
        };
        // Файлы модулей приложения названы по id из заголовка, а не по имени записи корня.
        let own = cfg.root.header.id.clone();
        cfg.root.files = cfg.files_of(&own, kind);
        if cfg.root.files.is_empty() {
            cfg.root.files = cfg.files_of(&root_id, kind);
        }
        if kind != "Configuration" {
            let mut children = Vec::new();
            cfg.children_in(&root_node, &mut children);
            cfg.root.children = children;
            return Ok(cfg);
        }
        let mut cols = Vec::new();
        collections(&root_node, &mut cols);
        // Объекты всех классов разбираются по потокам одним списком, затем раскладываются по группам в прежнем порядке.
        let ids: Vec<(usize, &str)> = CLASSES.iter().enumerate().flat_map(|(k, c)| cols.iter().filter(move |(g, _)| *g == c.0).flat_map(|(_, it)| it.iter().filter_map(Node::atom)).map(move |id| (k, id))).collect();
        let mut objects = crate::par::map(&ids, || (), |_, &(k, id)| cfg.object(id, CLASSES[k].1)).into_iter().zip(&ids).peekable();
        for (k, &(_, xml, title, _)) in CLASSES.iter().enumerate() {
            let mut items = Vec::new();
            while let Some((it, _)) = objects.next_if(|(_, (c, _))| *c == k) {
                items.extend(it);
            }
            items.sort_by(|a, b| a.header.name.to_lowercase().cmp(&b.header.name.to_lowercase()));
            if !items.is_empty() {
                cfg.groups.push(Group { class: xml, title, items });
            }
        }
        Ok(cfg)
    }

    fn files_of(&self, id: &str, class: &str) -> Vec<(String, &'static str)> {
        // Кроме номеров бывают шестнадцатеричные суффиксы: .a–.c и .f (состав автономной конфигурации) у корня, .1c у справочника.
        let mut f: Vec<(String, &'static str)> = (0..20)
            .map(|n| n.to_string())
            .chain(["a", "b", "c", "f", "1c"].map(String::from))
            .filter_map(|n| {
                let name = format!("{id}.{n}");
                self.index.contains_key(&name).then(|| (name, own_role(class, &n).unwrap_or("Data")))
            })
            .collect();
        f.sort_by_key(|(_, r)| !is_module_role(r));
        f
    }

    pub fn entry_text(&self, name: &str) -> Option<String> {
        self.index.get(name).map(|&i| brace::text(&container::content(&self.entries[i])))
    }

    pub fn object(&self, id: &str, class: &'static str) -> Option<Item> {
        let text = self.entry_text(id)?;
        let node = brace::parse(&text).ok()?;
        let mut header = header(&node).unwrap_or_default();
        if header.id.is_empty() {
            header.id = id.into();
        }
        let mut children = Vec::new();
        self.children_in(&node, &mut children);
        let files = self.files_of(&header.id, class);
        let files = if files.is_empty() { self.files_of(id, class) } else { files };
        Some(Item { kind: class, header, files, children })
    }

    /// Элементы вложенных коллекций; внутрь элемента не спускается — его дети строятся рекурсивно
    /// (иначе реквизиты табличной части попали бы в реквизиты владельца).
    pub fn children_in(&self, n: &Node, out: &mut Vec<Item>) {
        let l = n.list();
        let kind = match (l.first().and_then(Node::atom), l.get(1).and_then(Node::atom)) {
            (Some(c), Some(cnt)) if cnt.parse::<usize>().ok() == Some(l.len() - 2) => SUBCLASSES.iter().find(|(g, _)| *g == c).map(|(_, k)| *k),
            _ => None,
        };
        let Some(kind) = kind else {
            for c in l {
                self.children_in(c, out);
            }
            return;
        };
        for it in &l[2..] {
            match it {
                // Формы, команды и макеты — отдельные записи контейнера.
                Node::Atom(a) if is_guid(a) => {
                    let mut ch = self.object(a, kind).unwrap_or(Item { kind, header: Header { id: a.clone(), ..Default::default() }, files: vec![], children: vec![] });
                    ch.kind = kind;
                    out.push(ch);
                }
                Node::List(_) => {
                    let Some(h) = self::header(it) else { continue };
                    let files = self.files_of(&h.id, kind);
                    let mut children = Vec::new();
                    self.children_in(it, &mut children);
                    out.push(Item { kind, header: h, files, children });
                }
                _ => {}
            }
        }
    }

    /// Текст модуля из записи: вложенный контейнер (`text`/`module`), форма (`{4,{...},"текст"}`) или сам текст.
    pub fn module_text(&self, entry: &str) -> Option<String> {
        let &i = self.index.get(entry)?;
        let raw = container::content(&self.entries[i]);
        if container::is_container(&raw) {
            let c = container::read_at(&raw, 0).ok()?;
            let e = c.entries.iter().find(|e| e.name == "text" || e.name == "module")?;
            if container::is_container(&e.data) {
                let inner = container::read_at(&e.data, 0).ok()?;
                return inner.entries.iter().find(|e| e.name == "text").map(|e| brace::text(&e.data));
            }
            return Some(brace::text(&e.data));
        }
        let t = brace::text(&raw);
        if t.starts_with('{') {
            let d = brace::Doc::parse(&t).ok()?;
            return d.roots.first()?.get(2).and_then(Node::str).map(str::to_string);
        }
        Some(t)
    }

    /// Записывает новый текст модуля в запись, сохраняя её устройство.
    pub fn set_module_text(&mut self, entry: &str, new_text: &str) -> Result<(), String> {
        let &i = self.index.get(entry).ok_or("нет записи")?;
        let raw = container::content(&self.entries[i]);
        // Модули 1С хранятся с CRLF; редактор вставляет одиночный LF.
        let crlf = new_text.replace("\r\n", "\n").replace('\n', "\r\n");
        let new_text = crlf.as_str();
        let bom =|s: &str| [&[0xef, 0xbb, 0xbf][..], s.as_bytes()].concat();
        let updated = if container::is_container(&raw) {
            let mut c = container::read_at(&raw, 0).map_err(|e| e.to_string())?;
            let e = c.entries.iter_mut().find(|e| e.name == "text" || e.name == "module").ok_or("во вложенном контейнере нет текста")?;
            e.data = bom(new_text);
            container::write(&c.entries)
        } else {
            let t = brace::text(&raw);
            if t.starts_with('{') {
                replace_form_module(&t, new_text).map(|s| bom(&s))?
            } else {
                bom(new_text)
            }
        };
        self.entries[i].data = container::deflate(&updated);
        Ok(())
    }

    /// Файл целиком: основная часть с правками и остальные части без изменений, в исходном порядке и исходной ширины.
    pub fn save(&self) -> Vec<u8> {
        let mut out = Vec::new();
        let write = |i: usize, e: &[Entry]| match self.parts.get(i) {
            Some((w, _)) => container::write_as(e, *w),
            None => container::write(e),
        };
        for i in 0..=self.other_parts.len() {
            if i == self.main_part {
                out.extend(write(i, &self.entries));
            } else if let Some((_, e)) = self.other_parts.iter().find(|(n, _)| *n == i) {
                out.extend(write(i, e));
            }
        }
        out
    }

    /// Модули конфигурации в порядке дерева: (запись, полное имя как у `describe`).
    pub fn module_entries(&self) -> Vec<(String, String)> {
        fn walk(it: &Item, path: &mut Vec<String>, out: &mut Vec<(String, String)>) {
            path.push(if it.header.name.is_empty() { it.header.id.clone() } else { it.header.name.clone() });
            for (f, role) in it.files.iter().filter(|(_, role)| is_module_role(role)) {
                out.push((f.clone(), format!("{}.{}", path.join("."), role_ru(role))));
            }
            for c in &it.children {
                walk(c, path, out);
            }
            path.pop();
        }
        let mut out = Vec::new();
        walk(&self.root, &mut Vec::new(), &mut out);
        for g in &self.groups {
            for it in &g.items {
                walk(it, &mut vec![CLASSES.iter().find(|c| c.1 == g.class).map(|c| c.3).unwrap_or(g.class).to_string()], &mut out);
            }
        }
        out
    }

    /// Поиск по текстам всех модулей: (запись, номер строки, строка).
    pub fn search(&self, needle: &str, limit: usize) -> Vec<(String, usize, String)> {
        let needle = needle.to_lowercase();
        let mut out = Vec::new();
        for (f, _) in self.module_entries() {
            let Some(t) = self.module_text(&f) else { continue };
            for (n, line) in t.lines().enumerate() {
                if out.len() >= limit {
                    return out;
                }
                if line.to_lowercase().contains(&needle) {
                    out.push((f.clone(), n + 1, line.trim().to_string()));
                }
            }
        }
        out
    }

    /// Полное имя объекта по записи-файлу: `Справочник.Валюты.Модуль объекта`.
    pub fn describe(&self, entry: &str) -> String {
        fn find(it: &Item, entry: &str, path: &mut Vec<String>) -> bool {
            path.push(if it.header.name.is_empty() { it.header.id.clone() } else { it.header.name.clone() });
            if let Some((_, r)) = it.files.iter().find(|(f, _)| f == entry) {
                path.push(role_ru(r).to_string());
                return true;
            }
            if it.children.iter().any(|c| find(c, entry, path)) {
                return true;
            }
            path.pop();
            false
        }
        let mut path = Vec::new();
        if find(&self.root, entry, &mut path) {
            return path.join(".");
        }
        for g in &self.groups {
            for it in &g.items {
                let mut p = vec![CLASSES.iter().find(|c| c.1 == g.class).map(|c| c.3).unwrap_or(g.class).to_string()];
                if find(it, entry, &mut p) {
                    return p.join(".");
                }
            }
        }
        entry.to_string()
    }

    /// Запись-файл с ролью, первая по обходу дерева, для которой `pred(запись, роль)`.
    pub fn find_file(&self, pred: impl Fn(&str, &str) -> bool) -> Option<(String, &'static str)> {
        fn find(it: &Item, pred: &dyn Fn(&str, &str) -> bool) -> Option<(String, &'static str)> {
            it.files.iter().find(|(f, r)| pred(f, r)).cloned().or_else(|| it.children.iter().find_map(|c| find(c, pred)))
        }
        find(&self.root, &pred).or_else(|| self.groups.iter().flat_map(|g| &g.items).find_map(|it| find(it, &pred)))
    }

    /// Версия записи из `versions` (`{…,"имя",версия,…}`) — ее отладка называет версией модуля объекта.
    pub fn record_version(&self, name: &str) -> Option<String> {
        let t = self.entry_text("versions")?;
        let at = t.find(&format!("\"{name}\""))? + name.len() + 2;
        let v: String = t[at..].trim_start_matches([',', '\r', '\n', ' ']).chars().take(36).collect();
        (v.len() == 36).then_some(v)
    }
}

/// Заменяет третий элемент `{4,{форма},"модуль",...}` на новый текст; остальное пишется как было.
fn replace_form_module(form: &str, module: &str) -> Result<String, String> {
    let mut d = brace::Doc::parse(form)?;
    let m = d.roots.first_mut().and_then(|r| r.at_mut(&[2])).filter(|n| n.str().is_some()).ok_or("в форме не найден текст модуля")?;
    *m = Node::Str(module.into());
    d.write()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_found_deep() {
        let n = brace::parse("{1,{12,{3,{1,0,0aa19482-2d94-47dc-bcaf-5e3f2c360e13},\"Валюты\",{1,\"ru\",\"Валюты мира\"},\"комм\",0}}}").unwrap();
        let h = header(&n).unwrap();
        assert_eq!((h.name.as_str(), h.synonym.as_str(), h.comment.as_str()), ("Валюты", "Валюты мира", "комм"));
    }

    /// Мини-конфигурация: корень, общий модуль (вложенный контейнер info/text) и общая форма.
    fn sample() -> Vec<u8> {
        use crate::container::{deflate, write, Entry};
        let e = |name: &str, data: Vec<u8>| Entry { name: name.into(), created: 1, modified: 2, data };
        let module = write(&[e("info", b"{3,1,0,\"\",0}".to_vec()), e("text", "\u{feff}Процедура А() КонецПроцедуры".into())]);
        let root = "{2,\r\n{r},7,\r\n{9cd510cd-abfc-11d4-9434-004095e12fc7,\r\n{1,\r\n{67,\r\n{0,\r\n{3,\r\n{1,0,11111111-1111-1111-1111-111111111111},\"Мини\",\r\n{1,\"ru\",\"Мини\"},\"\",0}}}}},\r\n{0fe48980-252d-11d6-a3c7-0050bae0a776,1,22222222-2222-2222-2222-222222222222},\r\n{07ee8426-87f1-11d5-b99c-0050bae0a95d,1,33333333-3333-3333-3333-333333333333}}";
        let obj = |id: &str, name: &str| format!("{{1,{{12,{{3,{{1,0,{id}}},\"{name}\",{{1,\"ru\",\"{name}\"}},\"\",0}}}}}}");
        write(&[
            e("root", deflate(b"{2,rootobj,}")),
            e("rootobj", deflate(root.as_bytes())),
            e("22222222-2222-2222-2222-222222222222", deflate(obj("22222222-2222-2222-2222-222222222222", "ОбщийМодуль1").as_bytes())),
            e("22222222-2222-2222-2222-222222222222.0", deflate(&module)),
            e("33333333-3333-3333-3333-333333333333", deflate(obj("33333333-3333-3333-3333-333333333333", "Форма1").as_bytes())),
            e("33333333-3333-3333-3333-333333333333.0", deflate("{4,{50,0},\"Процедура Ф() КонецПроцедуры\",{0,0}}".as_bytes())),
            e("versions", deflate(b"{1,2,
\"\",aaaaaaaa-0000-0000-0000-000000000001,
\"22222222-2222-2222-2222-222222222222\",bbbbbbbb-0000-0000-0000-000000000002}")),
        ])
    }

    /// Отладка называет модуль объектом, ролью записи и версией записи объекта из `versions`.
    #[test]
    fn debug_module_lookup() {
        let c = Config::open(&sample()).unwrap();
        let (f, r) = c.find_file(|f, _| f == "22222222-2222-2222-2222-222222222222.0").unwrap();
        assert_eq!((f.as_str(), r), ("22222222-2222-2222-2222-222222222222.0", "Module"));
        assert_eq!(c.record_version("22222222-2222-2222-2222-222222222222").as_deref(), Some("bbbbbbbb-0000-0000-0000-000000000002"));
        assert_eq!(c.record_version("33333333-3333-3333-3333-333333333333"), None);
    }

    #[test]
    fn open_edit_save_reopen() {
        let mut c = Config::open(&sample()).unwrap();
        assert_eq!(c.root.header.name, "Мини");
        assert_eq!(c.groups.iter().map(|g| (g.class, g.items[0].header.name.as_str())).collect::<Vec<_>>(), [("CommonModule", "ОбщийМодуль1"), ("CommonForm", "Форма1")]);
        let (m, f) = ("22222222-2222-2222-2222-222222222222.0", "33333333-3333-3333-3333-333333333333.0");
        assert_eq!(c.module_text(m).unwrap(), "Процедура А() КонецПроцедуры");
        assert_eq!(c.module_text(f).unwrap(), "Процедура Ф() КонецПроцедуры");
        c.set_module_text(m, "// новый \"модуль\"\nА = 1;\r\nБ = 2;").unwrap();
        assert_eq!(c.module_text(m).unwrap(), "// новый \"модуль\"\r\nА = 1;\r\nБ = 2;");
        c.set_module_text(m, "// новый \"модуль\"").unwrap();
        c.set_module_text(f, "// новая \"форма\"").unwrap();
        let again = Config::open(&c.save()).unwrap();
        assert_eq!(again.module_text(m).unwrap(), "// новый \"модуль\"");
        assert_eq!(again.module_text(f).unwrap(), "// новая \"форма\"");
        assert_eq!(again.describe(m), "Общий модуль.ОбщийМодуль1.Модуль");
        assert_eq!(again.search("НОВАЯ", 10).len(), 1);
    }

    /// Как у БП 3.0: 32-битная первая часть и 64-битная основная — после сохранения ширина частей та же.
    #[test]
    fn save_keeps_part_widths() {
        use crate::container::{read_all, read_at, write, write_as, Entry, Width};
        let main = read_at(&sample(), 0).unwrap().entries;
        let mut b = write(&[Entry { name: "x".into(), created: 0, modified: 0, data: vec![1] }]);
        b.extend(write_as(&main, Width::W64));
        let mut c = Config::open(&b).unwrap();
        c.set_module_text("22222222-2222-2222-2222-222222222222.0", "// 1").unwrap();
        let w: Vec<Width> = read_all(&c.save()).unwrap().iter().map(|p| p.width).collect();
        assert_eq!(w, [Width::W32, Width::W64]);
    }

    #[test]
    fn form_module_replaced_in_place() {
        let f = "{4,\r\n{50,\"x,}\"},\"Процедура А()\r\nКонецПроцедуры\",\r\n{0,0}\r\n}";
        let out = replace_form_module(f, "// \"новый\"").unwrap();
        assert_eq!(out, "{4,\r\n{50,\"x,}\"},\"// \"\"новый\"\"\",\r\n{0,0}\r\n}");
        let n = brace::parse(&out).unwrap();
        assert_eq!(n.get(2).unwrap().str(), Some("// \"новый\""));
        // Символ вне BMP 1С пишет суррогатами "\d83d"\de00: старый текст не должен оставаться хвостом.
        let f = "{4,\r\n{50,0},\"// a\"\\d83d\"\\de00 TAIL\",\r\n{0,0}\r\n}";
        let out = replace_form_module(f, "NEW😀").unwrap();
        assert_eq!(brace::parse(&out).unwrap().get(2).unwrap().str(), Some("NEW😀"));
        assert!(!out.contains("TAIL"), "{out}");
    }
}
