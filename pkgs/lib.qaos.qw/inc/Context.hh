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

#include "Basis.hh"
#include "Exprs.hh"
#include "Types.hh"
#include <fcntl.h>
#include <llvm/IR/LLVMContext.h>
#include <llvm/IR/Verifier.h>
#include <optional>
#include <ostream>
#include <string_view>
#include <unordered_map>
#include <vector>

#include <sys/stat.h>
#include <sys/mman.h>

#define ef else if

using namespace std;



namespace qw
{
  fun operator <<(ostream &os, qIdentifier *ident) -> ostream&;

  

  struct gst
  {
    fun add_ident(string name, qIdentifier *ident) {
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


    fun find(string name) -> optional<qIdentifier*> {
      auto it = m_idents.find(name);

      if (it != m_idents.end())
        return it->second;
      else
        return nullopt;
    }


    private:
      unordered_map<string, qIdentifier*> m_idents;

    public:
      inline fun idents() { return m_idents; }
  
  };


  struct mappedFile
  {
    public:
      explicit inline mappedFile(string fpath) {
        fd = open(fpath.c_str(), O_RDONLY);
        if (fd == -1)
          throw std::system_error(errno, std::generic_category());

        struct stat st;
        fstat(fd, &st);
        size = st.st_size;

        data = mmap(nullptr, size, PROT_READ, MAP_PRIVATE, fd, 0);
        if (data == MAP_FAILED)
          throw std::system_error(errno, std::generic_category());
      }

      inline ~mappedFile() {
        if (data != MAP_FAILED) munmap(data, size);
        if (fd != -1) close(fd);
      }

    public:
      inline fun view() const -> const string_view {
        return {static_cast<const char*>(data), size};
      }

    private:
      int fd{-1};
      void *data{MAP_FAILED};
      u0 size{};
  };
  


  enum qProgBits: u8 { pb16 = 2, pb32 = 4, pb64 = 8 };



  class module;

  class context
  {
    #pragma region New
    protected:
      context(qProgBits pb);
    
    public:
      ~context();


    public:
      static inline fun make(qProgBits pb) {
        return make_uptr(new context(pb));
      }

      fun make_module(string name, string fpath) -> module*;
    #pragma endregion
    


    protected:
      vector<module*> m_modules{};
      vector<types::qType*> m_types{};
      vector<decls::qDecl*> m_decls{};
      vector<exprs::qExpr*> m_exprs{};
      vector<stmts::qStmt*> m_stmts{};
    
    
    protected:
      sptr<llvm::LLVMContext> m_llvm{};
      gst m_gst;
      qProgBits m_progBits;

    public:
      inline fun llvm() { return m_llvm.get(); }
      inline fun& gst() { return m_gst; }
      inline fun progBits() { return m_progBits; }


    
    #pragma region SystemTypes
    private:
      types::qType *mf_intU0{};
      types::qType *mf_intU8{};
      types::qType *mf_intU16{};
      types::qType *mf_intU32{};
      types::qType *mf_intU64{};
      types::qType *mf_intU128{};

      types::qType *mf_intS0{};
      types::qType *mf_intS8{};
      types::qType *mf_intS16{};
      types::qType *mf_intS32{};
      types::qType *mf_intS64{};
      types::qType *mf_intS128{};

      types::qType *mf_flo0{};
      types::qType *mf_flo16{};
      types::qType *mf_flo32{};
      types::qType *mf_flo64{};
      types::qType *mf_flo128{};
      
      types::qType *mf_char{};
      
      types::qType *mf_bool{};
      
      types::qType *mf_void{};

      types::qType *mf_ptr{};

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
      fun push(qIdentifier *obj) { switch (obj->type()) {
        case qIdentifier::qie_Unkown: assert(false && "Unknown Type");

        case qIdentifier::qie_Type: m_types.push_back((types::qType*)obj); break;
        case qIdentifier::qie_Decl: m_decls.push_back((decls::qDecl*)obj); break;
        case qIdentifier::qie_Expr: m_exprs.push_back((exprs::qExpr*)obj); break;
        case qIdentifier::qie_Stmt: m_stmts.push_back((stmts::qStmt*)obj); break;
      }}
  };


  class module
  {
    friend class context;

    protected:
      module(context *ctx, string name, string fpath);

      
    public:
      inline fun write(string_view target) {
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
        
        m_llvm->print(dest, Nil);
        return true;
      }


    private:
      string m_fpath;
      mappedFile m_mmap;
      qw::context *m_ctx{};
      uptr<llvm::Module> m_llvm;
      decls::qNameSpaceDecl *m_ns{};

    public:
      inline fun fpath() { return string_view(m_fpath); }
      inline fun& mmap() { return m_mmap; }
      inline fun nameSpace() { return m_ns; }
      inline fun llvm() { return m_llvm.get(); }
      inline fun ctx() { return m_ctx; }
      inline fun layouter() { return m_llvm->getDataLayout(); }
  };

}
