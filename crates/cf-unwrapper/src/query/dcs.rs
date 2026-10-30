//! Набор компоновки списка со своим запросом (dcs.dll): что он добавляет в корень дерева полей сверх полей запроса.

use super::lex::{tokens, Tok};
use super::parse::{Batch, Expr, Field, Source, Table};
use super::resolve::{check, eq, Meta, Ty};
use crate::xml::imp::index::Index;
use std::cell::{Cell, RefCell};

type R<T> = Result<T, String>;

/// Есть ли `DataParameters` в конце корня дерева полей (mngbase FUN_1802b76a0 зовет dcs FUN_1800ae580 в режиме 9).
/// dcs FUN_1800b1e00: при пустом пути и режиме не из маски 0x122 (1, 5, 8) элемент добавляется, если у какого-нибудь
/// параметра набора (FUN_1800ba550, вектор composer+0x240) байт +0x148 ≠ 0. FUN_1800ba550: параметр запроса набора —
/// +0x148 = 1, а если в схеме объявлен параметр с тем же именем — его `availableAsField` (vtable +0xa8: чтение XDTO
/// dcs FUN_180402080, запись FUN_1804030c0 пишет свойство только при «ложь» — умолчание истина); объявленный параметр,
/// которого нет в запросе, — тоже `availableAsField`. Объявленные параметры — `<Parameter>` настроек списка (mngbase
/// FUN_18067c8c0 копирует коллекцию +0x80 ULSchemeInfoImpl, FUN_18067f990, в параметры схемы). Параметр запроса —
/// `&Имя` текста (справка платформы, «Параметры запроса»). `declared` — (имя, `availableAsField`).
pub fn data_parameters(m: &dyn Meta, text: &str, b: &Batch, declared: &[(String, bool)]) -> R<bool> {
    if declared.iter().any(|(_, f)| *f) {
        return Ok(true);
    }
    // Параметры виртуальной таблицы (dcs FUN_18030c790, второй проход по источникам) — `vt_registered`. Регистрация идет
    // при автозаполнении доступных полей набора (FUN_18030c790: +0x10 посетителя, источник — `dlist::from_query`);
    // список с выключенным автозаполнением сюда не доходит (отказ).
    let vt = Vt { m, seen: Cell::new(false), names: RefCell::new(Vec::new()) };
    check(&vt, b)?;
    if vt.seen.get() {
        let mut vts = Vec::new();
        for q in &b.queries {
            for s in &q.parts {
                for f in &s.from {
                    vt_sources(f, &mut vts)?;
                }
            }
        }
        if vts.is_empty() {
            return Err("виртуальная таблица не в источниках запроса: параметры набора не выведены".into());
        }
        // одноименный объявленный параметр задает +0x148 своим `availableAsField` (FUN_1800ba550, `parameter_fields`)
        let off = |n: &str| declared.iter().any(|(k, f)| !*f && eq(k, n));
        if vt_registered(&vts)?.iter().any(|(n, alt)| !off(n) && !off(alt)) {
            return Ok(true);
        }
    }
    Ok(tokens(text)?.iter().any(|t| matches!(t, Tok::Param(p) if !declared.iter().any(|(n, _)| eq(n, p)))))
}

/// Параметры набора от виртуальных таблиц `vts` (имя, второе имя): параметр-значение описателя (FUN_18030c790: флаги
/// +0x48 — бит 1, не бит 4) без полей в выражении (`with_fields`) или не заданный набор регистрирует под именами
/// описателя (FUN_180318210: 0, 1) через FUN_18030ea00. У срезов и остатков параметр-значение — первый, Период (трасса
/// qp-2 D2: Period); у оборотов и остатков с оборотами — НачалоПериода и КонецПериода (пары имен basic.dll
/// BeginOfPeriod/НачалоПериода, EndOfPeriod/КонецПериода). shortcut: описатели (записи 0x88, FUN_1803106a0) строит не
/// dcs, флаги +0x48 оборотов не прочитаны — по справке первые два; при расхождении номеров у списка с оборотами —
/// искать построение описателей
fn vt_registered(vts: &[(String, Option<Vec<Option<Expr>>>)]) -> R<Vec<(&'static str, &'static str)>> {
    let mut out = Vec::new();
    for (name, params) in vts {
        let turnovers = ["Обороты", "Turnovers", "ОстаткиИОбороты", "BalanceAndTurnovers"].iter().any(|v| eq(v, name));
        let own: &[(&'static str, &'static str)] = if turnovers { &[("НачалоПериода", "BeginOfPeriod"), ("КонецПериода", "EndOfPeriod")] } else { &[("Период", "Period")] };
        for (i, p) in own.iter().enumerate() {
            if !with_fields(params.as_ref().and_then(|ps| ps.get(i).cloned().flatten()))? {
                out.push(*p);
            }
        }
    }
    Ok(out)
}

/// Есть ли поля в выражении параметра виртуальной таблицы (FUN_18020c340: вектор полей выражения не пуст).
fn with_fields(e: Option<Expr>) -> R<bool> {
    let Some(e) = e else { return Ok(false) };
    let d = format!("{e:?}");
    if d.contains("Sub(") {
        return Err("подзапрос в параметре виртуальной таблицы: параметры набора не выведены".into());
    }
    Ok(d.contains("Path(") || d.contains("Star(") || d.contains("Dot("))
}

/// Дети узла `DataParameters` (имена по порядку вектора параметров набора): dcs FUN_1800b1e00 по пути `DataParameters`
/// (или второму имени) отдает каждый параметр вектора composer+0x240 с байтом +0x148 ≠ 0 (FUN_1800c22f0). Вектор
/// (FUN_1800ba550) — параметры запросов наборов, объявленные параметры схемы; одноименные (по любому из двух имен)
/// сливаются, +0x148 — `availableAsField` объявленного, у прочих 1 (см. `data_parameters`). Параметр виртуальной
/// таблицы — `vt_registered`. shortcut: имя — русское (язык имени ребенка FUN_1800c22f0 не прочитан, для номеров важно
/// только число детей); число детей у параметра виртуальной таблицы, заданного выражением без полей (`СрезПоследних(&Дата)`
/// — Дата и Период), с образцом не сверено (у двух форм ERP с узлом виртуальной таблицы нет или параметры пусты) —
/// пересмотреть при расхождении номеров такого списка
pub fn parameter_fields(m: &dyn Meta, text: &str, b: &Batch, declared: &[(String, bool)]) -> R<Vec<String>> {
    // имя и второе имя параметра
    let mut names: Vec<(String, String)> = Vec::new();
    let mut add = |n: &str, alt: &str| {
        if !names.iter().any(|(k, a)| [n, alt].iter().any(|x| eq(k, x) || eq(a, x))) {
            names.push((n.to_string(), alt.to_string()));
        }
    };
    for t in tokens(text)? {
        if let Tok::Param(p) = t {
            add(&p, &p);
        }
    }
    let vt = Vt { m, seen: Cell::new(false), names: RefCell::new(Vec::new()) };
    check(&vt, b)?;
    if vt.seen.get() {
        let mut vts = Vec::new();
        for q in &b.queries {
            for s in &q.parts {
                for f in &s.from {
                    vt_sources(f, &mut vts)?;
                }
            }
        }
        if vts.is_empty() || vt.names.borrow().iter().any(|n| !vts.iter().any(|(v, _)| eq(v, n))) {
            return Err("виртуальная таблица не из срезов, остатков и оборотов: параметры набора не выведены".into());
        }
        for (n, alt) in vt_registered(&vts)? {
            add(n, alt);
        }
    }
    for (n, _) in declared {
        add(n, n);
    }
    let off = |n: &str| declared.iter().any(|(k, f)| !*f && eq(k, n));
    Ok(names.into_iter().filter(|(n, a)| !off(n) && !off(a)).map(|(n, _)| n).collect())
}

/// Поле выборки `f` своего запроса, которого нет в корне набора.
/// - Режим пароля. dcs FUN_1800c4b90 отбрасывает поле набора (режимы, кроме 7 и 8), если у набора байт +0xb2
///   (FUN_1800c7fe0: совместимость > 0x139b7) и у описания поля +0x20c — это +0x1fc записи 0x240, его ставит
///   FUN_180303ac0 из слота +0x88 описателя поля запроса (интерфейс 6f3b3628…, entext FUN_1803e2240: байт +0x280).
///   Откуда описатель берет байт, не выведено (трасса qp-dcs-14: 1 только у реквизита с `PasswordMode`) — поля нет,
///   если выражение называет реквизит с `PasswordMode` любого объекта (сверка ЛентыНовостей Клиента ЭДО).
///   shortcut: признак по имени реквизита, а не по описателю; при расхождении — искать запись +0x88 описателя.
///   Совместимость < 0x139bf: схему списка строит mngbase FUN_1802b6b10 (FUN_1802d5600) без конфигурации — у схемы
///   +0x1f8 пуст, и FUN_1800c7fe0 берет совместимость процесса (+0x138), а не конфигурации. shortcut: что она дает при
///   загрузке, не прослежено — по сверке (УправлениеНовостями БП 3.0.107 при 8.3.16: Пароль в корне) поле остается;
///   пересмотреть, если при старой совместимости поле с режимом пароля разойдется.
/// Представления — `presentation_dropped`: `ПРЕДСТАВЛЕНИЕ(поле)` и стандартного `Источник.Представление` в полях набора
/// нет (трасса qp-3 E1, qp-settings-2), где их отбрасывает компиляция набора (dcs FUN_18030a0b0 / entext +0x70 части), не
/// найдено. Реквизит с именем Представление (ресурс регистра, реквизит табличной части-источника) — обычное поле.
pub fn password_dropped(x: &Index, f: &Field) -> bool {
    if x.compat.is_some_and(|c| c < 0x139bf) {
        return false;
    }
    let dbg = format!("{:?}", f.expr).to_lowercase();
    let common_pw = x.objects.iter().filter(|(k, o)| k.starts_with("CommonAttribute.") && o.props.get("PasswordMode").is_some_and(|v| v == "true")).map(|(k, _)| k["CommonAttribute.".len()..].to_lowercase());
    x.password.iter().cloned().chain(common_pw).any(|p| dbg.contains(&format!("\"{p}\"")))
}

/// Поле-представление (`ПРЕДСТАВЛЕНИЕ(…)`, стандартное `Источник.Представление`) выборки `all`, которого в корне
/// набора нет. shortcut: правило по трассам и сверке Клиента ЭДО, место в коде компиляции набора не найдено (см.
/// `password_dropped`); при расхождении — искать в dcs FUN_18030a0b0.
pub fn presentation_dropped(x: &Index, f: &Field, all: &[Field], srcs: &[(&Source, Option<(String, String)>)], ts_source: &dyn Fn(&Table) -> bool) -> bool {
    let tail = match &f.expr {
        Expr::Path(p) => p.last(),
        Expr::Dot(_, v) => v.last(),
        _ => None,
    };
    // `ПРЕДСТАВЛЕНИЕ(Т.Поле)`, когда `Т.Поле` выбрано и само (СписокПубликаций: ТорговоеПредложение), — нет;
    // поле не выбрано (УчетныеЗаписиЭДО: СпособОбменаЭД) или аргумент — агрегат (НастройкиЭДО) — обычное поле корня
    if let Expr::Call { name, args, .. } = &f.expr {
        // ПРЕДСТАВЛЕНИЕССЫЛКИ — так же (журнал РегламентныеДокументыМеждународныйУчет ERP: `… КАК Представление`)
        if matches!(name.as_str(), "ПРЕДСТАВЛЕНИЕ" | "PRESENTATION" | "ПРЕДСТАВЛЕНИЕССЫЛКИ" | "REFPRESENTATION") {
            let same = |p: &[String], q: &[String]| p.len() == q.len() && p.iter().zip(q).all(|(a, b)| eq(a, b));
            return matches!(args.as_slice(), [Expr::Path(p)] if all.iter().any(|g| matches!(&g.expr, Expr::Path(q) if same(p, q))));
        }
    }
    if tail.is_some_and(|t| eq(t, "Представление") || eq(t, "Presentation")) {
        // `Поле` единственного источника без псевдонима — как `Источник.Поле` с пустым именем источника (как в dlist
        // `query_root_fields`, `alias_of`)
        let sole = matches!(srcs, [(s, _)] if s.alias.is_none());
        let full = |p: &[String]| match p {
            [n] if sole => Some(vec![String::new(), n.clone()]),
            [_, _] => Some(p.to_vec()),
            _ => None,
        };
        let of = |s: &Source, a: &str| s.alias.as_deref().map_or(sole && a.is_empty(), |x| eq(x, a));
        let path = match &f.expr {
            Expr::Path(p) => full(p),
            _ => None,
        };
        let attr = path.as_ref().is_some_and(|p| {
            srcs.iter().any(|(s, t)| {
                // источник — табличная часть или виртуальная таблица (`Задача.Х.ЗадачиПоИсполнителю`: Presentation в
                // корне — номера FieldsMap МоиЗадачи БП)
                let vt = matches!(&s.table, Table::Name { path, .. } if path.len() == 3);
                of(s, &p[0]) && (ts_source(&s.table) || vt || t.as_ref().and_then(|(_, o)| x.objects.get(o)).is_some_and(|o| o.kids.iter().any(|(k, n)| matches!(k.as_str(), "Attribute" | "Dimension" | "Resource") && eq(n, &p[1]))))
            })
        });
        // ссылочная таблица, чья `Ссылка` не выбрана, — представление остается полем корня (ПКО ФормаСпискаДокументов
        // ERP: документ присоединен к регистру). shortcut: по сверке ERP и БП, в dcs не прослежено
        let reft = path.as_ref().is_some_and(|p| {
            srcs.iter().any(|(s, t)| {
                of(s, &p[0])
                    && t.as_ref().is_some_and(|(_, o)| matches!(o.split('.').next(), Some("Catalog" | "Document" | "ChartOfCharacteristicTypes" | "ChartOfAccounts" | "ChartOfCalculationTypes" | "ExchangePlan" | "BusinessProcess" | "Task")))
                    && !all.iter().any(|g| matches!(&g.expr, Expr::Path(q) if full(q).is_some_and(|q| eq(&q[0], &p[0]) && (eq(&q[1], "Ссылка") || eq(&q[1], "Ref")))))
            })
        });
        return !attr && !reft;
    }
    false
}

/// Имя и параметры (`None` — без скобок) виртуальных таблиц в источнике `s` и его соединениях, вложенных запросах.
fn vt_sources(s: &super::parse::Source, out: &mut Vec<(String, Option<Vec<Option<super::parse::Expr>>>)>) -> R<()> {
    use super::parse::Table;
    match &s.table {
        // виртуальные таблицы, у которых первый параметр — значение (срезы, остатки — Период; обороты, остатки и
        // обороты — Начало); табличная часть — не она. shortcut: обороты — по справке, трассы описателей нет
        Table::Name { path, params } if path.len() == 3 && ["СрезПоследних", "СрезПервых", "SliceLast", "SliceFirst", "Остатки", "Balance", "Обороты", "Turnovers", "ОстаткиИОбороты", "BalanceAndTurnovers"].iter().any(|v| eq(v, &path[2])) => out.push((path[2].clone(), params.clone())),
        Table::Name { .. } => {}
        Table::Sub(q) => {
            for p in &q.parts {
                for f in &p.from {
                    vt_sources(f, out)?;
                }
            }
        }
        Table::Group(g) => vt_sources(g, out)?,
    }
    for j in &s.joins {
        vt_sources(&j.source, out)?;
    }
    Ok(())
}

/// Метаданные, которые отмечают обращение к виртуальной таблице (и ее имя).
struct Vt<'a> {
    m: &'a dyn Meta,
    seen: Cell<bool>,
    names: RefCell<Vec<String>>,
}

impl Meta for Vt<'_> {
    fn object(&self, obj: &str) -> Option<String> {
        self.m.object(obj)
    }
    fn field(&self, obj: &str, name: &str) -> R<Option<Vec<Ty>>> {
        self.m.field(obj, name)
    }
    fn ts(&self, obj: &str, ts: &str) -> bool {
        self.m.ts(obj, ts)
    }
    fn ts_field(&self, obj: &str, ts: &str, name: &str) -> R<Option<Vec<Ty>>> {
        self.m.ts_field(obj, ts, name)
    }
    fn vt_field(&self, obj: &str, vt: &str, name: &str) -> R<Option<Vec<Ty>>> {
        // таблица констант параметров не имеет
        if obj != super::resolve::CONSTANTS {
            self.seen.set(true);
            self.names.borrow_mut().push(vt.to_string());
        }
        self.m.vt_field(obj, vt, name)
    }
    fn vt_param(&self, obj: &str, vt: &str, name: &str) -> R<Option<Vec<Ty>>> {
        self.seen.set(true);
        self.names.borrow_mut().push(vt.to_string());
        self.m.vt_param(obj, vt, name)
    }
    fn value(&self, obj: &str, name: &str) -> bool {
        self.m.value(obj, name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct M;
    impl Meta for M {
        fn object(&self, obj: &str) -> Option<String> {
            Some(obj.to_string())
        }
        fn field(&self, _: &str, _: &str) -> R<Option<Vec<Ty>>> {
            Ok(Some(vec![Ty::Prim]))
        }
        fn ts(&self, _: &str, _: &str) -> bool {
            false
        }
        fn ts_field(&self, _: &str, _: &str, _: &str) -> R<Option<Vec<Ty>>> {
            Ok(None)
        }
        fn vt_field(&self, _: &str, _: &str, _: &str) -> R<Option<Vec<Ty>>> {
            Ok(Some(vec![Ty::Prim]))
        }
        fn vt_param(&self, _: &str, _: &str, _: &str) -> R<Option<Vec<Ty>>> {
            Ok(Some(vec![Ty::Prim]))
        }
        fn value(&self, _: &str, _: &str) -> bool {
            true
        }
    }

    fn dp(text: &str, declared: &[(&str, bool)]) -> R<bool> {
        let d: Vec<_> = declared.iter().map(|(n, f)| (n.to_string(), *f)).collect();
        data_parameters(&M, text, &crate::query::parse(text)?, &d)
    }

    #[test]
    fn data_parameters_rule() {
        let q = "ВЫБРАТЬ Т.Ссылка КАК Ссылка ИЗ Справочник.Банки КАК Т";
        assert_eq!(dp(q, &[]), Ok(false));
        assert_eq!(dp(&format!("{q} ГДЕ Т.Код = &Код"), &[]), Ok(true));
        // параметр в строке — не параметр
        assert_eq!(dp("ВЫБРАТЬ \"&Код\" КАК А ИЗ Справочник.Банки КАК Т", &[]), Ok(false));
        // объявлен с availableAsField = false — не дает; другой объявленный (истина) — дает
        assert_eq!(dp(&format!("{q} ГДЕ Т.Код = &код"), &[("Код", false)]), Ok(false));
        assert_eq!(dp(q, &[("Прочий", true)]), Ok(true));
        // параметр виртуальной таблицы без полей (не задан или `&Период`) — параметр набора
        assert_eq!(dp("ВЫБРАТЬ Т.Период КАК П ИЗ РегистрСведений.Курсы.СрезПоследних КАК Т", &[]), Ok(true));
        assert_eq!(dp("ВЫБРАТЬ Т.Период КАК П ИЗ РегистрСведений.Курсы.СрезПоследних(&Дата, ) КАК Т", &[]), Ok(true));
        // у оборотов — по тому же правилу, что дети узла (`vt_registered`): начало с полями, конец не задан — КонецПериода;
        // объявленный одноименный с availableAsField = false — не дает
        let q = "ВЫБРАТЬ Т.Сумма КАК С ИЗ РегистрНакопления.Р.Обороты(Период, ) КАК Т";
        assert_eq!(dp(q, &[]), Ok(true));
        assert_eq!(dp(q, &[("КонецПериода", false)]), Ok(false));
    }

    #[test]
    fn parameter_fields_rule() {
        let pf = |text: &str, declared: &[(&str, bool)]| {
            let d: Vec<_> = declared.iter().map(|(n, f)| (n.to_string(), *f)).collect();
            parameter_fields(&M, text, &crate::query::parse(text).unwrap(), &d).unwrap()
        };
        // параметры запроса и объявленные, одноименные — одно; объявленный с availableAsField = false — нет
        assert_eq!(pf("ВЫБРАТЬ 1 КАК А ИЗ Справочник.Банки КАК Т ГДЕ Т.Код = &Код И Т.Имя = &Имя", &[("код", true), ("Имя", false), ("Прочий", true)]), ["Код", "Прочий"]);
        // параметры оборотов без полей — НачалоПериода и КонецПериода, объявленные с теми же именами сливаются
        assert_eq!(pf("ВЫБРАТЬ Т.Сумма КАК С ИЗ РегистрНакопления.Р.ОстаткиИОбороты(, , Авто, , ) КАК Т", &[("НачалоПериода", true)]), ["НачалоПериода", "КонецПериода"]);
        // срез с параметром-выражением без полей: Период и сам параметр запроса
        assert_eq!(pf("ВЫБРАТЬ Т.Период КАК П ИЗ РегистрСведений.Курсы.СрезПоследних(&Дата, ) КАК Т", &[]), ["Дата", "Период"]);
    }
}
