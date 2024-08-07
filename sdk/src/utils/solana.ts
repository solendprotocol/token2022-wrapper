import * as web3 from "@solana/web3.js";

export const requestComputeUnits = (
  unitsRequested: number,
  additionalFee: number
): web3.TransactionInstruction[] => {
  const computeLimitIx = web3.ComputeBudgetProgram.setComputeUnitLimit({
    units: unitsRequested,
  });

  const computeUnitPriceIx = web3.ComputeBudgetProgram.setComputeUnitPrice({
    microLamports: additionalFee,
  });

  return [computeLimitIx, computeUnitPriceIx];
};
