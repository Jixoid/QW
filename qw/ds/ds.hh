/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025 by Kadir Aydın.
*/

/**
 * @file ds.hh
 * @brief Data structures and dynamic typing system for QAOS.
 */

#pragma once

#include "qw/basis.hh"
#include <algorithm>
#include <cassert>
#include <cstring>
#include <iterator>
#include <ostream>
#include <span>
#include <string>
#include <string_view>
#include <stdexcept>
#include <vector>

using namespace std;



namespace ds
{
  /**
   * @brief Enumeration of available data types.
   */
  enum etype: u8 {
    etNull = 0, ///< Null type

    etStr = 1,  ///< String type
    etArr = 2,  ///< Array type
    etStc = 3,  ///< Struct type
    etTup = 4,  ///< Tuple type
    etTyp = 5,  ///< Type descriptor
    etRef = 6,  ///< Reference type
    etI64 = 7,  ///< 64-bit integer type
    etF64 = 8,  ///< 64-bit floating-point type
    etBoo = 9,  ///< Boolean type
  };


  struct payload {
    void* ptr{};
    etype type{};
    bool view{};
  };

  struct type_str  { string val{}; };
  struct type_vstr { string_view val{}; };
  
  struct type_arr  { vector<sptr<payload>> reals{}; };
  struct type_varr { span<void*> vals{}; span<etype> types{}; };
  
  struct type_stc:  type_arr  { vector<string> names{}; };
  struct type_vstc: type_varr { span<string_view> names{}; };
  
  struct type_tup:  type_arr  {};
  struct type_vtup: type_varr {};
  
  struct type_typ  { pair<string, string> val{}; };
  struct type_vtyp { string_view typ{}, val{}; };
  
  union type_val  { i64 I; f64 F; bool B; };
  union type_vval { i64* I; f64* F; bool* B; };
  
  struct type_ref  { string val{}; };
  struct type_vref { string_view val{}; };



  namespace
  {
    inline fun type_free(payload Payload) -> void {
      auto [ptr,type,span] = Payload;
      
      switch (type) {
        case etNull: break;
        
        case etStr: span ? delete (type_vstr*)ptr : delete (type_str*)ptr; break;
        case etArr: span ? delete (type_varr*)ptr : delete (type_arr*)ptr; break;
        case etStc: span ? delete (type_vstc*)ptr : delete (type_stc*)ptr; break;
        case etTup: span ? delete (type_vtup*)ptr : delete (type_tup*)ptr; break;
        case etTyp: span ? delete (type_vtyp*)ptr : delete (type_typ*)ptr; break;
        case etRef: span ? delete (type_vref*)ptr : delete (type_ref*)ptr; break;
        
        case etI64:
        case etF64:
        case etBoo: span ? delete (type_vval*)ptr : delete (type_val*)ptr; break;

        default: throw runtime_error("unknown ds type");
      }
    }
    

    inline fun create_sptr(payload Payload) -> sptr<payload> {
      return sptr<payload>(new payload(Payload), [](payload* res){ type_free(*res); delete res; });
    }
  }

  

  /**
   * @brief Represents a dynamically typed value.
   * 
   * Provides methods for creating, managing, and interacting with dynamically
   * typed data such as primitives, arrays, structs, and tuples.
   */
  struct value
  {
    public:
      value() = delete;

      inline constexpr value(sptr<payload> Payload) : m_real(std::move(Payload)) {}

      inline constexpr value(value&& it) : m_real(std::move(it.m_real)) {}

      inline constexpr fun operator=(value&& it) noexcept -> value& {
        if (this != &it)
          m_real = std::move(it.m_real);

        return *this;
      }

      value(const value&) = delete;
      fun operator=(const value&) -> value& = delete;

    public:
      /** @brief Creates a value by parsing from a standard stream. */
      [[nodiscard]] static fun makeFromStream(istream*) -> value;
      /** @brief Creates a value by parsing from a raw stream. */
      [[nodiscard]] static fun makeFromStreamRaw(istream*) -> value;
      /** @brief Creates a value by parsing from a binary stream. */
      [[nodiscard]] static fun makeFromStreamBin(istream*) -> value;

      /** @brief Saves a value to an output stream in raw format. */
      static fun saveToStreamRaw(ostream*, value&) -> void;
      /** @brief Saves a value to an output stream in binary format. */
      static fun saveToStreamBin(ostream*, value&) -> void;

    public:
      /** @brief Creates a deep copy of a value. */
      [[nodiscard]] static fun makeCopy(const value&) -> value;
      /** @brief Creates a new reference pointing to the same data. */
      [[nodiscard]] static fun makeNRef(const value&) -> value;

      /** @brief Returns a deep copy of this value. */
      [[nodiscard]] inline fun copy() -> value { return value::makeCopy(*this); }
      /** @brief Returns a new reference to this value. */
      [[nodiscard]] inline fun nref() -> value { return value::makeNRef(*this); }

      /** @brief Saves this value in raw format. */
      inline fun saveRaw(ostream*) -> void;
      /** @brief Saves this value in binary format. */
      inline fun saveBin(ostream*) -> void;

    public:
      /** @brief Creates a null value. */
      [[nodiscard]] inline constexpr static fun makeNull() -> value {
        return value(create_sptr({nullptr, etype::etNull, false}));
      }
      /** @brief Creates an undefined value. */
      [[nodiscard]] inline constexpr static fun makeUnDef() -> value {
        return value(sptr<payload>(nullptr));
      }


      /** @brief Creates a string value. */
      [[nodiscard]] inline constexpr static fun makeString(string_view val) -> value {
        return value(create_sptr({new type_str{string(val)}, etype::etStr, false}));
      }
      /** @brief Creates an empty array value. */
      [[nodiscard]] inline constexpr static fun makeArray() -> value {
        return value(create_sptr({new type_arr{}, etype::etArr, false}));
      }
      /** @brief Creates an empty struct value. */
      [[nodiscard]] inline constexpr static fun makeStruct() -> value {
        return value(create_sptr({new type_stc{}, etype::etStc, false}));
      }
      /** @brief Creates an empty tuple value. */
      [[nodiscard]] inline constexpr static fun makeTuple() -> value {
        return value(create_sptr({new type_tup{}, etype::etTup, false}));
      }
      /** @brief Creates a type descriptor value. */
      [[nodiscard]] inline constexpr static fun makeType(string_view typ, string_view val) -> value {
        return value(create_sptr({new type_typ{{string(typ),string(val)}}, etype::etTyp, false}));
      }
      /** @brief Creates a reference value. */
      [[nodiscard]] inline constexpr static fun makeRef(string_view val) -> value {
        return value(create_sptr({new type_ref{string(val)}, etype::etRef, false}));
      }
      /** @brief Creates a 64-bit integer value. */
      [[nodiscard]] inline constexpr static fun makeI64(i64 val) -> value {
        return value(create_sptr({new type_val{.I = val}, etype::etI64, false}));
      }
      /** @brief Creates a 64-bit floating-point value. */
      [[nodiscard]] inline constexpr static fun makeF64(f64 val) -> value {
        return value(create_sptr({new type_val{.F = val}, etype::etF64, false}));
      }
      /** @brief Creates a boolean value. */
      [[nodiscard]] inline constexpr static fun makeBool(bool val) -> value {
        return value(create_sptr({new type_val{.B = val}, etype::etBoo, false}));
      }
      
      /** @brief Creates a view (non-owning) string value. */
      [[nodiscard]] inline constexpr static fun makeVString(string_view val) -> value {
        return value(create_sptr({new type_vstr{val}, etype::etStr, true}));
      }
      /** @brief Creates a view (non-owning) array value. */
      [[nodiscard]] inline constexpr static fun makeVArray(span<void*> vals, span<etype> types) -> value {
        return value(create_sptr({new type_varr{vals, types}, etype::etArr, true}));
      }
      /** @brief Creates a view (non-owning) struct value. */
      [[nodiscard]] inline constexpr static fun makeVStruct(span<void*> vals, span<etype> types, span<string_view> names) -> value {
        return value(create_sptr({new type_vstc{vals, types, names}, etype::etStc, true}));
      }
      /** @brief Creates a view (non-owning) tuple value. */
      [[nodiscard]] inline constexpr static fun makeVTuple(span<void*> vals, span<etype> types) -> value {
        return value(create_sptr({new type_vtup{vals, types}, etype::etTup, true}));
      }
      /** @brief Creates a view (non-owning) type descriptor value. */
      [[nodiscard]] inline constexpr static fun makeVType(string_view typ, string_view val) -> value {
        return value(create_sptr({new type_vtyp{typ,val}, etype::etTyp, true}));
      }
      /** @brief Creates a view (non-owning) reference value. */
      [[nodiscard]] inline constexpr static fun makeVRef(string_view val) -> value {
        return value(create_sptr({new type_vref{val}, etype::etRef, true}));
      }
      /** @brief Creates a view (non-owning) 64-bit integer value. */
      [[nodiscard]] inline constexpr static fun makeVI64(i64* val) -> value {
        return value(create_sptr({new type_vval{.I = val}, etype::etI64, true}));
      }
      /** @brief Creates a view (non-owning) 64-bit floating-point value. */
      [[nodiscard]] inline constexpr static fun makeVF64(f64* val) -> value {
        return value(create_sptr({new type_vval{.F = val}, etype::etF64, true}));
      }
      /** @brief Creates a view (non-owning) boolean value. */
      [[nodiscard]] inline constexpr static fun makeVBool(bool* val) -> value {
        return value(create_sptr({new type_vval{.B = val}, etype::etBoo, true}));
      }

    private:
      sptr<payload> m_real{};

    public:
      inline fun real() { return m_real; }
    
    public:
      /** @brief Checks if the value is undefined. */
      inline fun isUnDef() const noexcept -> bool { return !m_real; }
      /** @brief Checks if the value is null. */
      inline fun isNull() const noexcept -> bool  { return m_real && (m_real->type == etype::etNull); }

      /** @brief Checks if the value is a string. */
      inline fun isString() const noexcept -> bool { return m_real && (m_real->type == etype::etStr); }
      /** @brief Checks if the value is an array. */
      inline fun isArray() const noexcept -> bool  { return m_real && (m_real->type == etype::etArr); }
      /** @brief Checks if the value is a struct. */
      inline fun isStruct() const noexcept -> bool { return m_real && (m_real->type == etype::etStc); }
      /** @brief Checks if the value is a tuple. */
      inline fun isTuple() const noexcept -> bool  { return m_real && (m_real->type == etype::etTup); }
      /** @brief Checks if the value is a type descriptor. */
      inline fun isType() const noexcept -> bool   { return m_real && (m_real->type == etype::etTyp); }
      /** @brief Checks if the value is a reference. */
      inline fun isRef() const noexcept -> bool    { return m_real && (m_real->type == etype::etRef); }
      /** @brief Checks if the value is an integer. */
      inline fun isInt() const noexcept -> bool    { return m_real && (m_real->type == etype::etI64); }
      /** @brief Checks if the value is a floating-point number. */
      inline fun isFloat() const noexcept -> bool  { return m_real && (m_real->type == etype::etF64); }
      /** @brief Checks if the value is a boolean. */
      inline fun isBool() const noexcept -> bool   { return m_real && (m_real->type == etype::etBoo); }

      /** @brief Checks if the value is a view (non-owning reference to external data). */
      inline fun isView() const noexcept -> bool { assert(m_real); return m_real->view; }

    public:
      /** @brief Gets the underlying string value. */
      inline fun w_string() const noexcept -> string { assert(isString()); return isView() ? string(((type_vstr*)m_real->ptr)->val) : ((type_str*)m_real->ptr)->val; }
      /** @brief Gets the underlying integer value. */
      inline fun w_int() const noexcept -> i64       { assert(isInt());    return isView() ? *((type_vval*)m_real->ptr)->I : ((type_val*)m_real->ptr)->I; }
      /** @brief Gets the underlying floating-point value. */
      inline fun w_float() const noexcept -> f64     { assert(isFloat());  return isView() ? *((type_vval*)m_real->ptr)->F : ((type_val*)m_real->ptr)->F; }
      /** @brief Gets the underlying boolean value. */
      inline fun w_bool() const noexcept -> bool     { assert(isBool());   return isView() ? *((type_vval*)m_real->ptr)->B : ((type_val*)m_real->ptr)->B; }
      
      /** @brief Gets the underlying reference value. */
      inline fun w_ref() const noexcept -> string    { assert(isRef());    return isView() ? string(((type_vref*)m_real->ptr)->val) : ((type_ref*)m_real->ptr)->val; }

      /** @brief Gets the underlying type descriptor value. */
      inline fun w_type() const noexcept -> pair<string,string>
      {
        assert(isType());
        
        return isView()
          ? pair{string(((type_vtyp*)m_real->ptr)->typ), string(((type_vtyp*)m_real->ptr)->val)}
          : ((type_typ*)m_real->ptr)->val;
      }

    public:
      /** @brief Gets the size of an array or struct. */
      inline fun size() const noexcept -> u32 { assert(isArray() || isStruct()); return isView() ? ((type_varr*)m_real->ptr)->vals.size() : ((type_arr*)m_real->ptr)->reals.size(); }

      /** @brief Checks if a tuple contains a specific type. */
      inline fun contains(etype val) const noexcept -> bool
      {
        assert(isTuple());
        
        if (isView()) {
          for (const auto X: ((type_vtup*)m_real->ptr)->types)
            if (val == X)
              return true;

          return false;
        }
        else {
          for (const auto X: ((type_tup*)m_real->ptr)->reals)
            if (val == X->type)
              return true;

          return false;
        }
      }

    public:
      /** @brief Appends a value to an array or tuple. */
      inline fun push_back(value&& val) -> void
      {
        assert((isArray() || isTuple()) && !isView());

        auto real = ((type_arr*)m_real->ptr);

        real->reals.push_back(std::move(val.m_real));
      }

      /** @brief Appends a value to a struct with a given key. */
      inline fun push_back(value&& val, string_view key) -> void
      {
        assert(isStruct() && !isView());

        auto real = ((type_stc*)m_real->ptr);

        real->reals.push_back(std::move(val.m_real));
        real->names.push_back(string(key));
      }

      /** @brief Appends a value to a tuple associated with a specific type. */
      inline fun push_back(value&& val, etype key) -> void
      {
        assert(isTuple() && !isView());

        auto real = ((type_tup*)m_real->ptr);

        real->reals.push_back(std::move(val.m_real));
      }

      /** @brief Gets a struct member by index, returning its key-value pair. */
      inline fun get_stc(u32 index) const -> pair<string, value>
      {
        assert(isStruct());

        if (index >= size()) return {"", value::makeUnDef()};
        
        auto real = isView()
          ? create_sptr({
              ((type_vstc*)m_real->ptr)->vals[index],
              ((type_vstc*)m_real->ptr)->types[index],
              true
            })
          : ((type_stc*)m_real->ptr)->reals[index];

        auto key = isView()
          ? string(((type_vstc*)m_real->ptr)->names[index])
          : ((type_stc*)m_real->ptr)->names[index];

        return {key, value(real)};
      }


    public:
      [[nodiscard]] inline fun operator[](string_view key) const -> value
      {
        assert(isStruct());

        auto idx = isView()
          ? [key](span<string_view> names) -> i64 {
              auto it = std::find(names.begin(), names.end(), key);

              return (it != names.end()) ? std::distance(names.begin(), it) : -1;
            }(((type_vstc*)m_real->ptr)->names)
          : [key](const vector<string>& names) -> i64 {
              auto it = std::find(names.begin(), names.end(), key);

              return (it != names.end()) ? std::distance(names.begin(), it) : -1;
            }(((type_stc*)m_real->ptr)->names);


        if (idx < 0)
          return value::makeUnDef();

        return (*this)[idx];
      }

      value operator[](const char* key) const { return (*this)[string_view(key, strlen(key))]; }

      inline fun operator[](u32 index) const -> value
      {
        assert(isArray() || isStruct());

        if (index >= size()) return value::makeUnDef();
        
        auto real = isView()
          ? sptr<payload>(create_sptr({
              ((type_varr*)m_real->ptr)->vals[index],
              ((type_varr*)m_real->ptr)->types[index],
              false,
            }))
          : ((type_arr*)m_real->ptr)->reals[index];

        return value(real);
      }

      inline fun operator[](etype val) const -> value
      {
        assert(isTuple());

        if (!contains(val)) return value::makeUnDef();

        u0 idx = isView()
          ? [this, val](){
              for (u0 i{}; i < ((type_vtup*)m_real->ptr)->types.size(); i++)
                if (val == ((type_vtup*)m_real->ptr)->types[i])
                  return i;

              throw runtime_error("internal error");
            }()
          : [this, val](){
              for (u0 i{}; i < ((type_tup*)m_real->ptr)->reals.size(); i++)
                if (val == ((type_tup*)m_real->ptr)->reals[i]->type)
                  return i;

              throw runtime_error("internal error");
            }();
        
        auto real = isView()
          ? sptr<payload>(create_sptr({
              ((type_vtup*)m_real->ptr)->vals[idx],
              ((type_vtup*)m_real->ptr)->types[idx],
              true,
            }))
          : ((type_tup*)m_real->ptr)->reals[idx];

        return value(real);
      }

  };

}
