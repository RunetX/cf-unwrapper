//! Табличный документ макета (`MOXCEL` + скобочный текст) → Ext/Template.xml (`<document>`).
//! Раскладка полей и таблицы значений — docs/research/xml-mxl.md.
//! Незнакомое значение — ошибка: файл не пишется и виден в сверке как отсутствующий.

use crate::brace::{self, Node};
use crate::meta::Config;

const HEAD: &str = "\u{feff}<?xml version=\"1.0\" encoding=\"UTF-8\"?>\r\n<document xmlns=\"http://v8.1c.ru/8.2/data/spreadsheet\" xmlns:pal=\"http://v8.1c.ru/8.1/data/ui/colors/palette\" xmlns:style=\"http://v8.1c.ru/8.1/data/ui/style\" xmlns:v8=\"http://v8.1c.ru/8.1/data/core\" xmlns:v8ui=\"http://v8.1c.ru/8.1/data/ui\" xmlns:xs=\"http://www.w3.org/2001/XMLSchema\" xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\">\r\n";
const ZERO: &str = "00000000-0000-0000-0000-000000000000";
const CELL_LINE: &str = "f527dc88-1d39-40b3-bcbb-d98b690ead68";
const DRAWING_LINE: &str = "b7438842-27cc-42a3-846f-2250cd9c1bc3";
const DCS_DETAIL: &str = "3031edd8-c3df-47b2-98ca-47f628d4ec18";
const STRUCTURE: &str = "4238019d-7e49-4fc9-91db-b6b951d5cf8e";
const MAP: &str = "3d48feae-a9c6-4c5a-a099-9eb6477630c6";
const CHART: &str = "a8b97779-1a4b-4059-b09c-807f86d2a461";
const GANTT: &str = "e5fdc112-5c84-4a16-9728-72b85692b6e2";
const VALUE_LIST: &str = "4772b3b4-f4a3-49c0-a1a5-8cb5961511a3";
const ARRAY: &str = "51e7a0d2-530b-11d4-b98a-008048da3034";
/// Стандартные картинки платформы по uuid.
/// shortcut: только встреченные в макетах БП 3.0.171; при слиянии заменить общей таблицей стандартных картинок.
const STD_PICTURES: &[(&str, &str)] = &[
    ("4b54770b-d069-4c0e-9b17-5cc2a01134d9", "Information"),
    ("818ab7d0-4654-4542-bd5e-fd9d1352b5a1", "SaveFile"),
    ("85998f14-805b-4e2b-ba19-9d79b0464042", "AppearanceCheckIcon"),
];

/// Порядок обхода Соответствия со строковыми ключами, вставленными в порядке записи (индексы `keys`).
/// core85.dll 8.5.1.1529 (xml-mxl.md «Порядок пар Соответствия»): boost::unordered_map — 16 корзин (конструктор
/// 0x1801c57c0), коэффициент 1.0, число корзин — степень двойки не меньше 4 (0x1800b3ed0); хеш — Wang-перемешивание
/// 64 бит от hashCode ключа (0x1801d1370), hashCode строки — `h·37 + код UTF-16` (core::hash 0x1802d8630); вставка
/// (0x1801d2f30): ключ в пустую корзину — в начало списка, в занятую — в начало ее группы; перехеширование 0x1801d3240.
fn map_order(keys: &[&str]) -> Vec<usize> {
    let hash = |s: &str| {
        let h = s.encode_utf16().fold(0u32, |h, c| h.wrapping_mul(37).wrapping_add(c as u32));
        let mut k = h as i32 as i64 as u64;
        k = k.wrapping_mul(0x200000).wrapping_add(!k);
        k = ((k >> 24) ^ k).wrapping_mul(0x109);
        k = ((k >> 14) ^ k).wrapping_mul(0x15);
        ((k >> 28) ^ k).wrapping_mul(0x8000_0001)
    };
    // число корзин для n элементов: floor(n / 1.0) + 1 вверх до степени двойки, не меньше 4
    let min_buckets = |n: usize| (n + 1).max(4).next_power_of_two();
    let mut buckets = 16usize.max(min_buckets(1));
    let mut list: Vec<usize> = Vec::new();
    let place = |list: &mut Vec<usize>, i: usize, nb: usize| {
        let b = hash(keys[i]) & (nb as u64 - 1);
        match list.iter().position(|&j| hash(keys[j]) & (nb as u64 - 1) == b) {
            Some(p) => list.insert(p, i),
            None => list.insert(0, i),
        }
    };
    for i in 0..keys.len() {
        if keys[..i].contains(&keys[i]) {
            continue; // повтор ключа заменяет значение, место прежнее
        }
        let size = list.len();
        if size + 1 > buckets {
            let want = if size + size / 2 > size + 1 { size + size / 2 } else { size + 1 };
            let nb = min_buckets(want);
            if nb != buckets {
                // перехеширование: обход в прежнем порядке, в пустую корзину — на место, в занятую — в начало группы
                let old = std::mem::take(&mut list);
                for j in old {
                    let b = hash(keys[j]) & (nb as u64 - 1);
                    match list.iter().position(|&x| hash(keys[x]) & (nb as u64 - 1) == b) {
                        Some(p) => list.insert(p, j),
                        None => list.push(j),
                    }
                }
                buckets = nb;
            }
        }
        place(&mut list, i, buckets);
    }
    list
}

/// Табличный документ ли это: запись макета начинается с `MOXCEL\0`.
pub fn is_mxl(raw: &[u8]) -> bool {
    raw.starts_with(b"MOXCEL\0")
}

type R<T> = Result<T, String>;

/// Макет из записи контейнера: None — не табличный документ.
pub fn template_entry(cfg: &Config, entry: &str) -> Option<R<Vec<u8>>> {
    template_data(cfg, &crate::container::content(&cfg.entries[*cfg.index.get(entry)?]))
}

/// Содержимое записи макета → `Template.xml`; `None` — это не табличный документ.
pub fn template_data(cfg: &Config, raw: &[u8]) -> Option<R<Vec<u8>>> {
    let names = |class: &str, id: &str| match class {
        "Type" => ref_type(cfg, id),
        _ => cfg.groups.iter().filter(|g| g.class == class).flat_map(|g| &g.items).find(|it| it.header.id == id).map(|it| it.header.name.clone()),
    };
    is_mxl(raw).then(|| template(raw, &names))
}

/// Ссылочный тип по его uuid: `CatalogRef.Имя`. Uuid типа стоит в корне описания объекта, `[1][позиция]`.
/// shortcut: перебор описаний (кэш по uuid на процесс) и только типы ссылок; при слиянии заменить общим индексом типов.
fn ref_type(cfg: &Config, id: &str) -> Option<String> {
    static CACHE: std::sync::Mutex<Vec<(String, Option<String>)>> = std::sync::Mutex::new(Vec::new());
    if let Some((_, n)) = CACHE.lock().ok()?.iter().find(|(k, _)| k == id) {
        return n.clone();
    }
    let n = find_ref_type(cfg, id);
    CACHE.lock().ok()?.push((id.to_string(), n.clone()));
    n
}

fn find_ref_type(cfg: &Config, id: &str) -> Option<String> {
    const REF: &[(&str, usize)] = &[("Catalog", 3), ("Document", 3), ("Enum", 1), ("ChartOfAccounts", 3), ("ChartOfCharacteristicTypes", 3), ("ExchangePlan", 3), ("ChartOfCalculationTypes", 4), ("BusinessProcess", 5), ("Task", 5)];
    for g in &cfg.groups {
        let Some(&(class, pos)) = REF.iter().find(|(c, _)| *c == g.class) else { continue };
        for it in &g.items {
            let Some(t) = cfg.entry_text(&it.header.id).filter(|t| t.contains(id)) else { continue };
            let doc = brace::Doc::parse(&t).ok()?;
            if doc.roots.first().and_then(|r| r.at(&[1, pos])).and_then(Node::atom) == Some(id) {
                return Some(format!("{class}Ref.{}", it.header.name));
            }
        }
    }
    None
}

/// Последовательное чтение элементов списка верхнего уровня.
struct Cur<'a> {
    l: &'a [Node],
    i: usize,
}

impl<'a> Cur<'a> {
    fn new(l: &'a [Node]) -> Self {
        Cur { l, i: 0 }
    }
    fn next(&mut self) -> R<&'a Node> {
        let n = self.l.get(self.i).ok_or_else(|| format!("нет элемента {}", self.i))?;
        self.i += 1;
        Ok(n)
    }
    fn int(&mut self) -> R<i64> {
        int(self.next()?)
    }
    fn count(&mut self) -> R<usize> {
        usize::try_from(self.int()?).map_err(|_| "отрицательное число элементов".to_string())
    }
    fn list(&mut self) -> R<&'a [Node]> {
        match self.next()? {
            Node::List(v) => Ok(v),
            n => Err(format!("ждали список, а не {n:?}")),
        }
    }
    fn rest(&self) -> &'a [Node] {
        &self.l[self.i.min(self.l.len())..]
    }
    fn expect(&mut self, what: &str) -> R<()> {
        let n = self.next()?;
        if atom(n) == Some(what) {
            Ok(())
        } else {
            Err(format!("ждали {what}, а не {}", short(n)))
        }
    }
}

fn atom(n: &Node) -> Option<&str> {
    n.atom()
}

fn int(n: &Node) -> R<i64> {
    n.atom().and_then(|a| a.parse().ok()).ok_or_else(|| format!("ждали число, а не {}", short(n)))
}

fn short(n: &Node) -> String {
    let s = brace::write(n).unwrap_or_else(|_| format!("{n:?}")).replace("\r\n", "");
    s.chars().take(120).collect()
}

fn list(n: &Node) -> R<&[Node]> {
    match n {
        Node::List(v) => Ok(v),
        _ => Err(format!("ждали список, а не {}", short(n))),
    }
}

/// Нумерация по первому использованию: платформа пишет только используемые элементы, в порядке обращения.
struct Order<K> {
    keys: Vec<K>,
}

impl<K: PartialEq + Clone> Order<K> {
    fn new() -> Self {
        Order { keys: Vec::new() }
    }
    fn id(&mut self, k: K) -> usize {
        match self.keys.iter().position(|x| *x == k) {
            Some(i) => i,
            None => {
                self.keys.push(k);
                self.keys.len() - 1
            }
        }
    }
}

/// XML в раскладке платформы: табуляции, CRLF.
struct X {
    o: String,
    d: usize,
}

impl X {
    fn pad(&mut self) {
        for _ in 0..self.d {
            self.o.push('\t');
        }
    }
    fn open(&mut self, tag: &str) {
        self.pad();
        self.o.push('<');
        self.o.push_str(tag);
        self.o.push_str(">\r\n");
        self.d += 1;
    }
    fn close(&mut self, tag: &str) {
        self.d -= 1;
        self.pad();
        self.o.push_str("</");
        self.o.push_str(tag);
        self.o.push_str(">\r\n");
    }
    fn empty(&mut self, tag: &str) {
        self.pad();
        self.o.push('<');
        self.o.push_str(tag);
        self.o.push_str("/>\r\n");
    }
    /// `<tag>текст</tag>`; текст уже экранирован.
    fn leaf(&mut self, tag: &str, v: &str) {
        self.pad();
        let name = tag.split(' ').next().unwrap_or(tag);
        self.o.push_str(&format!("<{tag}>{v}</{name}>\r\n"));
    }
    fn text(&mut self, tag: &str, v: &str) {
        let v = esc(v);
        self.leaf(tag, &v);
    }
    /// Многоязычная строка `{1,n,{"ru","текст"},...}`; пустая — `<tag/>`.
    fn ml(&mut self, tag: &str, n: &Node) -> R<()> {
        let l = list(n)?;
        if l.first().and_then(atom) != Some("1") {
            return Err(format!("многоязычная строка: {}", short(n)));
        }
        let cnt = int(l.get(1).ok_or("многоязычная строка без числа")?)? as usize;
        if l.len() != 2 + cnt {
            return Err(format!("многоязычная строка: {}", short(n)));
        }
        let mut items = Vec::new();
        for it in &l[2..cnt + 2] {
            let p = list(it)?;
            let (Some(lang), Some(text)) = (p.first().and_then(Node::str), p.get(1).and_then(Node::str)) else {
                return Err(format!("многоязычная строка: {}", short(it)));
            };
            // Пустые строки платформа не пишет.
            if !text.is_empty() {
                items.push((lang, text));
            }
        }
        if items.is_empty() {
            self.empty(tag);
            return Ok(());
        }
        self.open(tag);
        for (lang, text) in items {
            self.open("v8:item");
            if lang.is_empty() {
                self.empty("v8:lang");
            } else {
                self.text("v8:lang", lang);
            }
            self.text("v8:content", text);
            self.close("v8:item");
        }
        self.close(tag);
        Ok(())
    }
}

/// Экранирование текста: переводы строк 1С хранит как CRLF, платформа пишет LF.
fn esc(s: &str) -> String {
    let mut o = String::with_capacity(s.len());
    let s = s.replace("\r\n", "\n");
    for c in s.chars() {
        match c {
            '&' => o.push_str("&amp;"),
            '<' => o.push_str("&lt;"),
            '>' => o.push_str("&gt;"),
            c => o.push(c),
        }
    }
    o
}

fn attr(s: &str) -> String {
    esc(s).replace('"', "&quot;")
}

/// Первая строка многоязычной `{1,1,{"","Имя"}}` — имя параметра.
fn ml_first(n: &Node) -> R<&str> {
    let l = list(n)?;
    if l.first().and_then(atom) == Some("1") && l.get(1).and_then(atom) == Some("1") && l.len() == 3 {
        if let Some(s) = l[2].get(1).and_then(Node::str) {
            return Ok(s);
        }
    }
    Err(format!("параметр: {}", short(n)))
}

/// Ячейка `{флаги,формат,...}`; поля по возрастанию флагов, кроме параметра картинки (64) — он перед текстом.
struct Cell<'a> {
    fmt: usize,
    text: Option<&'a Node>,
    detail: Option<&'a str>,
    value: Option<&'a Node>,
    /// Значение расшифровки.
    dvalue: Option<&'a Node>,
    picture: Option<&'a str>,
    /// Форматированная строка `1,{1,текст,1}` после текста — в XML `tfl` вместо `tl`.
    formatted: Option<&'a Node>,
    control: Option<&'a str>,
    /// Примечание: рисунок вида «Комментарий».
    note: Option<&'a Node>,
}

fn cell(n: &Node) -> R<Cell<'_>> {
    let l = list(n)?;
    let mut c = Cur::new(l);
    let flags = c.int()?;
    let fmt = c.count()?;
    let mut r = Cell { fmt, text: None, detail: None, value: None, dvalue: None, picture: None, formatted: None, control: None, note: None };
    if flags & !(1 | 2 | 4 | 8 | 16 | 32 | 64) != 0 {
        return Err(format!("ячейка: флаги {flags}"));
    }
    // Элемент управления: `1,{#base64:...}`.
    if flags & 1 != 0 {
        c.expect("1")?;
        let b = c.next()?.get(0).and_then(atom).and_then(|a| a.strip_prefix("#base64:"));
        r.control = Some(b.ok_or_else(|| format!("ячейка: элемент управления {}", short(n)))?);
    }
    if flags & 2 != 0 {
        r.value = Some(c.next()?);
    }
    if flags & 4 != 0 {
        r.dvalue = Some(c.next()?);
    }
    if flags & 8 != 0 {
        r.detail = Some(c.next()?.str().ok_or("ячейка: параметр расшифровки не строка")?);
    }
    if flags & 64 != 0 {
        r.picture = Some(c.next()?.str().ok_or("ячейка: параметр картинки не строка")?);
    }
    if flags & 16 != 0 {
        r.text = Some(c.next()?);
    }
    // Примечание: текст, 1, рисунок (текст рисунка тот же); с пустым текстом платформа его не пишет.
    if flags & 32 != 0 {
        let t = c.next()?;
        c.expect("1")?;
        let d = c.next()?;
        if short(t) != "{1,0}" {
            r.note = Some(d);
        }
    }
    // Хвост у новых версий: `0` или форматированная строка.
    match c.rest() {
        [] => {}
        [z] if atom(z) == Some("0") => {}
        [o, f] if atom(o) == Some("1") && r.text.is_some() => match list(f)? {
            [a, t, b] if atom(a) == Some("1") && atom(b) == Some("1") => r.formatted = Some(t),
            _ => return Err(format!("ячейка: хвост {}", short(n))),
        },
        _ => return Err(format!("ячейка: хвост {}", short(n))),
    }
    Ok(r)
}

struct Row<'a> {
    idx: i64,
    fmt: usize,
    cells: Vec<(i64, Cell<'a>)>,
    cols: Option<usize>,
}

struct ColSet<'a> {
    size: i64,
    fmt: usize,
    id: &'a str,
    items: Vec<(i64, usize)>,
}

fn colset(n: &Node) -> R<ColSet<'_>> {
    let l = list(n)?;
    let mut c = Cur::new(l);
    let size = c.int()?;
    let fmt = c.count()?;
    let id = atom(c.next()?).ok_or("колонки: нет id")?;
    let k = c.count()?;
    let mut items = Vec::new();
    for _ in 0..k {
        items.push((c.int()?, c.count()?));
    }
    if !c.rest().is_empty() {
        return Err(format!("колонки: хвост {}", short(n)));
    }
    Ok(ColSet { size, fmt, id, items })
}

/// Формат: маска свойств и значения по возрастанию битов.
struct Fmt<'a> {
    vals: Vec<(u32, &'a Node)>,
}

fn fmt(n: &Node) -> R<Fmt<'_>> {
    let l = list(n)?;
    let mask: u64 = l.first().and_then(atom).and_then(|a| a.parse().ok()).ok_or_else(|| format!("формат: {}", short(n)))?;
    let mut bits: Vec<u32> = (0..64).filter(|b| mask >> b & 1 == 1).collect();
    // Поля отступов записаны в порядке левый (42), правый (44), верхний (43), нижний (45).
    bits.sort_by_key(|&b| match b {
        43 => 44,
        44 => 43,
        b => b,
    });
    if bits.len() != l.len() - 1 {
        return Err(format!("формат: значений {} при {} битах: {}", l.len() - 1, bits.len(), short(n)));
    }
    Ok(Fmt { vals: bits.into_iter().zip(&l[1..]).collect() })
}

impl Fmt<'_> {
    fn get(&self, b: u32) -> Option<&Node> {
        self.vals.iter().find(|(x, _)| *x == b).map(|(_, n)| *n)
    }
}

struct Doc<'a> {
    lang: &'a [Node],
    default: &'a Node,
    lines: Vec<&'a Node>,
    hf: Vec<&'a Node>,
    rows: Vec<Row<'a>>,
    cols: Vec<ColSet<'a>>,
    height: i64,
    /// Объединения ячеек, строк целиком, колонок (с набором колонок).
    merges: Vec<&'a [Node]>,
    vbreaks: &'a [Node],
    vg: Vec<&'a Node>,
    hg: Vec<&'a Node>,
    print: &'a Node,
    print_area: &'a Node,
    drawings: &'a [Node],
    pictures: &'a [Node],
    named: &'a [Node],
    formats: &'a [Node],
    fonts: &'a [Node],
    fstrings: &'a [Node],
    types: &'a [Node],
    controls: &'a [Node],
    masks: &'a [Node],
    colors: &'a [Node],
    step: Option<&'static str>,
    /// Режим макета (templateMode): у макетов всегда 1, у реквизита формы бывает 0 — тогда не пишется.
    template_mode: bool,
    /// Повторяемые строки (начало, конец); (0, 0) — нет.
    repeat_rows: (i64, i64),
    /// Цвета фона и текста группировок и заголовков — номера в таблице цветов.
    special: [usize; 4],
    /// Цвета стиля поверх таблицы цветов: (номер, цвет); таблица хранит их RGB.
    restyle: Vec<(usize, &'a Node)>,
    /// Источники данных диаграмм (области ячеек).
    sources: &'a [Node],
}

fn take<'a>(c: &mut Cur<'a>) -> R<&'a [Node]> {
    let n = c.count()?;
    let s = c.rest().get(..n).ok_or("список короче числа")?;
    c.i += n;
    Ok(s)
}

fn none(what: &str, l: &[Node]) -> R<()> {
    if l.is_empty() {
        Ok(())
    } else {
        Err(format!("{what}: не поддержано ({})", l.len()))
    }
}

fn decode(t: &[Node]) -> R<Doc<'_>> {
    let mut c = Cur::new(t);
    c.expect("8")?;
    c.expect("1")?;
    let ver = c.int()?;
    let lang = c.list()?;
    let default = c.next()?;
    let ll = c.list()?;
    let mut lc = Cur::new(ll);
    let mut lines = Vec::new();
    for _ in 0..lc.count()? {
        lc.expect("1")?;
        lines.push(lc.next()?);
        lc.expect("0")?;
    }
    c.expect("0")?;
    let hf: Vec<&Node> = (0..6).map(|_| c.next()).collect::<R<_>>()?;
    let template_mode = match c.int()? {
        0 => false,
        1 => true,
        x => return Err(format!("режим макета {x}")),
    };
    // Направление шага: 2 — по колонкам (по умолчанию, не пишется), 3 — без перемещения.
    let step = match c.int()? {
        2 => None,
        3 => Some("WithoutMove"),
        x => return Err(format!("направление шага {x}")),
    };
    let mut rows = Vec::new();
    for _ in 0..c.count()? {
        let idx = c.int()?;
        let fmt = c.count()?;
        let mut cells = Vec::new();
        for _ in 0..c.count()? {
            let col = c.int()?;
            cells.push((col, cell(c.next()?)?));
        }
        rows.push(Row { idx, fmt, cells, cols: None });
    }
    let mut cols = vec![colset(c.next()?)?];
    let height = c.int()?;
    for n in take(&mut c)? {
        cols.push(colset(n)?);
    }
    let rowsets = c.count()?;
    for _ in 0..rowsets {
        let r = c.int()?;
        let s = c.count()?;
        match rows.iter_mut().find(|x| x.idx == r) {
            Some(row) => row.cols = Some(s + 1),
            None => rows.push(Row { idx: r, fmt: 0, cells: vec![], cols: Some(s + 1) }),
        }
    }
    rows.sort_by_key(|r| r.idx);
    let _drawnext = c.int()?;
    let drawings = take(&mut c)?;
    // Группировки строк: пары ({начало,конец,уровень,{заголовок},свернута,0}, родитель).
    let mut vg = Vec::new();
    for _ in 0..c.count()? {
        let g = c.next()?;
        c.expect("-1")?;
        vg.push(g);
    }
    // Группировки колонок — как строк (moxel FUN_1803ffb40: `hg` сразу после всех `vg`)
    let mut hg = Vec::new();
    for _ in 0..c.count()? {
        let g = c.next()?;
        c.expect("-1")?;
        hg.push(g);
    }
    let vbreaks = take(&mut c)?;
    none("горизонтальные разрывы", take(&mut c)?)?;
    let mut merges = Vec::new();
    for _ in 0..3 {
        let ml = c.list()?;
        if ml.first().map(int).transpose()? != Some(ml.len() as i64 - 1) {
            return Err("объединения: число".into());
        }
        merges.push(&ml[1..]);
    }
    let nl = c.list()?;
    let named = &nl[1..];
    c.next()?.str().filter(|s| s.is_empty()).ok_or("ждали пустую строку")?;
    let print = c.next()?;
    let print_area = c.next()?;
    let repeat_rows = (c.int()?, c.int()?);
    // Повторяемые колонки: начало, конец, флаг[, id набора колонок].
    let rc = (c.int()?, c.int()?, c.int()?);
    let rc_id = if rc.2 == 1 { atom(c.next()?) } else { Some(ZERO) };
    if (rc.0, rc.1) != (0, 0) || !(0..=1).contains(&rc.2) || rc_id != Some(ZERO) {
        return Err(format!("повторяемые колонки: не поддержано {rc:?}"));
    }
    // Источники данных диаграмм: пары (id рисунка, `{3,к1,с1,к2,с2,нулевой GUID}` — область ячеек;
    // ДиагностикаОтчетности_Подсказки).
    let n = c.count()?;
    let sources = c.rest().get(..2 * n).ok_or("источники данных рисунков: список короче числа")?;
    c.i += 2 * n;
    for s in sources.chunks(2) {
        let id = int(&s[0])?;
        let chart = |d: &Node| d.get(1).and_then(atom) == Some("10") && d.get(10).and_then(|x| int(x).ok()) == Some(id) && d.get(11).and_then(atom) == Some(CHART);
        if !drawings.iter().any(chart) {
            return Err(format!("источник данных рисунка {id}: не диаграмма"));
        }
    }
    let special = [c.count()?, c.count()?, c.count()?, c.count()?];
    let formats = take(&mut c)?;
    let fonts = take(&mut c)?;
    let fstrings = take(&mut c)?;
    let types = take(&mut c)?;
    let controls = take(&mut c)?;
    let colors = take(&mut c)?;
    let pictures = take(&mut c)?;
    // Хвост: 0,0,"",0,{...},{0},маски,...
    c.expect("0")?;
    c.expect("0")?;
    let mut restyle = Vec::new();
    let mut masks: &[Node] = &[];
    // Записи версий 8.1.0 и 8.1.1 на этом кончаются.
    if ver >= 5 || !c.rest().is_empty() {
        c.next()?.str().filter(|s| s.is_empty()).ok_or("хвост: ждали пустую строку")?;
        c.expect("0")?;
        c.list()?;
        let rl = c.list()?;
        if rl.first().map(int).transpose()? != Some((rl.len() as i64 - 1) / 2) || rl.len() % 2 == 0 {
            return Err(format!("цвета стиля: {}", short(&Node::List(rl.to_vec()))));
        }
        for p in rl[1..].chunks(2) {
            restyle.push((usize::try_from(int(&p[0])?).map_err(|_| "цвета стиля: номер")?, &p[1]));
        }
        if ver >= 8 {
            masks = take(&mut c)?;
        }
    }
    for x in c.rest() {
        if !matches!(atom(x), Some("0" | "1")) {
            return Err(format!("хвост: {}", short(x)));
        }
    }
    Ok(Doc { lang, default, lines, hf, rows, cols, height, merges, vbreaks, vg, hg, print, print_area, drawings, pictures, named, formats, fonts, fstrings, types, controls, masks, colors, step, template_mode, repeat_rows, special, restyle, sources })
}

/// Табличный документ → XML. `names` — имя объекта конфигурации по классу и идентификатору (стиль, картинка).
pub fn template(raw: &[u8], names: &dyn Fn(&str, &str) -> Option<String>) -> R<Vec<u8>> {
    let rec = raw.get(13..).ok_or("короткая запись")?;
    let text = std::str::from_utf8(rec).map_err(|e| e.to_string())?;
    // Без потерь: переводы строк в конце base64 картинок значимы.
    let doc = brace::Doc::parse(text)?;
    let root = doc.roots.first().ok_or("пустая запись")?;
    Ok(format!("{HEAD}{}</document>", body(root, names)?).into_bytes())
}

/// Содержимое `<document>` (элементы с отступа 1) по корню `{8,1,версия,...}`: макет и реквизит формы.
pub fn body(root: &Node, names: &dyn Fn(&str, &str) -> Option<String>) -> R<String> {
    let d = decode(root.list())?;
    let mut w = Writer { names, d: &d, x: X { o: String::new(), d: 1 }, fmts: Order::new(), lines: Order::new(), fonts: Order::new(), dfmts: Vec::new(), bad: None };
    w.doc()?;
    Ok(w.x.o)
}

struct Writer<'a> {
    names: &'a dyn Fn(&str, &str) -> Option<String>,
    d: &'a Doc<'a>,
    x: X,
    /// Форматы по первому использованию: номер в списке с 1, 0 — формат по умолчанию.
    fmts: Order<usize>,
    lines: Order<usize>,
    fonts: Order<usize>,
    /// Форматы рисунков: у них граница — drawingBorder.
    dfmts: Vec<usize>,
    bad: Option<String>,
}

impl<'a> Writer<'a> {
    fn f(&mut self, i: usize) -> usize {
        let d = self.d;
        match d.formats.get(i - 1) {
            Some(_) => self.fmts.id(i) + 1,
            None => {
                self.bad = Some(format!("нет формата {i}"));
                0
            }
        }
    }

    /// Номер формата; 0 — без формата.
    fn fi(&mut self, i: usize) -> usize {
        if i == 0 {
            0
        } else {
            self.f(i)
        }
    }

    fn doc(&mut self) -> R<()> {
        self.lang()?;
        let d = self.d;
        // Порядок нумерации форматов: колонки, строки и ячейки, формат по умолчанию, колонтитулы.
        for s in &d.cols {
            self.columns(s)?;
        }
        // Пустые строки подряд (и отсутствующие) с одинаковым форматом и колонками — один rowsItem с indexTo.
        let max = d.rows.last().map_or(0, |r| r.idx);
        // первая строка с номером, как find, но без перебора всех строк на каждый номер
        let mut by_idx = std::collections::HashMap::new();
        for r in &d.rows {
            by_idx.entry(r.idx).or_insert(r);
        }
        let blank = |i: i64| match by_idx.get(&i) {
            None => Some((0, None)),
            Some(r) if r.cells.is_empty() => Some((r.fmt, r.cols)),
            Some(_) => None,
        };
        let mut i = 0;
        while i <= max {
            let Some(key) = blank(i) else {
                self.row(by_idx[&i])?;
                i += 1;
                continue;
            };
            let mut j = i;
            while j < max && blank(j + 1) == Some(key) {
                j += 1;
            }
            self.x.open("rowsItem");
            self.x.leaf("index", &i.to_string());
            if j > i {
                self.x.leaf("indexTo", &j.to_string());
            }
            self.x.open("row");
            self.row_head(key.0, key.1)?;
            self.x.leaf("empty", "true");
            self.x.close("row");
            self.x.close("rowsItem");
            i = j + 1;
        }
        for (k, n) in d.drawings.iter().enumerate() {
            self.drawing(n, "drawing", Some(k + 1))?;
        }
        // Формат по умолчанию платформа ищет по содержимому среди уже записанных (одинаковые форматы списка не сливает),
        // из совпавших берет последний по списку. Не нашелся: пустой не пишется, непустой пишется отдельно.
        let eq = (0..self.fmts.keys.len()).filter(|&p| d.formats.get(self.fmts.keys[p].wrapping_sub(1)) == Some(d.default));
        let def = match eq.max_by_key(|&p| self.fmts.keys[p]) {
            Some(p) => Some(p + 1),
            None if fmt(d.default)?.vals.is_empty() => None,
            None => Some(self.fmts.id(0) + 1),
        };
        const HF: [&str; 6] = ["leftHeader", "leftFooter", "centerHeader", "centerFooter", "rightHeader", "rightFooter"];
        let mut hf = Vec::new();
        for (k, n) in d.hf.iter().enumerate() {
            let c = cell(n)?;
            if c.detail.is_some() || c.value.is_some() || c.dvalue.is_some() || c.picture.is_some() {
                return Err("колонтитул: не поддержано".into());
            }
            if c.fmt != 0 || c.text.is_some() {
                hf.push((k, self.fi(c.fmt), c.formatted.map(|t| ("tfl", t)).or(c.text.map(|t| ("tl", t)))));
            }
        }
        for name in ["leftHeader", "centerHeader", "rightHeader", "leftFooter", "centerFooter", "rightFooter"] {
            if let Some((_, f, t)) = hf.iter().find(|(k, _, _)| HF[*k] == name) {
                self.x.open(name);
                self.x.leaf("f", &f.to_string());
                if let Some((tag, t)) = t {
                    self.x.ml(tag, t)?;
                }
                self.x.close(name);
            }
        }
        if d.template_mode {
            self.x.leaf("templateMode", "true");
        }
        if let Some(s) = d.step {
            self.x.leaf("stepDirection", s);
        }
        if let Some(def) = def {
            self.x.leaf("defaultFormatIndex", &def.to_string());
        }
        if d.height != 0 {
            self.x.leaf("height", &d.height.to_string());
        }
        // Группировка `{начало,конец,уровень,{заголовок},свернута,вид начала}`; вид начала 1 — `<g>Begin</g>`, 0 (Auto) не пишется
        // (moxel FUN_1803ffb40); колонки — тем же видом `hg`, без hgLevels.
        fn groups_of<'n>(src: &[&'n Node]) -> R<Vec<(Vec<i64>, &'n Node)>> {
            let mut groups: Vec<(Vec<i64>, &Node)> = Vec::new();
            for g in src {
                let l = list(g)?;
                let bad = || format!("группировка: {}", short(g));
                let v: Vec<i64> = [0, 1, 2, 4, 5].iter().map(|&k| l.get(k).ok_or_else(bad).and_then(int)).collect::<R<_>>()?;
                if l.len() != 6 || !(0..=1).contains(&v[4]) || !(0..=1).contains(&v[3]) {
                    return Err(bad());
                }
                // Повтор той же группировки подряд платформа не пишет.
                if groups.last().is_some_and(|(p, _)| p[..3] == v[..3]) {
                    continue;
                }
                groups.push((v, &l[3]));
            }
            Ok(groups)
        }
        let vgroups = groups_of(&d.vg)?;
        if let Some(max) = vgroups.iter().map(|(v, _)| v[2]).max() {
            self.x.leaf("vgLevels", &(max + 1).to_string());
        }
        self.x.leaf("vgRows", &d.height.to_string());
        let hgroups = groups_of(&d.hg)?;
        for (tag, groups) in [("vg", vgroups), ("hg", hgroups)] {
            for (v, t) in groups {
                self.x.open(tag);
                self.x.leaf("b", &v[0].to_string());
                if v[1] != v[0] {
                    self.x.leaf("e", &v[1].to_string());
                }
                if list(t)?.len() > 2 {
                    self.x.ml("t", t)?;
                }
                if v[3] == 1 {
                    self.x.leaf("o", "false");
                }
                if v[4] == 1 {
                    self.x.leaf("g", "Begin");
                }
                self.x.close(tag);
            }
        }
        for b in d.vbreaks {
            self.x.open("verticalPageBreak");
            self.x.leaf("position", &int(b)?.to_string());
            self.x.close("verticalPageBreak");
        }
        // Вид в пятом поле: 0 — объединение, 2 — отмена по вертикали, 1 — по горизонтали.
        let mut all = Vec::new();
        // Порядок в XML: ячейки, колонки (в наборах), строки целиком.
        for k in [0, 2, 1] {
            let l = &d.merges[k];
            let mut part = l.iter().map(|m| merge(m, k == 2)).collect::<R<Vec<_>>>()?;
            // Объединения ячеек платформа упорядочивает по левой верхней ячейке и из начинающихся в одной оставляет наибольшее.
            if k == 0 {
                part.sort_by_key(|m| (m.r, m.c, m.kind));
                let mut kept: Vec<Merge> = Vec::new();
                for m in part {
                    match kept.last_mut() {
                        Some(p) if (p.kind, p.r, p.c) == (m.kind, m.r, m.c) => {
                            if !(m.h >= p.h && m.w >= p.w || m.h <= p.h && m.w <= p.w) {
                                return Err(format!("объединения из ячейки {},{} не вложены", m.r, m.c));
                            }
                            if m.h >= p.h && m.w >= p.w {
                                *p = m;
                            }
                        }
                        _ => kept.push(m),
                    }
                }
                part = kept;
            }
            all.extend(part);
        }
        for (kind, tag) in [(0, "merge"), (2, "verticalUnmerge"), (1, "horizontalUnmerge")] {
            for m in all.iter().filter(|m| m.kind == kind) {
                self.x.open(tag);
                self.x.leaf("r", &m.r.to_string());
                self.x.leaf("c", &m.c.to_string());
                if m.h != 0 {
                    self.x.leaf("h", &m.h.to_string());
                }
                if m.w != 0 {
                    self.x.leaf("w", &m.w.to_string());
                }
                if m.id != ZERO {
                    self.x.leaf("columnsID", m.id);
                }
                self.x.close(tag);
            }
        }
        for k in (0..d.named.len()).step_by(2) {
            self.named(&d.named[k], d.named.get(k + 1).ok_or("именованная область без области")?)?;
        }
        self.print_settings()?;
        if d.print_area.get(0).and_then(atom) != Some("0") {
            self.area("printArea", d.print_area)?;
        } else if short(d.print_area) != format!("{{0,-1,-1,-1,-1,{ZERO}}}") {
            return Err(format!("область печати: {}", short(d.print_area)));
        }
        if d.repeat_rows != (0, 0) {
            let (b, e) = d.repeat_rows;
            self.x.open("repeatRows");
            self.x.leaf("type", "Rows");
            self.x.leaf("beginRow", &b.to_string());
            self.x.leaf("endRow", &e.to_string());
            self.x.leaf("beginColumn", "0");
            self.x.leaf("endColumn", "0");
            self.x.close("repeatRows");
        }
        // Пишутся, только если отличаются от цветов формы по умолчанию.
        for (k, tag) in ["groupsBackColor", "groupsColor", "headersBackColor", "headersColor"].iter().enumerate() {
            let (ns, c) = self.color_at(d.special[k])?;
            if c != if k % 2 == 0 { "style:FormBackColor" } else { "style:FormTextColor" } {
                self.x.leaf(&format!("{tag}{ns}"), &c);
            }
        }
        // Форматы пишутся после линий и шрифтов, но номера линий и шрифтов берутся при обходе форматов.
        let mut body = std::mem::replace(&mut self.x.o, String::new());
        let mut k = 0;
        while k < self.fmts.keys.len() {
            let key = self.fmts.keys[k];
            let n = match key {
                0 => d.default,
                i => &d.formats[i - 1],
            };
            self.format(n, self.dfmts.contains(&key))?;
            k += 1;
        }
        let formats = std::mem::replace(&mut self.x.o, String::new());
        for k in 0..self.lines.keys.len() {
            let i = self.lines.keys[k];
            self.line(i)?;
        }
        for k in 0..self.fonts.keys.len() {
            let i = self.fonts.keys[k];
            self.font(i)?;
        }
        body.push_str(&self.x.o);
        body.push_str(&formats);
        self.x.o = String::new();
        for (k, p) in d.pictures.iter().enumerate() {
            self.picture(k, p)?;
        }
        body.push_str(&self.x.o);
        self.x.o = body;
        match self.bad.take() {
            Some(e) => Err(e),
            None => Ok(()),
        }
    }

    fn lang(&mut self) -> R<()> {
        let l = self.d.lang;
        let s = |i: usize| l.get(i).and_then(Node::str).ok_or_else(|| "языки: ждали строку".to_string());
        let n = l.get(3).map(int).transpose()?.ok_or("языки")? as usize;
        // третье поле: 1 у записей старых платформ, 0 — как пишет загрузка 8.5 (imp/mxl.rs); в XML не попадает
        if !matches!(l.get(2).and_then(atom), Some("0" | "1")) || !(l.len() == 4 + 3 * n || l.len() == 5 + 3 * n && matches!(l[4 + 3 * n].atom(), Some("0" | "1"))) {
            return Err(format!("языки: {}", short(&Node::List(l.to_vec()))));
        }
        // Единственный язык «по умолчанию» `{"#","",1,1,"#","Язык по умолчанию",...}` (реквизит формы) платформа не пишет.
        if n == 1 && (s(0)?, s(1)?, s(4)?) == ("#", "", "#") {
            return Ok(());
        }
        self.x.open("languageSettings");
        for (k, tag) in ["currentLanguage", "defaultLanguage"].iter().enumerate() {
            match s(k)? {
                "" => self.x.empty(tag),
                v => self.x.text(tag, v),
            }
        }
        for k in 0..n {
            self.x.open("languageInfo");
            // пустая строка — пустой элемент, как у currentLanguage (Клиент ЭДО: description английского языка)
            for (i, tag) in ["id", "code", "description"].iter().enumerate() {
                match s(4 + 3 * k + i)? {
                    "" => self.x.empty(tag),
                    v => self.x.text(tag, v),
                }
            }
            self.x.close("languageInfo");
        }
        self.x.close("languageSettings");
        Ok(())
    }

    fn columns(&mut self, s: &ColSet) -> R<()> {
        self.x.open("columns");
        if s.id != ZERO {
            self.x.leaf("id", s.id);
        }
        if s.fmt != 0 {
            let f = self.f(s.fmt);
            self.x.leaf("formatIndex", &f.to_string());
        }
        self.x.leaf("size", &s.size.to_string());
        for &(i, f) in &s.items {
            self.x.open("columnsItem");
            self.x.leaf("index", &i.to_string());
            self.x.open("column");
            let f = self.fi(f);
            self.x.leaf("formatIndex", &f.to_string());
            self.x.close("column");
            self.x.close("columnsItem");
        }
        self.x.close("columns");
        Ok(())
    }

    fn row(&mut self, r: &Row) -> R<()> {
        self.x.open("rowsItem");
        self.x.leaf("index", &r.idx.to_string());
        self.x.open("row");
        self.row_head(r.fmt, r.cols)?;
        let mut next = 0;
        for (col, c) in &r.cells {
            self.x.open("c");
            if *col != next {
                self.x.leaf("i", &col.to_string());
            }
            next = col + 1;
            self.x.open("c");
            let f = self.fi(c.fmt);
            self.x.leaf("f", &f.to_string());
            let fill = match self.d.formats.get(c.fmt.wrapping_sub(1)) {
                Some(n) => fmt(n)?.get(15).map(int).transpose()?.unwrap_or(0),
                None => 0,
            };
            if let Some(t) = c.text {
                match c.formatted {
                    Some(f) => self.x.ml("tfl", f)?,
                    None if fill == 1 => self.x.text("parameter", ml_first(t)?),
                    None => self.x.ml("tl", t)?,
                }
            }
            if let Some(v) = c.value {
                self.value(v)?;
            }
            if let Some(dp) = c.detail {
                self.x.text("detailParameter", dp);
            }
            if let Some(v) = c.dvalue {
                match (v.get(0).and_then(Node::str), v.get(1).and_then(atom)) {
                    // Расшифровка СКД `{"#",id,{номер}}` — номер элемента расшифровки.
                    (Some("#"), Some(DCS_DETAIL)) => match v.get(2).map(Node::list) {
                        Some([n]) => self.x.leaf("r", &int(n)?.to_string()),
                        _ => return Err(format!("значение расшифровки: {}", short(v))),
                    },
                    _ => self.xvalue("d", v).map_err(|e| format!("значение расшифровки: {e}"))?,
                }
            }
            if let Some(nt) = c.note {
                self.drawing(nt, "note", None)?;
            }
            if let Some(b) = c.control {
                self.x.leaf("control xsi:type=\"xs:base64Binary\"", &base64_text(b));
            }
            if let Some(p) = c.picture {
                self.x.text("pictureParameter", p);
            }
            self.x.close("c");
            self.x.close("c");
        }
        self.x.close("row");
        self.x.close("rowsItem");
        Ok(())
    }

    /// Рисунок `{{ячейка},вид,к1,с1,смещ.к1,смещ.с1,к2,с2,смещ.к2,смещ.с2,id,...}`; `z` — порядок (у примечаний нет).
    fn drawing(&mut self, n: &Node, tag: &str, z: Option<usize>) -> R<()> {
        let l = list(n)?;
        let bad = || format!("рисунок: {}", short(n));
        let c = cell(l.first().ok_or_else(bad)?)?;
        if c.picture.is_some() || c.formatted.is_some() || c.control.is_some() || c.note.is_some() {
            return Err(bad());
        }
        let v: Vec<i64> = l.get(1..10).ok_or_else(bad)?.iter().map(int).collect::<R<_>>()?;
        // После id: у картинки номер картинки и размер, у всех — автоматический размер.
        let rest = l.get(11..).ok_or_else(bad)?;
        let (ty, pic) = match (v[0], rest.len()) {
            (1, 1) => ("Line", None),
            (2, 1) => ("Rectangle", None),
            (3, 1) => ("Text", None),
            (5, 3) => ("Picture", Some(&rest[0])),
            // Диаграмма: `a8b97779-…,{{11},{диаграмма}},авторазмер`.
            (10, 3) if atom(&rest[0]) == Some(CHART) => ("Chart", None),
            // Диаграмма Ганта: `e5fdc112-…,{{версия,{0,{11},{диаграмма}},...}},авторазмер`.
            (10, 3) if atom(&rest[0]) == Some(GANTT) => ("GanttChart", None),
            (6, 1) if z.is_none() => ("Comment", None),
            _ => return Err(bad()),
        };
        let auto = match atom(&rest[rest.len() - 1]) {
            Some("0") => "false",
            Some("1") => "true",
            _ => return Err(bad()),
        };
        let fi = self.fi(c.fmt);
        if c.fmt != 0 && !self.dfmts.contains(&c.fmt) {
            self.dfmts.push(c.fmt);
        }
        self.x.open(tag);
        self.x.leaf("drawingType", ty);
        self.x.leaf("id", &int(&l[10])?.to_string());
        self.x.leaf("formatIndex", &fi.to_string());
        if let Some(t) = c.text {
            let fill = match self.d.formats.get(c.fmt.wrapping_sub(1)) {
                Some(n) => fmt(n)?.get(15).map(int).transpose()?.unwrap_or(0),
                None => 0,
            };
            if fill == 1 {
                self.x.text("parameter", ml_first(t)?);
            } else {
                self.x.ml("text", t)?;
            }
        }
        if let Some(val) = c.value {
            self.value_as("value", val)?;
        }
        if let Some(dp) = c.detail {
            self.x.text("detailParameter", dp);
        }
        if let Some(dv) = c.dvalue {
            self.xvalue("detailValue", dv)?;
        }
        for (tag, k) in [("beginRow", 2), ("beginRowOffset", 4), ("endRow", 6), ("endRowOffset", 8), ("beginColumn", 1), ("beginColumnOffset", 3), ("endColumn", 5), ("endColumnOffset", 7)] {
            self.x.leaf(tag, &v[k].to_string());
        }
        self.x.leaf("autoSize", auto);
        let size = match rest.get(1).filter(|_| pic.is_some()).map(int).transpose()? {
            None | Some(1) => "Stretch",
            Some(0) => "RealSize",
            Some(2) => "Proportionally",
            Some(3) => "Tile",
            Some(4) => "AutoSize",
            Some(x) => return Err(format!("размер картинки {x}")),
        };
        self.x.leaf("pictureSize", size);
        if let Some(z) = z {
            self.x.leaf("zOrder", &z.to_string());
        }
        if let Some(p) = pic {
            self.x.leaf("pictureIndex", &int(p)?.to_string());
        }
        if ty == "Chart" || ty == "GanttChart" {
            // диаграмма версии 75 (8.5): цвета и шрифты в прежние версии, как у формы (`form_v59`)
            let mut v85 = rest[1].clone();
            // у диаграммы Ганта `{{версия,{0,{11},диаграмма},…}}` — версия вложенной диаграммы
            let chart = if ty == "Chart" { v85.get(1) } else { v85.get(0).and_then(|g| g.get(1)).and_then(|c| c.get(2)) };
            if chart.and_then(|c| c.get(0)).and_then(Node::atom) == Some("75") {
                crate::xml::form_v59::styles(&mut v85)?;
            }
            let o = list(&v85)?;
            if ty == "Chart" && (o.len() != 2 || short(&o[0]) != "{11}") {
                return Err(bad());
            }
            let p = format!("d{}p1", self.x.d + 1);
            self.x.open(&format!("object xmlns:{p}=\"http://v8.1c.ru/8.2/data/chart\" xsi:type=\"{p}:{ty}\""));
            let lines = if ty == "Chart" {
                // Текст ячейки (строка, колонка с 0) для диаграммы с источником данных: одноязычный текст; пусто — None.
                let rows = &self.d.rows;
                let cell = |r: usize, k: usize| -> R<Option<String>> {
                    let row = rows.iter().find(|x| x.idx == r as i64);
                    let Some(c) = row.and_then(|x| x.cells.iter().find(|c| c.0 == k as i64)).map(|c| &c.1) else { return Ok(None) };
                    if c.value.is_some() || c.formatted.is_some() || c.detail.is_some() || c.dvalue.is_some() {
                        return Err(format!("источник данных: ячейка R{}C{} не текст", r + 1, k + 1));
                    }
                    c.text.map(|t| ml_first(t).map(str::to_string)).transpose()
                };
                let id = int(&l[10])?;
                let mine = self.d.sources.chunks(2).position(|s| int(&s[0]).ok() == Some(id)).map_or(&[][..], |k| &self.d.sources[2 * k + 1..2 * k + 2]);
                super::chart::body_with(list(&o[1])?, &p, mine, &cell)?
            } else {
                super::chart::gantt(o, &p)?
            };
            for l in lines {
                self.x.pad();
                self.x.o.push_str(&l);
                self.x.o.push_str("\r\n");
            }
            self.x.close("object");
        }
        self.x.close(tag);
        Ok(())
    }

    /// Картинка `{версия,вид,ссылка,"",tx,ty,прозрачность,{{#base64:...}},0[,""]}`.
    fn picture(&mut self, k: usize, n: &Node) -> R<()> {
        let l = list(n)?;
        let bad = || format!("картинка: {}", short(n));
        let a = |i: usize| l.get(i).and_then(atom).ok_or_else(bad);
        // Данные (только у вида 3) — восьмое поле, за ними 0; у версии 4 в конце еще "".
        let len = if a(1)? == "3" { 9 } else { 8 };
        let tail_ok = match a(0)? {
            "3" => l.len() == len,
            "4" => l.len() == len + 1 && l[len].str() == Some(""),
            _ => false,
        } && (len == 8 || a(8)? == "0");
        if !tail_ok || l[3].str() != Some("") {
            return Err(bad());
        }
        let (tx, ty) = (a(4)?, a(5)?);
        let mut attrs = String::new();
        if tx != "-1" || ty != "-1" {
            attrs.push_str(&format!(" tx=\"{tx}\" ty=\"{ty}\""));
        }
        match a(6)? {
            "0" => attrs.push_str(" t=\"false\""),
            "1" => {}
            _ => return Err(bad()),
        }
        self.x.open("picture");
        self.x.leaf("index", &k.to_string());
        let r = list(&l[2])?;
        match (a(1)?, r) {
            ("3", [z]) if atom(z) == Some("0") => {
                let b = l[7].at(&[0, 0]).and_then(atom).and_then(|x| x.strip_prefix("#base64:")).ok_or_else(bad)?;
                self.x.leaf(&format!("picture{attrs}"), &base64_text(b));
            }
            ("0", [z]) if atom(z) == Some("0") && a(7)? == "0" => self.x.empty(&format!("picture{attrs}")),
            ("1", r) if a(7)? == "0" => {
                let name = match r {
                    // Общая картинка конфигурации или стандартная картинка платформы.
                    [z, g] if atom(z) == Some("0") => {
                        let g = atom(g).unwrap_or("");
                        (self.names)("CommonPicture", g)
                            .or_else(|| STD_PICTURES.iter().find(|(id, _)| *id == g).map(|(_, n)| *n).or_else(|| crate::xml::imp::form::platform::std_picture(g)).map(str::to_string))
                            .ok_or_else(bad)?
                    }
                    [x] => match atom(x) {
                        Some("-6") => "InputFieldCalculator".into(),
                        Some("-13") => "Print".into(),
                        _ => return Err(bad()),
                    },
                    _ => return Err(bad()),
                };
                self.x.empty(&format!("picture{attrs} ref=\"v8ui:{}\"", attr(&name)));
            }
            _ => return Err(bad()),
        }
        self.x.close("picture");
        Ok(())
    }

    fn row_head(&mut self, fmt: usize, cols: Option<usize>) -> R<()> {
        if let Some(cols) = cols {
            let id = self.d.cols.get(cols).ok_or("нет набора колонок")?.id;
            self.x.leaf("columnsID", id);
        }
        if fmt != 0 {
            let f = self.f(fmt);
            self.x.leaf("formatIndex", &f.to_string());
        }
        Ok(())
    }

    fn value(&mut self, v: &Node) -> R<()> {
        self.value_as("v", v)
    }

    /// Произвольное значение: Неопределено, примитив, Структура, Соответствие, СписокЗначений, Массив.
    fn xvalue(&mut self, tag: &str, v: &Node) -> R<()> {
        let bad = || short(v);
        // Пары `{n,{ключ,значение}*n}`.
        fn pairs(b: &Node) -> Option<Vec<(&Node, &Node)>> {
            let l = b.list();
            if atom(l.first()?)?.parse::<usize>().ok()? != l.len() - 1 {
                return None;
            }
            l[1..].iter().map(|p| match p.list() {
                [k, v] => Some((k, v)),
                _ => None,
            }).collect()
        }
        match list(v)? {
            [u] if u.str() == Some("U") => self.x.empty(&format!("{tag} xsi:nil=\"true\"")),
            [h, id, b] if h.str() == Some("#") => match atom(id) {
                Some(STRUCTURE) => {
                    self.x.open(&format!("{tag} xsi:type=\"v8:Structure\""));
                    for (k, val) in pairs(b).ok_or_else(bad)? {
                        let k = match k.list() {
                            [s, n] if s.str() == Some("S") => n.str().ok_or_else(bad)?,
                            _ => return Err(bad()),
                        };
                        self.x.open(&format!("v8:Property name=\"{}\"", attr(k)));
                        self.xvalue("v8:Value", val)?;
                        self.x.close("v8:Property");
                    }
                    self.x.close(tag);
                }
                // `{n,значение*n}`: core85 0x1801ce5e0 (запись Массива) пишет число элементов и каждый GenericValue.
                Some(ARRAY) => {
                    let l = b.list();
                    if l.first().and_then(atom).and_then(|n| n.parse::<usize>().ok()) != Some(l.len().wrapping_sub(1)) {
                        return Err(bad());
                    }
                    // shortcut: пустой Массив в корпусах не встречен, форма — как у пустого Соответствия; пересмотреть
                    // на первом образце
                    if l.len() == 1 {
                        self.x.empty(&format!("{tag} xsi:type=\"v8:Array\""));
                        return Ok(());
                    }
                    self.x.open(&format!("{tag} xsi:type=\"v8:Array\""));
                    for it in &l[1..] {
                        self.xvalue("v8:Value", it)?;
                    }
                    self.x.close(tag);
                }
                // Пары — в порядке обхода хеш-таблицы соответствия (map_order); ключи не строки — только до 4 пар
                // в обратном порядке (без столкновений корзин порядок тот же), больше — отказ.
                Some(MAP) => {
                    let p = pairs(b).ok_or_else(bad)?;
                    let keys: Option<Vec<&str>> = p.iter().map(|(k, _)| match k.list() {
                        [s, n] if s.str() == Some("S") => n.str(),
                        _ => None,
                    }).collect();
                    let order: Vec<usize> = match keys {
                        Some(keys) => map_order(&keys),
                        None if p.len() <= 4 => (0..p.len()).rev().collect(),
                        None => return Err(format!("соответствие из {} пар с нестроковыми ключами: порядок пар не восстановлен", p.len())),
                    };
                    if p.is_empty() {
                        self.x.empty(&format!("{tag} xsi:type=\"v8:Map\""));
                        return Ok(());
                    }
                    self.x.open(&format!("{tag} xsi:type=\"v8:Map\""));
                    for (k, val) in order.into_iter().map(|i| p[i]) {
                        self.x.open("v8:pair");
                        self.xvalue("v8:Key", k)?;
                        self.xvalue("v8:Value", val)?;
                        self.x.close("v8:pair");
                    }
                    self.x.close(tag);
                }
                // `{6,id элемента,0,0,{n,{id элемента,{представление,0,значение,картинка,id,0,""}}*n},{"Pattern"},0,последний id}`.
                Some(VALUE_LIST) => {
                    let l = b.list();
                    let z = |k: usize| l.get(k).and_then(atom) == Some(if k == 0 { "6" } else { "0" });
                    if l.len() != 8 || ![0, 2, 3, 6].into_iter().all(z) || short(&l[5]) != "{\"Pattern\"}" {
                        return Err(bad());
                    }
                    let items = l[4].list();
                    if items.first().and_then(atom).and_then(|n| n.parse::<usize>().ok()) != Some(items.len() - 1) || items.len() < 2 {
                        return Err(bad());
                    }
                    self.x.open(&format!("{tag} xsi:type=\"v8:ValueListType\""));
                    self.x.empty("v8:valueType");
                    self.x.leaf("v8:lastId xsi:type=\"xs:decimal\"", &int(&l[7])?.to_string());
                    for it in &items[1..] {
                        let f = it.get(1).map(Node::list).ok_or_else(bad)?;
                        let ok = f.len() == 7
                            && atom(&f[1]) == Some("0")
                            && short(&f[3]) == "{4,0,{0},\"\",-1,-1,0,0,\"\"}"
                            && atom(&f[5]) == Some("0")
                            && f[6].str() == Some("");
                        let pres = f.first().and_then(Node::str).filter(|p| ok && !p.is_empty()).ok_or_else(bad)?;
                        self.x.open("v8:item");
                        self.xvalue("v8:value", &f[2])?;
                        self.x.text("v8:presentation", pres);
                        self.x.leaf("v8:id xsi:type=\"xs:decimal\"", &int(&f[4])?.to_string());
                        self.x.close("v8:item");
                    }
                    self.x.close(tag);
                }
                _ => return Err(bad()),
            },
            _ => self.value_as(tag, v)?,
        }
        Ok(())
    }

    fn value_as(&mut self, tag: &str, v: &Node) -> R<()> {
        let l = list(v)?;
        match (l.first().and_then(Node::str), l.get(1)) {
            (Some("S"), Some(Node::Str(s))) if l.len() == 2 => {
                if s.is_empty() {
                    self.x.empty(&format!("{tag} xsi:type=\"xs:string\""));
                } else {
                    self.x.leaf(&format!("{tag} xsi:type=\"xs:string\""), &esc(s));
                }
            }
            (Some("N"), Some(Node::Atom(a))) if l.len() == 2 => self.x.leaf(&format!("{tag} xsi:type=\"xs:decimal\""), a),
            (Some("B"), Some(Node::Atom(a))) if l.len() == 2 => self.x.leaf(&format!("{tag} xsi:type=\"xs:boolean\""), if a == "0" { "false" } else { "true" }),
            (Some("D"), Some(Node::Atom(a))) if l.len() == 2 && a.len() == 14 => {
                let t = format!("{}-{}-{}T{}:{}:{}", &a[..4], &a[4..6], &a[6..8], &a[8..10], &a[10..12], &a[12..]);
                self.x.leaf(&format!("{tag} xsi:type=\"xs:dateTime\""), &t)
            }
            _ => return Err(format!("значение ячейки: {}", short(v))),
        }
        Ok(())
    }

    fn named(&mut self, name: &Node, a: &Node) -> R<()> {
        let name = name.str().ok_or("имя области не строка")?;
        let l = list(a)?;
        match l {
            // Область ячеек `{1,{область},0}` или рисунок `{2,id,0}`.
            [k, ar, z] if atom(k) == Some("1") && atom(z) == Some("0") => {
                self.x.open("namedItem xsi:type=\"NamedItemCells\"");
                self.x.text("name", name);
                self.area("area", ar)?;
            }
            [k, id, z] if atom(k) == Some("2") && atom(z) == Some("0") => {
                self.x.open("namedItem xsi:type=\"NamedItemDrawing\"");
                self.x.text("name", name);
                self.x.leaf("drawingID", &int(id)?.to_string());
            }
            _ => return Err(format!("именованная область: {}", short(a))),
        }
        self.x.close("namedItem");
        Ok(())
    }

    /// Область `{тип,к1,с1,к2,с2,id колонок}`.
    fn area(&mut self, tag: &str, ar: &Node) -> R<()> {
        let ar = list(ar)?;
        if ar.len() != 6 {
            return Err(format!("область: {}", short(&Node::List(ar.to_vec()))));
        }
        let v: Vec<i64> = ar[..5].iter().map(int).collect::<R<_>>()?;
        let id = atom(&ar[5]).ok_or("область: нет id колонок")?;
        let ty = match v[0] {
            1 => "Rows",
            2 => "Columns",
            3 => "Rectangle",
            _ => return Err(format!("тип области {}", v[0])),
        };
        self.x.open(tag);
        self.x.leaf("type", ty);
        self.x.leaf("beginRow", &v[2].to_string());
        self.x.leaf("endRow", &v[4].to_string());
        self.x.leaf("beginColumn", &v[1].to_string());
        self.x.leaf("endColumn", &v[3].to_string());
        if id != ZERO {
            self.x.leaf("columnsID", id);
        }
        self.x.close(tag);
        Ok(())
    }

    /// Параметры печати `{{0,N,(ключ,значение)*N}}`; только поля по умолчанию (отступы 1000) — не пишутся.
    fn print_settings(&mut self) -> R<()> {
        let p = self.d.print;
        let l = p.get(0).map(list).transpose()?.unwrap_or(&[]);
        let bad = || format!("параметры печати: {}", short(p));
        if p.list().len() != 1 || l.first().and_then(atom) != Some("0") || int(l.get(1).ok_or_else(bad)?)? as usize * 2 + 2 != l.len() {
            return Err(bad());
        }
        let mut kv = Vec::new();
        for k in (2..l.len()).step_by(2) {
            kv.push((int(&l[k])?, &l[k + 1]));
        }
        let default = kv.len() == 6 && kv.iter().all(|(k, v)| (6..=11).contains(k) && short(v) == "{\"N\",1000}");
        if default {
            return Ok(());
        }
        const ORDER: [(i64, &str); 22] = [(1, "pageOrientation"), (2, "scale"), (3, "collate"), (4, "copies"), (5, "perPage"), (6, "topMargin"), (7, "leftMargin"), (8, "bottomMargin"), (9, "rightMargin"), (10, "headerSize"), (11, "footerSize"), (12, "fitToPage"), (13, "blackAndWhite"), (14, "printerName"), (0, "paper"), (15, "paperSource"), (16, "pageWidth"), (17, "pageHeight"), (18, "printAccuracy"), (19, "duplexType"), (20, "pagePlacementAlternation"), (21, "firstPageNumber")];
        for (k, v) in &kv {
            // Номер первой страницы 1 — по умолчанию, не пишется.
            if *k == 21 && short(v) == "{\"N\",1}" {
                continue;
            }
            if !ORDER.iter().any(|(o, _)| o == k) {
                return Err(format!("параметры печати: ключ {k}"));
            }
        }
        self.x.open("printSettings");
        for (k, tag) in ORDER {
            let Some((_, v)) = kv.iter().find(|(x, _)| *x == k) else { continue };
            if k == 21 && short(v) == "{\"N\",1}" {
                continue;
            }
            let v = list(v)?;
            let val = match (v.first().and_then(Node::str), v.get(1)) {
                (Some("U"), None) => continue,
                // Ключ 0: число — номер бумаги (paper), строка — имя размера (pageSize).
                (Some("S"), Some(Node::Str(s))) => {
                    let tag = if k == 0 { "pageSize" } else { tag };
                    match s.as_str() {
                        "" => self.x.empty(tag),
                        s => self.x.text(tag, s),
                    }
                    continue;
                }
                (Some("N"), Some(Node::Atom(a))) => {
                    // целое — у перечислений; размеры бывают дробными (pageWidth 60.5, ERP 2.5.22)
                    let n: i64 = if matches!(k, 1 | 3 | 12 | 13 | 18 | 19 | 20) { a.parse().map_err(|_| bad())? } else { 0 };
                    match k {
                        1 => match n {
                            1 => "Portrait".into(),
                            2 => "Landscape".into(),
                            _ => return Err(bad()),
                        },
                        3 | 12 | 13 => match n {
                            0 => "false".into(),
                            1 => "true".into(),
                            _ => return Err(bad()),
                        },
                        18 => match n {
                            0 => "Auto".into(),
                            1 => "Accurate".into(),
                            _ => return Err(format!("точность печати {n}")),
                        },
                        19 => match n {
                            1 => "None".into(),
                            // ERP 2.5.22; 3 (FlipPagesTop) не встречен
                            2 => "FlipPagesLeft".into(),
                            4 => "UsePrinterSettings".into(),
                            _ => return Err(format!("двусторонняя печать {n}")),
                        },
                        20 => match n {
                            0 => "Auto".into(),
                            _ => return Err(format!("чередование страниц {n}")),
                        },
                        _ => a.clone(),
                    }
                }
                _ => return Err(bad()),
            };
            self.x.leaf(tag, &val);
        }
        self.x.close("printSettings");
        Ok(())
    }

    /// Линия `{4,0,{0},тип,толщина,0,вид,0}`; вид — линия ячейки или рисунка.
    fn line(&mut self, i: usize) -> R<()> {
        let n = self.d.lines.get(i).ok_or_else(|| format!("нет линии {i}"))?;
        let l = list(n)?;
        let v: Vec<&str> = l.iter().map(|x| atom(x).unwrap_or("?")).collect();
        // Шестое поле (встречены 0, 3, 7) в XML не выводится.
        let ["4", "0", _, style, width, _, kind, gap @ ("0" | "1")] = v[..] else { return Err(format!("линия: {}", short(n))) };
        if short(&l[2]) != "{0}" {
            return Err(format!("линия: {}", short(n)));
        }
        let drawing = match kind {
            CELL_LINE => false,
            DRAWING_LINE => true,
            _ => return Err(format!("линия: вид {kind}")),
        };
        let style = match (style, drawing) {
            // У линий рисунков свой перечень.
            ("2", true) => "Dashed",
            ("3", true) => "Dotted",
            ("4", true) => "DashDotted",
            ("5", true) => "DashDottedDotted",
            (s, true) if !matches!(s, "0" | "1") => return Err(format!("линия рисунка: тип {s}")),
            ("0", _) => "None",
            ("1", _) => "Solid",
            ("2", _) => "Dotted",
            ("3", _) => "Double",
            ("4", _) => "ThinDashed",
            ("5", _) => "ThickDashed",
            ("6", _) => "LargeDashed",
            (s, _) => return Err(format!("линия: тип {s}")),
        };
        let t = if drawing { "SpreadsheetDocumentDrawingLineType" } else { "SpreadsheetDocumentCellLineType" };
        self.x.open(&format!("line width=\"{width}\" gap=\"{}\"", gap == "1"));
        self.x.leaf(&format!("v8ui:style xsi:type=\"v8ui:{t}\""), style);
        self.x.close("line");
        Ok(())
    }

    fn font(&mut self, i: usize) -> R<()> {
        let n = self.d.fonts.get(i).ok_or_else(|| format!("нет шрифта {i}"))?;
        let v85 = crate::xml::form_v59::style_node(n)?; // шрифт 8.5 `{8,…}` — прежний `{7,…}`
        let n = v85.as_ref().unwrap_or(n);
        let l = list(n)?;
        let bad = || format!("шрифт: {}", short(n));
        let a = |k: usize| l.get(k).and_then(atom).ok_or_else(bad);
        let flag = |v: &str| -> R<&'static str> {
            match v {
                "0" => Ok("false"),
                "1" => Ok("true"),
                _ => Err(bad()),
            }
        };
        let weight = |v: &str| -> R<&'static str> {
            match v {
                "0" | "400" => Ok("false"),
                "700" => Ok("true"),
                _ => Err(bad()),
            }
        };
        let mut s = String::from("<font");
        match (a(0)?, a(1)?) {
            // Абсолютный: все свойства, маска не важна.
            (v @ ("6" | "7"), "0") => {
                let face = l.get(16).and_then(Node::str).ok_or_else(bad)?;
                let scale = match (v, l.len()) {
                    ("6", 18) => "100",
                    ("7", 19) => a(18)?,
                    _ => return Err(bad()),
                };
                s.push_str(&format!(" faceName=\"{}\" height=\"{}\" bold=\"{}\" italic=\"{}\" underline=\"{}\" strikeout=\"{}\" kind=\"Absolute\" scale=\"{scale}\"", attr(face), height(a(3)?)?, weight(a(7)?)?, flag(a(8)?)?, flag(a(9)?)?, flag(a(10)?)?));
            }
            // Шрифт Windows (1), стиля (2) или авто (3, без ссылки): только заданные маской свойства.
            (v @ ("6" | "7"), kind @ ("1" | "2" | "3")) => {
                let mask: u32 = a(2)?.parse().map_err(|_| bad())?;
                if mask & !(1 | 2 | 4 | 8 | 16 | 32 | 512) != 0 {
                    return Err(bad());
                }
                let r = if kind == "3" { &[][..] } else { list(&l[3])? };
                let name = match (kind, r) {
                    ("3", []) => String::new(),
                    ("1", [x]) => match atom(x) {
                        Some("0") => "sys:DefaultGUIFont",
                        Some("1") => "sys:OEMFixedFont",
                        Some("2") => "sys:ANSIFixedFont",
                        Some("3") => "sys:ANSIVariableFont",
                        Some("4") => "sys:SystemFont",
                        _ => return Err(bad()),
                    }
                    .to_string(),
                    ("2", [x]) => match atom(x) {
                        Some("-20") => "style:TextFont",
                        Some("-30") => "style:SmallTextFont",
                        Some("-31") => "style:NormalTextFont",
                        Some("-32") => "style:LargeTextFont",
                        Some("-33") => "style:ExtraLargeTextFont",
                        _ => return Err(bad()),
                    }
                    .to_string(),
                    ("2", [z, g]) if atom(z) == Some("0") => format!("style:{}", (self.names)("StyleItem", atom(g).unwrap_or("")).ok_or_else(bad)?),
                    _ => return Err(bad()),
                };
                let mut c = Cur::new(&l[if kind == "3" { 3 } else { 4 }..]);
                let mut props = String::new();
                if mask & 2 != 0 {
                    props.push_str(&format!(" height=\"{}\"", height(atom(c.next()?).ok_or_else(bad)?)?));
                }
                if mask & 4 != 0 {
                    props.push_str(&format!(" bold=\"{}\"", weight(atom(c.next()?).ok_or_else(bad)?)?));
                }
                for (bit, tag) in [(8, "italic"), (16, "underline"), (32, "strikeout")] {
                    if mask & bit != 0 {
                        props.push_str(&format!(" {tag}=\"{}\"", flag(atom(c.next()?).ok_or_else(bad)?)?));
                    }
                }
                let face = if mask & 1 != 0 { Some(c.next()?.str().ok_or_else(bad)?) } else { None };
                // Флаг (0/1, как у абсолютного шрифта перед масштабом) в XML не выводится; у версии 6 масштаба нет.
                if !matches!(c.next().map(atom)?, Some("0" | "1")) {
                    return Err(bad());
                }
                let scale = if v == "7" { Some(c.int()?) } else { None };
                if !c.rest().is_empty() {
                    return Err(bad());
                }
                if kind == "1" {
                    s.push_str(" xmlns:sys=\"http://v8.1c.ru/8.1/data/ui/fonts/system\"");
                }
                if !name.is_empty() {
                    s.push_str(&format!(" ref=\"{}\"", attr(&name)));
                }
                if let Some(f) = face {
                    s.push_str(&format!(" faceName=\"{}\"", attr(f)));
                }
                s.push_str(&props);
                s.push_str(match kind {
                    "1" => " kind=\"WindowsFont\"",
                    "2" => " kind=\"StyleItem\"",
                    _ => " kind=\"AutoFont\"",
                });
                if mask & 512 != 0 {
                    s.push_str(&format!(" scale=\"{}\"", scale.ok_or_else(bad)?));
                }
            }
            _ => return Err(bad()),
        }
        s.push_str("/>\r\n");
        self.x.pad();
        self.x.o.push_str(&s);
        Ok(())
    }

    /// Цвет: (объявление пространства имен, значение).
    fn color(&self, i: &Node) -> R<(&'static str, String)> {
        self.color_at(int(i)? as usize)
    }

    fn color_at(&self, i: usize) -> R<(&'static str, String)> {
        let n = match self.d.restyle.iter().find(|(k, _)| *k == i) {
            Some((_, n)) => *n,
            None => self.d.colors.get(i).ok_or_else(|| format!("нет цвета {i}"))?,
        };
        // цвет 8.5 `{4,вид,{значение},вид}` (запись версии 12, как пишет загрузка) — прежний `{3,вид,{значение}}`
        let v85 = crate::xml::form_v59::style_node(n)?;
        let n = v85.as_ref().unwrap_or(n);
        let l = list(n)?;
        let v = l.get(2).map(list).transpose()?.unwrap_or(&[]);
        match (l.first().and_then(atom), l.get(1).and_then(atom), v) {
            (Some("3"), Some("0"), [c]) => {
                let c = int(c)?;
                Ok(("", format!("#{:02X}{:02X}{:02X}", c & 0xff, c >> 8 & 0xff, c >> 16 & 0xff)))
            }
            (Some("3"), Some("1"), [c]) => {
                let k = int(c)?;
                let name = WINDOWS.iter().find(|(i, _)| *i == k).map(|(_, n)| *n).ok_or_else(|| format!("цвет Windows {k}"))?;
                Ok((" xmlns:d3p1=\"http://v8.1c.ru/8.1/data/ui/colors/windows\"", format!("d3p1:{name}")))
            }
            (Some("3"), Some("2"), [c]) => {
                let k = int(c)?;
                let name = WEB.iter().find(|(i, _)| *i == k).map(|(_, n)| *n).ok_or_else(|| format!("веб-цвет {k}"))?;
                Ok((" xmlns:d3p1=\"http://v8.1c.ru/8.1/data/ui/colors/web\"", format!("d3p1:{name}")))
            }
            (Some("3"), Some("3"), [c]) => {
                let k = int(c)?;
                let name = STYLE.iter().find(|(i, _)| *i == k).map(|(_, n)| *n).ok_or_else(|| format!("цвет стиля {k}"))?;
                Ok(("", format!("style:{name}")))
            }
            (Some("3"), Some("4"), [z]) if atom(z) == Some("0") => Ok(("", "auto".into())),
            (Some("3"), Some("3"), [z, g]) if atom(z) == Some("0") => {
                let g = atom(g).unwrap_or("");
                (self.names)("StyleItem", g).map(|s| ("", format!("style:{s}"))).ok_or_else(|| format!("нет элемента стиля {g}"))
            }
            _ => Err(format!("цвет: {}", short(n))),
        }
    }

    fn format(&mut self, n: &Node, drawing: bool) -> R<()> {
        let f = fmt(n)?;
        if f.vals.is_empty() {
            self.x.empty("format");
            return Ok(());
        }
        let mut out: Vec<(usize, String)> = Vec::new();
        let mut put = |rank: usize, s: String| out.push((rank, s));
        let border: Vec<Option<&Node>> = (1..=4).map(|b| f.get(b)).collect();
        let all4 = border.iter().all(|b| b.is_some()) && border.windows(2).all(|w| w[0] == w[1]);
        for &(b, v) in &f.vals {
            let i = || int(v);
            let bool01 = |inv: bool| -> R<String> {
                match (int(v)?, inv) {
                    (0, false) | (1, true) => Ok("false".into()),
                    (1, false) | (0, true) => Ok("true".into()),
                    (x, _) => Err(format!("формат: бит {b} = {x}")),
                }
            };
            let (rank, tag, val): (usize, &str, String) = match b {
                0 => (10, "font", self.fonts.id(i()? as usize).to_string()),
                1 if drawing => (5, "drawingBorder", self.lines.id(i()? as usize).to_string()),
                // Встречен дважды, со значениями 0 и 1, — в XML его нет.
                2 if drawing && matches!(atom(v), Some("0" | "1")) => continue,
                2 if drawing => return Err(format!("формат рисунка: бит {b}")),
                // У рисунка бит 3 — маска наличия границ, бит 4 — печать.
                3 if drawing => {
                    let m = i()?;
                    if m & !15 != 0 {
                        return Err(format!("формат рисунка: границы {m}"));
                    }
                    for (k, side) in ["Left", "Top", "Right", "Bottom"].iter().enumerate() {
                        put(6 + k, format!("\t\t<drawingHave{side}Border>{}</drawingHave{side}Border>\r\n", m >> k & 1 == 1));
                    }
                    continue;
                }
                // Как и защита, хранится инвертированным.
                4 if drawing => (1, "print", bool01(true)?),
                1..=4 => {
                    let id = self.lines.id(i()? as usize).to_string();
                    if all4 {
                        if b != 1 {
                            continue;
                        }
                        (20, "border", id)
                    } else {
                        (10 + b as usize, ["", "leftBorder", "topBorder", "rightBorder", "bottomBorder"][b as usize], id)
                    }
                }
                5 | 10 | 11 | 13 => {
                    let (ns, c) = self.color(v)?;
                    let (rank, tag) = match b {
                        5 => (32, "borderColor"),
                        10 => (44, "textColor"),
                        11 => (46, "backColor"),
                        _ => (47, "patternColor"),
                    };
                    put(rank, format!("\t\t<{tag}{ns}>{c}</{tag}>\r\n"));
                    continue;
                }
                6 => (30, "height", atom(v).ok_or("высота")?.to_string()),
                7 => (34, "width", atom(v).ok_or("ширина")?.to_string()),
                8 => (40, "horizontalAlignment", halign(i()?)?.into()),
                9 => (42, "verticalAlignment", valign(i()?)?.into()),
                14 => (52, "textPlacement", match i()? {
                    0 | 255 => "Auto",
                    1 => "Cut",
                    2 => "Block",
                    3 => "Wrap",
                    x => return Err(format!("размещение текста {x}")),
                }.into()),
                15 => (54, "fillType", match i()? {
                    0 => "Text",
                    1 => "Parameter",
                    2 => "Template",
                    x => return Err(format!("заполнение {x}")),
                }.into()),
                16 => (56, "protection", bool01(true)?),
                17 => (58, "hidden", bool01(false)?),
                20 => (63, "bySelectedColumns", bool01(false)?),
                18 => (61, "textOrientation", i()?.to_string()),
                19 => (62, "detailsUse", match i()? {
                    0 => "Cell",
                    1 => "Row",
                    2 => "WithoutProcessing",
                    x => return Err(format!("использование расшифровки {x}")),
                }.into()),
                21 => (66, "markNegatives", bool01(false)?),
                22 => (68, "containsValue", bool01(false)?),
                26 => (78, "hyperLink", bool01(false)?),
                28 => (80, "autoMarkIncomplete", bool01(false)?),
                30 => (82, "indent", i()?.to_string()),
                31 => (86, "autoIndent", i()?.to_string()),
                33 => (90, "columnSizeChange", match i()? {
                    0 => "Normal",
                    1 => "QuickChange",
                    x => return Err(format!("изменение размера колонки {x}")),
                }.into()),
                12 => (48, "pattern", match i()? {
                    0 => "Solid".into(),
                    255 => "WithoutPattern".into(),
                    x @ 1..=20 => format!("Pattern{x}"),
                    x => return Err(format!("узор {x}")),
                }),
                35 => (100, "picIndex", i()?.to_string()),
                38 => (101, "pictureSizeMode", match i()? {
                    0 => "RealSize",
                    1 => "Stretch",
                    2 => "Proportionally",
                    3 => "Tile",
                    4 => "AutoSize",
                    5 => "RealSizeIgnoreScale",
                    6 => "AutoSizeIgnoreScale",
                    7 => "ByFontSize",
                    x => return Err(format!("размер картинки {x}")),
                }.into()),
                36 => (102, "picHorizontalAlignment", halign(i()?)?.into()),
                37 => (103, "picVerticalAlignment", valign(i()?)?.into()),
                39 => (104, "textPosition", match i()? {
                    0 => "Left",
                    1 => "Right",
                    2 => "Top",
                    3 => "Bottom",
                    4 => "OnTop",
                    5 => "Auto",
                    x => return Err(format!("положение текста {x}")),
                }.into()),
                40 => (35, "autoWidthCalculation", bool01(false)?),
                41 => (36, "widthWeightFactor", i()?.to_string()),
                42 => (105, "leftMargin", i()?.to_string()),
                43 => (106, "topMargin", i()?.to_string()),
                44 => (107, "rightMargin", i()?.to_string()),
                45 => (108, "bottomMargin", i()?.to_string()),
                27 => (79, "quickDrag", bool01(false)?),
                29 => (81, "markIncomplete", bool01(false)?),
                24 | 32 | 34 => {
                    let (list, rank, tag) = match b {
                        24 => (self.d.fstrings, 72, "format"),
                        32 => (self.d.fstrings, 88, "editFormat"),
                        _ => (self.d.masks, 99, "mask"),
                    };
                    let s = list.get(i()? as usize).ok_or_else(|| format!("нет строки для {tag}"))?;
                    let mut x = X { o: String::new(), d: 2 };
                    x.ml(tag, s)?;
                    put(rank, x.o);
                    continue;
                }
                23 => {
                    let t = self.d.types.get(i()? as usize).ok_or("нет типа значения")?;
                    put(70, value_type(t, self.names)?);
                    continue;
                }
                25 => (74, "controlType", self.d.controls.get(i()? as usize).and_then(atom).ok_or("нет вида элемента управления")?.to_string()),
                _ => return Err(format!("формат: бит {b} в {}", short(n))),
            };
            put(rank, format!("\t\t<{tag}>{val}</{tag}>\r\n"));
        }
        out.sort_by_key(|(r, _)| *r);
        self.x.open("format");
        for (_, s) in out {
            self.x.o.push_str(&s);
        }
        self.x.close("format");
        Ok(())
    }
}

struct Merge<'a> {
    kind: i64,
    r: i64,
    c: i64,
    h: i64,
    w: i64,
    id: &'a str,
}

/// `{к1,с1,к2,с2,вид}`, у объединений в наборах колонок еще id набора.
fn merge(m: &Node, with_id: bool) -> R<Merge<'_>> {
    let l = list(m)?;
    if l.len() != 5 + with_id as usize {
        return Err(format!("объединение: {}", short(m)));
    }
    let v: Vec<i64> = l[..5].iter().map(int).collect::<R<_>>()?;
    let id = if with_id { atom(&l[5]).ok_or("объединение: нет id")? } else { ZERO };
    if !(0..=2).contains(&v[4]) {
        return Err(format!("объединение: вид {}", v[4]));
    }
    Ok(Merge { kind: v[4], r: v[1], c: v[0], h: v[3] - v[1], w: v[2] - v[0], id })
}

/// Описание типов `{"Pattern",{"S",длина,1},{"N",..},..}` → `<valueType>`: типы по порядку, затем квалификаторы
/// числа, строки, даты. Ссылочный тип `{"#",uuid}` — по имени из `names("Type", uuid)`.
fn value_type(n: &Node, names: &dyn Fn(&str, &str) -> Option<String>) -> R<String> {
    let l = list(n)?;
    let bad = || format!("тип значения: {}", short(n));
    if l.first().and_then(Node::str) != Some("Pattern") {
        return Err(bad());
    }
    let mut x = X { o: String::new(), d: 2 };
    // описание без типов — пустой элемент (ERP 2.5.22)
    if l.len() == 1 {
        x.pad();
        x.o.push_str("<valueType/>\r\n");
        return Ok(x.o);
    }
    x.open("valueType");
    let (mut num, mut st, mut date) = (None, None, None);
    for t in &l[1..] {
        let t = list(t)?;
        match (t.first().and_then(Node::str), t.len()) {
            (Some("S"), 1 | 3) => {
                st = Some(t);
                x.leaf("v8:Type", "xs:string");
            }
            (Some("N"), 1 | 4) => {
                num = Some(t);
                x.leaf("v8:Type", "xs:decimal");
            }
            (Some("D"), 1 | 2) => {
                date = Some(t);
                x.leaf("v8:Type", "xs:dateTime");
            }
            (Some("B"), 1) => x.leaf("v8:Type", "xs:boolean"),
            // тип без имени (объект удален) XDTO пишет по GUID
            (Some("#"), 2) => match names("Type", atom(&t[1]).unwrap_or("")) {
                Some(name) => x.leaf("v8:Type xmlns:d4p1=\"http://v8.1c.ru/8.1/data/enterprise/current-config\"", &format!("d4p1:{}", esc(&name))),
                None => x.leaf("v8:TypeId", atom(&t[1]).filter(|g| crate::brace::is_guid(g)).ok_or_else(bad)?),
            },
            _ => return Err(bad()),
        }
    }
    let a = |t: &[Node], k: usize| t.get(k).map(int).transpose();
    if let Some(t) = num {
        x.open("v8:NumberQualifiers");
        x.leaf("v8:Digits", &a(t, 1)?.unwrap_or(0).to_string());
        x.leaf("v8:FractionDigits", &a(t, 2)?.unwrap_or(0).to_string());
        let sign = match a(t, 3)? {
            None | Some(0) => "Any",
            Some(1) => "Nonnegative",
            _ => return Err(bad()),
        };
        x.leaf("v8:AllowedSign", sign);
        x.close("v8:NumberQualifiers");
    }
    if let Some(t) = st {
        x.open("v8:StringQualifiers");
        x.leaf("v8:Length", &a(t, 1)?.unwrap_or(0).to_string());
        let al = match a(t, 2)? {
            None | Some(1) => "Variable",
            Some(0) => "Fixed",
            _ => return Err(bad()),
        };
        x.leaf("v8:AllowedLength", al);
        x.close("v8:StringQualifiers");
    }
    if let Some(t) = date {
        // Без квалификатора — дата и время.
        let f = match t.get(1).map(|n| n.str()) {
            None => "DateTime",
            Some(Some("D")) => "Date",
            Some(Some("T")) => "Time",
            _ => return Err(bad()),
        };
        x.open("v8:DateQualifiers");
        x.leaf("v8:DateFractions", f);
        x.close("v8:DateQualifiers");
    }
    x.close("valueType");
    Ok(x.o)
}

/// Высота шрифта хранится в десятых пункта: 98 → 9.8.
fn height(v: &str) -> R<String> {
    let h: i64 = v.parse().map_err(|_| format!("высота шрифта {v}"))?;
    Ok(if h % 10 == 0 { (h / 10).to_string() } else { format!("{}.{}", h / 10, h % 10) })
}

/// base64 как в записи: строки по 64 символа через CR CR LF, платформа пишет CRLF.
fn base64_text(b: &str) -> String {
    b.replace("\r\r\n", "\r\n")
}

fn halign(v: i64) -> R<&'static str> {
    Ok(match v {
        0 => "Left",
        2 => "Right",
        4 => "Justify",
        5 => "Auto",
        6 => "Center",
        x => return Err(format!("горизонтальное положение {x}")),
    })
}

fn valign(v: i64) -> R<&'static str> {
    Ok(match v {
        0 => "Top",
        8 => "Bottom",
        24 => "Center",
        x => return Err(format!("вертикальное положение {x}")),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mxl(body: &str) -> Vec<u8> {
        [b"MOXCEL\0\x08\0\x01\0\x09\0".as_slice(), "\u{feff}".as_bytes(), body.as_bytes()].concat()
    }

    #[test]
    fn empty_rows_and_column_formats() {
        // Synthetic empty spreadsheet: four columns, repeated format index, arbitrary widths.
        let raw = mxl(r#"{8,1,9,{"ru","ru",1,1,"ru","Русский","Русский"},{128,72},{0},0,{0,0},{0,0},{0,0},{0,0},{0,0},{0,0},1,2,0,{4,0,00000000-0000-0000-0000-000000000000,4,0,1,1,2,2,3,3,2},0,0,0,0,0,0,0,0,0,{0},{0},{0},{0},"",{{0,6,6,{"N",1000},7,{"N",1000},8,{"N",1000},9,{"N",1000},10,{"N",1000},11,{"N",1000}}},{0,-1,-1,-1,-1,00000000-0000-0000-0000-000000000000},0,0,0,0,0,0,0,1,0,1,3,{128,211},{128,213},{128,217},0,0,0,0,2,{3,3,{-1}},{3,3,{-3}},0,0,0,"",0,{3,0,0,100,1,1,0,1,1,0,0,0,0,0,0,0,0,0,0,0,0,"",0,0,0,0,0,0,0},{0},0,0,0,1}"#);
        let x = String::from_utf8(template(&raw, &|_, _| None).unwrap()).unwrap();
        assert!(x.starts_with("\u{feff}<?xml version=\"1.0\" encoding=\"UTF-8\"?>\r\n<document xmlns="));
        let body: String = x.split("\r\n").skip(11).map(str::trim).collect();
        assert_eq!(
            body,
            "<columns><size>4</size><columnsItem><index>0</index><column><formatIndex>1</formatIndex></column></columnsItem>\
             <columnsItem><index>1</index><column><formatIndex>2</formatIndex></column></columnsItem>\
             <columnsItem><index>2</index><column><formatIndex>3</formatIndex></column></columnsItem>\
             <columnsItem><index>3</index><column><formatIndex>2</formatIndex></column></columnsItem></columns>\
             <rowsItem><index>0</index><row><empty>true</empty></row></rowsItem><templateMode>true</templateMode>\
             <defaultFormatIndex>4</defaultFormatIndex><vgRows>0</vgRows><format><width>211</width></format>\
             <format><width>213</width></format><format><width>217</width></format><format><width>72</width></format></document>"
        );
    }

    /// Наименьший макет (как в empty_rows_and_column_formats) — основа для подстановок.
    const BASE: &str = r#"{8,1,9,{"ru","ru",1,1,"ru","Русский","Русский"},{128,72},{0},0,{0,0},{0,0},{0,0},{0,0},{0,0},{0,0},1,2,0,{4,0,00000000-0000-0000-0000-000000000000,4,0,1,1,2,2,3,3,2},0,0,0,0,0,0,0,0,0,{0},{0},{0},{0},"",{{0,6,6,{"N",1000},7,{"N",1000},8,{"N",1000},9,{"N",1000},10,{"N",1000},11,{"N",1000}}},{0,-1,-1,-1,-1,00000000-0000-0000-0000-000000000000},0,0,0,0,0,0,0,1,0,1,3,{128,211},{128,213},{128,217},0,0,0,0,2,{3,3,{-1}},{3,3,{-3}},0,0,0,"",0,{3,0,0,100,1,1,0,1,1,0,0,0,0,0,0,0,0,0,0,0,0,"",0,0,0,0,0,0,0},{0},0,0,0,1}"#;

    #[test]
    fn map_order_synthetic_collision() {
        // Two synthetic keys in one bucket, separated by a key in another bucket.
        let rec = ["Key2", "Key1", "Key3"];
        assert_eq!(map_order(&rec), [1, 2, 0]);
        // без столкновений — обратный порядок; 17 ключей — с перехешированием, все на месте
        assert_eq!(map_order(&["к1", "к2"]), [1, 0]);
        let many: Vec<String> = (0..17).map(|i| format!("к{i}")).collect();
        let mut o = map_order(&many.iter().map(String::as_str).collect::<Vec<_>>());
        o.sort();
        assert_eq!(o, (0..17).collect::<Vec<_>>());
    }

    #[test]
    fn detail_values() {
        // Строка 0 с расшифровками: Структура, Соответствие (пары в обратном порядке), СписокЗначений, СКД.
        let row = concat!(
            r##"1,2,1,0,0,4,"##,
            r##"0,{4,0,{"#",4238019d-7e49-4fc9-91db-b6b951d5cf8e,{2,{{"S","А"},{"U"}},{{"S","Б"},{"B",1}}}}},"##,
            r##"1,{4,0,{"#",3d48feae-a9c6-4c5a-a099-9eb6477630c6,{2,{{"S","к1"},{"N",1}},{{"S","к2"},{"S",""}}}}},"##,
            r##"2,{4,0,{"#",4772b3b4-f4a3-49c0-a1a5-8cb5961511a3,{6,11111111-1111-4111-8111-111111111111,0,0,{1,{11111111-1111-4111-8111-111111111111,{"П",0,{"D",20260101000000},{4,0,{0},"",-1,-1,0,0,""},3,0,""}}},{"Pattern"},0,3}}},"##,
            r##"3,{4,0,{"#",3031edd8-c3df-47b2-98ca-47f628d4ec18,{297}}},"##,
        );
        let raw = mxl(&BASE.replacen("1,2,0,", row, 1));
        let x = String::from_utf8(template(&raw, &|_, _| None).unwrap()).unwrap();
        let body: String = x.split("\r\n").map(str::trim).collect();
        let cells = concat!(
            r#"<c><c><f>0</f><d xsi:type="v8:Structure"><v8:Property name="А"><v8:Value xsi:nil="true"/></v8:Property><v8:Property name="Б"><v8:Value xsi:type="xs:boolean">true</v8:Value></v8:Property></d></c></c>"#,
            r#"<c><c><f>0</f><d xsi:type="v8:Map"><v8:pair><v8:Key xsi:type="xs:string">к2</v8:Key><v8:Value xsi:type="xs:string"/></v8:pair><v8:pair><v8:Key xsi:type="xs:string">к1</v8:Key><v8:Value xsi:type="xs:decimal">1</v8:Value></v8:pair></d></c></c>"#,
            r#"<c><c><f>0</f><d xsi:type="v8:ValueListType"><v8:valueType/><v8:lastId xsi:type="xs:decimal">3</v8:lastId><v8:item><v8:value xsi:type="xs:dateTime">2026-01-01T00:00:00</v8:value><v8:presentation>П</v8:presentation><v8:id xsi:type="xs:decimal">3</v8:id></v8:item></d></c></c>"#,
            r#"<c><c><f>0</f><r>297</r></c></c>"#,
        );
        assert!(body.contains(cells), "{body}");
    }

    #[test]
    fn detail_array() {
        // Synthetic structure array and empty array.
        let row = concat!(
            r##"1,2,1,0,0,2,"##,
            r##"0,{4,0,{"#",51e7a0d2-530b-11d4-b98a-008048da3034,{2,{"#",4238019d-7e49-4fc9-91db-b6b951d5cf8e,{1,{{"S","А"},{"N",2}}}},{"U"}}}},"##,
            r##"1,{4,0,{"#",51e7a0d2-530b-11d4-b98a-008048da3034,{0}}},"##,
        );
        let raw = mxl(&BASE.replacen("1,2,0,", row, 1));
        let x = String::from_utf8(template(&raw, &|_, _| None).unwrap()).unwrap();
        let body: String = x.split("\r\n").map(str::trim).collect();
        let cells = concat!(
            r#"<c><c><f>0</f><d xsi:type="v8:Array"><v8:Value xsi:type="v8:Structure"><v8:Property name="А"><v8:Value xsi:type="xs:decimal">2</v8:Value></v8:Property></v8:Value><v8:Value xsi:nil="true"/></d></c></c>"#,
            r#"<c><c><f>0</f><d xsi:type="v8:Array"/></c></c>"#,
        );
        assert!(body.contains(cells), "{body}");
    }

    #[test]
    fn merges_from_one_cell_keep_largest() {
        let raw = mxl(&BASE.replacen(r#"{0},{0},{0},{0},"""#, r#"{3,{5,4,7,4,0},{5,4,6,4,0},{5,4,12,4,0}},{0},{0},{0},"""#, 1));
        let x = String::from_utf8(template(&raw, &|_, _| None).unwrap()).unwrap();
        let body: String = x.split("\r\n").map(str::trim).collect();
        assert_eq!(body.matches("<merge>").count(), 1);
        assert!(body.contains("<merge><r>4</r><c>5</c><w>7</w></merge>"), "{body}");
    }

    #[test]
    fn multilang_text_and_enums() {
        let mut x = X { o: String::new(), d: 0 };
        x.ml("tl", &brace::parse("{1,1,{\"ru\",\"a<b\r\nc\"}}").unwrap()).unwrap();
        assert_eq!(x.o, "<tl>\r\n\t<v8:item>\r\n\t\t<v8:lang>ru</v8:lang>\r\n\t\t<v8:content>a&lt;b\nc</v8:content>\r\n\t</v8:item>\r\n</tl>\r\n");
        assert_eq!(halign(6).unwrap(), "Center");
        assert!(valign(3).is_err());
    }
}

/// Имя цвета по номеру в таблице `STYLE`, `WINDOWS` или `WEB`.
pub(crate) fn color_name(t: &[(i64, &'static str)], code: i64) -> Option<&'static str> {
    t.iter().find(|(i, _)| *i == code).map(|(_, n)| *n)
}

/// Цвета стиля платформы `{3,3,{-N}}` (идентификаторы из ресурсов платформы).
/// Элементы стиля платформы: номер → имя, в порядке платформы (так их пишет Style.xml): core85 FUN_180188be0, новые
/// (-42…-59) — FUN_18018ad60; пары имен 0x1805484b0–0x180548770 (снято 30.10.2026).
pub(crate) const STYLE: &[(i64, &str)] = &[
    (-1, "FormBackColor"),
    (-3, "FormTextColor"),
    (-7, "ButtonBackColor"),
    (-21, "ButtonTextColor"),
    (-10, "FieldBackColor"),
    (-11, "FieldTextColor"),
    (-14, "FieldSelectionBackColor"),
    (-15, "FieldSelectedTextColor"),
    (-13, "FieldAlternativeBackColor"),
    (-23, "ToolTipBackColor"),
    (-24, "ToolTipTextColor"),
    (-16, "SpecialTextColor"),
    (-17, "NegativeTextColor"),
    (-22, "BorderColor"),
    (-25, "ReportHeaderBackColor"),
    (-26, "ReportGroup1BackColor"),
    (-27, "ReportGroup2BackColor"),
    (-28, "ReportLineColor"),
    (-18, "ControlBorder"),
    (-20, "TextFont"),
    (-30, "SmallTextFont"),
    (-31, "NormalTextFont"),
    (-32, "LargeTextFont"),
    (-33, "ExtraLargeTextFont"),
    (-48, "TitleLevel1"),
    (-59, "TitleLevel2"),
    (-50, "TitleLevel3"),
    (-51, "TitleLevel4"),
    (-52, "TitleLevel5"),
    (-53, "TextLevel1"),
    (-54, "TextLevel2"),
    (-55, "TextLevel3"),
    (-56, "SubtitleLevel1"),
    (-57, "SubtitleLevel2"),
    (-34, "ButtonBorderColor"),
    (-35, "TableHeaderBackColor"),
    (-36, "TableHeaderTextColor"),
    (-37, "TableFooterBackColor"),
    (-38, "TableFooterTextColor"),
    (-44, "ActivityColor"),
    (-46, "AccentColor"),
    (-42, "NavigationColor"),
    (-43, "AuxiliaryNavigationColor"),
    (-47, "ImportantColor"),
];

/// Системные цвета Windows `{3,1,{N}}`.
pub(crate) const WINDOWS: &[(i64, &str)] = &[
    (-1, "MiddleGradientActiveCaption"),
    (-2, "MiddleGradientInactiveCaption"),
    (0, "ScrollBar"),
    (1, "Desktop"),
    (2, "ActiveTitleBar"),
    (3, "InactiveTitleBar"),
    (4, "MenuBar"),
    (5, "WindowBackground"),
    (6, "WindowFrame"),
    (7, "MenuItemText"),
    (8, "WindowText"),
    (9, "ActiveTitleBarText"),
    (10, "ActiveBorder"),
    (11, "InactiveBorder"),
    (12, "ApplicationWorkspace"),
    (13, "Highlight"),
    (14, "HighlightText"),
    (15, "ButtonFace"),
    (16, "ButtonShadow"),
    (17, "DisabledText"),
    (18, "ButtonText"),
    (19, "InactiveTitleBarText"),
    (20, "ButtonHighlight"),
    (21, "ButtonDarkShadow"),
    (22, "ButtonLightShadow"),
    (23, "ToolTipText"),
    (24, "ToolTip"),
    (26, "HotLight"),
    (27, "GradientActiveCaption"),
    (28, "GradientInactiveCaption"),
];

/// Веб-цвета `{3,2,{N}}` (идентификаторы из ресурсов платформы).
pub(crate) const WEB: &[(i64, &str)] = &[
    (1, "AliceBlue"),
    (2, "AntiqueWhite"),
    (3, "Aqua"),
    (4, "Aquamarine"),
    (5, "Azure"),
    (6, "Beige"),
    (7, "Bisque"),
    (8, "Black"),
    (9, "BlanchedAlmond"),
    (10, "Blue"),
    (11, "BlueViolet"),
    (12, "Brown"),
    (13, "BurlyWood"),
    (14, "CadetBlue"),
    (15, "Chartreuse"),
    (16, "Chocolate"),
    (17, "Coral"),
    (18, "CornFlowerBlue"),
    (19, "CornSilk"),
    (20, "Cream"),
    (21, "Crimson"),
    (22, "Cyan"),
    (23, "DarkBlue"),
    (24, "DarkCyan"),
    (25, "DarkGoldenRod"),
    (26, "DarkGray"),
    (27, "DarkGreen"),
    (28, "DarkKhaki"),
    (29, "DarkMagenta"),
    (30, "DarkOliveGreen"),
    (31, "DarkOrange"),
    (32, "DarkOrchid"),
    (33, "DarkRed"),
    (34, "DarkSalmon"),
    (35, "DarkSeaGreen"),
    (36, "DarkSlateBlue"),
    (37, "DarkSlateGray"),
    (38, "DarkTurquoise"),
    (39, "DarkViolet"),
    (40, "DeepPink"),
    (41, "DeepSkyBlue"),
    (42, "DimGray"),
    (43, "DodgerBlue"),
    (44, "FireBrick"),
    (45, "FloralWhite"),
    (46, "ForestGreen"),
    (47, "Fuchsia"),
    (48, "Gainsboro"),
    (49, "GhostWhite"),
    (50, "Gold"),
    (51, "Goldenrod"),
    (52, "Gray"),
    (53, "Green"),
    (54, "GreenYellow"),
    (55, "HoneyDew"),
    (56, "HotPink"),
    (57, "IndianRed"),
    (58, "Indigo"),
    (59, "Ivory"),
    (60, "Khaki"),
    (61, "Lavender"),
    (62, "LavenderBlush"),
    (63, "LawnGreen"),
    (64, "LemonChiffon"),
    (65, "LightBlue"),
    (66, "LightCoral"),
    (67, "LightCyan"),
    (68, "LightGoldenRod"),
    (69, "LightGoldenRodYellow"),
    (70, "LightGreen"),
    (71, "LightGray"),
    (72, "LightPink"),
    (73, "LightSalmon"),
    (74, "LightSeaGreen"),
    (75, "LightSkyBlue"),
    (76, "LightSlateBlue"),
    (77, "LightSlateGray"),
    (78, "LightSteelBlue"),
    (79, "LightYellow"),
    (80, "Lime"),
    (81, "LimeGreen"),
    (82, "Linen"),
    (83, "Magenta"),
    (84, "Maroon"),
    (85, "MediumAquaMarine"),
    (86, "MediumBlue"),
    (87, "MediumGray"),
    (88, "MediumGreen"),
    (89, "MediumOrchid"),
    (90, "MediumPurple"),
    (91, "MediumSeaGreen"),
    (92, "MediumSlateBlue"),
    (93, "MediumSpringGreen"),
    (94, "MediumTurquoise"),
    (95, "MediumVioletRed"),
    (96, "MidnightBlue"),
    (97, "MintCream"),
    (98, "MistyRose"),
    (99, "Moccasin"),
    (100, "NavajoWhite"),
    (101, "Navy"),
    (102, "OldLace"),
    (103, "Olive"),
    (104, "Olivedrab"),
    (105, "Orange"),
    (106, "OrangeRed"),
    (107, "Orchid"),
    (108, "PaleGoldenrod"),
    (109, "PaleGreen"),
    (110, "PaleTurquoise"),
    (111, "PaleVioletRed"),
    (112, "PapayaWhip"),
    (113, "PeachPuff"),
    (114, "Peru"),
    (115, "Pink"),
    (116, "Plum"),
    (117, "PowderBlue"),
    (118, "Purple"),
    (119, "Red"),
    (120, "RosyBrown"),
    (121, "RoyalBlue"),
    // В ресурсах у SaddleBrown стоит 124, как у SandyBrown; по алфавитному порядку и сверке с БП — 122.
    (122, "SaddleBrown"),
    (123, "Salmon"),
    (124, "SandyBrown"),
    (125, "Seagreen"),
    (126, "SeaShell"),
    (127, "Sienna"),
    (128, "Silver"),
    (129, "SkyBlue"),
    (130, "SlateBlue"),
    (131, "SlateGray"),
    (132, "Snow"),
    (133, "SpringGreen"),
    (134, "SteelBlue"),
    (135, "Tan"),
    (136, "Teal"),
    (137, "Thistle"),
    (138, "Tomato"),
    (139, "Turquoise"),
    (140, "Violet"),
    (141, "VioletRed"),
    (142, "Wheat"),
    (143, "White"),
    (144, "WhiteSmoke"),
    (145, "Yellow"),
    (146, "YellowGreen"),
];
