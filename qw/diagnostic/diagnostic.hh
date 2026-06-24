/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#pragma once

#include "qw/basis.hh"
#include "qw/pretype.hh"
#include <ostream>
#include <string>



namespace qw::diagnostic
{

  enum struct MsgType: u8 { Fatal, Error, Warning, Hint, Note };

  struct message
  {
    protected:
      inline message(MsgType MsgType, word pos, std::string msg, std::vector<std::string> params)
        : m_msgType(MsgType)
        , m_pos(pos)
        , m_msg(msg)
        , m_params(params)
      {}

    private:
      MsgType m_msgType{};
      word m_pos;
      std::string m_msg;
      std::vector<std::string> m_params;

    public:
      fun type() { return m_msgType; }
      fun pos() { return m_pos; }
      fun msg() { return m_msg; }
      fun params() { return m_params; }
  };
  fun operator<<(std::ostream &os, message *msg) -> std::ostream&;

  struct fatal: message
  {
    inline fatal(std::string Msg, std::vector<std::string> Params)
      : message(MsgType::Fatal, word{}, Msg, Params)
    {}

    static fun make(std::string Msg, std::vector<std::string> Params = {}) { return make_uptr<fatal>(Msg, Params); }
  };

  struct error: message
  {
    inline error(word Pos, std::string Msg, std::vector<std::string> Params)
      : message(MsgType::Error, Pos, Msg, Params)
    {}

    static fun make(word Pos, std::string Msg, std::vector<std::string> Params = {}) { return make_uptr<error>(Pos, Msg, Params); }
  };

  struct warning: message
  {
    inline warning(word Pos, std::string Msg, std::vector<std::string> Params)
      : message(MsgType::Warning, Pos, Msg, Params)
    {}

    static fun make(word Pos, std::string Msg, std::vector<std::string> Params = {}) { return make_uptr<warning>(Pos, Msg, Params); }
  };

  struct hint: message
  {
    inline hint(word Pos, std::string Msg, std::vector<std::string> Params)
      : message(MsgType::Hint, Pos, Msg, Params)
    {}

    static fun make(word Pos, std::string Msg, std::vector<std::string> Params = {}) { return make_uptr<hint>(Pos, Msg, Params); }
  };

  struct note: message
  {
    inline note(word Pos, std::string Msg, std::vector<std::string> Params)
      : message(MsgType::Note, Pos, Msg, Params)
    {}

    static fun make(word Pos, std::string Msg, std::vector<std::string> Params = {}) { return make_uptr<note>(Pos, Msg, Params); }
  };

  struct summary
  {
    private:
      u32 m_fatal{}, m_error{}, m_warning{}, m_hint{}, m_note{};

    public:
      fun fatal() { return m_fatal; }
      fun error() { return m_error; }
      fun warning() { return m_warning; }
      fun hint() { return m_hint; }
      fun note() { return m_note; }

    public:
      fun add(message *Msg)
      {
        switch (Msg->type()) {
          case MsgType::Fatal: m_fatal++; return;
          case MsgType::Error: m_error++; return;
          case MsgType::Warning: m_warning++; return;
          case MsgType::Hint: m_hint++; return;
          case MsgType::Note: m_note++; return;
        }
      }

      fun sumall() { return (m_fatal + m_error + m_warning + m_hint + m_note); }
      fun sumerr() { return (m_fatal + m_error); }
  };
  fun operator<<(std::ostream &os, summary &msg) -> std::ostream&;

  [[gnu::noreturn]] fun fatal(std::string) -> void;
}
