import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Bank } from "../target/types/bank";

describe("bank", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.Bank as Program<Bank>;

  it("Is initialized!", async () => {});
});
