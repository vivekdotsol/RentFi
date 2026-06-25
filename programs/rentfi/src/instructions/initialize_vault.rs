use crate::{
    errors::RentFiError,
    events::VaultInitialized,
    state::{LeaseAttestation, LeaseStatus, RentVault},
};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct InitializeVaultParams {
    /// Total RST tokens to mint (represents total lease tokenisation)
    pub total_rst_supply: u64,
    /// Primary-sale price per RST token in USDC micro-units
    pub rst_price_usdc: u64,
    /// The oracle pubkey authorised to flag defaults
    pub oracle: Pubkey,
}

#[derive(Accounts)]
#[instruction(params: InitializeVaultParams)]
pub struct InitializeVault<'info> {
    #[account(mut)]
    pub landlord: Signer<'info>,

    /// The verified lease this vault is backed by
    #[account(
        mut,
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

    /// Vault PDA — authority over the RST mint and USDC token account
    #[account(
        init,
        payer = landlord,
        space = 8 + RentVault::INIT_SPACE,
        seeds = [b"vault", lease.key().as_ref()],
        bump,
    )]
    pub vault: Account<'info, RentVault>,

    /// RST SPL Mint — owned by the vault PDA
    #[account(
        init,
        payer = landlord,
        mint::decimals = 6,
        mint::authority = vault,
        seeds = [b"rst_mint", vault.key().as_ref()],
        bump,
    )]
    pub rst_mint: Account<'info, Mint>,

    /// Vault's USDC holding account (ATA owned by vault PDA)
    #[account(
        init,
        payer = landlord,
        associated_token::mint = usdc_mint,
        associated_token::authority = vault,
    )]
    pub usdc_vault: Account<'info, TokenAccount>,

    /// Vault's own RST token account (holds unsold tokens)
    #[account(
        init,
        payer = landlord,
        associated_token::mint = rst_mint,
        associated_token::authority = vault,
    )]
    pub vault_rst_account: Account<'info, TokenAccount>,

    /// USDC mint (must be real USDC on mainnet; mock on devnet)
    pub usdc_mint: Account<'info, Mint>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handler(ctx: Context<InitializeVault>, params: InitializeVaultParams) -> Result<()> {
    require!(params.total_rst_supply > 0, RentFiError::ZeroRstSupply);
    require!(params.rst_price_usdc > 0, RentFiError::ZeroRstPrice);

    let lease_key = ctx.accounts.lease.key();
    let vault_key = ctx.accounts.vault.key();
    let vault_bump = ctx.bumps.vault;

    {
        let vault = &mut ctx.accounts.vault;

        vault.lease = lease_key;
        vault.landlord = ctx.accounts.landlord.key();
        vault.oracle = params.oracle;
        vault.rst_mint = ctx.accounts.rst_mint.key();
        vault.usdc_vault = ctx.accounts.usdc_vault.key();
        vault.total_rst_supply = params.total_rst_supply;
        vault.rst_price_usdc = params.rst_price_usdc;
        vault.total_rent_deposited = 0;
        vault.yield_per_token_cumulative = 0;
        vault.months_paid = 0;
        vault.is_defaulted = false;
        vault.sale_open = true;
        vault.bump = vault_bump;
    } // <-- mutable borrow ends here

    let signer_seeds: &[&[u8]] = &[b"vault", lease_key.as_ref(), &[vault_bump]];

    anchor_spl::token::mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            anchor_spl::token::MintTo {
                mint: ctx.accounts.rst_mint.to_account_info(),
                to: ctx.accounts.vault_rst_account.to_account_info(),
                authority: ctx.accounts.vault.to_account_info(),
            },
            &[signer_seeds],
        ),
        params.total_rst_supply,
    )?;

    emit!(VaultInitialized {
        vault_pda: vault_key,
        lease_pda: lease_key,
        rst_mint: ctx.accounts.rst_mint.key(),
        total_rst_supply: params.total_rst_supply,
        rst_price_usdc: params.rst_price_usdc,
    });

    Ok(())
}
