/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#pragma once

#include "Basis.hh"
#include "Context.hh"
#include "Types.hh"

using namespace std;



namespace qw
{

  class scopeManager
  {
    public:
      inline scopeManager(qw::context *ctx, vector<gst*> nGST, vector<string> nANS): ctx(ctx), m_gst(nGST), m_ans(nANS) {}
  

    private:
      qw::context *ctx;
      vector<gst*> m_gst;
      vector<string> m_ans;

    public:
      static fun mangling_abi_qw(qIdentifier*) -> string;
      static fun humanreadableTypes(types::qType*) -> string;

    public:
      fun fetch_type(qIdentifier*) -> types::qType*;
      fun fetch_expr(qIdentifier*) -> exprs::qExpr*;
      fun lookup(qIdentifier*, vector<string> names) -> qIdentifier*;

    public:
      inline fun& gst() { return m_gst; } 
      inline fun& ans() { return m_ans; } 

  };

}
