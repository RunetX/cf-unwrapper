//! Ссылочные объекты (обратное `xml/classes_refs.rs`): справочники, документы. Свойства — места `$P:<Свойство>`
//! шаблона, значение берется по таблице выгрузки (`classes_refs::table`): вид (булево, перечисление, ссылка, строка…)
//! и словарь значений — оттуда, позиция — в шаблоне (тест `paths_match_export_tables` сверяет ее с таблицей).
//! Вложенные элементы: реквизиты, табличные части, команды — полностью; формы и макеты — uuid по именам.

use super::desc::Obj;
use super::dom::El;
use super::index::Index;
use super::tpl::{a, boolean, int, l, mlstring, reference, s, token, type_desc, Props};
use crate::brace::Node;
use crate::xml::classes::P;
use crate::xml::common::ZERO;

/// Тело записи и коллекции вложенных (по GUID коллекции, как в записях БП).
pub fn template(class: &str) -> Option<&'static str> {
    Some(match class {
        "Catalog" => "{1,{57,$gt:Object,$gv:Object,$gt:Ref,$gv:Ref,$gt:Selection,$gv:Selection,$gt:List,$gv:List,{0,$H},\
$P:LevelCount,$P:EditType,$mdl:Owners,$P:FoldersOnTop,$P:CheckUnique,$P:Autonumbering,$P:CodeSeries,$P:CodeLength,$P:CodeType,\
$P:DescriptionLength,$P:DefaultPresentation,$P:DefaultObjectForm,$P:DefaultFolderForm,$P:DefaultListForm,$P:DefaultChoiceForm,\
$P:DefaultFolderChoiceForm,$P:AuxiliaryObjectForm,$P:AuxiliaryFolderForm,$P:AuxiliaryListForm,$P:AuxiliaryChoiceForm,\
$P:AuxiliaryFolderChoiceForm,$P:UseStandardCommands,$mdl:BasedOn,$P:IncludeHelpInContents,$gt:Manager,$gv:Manager,$P:HierarchyType,\
$P:Hierarchical,$P:LimitLevelCount,$P:SubordinationUse,$P:ChoiceMode,$P:QuickChoice,$fields:InputByString,$P:DataLockControlMode,\
$P:FullTextSearch,$std,$P:ObjectPresentation,$P:ExtendedObjectPresentation,$P:ListPresentation,$P:ExtendedListPresentation,\
$P:Explanation,$P:CodeAllowedLength,$chars,$P:CreateOnInput,$fields:DataLockFields,$P:PredefinedDataUpdate,\
{$P:SearchStringModeOnInputByString,$P:FullTextSearchOnInputByString,$P:ChoiceDataGetModeOnInputByString},$P:ChoiceHistoryOnInput,\
$P:DataHistory,$P:UpdateDataHistoryImmediatelyAfterWrite,$P:ExecuteAfterWriteDataHistoryVersionProcessing},5,\
$coll:3daea016-69b7-4ed4-9453-127911372fe6:Template,$cmds:4fe87c89-9ad4-43f6-9fdb-9dc83b3879c6,\
$ts:932159f9-95b2-4e76-a8dd-8849fe5c5ded,$attrs:cf4abea7-37b2-11d4-940f-008048da11f9,$coll:fdf816d2-1ead-11d5-b975-0050bae0a95d:Form}",
        "Document" => "{1,{40,$gt:Object,$gv:Object,$gt:Ref,$gv:Ref,$gt:Selection,$gv:Selection,$gt:List,$gv:List,{0,$H},\
$P:Numerator,$P:NumberType,$P:NumberLength,$P:NumberPeriodicity,$P:CheckUnique,$P:Autonumbering,$P:DefaultObjectForm,\
$P:DefaultListForm,$P:DefaultChoiceForm,$P:Posting,$P:RegisterRecordsDeletion,$P:RealTimePosting,$mdl:BasedOn,\
$P:UseStandardCommands,$mdl:RegisterRecords,$P:IncludeHelpInContents,$gt:Manager,$gv:Manager,$P:SequenceFilling,\
$fields:InputByString,$P:DataLockControlMode,$P:FullTextSearch,$std,$P:PostInPrivilegedMode,$P:UnpostInPrivilegedMode,\
$P:AuxiliaryObjectForm,$P:AuxiliaryListForm,$P:AuxiliaryChoiceForm,$P:ObjectPresentation,$P:ExtendedObjectPresentation,\
$P:ListPresentation,$P:ExtendedListPresentation,$P:Explanation,$P:RegisterRecordsWritingOnPost,$P:NumberAllowedLength,$chars,\
$P:CreateOnInput,$fields:DataLockFields,\
{$P:SearchStringModeOnInputByString,$P:FullTextSearchOnInputByString,$P:ChoiceDataGetModeOnInputByString},$P:ChoiceHistoryOnInput,\
$P:DataHistory,$P:UpdateDataHistoryImmediatelyAfterWrite,$P:ExecuteAfterWriteDataHistoryVersionProcessing},5,\
$ts:21c53e09-8950-4b5e-a6a0-1054f1bbc274,$coll:3daea016-69b7-4ed4-9453-127911372fe6:Template,\
$attrs:45e46cbc-3e24-4165-8b7b-cc98a6f80211,$cmds:b544fc6a-2ba3-4885-8fb2-cb289fb6d65e,$coll:fb880e93-47d7-4127-9357-a20e69c17545:Form}",
        "DataProcessor" => "{1,{17,$gt:Object,$gv:Object,{0,$H},$P:DefaultForm,$P:UseStandardCommands,$P:IncludeHelpInContents,\
$gt:Manager,$gv:Manager,$P:AuxiliaryForm,$P:ExtendedPresentation,$P:Explanation},5,$ts:2bcef0d1-0981-11d6-b9b8-0050bae0a95d,\
$coll:3daea016-69b7-4ed4-9453-127911372fe6:Template,$cmds:45556acb-826a-4f73-898a-6025fc9536e1,\
$coll:d5b0e5ed-256d-401c-9c36-f630cafd8a62:Form,$attrs:ec6bb5e5-b7a8-4d75-bec9-658107a699cf}",
        // корень .epf: запись `<uuid>` оборачивает тело; заголовок — по ObjectId (`$uuid` — uuid описания)
        "ExternalDataProcessor" => "{1,{$uuid},1,{c3831ec8-d8d5-4f93-8a22-f9bfae07327f,{1,{4,$gt:Object,$gv:Object,{0,$H},$P:DefaultForm,\"\",$P:AuxiliaryForm},4,$ts:2bcef0d1-0981-11d6-b9b8-0050bae0a95d,$coll:3daea016-69b7-4ed4-9453-127911372fe6:Template,$coll:d5b0e5ed-256d-401c-9c36-f630cafd8a62:Form,$attrs:ec6bb5e5-b7a8-4d75-bec9-658107a699cf}}}",
        // AuxiliaryForm, AuxiliarySettingsForm, SettingsStorage — в одной из позиций 9, 14, 17 (какая где, не
        // установлено: в БП и 8.5 все пусты) — загружаются только пустыми
        "Report" => "{1,{20,$gt:Object,$gv:Object,{0,$H},$P:DefaultForm,$P:MainDataCompositionSchema,$P:DefaultSettingsForm,\
$P:UseStandardCommands,$P:VariantsStorage,$P:SettingsStorage,$P:DefaultVariantForm,$P:IncludeHelpInContents,$gt:Manager,$gv:Manager,\
$empty:AuxiliarySettingsForm,$P:ExtendedPresentation,$P:Explanation,$empty:AuxiliaryForm,$empty:AuxiliaryVariantForm},5,\
$coll:3daea016-69b7-4ed4-9453-127911372fe6:Template,$attrs:7e7123e0-29e2-11d6-a3c7-0050bae0a776,\
$coll:a3b368c0-29e2-11d6-a3c7-0050bae0a776:Form,$ts:b077d780-29e2-11d6-a3c7-0050bae0a776,$cmds:e7ff38c0-ec3c-47a0-ae90-20c73ca72246}",
        "ExchangePlan" => "{1,{37,$gt:Object,$gv:Object,$gt:Ref,$gv:Ref,$gt:Selection,$gv:Selection,$gt:List,$gv:List,$gt:Manager,$gv:Manager,$thisnode,$H,$P:UseStandardCommands,$P:DefaultObjectForm,$P:CodeLength,0,$P:DescriptionLength,$P:IncludeHelpInContents,$P:DefaultListForm,$P:DefaultChoiceForm,$P:EditType,$P:ChoiceMode,$P:QuickChoice,$mdl:BasedOn,$P:DefaultPresentation,$P:DistributedInfoBase,$fields:InputByString,$P:DataLockControlMode,$P:FullTextSearch,$std,$P:AuxiliaryObjectForm,$P:AuxiliaryListForm,$P:AuxiliaryChoiceForm,$P:ObjectPresentation,$P:ExtendedObjectPresentation,$P:ListPresentation,$P:ExtendedListPresentation,$P:Explanation,$P:CodeAllowedLength,$chars,$P:CreateOnInput,$fields:DataLockFields,{$P:SearchStringModeOnInputByString,$P:FullTextSearchOnInputByString,$P:ChoiceDataGetModeOnInputByString},$P:ChoiceHistoryOnInput,$P:IncludeConfigurationExtensions,$P:DataHistory,$P:UpdateDataHistoryImmediatelyAfterWrite,$P:ExecuteAfterWriteDataHistoryVersionProcessing,0,1},5,$attrs:1a1b4fea-e093-470d-94ff-1d2f16cda2ab,$coll:3daea016-69b7-4ed4-9453-127911372fe6:Template,$ts:52293f4b-f98c-43ea-a80f-41047ae7ab58,$coll:87c509ab-3d38-4d67-b379-aca796298578:Form,$cmds:d5207c64-11d5-4d46-bba2-55b7b07ff4eb}",
        "ChartOfCharacteristicTypes" => "{1,{34,$gt:Object,$gv:Object,$gt:Ref,$gv:Ref,$gt:Selection,$gv:Selection,$gt:List,$gv:List,$gt:Characteristic,$gv:Characteristic,$gt:Manager,$gv:Manager,{0,$H},$P:UseStandardCommands,$mdl:BasedOn,$P:IncludeHelpInContents,$P:CharacteristicExtValues,$P:Type,$P:Hierarchical,$P:FoldersOnTop,$P:CodeLength,$P:Autonumbering,$P:DescriptionLength,$P:DefaultPresentation,$P:EditType,$P:DefaultObjectForm,$P:DefaultFolderForm,$P:DefaultListForm,$P:DefaultChoiceForm,$P:DefaultFolderChoiceForm,$P:ChoiceMode,$P:QuickChoice,$fields:InputByString,$P:CheckUnique,$P:CodeSeries,$P:DataLockControlMode,$P:FullTextSearch,$std,$P:AuxiliaryObjectForm,$P:AuxiliaryFolderForm,$P:AuxiliaryListForm,$P:AuxiliaryChoiceForm,$P:AuxiliaryFolderChoiceForm,$P:ObjectPresentation,$P:ExtendedObjectPresentation,$P:ListPresentation,$P:ExtendedListPresentation,$P:Explanation,$P:CodeAllowedLength,$chars,$P:CreateOnInput,$fields:DataLockFields,$P:PredefinedDataUpdate,{$P:SearchStringModeOnInputByString,$P:FullTextSearchOnInputByString,$P:ChoiceDataGetModeOnInputByString},$P:ChoiceHistoryOnInput,$P:DataHistory,$P:UpdateDataHistoryImmediatelyAfterWrite,$P:ExecuteAfterWriteDataHistoryVersionProcessing},5,$attrs:31182525-9346-4595-81f8-6f91a72ebe06,$coll:3daea016-69b7-4ed4-9453-127911372fe6:Template,$ts:54e36536-7863-42fd-bea3-c5edd3122fdc,$cmds:95b5e1d4-abfa-4a16-818d-a5b07b7d3f73,$coll:eb2b78a8-40a6-4b7e-b1b3-6ca9966cbc94:Form}",
        "ChartOfAccounts" => "{1,{32,$gt:Object,$gv:Object,$gt:Ref,$gv:Ref,$gt:Selection,$gv:Selection,$gt:List,$gv:List,$gt:Manager,$gv:Manager,$gt:ExtDimensionTypes,$gv:ExtDimensionTypes,$gt:ExtDimensionTypesRow,$gv:ExtDimensionTypesRow,{0,$H},$P:UseStandardCommands,$P:IncludeHelpInContents,$mdl:BasedOn,$P:ExtDimensionTypes,$P:MaxExtDimensionCount,$P:CodeMask,$P:CodeLength,$P:DescriptionLength,$P:AutoOrderByCode,$P:OrderLength,$P:DefaultPresentation,$P:EditType,$P:DefaultObjectForm,$P:DefaultListForm,$P:DefaultChoiceForm,$P:ChoiceMode,$P:QuickChoice,$fields:InputByString,$P:CheckUnique,$P:CodeSeries,$P:DataLockControlMode,$P:FullTextSearch,$std,$stdts,$P:AuxiliaryObjectForm,$P:AuxiliaryListForm,$P:AuxiliaryChoiceForm,$P:ObjectPresentation,$P:ExtendedObjectPresentation,$P:ListPresentation,$P:ExtendedListPresentation,$P:Explanation,$chars,$P:CreateOnInput,$fields:DataLockFields,$P:PredefinedDataUpdate,{$P:SearchStringModeOnInputByString,$P:FullTextSearchOnInputByString,$P:ChoiceDataGetModeOnInputByString},$P:ChoiceHistoryOnInput,$P:DataHistory,$P:UpdateDataHistoryImmediatelyAfterWrite,$P:ExecuteAfterWriteDataHistoryVersionProcessing},7,$cmds:0df30176-6865-4787-9fc8-609eb144174f,$coll:3daea016-69b7-4ed4-9453-127911372fe6:Template,$ts:4c7fec95-d1bd-4508-8a01-f1db090d9af8,$coll:5372e285-03db-4f8c-8565-fe56f1aea40e:Form,$attrs:6e65cbf5-daa8-4d8d-bef8-59723f4e5777,$kattrs:78bd1243-c4df-46c3-8138-e147465cb9a4:AccountingFlag,$kattrs:c70ca527-5042-4cad-a315-dcb4007e32a3:ExtDimensionAccountingFlag}",
        "ChartOfCalculationTypes" => "{1,{35,{0,$H},$gt:Object,$gv:Object,$gt:Ref,$gv:Ref,$gt:Selection,$gv:Selection,$gt:List,$gv:List,$gt:Manager,$gv:Manager,$gt:DisplacingCalculationTypes,$gv:DisplacingCalculationTypes,$gt:DisplacingCalculationTypesRow,$gv:DisplacingCalculationTypesRow,$gt:BaseCalculationTypes,$gv:BaseCalculationTypes,$gt:BaseCalculationTypesRow,$gv:BaseCalculationTypesRow,$gt:LeadingCalculationTypes,$gv:LeadingCalculationTypes,$gt:LeadingCalculationTypesRow,$gv:LeadingCalculationTypesRow,$P:EditType,$P:CodeLength,$P:CodeType,$P:DependenceOnCalculationTypes,$mdl:BaseCalculationTypes,$P:ActionPeriodUse,$P:DescriptionLength,$P:DefaultPresentation,$P:DefaultObjectForm,$P:DefaultListForm,$P:DefaultChoiceForm,$P:UseStandardCommands,$mdl:BasedOn,$P:IncludeHelpInContents,$P:ChoiceMode,$P:QuickChoice,$fields:InputByString,$P:DataLockControlMode,$P:FullTextSearch,$std,$stdts,$P:AuxiliaryObjectForm,$P:AuxiliaryListForm,$P:AuxiliaryChoiceForm,$P:ObjectPresentation,$P:ExtendedObjectPresentation,$P:ListPresentation,$P:ExtendedListPresentation,$P:Explanation,$P:CodeAllowedLength,$chars,$P:CreateOnInput,$fields:DataLockFields,$P:PredefinedDataUpdate,{$P:SearchStringModeOnInputByString,$P:FullTextSearchOnInputByString,$P:ChoiceDataGetModeOnInputByString},$P:ChoiceHistoryOnInput,$P:DataHistory,$P:UpdateDataHistoryImmediatelyAfterWrite,$P:ExecuteAfterWriteDataHistoryVersionProcessing},5,$ts:054aa8cf-faa6-4634-aef4-1087ca0d88fc,$attrs:0dc22ad2-476a-4794-afae-cfa7ed251752,$cmds:2e90c75b-2f0c-4899-a7d4-5426eaefc96e,$coll:3daea016-69b7-4ed4-9453-127911372fe6:Template,$coll:a7f8f92a-7a4b-484b-937e-42d242e64144:Form}",
        "BusinessProcess" => "{1,{30,$H,$P:UseStandardCommands,$gt:Object,$gv:Object,$gt:Ref,$gv:Ref,$gt:Selection,$gv:Selection,$gt:List,$gv:List,$gt:Manager,$gv:Manager,$gt:RoutePointRef,$gv:RoutePointRef,$mdl:BasedOn,$P:NumberType,$P:NumberPeriodicity,$P:NumberLength,$P:CheckUnique,$P:Autonumbering,$P:EditType,$P:DefaultObjectForm,$P:DefaultListForm,$P:DefaultChoiceForm,$P:Task,$P:IncludeHelpInContents,$fields:InputByString,$P:DataLockControlMode,$P:FullTextSearch,$std,$P:CreateTaskInPrivilegedMode,$P:AuxiliaryObjectForm,$P:AuxiliaryListForm,$P:AuxiliaryChoiceForm,$P:ObjectPresentation,$P:ExtendedObjectPresentation,$P:ListPresentation,$P:ExtendedListPresentation,$P:Explanation,$P:NumberAllowedLength,$chars,$P:CreateOnInput,$fields:DataLockFields,{$P:SearchStringModeOnInputByString,$P:FullTextSearchOnInputByString,$P:ChoiceDataGetModeOnInputByString},$P:ChoiceHistoryOnInput,$P:DataHistory,$P:UpdateDataHistoryImmediatelyAfterWrite,$P:ExecuteAfterWriteDataHistoryVersionProcessing},5,$coll:3daea016-69b7-4ed4-9453-127911372fe6:Template,$coll:3f7a8120-b71a-4265-98bf-4d9bc09b7719:Form,$cmds:7a3e533c-f232-40d5-a932-6a311d2480bf,$attrs:87c988de-ecbf-413b-87b0-b9516df05e28,$ts:a3fe6537-d787-40f7-8a06-419d2f0c1cfd}",
        "Task" => "{1,{33,$H,$P:UseStandardCommands,$gt:Object,$gv:Object,$gt:Ref,$gv:Ref,$gt:Selection,$gv:Selection,$gt:List,$gv:List,$gt:Manager,$gv:Manager,00000000-0000-0000-0000-000000000000,00000000-0000-0000-0000-000000000000,$P:DefaultObjectForm,$P:DefaultListForm,$P:DefaultChoiceForm,$P:NumberType,$P:NumberLength,$P:CheckUnique,$P:Autonumbering,$P:DescriptionLength,$P:EditType,$P:IncludeHelpInContents,$P:Addressing,$P:MainAddressingAttribute,$P:DefaultPresentation,$fields:InputByString,$P:CurrentPerformer,$mdl:BasedOn,$P:TaskNumberAutoPrefix,$P:FullTextSearch,$P:DataLockControlMode,$std,$P:AuxiliaryObjectForm,$P:AuxiliaryListForm,$P:AuxiliaryChoiceForm,$P:ObjectPresentation,$P:ExtendedObjectPresentation,$P:ListPresentation,$P:ExtendedListPresentation,$P:Explanation,$P:NumberAllowedLength,$chars,$P:CreateOnInput,$fields:DataLockFields,{$P:SearchStringModeOnInputByString,$P:FullTextSearchOnInputByString,$P:ChoiceDataGetModeOnInputByString},$P:ChoiceHistoryOnInput,$P:DataHistory,$P:UpdateDataHistoryImmediatelyAfterWrite,$P:ExecuteAfterWriteDataHistoryVersionProcessing},6,$coll:3daea016-69b7-4ed4-9453-127911372fe6:Template,$coll:3f58cbfb-4172-4e54-be49-561a579bb38b:Form,$attrs:8ddfb495-c5fc-46b9-bdc5-bcf58341bff0,$kattrs:e97c0570-251c-4566-b0f1-10686820f143:AddressingAttribute,$ts:ee865d4b-a458-48a0-b38f-5a26898feeb0,$cmds:f27c2152-a2c9-4c30-adb1-130f5eb2590f}",
        "Subsystem" => "{1,{22,$H,$P:IncludeHelpInContents,{0,0},$P:IncludeInCommandInterface,$p:Picture,$P:Explanation,$mdl:Content,$P:UseOneCommand},1,$coll:37f2fa9a-b276-11d4-9435-004095e12fc7:Subsystem}",
        "DocumentJournal" => "{1,{26,$gt:List,$gv:List,{0,$H},$P:DefaultForm,$P:UseStandardCommands,$mdl:RegisteredDocuments,$P:IncludeHelpInContents,$gt:Manager,$gv:Manager,$gt:Selection,$gv:Selection,$std,$P:AuxiliaryForm,$P:ListPresentation,$P:ExtendedListPresentation,$P:Explanation},4,$coll:3daea016-69b7-4ed4-9453-127911372fe6:Template,$columns:5aee69df-0513-4c6c-9815-103102471712,$cmds:a49a35ce-120a-4c80-8eea-b0618479cd70,$coll:ec81ad10-ca07-11d5-b9a5-0050bae0a95d:Form}",
        "Sequence" => "{1,{6,$gt:Record,$gv:Record,$gt:Manager,$gv:Manager,$gt:RecordSet,$gv:RecordSet,{0,$H},$mdl:Documents,$mdl:RegisterRecords,$P:MoveBoundaryOnPosting,$P:DataLockControlMode},1,$seqdims:437488c0-35e2-11d6-a3c7-0050bae0a776}",
        "AccountingRegister" => "{1,{$*accver,$gt:Record,$gv:Record,$gt:ExtDimensions,$gv:ExtDimensions,$gt:RecordSet,$gv:RecordSet,$gt:RecordKey,$gv:RecordKey,$gt:Selection,$gv:Selection,$gt:List,$gv:List,$gt:Manager,$gv:Manager,{0,$H},$P:UseStandardCommands,$P:IncludeHelpInContents,$P:ChartOfAccounts,$P:DefaultListForm,$P:Correspondence,$P:DataLockControlMode,$P:FullTextSearch,$P:EnableTotalsSplitting,$std,$P:AuxiliaryListForm,$P:ListPresentation,$P:ExtendedListPresentation,$P:Explanation,$n:PeriodAdjustmentLength},6,$el:35b63b9d-0adf-4625-a047-10ae874c19a3:Dimension,$coll:3daea016-69b7-4ed4-9453-127911372fe6:Template,$el:63405499-7491-4ce3-ac72-43433cbe4112:Resource,$cmds:7162da60-f7fe-4d78-ad5d-e31700f9af18,$el:9d28ee33-9c7e-4a1b-8f13-50aa9b36607b:Attribute,$coll:d3b5d6eb-4ea2-4610-a3e2-624d4e815934:Form}",
        "DocumentNumerator" => "{1,{3,$H,$P:NumberType,$n:NumberLength,$P:NumberPeriodicity,$P:CheckUnique,$P:NumberAllowedLength},0}",
        "FilterCriterion" => "{1,{14,$gt:Manager,$gv:Manager,$gt:List,$gv:List,{2,$H,$P:Type},$mdl:Content,$P:UseStandardCommands,$P:DefaultForm,$P:AuxiliaryForm,$P:ListPresentation,$P:ExtendedListPresentation,$P:Explanation},2,$coll:00867c40-06b1-11d6-a3c7-0050bae0a776:Form,$cmds:23fa3b84-220a-40e9-8331-e588bed87f7d}",
        "WebService" => "{1,{4,$P:Namespace,$H,$wsrefs,$P:DescriptorFileName,$wsns,$P:ReuseSessions,$n:SessionMaxAge},1,$wsops:36186084-c23a-43bd-876c-a3a8ba1a9622}",
        "HTTPService" => "{1,{2,$P:RootURL,$H,$P:ReuseSessions,$n:SessionMaxAge},1,$urls:ec6896c2-9b28-42d8-9140-48491146b8ea}",
        "WSReference" => "{1,{2,{$P:LocationURL,0},$H,$gt:Manager,$gv:Manager},0}",
        // integ.dll: запись FUN_1800e0a60/FUN_1800f2110, коллекция каналов — класс acb7e81f (FUN_1800f2470)
        "IntegrationService" => "{1,{0,$H,$gt:Manager,$gv:Manager,$P:ExternalIntegrationServiceAddress},1,$chans:acb7e81f-0637-4ebd-88ff-954ba075ae51}",
        "InformationRegister" => "{1,{33,$gt:Record,$gv:Record,$gt:Manager,$gv:Manager,$gt:Selection,$gv:Selection,$gt:List,$gv:List,\
$gt:RecordSet,$gv:RecordSet,$gt:RecordKey,$gv:RecordKey,$gt:RecordManager,$gv:RecordManager,{0,$H},$P:DefaultRecordForm,$P:DefaultListForm,\
$P:InformationRegisterPeriodicity,$P:WriteMode,$P:EditType,$P:UseStandardCommands,$P:IncludeHelpInContents,$b:MainFilterOnPeriod,\
$P:DataLockControlMode,$P:FullTextSearch,$std,$P:AuxiliaryRecordForm,$P:AuxiliaryListForm,$P:RecordPresentation,\
$P:ExtendedRecordPresentation,$P:ListPresentation,$P:ExtendedListPresentation,$P:Explanation,$P:EnableTotalsSliceLast,\
$P:EnableTotalsSliceFirst,$P:DataHistory,$P:UpdateDataHistoryImmediatelyAfterWrite,$P:ExecuteAfterWriteDataHistoryVersionProcessing},6,\
$el:13134202-f60b-11d5-a3c7-0050bae0a776:Resource,$el:13134203-f60b-11d5-a3c7-0050bae0a776:Dimension,\
$coll:13134204-f60b-11d5-a3c7-0050bae0a776:Form,$coll:3daea016-69b7-4ed4-9453-127911372fe6:Template,\
$el:a2207540-1400-11d6-a3c7-0050bae0a776:Attribute,$cmds:b44ba719-945c-445c-8aab-1088fa4df16e}",
        "AccumulationRegister" => "{1,{28,$gt:Record,$gv:Record,$gt:Manager,$gv:Manager,$gt:Selection,$gv:Selection,$gt:List,$gv:List,\
$gt:RecordSet,$gv:RecordSet,$gt:RecordKey,$gv:RecordKey,{0,$H},$P:DefaultListForm,$P:RegisterType,$P:UseStandardCommands,\
$P:IncludeHelpInContents,$P:DataLockControlMode,$P:FullTextSearch,$P:EnableTotalsSplitting,$std,$P:AuxiliaryListForm,\
$P:ListPresentation,$P:ExtendedListPresentation,$P:Explanation},6,\
$coll:3daea016-69b7-4ed4-9453-127911372fe6:Template,$cmds:99f328af-a77f-4572-a2d8-80ed20c81890,\
$el:b64d9a41-1642-11d6-a3c7-0050bae0a776:Resource,$el:b64d9a42-1642-11d6-a3c7-0050bae0a776:Attribute,\
$el:b64d9a43-1642-11d6-a3c7-0050bae0a776:Dimension,$coll:b64d9a44-1642-11d6-a3c7-0050bae0a776:Form}",
        // регистр расчета (ERP 2.5.22 Начисления, Удержания): коллекции по возрастанию GUID; перерасчеты (274bf899) и две
        // коллекции, вид которых не сверен (a2cb086c, acdf0f11: формы и команды), — пустые: непустые в XML останутся
        // непрочитанными (отказ)
        "CalculationRegister" => "{1,{21,$gt:Record,$gv:Record,$gt:Manager,$gv:Manager,$gt:Selection,$gv:Selection,$gt:List,$gv:List,$gt:RecordSet,$gv:RecordSet,$gt:RecordKey,$gv:RecordKey,$gt:Recalcs,$gv:Recalcs,{0,$H},$P:Periodicity,$P:ActionPeriod,$P:BasePeriod,$P:Schedule,$P:ScheduleValue,$P:ScheduleDate,$P:ChartOfCalculationTypes,$P:DefaultListForm,$P:UseStandardCommands,$P:IncludeHelpInContents,$P:DataLockControlMode,$P:FullTextSearch,$std,$P:AuxiliaryListForm,$P:ListPresentation,$P:ExtendedListPresentation,$P:Explanation},7,$el:1b304502-2216-440b-960f-60decd04bb5d:Attribute,{274bf899-db0e-4df6-8ab5-67bf6371ec0b,0},$coll:3daea016-69b7-4ed4-9453-127911372fe6:Template,$el:702b33ad-843e-41aa-8064-112cd38cc92c:Resource,{a2cb086c-db98-43e4-a1a9-0760ab048f8d,0},{acdf0f11-2d59-4e37-9945-c6721871a8fe,0},$el:b12fc850-8210-43c8-ae05-89567e698fbb:Dimension}",
        _ => return None,
    })
}

/// Таблица выгрузки класса.
fn table(class: &str) -> Option<&'static [(&'static str, P)]> {
    crate::xml::classes_refs::table(class).or_else(|| crate::xml::classes_regs::tables(class).map(|t| t.0)).or_else(|| crate::xml::classes_tools::table(class))
}

/// Элемент регистра: шаблон по классу и виду; `$E:<Свойство>` — по таблице элемента выгрузки.
fn element_template(class: &str, kind: &str) -> Option<&'static str> {
    Some(match (class, kind) {
        ("AccountingRegister", "Dimension") => "{{6,$attr,$E:Balance,$E:AccountingFlag,$E:Indexing,$E:DenyIncompleteValues,$E:FullTextSearch},0}",
        ("AccountingRegister", "Resource") => "{{2,$attr,$E:Balance,$E:AccountingFlag,$E:ExtDimensionAccountingFlag,$E:FullTextSearch},0}",
        ("AccountingRegister", "Attribute") => "{{3,$attr,$E:Indexing,$E:FullTextSearch,0,{1,00000000-0000-0000-0000-000000000000}},0}",
        ("InformationRegister", "Attribute") => "{{5,$attr,$E:Indexing,$E:FullTextSearch,$E:DataHistory,$storage_use,{1,00000000-0000-0000-0000-000000000000}},0}",
        ("InformationRegister", "Resource") => "{{8,$attr,$E:Indexing,$E:FullTextSearch,$E:DataHistory,$storage_use,{1,00000000-0000-0000-0000-000000000000}},0}",
        ("InformationRegister", "Dimension") => "{{10,$attr,$E:Master,$E:DenyIncompleteValues,$E:Indexing,$E:MainFilter,$E:FullTextSearch,$E:DataHistory,$E:TypeReductionMode},0}",
        ("AccumulationRegister", "Resource") => "{{5,$attr,$E:FullTextSearch},0}",
        ("CalculationRegister", "Resource") => "{{4,$attr,$E:FullTextSearch},0}",
        ("CalculationRegister", "Attribute") => "{{3,$attr,$E:ScheduleLink,$E:Indexing,$E:FullTextSearch,0,{1,00000000-0000-0000-0000-000000000000}},0}",
        ("CalculationRegister", "Dimension") => "{{5,$attr,$E:DenyIncompleteValues,$E:BaseDimension,$E:ScheduleLink,$E:Indexing,$E:FullTextSearch},0}",
        ("AccumulationRegister", "Attribute") => "{{4,$attr,$E:Indexing,$E:FullTextSearch,0,{1,00000000-0000-0000-0000-000000000000}},0}",
        ("AccumulationRegister", "Dimension") => "{{8,$attr,$E:DenyIncompleteValues,$E:Indexing,$E:FullTextSearch,$E:UseInTotals},0}",
        _ => return None,
    })
}

/// Элементы регистра вида `kind` коллекции `guid`.
fn elements(x: &Index, o: &Obj, guid: &str, kind: &str) -> Result<Node, String> {
    let tpl = element_template(o.class, kind).ok_or(format!("{kind} класса {}: загрузка не сделана", o.class))?;
    let (_, kinds) = crate::xml::classes_regs::tables(o.class).ok_or(format!("нет таблиц класса {}", o.class))?;
    let (_, _, et) = kinds.iter().find(|(k, _, _)| *k == kind).ok_or(format!("нет таблицы {kind}"))?;
    let mut items = Vec::new();
    for e in o.children(kind) {
        let id = e.req_attr("uuid")?;
        let p = Props { el: e.req("Properties")?, x, id };
        let n = super::tpl::fill(tpl, &mut |q| match q {
            "attr" => Ok(vec![super::attr::block(&p, o.full)?]),
            "storage_use"=>Ok(vec![match p.el.child("BinaryDataStorageLocationUse") {Some(e)=>token(e,&[("0","Use"),("1","DontUse")])?,None=>a("0")}]),
            _ => {
                let tag = q.strip_prefix("E:").ok_or(format!("шаблон: место ${q}"))?;
                let (_, pp) = et.iter().find(|(t, _)| *t == tag).ok_or(format!("нет свойства {tag} у {kind}"))?;
                Ok(vec![p_value(pp, p.get(tag)?, x)?])
            }
        })?;
        items.push(n);
    }
    Ok(coll(guid, items))
}

/// Значение свойства по виду из таблицы выгрузки.
pub fn p_value(p: &P, e: &El, x: &Index) -> Result<Node, String> {
    Ok(match p {
        P::Bool(_) => boolean(e)?,
        P::Tok(_, map) | P::TokOr(_, map, _) if map.is_empty() => {
            let t = e.text()?;
            a(int(&t).ok_or(format!("строка {}: <{}>: не число {t:?}", e.line, e.name))?)
        }
        P::Tok(_, map) | P::TokOr(_, map, _) => token(e, map)?,
        P::Ref(_) => reference(x, e)?,
        P::Ml(_) => mlstring(e)?,
        P::Str(_) => s(super::tpl::crlf(&e.text()?)),
        P::Type(_) => type_desc(x, e)?,
        _ => return Err(format!("строка {}: <{}>: вид свойства не загружается местом $P", e.line, e.name)),
    })
}

/// Места этого модуля; `None` — не наше место.
pub fn custom(q: &str, p: &Props, o: &Obj) -> Option<Result<Vec<Node>, String>> {
    let (k, arg) = q.split_once(':').unwrap_or((q, ""));
    let one = |r: Result<Node, String>| Some(r.map(|n| vec![n]));
    match k {
        "P" => {
            let t = table(o.class)?;
            let r = match t.iter().find(|(tag, _)| *tag == arg) {
                Some((_, pp)) => p.get(arg).and_then(|e| p_value(pp, e, p.x)),
                None => Err(format!("нет свойства {arg} в таблице класса {}", o.class)),
            };
            one(r)
        }
        "mdl" => one(p.get(arg).and_then(|e| super::tpl::md_list(p.x, e, "xr:Item", &|r| r))),
        "fields" => one(p.get(arg).and_then(|e| fields(p.x, o, e))),
        "chars" => one(p.get("Characteristics").and_then(|e| characteristics(p.x, e))),
        "attrs" => one(attributes(p.x, o, arg)),
        "ts" => one(tabular_sections(p.x, o, arg)),
        "cmds" => one(commands(p.x, o, arg)),
        // ссылка, которая загружается только пустой (позиция не установлена)
        "empty" => one(p.get(arg).and_then(|e| match e.text()?.as_str() {
            "" => Ok(a(ZERO)),
            t => Err(format!("<{arg}>: непустое значение {t:?} не загружается — позиция в записи не установлена")),
        })),
        // accnt FUN_180370810: версия записи регистра бухгалтерии — 21, при PeriodAdjustmentLength (член +0x2e4, последнее
        // поле записи) ≠ 0 — 22 и еще раз 22
        "accver" => Some(p.text("PeriodAdjustmentLength").map(|t| if t.trim() == "0" { vec![a("21")] } else { vec![a("22"), a("22")] })),
        "thisnode" => one(o.gen.ok_or("нет InternalInfo".to_string()).and_then(|g| g.text_of("xr:ThisNode")).map(a)),
        // как стандартные реквизиты (`$std`): нет элемента — `{0}` (выгрузка не пишет элемент без частей)
        "stdts" => one(match p.el.child("StandardTabularSections") {
            Some(e) => standard_tabular_sections(p.x, o, e),
            None => Ok(l(vec![a("0")])),
        }),
        "kattrs" => {
            let (guid, kind) = arg.split_once(':').unwrap_or((arg, ""));
            let items: Result<Vec<Node>, String> = o.children(kind).into_iter().map(|e| kind_attribute(p.x, o, kind, e)).collect();
            one(items.map(|v| coll(guid, v)))
        }
        "columns" => one(columns(p.x, o, arg)),
        "seqdims" => one(sequence_dimensions(p.x, o, arg)),
        "wsrefs" | "wsns" => one(p.get("XDTOPackages").and_then(|e| ws_packages(p.x, e, k == "wsrefs"))),
        "wsops" => one(nested_items(p.x, o, arg, "Operation", &operation)),
        "urls" => one(nested_items(p.x, o, arg, "URLTemplate", &url_template)),
        "chans" => one(nested_items(p.x, o, arg, "IntegrationServiceChannel", &channel)),
        "el" => {
            let (guid, kind) = arg.split_once(':').unwrap_or((arg, ""));
            one(elements(p.x, o, guid, kind))
        }
        _ => None,
    }
}

const FIELD_REF: &str = "60ea359f-3a6e-48bb-8e71-d2a457572918";

/// Ссылка на поле: `<таблица>.StandardAttribute.<Имя>` → `{-код}`, реквизит → `{0,<uuid>}`, `-1`/`0` как есть.
fn field_ref(x: &Index, t: &str, at: &El) -> Result<Node, String> {
    if t == "-1" || t == "0" {
        return Ok(l(vec![a(t)]));
    }
    if let Some((table, name)) = t.split_once(".StandardAttribute.") {
        let class = table.split('.').next().unwrap_or_default();
        let names = crate::xml::standard::NAMES.iter().find(|(c, _)| *c == class).map(|(_, n)| *n).unwrap_or(&[]);
        let code = names.iter().find(|(_, n)| *n == name).map(|(c, _)| *c).ok_or(format!("строка {}: стандартный реквизит {t}", at.line))?;
        return Ok(l(vec![a(code)]));
    }
    Ok(l(vec![a("0"), a(x.id(t).ok_or(format!("строка {}: нет поля {t}", at.line))?)]))
}

/// Поля (ввод по строке, блокировка) `{1,{0,N,{"#",60ea359f-…,<поле>}…}}`.
fn fields(x: &Index, _o: &Obj, e: &El) -> Result<Node, String> {
    let mut v = vec![a("0"), a("0")];
    for f in e.elems() {
        if f.name != "xr:Field" {
            return Err(format!("строка {}: <{}> вместо xr:Field", f.line, f.name));
        }
        v.push(l(vec![s("#"), a(FIELD_REF), field_ref(x, &f.text()?, f)?]));
    }
    v[1] = a((v.len() - 2).to_string());
    Ok(l(vec![a("1"), l(v)]))
}

const CHARACTERISTIC: &str = "fe839d42-d094-40ba-b903-75bccc21ba30";

/// Характеристики `{0,{N,{"#",fe839d42-…,{4,{1,<типы>},{1,<значения>},<поля 3–12>}}…}}`; поле — `{1,<ссылка>,0}`.
fn characteristics(x: &Index, e: &El) -> Result<Node, String> {
    let mut items = vec![a("0")];
    for c in e.elems() {
        if c.name != "xr:Characteristic" {
            return Err(format!("строка {}: <{}> в Characteristics", c.line, c.name));
        }
        let t = c.req("xr:CharacteristicTypes")?;
        let v = c.req("xr:CharacteristicValues")?;
        let (tf, vf) = (t.req_attr("from")?, v.req_attr("from")?);
        let fld = |el: &El, tag: &str| -> Result<Node, String> {
            let f = el.req(tag)?;
            Ok(l(vec![a("1"), field_ref(x, &f.text()?, f)?, a("0")]))
        };
        let id = |n: &str| x.id(n).map(a).ok_or(format!("строка {}: нет {n}", c.line));
        let body = l(vec![
            a("4"),
            l(vec![a("1"), id(tf)?]),
            l(vec![a("1"), id(vf)?]),
            fld(v, "xr:ObjectField")?,
            fld(v, "xr:TypeField")?,
            fld(v, "xr:ValueField")?,
            fld(t, "xr:TypesFilterField")?,
            super::attr::value(x, t.req("xr:TypesFilterValue")?)?,
            fld(t, "xr:KeyField")?,
            fld(t, "xr:DataPathField")?,
            fld(t, "xr:MultipleValuesUseField")?,
            fld(v, "xr:MultipleValuesKeyField")?,
            fld(v, "xr:MultipleValuesOrderField")?,
        ]);
        items.push(l(vec![s("#"), a(CHARACTERISTIC), body]));
    }
    items[0] = a((items.len() - 1).to_string());
    Ok(l(vec![a("0"), l(items)]))
}

/// Коллекция `{<guid>,N,<элементы>…}`.
fn coll(guid: &str, items: Vec<Node>) -> Node {
    let mut v = vec![a(guid), a(items.len().to_string())];
    v.extend(items);
    l(v)
}

/// Версия записи реквизита объекта по классу (у реквизита табличной части — 8 у всех).
fn attr_version(class: &str) -> &'static str {
    match class {
        "Catalog" | "Document" => "6",
        "ExchangePlan" | "ChartOfCalculationTypes" => "4",
        _ => "3",
    }
}

/// Иерархические классы: у реквизитов и табличных частей есть «использование» (для групп/элементов).
fn has_use(class: &str) -> bool {
    matches!(class, "Catalog" | "ChartOfCharacteristicTypes")
}

/// Признак учета `{{6,{27…},<история данных>},0}`, реквизит адресации `{{4,{27…},<индексирование>,<измерение>,
/// <полнотекстовый поиск>,<история данных>},0}`.
fn kind_attribute(x: &Index, o: &Obj, kind: &str, e: &El) -> Result<Node, String> {
    let id = e.req_attr("uuid")?;
    let p = Props { el: e.req("Properties")?, x, id };
    const USE: &[(&str, &str)] = &[("0", "DontUse"), ("1", "Use")];
    let tok = |tag: &str, map: &[(&str, &str)]| p.get(tag).and_then(|e| token(e, map));
    let mut v = vec![a(if kind == "AddressingAttribute" { "4" } else { "6" }), super::attr::block(&p, o.full)?];
    if kind == "AddressingAttribute" {
        v.push(tok("Indexing", &[("0", "DontIndex"), ("1", "Index"), ("2", "IndexWithAdditionalOrder")])?);
        v.push(p.reference("AddressingDimension")?);
        v.push(tok("FullTextSearch", USE)?);
    }
    v.push(tok("DataHistory", USE)?);
    Ok(l(vec![l(v), a("0")]))
}

/// Стандартные табличные части `{1,{0,N,<код>,{3,<синоним>,"комментарий",0,<проверка>,{1,<реквизиты>},<подсказка>},…}}`.
fn standard_tabular_sections(x: &Index, o: &Obj, e: &El) -> Result<Node, String> {
    const CODES: &[(&str, &str)] = &[("-12", "ExtDimensionTypes"), ("-30", "LeadingCalculationTypes"), ("-20", "DisplacingCalculationTypes"), ("-10", "BaseCalculationTypes")];
    let names = crate::xml::standard::NAMES.iter().find(|(c, _)| *c == format!("{}.StandardTabularSection", o.class)).map(|(_, n)| *n).ok_or("стандартные табличные части класса")?;
    let mut v = vec![a("0"), a("0")];
    for t in e.elems() {
        if t.name != "xr:StandardTabularSection" {
            return Err(format!("строка {}: <{}>", t.line, t.name));
        }
        let name = t.req_attr("name")?;
        let code = CODES.iter().find(|(_, n)| *n == name).map(|(c, _)| *c).ok_or(format!("строка {}: стандартная табличная часть {name}", t.line))?;
        let std = super::standard::attributes_named(x, names, o.full, t.req("xr:StandardAttributes")?)?;
        v.push(a(code));
        v.push(l(vec![
            a("3"),
            mlstring(t.req("xr:Synonym")?)?,
            s(super::tpl::crlf(&t.text_of("xr:Comment")?)),
            a("0"),
            token(t.req("xr:FillChecking")?, &[("0", "DontCheck"), ("1", "ShowError")])?,
            std,
            mlstring(t.req("xr:ToolTip")?)?,
        ]));
    }
    v[1] = a(((v.len() - 2) / 2).to_string());
    Ok(l(vec![a("1"), l(v)]))
}

/// Хвост реквизита после блока `{27,…}`: Indexing, [Use], FullTextSearch, DataHistory — по классу и месту.
fn attr_tail(p: &Props, class: &str, in_ts: bool) -> Result<Vec<Node>, String> {
    let tok = |tag: &str, map: &[(&str, &str)]| p.get(tag).and_then(|e| token(e, map));
    const USE: &[(&str, &str)] = &[("0", "DontUse"), ("1", "Use")];
    let mut v = vec![tok("Indexing", &[("0", "DontIndex"), ("1", "Index"), ("2", "IndexWithAdditionalOrder")])?];
    if !in_ts && has_use(class) {
        v.push(tok("Use", &[("0", "ForItem"), ("1", "ForFolder"), ("2", "ForFolderAndItem")])?);
    }
    v.push(tok("FullTextSearch", USE)?);
    v.push(tok("DataHistory", USE)?);
    Ok(v)
}

/// Обработки и отчеты: реквизит — только блок.
fn tool(class: &str) -> bool {
    matches!(class, "DataProcessor" | "Report" | "ExternalDataProcessor")
}

/// Реквизит: `{{6,{27…},<хвост>,0,{1,<нулевой>}},0}` у объекта, `{{8,{27…},<хвост>},0}` у табличной части,
/// `{{0,{27…}},0}` у обработок и отчетов.
fn attribute(x: &Index, o: &Obj, in_ts: bool, e: &El) -> Result<Node, String> {
    let id = e.req_attr("uuid")?;
    let p = Props { el: e.req("Properties")?, x, id };
    if tool(o.class) {
        return Ok(l(vec![l(vec![a("0"), super::attr::block(&p, o.full)?]), a("0")]));
    }
    let mut v = vec![a(if in_ts { "8" } else { attr_version(o.class) }), super::attr::block(&p, o.full)?];
    v.extend(attr_tail(&p, o.class, in_ts)?);
    if !in_ts {
        v.extend([a("0"), l(vec![a("1"), a(ZERO)])]);
    }
    Ok(l(vec![l(v), a("0")]))
}

fn attributes(x: &Index, o: &Obj, guid: &str) -> Result<Node, String> {
    let items: Result<Vec<Node>, String> = o.children("Attribute").into_iter().map(|e| attribute(x, o, false, e)).collect();
    Ok(coll(guid, items?))
}

/// Код LineNumber стандартного реквизита табличной части по классу владельца.
fn line_number_code(class: &str) -> &'static str {
    match class {
        "ChartOfCalculationTypes" => "-100",
        "DataProcessor" | "Report" | "ExternalDataProcessor" => "-3",
        _ => "-10",
    }
}

/// Табличная часть `{{2,{11,<типы>,{0,<заголовок>},<проверка>,<стандартные>,<подсказка>},[<использование>],<длина номера>},1,
/// {888744e1-…,N,<реквизиты>}}`.
fn tabular_section(x: &Index, o: &Obj, e: &El) -> Result<Node, String> {
    let id = e.req_attr("uuid")?;
    let pe = e.req("Properties")?;
    let p = Props { el: pe, x, id };
    let ii = e.req("InternalInfo")?;
    let gen = |cat: &str, tag: &str| -> Result<Node, String> {
        let g = ii.all("xr:GeneratedType").find(|g| g.attr("category") == Some(cat)).ok_or(format!("нет GeneratedType {cat}"))?;
        g.req_attr("name")?;
        Ok(a(g.text_of(tag)?))
    };
    let std = match pe.child("StandardAttributes") {
        Some(sa) => l(vec![a("1"), super::standard::attributes_named(x, &[(line_number_code(o.class), "LineNumber")], o.full, sa)?]),
        None => l(vec![a("0")]),
    };
    let body = l(vec![
        a("11"),
        gen("TabularSection", "xr:TypeId")?,
        gen("TabularSection", "xr:ValueId")?,
        gen("TabularSectionRow", "xr:TypeId")?,
        gen("TabularSectionRow", "xr:ValueId")?,
        l(vec![a("0"), p.header()?]),
        token(p.get("FillChecking")?, &[("0", "DontCheck"), ("1", "ShowError")])?,
        std,
        p.ml("ToolTip")?,
    ]);
    let attrs: Result<Vec<Node>, String> = match e.child("ChildObjects") {
        Some(co) => co.elems().map(|c| if c.name == "Attribute" { attribute(x, o, true, c) } else { Err(format!("строка {}: <{}> в табличной части", c.line, c.name)) }).collect(),
        None => Ok(Vec::new()),
    };
    let attr_coll = match o.class {
        "DataProcessor" | "ExternalDataProcessor" => "5d24a9d1-098e-11d6-b9b8-0050bae0a95d",
        "Report" => "c339c860-29e2-11d6-a3c7-0050bae0a776",
        _ => "888744e1-b616-11d4-9436-004095e12fc7",
    };
    if tool(o.class) {
        return Ok(l(vec![l(vec![a("0"), body]), a("1"), coll(attr_coll, attrs?)]));
    }
    let mut head = vec![a(if matches!(o.class, "Catalog" | "Document") { "2" } else { "1" }), body];
    if has_use(o.class) {
        head.push(token(p.get("Use")?, &[("0", "ForItem"), ("1", "ForFolder"), ("2", "ForFolderAndItem")])?);
    }
    // последний элемент заголовка: у БП 5, у табличной части, добавленной Конфигуратором 8.5, — 9 там же
    let lnl = p.text("LineNumberLength")?;
    head.push(a(int(&lnl).filter(|n| !n.starts_with('-')).ok_or(format!("LineNumberLength {lnl:?}: не число"))?));
    Ok(l(vec![l(head), a("1"), coll(attr_coll, attrs?)]))
}

fn tabular_sections(x: &Index, o: &Obj, guid: &str) -> Result<Node, String> {
    let items: Result<Vec<Node>, String> = o.children("TabularSection").into_iter().map(|e| tabular_section(x, o, e)).collect();
    Ok(coll(guid, items?))
}

const COMMAND_CLASS: &str = "078a6af8-d22c-4248-9c33-7e90075a3d2c";

/// Команда объекта `{{0,{0,0,0,{1,{2,<uuid>,078a6af8-…},{9,<картинка>,<представление>,<подсказка>,1,<клавиши>,0,<группа>,
/// <тип параметра>,<заголовок>,<изменяет данные>,<режим>,<поведение>}}}},0}` (позиции `{9,…}` — как у общей команды).
pub fn command_body(p: &Props, include_help: Node) -> Result<Node, String> {
    let rep = token(p.get("Representation")?, crate::xml::classes::REPRESENTATION)?;
    Ok(l(vec![
        a("9"),
        super::tpl::picture(p.x, p.get("Picture")?)?,
        rep,
        p.ml("ToolTip")?,
        a("1"),
        shortcut(p.get("Shortcut")?)?,
        include_help,
        group(p.x, p.get("Group")?)?,
        p.types("CommandParameterType")?,
        p.header()?,
        p.bool("ModifiesData")?,
        token(p.get("ParameterUseMode")?, &[("0", "Single"), ("1", "Multiple")])?,
        token(p.get("OnMainServerUnavalableBehavior")?, crate::xml::classes::ON_MAIN_SERVER_UNAVAILABLE)?,
    ]))
}

/// У регистров узел команды без обертки `{0,0,0,…}`.
fn command(x: &Index, class: &str, e: &El) -> Result<Node, String> {
    let id = e.req_attr("uuid")?;
    let p = Props { el: e.req("Properties")?, x, id };
    let n = l(vec![a("1"), l(vec![a("2"), a(id), a(COMMAND_CLASS)]), command_body(&p, a("0"))?]);
    let inner = if class.ends_with("Register") || tool(class) || class == "DocumentJournal" { n } else { l(vec![a("0"), a("0"), a("0"), n]) };
    Ok(l(vec![l(vec![a("0"), inner]), a("0")]))
}

fn commands(x: &Index, o: &Obj, guid: &str) -> Result<Node, String> {
    let items: Result<Vec<Node>, String> = o.children("Command").into_iter().map(|e| command(x, o.class, e)).collect();
    Ok(coll(guid, items?))
}

/// Графы журнала `{{4,<заголовок>,{0,N,<ссылки на реквизиты>},<индексирование>},0}`.
fn columns(x: &Index, o: &Obj, guid: &str) -> Result<Node, String> {
    let mut items = Vec::new();
    for e in o.children("Column") {
        let id = e.req_attr("uuid")?;
        let p = Props { el: e.req("Properties")?, x, id };
        let refs = super::tpl::md_list(x, p.get("References")?, "xr:Item", &|r| r)?;
        let ix = token(p.get("Indexing")?, &[("0", "DontIndex"), ("1", "Index"), ("2", "IndexWithAdditionalOrder")])?;
        items.push(l(vec![l(vec![a("4"), p.header()?, refs, ix]), a("0")]));
    }
    Ok(coll(guid, items))
}

/// Измерения последовательности `{{0,{2,<заголовок>,<типы>},{0,N,<реквизиты документов>},{0,N,<измерения регистров>}},0}`.
fn sequence_dimensions(x: &Index, o: &Obj, guid: &str) -> Result<Node, String> {
    let mut items = Vec::new();
    for e in o.children("Dimension") {
        let id = e.req_attr("uuid")?;
        let p = Props { el: e.req("Properties")?, x, id };
        let dm = super::tpl::md_list(x, p.get("DocumentMap")?, "xr:Item", &|r| r)?;
        let rm = super::tpl::md_list(x, p.get("RegisterRecordsMap")?, "xr:Item", &|r| r)?;
        items.push(l(vec![l(vec![a("0"), l(vec![a("2"), p.header()?, p.types("Type")?]), dm, rm]), a("0")]));
    }
    Ok(coll(guid, items))
}

/// Пакеты web-сервиса: ссылки `{0,N,<ссылка>…}` (`refs`) или пространства имен `{N,"ns"…}`.
fn ws_packages(x: &Index, e: &El, refs: bool) -> Result<Node, String> {
    let mut r = vec![a("0"), a("0")];
    let mut ns = vec![a("0")];
    for it in e.elems() {
        if it.name != "xr:Item" || !it.text_of("xr:Presentation")?.is_empty() || it.text_of("xr:CheckState")? != "0" {
            return Err(format!("строка {}: XDTOPackages: элемент не как у выгрузки", it.line));
        }
        let v = it.req("xr:Value")?;
        match v.req_attr("xsi:type")? {
            "xr:MDObjectRef" => r.push(super::tpl::md_ref(x, v)?),
            "xs:string" => ns.push(s(v.text()?)),
            t => return Err(format!("строка {}: XDTOPackages: {t}", v.line)),
        }
    }
    r[1] = a((r.len() - 2).to_string());
    ns[0] = a((ns.len() - 1).to_string());
    Ok(l(if refs { r } else { ns }))
}

/// Тип XDTO `xs:string` / `<… xmlns:dNp1="ns">dNp1:Тип` → `{0,"ns","Тип"}`.
fn xdto_type(e: &El) -> Result<Node, String> {
    let t = e.text()?;
    let (p, name) = t.split_once(':').ok_or(format!("строка {}: тип XDTO {t:?}", e.line))?;
    let ns = match e.attr(&format!("xmlns:{p}")) {
        Some(ns) => ns.to_string(),
        None => crate::xml::w::namespace(p).ok_or(format!("строка {}: префикс {p}", e.line))?.to_string(),
    };
    Ok(l(vec![a("0"), s(ns), s(name)]))
}

/// Операция `{{1,<заголовок>,<тип>,<Nillable>,<Transactioned>,"процедура",<блокировка>},1,{b78a00b2-…,N,<параметры>}}`.
fn operation(x: &Index, _o: &Obj, e: &El) -> Result<Node, String> {
    let id = e.req_attr("uuid")?;
    let p = Props { el: e.req("Properties")?, x, id };
    let body = l(vec![
        a("1"),
        p.header()?,
        xdto_type(p.get("XDTOReturningValueType")?)?,
        p.bool("Nillable")?,
        p.bool("Transactioned")?,
        p.string("ProcedureName")?,
        token(p.get("DataLockControlMode")?, &[("0", "Automatic"), ("1", "Managed"), ("2", "AutomaticAndManaged")])?,
    ]);
    let mut params = Vec::new();
    if let Some(co) = e.child("ChildObjects") {
        co.text().ok();
        for pe in co.all("Parameter") {
            let pid = pe.req_attr("uuid")?;
            let pp = Props { el: pe.req("Properties")?, x, id: pid };
            let dir = token(pp.get("TransferDirection")?, &[("0", "In"), ("1", "Out"), ("2", "InOut")])?;
            params.push(l(vec![l(vec![a("0"), pp.header()?, xdto_type(pp.get("XDTOValueType")?)?, pp.bool("Nillable")?, dir]), a("0")]));
        }
    }
    Ok(l(vec![body, a("1"), coll("b78a00b2-2260-4ef5-a70c-17889cfee695", params)]))
}

/// Шаблон URL `{{0,"шаблон",<заголовок>},1,{21c96ea8-…,N,{{0,"обработчик",<метод>,<заголовок>},0}…}}`.
fn url_template(x: &Index, _o: &Obj, e: &El) -> Result<Node, String> {
    let id = e.req_attr("uuid")?;
    let p = Props { el: e.req("Properties")?, x, id };
    let body = l(vec![a("0"), p.string("Template")?, p.header()?]);
    let mut methods = Vec::new();
    if let Some(co) = e.child("ChildObjects") {
        co.text().ok();
        for me in co.all("Method") {
            let mid = me.req_attr("uuid")?;
            let mp = Props { el: me.req("Properties")?, x, id: mid };
            let m = token(mp.get("HTTPMethod")?, crate::xml::classes_tools::http_methods())?;
            methods.push(l(vec![l(vec![a("0"), mp.string("Handler")?, m, mp.header()?]), a("0")]));
        }
    }
    Ok(l(vec![body, a("1"), coll("21c96ea8-c8fc-424a-a0b4-e1ffb2fa1a73", methods)]))
}

/// Канал сервиса интеграции `{{1,<заголовок>,<TypeId>,<ValueId>,"имя канала","обработчик",<направление>,<в транзакции>},0}`:
/// integ.dll FUN_1800e2c30 пишет версию 1 всегда (обратное `classes_tools::channels`).
fn channel(x: &Index, _o: &Obj, e: &El) -> Result<Node, String> {
    let id = e.req_attr("uuid")?;
    let p = Props { el: e.req("Properties")?, x, id };
    let g = e.req("InternalInfo")?.req("xr:GeneratedType")?;
    if g.req_attr("category")? != "Manager" {
        return Err(format!("строка {}: генерируемый тип канала не Manager", g.line));
    }
    g.req_attr("name")?;
    let gid = |tag: &str| g.text_of(tag).and_then(|v| if crate::brace::is_guid(&v) { Ok(a(v)) } else { Err(format!("{tag}: не GUID {v:?}")) });
    Ok(l(vec![
        l(vec![
            a("1"),
            p.header()?,
            gid("xr:TypeId")?,
            gid("xr:ValueId")?,
            p.string("ExternalIntegrationServiceChannelName")?,
            p.string("ReceiveMessageProcessing")?,
            token(p.get("MessageDirection")?, crate::xml::classes_tools::MESSAGE_DIRECTION)?,
            p.bool("Transactioned")?,
        ]),
        a("0"),
    ]))
}

type ItemFn = dyn Fn(&Index, &Obj, &El) -> Result<Node, String>;

/// Вложенные элементы с полными свойствами (операции web-сервиса, шаблоны URL).
fn nested_items(x: &Index, o: &Obj, guid: &str, kind: &str, f: &ItemFn) -> Result<Node, String> {
    let items: Result<Vec<Node>, String> = o.children(kind).into_iter().map(|e| f(x, o, e)).collect();
    Ok(coll(guid, items?))
}

/// Группа команды `{1,<uuid>}`: стандартная или CommandGroup.
fn group(x: &Index, e: &El) -> Result<Node, String> {
    let t = e.text()?;
    let id = match crate::xml::classes::STD_COMMAND_GROUPS.iter().find(|(_, n)| *n == t) {
        Some((g, _)) => g.to_string(),
        None if t.is_empty() => ZERO.to_string(),
        None => x.id(&t).ok_or(format!("строка {}: нет группы {t}", e.line))?.to_string(),
    };
    Ok(l(vec![a("1"), a(id)]))
}

/// Сочетание клавиш `{0,<код>,<модификаторы>}` (обратное `ui::shortcut`): F1–F12, буквы; Ctrl, Ctrl+Shift, Ctrl+Alt.
fn shortcut(e: &El) -> Result<Node, String> {
    let t = e.text()?;
    if t.is_empty() {
        return Ok(l(vec![a("0"), a("0"), a("0")]));
    }
    let (mods, key) = match t.rsplit_once('+') {
        Some((m, k)) => (format!("{m}+"), k.to_string()),
        None => (String::new(), t.clone()),
    };
    let m = match mods.as_str() {
        "" => "0",
        "Ctrl+" => "8",
        "Ctrl+Shift+" => "12",
        "Ctrl+Alt+" => "24",
        _ => return Err(format!("строка {}: клавиши {t}", e.line)),
    };
    let code = match key.strip_prefix('F').and_then(|n| n.parse::<u32>().ok()) {
        Some(n @ 1..=12) => 111 + n,
        _ if key.len() == 1 && key.as_bytes()[0].is_ascii_uppercase() => key.as_bytes()[0] as u32,
        _ => return Err(format!("строка {}: клавиши {t}", e.line)),
    };
    Ok(l(vec![a("0"), a(code.to_string()), a(m)]))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Место `$P:Тег` стоит в шаблоне там же, где свойство читает выгрузка.
    #[test]
    fn paths_match_export_tables() {
        for class in [
            "Catalog", "Document", "InformationRegister", "AccumulationRegister", "DataProcessor", "Report", "ExchangePlan", "ChartOfCharacteristicTypes",
            "ChartOfAccounts", "ChartOfCalculationTypes", "BusinessProcess", "Task", "Subsystem", "DocumentJournal", "Sequence", "AccountingRegister",
            "DocumentNumerator", "FilterCriterion", "WebService", "HTTPService", "WSReference", "IntegrationService", "CalculationRegister",
        ] {
            // позиции выгрузки — по записи версии 22 (`upgrade_21`): `$accver` — два поля
            let n = super::super::tpl::fill(template(class).unwrap(), &mut |q| Ok(if q == "accver" { vec![a("22"), a("22")] } else { vec![a(format!("${q}"))] })).unwrap();
            for (tag, p) in table(class).unwrap() {
                let path: &[usize] = match p {
                    P::Bool(p) | P::Tok(p, _) | P::TokOr(p, _, _) | P::Ref(p) | P::Ml(p) | P::Str(p) | P::Type(p) | P::Attr(p) => p,
                    _ => continue,
                };
                assert_eq!(n.at(path).and_then(Node::atom), Some(format!("$P:{tag}").as_str()), "{class}.{tag} {path:?}");
            }
        }
    }
}
