//! Язык запросов 1С: свой лексер и разборщик (для полей динамических списков со своим запросом).
pub mod dcs;
pub mod lex;
pub mod names;
pub mod parse;
pub mod resolve;
pub mod roles;
pub mod settings;

pub use parse::{parse, Batch, Expr, Field, Join, Query, Select, Source, Table, Type};
