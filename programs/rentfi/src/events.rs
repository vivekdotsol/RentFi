use anchor_lang::prelude::*;

#[event]
pub struct LeaseRegistered {
    pub landlord: Pubkey,
    pub lease_pda: Pubkey,
    pub property_id: String,
    pub monthly_rent_usdc: u64,
    pub lease_start: i64,
    pub lease_end: i64,
}

#[event]
pub struct VaultInitialized {
    pub vault_pda: Pubkey,
    pub lease_pda: Pubkey,
    pub rst_mint: Pubkey,
    pub total_rst_supply: u64,
    pub rst_price_usdc: u64,
}

#[event]
pub struct RstPurchased {
    pub vault: Pubkey,
    pub investor: Pubkey,
    pub rst_amount: u64,
    pub usdc_paid: u64,
}

#[event]
pub struct RentDeposited {
    pub vault: Pubkey,
    pub landlord: Pubkey,
    pub amount: u64,
    pub months_paid: u32,
    pub new_yield_per_token: u128,
}

#[event]
pub struct YieldClaimed {
    pub vault: Pubkey,
    pub investor: Pubkey,
    pub usdc_claimed: u64,
}

#[event]
pub struct DefaultFlagged {
    pub vault: Pubkey,
    pub oracle: Pubkey,
    pub reason: String,
    pub flagged_at: i64,
}

#[event]
pub struct DefaultResolved {
    pub vault: Pubkey,
    pub landlord: Pubkey,
    pub resolved_at: i64,
}

#[event]
pub struct LeaseClosed {
    pub vault: Pubkey,
    pub landlord: Pubkey,
    pub closed_at: i64,
}
