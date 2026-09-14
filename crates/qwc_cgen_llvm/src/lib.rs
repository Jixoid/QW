use inkwell::{GlobalVisibility, builder::Builder, context::Context, llvm_sys::core::{LLVMSetLinkage, LLVMSetVisibility}, module::{Linkage, Module}, types::{AnyTypeEnum, BasicMetadataTypeEnum, BasicType, BasicTypeEnum}, values::{AsValueRef, FunctionValue}};
use qwc_mir::{Block, BlokId, Krate, Symbol, SymbolKind, SymbolStat, Type, TypeId, TypeKind, id::NodeKind};
use qwc_cgen::*;


pub struct CGenLLVM;

impl ICGen for CGenLLVM {
  fn generate(cre: &Krate) {
    let ctx = Context::create();
		let mol = ctx.create_module("test");
		let builder = ctx.create_builder();

    for it in cre.iter::<Type>() { low_type(&ctx, cre, it); }

    for it in cre.iter::<Symbol>() { low_symbol(&ctx, &mol, &builder, cre, it); }

    mol.print_to_stderr();
  }
}


fn any_type_to_basic<'ctx>(any_ty: AnyTypeEnum<'ctx>) -> BasicTypeEnum<'ctx> {
  match any_ty {
    AnyTypeEnum::IntType(t) => t.into(),
    AnyTypeEnum::FloatType(t) => t.into(),
    AnyTypeEnum::PointerType(t) => t.into(),
    AnyTypeEnum::StructType(t) => t.into(),
    AnyTypeEnum::ArrayType(t) => t.into(),
    _ => panic!("Expected basic type, found {:?}", any_ty),
  }
}


fn low_type<'ctx>(ctx: &'ctx Context, cre: &Krate, it: &Type) -> AnyTypeEnum<'ctx> {
  match it.kind {
    TypeKind::Struct(rng) => {
      let mut types: Vec<BasicTypeEnum> = Vec::new();
      for (id, kind) in cre.extra_get(rng) {
        let ty_id = TypeId::new_from((id, kind));
        let param_ty = any_type_to_basic(low_type(ctx, cre, cre.get(ty_id)));
        types.push(param_ty.into());
      }

      ctx.struct_type(&types, false).into()
    }
    
    TypeKind::Fun{args, ret} => {
      let mut param: Vec<BasicMetadataTypeEnum> = Vec::new();
      for (id, kind) in cre.extra_get(args) {
        let ty_id = TypeId::new_from((id, kind));
        let param_ty = any_type_to_basic(low_type(ctx, cre, cre.get(ty_id)));
        param.push(param_ty.into());
      }

      let ret_ty: &Type = cre.get(ret);
      let fn_ty = match ret_ty.kind {
        TypeKind::Unit => ctx.void_type().fn_type(&param, false),
        _ => any_type_to_basic(low_type(ctx, cre, ret_ty)).fn_type(&param, false),
      };
      
      fn_ty.into()
    }

    TypeKind::Int(len, ..) => ctx.custom_width_int_type(len).unwrap().into(),

    TypeKind::Unit => ctx.struct_type(&[], true).into(),
    TypeKind::Bool => ctx.bool_type().into(),    

    _ => todo!("{it:#?}")
  }
}


fn low_symbol<'ctx>(ctx: &'ctx Context, mol: &Module<'ctx>, builder: &Builder<'ctx>, cre: &Krate, it: &Symbol) {
  let gs = match it.kind {
    SymbolKind::Variable{..} => {
      let ty = any_type_to_basic(low_type(ctx, cre, cre.get(it.ety)));
      
      let gv = mol.add_global(ty, None, cre.sym_str(it.name));

      gv.as_value_ref()
    }
    
    SymbolKind::Function{ blok } => {
      let ty = low_type(ctx, cre, cre.get(it.ety)).into_function_type();

      let fv = mol.add_function(cre.sym_str(it.name), ty, None);

      if it.stat != SymbolStat::Import {
        low_block(ctx, builder, cre, fv, it, blok);
      }

      fv.as_value_ref()
    }
  };

  // Set Linkage
  unsafe {
    match it.stat {
      SymbolStat::Private => {
        LLVMSetLinkage(gs, Linkage::Private.into());
      }
      
      SymbolStat::Normal => {
        LLVMSetLinkage(gs, Linkage::External.into());
        LLVMSetVisibility(gs, GlobalVisibility::Hidden.into());
      }
      
      SymbolStat::Export => LLVMSetLinkage(gs, Linkage::External.into()),
      SymbolStat::Import => LLVMSetLinkage(gs, Linkage::External.into()),
    }
  }
}


fn low_block<'ctx>(ctx: &'ctx Context, builder: &Builder<'ctx>, cre: &Krate, fv: FunctionValue<'ctx>, it: &Symbol, blok: BlokId) {
  let bb = ctx.append_basic_block(fv, "entry");
  builder.position_at_end(bb);

  let b: &Block = cre.get(blok);
  for (id, kind) in cre.extra_get(b.stack) {
    if kind == NodeKind::Type {
      let ty_id = TypeId::new_from((id, kind));
      let ty = any_type_to_basic(low_type(ctx, cre, cre.get(ty_id)));
      builder.build_alloca(ty, "").unwrap();
    }
  }

  let fty: &Type = cre.get(it.ety);
  if let TypeKind::Fun{ret, ..} = fty.kind {
    let ret_ty: &Type = cre.get(ret);
    if ret_ty.kind == TypeKind::Unit {
      builder.build_return(None).unwrap();
    } else {
      let ret_basic_ty = any_type_to_basic(low_type(ctx, cre, ret_ty));
      builder.build_return(Some(&ret_basic_ty.const_zero())).unwrap();
    }
  } else {
    builder.build_return(None).unwrap();
  }
}
