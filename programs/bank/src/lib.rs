use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

declare_id!("83zrVmcBMziMhvMPBE1WWnexBz6UhMtiRrNF7F8nLS7e");

#[program]
pub mod bank {
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

    pub fn add_token(ctx: Context<AddToken>, mint: Pubkey) -> Result<()> {
        let bank = &mut ctx.accounts.bank;

        if bank.is_token_whitelisted(mint) {
            return Err(ErrorCode::TokenAlreadyWhitelisted.into());
        }

        bank.add_token(mint);

        msg!("Token {:?} đã được thêm vào danh sách whitelist.", mint);
        Ok(())
    }

    pub fn initialize_bank(ctx: Context<InitializeBank>) -> Result<()> {
        let bank = &mut ctx.accounts.bank;
        bank.whitelist_tokens = Vec::new();
        msg!("Bank initialized successfully");
        Ok(())
    }

    pub fn initialize_bank_account(ctx: Context<InitializeBankAccount>) -> Result<()> {
        let user_bank_account = &mut ctx.accounts.user_bank_account;
        user_bank_account.owner = ctx.accounts.owner.key();
        user_bank_account.balances = Vec::new();
        msg!("User bank account initialized successfully");
        Ok(())
    }
}

#[account]
pub struct Bank {
    pub whitelist_tokens: Vec<Pubkey>,
}

impl Bank {
    pub fn add_token(&mut self, mint: Pubkey) {
        self.whitelist_tokens.push(mint);
    }

    pub fn is_token_whitelisted(&self, mint: Pubkey) -> bool {
        self.whitelist_tokens.contains(&mint)
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
pub struct InitializeBankAccount<'info> {
    #[account(init, payer = owner, space = 8 + 32 + 64 * 100)] // Định rõ dung lượng bộ nhớ
    pub user_bank_account: Account<'info, BankAccount>,
    #[account(mut)]
    pub owner: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct InitializeBank<'info> {
    #[account(init, payer = authority, space = 8 + 32 * 100)]
    pub bank: Account<'info, Bank>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AddToken<'info> {
    #[account(mut)]
    pub bank: Account<'info, Bank>,
    pub authority: Signer<'info>,
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
    #[msg("Token is already whitelisted.")]
    TokenAlreadyWhitelisted,
}
