use crate::{
    errors::RentFiError,
    events::LeaseClosed,
    state::{LeaseAttestation, LeaseStatus, RentVault},
};

use anchor_lang::prelude::*;
use anchor_spl::token::TokenAccount;

#[derive(Accounts)]
pub struct CloseLease<'info> {
    #[account(mut)]
    pub landlord: Signer<'info>,

    #[account(
        mut,
        has_one = landlord
    )]
    pub lease: Account<'info, LeaseAttestation>,

    #[account(
        mut,
        seeds = [b"vault", lease.key().as_ref()],
        bump = vault.bump,
    )]
    pub vault: Account<'info, RentVault>,

    #[account(
        associated_token::mint = rst_mint,
        associated_token::authority = vault,
    )]
    pub vault_rst_account: Account<'info, TokenAccount>,

    pub rst_mint: Account<'info, anchor_spl::token::Mint>,
}

pub fn handler(ctx: Context<CloseLease>) -> Result<()> {
    require!(
        ctx.accounts.vault_rst_account.amount == ctx.accounts.vault.total_rst_supply,
        RentFiError::OutstandingRstTokens
    );

    ctx.accounts.lease.status = LeaseStatus::Closed;
    ctx.accounts.vault.sale_open = false;

    emit!(LeaseClosed {
        vault: ctx.accounts.vault.key(),
        landlord: ctx.accounts.landlord.key(),
        closed_at: Clock::get()?.unix_timestamp,
    });

    Ok(())
}
