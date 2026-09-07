use qwc_diagnostic::Summary;
use qwc_hir::{AnyId, Item, ItemId, Krate, Visitor};

use crate::{Export, ExportKind, ExportMap};



impl Visitor for ExportMap {
  type Type = Self;

  fn visit(cre: &Krate) -> Result<Self::Type, Summary> {
    ExportCollector::collect(cre)
  }
}

pub struct ExportCollector<'a> {
  cre: &'a Krate,
  scp: ExportMap,
  sum: Summary,
}

impl<'a> ExportCollector<'a> {

  pub fn collect(cre: &'a Krate) -> Result<ExportMap, Summary> {
    let mut collector = Self {
      cre,
      scp: ExportMap::new(),
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
      let item: &Item = self.cre.get(id);

      // if !item.is_public() { continue; }

      let (name, kind) = match item {
        // Modüller / İsim Alanları
        Item::NameSpace { name, .. } => (*name, ExportKind::NameSpace(id)),

        // Tipler, Fonksiyonlar ve Değişkenler
        //Item::Fun { name, .. } => (*name, ExportKind::Fun(id)),
        //Item::Type { name, .. } | Item::Struct { name, .. } => (*name, ExportKind::Type(id)),
        //Item::Using { name, .. } => (*name, ExportKind::Alias(id)),
        //Item::Const { name, .. } => (*name, ExportKind::Const(id)),

        // Generic Fonksiyon/Yapı tanımları (Generic kabuğun kendisi export edilir)
        //Item::GenericNS { name, .. } => (*name, ExportKind::Generic(id)),

        _ => todo!("{item:#?}"),
      };

      current_exports.insert(name, kind);
    }

    // Kapsamı kaydet
    self.scp.map.insert(container_id, current_exports);

    // Yalnızca alt NameSpace'lerin içine gir (Özyineleme)
    for &id in &item_ids {
      let item: &Item = self.cre.get(id);
      match item {
        Item::NameSpace { .. } => {
          self.process_container(id.to_any());
        }

        _ => {}
      }
    }
  }


  fn get_container_items(&self, id: AnyId) -> Vec<ItemId> {
    let item: &Item = self.cre.get(ItemId::from_any(id));
    match item {
      // Kök krate veya alt isim alanlarının çocuklarını çek
      Item::NameSpace {rng, ..} => {
        self.cre.extra_get(*rng).map(ItemId::new_from).collect()
      }

      _ => Vec::new(),
    }
  }
}
