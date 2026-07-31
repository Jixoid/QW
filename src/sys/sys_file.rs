use crate::{ast::{Type, TypeVari, TypeState, Decl, DeclVari, VarDecl, Visibility}, control::{identy::IdentyId, module::{CompilerError, Module}}};


pub struct SysFile<'a> {
  pub mol: Module<'a,'a>,

  pub ty_void: IdentyId,
  pub ty_null: IdentyId,
  pub ty_bool: IdentyId,
  pub ty_char: IdentyId,
  pub ty_isize: IdentyId,
  pub ty_usize: IdentyId,
  pub ty_i8: IdentyId,
  pub ty_i16: IdentyId,
  pub ty_i32: IdentyId,
  pub ty_i64: IdentyId,
  pub ty_i128: IdentyId,
  pub ty_u8: IdentyId,
  pub ty_u16: IdentyId,
  pub ty_u32: IdentyId,
  pub ty_u64: IdentyId,
  pub ty_u128: IdentyId,
  pub ty_f16: IdentyId,
  pub ty_f32: IdentyId,
  pub ty_f64: IdentyId,
  pub ty_f128: IdentyId,
  pub ty_ptr: IdentyId,
}

impl<'a> SysFile<'a> {

  pub fn new() -> Result<SysFile<'a>, CompilerError> {
    let mut mol = Module::new_rtl("sys".to_string())?;

    let mut add_prim = |name: &str, vari: TypeVari<'a>| -> IdentyId {
      let id = mol.new_type(Type{state: TypeState::Resolved, vari});
      mol.new_decl(Decl::new_str(name.to_string(), DeclVari::Using(id), Visibility::Public));
      id
    };


    let ty_void = add_prim("void", TypeVari::Void);
    let ty_null = add_prim("null_t", TypeVari::Null);
    let ty_bool = add_prim("bool", TypeVari::Bool);
    let ty_char = add_prim("char", TypeVari::Char);
    let ty_isize = add_prim("isize", TypeVari::ISize);
    let ty_usize = add_prim("usize", TypeVari::USize);
    let ty_i8 = add_prim("i8", TypeVari::I8);
    let ty_i16 = add_prim("i16", TypeVari::I16);
    let ty_i32 = add_prim("i32", TypeVari::I32);
    let ty_i64 = add_prim("i64", TypeVari::I64);
    let ty_i128 = add_prim("i128", TypeVari::I128);
    let ty_u8 = add_prim("u8", TypeVari::U8);
    let ty_u16 = add_prim("u16", TypeVari::U16);
    let ty_u32 = add_prim("u32", TypeVari::U32);
    let ty_u64 = add_prim("u64", TypeVari::U64);
    let ty_u128 = add_prim("u128", TypeVari::U128);
    let ty_f16 = add_prim("f16", TypeVari::F16);
    let ty_f32 = add_prim("f32", TypeVari::F32);
    let ty_f64 = add_prim("f64", TypeVari::F64);
    let ty_f128 = add_prim("f128", TypeVari::F128);
    let ty_ptr = add_prim("ptr", TypeVari::Ptr);
    drop(add_prim);

    mol.new_decl(Decl::new_str("true".to_string(), DeclVari::Var(VarDecl{kind: ty_bool, comptime: true, init: None, acck: crate::ast::types::AccessKind::IMM}), Visibility::Public));
    mol.new_decl(Decl::new_str("false".to_string(), DeclVari::Var(VarDecl{kind: ty_bool, comptime: true, init: None, acck: crate::ast::types::AccessKind::IMM}), Visibility::Public));
    mol.new_decl(Decl::new_str("null".to_string(), DeclVari::Var(VarDecl{kind: ty_null, comptime: true, init: None, acck: crate::ast::types::AccessKind::IMM}), Visibility::Public));
    mol.new_decl(Decl::new_str("is_debug".to_string(), DeclVari::Var(VarDecl{kind: ty_bool, comptime: true, init: None, acck: crate::ast::types::AccessKind::IMM}), Visibility::Public));


    Ok(SysFile{
      mol,
      ty_void, ty_null,
      ty_bool, ty_char,
      ty_isize, ty_usize,
      ty_i8, ty_i16, ty_i32, ty_i64, ty_i128,
      ty_u8, ty_u16, ty_u32, ty_u64, ty_u128,
      ty_f16, ty_f32, ty_f64, ty_f128,
      ty_ptr,
    })
  }

}
