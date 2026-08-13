use std::mem::{align_of, size_of};
use std::ptr;

const PAGE_SIZE: usize = 64 * 1024 * 1024; // 64 MB

pub struct Arena {
	data: Vec<Vec<u8>>,
	soff: usize,
}


impl Arena {
  
	pub fn new() -> Self {
		Self {
			data: vec![Vec::with_capacity(PAGE_SIZE)],
			soff: 0,
		}
	}

	
	#[inline]
	fn prepare(&mut self, required_bytes: usize) {
		let req_page_count = (required_bytes / PAGE_SIZE) + 1;

		if self.data.len() < req_page_count {
			let new_pages = req_page_count - self.data.len();
			for _ in 0..new_pages {
				self.data.push(Vec::with_capacity(PAGE_SIZE));
			}
		}
	}

	#[inline]
	fn addr_of(&self, off: usize) -> *const u8 {
		let gidx = off / PAGE_SIZE;
		let sidx = off % PAGE_SIZE;

		unsafe { self.data[gidx].as_ptr().add(sidx) }
	}


	pub fn push<T>(&mut self, obj: T) -> u32 {
		let size = size_of::<T>();
		let align = align_of::<T>();

		// Alignment
		let align_offset = (align - (self.soff % align)) % align;
		let mut aligned_soff = self.soff + align_offset;

		// Page Bound Check
		let page_offset = aligned_soff % PAGE_SIZE;
		if page_offset + size > PAGE_SIZE {
			aligned_soff = ((aligned_soff / PAGE_SIZE) + 1) * PAGE_SIZE;
		}

		let noff = aligned_soff + size;
		self.prepare(noff);

		// Unstarted Memory Write
		let target = self.addr_of(aligned_soff) as *mut T;
		unsafe {
			ptr::write(target, obj);
		}

		self.soff = noff;
		aligned_soff as u32
	}


	pub fn get<T>(&self, off: u32) -> &T {
		let target = self.addr_of(off as usize) as *const T;
		unsafe { &*target }
	}

	pub fn get_mut<T>(&mut self, off: u32) -> &mut T {
		let target = self.addr_of(off as usize) as *mut T;
		unsafe { &mut *target }
	}

}
