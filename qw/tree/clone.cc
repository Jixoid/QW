#include "qw/tree/clone.hh"

namespace qw::tree {

  types::Type* Cloner::clone_Type(types::Type* t) {
    if (!t) return nullptr;
    if (type_clone_map.count(t)) return type_clone_map[t];
    
    types::Type* nt = nullptr;

    if (t->is<types::TypeParamType>()) {
      auto param = t->as<types::TypeParamType>();
      if (type_map.count(param->decl)) {
        nt = type_map[param->decl];
      } else {
        nt = t;
      }
      type_clone_map[t] = nt;
      return nt;
    }
    
    if (t->is<types::PointerType>()) {
      auto p = t->as<types::PointerType>();
      nt = types::Type::make_Pointer(ctx, clone_Type(p->sub));
      type_clone_map[t] = nt;
      return nt;
    }
    if (t->is<types::ReferenceType>()) {
      auto p = t->as<types::ReferenceType>();
      nt = types::Type::make_Reference(ctx, clone_Type(p->sub));
      type_clone_map[t] = nt;
      return nt;
    }
    if (t->is<types::ZArrayType>()) {
      auto p = t->as<types::ZArrayType>();
      nt = types::Type::make_ZArray(ctx, clone_Type(p->sub));
      type_clone_map[t] = nt;
      return nt;
    }
    if (t->is<types::PArrayType>()) {
      auto p = t->as<types::PArrayType>();
      nt = types::Type::make_PArray(ctx, clone_Type(p->sub), p->size);
      type_clone_map[t] = nt;
      return nt;
    }
    if (t->is<types::GenericType>()) {
      auto p = t->as<types::GenericType>();
      std::vector<types::Type*> fields;
      for (auto f : p->fields) fields.push_back(clone_Type(f));
      nt = types::Type::make_Generic(ctx, clone_Type(p->sub), std::move(fields));
      type_clone_map[t] = nt;
      return nt;
    }
    if (t->is<types::FuncType>()) {
      auto p = t->as<types::FuncType>();
      std::vector<types::FieldType> pars;
      for (auto &f : p->pars) pars.push_back({f.name, clone_Type(f.type), f.vis});
      nt = types::Type::make_Func(ctx, std::move(pars), clone_Type(p->ret));
      type_clone_map[t] = nt;
      return nt;
    }
    if (t->is<types::StructType>()) {
      auto p = t->as<types::StructType>();
      nt = types::Type::make_Struct(ctx, {}, {}, nullptr, {}, p->baseTypePos, std::string(t->typname()));
      nt->owner_ident = t->owner_ident;
      type_clone_map[t] = nt;
      std::vector<types::Type*> bases;
      for (auto b : p->baseTypes) bases.push_back(clone_Type(b));
      std::vector<types::FieldType> vars, typs;
      for (auto &f : p->vars) vars.push_back({f.name, clone_Type(f.type), f.vis});
      for (auto &f : p->typs) typs.push_back({f.name, clone_Type(f.type), f.vis});
      nt->as<types::StructType>()->decl = clone_Decl(p->decl, p->decl ? (decls::Decl*)p->decl->parent() : nullptr);
      nt->as<types::StructType>()->baseTypes = std::move(bases);
      nt->as<types::StructType>()->vars = std::move(vars);
      nt->as<types::StructType>()->typs = std::move(typs);
      return nt;
    }
    if (t->is<types::EnumType>()) {
      auto p = t->as<types::EnumType>();
      nt = types::Type::make_Enum(ctx, p->vals, {}, nullptr, clone_Type(p->baseType), p->baseTypePos, std::string(t->typname()));
      nt->owner_ident = t->owner_ident;
      type_clone_map[t] = nt;
      std::vector<types::FieldType> typs;
      for (auto &f : p->typs) typs.push_back({f.name, clone_Type(f.type), f.vis});
      nt->as<types::EnumType>()->decl = clone_Decl(p->decl, p->decl ? (decls::Decl*)p->decl->parent() : nullptr);
      nt->as<types::EnumType>()->typs = std::move(typs);
      return nt;
    }
    
    nt = t;
    type_clone_map[t] = nt;
    return nt;
  }

  exprs::Expr* Cloner::clone_Expr(exprs::Expr* e, identy* parent) {
    if (!e) return nullptr;
    if (ident_map.count(e)) return (exprs::Expr*)ident_map[e];

    exprs::Expr* ne = nullptr;

    if (e->is<exprs::IntegerLiteral>() || e->is<exprs::FloatingLiteral>() || 
        e->is<exprs::CharLiteral>() || e->is<exprs::BoolLiteral>() || 
        e->is<exprs::PtrLiteral>() || e->is<exprs::StringLiteral>()) {
      // For literal expressions, we can just reuse them because they don't have sub-expressions.
      // Wait, we need a new instance because the parent might be different.
      if (e->is<exprs::IntegerLiteral>()) {
        auto p = e->as<exprs::IntegerLiteral>();
        if (std::holds_alternative<u64>(p->val))
          ne = exprs::Expr::make_IntegerLiteral(ctx, parent, std::get<u64>(p->val), e->pos());
        else
          ne = exprs::Expr::make_IntegerLiteral(ctx, parent, std::get<i64>(p->val), e->pos());
      }
      else if (e->is<exprs::FloatingLiteral>()) {
        auto p = e->as<exprs::FloatingLiteral>();
        ne = exprs::Expr::make_FloatingLiteral(ctx, parent, p->val, e->pos());
      }
      else if (e->is<exprs::CharLiteral>()) {
        auto p = e->as<exprs::CharLiteral>();
        ne = exprs::Expr::make_CharLiteral(ctx, parent, p->val, e->pos());
      }
      else if (e->is<exprs::BoolLiteral>()) {
        auto p = e->as<exprs::BoolLiteral>();
        ne = exprs::Expr::make_BoolLiteral(ctx, parent, p->val, e->pos());
      }
      else if (e->is<exprs::PtrLiteral>()) {
        auto p = e->as<exprs::PtrLiteral>();
        ne = exprs::Expr::make_PtrLiteral(ctx, parent, p->val, e->pos());
      }
      else if (e->is<exprs::StringLiteral>()) {
        auto p = e->as<exprs::StringLiteral>();
        ne = exprs::Expr::make_StringLiteral(ctx, parent, p->val, e->pos());
      }
      ident_map[e] = ne;
      ne->targetType() = clone_Type(e->targetType());
    }
    else if (e->is<exprs::UnaryOp>()) {
      auto p = e->as<exprs::UnaryOp>();
      ne = exprs::Expr::make_UnaryOp(ctx, parent, p->kind, nullptr, e->pos());
      ident_map[e] = ne;
      ne->as<exprs::UnaryOp>()->o1 = clone_Expr(p->o1, ne);
      ne->targetType() = clone_Type(e->targetType());
    }
    else if (e->is<exprs::BinaryOp>()) {
      auto p = e->as<exprs::BinaryOp>();
      ne = exprs::Expr::make_BinaryOp(ctx, parent, p->kind, nullptr, nullptr, e->pos());
      ident_map[e] = ne;
      auto np = ne->as<exprs::BinaryOp>();
      np->o1 = clone_Expr(p->o1, ne);
      np->o2 = clone_Expr(p->o2, ne);
      np->computationType = clone_Type(p->computationType);
      ne->targetType() = clone_Type(e->targetType());
    }
    else if (e->is<exprs::PostfixOp>()) {
      auto p = e->as<exprs::PostfixOp>();
      ne = exprs::Expr::make_PostfixOp(ctx, parent, p->kind, nullptr, {}, e->pos());
      ident_map[e] = ne;
      auto np = ne->as<exprs::PostfixOp>();
      np->obj = clone_Expr(p->obj, ne);
      for (auto op : p->operands) {
        np->operands.push_back(clone_Expr(op, ne));
      }
      ne->targetType() = clone_Type(e->targetType());
    }
    else if (e->is<exprs::MemberOp>()) {
      auto p = e->as<exprs::MemberOp>();
      ne = exprs::Expr::make_MemberOp(ctx, parent, p->kind, nullptr, nullptr, e->pos());
      ident_map[e] = ne;
      auto np = ne->as<exprs::MemberOp>();
      np->obj = clone_Expr(p->obj, ne);
      np->mem = clone_Expr(p->mem, ne);
      ne->targetType() = clone_Type(e->targetType());
    }
    else if (e->is<exprs::GenericOp>()) {
      auto p = e->as<exprs::GenericOp>();
      ne = exprs::Expr::make_GenericOp(ctx, parent, nullptr, {}, e->pos());
      ident_map[e] = ne;
      auto np = ne->as<exprs::GenericOp>();
      np->obj = clone_Expr(p->obj, ne);
      for (auto t : p->args) {
        np->args.push_back(clone_Type(t));
      }
      ne->targetType() = clone_Type(e->targetType());
    }
    else if (e->is<exprs::VarExpr>()) {
      auto p = e->as<exprs::VarExpr>();
      identy *cloned_var = nullptr;
      if (ident_map.count(p->var)) {
        cloned_var = ident_map[p->var];
      } else {
        cloned_var = p->var; // Unresolved variable reference, could be global or self
      }
      ne = exprs::Expr::make_VarExpr(ctx, parent, cloned_var, e->pos());
      ident_map[e] = ne;
      ne->targetType() = clone_Type(e->targetType());
    }
    else if (e->is<exprs::ValExpr>()) {
      auto p = e->as<exprs::ValExpr>();
      // We pass the same LLVM value and clone its type. But wait! make_ValExpr takes type and value
      ne = exprs::Expr::make_ValExpr(ctx, parent, clone_Type(e->targetType()), e->llvm(), e->pos());
      ident_map[e] = ne;
    }
    else if (e->is<exprs::NickExpr>()) {
      auto p = e->as<exprs::NickExpr>();
      ne = exprs::Expr::make_Nick(ctx, parent, p->unresolved, e->pos());
      ident_map[e] = ne;
      ne->targetType() = clone_Type(e->targetType());
      ne->sema() = e->sema(); // It might have been evaluated already
    }
    else {
      ne = e; // fallback
    }

    return ne;
  }

  stmts::Stmt* Cloner::clone_Stmt(stmts::Stmt* s, identy* parent) {
    if (!s) return nullptr;
    if (ident_map.count(s)) return (stmts::Stmt*)ident_map[s];

    stmts::Stmt* ns = nullptr;

    if (s->is<stmts::CodeBlock>()) {
      auto p = s->as<stmts::CodeBlock>();
      ns = stmts::Stmt::make_CodeBlock(ctx, parent, s->pos());
      ident_map[s] = ns;
      
      auto np = ns->as<stmts::CodeBlock>();
      for (auto v : p->vars) np->vars.push_back(clone_Stmt(v, ns));
      for (auto c : p->codes) np->codes.push_back(clone_Stmt(c, ns));
    }
    else if (s->is<stmts::CodeVar>()) {
      auto p = s->as<stmts::CodeVar>();
      // We do not pass initialy here because CodeVar constructor pushes an ExprStmt assignment if initialy is passed.
      // Wait, CodeBlock AST already contains the ExprStmt for assignment!
      // So we must pass nullptr as initialy, to avoid double assignment generation!
      ns = stmts::Stmt::make_CodeVar(ctx, parent, p->name, clone_Type(p->targetType), s->pos(), nullptr, std::nullopt);
      ident_map[s] = ns;
      ns->as<stmts::CodeVar>()->has_init_expr = p->has_init_expr;
    }
    else if (s->is<stmts::ExprStmt>()) {
      auto p = s->as<stmts::ExprStmt>();
      ns = stmts::Stmt::make_ExprStmt(ctx, parent, nullptr, s->pos());
      ident_map[s] = ns;
      ns->as<stmts::ExprStmt>()->expr = clone_Expr(p->expr, ns);
    }
    else if (s->is<stmts::IfStmt>()) {
      auto p = s->as<stmts::IfStmt>();
      ns = stmts::Stmt::make_IfStmt(ctx, parent, s->pos(), nullptr, nullptr, nullptr);
      ident_map[s] = ns;
      
      auto np = ns->as<stmts::IfStmt>();
      np->condition = clone_Expr(p->condition, ns);
      np->then_block = clone_Stmt(p->then_block, ns);
      np->else_block = clone_Stmt(p->else_block, ns);
    }
    else if (s->is<stmts::WhileStmt>()) {
      auto p = s->as<stmts::WhileStmt>();
      ns = stmts::Stmt::make_WhileStmt(ctx, parent, s->pos(), nullptr, nullptr);
      ident_map[s] = ns;
      
      auto np = ns->as<stmts::WhileStmt>();
      np->condition = clone_Expr(p->condition, ns);
      np->body = clone_Stmt(p->body, ns);
    }
    else if (s->is<stmts::ReturnStmt>()) {
      auto p = s->as<stmts::ReturnStmt>();
      ns = stmts::Stmt::make_Return(ctx, parent, s->pos(), nullptr);
      ident_map[s] = ns;
      ns->as<stmts::ReturnStmt>()->expr = clone_Expr(p->expr, ns);
    }
    else if (s->is<stmts::BreakStmt>()) {
      ns = stmts::Stmt::make_Break(ctx, parent, s->pos());
      ident_map[s] = ns;
    }
    else if (s->is<stmts::ContinueStmt>()) {
      ns = stmts::Stmt::make_Continue(ctx, parent, s->pos());
      ident_map[s] = ns;
    }
    else if (s->is<stmts::UnsafeStmt>()) {
      auto p = s->as<stmts::UnsafeStmt>();
      ns = stmts::Stmt::make_Unsafe(ctx, parent, s->pos(), nullptr);
      ident_map[s] = ns;
      ns->as<stmts::UnsafeStmt>()->stmt = clone_Stmt(p->stmt, ns);
    }
    else {
      ns = s; // fallback
    }

    return ns;
  }

  decls::Decl* Cloner::clone_Decl(decls::Decl* d, decls::Decl* parent) {
    if (!d) return nullptr;
    if (ident_map.count(d)) return (decls::Decl*)ident_map[d];
    
    decls::Decl* nd = nullptr;
    
    if (d->is<decls::StructDecl>()) {
      auto p = d->as<decls::StructDecl>();
      nd = decls::Decl::make_Struct(ctx, parent, d->name(), d->pos(), d->vis());
      ident_map[d] = nd;
      
      auto np = nd->as<decls::StructDecl>();
      for (auto f : p->func) clone_Decl(f, nd);
      for (auto c : p->constructors) clone_Decl(c, nd);
      for (auto d : p->destructors) clone_Decl(d, nd);
    }
    else if (d->is<decls::FuncDecl>()) {
      auto p = d->as<decls::FuncDecl>();
      nd = decls::Decl::make_Func(ctx, parent, d->name(), d->pos(), clone_Type(p->funcType), d->vis());
      ident_map[d] = nd;
      
      auto np = nd->as<decls::FuncDecl>();
      np->body = clone_Stmt(p->body, nd);
    }
    else if (d->is<decls::ConstructorDecl>()) {
      auto p = d->as<decls::ConstructorDecl>();
      nd = decls::Decl::make_Constructor(ctx, parent, d->pos(), clone_Type(p->funcType), d->vis());
      ident_map[d] = nd;
      
      auto np = nd->as<decls::ConstructorDecl>();
      np->body = clone_Stmt(p->body, nd);
    }
    else if (d->is<decls::DestructorDecl>()) {
      auto p = d->as<decls::DestructorDecl>();
      nd = decls::Decl::make_Destructor(ctx, parent, d->pos(), clone_Type(p->funcType), d->vis());
      ident_map[d] = nd;
      
      auto np = nd->as<decls::DestructorDecl>();
      np->body = clone_Stmt(p->body, nd);
    }
    // ... (other decl types can be added later)
    else {
      nd = d; // fallback
    }
    
    if (nd != d && d->is_generic()) {
      // Don't copy generic context for the instantiation! It becomes a concrete type.
      // So no nd->set_generic() needed.
    }
    
    return nd;
  }

}
