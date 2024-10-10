use std::{fs, io::Write, path::PathBuf};

use anyhow::Result;
use clap::Parser;
use wasmtime::{Engine, Module};
use wasmtime_cli_flags::CommonOptions;

/// Display a WebAssembly module addr map.
#[derive(Parser, PartialEq)]
#[command(
    version,
)]
pub struct AddrmapCommand {
	#[command(flatten)]
	/// common options
    pub common: CommonOptions,
	/// The path of AOT file to parse
	#[arg(index = 1, value_name = "FILE")]
	pub aot_file: PathBuf,

    /// The path of the output addr map; defaults to `<MODULE>.map`
    #[arg(short = 'o', long, value_name = "OUTPUT")]
    pub output: PathBuf,
}

impl AddrmapCommand {
	/// execute
	pub fn execute(mut self) -> Result<()> {
		let config = self.common.config(None, None)?;
		let engine = Engine::new(&config)?;
		let module = unsafe { Module::from_trusted_file(&engine, &self.aot_file)? };
		let mut out = fs::OpenOptions::new()
			.write(true)
			.create(true)
			.open(&self.output)?;
		if let Some(map_iter) = module.address_map() {
			for (ins_offset, line) in map_iter {
				if let Some(line) = line {
					out.write(format!("0x{ins_offset:x} {line}\n").as_bytes())?;
				}
			}
		}
		Ok(())
	}
}
