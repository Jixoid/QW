/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#pragma once

#include <iostream>
#include <llvm/IR/Module.h>    

#include "qw/basis.hh"
#include "qw/pretype.hh"
#include "qw/vfs/vfs.hh"
#include <fcntl.h>
#include <llvm/IR/LLVMContext.h>
#include <llvm/IR/Verifier.h>
#include <optional>
#include <ostream>
#include <string>
#include <string_view>
#include <unordered_map>
#include <vector>

#include <sys/stat.h>
#include <sys/mman.h>

#define ef else if



namespace qw
{
  fun operator <<(std::ostream &os, identy *ident) -> std::ostream&;

  

  struct gst
  {
    fun add_ident(std::string name, identy *ident) {
      assert(ident && "object to be added is null");

      auto it = m_idents.find(name);
      
      //cerr << color::B_YELLOW << "reg " << color::GRAY << "{"
      //  << color::RESET << "symbol" << color::GRAY << ": " << color::B_BLUE << "\"" << name << "\"" << color::GRAY << ", "
      //  << color::RESET << "kind" << color::GRAY <<  ": " << color::RESET << ident << color::GRAY << "}" << color::RESET
      //  << endl;

      if (it == m_idents.end())
        m_idents[name] = ident;
      else
        assert(false && "DIAGNOSTIC");
    }


    fun find(std::string name) -> std::optional<identy*> {
      auto it = m_idents.find(name);

      if (it != m_idents.end())
        return it->second;
      else
        return std::nullopt;
    }


    private:
      std::unordered_map<std::string, identy*> m_idents;

    public:
      inline fun idents() { return m_idents; }
  
  };


  enum struct ProgBits: u8 { Bit16 = 2, Bit32 = 4, Bit64 = 8 };



  class module;

  class context
  {
    #pragma region New
    protected:
      context(ProgBits pb);
    
    public:
      ~context();


    public:
      static inline fun make(ProgBits pb) {
        return make_uptr(new context(pb));
      }

      fun make_module(std::string name, std::string fpath) -> module*;
    #pragma endregion
    


    protected:
      std::vector<module*> m_modules{};
      std::vector<types::Type*> m_types{};
      std::vector<decls::Decl*> m_decls{};
      std::vector<exprs::Expr*> m_exprs{};
      std::vector<stmts::Stmt*> m_stmts{};
    
    
    protected:
      sptr<llvm::LLVMContext> m_llvm{};
      gst m_gst;
      ProgBits m_progBits;

    public:
      inline fun llvm() { return m_llvm.get(); }
      inline fun& gst() { return m_gst; }
      inline fun progBits() { return m_progBits; }


    
    #pragma region SystemTypes
    private:
      types::Type *mf_intU0{};
      types::Type *mf_intU8{};
      types::Type *mf_intU16{};
      types::Type *mf_intU32{};
      types::Type *mf_intU64{};
      types::Type *mf_intU128{};

      types::Type *mf_intS0{};
      types::Type *mf_intS8{};
      types::Type *mf_intS16{};
      types::Type *mf_intS32{};
      types::Type *mf_intS64{};
      types::Type *mf_intS128{};

      types::Type *mf_flo0{};
      types::Type *mf_flo16{};
      types::Type *mf_flo32{};
      types::Type *mf_flo64{};
      types::Type *mf_flo128{};
      
      types::Type *mf_char{};
      
      types::Type *mf_bool{};
      
      types::Type *mf_void{};

      types::Type *mf_ptr{};

    public:
      inline fun intU0_t() { return mf_intU0; };
      inline fun intU8_t() { return mf_intU8; };
      inline fun intU16_t() { return mf_intU16; };
      inline fun intU32_t() { return mf_intU32; };
      inline fun intU64_t() { return mf_intU64; };
      inline fun intU128_t() { return mf_intU128; };

      inline fun intS0_t() { return mf_intS0; };
      inline fun intS8_t() { return mf_intS8; };
      inline fun intS16_t() { return mf_intS16; };
      inline fun intS32_t() { return mf_intS32; };
      inline fun intS64_t() { return mf_intS64; };
      inline fun intS128_t() { return mf_intS128; };

      inline fun flo0_t() { return mf_flo0; };
      inline fun flo16_t() { return mf_flo16; };
      inline fun flo32_t() { return mf_flo32; };
      inline fun flo64_t() { return mf_flo64; };
      inline fun flo128_t() { return mf_flo128; };
      
      inline fun char_t() { return mf_char; };
      
      inline fun bool_t() { return mf_bool; };
      
      inline fun void_t() { return mf_void; };

      inline fun ptr_t() { return mf_ptr; };
    #pragma endregion


    public:
      fun push(identy *obj) { switch (obj->type()) {
        case IdentyEnum::Type: m_types.push_back((types::Type*)obj); break;
        case IdentyEnum::Decl: m_decls.push_back((decls::Decl*)obj); break;
        case IdentyEnum::Expr: m_exprs.push_back((exprs::Expr*)obj); break;
        case IdentyEnum::Stmt: m_stmts.push_back((stmts::Stmt*)obj); break;
      }}
  };


  class module
  {
    friend struct context;

    protected:
      module(context *ctx, std::string name, std::string fpath);

      
    public:
      inline fun write(std::string_view target) {
        std::error_code ErrorInfo;
        llvm::raw_fd_ostream dest(target, ErrorInfo);

        if (ErrorInfo) {
          llvm::errs() << "Dosya acilamadi: " << ErrorInfo.message() << "\n";
          return false;
        }

        //if (llvm::verifyModule(*m_llvm, &llvm::errs())) {
        //  llvm::errs() << "--- ERROR: The module failed the validation process! ---\n";
        //  m_llvm->print(llvm::errs(), nullptr); 
        //  return false;
        //}
        
        m_llvm->print(dest, nil);
        return true;
      }


    private:
      std::string m_fpath;
      sptr<vfs::mapped> m_mmap;
      qw::context *m_ctx{};
      uptr<llvm::Module> m_llvm;
      decls::NameSpace *m_ns{};

    public:
      inline fun fpath() { return (std::string_view)m_fpath; }
      inline fun& mmap() { return m_mmap; }
      inline fun nameSpace() { return m_ns; }
      inline fun llvm() { return m_llvm.get(); }
      inline fun ctx() { return m_ctx; }
      inline fun layouter() { return m_llvm->getDataLayout(); }
  };

}
