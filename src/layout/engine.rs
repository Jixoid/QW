use std::collections::HashMap;

use crate::{ast::types::TypeVari, control::{AstId, module::Module} };

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
	cache: HashMap<AstId, TypeLayout>,
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

	pub fn get_layout(&mut self, ty_id: AstId) -> Result<TypeLayout, String> {
		if let Some(layout) = self.cache.get(&ty_id) {
			return Ok(*layout);
		}

		let ty = self.module.get_type(ty_id);
		
		let layout = match &ty.vari {
			TypeVari::Void | TypeVari::Null | TypeVari::SelfType => TypeLayout::new(0, 1, false),
			TypeVari::Bool | TypeVari::Char => TypeLayout::new(1, 1, false),

			TypeVari::Int{bit, ..} => TypeLayout::new(*bit as u64, *bit as u64, false),
			TypeVari::Float{bit} => TypeLayout::new(*bit as u64, *bit as u64, false),


			TypeVari::ArchSize{..} | TypeVari::Ptr | 
			TypeVari::PointerOf{..} | TypeVari::ReferenceOf { .. } |
			
			TypeVari::Function(_) => {
				TypeLayout::new(self.target.pointer_size, self.target.pointer_size, false)
			}
			
			TypeVari::ArrayOf{sub, ext} => {
				let sub_layout = self.get_layout(*sub)?;
				if ext.is_empty() {
					// Slice / dynamic array fat pointer (pointer + len)
					TypeLayout::new(self.target.pointer_size * 2, self.target.pointer_size, sub_layout.needs_drop)
				} else {
					// Fixed-size array: count elements and calculate contiguous total size
					let count: u64 = ext.iter().fold(1u64, |acc, &len| acc.saturating_mul(len as u64));
					let total_size = sub_layout.size * count;
					TypeLayout::new(total_size, sub_layout.align, sub_layout.needs_drop)
				}
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
			TypeVari::Flags(..) => {
				TypeLayout::new(4, 4, false)
			}
			TypeVari::Iface(..) => {
				TypeLayout::new(16, self.target.pointer_size, false)
			}
			TypeVari::Trait(_) => {
				TypeLayout::new(0, 1, false)
			}
			TypeVari::GenericInstance{..} => unreachable!("GenericInstance should be resolved in Sema"),
			TypeVari::Nick(..) | TypeVari::UnresolvedPath(..) => return Err(format!("Unresolved type: {:?}", ty.vari)),
			TypeVari::Path { .. } => {
				return Err("Unresolved Path type".to_string());
			}
		};

		self.cache.insert(ty_id, layout);
		Ok(layout)
	}

}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::ast::types::{Type, TypeVari, TypeState};
	use crate::control::module::Module;

	#[test]
	fn test_array_layout() {
		let mut module = Module::new_rtl("test".to_string()).unwrap();
		let target = Target::new_64bit();

		// Create i32 type (bit represents bit width e.g., 32 bits = 32 size in TypeVari::Int bit layout)
		let i32_ty = Type {
			vari: TypeVari::Int { bit: 32, sig: true },
			state: TypeState::Resolved,
		};
		let i32_id = module.new_type(i32_ty);

		// Fixed size array [i32, 5] -> size: 32 * 5 = 160, align: 32
		let fixed_arr_ty = Type {
			vari: TypeVari::ArrayOf { sub: i32_id, ext: vec![5] },
			state: TypeState::Resolved,
		};
		let fixed_arr_id = module.new_type(fixed_arr_ty);

		// Fat pointer slice [i32] -> size: 16 bytes (ptr + len = 8 + 8), align: 8
		let fat_arr_ty = Type {
			vari: TypeVari::ArrayOf { sub: i32_id, ext: vec![] },
			state: TypeState::Resolved,
		};
		let fat_arr_id = module.new_type(fat_arr_ty);

		let mut engine = LayoutEngine::new(target, &module);
		
		let fixed_layout = engine.get_layout(fixed_arr_id).unwrap();
		assert_eq!(fixed_layout.size, 160);
		assert_eq!(fixed_layout.align, 32);

		let fat_layout = engine.get_layout(fat_arr_id).unwrap();
		assert_eq!(fat_layout.size, 16); // 2 * pointer_size (8) = 16
		assert_eq!(fat_layout.align, 8);
	}
}
