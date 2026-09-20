mod mgen;
mod layout;
mod symb_p;
mod type_p;
mod blok_p;
mod expr_p;
mod builder;
mod ty_interner;

type MayFail<T> = Result<(), T>;

use mgen::Ctx;
use {symb_p::*, type_p::*, blok_p::*, expr_p::*};
use builder::*;

pub use mgen::{MGen, CacheMap};
