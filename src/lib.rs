pub mod error;
pub mod instruction;
pub mod instruction_builders;
pub mod processor;
pub mod utils;

use processor::process_instruction;
use solana_program::entrypoint;

#[cfg(not(feature = "no-entrypoint"))]
entrypoint!(process_instruction);

solana_program::declare_id!("B7aQx94uA3crMRRqeQgfcBQeGiCPZunDeS9fH7ywjrci");
