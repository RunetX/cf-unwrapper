//! Форма версии 28 (без хвоста корня; в БП 3.0.171 — 6 форм ПодробнаяИнформация планов обмена).
//!
//! У элементов таких форм в записи нет расширенной подсказки (кнопка версии 19, поле HTML-документа версии 30), а
//! платформа при выгрузке пишет ее каждой кнопке и полю: `<ExtendedTooltip name="<имя>РасширеннаяПодсказка" id="N"/>`,
//! N — с наибольшего id элемента формы + 1 по порядку элементов в XML (командная панель, затем элементы).
//! VerticalScroll корня без хвоста — useIfNecessary (`form.rs`). Сверено на всех 6 формах; в ЗУП 3.1.37 — еще обычная
//! группа и поле ввода (ВидыОтправляемыхДокументов.ФормаГруппы) и форма без элементов. Элементы других видов
//! в форме версии 28 не встречались — отказ.

/// Поле VerticalScroll хвоста корня, которого у формы версии 28 нет: 2 — useIfNecessary.
pub fn scroll() -> &'static crate::brace::Node {
    static N: std::sync::OnceLock<crate::brace::Node> = std::sync::OnceLock::new();
    N.get_or_init(|| crate::brace::Node::Atom("2".into()))
}

/// Дописывает расширенные подсказки в готовый текст формы (`<Form ...>` без закрывающего тега).
pub fn tooltips(s: &mut String) -> Result<(), String> {
    let lines: Vec<&str> = s.split("\r\n").collect();
    // Элементы — от командной панели формы до конца ChildItems формы (отступ 1).
    let from = lines.iter().position(|l| l.starts_with("\t<AutoCommandBar ")).ok_or("версия формы 28: нет командной панели")?;
    let to = match (from..lines.len()).find(|&i| lines[i] == "\t</ChildItems>") {
        Some(i) => i,
        // Без элементов формы — только командная панель (ЗУП ЗаявлениеАбонентаСпецоператораСвязи.ФормаСписка).
        None if lines[from].ends_with("/>") => from + 1,
        None => 1 + (from..lines.len()).find(|&i| lines[i] == "\t</AutoCommandBar>").ok_or("версия формы 28: нет конца командной панели")?,
    };
    let head = |l: &str| -> Option<(usize, String, String, String)> {
        let t = l.trim_start_matches('\t');
        let (tag, rest) = t.strip_prefix('<')?.split_once(" name=\"")?;
        let (name, rest) = rest.split_once("\" id=\"")?;
        let id = rest.strip_suffix("\">").or_else(|| rest.strip_suffix("\"/>"))?;
        Some((l.len() - t.len(), tag.to_string(), name.to_string(), id.to_string()))
    };
    let mut next = 1 + lines[from..to].iter().filter_map(|l| head(l)).filter_map(|h| h.3.parse::<i64>().ok()).max().unwrap_or(0);
    let mut out: Vec<String> = lines[..from].iter().map(|l| l.to_string()).collect();
    let mut i = from;
    while i < to {
        let l = lines[i];
        out.push(l.to_string());
        i += 1;
        let Some((d, tag, name, _)) = head(l) else { continue };
        if matches!(tag.as_str(), "AutoCommandBar" | "ContextMenu" | "ExtendedTooltip") {
            continue;
        }
        if !matches!(tag.as_str(), "Button" | "HTMLDocumentField" | "UsualGroup" | "InputField") || l.ends_with("/>") {
            return Err(format!("версия формы 28: расширенная подсказка у {tag} не сверена"));
        }
        let close = format!("{}</{tag}>", "\t".repeat(d));
        let end = (i..to).find(|&k| lines[k] == close).ok_or("версия формы 28: нет закрывающего тега")?;
        let child = "\t".repeat(d + 1);
        if lines[i..end].iter().any(|c| c.starts_with(&format!("{child}<ExtendedTooltip "))) {
            continue;
        }
        // Место по порядку свойств: у кнопки — последнее, у поля — перед Events, у группы — перед Events и ChildItems
        // (вложенные элементы обходятся дальше по строкам).
        let at = (i..end).find(|&k| lines[k] == format!("{child}<Events>") || lines[k] == format!("{child}<ChildItems>")).unwrap_or(end);
        out.extend(lines[i..at].iter().map(|l| l.to_string()));
        out.push(format!("{child}<ExtendedTooltip name=\"{name}РасширеннаяПодсказка\" id=\"{next}\"/>"));
        // shortcut: поле ввода занимает два номера (подсказка и следующий, в XML его нет) — по одной форме ЗУП
        // (ВидыОтправляемыхДокументов.ФормаГруппы: 3 поля, у групп — по одному); создание элементов старой версии
        // в mngcore не снято — пересмотреть, когда встретится другая форма версии 28 с полями ввода.
        next += if tag == "InputField" { 2 } else { 1 };
        i = at;
    }
    out.extend(lines[to..].iter().map(|l| l.to_string()));
    *s = out.join("\r\n");
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn tooltips() {
        let mut s = "<Form>\r\n\t<AutoCommandBar name=\"П\" id=\"-1\">\r\n\t\t<ChildItems>\r\n\t\t\t<Button name=\"К\" id=\"3\">\r\n\t\t\t\t<Type>CommandBarButton</Type>\r\n\t\t\t</Button>\r\n\t\t</ChildItems>\r\n\t</AutoCommandBar>\r\n\t<ChildItems>\r\n\t\t<HTMLDocumentField name=\"П\" id=\"1\">\r\n\t\t\t<ContextMenu name=\"М\" id=\"2\"/>\r\n\t\t</HTMLDocumentField>\r\n\t</ChildItems>".to_string();
        super::tooltips(&mut s).unwrap();
        assert!(s.contains("\t\t\t\t<Type>CommandBarButton</Type>\r\n\t\t\t\t<ExtendedTooltip name=\"КРасширеннаяПодсказка\" id=\"4\"/>\r\n\t\t\t</Button>"));
        assert!(s.contains("\t\t\t<ContextMenu name=\"М\" id=\"2\"/>\r\n\t\t\t<ExtendedTooltip name=\"ПРасширеннаяПодсказка\" id=\"5\"/>\r\n\t\t</HTMLDocumentField>"));
    }

    /// Группа — подсказка перед ChildItems, поле ввода — два номера (ЗУП ВидыОтправляемыхДокументов.ФормаГруппы).
    #[test]
    fn group_and_input_field() {
        let mut s = "<Form>\r\n\t<AutoCommandBar name=\"П\" id=\"-1\"/>\r\n\t<ChildItems>\r\n\t\t<UsualGroup name=\"Г\" id=\"5\">\r\n\t\t\t<ShowTitle>false</ShowTitle>\r\n\t\t\t<ChildItems>\r\n\t\t\t\t<InputField name=\"Н\" id=\"3\">\r\n\t\t\t\t\t<ContextMenu name=\"М\" id=\"4\"/>\r\n\t\t\t\t</InputField>\r\n\t\t\t\t<InputField name=\"К\" id=\"1\">\r\n\t\t\t\t\t<ContextMenu name=\"Л\" id=\"2\"/>\r\n\t\t\t\t</InputField>\r\n\t\t\t</ChildItems>\r\n\t\t</UsualGroup>\r\n\t\t<UsualGroup name=\"О\" id=\"6\">\r\n\t\t\t<Representation>None</Representation>\r\n\t\t</UsualGroup>\r\n\t</ChildItems>".to_string();
        super::tooltips(&mut s).unwrap();
        assert!(s.contains("\t\t\t<ShowTitle>false</ShowTitle>\r\n\t\t\t<ExtendedTooltip name=\"ГРасширеннаяПодсказка\" id=\"7\"/>\r\n\t\t\t<ChildItems>"));
        assert!(s.contains("<ExtendedTooltip name=\"НРасширеннаяПодсказка\" id=\"8\"/>"));
        assert!(s.contains("<ExtendedTooltip name=\"КРасширеннаяПодсказка\" id=\"10\"/>"));
        assert!(s.contains("<ExtendedTooltip name=\"ОРасширеннаяПодсказка\" id=\"12\"/>"));
        let mut e = "<Form>\r\n\t<AutoCommandBar name=\"П\" id=\"-1\">\r\n\t\t<Autofill>false</Autofill>\r\n\t</AutoCommandBar>\r\n\t<Attributes>\r\n\t\t<Attribute name=\"А\" id=\"1\">\r\n\t\t</Attribute>\r\n\t</Attributes>".to_string();
        let before = e.clone();
        super::tooltips(&mut e).unwrap();
        assert_eq!(e, before);
    }
}
