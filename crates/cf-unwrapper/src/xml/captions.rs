//! Русские подписи палитры свойств: свойство XML → подпись и подсказка, категория, представления значений.
//!
//! Подписи Конфигуратора — строки `IDS_<имя>_MDPROP` («Подпись\nПодсказка»), категории — `IDS_<имя>_MDPROP_CATEGORY`,
//! значения — `IDS_…_MDENUMVAL` и `IDS_…_MDENUM_…` в `basic_ru.res`, `backend_ru.res`, `accnt_ru.res`, `calc_ru.res`, `core85_ru.res`,
//! `bp_ru.res` установленной платформы (docs/editor.md). Имена ресурсов с именами XML не совпадают
//! (`FoldersOnTop` — `IDS_FOLDERSFIRST_MDPROP`, `Comment` — `IDS_DESCRIPTION_MDPROP`), поэтому соответствие — таблица ниже,
//! сверенная по строкам 8.5.1.1529; подпись в таблице — та же строка ресурса на случай, когда платформы нет.

use std::collections::HashMap;

/// Категории палитры по порядку: (код, ключ ресурса, подпись без платформы).
const CATEGORIES: &[(char, &str, &str)] = &[
    ('M', "IDS_MAIN_MDPROP_CATEGORY", "Основные"),
    ('D', "IDS_DATA_MDPROP_CATEGORY", "Данные"),
    ('U', "IDS_USAGE_MDPROP_CATEGORY", "Использование"),
    ('P', "IDS_PRESENT_MDPROP_CATEGORY", "Представление"),
    ('H', "IDS_HELP_MDPROP_CATEGORY", "Справочная информация"),
    ('C', "IDS_COMPATIBILITY_MDPROP_CATEGORY", "Совместимость"),
    ('O', "", "Прочее"),
];

/// Свойство XML → (ключ ресурса без `IDS_` и `_MDPROP`, подпись, категория).
/// Подпись свойства из таблицы (без ресурсов платформы): «Основная форма объекта».
pub fn prop_ru(tag: &str) -> Option<&'static str> {
    PROPS.iter().find(|p| p.0 == tag).map(|p| p.2)
}

const PROPS: &[(&str, &str, &str, char)] = &[
    ("Name", "IDENTIFIER", "Имя", 'M'),
    ("Synonym", "SYNONYM", "Синоним", 'M'),
    ("Comment", "DESCRIPTION", "Комментарий", 'M'),
    // общий модуль
    ("Global", "GLOBAL", "Глобальный", 'M'),
    ("ClientManagedApplication", "MNGANDORDINARYCLIENT", "Клиент (управляемое приложение)", 'M'),
    ("Server", "SERVER", "Сервер", 'M'),
    ("ExternalConnection", "EXTERNALCONNECTION", "Внешнее соединение", 'M'),
    ("ClientOrdinaryApplication", "CLIENT", "Клиент (обычное приложение)", 'M'),
    ("ServerCall", "SERVERCALL", "Вызов сервера", 'M'),
    ("Privileged", "PRIVILEGED", "Привилегированный", 'M'),
    ("ReturnValuesReuse", "RETURNVALUEREUSING", "Повторное использование возвращаемых значений", 'M'),
    ("FormType", "FORM_TYPE", "Тип формы", 'M'),
    ("TemplateType", "TEMPLATE_TYPE", "Тип макета", 'M'),
    ("LanguageCode", "LANGUAGECODE", "Код языка", 'M'),
    ("Namespace", "NAMESPACEURI", "URI пространства имен", 'M'),
    ("RegisterType", "ACCUMREG_KIND", "Вид регистра", 'M'),
    ("Value", "STYLEENTRYVALUE", "Значение", 'M'),
    // данные
    ("Hierarchical", "HIERARCHY", "Иерархический", 'D'),
    ("HierarchyType", "HIERARCHY_KIND", "Вид иерархии", 'D'),
    ("LimitLevelCount", "LIMIT_LEVELS_COUNT", "Ограничивать кол-во уровней", 'D'),
    ("LevelCount", "LEVELSLIMIT", "Количество уровней", 'D'),
    ("FoldersOnTop", "FOLDERSFIRST", "Размещать группы сверху", 'D'),
    ("Owners", "OWNERS", "Владельцы", 'D'),
    ("SubordinationUse", "SUBORDINATION_USING", "Использование подчинения", 'D'),
    ("CodeLength", "CODELENGTH", "Длина кода", 'D'),
    ("DescriptionLength", "DESCRIPTIONLEN", "Длина наименования", 'D'),
    ("CodeType", "CODETYPE", "Тип кода", 'D'),
    ("CodeAllowedLength", "CODEALLOWEDLENGTH", "Допустимая длина кода", 'D'),
    ("CodeSeries", "CODESERIES", "Серии кодов", 'D'),
    ("CheckUnique", "CHECKUNIQUE", "Контроль уникальности", 'D'),
    ("Autonumbering", "CODEAUTO", "Автонумерация", 'D'),
    ("NumberType", "DOC_NUMBERTYPE", "Тип номера", 'D'),
    ("NumberLength", "DOC_NUMBERLEN", "Длина номера", 'D'),
    ("NumberAllowedLength", "NUMBERALLOWEDLENGTH", "Допустимая длина номера", 'D'),
    ("NumberPeriodicity", "DOC_NUMBERPERIOD", "Периодичность", 'D'),
    ("Numerator", "DOC_NUMERATOR", "Нумератор", 'D'),
    ("StandardAttributes", "STANDARDPROPERTIES", "Стандартные реквизиты", 'D'),
    ("StandardTabularSections", "STANDARDTABULARPARTS", "Стандартные табличные части", 'D'),
    ("Characteristics", "CHARACTERISTICDESCRIPTIONS", "Характеристики", 'D'),
    // не свойства описания: предопределенные данные (Ext/Predefined.xml) и дополнительные индексы 8.5 — строки «Открыть»
    ("PredefinedData", "PREDEFINEDDATA", "Предопределенные", 'D'),
    ("AdditionalIndexes", "ADDITIONALINDEXES", "Дополнительные индексы", 'D'),
    ("CharacteristicExtValues", "CHARACTERISTIC_VALUES", "Дополнительные значения характеристик", 'D'),
    ("PredefinedDataUpdate", "UPDATE_PREDEFINED_DATA_MODE", "Обновление предопределенных данных", 'D'),
    // «Тип» — в «Основных» после имени, синонима, комментария (палитра реквизита и константы Конфигуратора 8.5)
    ("Type", "TYPE", "Тип", 'M'),
    ("DataLockControlMode", "TRANSACTIONLOCKMODE", "Режим управления блокировкой данных", 'D'),
    ("DataLockFields", "FIELDSFORLOCKING", "Поля блокировки данных", 'D'),
    ("InformationRegisterPeriodicity", "INFOREG_PERIOD", "Периодичность", 'D'),
    ("WriteMode", "RECORDING", "Режим записи", 'D'),
    ("MainFilterOnPeriod", "INFOREG_MAIN_FILTER_ON_PERIOD", "Основной отбор по периоду", 'D'),
    ("EnableTotalsSliceFirst", "INFOREG_ENABLE_TOTALS_SLICE_FIRST", "Разрешить итоги: срез первых", 'D'),
    ("EnableTotalsSliceLast", "INFOREG_ENABLE_TOTALS_SLICE_LAST", "Разрешить итоги: срез последних", 'D'),
    ("EnableTotalsSplitting", "ENABLETOTALSSPLITTING", "Разрешить разделение итогов", 'D'),
    ("ChartOfAccounts", "CHARTOFACCOUNT", "План счетов", 'D'),
    ("Correspondence", "CORRESPOND", "Корреспонденция", 'D'),
    ("PeriodAdjustmentLength", "PERIOD_ADJUSTMENT_LENGTH", "Длина уточнения периода", 'D'),
    ("ExtDimensionTypes", "EXTDIMENSIONSKINDS", "Виды субконто", 'D'),
    ("MaxExtDimensionCount", "MAXEXTDIMENSIONSCOUNT", "Макс. количество субконто", 'D'),
    ("CodeMask", "ACCOUNTCODEMASK", "Маска кода", 'D'),
    ("AutoOrderByCode", "AUTOORDERBYCODE", "Автопорядок по коду", 'D'),
    ("OrderLength", "ORDERLEN", "Длина порядка", 'D'),
    ("DependenceOnCalculationTypes", "BASEONCR", "Зависимость от базы", 'D'),
    ("BaseCalculationTypes", "BASICCALCKINDS", "Базовые планы видов расчета", 'D'),
    ("ActionPeriodUse", "APUSE", "Использует период действия", 'D'),
    ("Content", "CONTENT", "Состав", 'D'),
    ("Documents", "SEQ_DOCUMENTS", "Документы, входящие в последовательность", 'D'),
    ("RegisteredDocuments", "DOC_REGISTEREDDOCUMENTS", "Регистрируемые документы", 'D'),
    ("RegisterRecords", "SUBORDINATE_RECORDS", "Движения", 'D'),
    ("DistributedInfoBase", "DISTRIBUTED_IB", "Распределенная ИБ", 'D'),
    ("IncludeConfigurationExtensions", "DISTRIBUTED_EXTENSIONS", "Включать расширения конфигурации", 'D'),
    ("Location", "FUNCTIONAL_OPTION_LOCATION", "Хранение", 'D'),
    ("PrivilegedGetMode", "FUNCTIONAL_OPTION_PRIVILEGEDGETMODE", "Привилегированный режим при получении", 'D'),
    ("Source", "EVENTSOURCE", "Источник", 'D'),
    ("Event", "EVENTNAME", "Событие", 'D'),
    ("Handler", "EVENTHANDLER", "Обработчик", 'D'),
    ("MethodName", "METHODNAME", "Имя метода", 'D'),
    ("Description", "NAME", "Наименование", 'D'),
    ("Key", "KEY", "Ключ", 'D'),
    ("Predefined", "PREDEFINED", "Предопределенное", 'D'),
    ("RestartCountOnFailure", "RESTARTCOUNT", "Количество повторов при аварийном завершении", 'D'),
    ("RestartIntervalOnFailure", "RESTARTPERIOD", "Интервал повтора при аварийном завершении", 'D'),
    ("Task", "TASK", "Задачи", 'D'),
    ("TaskNumberAutoPrefix", "BPTASK_AUTOPREFIX", "Авто префикс", 'D'),
    ("Addressing", "BPTASK_ADDRESSING", "Адресация", 'D'),
    ("MainAddressingAttribute", "BPTASK_MAINPROPADDR", "Основной реквизит адресации", 'D'),
    ("CurrentPerformer", "BPTASK_SESSION_PARAMETER_CURRENT_EXECUTOR", "Текущий исполнитель", 'D'),
    ("MainDataCompositionSchema", "REPORT_COMPOSITION_SCHEME", "Основная схема компоновки данных", 'D'),
    ("VariantsStorage", "REPORT_VARIANTS_STORAGE", "Хранилище вариантов", 'D'),
    ("SettingsStorage", "REPORT_SELECTED_SETTINGS_STORAGE", "Хранилище настроек", 'D'),
    ("XDTOPackages", "XDTOPACKAGES", "Пакеты XDTO", 'D'),
    ("DescriptorFileName", "DESCRIPTORFILENAME", "Имя файла публикации", 'D'),
    ("RootURL", "HTTPSERVICE_SERVICE_ROOTURL", "Корневой URL", 'D'),
    ("LocationURL", "LOCATIONURL", "URL источника", 'D'),
    ("ReuseSessions", "REUSESEANCESMODE", "Повторное использование сеансов", 'D'),
    ("SessionMaxAge", "SEANCEMAXAGE", "Время жизни сеанса", 'D'),
    ("Indexing", "INDEXING", "Индексировать", 'D'),
    ("AutoUse", "INNCLUDEINNEW", "Автоиспользование", 'D'),
    ("DataSeparation", "DATASEPARATION", "Разделение данных", 'D'),
    ("SeparatedDataUse", "SDU", "Использование разделяемых данных", 'D'),
    ("DataSeparationValue", "SEPARATIONVALUE", "Значение разделения данных", 'D'),
    ("DataSeparationUse", "SEPARATIONUSE", "Использование разделения данных", 'D'),
    ("ConditionalSeparation", "CONDITIONALSEPARATION", "Условное разделение", 'D'),
    ("UsersSeparation", "USERSEPARATION", "Разделение пользователей", 'D'),
    ("AuthenticationSeparation", "AUTHENTICATIONSEPARATION", "Разделение аутентификации", 'D'),
    ("ConfigurationExtensionsSeparation", "EXTENSIONSEPARATION", "Разделение расширений конфигурации", 'D'),
    ("FillChecking", "FILLCHECK", "Проверка заполнения", 'D'),
    ("FillFromFillingValue", "FILLFROMFILLINGVALUE", "Заполнять из данных заполнения", 'D'),
    ("FillValue", "FILLVALUE", "Значение заполнения", 'D'),
    ("MinValue", "MINVALUE", "Минимальное значение", 'D'),
    ("MaxValue", "MAXVALUE", "Максимальное значение", 'D'),
    ("CommandParameterType", "COMMANDPARAMETERTYPE", "Тип параметра команды", 'D'),
    ("ParameterUseMode", "PARAMETERUSEGEMODE", "Режим использования параметра", 'D'),
    ("ModifiesData", "COMMANDMODIFIESDATA", "Изменяет данные", 'D'),
    ("Group", "INCLUDEINCOMMANDGROUP", "Группа", 'D'),
    ("Category", "COMMANDCATEGORY", "Категория", 'D'),
    ("AvailabilityForChoice", "ENABLEDFORCHOOSE", "Доступность для выбора", 'D'),
    ("AvailabilityForAppearance", "ENABLEDFORAPPEARANCE", "Доступность для оформления", 'D'),
    // использование (по снимку Конфигуратора у справочника стандартные команды, ввод и выбор — в «Представлении»)
    ("UseStandardCommands", "STANDARDINCLUSIONINCOMMANDINTERFACE", "Использовать стандартные команды", 'P'),
    ("IncludeInCommandInterface", "INCLUDESUBSYSTEMINGLOBALINTERFACE", "Включать в командный интерфейс", 'U'),
    ("UseOneCommand", "ONECOMMANDSUBSYSTEM", "Подсистема с одной командой", 'U'),
    ("Use", "USE", "Использование", 'U'),
    ("Posting", "POSTING", "Проведение", 'U'),
    ("RealTimePosting", "OPERATIONALPOSTING", "Оперативное проведение", 'U'),
    ("RegisterRecordsDeletion", "DELETIONACTIONS", "Удаление движений", 'U'),
    ("RegisterRecordsWritingOnPost", "ACTIONSWRITINGONPOST", "Запись движений при проведении", 'U'),
    ("SequenceFilling", "DOC_SEQUENCE_FILLING", "Заполнение последовательностей", 'U'),
    ("PostInPrivilegedMode", "DOC_PRIVILAGEPOST", "Прив. режим при проведении", 'U'),
    ("UnpostInPrivilegedMode", "DOC_PRIVILAGEUNPOST", "Прив. режим при отмене проведения", 'U'),
    ("CreateTaskInPrivilegedMode", "TASKCREATINGPRIVILAGEDMODE", "Прив. режим при создании задач", 'U'),
    ("MoveBoundaryOnPosting", "MOVE_BOUNDARY_ON_POSTING", "Перемещение границы при проведении", 'U'),
    ("BasedOn", "DERIVEDFROM", "Вводится на основании", 'D'),
    ("CreateOnInput", "CREATEONINPUT", "Создание при вводе", 'P'),
    ("ChoiceHistoryOnInput", "CHOICE_HISTORY_ON_INPUT", "История выбора при вводе", 'P'),
    ("QuickChoice", "QUICKCHOICE", "Быстрый выбор", 'P'),
    ("ChoiceMode", "CHOICEMODE", "Способ выбора", 'P'),
    ("EditType", "EDITTYPE", "Способ редактирования", 'P'),
    ("InputByString", "FIELDSOFINPUT", "Ввод по строке", 'P'),
    ("SearchStringModeOnInputByString", "INPUTBYSTRINGMODE", "Способ поиска строки при вводе по строке", 'P'),
    ("FullTextSearchOnInputByString", "FULLTEXTSEARCHONINPUTBYSTRING", "Полнотекстовый поиск при вводе по строке", 'P'),
    ("ChoiceDataGetModeOnInputByString", "INPUTBYSTRINGRUNMODE", "Режим получения данных выбора при вводе по строке", 'P'),
    ("FullTextSearch", "FULLTEXTSEARCH", "Полнотекстовый поиск", 'D'),
    ("DataHistory", "DATAHISTORY", "История данных", 'D'),
    ("UpdateDataHistoryImmediatelyAfterWrite", "UPDATE_DATA_HISTORY_IMMEDIATELY_AFTER_WRITE", "Обновлять историю данных сразу после записи", 'D'),
    ("ExecuteAfterWriteDataHistoryVersionProcessing", "EXECUTE_AFTER_WRITE_DATA_HISTORY_VERSION_PROCESSING", "Выполнять обработку после записи версии истории данных", 'D'),
    ("UsePurposes", "CONFIG_PURPOSES", "Назначения использования", 'U'),
    ("Shortcut", "COMMANDSHORTCUT", "Сочетание клавиш", 'U'),
    ("OnMainServerUnavalableBehavior", "MAINSERVERINACCESSIBILITYBEHAVIOR", "Поведение при недоступности основного сервера", 'U'),
    // представление
    ("ObjectPresentation", "OBJECTPRESENTATION", "Представление объекта", 'P'),
    ("ExtendedObjectPresentation", "EXTENDEDITEMPRESENTATION", "Расширенное представление объекта", 'P'),
    ("ListPresentation", "LISTPRESENTATION", "Представление списка", 'P'),
    ("ExtendedListPresentation", "EXTENDEDLISTPRESENTATION", "Расширенное представление списка", 'P'),
    ("RecordPresentation", "RECORDPRESENTATION", "Представление записи", 'P'),
    ("ExtendedRecordPresentation", "EXTENDEDRECORDPRESENTATION", "Расширенное представление записи", 'P'),
    ("ExtendedPresentation", "EXTENDEDPRESENTATION", "Расширенное представление", 'P'),
    ("Explanation", "EXPLANATION", "Пояснение", 'P'),
    ("DefaultPresentation", "REFERENCEMAINPRESENT", "Основное представление", 'P'),
    ("Picture", "PICTURE", "Картинка", 'P'),
    ("Representation", "COMMANDDISPLAY", "Отображение", 'P'),
    ("ToolTip", "TOOLTIP", "Подсказка", 'P'),
    ("DefaultForm", "DEFAULTFORM", "Основная форма", 'P'),
    ("AuxiliaryForm", "DEFAULTADDITIONALFORM", "Дополнительная форма", 'P'),
    ("DefaultObjectForm", "MAINITEMFORM", "Основная форма объекта", 'P'),
    ("AuxiliaryObjectForm", "MAINITEMADDITIONALFORM", "Дополнительная форма объекта", 'P'),
    ("DefaultFolderForm", "MAINGROUPFORM", "Основная форма группы", 'P'),
    ("AuxiliaryFolderForm", "MAINGROUPADDITIONALFORM", "Дополнительная форма группы", 'P'),
    ("DefaultListForm", "MAINLISTFORM", "Основная форма списка", 'P'),
    ("AuxiliaryListForm", "MAINLISTADDITIONALFORM", "Дополнительная форма списка", 'P'),
    ("DefaultChoiceForm", "MAINCHOICEFORM", "Основная форма выбора", 'P'),
    ("AuxiliaryChoiceForm", "MAINCHOICEADDITIONALFORM", "Дополнительная форма выбора", 'P'),
    ("DefaultFolderChoiceForm", "MAINGROUPCHOICEFORM", "Основная форма выб. группы", 'P'),
    ("AuxiliaryFolderChoiceForm", "MAINGROUPCHOICEADDITIONALFORM", "Дополнительная форма выб. группы", 'P'),
    ("DefaultRecordForm", "INFOREGRCRDMAINFORM", "Форма редакт. записи", 'P'),
    ("AuxiliaryRecordForm", "INFOREGRCRDMAINADDITIONALFORM", "Дополнительная форма редакт. записи", 'P'),
    ("DefaultSettingsForm", "REPORT_SETTINGS_MAIN_FORM", "Основная форма настроек", 'P'),
    ("AuxiliarySettingsForm", "REPORT_SETTINGS_ADDMAIN_FORM", "Дополнительная форма настроек", 'P'),
    ("DefaultVariantForm", "REPORT_VARIANT_EDIT_MAIN_FORM", "Основная форма варианта", 'P'),
    ("AuxiliaryVariantForm", "REPORT_VARIANT_EDIT_ADDMAIN_FORM", "Дополнительная форма варианта", 'P'),
    ("DefaultSaveForm", "SETTINGSSTORAGE_DEFAULTSAVEFORM", "Основная форма сохранения", 'P'),
    ("DefaultLoadForm", "SETTINGSSTORAGE_DEFAULTLOADFORM", "Основная форма загрузки", 'P'),
    ("AuxiliarySaveForm", "SETTINGSSTORAGE_DEFAULTSAVEADDFORM", "Дополнительная форма сохранения", 'P'),
    ("AuxiliaryLoadForm", "SETTINGSSTORAGE_DEFAULTLOADADDFORM", "Дополнительная форма загрузки", 'P'),
    ("PasswordMode", "PASSWORDSTYLE", "Режим пароля", 'P'),
    ("Format", "FORMAT", "Формат", 'P'),
    ("EditFormat", "FORMATOFINPUT", "Формат редактирования", 'P'),
    ("MarkNegatives", "MARKNEGATIVE", "Выделять отрицательные", 'P'),
    ("Mask", "MASK", "Маска", 'P'),
    ("MultiLine", "MULTILINE", "Многострочный режим", 'P'),
    ("ExtendedEdit", "EXTEDIT", "Расширенное редактирование", 'P'),
    ("ChoiceFoldersAndItems", "CHOICEFOLDERSANDITEMS", "Выбор групп и элементов", 'P'),
    ("ChoiceParameterLinks", "CHOICEPARAMETERLINKS", "Связи параметров выбора", 'P'),
    ("ChoiceParameters", "CHOICEPARAMETERS", "Параметры выбора", 'P'),
    ("ChoiceForm", "SELECTIONFORM", "Форма выбора", 'P'),
    ("LinkByType", "TYPELINK", "Связь по типу", 'P'),
    // справка, совместимость
    ("IncludeHelpInContents", "INCLUDEHELP", "Включать в содержание справки", 'H'),
    ("UseInInterfaceCompatibilityMode", "FORM_PREFERABLE_INTERFACE", "Режим совместимости интерфейса формы", 'C'),
];

/// Значения перечислений: (свойство или пусто — для любого, значение XML, ключ ресурса, представление).
const VALUES: &[(&str, &str, &str, &str)] = &[
    ("", "HierarchyFoldersAndItems", "IDS_HIERARCHY_FOLDERS_AND_ITEMS_MDENUMVAL", "Иерархия групп и элементов"),
    ("", "HierarchyOfItems", "IDS_HIERARCHY_ITEMS_MDENUMVAL", "Иерархия элементов"),
    ("", "ToItems", "IDS_SUBORDINATION_USAGE_TO_ITEMS_MDENUMVAL", "Элементам"),
    ("", "ToFolders", "IDS_SUBORDINATION_USAGE_TO_FOLDERS_MDENUMVAL", "Группам"),
    ("", "ToFoldersAndItems", "IDS_SUBORDINATION_USAGE_TO_FOLDERS_AND_ITEMS_MDENUMVAL", "Группам и элементам"),
    ("", "String", "IDS_TEXT_MDENUMVAL", "Строка"),
    ("", "Number", "IDS_NUMERIC_MDENUMVAL", "Число"),
    ("", "Fixed", "", "Фиксированная"),
    ("", "Variable", "", "Переменная"),
    ("", "WholeCatalog", "IDS_WHOLEREFERENCE_MDENUMVAL", "Во всем справочнике"),
    ("", "WithinSubordination", "IDS_WITINSUBORDINATION_MDENUMVAL", "В пределах подчинения"),
    ("", "WithinOwnerSubordination", "IDS_WITINOWNERSUBORDINATION_MDENUMVAL", "В пределах подчинения владельцу"),
    ("", "AsCode", "IDS_ASCODE_MDENUMVAL", "В виде кода"),
    ("", "AsDescription", "IDS_ASDESCRIPTION_MDENUMVAL", "В виде наименования"),
    ("", "AsNumber", "IDS_ASNAME_MDENUMVAL", "В виде номера"),
    ("", "InList", "IDS_INLIST_MDENUMVAL", "В списке"),
    ("", "InDialog", "IDS_INDIALOG_MDENUMVAL", "В диалоге"),
    ("", "BothWays", "IDS_BOTHWAYS_MDENUMVAL", "Обоими способами"),
    ("", "FromForm", "IDS_INFORMCHOICEMODE_MDENUMVAL", "Из формы"),
    ("ChoiceMode", "QuickChoice", "IDS_FASTCHOICEMODE_MDENUMVAL", "Быстрый выбор"),
    ("", "Automatic", "IDS_TRANSACTIONLOCKMODE_AUTOMATIC_MDENUMVAL", "Автоматический"),
    ("", "Managed", "IDS_TRANSACTIONLOCKMODE_CONTROLLED_MDENUMVAL", "Управляемый"),
    ("", "AutomaticAndManaged", "IDS_TRANSACTIONLOCKMODE_AUTOMATICANDCONTROLLED_MDENUMVAL", "Автоматический и управляемый"),
    ("FormType", "Managed", "", "Управляемая"),
    ("FormType", "Ordinary", "", "Обычная"),
    ("Representation", "Auto", "IDS_DISPLAYAUTO_MDENUMVAL", "Авто"),
    ("Representation", "Text", "IDS_DISPLAYLABEL_MDENUMVAL", "Надпись"),
    ("Representation", "Picture", "IDS_DISPLAYPICTURE_MDENUMVAL", "Картинка"),
    ("Representation", "PictureAndText", "IDS_DISPLAYLABELANDPICTURE_MDENUMVAL", "Надпись и картинка"),
    ("Type", "Color", "IDS_STYLEENTRYKIND_COLOR_MDENUMVAL", "Цвет"),
    ("Type", "Font", "IDS_STYLEENTRYKIND_FONT_MDENUMVAL", "Шрифт"),
    ("Type", "Border", "IDS_STYLEENTRYKIND_BORDER_MDENUMVAL", "Рамка"),
    ("", "Day", "IDS_CRDAY_MDENUMVAL", "День"),
    ("", "Month", "IDS_CRMONTH_MDENUMVAL", "Месяц"),
    ("", "Quarter", "IDS_CRQUARTER_MDENUMVAL", "Квартал"),
    ("", "Year", "IDS_CRYEAR_MDENUMVAL", "Год"),
    ("", "Nonperiodical", "IDS_CRNONEPERIODICITY_MDENUMVAL", "Непериодический"),
    ("", "ForItem", "IDS_FORITEM_MDENUMVAL", "Для элемента"),
    ("", "ForFolder", "IDS_FORFOLDER_MDENUMVAL", "Для группы"),
    ("", "ForFolderAndItem", "IDS_FORBOTH_MDENUMVAL", "Для группы и элемента"),
    ("ReturnValuesReuse", "DontUse", "IDS_RVR_MDENUM_NOTUSE", "Не использовать"),
    ("ReturnValuesReuse", "DuringRequest", "IDS_RVR_MDENUM_DURINGCALL", "На время вызова"),
    ("ReturnValuesReuse", "DuringSession", "IDS_RVR_MDENUM_DURINGSESSION", "На время сеанса"),
    ("PredefinedDataUpdate", "Auto", "IDS_UPDATE_PREDEFINED_DATA_MODE_MDENUM_AUTO", "Авто"),
    ("PredefinedDataUpdate", "AutoUpdate", "IDS_UPDATE_PREDEFINED_DATA_MODE_MDENUM_ON", "Обновлять автоматически"),
    ("PredefinedDataUpdate", "DontAutoUpdate", "IDS_UPDATE_PREDEFINED_DATA_MODE_MDENUM_OFF", "Не обновлять автоматически"),
    ("Posting", "Allow", "IDS_POSTING_MDENUM_ENABLE", "Разрешить"),
    ("Posting", "Deny", "IDS_POSTING_MDENUM_DISABLE", "Запретить"),
    ("RealTimePosting", "Allow", "IDS_OPERATIONALPOSTING_MDENUM_ENABLE", "Разрешить"),
    ("RealTimePosting", "Deny", "IDS_OPERATIONALPOSTING_MDENUM_DISABLE", "Запретить"),
    ("RegisterRecordsDeletion", "AutoDelete", "IDS_DELETEACTIONS_MDENUM_AUTO", "Удалять автоматически"),
    ("RegisterRecordsDeletion", "AutoDeleteOnUnpost", "IDS_DELETEACTIONS_MDENUM_AUTOONUNPOST", "Удалять автоматически при отмене проведения"),
    ("RegisterRecordsDeletion", "AutoDeleteOff", "IDS_DELETEACTIONS_MDENUM_DONOTAUTO", "Не удалять автоматически"),
    ("RegisterRecordsWritingOnPost", "WriteModified", "IDS_ACTIONSWRITING_MDENUM_MODIFIED", "Записывать модифицированные"),
    ("RegisterRecordsWritingOnPost", "WriteSelected", "IDS_ACTIONSWRITING_MDENUM_SELECTED", "Записывать выбранные"),
    ("SequenceFilling", "AutoFill", "IDS_FILL_MDENUM_AUTO", "Заполнять автоматически"),
    ("SequenceFilling", "AutoFillOff", "IDS_FILL_MDENUM_DONOTAUTO", "Не заполнять автоматически"),
    ("WriteMode", "Independent", "IDS_RECORDING_MDENUM_INDEPENDENT", "Независимый"),
    ("WriteMode", "RecorderSubordinate", "IDS_RECORDING_MDENUM_SUBORDINATE", "Подчинение регистратору"),
    ("FullTextSearch", "Use", "IDS_USING_FULLTEXTSEARCH_MDENUM_USE", "Использовать"),
    ("FullTextSearch", "DontUse", "IDS_USING_FULLTEXTSEARCH_MDENUM_NOTUSE", "Не использовать"),
    ("DataHistory", "Use", "IDS_USING_DATAHISTORY_MDENUM_USE", "Использовать"),
    ("DataHistory", "DontUse", "IDS_USING_DATAHISTORY_MDENUM_NOTUSE", "Не использовать"),
    ("FillChecking", "DontCheck", "IDS_FILLCHECK_MDENUM_DONOTCHECK", "Не проверять"),
    ("FillChecking", "ShowError", "IDS_FILLCHECK_MDENUM_SHOW_ERROR", "Выдавать ошибку"),
    ("Indexing", "DontIndex", "IDS_INDEXING_MDENUM_NOTINDEX", "Не индексировать"),
    ("Indexing", "Index", "IDS_INDEXING_MDENUM_INDEX", "Индексировать"),
    ("Indexing", "IndexWithAdditionalOrder", "IDS_INDEXING_MDENUM_INDEXWITHADDORDER", "Индексировать с доп. упорядочиванием"),
    ("ReuseSessions", "DontUse", "IDS_SEANCEREUSEMODE_MDENUM_DONTUSE", "Не использовать"),
    ("ReuseSessions", "Use", "IDS_SEANCEREUSEMODE_MDENUM_USE", "Использовать"),
    ("ReuseSessions", "AutoUse", "IDS_SEANCEREUSEMODE_MDENUM_AUTOUSE", "Использовать автоматически"),
    ("Category", "NavigationPanel", "IDS_COMMANDGROUPCATEGORYNAVIGATOR_MDENUMVAL", "Панель навигации"),
    ("Category", "FormNavigationPanel", "IDS_COMMANDGROUPCATEGORYNAVIGATOROFFORM_MDENUMVAL", "Панель навигации формы"),
    ("Category", "ActionsPanel", "IDS_COMMANDGROUPCATEGORYACTIONPANEL_MDENUMVAL", "Панель действий"),
    ("Category", "FormCommandBar", "IDS_COMMANDGROUPCATEGORYFORMCOMMANDPANEL_MDENUMVAL", "Командная панель формы"),
    ("UseInInterfaceCompatibilityMode", "Any", "", "Любой"),
    ("UseInInterfaceCompatibilityMode", "Taxi", "IDS_INTERFACECOMPATIBILITY_MODE_VERSION_8_3_MDENUMVAL", "Такси"),
    ("UseInInterfaceCompatibilityMode", "Version85", "IDS_INTERFACECOMPATIBILITY_MODE_VERSION_8_5_MDENUMVAL", "Версия 8.5"),
    ("RegisterType", "Balance", "IDS_ACCREG_KIND_BALANCE_MDENUMVAL", "Остатки"),
    ("RegisterType", "Turnovers", "IDS_ACCREG_KIND_TURNOVERS_MDENUMVAL", "Обороты"),
    ("MoveBoundaryOnPosting", "Move", "IDS_MOVE_BOUNDARY_ON_POSTING_MDENUM_MOVE", "Перемещать"),
    ("MoveBoundaryOnPosting", "DontMove", "IDS_MOVE_BOUNDARY_ON_POSTING_MDENUM_DONOTMOVE", "Не перемещать"),
    ("SeparatedDataUse", "Independently", "IDS_SDU_MDENUM_INDEPENDENTLY", "Независимо"),
    ("SeparatedDataUse", "IndependentlyAndSimultaneously", "IDS_SDU_MDENUM_INDEPENDENTLYANDSIMULTANEOUSLY", "Независимо и совместно"),
    ("TemplateType", "SpreadsheetDocument", "", "Табличный документ"),
    ("TemplateType", "BinaryData", "", "Двоичные данные"),
    ("TemplateType", "TextDocument", "", "Текстовый документ"),
    ("TemplateType", "HTMLDocument", "", "HTML документ"),
    ("TemplateType", "DataCompositionSchema", "", "Схема компоновки данных"),
    ("TemplateType", "DataCompositionAppearanceTemplate", "", "Макет оформления компоновки данных"),
    ("TemplateType", "GraphicalSchema", "", "Графическая схема"),
    ("TemplateType", "GeographicalSchema", "", "Географическая схема"),
    ("TemplateType", "ActiveDocument", "", "Active документ"),
    ("TemplateType", "AddIn", "", "Внешняя компонента"),
    ("OnMainServerUnavalableBehavior", "MakeDisable", "", "Сделать недоступной"),
    ("OnMainServerUnavalableBehavior", "DontChangeBehavior", "", "Не изменять поведение"),
    ("", "Auto", "IDS_QUICKCHOICEAUTO_MDENUMVAL", "Авто"),
    ("", "Use", "IDS_QUICKCHOICEYES_MDENUMVAL", "Использовать"),
    ("", "DontUse", "IDS_QUICKCHOICENO_MDENUMVAL", "Не использовать"),
    // как в палитре Конфигуратора (docs/ui-ref/properties.png)
    ("", "Begin", "", "Начало"),
    ("ChoiceFoldersAndItems", "Items", "", "Элементы"),
    ("ChoiceFoldersAndItems", "Folders", "", "Группы"),
    ("ChoiceFoldersAndItems", "FoldersAndItems", "", "Группы и элементы"),
    ("", "AnyPart", "", "Любая часть"),
    ("", "Directly", "", "Непосредственно"),
    ("", "Background", "", "Фоново"),
];

/// Строки платформы по ключам; без установленной платформы — пусто, подписи берутся из таблиц.
#[derive(Default)]
pub struct Captions {
    strings: HashMap<String, String>,
}

impl Captions {
    /// Статические подписи; установленная платформа не требуется.
    pub fn load() -> Captions {
        Captions::default()
    }

    /// Число строк платформы (0 — платформы нет).
    pub fn platform_strings(&self) -> usize {
        self.strings.len()
    }

    fn res(&self, key: &str) -> Option<&str> {
        self.strings.get(key).map(String::as_str).filter(|s| !s.is_empty())
    }

    /// Подпись и подсказка свойства; свойства нет в таблице — имя XML.
    pub fn prop(&self, tag: &str) -> (String, String) {
        let Some((_, key, ru, _)) = PROPS.iter().find(|p| p.0 == tag) else { return (tag.to_string(), String::new()) };
        match self.res(&format!("IDS_{key}_MDPROP")) {
            Some(s) => {
                let (c, t) = s.split_once('\n').unwrap_or((s, ""));
                (c.trim().to_string(), t.trim().to_string())
            }
            None => (ru.to_string(), String::new()),
        }
    }

    /// Номер категории свойства (порядок в палитре) и ее подпись.
    pub fn category(&self, tag: &str) -> (usize, String) {
        let c = PROPS.iter().find(|p| p.0 == tag).map_or('O', |p| p.3);
        let i = CATEGORIES.iter().position(|x| x.0 == c).unwrap_or(CATEGORIES.len() - 1);
        let (_, key, ru) = CATEGORIES[i];
        (i, self.res(key).unwrap_or(ru).to_string())
    }

    /// Подпись категории по номеру из [`Self::category`].
    pub fn category_title(&self, n: usize) -> String {
        let (_, key, ru) = CATEGORIES[n.min(CATEGORIES.len() - 1)];
        self.res(key).unwrap_or(ru).to_string()
    }

    /// Представление значения перечисления; нет в таблице — значение XML.
    pub fn value(&self, tag: &str, token: &str) -> String {
        let v = VALUES.iter().find(|v| v.0 == tag && v.1 == token).or_else(|| VALUES.iter().find(|v| v.0.is_empty() && v.1 == token));
        match v {
            Some((_, _, key, ru)) => self.res(key).map(|s| s.split('\n').next().unwrap_or(s).trim()).unwrap_or(ru).to_string(),
            None => token.to_string(),
        }
    }
}

/// Подгруппа свойства внутри категории: палитра Конфигуратора отделяет подгруппы пустой строкой с чертой
/// (по снимку справочника): в «Основных» — имя, синоним, комментарий и остальное; в «Данных» — иерархия
/// и остальное; в «Представлении» — команды, выбор и ввод, затем формы, затем представления и пояснение.
pub fn subgroup(tag: &str) -> u8 {
    match tag {
        "Name" | "Synonym" | "Comment" => 0,
        "Hierarchical" | "HierarchyType" | "LimitLevelCount" | "LevelCount" | "FoldersOnTop" => 0,
        t if (t.starts_with("Default") || t.starts_with("Auxiliary")) && t.ends_with("Form") => 1,
        "ObjectPresentation" | "ExtendedObjectPresentation" | "ListPresentation" | "ExtendedListPresentation" | "RecordPresentation" | "ExtendedRecordPresentation" | "ExtendedPresentation" | "Explanation" => 2,
        _ => match PROPS.iter().find(|p| p.0 == tag).map(|p| p.3) {
            Some('M') | Some('D') => 1,
            _ => 0,
        },
    }
}

/// Свойства только обычного приложения: Конфигуратор в режиме «Управляемое приложение» их не показывает
/// (на снимке справочника нет дополнительных форм и способа редактирования).
pub fn ordinary_only(tag: &str) -> bool {
    tag == "EditType" || (tag.starts_with("Auxiliary") && tag.ends_with("Form"))
}

/// Строки палитры без своего элемента в описании: (строка, свойство, перед которым она стоит; пусто — в конце
/// «Данных»), классы. Предопределенные данные — Ext/Predefined.xml, дополнительные индексы — новые в 8.5
/// (в описании формата 2.21 их нет).
pub fn pseudo(class: &str) -> Vec<(&'static str, &'static str)> {
    let mut v = Vec::new();
    if matches!(class, "Catalog" | "ChartOfAccounts" | "ChartOfCharacteristicTypes" | "ChartOfCalculationTypes") {
        v.push(("PredefinedData", "PredefinedDataUpdate"));
    }
    // shortcut: классы с дополнительными индексами — по смыслу (у кого есть таблица данных), сверено только у справочника
    if matches!(
        class,
        "Catalog" | "Document" | "ChartOfAccounts" | "ChartOfCharacteristicTypes" | "ChartOfCalculationTypes" | "ExchangePlan" | "BusinessProcess" | "Task" | "InformationRegister" | "AccumulationRegister" | "AccountingRegister" | "CalculationRegister"
    ) {
        v.push(("AdditionalIndexes", ""));
    }
    v
}

/// Подпись стандартного реквизита в списке полей (ввод по строке, поля блокировки), как у Конфигуратора.
pub fn standard_attribute(name: &str) -> &str {
    match name {
        "Code" => "Код",
        "Description" => "Наименование",
        "Number" => "Номер",
        "Date" => "Дата",
        "Owner" => "Владелец",
        "Parent" => "Родитель",
        "Ref" => "Ссылка",
        "DeletionMark" => "Пометка удаления",
        "Posted" => "Проведен",
        "IsFolder" => "Это группа",
        "Predefined" => "Предопределенный",
        "PredefinedDataName" => "Имя предопределенных данных",
        "Order" => "Порядок",
        "Type" => "Тип значения",
        "OffBalance" => "Забалансовый",
        "Executed" => "Выполнена",
        "Started" => "Стартован",
        "Completed" => "Завершен",
        "BusinessProcess" => "Бизнес-процесс",
        "RoutePoint" => "Точка маршрута",
        "ThisNode" => "Этот узел",
        "SentNo" => "Номер отправленного",
        "ReceivedNo" => "Номер принятого",
        _ => name,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tables_without_platform() {
        let c = Captions::default();
        assert_eq!(c.prop("FoldersOnTop").0, "Размещать группы сверху");
        assert_eq!(c.prop("НетТакого").0, "НетТакого");
        assert_eq!(c.category("Name"), (0, "Основные".to_string()));
        assert_eq!(c.category("НетТакого").1, "Прочее");
        assert_eq!(c.value("ChoiceMode", "QuickChoice"), "Быстрый выбор");
        assert_eq!(c.value("FormType", "Managed"), "Управляемая");
        assert_eq!(c.value("DataLockControlMode", "Managed"), "Управляемый");
        // в таблице нет повторов свойств
        let mut tags: Vec<&str> = PROPS.iter().map(|p| p.0).collect();
        tags.sort();
        tags.dedup();
        assert_eq!(tags.len(), PROPS.len());
    }

    #[test]
    fn platform_keys_exist() {
        let c = Captions::load();
        if c.platform_strings() == 0 {
            return;
        }
        // все ключи таблиц есть в ресурсах установленной платформы
        let missing: Vec<String> = PROPS.iter().map(|p| format!("IDS_{}_MDPROP", p.1)).chain(VALUES.iter().filter(|v| !v.2.is_empty()).map(|v| v.2.to_string())).filter(|k| c.res(k).is_none()).collect();
        assert!(missing.is_empty(), "{missing:?}");
        assert_eq!(c.prop("Comment").0, "Комментарий");
        assert_eq!(c.prop("CheckUnique").1, "Проводить проверку уникальности кодов");
    }
}
