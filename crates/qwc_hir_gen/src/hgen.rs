/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use qwc_arena::Files;
use qwc_ast as ast;
use qwc_diagnostic::{Label, Message, Span, Summary, msg::*};
use qwc_hir::{self as hir, CID, Deps, PrimTypes};
use qwc_resolve::{ImplFor, LocalScopeManager, Scope, ScopeMap};
use qwc_string_interner::{Sid, StrInterner};
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
  pub prims: &'imod PrimTypes,
  pub type_impls: &'loc FxHashMap<hir::TypeId, TypeMethods>,
  pub path: hir::DefPathId,
}


pub struct TypeMethods {
  pub traits: FxHashSet<hir::TypeId>,
  pub methods: FxHashMap<Sid, (usize, ast::ItemId)>,
}


pub struct FunCtx {
  pub ret_ty: hir::TypeId,
  pub sign_pos: Span,
}


pub struct CacheMap {
  pub(crate) cache_type: FxHashMap<ast::TypeId, hir::TypeId>,
  pub(crate) cache_item: FxHashMap<ast::ItemId, Option<hir::ItemId>>,
  pub(crate) cache_expr: FxHashMap<ast::ExprId, hir::ExprId>,
  
  pub(crate) cache_generic_type: FxHashMap<(hir::TypeId, Vec<hir::TypeId>), hir::TypeId>,
  pub(crate) cache_generic_item: FxHashMap<(hir::ItemId, Vec<hir::TypeId>), hir::ItemId>,
  pub(crate) cache_generic_expr: FxHashMap<(hir::ExprId, Vec<hir::TypeId>), hir::ExprId>,

  pub(crate) self_ty: Vec<hir::TypeId>,
}

impl<'ast, 'hir, 'loc, 'imod> Ctx<'ast, 'hir, 'loc, 'imod> {

  pub fn get_krate(&self, cid: CID) -> &hir::Krate {
    if cid == self.cre.cid() {
      self.cre
    } else {
      self.ideps.get(cid)
    }
  }


  pub fn get<T: hir::HirKind + hir::GetApi<T>>(&self, id: hir::HirId<T>) -> &T { self.get_krate(id.cid()).get(id) }
  
  pub fn extra_get<T: hir::HirKind>(&self, rng: hir::Rng<T>) -> impl Iterator<Item = hir::HirId<T>> { self.get_krate(rng.cid()).extra_get(rng) }


  pub fn expr_name(&self, _id: hir::ExprId) -> String {
    todo!()
  }

  pub fn type_name(&self, id: hir::TypeId) -> String {
    let ty = self.get(id);
    match ty.kind {
      // Generic
      hir::TypeKind::GenericType{idx} => format!("<{}>", idx),
      hir::TypeKind::GenericRaw{..} => "<raw generic>".to_string(),
      hir::TypeKind::GenericSelfType => "Self".to_string(),
      hir::TypeKind::Error => "{error}".to_string(),

      // Primitive
      hir::TypeKind::Unit => "()".to_string(),
      hir::TypeKind::Never => "!".to_string(),
      hir::TypeKind::Bool => "bool".to_string(),
      hir::TypeKind::Str => "str".to_string(),

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
      
      
      // Trait
      hir::TypeKind::Trait(..) => format!("trait {{}}"),
      hir::TypeKind::Iface(..) => format!("iface {{}}"),

      hir::TypeKind::TraitFrom {trait_ty, hidden} => format!("trait({}, hidden: {})", self.type_name(trait_ty), self.type_name(hidden)),


      // Enum
      hir::TypeKind::Enum(..) => format!("enum {{}}"),

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
    &mut Ctx{cre: $ctx.cre, tin: $ctx.tin, sum: $ctx.sum, cmap: $ctx.cmap, imods: $ctx.imods, ideps: $ctx.ideps, prims: $ctx.prims, src: $ctx.src, sin: $ctx.sin, far: $ctx.far, scp: $ctx.scp, type_impls: $ctx.type_impls, loc: &mut *$ctx.loc, path: $ctx.path, $lscp}
  };

  ($lscp:ident, $path:ident -> $ctx:expr) => {
    &mut Ctx{cre: $ctx.cre, tin: $ctx.tin, sum: $ctx.sum, cmap: $ctx.cmap, imods: $ctx.imods, ideps: $ctx.ideps, prims: $ctx.prims, src: $ctx.src, sin: $ctx.sin, far: $ctx.far, scp: $ctx.scp, type_impls: $ctx.type_impls, loc: &mut *$ctx.loc, $lscp, path: $path}
  };

  (path $path:ident -> $ctx:expr) => {
    &mut Ctx{cre: $ctx.cre, tin: $ctx.tin, sum: $ctx.sum, cmap: $ctx.cmap, imods: $ctx.imods, ideps: $ctx.ideps, prims: $ctx.prims, src: $ctx.src, sin: $ctx.sin, far: $ctx.far, scp: $ctx.scp, type_impls: $ctx.type_impls, loc: &mut *$ctx.loc, lscp: $ctx.lscp, path: $path}
  };

  (loc $loc:ident -> $ctx:expr) => {
    &mut Ctx{cre: $ctx.cre, tin: $ctx.tin, sum: $ctx.sum, cmap: $ctx.cmap, imods: $ctx.imods, ideps: $ctx.ideps, prims: $ctx.prims, src: $ctx.src, sin: $ctx.sin, far: $ctx.far, scp: $ctx.scp, type_impls: $ctx.type_impls, loc: &mut $loc, lscp: $ctx.lscp, path: $ctx.path}
  }
}



pub struct HGen;

impl<'ast, 'hir, 'imod> HGen {

  pub fn low(src: &'ast ast::Krate, sin: &'ast StrInterner, far: &'ast Files, scp: &'ast ScopeMap, implst: Vec<ImplFor>, imods: &'imod [CID], ideps: &'imod mut Deps) -> (Option<CID>, Summary) {
    let cid = ideps.get_next_id();
    let mut cre = hir::Krate::new(cid);
    
    let prims = *ideps.prims().expect("core primitive types must be initialized in deps");
    let mut tin = TypeInterner::new();
    let mut sum = Summary::new();
    let mut cmap = CacheMap{
      cache_type: FxHashMap::default(),
      cache_item: FxHashMap::default(),
      cache_expr: FxHashMap::default(),
      cache_generic_type: FxHashMap::default(),
      cache_generic_item: FxHashMap::default(),
      cache_generic_expr: FxHashMap::default(),
      self_ty: vec![],
    };
    let mut loc = LocalScopeManager::new();
    
    let root = src.root().unwrap();
    let root_it = src.get(root);
    let root_name = root_it.name.map(|n| n.sid()).or_else(|| sin.get("main")).unwrap_or_else(|| sin.sid_entry());
    let root_path = cre.push(hir::DefPath::Root(root_name));
    let lscp = scp.get(&root.to_any()).unwrap();
    
    // Pre 1
    let type_impls = match Self::pre1(&mut Ctx{cre: &mut cre, tin: &mut tin, sum: &mut sum, cmap: &mut cmap, src, sin, far, scp, lscp, imods, ideps, prims: &prims, type_impls: &FxHashMap::default(), loc: &mut loc, path: root_path}, implst) {
      Err(..) => return (None, sum),
      Ok(v) => v,
    };

    // Start
    match ItemLow::low(&mut Ctx{cre: &mut cre, tin: &mut tin, sum: &mut sum, cmap: &mut cmap, src, sin, far, scp, lscp, imods, ideps, prims: &prims, type_impls: &type_impls, loc: &mut loc, path: root_path}, root) {
      Ok(root) => cre.set_root(root.unwrap()),
      
      Err(msg) => sum.add(msg),
    };
    
    ideps.add(cre);

    (Some(cid), sum)
  }


  fn pre1(ctx: &mut Ctx, implst: Vec<ImplFor>) -> Result<FxHashMap<hir::TypeId, TypeMethods>, ()> {
    let mut type_impls: FxHashMap<hir::TypeId, TypeMethods> = FxHashMap::default();

    for ImplFor{ container, item } in implst {
      let ast::ItemKind::Impl { type_ty, trait_ty, ctn: fields } = ctx.src.get(item).kind else { panic!() };

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


      // Reg
      let reg = |tyfuns: &mut TypeMethods| -> Result<(), Message> {
        if let Some(trait_ty) = trait_ty { tyfuns.traits.insert(trait_ty); }
        
        for (idx, id) in ctx.src.extra_get(fields).enumerate() {
          if let ast::Field{kind: ast::FieldKind::Fun{..}, pos, name, ..} = ctx.src.get(id) {
            let sid = name.unwrap().sid();
            
            if let Some(..) = tyfuns.methods.insert(sid, (idx, item)) {
              return Err(Message::error(DUPLICATE_IDENTIFIER, Label::new_pos(*pos)))
            }
          }
        }

        Ok(())
      };


      // Entry Api
      use std::collections::hash_map::Entry::*;

      match type_impls.entry(type_ty) {
        Occupied(mut entry) => {
          if let Err(msg) = reg(entry.get_mut()) { ctx.sum.add(msg); continue };
        }
        
        Vacant(entry) => {
          let tyfuns = TypeMethods {
            traits: FxHashSet::default(),
            methods: FxHashMap::default(),
          };
          
          if let Err(msg) = reg(entry.insert(tyfuns)) { ctx.sum.add(msg); continue };
        }
      }

    }

    Ok(type_impls)
  }

}
