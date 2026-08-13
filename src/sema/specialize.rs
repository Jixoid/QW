use std::collections::HashMap;
use crate::{ast::*, control::identy::{AstId, IdentyKind}, control::Module};


pub struct Specializer<'f, 'a, 'd> {
	pub mol: &'f mut Module<'a, 'd>,
	pub sub_map: HashMap<String, AstId>,
	pub cloned_types: HashMap<AstId, AstId>,
	pub cloned_decls: HashMap<AstId, AstId>,
}

impl<'f, 'a, 'd> Specializer<'f, 'a, 'd> {

	pub fn new(mol: &'f mut Module<'a, 'd>, sub_map: HashMap<String, AstId>) -> Self {
		Self { 
			mol, 
			sub_map,
			cloned_types: HashMap::new(),
			cloned_decls: HashMap::new(),
		}
	}

	pub fn clone_decl(&mut self, decl_id: AstId) -> AstId {
		if let Some(&new_id) = self.cloned_decls.get(&decl_id) {
			return new_id;
		}

		let old_decl_ptr = self.mol.get_decl(decl_id) as *const Decl;
		let old_decl = unsafe { &*old_decl_ptr };

		let name_clone = match &old_decl.name {
			DeclName::Word(w) => DeclName::Word(*w),
			DeclName::Name(n) => DeclName::Name(n.clone()),
		};
		let vis = old_decl.vis;

		let new_vari = match &old_decl.vari {
			DeclVari::Var(v) => DeclVari::Var(VarDecl {
				kind: self.clone_type(v.kind),
				comptime: v.comptime,
				init: v.init.map(|e| self.clone_expr(e)),
				acck: v.acck,
			}),
			DeclVari::Fun(f) => DeclVari::Fun(FunDecl {
				kind: self.clone_type(f.kind),
				blok: if f.blok.kind() == IdentyKind::Null { AstId::null() } else { self.clone_expr(f.blok) },
			}),
			DeclVari::Using(u) => DeclVari::Using(self.clone_type(*u)),
			DeclVari::Module(m) => {
				let decls = m.decls.iter().map(|&d| self.clone_decl(d)).collect();
				DeclVari::Module(ModuleDecl { decls })
			},
			DeclVari::Extend(e) => {
				let impls = e.impls.iter().map(|&d| self.clone_decl(d)).collect();
				DeclVari::Extend(ExtendDecl {
					target: self.clone_type(e.target),
					tr: self.clone_type(e.tr),
					impls,
				})
			},
			DeclVari::Import(p, r) => DeclVari::Import(p.clone(), *r),
			DeclVari::ImportWildcard(p, r) => DeclVari::ImportWildcard(p.clone(), *r),
			DeclVari::Generic(g) => {
				let decls = g.decls.iter().map(|&d| self.clone_decl(d)).collect();
				DeclVari::Generic(GenericDecl {
					params: g.params.clone(),
					requires: g.requires.map(|e| self.clone_expr(e)),
					decls,
				})
			}
		};

		let new_decl = Decl {
			name: name_clone,
			vari: new_vari,
			vis,
		};

		let new_id = self.mol.new_decl(new_decl);
		self.cloned_decls.insert(decl_id, new_id);
		
		if let Some(attrs) = self.mol.map_attr.get(&decl_id).cloned() {
			self.mol.map_attr.insert(new_id, attrs);
		}

		new_id
	}

	pub fn clone_type(&mut self, ty_id: AstId) -> AstId {
		if let Some(&new_id) = self.cloned_types.get(&ty_id) {
			return new_id;
		}
		
		let dummy_ty = Type {
			vari: TypeVari::Null,
			state: crate::ast::TypeState::Unresolved,
		};
		self.mol.list_type.push(dummy_ty);
		let new_id = crate::control::identy::AstId::new(crate::control::identy::IdentyKind::Type, 0, self.mol.list_type.len() as u32 - 1);
		self.cloned_types.insert(ty_id, new_id);
		
		let old_ty_ptr = self.mol.get_type(ty_id) as *const Type;
		let old_ty = unsafe { &*old_ty_ptr };
		
		if let TypeVari::Nick(n) = &old_ty.vari {
			let name_str = self.mol.nick_map[n.idx as usize].clone();
			if let Some(&sub_id) = self.sub_map.get(&name_str) {
				return sub_id;
			}
		} else if let TypeVari::UnresolvedPath(p) = &old_ty.vari {
			if p.len() == 1 {
				let name_str = self.mol.nick_map[p[0].idx as usize].clone();
				if let Some(&sub_id) = self.sub_map.get(&name_str) {
					return sub_id;
				}
			}
		}

		let new_vari = match &old_ty.vari {
			TypeVari::PointerOf { sub, acc } => TypeVari::PointerOf { sub: self.clone_type(*sub), acc: *acc },
			TypeVari::ReferenceOf { sub, acc } => TypeVari::ReferenceOf { sub: self.clone_type(*sub), acc: *acc },
			TypeVari::ArrayOf{sub, ext} => TypeVari::ArrayOf{sub: self.clone_type(*sub), ext: ext.clone()},
			TypeVari::Function(f) => {
				let args = f.args.iter().map(|a| crate::ast::types::FieldType {
					name: a.name,
					kind: self.clone_type(a.kind),
					vis: a.vis,
					attrs: a.attrs.clone(),
				}).collect();
				TypeVari::Function(crate::ast::types::FunType {
					args,
					is_static: f.is_static,
					is_const: f.is_const,
					ret: self.clone_type(f.ret),
				})
			},
			TypeVari::Struct(s) => {
				let base = s.base.iter().map(|&b| self.clone_type(b)).collect();
				let vars = s.vars.iter().map(|a| crate::ast::types::FieldType {
					name: a.name,
					kind: self.clone_type(a.kind),
					vis: a.vis,
					attrs: a.attrs.clone(),
				}).collect();
				let funs = s.funs.iter().map(|&f| self.clone_decl(f)).collect();
				TypeVari::Struct(crate::ast::types::StructType { base, vars, funs })
			},
			TypeVari::Iface(i) => {
				let funs = i.funs.iter().map(|a| crate::ast::types::FieldType {
					name: a.name,
					kind: self.clone_type(a.kind),
					vis: a.vis,
					attrs: a.attrs.clone(),
				}).collect();
				TypeVari::Iface(crate::ast::types::IfaceType { funs })
			},
			TypeVari::Trait(t) => {
				let funs = t.funs.iter().map(|a| crate::ast::types::FieldType {
					name: a.name,
					kind: self.clone_type(a.kind),
					vis: a.vis,
					attrs: a.attrs.clone(),
				}).collect();
				TypeVari::Trait(crate::ast::types::TraitType { funs })
			},
			TypeVari::GenericInstance { base, args } => {
				let new_base = self.clone_type(*base);
				let new_args = args.iter().map(|&a| self.clone_type(a)).collect();
				TypeVari::GenericInstance { base: new_base, args: new_args }
			}
			TypeVari::Int{bit, sig} => TypeVari::Int{bit: *bit, sig: *sig},
			TypeVari::Float{bit} => TypeVari::Float{bit: *bit},
			TypeVari::ArchSize{sig} => TypeVari::ArchSize{sig: *sig},
			TypeVari::Bool => TypeVari::Bool,
			TypeVari::Char => TypeVari::Char,
			TypeVari::Ptr => TypeVari::Ptr,
			TypeVari::Void => TypeVari::Void,
			TypeVari::Null => TypeVari::Null,
			TypeVari::Nick(n) => TypeVari::Nick(crate::ast::types::NickType { pos: n.pos, idx: n.idx }),
			TypeVari::SelfType => TypeVari::SelfType,
			TypeVari::UnresolvedPath(p) => {
				let p2 = p.iter().map(|x| crate::ast::types::NickType { pos: x.pos, idx: x.idx }).collect();
				TypeVari::UnresolvedPath(p2)
			}
			TypeVari::Path(p) => TypeVari::Path(p.clone()),
			_ => panic!("Clone not fully implemented for type"),
		};

		let new_ty = Type {
			vari: new_vari,
			state: old_ty.state,
		};

		// Replace the dummy
		*self.mol.get_mut_type(new_id) = new_ty;

		new_id
	}

	pub fn clone_expr(&mut self, expr_id: AstId) -> AstId {
		if expr_id.kind() == IdentyKind::Null { return expr_id; }
		let old_expr_ptr = self.mol.get_expr(expr_id) as *const Expr;
		let old_expr = unsafe { &*old_expr_ptr };

		let new_vari = match &old_expr.vari {
			ExprVari::Block(b) => {
				let ctn = b.ctn.iter().map(|&s| self.clone_stmt(s)).collect();
				ExprVari::Block(crate::ast::exprs::BlockExpr { ctn, label: b.label })
			},
			ExprVari::If(i) => ExprVari::If(crate::ast::exprs::IfExpr {
				cond: self.clone_expr(i.cond),
				then_block: self.clone_expr(i.then_block),
				else_block: i.else_block.map(|e| self.clone_expr(e)),
			}),
			ExprVari::Match(m) => {
				let arms = m.arms.iter().map(|a| crate::ast::exprs::MatchArm {
					pat: self.clone_expr(a.pat),
					body: self.clone_expr(a.body),
				}).collect();
				ExprVari::Match(crate::ast::exprs::MatchExpr {
					val: self.clone_expr(m.val),
					arms,
				})
			},
			ExprVari::Binary(b) => ExprVari::Binary(crate::ast::exprs::BinaryExpr {
				op: b.op,
				lhs: self.clone_expr(b.lhs),
				rhs: self.clone_expr(b.rhs),
			}),
			ExprVari::Unary(u) => ExprVari::Unary(crate::ast::exprs::UnaryExpr {
				op: u.op,
				val: self.clone_expr(u.val),
			}),
			ExprVari::Nick(n) => ExprVari::Nick(crate::ast::exprs::NickExpr { pos: n.pos, idx: n.idx, resolved: n.resolved }),
			ExprVari::Path(p) => ExprVari::Path(p.iter().map(|x| crate::ast::exprs::NickExpr { pos: x.pos, idx: x.idx, resolved: x.resolved }).collect()),
			ExprVari::Number(n) => ExprVari::Number(crate::ast::exprs::NumberExpr { pos: n.pos }),
			ExprVari::Call(c) => ExprVari::Call(crate::ast::exprs::CallExpr {
				callee: self.clone_expr(c.callee),
				args: c.args.iter().map(|&a| self.clone_expr(a)).collect(),
				generic_args: c.generic_args.iter().map(|&g| self.clone_type(g)).collect(),
			}),
			ExprVari::Index(idx) => ExprVari::Index(crate::ast::exprs::IndexExpr {
				target: self.clone_expr(idx.target),
				index: self.clone_expr(idx.index),
			}),
		};

		let new_expr = Expr {
			ty: AstId::null(),
			vari: new_vari,
		};

		self.mol.new_expr(new_expr)
	}

	pub fn clone_stmt(&mut self, stmt_id: AstId) -> AstId {
		if stmt_id.kind() == IdentyKind::Null { return stmt_id; }
		let old_stmt_ptr = self.mol.get_stmt(stmt_id) as *const Stmt;
		let old_stmt = unsafe { &*old_stmt_ptr };

		let new_vari = match &old_stmt.vari {
			StmtVari::Let(l) => StmtVari::Let(crate::ast::stmts::LetStmt {
				name: l.name,
				kind: self.clone_type(l.kind),
				init: l.init.map(|e| self.clone_expr(e)),
				acck: l.acck,
			}),
			StmtVari::Ret(r) => StmtVari::Ret(crate::ast::stmts::RetStmt {
				val: self.clone_expr(r.val),
			}),
			StmtVari::Expr(e) => StmtVari::Expr(crate::ast::stmts::ExprStmt {
				expr: self.clone_expr(e.expr),
			}),
		};

		let new_stmt = Stmt {
			vari: new_vari,
		};

		self.mol.new_stmt(new_stmt)
	}

}
