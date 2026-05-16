/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#include "qw/basis.hh"
#include "qw/control/context.hh"
#include "qw/pretype.hh"
#include "qw/tree/types.hh"
#include "qw/tree/exprs.hh"
#include "qw/tree/stmts.hh"
#include "qw/tree/decls.hh"
#include "qw/vfs/vfs.hh"
#include <llvm/IR/LLVMContext.h>
#include <ostream>
#include <string_view>

#define ef else if



namespace qw
{
  
  fun operator <<(std::ostream &os, identy *ident) -> std::ostream&
  {
    auto IDecl = [&os](decls::Decl *now)
    {
      os << color::GREEN << "decl" << color::GRAY << ":" << color::BLUE;

      switch (now->subType())
      {
        case decls::DeclEnum::Module: os << "Module"; break;
        case decls::DeclEnum::NameSpace: os << "NameSpace"; break;
        case decls::DeclEnum::Func: os << "Func"; break;
        case decls::DeclEnum::Var: os << "Var"; break;
        case decls::DeclEnum::Type: os << "Type"; break;
        case decls::DeclEnum::Alias: os << "Alias"; break;
      }

      os << color::RESET;
    };

    auto IType = [&os](types::Type *now)
    {
      os << color::GREEN << "type" << color::GRAY << ":" << color::BLUE;

      switch (now->subType())
      {
        case types::TypeEnum::Pointer: os << "Pointer"; break;
        case types::TypeEnum::Reference: os << "Reference"; break;
        case types::TypeEnum::CArray: os << "CArray"; break;

        case types::TypeEnum::Nick: os << "Nick"; break;
        case types::TypeEnum::Func: os << "Func"; break;
        case types::TypeEnum::Record: os << "Record"; break;

        case types::TypeEnum::MemberField: os << "MemberField"; break;


        // Primitive
        case types::TypeEnum::IntU8: os << "IntU8"; break;
        case types::TypeEnum::IntU16: os << "IntU16"; break;
        case types::TypeEnum::IntU32: os << "IntU32"; break;
        case types::TypeEnum::IntU64: os << "IntU64"; break;
        case types::TypeEnum::IntU128: os << "IntU128"; break;

        case types::TypeEnum::IntS8: os << "IntS8"; break;
        case types::TypeEnum::IntS16: os << "IntS16"; break;
        case types::TypeEnum::IntS32: os << "IntS32"; break;
        case types::TypeEnum::IntS64: os << "IntS64"; break;
        case types::TypeEnum::IntS128: os << "IntS128"; break;

        case types::TypeEnum::Flo16: os << "Flo16"; break;
        case types::TypeEnum::Flo32: os << "Flo32"; break;
        case types::TypeEnum::Flo64: os << "Flo64"; break;
        case types::TypeEnum::Flo128: os << "Flo128"; break;

        case types::TypeEnum::Char: os << "Char"; break;

        case types::TypeEnum::Bool: os << "Bool"; break;

        case types::TypeEnum::Void: os << "Void"; break;

        case types::TypeEnum::Ptr: os << "Ptr"; break;
      }

      os << color::RESET;
    };

    auto IStmt = [&os](stmts::Stmt *now)
    {
      os << color::GREEN << "stmt" << color::GRAY << ":" << color::BLUE;

      switch (now->subType())
      {
        case stmts::StmtEnum::CodeBlock: os << "CodeBlock"; break;
        case stmts::StmtEnum::CodeVar: os << "CodeVar"; break;
        
        case stmts::StmtEnum::Return: os << "Return"; break;
        
        case stmts::StmtEnum::ExprStmt: os << "ExprStmt"; break;
      }

      os << color::RESET;
    };

    auto IExpr = [&os](exprs::Expr *now)
    {
      os << color::GREEN << "expr" << color::GRAY << ":" << color::BLUE;

      switch (now->subType())
      {
        case exprs::ExprEnum::IntegerLiteral: os << "IntegerLiteral"; break;
        case exprs::ExprEnum::FloatingLiteral: os << "FloatingLiteral"; break;
        case exprs::ExprEnum::CharLiteral: os << "CharLiteral"; break;
        case exprs::ExprEnum::BoolLiteral: os << "BoolLiteral"; break;
        case exprs::ExprEnum::PtrLiteral: os << "PtrLiteral"; break;

        case exprs::ExprEnum::ValExpr: os << "ValExpr"; break;

        case exprs::ExprEnum::Nick: os << "Nick"; break;

        case exprs::ExprEnum::BinaryOp: os << "BinaryOp"; break;
        case exprs::ExprEnum::PrimaryOp: os << "PrimaryOp"; break;
      }

      os << color::RESET;
    };



    switch (ident->type())
    {
      case IdentyEnum::Decl: IDecl((decls::Decl*)ident); break;
      case IdentyEnum::Type: IType((types::Type*)ident); break;
      case IdentyEnum::Stmt: IStmt((stmts::Stmt*)ident); break;
      case IdentyEnum::Expr: IExpr((exprs::Expr*)ident); break;
    }

    return os;
  }



  context::context(ProgBits pb): m_progBits(pb)
  {
    m_llvm = make_sptr(new llvm::LLVMContext());

    
    mf_intU8   = types::Type::make_intU8(this, nil);
    mf_intU16  = types::Type::make_intU16(this, nil);
    mf_intU32  = types::Type::make_intU32(this, nil);
    mf_intU64  = types::Type::make_intU64(this, nil);
    mf_intU128 = types::Type::make_intU128(this, nil);
    switch (progBits()) {
      case ProgBits::Bit16: mf_intU0 = mf_intU16;
      case ProgBits::Bit32: mf_intU0 = mf_intU32;
      case ProgBits::Bit64: mf_intU0 = mf_intU64;
    }
    
    mf_intS8   = types::Type::make_intS8(this, nil);
    mf_intS16  = types::Type::make_intS16(this, nil);
    mf_intS32  = types::Type::make_intS32(this, nil);
    mf_intS64  = types::Type::make_intS64(this, nil);
    mf_intS128 = types::Type::make_intS128(this, nil);
    switch (progBits()) {
      case ProgBits::Bit16: mf_intS0 = mf_intS16;
      case ProgBits::Bit32: mf_intS0 = mf_intS32;
      case ProgBits::Bit64: mf_intS0 = mf_intS64;
    }

    mf_flo16  = types::Type::make_float16(this, nil);
    mf_flo32  = types::Type::make_float32(this, nil);
    mf_flo64  = types::Type::make_float64(this, nil);
    mf_flo128 = types::Type::make_float128(this, nil);
    switch (progBits()) {
      case ProgBits::Bit16: mf_flo0 = mf_flo16;
      case ProgBits::Bit32: mf_flo0 = mf_flo32;
      case ProgBits::Bit64: mf_flo0 = mf_flo64;
    }
    
    mf_char   = types::Type::make_char(this, nil);

    mf_bool   = types::Type::make_bool(this, nil);

    mf_void   = types::Type::make_void(this, nil);

    switch (progBits()) {
      case ProgBits::Bit16: mf_ptr = types::Type::make_intU16(this, nil);;
      case ProgBits::Bit32: mf_ptr = types::Type::make_intU32(this, nil);;
      case ProgBits::Bit64: mf_ptr = types::Type::make_intU64(this, nil);;
    }
  }

  context::~context()
  {
    for (auto X: m_exprs) X->dis(); m_exprs.clear();
    for (auto X: m_stmts) X->dis(); m_stmts.clear();
    for (auto X: m_decls) X->dis(); m_decls.clear();
    for (auto X: m_types) X->dis(); m_types.clear();


    for (auto X: m_modules) delete X;
  }



  fun context::make_module(std::string name, std::string fpath) -> module*
  {
    auto obj = new module(this, name, fpath);
    m_modules.push_back(obj);
    return obj;
  }
  


  module::module(context *ctx, std::string name, std::string fpath)
    : m_ctx(ctx)
    , m_fpath(fpath)
    , m_mmap(vfs::resolve_map("file://"+fpath))
    , m_llvm(new llvm::Module(name, *ctx->llvm()))
    , m_ns(decls::Decl::make_NameSpace(ctx, nil, "", word{}))
  {}
}
