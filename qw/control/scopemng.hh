/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#pragma once

#include "qw/basis.hh"
#include "qw/control/context.hh"
#include "qw/pretype.hh"
#include "qw/control/context.hh"
#include <string>



namespace qw
{

  class scopemng
  {
    public:
      inline scopemng(qw::context *ctx, std::vector<gst*> nGST, std::vector<std::string> nANS): ctx(ctx), m_gst(nGST), m_ans(nANS) {}
  

    private:
      qw::context *ctx;
      std::vector<gst*> m_gst;
      std::vector<std::string> m_ans;

    public:
      static fun mangling_abi_qw(identy*) -> std::string;
      static fun humanreadableTypes(types::Type*) -> std::string;

    public:
      fun fetch_type(identy*) -> types::Type*;
      fun fetch_expr(identy*) -> exprs::Expr*;
      fun lookup(identy*, std::vector<std::string> names) -> identy*;

    public:
      inline fun& gst() { return m_gst; } 
      inline fun& ans() { return m_ans; } 

  };

}
