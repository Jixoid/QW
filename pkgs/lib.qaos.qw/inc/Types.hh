/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#pragma once

#include "Basis.hh"
#include <string_view>
#include <utility>

using namespace std;



namespace qw
{
  enum abi: u8 { abiQW, abiItanium };

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
      inline fun mod() { return m_mod; }
      inline fun off() { return m_off; }
      inline fun size() { return m_size; }

      fun file() -> string_view;
      fun fpath() -> string_view;
      
      fun view() -> string_view;
      [[nodiscard]] fun str() -> string;

      [[nodiscard]] fun interval() -> pair<humanPos,humanPos>;


    public:
      inline operator bool() { return (bool)m_mod; }
  };



  struct qIdentifier
  {
    public:
      enum qEnum: u16 {
        qie_Unkown,
        
        qie_Decl,
        qie_Type,
        qie_Stmt,
        qie_Expr,
      };
    

    protected:
      inline qIdentifier(qEnum type, qIdentifier *parent, word pos)
        : m_parent(parent)
        , m_type(type)
        , m_pos(pos)
      {}
      

    private:
      qIdentifier *m_parent{};
      word m_pos{};
      qEnum m_type{qie_Unkown};

    public:
      inline fun pos() { return m_pos; }
      inline fun& parent() { return m_parent; }
      inline fun type() const { return m_type; }
  };

}



namespace qw::color
{
  // Format Sıfırlama
  inline constinit const char *RESET = "\033[0m";

  // Metin Stilleri
  inline constinit const char *UNDERLINE = "\033[4m";

  // Metin Renkleri
  inline constinit const char *GRAY   = "\033[1;30m";
  inline constinit const char *RED     = "\033[1;31m";
  inline constinit const char *GREEN   = "\033[1;32m";
  inline constinit const char *YELLOW  = "\033[1;33m";
  inline constinit const char *BLUE    = "\033[1;34m";
  inline constinit const char *MAGENTA = "\033[1;35m";
  inline constinit const char *CYAN    = "\033[1;36m";
  inline constinit const char *WHITE   = "\033[1;37m";
}



namespace qw::decls
{
  struct qDecl;

  struct qNameSpaceDecl;
  struct qModuleDecl;
  struct qVarDecl;
  struct qTypeDecl;
  struct qFuncDecl;
  struct qAliasDecl;


  #pragma region SystemType
  struct qIntU8;
  struct qIntU16;
  struct qIntU32;
  struct qIntU64;
  struct qIntU128;

  struct qIntS8;
  struct qIntS16;
  struct qIntS32;
  struct qIntS64;
  struct qIntS128;

  struct qFlo16;
  struct qFlo32;
  struct qFlo64;
  struct qFlo128;
  
  struct qChar;

  struct qBool;

  struct qVoid;
  #pragma endregion
}


namespace qw::types
{
  struct qType;

  struct qSubType;
  struct qNickType;
  struct qFuncType;
  struct qRecordType;

  struct qMemberFieldType;
}


namespace qw::exprs
{
  struct qExpr;

  struct qLiteral;
  struct qIntegerLiteral;
  struct qFloatingLiteral;
  struct qCharLiteral;
  struct qBoolLiteral;
  struct qPtrLiteral;

  struct qValExpr;

  struct qNickExpr;

  struct qPrimaryOp;
  struct qBinaryOp;
}


namespace qw::stmts
{
  struct qStmt;

  struct qCodeBlock;
  struct qCodeVar;
  
  struct qReturn;
  
  struct qExprStmt;
}
