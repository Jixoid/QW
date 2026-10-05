use qwc_arena::Files;
use qwc_ast as ast;
use qwc_diagnostic::Summary;
use qwc_hir::{self as hir, CID, Deps};
use qwc_resolve::{ImplFor, LocalScopeManager, Scope, ScopeMap};
use qwc_string_interner::{StrInterner};
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{ItemLow, ty_interner::TypeInterner, type_p::TypeLow};


pub struct Ctx<'ast, 'hir, 'loc, 'imod> {
  pub src:  &'ast ast::Krate,
  pub sin:  &'ast StrInterner,
  pub far:  &'ast Files,
  pub scp:  &'ast ScopeMap,
  pub lscp: &'ast Scope,
  pub cre:  &'hir mut hir::Krate,
  pub tin:  &'hir mut TypeInterner,
  pub sum:  &'hir mut Summary,
  pub cmap: &'hir mut CacheMap,
  pub loc:  &'loc mut LocalScopeManager,
  pub imods: &'imod [CID],
  pub ideps: &'imod mut Deps,
  pub type_impls: &'loc FxHashMap<hir::TypeId, TypeMethods>,
}


pub struct TypeMethods {
  pub traits: FxHashSet<hir::TypeId>,
  //pub methods: FxHashMap<Sid, hir::ItemId>,
}


pub struct CacheMap {
  pub(crate) cache_type: FxHashMap<ast::TypeId, hir::TypeId>,
  pub(crate) cache_item: FxHashMap<ast::ItemId, Option<hir::ItemId>>,
  pub(crate) cache_expr: FxHashMap<ast::ExprId, hir::ExprId>,
  pub(crate) cache_type_import: FxHashMap<hir::TypeId, hir::TypeId>,
  pub(crate) self_ty: Vec<hir::TypeId>,
}

impl<'ast, 'hir, 'loc, 'imod> Ctx<'ast, 'hir, 'loc, 'imod> {
  pub fn low_hir_type(&mut self, id: hir::TypeId) -> hir::TypeId {
    if id.cid() == self.cre.cid() {
      return id;
    }

    if let Some(&cached) = self.cmap.cache_type_import.get(&id) {
      return cached;
    }

    let other_krate = self.ideps.get(id.cid());
    let other_ty: hir::Type = *other_krate.get(id);

    let local_id = match other_ty.kind {
      hir::TypeKind::Bool => self.tin.ty_bool(),
      hir::TypeKind::Unit => self.tin.ty_unit(),
      hir::TypeKind::Never => self.tin.ty_never(),
      hir::TypeKind::GenericType => self.tin.ty_generic_type(),
      hir::TypeKind::ArchInt(true) => self.tin.ty_isize(),
      hir::TypeKind::ArchInt(false) => self.tin.ty_usize(),
      hir::TypeKind::Int(8, true) => self.tin.ty_i8(),
      hir::TypeKind::Int(16, true) => self.tin.ty_i16(),
      hir::TypeKind::Int(32, true) => self.tin.ty_i32(),
      hir::TypeKind::Int(64, true) => self.tin.ty_i64(),
      hir::TypeKind::Int(128, true) => self.tin.ty_i128(),
      hir::TypeKind::Int(8, false) => self.tin.ty_u8(),
      hir::TypeKind::Int(16, false) => self.tin.ty_u16(),
      hir::TypeKind::Int(32, false) => self.tin.ty_u32(),
      hir::TypeKind::Int(64, false) => self.tin.ty_u64(),
      hir::TypeKind::Int(128, false) => self.tin.ty_u128(),
      _ => self.cre.push(other_ty),
    };

    self.cmap.cache_type_import.insert(id, local_id);
    local_id
  }

  pub fn get_krate(&self, cid: CID) -> &hir::Krate {
    if cid == self.cre.cid() {
      self.cre
    } else {
      self.ideps.get(cid)
    }
  }


  pub fn get_type(&self, id: hir::TypeId) -> &hir::Type {
    self.get_krate(id.cid()).get(id)
  }


  pub fn expr_name(&self, _id: hir::ExprId) -> String {
    todo!()
  }

  pub fn type_name(&self, id: hir::TypeId) -> String {
    let ty = self.get_type(id);
    match ty.kind {
      // Generic
      hir::TypeKind::GenericType => "<generic>".to_string(),
      hir::TypeKind::GenericSelfType => "Self".to_string(),


      // Primitive
      hir::TypeKind::Unit => "()".to_string(),
      hir::TypeKind::Never => "!".to_string(),
      hir::TypeKind::Bool => "bool".to_string(),

      hir::TypeKind::Bit(bits) => format!("b{}", bits),
      
      hir::TypeKind::Int(bits, true) => format!("i{}", bits),
      hir::TypeKind::Int(bits, false) => format!("u{}", bits),
      
      hir::TypeKind::Float(bits) => format!("f{}", bits),

      hir::TypeKind::Meta(kind) => format!("#{}", self.type_name(kind)),
      hir::TypeKind::Option(sub) => format!("?{}", self.type_name(sub)),
      
      hir::TypeKind::ArchInt(true) => "isize".to_string(),
      hir::TypeKind::ArchInt(false) => "usize".to_string(),


      // Reference
      hir::TypeKind::Ref(sub, ism) => format!("&{}{}", if ism {"mut "} else {""}, self.type_name(sub)),
      hir::TypeKind::Ptr(sub, ism) => format!("^{}{}", if ism {"mut "} else {""}, self.type_name(sub)),
      

      // Vector
      hir::TypeKind::VScale(sub) => format!("[{} *]", self.type_name(sub)),

      hir::TypeKind::Vector(sub, len) => format!("[{} * {}]", self.type_name(sub), self.expr_name(len)),
      

      // Array
      hir::TypeKind::Slice(sub) => format!("[{}]", self.type_name(sub)),
      
      hir::TypeKind::Array(sub, len) => format!("[{}; {}]", self.type_name(sub), self.expr_name(len)),
      
      
      // Combinated
      hir::TypeKind::Struct(..) => format!("struct {{}}"),
      hir::TypeKind::Tuple(..) => format!("tuple {{}}"),
      
      hir::TypeKind::Trait(..) => format!("trait {{}}"),
      hir::TypeKind::Iface(..) => format!("iface {{}}"),

      hir::TypeKind::TraitFrom {trait_ty, hidden} => format!("trait({}, hidden: {})", self.type_name(trait_ty), self.type_name(hidden)),


      // Function
      hir::TypeKind::Fun{self_kind, args, ret} => {
        let krate = self.get_krate(id.cid());
        let mut args_str = vec![];
        
        if let Some(id) = self_kind {
          args_str.push(self.type_name(id));
        }
        
        for id in krate.extra_get(args) {
          let hir::Thing::NamedType(name, kind) = *self.cre.get(id) else { panic!() };

          args_str.push(format!("{}: {}", self.sin.str(name), self.type_name(kind)));
        }
        
        let ret_name = self.type_name(ret);
        format!("fun({}) -> {}", args_str.join(", "), ret_name)
      }
    }
  }
}


#[macro_export]
macro_rules! ctx {
  ($lscp:ident -> $ctx:expr) => {
    &mut Ctx{cre: $ctx.cre, tin: $ctx.tin, sum: $ctx.sum, cmap: $ctx.cmap, imods: $ctx.imods, ideps: $ctx.ideps, src: $ctx.src, sin: $ctx.sin, far: $ctx.far, scp: $ctx.scp, type_impls: $ctx.type_impls, loc: &mut *$ctx.loc, $lscp}
  };

  (loc $loc:ident -> $ctx:expr) => {
    &mut Ctx{cre: $ctx.cre, tin: $ctx.tin, sum: $ctx.sum, cmap: $ctx.cmap, imods: $ctx.imods, ideps: $ctx.ideps, src: $ctx.src, sin: $ctx.sin, far: $ctx.far, scp: $ctx.scp, type_impls: $ctx.type_impls, loc: &mut $loc, lscp: $ctx.lscp}
  }
}



pub struct HGen;

impl<'ast, 'hir, 'imod> HGen {

  pub fn low(src: &'ast ast::Krate, sin: &'ast StrInterner, far: &'ast Files, scp: &'ast ScopeMap, implst: Vec<ImplFor>, imods: &'imod [CID], ideps: &'imod mut Deps) -> (Option<CID>, Summary) {
    let cid = ideps.get_next_id();
    let mut cre = hir::Krate::new(cid);
    
    let mut tin = TypeInterner::new(&mut cre);
    let mut sum = Summary::new();
    let mut cmap = CacheMap{ cache_type: FxHashMap::default(), cache_item: FxHashMap::default(), cache_expr: FxHashMap::default(), cache_type_import: FxHashMap::default(), self_ty: vec![]};
    let mut loc = LocalScopeManager::new();
    
    let root = src.root().unwrap();
    let lscp = scp.get(&root.to_any()).unwrap();
    
    // Pre 1
    let type_impls = match Self::pre1(&mut Ctx{cre: &mut cre, tin: &mut tin, sum: &mut sum, cmap: &mut cmap, src, sin, far, scp, lscp, imods, ideps, type_impls: &FxHashMap::default(), loc: &mut loc}, implst) {
      Err(..) => return (None, sum),
      Ok(v) => v,
    };

    // Start
    match ItemLow::low(&mut Ctx{cre: &mut cre, tin: &mut tin, sum: &mut sum, cmap: &mut cmap, src, sin, far, scp, lscp, imods, ideps, type_impls: &type_impls, loc: &mut loc}, root) {
      Ok(root) => cre.set_root(root.unwrap()),
      
      Err(msg) => sum.add(msg),
    };
    
    ideps.add(cre);

    (Some(cid), sum)
  }


  fn pre1(ctx: &mut Ctx, implst: Vec<ImplFor>) -> Result<FxHashMap<hir::TypeId, TypeMethods>, ()> {
    let mut type_impls: FxHashMap<hir::TypeId, TypeMethods> = FxHashMap::default();

    for ImplFor{ type_ty, trait_ty, container } in implst {
      let lscp = ctx.scp.get(&container).unwrap();
      let ctx = ctx!(lscp -> ctx);

      let type_ty = match TypeLow::low(ctx, type_ty) {
        Ok(v) => v,
        Err(err) => { ctx.sum.add(err); continue },
      };
      
      let trait_ty = match trait_ty.map(|id| TypeLow::low(ctx, id)).transpose() {
        Ok(v) => v,
        Err(err) => { ctx.sum.add(err); continue },
      };


      use std::collections::hash_map::Entry::*;

      match type_impls.entry(type_ty) {
        Occupied(mut entry) => {
          let tyfuns = entry.get_mut();

          if let Some(trait_ty) = trait_ty { tyfuns.traits.insert(trait_ty); }
        }
        
        Vacant(entry) => {
          let mut tyfuns = TypeMethods {
            traits: FxHashSet::default(),
            //methods: FxHashMap::default(),
          };
          
          if let Some(trait_ty) = trait_ty { tyfuns.traits.insert(trait_ty); }
          
          entry.insert(tyfuns);
        }
      };
    }

    Ok(type_impls)
  }

}
