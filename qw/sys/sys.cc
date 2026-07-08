/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#include "qw/sys/sys.hh"
#include "qw/control/scopemng.hh"



namespace qw::sys
{

  fun build_sys_module(qw::context *ctx) -> decls::Decl*
  {
    /// sys NS
    auto sys = ctx->sys_api.sys_ns = decls::Decl::make_NameSpace(ctx, nullptr, "sys", word{});



    #pragma region sys::* types

    #define NewSys(Name, Type) { \
      auto tdecl = decls::Decl::make_Type(ctx, ctx->sys_api.sys_ns, #Name, word{}, Type); \
      ctx->gst().add_ident(scopemng::mangling_abi_qw(tdecl), tdecl); \
    }

    NewSys(u8, ctx->intU8_t())
    NewSys(u16, ctx->intU16_t())
    NewSys(u32, ctx->intU32_t())
    NewSys(u64, ctx->intU64_t())
    NewSys(u128, ctx->intU128_t())
    NewSys(usize, ctx->intU0_t())

    NewSys(i8, ctx->intS8_t())
    NewSys(i16, ctx->intS16_t())
    NewSys(i32, ctx->intS32_t())
    NewSys(i64, ctx->intS64_t())
    NewSys(i128, ctx->intS128_t())
    NewSys(isize, ctx->intS0_t())

    NewSys(f16, ctx->flo16_t())
    NewSys(f32, ctx->flo32_t())
    NewSys(f64, ctx->flo64_t())
    NewSys(f128, ctx->flo128_t())
    
    NewSys(char, ctx->char_t())
    NewSys(bool, ctx->bool_t())
    NewSys(void, ctx->void_t())
    NewSys(ptr, ctx->ptr_t())

    #undef NewSys

    #pragma endregion
    
    

    // sys::syscall overloads for 1 to 7 parameters
    for (int i = 1; i <= 7; ++i) {
      std::vector<types::FieldType> args;
      args.push_back({"no", ctx->intU64_t(), Visibility::Public});
      for (int a = 1; a < i; ++a) {
        args.push_back({"a" + std::to_string(a), ctx->intU64_t(), Visibility::Public});
      }
      auto syscall_func = decls::Decl::make_Func(ctx, sys, "syscall", word{}, types::Type::make_Func(ctx, args, ctx->intU64_t()), Visibility::Public);
      ctx->gst().add_ident("qwrtl_syscall" + std::to_string(i), syscall_func);
    }

    #pragma region sys::heap

    ctx->sys_api.heap_ns = decls::Decl::make_NameSpace(ctx, sys, "heap", word{});

    // sys::heap::alloc(align, size: usize) -> ptr
    ctx->sys_api.heap_alloc = decls::Decl::make_Func(ctx, ctx->sys_api.heap_ns, "alloc", word{}, types::Type::make_Func(ctx,
      {
        {"align", ctx->intU0_t()},
        {"size", ctx->intU0_t()},
      },
      ctx->ptr_t()), Visibility::Public
    );
    ctx->gst().add_ident("qwrtl_heap_alloc", ctx->sys_api.heap_alloc);

    // sys::heap::dispose(p: ptr, align, size: usize) -> void
    ctx->sys_api.heap_dispose = decls::Decl::make_Func(ctx, ctx->sys_api.heap_ns, "dispose", word{}, types::Type::make_Func(ctx,
      {
        {"p", ctx->ptr_t()},
        {"align", ctx->intU0_t()},
        {"size", ctx->intU0_t()},
      },
      ctx->void_t()), Visibility::Public
    );
    ctx->gst().add_ident("qwrtl_heap_dispose", ctx->sys_api.heap_dispose);

    // sys::heap::realloc(p: ptr, align, old_size, new_size: usize) -> ptr
    ctx->sys_api.heap_realloc = decls::Decl::make_Func(ctx, ctx->sys_api.heap_ns, "realloc", word{}, types::Type::make_Func(ctx,
      {
        {"p", ctx->ptr_t()},
        {"align", ctx->intU0_t()},
        {"old_size", ctx->intU0_t()},
        {"new_size", ctx->intU0_t()},
      },
      ctx->ptr_t()), Visibility::Public
    );
    ctx->gst().add_ident("qwrtl_heap_realloc", ctx->sys_api.heap_realloc);

    #pragma endregion


    return sys;
  }
}
