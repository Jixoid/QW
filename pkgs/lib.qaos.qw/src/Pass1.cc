/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#include "Pass1.hh"
#include "Basis.hh"
#include "Exprs.hh"
#include "ScopeManager.hh"
#include "Types.hh"
#include "Decls.hh"
#include "Stmts.hh"
#include "Diagnostic.hh"
#include "Typs.hh"
#include "Msgs.hh"
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
      return unexpected(std::move(E.error())); \
    else { \
      sum.add(E.error().get()); \
      cerr << E.error(); \
    } \
  } \
}


#define if_except(X) { \
  auto E = X; \
  \
  if (!E.has_value()) \
    return unexpected(std::move(E.error())); \
}


#define print(E) { \
  if (E.error()->type() == qw::diagnostic::qMsgType::mtFatal) \
    return unexpected(std::move(E.error())); \
  else { \
    sum.add(E.error().get()); \
    cerr << E.error(); \
  } \
}


#define val_error(X) { \
  if (!X.has_value()) \
    return unexpected(std::move(X.error())); \
}



namespace qw
{

  fun pass_1::check_NameSpace(decls::qNameSpaceDecl *now) -> expected<void, uptr<diagnostic::qMessage>>
  {
    SMng.ans().push_back(scopeManager::mangling_abi_qw(now));

    for (auto &X: now->decls())
      if (X->subType() == decls::qDecl::qde_NameSpaceDecl) if_error(check_NameSpace((decls::qNameSpaceDecl*)(X)))
      ef (X->subType() == decls::qDecl::qde_FuncDecl) if_error(check_FuncDecl((decls::qFuncDecl*)(X)))
      ef (X->subType() == decls::qDecl::qde_TypeDecl) if_error(check_Type(((decls::qTypeDecl*)X)->targetType()))
      else
        diagnostic::fatal(fatals::Internal_UnknownDecl().error()->msg());

    SMng.ans().pop_back();
    return {};
  }


  

  fun pass_1::check_FuncDecl(decls::qFuncDecl *now) -> expected<void, uptr<diagnostic::qMessage>>
  {
    if_except(check_FuncType(now->funcType()));

    now->llvm() = llvm::Function::Create((llvm::FunctionType*)now->funcType()->llvm(), llvm::GlobalValue::ExternalLinkage, scopeManager::mangling_abi_qw(now), mod->llvm());

    if (now->body()) {
      auto BB = llvm::BasicBlock::Create(*ctx->llvm(), "", now->llvm());
      auto IR = llvm::IRBuilder<> (*ctx->llvm());
      IR.SetInsertPoint(BB);

      if_except(check_CodeBlock(IR, now->funcType(), now->body()));
    }

    return {};
  }

  fun pass_1::check_VarDecl(decls::qVarDecl *now) -> expected<void, uptr<diagnostic::qMessage>>
  {
    return check_Type(now->varType());
  }




  fun pass_1::check_CodeBlock(llvm::IRBuilder<> &IR, types::qFuncType *llfun, stmts::qCodeBlock *now) -> expected<void, uptr<diagnostic::qMessage>>
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
      case stmts::qStmt::qse_Return: if_error(check_ReturnStmt(IR, llfun, (stmts::qReturn*)X)); goto l_end;
    
      default:
        diagnostic::fatal(fatals::Internal_UnknownStmt().error()->msg());
    }
    l_end:

    return {};
  }

  fun pass_1::check_VarStmt(llvm::IRBuilder<> &IR, stmts::qCodeVar *now) -> expected<void, uptr<diagnostic::qMessage>>
  {
    if_except(check_Type(now->targetType()));

    now->llvm() = IR.CreateAlloca(now->targetType()->llvm());

    return {};
  }

  fun pass_1::check_ReturnStmt(llvm::IRBuilder<> &IR, types::qFuncType *llfun, stmts::qReturn *now) -> expected<void, uptr<diagnostic::qMessage>>
  {
    if_except(check_Expr(IR, now->expr()));

    auto con = check_Convert(IR, llfun->ret(), now->expr(), now->pos());
    val_error(con);


    IR.CreateRet(*con);

    return {};
  }




  fun pass_1::check_Convert(llvm::IRBuilder<> &IR, types::qType *typ, exprs::qExpr *val, word errpos) -> expected<llvm::Value*, uptr<diagnostic::qMessage>>
  {
    auto src = val->llvm();
    auto styp = val->targetType();
    

    l_re:
    if (typ == styp) return src;

    ef (styp->subType() == types::qType::qte_Reference) {
      styp = static_cast<types::qSubType*>(styp)->sub();
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
      return errors::NoMatchOperator(errpos, "=", scopeManager::humanreadableTypes(typ), scopeManager::humanreadableTypes(val->targetType()));
  }

  


  fun pass_1::check_Expr(llvm::IRBuilder<> &IR, exprs::qExpr *&now) -> expected<void, uptr<diagnostic::qMessage>>
  {
    l_begin:

    switch (now->subType())
    {
      case exprs::qExpr::qee_IntegerLiteral:
      case exprs::qExpr::qee_FloatingLiteral: 
      case exprs::qExpr::qee_CharLiteral:
      case exprs::qExpr::qee_BoolLiteral:
      case exprs::qExpr::qee_PtrLiteral: return {};

      case exprs::qExpr::qee_ValExpr: return {};

      case exprs::qExpr::qee_BinaryOp: {
        auto C = (exprs::qBinaryOp*)now;

        if_except(check_Expr(IR, C->o1()));
        if_except(check_Expr(IR, C->o2()));

        
        // Resulation
        auto t1 = C->o1()->targetType(), t2 = C->o2()->targetType();
        auto v1 = C->o1()->llvm(), v2 = C->o2()->llvm();


        l_re:
        // integer operators
        if (t1->isInteger() && t2->isInteger()) {
          types::qType *target{};


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
            case exprs::boe_Add: C->llvm() = IR.CreateAdd(v1, v2); return {};
            case exprs::boe_Sub: C->llvm() = IR.CreateSub(v1, v2); return {};
            case exprs::boe_Mul: C->llvm() = IR.CreateMul(v1, v2); return {};
            case exprs::boe_Div: C->llvm() = target->isSigned() ? IR.CreateSDiv(v1,v2) : IR.CreateUDiv(v1,v2); return {};
            case exprs::boe_Rem: C->llvm() = target->isSigned() ? IR.CreateSRem(v1,v2) : IR.CreateURem(v1,v2); return {};

            default:
              diagnostic::fatal(fatals::Internal_UnknownExpr().error()->msg());
          }
        }

        // dereference & retry
        if (t1->isReference() || t2->isReference()) {

          if (t1->isReference()) {
            t1 = static_cast<types::qSubType*>(t1)->sub();
            v1 = IR.CreateLoad(t1->llvm(), v1);
          }

          if (t2->isReference()) {
            t2 = static_cast<types::qSubType*>(t2)->sub();
            v2 = IR.CreateLoad(t2->llvm(), v2);
          }

          goto l_re;
        }



        unordered_map<exprs::qBinaryOpEnum, string> Ops  = {
          {exprs::boe_Add, "+"},
          {exprs::boe_Sub, "-"},
          {exprs::boe_Mul, "*"},
          {exprs::boe_Div, "/"},
          {exprs::boe_Rem, "%"},
        };

        return errors::NoMatchOperator(C->pos(), Ops[C->kind()], scopeManager::humanreadableTypes(C->o1()->targetType()), scopeManager::humanreadableTypes(C->o2()->targetType()));
      }

      case exprs::qExpr::qee_Nick: {
        auto ret = check_NickExpr((exprs::qNickExpr*)(now));

        if (ret.has_value()) {
          now = *ret;
          goto l_begin;
        } else
          return unexpected(std::move(ret.error()));
      }

      default:
        diagnostic::fatal(fatals::Internal_UnknownExpr().error()->msg());
    }
  }

  fun pass_1::check_NickExpr(exprs::qNickExpr *now) -> expected<exprs::qExpr*, uptr<diagnostic::qMessage>>
  {
    const auto __join_human = [](vector<string> &ps) -> string
    {
      string ret = ps[0];

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




  fun pass_1::check_Type(types::qType *&now) -> expected<void, uptr<diagnostic::qMessage>>
  {
    l_begin:
    if (now->subType() >= types::qType::qte_IntU8)
      return {};

    
    switch (now->subType())
    {
      case types::qType::qte_Record: return check_RecordType((types::qRecordType*)(now));
      case types::qType::qte_Func: return check_FuncType((types::qFuncType*)(now));
      
      case types::qType::qte_Nick: {
        auto ret = check_NickType((types::qNickType*)(now));

        if (ret.has_value()) {
          now = *ret;
          goto l_begin;
        } else
          return unexpected(std::move(ret.error()));
      }
      
      default:
        diagnostic::fatal(fatals::Internal_UnknownType().error()->msg());
    }
  }

  fun pass_1::check_RecordType(types::qRecordType *now) -> expected<void, uptr<diagnostic::qMessage>>
  {
    // Check Sub
    for (auto &X: now->vars()) if_except(check_FieldMember(X));


    // LLVM
    vector<llvm::Type*> body;
    
    for (auto &X: now->vars())
      body.push_back(X->targetType()->llvm());
  
    now->llvm() = llvm::StructType::create(*ctx->llvm(), body);

    /*
    // Layout Sub
    struct member {
      u0 size, align, offset;
    };

    vector<member> Vars;
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

  fun pass_1::check_NickType(types::qNickType *now) -> expected<types::qType*, uptr<diagnostic::qMessage>>
  {
    const auto __join_human = [](vector<string> &ps) -> string
    {
      string ret = ps[0];

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
  
  fun pass_1::check_FuncType(types::qFuncType *now) -> expected<void, uptr<diagnostic::qMessage>>
  {
    // Check Sub
    for (auto &X: now->pars()) if_except(check_FieldMember(X));
    if_except(check_Type(now->ret()));


    // LLVM Types
    vector<llvm::Type*> LLTypes;
    LLTypes.reserve(now->pars().size());
    for (auto X: now->pars())
      LLTypes.push_back(X->targetType()->llvm());


    // Create
    now->llvm() = llvm::FunctionType::get(now->ret()->llvm(), LLTypes, false);

    return {};
  }




  fun pass_1::check_FieldMember(types::qMemberFieldType *now) -> expected<void, uptr<diagnostic::qMessage>>
  {
    return check_Type(now->targetType());
  }

}
