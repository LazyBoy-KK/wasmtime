/// Debug text for wa2x test
use std::{string::String, vec::Vec};

use crate::ir::{self, RelSourceLoc, SourceLoc};

#[derive(Default)]
struct DebugFunction {
	info: Vec<String>,
	base: SourceLoc,
}

#[derive(Default)]
/// Debug context
pub struct DebugCtx {
	funcs: Vec<DebugFunction>,
	line: u32
}

impl DebugCtx {
	/// add wa2x debug info
	pub fn add_debug_info(&mut self, text: String) -> (ir::SourceLoc, ir::RelSourceLoc) {
		assert!(self.funcs.len() > 0);
		for c in text.chars() {
			if c == '\n' {
				self.line += 1;
			}
		}
		let line = self.line();
		let func = self.current_func_mut();
		func.info.push(text.clone());
		(line, RelSourceLoc::from_base_offset(func.base, line))
	}

	/// get line of debug info
	pub fn line(&self) -> ir::SourceLoc {
		ir::SourceLoc::new(self.line)
	}

	/// get relative line of debug info
	pub fn rel_line(&self) -> ir::RelSourceLoc {
		let func = self.current_func();
		RelSourceLoc::from_base_offset(func.base, self.line())
	}

	fn current_func(&self) -> &DebugFunction {
		let len = self.funcs.len();
		&self.funcs[len - 1]
	}

	fn current_func_mut(&mut self) -> &mut DebugFunction {
		let len = self.funcs.len();
		&mut self.funcs[len - 1]
	}

	/// add function debug info
	pub fn add_func_debug_info(&mut self, text: String) -> ir::SourceLoc {
		for c in text.chars() {
			if c == '\n' {
				self.line += 1;
			}
		}
		let func = DebugFunction {
			info: vec![text],
			base: Default::default(),
		};
		self.funcs.push(func);
		self.line()
	}

	/// set current func base
	pub fn set_current_base(&mut self, base: ir::SourceLoc) {
		self.current_func_mut().base = base;
	}

	/// finish recording wa2x debug info
	pub fn get_debug_info(&self) -> String {
		let mut source = String::new();
		for func in &self.funcs {
			for info in &func.info {
				source.push_str(info);
			}
		}
		source
	}
}
