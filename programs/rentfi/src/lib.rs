use anchor_lang::prelude::*;

pub mod errors;
pub mod events;
pub mod instructions;
pub mod state;

use instructions::*;

declare_id!("Fi7mt4CqYE158ULufrx818hKLPKC8Jaz51o8rAZeaJWg");

#[program]
pub mod rentfi {
    use super::*;

    /// Step 1 — Landlord registers a lease on-chain.
    /// An attestation PDA is created that stores lease metadata and a hash
    /// of the off-chain document (IPFS CID or SHA-256).
    pub fn register_lease(ctx: Context<RegisterLease>, params: RegisterLeaseParams) -> Result<()> {
        instructions::register_lease::handler(ctx, params)
    }

    /// Step 2 — Landlord initialises a vault + mints RST tokens.
    /// The vault holds all incoming rent payments.
    /// RST SPL tokens represent pro-rata claims on vault income.
    pub fn initialize_vault(
        ctx: Context<InitializeVault>,
        params: InitializeVaultParams,
    ) -> Result<()> {
        instructions::initialize_vault::handler(ctx, params)
    }

    pub fn purchase_rst(ctx: Context<PurchaseRst>, usdc_amount: u64) -> Result<()> {
        instructions::purchase_rst::handler(ctx, usdc_amount)
    }

    pub fn deposit_rent(ctx: Context<DepositRent>, amount: u64) -> Result<()> {
        instructions::deposit_rent::handler(ctx, amount)
    }

    pub fn claim_yield(ctx: Context<ClaimYield>) -> Result<()> {
        instructions::claim_yield::handler(ctx)
    }

    pub fn flag_default(ctx: Context<FlagDefault>, reason: String) -> Result<()> {
        instructions::flag_default::handler(ctx, reason)
    }

    pub fn resolve_default(ctx: Context<ResolveDefault>) -> Result<()> {
        instructions::resolve_default::handler(ctx)
    }

    pub fn close_lease(ctx: Context<CloseLease>) -> Result<()> {
        instructions::close_lease::handler(ctx)
    }
}
