//! Запись формы версии 59 (так 8.5 сохраняет все формы) → прежние версии, которые читает `form.rs`.
//!
//! Сверка каркаса (формы БП, сохраненные 8.5) с теми же формами БП 3.0.171: 8.5 поднимает версию узла и дописывает
//! поля в конец, прежние поля остаются на местах. Цвет `{4,вид,значение,вид}` — `{3,вид,значение}`, шрифт `{8,...}` —
//! `{7,...}` без новых полей; остальные узлы — по таблице [`RULES`], ключ — тип элемента, версия родителя, версия
//! узла и вид его второго поля. Дописанные поля отбрасываются, только если значение каждого есть в [`TAILS`]
//! (выгрузка платформы от него не зависит); иначе форма отвергается. Таблица выводится заново так:
//! `CFKIT_V59_TAILS=1 cfkit export-forms` по каркасу (каждая форма отвергается со списком отброшенных значений) и
//! `tools/skel/v59/tails.py` — значения, встреченные в формах, совпавших с выгрузкой платформы.

use crate::brace::Node;
use std::collections::HashMap;

/// Флаги [`downgrade`] по id элемента (`rec` — сама форма): свойства 8.5, которые прежняя версия не выражает.
pub const UNSET_GROUP: u32 = 1;
pub const UNSET_ROW_SELECTION: u32 = 2;
pub const SHOW_COMMAND_BAR: u32 = 4;
pub const UNSET_AUTO_EDIT: u32 = 8;
pub const UNSET_WINDOW_MODE: u32 = 16;
pub const UNSET_SCROLL: u32 = 32;
pub const UNSET_REPRESENTATION: u32 = 64;
pub const FIELD_V59: u32 = 128;
pub const MARK_REQUIRED_FALSE: u32 = 256;
pub const MARK_REQUIRED_TRUE: u32 = 512;
pub const HLINES_FALSE: u32 = 1024;
pub const HLINES_TRUE: u32 = 2048;
pub const VLINES_FALSE: u32 = 4096;
pub const VLINES_TRUE: u32 = 8192;
pub const ALTERNATION_FALSE: u32 = 16384;
pub const ALTERNATION_TRUE: u32 = 32768;
pub const USE_COPY_FALSE: u32 = 1<<16;
pub const USE_COPY_TRUE: u32 = 1<<17;
pub const AUTO_WIDTH_SHIFT: u32 = 18;
pub const INITIAL_ROW_SHIFT: u32 = 20;
pub const GROUP_TITLE_TRUE: u32 = 1<<22;
pub const GROUP_TITLE_FALSE: u32 = 1<<23;
pub const GROUP_TITLE_V59: u32 = 1<<24;
pub const HIERARCHY_NONE: u32 = 1<<25;

/// Дописанные поля со значением «свойство 8.5»: (как в [`TAILS`]; значение; флаг). Формы конструктора 8.5 (режимы
/// «Такси» и «Версия 8.5» одинаково) и опыт ibcmd import → save с заданными в XML свойствами: Group 0–2 и
/// RowSelectionMode 0/1 стоят и на прежнем месте, «не задано» — только в хвосте (Group 4, RowSelectionMode 2);
/// AutoEditMode — хвост поля 2 (3 — прежнее место, 2 — не задано; XML false загружается как 3 и выгружается true);
/// хвост таблицы 11 = 1 — ShowCommandBar по CommandBarLocation (None false, по умолчанию auto, Top true);
/// WindowOpeningMode «не задано» — хвост 4 = 3 (формы списка, произвольная и документа конструктора 8.5 28.10.2026:
/// платформа не пишет WindowOpeningMode, прежнее место — 0); у страницы Group «не задано» — хвост 0 = 4,
/// ScrollOnCompress — хвост 1 = 2 (форма элемента справочника с двумя табличными частями, как у загрузки `imp/form/items.rs` `page`).
const FLAGS: &[(&str, &str, &str, u8, usize, &str, u32)] = &[
    ("rec", "4", "59", b'A', 7, "4", UNSET_GROUP),
    ("rec", "4", "59", b'A', 4, "3", UNSET_WINDOW_MODE),
    ("cd5394d0", "22", "22", b'L', 0, "4", UNSET_GROUP),
    ("cd5394d0", "22", "22", b'L', 1, "2", UNSET_SCROLL),
    // обычная группа, созданная Конфигуратором 8.5 (эталон 28.10.2026): Representation и Group не заданы — хвост 6 и 7 = 4
    ("cd5394d0", "22", "38", b'A', 6, "4", UNSET_REPRESENTATION),
    ("cd5394d0", "22", "38", b'A', 7, "4", UNSET_GROUP),
    ("143c00f7", "", "73", b'L', 17, "2", UNSET_ROW_SELECTION),
    ("143c00f7", "", "73", b'L', 11, "1", SHOW_COMMAND_BAR),
    ("143c00f7", "", "73", b'L', 2, "1", HIERARCHY_NONE),
    ("77ffcc29", "", "48", b'L', 2, "2", UNSET_AUTO_EDIT),
];

const ID: &str = "02023637-7868-4a5f-8576-835a76e0c9ba";

/// (тип элемента — 8 знаков GUID, "-" без GUID, "rec" вне элементов; версия родителя, "" у самого элемента;
/// версия узла; вид поля [1]: A атом, S строка, L список) → (прежняя версия, сколько полей отбросить с конца,
/// длина узла — где по ключу не различить узлы, 0 — любая).
const RULES: &[(&str, &str, &str, u8, &str, usize, usize)] = &[
    ("rec", "4", "59", b'A', "50", 12, 0),
    ("rec", "0", "11", b'L', "9", 2, 0),
    ("77ffcc29", "", "48", b'L', "37", 15, 0),
    ("77ffcc29", "48", "38", b'L', "36", 5, 0),
    ("77ffcc29", "48", "12", b'A', "11", 1, 21),
    ("77ffcc29", "48", "13", b'A', "11", 2, 0),
    ("77ffcc29", "48", "11", b'L', "8", 3, 0),
    ("77ffcc29", "48", "15", b'A', "13", 2, 0),
    ("77ffcc29", "#", "1", b'A', "0", 1, 0),
    ("a9f3b1ac", "", "34", b'L', "31", 7, 0),
    ("143c00f7", "", "73", b'L', "55", 28, 0),
    ("c5259a1d", "", "6", b'L', "5", 1, 0),
    ("-", "", "6", b'L', "5", 1, 0),
    ("-", "22", "1", b'A', "0", 1, 4),
    ("cd5394d0", "22", "38", b'A', "29", 13, 0),
    ("cd5394d0", "22", "22", b'L', "18", 4, 0),
    ("cd5394d0", "22", "5", b'A', "2", 4, 0),
    ("cd5394d0", "22", "2", b'A', "1", 1, 4),
    ("cd5394d0", "22", "8", b'L', "7", 1, 0),
    ("3d3cb80c", "12", "6", b'L', "4", 2, 0),
];

/// Отброшенные поля: (тип элемента, версия родителя, версия узла, вид поля [1] — как в [`RULES`]; номер поля среди
/// отброшенных; значения в виде [`norm`], при которых выгрузка платформы от поля не зависит — встречены в формах каркаса,
/// совпавших с ней байт в байт). Другое значение — отказ формы: свойство 8.5, которое прежняя версия не выражает.
/// Панели и элементы дополнения таблицы с id `{0}` (`{22,{0},…}`, `{6,{0},…,"S"}`) — id 0 после загрузки XML (`imp/form/fields.rs` `rows_panel`), сверено
/// выгрузкой БП, загруженной платформой из XML (`cfkit-rtbp4\ref.cf`).
const TAILS: &[(&str, &str, &str, u8, usize, &[&str])] = &[
    ("-", "", "6", b'L', 0, &["\"\"", "\"S\""]),
    ("-", "22", "1", b'A', 0, &["0"]),
    ("143c00f7", "", "73", b'L', 0, &["0", "1", "2"]),
    ("143c00f7", "", "73", b'L', 1, &["0", "1", "2"]),
    ("143c00f7", "", "73", b'L', 2, &["0"]),
    ("143c00f7", "", "73", b'L', 3, &["0"]),
    ("143c00f7", "", "73", b'L', 4, &["\"\""]),
    ("143c00f7", "", "73", b'L', 5, &["\"\""]),
    ("143c00f7", "", "73", b'L', 6, &["0", "1", "2"]),
    ("143c00f7", "", "73", b'L', 7, &["1"]),
    ("143c00f7", "", "73", b'L', 8, &["{22,{ID},1,0,0,10,\"S\",{1,0},{1,0},0,1,0,0,0,2,2,{3,4,{0}},{7,3,0,1,100},{0,0,0},1,{0,1,0,1},0,1,0,0,0,3,3,0}", "{22,{ID},1,0,0,10,\"S\",{T,0},{T,0},0,1,0,0,0,2,2,{3,4,{0}},{7,3,0,1,100},{0,0,0},1,{0,1,0,1},0,1,0,0,0,3,3,0}", "{22,{0},1,0,0,10,\"S\",{1,0},{1,0},0,1,0,0,0,2,2,{3,4,{0}},{7,3,0,1,100},{0,0,0},1,{0,1,0,1},0,1,0,0,0,3,3,0}", "{22,{0},1,0,0,10,\"S\",{T,0},{T,0},0,1,0,0,0,2,2,{3,4,{0}},{7,3,0,1,100},{0,0,0},1,{0,1,0,1},0,1,0,0,0,3,3,0}"]),
    ("143c00f7", "", "73", b'L', 9, &["0"]),
    ("143c00f7", "", "73", b'L', 10, &["0", "1", "2"]),
    ("143c00f7", "", "73", b'L', 11, &["0"]),
    ("143c00f7", "", "73", b'L', 12, &["0", "1", "2"]),
    ("143c00f7", "", "73", b'L', 13, &["1"]),
    ("143c00f7", "", "73", b'L', 14, &["{22,{ID},1,0,0,11,\"S\",{1,0},{1,0},0,1,0,0,0,2,2,{3,4,{0}},{7,3,0,1,100},{0,0,0},1,{0,1,0,1},0,1,0,0,0,3,3,0}", "{22,{ID},1,0,0,11,\"S\",{T,0},{T,0},0,1,0,0,0,2,2,{3,4,{0}},{7,3,0,1,100},{0,0,0},1,{0,1,0,1},0,1,0,0,0,3,3,0}", "{22,{0},1,0,0,11,\"S\",{1,0},{1,0},0,1,0,0,0,2,2,{3,4,{0}},{7,3,0,1,100},{0,0,0},1,{0,1,0,1},0,1,0,0,0,3,3,0}", "{22,{0},1,0,0,11,\"S\",{T,0},{T,0},0,1,0,0,0,2,2,{3,4,{0}},{7,3,0,1,100},{0,0,0},1,{0,1,0,1},0,1,0,0,0,3,3,0}"]),
    ("143c00f7", "", "73", b'L', 15, &["2"]),
    ("143c00f7", "", "73", b'L', 16, &["0"]),
    ("143c00f7", "", "73", b'L', 17, &["0", "1"]),
    ("143c00f7", "", "73", b'L', 18, &["1"]),
    ("143c00f7", "", "73", b'L', 19, &["0"]),
    ("143c00f7", "", "73", b'L', 20, &["0"]),
    ("143c00f7", "", "73", b'L', 21, &["0"]),
    ("143c00f7", "", "73", b'L', 22, &["0"]),
    ("143c00f7", "", "73", b'L', 23, &["0"]),
    ("143c00f7", "", "73", b'L', 24, &["1"]),
    ("143c00f7", "", "73", b'L', 25, &["{5,{ID},0,0,0,0,\"S\",{1,0},{1,0},1,1,0,1,{1,0,2,{3,4,{0}},{3,4,{0}},{3,4,{0}},{7,3,0,1,100},{0,1,0},1,0,0},1,{22,{ID},0,0,0,8,\"S\",{1,0},{1,0},0,1,0,0,0,2,2,{3,4,{0}},{7,3,0,1,100},{0,0,0},1,{1,1},0,1,0,0,0,3,3,0},1,{12,{ID},0,0,0,0,\"S\",{1,0},{1,0},1,0,0,2,2,{3,4,{0}},{7,3,0,1,100},{0,0,0},1,{5,0,0,3,0,{0,1,0},{3,4,{0}},{3,4,{0}},{3,0,{0},0,1,0,48312c09-257f-4b29-b280-284dd89efc1e}},0,1,2,{1,{1,0},0},0,0,1,0,0,1,0,3,3,0,0},2,{T,0},0,3,3,0}", "{5,{ID},0,0,0,0,\"S\",{T,0},{T,0},1,1,0,1,{1,0,2,{3,4,{0}},{3,4,{0}},{3,4,{0}},{7,3,0,1,100},{0,1,0},1,0,0},1,{22,{ID},0,0,0,8,\"S\",{T,0},{T,0},0,1,0,0,0,2,2,{3,4,{0}},{7,3,0,1,100},{0,0,0},1,{T,1},0,1,0,0,0,3,3,0},1,{12,{ID},0,0,0,0,\"S\",{T,0},{T,0},1,0,0,2,2,{3,4,{0}},{7,3,0,1,100},{0,0,0},1,{5,0,0,3,0,{0,1,0},{3,4,{0}},{3,4,{0}},{3,0,{0},0,1,0,48312c09-257f-4b29-b280-284dd89efc1e}},0,1,2,{1,{T,0},0},0,0,1,0,0,1,0,3,3,0,0},2,{T,0},0,3,3,0}", "{6,{0},0,0,0,0,\"S\",{1,0},{1,0},1,1,0,1,{1,0,2,{3,4,{0}},{3,4,{0}},{3,4,{0}},{7,3,0,1,100},{0,1,0},1,0,0},1,{22,{0},0,0,0,8,\"S\",{1,0},{1,0},0,1,0,0,0,2,2,{3,4,{0}},{7,3,0,1,100},{0,0,0},1,{1,1},0,1,0,0,0,3,3,0},1,{12,{0},0,0,0,0,\"S\",{1,0},{1,0},1,0,0,2,2,{3,4,{0}},{7,3,0,1,100},{0,0,0},1,{5,0,0,3,0,{0,1,0},{3,4,{0}},{3,4,{0}},{3,0,{0},0,1,0,48312c09-257f-4b29-b280-284dd89efc1e}},0,1,2,{1,{1,0},0},0,0,1,0,0,1,0,3,3,0,0},2,{T,0},0,3,3,0,\"S\"}", "{6,{0},0,0,0,0,\"S\",{T,0},{T,0},1,1,0,1,{1,0,2,{3,4,{0}},{3,4,{0}},{3,4,{0}},{7,3,0,1,100},{0,1,0},1,0,0},1,{22,{0},0,0,0,8,\"S\",{T,0},{T,0},0,1,0,0,0,2,2,{3,4,{0}},{7,3,0,1,100},{0,0,0},1,{T,1},0,1,0,0,0,3,3,0},1,{12,{0},0,0,0,0,\"S\",{T,0},{T,0},1,0,0,2,2,{3,4,{0}},{7,3,0,1,100},{0,0,0},1,{5,0,0,3,0,{0,1,0},{3,4,{0}},{3,4,{0}},{3,0,{0},0,1,0,48312c09-257f-4b29-b280-284dd89efc1e}},0,1,2,{1,{T,0},0},0,0,1,0,0,1,0,3,3,0,0},2,{T,0},0,3,3,0,\"S\"}"]),
    ("143c00f7", "", "73", b'L', 26, &["1"]),
    ("143c00f7", "", "73", b'L', 27, &["{5,{ID},0,0,0,2,\"S\",{1,0},{1,0},1,1,0,1,{1,0,{3,4,{0}},{3,4,{0}},{3,4,{0}},{7,3,0,1,100},{0,1,0},1,0,0,2},1,{22,{ID},0,0,0,8,\"S\",{1,0},{1,0},0,1,0,0,0,2,2,{3,4,{0}},{7,3,0,1,100},{0,0,0},1,{1,1},0,1,0,0,0,3,3,0},1,{12,{ID},0,0,0,0,\"S\",{1,0},{1,0},1,0,0,2,2,{3,4,{0}},{7,3,0,1,100},{0,0,0},1,{5,0,0,3,0,{0,1,0},{3,4,{0}},{3,4,{0}},{3,0,{0},0,1,0,48312c09-257f-4b29-b280-284dd89efc1e}},0,1,2,{1,{1,0},0},0,0,1,0,0,1,0,3,3,0,0},2,{T,2},0,3,3,0}", "{5,{ID},0,0,0,2,\"S\",{T,0},{T,0},1,1,0,1,{1,0,{3,4,{0}},{3,4,{0}},{3,4,{0}},{7,3,0,1,100},{0,1,0},1,0,0,2},1,{22,{ID},0,0,0,8,\"S\",{T,0},{T,0},0,1,0,0,0,2,2,{3,4,{0}},{7,3,0,1,100},{0,0,0},1,{T,1},0,1,0,0,0,3,3,0},1,{12,{ID},0,0,0,0,\"S\",{T,0},{T,0},1,0,0,2,2,{3,4,{0}},{7,3,0,1,100},{0,0,0},1,{5,0,0,3,0,{0,1,0},{3,4,{0}},{3,4,{0}},{3,0,{0},0,1,0,48312c09-257f-4b29-b280-284dd89efc1e}},0,1,2,{1,{T,0},0},0,0,1,0,0,1,0,3,3,0,0},2,{T,2},0,3,3,0}", "{6,{0},0,0,0,2,\"S\",{1,0},{1,0},1,1,0,1,{1,0,{3,4,{0}},{3,4,{0}},{3,4,{0}},{7,3,0,1,100},{0,1,0},1,0,0,2},1,{22,{0},0,0,0,8,\"S\",{1,0},{1,0},0,1,0,0,0,2,2,{3,4,{0}},{7,3,0,1,100},{0,0,0},1,{1,1},0,1,0,0,0,3,3,0},1,{12,{0},0,0,0,0,\"S\",{1,0},{1,0},1,0,0,2,2,{3,4,{0}},{7,3,0,1,100},{0,0,0},1,{5,0,0,3,0,{0,1,0},{3,4,{0}},{3,4,{0}},{3,0,{0},0,1,0,48312c09-257f-4b29-b280-284dd89efc1e}},0,1,2,{1,{1,0},0},0,0,1,0,0,1,0,3,3,0,0},2,{T,2},0,3,3,0,\"S\"}", "{6,{0},0,0,0,2,\"S\",{T,0},{T,0},1,1,0,1,{1,0,{3,4,{0}},{3,4,{0}},{3,4,{0}},{7,3,0,1,100},{0,1,0},1,0,0,2},1,{22,{0},0,0,0,8,\"S\",{T,0},{T,0},0,1,0,0,0,2,2,{3,4,{0}},{7,3,0,1,100},{0,0,0},1,{T,1},0,1,0,0,0,3,3,0},1,{12,{0},0,0,0,0,\"S\",{T,0},{T,0},1,0,0,2,2,{3,4,{0}},{7,3,0,1,100},{0,0,0},1,{5,0,0,3,0,{0,1,0},{3,4,{0}},{3,4,{0}},{3,0,{0},0,1,0,48312c09-257f-4b29-b280-284dd89efc1e}},0,1,2,{1,{T,0},0},0,0,1,0,0,1,0,3,3,0,0},2,{T,2},0,3,3,0,\"S\"}"]),
    ("3d3cb80c", "12", "6", b'L', 0, &["0"]),
    ("3d3cb80c", "12", "6", b'L', 1, &["{3,4,{0}}"]),
    ("77ffcc29", "", "48", b'L', 0, &["0", "1", "2"]),
    ("77ffcc29", "", "48", b'L', 1, &["0"]),
    ("77ffcc29", "", "48", b'L', 2, &["0", "1", "3"]),
    ("77ffcc29", "", "48", b'L', 3, &["1"]),
    ("77ffcc29", "", "48", b'L', 4, &["{22,{ID},1,0,0,10,\"S\",{1,0},{1,0},0,1,0,0,0,2,2,{3,4,{0}},{7,3,0,1,100},{0,0,0},1,{0,1,0,1},0,1,0,0,0,3,3,0}", "{22,{ID},1,0,0,10,\"S\",{T,0},{T,0},0,1,0,0,0,2,2,{3,4,{0}},{7,3,0,1,100},{0,0,0},1,{0,1,0,1},0,1,0,0,0,3,3,0}", "{22,{0},1,0,0,10,\"S\",{1,0},{1,0},0,1,0,0,0,2,2,{3,4,{0}},{7,3,0,1,100},{0,0,0},1,{0,1,0,1},0,1,0,0,0,3,3,0}", "{22,{0},1,0,0,10,\"S\",{T,0},{T,0},0,1,0,0,0,2,2,{3,4,{0}},{7,3,0,1,100},{0,0,0},1,{0,1,0,1},0,1,0,0,0,3,3,0}"]),
    ("77ffcc29", "", "48", b'L', 5, &["0"]),
    ("77ffcc29", "", "48", b'L', 6, &["0"]),
    ("77ffcc29", "", "48", b'L', 7, &["2"]),
    ("77ffcc29", "", "48", b'L', 8, &["0"]),
    ("77ffcc29", "", "48", b'L', 9, &["0"]),
    ("77ffcc29", "", "48", b'L', 10, &["1"]),
    ("77ffcc29", "", "48", b'L', 11, &["2"]),
    ("77ffcc29", "", "48", b'L', 12, &["0", "1", "2", "3"]),
    ("77ffcc29", "", "48", b'L', 13, &["0", "1"]),
    ("77ffcc29", "", "48", b'L', 14, &["0", "1"]),
    ("77ffcc29", "#", "1", b'A', 0, &["{4,0,{0},\"\",-1,-1,1,0,\"\"}"]),
    ("77ffcc29", "48", "11", b'L', 0, &["0"]),
    ("77ffcc29", "48", "11", b'L', 1, &["1"]),
    ("77ffcc29", "48", "11", b'L', 2, &["2"]),
    ("77ffcc29", "48", "12", b'A', 0, &["0", "1", "2"]),
    ("77ffcc29", "48", "13", b'A', 0, &["0"]),
    ("77ffcc29", "48", "13", b'A', 1, &["2"]),
    ("77ffcc29", "48", "15", b'A', 0, &["0"]),
    ("77ffcc29", "48", "15", b'A', 1, &["0"]),
    ("77ffcc29", "48", "38", b'L', 0, &["{1,0}"]),
    ("77ffcc29", "48", "38", b'L', 1, &["{4,0,{0},\"\",-1,-1,1,0,\"\"}"]),
    ("77ffcc29", "48", "38", b'L', 2, &["{1,0}"]),
    ("77ffcc29", "48", "38", b'L', 3, &["1"]),
    ("77ffcc29", "48", "38", b'L', 4, &["0"]),
    ("a9f3b1ac", "", "34", b'L', 0, &["0"]),
    ("a9f3b1ac", "", "34", b'L', 1, &["{4,0,{0},\"\",-1,-1,1,0,\"\"}"]),
    ("a9f3b1ac", "", "34", b'L', 2, &["0"]),
    ("a9f3b1ac", "", "34", b'L', 3, &["0"]),
    ("a9f3b1ac", "", "34", b'L', 4, &["0"]),
    ("a9f3b1ac", "", "34", b'L', 5, &["0", "1", "2"]),
    ("a9f3b1ac", "", "34", b'L', 6, &["\"\""]),
    ("c5259a1d", "", "6", b'L', 0, &["\"\""]),
    ("cd5394d0", "22", "2", b'A', 0, &["0"]),
    ("cd5394d0", "22", "22", b'L', 0, &["0", "1", "2", "3"]),
    ("cd5394d0", "22", "22", b'L', 1, &["0", "1"]),
    ("cd5394d0", "22", "22", b'L', 2, &["0"]),
    ("cd5394d0", "22", "22", b'L', 3, &["0", "1", "2"]),
    ("cd5394d0", "22", "38", b'A', 0, &["0"]),
    ("cd5394d0", "22", "38", b'A', 1, &["0"]),
    ("cd5394d0", "22", "38", b'A', 2, &["0"]),
    ("cd5394d0", "22", "38", b'A', 3, &["{4,0,{0},\"\",-1,-1,1,0,\"\"}"]),
    ("cd5394d0", "22", "38", b'A', 4, &["{0,1,0}"]),
    ("cd5394d0", "22", "38", b'A', 5, &["0"]),
    ("cd5394d0", "22", "38", b'A', 6, &["0", "1", "2", "3"]),
    ("cd5394d0", "22", "38", b'A', 7, &["0", "1", "2", "3"]),
    ("cd5394d0", "22", "38", b'A', 8, &["0"]),
    ("cd5394d0", "22", "38", b'A', 9, &["2"]),
    ("cd5394d0", "22", "38", b'A', 10, &["0"]),
    ("cd5394d0", "22", "38", b'A', 11, &["0"]),
    ("cd5394d0", "22", "38", b'A', 12, &["0", "1", "2"]),
    ("cd5394d0", "22", "5", b'A', 0, &["1"]),
    ("cd5394d0", "22", "5", b'A', 1, &["2"]),
    ("cd5394d0", "22", "5", b'A', 2, &["2"]),
    ("cd5394d0", "22", "5", b'A', 3, &["0", "1", "2"]),
    ("cd5394d0", "22", "8", b'L', 0, &["1"]),
    ("rec", "0", "11", b'L', 0, &["0"]),
    ("rec", "0", "11", b'L', 1, &["0"]),
    ("rec", "4", "59", b'A', 0, &["{1,0}"]),
    ("rec", "4", "59", b'A', 1, &["{4,0,{0},\"\",-1,-1,1,0,\"\"}"]),
    ("rec", "4", "59", b'A', 2, &["0", "1"]),
    ("rec", "4", "59", b'A', 3, &["0", "1"]),
    ("rec", "4", "59", b'A', 4, &["0", "1", "2"]),
    ("rec", "4", "59", b'A', 5, &["0"]),
    ("rec", "4", "59", b'A', 6, &["0", "1", "2"]),
    ("rec", "4", "59", b'A', 7, &["0", "1", "2", "3"]),
    ("rec", "4", "59", b'A', 8, &["0"]),
    ("rec", "4", "59", b'A', 9, &["0", "1"]),
    ("rec", "4", "59", b'A', 10, &["0", "2"]),
    ("rec", "4", "59", b'A', 11, &["0"]),
];

/// Версия корня дерева формы.
pub fn is_v59(root: &Node) -> bool {
    root.get(1).and_then(|f| f.get(0)).and_then(Node::atom) == Some("59")
}

/// Ошибка — значение, которое прежняя версия не выражает (выгрузка формы отказывает, а не пишет неверное).
pub fn downgrade(root: &mut Node) -> Result<HashMap<String, u32>, String> {
    let mut tails = Vec::new();
    let mut flags = HashMap::new();
    // Узлы `{22,{-k,ID},…}` (k ≥ 2) в конце корня — свои панели редактора форм 8.5 (форма, правленная Конфигуратором,
    // эталон 28.10.2026: -2, -3, -4 за признаком 1 в хвостовых полях 2, 3, 9); в XML их нет — отбрасываются.
    if let Some(form) = root.list_mut().and_then(|v| v.get_mut(1)).and_then(Node::list_mut) {
        form.retain(|n| !n.get(1).is_some_and(|i| i.list().len() == 2 && i.get(1).and_then(Node::atom) == Some(ID) && i.get(0).and_then(Node::atom).and_then(|k| k.parse::<i64>().ok()).is_some_and(|k| k <= -2)));
    }
    walk(root, "rec", "", "", &mut tails, &mut flags)?;
    if std::env::var_os("CFKIT_V59_TAILS").is_some() {
        return Err(format!("TAILS{}", tails.join("\u{2}")));
    }
    Ok(flags)
}

/// Значение отброшенного поля для сверки с [`TAILS`]: id элемента — `{ID}`, непустая строка — `"S"`,
/// ссылка на сам узел `{id,вид}` (источник дополнения таблицы) — `{T,вид}`.
fn norm(n: &Node, owner: &str, out: &mut String) {
    match n {
        Node::Atom(a) => out.push_str(a),
        Node::Str(s) => out.push_str(if s.is_empty() { "\"\"" } else { "\"S\"" }),
        Node::List(v) if v.len() == 2 && v[1].atom() == Some(ID) => out.push_str("{ID}"),
        Node::List(v) => {
            out.push('{');
            for (i, x) in v.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                if i == 0 && v.len() == 2 && !owner.is_empty() && x.atom() == Some(owner) {
                    out.push('T');
                } else {
                    norm(x, owner, out);
                }
            }
            out.push('}');
        }
    }
}

fn head(n: &Node) -> String {
    match n.get(0) {
        Some(Node::Atom(a)) => a.clone(),
        Some(Node::Str(s)) => s.clone(),
        _ => String::new(),
    }
}

fn is_element(n: &Node) -> bool {
    n.list().len() > 2 && n.get(1).is_some_and(|i| i.list().len() == 2 && i.get(1).and_then(Node::atom) == Some(ID))
}

/// Список с числом в начале — не версия: путь к данным `{N,{k}|{k,guid}×N}` (11 частей принимался за узел 48/11),
/// исключенные команды `{N,guid×N}` (15 команд — за узел 48/15).
fn is_counted(v: &[Node]) -> bool {
    let n = v[0].atom().and_then(|a| a.parse::<usize>().ok()).unwrap_or(0);
    n > 0 && v.len() == n + 1 && (v[1..].iter().all(|p| matches!(p.list(), [k] | [k, _] if k.atom().is_some()) && p.list().iter().all(|x| x.atom().is_some()))
            || v[1..].iter().all(|g| g.atom().is_some_and(crate::brace::is_guid)))
}

/// Цвет `{4,вид,значение,вид}` → (`3`, отбросить 1) и шрифт `{8,…}` → (`7`, 0 или 1) — версии 8.5 и прежние.
fn style(v: &[Node]) -> Result<Option<(&'static str, usize)>, String> {
    let own = v.first().and_then(Node::atom).unwrap_or("");
    Ok(if v.len() == 4 && own == "4" && matches!(v[2], Node::List(_)) && v[1].atom().is_some() && v[1] == v[3] {
        Some(("3", 1)) // цвет: поле 3 повторяет 1
    } else if own == "8" && v.len() >= 5 && matches!(v[1].atom(), Some("1" | "2" | "3")) && v[v.len() - 2].atom() == Some("1") {
        Some(("7", 0)) // шрифт
    } else if own == "8" && v.len() == 20 && v[1].atom() == Some("0") && v[17].atom() == Some("1") {
        // Абсолютный шрифт: 8.5 дописывает поле после масштаба.
        if v[19].atom() != Some("0") {
            return Err(format!("версия 8.5: шрифт, поле 19 {:?}", v[19]));
        }
        Some(("7", 1))
    } else {
        None
    })
}

/// Цвет или шрифт 8.5 → прежняя версия; `None` — узел не цвет 8.5 и не шрифт 8.5 (как есть).
pub fn style_node(n: &Node) -> Result<Option<Node>, String> {
    let v = n.list();
    Ok(style(v)?.map(|(new, cut)| {
        let mut w = v[..v.len() - cut].to_vec();
        w[0] = Node::Atom(new.to_string());
        Node::List(w)
    }))
}

/// Цвета и шрифты 8.5 во всех узлах ниже `n` (не в нем самом) — в прежние версии: схемы, диаграммы, сохраненные 8.5
/// (у них прочие узлы прежние). Для табличного документа не годится: узлы объединений похожи на шрифт.
pub fn styles(n: &mut Node) -> Result<(), String> {
    for c in n.list_mut().into_iter().flatten() {
        if !matches!(c, Node::List(_)) {
            continue;
        }
        styles(c)?;
        let v = c.list_mut().unwrap();
        if let Some((new, cut)) = style(v)? {
            v.truncate(v.len() - cut);
            v[0] = Node::Atom(new.to_string());
        }
    }
    Ok(())
}

/// `elem` — id элемента, в котором узел (ключ флагов вложенных узлов).
fn walk(n: &mut Node, etype: &str, parent: &str, elem: &str, tails: &mut Vec<String>, flags: &mut HashMap<String, u32>) -> Result<(), String> {
    let own = head(n);
    let shape = match n.get(1) {
        Some(Node::Atom(_)) => b'A',
        Some(Node::Str(_)) => b'S',
        Some(Node::List(_)) => b'L',
        None => b'-',
    };
    let owner = if is_element(n) { n.get(1).and_then(|i| i.get(0)).and_then(Node::atom).unwrap_or("").to_string() } else { String::new() };
    let Some(v) = n.list_mut() else { return Ok(()) };
    if v.len() == 2 && v[1].atom() == Some(ID) {
        return Ok(()); // id элемента {N,02023637-...}: не узел с версией
    }
    // Дети — по исходным версиям: правило родителя меняет его версию уже после.
    let mut prev: Option<String> = None;
    for c in v.iter_mut() {
        if is_element(c) {
            // Embedded chart tables have an element id but no preceding class GUID.
            let t = prev.as_deref().filter(|p| p.len() == 36).map_or_else(|| if head(c)=="73" {"143c00f7"} else {"-"}, |p| &p[..8]).to_string();
            walk(c, &t, "", "", tails, flags)?;
        } else if matches!(c, Node::List(_)) {
            walk(c, etype, &own, if owner.is_empty() { elem } else { &owner }, tails, flags)?;
        }
        prev = c.atom().map(str::to_string);
    }
    let (new, cut, rule) = if let Some((new, cut)) = style(v)? {
        (new, cut, false)
    } else if is_counted(v) {
        return Ok(());
    } else if let Some(r) = RULES.iter().find(|r| r.0 == etype && r.1 == parent && r.2 == own && r.3 == shape && (r.6 == 0 || r.6 == v.len())) {
        (r.4, r.5, true)
    } else {
        return Ok(());
    };
    if cut >= v.len() {
        return Ok(());
    }
    if etype == "143c00f7" && own == "73" {
        // Таблица: VerticalLinesBWA (33, за флагом видимости по ролям в 4 — 34) 8.5 держит в дописанном поле 1
        // как 0/2, а в 33 пишет 0 (сверка 4 466 форм каркаса с БП).
        let extra = usize::from(v[4].atom() == Some("1"));
        let tail = v.len() - cut;
        let activation:u32=v[tail+6].atom().and_then(|v|v.parse().ok()).filter(|v|*v<=2).ok_or("неизвестное InitialRowActivation")?;
        *flags.entry(owner.clone()).or_default()|=activation<<INITIAL_ROW_SHIFT;
        // The v59 writer stores all three properties in the tail (see
        // imp/form/table.rs). 0=false, 1=true, 2=auto; preserve the distinction
        // between explicit true and auto, absent in the old layout.
        for (old,new,no,yes) in [(32,0,HLINES_FALSE,HLINES_TRUE),(33,1,VLINES_FALSE,VLINES_TRUE),(36,10,ALTERNATION_FALSE,ALTERNATION_TRUE)] {
            let flag=match v[tail+new].atom() {Some("0")=>no,Some("1")=>yes,Some("2")=>0,_=>return Err(format!("неизвестное булево поле таблицы {new}"))};
            *flags.entry(owner.clone()).or_default()|=flag;
            v[old+extra]=match v[tail+new].atom() {
                Some("0")=>Node::Atom("0".into()),
                Some("1"|"2")=>Node::Atom("1".into()),
                _=>return Err(format!("версия формы 59: таблица, булево поле хвоста {new}: {:?}",v[tail+new]))
            };
        }
    }
    let tail = v.len() - cut;
    let input_picture=(etype=="77ffcc29" && parent=="48" && own=="38" && shape==b'L').then(||v[tail+1].clone());
    let radio_extra=if etype=="77ffcc29" && parent=="48" && own=="11" && shape==b'L' {
        let valid=|n:&Node,max:u32|n.atom().and_then(|a|a.parse::<u32>().ok()).is_some_and(|a|a<=max);
        if !valid(&v[tail],2) || !valid(&v[tail+1],3) || !valid(&v[tail+2],2) {return Err("неизвестные свойства переключателя".into());}
        let extra=vec![v[tail].clone(),v[tail+1].clone()];
        v[11]=v[tail+2].clone();
        Some(extra)
    } else {None};
    if etype == "77ffcc29" && parent.is_empty() && own == "48" {
        let mark=match v[tail].atom() {
            Some("0")=>MARK_REQUIRED_FALSE,Some("1")=>MARK_REQUIRED_TRUE,Some("2")=>0,
            _=>return Err("версия формы 59: неизвестное MarkRequiredComplete".into())
        };
        *flags.entry(owner.clone()).or_default()|=FIELD_V59|mark;
        let width:u32=v[tail+12].atom().and_then(|v|v.parse().ok()).filter(|v|*v<=3).ok_or("неизвестное AutoWidthInTable")?;
        *flags.entry(owner.clone()).or_default()|=width<<AUTO_WIDTH_SHIFT;
    }
    if etype=="77ffcc29" && parent=="48" && own=="12" && shape==b'A' {
        let value=match v[tail].atom() {Some("0")=>USE_COPY_FALSE,Some("1")=>USE_COPY_TRUE,Some("2")=>0,_=>return Err("неизвестное UseCopy".into())};
        *flags.entry(elem.to_string()).or_default()|=value;
    }
    if etype=="cd5394d0" && parent=="22" && (own=="38" && shape==b'A' || own=="22" && shape==b'L' || own=="5" && shape==b'A') {
        let at=if own=="38" {12} else {3};
        let mark=match v[tail+at].atom() {Some("0")=>GROUP_TITLE_FALSE,Some("1")=>GROUP_TITLE_TRUE,Some("2")=>0,_=>return Err("неизвестное ShowTitle группы".into())};
        *flags.entry(elem.to_string()).or_default()|=GROUP_TITLE_V59|mark;
    }
    // Картинка-декорация: PictureColor 8.5 пишет в дописанное поле 1 свойств вида (поле 14 элемента у нее — TextColor);
    // оставляем его полем 13 свойств — `form.rs` читает его оттуда.
    let picture_color = (etype == "3d3cb80c" && parent == "12" && own == "6").then(|| v[tail + 1].clone());
    if rule {
        for (i, x) in v[tail..].iter().enumerate() {
            if input_picture.is_some() && i==1 {continue;}
            if radio_extra.is_some() {continue;}
            if i == 1 && picture_color.is_some() {
                continue;
            }
            let mut val = String::new();
            norm(x, &owner, &mut val);
            let flag = FLAGS.iter().find(|f| f.0 == etype && f.1 == parent && f.2 == own && f.3 == shape && f.4 == i && f.5 == val).map(|f| f.6);
            if let Some(f) = flag {
                *flags.entry(if etype == "rec" { "rec".to_string() } else if owner.is_empty() { elem.to_string() } else { owner.clone() }).or_default() |= f;
            }
            let ok = flag.is_some() || TAILS.iter().any(|t| t.0 == etype && t.1 == parent && t.2 == own && t.3 == shape && t.4 == i && t.5.contains(&val.as_str()));
            if std::env::var_os("CFKIT_V59_TAILS").is_some() {
                tails.push(format!("{etype}|{parent}|{own}|{}|{i}|{val}", shape as char));
            } else if !ok {
                let short: String = val.chars().take(160).collect();
                return Err(format!("версия формы 59: узел {etype} {parent}/{own}, отброшенное поле {i} = {short} (значение не сверено с выгрузкой платформы)"));
            }
        }
    }
    v.truncate(v.len() - cut);
    if let Some(picture)=input_picture {v.push(picture);}
    if let Some(extra)=radio_extra {v.extend(extra);}
    v.extend(picture_color);
    v[0] = Node::Atom(new.to_string());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::brace;

    const ROOT: &str = "{4,{59,1,{4,3,{-3},3},{8,3,0,1,100},{4,0,{0},\"\",-1,-1,1,0,\"\"},{1,0},{4,0,{0},\"\",-1,-1,1,0,\"\"},0,0,0,0,0,0,0,0,0,0}}";

    #[test]
    fn colors_fonts_root() {
        let mut n = brace::parse(ROOT).unwrap();
        downgrade(&mut n).unwrap();
        assert_eq!(brace::write(&n).unwrap().replace("\r\n", ""), "{4,{50,1,{3,3,{-3}},{7,3,0,1,100},{4,0,{0},\"\",-1,-1,1,0,\"\"}}}");
    }

    #[test]
    fn unset_group_flag() {
        let mut n = brace::parse(&ROOT.replace(",0,0,0,0,0,0,0,0,0,0}}", ",0,0,0,0,0,4,0,0,0,0}}")).unwrap();
        assert_eq!(downgrade(&mut n).unwrap().get("rec"), Some(&UNSET_GROUP));
        let mut n = brace::parse(ROOT).unwrap();
        assert!(downgrade(&mut n).unwrap().is_empty());
    }

    #[test]
    fn counted_lists_kept() {
        assert!(is_counted(&brace::parse("{2,{1},{3,5bdad865-f2c5-434b-8041-ba4aad3b6687}}").unwrap().list()));
        assert!(is_counted(&brace::parse("{1,d8e20c4d-3519-49aa-80e5-d6d66fee741a}").unwrap().list()));
        assert!(!is_counted(&brace::parse("{2,0,{1}}").unwrap().list()));
    }

    #[test]
    fn unknown_tail_value_refuses() {
        let mut n = brace::parse(&ROOT.replace(",0}}", ",5}}")).unwrap();
        let e = downgrade(&mut n).unwrap_err();
        assert!(e.contains("rec 4/59, отброшенное поле 11 = 5"), "{e}");
        let mut v = String::new();
        norm(&brace::parse("{5,{7,02023637-7868-4a5f-8576-835a76e0c9ba},\"Имя\",\"\",{7,0}}").unwrap(), "7", &mut v);
        assert_eq!(v, "{5,{ID},\"S\",\"\",{T,0}}");
    }
}
