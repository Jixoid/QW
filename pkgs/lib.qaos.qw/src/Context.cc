/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#include "Basis.hh"
#include "Context.hh"
#include "Types.hh"
#include "Typs.hh"
#include "Exprs.hh"
#include "Stmts.hh"
#include "Decls.hh"
#include <llvm/IR/LLVMContext.h>

#define ef else if

using namespace std;



namespace qw
{
  
  fun operator <<(ostream &os, qIdentifier *ident) -> ostream&
  {
    auto IDecl = [&os](decls::qDecl *now)
    {
      os << color::GREEN << "decl" << color::GRAY << ":" << color::BLUE;

      switch (now->subType()) {
        case decls::qDecl::qde_Unknown: os << color::RED << "<unknown>"; break;

        case decls::qDecl::qde_ModuleDecl: os << "Module"; break;
        case decls::qDecl::qde_NameSpaceDecl: os << "NameSpace"; break;
        case decls::qDecl::qde_FuncDecl: os << "Func"; break;
        case decls::qDecl::qde_VarDecl: os << "Var"; break;
        case decls::qDecl::qde_TypeDecl: os << "Type"; break;
        case decls::qDecl::qde_AliasDecl: os << "Alias"; break;
      }

      os << color::RESET;
    };

    auto IType = [&os](types::qType *now)
    {
      os << color::GREEN << "type" << color::GRAY << ":" << color::BLUE;

      switch (now->subType()) {
        case types::qType::qte_Unknown: os << color::RED << "<unknown>"; break;

        case types::qType::qte_Pointer: os << "Pointer"; break;
        case types::qType::qte_Reference: os << "Reference"; break;
        case types::qType::qte_CArray: os << "CArray"; break;

        case types::qType::qte_Nick: os << "Nick"; break;
        case types::qType::qte_Func: os << "Func"; break;
        case types::qType::qte_Record: os << "Record"; break;

        case types::qType::qte_MemberField: os << "MemberField"; break;


        // Primitive
        case types::qType::qte_IntU8: os << "IntU8"; break;
        case types::qType::qte_IntU16: os << "IntU16"; break;
        case types::qType::qte_IntU32: os << "IntU32"; break;
        case types::qType::qte_IntU64: os << "IntU64"; break;
        case types::qType::qte_IntU128: os << "IntU128"; break;

        case types::qType::qte_IntS8: os << "IntS8"; break;
        case types::qType::qte_IntS16: os << "IntS16"; break;
        case types::qType::qte_IntS32: os << "IntS32"; break;
        case types::qType::qte_IntS64: os << "IntS64"; break;
        case types::qType::qte_IntS128: os << "IntS128"; break;

        case types::qType::qte_Flo16: os << "Flo16"; break;
        case types::qType::qte_Flo32: os << "Flo32"; break;
        case types::qType::qte_Flo64: os << "Flo64"; break;
        case types::qType::qte_Flo128: os << "Flo128"; break;

        case types::qType::qte_Char: os << "Char"; break;

        case types::qType::qte_Bool: os << "Bool"; break;

        case types::qType::qte_Void: os << "Void"; break;

        case types::qType::qte_Ptr: os << "Ptr"; break;
      }

      os << color::RESET;
    };

    auto IStmt = [&os](stmts::qStmt *now)
    {
      os << color::GREEN << "stmt" << color::GRAY << ":" << color::BLUE;

      switch (now->subType()) {
        case stmts::qStmt::qse_Unknown: os << color::RED << "<unknown>"; break;

        case stmts::qStmt::qse_CodeBlock: os << "CodeBlock"; break;
        case stmts::qStmt::qse_CodeVar: os << "CodeVar"; break;
        
        case stmts::qStmt::qse_Return: os << "Return"; break;
        
        case stmts::qStmt::qse_ExprStmt: os << "ExprStmt"; break;
      }

      os << color::RESET;
    };

    auto IExpr = [&os](exprs::qExpr *now)
    {
      os << color::GREEN << "expr" << color::GRAY << ":" << color::BLUE;

      switch (now->subType()) {
        case exprs::qExpr::qee_Unknown: os << color::RED << "<unknown>"; break;

        case exprs::qExpr::qee_IntegerLiteral: os << "IntegerLiteral"; break;
        case exprs::qExpr::qee_FloatingLiteral: os << "FloatingLiteral"; break;
        case exprs::qExpr::qee_CharLiteral: os << "CharLiteral"; break;
        case exprs::qExpr::qee_BoolLiteral: os << "BoolLiteral"; break;
        case exprs::qExpr::qee_PtrLiteral: os << "PtrLiteral"; break;

        case exprs::qExpr::qee_ValExpr: os << "ValExpr"; break;

        case exprs::qExpr::qee_Nick: os << "Nick"; break;

        case exprs::qExpr::qee_BinaryOp: os << "BinaryOp"; break;
        case exprs::qExpr::qee_PrimaryOp: os << "PrimaryOp"; break;
      }

      os << color::RESET;
    };



    switch (ident->type())
    {
      case qIdentifier::qie_Unkown: os << color::RED << "<unknown>"; break;

      case qIdentifier::qie_Decl: IDecl((decls::qDecl*)ident); break;
      case qIdentifier::qie_Type: IType((types::qType*)ident); break;
      case qIdentifier::qie_Stmt: IStmt((stmts::qStmt*)ident); break;
      case qIdentifier::qie_Expr: IExpr((exprs::qExpr*)ident); break;
    }

    return os;
  }



  context::context(qProgBits pb): m_progBits(pb)
  {
    m_llvm = make_sptr(new llvm::LLVMContext());

    
    mf_intU8   = types::qType::make_intU8(this, Nil);
    mf_intU16  = types::qType::make_intU16(this, Nil);
    mf_intU32  = types::qType::make_intU32(this, Nil);
    mf_intU64  = types::qType::make_intU64(this, Nil);
    mf_intU128 = types::qType::make_intU128(this, Nil);
    switch (progBits()) {
      case pb16: mf_intU0 = mf_intU16;
      case pb32: mf_intU0 = mf_intU32;
      case pb64: mf_intU0 = mf_intU64;
    }
    
    mf_intS8   = types::qType::make_intS8(this, Nil);
    mf_intS16  = types::qType::make_intS16(this, Nil);
    mf_intS32  = types::qType::make_intS32(this, Nil);
    mf_intS64  = types::qType::make_intS64(this, Nil);
    mf_intS128 = types::qType::make_intS128(this, Nil);
    switch (progBits()) {
      case pb16: mf_intS0 = mf_intS16;
      case pb32: mf_intS0 = mf_intS32;
      case pb64: mf_intS0 = mf_intS64;
    }

    mf_flo16  = types::qType::make_float16(this, Nil);
    mf_flo32  = types::qType::make_float32(this, Nil);
    mf_flo64  = types::qType::make_float64(this, Nil);
    mf_flo128 = types::qType::make_float128(this, Nil);
    switch (progBits()) {
      case pb16: mf_flo0 = mf_flo16;
      case pb32: mf_flo0 = mf_flo32;
      case pb64: mf_flo0 = mf_flo64;
    }
    
    mf_char   = types::qType::make_char(this, Nil);

    mf_bool   = types::qType::make_bool(this, Nil);

    mf_void   = types::qType::make_void(this, Nil);

    switch (progBits()) {
      case pb16: mf_ptr = types::qType::make_intU16(this, Nil);;
      case pb32: mf_ptr = types::qType::make_intU32(this, Nil);;
      case pb64: mf_ptr = types::qType::make_intU64(this, Nil);;
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



  fun context::make_module(string name, string fpath) -> module*
  {
    auto obj = new module(this, name, fpath);
    m_modules.push_back(obj);
    return obj;
  }
  


  module::module(context *ctx, string name, string fpath)
    : m_ctx(ctx)
    , m_fpath(fpath)
    , m_mmap(fpath)
    , m_llvm(new llvm::Module(name, *ctx->llvm()))
    , m_ns(decls::qDecl::make_NameSpace(ctx, Nil, "", word{}))
  {}
}
