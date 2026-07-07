#pragma once

#include "qw/tree/types.hh"
#include "qw/tree/decls.hh"
#include "qw/tree/stmts.hh"
#include "qw/tree/exprs.hh"
#include <unordered_map>

namespace qw::tree {

  struct Cloner {
    qw::context *ctx;
    std::unordered_map<decls::Decl*, types::Type*> type_map;
    std::unordered_map<identy*, identy*> ident_map;
    std::unordered_map<types::Type*, types::Type*> type_clone_map;

    Cloner(qw::context *ctx) : ctx(ctx) {}

    types::Type* clone_Type(types::Type* t);
    exprs::Expr* clone_Expr(exprs::Expr* e, identy* parent);
    stmts::Stmt* clone_Stmt(stmts::Stmt* s, identy* parent);
    decls::Decl* clone_Decl(decls::Decl* d, decls::Decl* parent);
  };

}
