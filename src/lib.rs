pub mod error;
pub mod instruction;
pub mod processor;
pub mod utils;

use processor::process_instruction;
use solana_program::entrypoint;

#[cfg(not(feature = "no-entrypoint"))]
entrypoint!(process_instruction);

solana_program::declare_id!("6E9iP7p4Gx2e6c2Yt4MHY5T1aZ8RWhrmF9p6bXkGWiza");
