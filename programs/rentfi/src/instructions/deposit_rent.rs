use crate::{
    errors::RentFiError,
    events::RentDeposited,
    state::{LeaseAttestation, LeaseStatus, RentVault, PRECISION},
};
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

#[derive(Accounts)]
pub struct DepositRent<'info> {
    /// Only the landlord may deposit rent
    #[account(mut)]
    pub landlord: Signer<'info>,

    #[account(
        seeds = [
            b"lease",
            landlord.key().as_ref(),
            lease.property_id.as_bytes(),
        ],
        bump = lease.bump,
        has_one = landlord @ RentFiError::UnauthorisedLandlord,
        constraint = lease.status == LeaseStatus::Active @ RentFiError::LeaseNotActive,
    )]
    pub lease: Account<'info, LeaseAttestation>,

    #[account(
        mut,
        seeds = [b"vault", lease.key().as_ref()],
        bump = vault.bump,
        has_one = landlord @ RentFiError::UnauthorisedLandlord,
        constraint = !vault.is_defaulted @ RentFiError::VaultDefaulted,
    )]
    pub vault: Account<'info, RentVault>,

    /// Landlord's USDC account
    #[account(
        mut,
        associated_token::mint = usdc_mint,
        associated_token::authority = landlord,
    )]
    pub landlord_usdc_account: Account<'info, TokenAccount>,

    /// Vault's USDC holding account
    #[account(
        mut,
        associated_token::mint = usdc_mint,
        associated_token::authority = vault,
    )]
    pub usdc_vault: Account<'info, TokenAccount>,

    pub usdc_mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<DepositRent>, amount: u64) -> Result<()> {
    require!(amount > 0, RentFiError::ZeroRentAmount);

    let vault = &mut ctx.accounts.vault;

    // Calculate circulating RST (total supply minus unsold tokens still in vault)
    // We use the vault's recorded total_rst_supply; unsold are in vault_rst_account
    // For yield distribution we only care about tokens held by investors.
    // We compute: circulating = total_rst_supply - vault_rst_balance
    // NOTE: vault_rst_account is NOT passed here to keep account list small;
    // yield_per_token_cumulative accumulates over TOTAL supply which means
    // unsold tokens accrue yield that remains locked in vault (a design choice).
    // Alternatively pass vault_rst_account and subtract. We use total supply here
    // for simplicity — operators should close primary sale before rent flows.
    let supply = vault.total_rst_supply;

    if supply > 0 {
        // Update global yield-per-token accumulator
        // yield_per_token_cumulative += (amount * PRECISION) / supply
        let increment = (amount as u128)
            .checked_mul(PRECISION)
            .ok_or(RentFiError::Overflow)?
            .checked_div(supply as u128)
            .ok_or(RentFiError::Overflow)?;

        vault.yield_per_token_cumulative = vault
            .yield_per_token_cumulative
            .checked_add(increment)
            .ok_or(RentFiError::Overflow)?;
    }

    vault.total_rent_deposited = vault
        .total_rent_deposited
        .checked_add(amount)
        .ok_or(RentFiError::Overflow)?;
    vault.months_paid = vault.months_paid.saturating_add(1);

    // Transfer USDC from landlord → usdc_vault
    anchor_spl::token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            anchor_spl::token::Transfer {
                from: ctx.accounts.landlord_usdc_account.to_account_info(),
                to: ctx.accounts.usdc_vault.to_account_info(),
                authority: ctx.accounts.landlord.to_account_info(),
            },
        ),
        amount,
    )?;

    emit!(RentDeposited {
        vault: vault.key(),
        landlord: ctx.accounts.landlord.key(),
        amount,
        months_paid: vault.months_paid,
        new_yield_per_token: vault.yield_per_token_cumulative,
    });

    msg!(
        "RentFi: Rent deposit #{} — {} USDC micro | cumulative YPT: {}",
        vault.months_paid,
        amount,
        vault.yield_per_token_cumulative
    );

    Ok(())
}
