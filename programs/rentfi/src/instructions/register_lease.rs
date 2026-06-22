use crate::{
    errors::RentFiError,
    events::LeaseRegistered,
    state::{LeaseAttestation, LeaseStatus},
};
use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct RegisterLeaseParams {
    /// SHA-256 of the lease PDF bytes
    pub document_hash: [u8; 32],
    /// IPFS CIDv1 of the lease document
    pub ipfs_cid: String,
    /// Monthly rent in USDC micro-units (1 USDC = 1_000_000)
    pub monthly_rent_usdc: u64,
    /// Unix timestamp — lease start date
    pub lease_start: i64,
    /// Unix timestamp — lease end date  
    pub lease_end: i64,
    /// Free-form property identifier (address, unit, etc.)
    pub property_id: String,
}

#[derive(Accounts)]
#[instruction(params: RegisterLeaseParams)]
pub struct RegisterLease<'info> {
    /// The landlord who owns the property
    #[account(mut)]
    pub landlord: Signer<'info>,

    /// LeaseAttestation PDA — one per (landlord, property_id)
    #[account(
        init,
        payer = landlord,
        space = 8 + LeaseAttestation::INIT_SPACE,
        seeds = [
            b"lease",
            landlord.key().as_ref(),
            params.property_id.as_bytes(),
        ],
        bump,
    )]
    pub lease: Account<'info, LeaseAttestation>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<RegisterLease>, params: RegisterLeaseParams) -> Result<()> {
    let clock = Clock::get()?;

    require!(params.monthly_rent_usdc > 0, RentFiError::ZeroRentAmount);
    require!(
        params.lease_end > clock.unix_timestamp,
        RentFiError::LeaseExpired
    );

    let lease = &mut ctx.accounts.lease;
    lease.landlord = ctx.accounts.landlord.key();
    lease.document_hash = params.document_hash;
    lease.ipfs_cid = params.ipfs_cid.clone();
    lease.monthly_rent_usdc = params.monthly_rent_usdc;
    lease.lease_start = params.lease_start;
    lease.lease_end = params.lease_end;
    lease.property_id = params.property_id.clone();
    lease.status = LeaseStatus::Active;
    lease.bump = ctx.bumps.lease;

    emit!(LeaseRegistered {
        landlord: ctx.accounts.landlord.key(),
        lease_pda: lease.key(),
        property_id: params.property_id,
        monthly_rent_usdc: params.monthly_rent_usdc,
        lease_start: params.lease_start,
        lease_end: params.lease_end,
    });

    msg!(
        "RentFi: Lease registered — property: {} | monthly rent: {} USDC micro",
        lease.property_id,
        lease.monthly_rent_usdc
    );

    Ok(())
}
