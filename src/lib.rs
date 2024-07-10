pub mod instruction;
pub mod processor;

use solana_program::entrypoint;
use processor::process_instruction;

#[cfg(not(feature = "no-entrypoint"))]
entrypoint!(process_instruction);

solana_program::declare_id!("6E9iP7p4Gx2e6c2Yt4MHY5T1aZ8RWhrmF9p6bXkGWiza");