import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Bank } from "../target/types/bank";

import {
  TOKEN_PROGRAM_ID,
  createMint,
  createAssociatedTokenAccount,
  mintTo,
} from "@solana/spl-token";
import * as web3 from "@solana/web3.js";
import privateKey from "../key.json";
import { assert } from "chai";

describe("Deposit", () => {
  const provider = anchor.AnchorProvider.local("http://127.0.0.1:8899");
  anchor.setProvider(provider);

  const program = anchor.workspace.Bank as Program<Bank>;
  const payer = web3.Keypair.fromSecretKey(Uint8Array.from(privateKey));
  const toKp = new web3.Keypair();
  const bank = web3.Keypair.generate();
  const bankAccount = web3.Keypair.generate();

  it("Deposit with valid token", async () => {
    const mint = await createMint(
      provider.connection,
      payer,
      payer.publicKey,
      null,
      9
    );
    console.log("Mint Address: ", mint.toString());

    const userAta = await createAssociatedTokenAccount(
      provider.connection,
      payer,
      mint,
      payer.publicKey
    );

    const bankAta = await createAssociatedTokenAccount(
      provider.connection,
      payer,
      mint,
      toKp.publicKey
    );

    const mintAmount = 1000;
    await mintTo(
      provider.connection,
      payer,
      mint,
      userAta,
      payer.publicKey,
      mintAmount
    );

    await program.methods
      .initializeBank()
      .accounts({
        bank: bank.publicKey,
        authority: payer.publicKey,
      })
      .signers([bank, payer])
      .rpc();

    await program.methods
      .initializeBankAccount()
      .accounts({
        bank: bank.publicKey,
        userBankAccount: bankAccount.publicKey,
        owner: payer.publicKey,
      })
      .signers([bankAccount, payer])
      .rpc();

    // Add token to whitelist
    await program.methods
      .addToken(mint)
      .accounts({
        bank: bank.publicKey,
        authority: payer.publicKey,
      })
      .signers([payer])
      .rpc();

    const transferAmount = new anchor.BN(500);
    await program.methods
      .deposit(transferAmount)
      .accounts({
        userAta,
        bankAta,
        userBankAccount: bankAccount.publicKey,
        bank: bank.publicKey,
        userAuthority: payer.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([payer])
      .rpc();

    const bankTokenBalance = await provider.connection.getTokenAccountBalance(
      bankAta
    );
    assert.strictEqual(
      bankTokenBalance.value.uiAmount,
      transferAmount.toNumber() / 1e9,
      "The bank token account should reflect the deposited amount"
    );
  });
});
