use super::identy::HirId;

#[derive(Debug, Clone, PartialEq)]
pub enum HirValue {
  /// Sabit bir Integer değeri.
  ConstInt(i64),
  
  /// Sabit bir Float değeri.
  ConstFloat(f64),
  
  /// Sabit Boolean değeri.
  ConstBool(bool),
  
  /// Bir SSA Virtual Register (Aslında bir Instr'in ürettiği sonuç referansıdır).
  Reg(HirId),
  
  /// Global bir değişkene veya fonksiyona erişim.
  Global(HirId),
  
  /// Null pointer veya void durumları.
  Null,
}
