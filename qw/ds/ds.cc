/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025 by Kadir Aydın.
*/


#include "qw/ds/ds.hh"
#include "qw/basis.hh"
#include <cmath>
#include <istream>
#include <stdexcept>
#include <string>
#include <string_view>
#include <iostream>
#include <vector>
#include <utility>
#include <charconv>

#define el  else
#define ef  else if

using namespace std;
using namespace ds;



namespace // others
{
  // Header
  struct alignas(8) magic
  {
    char Magic[8]{};
    char SubMagic[8]{};
  };

  magic ExpectedMagic = {
    .Magic =    {'\e','^','Q','A','O','S','!','\a'},
    .SubMagic = {'C', 'o','n','f','L','a','n','g'},
  };


  struct header
  {
    magic Magic{};
    u32 VerMaj{}, VerMin{};
  };

  constinit u32 VerMaj{1};
  constinit u32 VerMin{0};
}


namespace // read bin
{
  fun parse_bin_stc(istream*) -> value;
  fun parse_bin_arr(istream*) -> value;
  fun parse_bin_tup(istream*) -> value;


  fun parse_bin_null(istream* stream) {
    return value::makeNull();
  }

  fun parse_bin_str(istream* stream) {
    u32 Size{};
    stream->read((char*)&Size, sizeof(Size));
    
    vector<char> CStr(Size);
    stream->read(CStr.data(), Size);

    auto Pad = (8 -((sizeof(Size)+Size) & 7)) & 7;
    stream->seekg(Pad, ios::cur);

    
    return value::makeString(string(CStr.data(), Size));
  }

  fun parse_bin_typ(istream* stream) {
    u32 Size1{};
    stream->read((char*)&Size1, sizeof(Size1));

    u32 Size2{};
    stream->read((char*)&Size2, sizeof(Size2));
    

    vector<char> CStr1(Size1);
    stream->read(CStr1.data(), Size1);

    vector<char> CStr2(Size2);
    stream->read(CStr2.data(), Size2);

    auto Pad = (8 -((Size1+Size2) & 7)) & 7;
    stream->seekg(Pad, ios::cur);

    
    return value::makeType(string(CStr1.data(), Size1), string(CStr2.data(), Size2));
  }

  fun parse_bin_i64(istream* stream) {
    i64 Buf{};
    stream->read((char*)&Buf, sizeof(Buf));

    return value::makeI64(Buf);
  }

  fun parse_bin_f64(istream* stream) {
    f64 Buf{};
    stream->read((char*)&Buf, sizeof(Buf));
    
    return value::makeF64(Buf);
  }

  fun parse_bin_bool(istream* stream) {
    u64 Buf{};
    stream->read((char*)&Buf, sizeof(Buf));

    return value::makeBool((bool)Buf);
  }


  inline fun parse_bin_route(istream* stream, etype type) { switch (type) {
    case etype::etNull: return parse_bin_null(stream); break;
    case etype::etStr:  return parse_bin_str(stream); break;
    case etype::etTyp:  return parse_bin_typ(stream); break;
    case etype::etI64:  return parse_bin_i64(stream); break;
    case etype::etF64:  return parse_bin_f64(stream); break;
    case etype::etBoo:  return parse_bin_bool(stream); break;
    case etype::etArr:  return parse_bin_arr(stream); break;
    case etype::etStc:  return parse_bin_stc(stream); break;
    case etype::etTup:  return parse_bin_tup(stream); break;

    default: throw runtime_error("internal error");
  }}


  fun parse_bin_arr(istream* stream) -> value {
    // Size
    u64 Size{};
    stream->read((char*)&Size, sizeof(Size));

    
    vector<etype> Types(Size);
    stream->read((char*)Types.data(), Size*sizeof(etype));

    
    auto Self = value::makeArray();
    ((type_arr*)Self.real()->ptr)->reals.reserve(Size);


    // Read Sub
    for (u0 i = 0; i < Size; i++)
      Self.push_back(parse_bin_route(stream, Types[i]));

    // Ret
    return std::move(Self);
  }

  fun parse_bin_stc(istream* stream) -> value {
    // Size
    u64 Size{};
    stream->read((char*)&Size, sizeof(Size));

    
    vector<etype> Types(Size);
    stream->read((char*)Types.data(), Size*sizeof(etype));

    
    auto Self = value::makeStruct();
    ((type_stc*)Self.real()->ptr)->reals.reserve(Size);
    ((type_stc*)Self.real()->ptr)->names.reserve(Size);


    vector<string> Names;
    Names.reserve(Size);
    { // Read names
      u0 TSize{};

      for (u0 i = 0; i < Size; i++) {
        u32 S{};
        stream->read((char*)&S, sizeof(S));

        vector<char> CStr(Size);
        stream->read(CStr.data(), Size);
        
        Names.push_back(string(CStr.data(), Size));

        TSize += sizeof(S)+S;
      }

      auto Pad = (8 -(TSize & 7)) & 7;
      stream->seekg(Pad, ios::cur);
    }


    // Read Sub
    for (u0 i = 0; i < Size; i++)
      Self.push_back(parse_bin_route(stream, Types[i]), Names[i]);
  

    // Ret
    return std::move(Self);
  }

  fun parse_bin_tup(istream* stream) -> value {
    // Size
    u64 Size{};
    stream->read((char*)&Size, sizeof(Size));

    
    vector<etype> Types(Size);
    stream->read((char*)Types.data(), Size*sizeof(etype));

    
    auto Self = value::makeTuple();
    ((type_tup*)Self.real()->ptr)->reals.reserve(Size);


    // Read Sub
    for (u0 i = 0; i < Size; i++)
      Self.push_back(parse_bin_route(stream, Types[i]));

    // Ret
    return std::move(Self);
  }
}

namespace // write bin
{
  fun write_bin_stc(ostream*, value) -> void;
  fun write_bin_arr(ostream*, value) -> void;
  fun write_bin_tup(ostream*, value) -> void;

  fun write_bin_null(ostream* stream, value val) {}

  fun write_bin_str(ostream* stream, value val) {
    auto Str = val.w_string();

    u32 Size{(u32)Str.size()};
    stream->write((char*)&Size, sizeof(Size));
    
    stream->write(Str.data(), Size);

    auto Pad = (8 -((sizeof(Size)+Size) & 7)) & 7;
    stream->seekp(Pad, ios::cur);
  }
  
  fun write_bin_typ(ostream* stream, value val) {
    auto Str = val.w_type();

    u32 Size1{(u32)Str.first.size()};
    stream->write((char*)&Size1, sizeof(Size1));
    
    u32 Size2{(u32)Str.second.size()};
    stream->write((char*)&Size2, sizeof(Size2));
    
    stream->write(Str.first.data(), Size1);
    stream->write(Str.second.data(), Size2);

    auto Pad = (8 -((Size1+Size2) & 7)) & 7;
    stream->seekp(Pad, ios::cur);
  }

  fun write_bin_i64(ostream* stream, value val) {
    i64 Buf{val.w_int()};
    stream->write((char*)&Buf, sizeof(Buf));
  }

  fun write_bin_f64(ostream* stream, value val) {
    f64 Buf{val.w_float()};
    stream->write((char*)&Buf, sizeof(Buf));
  }

  fun write_bin_bool(ostream* stream, value val) {
    u64 Buf{val.w_bool()};
    stream->write((char*)&Buf, sizeof(Buf));
  }


  inline fun write_bin_route(ostream* stream, etype type, value val) { switch (type) {
    case etype::etNull: write_bin_null(stream, std::move(val)); break;
    case etype::etStr:  write_bin_str(stream, std::move(val)); break;
    case etype::etTyp:  write_bin_typ(stream, std::move(val)); break;
    case etype::etI64:  write_bin_i64(stream, std::move(val)); break;
    case etype::etF64:  write_bin_f64(stream, std::move(val)); break;
    case etype::etBoo:  write_bin_bool(stream, std::move(val)); break;
    case etype::etArr:  write_bin_arr(stream, std::move(val)); break;
    case etype::etStc:  write_bin_stc(stream, std::move(val)); break;
    case etype::etTup:  write_bin_tup(stream, std::move(val)); break;

    default: throw runtime_error("internal error");
  }}


  fun write_bin_arr(ostream* stream, value val) -> void {
    // Size
    u64 Size{val.size()};
    stream->write((char*)&Size, sizeof(Size));


    for (auto &X: ((type_arr*)val.real()->ptr)->reals) {
      const auto Cac{X->type};

      stream->write((char*)Cac, sizeof(Cac));
    }


    // Write Sub
    for (auto &X: ((type_arr*)val.real()->ptr)->reals)
      write_bin_route(stream, X->type, value(X));
  }

  fun write_bin_stc(ostream* stream, value val) -> void {
    // Size
    u64 Size{val.size()};
    stream->write((char*)&Size, sizeof(Size));


    for (auto &X: ((type_stc*)val.real()->ptr)->reals) {
      const auto Cac{X->type};

      stream->write((char*)Cac, sizeof(Cac));
    }

    
    _l_Names: {
      u0 TSize{};

      for (auto &X: ((type_stc*)val.real()->ptr)->names) {
        u32 S{(u32)X.size()};
        stream->write((char*)&S, sizeof(S));

        stream->write(X.c_str(), X.size());
        
        TSize += sizeof(S)+S;
      }

      auto Pad = (8 -(TSize & 7)) & 7;
      stream->seekp(Pad, ios::cur);
    }


    // Write Sub
    for (auto &X: ((type_stc*)val.real()->ptr)->reals)
      write_bin_route(stream, X->type, value(X));
  }

  fun write_bin_tup(ostream* stream, value val) -> void {
    // Size
    u64 Size{val.size()};
    stream->write((char*)&Size, sizeof(Size));


    for (auto &X: ((type_tup*)val.real()->ptr)->reals) {
      const auto Cac{X->type};

      stream->write((char*)Cac, sizeof(Cac));
    }


    // Write Sub
    for (auto &X: ((type_tup*)val.real()->ptr)->reals)
      write_bin_route(stream, X->type, value(X));
  }
}



namespace ds
{
  // Reader
  [[nodiscard]] fun value::makeFromStream(istream* stream) -> value
  {
    magic Magic;
    
    auto Pos{stream->tellg()};
    stream->read((char*)&Magic, sizeof(Magic));
    stream->seekg(Pos);

    return __builtin_memcmp(&Magic, &ExpectedMagic, sizeof(Magic)) == 0
      ? value::makeFromStreamBin(stream)
      : value::makeFromStreamRaw(stream);
  }

  [[nodiscard]] fun value::makeFromStreamBin(istream* stream) -> value
  {
    header Head;
    
    stream->read((char*)&Head, sizeof(Head));
    
    if (__builtin_memcmp(&Head.Magic, &ExpectedMagic, sizeof(Head.Magic)) != 0)
      throw runtime_error("file is not a ds file");

    return parse_bin_stc(stream);
  }

  [[nodiscard]] fun value::makeFromStreamRaw(istream* stream) -> value
  {
    struct word {i32 x{},y{},l{};};
    vector<word> Tokens;
    vector<string> Lines;


    string Chars[] = {
      " ","@",".","::",":",",",";",
      "(",")","[","]","{","}","<",">"
    };

    enum eToken {Str,Int,Sym,Txt};


    #pragma region Parse File
    {
      word Temp;
      string Line;
      bool InString{false};

      i32 y{-1};
      while (getline(*stream, Line))
      {
        Lines.push_back(Line);
        y++;

        if (Temp.l != 0 && !InString) {
          Tokens.push_back(Temp);
          Temp = {};
        }

        if (InString)
          Temp.l += +1;

        
        // Line Scan
        for (i32 x{}; x < Line.size(); x++)
        {
          const auto C = Line[x];
          

          // String
          if (C == '"')
          {
            int Backslashes{};
            int Check{x-1};
            while (Check >= 0 && Line[Check] == '\\') {
              Backslashes++;
              Check--;
            }

            bool IsEscaped = (Backslashes % 2 == 1);

            if (!IsEscaped)
            {
              if (!InString && Temp.l == 0) {
                Temp.x = x; Temp.y = y;
              }
              InString = !InString;
            }
          }

          if (InString) {
            Temp.l++;
            continue;
          }


          // Space
          if (C == ' ') {
            if (Temp.l != 0) {
              Tokens.push_back(Temp);
              Temp = {};
            }
            continue;
          }


          // Comment
          if (C == '#') {
            if (Temp.l != 0) {
              Tokens.push_back(Temp);
              Temp = {};
            }

            goto _l_NextLine;
          }


          // Others
          for (const auto &Delim: Chars)
            if (Line.compare(x, Delim.size(), Delim) == 0) {
              if (Temp.l != 0) {
                Tokens.push_back(Temp);
                Temp = {};
              }

              Tokens.push_back({x,y, (i32)Delim.size()});
              x += Delim.size() -1;

              goto _l_NextChar;
            }


          // Add
          if (Temp.l == 0) {
            Temp.x = x;
            Temp.y = y;
          }
          Temp.l++;

          // Next
          _l_NextChar: {}
        }

        _l_NextLine: {}
      }

      if (Temp.l != 0) {
        Tokens.push_back(Temp);
        Temp = {};
      }

    }
    #pragma endregion


    auto digitCount = [](u64 n) { return (n == 0) ? 1 : static_cast<u32>(std::log10(n)) +1; };

    auto utf8_length = [](string_view s) { u0 n{}; for (char c: s) if ((c & 0xC0) != 0x80) n++; return n; };


    auto onError = [digitCount, utf8_length, &Lines][[noreturn]] (const word& Word, string_view Msg)
    {
      const auto GRAY =    "\033[1;30m";
      const auto GREEN =   "\033[1;32m";
      const auto BLUE =    "\033[1;34m";
      const auto ORANGE =  "\033[1;33m";
      const auto RED =     "\033[1;91m";
      const auto RESET =   "\033[0m";

      string GLine{Lines[Word.y]};

      cerr
        << BLUE << "anonimous file" << GRAY << " :" << RESET << (Word.y+1) << GRAY << ":" << RESET << (Word.x+1) << GRAY << ": "
        << RED << "error " << RESET << Msg << ": \"" << string_view(GLine).substr(Word.x, Word.l) << "\""
        << endl;
      
      cerr
        << RESET << "  " << to_string(Word.y+1) << GRAY << " | " << RESET 
        << (GLine.empty() ? "":(GLine
          .insert(Word.x +Word.l, RESET)
          .insert(Word.x, ORANGE)))
        << endl;

      cerr << "  ";
      for (i64 i{}; i < digitCount(Word.y+1); i++)
        cerr << " ";

      cerr << GRAY << " | " << RESET;

      for (i64 i{}; i < Word.x; i++)
        cerr << " ";

      cerr << GREEN << "^";

      for (i64 i{1}; i < utf8_length(string_view(GLine).substr(Word.x, Word.l)); i++)
        cerr << "~";

      cerr << RESET << endl;

      throw runtime_error("an error encountered");
    };

    auto getType = [&Chars](string_view Key) -> eToken
    {
      if (Key[0] == '"')
        return eToken::Str;

      if (isdigit(Key[0]))
        return eToken::Int;

      for (const auto &Delim: Chars)
        if (Key.compare(0, Delim.size(), Delim) == 0)
          return eToken::Sym;

      return eToken::Txt;
    };

    auto unquote = [](string_view Str) -> string
    {
      // En az 2 karakter olmalı ("")
      if (Str.size() < 2 || Str.front() != '"' || Str.back() != '"')
        return string(Str);

      string Raw = string(Str.substr(1, Str.size() -2));
      string Res;
      Res.reserve(Raw.size());

      for (u0 i = 0; i < Raw.size(); ++i)
      {
        // Kaçış karakteri mi? (\)
        if (Raw[i] == '\\' && i +1 < Raw.size())
        {
          switch (Raw[i +1])
          {
            case '"':  Res += '"';  break;
            case '\'': Res += '\''; break;
            case '\\': Res += '\\'; break;
            case 'a':  Res += '\a'; break;
            case 'b':  Res += '\b'; break;
            case 'e':  Res += '\e'; break;
            case 'f':  Res += '\f'; break;
            case 'n':  Res += '\n'; break;
            case 'r':  Res += '\r'; break;
            case 't':  Res += '\t'; break;
            case 'v':  Res += '\v'; break;

            // Bilinmeyen escape ise olduğu gibi bırak (örn: \a)
            default:
              Res += '\\';
              Res += Raw[i +1];
              break;
          }
          i++; // Bir sonraki karakteri (örn: n) atla, çünkü işledik.
        }
        else
          Res += Raw[i];
      }

      return Res;
    };



    // Root
    value Root = value::makeStruct();


    #pragma region Interpreter
    {
      #define isEnd  (Step >= Size)
      #define Next()  {Step++; if (isEnd) goto _l_Escape;}
      #define Get  (Tokens[Step])
      #define Word  (string_view(Lines[Get.y]).substr(Get.x, Get.l))
      #define PeekWord(off) (string_view(Lines[Tokens[Step+(off)].y]).substr(Tokens[Step+(off)].x, Tokens[Step+(off)].l))

      u0 Size = Tokens.size();
      u0 Step = 0;

      // Root
      value Cov = Root.nref();
      vector<value> Chain;
      Chain.push_back(Cov.nref());

      
      // Start
      while (!isEnd)
      {
        string Name;

        _l_Start:

        // Exit Array
        if (Cov.isArray() && Word[0] == ']') {
          Chain.pop_back();
          Cov = Chain.back().nref();

          Next();

          goto _l_Finish;
        }


        if (Cov.isArray())
          goto _l_Content;


        // Exit Struct
        if (Cov.isStruct() && Word[0] == '}') {
          Chain.pop_back();
          Cov = Chain.back().nref();

          Next();

          goto _l_Finish;
        }



        // Name
        if (getType(Word) != eToken::Txt)
          onError(Get, "invalid identifier");

        Name = Word;
        Next();


        // :
        if (Word == ":")
          Next();


        _l_Content: {
          vector<value> TempTuple;
          u8 Mask{};
          bool EntersNode{false};
          auto NodeRef = value::makeUnDef();


          _l_Tuple_Start:

          auto O = value::makeUnDef();
          bool isNode{false};


          // bool ~1
          if (Word == "true" || Word == "false" || Word == "yes" || Word == "no")
          {
            if (Mask & 1) onError(Get, "tuple: duplicate bool");
            Mask |= 1;

            O = value::makeBool(Word == "true" || Word == "yes");
            Next();
          }

          // string ~2
          ef (getType(Word) == eToken::Str)
          {
            if (Mask & 2) onError(Get, "tuple: duplicate string");
            Mask |= 2;

            O = value::makeString(unquote(Word));
            Next();
          }

          // type ~4
          ef (getType(Word) == eToken::Txt)
          {
            if (Mask & 4) onError(Get, "tuple: duplicate type");
            Mask |= 4;

            string Typ{Word};
            Next();


            if (Word != "::")
              onError(Get, "expected \"::\"");
            Next();

            if (getType(Word) != eToken::Txt)
              onError(Get, "expected text");

            O = value::makeType(Typ, Word);
            Next();
          }

          // reference ~8
          ef (Word[0] == '@')
          {
            if (Mask & 8) onError(Get, "tuple: duplicate reference");
            Mask |= 8;

            Next();

            if (Word != "^" && getType(Word) != eToken::Str)
              onError(Get, "expected string or ^");

            O = value::makeRef(unquote(Word));
            Next();
          }

          // integer / float ~16
          ef (getType(Word) == eToken::Int || Word == "-")
          {
            if (Mask & 16) onError(Get, "tuple: duplicate number");
            Mask |= 16;

            string Temp;
            if (Word == "-") {
              Next();
              Temp = "-" +string(Word);
            }
            el
              Temp = Word;
            Next();


            bool IsFloat{};
            if (!isEnd && Word == ".") {
              IsFloat = true;
              Next();
              Temp += "." +string(Word);
              Next();
            }


            if (IsFloat) {
            #ifndef __wasm__
              f64 T{};
              auto eret = from_chars(Temp.data(), Temp.data() +Temp.size(), T);
              if (eret.ec != std::errc())
                onError(Get, "convert to float");
              
              O = value::makeF64(T);
            #else
              f64 T{};
              char* end;
              T = std::strtod(Temp.data(), &end);
              if (end == Temp.data())
                onError(Get, "convert to float");

              O = value::makeF64(T);
            #endif
            } else {
              i64 T{};
              auto eret = from_chars(Temp.data(), Temp.data() +Temp.size(), T);
              if (eret.ec != std::errc())
                onError(Get, "convert to integer");

              O = value::makeI64(T);;
            }
          }

          // struct ~32
          ef (Word[0] == '{')
          {
            if (Mask & 32) onError(Get, "tuple: duplicate struct");
            Mask |= 32;

            O = value::makeStruct();
            NodeRef = O.nref();
            isNode = true;
            EntersNode = true;
          }

          // array ~64
          ef (Word[0] == '[')
          {
            if (Mask & 64) onError(Get, "tuple: duplicate array");
            Mask |= 64;

            O = value::makeArray();
            NodeRef = O.nref();
            isNode = true;
            EntersNode = true;
          }

          // null ~128
          ef (Word == "null")
          {
            if (Mask & 128) onError(Get, "tuple: duplicate null");
            Mask |= 128;

            O = value::makeNull();
            Next();
          }

          else
            onError(Get, "invalid value");

          
          TempTuple.push_back(std::move(O));


          if (!isNode && !isEnd && Word[0] != ',' && Word[0] != '}' && Word[0] != ']')
          {
            bool isNextKey{false};
            
            if (Cov.isStruct() && getType(Word) == eToken::Txt) {
              if (Word == "true" || Word == "false" || Word == "yes" || Word == "no")
                isNextKey = false;
              ef (Step + 1 < Size && PeekWord(1) == "::")
                isNextKey = false;
              el
                isNextKey = true;
            }

            if (!isNextKey)
              goto _l_Tuple_Start;
          }

          // Tuple Sepetini Kaydet
          auto FinalVal = value::makeUnDef();
          if (TempTuple.size() == 1)
            FinalVal = std::move(TempTuple[0]);
          else {
            FinalVal = value::makeTuple();

            for (auto &v: TempTuple)
            {
              auto T = v.m_real->type;
              FinalVal.push_back(std::move(v), T);
            }
          }

          // Ana nesneye aktar
          if (Cov.isArray())  Cov.push_back(std::move(FinalVal));
          ef (Cov.isStruct()) Cov.push_back(std::move(FinalVal), Name);

          // Eğer `{` veya `[` geldiyse, nesting root (kök) değişimi yap
          if (EntersNode) {
            if (Chain.size() >= 256) onError(Get, "nested overflow");
            Cov = std::move(NodeRef);
            Chain.push_back(Cov.nref());
            Next();
          }
        }

        _l_Finish:

        // ,
        if (Word[0] == ',')
          Next();
      }

      _l_Escape:

      #undef PeekWord
      #undef Word
      #undef Get
      #undef Next
      #undef isEnd
    }
    #pragma endregion


    return std::move(Root);
  }


  fun value::saveToStreamRaw(ostream* stream, value& val) -> void {}
  fun value::saveToStreamBin(ostream* stream, value& val) -> void
  {
    header Head = {ExpectedMagic, VerMaj, VerMin};

    stream->write((char*)&Head, sizeof(Head));

    write_bin_stc(stream, val.nref());
  }



  // Helpers
  [[nodiscard]] fun value::makeNRef(const value& it) -> value { return value(it.m_real); }

  [[nodiscard]] fun value::makeCopy(const value& it) -> value { switch (it.m_real->type) {
    case etype::etStr: return value::makeString(it.w_string());

    case etype::etI64: return value::makeI64(it.w_int());
    case etype::etF64: return value::makeF64(it.w_int());
    case etype::etBoo: return value::makeBool(it.w_bool());

    case etype::etArr:
    if (!it.isView())
      {
        auto Org = value::makeArray();

        ((type_arr*)Org.m_real->ptr)->reals.reserve(it.size());

        for (auto &Item: ((type_arr*)it.m_real->ptr)->reals)
          Org.push_back(value::makeCopy(Item));

        return Org;
      }
    else
      {
        auto Org = value::makeArray();

        ((type_arr*)Org.m_real->ptr)->reals.reserve(it.size());

        for (u0 i = 0; i < it.size(); i++)
          Org.push_back(value(create_sptr({
            ((type_varr*)it.m_real->ptr)->vals[i],
            ((type_varr*)it.m_real->ptr)->types[i],
            false
          })));

        return Org;
      }


    case etype::etStc:
    if (!it.isView())
      {
        auto Org = value::makeStruct();

        ((type_stc*)Org.m_real->ptr)->reals.reserve(it.size());
        ((type_stc*)Org.m_real->ptr)->names.reserve(it.size());

        for (u0 i = 0; i < ((type_stc*)it.m_real->ptr)->reals.size(); i++)
          Org.push_back(value::makeCopy(((type_stc*)it.m_real->ptr)->reals[i]), ((type_stc*)it.m_real->ptr)->names[i]);

        return Org;
      }
    else
      {
        auto Org = value::makeStruct();

        ((type_stc*)Org.m_real->ptr)->reals.reserve(it.size());
        ((type_stc*)Org.m_real->ptr)->names.reserve(it.size());

        for (u0 i = 0; i < ((type_vstc*)it.m_real->ptr)->vals.size(); i++)
          Org.push_back(value(create_sptr({
            ((type_vstc*)it.m_real->ptr)->vals[i],
            ((type_vstc*)it.m_real->ptr)->types[i],
            false
          })),
            ((type_stc*)it.m_real->ptr)->names[i]
          );

        return Org;
      }
      

    default: return value::makeUnDef();
  }}

}
