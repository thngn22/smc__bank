use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

declare_id!("83zrVmcBMziMhvMPBE1WWnexBz6UhMtiRrNF7F8nLS7e");

#[program]
pub mod bank_system {
    use super::*;

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        let token_mint = ctx.accounts.user_ata.mint;

        if !ctx.accounts.bank.is_token_whitelisted(token_mint) {
            return Err(ErrorCode::InvalidToken.into());
        }

        if ctx.accounts.user_ata.amount < amount {
            return Err(ErrorCode::InsufficientBalance.into());
        }

        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.user_ata.to_account_info(),
                to: ctx.accounts.bank_ata.to_account_info(),
                authority: ctx.accounts.user_authority.to_account_info(),
            },
        );
        token::transfer(cpi_ctx, amount)?;

        ctx.accounts
            .user_bank_account
            .add_balance(token_mint, amount)?;

        msg!(
            "Deposit successful. Amount: {} Token: {:?}",
            amount,
            token_mint
        );
        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        let token_mint = ctx.accounts.bank_ata.mint;

        if !ctx.accounts.bank.is_token_whitelisted(token_mint) {
            return Err(ErrorCode::InvalidToken.into());
        }

        if !ctx
            .accounts
            .user_bank_account
            .has_sufficient_balance(token_mint, amount)
        {
            return Err(ErrorCode::InsufficientUserBalance.into());
        }

        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.bank_ata.to_account_info(),
                to: ctx.accounts.user_ata.to_account_info(),
                authority: ctx.accounts.bank_authority.to_account_info(),
            },
        );
        token::transfer(cpi_ctx, amount)?;

        ctx.accounts
            .user_bank_account
            .subtract_balance(token_mint, amount)?;

        msg!(
            "Withdraw successful. Amount: {} Token: {:?}",
            amount,
            token_mint
        );
        Ok(())
    }
}

#[account]
pub struct Bank {
    pub whitelist_tokens: Vec<Pubkey>,
}

impl Bank {
    pub fn is_token_whitelisted(&self, mint: Pubkey) -> bool {
        let whitelist_tokens: [Pubkey; 3] = [
            Pubkey::new_from_array([
                0x39, 0x6E, 0xFC, 0x2F, 0x1E, 0x8B, 0x35, 0xB9, 0x8A, 0xDB, 0x6C, 0x70, 0x74, 0xFD,
                0x34, 0xDF, 0xF2, 0x0B, 0x36, 0xAD, 0x89, 0x59, 0xE3, 0x9B, 0x16, 0x42, 0x17, 0x91,
                0x44, 0x82, 0x74, 0xE2,
            ]),
            Pubkey::new_from_array([
                0x01, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11,
                0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1A, 0x1B, 0x1C, 0x1D, 0x1E,
                0x1F, 0x20, 0x21, 0x22,
            ]),
            Pubkey::new_from_array([
                0x4A, 0xBC, 0xD1, 0xAB, 0x5B, 0x6F, 0xE7, 0x94, 0xC8, 0xE3, 0x93, 0x8E, 0xB5, 0xDC,
                0xF8, 0xD1, 0x82, 0xAA, 0x71, 0x8F, 0xC9, 0x7F, 0x3F, 0x8A, 0xD4, 0x23, 0x6A, 0x72,
                0xF1, 0xBB, 0x62, 0xFF,
            ]),
        ];

        whitelist_tokens.contains(&mint)
    }
}

#[account]
pub struct BankAccount {
    pub owner: Pubkey,
    pub balances: Vec<(Pubkey, u64)>,
}

impl BankAccount {
    pub fn add_balance(&mut self, token: Pubkey, amount: u64) -> Result<()> {
        for (key, balance) in &mut self.balances {
            if *key == token {
                *balance += amount;
                return Ok(());
            }
        }
        self.balances.push((token, amount));
        Ok(())
    }

    pub fn subtract_balance(&mut self, token: Pubkey, amount: u64) -> Result<()> {
        for (key, balance) in &mut self.balances {
            if *key == token {
                if *balance < amount {
                    return Err(ErrorCode::InsufficientUserBalance.into());
                }
                *balance -= amount;
                return Ok(());
            }
        }
        Err(ErrorCode::InsufficientUserBalance.into())
    }

    pub fn has_sufficient_balance(&self, token: Pubkey, amount: u64) -> bool {
        self.balances
            .iter()
            .find(|(key, _)| *key == token)
            .map_or(false, |(_, balance)| *balance >= amount)
    }
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(
        mut,
        constraint = user_ata.mint == bank_ata.mint,
        token::authority = user_authority
    )]
    pub user_ata: Account<'info, TokenAccount>,

    #[account(mut)]
    pub bank_ata: Account<'info, TokenAccount>,

    #[account(mut)]
    pub user_bank_account: Account<'info, BankAccount>,

    #[account(mut)]
    pub bank: Account<'info, Bank>,

    pub user_authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(
        mut,
        constraint = user_ata.mint == bank_ata.mint,
        token::authority = bank_authority
    )]
    pub user_ata: Account<'info, TokenAccount>,

    #[account(mut)]
    pub bank_ata: Account<'info, TokenAccount>,

    #[account(mut)]
    pub user_bank_account: Account<'info, BankAccount>,

    #[account(mut)]
    pub bank: Account<'info, Bank>,

    pub bank_authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Insufficient token balance in your wallet.")]
    InsufficientBalance,
    #[msg("You do not have enough balance in your bank account.")]
    InsufficientUserBalance,
    #[msg("The provided token is not supported.")]
    InvalidToken,
}
