use crate::{
    errors::RentFiError,
    events::RstPurchased,
    state::{InvestorPosition, RentVault, PRECISION},
};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};

#[derive(Accounts)]
pub struct PurchaseRst<'info> {
    #[account(mut)]
    pub investor: Signer<'info>,

    #[account(
        mut,
        seeds = [b"vault", vault.lease.as_ref()],
        bump = vault.bump,
        constraint = !vault.is_defaulted @ RentFiError::VaultDefaulted,
        constraint = vault.sale_open @ RentFiError::SaleClosed,
    )]
    pub vault: Account<'info, RentVault>,

    /// Investor's position account (tracks RST balance + yield snapshot)
    #[account(
        init_if_needed,
        payer = investor,
        space = 8 + InvestorPosition::INIT_SPACE,
        seeds = [b"position", vault.key().as_ref(), investor.key().as_ref()],
        bump,
    )]
    pub investor_position: Account<'info, InvestorPosition>,

    /// Vault's RST token account (source of tokens for primary sale)
    #[account(
        mut,
        associated_token::mint = rst_mint,
        associated_token::authority = vault,
    )]
    pub vault_rst_account: Account<'info, TokenAccount>,

    /// Investor's RST token account (destination)
    #[account(
        init_if_needed,
        payer = investor,
        associated_token::mint = rst_mint,
        associated_token::authority = investor,
    )]
    pub investor_rst_account: Account<'info, TokenAccount>,

    /// Investor's USDC account (source of payment)
    #[account(
        mut,
        associated_token::mint = usdc_mint,
        associated_token::authority = investor,
    )]
    pub investor_usdc_account: Account<'info, TokenAccount>,

    /// Vault's USDC holding account (destination of payment)
    #[account(
        mut,
        associated_token::mint = usdc_mint,
        associated_token::authority = vault,
    )]
    pub usdc_vault: Account<'info, TokenAccount>,

    pub rst_mint: Account<'info, Mint>,
    pub usdc_mint: Account<'info, Mint>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<PurchaseRst>, usdc_amount: u64) -> Result<()> {
    let vault = &mut ctx.accounts.vault;

    // Validate purchase amount is exact multiple of RST price
    require!(
        usdc_amount % vault.rst_price_usdc == 0,
        RentFiError::InvalidPurchaseAmount
    );

    let rst_amount = usdc_amount
        .checked_div(vault.rst_price_usdc)
        .ok_or(RentFiError::Overflow)?;

    // Check vault has enough unsold RSTs
    require!(
        ctx.accounts.vault_rst_account.amount >= rst_amount,
        RentFiError::InsufficientRstSupply
    );

    // --- Settle any pending yield for this investor BEFORE updating balance ---
    let position = &mut ctx.accounts.investor_position;
    if position.rst_balance > 0 {
        let pending = pending_yield(
            position.rst_balance,
            vault.yield_per_token_cumulative,
            position.yield_per_token_snapshot,
        )?;
        position.total_claimed = position
            .total_claimed
            .checked_add(pending)
            .ok_or(RentFiError::Overflow)?;
        // NOTE: actual USDC transfer of pending yield happens via claim_yield;
        // here we just advance the snapshot to prevent double-counting.
    }
    position.yield_per_token_snapshot = vault.yield_per_token_cumulative;
    position.vault = vault.key();
    position.investor = ctx.accounts.investor.key();
    position.rst_balance = position
        .rst_balance
        .checked_add(rst_amount)
        .ok_or(RentFiError::Overflow)?;

    // 1. Transfer USDC from investor → usdc_vault
    anchor_spl::token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            anchor_spl::token::Transfer {
                from: ctx.accounts.investor_usdc_account.to_account_info(),
                to: ctx.accounts.usdc_vault.to_account_info(),
                authority: ctx.accounts.investor.to_account_info(),
            },
        ),
        usdc_amount,
    )?;

    // 2. Transfer RST from vault_rst_account → investor_rst_account
    let lease_key = vault.lease;
    let vault_bump = vault.bump;
    let vault_seeds: &[&[u8]] = &[b"vault", lease_key.as_ref(), &[vault_bump]];
    let signer_seeds = &[vault_seeds];

    anchor_spl::token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            anchor_spl::token::Transfer {
                from: ctx.accounts.vault_rst_account.to_account_info(),
                to: ctx.accounts.investor_rst_account.to_account_info(),
                authority: ctx.accounts.vault.to_account_info(),
            },
            signer_seeds,
        ),
        rst_amount,
    )?;

    emit!(RstPurchased {
        vault: vault.key(),
        investor: ctx.accounts.investor.key(),
        rst_amount,
        usdc_paid: usdc_amount,
    });

    msg!(
        "RentFi: {} RST purchased by {} for {} USDC micro",
        rst_amount,
        ctx.accounts.investor.key(),
        usdc_amount
    );

    Ok(())
}

/// Compute pending yield for an investor using the global accumulator pattern.
pub fn pending_yield(balance: u64, global_ypt: u128, snapshot_ypt: u128) -> Result<u64> {
    let delta = global_ypt.saturating_sub(snapshot_ypt);
    let raw = (balance as u128)
        .checked_mul(delta)
        .ok_or(error!(RentFiError::Overflow))?;
    Ok((raw / PRECISION) as u64)
}
