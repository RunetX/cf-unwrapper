//! Проверка текста запроса по метаданным: каждая таблица, поле и значение `ЗНАЧЕНИЕ()` существуют. У платформы
//! запрос с несуществующим объектом или полем не компилируется — набор компоновки остается без полей (выборка 8.5:
//! номера FieldsMap идут подряд), а этот путь не разобран; поэтому все, что не удалось проверить, — ошибка (отказ
//! формы), а не догадка. Типы полей — по метаданным (`Meta`), через ссылочные поля путь идет дальше.

use super::parse::*;

/// Тип значения поля: ссылка на объект (`Catalog.Банки`), примитив, хранилище значения, табличная часть, не выведен.
#[derive(Clone, Debug, PartialEq)]
pub enum Ty {
    Ref(String),
    Prim,
    /// Дата: у поля дерева компоновки дети ДатыНачала, ДатыКонца, ЧастиДат (dlist `children`).
    Date,
    /// УникальныйИдентификатор: в путях как примитив, но не аргумент `СТРОКА` (names::string_type).
    Uuid,
    Storage,
    Table,
    Unknown,
}

/// Метаданные для проверки (реализует загрузка по своему индексу).
pub trait Meta {
    /// Объект `Catalog.Банки` (имя без учета регистра): ключ в метаданных, `None` — нет такого.
    fn object(&self, obj: &str) -> Option<String>;
    /// Поле основной таблицы объекта: типы; `Ok(None)` — поля нет; `Err` — состав полей класса не выведен.
    fn field(&self, obj: &str, name: &str) -> Result<Option<Vec<Ty>>, String>;
    /// Табличная часть объекта есть.
    fn ts(&self, obj: &str, ts: &str) -> bool;
    /// Поле табличной части.
    fn ts_field(&self, obj: &str, ts: &str, name: &str) -> Result<Option<Vec<Ty>>, String>;
    /// Поле виртуальной таблицы (`СрезПоследних`); `Err` — таблица не поддержана.
    fn vt_field(&self, obj: &str, vt: &str, name: &str) -> Result<Option<Vec<Ty>>, String>;
    /// Поле в параметрах виртуальной таблицы (условие, параметр компоновки); `Err` — не проверено.
    fn vt_param(&self, obj: &str, vt: &str, name: &str) -> Result<Option<Vec<Ty>>, String>;
    /// `ЗНАЧЕНИЕ(<obj>.<name>)`: предопределенный элемент, значение перечисления, пустая ссылка.
    fn value(&self, obj: &str, name: &str) -> bool;
    /// Типы параметра `&<name>` как выражения; `None` — не выведены.
    fn param(&self, _name: &str) -> Option<Vec<Ty>> {
        None
    }
}

/// Источник `Константы`: в `Meta::vt_field` — объект с этим именем.
pub const CONSTANTS: &str = "Constants";

/// Периодичность виртуальной таблицы оборотов (справка платформы).
const PERIODICITY: &[&str] = &[
    "Авто", "Запись", "Регистратор", "Секунда", "Минута", "Час", "День", "Неделя", "Декада", "Месяц", "Квартал", "Полугодие", "Год",
    "Auto", "Record", "Recorder", "Second", "Minute", "Hour", "Day", "Week", "TenDays", "Month", "Quarter", "HalfYear", "Year",
];

/// Класс метаданных по имени в запросе (`Справочник` / `Catalog`).
pub const CLASSES: &[(&str, &str)] = &[
    ("Справочник", "Catalog"),
    ("Документ", "Document"),
    ("ПланВидовХарактеристик", "ChartOfCharacteristicTypes"),
    ("РегистрСведений", "InformationRegister"),
    ("РегистрНакопления", "AccumulationRegister"),
    ("Перечисление", "Enum"),
    ("ЖурналДокументов", "DocumentJournal"),
    ("ПланСчетов", "ChartOfAccounts"),
    ("ПланОбмена", "ExchangePlan"),
    ("БизнесПроцесс", "BusinessProcess"),
    ("Задача", "Task"),
    ("РегистрБухгалтерии", "AccountingRegister"),
    ("ПланВидовРасчета", "ChartOfCalculationTypes"),
    ("РегистрРасчета", "CalculationRegister"),
    ("Константа", "Constant"),
];

pub fn lower(s: &str) -> String {
    s.chars().flat_map(char::to_lowercase).collect()
}

pub fn eq(a: &str, b: &str) -> bool {
    lower(a) == lower(b)
}

pub fn class_en(c: &str) -> Option<&'static str> {
    CLASSES.iter().find(|(r, e)| eq(r, c) || eq(e, c)).map(|(_, e)| *e)
}

/// Классы со ссылочным типом (тип в `ВЫРАЗИТЬ`/`ССЫЛКА`/`ТИП`, `ЗНАЧЕНИЕ()`); у регистров и журналов ссылки нет.
const REF_CLASSES: &[&str] = &["Catalog", "Document", "ChartOfCharacteristicTypes", "Enum", "ChartOfAccounts", "ExchangePlan", "BusinessProcess", "Task", "ChartOfCalculationTypes"];

/// Системные перечисления в `ЗНАЧЕНИЕ()`: имя (рус./англ.) и значения.
const SYSTEM_ENUMS: &[(&str, &str, &[&str])] = &[
    ("ВидДвиженияНакопления", "AccumulationRecordType", &["Приход", "Расход", "Receipt", "Expense"]),
    ("ВидДвиженияБухгалтерии", "AccountingRecordType", &["Дебет", "Кредит", "Debit", "Credit"]),
    ("ВидСчета", "AccountType", &["Активный", "Пассивный", "АктивноПассивный", "Active", "Passive", "ActivePassive"]),
];

/// Функции языка запросов (имя в верхнем регистре): тип результата (`Some(i)` — тип аргумента `i`, `None` —
/// примитив) и число аргументов от и до (справка платформы shquery_ru.hbk: страницы функций и агрегатных функций).
const FUNCS: &[(&str, Option<usize>, usize, usize)] = &[
    ("ЕСТЬNULL", Some(0), 2, 2),
    ("ISNULL", Some(0), 2, 2),
    ("МАКСИМУМ", Some(0), 1, 1),
    ("MAX", Some(0), 1, 1),
    ("МИНИМУМ", Some(0), 1, 1),
    ("MIN", Some(0), 1, 1),
    ("СУММА", None, 1, 1),
    ("SUM", None, 1, 1),
    ("КОЛИЧЕСТВО", None, 1, 1),
    ("COUNT", None, 1, 1),
    ("СРЕДНЕЕ", None, 1, 1),
    ("AVG", None, 1, 1),
    ("ПОДСТРОКА", None, 3, 3),
    ("SUBSTRING", None, 3, 3),
    ("ГОД", None, 1, 1),
    ("КВАРТАЛ", None, 1, 1),
    ("МЕСЯЦ", None, 1, 1),
    ("ДЕНЬГОДА", None, 1, 1),
    ("ДЕНЬ", None, 1, 1),
    ("НЕДЕЛЯ", None, 1, 1),
    ("ДЕНЬНЕДЕЛИ", None, 1, 1),
    ("ЧАС", None, 1, 1),
    ("МИНУТА", None, 1, 1),
    ("СЕКУНДА", None, 1, 1),
    ("НАЧАЛОПЕРИОДА", None, 2, 2),
    ("КОНЕЦПЕРИОДА", None, 2, 2),
    ("ДОБАВИТЬКДАТЕ", None, 3, 3),
    ("РАЗНОСТЬДАТ", None, 3, 3),
    // литерал даты: 3 или 6 чисел
    ("ДАТАВРЕМЯ", None, 3, 6),
    ("ПРЕДСТАВЛЕНИЕ", None, 1, 1),
    ("ПРЕДСТАВЛЕНИЕССЫЛКИ", None, 1, 1),
    ("ТИПЗНАЧЕНИЯ", None, 1, 1),
    ("ДЛИНАСТРОКИ", None, 1, 1),
    ("СОКРЛ", None, 1, 1),
    ("СОКРП", None, 1, 1),
    ("СОКРЛП", None, 1, 1),
    ("ЛЕВ", None, 2, 2),
    ("ПРАВ", None, 2, 2),
    ("СТРНАЙТИ", None, 2, 2),
    ("ВРЕГ", None, 1, 1),
    ("НРЕГ", None, 1, 1),
    ("СТРЗАМЕНИТЬ", None, 3, 3),
    ("ОКР", None, 1, 2),
    ("ЦЕЛ", None, 1, 1),
];

/// Периодичность в функциях даты (справка платформы: BEGINOFPERIOD/ENDOFPERIOD, DATEADD, DATEDIFF).
const PERIODS_BEGIN: &[&str] = &["МИНУТА", "ЧАС", "ДЕНЬ", "НЕДЕЛЯ", "МЕСЯЦ", "КВАРТАЛ", "ГОД", "ДЕКАДА", "ПОЛУГОДИЕ", "MINUTE", "HOUR", "DAY", "WEEK", "MONTH", "QUARTER", "YEAR", "TENDAYS", "HALFYEAR"];
const PERIODS_ADD: &[&str] = &["СЕКУНДА", "МИНУТА", "ЧАС", "ДЕНЬ", "НЕДЕЛЯ", "МЕСЯЦ", "КВАРТАЛ", "ГОД", "ДЕКАДА", "ПОЛУГОДИЕ", "SECOND", "MINUTE", "HOUR", "DAY", "WEEK", "MONTH", "QUARTER", "YEAR", "TENDAYS", "HALFYEAR"];
const PERIODS_DIFF: &[&str] = &["СЕКУНДА", "МИНУТА", "ЧАС", "ДЕНЬ", "МЕСЯЦ", "КВАРТАЛ", "ГОД", "SECOND", "MINUTE", "HOUR", "DAY", "MONTH", "QUARTER", "YEAR"];

/// Поля источника (вложенный запрос, временная таблица): имя (пустое — не выведено) и типы (`None` — не выведены).
pub type Out = Vec<(String, Option<Vec<Ty>>)>;

#[derive(Clone)]
enum Src {
    Obj(String),
    Ts(String, String),
    Vt(String, String),
    /// параметры виртуальной таблицы
    VtParam(String, String),
    Derived(Out),
}

/// Источники области: псевдоним (или имя таблицы), источник, псевдоним задан явно.
type Srcs = Vec<(Option<String>, Src, bool)>;

struct Scope<'a> {
    srcs: Srcs,
    /// имена полей выборки (для `УПОРЯДОЧИТЬ ПО`, `ИТОГИ`)
    outs: Out,
    outer: Option<&'a Scope<'a>>,
}

type R<T> = Result<T, String>;

struct Ck<'m> {
    m: &'m dyn Meta,
    temps: Vec<(String, Out)>,
}

fn missing(what: &str) -> String {
    format!("{what} не найдено в метаданных: у платформы такой запрос не компилируется (корень набора не выведен)")
}

/// Признак ошибки «объекта `Класс.Имя` нет в конфигурации»: такой запрос у платформы не компилируется наверняка. У поля —
/// когда `Meta` ответил «поля нет» (`Ok(None)` — только при составе таблицы, выведенном полностью).
pub const ABSENT: &str = "объекта нет в конфигурации";

fn absent(what: &str) -> String {
    missing(&format!("{what} ({ABSENT})"))
}

/// Поля нет в источнике: у таблицы метаданных — наверняка (`Meta` отвечает «нет» только при полном составе), у
/// вложенного запроса и временной таблицы — нет (имена их полей без псевдонима выведены не для всех выражений).
fn gone(src: &Src, what: &str) -> String {
    if matches!(src, Src::Derived(_)) { missing(what) } else { absent(what) }
}

impl Ck<'_> {
    fn ty_of(&self, t: &Type) -> R<Vec<Ty>> {
        if let (Some(c), 2) = (class_en(&t.path[0]), t.path.len()) {
            if !REF_CLASSES.contains(&c) {
                return Err(format!("тип {}: у класса {c} нет ссылки", t.path.join(".")));
            }
            return match self.m.object(&format!("{c}.{}", t.path[1])) {
                Some(obj) => Ok(vec![Ty::Ref(obj)]),
                None => Err(missing(&format!("тип {}", t.path.join(".")))),
            };
        }
        if t.path.len() == 1 && ["Дата", "Date"].iter().any(|p| eq(p, &t.path[0])) {
            return Ok(vec![Ty::Date]);
        }
        let prim = ["Строка", "Число", "Булево", "String", "Number", "Boolean"];
        if t.path.len() == 1 && prim.iter().any(|p| eq(p, &t.path[0])) {
            return Ok(vec![Ty::Prim]);
        }
        Err(format!("тип {} в запросе не поддержан", t.path.join(".")))
    }

    fn value(&self, p: &[String]) -> R<Vec<Ty>> {
        if p.len() == 2 {
            if let Some((_, _, vals)) = SYSTEM_ENUMS.iter().find(|(r, e, _)| eq(r, &p[0]) || eq(e, &p[0])) {
                return if vals.iter().any(|v| eq(v, &p[1])) { Ok(vec![Ty::Prim]) } else { Err(missing(&format!("значение {}", p.join(".")))) };
            }
        }
        let c = class_en(&p[0]).filter(|c| p.len() == 3 && REF_CLASSES.contains(c)).ok_or_else(|| format!("ЗНАЧЕНИЕ({}) не поддержано", p.join(".")))?;
        match self.m.object(&format!("{c}.{}", p[1])) {
            Some(obj) if self.m.value(&obj, &p[2]) => Ok(vec![Ty::Ref(obj)]),
            _ => Err(missing(&format!("значение {}", p.join(".")))),
        }
    }

    fn src_field(&self, s: &Src, n: &str) -> R<Option<Vec<Ty>>> {
        match s {
            Src::Obj(o) => self.m.field(o, n),
            Src::Ts(o, t) => self.m.ts_field(o, t, n),
            Src::Vt(o, v) => self.m.vt_field(o, v, n),
            Src::VtParam(o, v) => self.m.vt_param(o, v, n),
            Src::Derived(out) => match out.iter().find(|(k, _)| eq(k, n)) {
                Some((_, t)) => Ok(Some(t.clone().unwrap_or_else(|| vec![Ty::Unknown]))),
                None if out.iter().any(|(k, _)| k.is_empty()) => Err(format!("поле {n}: у источника есть поле без псевдонима, имя которого не выведено")),
                None => Ok(None),
            },
        }
    }

    /// Путь дальше по типам: через ссылку — поле объекта; у примитива подполей нет.
    fn walk(&self, mut types: Vec<Ty>, rest: &[String], p: &[String]) -> R<Option<Vec<Ty>>> {
        for seg in rest {
            let mut next = Vec::new();
            let mut found = false;
            // тип, у которого состав полей не выведен (`Meta` — Err), не мешает найти поле в другом типе составной
            // ссылки (как до полного состава: там «поля нет»); не найдено нигде — его ошибка
            let mut unsure = None;
            for t in &types {
                match t {
                    Ty::Ref(o) => match self.m.field(o, seg) {
                        Ok(Some(v)) => {
                            found = true;
                            next.extend(v);
                        }
                        Ok(None) => {}
                        Err(e) => unsure = unsure.or(Some(e)),
                    },
                    Ty::Prim | Ty::Date | Ty::Uuid | Ty::Storage => {}
                    Ty::Table => return Err(format!("путь {} через табличную часть не поддержан", p.join("."))),
                    Ty::Unknown => return Err(format!("тип поля на пути {} не выведен", p.join("."))),
                }
            }
            if !found {
                return Err(unsure.unwrap_or_else(|| absent(&format!("поле {}", p.join(".")))));
            }
            next.dedup();
            types = next;
        }
        Ok(Some(types))
    }

    fn path(&self, p: &[String], sc: &Scope) -> R<Option<Vec<Ty>>> {
        let mut s = Some(sc);
        while let Some(c) = s {
            if let Some((_, src, _)) = c.srcs.iter().find(|(a, ..)| a.as_deref().is_some_and(|a| eq(a, &p[0]))) {
                if p.len() == 1 {
                    return Err(format!("таблица {} как значение не поддержана", p[0]));
                }
                // `Т.ТЧ.Реквизит` — реквизит табличной части объекта (справка: «Обращение к полям табличной части»)
                if let (Src::Obj(o), true) = (src, p.len() > 2) {
                    if self.m.ts(o, &p[1]) {
                        let t = self.m.ts_field(o, &p[1], &p[2])?.ok_or_else(|| absent(&format!("поле {}", p.join("."))))?;
                        return self.walk(t, &p[3..], p);
                    }
                }
                let t = self.src_field(src, &p[1])?.ok_or_else(|| gone(src, &format!("поле {}", p.join("."))))?;
                return self.walk(t, &p[2..], p);
            }
            s = c.outer;
        }
        // таблица полным именем — только источник без псевдонима (справка: «Псевдонимы источников данных»)
        if let (true, Some(c)) = (p.len() >= 3, class_en(&p[0])) {
            let o = self.m.object(&format!("{c}.{}", p[1]));
            let mut s = Some(sc);
            while let Some(c) = s {
                if let Some((_, src, _)) = c.srcs.iter().find(|(_, x, explicit)| !explicit && matches!(x, Src::Obj(k) if Some(k) == o.as_ref())) {
                    let t = self.src_field(src, &p[2])?.ok_or_else(|| absent(&format!("поле {}", p.join("."))))?;
                    return self.walk(t, &p[3..], p);
                }
                s = c.outer;
            }
            return Err(format!("поле по полному имени таблицы {} не из источника без псевдонима: не поддержано", p.join(".")));
        }
        // поле без псевдонима таблицы: ровно один источник ближней области, где оно есть
        let mut s = Some(sc);
        let mut unsure = None;
        while let Some(c) = s {
            let mut hits = Vec::new();
            for (_, src, _) in &c.srcs {
                // источник без выведенного состава полей — как в `walk`: не найдено нигде — его ошибка
                match self.src_field(src, &p[0]) {
                    Ok(Some(t)) => hits.push(t),
                    Ok(None) => {}
                    Err(e) => unsure = unsure.or(Some(e)),
                }
            }
            if hits.len() > 1 {
                return Err(format!("поле {} без псевдонима таблицы неоднозначно", p[0]));
            }
            if let Some(t) = hits.pop() {
                return self.walk(t, &p[1..], p);
            }
            s = c.outer;
        }
        if let Some(e) = unsure {
            return Err(e);
        }
        // поля нет ни в одном источнике: наверняка — если среди них нет вложенного запроса или временной таблицы
        let mut s = Some(sc);
        let mut derived = false;
        while let Some(c) = s {
            derived |= c.srcs.iter().any(|(_, x, _)| matches!(x, Src::Derived(_)));
            s = c.outer;
        }
        let what = format!("поле {}", p.join("."));
        Err(if derived { missing(&what) } else { absent(&what) })
    }

    fn union(v: Vec<Option<Vec<Ty>>>) -> Option<Vec<Ty>> {
        let mut out = Vec::new();
        for t in v {
            for x in t? {
                if !out.contains(&x) {
                    out.push(x);
                }
            }
        }
        Some(out)
    }

    fn expr(&mut self, e: &Expr, sc: &Scope) -> R<Option<Vec<Ty>>> {
        let prim = Ok(Some(vec![Ty::Prim]));
        match e {
            Expr::Path(p) => self.path(p, sc),
            Expr::Star(p) => {
                // `Т.*` — источник области; `Т.Поле.*` — поле ссылочного типа
                let alias = |p: &[String]| sc.srcs.iter().any(|(a, ..)| a.as_deref().is_some_and(|a| eq(a, &p[0])));
                // `Поле.*` в параметре виртуальной таблицы — поле ее источника без псевдонима (`СрезПоследних(…,
                // {Номенклатура.* КАК Номенклатура})`, ERP ЦеныНоменклатуры)
                let field = p.len() == 1 && !alias(p) && sc.srcs.iter().any(|(a, s, _)| a.is_none() && matches!(s, Src::VtParam(..)));
                if p.len() > 1 || field {
                    let t = self.path(p, sc)?;
                    // `Т.ТЧ.*` — табличная часть одной колонкой вложенной таблицы (entext FUN_18034a270, dlist query_root_fields)
                    if p.len() == 2 && t.as_deref() == Some(&[Ty::Table][..]) {
                        return Ok(None);
                    }
                    Self::refs(t, &p.join("."))?;
                } else if !p.is_empty() && !alias(p) {
                    return Err(missing(&format!("таблица {}", p.join("."))));
                }
                Ok(None)
            }
            Expr::Num(_) | Expr::Str(_) | Expr::Const(_) => prim,
            Expr::Param(p) => Ok(self.m.param(p)),
            Expr::Neg(x) | Expr::Not(x) => {
                self.expr(x, sc)?;
                prim
            }
            Expr::Bin(_, a, b) => {
                self.expr(a, sc)?;
                self.expr(b, sc)?;
                prim
            }
            Expr::Case { of, when, other } => {
                if let Some(o) = of {
                    self.expr(o, sc)?;
                }
                let mut v = Vec::new();
                for (c, t) in when {
                    self.expr(c, sc)?;
                    v.push(self.expr(t, sc)?);
                }
                match other {
                    Some(o) => v.push(self.expr(o, sc)?),
                    None => v.push(Some(vec![Ty::Prim])),
                }
                Ok(Self::union(v))
            }
            Expr::Call { name, distinct, args } => {
                if name == "СТРОКА" || name == "STRING" {
                    let t = match args.as_slice() {
                        [a] if !*distinct => self.expr(a, sc)?,
                        _ => None,
                    };
                    return super::names::string_type(args, *distinct, t.as_deref()).map(|_| Some(vec![Ty::Prim]));
                }
                let (_, r, lo, hi) = FUNCS.iter().find(|(f, ..)| *f == name).ok_or_else(|| format!("функция {name} в запросе не поддержана"))?;
                let count = matches!(name.as_str(), "КОЛИЧЕСТВО" | "COUNT");
                if args.len() < *lo || args.len() > *hi || (name == "ДАТАВРЕМЯ" && matches!(args.len(), 4 | 5)) {
                    return Err(format!("функция {name}: {} аргументов", args.len()));
                }
                // РАЗЛИЧНЫЕ и `*` — только у КОЛИЧЕСТВО (справка: агрегатные функции)
                if (*distinct || matches!(args.first(), Some(Expr::Star(p)) if p.is_empty())) && !count {
                    return Err(format!("функция {name}: РАЗЛИЧНЫЕ или * не поддержаны"));
                }
                // периодичность — слово, не поле: НАЧАЛОПЕРИОДА(д, ДЕНЬ), ДОБАВИТЬКДАТЕ(д, МЕСЯЦ, n), РАЗНОСТЬДАТ(а, б, СЕКУНДА)
                let period_at = match name.as_str() {
                    "НАЧАЛОПЕРИОДА" | "КОНЕЦПЕРИОДА" => Some((1, PERIODS_BEGIN)),
                    "ДОБАВИТЬКДАТЕ" => Some((1, PERIODS_ADD)),
                    "РАЗНОСТЬДАТ" => Some((2, PERIODS_DIFF)),
                    _ => None,
                };
                let mut v = Vec::new();
                for (i, a) in args.iter().enumerate() {
                    if let Some((_, words)) = period_at.filter(|(k, _)| *k == i) {
                        match a {
                            Expr::Path(p) if p.len() == 1 && words.iter().any(|w| eq(w, &p[0])) => {
                                v.push(Some(vec![Ty::Prim]));
                                continue;
                            }
                            _ => return Err(format!("периодичность в {name} не поддержана")),
                        }
                    }
                    v.push(self.expr(a, sc)?);
                }
                Ok(match r {
                    Some(0) if name == "ЕСТЬNULL" || name == "ISNULL" => Self::union(v),
                    Some(i) => v.get(*i).cloned().flatten(),
                    // функции, возвращающие дату (справка платформы: BEGINOFPERIOD, ENDOFPERIOD, DATEADD, DATETIME)
                    None if matches!(name.as_str(), "НАЧАЛОПЕРИОДА" | "КОНЕЦПЕРИОДА" | "ДОБАВИТЬКДАТЕ" | "ДАТАВРЕМЯ") => Some(vec![Ty::Date]),
                    None => Some(vec![Ty::Prim]),
                })
            }
            Expr::Cast(x, t) => {
                self.expr(x, sc)?;
                Ok(Some(self.ty_of(t)?))
            }
            Expr::Refs(x, t) => {
                self.expr(x, sc)?;
                self.ty_of(t)?;
                prim
            }
            Expr::TypeOf(t) => {
                self.ty_of(t)?;
                prim
            }
            Expr::Value(p) => Ok(Some(self.value(p)?)),
            // вложенный запрос — только список `В (ВЫБРАТЬ …)`, список значений — только в `В`
            Expr::In { x, list, .. } => {
                for i in std::iter::once(x.as_ref()).chain(list) {
                    match i {
                        Expr::Sub(q) => {
                            self.sub(q, Some(sc))?;
                        }
                        Expr::Tuple(v) => {
                            for x in v {
                                self.expr(x, sc)?;
                            }
                        }
                        e => {
                            self.expr(e, sc)?;
                        }
                    }
                }
                prim
            }
            Expr::Sub(_) => Err("вложенный запрос в выражении (не в ИЗ, СОЕДИНЕНИЕ, В) не поддержан".into()),
            Expr::Like { x, pat, esc, .. } => {
                self.expr(x, sc)?;
                self.expr(pat, sc)?;
                if let Some(e) = esc {
                    self.expr(e, sc)?;
                }
                prim
            }
            Expr::Between { x, lo, hi, .. } => {
                self.expr(x, sc)?;
                self.expr(lo, sc)?;
                self.expr(hi, sc)?;
                prim
            }
            Expr::IsNull { x, .. } => {
                self.expr(x, sc)?;
                prim
            }
            Expr::Tuple(_) => Err("список значений вне В не поддержан".into()),
            Expr::Dot(x, segs) => {
                let t = self.expr(x, sc)?;
                if segs.iter().any(|s| s == "*") {
                    Self::refs(t, "(…).*")?;
                    return Ok(None);
                }
                let t = t.ok_or("тип выражения перед точкой не выведен")?;
                self.walk(t, segs, segs)
            }
            Expr::Nested(..) => Err("вложенная табличная часть вне списка выборки не поддержана".into()),
            Expr::Dcs(_) => Err("параметр компоновки вне виртуальной таблицы".into()),
        }
    }

    /// Типы поля перед `.*`: все ссылочные.
    fn refs(t: Option<Vec<Ty>>, what: &str) -> R<()> {
        match t {
            Some(t) if !t.is_empty() && t.iter().any(|t| matches!(t, Ty::Ref(_))) => Ok(()),
            _ => Err(format!("{what}: тип поля не выведен как ссылочный")),
        }
    }

    /// Поля вложенной табличной части в списке выборки `Т.ТЧ.(…)`: имя без псевдонима — поле табличной части, иначе
    /// поле внешних источников (реквизит шапки `ЕдиницаИзмерения` в `Товары.(…)` УведомлениеОВвозеПрослеживаемыхТоваров
    /// БП: запрос у платформы компилируется). shortcut: по одной форме, чей реквизит при совпадении имен — не выведено
    fn nested(&mut self, p: &[String], fields: &[Field], sc: &Scope) -> R<Option<Vec<Ty>>> {
        let (obj, ts) = match sc.srcs.iter().find(|(a, ..)| p.len() == 2 && a.as_deref().is_some_and(|a| eq(a, &p[0]))) {
            Some((_, Src::Obj(o), _)) if self.m.ts(o, &p[1]) => (o.clone(), p[1].clone()),
            _ => return Err(format!("поля табличной части {} не проверены", p.join("."))),
        };
        let inner = Scope { srcs: vec![(None, Src::Ts(obj, ts), false)], outs: Vec::new(), outer: Some(sc) };
        for f in fields {
            self.expr(&f.expr, &inner).map_err(|e| format!("{}.(…): {e}", p.join(".")))?;
        }
        Ok(Some(vec![Ty::Table]))
    }

    /// Вложенный запрос источника или списка `В`: без `ПОМЕСТИТЬ`.
    fn sub(&mut self, q: &Query, outer: Option<&Scope>) -> R<Out> {
        if q.parts.iter().any(|p| p.into.is_some()) {
            return Err("ПОМЕСТИТЬ во вложенном запросе не поддержано".into());
        }
        star_all(q)?;
        self.query(q, outer)
    }

    fn source(&mut self, s: &Source, out: &mut Srcs, outer: Option<&Scope>) -> R<()> {
        let (src, own_alias) = match &s.table {
            Table::Name { path, params } => {
                let src = match (class_en(&path[0]), path.len()) {
                    (Some(c), 2) if params.is_none() => Src::Obj(self.m.object(&format!("{c}.{}", path[1])).ok_or_else(|| absent(&format!("таблица {}", path.join("."))))?),
                    (Some(c), 3) => {
                        let o = self.m.object(&format!("{c}.{}", path[1])).ok_or_else(|| absent(&format!("таблица {}", path.join("."))))?;
                        if params.is_none() && self.m.ts(&o, &path[2]) {
                            Src::Ts(o, path[2].clone())
                        } else {
                            // проверка: виртуальная таблица поддержана (поле с пустым именем — только признак)
                            self.m.vt_field(&o, &path[2], "")?;
                            if let Some(ps) = params {
                                // (Период, Условие) у срезов и остатков, (Начало, Конец, Периодичность, Условие) у оборотов;
                                // условие — по полям параметров виртуальной таблицы, периодичность — слово из списка
                                let turnovers = eq(&path[2], "Обороты") || eq(&path[2], "Turnovers");
                                // остатки регистра бухгалтерии: (Период, УсловиеСчета, Субконто, Условие); список видов
                                // субконто не проверяется
                                let acc = c == "AccountingRegister" && (eq(&path[2], "Остатки") || eq(&path[2], "Balance"));
                                // остатки и обороты регистра бухгалтерии: (Начало, Конец, Периодичность, МетодДополнения,
                                // УсловиеСчета, Субконто, Условие); периодичность, метод и список субконто не проверяются
                                let acc_turn = c == "AccountingRegister" && (eq(&path[2], "ОстаткиИОбороты") || eq(&path[2], "BalanceAndTurnovers"));
                                // остатки и обороты регистра накопления: (Начало, Конец, Периодичность, МетодДополнения, Условие)
                                let bt = c == "AccumulationRegister" && (eq(&path[2], "ОстаткиИОбороты") || eq(&path[2], "BalanceAndTurnovers"));
                                if ps.len() > if acc_turn { 7 } else if bt { 5 } else if turnovers || acc { 4 } else { 2 } {
                                    return Err(format!("у виртуальной таблицы {} лишние параметры", path.join(".")));
                                }
                                let period = |e: &Option<Expr>| match e {
                                    None => true,
                                    Some(Expr::Path(w)) => w.len() == 1 && PERIODICITY.iter().any(|p| eq(p, &w[0])),
                                    _ => false,
                                };
                                if (turnovers || bt) && ps.get(2).is_some_and(|e| !period(e)) {
                                    return Err(format!("периодичность виртуальной таблицы {} не из списка", path.join(".")));
                                }
                                let reg = Scope { srcs: vec![(None, Src::VtParam(o.clone(), path[2].clone()), false)], outs: Vec::new(), outer: None };
                                let skip = |i: usize| ((turnovers || acc) && i == 2) || (acc_turn && matches!(i, 2 | 3 | 5)) || (bt && matches!(i, 2 | 3));
                                for p in ps.iter().enumerate().filter(|(i, _)| !skip(*i)).filter_map(|(_, p)| p.as_ref()) {
                                    match p {
                                        Expr::Dcs(fs) => {
                                            for f in fs {
                                                self.expr(&f.expr, &reg)?;
                                            }
                                        }
                                        e => {
                                            self.expr(e, &reg)?;
                                        }
                                    }
                                }
                            }
                            Src::Vt(o, path[2].clone())
                        }
                    }
                    // критерий отбора: один параметр — значение отбора, поле Ссылка (как виртуальная таблица без имени)
                    (None, 2) if eq(&path[0], "КритерийОтбора") || eq(&path[0], "FilterCriterion") => {
                        let o = self.m.object(&format!("FilterCriterion.{}", path[1])).ok_or_else(|| absent(&format!("таблица {}", path.join("."))))?;
                        let ps = params.as_deref().unwrap_or_default();
                        if ps.len() > 1 {
                            return Err(format!("у критерия отбора {} лишние параметры", path.join(".")));
                        }
                        let none = Scope { srcs: Vec::new(), outs: Vec::new(), outer: None };
                        for p in ps.iter().flatten() {
                            self.expr(p, &none)?;
                        }
                        Src::Vt(o, String::new())
                    }
                    // таблица констант: поля — константы (временная таблица с этим именем — она)
                    (None, 1) if params.is_none() && (eq(&path[0], "Константы") || eq(&path[0], "Constants")) && !self.temps.iter().any(|(n, _)| eq(n, &path[0])) => {
                        Src::Vt(CONSTANTS.into(), String::new())
                    }
                    (None, 1) if params.is_none() => {
                        let t = self.temps.iter().find(|(n, _)| eq(n, &path[0])).ok_or_else(|| missing(&format!("таблица {}", path[0])))?;
                        Src::Derived(t.1.clone())
                    }
                    _ => return Err(format!("источник {} не поддержан", path.join("."))),
                };
                (src, path.last().cloned())
            }
            Table::Sub(q) => (Src::Derived(self.sub(q, outer)?), None),
            Table::Group(g) => {
                self.source(g, out, outer)?;
                return self.joins(s, out, outer);
            }
        };
        out.push((s.alias.clone().or(own_alias), src, s.alias.is_some()));
        self.joins(s, out, outer)
    }

    fn joins(&mut self, s: &Source, out: &mut Srcs, outer: Option<&Scope>) -> R<()> {
        for j in &s.joins {
            self.source(&j.source, out, outer)?;
        }
        Ok(())
    }

    fn check_joins(&mut self, s: &Source, sc: &Scope) -> R<()> {
        if let Table::Group(g) = &s.table {
            self.check_joins(g, sc)?;
        }
        for j in &s.joins {
            self.expr(&j.on, sc)?;
            self.check_joins(&j.source, sc)?;
        }
        Ok(())
    }

    fn select(&mut self, s: &Select, outer: Option<&Scope>) -> R<Out> {
        let mut srcs = Vec::new();
        for f in &s.from {
            self.source(f, &mut srcs, outer)?;
        }
        for (i, (a, ..)) in srcs.iter().enumerate() {
            if let Some(a) = a.as_deref().filter(|a| srcs[..i].iter().any(|(b, ..)| b.as_deref().is_some_and(|b| eq(a, b)))) {
                return Err(format!("повторяющийся псевдоним источника {a}"));
            }
        }
        let mut sc = Scope { srcs, outs: Vec::new(), outer };
        for f in &s.from {
            self.check_joins(f, &sc)?;
        }
        let mut outs = Vec::new();
        for f in &s.fields {
            let t = match &f.expr {
                Expr::Nested(p, fs) => self.nested(p, fs, &sc)?,
                e => self.expr(e, &sc)?,
            };
            // имя без псевдонима (names.rs): у `Поле` и `Источник.Поле` — имя поля, `&Имя` — имя параметра, `Т.ТЧ.(…)` —
            // табличная часть (имена сравниваются без учета регистра); у прочих выражений не выведено (пустое)
            let src = |a: &str| sc.srcs.iter().any(|(x, ..)| x.as_deref().is_some_and(|x| eq(x, a)));
            let name = f.alias.clone().or_else(|| match super::names::unnamed(&f.expr) {
                super::names::Unnamed::Field(p) if p.len() == 1 || (p.len() == 2 && src(&p[0])) => p.last().cloned(),
                super::names::Unnamed::Fixed(n) => Some(n.to_string()),
                super::names::Unnamed::Table(p) => p.last().cloned(),
                _ => None,
            });
            outs.push((name.unwrap_or_default(), t));
        }
        for e in s.filter.iter().chain(&s.group).chain(&s.having) {
            self.expr(e, &sc)?;
        }
        // `{ВЫБРАТЬ Имя}` — имя поля выборки: платформа такой запрос компилирует, запись поля набора — под этим именем
        // (трасса qp-dcs-14-3, ИнтерфейсДокументовЭДО: КонтрольОтраженияВУчете, корень режима 9 построен)
        // `{ВЫБРАТЬ Имя.*}` — то же поле выборки с подполями; `{ВЫБРАТЬ Имя.* КАК Имя}` — то же (элемент расширения
        // называет его имя, mngbase FUN_1806692a0, см. dlist `query_root_fields`)
        let out_name = |f: &Field| matches!(&f.expr, Expr::Path(p) | Expr::Star(p) if p.len() == 1 && f.alias.as_deref().is_none_or(|a| eq(a, &p[0])) && outs.iter().any(|(n, _)| eq(n, &p[0])));
        for f in s.dcs_select.iter().filter(|f| !out_name(f)).chain(&s.dcs_where) {
            self.expr(&f.expr, &sc)?;
        }
        sc.outs = outs.clone();
        Ok(outs)
    }

    fn query(&mut self, q: &Query, outer: Option<&Scope>) -> R<Out> {
        Ok(self.query_parts(q, outer)?.into_iter().next().unwrap_or_default())
    }

    /// Поля каждой части `ОБЪЕДИНИТЬ` (поля запроса — первой части).
    fn query_parts(&mut self, q: &Query, outer: Option<&Scope>) -> R<Vec<Out>> {
        if q.parts.len() > 1 {
            star_all(q)?;
        }
        let mut parts: Vec<Out> = Vec::new();
        for p in &q.parts {
            let o = self.select(p, outer)?;
            if parts.first().is_some_and(|f| f.len() != o.len()) {
                return Err("разное количество полей в частях ОБЪЕДИНИТЬ".into());
            }
            parts.push(o);
        }
        let out = parts.first().cloned().unwrap_or_default();
        // ИНДЕКСИРОВАТЬ ПО — поля временной таблицы (справка платформы, INDEXBY): имена первой части (у ОБЪЕДИНИТЬ
        // индекс записан при последней части, ПОМЕСТИТЬ — при первой)
        for e in q.parts.iter().flat_map(|p| &p.index) {
            match e {
                Expr::Path(p) if q.parts.first().is_some_and(|f| f.into.is_some()) && p.len() == 1 && out.iter().any(|(n, _)| eq(n, &p[0])) => {}
                // поле источника первой части (`Т.Поле`, `Поле` не из выборки — ERP КОформлениюНакладных `Ссылка`): индекс
                // полей не меняет — только проверка, что поле есть
                Expr::Path(_) if q.parts.first().is_some_and(|f| f.into.is_some()) => {
                    let mut srcs = Vec::new();
                    for f in &q.parts[0].from {
                        self.source(f, &mut srcs, outer)?;
                    }
                    let sc = Scope { srcs, outs: out.clone(), outer };
                    self.expr(e, &sc)?;
                }
                _ => return Err("ИНДЕКСИРОВАТЬ ПО не по полю выборки временной таблицы не поддержано".into()),
            }
        }
        // УПОРЯДОЧИТЬ ПО — поля выборки или поля источников первой части
        for e in &q.order {
            if let Expr::Path(p) = e {
                if p.len() == 1 && out.iter().any(|(n, _)| eq(n, &p[0])) {
                    continue;
                }
            }
            let mut srcs = Vec::new();
            if let Some(s) = q.parts.first() {
                for f in &s.from {
                    self.source(f, &mut srcs, outer)?;
                }
            }
            let sc = Scope { srcs, outs: out.clone(), outer };
            self.expr(e, &sc)?;
        }
        Ok(parts)
    }
}

/// `ВЫБРАТЬ *` у запроса, чьи поля берутся по имени (вложенный запрос, временная таблица) или считаются по частям
/// (`ОБЪЕДИНИТЬ`): `*` здесь — одно поле без имени, состав не выводится — отказ, а не «поля нет» или неверное число полей.
fn star_all(q: &Query) -> R<()> {
    if q.parts.iter().any(|p| p.fields.iter().any(|f| matches!(&f.expr, Expr::Star(p) if p.is_empty()))) {
        return Err("`ВЫБРАТЬ *` во вложенном запросе, временной таблице или ОБЪЕДИНИТЬ: состав полей не выведен".into());
    }
    Ok(())
}

/// Проверка пакета: все таблицы, поля, значения существуют (см. модуль). Результат — поля последнего запроса.
pub fn check(m: &dyn Meta, b: &Batch) -> R<Out> {
    Ok(check_parts(m, b)?.into_iter().next().unwrap_or_default())
}

/// Как `check`, но поля каждой части `ОБЪЕДИНИТЬ` последнего запроса.
pub fn check_parts(m: &dyn Meta, b: &Batch) -> R<Vec<Out>> {
    let mut ck = Ck { m, temps: Vec::new() };
    let mut last = Vec::new();
    for i in 0..=b.queries.len() {
        // УНИЧТОЖИТЬ несуществующей таблицы — ошибка (справка платформы, temp_DROP)
        for (_, n) in b.drops.iter().filter(|(k, _)| *k == i) {
            let k = ck.temps.iter().position(|(t, _)| eq(t, n)).ok_or_else(|| missing(&format!("временная таблица {n}")))?;
            ck.temps.remove(k);
        }
        let Some(q) = b.queries.get(i) else { break };
        if q.parts.iter().skip(1).any(|p| p.into.is_some()) {
            return Err("ПОМЕСТИТЬ не в первой части ОБЪЕДИНИТЬ не поддержано".into());
        }
        last = ck.query_parts(q, None)?;
        if let Some(t) = q.parts.first().and_then(|p| p.into.clone()) {
            star_all(q)?;
            ck.temps.push((t, last.first().cloned().unwrap_or_default()));
        }
    }
    Ok(last)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct M;
    impl Meta for M {
        fn object(&self, obj: &str) -> Option<String> {
            ["Catalog.Банки", "Catalog.Города", "Enum.Виды", "InformationRegister.Курсы"].into_iter().find(|o| eq(o, obj)).map(String::from)
        }
        fn field(&self, obj: &str, name: &str) -> Result<Option<Vec<Ty>>, String> {
            Ok(match (obj, name) {
                ("Catalog.Банки", "Ссылка") => Some(vec![Ty::Ref("Catalog.Банки".into())]),
                ("Catalog.Банки", "Город") => Some(vec![Ty::Ref("Catalog.Города".into())]),
                ("Catalog.Города", "Наименование") => Some(vec![Ty::Prim]),
                _ => None,
            })
        }
        fn ts(&self, _: &str, _: &str) -> bool {
            false
        }
        fn ts_field(&self, _: &str, _: &str, _: &str) -> Result<Option<Vec<Ty>>, String> {
            Ok(None)
        }
        fn vt_field(&self, _: &str, _: &str, _: &str) -> Result<Option<Vec<Ty>>, String> {
            Err("нет".into())
        }
        fn vt_param(&self, _: &str, _: &str, _: &str) -> Result<Option<Vec<Ty>>, String> {
            Err("нет".into())
        }
        fn value(&self, obj: &str, name: &str) -> bool {
            obj == "Enum.Виды" && name == "Первый"
        }
    }

    #[test]
    fn paths_and_values() {
        let ok = |q: &str| check(&M, &parse(q).unwrap());
        assert!(ok("ВЫБРАТЬ Б.Ссылка, Б.Город.Наименование ИЗ Справочник.Банки КАК Б ГДЕ Б.Город <> ЗНАЧЕНИЕ(Перечисление.Виды.Первый)").is_ok());
        assert!(ok("ВЫБРАТЬ Б.Город.Код ИЗ Справочник.Банки КАК Б").unwrap_err().contains("не найдено"));
        assert!(ok("ВЫБРАТЬ 1 ИЗ Справочник.Нет КАК Б").unwrap_err().contains("не найдено"));
        assert!(ok("ВЫБРАТЬ ЗНАЧЕНИЕ(Перечисление.Виды.Второй) ИЗ Справочник.Банки КАК Б").is_err());
        assert!(ok("ВЫБРАТЬ Т.Город.Наименование ИЗ (ВЫБРАТЬ Б.Город КАК Город ИЗ Справочник.Банки КАК Б) КАК Т").is_ok());
        assert!(ok("ВЫБРАТЬ Б.Город КАК Г ПОМЕСТИТЬ ВТ ИЗ Справочник.Банки КАК Б; ВЫБРАТЬ ВТ.Г.Наименование ИЗ ВТ КАК ВТ").is_ok());
        assert!(ok("ВЫБРАТЬ Город ИЗ Справочник.Банки КАК Б").is_ok());
        assert!(ok("ВЫБРАТЬ Б.Ссылка ИЗ Справочник.банки КАК Б").is_ok());
        assert!(ok("ВЫБРАТЬ Б.Город.* ИЗ Справочник.Банки КАК Б").is_ok());
        assert!(ok("ВЫБРАТЬ * ИЗ Справочник.Банки КАК Б").is_ok());
        let bad = [
            "ВЫБРАТЬ Б.Город.Наименование.* ИЗ Справочник.Банки КАК Б",
            "ВЫБРАТЬ (ВЫБРАТЬ Г.Ссылка ИЗ Справочник.Города КАК Г) КАК Х ИЗ Справочник.Банки КАК Б",
            "ВЫБРАТЬ Б.Ссылка ИЗ Справочник.Банки КАК Б ГДЕ (Б.Ссылка, Б.Город) = 1",
            "ВЫБРАТЬ Б.Город.Наименование ПОМЕСТИТЬ ВТ ИЗ Справочник.Банки КАК Б; ВЫБРАТЬ ВТ.Наименование ИЗ ВТ КАК ВТ",
            "ВЫБРАТЬ Б.Ссылка КАК С ПОМЕСТИТЬ ВТ ИЗ Справочник.Банки КАК Б ИНДЕКСИРОВАТЬ ПО Нет",
            "ВЫБРАТЬ ВЫРАЗИТЬ(Б.Ссылка КАК РегистрСведений.Курсы) КАК Х ИЗ Справочник.Банки КАК Б",
            "ВЫБРАТЬ ЕСТЬNULL(Б.Ссылка) КАК Х ИЗ Справочник.Банки КАК Б",
            "ВЫБРАТЬ СУММА(РАЗЛИЧНЫЕ Б.Ссылка) КАК Х ИЗ Справочник.Банки КАК Б",
            "ВЫБРАТЬ Б.Ссылка КАК С ПОМЕСТИТЬ ВТ ИЗ Справочник.Банки КАК Б; УНИЧТОЖИТЬ ВТ; ВЫБРАТЬ ВТ.С ИЗ ВТ КАК ВТ",
            "ВЫБРАТЬ Б.Ссылка ИЗ Справочник.Банки КАК Б ЛЕВОЕ СОЕДИНЕНИЕ Справочник.Города КАК Б ПО ИСТИНА",
            "ВЫБРАТЬ Б.Ссылка ИЗ Справочник.Банки КАК Б ОБЪЕДИНИТЬ ВЫБРАТЬ Б.Ссылка, Б.Город ИЗ Справочник.Банки КАК Б",
            "ВЫБРАТЬ Т.Город ИЗ (ВЫБРАТЬ * ИЗ Справочник.Банки КАК Б) КАК Т",
            "ВЫБРАТЬ * ПОМЕСТИТЬ ВТ ИЗ Справочник.Банки КАК Б; ВЫБРАТЬ ВТ.Город ИЗ ВТ КАК ВТ",
            "ВЫБРАТЬ *, Б.Город ИЗ Справочник.Банки КАК Б ОБЪЕДИНИТЬ ВЫБРАТЬ Б.Ссылка, Б.Город ИЗ Справочник.Банки КАК Б",
        ];
        for q in bad {
            assert!(ok(q).is_err(), "{q}");
        }
        assert!(ok("ВЫБРАТЬ Справочник.Банки.Город.Наименование КАК С ИЗ Справочник.Банки").is_ok());
        assert!(ok("ВЫБРАТЬ Справочник.Банки.Ссылка КАК С ИЗ Справочник.Банки КАК Б").unwrap_err().contains("полному имени"));
    }
}
