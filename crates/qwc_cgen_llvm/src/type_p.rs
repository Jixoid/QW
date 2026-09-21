use inkwell::{
  AddressSpace,
  context::Context,
  types::{AnyTypeEnum, BasicMetadataTypeEnum, BasicType, BasicTypeEnum},
};
use qwc_mir::{FloatKind, Krate, Type, TypeId, TypeKind};
use rustc_hash::FxHashMap;


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

  pub fn low<'ctx>(ctx: &'ctx Context, cre: &Krate, it: &Type) -> AnyTypeEnum<'ctx> {
    match it.kind {
      TypeKind::Unit => ctx.struct_type(&[], false).into(),

      TypeKind::Bool => ctx.bool_type().into(),

      TypeKind::Int(len, ..) => ctx.custom_width_int_type(len).unwrap().into(),

      TypeKind::Float(fk) => match fk {
        FloatKind::BF16 => ctx.bf16_type().into(),
        FloatKind::F16 => ctx.f16_type().into(),
        FloatKind::F32 => ctx.f32_type().into(),
        FloatKind::F64 => ctx.f64_type().into(),
        FloatKind::F128 => ctx.f128_type().into(),
      },

      TypeKind::Ptr => ctx.ptr_type(AddressSpace::from(0)).into(),

      TypeKind::Array(elem, count) => {
        let elem_ty = any_type_to_basic(Self::low(ctx, cre, cre.get(elem)));
        elem_ty.array_type(count).into()
      }

      TypeKind::Struct(rng) => {
        let mut types: Vec<BasicTypeEnum> = Vec::new();
        for (id, kind) in cre.extra_get(rng) {
          let ty_id = TypeId::new_from((id, kind));
          let param_ty = any_type_to_basic(Self::low(ctx, cre, cre.get(ty_id)));
          types.push(param_ty);
        }

        ctx.struct_type(&types, false).into()
      }

      TypeKind::Fun { args, ret } => {
        let mut param: Vec<BasicMetadataTypeEnum> = Vec::new();
        for (id, kind) in cre.extra_get(args) {
          let ty_id = TypeId::new_from((id, kind));
          let param_ty = any_type_to_basic(Self::low(ctx, cre, cre.get(ty_id)));
          param.push(param_ty.into());
        }

        let ret_ty: &Type = cre.get(ret);
        let fn_ty = match ret_ty.kind {
          TypeKind::Unit => ctx.void_type().fn_type(&param, false),
          _ => any_type_to_basic(Self::low(ctx, cre, ret_ty)).fn_type(&param, false),
        };

        fn_ty.into()
      }
    }
  }

  pub fn low_basic_cached<'ctx>(
    ctx: &'ctx Context,
    cre: &Krate,
    ty_id: TypeId,
    cache: &mut FxHashMap<TypeId, BasicTypeEnum<'ctx>>,
  ) -> BasicTypeEnum<'ctx> {
    if let Some(&cached) = cache.get(&ty_id) {
      return cached;
    }

    let ty: &Type = cre.get(ty_id);
    let basic_ty = any_type_to_basic(Self::low(ctx, cre, ty));
    cache.insert(ty_id, basic_ty);
    basic_ty
  }

}
