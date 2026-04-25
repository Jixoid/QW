/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#pragma once

#include "Basis.hh"
#include "Types.hh"
#include <string>

using namespace std;


namespace qw::diagnostic
{

  enum qMsgType: u8 { mtFatal, mtError, mtWarning, mtHint, mtNote };


  struct qMessage
  {
    protected:
      inline qMessage(qMsgType MsgType, word Pos, string Msg, vector<string> Params)
        : m_msgType(MsgType)
        , m_pos(Pos)
        , m_msg(Msg)
        , m_params(Params)
      {}

    private:
      qMsgType m_msgType{};
      word m_pos;
      string m_msg;
      vector<string> m_params;

    public:
      inline fun type() { return m_msgType; }
      inline fun pos() { return m_pos; }
      inline fun msg() { return m_msg; }
      inline fun params() { return m_params; }
  };
  fun operator <<(ostream &os, qMessage *msg) -> ostream&;


  struct qFatal: qMessage
  {
    protected:
      inline qFatal(string Msg, vector<string> Params)
        : qMessage(qMsgType::mtFatal, word{}, Msg, Params)
      {}

    public:
      static inline fun make(string Msg, vector<string> Params = {}) {
        return make_uptr(new qFatal(Msg, Params));
      }
  };

  struct qError: qMessage
  {
    protected:
      inline qError(word Pos, string Msg, vector<string> Params)
        : qMessage(qMsgType::mtError, std::move(Pos), Msg, Params)
      {}

    public:
      static inline fun make(word Pos, string Msg, vector<string> Params = {}) {
        return make_uptr(new qError(std::move(Pos), Msg, Params));
      }
  };

  struct qWarning: qMessage
  {
    protected:
      inline qWarning(word Pos, string Msg, vector<string> Params)
        : qMessage(qMsgType::mtWarning, std::move(Pos), Msg, Params)
      {}

    public:
      static inline fun make(word Pos, string Msg, vector<string> Params = {}) {
        return make_uptr(new qWarning(std::move(Pos), Msg, Params));
      }
  };

  struct qHint: qMessage
  {
    protected:
      inline qHint(word Pos, string Msg, vector<string> Params)
        : qMessage(qMsgType::mtHint, std::move(Pos), Msg, Params)
      {}

    public:
      static inline fun make(word Pos, string Msg, vector<string> Params = {}) {
        return make_uptr(new qHint(std::move(Pos), Msg, Params));
      }
  };

  struct qNote: qMessage
  {
    protected:
      inline qNote(word Pos, string Msg, vector<string> Params)
        : qMessage(qMsgType::mtNote, std::move(Pos), Msg, Params)
      {}

    public:
      static inline fun make(word Pos, string Msg, vector<string> Params = {}) {
        return make_uptr(new qNote(std::move(Pos), Msg, Params));
      }
  };



  
  struct qSummary
  {
    private:
      u32 m_fatal{}, m_error{}, m_warning{}, m_hint{}, m_note{};

    public:
      inline fun fatal() { return m_fatal; }
      inline fun error() { return m_error; }
      inline fun warning() { return m_warning; }
      inline fun hint() { return m_hint; }
      inline fun note() { return m_note; }

    public:
      inline fun add(qMessage* Msg) { switch (Msg->type()) {
        case qMsgType::mtFatal: m_fatal++; return;
        case qMsgType::mtError: m_error++; return;
        case qMsgType::mtWarning: m_warning++; return;
        case qMsgType::mtHint: m_hint++; return;
        case qMsgType::mtNote: m_note++; return;
      }}

      inline fun suma() { return (m_fatal + m_error + m_warning + m_hint + m_note); }
      inline fun sum() { return (m_fatal + m_error); }
  };
  fun operator <<(ostream &os, qSummary &msg) -> ostream&;



  [[gnu::noreturn]] fun fatal(string) -> void;
}
