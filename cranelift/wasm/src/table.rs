use cranelift_codegen::cursor::FuncCursor;
use cranelift_codegen::ir::{self, condcodes::IntCC, immediates::Imm64, InstBuilder};
use cranelift_frontend::FunctionBuilder;
#[cfg(feature = "wa2x-test")]
use crate::state::FuncTranslationState;

/// Size of a WebAssembly table, in elements.
#[derive(Clone)]
pub enum TableSize {
    /// Non-resizable table.
    Static {
        /// Non-resizable tables have a constant size known at compile time.
        bound: u32,
    },
    /// Resizable table.
    Dynamic {
        /// Resizable tables declare a Cranelift global value to load the
        /// current size from.
        bound_gv: ir::GlobalValue,
    },
}

impl TableSize {
    /// Get a CLIF value representing the current bounds of this table.
    pub fn bound(&self, mut pos: FuncCursor, index_ty: ir::Type) -> ir::Value {
        match *self {
            TableSize::Static { bound } => pos.ins().iconst(index_ty, Imm64::new(i64::from(bound))),
            TableSize::Dynamic { bound_gv } => pos.ins().global_value(index_ty, bound_gv),
        }
    }

	#[cfg(feature = "wa2x-test")]
	/// Get table size with wa2x debug info
	pub fn bound_with_debug(&self, mut pos: FuncCursor, state: &mut Option<&mut FuncTranslationState>, index_ty: ir::Type) -> ir::Value {
		match *self {
            TableSize::Static { bound } => pos.ins().iconst(index_ty, Imm64::new(i64::from(bound))),
            TableSize::Dynamic { bound_gv } => build_cursor_global_value(&mut pos, state, index_ty, bound_gv)
        }
	}
}

/// An implementation of a WebAssembly table.
#[derive(Clone)]
pub struct TableData {
    /// Global value giving the address of the start of the table.
    pub base_gv: ir::GlobalValue,

    /// The size of the table, in elements.
    pub bound: TableSize,

    /// The size of a table element, in bytes.
    pub element_size: u32,
}

impl TableData {
    /// Return a CLIF value containing a native pointer to the beginning of the
    /// given index within this table.
    pub fn prepare_table_addr(
        &self,
        pos: &mut FunctionBuilder,
        mut index: ir::Value,
        addr_ty: ir::Type,
        enable_table_access_spectre_mitigation: bool,
    ) -> (ir::Value, ir::MemFlags) {
        let index_ty = pos.func.dfg.value_type(index);

        // Start with the bounds check. Trap if `index + 1 > bound`.
        let bound = self.bound.bound(pos.cursor(), index_ty);

        // `index > bound - 1` is the same as `index >= bound`.
        let oob = pos
            .ins()
            .icmp(IntCC::UnsignedGreaterThanOrEqual, index, bound);

        if !enable_table_access_spectre_mitigation {
            pos.ins().trapnz(oob, ir::TrapCode::TableOutOfBounds);
        }

        // Convert `index` to `addr_ty`.
        if index_ty != addr_ty {
            index = pos.ins().uextend(addr_ty, index);
        }

        // Add the table base address base
        let base = pos.ins().global_value(addr_ty, self.base_gv);

        let element_size = self.element_size;
        let offset = if element_size == 1 {
            index
        } else if element_size.is_power_of_two() {
            pos.ins()
                .ishl_imm(index, i64::from(element_size.trailing_zeros()))
        } else {
            pos.ins().imul_imm(index, element_size as i64)
        };

        let element_addr = pos.ins().iadd(base, offset);

        let base_flags = ir::MemFlags::new()
            .with_aligned()
            .with_alias_region(Some(ir::AliasRegion::Table));
        if enable_table_access_spectre_mitigation {
            // Short-circuit the computed table element address to a null pointer
            // when out-of-bounds. The consumer of this address will trap when
            // trying to access it.
            let zero = pos.ins().iconst(addr_ty, 0);
            (
                pos.ins().select_spectre_guard(oob, zero, element_addr),
                base_flags.with_trap_code(Some(ir::TrapCode::TableOutOfBounds)),
            )
        } else {
            (element_addr, base_flags.with_trap_code(None))
        }
    }

	#[cfg(feature = "wa2x-test")]
	/// Return a CLIF value containing a native pointer to the beginning of the
    /// given index within this table.
    pub fn prepare_table_addr_with_debug(
        &self,
        pos: &mut FunctionBuilder,
		state: &mut Option<&mut FuncTranslationState>,
        mut index: ir::Value,
        addr_ty: ir::Type,
        enable_table_access_spectre_mitigation: bool,
    ) -> (ir::Value, ir::MemFlags) {
        let index_ty = pos.func.dfg.value_type(index);

        // Start with the bounds check. Trap if `index + 1 > bound`.
        let bound = self.bound.bound_with_debug(pos.cursor(), state, index_ty);

        // `index > bound - 1` is the same as `index >= bound`.
        let oob = pos
            .ins()
            .icmp(IntCC::UnsignedGreaterThanOrEqual, index, bound);

        if !enable_table_access_spectre_mitigation {
			build_trapnz(pos, state, oob, ir::TrapCode::TableOutOfBounds);
        }

        // Convert `index` to `addr_ty`.
        if index_ty != addr_ty {
            index = pos.ins().uextend(addr_ty, index);
        }

        // Add the table base address base
		let base = build_global_value(pos, state, addr_ty, self.base_gv);

        let element_size = self.element_size;
        let offset = if element_size == 1 {
            index
        } else if element_size.is_power_of_two() {
            pos.ins()
                .ishl_imm(index, i64::from(element_size.trailing_zeros()))
        } else {
            pos.ins().imul_imm(index, element_size as i64)
        };

        let element_addr = pos.ins().iadd(base, offset);

        let base_flags = ir::MemFlags::new()
            .with_aligned()
            .with_alias_region(Some(ir::AliasRegion::Table));
        if enable_table_access_spectre_mitigation {
            // Short-circuit the computed table element address to a null pointer
            // when out-of-bounds. The consumer of this address will trap when
            // trying to access it.
            let zero = pos.ins().iconst(addr_ty, 0);
            (
                pos.ins().select_spectre_guard(oob, zero, element_addr),
                base_flags.with_trap_code(Some(ir::TrapCode::TableOutOfBounds)),
            )
        } else {
            (element_addr, base_flags.with_trap_code(None))
        }
    }

}

#[cfg(feature = "wa2x-test")]
fn build_trapnz<T1: Into<ir::TrapCode>>(
	builder: &mut FunctionBuilder,
	state: &mut Option<&mut FuncTranslationState>,
	c: ir::Value,
	code: T1
) -> ir::Inst {
	if let Some(state) = state {
		let source_location = std::panic::Location::caller();
		state.add_debug_info(builder, "CondBranch", source_location);
	}
	builder.ins().trapnz(c, code)
}

#[cfg(feature = "wa2x-test")]
fn build_cursor_global_value(
	pos: &mut FuncCursor,
	state: &mut Option<&mut FuncTranslationState>,
	mem: ir::Type,
	gv: ir::GlobalValue
) -> ir::Value {
	if let Some(state) = state {
		if pos.is_load_global_value(gv) {
			let source_location = std::panic::Location::caller();
			state.add_debug_info_cursor(pos, "Load", source_location);
		}
	}
	pos.ins().global_value(mem, gv)
}

#[cfg(feature = "wa2x-test")]
fn build_global_value(
	builder: &mut FunctionBuilder,
	state: &mut Option<&mut FuncTranslationState>,
	mem: ir::Type,
	gv: ir::GlobalValue
) -> ir::Value {
	if let Some(state) = state {
		if builder.is_load_global_value(gv) {
			let source_location = std::panic::Location::caller();
			state.add_debug_info(builder, "Load", source_location);
		}
	}
	builder.ins().global_value(mem, gv)
}
