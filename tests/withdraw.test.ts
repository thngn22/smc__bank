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

describe("Withdraw", () => {
  const provider = anchor.AnchorProvider.local("http://127.0.0.1:8899");
  anchor.setProvider(provider);

  const program = anchor.workspace.Bank as Program<Bank>;
  const payer = web3.Keypair.fromSecretKey(Uint8Array.from(privateKey));
  const fromKp = new web3.Keypair();
  const bank = web3.Keypair.generate();
  const bankAccount = web3.Keypair.generate();

  it("Withdraw test case", async () => {
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
    console.log("userAta :>> ", userAta);

    const bankAta = await createAssociatedTokenAccount(
      provider.connection,
      payer,
      mint,
      fromKp.publicKey
    );
    console.log("bankAta :>> ", bankAta);

    // Mint tokens to user ATA
    const mintAmount = 1000; // Mint 1000 tokens to user
    await mintTo(
      provider.connection,
      payer,
      mint,
      userAta,
      payer.publicKey,
      mintAmount
    );
    console.log('mintAmount :>> ', mintAmount);

    // Initialize bank
    await program.methods
      .initializeBank()
      .accounts({
        bank: bank.publicKey,
        authority: payer.publicKey,
      })
      .signers([bank, payer])
      .rpc();

    // Initialize user bank account
    await program.methods
      .initializeBankAccount()
      .accounts({
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

    // Deposit into the bank
    const depositAmount = new anchor.BN(500); // Deposit 500 tokens
    await program.methods
      .deposit(depositAmount)
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

    // Assert bank account balance after deposit
    let bankTokenBalance = await provider.connection.getTokenAccountBalance(
      bankAta
    );
    assert.strictEqual(
      bankTokenBalance.value.uiAmount,
      depositAmount.toNumber() / 1e9,
      "The bank token account should reflect the deposited amount"
    );

    // Withdraw from the bank
    const withdrawAmount = new anchor.BN(600); // Withdraw 300 tokens
    await program.methods
      .withdraw(withdrawAmount)
      .accounts({
        userAta,
        bankAta,
        userBankAccount: bankAccount.publicKey,
        bank: bank.publicKey,
        bankAuthority: fromKp.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([fromKp]) // The bank authority signs the withdraw
      .rpc();

    // Assert bank account balance after withdrawal
    bankTokenBalance = await provider.connection.getTokenAccountBalance(
      bankAta
    );
    assert.strictEqual(
      bankTokenBalance.value.uiAmount,
      (depositAmount.toNumber() - withdrawAmount.toNumber()) / 1e9,
      "The bank token account should reflect the withdrawn amount"
    );

    // Assert user ATA balance after withdrawal
    const userTokenBalance = await provider.connection.getTokenAccountBalance(
      userAta
    );
    assert.strictEqual(
      userTokenBalance.value.uiAmount,
      (mintAmount - depositAmount.toNumber() + withdrawAmount.toNumber()) / 1e9,
      "The user token account should reflect the withdrawn amount"
    );
  });
});
