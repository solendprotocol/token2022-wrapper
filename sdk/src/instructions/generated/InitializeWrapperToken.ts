/**
 * This code was GENERATED using the solita package.
 * Please DO NOT EDIT THIS FILE, instead rerun solita to update it or write a wrapper to add functionality.
 *
 * See: https://github.com/metaplex-foundation/solita
 */

import * as beet from '@metaplex-foundation/beet'
import * as web3 from '@solana/web3.js'

/**
 * @category Instructions
 * @category InitializeWrapperToken
 * @category generated
 */
export const InitializeWrapperTokenStruct = new beet.BeetArgsStruct<{
  instructionDiscriminator: number
}>(
  [['instructionDiscriminator', beet.u8]],
  'InitializeWrapperTokenInstructionArgs'
)

export const initializeWrapperTokenInstructionDiscriminator = 0

/**
 * Creates a _InitializeWrapperToken_ instruction.
 *
 * @category Instructions
 * @category InitializeWrapperToken
 * @category generated
 */
export function createInitializeWrapperTokenInstruction(
  programId = new web3.PublicKey('6E9iP7p4Gx2e6c2Yt4MHY5T1aZ8RWhrmF9p6bXkGWiza')
) {
  const [data] = InitializeWrapperTokenStruct.serialize({
    instructionDiscriminator: initializeWrapperTokenInstructionDiscriminator,
  })
  const keys: web3.AccountMeta[] = []

  const ix = new web3.TransactionInstruction({
    programId,
    keys,
    data,
  })
  return ix
}
