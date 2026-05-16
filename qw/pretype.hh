/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#pragma once

#include "qw/basis.hh"
#include <string_view>
#include <utility>



namespace qw
{
  enum struct abi: u8 { QW, Itanium };

  class context;
  class module;


  struct word
  {
    public:
      struct humanPos { u0 y,x; };

    public:
      inline word() {}
      inline word(qw::module *mod, u0 off, u0 size): m_mod(mod), m_off(off), m_size(size) {}


    private:
      qw::module *m_mod{};
      u0 m_off{}, m_size{};
    
    public:
      inline fun mod() const { return m_mod; }
      inline fun off() const { return m_off; }
      inline fun size() const { return m_size; }


    public:
      fun file() -> std::string_view;
      fun fpath() -> std::string_view;
      
      fun view() -> std::string_view;
      [[nodiscard]] fun str() -> std::string;

      [[nodiscard]] fun interval() -> std::pair<humanPos,humanPos>;


    public:
      inline operator bool() { return (bool)m_mod; }
  };



  enum struct IdentyEnum: u8 { Decl, Type, Stmt, Expr };

  struct identy
  {
    protected:
      explicit inline identy(IdentyEnum type, identy *parent, word pos)
        : m_parent(parent)
        , m_type(type)
        , m_pos(pos)
      {}
      

    private:
      identy *m_parent{};
      word m_pos{};
      IdentyEnum m_type;

    public:
      inline fun  pos() { return m_pos; }
      inline fun& parent() { return m_parent; }
      inline fun  type() const { return m_type; }
  };

}



namespace qw::color
{
  inline constinit const char *RESET = "\033[0m";

  inline constinit const char *UNDERLINE = "\033[4m";

  inline constinit const char *GRAY    = "\033[1;30m";
  inline constinit const char *RED     = "\033[1;31m";
  inline constinit const char *GREEN   = "\033[1;32m";
  inline constinit const char *YELLOW  = "\033[1;33m";
  inline constinit const char *BLUE    = "\033[1;34m";
  inline constinit const char *MAGENTA = "\033[1;35m";
  inline constinit const char *CYAN    = "\033[1;36m";
  inline constinit const char *WHITE   = "\033[1;37m";
}



namespace qw::types
{
  struct Type;

  struct Sub;
  struct Nick;
  struct Func;
  struct Record;

  struct MemberField;
}

namespace qw::stmts
{
  struct Stmt;

  struct CodeBlock;
  struct CodeVar;
  
  struct Return;
  
  struct ExprStmt;
}

namespace qw::exprs
{
  struct Expr;

  struct Literal;
  struct IntegerLiteral;
  struct FloatingLiteral;
  struct CharLiteral;
  struct BoolLiteral;
  struct PtrLiteral;

  struct ValExpr;

  struct NickExpr;

  struct PrimaryOp;
  struct BinaryOp;
}

namespace qw::decls
{
  struct Decl;

  struct NameSpace;
  struct Module;
  struct Var;
  struct Type;
  struct Func;
  struct Alias;


  #pragma region SystemType
  struct IntU8;
  struct IntU16;
  struct IntU32;
  struct IntU64;
  struct IntU128;

  struct IntS8;
  struct IntS16;
  struct IntS32;
  struct IntS64;
  struct IntS128;

  struct Float16;
  struct Float32;
  struct Float64;
  struct Float128;
  
  struct Char;

  struct Bool;

  struct Void;
  #pragma endregion
}
