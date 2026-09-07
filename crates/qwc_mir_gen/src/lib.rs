mod mgen;
mod symb_p;
mod type_p;
mod blok_p;

type MayFail<T> = Result<(), T>;

use mgen::Ctx;
use {symb_p::*, type_p::*, blok_p::*};

pub use mgen::MGen;
