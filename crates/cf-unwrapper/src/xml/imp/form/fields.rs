//! Поля формы (узел версии 48): общая часть одна у всех видов, свойства вида — вложенный узел (поле 39). Раскладки
//! сняты со сверки XML ↔ записи загрузки БП (поле ввода — 9 384 элемента).

use super::items::*;
use super::*;

pub(super) const FIELD: &str = "77ffcc29-7f2d-4223-b22f-19666e7250ba";

/// Суффикс имени панели действий выделенных строк, которую платформа добавляет каждому полю.
const ROWS_PANEL: &str = "ПанельДействийВыделенныхСтрок";

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn autofill_hint_does_not_overwrite_keyboard_return_key() {
        // Independent slots verified against platform-created records. Writing
        // Email=16 into slot 61 makes the platform reject the entire CFE export.
        let x=Index::default();
        let read=|_:&str|->Result<Vec<u8>,String>{Err(String::new())};
        let f=F {x:&x,attrs:Default::default(),main:None,cmds:Default::default(),items:Default::default(),referenced:Default::default(),read:&read,used:Default::default(),lists:Default::default(),list_req:Default::default()};
        let e=crate::xml::imp::dom::parse("<InputField><SpecialTextInputMode>Email</SpecialTextInputMode><AutofillHint>Email</AutofillHint><OnScreenKeyboardReturnKeyText>Send</OnScreenKeyboardReturnKeyText></InputField>").unwrap();
        let p=Props::new(&e).unwrap();
        let (_,n)=f.input_sub(&e,&p).unwrap();
        assert_eq!(n.get(61).and_then(Node::atom),Some("6"));
        assert_eq!(n.get(63).and_then(Node::atom),Some("16"));
        assert!(e.unread().is_none());
    }
}

/// Граница числа `{"N",<x>}`; нет — `{"U"}`.
fn bound(e: Option<&El>) -> R<Node> {
    let Some(e) = e else { return Ok(l(vec![s("U")])) };
    match e.req_attr("xsi:type")? {
        "xs:decimal" => Ok(l(vec![s("N"), a(num_text(e)?)])),
        // строкой (обратное value_xml выгрузки: `{"S","…"}`)
        "xs:string" => Ok(l(vec![s("S"), s(crlf(&e.text()?))])),
        _ => Err(err(e, "граница не числом и не строкой пока не загружается")),
    }
}

/// Число как в XML (дробное — как есть: платформа пишет его тем же текстом).
fn num_text(e: &El) -> R<String> {
    let t = e.text()?;
    let ok = !t.is_empty() && t.trim_start_matches('-').split_once('.').map_or(t.trim_start_matches('-').bytes().all(|c| c.is_ascii_digit()), |(i, f)| {
        !i.is_empty() && i.bytes().all(|c| c.is_ascii_digit()) && !f.is_empty() && f.bytes().all(|c| c.is_ascii_digit())
    });
    if !ok {
        return Err(err(e, &format!("не число {t:?}")));
    }
    Ok(t)
}

impl F<'_> {
    /// Поле: `kind` — вид (поле ввода 2…), `tag` — имя элемента XML (для событий).
    pub(super) fn field(&self, e: &El, kind: i64, tag: &str) -> R<Node> {
        let p = Props::new(e)?;
        let name = e.req_attr("name")?;
        let mut v = vec![a("48"), l(vec![a(e.req_attr("id")?), a(ELEM)]), a("0"), a("0")];
        match p.get("UserVisible") {
            Some(u) => v.extend([a("1"), adjustable(self.x, Some(u))?]),
            None => v.push(a("0")),
        }
        let edit = p.get("EditMode").map(txt).transpose()?;
        let (em26, mut em61) = match edit.as_deref() {
            None => (1, 1),
            Some("EnterOnInput") => (2, 3),
            Some("Directly") => (0, 0),
            Some(o) => return Err(err(e, &format!("EditMode {o}"))),
        };
        // AutoEditMode выгрузка пишет вместе с EditMode=EnterOnInput (свое поле не найдено); у формы конструктора 8.5
        // его нет — «не задано», хвост поля 2 = 2 (`form_v59::UNSET_AUTO_EDIT`; эталоны Конфигуратора 27.10.2026)
        match (edit.as_deref(), p.get("AutoEditMode").map(boolean).transpose()?) {
            (Some("EnterOnInput"), Some(true)) | (Some("Directly") | None, None) => {}
            (Some("EnterOnInput"), None) => em61 = 2,
            _ => return Err(err(e, "AutoEditMode не как у выгрузки")),
        }
        // The common field and the InputField subtype store independent
        // properties: MarkRequiredComplete in the v48 tail, AutoMarkIncomplete
        // in the input subtype. Their values need not be equal.
        let ami = p.tri("MarkRequiredComplete")?;
        let hyper = p.b("CellHyperlink", "0")?;
        let data_path = match p.get("DataPath") {
            Some(d) => self.data_path(d)?,
            None => l(vec![a("0")]),
        };
        let footer_path = match p.get("FooterDataPath") {
            Some(d) => self.data_path(d)?,
            None => l(vec![a("0")]),
        };
        let (outer_ev, sub) = match tag {
            "InputField" => self.input_sub(e, &p)?,
            "CheckBoxField" => self.check_sub(&p)?,
            "LabelField" => self.label_sub(&p)?,
            "RadioButtonField" => self.radio_sub(&p)?,
            "SpreadSheetDocumentField" => self.spreadsheet_sub(&p)?,
            "HTMLDocumentField" => self.html_sub(&p)?,
            "PictureField" => self.picture_sub(e, &p)?,
            "ProgressBarField" => self.progress_sub(&p)?,
            "TextDocumentField" => self.text_doc_sub(&p)?,
            "FormattedDocumentField" => self.formatted_doc_sub(&p)?,
            "CalendarField" => self.calendar_sub(&p)?,
            "PDFDocumentField" => self.pdf_sub(&p)?,
            "PlannerField" => Self::planner_sub(&p)?,
            "TrackBarField" => self.track_bar_sub(&p)?,
            "GraphicalSchemaField" => self.schema_sub(&p)?,
            "ChartField" => self.chart_sub(&p)?,
            "GanttChartField" => self.gantt_sub(&p)?,
            "PeriodField" => period_sub()?,
            t => return Err(err(e, &format!("свойства поля {t} пока не загружаются"))),
        };
        v.extend([
            an(kind),
            s(name),
            p.tok("TitleLocation", &[("None", 0), ("Auto", 1), ("Left", 2), ("Top", 3), ("Right", 4), ("Bottom", 5)], 1)?,
            p.num("TitleHeight", "0")?,
            p.ml("Title")?,
            p.ml("ToolTip")?,
            data_path,
            footer_path,
            p.b("Enabled", "1")?,
            p.b("ReadOnly", "0")?,
            p.tri("SkipOnInput")?,
            p.b("DefaultItem", "0")?,
            p.tok("WarningOnEditRepresentation", &[("Show", 0), ("DontShow", 1)], 2)?,
            p.ml("WarningOnEdit")?,
            p.ml("FooterText")?,
            p.b("ShowInHeader", "1")?,
            p.b("ShowInFooter", "1")?,
            hyper.clone(),
            p.tok("HorizontalAlign", H_ALIGN, 3)?,
            p.tok("HeaderHorizontalAlign", super::platform::H_ALIGN, 0)?,
            p.tok("FooterHorizontalAlign", &[("Left", 0), ("Center", 1), ("Right", 2)], 3)?,
            an(em26),
            p.tok("VerticalAlign", V_ALIGN, 3)?,
            p.b("AutoCellHeight", "0")?,
            self.pic(e, &p, "HeaderPicture")?,
            self.pic(e, &p, "FooterPicture")?,
            p.color(self.x, "TitleTextColor")?,
            p.font(self.x, "TitleFont")?,
            p.color(self.x, "TitleBackColor")?,
            p.color(self.x, "FooterTextColor")?,
            p.color(self.x, "FooterBackColor")?,
            p.font(self.x, "FooterFont")?,
            shortcut(p.get("Shortcut"))?,
            a("1"),
            sub,
        ]);
        v.extend([
            events_where(p.get("Events"), &|n| outer_ev(n), &|n| item_event(tag, n))?,
            a("1"),
            self.context_menu(p.get("ContextMenu").ok_or_else(|| err(e, "нет <ContextMenu>"))?)?,
            p.b("Visible", "1")?,
            l(vec![s("Pattern")]),
            l(vec![s("Pattern")]),
            s(""),
            s(""),
            self.field_command_set(p.get("CommandSet"), tag)?,
            p.tok("FixingInTable", super::platform::FIXING_IN_TABLE, 0)?,
            p.tok("ToolTipRepresentation", TOOLTIP_REPR, 0)?,
            a("1"),
            self.extended_tooltip(p.get("ExtendedTooltip").ok_or_else(|| err(e, "нет <ExtendedTooltip>"))?)?,
            p.tok("GroupHorizontalAlign", H_ALIGN, 3)?,
            p.tok("GroupVerticalAlign", V_ALIGN, 3)?,
            importance(e)?,
            a("0"),
        ]);
        // состояние просмотра поля PDF — дополнение за флагом (выгрузка: поле k + 4)
        // PointerType поля табличного документа выгрузка читает отсюда (k + 4), но в эталоне загрузки
        // (ОбращениеСотрудника.ДействиеВыполнить) здесь 0 — загрузка платформы значение Regular теряет.
        // shortcut: по одной форме БП; прочие значения — отказ
        if let Some(t) = p.get("PointerType").filter(|_| tag == "SpreadSheetDocumentField") {
            if t.text()? != "Regular" {
                return Err(err(t, "PointerType поля табличного документа: не Regular — не выведено"));
            }
        }
        match p.get("ViewStatusAddition").filter(|_| tag == "PDFDocumentField") {
            Some(x) => v.extend([a("1"), self.addition(x)?]),
            None => v.push(a("0")),
        }
        // своя таблица поля диаграммы Ганта — предопределенный дочерний элемент вида 6 (mngbase FUN_180710f40)
        match p.get("Table").filter(|_| tag == "GanttChartField") {
            Some(t) => {
                let mut tn = self.table(t)?;
                // shortcut: у таблицы поля Ганта платформа ставит признак панели команд (поле за CommandBarLocation − 1,
                // перед панелью ДействияСтроки) в 1 без свойства в XML (ERP: 15 форм); создание таблицы в mngcore не разобрано
                if let Node::List(items) = &mut tn {
                    if let Some(k) = items.iter().position(|n| n.get(6).and_then(Node::str).is_some_and(|s| s.ends_with("ДействияСтроки"))) {
                        if k >= 3 && items[k - 3].atom() == Some("0") {
                            items[k - 3] = a("1");
                        }
                    }
                }
                v.extend([a("1"), tn]);
            }
            None if tag == "GanttChartField" => return Err(err(e, "нет <Table> поля диаграммы Ганта")),
            None => v.push(a("0")),
        }
        v.extend([
            ami,
            a("0"),
            an(em61),
            a("1"),
            rows_panel(name),
            a("0"),
            a("0"),
            a("2"),
            a("0"),
            a("0"),
            a("1"),
            a("2"),
            p.tok("AutoWidthInTable",&[("Auto",0),("ByData",1),("None",2),("ByDataAndTitle",3)],0)?,
            p.tok("CellHyperlinkDisplayVariant", &[("Always", 1)], 0)?,
            p.tok("CellHyperlinkRepresentation", &[("Show", 1)], 0)?,
        ]);
        // подсказки по ячейке-гиперссылке выгрузка пишет только вместе
        if (p.get("CellHyperlink").is_some(), p.get("CellHyperlinkDisplayVariant").is_some(), p.get("CellHyperlinkRepresentation").is_some()) != (false, false, false)
            && hyper.atom() != Some("1")
        {
            return Err(err(e, "свойства ячейки-гиперссылки без CellHyperlink"));
        }
        Ok(l(v))
    }

    /// Список выбора `{3,N,("<представление>",<значение>)…,({<пометка>,<картинка>})…}`; значение
    /// `{"#",0e704aa2…,{1,<простое>,<значение>,<тип>,<uuid значения>,<представление>,<картинка>}}`: простое —
    /// `{1,1,{"S"|"N"|"B"|"D"…}|{"U"},0…,0…}`, значение перечисления — `{1,0,{"U"},<TypeId ссылки>,<uuid значения>}`.
    pub(super) fn choice_list(&self, e: Option<&El>) -> R<Node> {
        let mut items = Vec::new();
        let mut tails = Vec::new();
        if let Some(e) = e {
            for it in e.elems() {
                if it.name != "xr:Item" {
                    return Err(err(it, "ожидался <xr:Item>"));
                }
                let mut p: HashMap<&str, &El> = HashMap::new();
                for c in it.elems() {
                    if p.insert(c.name.as_str(), c).is_some() || !matches!(c.name.as_str(), "xr:Presentation" | "xr:CheckState" | "xr:Value") {
                        return Err(err(c, "в элементе списка выбора"));
                    }
                }
                let check = p.get("xr:CheckState").map(|c| num(c)).transpose()?.unwrap_or_else(|| "0".into());
                let pres = p.get("xr:Presentation").map(|c| c.text()).transpose()?.unwrap_or_default();
                let val = p.get("xr:Value").ok_or_else(|| err(it, "нет <xr:Value>"))?;
                items.extend([s(pres), self.choice_value(val)?]);
                tails.push(l(vec![a(check), picture_none()]));
            }
            if items.is_empty() {
                e.text()?;
            }
        }
        let mut v = vec![a("3"), an(tails.len())];
        v.extend(items);
        v.extend(tails);
        Ok(l(v))
    }

    /// Связи параметров выбора: `{<версия>,N,("<имя>",<путь без скобок>,<изменение>[,"",""])…}` (5006 — без,
    /// 5007 — с двумя пустыми строками).
    fn choice_links(&self, e: Option<&El>, ver: &str) -> R<Node> {
        let mut v = vec![a(ver), a("0")];
        let mut n = 0;
        if let Some(e) = e {
            for link in e.elems() {
                if link.name != "xr:Link" {
                    return Err(err(link, "ожидался <xr:Link>"));
                }
                let mut p: HashMap<&str, &El> = HashMap::new();
                for c in link.elems() {
                    if p.insert(c.name.as_str(), c).is_some() || !matches!(c.name.as_str(), "xr:Name" | "xr:DataPath" | "xr:ValueChange") {
                        return Err(err(c, "в связи параметров выбора"));
                    }
                }
                let dp = p.get("xr:DataPath").ok_or_else(|| err(link, "нет <xr:DataPath>"))?;
                if dp.req_attr("xsi:type")? != "xs:string" {
                    return Err(err(dp, "путь связи не строкой"));
                }
                v.push(s(p.get("xr:Name").ok_or_else(|| err(link, "нет <xr:Name>"))?.text()?));
                v.extend(self.data_path(dp)?.list().iter().cloned());
                v.push(match p.get("xr:ValueChange") {
                    None => return Err(err(link, "нет <xr:ValueChange>")),
                    Some(c) => token(c, &[("Clear", 0), ("DontChange", 1)])?,
                });
                if ver == "5007" {
                    v.extend([s(""), s("")]);
                }
                n += 1;
            }
            if n == 0 {
                e.text()?;
            }
        }
        v[1] = an(n);
        Ok(l(v))
    }

    /// Параметры выбора `{0,N,("<имя>",<значение>)…}`.
    fn choice_params(&self, e: Option<&El>) -> R<Node> {
        let mut v = vec![a("0"), a("0")];
        let mut n = 0;
        if let Some(e) = e {
            for it in e.elems() {
                if it.name != "app:item" {
                    return Err(err(it, "ожидался <app:item>"));
                }
                v.push(s(it.req_attr("name")?));
                v.push(self.choice_value(it.req("app:value")?)?);
                for c in it.elems() {
                    if c.name != "app:value" {
                        return Err(err(c, "в параметре выбора"));
                    }
                }
                n += 1;
            }
            if n == 0 {
                e.text()?;
            }
        }
        v[1] = an(n);
        Ok(l(v))
    }

    fn choice_value(&self, e: &El) -> R<Node> {
        // Неопределено вместо значения списка выбора — `{"U"}` (обратное value_xml выгрузки)
        if e.attr("xsi:nil") == Some("true") && e.attrs.len() == 1 {
            e.text()?;
            return Ok(l(vec![s("U")]));
        }
        if e.req_attr("xsi:type")? != "FormChoiceListDesTimeValue" {
            return Err(err(e, "значение списка выбора не FormChoiceListDesTimeValue"));
        }
        let mut pres = None;
        let mut value = None;
        for c in e.elems() {
            match c.name.as_str() {
                "Presentation" if pres.is_none() => pres = Some(c),
                "Value" if value.is_none() => value = Some(c),
                _ => return Err(err(c, "в значении списка выбора")),
            }
        }
        let value = value.ok_or_else(|| err(e, "нет <Value>"))?;
        let zero = || a(crate::xml::common::ZERO);
        let (simple, v, ty, id) = if value.attr("xsi:nil") == Some("true") {
            value.text()?;
            ("1", l(vec![s("U")]), zero(), zero())
        } else {
            let ty = value.req_attr("xsi:type")?;
            let t = if ty == "v8:FixedArray" { String::new() } else { value.text()? };
            match ty {
                "xs:string" => ("1", l(vec![s("S"), s(crlf(&t))]), zero(), zero()),
                "xs:decimal" => ("1", l(vec![s("N"), a(num_text(value)?)]), zero(), zero()),
                "xs:boolean" => ("1", l(vec![s("B"), a(if boolean(value)? { "1" } else { "0" })]), zero(), zero()),
                // `ГГГГ-ММ-ДДTчч:мм:сс` → `{"D","ГГГГММДДччммсс"}` (обратное `value` выгрузки)
                "xs:dateTime" => {
                    let d: String = t.chars().filter(char::is_ascii_digit).collect();
                    let shape = t.len() == 19 && t.char_indices().all(|(i, c)| match i { 4 | 7 => c == '-', 10 => c == 'T', 13 | 16 => c == ':', _ => c.is_ascii_digit() });
                    if !shape {
                        return Err(err(value, &format!("дата {t:?}")));
                    }
                    ("1", l(vec![s("D"), a(d)]), zero(), zero())
                }
                "v8:FixedArray" => {
                    let mut items = vec![a("0")];
                    for x in value.elems() {
                        if x.name != "v8:Value" {
                            return Err(err(x, "в фиксированном массиве"));
                        }
                        items.push(self.choice_value(x)?);
                    }
                    items[0] = an(items.len() - 1);
                    ("1", l(vec![s("#"), a("4500381b-db30-4a10-9db4-990038032acf"), l(items)]), zero(), zero())
                }
                "xr:DesignTimeRef" if t.ends_with(".EmptyRef") => {
                    let owner = t.strip_suffix(".EmptyRef").unwrap_or_default();
                    let (class, name) = owner.split_once('.').unwrap_or_default();
                    let ty = self.x.types.get(&format!("{class}Ref.{name}")).ok_or_else(|| err(value, &format!("нет типа {owner}")))?;
                    ("0", l(vec![s("U")]), a(ty.0.clone()), zero())
                }
                // `<тип>.<uuid>` — тип не из конфигурации (обратное choice_value выгрузки; ЗУП ИндексацияЗаработка)
                "xr:DesignTimeRef" if t.split_once('.').is_some_and(|(g1, g2)| crate::brace::is_guid(g1) && crate::brace::is_guid(g2)) => {
                    let (ty, id) = t.split_once('.').unwrap_or_default();
                    ("0", l(vec![s("U")]), a(ty), a(id))
                }
                "xr:DesignTimeRef" if !t.contains(".EnumValue.") => {
                    // предопределенный элемент: `<Класс>.<Имя>.<Предопределенный>`
                    let id = self.x.predefined.get(&t).ok_or_else(|| err(value, &format!("значение {t} пока не загружается")))?;
                    let owner = t.rsplit_once('.').map(|(o, _)| o).unwrap_or_default();
                    let (class, name) = owner.split_once('.').unwrap_or_default();
                    let ty = self.x.types.get(&format!("{class}Ref.{name}")).ok_or_else(|| err(value, &format!("нет типа {owner}")))?;
                    ("0", l(vec![s("U")]), a(ty.0.clone()), a(id.clone()))
                }
                "xr:DesignTimeRef" => {
                    let (owner, _) = t.split_once(".EnumValue.").ok_or_else(|| err(value, &format!("значение {t} пока не загружается")))?;
                    let ty = self.x.types.get(&format!("EnumRef.{}", owner.strip_prefix("Enum.").unwrap_or(owner))).ok_or_else(|| err(value, &format!("нет типа {owner}")))?;
                    let id = self.x.id(&t).ok_or_else(|| err(value, &format!("нет значения {t}")))?;
                    ("0", l(vec![s("U")]), a(ty.0.clone()), a(id))
                }
                // вид счета: Active 0, Passive 1, ActivePassive 2 (обратное `value` выгрузки)
                "ent:AccountType" => {
                    let k = ["Active", "Passive", "ActivePassive"].iter().position(|x| *x == t).ok_or_else(|| err(value, &format!("вид счета {t:?}")))?;
                    ("1", l(vec![s("#"), a("872f7198-7083-4e3e-b57e-a2a9802c769e"), an(k)]), zero(), zero())
                }
                // пустой список значений (обратное `value` выгрузки; непустой не встречался)
                "xr:ValueList" if t.is_empty() && value.peek_elems().next().is_none() => {
                    ("1", crate::brace::parse(r##"{"#",4772b3b4-f4a3-49c0-a1a5-8cb5961511a3,{6,1e512aab-1b41-4ef6-9375-f0137be9dd91,0,0,{0},{"Pattern"},0,-1}}"##)?, zero(), zero())
                }
                o => return Err(err(value, &format!("значение {o} списка выбора пока не загружается"))),
            }
        };
        Ok(l(vec![s("#"), a("0e704aa2-07bd-48b9-8223-a0212c4d5fc2"), l(vec![a("1"), a(simple), v, ty, id, ml(pres)?, picture_none()])]))
    }

    /// Свойства переключателя (версия 11).
    fn radio_sub(&self, p: &Props) -> R<(fn(&str) -> bool, Node)> {
        let v = vec![
            a("11"),
            self.choice_list(p.get("ChoiceList"))?,
            p.num("ColumnsCount", "0")?,
            p.color(self.x, "TextColor")?,
            p.font(self.x, "Font")?,
            color_auto(),
            p.num("ItemHeight", "0")?,
            p.tok("RadioButtonType", &[("Auto", 0), ("RadioButtons", 1), ("Tumbler", 2)], 0)?,
            color_auto(),
            p.num("ItemTitleHeight", "0")?,
            p.num("ItemWidth", "0")?,
            p.tri("EqualColumnsWidth")?,
            p.tok("TumblerRepresentation",&[("Auto",0),("Text",1),("Picture",2)],0)?,
            p.tok("Orientation",&[("Vertical",0),("Horizontal",1),("HorizontalIfPossible",2),("AlwaysHorizontal",3)],1)?,
            p.tri("EqualColumnsWidth")?,
        ];
        Ok((|_| true, l(v)))
    }

    /// Свойства поля картинки (версия 12); TextColor выгрузка пишет равным PictureColor.
    fn picture_sub(&self, e: &El, p: &Props) -> R<(fn(&str) -> bool, Node)> {
        if p.get("TextColor").map(txt).transpose()? != p.get("PictureColor").map(txt).transpose()? {
            return Err("TextColor поля картинки не равен PictureColor".into());
        }
        let v = vec![
            a("12"),
            p.num("Width", "0")?,
            p.num("Height", "0")?,
            p.b("HorizontalStretch", "1")?,
            p.b("VerticalStretch", "1")?,
            self.pic(e, p, "ValuesPicture")?,
            p.tok("PictureSize", super::platform::PICTURE_SIZE, 0)?,
            p.b("Zoomable", "0")?,
            p.b("Hyperlink", "0")?,
            p.ml("NonselectedPictureText")?,
            p.color(self.x, "TextColor")?,
            p.color(self.x, "BorderColor")?,
            p.font(self.x, "Font")?,
            border(p.get("Border"), 1)?,
            a("0"),
            p.b("EnableDrag", "0")?,
            events_where(p.get("Events"), &|n| n != "OnChange", &|n| item_event("PictureField", n))?,
            p.b("AutoMaxWidth", "1")?,
            p.num("MaxWidth", "0")?,
            a("0"),
            p.b("AutoMaxHeight", "1")?,
            p.num("MaxHeight", "0")?,
            p.tok("FileDragMode", &[("AsFile", 0)], 1)?,
            p.num("ImageScale", "100")?,
            a("0"),
            p.color(self.x, "PictureColor")?,
        ];
        Ok((|n| n == "OnChange", l(v)))
    }

    /// Свойства поля индикатора (версия 4).
    fn progress_sub(&self, p: &Props) -> R<(fn(&str) -> bool, Node)> {
        // раскладка mngcore FUN_180310e40 (версия 4)
        let v = vec![
            a("4"),
            p.num("Width", "32")?,
            p.num("Height", "1")?,
            p.b("HorizontalStretch", "1")?,
            p.b("VerticalStretch", "0")?,
            p.num("MinValue", "0")?,
            p.num("MaxValue", "100")?,
            p.tok("Orientation", &[("Horizontal", 0)], 0)?,
            p.tok("Representation", &[("Smooth", 1)], 1)?,
            p.b("ShowPercent", "0")?,
            p.color(self.x, "BorderColor")?,
            p.b("AutoMaxWidth", "1")?,
            p.num("MaxWidth", "0")?,
            p.num("MinWidth", "0")?,
            p.b("AutoMaxHeight", "1")?,
            p.num("MaxHeight", "0")?,
        ];
        Ok((|_| true, l(v)))
    }

    /// Свойства поля диаграммы (версия 1); обратное выгрузке (вид 11): {1,Width,Height,HorizontalStretch,VerticalStretch,
    /// события,AutoMaxWidth,MaxWidth,0,AutoMaxHeight,MaxHeight}.
    fn chart_sub(&self, p: &Props) -> R<(fn(&str) -> bool, Node)> {
        let v = vec![
            a("1"),
            p.num("Width", "50")?,
            p.num("Height", "10")?,
            p.b("HorizontalStretch", "1")?,
            p.b("VerticalStretch", "1")?,
            events(p.get("Events"), &|n| item_event("ChartField", n))?,
            p.b("AutoMaxWidth", "1")?,
            p.num("MaxWidth", "0")?,
            a("0"),
            p.b("AutoMaxHeight", "1")?,
            p.num("MaxHeight", "0")?,
        ];
        Ok((|_| false, l(v)))
    }

    /// Свойства поля диаграммы Ганта (версия 3, mngcore запись FUN_1803124c0): {3,Width,Height,HorizontalStretch,
    /// VerticalStretch,события,AutoMaxWidth,MaxWidth,MinWidth,AutoMaxHeight,MaxHeight,TableLocation,ValuesSelectionMode,
    /// IntervalsSelectionMode,ShowHorizontalLinesFlag,ShowVerticalLinesFlag}.
    fn gantt_sub(&self, p: &Props) -> R<(fn(&str) -> bool, Node)> {
        let sel = &[("Auto", 0), ("Multiple", 1), ("Single", 2), ("None", 3)];
        let v = vec![
            a("3"),
            p.num("Width", "50")?,
            p.num("Height", "10")?,
            p.b("HorizontalStretch", "1")?,
            p.b("VerticalStretch", "1")?,
            events_where(p.get("Events"), &|n| n != "OnChange", &|n| item_event("GanttChartField", n))?,
            p.b("AutoMaxWidth", "1")?,
            p.num("MaxWidth", "0")?,
            p.num("MinWidth", "0")?,
            p.b("AutoMaxHeight", "1")?,
            p.num("MaxHeight", "0")?,
            p.tok("TableLocation", &[("Auto", 0), ("Left", 1), ("Right", 2), ("None", 3)], 0)?,
            p.tok("ValuesSelectionMode", sel, 0)?,
            p.tok("IntervalsSelectionMode", sel, 0)?,
            p.tok("ShowHorizontalLinesFlag", &[("false", 0), ("true", 1)], 2)?,
            p.tok("ShowVerticalLinesFlag", &[("false", 0), ("true", 1)], 2)?,
        ];
        Ok((|n| n == "OnChange", l(v)))
    }

    /// Исключенные команды поля `{N,<guid>…}` по возрастанию GUID (как у таблицы); GUID — стандартная команда поля
    /// (`platform::field_command`).
    fn field_command_set(&self, cs: Option<&El>, tag: &str) -> R<Node> {
        let Some(cs) = cs else { return Ok(l(vec![a("0")])) };
        let mut gs = Vec::new();
        for c in cs.elems() {
            if c.name != "ExcludedCommand" {
                return Err(err(c, "ожидалась <ExcludedCommand>"));
            }
            let n = c.text()?;
            gs.push(super::platform::field_command(tag, &n).ok_or_else(|| err(c, &format!("исключенная команда поля {n} не встречена")))?);
        }
        gs.sort_unstable();
        if gs.is_empty() || gs.windows(2).any(|w| w[0] == w[1]) {
            return Err(err(cs, "пустой состав или повтор исключенной команды"));
        }
        let mut n = vec![an(gs.len())];
        n.extend(gs.into_iter().map(a));
        Ok(l(n))
    }

    /// Свойства поля форматированного документа (версия 1); обратное выгрузке (form.rs, вид 17): OnChange — в общей
    /// части поля, как у прочих полей.
    fn formatted_doc_sub(&self, p: &Props) -> R<(fn(&str) -> bool, Node)> {
        let v = vec![
            a("1"),
            p.num("Width", "50")?,
            p.num("Height", "10")?,
            p.b("HorizontalStretch", "1")?,
            p.b("VerticalStretch", "1")?,
            a("0"),
            color_auto(),
            color_auto(),
            p.color(self.x, "BorderColor")?,
            p.font(self.x, "Font")?,
            events_where(p.get("Events"), &|n| n != "OnChange", &|n| item_event("FormattedDocumentField", n))?,
            p.b("AutoMaxWidth", "1")?,
            p.num("MaxWidth", "0")?,
            a("0"),
            p.b("AutoMaxHeight", "1")?,
            p.num("MaxHeight", "0")?,
        ];
        Ok((|n| n == "OnChange", l(v)))
    }

    /// Свойства поля календаря (версия 6); обратное выгрузке (вид 8). Поля 3 и 19 выгрузка знает только оба 1 или оба
    /// 0 (AutoMaxWidth и HorizontalStretch false, какое какое — не установлено) — иное отказ.
    // Поля 3 и 19 — AutoMaxWidth и HorizontalStretch в неизвестном порядке; загрузка принимает только сочетания, где
    // порядок не важен (оба по умолчанию — 1, оба false — 0), как и выгрузка — значит, правило не зависит от эталона.
    fn calendar_sub(&self, p: &Props) -> R<(fn(&str) -> bool, Node)> {
        let pair = match (p.get("AutoMaxWidth").map(boolean).transpose()?, p.get("HorizontalStretch").map(boolean).transpose()?) {
            (None, None) => "1",
            (Some(false), Some(false))
                if p.get("SelectionMode").is_none()
                    && p.get("ShowCurrentDate").is_none()
                    && p.get("ShowMonthsPanel").is_none()
                    && p.get("WidthInMonths").is_none()
                    && p.get("HeightInMonths").is_none() =>
            {
                "0"
            }
            _ => return Err("AutoMaxWidth и HorizontalStretch календаря: сочетание не выведено".into()),
        };
        let v = vec![
            a("6"),
            p.num("Width", "16")?,
            p.num("Height", "9")?,
            a(pair),
            a("1"),
            p.tok("SelectionMode", &[("Interval", 2)], 0)?,
            p.b("ShowCurrentDate", "1")?,
            a("1"),
            a("00010101000000"),
            a("00010101000000"),
            font_auto(),
            color_auto(),
            a("0"),
            a("0"),
            events_where(p.get("Events"), &|n| n != "OnChange", &|n| item_event("CalendarField", n))?,
            p.b("ShowMonthsPanel", "0")?,
            p.num("WidthInMonths", "1")?,
            p.num("HeightInMonths", "1")?,
            border(None, 1)?,
            a(pair),
            a("0"),
            a("0"),
            a("1"),
            a("0"),
        ];
        Ok((|n| n == "OnChange", l(v)))
    }

    /// Свойства поля PDF-документа (версия 1); обратное выгрузке (вид 20): прочие поля постоянны.
    fn pdf_sub(&self, p: &Props) -> R<(fn(&str) -> bool, Node)> {
        // ViewStatusLocation=None загрузка платформы пишет не 1, как выгрузка читает, а 7274574 = 0x006F004E — первые
        // два символа UTF-16 «No» как число (эталон ЭлектронныйДокументИсходящийЭДО, БП ОтправитьФайлы).
        // shortcut: код чтения не найден, прочие значения (Top, Bottom) не встречены — отказ
        let status = match p.get("ViewStatusLocation").map(|v| (v, v.text())) {
            None => "0",
            Some((_, Ok(t))) if t == "None" => "7274574",
            Some((v, _)) => return Err(err(v, "ViewStatusLocation поля PDF: значение в записи не выведено")),
        };
        let v = vec![
            // раскладка mngcore FUN_180314390 (версия 1)
            a("1"),
            p.num("Width", "50")?,
            p.num("Height", "10")?,
            p.color(self.x, "BorderColor")?,
            p.tok("Output", &[("Auto", 0), ("Enable", 1), ("Disable", 2)], 0)?,
            events(p.get("Events"), &|n| item_event("PDFDocumentField", n))?,
            p.b("AutoMaxWidth", "1")?,
            p.num("MaxWidth", "0")?,
            p.num("MinWidth", "0")?,
            p.b("AutoMaxHeight", "1")?,
            p.num("MaxHeight", "0")?,
            p.b("HorizontalStretch", "1")?,
            p.b("VerticalStretch", "1")?,
            a(status),
        ];
        Ok((|_| false, l(v)))
    }

    /// Свойства поля планировщика (версия 1); обратное выгрузке (вид 19): прочие поля постоянны (одно поле в БП).
    fn planner_sub(p: &Props) -> R<(fn(&str) -> bool, Node)> {
        let v = vec![a("1"), a("50"), a("10"), a("1"), a("1"), a("0"), a("0"), events(p.get("Events"), &|n| item_event("PlannerField", n))?, a("0"), a("0"), a("0")];
        Ok((|_| false, l(v)))
    }

    /// Свойства полосы регулирования (версия 2); обратное выгрузке (вид 10); событий у вида нет.
    fn track_bar_sub(&self, p: &Props) -> R<(fn(&str) -> bool, Node)> {
        let v = vec![
            // раскладка mngcore FUN_180311670 (версия 2)
            a("2"),
            p.num("Width", "32")?,
            p.num("Height", "2")?,
            p.b("HorizontalStretch", "1")?,
            p.b("VerticalStretch", "0")?,
            p.num("MinValue", "0")?,
            p.num("MaxValue", "100")?,
            p.num("Step", "1")?,
            p.tok("Orientation", &[("Horizontal", 0)], 0)?,
            p.num("LargeStep", "10")?,
            p.num("MarkingStep", "5")?,
            p.tok("MarkingAppearance", &[("TopLeft", 1), ("BottomRight", 2)], 2)?,
            p.color(self.x, "BorderColor")?,
            p.b("AutoMaxWidth", "1")?,
            p.num("MaxWidth", "0")?,
            p.num("MinWidth", "0")?,
            p.b("AutoMaxHeight", "1")?,
            p.num("MaxHeight", "0")?,
        ];
        Ok((|_| true, l(v)))
    }

    /// Свойства поля графической схемы (версия 3); обратное выгрузке (вид 14): Edit выгрузка пишет всегда false.
    fn schema_sub(&self, p: &Props) -> R<(fn(&str) -> bool, Node)> {
        if p.get("Edit").map(boolean).transpose()? != Some(false) {
            return Err("Edit графической схемы не false: не выведено".into());
        }
        let v = vec![
            a("3"),
            p.num("Width", "50")?,
            p.num("Height", "10")?,
            a("0"),
            a("0"),
            color_auto(),
            events_where(p.get("Events"), &|n| n != "OnChange", &|n| item_event("GraphicalSchemaField", n))?,
            a("1"),
            a("0"),
            a("0"),
            a("1"),
            a("0"),
            a("1"),
            a("1"),
        ];
        Ok((|n| n == "OnChange", l(v)))
    }

    /// Свойства поля текстового документа (версия 5); события — в общей части поля.
    fn text_doc_sub(&self, p: &Props) -> R<(fn(&str) -> bool, Node)> {
        let v = vec![
            a("5"),
            p.num("Width", "50")?,
            p.num("Height", "10")?,
            // как выгрузка (xml/form.rs, вид 7): растягивание 3, 4; AutoMaxHeight 13, MaxHeight 14
            p.b("HorizontalStretch", "1")?,
            p.b("VerticalStretch", "1")?,
            a("0"),
            color_auto(),
            color_auto(),
            color_auto(),
            p.font(self.x, "Font")?,
            p.b("AutoMaxWidth", "1")?,
            p.num("MaxWidth", "0")?,
            a("0"),
            p.b("AutoMaxHeight", "1")?,
            p.num("MaxHeight", "0")?,
            l(vec![a("0"), a("1"), a("0")]),
        ];
        Ok((|_| true, l(v)))
    }

    /// Свойства поля HTML-документа (версия 4); события — все в свойствах вида.
    fn html_sub(&self, p: &Props) -> R<(fn(&str) -> bool, Node)> {
        let v = vec![
            a("4"),
            p.num("Width", "50")?,
            p.num("Height", "10")?,
            p.color(self.x, "BorderColor")?,
            p.tok("Output", &[("Enable", 1), ("Disable", 2)], 0)?,
            events(p.get("Events"), &|n| item_event("HTMLDocumentField", n))?,
            p.b("AutoMaxWidth", "1")?,
            p.num("MaxWidth", "0")?,
            a("0"),
            p.b("AutoMaxHeight", "1")?,
            p.num("MaxHeight", "0")?,
            p.b("HorizontalStretch", "1")?,
            p.b("VerticalStretch", "1")?,
            border(None, 0)?,
        ];
        Ok((|_| false, l(v)))
    }

    /// Свойства поля табличного документа (версия 15).
    fn spreadsheet_sub(&self, p: &Props) -> R<(fn(&str) -> bool, Node)> {
        const SEL: &[(&str, i64, i64)] = &[("WhenActive", 0, 0), ("DontShow", 1, 2), ("WhenMultipleCellsSelected", 1, 3)];
        let sel = p.get("SelectionShowMode").map(txt).transpose()?;
        let (s11, s30) = match sel.as_deref() {
            None => (1, 1),
            Some(v) => SEL.iter().find(|r| r.0 == v).map(|r| (r.1, r.2)).ok_or_else(|| format!("SelectionShowMode {v}"))?,
        };
        let bar = |k: &str| -> R<(Node, Node)> {
            Ok(match p.get(k).map(boolean).transpose()? {
                None => (a("1"), a("2")),
                Some(true) => (a("1"), a("1")),
                Some(false) => (a("0"), a("0")),
            })
        };
        let (v7, v28) = bar("VerticalScrollBar")?;
        let (h8, h29) = bar("HorizontalScrollBar")?;
        let v = vec![
            a("15"),
            p.num("Width", "50")?,
            p.num("Height", "10")?,
            p.b("HorizontalStretch", "1")?,
            p.b("VerticalStretch", "1")?,
            p.b("ShowGrid", "0")?,
            p.b("ShowHeaders", "0")?,
            v7,
            h8,
            a("0"),
            p.b("Protection", "0")?,
            an(s11),
            p.tok("Output", &[("Enable", 1), ("Disable", 2)], 0)?,
            p.b("Edit", "0")?,
            p.b("ShowGroups", "1")?,
            p.color(self.x, "BorderColor")?,
            // выгрузка: поле 16 — EnableStartDrag, 17 — EnableDrag
            p.b("EnableStartDrag", "1")?,
            p.b("EnableDrag", "1")?,
            events_where(p.get("Events"), &|n| n != "OnChange", &|n| item_event("SpreadSheetDocumentField", n))?,
            p.tok("ViewScalingMode", &[("Normal", 1)], 0)?,
            p.b("AutoMaxWidth", "1")?,
            p.num("MaxWidth", "0")?,
            a("0"),
            p.b("AutoMaxHeight", "1")?,
            p.num("MaxHeight", "0")?,
            p.b("ShowCellNames", "0")?,
            p.b("ShowRowAndColumnNames", "0")?,
            a("0"),
            v28,
            h29,
            an(s30),
            // DrawingSelectionShowMode (поле 31, раскладка mngcore FUN_1802f8630 вид 15): умолчание Show
            p.tok("DrawingSelectionShowMode", &[("Show", 0)], 2)?,
            a("0"),
            a("0"),
        ];
        Ok((|n| n == "OnChange", l(v)))
    }

    /// Свойства поля надписи (версия 12).
    fn label_sub(&self, p: &Props) -> R<(fn(&str) -> bool, Node)> {
        let v = vec![
            a("12"),
            p.num("Width", "0")?,
            p.num("Height", "0")?,
            p.tri("HorizontalStretch")?,
            p.tri("VerticalStretch")?,
            p.tri("MarkNegatives")?,
            p.ml("Format")?,
            p.b("Hiperlink", "0")?,
            p.color(self.x, "TextColor")?,
            p.color(self.x, "BackColor")?,
            p.font(self.x, "Font")?,
            p.tri("PasswordMode")?,
            events_where(p.get("Events"), &|n| n != "OnChange", &|n| item_event("LabelField", n))?,
            p.color(self.x, "BorderColor")?,
            border(p.get("Border"), 0)?,
            p.b("AutoMaxWidth", "1")?,
            p.num("MaxWidth", "0")?,
            a("0"),
            p.b("AutoMaxHeight", "1")?,
            p.num("MaxHeight", "0")?,
            p.tri("UseCopy")?,
        ];
        Ok((|n| n == "OnChange", l(v)))
    }

    /// Свойства флажка (версия 13).
    fn check_sub(&self, p: &Props) -> R<(fn(&str) -> bool, Node)> {
        let v = vec![
            // раскладка флажка (вид 13, версия 13)
            a("13"),
            p.b("ThreeState", "0")?,
            p.color(self.x, "BorderColor")?,
            p.color(self.x, "BackColor")?,
            p.tok("CheckBoxType", &[("Switcher", 0), ("CheckBox", 1), ("Tumbler", 2)], 0)?,
            p.ml("EditFormat")?,
            p.color(self.x, "TextColor")?,
            p.font(self.x, "Font")?,
            p.num("ItemTitleHeight", "0")?,
            p.num("ItemWidth", "0")?,
            p.num("ItemHeight", "0")?,
            p.tri("EqualItemsWidth")?,
            p.tok("CheckBoxType", &[("CheckBox", 1), ("Tumbler", 2), ("Switcher", 3)], 0)?,
            a("0"),
            a("2"),
        ];
        Ok((|_| true, l(v)))
    }

    /// Свойства поля ввода (версия 38) и какие события — в общей части поля.
    /// Множественные значения поля ввода (обратное `multiple_values` выгрузки): нет свойств — `{0}`; иначе
    /// `{1,2,AllowInputEmpty,0,ShowCheckBoxesInDropList (0 false, 2 авто),…,[9] путь значения,…,[15] путь
    /// представления,…}`, путь — `{1,{<колонка>}}` реквизита-таблицы из DataPath поля (путь из одной части).
    fn multiple_values(&self, p: &Props) -> R<Node> {
        const KEYS: &[&str] = &["AllowInputEmptyMultipleValues", "ShowCheckBoxesInDropList", "MultipleValueDataPath", "MultipleValuePresentDataPath"];
        if !KEYS.iter().any(|k| p.get(k).is_some()) {
            return Ok(l(vec![a("0")]));
        }
        let base = p.get("DataPath").map(|d| d.peek_text().unwrap_or_default()).unwrap_or_default();
        let path = |k: &str| -> R<Node> {
            let Some(e) = p.get(k) else { return Ok(l(vec![a("0")])) };
            let t = e.text()?;
            let col = t.strip_prefix(&format!("{base}.")).filter(|c| !base.contains('.') && !c.contains('.'));
            let id = col.and_then(|c| self.attrs.get(&base)?.cols.iter().find(|x| x.name == c)).map(|c| c.id.clone());
            let id = id.ok_or_else(|| err(e, "путь множественного значения не колонка реквизита поля"))?;
            Ok(l(vec![a("1"), l(vec![a(id)])]))
        };
        let show = match p.get("ShowCheckBoxesInDropList").map(boolean).transpose()? {
            None => "2",
            Some(false) => "0",
            // BWA (mngcore 0x18078ae70): 1 true
            Some(true) => "1",
        };
        let pat = || l(vec![s("Pattern")]);
        Ok(l(vec![
            a("1"),
            a("2"),
            p.b("AllowInputEmptyMultipleValues", "0")?,
            a("0"),
            a(show),
            font_auto(),
            color_auto(),
            color_auto(),
            crate::brace::parse("{4,0,{0},\"\",-1,-1,1,0,\"\"}")?,
            path("MultipleValueDataPath")?,
            s(""),
            pat(),
            l(vec![a("0")]),
            s(""),
            pat(),
            path("MultipleValuePresentDataPath")?,
            s(""),
            pat(),
            a("0"),
            a("0"),
        ]))
    }

    fn input_sub(&self, e: &El, p: &Props) -> R<(fn(&str) -> bool, Node)> {
        // связь по типу `{3,<путь без скобок>,<элемент связи>}`; нет — `{3,0,0}`
        let type_link = match p.get("TypeLink") {
            None => l(vec![a("3"), a("0"), a("0")]),
            Some(t) => {
                let mut v = vec![a("3")];
                v.extend(self.data_path(t.req("xr:DataPath")?)?.list().iter().cloned());
                v.push(a(num(t.req("xr:LinkItem")?)?));
                for c in t.elems() {
                    if !matches!(c.name.as_str(), "xr:DataPath" | "xr:LinkItem") {
                        return Err(err(c, "в связи по типу"));
                    }
                }
                l(v)
            }
        };
        let available = match p.get("AvailableTypes") {
            None => l(vec![s("Pattern")]),
            Some(t) => type_desc(self.x, t)?,
        };
        let choice_form = match p.get("ChoiceForm") {
            Some(f) => crate::xml::imp::tpl::reference(self.x, f)?,
            None => a(crate::xml::common::ZERO),
        };
        let sub_events = events_where(p.get("Events"), &|n| n != "OnChange", &|n| item_event("InputField", n))?;
        let v = vec![
            a("38"),
            self.choice_list(p.get("ChoiceList"))?,
            p.num("Width", "0")?,
            p.num("Height", "0")?,
            p.tri("HorizontalStretch")?,
            p.tri("VerticalStretch")?,
            p.b("Wrap", "1")?,
            p.tri("PasswordMode")?,
            p.tri("MultiLine")?,
            p.tri("ExtendedEdit")?,
            p.tri("MarkNegatives")?,
            p.tri("ChoiceListButton")?,
            p.tri("ChoiceButton")?,
            p.tri("ClearButton")?,
            p.tri("SpinButton")?,
            p.tri("OpenButton")?,
            bound(p.get("MinValue"))?,
            bound(p.get("MaxValue"))?,
            s(p.get("Mask").map(txt).transpose()?.unwrap_or_default()),
            p.b("ListChoiceMode", "0")?,
            self.pic(e, p, "ChoiceButtonPicture")?,
            p.num("ChoiceListHeight", "0")?,
            p.num("DropListWidth", "0")?,
            p.tri("QuickChoice")?,
            p.tok("ChoiceFoldersAndItems", &[("Items", 0), ("Folders", 1), ("FoldersAndItems", 2)], 3)?,
            choice_form,
            self.choice_links(p.get("ChoiceParameterLinks"), "5006")?,
            self.choice_params(p.get("ChoiceParameters"))?,
            p.tri("AutoChoiceIncomplete")?,
            p.ml("Format")?,
            p.ml("EditFormat")?,
            p.tri("AutoMarkIncomplete")?,
            p.b("ChooseType", "1")?,
            p.tok("IncompleteChoiceMode", &[("OnActivate", 1)], 0)?,
            available,
            p.b("TypeDomainEnabled", "1")?,
            sub_events,
            p.color(self.x, "TextColor")?,
            p.color(self.x, "BackColor")?,
            p.color(self.x, "BorderColor")?,
            p.font(self.x, "Font")?,
            p.b("TextEdit", "1")?,
            type_link,
            p.tok("EditTextUpdate", &[("DontUse", 1), ("OnValueChange", 2), ("Always", 3)], 0)?,
            p.ml("InputHint")?,
            p.tri("CreateButton")?,
            p.tok("ChoiceButtonRepresentation", &[("ShowInDropList", 1), ("ShowInDropListAndInInputField", 2), ("ShowInInputField", 3)], 0)?,
            p.tri("DropListButton")?,
            p.tok("ChoiceHistoryOnInput", &[("DontUse", 1)], 0)?,
            p.b("AutoMaxWidth", "1")?,
            p.num("MaxWidth", "0")?,
            a("0"),
            p.b("AutoMaxHeight", "1")?,
            p.num("MaxHeight", "0")?,
            p.tok("HeightControlVariant", &[("UseHeightInFormRows", 1), ("UseContentHeight", 2)], 0)?,
            p.tok("AutoShowClearButtonMode", &[("FilledOnly", 2)], 0)?,
            p.tok("AutoShowOpenButtonMode", &[("Always", 1), ("FilledOnly", 2)], 0)?,
            p.tok("AutoCorrectionOnTextInput", &[("DontUse", 2)], 0)?,
            p.tok("SpellCheckingOnTextInput", &[("DontUse", 2)], 0)?,
            a("0"),
            p.tok("SpecialTextInputMode", super::platform::SPECIAL_TEXT_INPUT_MODE, 0)?,
            p.get("OnScreenKeyboardReturnKeyText").map(|e|token_of(e,crate::xml::form::KEYBOARD_RETURN)).unwrap_or_else(||Ok(a("0")))?,
            self.multiple_values(p)?,
            p.get("AutofillHint").map(|e|token_of(e,crate::xml::form::AUTOFILL_HINT)).unwrap_or_else(||Ok(a("0")))?,
            self.choice_links(p.get("ChoiceParameterLinks"), "5007")?,
            p.b("ExtendedEditMultipleValues", "0")?,
            l(vec![a("1"), a("0")]),
            self.pic(e,p,"Picture")?,
            l(vec![a("1"), a("0")]),
            a("1"),
            a("0"),
        ];
        Ok((|n| n == "OnChange", l(v)))
    }
}

/// Свойства поля периода (версия 2): обратное выгрузке (вид 18) — свойств вида в XML нет; шрифт и цвет — по
/// умолчанию (выгрузка сверяет их старые версии `{7,3,0,1,100}`, `{3,4,{0}}`), прочие поля постоянны.
fn period_sub() -> R<(fn(&str) -> bool, Node)> {
    let mut v = vec![a("2"), a("0"), a("0"), font_auto(), color_auto()];
    v.extend(crate::brace::parse("{{0,1,0},1,1,{3,0,{0},1,1,0,48312c09-257f-4b29-b280-284dd89efc1e},1,0,0,1,0}")?.list().iter().cloned());
    Ok((|_| false, l(v)))
}

/// Панель действий выделенных строк поля (группа вида 10 без id; имя — от имени поля).
fn rows_panel(name: &str) -> Node {
    l(vec![
        a("22"),
        l(vec![a("0")]),
        a("1"),
        a("0"),
        a("0"),
        a("10"),
        s(format!("{name}{ROWS_PANEL}")),
        l(vec![a("1"), a("0")]),
        l(vec![a("1"), a("0")]),
        a("0"),
        a("1"),
        a("0"),
        a("0"),
        a("0"),
        a("2"),
        a("2"),
        color_auto(),
        font_auto(),
        l(vec![a("0"), a("0"), a("0")]),
        a("1"),
        l(vec![a("0"), a("1"), a("0"), a("1")]),
        a("0"),
        a("1"),
        a("0"),
        a("0"),
        a("0"),
        a("3"),
        a("3"),
        a("0"),
    ])
}
