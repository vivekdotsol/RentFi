use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct LeaseAttestation {
    pub landlord: Pubkey,

    /// Keccak/SHA-256 hex of the off-chain lease PDF (64 hex chars = 32 bytes)
    pub document_hash: [u8; 32],

    /// IPFS CID stored as up-to-64-byte string (base58 CIDv1)
    #[max_len(64)]
    pub ipfs_cid: String,

    /// Monthly rent amount in USDC micro-units (6 decimals)
    pub monthly_rent_usdc: u64,

    /// Lease start (Unix timestamp)
    pub lease_start: i64,

    /// Lease end (Unix timestamp)
    pub lease_end: i64,

    /// Human-readable property identifier
    #[max_len(128)]
    pub property_id: String,

    /// Current attestation status
    pub status: LeaseStatus,

    /// Bump for PDA derivation
    pub bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, InitSpace)]
pub enum LeaseStatus {
    Active,
    Defaulted,
    Closed,
}

#[account]
#[derive(InitSpace)]
pub struct RentVault {
    /// Back-reference to the lease PDA
    pub lease: Pubkey,

    /// Landlord authority
    pub landlord: Pubkey,

    /// Trusted oracle that can flag defaults
    pub oracle: Pubkey,

    /// The RST SPL mint
    pub rst_mint: Pubkey,

    /// Vault's USDC token account (ATA owned by vault PDA)
    pub usdc_vault: Pubkey,

    /// Total RST supply minted (mirrors SPL mint supply)
    pub total_rst_supply: u64,

    /// Price per RST token in USDC micro-units at primary sale
    pub rst_price_usdc: u64,

    /// Cumulative USDC deposited as rent (ever)
    pub total_rent_deposited: u64,

    /// Global "yield per token" accumulator (scaled by PRECISION)
    /// Each time rent arrives: yield_per_token += amount * PRECISION / supply
    pub yield_per_token_cumulative: u128,

    /// Number of months rent has been paid
    pub months_paid: u32,

    /// Whether the vault is flagged as defaulted
    pub is_defaulted: bool,

    /// Whether primary sale is still open
    pub sale_open: bool,

    /// Bump
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct InvestorPosition {
    pub vault: Pubkey,
    pub investor: Pubkey,

    /// RST balance held by this investor (mirrors ATA but cached)
    pub rst_balance: u64,

    /// yield_per_token_cumulative value at last claim / deposit
    pub yield_per_token_snapshot: u128,

    /// Total USDC yield claimed by this investor (ever)
    pub total_claimed: u64,

    pub bump: u8,
}

pub const PRECISION: u128 = 1_000_000_000_000; // 1e12
