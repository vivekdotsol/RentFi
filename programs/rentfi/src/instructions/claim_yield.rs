use crate::{
    errors::RentFiError,
    events::YieldClaimed,
    state::{InvestorPosition, RentVault, PRECISION},
};

use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

#[derive(Accounts)]
pub struct ClaimYield<'info> {
    #[account(mut)]
    pub investor: Signer<'info>,

    #[account(
        mut,
        seeds = [b"vault", vault.lease.as_ref()],
        bump = vault.bump,
    )]
    pub vault: Account<'info, RentVault>,

    #[account(
        mut,
        seeds = [
            b"position",
            vault.key().as_ref(),
            investor.key().as_ref()
        ],
        bump = investor_position.bump,
    )]
    pub investor_position: Account<'info, InvestorPosition>,

    #[account(
        mut,
        associated_token::mint = usdc_mint,
        associated_token::authority = vault,
    )]
    pub usdc_vault: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = usdc_mint,
        associated_token::authority = investor,
    )]
    pub investor_usdc_account: Account<'info, TokenAccount>,

    pub usdc_mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<ClaimYield>) -> Result<()> {
    let vault = &ctx.accounts.vault;
    let position = &mut ctx.accounts.investor_position;

    let delta = vault
        .yield_per_token_cumulative
        .saturating_sub(position.yield_per_token_snapshot);

    let claimable = ((position.rst_balance as u128)
        .checked_mul(delta)
        .ok_or(RentFiError::Overflow)?)
        / PRECISION;

    require!(claimable > 0, RentFiError::NoYieldAvailable);

    let amount = claimable as u64;

    let vault_seeds: &[&[u8]] = &[b"vault", vault.lease.as_ref(), &[vault.bump]];

    anchor_spl::token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            anchor_spl::token::Transfer {
                from: ctx.accounts.usdc_vault.to_account_info(),
                to: ctx.accounts.investor_usdc_account.to_account_info(),
                authority: ctx.accounts.vault.to_account_info(),
            },
            &[vault_seeds],
        ),
        amount,
    )?;

    position.yield_per_token_snapshot = vault.yield_per_token_cumulative;

    position.total_claimed = position
        .total_claimed
        .checked_add(amount)
        .ok_or(RentFiError::Overflow)?;

    emit!(YieldClaimed {
        vault: vault.key(),
        investor: ctx.accounts.investor.key(),
        usdc_claimed: amount,
    });

    Ok(())
}
