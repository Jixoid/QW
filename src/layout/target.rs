
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Arch {
	X86_64,
	AArch64,
	X86,
	Arm,
}

#[derive(Debug, Clone)]
pub struct Target {
	pub arch: Arch,
	pub pointer_size: u64,
}

impl Target {
	
	pub fn new_64bit() -> Self {
		Self {
			arch: Arch::X86_64,
			pointer_size: 8,
		}
	}

}
