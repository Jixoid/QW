/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

use std::num::NonZero;
use byteorder::{ByteOrder, LittleEndian};
use rustc_hash::FxHashMap;
use rustc_apfloat::{ieee, Float};

use qwc_hir::{
  AnyId, AnyRng, CID, Const, DefPath, Expr, ExprCategory,
  ExprKind, HirId, HirKind, IntArithmeticFlg, IntArithmeticOp,
  IntConditionOp, FloatArithmeticOp, BoolLogicOp, Item, ItemAttr, ItemKind,
  ItemVis, Layout, NodeKind, Rng, SymVis, Thing, Type, TypeAttr,
  TypeKind,
};
use qwc_resolve::{Export, ExportKind, ExportMap};
use qwc_string_interner::{Sid, StrInterner};
use qwc_diagnostic::Span;


pub struct Encoder<'a> {
  pub buf: Vec<u8>,
  pub sin: &'a StrInterner,
  pub string_map: FxHashMap<String, u32>,
  pub strings: Vec<String>,
}

impl<'a> Encoder<'a> {
  pub fn new(sin: &'a StrInterner) -> Self {
    Self {
      buf: Vec::new(),
      sin,
      string_map: FxHashMap::default(),
      strings: Vec::new(),
    }
  }

  #[inline]
  pub fn write_u8(&mut self, val: u8) {
    self.buf.push(val);
  }

  #[inline]
  pub fn write_u16(&mut self, val: u16) {
    let mut b = [0; 2];
    LittleEndian::write_u16(&mut b, val);
    self.buf.extend_from_slice(&b);
  }

  #[inline]
  pub fn write_u32(&mut self, val: u32) {
    let mut b = [0; 4];
    LittleEndian::write_u32(&mut b, val);
    self.buf.extend_from_slice(&b);
  }

  #[inline]
  pub fn write_u64(&mut self, val: u64) {
    let mut b = [0; 8];
    LittleEndian::write_u64(&mut b, val);
    self.buf.extend_from_slice(&b);
  }

  #[inline]
  pub fn write_u128(&mut self, val: u128) {
    let mut b = [0; 16];
    LittleEndian::write_u128(&mut b, val);
    self.buf.extend_from_slice(&b);
  }

  #[inline]
  pub fn write_bool(&mut self, val: bool) {
    self.write_u8(if val { 1 } else { 0 });
  }

  pub fn write_str(&mut self, s: &str) {
    self.write_u32(s.len() as u32);
    self.buf.extend_from_slice(s.as_bytes());
  }

  pub fn write_sid(&mut self, sid: Sid) {
    let s = self.sin.str(sid);
    let str_idx = if let Some(&idx) = self.string_map.get(s) {
      idx
    } else {
      let idx = self.strings.len() as u32;
      self.strings.push(s.to_string());
      self.string_map.insert(s.to_string(), idx);
      idx
    };
    self.write_u32(str_idx);
  }

  #[inline]
  pub fn write_cid(&mut self, cid: CID) {
    self.write_u16(cid.raw());
  }

  #[inline]
  pub fn write_hir_id<T: HirKind>(&mut self, id: HirId<T>) {
    self.write_cid(id.cid());
    self.write_u32(id.idx());
  }

  #[inline]
  pub fn write_rng<T: HirKind>(&mut self, rng: Rng<T>) {
    self.write_cid(rng.cid());
    self.write_u32(rng.start());
    self.write_u32(rng.end());
  }

  #[inline]
  pub fn write_any_id(&mut self, id: AnyId) {
    self.write_cid(id.cid());
    self.write_u32(id.idx());
    let k = match id.kind() {
      NodeKind::Any => 0,
      NodeKind::Type => 1,
      NodeKind::Expr => 2,
      NodeKind::Item => 3,
      NodeKind::Thing => 4,
      NodeKind::DefPath => 5,
    };
    self.write_u8(k);
  }

  #[inline]
  pub fn write_any_rng(&mut self, rng: AnyRng) {
    self.write_cid(rng.cid());
    self.write_u32(rng.start());
    self.write_u32(rng.end());
  }

  pub fn write_span(&mut self, span: Span) {
    if span.is_builtin() {
      self.write_bool(true);
    } else {
      self.write_bool(false);
      let (off, len, fid) = span.to();
      self.write_u32(off);
      self.write_u16(len.get());
      self.write_u16(fid);
    }
  }

  pub fn write_layout(&mut self, lay: Layout) {
    self.write_u8(lay.raw_kind());
    self.write_bool(lay.is_inhabited());
  }

  pub fn write_defpath(&mut self, dpath: &DefPath) {
    match dpath {
      DefPath::Root(sid) => {
        self.write_u8(0);
        self.write_sid(*sid);
      }
      DefPath::Path { base, name } => {
        self.write_u8(1);
        self.write_hir_id(*base);
        self.write_sid(*name);
      }
      DefPath::Impl { base, spec } => {
        self.write_u8(2);
        self.write_hir_id(*base);
        self.write_hir_id(*spec);
      }
      DefPath::Spec { base, spec } => {
        self.write_u8(3);
        self.write_hir_id(*base);
        self.write_hir_id(*spec);
      }
      DefPath::AnonymousTuple(rng) => {
        self.write_u8(4);
        self.write_rng(*rng);
      }
    }
  }

  pub fn write_const(&mut self, c: &Const) {
    match c {
      Const::Unit => self.write_u8(0),
      Const::Bool(b) => {
        self.write_u8(1);
        self.write_bool(*b);
      }
      Const::Str(sid) => {
        self.write_u8(2);
        self.write_sid(*sid);
      }
      Const::Char(ch) => {
        self.write_u8(3);
        self.write_u32(*ch as u32);
      }
      Const::U8(v) => { self.write_u8(4); self.write_u8(*v); }
      Const::U16(v) => { self.write_u8(5); self.write_u16(*v); }
      Const::U32(v) => { self.write_u8(6); self.write_u32(*v); }
      Const::U64(v) => { self.write_u8(7); self.write_u64(*v); }
      Const::U128(v) => { self.write_u8(8); self.write_u128(*v); }
      Const::I8(v) => { self.write_u8(9); self.write_u8(*v as u8); }
      Const::I16(v) => { self.write_u8(10); self.write_u16(*v as u16); }
      Const::I32(v) => { self.write_u8(11); self.write_u32(*v as u32); }
      Const::I64(v) => { self.write_u8(12); self.write_u64(*v as u64); }
      Const::I128(v) => { self.write_u8(13); self.write_u128(*v as u128); }
      Const::F16(v) => { self.write_u8(14); self.write_u16(v.to_bits() as u16); }
      Const::F32(v) => { self.write_u8(15); self.write_u32(v.to_bits() as u32); }
      Const::F64(v) => { self.write_u8(16); self.write_u64(v.to_bits() as u64); }
      Const::F128(v) => { self.write_u8(17); self.write_u128(v.to_bits()); }
      Const::Array(ty, rng) => {
        self.write_u8(18);
        self.write_hir_id(*ty);
        self.write_rng(*rng);
      }
    }
  }

  pub fn write_thing(&mut self, thing: &Thing) {
    match thing {
      Thing::Name(sid) => {
        self.write_u8(0);
        self.write_sid(*sid);
      }
      Thing::NamedType(sid, ty) => {
        self.write_u8(1);
        self.write_sid(*sid);
        self.write_hir_id(*ty);
      }
      Thing::NamedExpr(sid, expr) => {
        self.write_u8(2);
        self.write_sid(*sid);
        self.write_hir_id(*expr);
      }
      Thing::NamedConst(sid, c) => {
        self.write_u8(3);
        self.write_sid(*sid);
        self.write_const(c);
      }
    }
  }

  pub fn write_item(&mut self, item: &Item) {
    let vis_byte = match item.vis {
      ItemVis::Private => 0,
      ItemVis::Public(SymVis::Internal) => 1,
      ItemVis::Public(SymVis::Export) => 2,
      ItemVis::Public(SymVis::Import) => 3,
    };
    self.write_u8(vis_byte);

    match item.name {
      Some(sid) => {
        self.write_bool(true);
        self.write_sid(sid);
      }
      None => self.write_bool(false),
    }

    self.write_u16(item.attr.bits());
    self.write_hir_id(item.path);

    match item.kind {
      ItemKind::RootNS { rng } => {
        self.write_u8(0);
        self.write_rng(rng);
      }
      ItemKind::NameSpace { rng } => {
        self.write_u8(1);
        self.write_rng(rng);
      }
      ItemKind::GenericNS { rng, args } => {
        self.write_u8(2);
        self.write_rng(rng);
        self.write_rng(args);
      }
      ItemKind::Using { kind } => {
        self.write_u8(3);
        self.write_hir_id(kind);
      }
      ItemKind::Variable { kind, expr, ism } => {
        self.write_u8(4);
        self.write_hir_id(kind);
        self.write_hir_id(expr);
        self.write_bool(ism);
      }
      ItemKind::Function { kind, expr } => {
        self.write_u8(5);
        self.write_hir_id(kind);
        self.write_hir_id(expr);
      }
      ItemKind::Task { kind, expr } => {
        self.write_u8(6);
        self.write_hir_id(kind);
        self.write_hir_id(expr);
      }
      ItemKind::Impl { type_ty, trait_ty, methods } => {
        self.write_u8(7);
        self.write_hir_id(type_ty);
        match trait_ty {
          Some(tt) => {
            self.write_bool(true);
            self.write_hir_id(tt);
          }
          None => self.write_bool(false),
        }
        self.write_rng(methods);
      }
    }
  }

  pub fn write_type(&mut self, ty: &Type) {
    self.write_layout(ty.layout);
    self.write_u16(ty.attr.bits());

    match ty.kind {
      TypeKind::GenericType { generic, idx } => {
        self.write_u8(0);
        self.write_hir_id(generic);
        self.write_u32(idx as u32);
      }
      TypeKind::GenericRaw { generic, kind } => {
        self.write_u8(1);
        self.write_hir_id(generic);
        self.write_hir_id(kind);
      }
      TypeKind::GenericSelfType => self.write_u8(2),
      TypeKind::AssociatedType { trait_ty, idx } => {
        self.write_u8(3);
        self.write_hir_id(trait_ty);
        self.write_u32(idx as u32);
      }
      TypeKind::Error => self.write_u8(4),
      TypeKind::Unit => self.write_u8(5),
      TypeKind::Never => self.write_u8(6),
      TypeKind::Int(w, s) => {
        self.write_u8(7);
        self.write_u16(w);
        self.write_bool(s);
      }
      TypeKind::ArchInt(s) => {
        self.write_u8(8);
        self.write_bool(s);
      }
      TypeKind::Float(w) => {
        self.write_u8(9);
        self.write_u16(w);
      }
      TypeKind::Bit(w) => {
        self.write_u8(10);
        self.write_u16(w);
      }
      TypeKind::Bool => self.write_u8(11),
      TypeKind::Char => self.write_u8(12),
      TypeKind::Str => self.write_u8(13),
      TypeKind::Meta(k) => {
        self.write_u8(14);
        self.write_hir_id(k);
      }
      TypeKind::Ref(k, ism) => {
        self.write_u8(15);
        self.write_hir_id(k);
        self.write_bool(ism);
      }
      TypeKind::Ptr(k, ism) => {
        self.write_u8(16);
        self.write_hir_id(k);
        self.write_bool(ism);
      }
      TypeKind::Vector(k, len) => {
        self.write_u8(17);
        self.write_hir_id(k);
        self.write_hir_id(len);
      }
      TypeKind::VScale(k) => {
        self.write_u8(18);
        self.write_hir_id(k);
      }
      TypeKind::Array(k, len) => {
        self.write_u8(19);
        self.write_hir_id(k);
        self.write_hir_id(len);
      }
      TypeKind::Slice(k) => {
        self.write_u8(20);
        self.write_hir_id(k);
      }
      TypeKind::Struct(fields) => {
        self.write_u8(21);
        self.write_rng(fields);
      }
      TypeKind::Tuple(types) => {
        self.write_u8(22);
        self.write_rng(types);
      }
      TypeKind::Range(k) => {
        self.write_u8(23);
        self.write_hir_id(k);
      }
      TypeKind::ItemTrait { methods, types, reqs, path } => {
        self.write_u8(24);
        self.write_rng(methods);
        self.write_rng(types);
        self.write_rng(reqs);
        self.write_hir_id(path);
      }
      TypeKind::ItemIface{methods, path} => {
        self.write_u8(25);
        self.write_rng(methods);
        self.write_hir_id(path);
      }
      TypeKind::TraitFrom { trait_ty, hidden } => {
        self.write_u8(26);
        self.write_hir_id(trait_ty);
        self.write_hir_id(hidden);
      }
      TypeKind::ItemEnum{values: variants, path} => {
        self.write_u8(27);
        self.write_rng(variants);
        self.write_hir_id(path);
      }
      TypeKind::ItemFlag{values: variants, path} => {
        self.write_u8(28);
        self.write_rng(variants);
        self.write_hir_id(path);
      }
      TypeKind::Option(k) => {
        self.write_u8(29);
        self.write_hir_id(k);
      }
      TypeKind::Fun { self_kind, args, ret } => {
        self.write_u8(30);
        match self_kind {
          Some(sk) => {
            self.write_bool(true);
            self.write_hir_id(sk);
          }
          None => self.write_bool(false),
        }
        self.write_rng(args);
        self.write_hir_id(ret);
      }
      TypeKind::ItemStruct { fields, path } => {
        self.write_u8(31);
        self.write_rng(fields);
        self.write_hir_id(path);
      }
      TypeKind::ItemTuple { fields, path } => {
        self.write_u8(32);
        self.write_rng(fields);
        self.write_hir_id(path);
      }
    }
  }

  pub fn write_expr(&mut self, expr: &Expr) {
    let cat_byte = match expr.category {
      ExprCategory::RValue => 0,
      ExprCategory::LValueMut => 1,
      ExprCategory::LValueImm => 2,
    };
    self.write_u8(cat_byte);
    self.write_span(expr.pos);
    self.write_hir_id(expr.ety);

    match expr.kind {
      ExprKind::GenericExpr { generic, idx } => {
        self.write_u8(0);
        self.write_hir_id(generic);
        self.write_u32(idx as u32);
      }
      ExprKind::GenericRaw { generic, kind } => {
        self.write_u8(1);
        self.write_hir_id(generic);
        self.write_hir_id(kind);
      }
      ExprKind::Error => self.write_u8(2),
      ExprKind::Const(c) => {
        self.write_u8(3);
        self.write_const(&c);
      }
      ExprKind::Ref(e, ism) => {
        self.write_u8(4);
        self.write_hir_id(e);
        self.write_bool(ism);
      }
      ExprKind::Deref(e) => {
        self.write_u8(5);
        self.write_hir_id(e);
      }
      ExprKind::GlobalRef(id) => {
        self.write_u8(6);
        self.write_hir_id(id);
      }
      ExprKind::LocalRef(local) => {
        self.write_u8(7);
        self.write_u32(local);
      }
      ExprKind::Let { local, init } => {
        self.write_u8(8);
        self.write_u32(local);
        self.write_hir_id(init);
      }
      ExprKind::Block { stmt, expr } => {
        self.write_u8(9);
        self.write_rng(stmt);
        match expr {
          Some(e) => { self.write_bool(true); self.write_hir_id(e); }
          None => self.write_bool(false),
        }
      }
      ExprKind::TypeOf { kind } => {
        self.write_u8(10);
        self.write_hir_id(kind);
      }
      ExprKind::Assign { lhs, rhs } => {
        self.write_u8(11);
        self.write_hir_id(lhs);
        self.write_hir_id(rhs);
      }
      ExprKind::Loop { blok, elsb } => {
        self.write_u8(12);
        self.write_hir_id(blok);
        match elsb {
          Some(e) => { self.write_bool(true); self.write_hir_id(e); }
          None => self.write_bool(false),
        }
      }
      ExprKind::Return(e) => {
        self.write_u8(13);
        self.write_hir_id(e);
      }
      ExprKind::Break(e) => {
        self.write_u8(14);
        match e {
          Some(e) => { self.write_bool(true); self.write_hir_id(e); }
          None => self.write_bool(false),
        }
      }
      ExprKind::Continue => self.write_u8(15),
      ExprKind::If { cond, then, elsb } => {
        self.write_u8(16);
        self.write_hir_id(cond);
        self.write_hir_id(then);
        match elsb {
          Some(e) => { self.write_bool(true); self.write_hir_id(e); }
          None => self.write_bool(false),
        }
      }
      ExprKind::Call { callee, args } => {
        self.write_u8(17);
        self.write_hir_id(callee);
        self.write_rng(args);
      }
      ExprKind::BoundSelfMethod { callee, this } => {
        self.write_u8(18);
        self.write_hir_id(callee);
        self.write_hir_id(this);
      }
      ExprKind::Field { target, idx } => {
        self.write_u8(19);
        self.write_hir_id(target);
        self.write_u32(idx);
      }
      ExprKind::CombinatedInit { kind, fields } => {
        self.write_u8(20);
        self.write_hir_id(kind);
        self.write_rng(fields);
      }
      ExprKind::IntArithmetic { op, flg, lhs, rhs } => {
        self.write_u8(21);
        self.write_u8(encode_int_arith_op(op));
        self.write_u8(encode_int_arith_flg(flg));
        self.write_hir_id(lhs);
        self.write_hir_id(rhs);
      }
      ExprKind::AssignIntArithmetic { op, flg, lhs, rhs } => {
        self.write_u8(22);
        self.write_u8(encode_int_arith_op(op));
        self.write_u8(encode_int_arith_flg(flg));
        self.write_hir_id(lhs);
        self.write_hir_id(rhs);
      }
      ExprKind::IntCondition { op, lhs, rhs } => {
        self.write_u8(23);
        let op_b = match op {
          IntConditionOp::Eq => 0,
          IntConditionOp::Ne => 1,
          IntConditionOp::Gt => 2,
          IntConditionOp::Lt => 3,
          IntConditionOp::GtEq => 4,
          IntConditionOp::LtEq => 5,
        };
        self.write_u8(op_b);
        self.write_hir_id(lhs);
        self.write_hir_id(rhs);
      }
      ExprKind::IntNeg(e) => {
        self.write_u8(24);
        self.write_hir_id(e);
      }
      ExprKind::FloatArithmetic { op, lhs, rhs } => {
        self.write_u8(25);
        let op_b = match op {
          FloatArithmeticOp::Add => 0,
          FloatArithmeticOp::Sub => 1,
          FloatArithmeticOp::Mul => 2,
          FloatArithmeticOp::Div => 3,
          FloatArithmeticOp::Rem => 4,
        };
        self.write_u8(op_b);
        self.write_hir_id(lhs);
        self.write_hir_id(rhs);
      }
      ExprKind::FloatNeg(e) => {
        self.write_u8(26);
        self.write_hir_id(e);
      }
      ExprKind::BoolLogic { op, lhs, rhs } => {
        self.write_u8(27);
        let op_b = match op {
          BoolLogicOp::And => 0,
          BoolLogicOp::Or => 1,
          BoolLogicOp::Xor => 2,
        };
        self.write_u8(op_b);
        self.write_hir_id(lhs);
        self.write_hir_id(rhs);
      }
      ExprKind::BoolNot(e) => {
        self.write_u8(28);
        self.write_hir_id(e);
      }
      ExprKind::IntCast(e) => {
        self.write_u8(29);
        self.write_hir_id(e);
      }
      ExprKind::CastToIfaceRef { ref_of_expr, ref_of_type, target_iface } => {
        self.write_u8(30);
        self.write_hir_id(ref_of_expr);
        self.write_hir_id(ref_of_type);
        self.write_hir_id(target_iface);
      }
      ExprKind::CastToTrait { expr, target_trait } => {
        self.write_u8(31);
        self.write_hir_id(expr);
        self.write_hir_id(target_trait);
      }
    }
  }

  pub fn write_export_map(&mut self, exp: &ExportMap) {
    self.write_cid(exp.cid());
    match exp.root() {
      Some(r) => { self.write_bool(true); self.write_hir_id(r); }
      None => self.write_bool(false),
    }

    // symbols
    self.write_u32(exp.symbols.len() as u32);
    for (&id, sym) in &exp.symbols {
      self.write_hir_id(id);
      self.write_str(sym);
    }

    // exports
    self.write_u32(exp.len() as u32);
    for (any_id, export) in exp.iter() {
      self.write_any_id(*any_id);
      match export.parent {
        Some(p) => { self.write_bool(true); self.write_any_id(p); }
        None => self.write_bool(false),
      }
      self.write_u32(export.len() as u32);
      for (name, kind) in export.iter() {
        self.write_str(name);
        match *kind {
          ExportKind::Type(ty) => {
            self.write_u8(0);
            self.write_hir_id(ty);
          }
          ExportKind::Expr(id, ty) => {
            self.write_u8(1);
            self.write_hir_id(id);
            self.write_hir_id(ty);
          }
          ExportKind::NameSpace(id) => {
            self.write_u8(2);
            self.write_hir_id(id);
          }
        }
      }
    }
  }
}

fn encode_int_arith_op(op: IntArithmeticOp) -> u8 {
  match op {
    IntArithmeticOp::Add => 0,
    IntArithmeticOp::Sub => 1,
    IntArithmeticOp::Mul => 2,
    IntArithmeticOp::Div => 3,
    IntArithmeticOp::Rem => 4,
  }
}

fn decode_int_arith_op(b: u8) -> IntArithmeticOp {
  match b {
    0 => IntArithmeticOp::Add,
    1 => IntArithmeticOp::Sub,
    2 => IntArithmeticOp::Mul,
    3 => IntArithmeticOp::Div,
    _ => IntArithmeticOp::Rem,
  }
}

fn encode_int_arith_flg(flg: IntArithmeticFlg) -> u8 {
  match flg {
    IntArithmeticFlg::Overflow => 0,
    IntArithmeticFlg::Checked => 1,
    IntArithmeticFlg::Saturating => 2,
  }
}

fn decode_int_arith_flg(b: u8) -> IntArithmeticFlg {
  match b {
    0 => IntArithmeticFlg::Overflow,
    1 => IntArithmeticFlg::Checked,
    _ => IntArithmeticFlg::Saturating,
  }
}


pub struct Decoder<'a> {
  pub slice: &'a [u8],
  pub cursor: usize,
  pub local_sids: &'a [Sid],
}

impl<'a> Decoder<'a> {
  pub fn new(slice: &'a [u8], local_sids: &'a [Sid]) -> Self {
    Self {
      slice,
      cursor: 0,
      local_sids,
    }
  }

  #[inline]
  pub fn read_u8(&mut self) -> u8 {
    let val = self.slice[self.cursor];
    self.cursor += 1;
    val
  }

  #[inline]
  pub fn read_u16(&mut self) -> u16 {
    let val = LittleEndian::read_u16(&self.slice[self.cursor..self.cursor + 2]);
    self.cursor += 2;
    val
  }

  #[inline]
  pub fn read_u32(&mut self) -> u32 {
    let val = LittleEndian::read_u32(&self.slice[self.cursor..self.cursor + 4]);
    self.cursor += 4;
    val
  }

  #[inline]
  pub fn read_u64(&mut self) -> u64 {
    let val = LittleEndian::read_u64(&self.slice[self.cursor..self.cursor + 8]);
    self.cursor += 8;
    val
  }

  #[inline]
  pub fn read_u128(&mut self) -> u128 {
    let val = LittleEndian::read_u128(&self.slice[self.cursor..self.cursor + 16]);
    self.cursor += 16;
    val
  }

  #[inline]
  pub fn read_bool(&mut self) -> bool {
    self.read_u8() != 0
  }

  pub fn read_str(&mut self) -> String {
    let len = self.read_u32() as usize;
    let s = std::str::from_utf8(&self.slice[self.cursor..self.cursor + len]).unwrap();
    self.cursor += len;
    s.to_string()
  }

  #[inline]
  pub fn read_sid(&mut self) -> Sid {
    let idx = self.read_u32() as usize;
    self.local_sids[idx]
  }

  #[inline]
  pub fn read_cid(&mut self) -> CID {
    CID::from_raw(self.read_u16())
  }

  #[inline]
  pub fn read_hir_id<T: HirKind>(&mut self) -> HirId<T> {
    let cid = self.read_cid();
    let idx = self.read_u32();
    HirId::new(cid, idx)
  }

  #[inline]
  pub fn read_rng<T: HirKind>(&mut self) -> Rng<T> {
    let cid = self.read_cid();
    let start = self.read_u32();
    let end = self.read_u32();
    Rng::new(cid, start, end)
  }

  #[inline]
  pub fn read_any_id(&mut self) -> AnyId {
    let cid = self.read_cid();
    let idx = self.read_u32();
    let kind = match self.read_u8() {
      0 => NodeKind::Any,
      1 => NodeKind::Type,
      2 => NodeKind::Expr,
      3 => NodeKind::Item,
      4 => NodeKind::Thing,
      _ => NodeKind::DefPath,
    };
    AnyId::new(cid, idx, kind)
  }

  #[inline]
  pub fn read_any_rng(&mut self) -> AnyRng {
    let cid = self.read_cid();
    let start = self.read_u32();
    let end = self.read_u32();
    AnyRng::new(cid, start, end)
  }

  pub fn read_span(&mut self) -> Span {
    if self.read_bool() {
      Span::BUILTIN
    } else {
      let off = self.read_u32();
      let len = self.read_u16();
      let fid = self.read_u16();
      Span::new(off, NonZero::new(len).unwrap(), fid)
    }
  }

  pub fn read_layout(&mut self) -> Layout {
    let kind = self.read_u8();
    let inhabited = self.read_bool();
    Layout::from_raw(kind, inhabited)
  }

  pub fn read_defpath(&mut self) -> DefPath {
    match self.read_u8() {
      0 => DefPath::Root(self.read_sid()),
      1 => {
        let base = self.read_hir_id();
        let name = self.read_sid();
        DefPath::Path { base, name }
      }
      2 => {
        let base = self.read_hir_id();
        let spec = self.read_hir_id();
        DefPath::Impl { base, spec }
      }
      3 => {
        let base = self.read_hir_id();
        let spec = self.read_hir_id();
        DefPath::Spec { base, spec }
      }
      _ => {
        let rng = self.read_rng();
        DefPath::AnonymousTuple(rng)
      }
    }
  }

  pub fn read_const(&mut self) -> Const {
    match self.read_u8() {
      0 => Const::Unit,
      1 => Const::Bool(self.read_bool()),
      2 => Const::Str(self.read_sid()),
      3 => Const::Char(char::from_u32(self.read_u32()).unwrap()),
      4 => Const::U8(self.read_u8()),
      5 => Const::U16(self.read_u16()),
      6 => Const::U32(self.read_u32()),
      7 => Const::U64(self.read_u64()),
      8 => Const::U128(self.read_u128()),
      9 => Const::I8(self.read_u8() as i8),
      10 => Const::I16(self.read_u16() as i16),
      11 => Const::I32(self.read_u32() as i32),
      12 => Const::I64(self.read_u64() as i64),
      13 => Const::I128(self.read_u128() as i128),
      14 => Const::F16(ieee::Half::from_bits(self.read_u16() as u128)),
      15 => Const::F32(ieee::Single::from_bits(self.read_u32() as u128)),
      16 => Const::F64(ieee::Double::from_bits(self.read_u64() as u128)),
      17 => Const::F128(ieee::Quad::from_bits(self.read_u128())),
      _ => {
        let ty = self.read_hir_id();
        let rng = self.read_rng();
        Const::Array(ty, rng)
      }
    }
  }

  pub fn read_thing(&mut self) -> Thing {
    match self.read_u8() {
      0 => Thing::Name(self.read_sid()),
      1 => {
        let sid = self.read_sid();
        let ty = self.read_hir_id();
        Thing::NamedType(sid, ty)
      }
      2 => {
        let sid = self.read_sid();
        let expr = self.read_hir_id();
        Thing::NamedExpr(sid, expr)
      }
      _ => {
        let sid = self.read_sid();
        let c = self.read_const();
        Thing::NamedConst(sid, c)
      }
    }
  }

  pub fn read_item(&mut self) -> Item {
    let vis = match self.read_u8() {
      1 => ItemVis::Public(SymVis::Internal),
      2 => ItemVis::Public(SymVis::Export),
      3 => ItemVis::Public(SymVis::Import),
      _ => ItemVis::Private,
    };

    let name = if self.read_bool() {
      Some(self.read_sid())
    } else {
      None
    };

    let attr = ItemAttr::from_bits_truncate(self.read_u16());
    let path = self.read_hir_id();

    let kind = match self.read_u8() {
      0 => ItemKind::RootNS { rng: self.read_rng() },
      1 => ItemKind::NameSpace { rng: self.read_rng() },
      2 => {
        let rng = self.read_rng();
        let args = self.read_rng();
        ItemKind::GenericNS { rng, args }
      }
      3 => ItemKind::Using { kind: self.read_hir_id() },
      4 => {
        let kind = self.read_hir_id();
        let expr = self.read_hir_id();
        let ism = self.read_bool();
        ItemKind::Variable { kind, expr, ism }
      }
      5 => {
        let kind = self.read_hir_id();
        let expr = self.read_hir_id();
        ItemKind::Function { kind, expr }
      }
      6 => {
        let kind = self.read_hir_id();
        let expr = self.read_hir_id();
        ItemKind::Task { kind, expr }
      }
      _ => {
        let type_ty = self.read_hir_id();
        let trait_ty = if self.read_bool() {
          Some(self.read_hir_id())
        } else {
          None
        };
        let methods = self.read_rng();
        ItemKind::Impl { type_ty, trait_ty, methods }
      }
    };

    Item { vis, name, kind, attr, path }
  }

  pub fn read_type(&mut self) -> Type {
    let layout = self.read_layout();
    let attr = TypeAttr::from_bits_truncate(self.read_u16());

    let kind = match self.read_u8() {
      0 => {
        let generic = self.read_hir_id();
        let idx = self.read_u32() as usize;
        TypeKind::GenericType { generic, idx }
      }
      1 => {
        let generic = self.read_hir_id();
        let kind = self.read_hir_id();
        TypeKind::GenericRaw { generic, kind }
      }
      2 => TypeKind::GenericSelfType,
      3 => {
        let trait_ty = self.read_hir_id();
        let idx = self.read_u32() as usize;
        TypeKind::AssociatedType { trait_ty, idx }
      }
      4 => TypeKind::Error,
      5 => TypeKind::Unit,
      6 => TypeKind::Never,
      7 => {
        let w = self.read_u16();
        let s = self.read_bool();
        TypeKind::Int(w, s)
      }
      8 => TypeKind::ArchInt(self.read_bool()),
      9 => TypeKind::Float(self.read_u16()),
      10 => TypeKind::Bit(self.read_u16()),
      11 => TypeKind::Bool,
      12 => TypeKind::Char,
      13 => TypeKind::Str,
      14 => TypeKind::Meta(self.read_hir_id()),
      15 => {
        let k = self.read_hir_id();
        let ism = self.read_bool();
        TypeKind::Ref(k, ism)
      }
      16 => {
        let k = self.read_hir_id();
        let ism = self.read_bool();
        TypeKind::Ptr(k, ism)
      }
      17 => {
        let k = self.read_hir_id();
        let len = self.read_hir_id();
        TypeKind::Vector(k, len)
      }
      18 => TypeKind::VScale(self.read_hir_id()),
      19 => {
        let k = self.read_hir_id();
        let len = self.read_hir_id();
        TypeKind::Array(k, len)
      }
      20 => TypeKind::Slice(self.read_hir_id()),
      21 => TypeKind::Struct(self.read_rng()),
      22 => TypeKind::Tuple(self.read_rng()),
      23 => TypeKind::Range(self.read_hir_id()),
      24 => {
        let methods = self.read_rng();
        let types = self.read_rng();
        let reqs = self.read_rng();
        let path = self.read_hir_id();
        TypeKind::ItemTrait { methods, types, reqs, path }
      }
      25 => TypeKind::ItemIface{methods: self.read_rng(), path: self.read_hir_id()},
      26 => {
        let trait_ty = self.read_hir_id();
        let hidden = self.read_hir_id();
        TypeKind::TraitFrom { trait_ty, hidden }
      }
      27 => TypeKind::ItemEnum{values: self.read_rng(), path: self.read_hir_id()},
      28 => TypeKind::ItemFlag{values: self.read_rng(), path: self.read_hir_id()},
      29 => TypeKind::Option(self.read_hir_id()),
      30 => {
        let self_kind = if self.read_bool() {
          Some(self.read_hir_id())
        } else {
          None
        };
        let args = self.read_rng();
        let ret = self.read_hir_id();
        TypeKind::Fun { self_kind, args, ret }
      }
      31 => {
        let fields = self.read_rng();
        let path = self.read_hir_id();
        TypeKind::ItemStruct { fields, path }
      }
      32 => {
        let fields = self.read_rng();
        let path = self.read_hir_id();
        TypeKind::ItemTuple { fields, path }
      }
      _ => unreachable!(),
    };

    Type { kind, layout, attr }
  }

  pub fn read_expr(&mut self) -> Expr {
    let category = match self.read_u8() {
      1 => ExprCategory::LValueMut,
      2 => ExprCategory::LValueImm,
      _ => ExprCategory::RValue,
    };
    let pos = self.read_span();
    let ety = self.read_hir_id();

    let kind = match self.read_u8() {
      0 => {
        let generic = self.read_hir_id();
        let idx = self.read_u32() as usize;
        ExprKind::GenericExpr { generic, idx }
      }
      1 => {
        let generic = self.read_hir_id();
        let kind = self.read_hir_id();
        ExprKind::GenericRaw { generic, kind }
      }
      2 => ExprKind::Error,
      3 => ExprKind::Const(self.read_const()),
      4 => {
        let e = self.read_hir_id();
        let ism = self.read_bool();
        ExprKind::Ref(e, ism)
      }
      5 => ExprKind::Deref(self.read_hir_id()),
      6 => ExprKind::GlobalRef(self.read_hir_id()),
      7 => ExprKind::LocalRef(self.read_u32()),
      8 => {
        let local = self.read_u32();
        let init = self.read_hir_id();
        ExprKind::Let { local, init }
      }
      9 => {
        let stmt = self.read_rng();
        let expr = if self.read_bool() { Some(self.read_hir_id()) } else { None };
        ExprKind::Block { stmt, expr }
      }
      10 => ExprKind::TypeOf { kind: self.read_hir_id() },
      11 => {
        let lhs = self.read_hir_id();
        let rhs = self.read_hir_id();
        ExprKind::Assign { lhs, rhs }
      }
      12 => {
        let blok = self.read_hir_id();
        let elsb = if self.read_bool() { Some(self.read_hir_id()) } else { None };
        ExprKind::Loop { blok, elsb }
      }
      13 => ExprKind::Return(self.read_hir_id()),
      14 => {
        let e = if self.read_bool() { Some(self.read_hir_id()) } else { None };
        ExprKind::Break(e)
      }
      15 => ExprKind::Continue,
      16 => {
        let cond = self.read_hir_id();
        let then = self.read_hir_id();
        let elsb = if self.read_bool() { Some(self.read_hir_id()) } else { None };
        ExprKind::If { cond, then, elsb }
      }
      17 => {
        let callee = self.read_hir_id();
        let args = self.read_rng();
        ExprKind::Call { callee, args }
      }
      18 => {
        let callee = self.read_hir_id();
        let this = self.read_hir_id();
        ExprKind::BoundSelfMethod { callee, this }
      }
      19 => {
        let target = self.read_hir_id();
        let idx = self.read_u32();
        ExprKind::Field { target, idx }
      }
      20 => {
        let kind = self.read_hir_id();
        let fields = self.read_rng();
        ExprKind::CombinatedInit { kind, fields }
      }
      21 => {
        let op = decode_int_arith_op(self.read_u8());
        let flg = decode_int_arith_flg(self.read_u8());
        let lhs = self.read_hir_id();
        let rhs = self.read_hir_id();
        ExprKind::IntArithmetic { op, flg, lhs, rhs }
      }
      22 => {
        let op = decode_int_arith_op(self.read_u8());
        let flg = decode_int_arith_flg(self.read_u8());
        let lhs = self.read_hir_id();
        let rhs = self.read_hir_id();
        ExprKind::AssignIntArithmetic { op, flg, lhs, rhs }
      }
      23 => {
        let op = match self.read_u8() {
          0 => IntConditionOp::Eq,
          1 => IntConditionOp::Ne,
          2 => IntConditionOp::Gt,
          3 => IntConditionOp::Lt,
          4 => IntConditionOp::GtEq,
          _ => IntConditionOp::LtEq,
        };
        let lhs = self.read_hir_id();
        let rhs = self.read_hir_id();
        ExprKind::IntCondition { op, lhs, rhs }
      }
      24 => ExprKind::IntNeg(self.read_hir_id()),
      25 => {
        let op = match self.read_u8() {
          0 => FloatArithmeticOp::Add,
          1 => FloatArithmeticOp::Sub,
          2 => FloatArithmeticOp::Mul,
          3 => FloatArithmeticOp::Div,
          _ => FloatArithmeticOp::Rem,
        };
        let lhs = self.read_hir_id();
        let rhs = self.read_hir_id();
        ExprKind::FloatArithmetic { op, lhs, rhs }
      }
      26 => ExprKind::FloatNeg(self.read_hir_id()),
      27 => {
        let op = match self.read_u8() {
          0 => BoolLogicOp::And,
          1 => BoolLogicOp::Or,
          _ => BoolLogicOp::Xor,
        };
        let lhs = self.read_hir_id();
        let rhs = self.read_hir_id();
        ExprKind::BoolLogic { op, lhs, rhs }
      }
      28 => ExprKind::BoolNot(self.read_hir_id()),
      29 => ExprKind::IntCast(self.read_hir_id()),
      30 => {
        let ref_of_expr = self.read_hir_id();
        let ref_of_type = self.read_hir_id();
        let target_iface = self.read_hir_id();
        ExprKind::CastToIfaceRef { ref_of_expr, ref_of_type, target_iface }
      }
      _ => {
        let expr = self.read_hir_id();
        let target_trait = self.read_hir_id();
        ExprKind::CastToTrait { expr, target_trait }
      }
    };

    Expr { kind, category, pos, ety }
  }

  pub fn read_export_map(&mut self) -> ExportMap {
    let cid = self.read_cid();
    let root = if self.read_bool() {
      Some(self.read_hir_id())
    } else {
      None
    };

    let sym_count = self.read_u32() as usize;
    let mut symbols = FxHashMap::default();
    for _ in 0..sym_count {
      let id = self.read_hir_id();
      let sym = self.read_str();
      symbols.insert(id, sym);
    }

    let exp_count = self.read_u32() as usize;
    let mut map = FxHashMap::default();
    for _ in 0..exp_count {
      let any_id = self.read_any_id();
      let parent = if self.read_bool() {
        Some(self.read_any_id())
      } else {
        None
      };
      let entries_count = self.read_u32() as usize;
      let mut export_entries = FxHashMap::default();
      for _ in 0..entries_count {
        let name = self.read_str();
        let kind = match self.read_u8() {
          0 => ExportKind::Type(self.read_hir_id()),
          1 => {
            let id = self.read_hir_id();
            let ty = self.read_hir_id();
            ExportKind::Expr(id, ty)
          }
          _ => ExportKind::NameSpace(self.read_hir_id()),
        };
        export_entries.insert(name, kind);
      }
      map.insert(any_id, Export::from_raw(parent, export_entries));
    }

    ExportMap::from_raw(cid, root, map, symbols)
  }
}
