/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#include "Context.hh"
#include "Front.hh"
#include "Pass1.hh"
#include "ScopeManager.hh"
#include "Types.hh"
#include <iostream>
#include <llvm/IR/Type.h>

using namespace std;
using namespace qw;



int main()
{
  setlocale(LC_ALL, "");
  bindtextdomain("main", "/home/alforce/Projeler/QW26/pkgs/lib.qaos.qw/i18n");
  textdomain("main");



  // Prepare
  auto ctx = context::make(qProgBits::pb64);
  auto mod = ctx->make_module("Test.qw", "pkgs/com.test.qw/src/Test.qw");


  // QLang system unit
  auto sys = decls::qDecl::make_NameSpace(ctx.get(), Nil, "sys", word{});
  
  l_sys: {
    #define NewSys(Name, Type) { \
      auto tdecl = decls::qDecl::make_Type(ctx.get(), sys, #Name, word{}, Type); \
      ctx->gst().add_ident(scopeManager::mangling_abi_qw(Type), Type); \
    }


    // Types
    NewSys(u8,   ctx->intU8_t())
    NewSys(u16,  ctx->intU16_t())
    NewSys(u32,  ctx->intU32_t())
    NewSys(u64,  ctx->intU64_t())
    NewSys(u128, ctx->intU128_t())

    NewSys(i8,   ctx->intS8_t())
    NewSys(i16,  ctx->intS16_t())
    NewSys(i32,  ctx->intS32_t())
    NewSys(i64,  ctx->intS64_t())
    NewSys(i128, ctx->intS128_t())

    NewSys(f16,  ctx->flo16_t())
    NewSys(f32,  ctx->flo32_t())
    NewSys(f64,  ctx->flo64_t())
    NewSys(f128, ctx->flo128_t())

    NewSys(char, ctx->char_t())

    NewSys(bool, ctx->bool_t())

    NewSys(void, ctx->void_t())
    
    NewSys(ptr,  ctx->ptr_t())

    #undef NewSys
  }




  // Pass 0 (Read & Discovery)
  cerr << color::RED << "# PASS 0" << color::RESET << endl;
  l_pass_0: {
    auto Sum = frontend(mod).process();
    
    if (Sum.sum()) {
      cerr << Sum;
      goto l_final;
    }
  }


  // Pass 1 (Resolve & Short Circuit)
  cerr << color::RED << "# PASS 1" << color::RESET << endl;
  l_pass_1: {
    auto Sum = pass_1::pass(mod, {scopeManager::mangling_abi_qw(sys)});

    if (Sum.sum()) {
      cerr << Sum;
      goto l_final;
    }
  }


  // CodeGen
  cerr << color::RED << "# PASS 3" << color::RESET << endl;
  mod->write("a.ll");
  

  
  // Final
  l_final:

  return 0;
}
