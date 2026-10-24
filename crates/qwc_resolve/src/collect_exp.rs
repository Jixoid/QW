/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use qwc_diagnostic::Summary;
use qwc_hir::{AnyId, Deps, Item, ItemAttr, ItemId, ItemKind, Krate};
use qwc_mangling::{Mangler, ManglerQW};

use qwc_string_interner::StrInterner;

use crate::{Export, ExportKind, ExportMap};


impl ExportMap {
  pub fn visit(cre: &Krate, deps: &Deps, sin: &StrInterner) -> Result<Self, Summary> {
    ExportCollector::collect(cre, deps, sin)
  }
}


pub struct ExportCollector<'a> {
  cre: &'a Krate,
  deps: &'a Deps,
  sin: &'a StrInterner,
  scp: ExportMap,
  sum: Summary,
}

impl<'a> ExportCollector<'a> {

  pub fn collect(cre: &'a Krate, deps: &'a Deps, sin: &'a StrInterner) -> Result<ExportMap, Summary> {
    let mut collector = Self {
      cre,
      deps,
      sin,
      scp: ExportMap::new(cre.cid(), cre.root()),
      sum: Summary::new(),
    };

    if let Some(root_id) = cre.root() {
      collector.process_container(root_id.to_any());
    }

    if collector.sum.is_empty() {
      Ok(collector.scp)
    } else {
      Err(collector.sum)
    }
  }

  fn process_container(&mut self, container_id: AnyId) {
    // Export tablosunda 'parent' zinciri tutulmaz; sadece sembol sözlüğü üretilir:
    let mut current_exports = Export::new(Some(container_id));

    // Kapsayıcının içindeki HIR item ID listesini al
    let item_ids = self.get_container_items(container_id);

    // Mevcut modülün dışa açtığı tüm sembolleri kaydet
    for &id in &item_ids {
      let it = self.cre.get(id);

      // if !item.is_public() { continue; }

      let (name_sid, kind) = match it.kind {
        // Modüller / İsim Alanları
        ItemKind::NameSpace{..} => (it.name.unwrap(), ExportKind::NameSpace(id)),

        ItemKind::Using{kind} => (it.name.unwrap(), ExportKind::Type(kind)),

        ItemKind::Variable{kind, ..} => {
          let sym = ManglerQW::new(self.cre, self.deps, self.sin, it.path);
          self.scp.add_symbol(id, sym);
          (it.name.unwrap(), ExportKind::Expr(id, kind))
        }

        ItemKind::Function{kind, ..} => {
          let sym = if it.attr.contains(ItemAttr::Entry) {
            String::from("qw_entry")
          } else {
            ManglerQW::new(self.cre, self.deps, self.sin, it.path)
          };
          self.scp.add_symbol(id, sym);
          (it.name.unwrap(), ExportKind::Expr(id, kind))
        }

        ItemKind::GenericNS{..} => continue,
        ItemKind::Impl{methods, ..} => {
          for method_id in self.cre.extra_get(methods) {
            let m_it = self.cre.get(method_id);
            if let ItemKind::Function { .. } = m_it.kind {
              let sym = ManglerQW::new(self.cre, self.deps, self.sin, m_it.path);
              self.scp.add_symbol(method_id, sym);
            }
          }
          continue;
        }

        _ => todo!("{it:#?}"),
      };

      let name = self.sin.str(name_sid).to_string();
      current_exports.insert(name, kind);
    }

    // Kapsamı kaydet
    self.scp.map.insert(container_id, current_exports);

    // Yalnızca alt NameSpace'lerin içine gir
    for &id in &item_ids {
      let item: &Item = self.cre.get(id);
      match item.kind {
        ItemKind::NameSpace{..} => self.process_container(id.to_any()),
        _ => {}
      }
    }
  }


  fn get_container_items(&self, id: AnyId) -> Vec<ItemId> {
    let item: &Item = self.cre.get(ItemId::from_any(id));
    match item.kind {
      ItemKind::RootNS{rng} | ItemKind::NameSpace{rng, ..} => self.cre.extra_get(rng).collect(),

      _ => vec![],
    }
  }
}
