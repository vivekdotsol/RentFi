use crate::{
    errors::RentFiError,
    events::DefaultResolved,
    state::{LeaseAttestation, LeaseStatus, RentVault},
};

use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct ResolveDefault<'info> {
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
}

pub fn handler(ctx: Context<ResolveDefault>) -> Result<()> {
    require!(ctx.accounts.vault.is_defaulted, RentFiError::NotDefaulted);

    ctx.accounts.vault.is_defaulted = false;
    ctx.accounts.lease.status = LeaseStatus::Active;

    emit!(DefaultResolved {
        vault: ctx.accounts.vault.key(),
        landlord: ctx.accounts.landlord.key(),
        resolved_at: Clock::get()?.unix_timestamp,
    });

    Ok(())
}
