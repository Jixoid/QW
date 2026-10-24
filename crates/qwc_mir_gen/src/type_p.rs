/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use qwc_comptime::{Evaluator, GetConst};
use qwc_diagnostic::{Status::{self, Success}, Summary};
use qwc_hir as hir;
use qwc_mir::{self as mir, PushOkApi};

use crate::{Ctx, Layouter};


pub struct TypeLow;

impl TypeLow {

  pub fn low(ctx: &mut Ctx, sum: &mut Summary, id: hir::TypeId) -> Status<mir::TypeId> {
    if let Some(&id) = ctx.cmap.cache_type.get(&id) { return Success(id) }

    let it = ctx.get(id);

    use hir::TypeKind::*;
    let ty = match it.kind {
      // Primitive
      Unit => ctx.tin.ty_unit(),
      
      Int(len, _) => Self::low_int(ctx, sum, len)?,
      ArchInt(_) => ctx.tin.ty_arch_int(),
      Float(len) => Self::low_float(ctx, sum, len)?,
      Bool => ctx.tin.ty_bool(),
      Char => ctx.tin.ty_i32(),

      Ref(id, _) => Self::low_ref(ctx, sum, id)?,

      // Combinated
      Struct(rng) => Self::low_struct(ctx, sum, rng, id.cid())?,
      ItemStruct{fields: rng, ..} => Self::low_struct(ctx, sum, rng, id.cid())?,
      Tuple(rng) => Self::low_tuple(ctx, sum, rng, id.cid())?,
      ItemTuple{fields: rng, ..} => Self::low_tuple(ctx, sum, rng, id.cid())?,
      
      ItemIface{methods: rng, ..} => Self::low_iface(ctx, sum, rng)?,
      
      TraitFrom{hidden, ..} => Self::low_trait_from(ctx, sum, hidden)?,
      
      ItemEnum{..} => ctx.tin.ty_i32(),
      ItemFlag{..} => ctx.tin.ty_i32(),
      
      Option(inner) => Self::low(ctx, sum, inner)?,
      
      // Sequential
      Array(kind, ext) => Self::low_array(ctx, sum, kind, ext)?,

      // Callable
      Fun{self_kind, args, ret} => Self::low_fun(ctx, sum, self_kind, args, ret, id.cid())?,
      
      // Logic Error
      ItemTrait{..} | AssociatedType{..} | GenericType{..} => panic!("this type must not have infiltrated this layer!"),

      kind @_ => todo!("{kind:#?}")
    };

    ctx.cmap.cache_type.insert(id, ty);

    Success(ty)
  }


  fn low_int(ctx: &mut Ctx, _sum: &mut Summary, len: u16) -> Status<mir::TypeId> {
    let it = match len {
      8   => ctx.tin.ty_i8(),
      16  => ctx.tin.ty_i16(),
      32  => ctx.tin.ty_i32(),
      64  => ctx.tin.ty_i64(),
      128 => ctx.tin.ty_i128(),
      _ => unreachable!()
    };

    Success(it)
  }

  fn low_float(ctx: &mut Ctx, _sum: &mut Summary, len: u16) -> Status<mir::TypeId> {
    let it = match len {
      16  => ctx.tin.ty_f16(),
      32  => ctx.tin.ty_f32(),
      64  => ctx.tin.ty_f64(),
      128 => ctx.tin.ty_f128(),
      _ => unreachable!()
    };

    Success(it)
  }


  fn low_ref(ctx: &mut Ctx, _sum: &mut Summary, id: hir::TypeId) -> Status<mir::TypeId> {
    use hir::TypeKind::*;

    let it = match ctx.get(id).kind {
      Str | Slice(..) => ctx.tin.ty_fatptrint(),
      
      ItemIface{..} => ctx.tin.ty_fatptr(),
    
      _ => ctx.tin.ty_ptr()
    };
    
    Success(it)
  }


  fn low_array(ctx: &mut Ctx, sum: &mut Summary, kind: hir::TypeId, ext: hir::ExprId) -> Status<mir::TypeId> {
    let elem_ty = Self::low(ctx, sum, kind)?;

    let count = Evaluator::new(ctx.src).eval(sum, ext)?.get_u64(sum)?;

    let kind = mir::TypeKind::Array(elem_ty, count);
    mir::Type {
      kind,
      layout: Layouter::layout(&kind, ctx.tin.layinfo, ctx.cre),
    }.push_ok(ctx.cre)
  }


  fn low_struct(ctx: &mut Ctx, sum: &mut Summary, rng: hir::ThingRng, cid: hir::CID) -> Status<mir::TypeId> {
    let kinds: Vec<hir::TypeId> = {
      let krate = ctx.get_krate(cid);
      krate.extra_get(rng).map(|id| {
        let hir::Thing::NamedType(_, kind) = *krate.get(id) else { unreachable!() };
        kind
      }).collect()
    };

    let mut sub = Vec::with_capacity(kinds.len());
    for kind in kinds {
      sub.push(TypeLow::low(ctx, sum, kind)?);
    }
    let rng = ctx.cre.extra(&sub);

    // Post
    let kind = mir::TypeKind::Struct(rng);

    mir::Type {
      kind,
      layout: Layouter::layout(&kind, ctx.tin.layinfo, ctx.cre),
    }.push_ok(ctx.cre)
  }

  fn low_tuple(ctx: &mut Ctx, sum: &mut Summary, rng: hir::TypeRng, cid: hir::CID) -> Status<mir::TypeId> {
    let kinds: Vec<hir::TypeId> = {
      let krate = ctx.get_krate(cid);
      krate.extra_get(rng).collect()
    };

    let mut sub = Vec::with_capacity(kinds.len());
    for kind in kinds {
      sub.push(TypeLow::low(ctx, sum, kind)?);
    }
    let rng = ctx.cre.extra(&sub);

    // Post
    let kind = mir::TypeKind::Struct(rng);

    mir::Type {
      kind,
      layout: Layouter::layout(&kind, ctx.tin.layinfo, ctx.cre),
    }.push_ok(ctx.cre)
  }

  fn low_iface(_ctx: &mut Ctx, _sum: &mut Summary, _rng: hir::ThingRng) -> Status<mir::TypeId> {
    todo!()
  }

  fn low_trait_from(ctx: &mut Ctx, sum: &mut Summary, hidden: hir::TypeId) -> Status<mir::TypeId> {
    TypeLow::low(ctx, sum, hidden)
  }

  fn low_fun(ctx: &mut Ctx, sum: &mut Summary, self_kind: Option<hir::TypeId>, args: hir::ThingRng, ret: hir::TypeId, cid: hir::CID) -> Status<mir::TypeId> {
    let kinds: Vec<hir::TypeId> = {
      let krate = ctx.get_krate(cid);
      krate.extra_get(args).map(|id| {
        let hir::Thing::NamedType(_, kind) = *krate.get(id) else { unreachable!() };
        kind
      }).collect()
    };

    let mut ctn = Vec::with_capacity(kinds.len() + (self_kind.is_some() as usize));
    if let Some(self_kind) = self_kind {
      ctn.push(Self::low(ctx, sum, self_kind)?);
    }
    for kind in kinds {
      ctn.push(Self::low(ctx, sum, kind)?);
    }
    let args = ctx.cre.extra(&ctn);

    let ret = Self::low(ctx, sum, ret)?;

    // Post
    let kind = mir::TypeKind::Fun{
      args, ret,
    };

    mir::Type{
      kind,
      layout: Layouter::layout(&kind, ctx.tin.layinfo, ctx.cre),
    }.push_ok(ctx.cre)
  }

}
