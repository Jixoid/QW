/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#include "qw/control/context.hh"
#include "qw/basis.hh"
#include "qw/pretype.hh"
#include "qw/tree/decls.hh"
#include "qw/tree/exprs.hh"
#include "qw/tree/stmts.hh"
#include "qw/tree/types.hh"
#include "qw/vfs/vfs.hh"
#include <llvm/IR/LLVMContext.h>
#include <ostream>
#include <string_view>

#define ef else if

namespace qw
{

  fun operator<<(std::ostream &os, types::Type *typ)->std::ostream &
  {
    os << color::GREEN << "type" << color::GRAY << ":" << color::BLUE << typ->typname() << color::RESET;
    return os;
  }

  fun operator<<(std::ostream &os, identy *ident)->std::ostream &
  {
    auto IDecl = [&os](decls::Decl *now) {
      os << color::GREEN << "decl" << color::GRAY << ":" << color::BLUE;

      if (now->is<decls::NameSpaceDecl>())
        os << "NameSpace";
      ef(now->is<decls::FuncDecl>()) os << "Func";
      ef(now->is<decls::VarDecl>()) os << "Var";
      ef(now->is<decls::TypeDecl>()) os << "Type";
      ef(now->is<decls::AliasDecl>()) os << "Alias";
      ef(now->is<decls::StructDecl>()) os << "Record";

      os << color::RESET;
    };

    auto IStmt = [&os](stmts::Stmt *now) {
      os << color::GREEN << "stmt" << color::GRAY << ":" << color::BLUE;

      if (now->is<stmts::CodeBlock>())
        os << "CodeBlock";
      ef(now->is<stmts::CodeVar>()) os << "CodeVar";
      ef(now->is<stmts::ExprStmt>()) os << "ExprStmt";
      ef(now->is<stmts::ReturnStmt>()) os << "Return";
      ef(now->is<stmts::IfStmt>()) os << "If";
      ef(now->is<stmts::WhileStmt>()) os << "While";
      ef(now->is<stmts::BreakStmt>()) os << "Break";
      ef(now->is<stmts::ContinueStmt>()) os << "Continue";

      os << color::RESET;
    };

    auto IExpr = [&os](exprs::Expr *now) {
      os << color::GREEN << "expr" << color::GRAY << ":" << color::BLUE;

      if (now->is<exprs::IntegerLiteral>())
        os << "IntegerLiteral";
      ef(now->is<exprs::FloatingLiteral>()) os << "FloatingLiteral";
      ef(now->is<exprs::CharLiteral>()) os << "CharLiteral";
      ef(now->is<exprs::BoolLiteral>()) os << "BoolLiteral";
      ef(now->is<exprs::PtrLiteral>()) os << "PtrLiteral";
      ef(now->is<exprs::StringLiteral>()) os << "StringLiteral";
      ef(now->is<exprs::UnaryOp>()) os << "UnaryOp";
      ef(now->is<exprs::BinaryOp>()) os << "BinaryOp";
      ef(now->is<exprs::PostfixOp>()) os << "PostfixOp";
      ef(now->is<exprs::MemberOp>()) os << "MemberOp";
      ef(now->is<exprs::ValExpr>()) os << "ValExpr";
      ef(now->is<exprs::NickExpr>()) os << "Nick";

      os << color::RESET;
    };

    switch (ident->type()) {
      case IdentyEnum::Decl: IDecl((decls::Decl *)ident); break;
      case IdentyEnum::Stmt: IStmt((stmts::Stmt *)ident); break;
      case IdentyEnum::Expr: IExpr((exprs::Expr *)ident); break;
    }

    return os;
  }

  context::context(ProgBits pb)
    : m_progBits(pb)
  {
    m_llvm = make_sptr(new llvm::LLVMContext());

    mf_intU8   = types::Type::make_Primitive(this, types::PrimitiveEnum::U8);
    mf_intU16  = types::Type::make_Primitive(this, types::PrimitiveEnum::U16);
    mf_intU32  = types::Type::make_Primitive(this, types::PrimitiveEnum::U32);
    mf_intU64  = types::Type::make_Primitive(this, types::PrimitiveEnum::U64);
    mf_intU128 = types::Type::make_Primitive(this, types::PrimitiveEnum::U128);
    switch (progBits()) {
      case ProgBits::Bit16: mf_intU0 = mf_intU16; break;
      case ProgBits::Bit32: mf_intU0 = mf_intU32; break;
      case ProgBits::Bit64: mf_intU0 = mf_intU64; break;
    }

    mf_intS8   = types::Type::make_Primitive(this, types::PrimitiveEnum::I8);
    mf_intS16  = types::Type::make_Primitive(this, types::PrimitiveEnum::I16);
    mf_intS32  = types::Type::make_Primitive(this, types::PrimitiveEnum::I32);
    mf_intS64  = types::Type::make_Primitive(this, types::PrimitiveEnum::I64);
    mf_intS128 = types::Type::make_Primitive(this, types::PrimitiveEnum::I128);
    switch (progBits()) {
      case ProgBits::Bit16: mf_intS0 = mf_intS16; break;
      case ProgBits::Bit32: mf_intS0 = mf_intS32; break;
      case ProgBits::Bit64: mf_intS0 = mf_intS64; break;
    }

    mf_flo16  = types::Type::make_Primitive(this, types::PrimitiveEnum::F16);
    mf_flo32  = types::Type::make_Primitive(this, types::PrimitiveEnum::F32);
    mf_flo64  = types::Type::make_Primitive(this, types::PrimitiveEnum::F64);
    mf_flo128 = types::Type::make_Primitive(this, types::PrimitiveEnum::F128);
    switch (progBits()) {
      case ProgBits::Bit16: mf_flo0 = mf_flo16; break;
      case ProgBits::Bit32: mf_flo0 = mf_flo32; break;
      case ProgBits::Bit64: mf_flo0 = mf_flo64; break;
    }

    mf_char = types::Type::make_Primitive(this, types::PrimitiveEnum::Char);

    mf_bool = types::Type::make_Primitive(this, types::PrimitiveEnum::Bool);

    mf_void = types::Type::make_Primitive(this, types::PrimitiveEnum::Void);

    mf_ptr = types::Type::make_Primitive(this, types::PrimitiveEnum::Ptr);
  }

  fun context::SysAPI::call_heap_alloc(context *ctx, exprs::Expr *align, exprs::Expr *size, word pos) -> exprs::Expr* {
      return exprs::Expr::make_PostfixOp(ctx, nil, exprs::PostfixOpEnum::Call, exprs::Expr::make_Nick(ctx, nil, {"sys", "heap", "alloc"}, pos), {align, size}, pos);
  }
  
  fun context::SysAPI::call_heap_dispose(context *ctx, exprs::Expr *p, exprs::Expr *align, exprs::Expr *size, word pos) -> exprs::Expr* {
      return exprs::Expr::make_PostfixOp(ctx, nil, exprs::PostfixOpEnum::Call, exprs::Expr::make_Nick(ctx, nil, {"sys", "heap", "dispose"}, pos), {p, align, size}, pos);
  }
  
  fun context::SysAPI::call_heap_realloc(context *ctx, exprs::Expr *p, exprs::Expr *align, exprs::Expr *old_size, exprs::Expr *new_size, word pos) -> exprs::Expr* {
      return exprs::Expr::make_PostfixOp(ctx, nil, exprs::PostfixOpEnum::Call, exprs::Expr::make_Nick(ctx, nil, {"sys", "heap", "realloc"}, pos), {p, align, old_size, new_size}, pos);
  }

  context::~context()
  {
    for (auto X : m_exprs)
      delete X;
    m_exprs.clear();
    for (auto X : m_stmts)
      delete X;
    m_stmts.clear();
    for (auto X : m_decls)
      delete X;
    m_decls.clear();
    for (auto X : m_types)
      delete X;
    m_types.clear();

    for (auto X : m_modules)
      delete X;
  }

  fun context::make_module(std::string name, std::string fpath) -> module *
  {
    auto obj = new module(this, name, fpath);
    m_modules.push_back(obj);
    return obj;
  }

  module::module(context *ctx, std::string name, std::string fpath) :m_ctx(ctx), m_fpath(fpath), m_mmap(vfs::resolve_map("file://" + fpath)),
    m_llvm(new llvm::Module(name, *ctx->llvm())), m_ns(decls::Decl::make_NameSpace(ctx, nil, "", word{}))
  {}
}
