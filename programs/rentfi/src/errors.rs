use anchor_lang::prelude::*;

#[error_code]
pub enum RentFiError {
    #[msg("Lease has already expired")]
    LeaseExpired,

    #[msg("Lease is not active")]
    LeaseNotActive,

    #[msg("Vault is currently flagged as defaulted")]
    VaultDefaulted,

    #[msg("Primary sale is closed; RST tokens must be purchased on secondary market")]
    SaleClosed,

    #[msg("Insufficient RST tokens remaining in vault for purchase")]
    InsufficientRstSupply,

    #[msg("USDC amount must be a multiple of RST price")]
    InvalidPurchaseAmount,

    #[msg("No yield available to claim")]
    NoYieldAvailable,

    #[msg("Caller is not the authorised oracle")]
    UnauthorisedOracle,

    #[msg("Caller is not the landlord")]
    UnauthorisedLandlord,

    #[msg("Vault is not in default; nothing to resolve")]
    NotDefaulted,

    #[msg("Lease is already closed")]
    LeaseClosed,

    #[msg("Cannot close vault while investors hold RST tokens")]
    OutstandingRstTokens,

    #[msg("Monthly rent amount must be greater than zero")]
    ZeroRentAmount,

    #[msg("RST price must be greater than zero")]
    ZeroRstPrice,

    #[msg("RST supply must be greater than zero")]
    ZeroRstSupply,

    #[msg("Arithmetic overflow")]
    Overflow,
}
