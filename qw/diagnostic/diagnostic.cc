/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#include "qw/diagnostic/diagnostic.hh"
#include "qw/basis.hh"
#include "qw/diagnostic/i18n.hh"
#include "qw/pretype.hh"
#include <format>
#include <iostream>
#include <ostream>
#include <string>
#include <string_view>

#define ef else if



namespace qw::diagnostic
{

  inline fun format_from_vector(const std::string &fmt_str, const std::vector<std::string> &v) -> std::string {
    switch (v.size()) {
      case 0: return fmt_str;
      case 1: return vformat(fmt_str, make_format_args(v[0]));
      case 2: return vformat(fmt_str, make_format_args(v[0], v[1]));
      case 3: return vformat(fmt_str, make_format_args(v[0], v[1], v[2]));
      case 4: return vformat(fmt_str, make_format_args(v[0], v[1], v[2], v[3]));
      default: return "Error: Too many parameters!";
    }
  }

  fun operator<<(std::ostream &os, message *msg) -> std::ostream &
  {
    // File
    if (msg->pos()) {
      auto [HR1, HR2] = msg->pos().interval();

      os << color::BLUE << msg->pos().fpath() << color::GRAY << ":" << color::RESET << HR1.y << color::GRAY << ":" << color::RESET << HR1.x << color::GRAY << ": " << color::RESET;
    }

    // Message
    switch (msg->type()) {
      case MsgType::Fatal:   os << color::RED    << _("fatal"); break;
      case MsgType::Error:   os << color::YELLOW << _("error"); break;
      case MsgType::Warning: os << color::YELLOW << _("warning"); break;
      case MsgType::Hint:    os << color::GREEN  << _("hint"); break;
      case MsgType::Note:    os << color::GREEN  << _("note"); break;
    }
    os << color::GRAY << ":" << color::RESET << " " << format_from_vector(msg->msg(), msg->params()) << '\n';

    // In file
    if (msg->pos()) {
      auto file       = msg->pos().file();
      auto [HR1, HR2] = msg->pos().interval();

      u0 Beg = file.rfind('\n', msg->pos().off());
      Beg    = (Beg == std::string_view::npos) ? 0 : Beg + 1;

      u0 End = file.find('\n', msg->pos().off() + msg->pos().size());
      if (End == std::string_view::npos)
        End = file.size();

      u0 max_line = std::max(HR1.y, HR2.y);
      int padd    = std::to_string(max_line).length();
      u0 line     = HR1.y;

      u0 off = msg->pos().off(), size = msg->pos().size(), end_off = off + size;

      std::string line_buf, under_buf;
      bool has_underline{};

      for (u0 i = Beg; i <= End; i++) {
        char c = (i < file.size()) ? file[i] : '\n';

        if (c != '\n' && i != End) {
          line_buf += c;

          if (i == off) {
            under_buf += '^';
            has_underline = true;
          }
          ef (i > off && i < end_off) {
            under_buf += '~';
            has_underline = true;
          }
          else
            under_buf += (c == '\t') ? '\t' : ' ';
        }

        if (c == '\n' || i == End) {
          if (i == End && line_buf.empty() && !has_underline && c == '\n')
            break;

          os << "  " << std::string(padd - std::to_string(line).length() + 1, ' ') << line << color::GRAY << " | " << color::RESET << line_buf << '\n';

          if (has_underline)
            os << "  " << std::string(padd + 1, ' ') << color::GRAY << " | " << color::RESET << color::GREEN << under_buf << color::RESET << '\n';

          line++;
          line_buf.clear();
          under_buf.clear();
          has_underline = false;
        }
      }

      os << '\n';
    }

    return os << std::flush;
  }

  fun operator<<(std::ostream &os, summary &sum) -> std::ostream &
  {
    std::string out;

    if (auto x = sum.fatal())   out += std::to_string(x) + " " + _("fatal")   + ", ";
    if (auto x = sum.error())   out += std::to_string(x) + " " + _("error")   + ", ";
    if (auto x = sum.warning()) out += std::to_string(x) + " " + _("warning") + ", ";
    if (auto x = sum.hint())    out += std::to_string(x) + " " + _("hint")    + ", ";
    if (auto x = sum.note())    out += std::to_string(x) + " " + _("note")    + ", ";

    return os << (sum.fatal() ? color::RED : color::YELLOW) << _("summary") << color::GRAY << ":" << color::RESET << " " << std::string_view(out).substr(0, out.size() - 2) << " " + _("generated.") << std::endl;
  }

  [[gnu::noreturn]] auto fatal(std::string msg) -> void
  {
    std::cerr << "qw encountered a fatal error:" << std::endl;

  #ifdef _DBG
    throw std::runtime_error(msg);
  #else
    std::cerr << "what(): " << msg << std::endl;
    exit(1);
  #endif
  }

}
