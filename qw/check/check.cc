/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#include "qw/basis.hh"
#include "qw/check/check.hh"
#include "qw/pretype.hh"
#include "qw/control/scopemng.hh"
#include "qw/tree/exprs.hh"
#include "qw/tree/types.hh"
#include "qw/tree/decls.hh"
#include "qw/tree/stmts.hh"
#include "qw/diagnostic/diagnostic.hh"
#include "qw/diagnostic/msgs.hh"
#include <expected>
#include <iostream>
#include <llvm/IR/BasicBlock.h>
#include <llvm/IR/Constants.h>
#include <llvm/IR/DerivedTypes.h>
#include <llvm/IR/Function.h>
#include <llvm/IR/GlobalValue.h>
#include <llvm/IR/IRBuilder.h>
#include <llvm/IR/Type.h>
#include <llvm/IR/Value.h>
#include <memory>
#include <string>
#include <unordered_map>
#include <vector>



#define if_error(X) { \
  auto E = X; \
  \
  if (!E.has_value()) \
  { \
    if (E.error()->type() == qw::diagnostic::qMsgType::mtFatal) \
      return std::unexpected(std::move(E.error())); \
    else { \
      sum.add(E.error().get()); \
      std::cerr << E.error(); \
    } \
  } \
}


#define if_except(X) { \
  auto E = X; \
  \
  if (!E.has_value()) \
    return std::unexpected(std::move(E.error())); \
}


#define print(E) { \
  if (E.error()->type() == qw::diagnostic::qMsgType::mtFatal) \
    return std::unexpected(std::move(E.error())); \
  else { \
    sum.add(E.error().get()); \
    std::cerr << E.error(); \
  } \
}


#define val_error(X) { \
  if (!X.has_value()) \
    return std::unexpected(std::move(X.error())); \
}



namespace qw
{

  fun check::check_NameSpace(decls::NameSpace *now) -> std::expected<void, uptr<diagnostic::qMessage>>
  {
    SMng.ans().push_back(scopemng::mangling_abi_qw(now));

    for (auto &X: now->decls())
      if (X->subType() == decls::DeclEnum::NameSpace) if_error(check_NameSpace((decls::NameSpace*)(X)))
      ef (X->subType() == decls::DeclEnum::Func) if_error(check_FuncDecl((decls::Func*)(X)))
      ef (X->subType() == decls::DeclEnum::Type) if_error(check_Type(((decls::Type*)X)->targetType()))
      else
        diagnostic::fatal(fatals::Internal_UnknownDecl().error()->msg());

    SMng.ans().pop_back();
    return {};
  }


  

  fun check::check_FuncDecl(decls::Func *now) -> std::expected<void, uptr<diagnostic::qMessage>>
  {
    if_except(check_FuncType(now->funcType()));

    now->llvm() = llvm::Function::Create((llvm::FunctionType*)now->funcType()->llvm(), llvm::GlobalValue::ExternalLinkage, scopemng::mangling_abi_qw(now), mod->llvm());

    if (now->body()) {
      auto BB = llvm::BasicBlock::Create(*ctx->llvm(), "", now->llvm());
      auto IR = llvm::IRBuilder<> (*ctx->llvm());
      IR.SetInsertPoint(BB);

      if_except(check_CodeBlock(IR, now->funcType(), now->body()));
    }

    return {};
  }

  fun check::check_VarDecl(decls::Var *now) -> std::expected<void, uptr<diagnostic::qMessage>>
  {
    return check_Type(now->varType());
  }




  fun check::check_CodeBlock(llvm::IRBuilder<> &IR, types::Func *llfun, stmts::CodeBlock *now) -> std::expected<void, uptr<diagnostic::qMessage>>
  {
    for (auto X: now->vars()) if_error(check_VarStmt(IR, X));

    for (auto X: now->vars()) if (X->initialy())
    {
      if_except(check_Expr(IR, X->initialy()));

      auto con = check_Convert(IR, X->targetType(), X->initialy(), X->initialy_pos());
      val_error(con);

      IR.CreateStore(*con, X->llvm());
    }


    for (auto &X: now->codes()) switch (X->subType())
    {
      case stmts::StmtEnum::Return: if_error(check_ReturnStmt(IR, llfun, (stmts::Return*)X)); goto l_end;
    
      default:
        diagnostic::fatal(fatals::Internal_UnknownStmt().error()->msg());
    }
    l_end:

    return {};
  }

  fun check::check_VarStmt(llvm::IRBuilder<> &IR, stmts::CodeVar *now) -> std::expected<void, uptr<diagnostic::qMessage>>
  {
    if_except(check_Type(now->targetType()));

    now->llvm() = IR.CreateAlloca(now->targetType()->llvm());

    return {};
  }

  fun check::check_ReturnStmt(llvm::IRBuilder<> &IR, types::Func *llfun, stmts::Return *now) -> std::expected<void, uptr<diagnostic::qMessage>>
  {
    if_except(check_Expr(IR, now->expr()));

    auto con = check_Convert(IR, llfun->ret(), now->expr(), now->pos());
    val_error(con);


    IR.CreateRet(*con);

    return {};
  }




  fun check::check_Convert(llvm::IRBuilder<> &IR, types::Type *typ, exprs::Expr *val, word errpos) -> std::expected<llvm::Value*, uptr<diagnostic::qMessage>>
  {
    auto src = val->llvm();
    auto styp = val->targetType();
    

    l_re:
    if (typ == styp) return src;

    ef (styp->subType() == types::TypeEnum::Reference) {
      styp = static_cast<types::Sub*>(styp)->sub();
      src = IR.CreateLoad(styp->llvm(), src);
      goto l_re;
    }
    
    ef (typ->isInteger() && styp->isInteger()) {
      return IR.CreateIntCast(src, typ->llvm(), typ->isSigned() && styp->isSigned());
    }

    ef (typ->isChar() && styp->isInteger()) {
      return IR.CreateIntCast(src, typ->llvm(), false);
    }
    ef (typ->isInteger() && styp->isChar()) {
      return IR.CreateIntCast(src, typ->llvm(), false);
    }

    else
      return errors::NoMatchOperator(errpos, "=", scopemng::humanreadableTypes(typ), scopemng::humanreadableTypes(val->targetType()));
  }

  


  fun check::check_Expr(llvm::IRBuilder<> &IR, exprs::Expr *&now) -> std::expected<void, uptr<diagnostic::qMessage>>
  {
    l_begin:

    switch (now->subType())
    {
      case exprs::ExprEnum::IntegerLiteral:
      case exprs::ExprEnum::FloatingLiteral: 
      case exprs::ExprEnum::CharLiteral:
      case exprs::ExprEnum::BoolLiteral:
      case exprs::ExprEnum::PtrLiteral: return {};

      case exprs::ExprEnum::ValExpr: return {};

      case exprs::ExprEnum::BinaryOp: {
        auto C = (exprs::BinaryOp*)now;

        if_except(check_Expr(IR, C->o1()));
        if_except(check_Expr(IR, C->o2()));

        
        // Resulation
        auto t1 = C->o1()->targetType(), t2 = C->o2()->targetType();
        auto v1 = C->o1()->llvm(), v2 = C->o2()->llvm();


        l_re:
        // integer operators
        if (t1->isInteger() && t2->isInteger()) {
          types::Type *target{};


          // implicit convertion
          if (t1 != t2)
            target = (t1->intBit() != t2->intBit())
              ? (t1->intBit() > t2->intBit()) ? t1:t2
              : (!t1->isSigned()) ? t1:t2;
          
          if (t1 != target)
            v1 = IR.CreateIntCast(v1, target->llvm(), t1->isSigned());
          else
            v2 = IR.CreateIntCast(v2, target->llvm(), t2->isSigned());


          C->targetType() = target;
          switch (C->kind())
          {
            case exprs::BinaryOpEnum::Add: C->llvm() = IR.CreateAdd(v1, v2); return {};
            case exprs::BinaryOpEnum::Sub: C->llvm() = IR.CreateSub(v1, v2); return {};
            case exprs::BinaryOpEnum::Mul: C->llvm() = IR.CreateMul(v1, v2); return {};
            case exprs::BinaryOpEnum::Div: C->llvm() = target->isSigned() ? IR.CreateSDiv(v1,v2) : IR.CreateUDiv(v1,v2); return {};
            case exprs::BinaryOpEnum::Rem: C->llvm() = target->isSigned() ? IR.CreateSRem(v1,v2) : IR.CreateURem(v1,v2); return {};

            default:
              diagnostic::fatal(fatals::Internal_UnknownExpr().error()->msg());
          }
        }

        // dereference & retry
        if (t1->isReference() || t2->isReference()) {

          if (t1->isReference()) {
            t1 = static_cast<types::Sub*>(t1)->sub();
            v1 = IR.CreateLoad(t1->llvm(), v1);
          }

          if (t2->isReference()) {
            t2 = static_cast<types::Sub*>(t2)->sub();
            v2 = IR.CreateLoad(t2->llvm(), v2);
          }

          goto l_re;
        }



        std::unordered_map<exprs::BinaryOpEnum, std::string> Ops  = {
          {exprs::BinaryOpEnum::Add, "+"},
          {exprs::BinaryOpEnum::Sub, "-"},
          {exprs::BinaryOpEnum::Mul, "*"},
          {exprs::BinaryOpEnum::Div, "/"},
          {exprs::BinaryOpEnum::Rem, "%"},
        };

        return errors::NoMatchOperator(C->pos(), Ops[C->kind()], scopemng::humanreadableTypes(C->o1()->targetType()), scopemng::humanreadableTypes(C->o2()->targetType()));
      }

      case exprs::ExprEnum::Nick: {
        auto ret = check_NickExpr((exprs::NickExpr*)(now));

        if (ret.has_value()) {
          now = *ret;
          goto l_begin;
        } else
          return std::unexpected(std::move(ret.error()));
      }

      default:
        diagnostic::fatal(fatals::Internal_UnknownExpr().error()->msg());
    }
  }

  fun check::check_NickExpr(exprs::NickExpr *now) -> std::expected<exprs::Expr*, uptr<diagnostic::qMessage>>
  {
    const auto __join_human = [](std::vector<std::string> &ps) -> std::string
    {
      std::string ret = ps[0];

      for (int i = 1; i < ps.size(); i++)
        ret += "::" +ps[i];

      return ret;
    };

    
    auto ret = SMng.lookup(now->parent(), now->unResolvedName());

    if (!ret)
      return errors::IdentifierNotFound(now->pos(), __join_human(now->unResolvedName()));

    ef (auto expr = SMng.fetch_expr(ret))
      return expr;

    else
      return errors::IdentifierNExpr(now->pos(), __join_human(now->unResolvedName()));
  }




  fun check::check_Type(types::Type *&now) -> std::expected<void, uptr<diagnostic::qMessage>>
  {
    l_begin:
    if (now->subType() >= types::TypeEnum::IntU8)
      return {};

    
    switch (now->subType())
    {
      case types::TypeEnum::Record: return check_RecordType((types::Record*)(now));
      case types::TypeEnum::Func: return check_FuncType((types::Func*)(now));
      
      case types::TypeEnum::Nick: {
        auto ret = check_NickType((types::Nick*)(now));

        if (ret.has_value()) {
          now = *ret;
          goto l_begin;
        } else
          return std::unexpected(std::move(ret.error()));
      }
      
      default:
        diagnostic::fatal(fatals::Internal_UnknownType().error()->msg());
    }
  }

  fun check::check_RecordType(types::Record *now) -> std::expected<void, uptr<diagnostic::qMessage>>
  {
    // Check Sub
    for (auto &X: now->vars()) if_except(check_FieldMember(X));


    // LLVM
    std::vector<llvm::Type*> body;
    
    for (auto &X: now->vars())
      body.push_back(X->targetType()->llvm());
  
    now->llvm() = llvm::StructType::create(*ctx->llvm(), body);

    /*
    // Layout Sub
    struct member {
      u0 size, align, offset;
    };

    std::vector<member> Vars;
    Vars.reserve(now->m_vars.size());

    for (auto &X: now->m_vars)
      Vars.push_back({
        .size  = X->size(),
        .align = X->align(),
      });


    if (now->attr_sorted && !now->attr_packed)
      std::stable_sort(Vars.begin(), Vars.end(), [](const member &a, const member &b) {
        return a.align > b.align;
      });
    

    u0 currentOffset{}, maxAlign{1};

    for (auto &var: Vars) {
      u0 effectiveAlign = now->attr_packed ? 1 : var.align;
      
      maxAlign = std::max(maxAlign, effectiveAlign);

      if (currentOffset % effectiveAlign != 0)
        currentOffset += (effectiveAlign - (currentOffset % effectiveAlign));

      var.offset = currentOffset;
      currentOffset += var.size;
    }

    
    u0 structSize = currentOffset;
    if (!now->attr_packed && structSize % maxAlign != 0)
      structSize += (maxAlign - (structSize % maxAlign));

    now->Size = structSize;
    now->Align = maxAlign;

    now->Stage_2 = true;
    */

    return {};
  }

  fun check::check_NickType(types::Nick *now) -> std::expected<types::Type*, uptr<diagnostic::qMessage>>
  {
    const auto __join_human = [](std::vector<std::string> &ps) -> std::string
    {
      std::string ret = ps[0];

      for (int i = 1; i < ps.size(); i++)
        ret += "::" +ps[i];

      return ret;
    };


    auto ret = SMng.lookup(now->parent(), now->unResolvedName());


    if (!ret)
      return errors::IdentifierNotFound(now->pos(), __join_human(now->unResolvedName()));

    ef (auto typ = SMng.fetch_type(ret))
      return typ;

    else
      return errors::IdentifierNType(now->pos(), __join_human(now->unResolvedName()));
  }
  
  fun check::check_FuncType(types::Func *now) -> std::expected<void, uptr<diagnostic::qMessage>>
  {
    // Check Sub
    for (auto &X: now->pars()) if_except(check_FieldMember(X));
    if_except(check_Type(now->ret()));


    // LLVM Types
    std::vector<llvm::Type*> LLTypes;
    LLTypes.reserve(now->pars().size());
    for (auto X: now->pars())
      LLTypes.push_back(X->targetType()->llvm());


    // Create
    now->llvm() = llvm::FunctionType::get(now->ret()->llvm(), LLTypes, false);

    return {};
  }




  fun check::check_FieldMember(types::MemberField *now) -> std::expected<void, uptr<diagnostic::qMessage>>
  {
    return check_Type(now->targetType());
  }

}
