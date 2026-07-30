use std::collections::HashMap;

use crate::{ast::types::TypeVari, control::{IdentyId, module::Module} };

use super::target::Target;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TypeLayout {
	pub size: u64,
	pub align: u64,
	pub needs_drop: bool,
}

impl TypeLayout {
	
	pub fn new(size: u64, align: u64, needs_drop: bool) -> Self {
		Self{ size, align, needs_drop }
	}

}

pub struct LayoutEngine<'a, 'd> {
	pub target: Target,
	pub module: &'a Module<'a, 'd>,
	cache: HashMap<IdentyId, TypeLayout>,
}

impl<'a, 'd> LayoutEngine<'a, 'd> {

	pub fn new(target: Target, module: &'a Module<'a, 'd>) -> Self {
		Self{
			target,
			module,
			cache: HashMap::new(),
		}
	}

	fn align_to(offset: u64, align: u64) -> u64 {
		if align == 0 {
			return offset;
		}

		let remainder = offset % align;
		if remainder == 0 {
			offset
		} else {
			offset + (align - remainder)
		}
	}

	pub fn get_layout(&mut self, ty_id: IdentyId) -> Result<TypeLayout, String> {
		if let Some(layout) = self.cache.get(&ty_id) {
			return Ok(*layout);
		}

		let ty = self.module.get_type(ty_id);
		
		let layout = match &ty.vari {
			TypeVari::Void | TypeVari::Null => TypeLayout::new(0, 1, false),
			TypeVari::Bool | TypeVari::I8 | TypeVari::U8 | TypeVari::Char => TypeLayout::new(1, 1, false),
			TypeVari::I16 | TypeVari::U16 | TypeVari::F16 => TypeLayout::new(2, 2, false),
			TypeVari::I32 | TypeVari::U32 | TypeVari::F32 => TypeLayout::new(4, 4, false),
			TypeVari::I64 | TypeVari::U64 | TypeVari::F64 => TypeLayout::new(8, 8, false),
			TypeVari::I128 | TypeVari::U128 | TypeVari::F128 => TypeLayout::new(16, 16, false),
			
			TypeVari::ISize | TypeVari::USize | TypeVari::Ptr | 
			TypeVari::PointerOf{..} | TypeVari::ReferenceOf { .. } |
			TypeVari::Function(_) => {
				TypeLayout::new(self.target.pointer_size, self.target.pointer_size, false)
			}
			TypeVari::ZArrayOf(sub) => {
				let sub_layout = self.get_layout(*sub)?;
				TypeLayout::new(self.target.pointer_size * 2, self.target.pointer_size, sub_layout.needs_drop)
			}
			TypeVari::PArrayOf(parray) => {
				let sub_layout = self.get_layout(parray.sub)?;
				let size = sub_layout.size * parray.size;
				TypeLayout::new(size, sub_layout.align, sub_layout.needs_drop)
			}
			TypeVari::Struct(stc) => {
				let mut field_layouts = Vec::new();
				for field in &stc.vars {
					let layout = self.get_layout(field.kind)?;
					field_layouts.push(layout);
				}

				field_layouts.sort_by(|a, b| b.align.cmp(&a.align));

				let mut current_offset = 0;
				let mut max_align = 1;
				let mut needs_drop = false;

				for layout in field_layouts {
					current_offset = Self::align_to(current_offset, layout.align);
					current_offset += layout.size;
					
					if layout.align > max_align {
						max_align = layout.align;
					}
					if layout.needs_drop {
						needs_drop = true;
					}
				}

				let total_size = Self::align_to(current_offset, max_align);

				TypeLayout::new(total_size, max_align, needs_drop)
			}
			TypeVari::Enum(..) => {
				TypeLayout::new(4, 4, false)
			}
			TypeVari::Iface(_) => {
				TypeLayout::new(0, 1, false)
			}
			TypeVari::Nick(_) | TypeVari::UnresolvedPath(_) => panic!("Unresolved types cannot be laid out!"),
			TypeVari::Path { .. } => {
				return Err("Unresolved Path type".to_string());
			}
		};

		self.cache.insert(ty_id, layout);
		Ok(layout)
	}

}
