/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#include "qw/basis.hh"
#include "qw/pretype.hh"
#include "qw/diagnostic/i18n.hh"
#include "qw/diagnostic/diagnostic.hh"
#include <format>
#include <iostream>
#include <ostream>
#include <string>
#include <string_view>


namespace qw::diagnostic
{

  inline fun format_from_vector(const std::string &fmt_str, const std::vector<std::string> &v) -> std::string
  {
    switch (v.size()) {
        case 0: return fmt_str;
        case 1: return vformat(fmt_str, make_format_args(v[0]));
        case 2: return vformat(fmt_str, make_format_args(v[0], v[1]));
        case 3: return vformat(fmt_str, make_format_args(v[0], v[1], v[2]));
        case 4: return vformat(fmt_str, make_format_args(v[0], v[1], v[2], v[3]));
        default: return "Hata: Çok fazla parametre!";
    }
  }

  fun operator <<(std::ostream &os, qMessage *msg) -> std::ostream&
  {
    // File
    if (msg->pos()) {
      auto [HR1, HR2] = msg->pos().interval();

      os
        << color::BLUE << msg->pos().fpath()
        << color::GRAY << ":" << color::RESET << HR1.y
        << color::GRAY << ":" << color::RESET << HR1.x
        << color::GRAY << ": " << color::RESET;
    }


    // Message
    switch (msg->type()) {
      case qMsgType::mtFatal:   os << color::RED    << _("fatal"); break;
      case qMsgType::mtError:   os << color::YELLOW << _("error"); break;
      case qMsgType::mtWarning: os << color::YELLOW << _("warning"); break;
      case qMsgType::mtHint:    os << color::GREEN  << _("hint"); break;
      case qMsgType::mtNote:    os << color::GREEN  << _("note"); break;
    }
    os << color::GRAY << ":" << color::RESET << " " << format_from_vector(msg->msg(), msg->params()) << '\n';


    // In file
    if (msg->pos()) {
      auto file = msg->pos().file();
      auto [HR1, HR2] = msg->pos().interval();

      u0 Beg = file.rfind('\n', msg->pos().off());
      Beg = (Beg == std::string_view::npos) ? 0 : Beg+1;

      u0 End = file.find('\n', msg->pos().off() + msg->pos().size());
      if (End == std::string_view::npos) End = file.size();

      u0 max_line = std::max(HR1.y, HR2.y);
      int padd = std::to_string(max_line).length();
      u0 line = HR1.y;


      u0 off = msg->pos().off(), size = msg->pos().size(), end_off = off + size;

      std::string line_buf, under_buf;
      bool has_underline{};


      for (u0 i = Beg; i <= End; i++)
      {
        char c = (i < file.size()) ? file[i] : '\n';

        if (c != '\n' && i != End) 
        {
          line_buf += c;
          
          if (i == off) {
            under_buf += '^';
            has_underline = true;
          } else if (i > off && i < end_off) {
            under_buf += '~';
            has_underline = true;
          } else
            under_buf += (c == '\t') ? '\t' : ' ';
        }

        if (c == '\n' || i == End) 
        {
          if (i == End && line_buf.empty() && !has_underline && c == '\n') break;

          os << "  " << std::string(padd - std::to_string(line).length() + 1, ' ') 
            << line << color::GRAY << " | " << color::RESET << line_buf << '\n';

          if (has_underline)
            os << "  " << std::string(padd + 1, ' ') 
              << color::GRAY << " | " << color::RESET 
              << color::GREEN << under_buf << color::RESET << '\n';

          line++;
          line_buf.clear();
          under_buf.clear();
          has_underline = false;
        }
      }
      
      os << '\n';


      /*
      u0 Beg = file.rfind('\n', msg->pos().off());
      u0 End = file.find('\n', msg->pos().off()+msg->pos().size());

      u0 padd = log10(max(HR1.y, HR2.y));
      u0 line = HR1.y;

      cerr << "HR " << HR1.y << ":" << HR2.y;


      for (u0 i = Beg; i < End; i++)
      {
        if (i == Beg || file[i] == '\n') {
          os << '\n' << "  " << std::string(padd-log10(line)+1, ' ') << line << color::GRAY << " | " << color::RESET;

          continue;
        }

        os << file[i];
      }


      //os << "  " << std::string(log10(msg->pos()->HY)+1, ' ') << color::GRAY << " | " << color::RESET << std::string(msg->pos()->HX, ' ') << color::B_GREEN << "^" << std::string(msg->pos()->Size-1, '~') << color::RESET << endl;
      
      //Beg = (Beg == std::string_view::npos) ? 0 : Beg+1;
      //if (End == std::string_view::npos) End = msg->pos().file().size();

      //os << "  " << msg->pos()->HY << color::GRAY << " | " << color::RESET << msg->pos()->File.substr(Beg, End-Beg) << endl;
      */
    }


    return os;
  }

  fun operator <<(std::ostream &os, qSummary &sum) -> std::ostream&
  {
    std::string out;
    
    if (auto x = sum.fatal())   out += std::to_string(x) + " " + _("fatal") + ", ";
    if (auto x = sum.error())   out += std::to_string(x) + " " + _("error") + ", ";
    if (auto x = sum.warning()) out += std::to_string(x) + " " + _("warning") + ", ";
    if (auto x = sum.hint())    out += std::to_string(x) + " " + _("hint") + ", ";
    if (auto x = sum.note())    out += std::to_string(x) + " " + _("note") + ", ";


    return os
      << (sum.fatal() ? color::RED : color::YELLOW)
        << _("summary") << color::GRAY << ":" << color::RESET << " "
      << std::string_view(out).substr(0, out.size()-2)
      << " " + _("generated.") << std::endl;
  }




  [[gnu::noreturn]] fun fatal(std::string msg) -> void
  {
    std::cerr << "qw encountered a fatal error:" << std::endl;

    #ifdef _DBG
    throw std::runtime_error(msg);
    #else
    cerr << "what(): " << msg << endl;
    exit(1);
    #endif
  }
  
}
