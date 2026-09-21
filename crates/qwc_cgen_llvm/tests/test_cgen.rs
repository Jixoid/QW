use std::num::NonZeroU32;
use inkwell::context::Context;
use qwc_cgen_llvm::CGenLLVM;
use qwc_mir::{
  Block, Const, Expr, FloatKind, Inst, Krate, Layout, LayoutBy, SSA, Symbol, SymbolKind, SymbolStat, Type, TypeKind, Value,
};


#[test]
fn test_compile_empty_crate() {
  let cre = Krate::new();
  let ctx = Context::create();
  let mol = CGenLLVM::compile_to_module(&ctx, "test_empty", &cre);
  assert!(mol.verify().is_ok());
}


#[test]
fn test_compile_global_var() {
  let mut cre = Krate::new();
  let i32_ty = cre.push(Type {
    kind: TypeKind::Int(NonZeroU32::new(32).unwrap(), true),
    layout: Layout::new(32, 32, LayoutBy::SYS),
  });

  let sym_name = cre.sym("g_counter");
  cre.push(Symbol {
    name: sym_name,
    kind: SymbolKind::Variable { ism: true },
    stat: SymbolStat::Export,
    ety: i32_ty,
  });

  let ir = CGenLLVM::generate_to_string(&cre).expect("Verification failed");
  assert!(ir.contains("@g_counter = global i32 0"));
}


#[test]
fn test_compile_simple_function() {
  let mut cre = Krate::new();

  let unit_ty = cre.push(Type {
    kind: TypeKind::Unit,
    layout: Layout::new(0, 8, LayoutBy::SYS),
  });

  let empty_args = cre.extra::<Type>(&[]);
  let fn_ty = cre.push(Type {
    kind: TypeKind::Fun {
      args: empty_args,
      ret: unit_ty,
    },
    layout: Layout::new(64, 64, LayoutBy::SYS),
  });

  let ret_inst = cre.push(Inst {
    kind: Expr::Return(Value::Const(Const::Unit)),
    dest: None,
  });
  let insts = cre.extra(&[ret_inst]);
  let stack = cre.extra::<Type>(&[]);

  let blok = cre.push(Block { insts, stack });

  let sym_name = cre.sym("main");
  cre.push(Symbol {
    name: sym_name,
    kind: SymbolKind::Function { blok },
    stat: SymbolStat::Export,
    ety: fn_ty,
  });

  let ir = CGenLLVM::generate_to_string(&cre).expect("Verification failed");
  assert!(ir.contains("define void @main()"));
  assert!(ir.contains("ret void"));
}


#[test]
fn test_compile_store_load_return() {
  let mut cre = Krate::new();

  let i32_ty = cre.push(Type {
    kind: TypeKind::Int(NonZeroU32::new(32).unwrap(), true),
    layout: Layout::new(32, 32, LayoutBy::SYS),
  });

  let empty_args = cre.extra::<Type>(&[]);
  let fn_ty = cre.push(Type {
    kind: TypeKind::Fun {
      args: empty_args,
      ret: i32_ty,
    },
    layout: Layout::new(64, 64, LayoutBy::SYS),
  });

  let g_var_name = cre.sym("g_val");
  let g_var_sym = cre.push(Symbol {
    name: g_var_name,
    kind: SymbolKind::Variable { ism: true },
    stat: SymbolStat::Private,
    ety: i32_ty,
  });

  // Store 42 into g_val
  let store_inst = cre.push(Inst {
    kind: Expr::Store {
      target: Value::GlobalRef(g_var_sym),
      kind: i32_ty,
      value: Value::Const(Const::Int(42)),
    },
    dest: None,
  });

  // Load from g_val into %0
  let load_inst = cre.push(Inst {
    kind: Expr::Load {
      target: Value::GlobalRef(g_var_sym),
      kind: i32_ty,
    },
    dest: Some(SSA::new(0)),
  });

  // Return %0
  let ret_inst = cre.push(Inst {
    kind: Expr::Return(Value::SSA(SSA::new(0))),
    dest: None,
  });

  let insts = cre.extra(&[store_inst, load_inst, ret_inst]);
  let stack = cre.extra::<Type>(&[]);
  let blok = cre.push(Block { insts, stack });

  let fn_sym = cre.sym("test_load_store");
  cre.push(Symbol {
    name: fn_sym,
    kind: SymbolKind::Function { blok },
    stat: SymbolStat::Export,
    ety: fn_ty,
  });

  let ir = CGenLLVM::generate_to_string(&cre).expect("Verification failed");
  assert!(ir.contains("store i32 42, ptr @g_val"));
  assert!(ir.contains("load i32, ptr @g_val"));
  assert!(ir.contains("ret i32"));
}


#[test]
fn test_compile_binary_add_and_stack_alloca() {
  let mut cre = Krate::new();

  let i32_ty = cre.push(Type {
    kind: TypeKind::Int(NonZeroU32::new(32).unwrap(), true),
    layout: Layout::new(32, 32, LayoutBy::SYS),
  });

  let empty_args = cre.extra::<Type>(&[]);
  let fn_ty = cre.push(Type {
    kind: TypeKind::Fun {
      args: empty_args,
      ret: i32_ty,
    },
    layout: Layout::new(64, 64, LayoutBy::SYS),
  });

  // Binary add: 10 + 20 -> %0
  let add_inst = cre.push(Inst {
    kind: Expr::Binary(Value::Const(Const::Int(10)), Value::Const(Const::Int(20))),
    dest: Some(SSA::new(0)),
  });

  // Return %0
  let ret_inst = cre.push(Inst {
    kind: Expr::Return(Value::SSA(SSA::new(0))),
    dest: None,
  });

  let insts = cre.extra(&[add_inst, ret_inst]);
  let stack = cre.extra(&[i32_ty]); // 1 stack slot allocated
  let blok = cre.push(Block { insts, stack });

  let fn_sym = cre.sym("test_add");
  cre.push(Symbol {
    name: fn_sym,
    kind: SymbolKind::Function { blok },
    stat: SymbolStat::Export,
    ety: fn_ty,
  });

  let ir = CGenLLVM::generate_to_string(&cre).expect("Verification failed");
  assert!(ir.contains("alloca i32"));
  assert!(ir.contains("ret i32 30") || ir.contains("add i32"));
}


#[test]
fn test_compile_types_struct_array_float() {
  let mut cre = Krate::new();

  let f32_ty = cre.push(Type {
    kind: TypeKind::Float(FloatKind::F32),
    layout: Layout::new(32, 32, LayoutBy::SYS),
  });

  let arr_ty = cre.push(Type {
    kind: TypeKind::Array(f32_ty, 4),
    layout: Layout::new(128, 32, LayoutBy::SYS),
  });

  let fields = cre.extra(&[f32_ty, arr_ty]);
  let struct_ty = cre.push(Type {
    kind: TypeKind::Struct(fields),
    layout: Layout::new(160, 32, LayoutBy::SYS),
  });

  let sym_name = cre.sym("g_data");
  cre.push(Symbol {
    name: sym_name,
    kind: SymbolKind::Variable { ism: false },
    stat: SymbolStat::Normal,
    ety: struct_ty,
  });

  let ir = CGenLLVM::generate_to_string(&cre).expect("Verification failed");
  assert!(ir.contains("@g_data = hidden constant { float, [4 x float] } zeroinitializer"));
}


#[test]
fn test_forward_reference_between_symbols() {
  let mut cre = Krate::new();

  let i32_ty = cre.push(Type {
    kind: TypeKind::Int(NonZeroU32::new(32).unwrap(), true),
    layout: Layout::new(32, 32, LayoutBy::SYS),
  });

  let empty_args = cre.extra::<Type>(&[]);
  let fn_ty = cre.push(Type {
    kind: TypeKind::Fun {
      args: empty_args,
      ret: i32_ty,
    },
    layout: Layout::new(64, 64, LayoutBy::SYS),
  });

  // Function 1 references g_later, which is defined AFTER Function 1!
  let g_later_name = cre.sym("g_later");
  let g_later_id = qwc_mir::SymbId::from_any(qwc_mir::AnyId::new(1, qwc_mir::id::NodeKind::Symb));

  let load_inst = cre.push(Inst {
    kind: Expr::Load {
      target: Value::GlobalRef(g_later_id),
      kind: i32_ty,
    },
    dest: Some(SSA::new(0)),
  });

  let ret_inst = cre.push(Inst {
    kind: Expr::Return(Value::SSA(SSA::new(0))),
    dest: None,
  });

  let insts = cre.extra(&[load_inst, ret_inst]);
  let stack = cre.extra::<Type>(&[]);
  let blok = cre.push(Block { insts, stack });

  let fn_sym = cre.sym("fn_earlier");
  cre.push(Symbol {
    name: fn_sym,
    kind: SymbolKind::Function { blok },
    stat: SymbolStat::Export,
    ety: fn_ty,
  });

  // Here is g_later defined AFTER fn_earlier
  cre.push(Symbol {
    name: g_later_name,
    kind: SymbolKind::Variable { ism: true },
    stat: SymbolStat::Export,
    ety: i32_ty,
  });

  let ir = CGenLLVM::generate_to_string(&cre).expect("Verification failed");
  assert!(ir.contains("define i32 @fn_earlier()"));
  assert!(ir.contains("load i32, ptr @g_later"));
  assert!(ir.contains("@g_later = global i32 0"));
}
