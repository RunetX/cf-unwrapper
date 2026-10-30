//! Разбор текста запроса (свой разборщик, грамматика написана по синтаксису языка запросов; RU/EN ключевые слова).
//! Разбираются пакет, объединения, список выборки, источники (таблицы, виртуальные таблицы с параметрами, вложенные
//! запросы, соединения), условия, группировка, упорядочивание, итоги и расширения компоновки `{ВЫБРАТЬ …}`,
//! `{ГДЕ …}`, `{… СОЕДИНЕНИЕ …}`. Непонятая конструкция — ошибка с ее описанием, а не догадка.

use super::lex::{tokens, Tok};

#[derive(Clone, Debug, PartialEq)]
pub struct Batch {
    pub queries: Vec<Query>,
    /// `УНИЧТОЖИТЬ Таблица` между запросами пакета: число запросов перед ним и имя.
    pub drops: Vec<(usize, String)>,
}

/// Запрос пакета: части `ОБЪЕДИНИТЬ` (первая задает имена полей), упорядочивание, итоги.
#[derive(Clone, Debug, PartialEq)]
pub struct Query {
    pub parts: Vec<Select>,
    pub order: Vec<Expr>,
    pub totals: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Select {
    pub distinct: bool,
    pub top: Option<String>,
    pub fields: Vec<Field>,
    pub into: Option<String>,
    pub from: Vec<Source>,
    pub filter: Option<Expr>,
    pub group: Vec<Expr>,
    pub having: Option<Expr>,
    /// Расширения компоновки: `{ВЫБРАТЬ …}` (поля) и `{ГДЕ …}` (условия).
    pub dcs_select: Vec<Field>,
    pub dcs_where: Vec<Field>,
    /// `ИНДЕКСИРОВАТЬ ПО` временной таблицы.
    pub index: Vec<Expr>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Field {
    pub expr: Expr,
    pub alias: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Table {
    /// `Справочник.Банки`, `РегистрСведений.Курсы.СрезПоследних(&Дата, Условие)`; пропущенный параметр — `None`.
    Name { path: Vec<String>, params: Option<Vec<Option<Expr>>> },
    Sub(Box<Query>),
    /// Источник в скобках `(А ЛЕВОЕ СОЕДИНЕНИЕ Б ПО …)`.
    Group(Box<Source>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Source {
    pub table: Table,
    pub alias: Option<String>,
    pub joins: Vec<Join>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Join {
    /// `ЛЕВОЕ`, `ПРАВОЕ`, `ПОЛНОЕ`, `ВНУТРЕННЕЕ` (по-английски — так же, первым словом).
    pub kind: &'static str,
    /// Соединение в фигурных скобках компоновки.
    pub optional: bool,
    pub source: Source,
    pub on: Expr,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    /// `Т.Поле.Подполе`.
    Path(Vec<String>),
    /// `Т.*`.
    Star(Vec<String>),
    Num(String),
    Str(String),
    Param(String),
    /// `ИСТИНА`, `ЛОЖЬ`, `NULL`, `НЕОПРЕДЕЛЕНО` — английским словом.
    Const(&'static str),
    Neg(Box<Expr>),
    Not(Box<Expr>),
    Bin(&'static str, Box<Expr>, Box<Expr>),
    /// `ВЫБОР [x] КОГДА … ТОГДА … [ИНАЧЕ …] КОНЕЦ`.
    Case { of: Option<Box<Expr>>, when: Vec<(Expr, Expr)>, other: Option<Box<Expr>> },
    /// Функция: имя как в тексте (в верхнем регистре), аргументы; `РАЗЛИЧНЫЕ` у агрегатной — `distinct`.
    Call { name: String, distinct: bool, args: Vec<Expr> },
    /// `ВЫРАЗИТЬ(x КАК Тип)`, `ССЫЛКА Тип`, `ТИП(Тип)`, `ЗНАЧЕНИЕ(Путь)`.
    Cast(Box<Expr>, Type),
    Refs(Box<Expr>, Type),
    TypeOf(Type),
    Value(Vec<String>),
    /// `x [НЕ] В (…)` / `В ИЕРАРХИИ (…)`: список значений или вложенный запрос.
    In { x: Box<Expr>, not: bool, hierarchy: bool, list: Vec<Expr> },
    Sub(Box<Query>),
    Like { x: Box<Expr>, not: bool, pat: Box<Expr>, esc: Option<Box<Expr>> },
    Between { x: Box<Expr>, not: bool, lo: Box<Expr>, hi: Box<Expr> },
    IsNull { x: Box<Expr>, not: bool },
    /// Кортеж `(a, b)` слева от `В`.
    Tuple(Vec<Expr>),
    /// Обращение через точку к результату выражения: `ВЫРАЗИТЬ(… КАК Документ.Х).Дата`, `(…).*` — часть `*`.
    Dot(Box<Expr>, Vec<String>),
    /// Поля табличной части в списке выборки: `Т.Товары.(Номенклатура, Количество)`.
    Nested(Vec<String>, Vec<Field>),
    /// Параметр виртуальной таблицы в скобках компоновки: `{(Организация) КАК Организация}`.
    Dcs(Vec<Field>),
}

/// Тип: `Строка(100)`, `Число(15, 2)`, `Справочник.Банки`.
#[derive(Clone, Debug, PartialEq)]
pub struct Type {
    pub path: Vec<String>,
    pub args: Vec<String>,
}

/// Ключевые слова: (русское, английское) — в верхнем регистре.
const RESERVED: &[(&str, &str)] = &[
    ("ВЫБРАТЬ", "SELECT"),
    ("РАЗРЕШЕННЫЕ", "ALLOWED"),
    ("РАЗЛИЧНЫЕ", "DISTINCT"),
    ("ПЕРВЫЕ", "TOP"),
    ("КАК", "AS"),
    ("ПОМЕСТИТЬ", "INTO"),
    ("ИЗ", "FROM"),
    ("ГДЕ", "WHERE"),
    ("СГРУППИРОВАТЬ", "GROUP"),
    ("ИМЕЮЩИЕ", "HAVING"),
    ("ОБЪЕДИНИТЬ", "UNION"),
    ("ВСЕ", "ALL"),
    ("УПОРЯДОЧИТЬ", "ORDER"),
    ("ИТОГИ", "TOTALS"),
    ("ПО", "BY"),
    ("ДЛЯ", "FOR"),
    ("ИЗМЕНЕНИЯ", "UPDATE"),
    ("ИНДЕКСИРОВАТЬ", "INDEX"),
    ("ЛЕВОЕ", "LEFT"),
    ("ПРАВОЕ", "RIGHT"),
    ("ПОЛНОЕ", "FULL"),
    ("ВНУТРЕННЕЕ", "INNER"),
    ("ВНЕШНЕЕ", "OUTER"),
    ("СОЕДИНЕНИЕ", "JOIN"),
    ("И", "AND"),
    ("ИЛИ", "OR"),
    ("НЕ", "NOT"),
    ("В", "IN"),
    ("ИЕРАРХИИ", "HIERARCHY"),
    ("ПОДОБНО", "LIKE"),
    ("СПЕЦСИМВОЛ", "ESCAPE"),
    ("МЕЖДУ", "BETWEEN"),
    ("ЕСТЬ", "IS"),
    ("ССЫЛКА", "REFS"),
    ("ВЫБОР", "CASE"),
    ("КОГДА", "WHEN"),
    ("ТОГДА", "THEN"),
    ("ИНАЧЕ", "ELSE"),
    ("КОНЕЦ", "END"),
    ("ИСТИНА", "TRUE"),
    ("ЛОЖЬ", "FALSE"),
    ("NULL", "NULL"),
    ("НЕОПРЕДЕЛЕНО", "UNDEFINED"),
    ("ВОЗР", "ASC"),
    ("УБЫВ", "DESC"),
    ("АВТОУПОРЯДОЧИВАНИЕ", "AUTOORDER"),
    ("УНИЧТОЖИТЬ", "DROP"),
];

fn upper(s: &str) -> String {
    s.chars().flat_map(char::to_uppercase).map(|c| if c == 'Ё' { 'Е' } else { c }).collect()
}

/// Слово `s` — ключевое `ru`/`en` (без учета регистра).
fn is_kw(s: &str, ru: &str) -> bool {
    let u = upper(s);
    RESERVED.iter().any(|(r, e)| *r == ru && (u == *r || u == *e))
}

/// Слова, которые бывают и именами (`КАК Ссылка`, `Т.Порядок`): ключевые только на своем месте.
const CONTEXTUAL: &[&str] = &["ССЫЛКА", "ВСЕ", "ИЗМЕНЕНИЯ", "ИЕРАРХИИ", "СПЕЦСИМВОЛ", "ВНЕШНЕЕ", "ВОЗР", "УБЫВ", "АВТОУПОРЯДОЧИВАНИЕ"];

fn reserved(s: &str) -> bool {
    let u = upper(s);
    RESERVED.iter().any(|(r, e)| (u == *r || u == *e) && !CONTEXTUAL.contains(r))
}

/// Псевдоним без `КАК`: не ключевое слово, в том числе контекстное (`ИЗ Справочник.Банки АВТОУПОРЯДОЧИВАНИЕ`).
fn bare_alias(s: &str) -> bool {
    let u = upper(s);
    !RESERVED.iter().any(|(r, e)| u == *r || u == *e)
}

pub type R<T> = Result<T, String>;

struct P {
    t: Vec<Tok>,
    i: usize,
}

impl P {
    fn peek(&self) -> Option<&Tok> {
        self.t.get(self.i)
    }
    fn peek_at(&self, k: usize) -> Option<&Tok> {
        self.t.get(self.i + k)
    }
    fn kw_at(&self, k: usize, ru: &str) -> bool {
        matches!(self.peek_at(k), Some(Tok::Name(s)) if is_kw(s, ru))
    }
    fn kw(&self, ru: &str) -> bool {
        self.kw_at(0, ru)
    }
    fn eat_kw(&mut self, ru: &str) -> bool {
        let y = self.kw(ru);
        if y {
            self.i += 1;
        }
        y
    }
    fn want_kw(&mut self, ru: &str) -> R<()> {
        if self.eat_kw(ru) {
            Ok(())
        } else {
            Err(self.unexpected(&format!("ожидалось {ru}")))
        }
    }
    fn p(&self, s: &str) -> bool {
        matches!(self.peek(), Some(Tok::P(x)) if *x == s)
    }
    fn eat_p(&mut self, s: &str) -> bool {
        let y = self.p(s);
        if y {
            self.i += 1;
        }
        y
    }
    fn want_p(&mut self, s: &str) -> R<()> {
        if self.eat_p(s) {
            Ok(())
        } else {
            Err(self.unexpected(&format!("ожидалось «{s}»")))
        }
    }
    fn unexpected(&self, what: &str) -> String {
        let at = match self.peek() {
            None => "конец текста".to_string(),
            Some(Tok::Name(s)) => s.clone(),
            Some(Tok::P(s)) => format!("«{s}»"),
            Some(t) => format!("{t:?}"),
        };
        format!("конструкция запроса не поддержана: {what}, а встречено {at} (лексема {})", self.i)
    }
    /// Имя, не ключевое слово.
    fn name(&mut self) -> R<String> {
        match self.peek() {
            Some(Tok::Name(s)) if !reserved(s) => {
                let s = s.clone();
                self.i += 1;
                Ok(s)
            }
            _ => Err(self.unexpected("ожидалось имя")),
        }
    }
    /// Имя после точки: допустимо и ключевое слово (`Т.Тип`).
    fn part(&mut self) -> R<String> {
        match self.peek() {
            Some(Tok::Name(s)) => {
                let s = s.clone();
                self.i += 1;
                Ok(s)
            }
            _ => Err(self.unexpected("ожидалось имя после точки")),
        }
    }
    /// Псевдоним поля выборки: `КАК Имя` или просто `Имя`.
    fn alias(&mut self) -> R<Option<String>> {
        if self.eat_kw("КАК") {
            return Ok(Some(self.name()?));
        }
        Ok(match self.peek() {
            Some(Tok::Name(s)) if bare_alias(s) => Some(self.name()?),
            _ => None,
        })
    }
    fn path_rest(&mut self, mut v: Vec<String>) -> R<Vec<String>> {
        while self.p(".") && !matches!(self.peek_at(1), Some(Tok::P("*" | "("))) {
            self.i += 1;
            v.push(self.part()?);
        }
        Ok(v)
    }

    fn batch(&mut self) -> R<Batch> {
        let (mut queries, mut drops) = (Vec::new(), Vec::new());
        loop {
            if self.eat_kw("УНИЧТОЖИТЬ") {
                drops.push((queries.len(), self.name()?));
            } else {
                queries.push(self.query()?);
            }
            if !self.eat_p(";") || self.peek().is_none() {
                break;
            }
        }
        if self.peek().is_some() {
            return Err(self.unexpected("ожидался конец запроса"));
        }
        Ok(Batch { queries, drops })
    }

    fn query(&mut self) -> R<Query> {
        let mut parts = vec![self.select()?];
        while self.eat_kw("ОБЪЕДИНИТЬ") {
            self.eat_kw("ВСЕ");
            parts.push(self.select()?);
        }
        let mut order = Vec::new();
        if self.kw("УПОРЯДОЧИТЬ") {
            self.i += 1;
            self.want_kw("ПО")?;
            order = self.order_list()?;
        }
        if self.eat_kw("АВТОУПОРЯДОЧИВАНИЕ") {}
        let totals = self.kw("ИТОГИ");
        if totals {
            return Err(self.unexpected("ИТОГИ не поддержаны"));
        }
        Ok(Query { parts, order, totals })
    }

    fn order_list(&mut self) -> R<Vec<Expr>> {
        let mut v = Vec::new();
        loop {
            v.push(self.expr()?);
            if !self.eat_kw("ВОЗР") {
                self.eat_kw("УБЫВ");
            }
            if !self.eat_p(",") {
                return Ok(v);
            }
        }
    }

    fn select(&mut self) -> R<Select> {
        self.want_kw("ВЫБРАТЬ")?;
        let mut distinct = false;
        let mut top = None;
        loop {
            if self.eat_kw("РАЗРЕШЕННЫЕ") {
            } else if self.eat_kw("РАЗЛИЧНЫЕ") {
                distinct = true;
            } else if self.eat_kw("ПЕРВЫЕ") {
                match self.peek() {
                    Some(Tok::Num(n)) => {
                        top = Some(n.clone());
                        self.i += 1;
                    }
                    _ => return Err(self.unexpected("ожидалось число после ПЕРВЫЕ")),
                }
            } else {
                break;
            }
        }
        let mut fields = Vec::new();
        loop {
            // `ВЫБРАТЬ *` — все поля всех источников (`Star` без пути), псевдонима у него нет
            if self.eat_p("*") {
                fields.push(Field { expr: Expr::Star(Vec::new()), alias: None });
                if !self.eat_p(",") {
                    break;
                }
                continue;
            }
            let expr = self.expr()?;
            let alias = self.alias()?;
            fields.push(Field { expr, alias });
            if !self.eat_p(",") {
                break;
            }
        }
        let mut s = Select { distinct, top, fields, into: None, from: Vec::new(), filter: None, group: Vec::new(), having: None, dcs_select: Vec::new(), dcs_where: Vec::new(), index: Vec::new() };
        self.dcs(&mut s)?;
        if self.eat_kw("ПОМЕСТИТЬ") {
            s.into = Some(self.name()?);
        }
        if self.eat_kw("ИЗ") {
            loop {
                s.from.push(self.source()?);
                if !self.eat_p(",") {
                    break;
                }
            }
        }
        self.dcs(&mut s)?;
        if self.eat_kw("ГДЕ") {
            s.filter = Some(self.expr()?);
        }
        self.dcs(&mut s)?;
        if self.kw("СГРУППИРОВАТЬ") {
            self.i += 1;
            self.want_kw("ПО")?;
            loop {
                s.group.push(self.expr()?);
                if !self.eat_p(",") {
                    break;
                }
            }
        }
        if self.eat_kw("ИМЕЮЩИЕ") {
            s.having = Some(self.expr()?);
        }
        self.dcs(&mut s)?;
        if self.kw("ИНДЕКСИРОВАТЬ") {
            self.i += 1;
            self.want_kw("ПО")?;
            loop {
                s.index.push(self.expr()?);
                if !self.eat_p(",") {
                    break;
                }
            }
        }
        if self.kw("ДЛЯ") && self.kw_at(1, "ИЗМЕНЕНИЯ") {
            return Err(self.unexpected("ДЛЯ ИЗМЕНЕНИЯ не поддержано"));
        }
        Ok(s)
    }

    /// Расширения компоновки `{ВЫБРАТЬ …}` и `{ГДЕ …}` (может быть несколько подряд).
    fn dcs(&mut self, s: &mut Select) -> R<()> {
        while self.p("{") && (self.kw_at(1, "ВЫБРАТЬ") || self.kw_at(1, "ГДЕ")) {
            self.i += 1;
            let sel = self.eat_kw("ВЫБРАТЬ");
            if !sel {
                self.want_kw("ГДЕ")?;
            }
            loop {
                let expr = self.expr()?;
                let alias = if self.eat_kw("КАК") { Some(self.name()?) } else { None };
                if sel {
                    s.dcs_select.push(Field { expr, alias });
                } else {
                    s.dcs_where.push(Field { expr, alias });
                }
                if !self.eat_p(",") {
                    break;
                }
            }
            self.want_p("}")?;
        }
        Ok(())
    }

    fn source(&mut self) -> R<Source> {
        let table = if self.eat_p("(") {
            if self.kw("ВЫБРАТЬ") {
                let q = self.query()?;
                self.want_p(")")?;
                Table::Sub(Box::new(q))
            } else {
                let s = self.source()?;
                self.want_p(")")?;
                Table::Group(Box::new(s))
            }
        } else {
            let n = self.name()?;
            let path = self.path_rest(vec![n])?;
            let params = if self.eat_p("(") { Some(self.params()?) } else { None };
            Table::Name { path, params }
        };
        let alias = if self.eat_kw("КАК") {
            Some(self.name()?)
        } else {
            match self.peek() {
                Some(Tok::Name(s)) if bare_alias(s) => Some(self.name()?),
                _ => None,
            }
        };
        let mut joins = Vec::new();
        let mut optional = false;
        loop {
            if optional && self.eat_p("}") {
                optional = false;
                continue;
            }
            if !optional && self.p("{") && self.join_ahead(1) {
                self.i += 1;
                optional = true;
            }
            if !self.join_ahead(0) {
                if optional {
                    return Err(self.unexpected("ожидалось «}»"));
                }
                break;
            }
            let kind = if self.eat_kw("ЛЕВОЕ") {
                "LEFT"
            } else if self.eat_kw("ПРАВОЕ") {
                "RIGHT"
            } else if self.eat_kw("ПОЛНОЕ") {
                "FULL"
            } else {
                self.eat_kw("ВНУТРЕННЕЕ");
                "INNER"
            };
            self.eat_kw("ВНЕШНЕЕ");
            self.want_kw("СОЕДИНЕНИЕ")?;
            let source = self.source()?;
            self.want_kw("ПО")?;
            let on = self.expr()?;
            joins.push(Join { kind, optional, source, on });
        }
        Ok(Source { table, alias, joins })
    }

    fn join_ahead(&self, k: usize) -> bool {
        ["ЛЕВОЕ", "ПРАВОЕ", "ПОЛНОЕ", "ВНУТРЕННЕЕ", "СОЕДИНЕНИЕ"].iter().any(|w| self.kw_at(k, w))
    }

    /// Параметры виртуальной таблицы: `(&Дата, , Условие)`, параметр компоновки `{(Поле) КАК Поле}`.
    fn params(&mut self) -> R<Vec<Option<Expr>>> {
        let mut v = Vec::new();
        if self.eat_p(")") {
            return Ok(v);
        }
        loop {
            if self.p(",") || self.p(")") {
                v.push(None);
            } else {
                // условие может идти перед частью компоновки: `ЗаказПоставщику ССЫЛКА Документ.ЗаказПоставщику {(Склад) КАК
                // Склад}` (ERP СписокНакладныеКОформлению) — условие первым полем без псевдонима
                let lead = if self.p("{") { None } else { Some(self.expr()?) };
                if self.eat_p("{") {
                    let mut f: Vec<Field> = lead.into_iter().map(|expr| Field { expr, alias: None }).collect();
                    loop {
                        let expr = self.expr()?;
                        let alias = if self.eat_kw("КАК") { Some(self.name()?) } else { None };
                        f.push(Field { expr, alias });
                        if !self.eat_p(",") {
                            break;
                        }
                    }
                    self.want_p("}")?;
                    v.push(Some(Expr::Dcs(f)));
                } else {
                    v.push(lead);
                }
            }
            if self.eat_p(")") {
                return Ok(v);
            }
            self.want_p(",")?;
        }
    }

    fn expr(&mut self) -> R<Expr> {
        let mut a = self.and()?;
        while self.eat_kw("ИЛИ") {
            a = Expr::Bin("OR", Box::new(a), Box::new(self.and()?));
        }
        Ok(a)
    }
    fn and(&mut self) -> R<Expr> {
        let mut a = self.not()?;
        while self.eat_kw("И") {
            a = Expr::Bin("AND", Box::new(a), Box::new(self.not()?));
        }
        Ok(a)
    }
    fn not(&mut self) -> R<Expr> {
        if self.eat_kw("НЕ") {
            return Ok(Expr::Not(Box::new(self.not()?)));
        }
        self.cmp()
    }
    fn cmp(&mut self) -> R<Expr> {
        let mut x = self.add()?;
        // сравнения подряд — слева направо: `&Флаг = Т.Ссылка <> ЗНАЧЕНИЕ(…)` (ERP НастройкаСпособовОбеспеченияПотребностей)
        let mut chained = false;
        while let Some(op) = ["=", "<>", "<=", ">=", "<", ">"].into_iter().find(|op| self.p(op)) {
            self.i += 1;
            x = Expr::Bin(op, Box::new(x), Box::new(self.add()?));
            chained = true;
        }
        if chained {
            return Ok(x);
        }
        let not = self.kw("НЕ") && (self.kw_at(1, "В") || self.kw_at(1, "ПОДОБНО") || self.kw_at(1, "МЕЖДУ"));
        if not {
            self.i += 1;
        }
        if self.eat_kw("В") {
            let hierarchy = self.eat_kw("ИЕРАРХИИ");
            self.want_p("(")?;
            let list = if self.kw("ВЫБРАТЬ") {
                vec![Expr::Sub(Box::new(self.query()?))]
            } else {
                let mut v = vec![self.expr()?];
                while self.eat_p(",") {
                    v.push(self.expr()?);
                }
                v
            };
            self.want_p(")")?;
            return Ok(Expr::In { x: Box::new(x), not, hierarchy, list });
        }
        if self.eat_kw("ПОДОБНО") {
            let pat = Box::new(self.add()?);
            let esc = if self.eat_kw("СПЕЦСИМВОЛ") { Some(Box::new(self.add()?)) } else { None };
            return Ok(Expr::Like { x: Box::new(x), not, pat, esc });
        }
        if self.eat_kw("МЕЖДУ") {
            let lo = Box::new(self.add()?);
            self.want_kw("И")?;
            let hi = Box::new(self.add()?);
            return Ok(Expr::Between { x: Box::new(x), not, lo, hi });
        }
        if not {
            return Err(self.unexpected("НЕ после выражения"));
        }
        if self.eat_kw("ЕСТЬ") {
            let not = self.eat_kw("НЕ");
            self.want_kw("NULL")?;
            return Ok(Expr::IsNull { x: Box::new(x), not });
        }
        if self.eat_kw("ССЫЛКА") {
            return Ok(Expr::Refs(Box::new(x), self.ty()?));
        }
        Ok(x)
    }
    fn add(&mut self) -> R<Expr> {
        let mut a = self.mul()?;
        loop {
            let op = if self.eat_p("+") {
                "+"
            } else if self.eat_p("-") {
                "-"
            } else {
                return Ok(a);
            };
            a = Expr::Bin(op, Box::new(a), Box::new(self.mul()?));
        }
    }
    fn mul(&mut self) -> R<Expr> {
        let mut a = self.unary()?;
        loop {
            let op = if self.eat_p("*") {
                "*"
            } else if self.eat_p("/") {
                "/"
            } else {
                return Ok(a);
            };
            a = Expr::Bin(op, Box::new(a), Box::new(self.unary()?));
        }
    }
    fn unary(&mut self) -> R<Expr> {
        if self.eat_p("-") {
            return Ok(Expr::Neg(Box::new(self.unary()?)));
        }
        if self.eat_p("+") {
            return self.unary();
        }
        self.primary()
    }

    fn ty(&mut self) -> R<Type> {
        let n = self.part()?;
        let path = self.path_rest(vec![n])?;
        let mut args = Vec::new();
        if self.eat_p("(") {
            loop {
                match self.peek() {
                    Some(Tok::Num(n)) => args.push(n.clone()),
                    Some(Tok::Name(n)) => args.push(n.clone()),
                    _ => return Err(self.unexpected("ожидался квалификатор типа")),
                }
                self.i += 1;
                if !self.eat_p(",") {
                    break;
                }
            }
            self.want_p(")")?;
        }
        Ok(Type { path, args })
    }

    fn primary(&mut self) -> R<Expr> {
        let paren = self.p("(");
        let x = self.atom()?;
        if self.p(".") && (paren || !matches!(x, Expr::Path(_) | Expr::Star(_) | Expr::Nested(..))) {
            let mut v = Vec::new();
            while self.eat_p(".") {
                if self.eat_p("*") {
                    v.push("*".into());
                    break;
                }
                v.push(self.part()?);
            }
            return Ok(Expr::Dot(Box::new(x), v));
        }
        Ok(x)
    }

    fn atom(&mut self) -> R<Expr> {
        let t = self.peek().cloned().ok_or_else(|| self.unexpected("ожидалось выражение"))?;
        match t {
            Tok::Num(n) => {
                self.i += 1;
                Ok(Expr::Num(n))
            }
            Tok::Str(s) => {
                self.i += 1;
                Ok(Expr::Str(s))
            }
            Tok::Param(s) => {
                self.i += 1;
                Ok(Expr::Param(s))
            }
            Tok::P("(") => {
                self.i += 1;
                if self.kw("ВЫБРАТЬ") {
                    let q = self.query()?;
                    self.want_p(")")?;
                    return Ok(Expr::Sub(Box::new(q)));
                }
                let mut v = vec![self.expr()?];
                while self.eat_p(",") {
                    v.push(self.expr()?);
                }
                self.want_p(")")?;
                Ok(if v.len() == 1 { v.pop().unwrap_or(Expr::Const("NULL")) } else { Expr::Tuple(v) })
            }
            Tok::Name(s) => {
                for (ru, en) in [("ИСТИНА", "TRUE"), ("ЛОЖЬ", "FALSE"), ("NULL", "NULL"), ("НЕОПРЕДЕЛЕНО", "UNDEFINED")] {
                    if is_kw(&s, ru) {
                        self.i += 1;
                        return Ok(Expr::Const(en));
                    }
                }
                if is_kw(&s, "ВЫБОР") {
                    self.i += 1;
                    return self.case();
                }
                if matches!(self.peek_at(1), Some(Tok::P("("))) && !reserved(&s) {
                    self.i += 2;
                    return self.call(upper(&s));
                }
                let n = self.name()?;
                let path = self.path_rest(vec![n])?;
                if self.p(".") && matches!(self.peek_at(1), Some(Tok::P("*"))) {
                    self.i += 2;
                    return Ok(Expr::Star(path));
                }
                if self.p(".") && matches!(self.peek_at(1), Some(Tok::P("("))) {
                    self.i += 2;
                    let mut v = Vec::new();
                    loop {
                        let expr = self.expr()?;
                        let alias = if self.eat_kw("КАК") { Some(self.name()?) } else { None };
                        v.push(Field { expr, alias });
                        if self.eat_p(")") {
                            break;
                        }
                        self.want_p(",")?;
                    }
                    return Ok(Expr::Nested(path, v));
                }
                Ok(Expr::Path(path))
            }
            _ => Err(self.unexpected("ожидалось выражение")),
        }
    }

    fn case(&mut self) -> R<Expr> {
        let of = if self.kw("КОГДА") { None } else { Some(Box::new(self.expr()?)) };
        let mut when = Vec::new();
        while self.eat_kw("КОГДА") {
            let c = self.expr()?;
            self.want_kw("ТОГДА")?;
            when.push((c, self.expr()?));
        }
        if when.is_empty() {
            return Err(self.unexpected("ВЫБОР без КОГДА"));
        }
        let other = if self.eat_kw("ИНАЧЕ") { Some(Box::new(self.expr()?)) } else { None };
        self.want_kw("КОНЕЦ")?;
        Ok(Expr::Case { of, when, other })
    }

    /// Функция после `имя(`.
    fn call(&mut self, name: String) -> R<Expr> {
        match name.as_str() {
            "ВЫРАЗИТЬ" | "CAST" => {
                let x = self.expr()?;
                self.want_kw("КАК")?;
                let t = self.ty()?;
                self.want_p(")")?;
                return Ok(Expr::Cast(Box::new(x), t));
            }
            "ЗНАЧЕНИЕ" | "VALUE" => {
                let n = self.part()?;
                let p = self.path_rest(vec![n])?;
                self.want_p(")")?;
                return Ok(Expr::Value(p));
            }
            "ТИП" | "TYPE" => {
                let t = self.ty()?;
                self.want_p(")")?;
                return Ok(Expr::TypeOf(t));
            }
            _ => {}
        }
        let distinct = self.eat_kw("РАЗЛИЧНЫЕ");
        let mut args = Vec::new();
        if !self.eat_p(")") {
            loop {
                if self.p("*") {
                    self.i += 1;
                    args.push(Expr::Star(Vec::new()));
                } else {
                    args.push(self.expr()?);
                }
                if self.eat_p(")") {
                    break;
                }
                self.want_p(",")?;
            }
        }
        Ok(Expr::Call { name, distinct, args })
    }
}

/// Разбор текста запроса целиком.
pub fn parse(text: &str) -> R<Batch> {
    P { t: tokens(text)?, i: 0 }.batch()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn select_with_dcs_and_joins() {
        let b = parse(
            "ВЫБРАТЬ РАЗЛИЧНЫЕ Т.Ссылка, Т.Сумма * 2 КАК Сумма, ВЫБОР КОГДА Т.Х В (&А) ТОГДА 1 ИНАЧЕ 0 КОНЕЦ КАК Ф
             ИЗ Документ.Счет КАК Т {ЛЕВОЕ СОЕДИНЕНИЕ РегистрСведений.Курсы.СрезПоследних(, Валюта = &В) КАК К ПО Т.Валюта = К.Валюта}
             ГДЕ НЕ Т.ПометкаУдаления {ГДЕ Т.*}; ВЫБРАТЬ 1 ОБЪЕДИНИТЬ ВСЕ ВЫБРАТЬ 2",
        )
        .unwrap();
        assert_eq!(b.queries.len(), 2);
        let s = &b.queries[0].parts[0];
        assert!(s.distinct);
        assert_eq!(s.fields[0], Field { expr: Expr::Path(vec!["Т".into(), "Ссылка".into()]), alias: None });
        assert_eq!(s.fields[2].alias.as_deref(), Some("Ф"));
        assert_eq!(s.from[0].alias.as_deref(), Some("Т"));
        let j = &s.from[0].joins[0];
        assert!(j.optional);
        assert_eq!(j.kind, "LEFT");
        assert!(matches!(&j.source.table, Table::Name { params: Some(p), .. } if p.len() == 2 && p[0].is_none()));
        assert_eq!(s.dcs_where, vec![Field { expr: Expr::Star(vec!["Т".into()]), alias: None }]);
        assert_eq!(b.queries[1].parts.len(), 2);
    }

    #[test]
    fn condition_before_dcs_param_and_chained_compare() {
        let b = parse("ВЫБРАТЬ 1 ИЗ Р.Остатки(, З ССЫЛКА Документ.З {(Склад) КАК Склад}) КАК Т ГДЕ &П = Т.А <> 1").unwrap();
        let s = &b.queries[0].parts[0];
        assert!(matches!(&s.from[0].table, Table::Name { params: Some(p), .. }
            if matches!(&p[1], Some(Expr::Dcs(f)) if f.len() == 2 && f[0].alias.is_none() && f[1].alias.as_deref() == Some("Склад"))));
        assert!(matches!(&s.filter, Some(Expr::Bin("<>", a, _)) if matches!(**a, Expr::Bin("=", ..))));
    }

    #[test]
    fn corpus_constructs() {
        let b = parse(
            "ВЫБРАТЬ Ложь ПометкаУдаления, ВЫРАЗИТЬ(Т.Д КАК Документ.А).Дата КАК Дата, (Т.Ссылка).*, Т.Товары.(Номенклатура КАК Н)
             ПОМЕСТИТЬ ВТ ИЗ РегистрСведений.Ставки.СрезПоследних(, {(Организация) КАК Организация, (ОС) КАК Ссылка}) КАК Т
             {ЛЕВОЕ СОЕДИНЕНИЕ А КАК А ПО ИСТИНА ЛЕВОЕ СОЕДИНЕНИЕ Б КАК Б ПО ИСТИНА} {ГДЕ Т.Организация КАК Организация};
             УНИЧТОЖИТЬ ВТ",
        )
        .unwrap();
        let s = &b.queries[0].parts[0];
        assert_eq!(s.fields[0].alias.as_deref(), Some("ПометкаУдаления"));
        assert!(matches!(&s.fields[1].expr, Expr::Dot(_, p) if p == &["Дата"]));
        assert!(matches!(&s.fields[2].expr, Expr::Dot(_, p) if p == &["*"]));
        assert!(matches!(&s.fields[3].expr, Expr::Nested(p, f) if p.len() == 2 && f.len() == 1));
        assert_eq!(s.from[0].joins.len(), 2);
        assert_eq!(s.dcs_where[0].alias.as_deref(), Some("Организация"));
        assert_eq!(b.drops, vec![(1, "ВТ".to_string())]);
    }

    #[test]
    fn refuses_unknown() {
        assert!(parse("ВЫБРАТЬ Т.А ИЗ Т ИТОГИ СУММА(А) ПО ОБЩИЕ").unwrap_err().contains("ИТОГИ"));
        assert!(parse("ВЫБРАТЬ Т.А ИЗ").is_err());
        // контекстное слово после источника без КАК — не псевдоним (ревью: АВТОУПОРЯДОЧИВАНИЕ)
        let b = parse("ВЫБРАТЬ 1 КАК А ИЗ Справочник.Банки АВТОУПОРЯДОЧИВАНИЕ").unwrap();
        assert_eq!(b.queries[0].parts[0].from[0].alias, None);
    }

}
