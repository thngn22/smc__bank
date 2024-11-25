import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Bank } from "../target/types/bank";
import * as web3 from "@solana/web3.js";
import privateKey from "../key.json";

describe("Withdraw", () => {
  const provider = anchor.AnchorProvider.local("http://127.0.0.1:8899");
  anchor.setProvider(provider);

  const program = anchor.workspace.Bank as Program<Bank>;
  const payer = web3.Keypair.fromSecretKey(Uint8Array.from(privateKey));

  it("Withraw test case", async () => {});
});
