use inkwell::{ AddressSpace, types::{AnyTypeEnum, BasicMetadataTypeEnum, BasicType, BasicTypeEnum} };

use qwc_mir::{FloatKind, Type, TypeId, TypeKind};

use crate::cgen::{CtxI, CtxM};


pub fn any_type_to_basic<'ctx>(any_ty: AnyTypeEnum<'ctx>) -> BasicTypeEnum<'ctx> {
  match any_ty {
    AnyTypeEnum::IntType(t) => t.into(),
    AnyTypeEnum::FloatType(t) => t.into(),
    AnyTypeEnum::PointerType(t) => t.into(),
    AnyTypeEnum::StructType(t) => t.into(),
    AnyTypeEnum::ArrayType(t) => t.into(),
    _ => panic!("Expected basic type, found {:?}", any_ty),
  }
}


pub struct TypeLow;

impl TypeLow {

  pub fn low<'ctx>(ictx: &CtxI<'ctx, '_>, it: &Type) -> AnyTypeEnum<'ctx> {
    match it.kind {
      TypeKind::Unit => ictx.ctx.struct_type(&[], false).into(),

      TypeKind::Bool => ictx.ctx.bool_type().into(),

      TypeKind::Int(len, ..) => ictx.ctx.custom_width_int_type(len).unwrap().into(),

      TypeKind::Float(fk) => match fk {
        FloatKind::BF16 => ictx.ctx.bf16_type().into(),
        FloatKind::F16 => ictx.ctx.f16_type().into(),
        FloatKind::F32 => ictx.ctx.f32_type().into(),
        FloatKind::F64 => ictx.ctx.f64_type().into(),
        FloatKind::F128 => ictx.ctx.f128_type().into(),
      }

      TypeKind::Ptr => ictx.ctx.ptr_type(AddressSpace::from(0)).into(),

      TypeKind::Array(elem, count) => {
        let elem_ty = any_type_to_basic(Self::low(ictx, ictx.cre.get(elem)));
        elem_ty.array_type(count).into()
      }

      TypeKind::Slice(..) => panic!(),

      TypeKind::Struct(rng) => {
        let mut types: Vec<BasicTypeEnum> = Vec::new();
        for id in ictx.cre.extra_get(rng) {
          let param_ty = any_type_to_basic(Self::low(ictx, ictx.cre.get(id)));
          types.push(param_ty);
        }

        ictx.ctx.struct_type(&types, false).into()
      }

      TypeKind::Fun { args, ret } => {
        let mut param: Vec<BasicMetadataTypeEnum> = Vec::new();
        for id in ictx.cre.extra_get(args) {
          let param_ty = any_type_to_basic(Self::low(ictx, ictx.cre.get(id)));
          param.push(param_ty.into());
        }

        let ret_ty: &Type = ictx.cre.get(ret);
        let fn_ty = match ret_ty.kind {
          TypeKind::Unit => ictx.ctx.void_type().fn_type(&param, false),
          _ => any_type_to_basic(Self::low(ictx, ret_ty)).fn_type(&param, false),
        };

        fn_ty.into()
      }
    }
  }

  pub fn low_basic_cached<'ctx>(uctx: &mut CtxM<'ctx>, ictx: &CtxI<'ctx, '_>, id: TypeId) -> BasicTypeEnum<'ctx> {
    if let Some(&cached) = uctx.type_cache.get(&id) {
      return cached;
    }

    let ty: &Type = ictx.cre.get(id);

    let basic_ty = any_type_to_basic(Self::low(ictx, ty));
    
    uctx.type_cache.insert(id, basic_ty);
    basic_ty
  }

}
