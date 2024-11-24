use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};

declare_id!("83zrVmcBMziMhvMPBE1WWnexBz6UhMtiRrNF7F8nLS7e");

#[program]
pub mod bank {
    use super::*;

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        let user_account = &mut ctx.accounts.user_account;
        let user_token_account = &mut ctx.accounts.user_token_account;
        let bank_token_account = &mut ctx.accounts.bank_token_account;

        // Kiểm tra số dư trên ví của người dùng
        if user_token_account.amount < amount {
            return Err(ErrorCode::InsufficientBalance.into());
        }

        // Kiểm tra loại token hợp lệ
        let token_mint = user_token_account.mint;
        if token_mint != BTC_MINT_KEY && token_mint != SOL_MINT_KEY && token_mint != USDC_MINT_KEY {
            return Err(ErrorCode::InvalidToken.into());
        }

        // Chuyển token từ ví người dùng vào ví ngân hàng
        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            anchor_spl::token::Transfer {
                from: user_token_account.to_account_info(),
                to: bank_token_account.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        );
        anchor_spl::token::transfer(cpi_ctx, amount)?;

        // Cập nhật số dư trong user_account (ngân hàng ghi nhận giao dịch)
        user_account.balance += amount;

        // Lưu xác nhận giao dịch
        msg!("Deposit of {} tokens from {} successful.", amount, ctx.accounts.user.key);

        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        let user_account = &mut ctx.accounts.user_account;
        let user_token_account = &mut ctx.accounts.user_token_account;
        let bank_token_account = &mut ctx.accounts.bank_token_account;

        // Kiểm tra số dư trước khi rút
        if user_account.balance < amount {
            return Err(ErrorCode::InsufficientFunds.into());
        }

        // Chuyển token từ ví ngân hàng vào ví người dùng
        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            anchor_spl::token::Transfer {
                from: bank_token_account.to_account_info(),
                to: user_token_account.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        );
        anchor_spl::token::transfer(cpi_ctx, amount)?;

        // Giảm số dư trong user_account (ngân hàng ghi nhận giao dịch)
        user_account.balance -= amount;

        // Lưu xác nhận giao dịch
        msg!("Withdrawal of {} tokens by {} successful.", amount, ctx.accounts.user.key);

        Ok(())
    }
}


#[account]
pub struct UserAccount {
    pub owner: Pubkey, // Người sở hữu tài khoản
    pub balance: u64,  // Số dư token
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(mut)]
    pub user_account: Account<'info, UserAccount>,

    #[account(mut, constraint = 
        user_token_account.mint == BTC_MINT_KEY || 
        user_token_account.mint == SOL_MINT_KEY || 
        user_token_account.mint == USDC_MINT_KEY
    )]
    pub user_token_account: Account<'info, TokenAccount>,

    #[account(mut, constraint = 
        bank_token_account.mint == BTC_MINT_KEY || 
        bank_token_account.mint == SOL_MINT_KEY || 
        bank_token_account.mint == USDC_MINT_KEY
    )]
    pub bank_token_account: Account<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut)]
    pub user_account: Account<'info, UserAccount>,

    #[account(mut, constraint = 
        user_token_account.mint == BTC_MINT_KEY || 
        user_token_account.mint == SOL_MINT_KEY || 
        user_token_account.mint == USDC_MINT_KEY
    )]
    pub user_token_account: Account<'info, TokenAccount>,

    #[account(mut, constraint = 
        bank_token_account.mint == BTC_MINT_KEY || 
        bank_token_account.mint == SOL_MINT_KEY || 
        bank_token_account.mint == USDC_MINT_KEY
    )]
    pub bank_token_account: Account<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}


#[error_code]
pub enum ErrorCode {
    #[msg("Insufficient token balance in your wallet.")]
    InsufficientBalance,
    #[msg("Not enough tokens in your account to withdraw.")]
    InsufficientFunds,
    #[msg("The provided token is not supported by the bank.")]
    InvalidToken,
}

pub const BTC_MINT_KEY: Pubkey = Pubkey::new_from_array([
    0x39, 0x6E, 0xFC, 0x2F, 0x1E, 0x8B, 0x35, 0xB9, 0x8A, 0xDB, 0x6C, 0x70, 0x74, 0xFD, 0x34, 0xDF,
    0xF2, 0x0B, 0x36, 0xAD, 0x89, 0x59, 0xE3, 0x9B, 0x16, 0x42, 0x17, 0x91, 0x44, 0x82, 0x74, 0xE2,
]);

pub const SOL_MINT_KEY: Pubkey = Pubkey::new_from_array([
    0x1, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x12,
    0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1A, 0x1B, 0x1C, 0x1D, 0x1E, 0x1F, 0x20, 0x21, 0x22,
]);

pub const USDC_MINT_KEY: Pubkey = Pubkey::new_from_array([
    0x4A, 0xBC, 0xD1, 0xAB, 0x5B, 0x6F, 0xE7, 0x94, 0xC8, 0xE3, 0x93, 0x8E, 0xB5, 0xDC, 0xF8, 0xD1,
    0x82, 0xAA, 0x71, 0x8F, 0xC9, 0x7F, 0x3F, 0x8A, 0xD4, 0x23, 0x6A, 0x72, 0xF1, 0xBB, 0x62, 0xFF,
]);
