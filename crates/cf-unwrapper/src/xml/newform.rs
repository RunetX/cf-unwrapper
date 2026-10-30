//! Новая управляемая форма, как ее строит «Конструктор формы» Конфигуратора 8.5.1.1529 с настройками по умолчанию
//! (эталоны 27.10.2026, src/xml/new/forms): формы списка, выбора и выбора группы — шаблон образца с подстановкой
//! основной таблицы; формы элемента, группы и документа — поля шапки, реквизиты и табличные части объекта.
//!
//! Номера элементов (образцы Справочник1…3, Документ1): поле — номер, контекстное меню +1, подсказка +2, следующий
//! элемент +4; страницы и страница — номер и подсказка +1, следующий +2; таблица — номер, меню +1, командная
//! панель +2, подсказка +3, строка поиска +4 (+5, +6), состояние просмотра +7 (+8, +9), управление поиском +10
//! (+11, +12), первая колонка — +21; после последней колонки — +4. Две табличные части и больше — на страницах
//! «Страницы» / «Группа<Имя>», одна — таблицей в форме.
//!
//! Страница «Расположение» («Далее >», эталоны 28.10.2026 src/xml/new/designer/*-layout, *-cols): в форме только
//! отмеченные строки по порядку списка, номера подряд; первая колонка таблицы — +21 и без «НомерСтроки». Колонок
//! больше одной — поля вне таблиц в группе «Шапка» (HorizontalIfPossible) из вертикальных «ЛеваяКолонка» /
//! «ПраваяКолонка» (две) или «Колонка1»…; группа — номер и подсказка +1, следующий +2.

/// Поля и реквизиты объекта для формы.
pub struct Spec {
    /// Catalog, Document.
    pub class: &'static str,
    pub name: String,
    /// Поля вне таблиц по порядку: стандартные реквизиты (Код, Наименование…; Номер, Дата…) и реквизиты.
    pub head: Vec<Field>,
    /// Табличные части: (имя, колонки; «НомерСтроки» — путь `LineNumber`).
    pub tables: Vec<(String, Vec<Field>)>,
    /// «Располагать элементы управления в N колонку(и)».
    pub columns: usize,
}

/// Поле формы: имя (у колонки — без имени таблицы), путь от `Объект.` (у колонки — от таблицы), булев — флажок.
#[derive(Clone, Debug, PartialEq)]
pub struct Field {
    pub name: String,
    pub path: String,
    pub boolean: bool,
}

const CRLF: &str = "\r\n";

/// Начало файла формы — первые две строки образца (BOM, заголовок, корень с пространствами имен).
fn head_lines() -> String {
    let t = include_str!("new/forms/CatalogList.xml");
    t.split_inclusive('\n').take(2).collect::<String>().replace("\r\n", "\n").replace('\n', CRLF)
}

fn field(out: &mut String, tab: &str, tag: &str, name: &str, path: &str, id: usize, multi: bool) {
    out.push_str(&format!("{tab}<{tag} name=\"{name}\" id=\"{id}\">{CRLF}"));
    out.push_str(&format!("{tab}\t<DataPath>{path}</DataPath>{CRLF}{tab}\t<EditMode>EnterOnInput</EditMode>{CRLF}"));
    if multi {
        out.push_str(&format!("{tab}\t<ExtendedEditMultipleValues>true</ExtendedEditMultipleValues>{CRLF}"));
    }
    out.push_str(&format!("{tab}\t<ContextMenu name=\"{name}КонтекстноеМеню\" id=\"{}\"/>{CRLF}", id + 1));
    out.push_str(&format!("{tab}\t<ExtendedTooltip name=\"{name}РасширеннаяПодсказка\" id=\"{}\"/>{CRLF}", id + 2));
    out.push_str(&format!("{tab}</{tag}>{CRLF}"));
}

fn attr(out: &mut String, tab: &str, name: &str, path: &str, boolean: bool, id: usize) {
    if path.ends_with(".LineNumber") {
        field(out, tab, "LabelField", name, path, id, false);
    } else if boolean {
        field(out, tab, "CheckBoxField", name, path, id, false);
    } else {
        field(out, tab, "InputField", name, path, id, true);
    }
}

fn title(out: &mut String, tab: &str, text: &str) {
    out.push_str(&format!("{tab}<Title>{CRLF}{tab}\t<v8:item>{CRLF}{tab}\t\t<v8:lang>ru</v8:lang>{CRLF}{tab}\t\t<v8:content>{text}</v8:content>{CRLF}{tab}\t</v8:item>{CRLF}{tab}</Title>{CRLF}"));
}

fn addition(out: &mut String, tab: &str, table: &str, tag: &str, suffix: &str, kind: &str, id: usize) {
    let n = format!("{table}{suffix}");
    out.push_str(&format!("{tab}<{tag} name=\"{n}\" id=\"{id}\">{CRLF}{tab}\t<AdditionSource>{CRLF}{tab}\t\t<Item>{table}</Item>{CRLF}{tab}\t\t<Type>{kind}</Type>{CRLF}{tab}\t</AdditionSource>{CRLF}"));
    out.push_str(&format!("{tab}\t<ContextMenu name=\"{n}КонтекстноеМеню\" id=\"{}\"/>{CRLF}{tab}\t<ExtendedTooltip name=\"{n}РасширеннаяПодсказка\" id=\"{}\"/>{CRLF}{tab}</{tag}>{CRLF}", id + 1, id + 2));
}

/// Таблица табличной части с номера `id`; следующий свободный номер.
fn table(out: &mut String, tab: &str, name: &str, cols: &[Field], id: usize) -> usize {
    out.push_str(&format!("{tab}<Table name=\"{name}\" id=\"{id}\">{CRLF}"));
    for l in ["<Representation>List</Representation>", "<AutoInsertNewRow>true</AutoInsertNewRow>", "<EnableStartDrag>true</EnableStartDrag>", "<EnableDrag>true</EnableDrag>"] {
        out.push_str(&format!("{tab}\t{l}{CRLF}"));
    }
    out.push_str(&format!("{tab}\t<DataPath>Объект.{name}</DataPath>{CRLF}{tab}\t<ShowCommandBar>auto</ShowCommandBar>{CRLF}{tab}\t<RowFilter xsi:nil=\"true\"/>{CRLF}"));
    out.push_str(&format!("{tab}\t<ContextMenu name=\"{name}КонтекстноеМеню\" id=\"{}\"/>{CRLF}", id + 1));
    out.push_str(&format!("{tab}\t<AutoCommandBar name=\"{name}КоманднаяПанель\" id=\"{}\"/>{CRLF}", id + 2));
    out.push_str(&format!("{tab}\t<ExtendedTooltip name=\"{name}РасширеннаяПодсказка\" id=\"{}\"/>{CRLF}", id + 3));
    let t2 = format!("{tab}\t");
    addition(out, &t2, name, "SearchStringAddition", "СтрокаПоиска", "SearchStringRepresentation", id + 4);
    addition(out, &t2, name, "ViewStatusAddition", "СостояниеПросмотра", "ViewStatusRepresentation", id + 7);
    addition(out, &t2, name, "SearchControlAddition", "УправлениеПоиском", "SearchControl", id + 10);
    out.push_str(&format!("{tab}\t<ChildItems>{CRLF}"));
    let t3 = format!("{tab}\t\t");
    let mut n = id + 21;
    for c in cols {
        attr(out, &t3, &format!("{name}{}", c.name), &format!("Объект.{name}.{}", c.path), c.boolean, n);
        n += 4;
    }
    out.push_str(&format!("{tab}\t</ChildItems>{CRLF}{tab}</Table>{CRLF}"));
    n
}

/// Начало группы без заголовка с номера `id` (подсказка +1) до списка ее элементов.
fn group(out: &mut String, tab: &str, name: &str, text: &str, kind: &str, id: usize) {
    out.push_str(&format!("{tab}<UsualGroup name=\"{name}\" id=\"{id}\">{CRLF}"));
    title(out, &format!("{tab}\t"), text);
    out.push_str(&format!("{tab}\t<Group>{kind}</Group>{CRLF}{tab}\t<Representation>None</Representation>{CRLF}{tab}\t<ShowTitle>false</ShowTitle>{CRLF}"));
    out.push_str(&format!("{tab}\t<ExtendedTooltip name=\"{name}РасширеннаяПодсказка\" id=\"{}\"/>{CRLF}{tab}\t<ChildItems>{CRLF}", id + 1));
}

/// Форма элемента (`folder` — группы) справочника или форма документа.
pub fn object_form(s: &Spec, folder: bool) -> String {
    let mut o = head_lines();
    if s.class == "Document" {
        o.push_str(&format!("\t<AutoTime>CurrentOrLast</AutoTime>{CRLF}\t<UsePostingMode>Auto</UsePostingMode>{CRLF}\t<RepostOnWrite>true</RepostOnWrite>{CRLF}"));
    } else {
        o.push_str(&format!("\t<WindowOpeningMode>LockOwner</WindowOpeningMode>{CRLF}\t<UseForFoldersAndItems>{}</UseForFoldersAndItems>{CRLF}", if folder { "Folders" } else { "Items" }));
    }
    o.push_str(&format!("\t<AutoCommandBar name=\"\" id=\"-1\"/>{CRLF}\t<ChildItems>{CRLF}"));
    let mut id = 1;
    if s.columns > 1 && !s.head.is_empty() {
        // shortcut: поля по колонкам поровну, первые на одно больше (эталоны 4 поля / 2, 7 / 4); колонки без полей
        // (полей меньше колонок) не выводятся — образца нет
        group(&mut o, "\t\t", "Шапка", "Шапка", "HorizontalIfPossible", id);
        id += 2;
        let (per, extra) = (s.head.len() / s.columns, s.head.len() % s.columns);
        let mut rest = s.head.iter();
        for k in 0..s.columns.min(s.head.len()) {
            let (name, text) = match (s.columns, k) {
                (2, 0) => ("ЛеваяКолонка".to_string(), "Левая колонка".to_string()),
                (2, _) => ("ПраваяКолонка".to_string(), "Правая колонка".to_string()),
                _ => (format!("Колонка{}", k + 1), format!("Колонка {}", k + 1)),
            };
            group(&mut o, "\t\t\t\t", &name, &text, "Vertical", id);
            id += 2;
            for f in rest.by_ref().take(per + usize::from(k < extra)) {
                attr(&mut o, "\t\t\t\t\t\t", &f.name, &format!("Объект.{}", f.path), f.boolean, id);
                id += 4;
            }
            o.push_str(&format!("\t\t\t\t\t</ChildItems>{CRLF}\t\t\t\t</UsualGroup>{CRLF}"));
        }
        o.push_str(&format!("\t\t\t</ChildItems>{CRLF}\t\t</UsualGroup>{CRLF}"));
    } else {
        for f in &s.head {
            attr(&mut o, "\t\t", &f.name, &format!("Объект.{}", f.path), f.boolean, id);
            id += 4;
        }
    }
    match s.tables.len() {
        0 => {}
        1 => {
            table(&mut o, "\t\t", &s.tables[0].0, &s.tables[0].1, id);
        }
        _ => {
            o.push_str(&format!("\t\t<Pages name=\"Страницы\" id=\"{id}\">{CRLF}"));
            title(&mut o, "\t\t\t", "Страницы");
            o.push_str(&format!("\t\t\t<ExtendedTooltip name=\"СтраницыРасширеннаяПодсказка\" id=\"{}\"/>{CRLF}\t\t\t<ChildItems>{CRLF}", id + 1));
            id += 2;
            for (t, cols) in &s.tables {
                let page = format!("Группа{t}");
                o.push_str(&format!("\t\t\t\t<Page name=\"{page}\" id=\"{id}\">{CRLF}"));
                title(&mut o, "\t\t\t\t\t", t);
                o.push_str(&format!("\t\t\t\t\t<ExtendedTooltip name=\"{page}РасширеннаяПодсказка\" id=\"{}\"/>{CRLF}\t\t\t\t\t<ChildItems>{CRLF}", id + 1));
                id = table(&mut o, "\t\t\t\t\t\t", t, cols, id + 2);
                o.push_str(&format!("\t\t\t\t\t</ChildItems>{CRLF}\t\t\t\t</Page>{CRLF}"));
            }
            o.push_str(&format!("\t\t\t</ChildItems>{CRLF}\t\t</Pages>{CRLF}"));
        }
    }
    o.push_str(&format!("\t</ChildItems>{CRLF}\t<Attributes>{CRLF}\t\t<Attribute name=\"Объект\" id=\"1\">{CRLF}\t\t\t<Type>{CRLF}"));
    o.push_str(&format!("\t\t\t\t<v8:Type>cfg:{}Object.{}</v8:Type>{CRLF}\t\t\t</Type>{CRLF}\t\t\t<MainAttribute>true</MainAttribute>{CRLF}\t\t\t<SavedData>true</SavedData>{CRLF}", s.class, s.name));
    if s.class == "Document" {
        o.push_str(&format!("\t\t\t<UseAlways>{CRLF}\t\t\t\t<Field>Объект.RegisterRecords</Field>{CRLF}\t\t\t</UseAlways>{CRLF}"));
    }
    o.push_str(&format!("\t\t</Attribute>{CRLF}\t</Attributes>{CRLF}</Form>"));
    o
}

/// Форма списка, выбора, выбора группы — образец с основной таблицей объекта (`tree` — иерархический справочник).
pub fn list_form(class: &str, name: &str, kind: &str, tree: bool) -> Result<String, String> {
    let t = match (class, kind, tree) {
        ("Catalog", "List", false) => include_str!("new/forms/CatalogList.xml"),
        ("Catalog", "List", true) => include_str!("new/forms/CatalogListTree.xml"),
        ("Catalog", "Choice", false) => include_str!("new/forms/CatalogChoice.xml"),
        ("Catalog", "Choice", true) => include_str!("new/forms/CatalogChoiceTree.xml"),
        ("Catalog", "FolderChoice", true) => include_str!("new/forms/CatalogFolderChoice.xml"),
        ("Document", "List", _) => include_str!("new/forms/DocumentList.xml"),
        ("Document", "Choice", _) => include_str!("new/forms/DocumentChoice.xml"),
        _ => return Err(format!("форма {kind} {class}: нет эталона Конфигуратора")),
    };
    let t = t.replace("\r\n", "\n").replace('\n', CRLF);
    let main = |c: &str, n: &str| format!("<MainTable>{c}.{n}</MainTable>");
    let sample = if class == "Document" { main("Document", "Документ1") } else if tree { main("Catalog", "Справочник3") } else { main("Catalog", "Справочник2") };
    Ok(t.replace(&sample, &main(class, name)))
}

/// «Произвольная форма» конструктора: пустая, без модуля (эталон Справочник3.Форма).
pub fn custom_form() -> String {
    format!("{}	<AutoCommandBar name=\"\" id=\"-1\"/>{CRLF}	<Attributes/>{CRLF}</Form>", head_lines())
}

/// Описание формы (Forms/<Имя>.xml): uuid, имя, синоним, комментарий — образец «ФормаЭлемента» (пустой синоним —
/// `<Synonym/>`, как у выгрузки).
pub fn props(uuid: &str, name: &str, synonym: &str, comment: &str) -> String {
    let t = include_str!("new/forms/Form.xml").replace("\r\n", "\n");
    synonym_comment(&t, "Форма элемента", synonym, comment)
        .replace("8bed9dc8-79d7-49e7-bf41-eeab6e7ec416", uuid)
        .replace("<Name>ФормаЭлемента</Name>", &format!("<Name>{name}</Name>"))
        .replace('\n', CRLF)
}

/// Синоним (был `old`) и комментарий описания `t` (строки через `\n`) из окна конструктора: пустой синоним —
/// `<Synonym/>`, как у выгрузки.
pub(super) fn synonym_comment(t: &str, old: &str, synonym: &str, comment: &str) -> String {
    let esc = |t: &str| t.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;");
    let syn_block = format!("\t\t\t<Synonym>\n\t\t\t\t<v8:item>\n\t\t\t\t\t<v8:lang>ru</v8:lang>\n\t\t\t\t\t<v8:content>{old}</v8:content>\n\t\t\t\t</v8:item>\n\t\t\t</Synonym>");
    let syn = if synonym.is_empty() { "\t\t\t<Synonym/>".to_string() } else { syn_block.replace(&format!(">{old}<"), &format!(">{}<", esc(synonym))) };
    let com = if comment.is_empty() { "\t\t\t<Comment/>".to_string() } else { format!("\t\t\t<Comment>{}</Comment>", esc(comment)) };
    t.replace(&syn_block, &syn).replace("\t\t\t<Comment/>", &com)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Формы Конфигуратора из выгрузки эталонов (%TEMP%\cfkit-s2-forms; нет — пропуск): сгенерированная = его.
    #[test]
    fn like_designer() {
        // Эталоны — формы «Конструктора формы» Конфигуратора 8.5.1.1529 в его же выгрузке (28.10.2026).
        let f = |n: &str, p: &str, b: bool| Field { name: n.into(), path: p.into(), boolean: b };
        let a = |n: &str| f(n, n, false);
        let line = || f("НомерСтроки", "LineNumber", false);
        let (code, desc) = (f("Код", "Code", false), f("Наименование", "Description", false));
        let mut s1 = Spec { class: "Catalog", name: "Справочник1".into(), head: vec![code.clone(), desc.clone(), a("Реквизит1")], tables: vec![("ТабличнаяЧасть1".into(), vec![line(), a("Реквизит1")])], columns: 1 };
        assert_eq!(object_form(&s1, false), include_str!("new/designer/c1-item.xml"));
        // «Далее >»: две колонки и отмеченный «Предопределенный»
        s1.head.insert(2, f("Предопределенный", "Predefined", true));
        s1.columns = 2;
        assert_eq!(object_form(&s1, false), include_str!("new/designer/c1-item-2cols.xml"));
        let attrs = |k: &[usize]| k.iter().map(|k| a(&format!("Реквизит{k}"))).collect::<Vec<_>>();
        let mut s2 = Spec {
            class: "Catalog",
            name: "Справочник2".into(),
            head: [vec![code.clone(), desc.clone()], attrs(&[1, 2, 3, 4, 5])].concat(),
            tables: vec![("ТабличнаяЧасть1".into(), [vec![line()], attrs(&[1, 2])].concat()), ("ТабличнаяЧасть2".into(), [vec![line()], attrs(&[1])].concat())],
            columns: 1,
        };
        assert_eq!(object_form(&s2, false), include_str!("new/designer/c2-item.xml"));
        s2.columns = 4;
        assert_eq!(object_form(&s2, false), include_str!("new/designer/c2-item-4cols.xml"));
        // отмечены Ссылка и ПометкаУдаления, сняты Реквизит2, НомерСтроки первой таблицы и вся вторая
        s2.head = [vec![f("Ссылка", "Ref", false), code.clone(), desc.clone(), f("ПометкаУдаления", "DeletionMark", true)], attrs(&[1, 3, 4, 5])].concat();
        s2.tables = vec![("ТабличнаяЧасть1".into(), attrs(&[1, 2]))];
        s2.columns = 1;
        assert_eq!(object_form(&s2, false), include_str!("new/designer/c2-item-layout.xml"));
        let s3 = Spec { class: "Catalog", name: "Справочник3".into(), head: vec![code, desc, f("Владелец", "Owner", false), f("Родитель", "Parent", false)], tables: vec![], columns: 1 };
        assert_eq!(object_form(&s3, false), include_str!("new/designer/c3-item.xml"));
        assert_eq!(object_form(&s3, true), include_str!("new/designer/c3-folder.xml"));
        let mut d1 = Spec { class: "Document", name: "Документ1".into(), head: vec![f("Номер", "Number", false), f("Дата", "Date", false), a("Реквизит1")], tables: vec![], columns: 1 };
        assert_eq!(object_form(&d1, false), include_str!("new/designer/d1-item.xml"));
        d1.head = vec![f("Номер", "Number", false), f("Проведен", "Posted", true), a("Реквизит1")];
        assert_eq!(object_form(&d1, false), include_str!("new/designer/d1-item-layout.xml"));
        assert_eq!(list_form("Catalog", "Справочник1", "List", false).unwrap(), include_str!("new/designer/c1-list.xml"));
        assert_eq!(list_form("Catalog", "Справочник3", "FolderChoice", true).unwrap(), include_str!("new/designer/c3-folderchoice.xml"));
        assert_eq!(list_form("Document", "Документ1", "Choice", false).unwrap(), include_str!("new/designer/d1-choice.xml"));
        assert_eq!(custom_form(), include_str!("new/designer/c3-custom.xml"));
        assert_eq!(props("8358e99d-5b2c-4670-9338-ad4d3dbadb39", "ФормаЭлемента", "Форма элемента", ""), include_str!("new/designer/c2-item-props.xml"));
    }

    /// Синоним и комментарий из окна конструктора — в описании формы и макета.
    #[test]
    fn wizard_synonym_comment() {
        assert!(props("x", "Ф", "", "к").contains("<Synonym/>") && props("x", "Ф", "", "к").contains("<Comment>к</Comment>"));
        let t = include_str!("new/Template.xml").replace("\r\n", "\n");
        let x = synonym_comment(&t, "Макет", "Печать <А&Б>", "для акта");
        assert!(x.contains("<v8:content>Печать &lt;А&amp;Б&gt;</v8:content>") && x.contains("<Comment>для акта</Comment>"), "{x}");
        assert!(synonym_comment(&t, "Макет", "", "").contains("<Synonym/>"));
    }
}
