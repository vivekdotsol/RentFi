use crate::{
    errors::RentFiError,
    events::DefaultFlagged,
    state::{LeaseAttestation, LeaseStatus, RentVault},
};

use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct FlagDefault<'info> {
    #[account(mut)]
    pub oracle: Signer<'info>,

    #[account(mut)]
    pub lease: Account<'info, LeaseAttestation>,

    #[account(
        mut,
        seeds = [b"vault", lease.key().as_ref()],
        bump = vault.bump,
    )]
    pub vault: Account<'info, RentVault>,
}

pub fn handler(ctx: Context<FlagDefault>, reason: String) -> Result<()> {
    require!(
        ctx.accounts.oracle.key() == ctx.accounts.vault.oracle,
        RentFiError::UnauthorisedOracle
    );

    ctx.accounts.vault.is_defaulted = true;
    ctx.accounts.lease.status = LeaseStatus::Defaulted;

    emit!(DefaultFlagged {
        vault: ctx.accounts.vault.key(),
        oracle: ctx.accounts.oracle.key(),
        reason,
        flagged_at: Clock::get()?.unix_timestamp,
    });

    Ok(())
}
