#![feature(never_type)]

mod parse;
mod meta_p;
mod type_p;
mod expr_p;
mod item_p;
mod attr_p;
mod patt_p;

type Fail<T> = Result<!, T>;

use parse::Ctx;
use {item_p::ItemParser, meta_p::{MetaParser, WordCheck}, attr_p::AttrParser, expr_p::ExprParser, type_p::TypeParser, patt_p::PattParser};

pub use parse::Parse;
