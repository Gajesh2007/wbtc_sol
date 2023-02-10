use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, MintTo, Burn, TokenAccount, Token};

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod controller {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let controller = &mut ctx.accounts.controller_state;

        controller.token_mint = ctx.accounts.token_mint.key();
        controller.owner = ctx.accounts.signer.key();

        Ok(())
    }

    pub fn set_members(ctx: Context<SetMembers>) -> Result<()> {
        let controller = &mut ctx.accounts.controller_state;
        controller.members = ctx.accounts.members.key();

        Ok(())
    }

    pub fn set_factory(ctx: Context<SetFactory>) -> Result<()> {
        let controller = &mut ctx.accounts.controller_state;
        controller.factory = ctx.accounts.factory.key();

        Ok(())
    }

    pub fn mint(ctx: Context<MintCtx>, amount: u64) -> Result<()> {
        let controller = &mut ctx.accounts.controller_state;
        
        let cpi_accounts = MintTo {
            mint: ctx.accounts.token_mint.to_account_info(),
            to: ctx.accounts.to_token_account.to_account_info(),
            authority: ctx.accounts.signer.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            cpi_accounts,
        );
        
        token::mint_to(cpi_ctx, amount)?;

        Ok(())
    }

    pub fn burn(ctx: Context<BurnCtx>, amount: u64) -> Result<()> {
        let controller = &mut ctx.accounts.controller_state;
        
        let cpi_accounts = Burn {
            mint: ctx.accounts.token_mint.to_account_info(),
            from: ctx.accounts.from_token_account.to_account_info(),
            authority: ctx.accounts.signer.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            cpi_accounts,
        );
        
        token::burn(cpi_ctx, amount)?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        space = 1000,
        payer = signer
    )]
    pub controller_state: Account<'info, Controller>,

    pub token_mint: Account<'info, Mint>,

    #[account(mut)]
    pub signer: Signer<'info>,

    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
pub struct SetMembers<'info> {
    #[account(
        mut
    )]
    pub controller_state: Account<'info, Controller>,

    /// CHECK: Should be Members 
    pub members: UncheckedAccount<'info>,

    #[account(
        mut,
        constraint = signer.key() == controller_state.owner
    )]
    pub signer: Signer<'info>,
}

#[derive(Accounts)]
pub struct SetFactory<'info> {
    #[account(
        mut
    )]
    pub controller_state: Account<'info, Controller>,

    /// CHECK: Should be factory 
    pub factory: UncheckedAccount<'info>,

    #[account(
        mut,
        constraint = signer.key() == controller_state.owner
    )]
    pub signer: Signer<'info>,
}

#[derive(Accounts)]
pub struct MintCtx<'info> {
    #[account(
        mut,
        constraint = signer.key() == controller_state.factory @ Errors::SenderNotAuthorized
    )]
    pub controller_state: Account<'info, Controller>,

    #[account(
        constraint = token_mint.key() == controller_state.token_mint
    )]
    pub token_mint: Account<'info, Mint>,
    #[account(mut)]
    pub to_token_account: Account<'info, TokenAccount>,

    #[account(
        constraint = signer.is_signer == true
    )]
    pub signer: AccountInfo<'info>,

    pub token_program: Program<'info, Token>
}

#[derive(Accounts)]
pub struct BurnCtx<'info> {
    #[account(
        mut,
        constraint = signer.key() == controller_state.factory @ Errors::SenderNotAuthorized
    )]
    pub controller_state: Account<'info, Controller>,

    #[account(
        constraint = token_mint.key() == controller_state.token_mint
    )]
    pub token_mint: Account<'info, Mint>,
    #[account(mut)]
    pub from_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
    )]
    pub signer: Signer<'info>,

    pub token_program: Program<'info, Token>
}

#[account]
pub struct Controller {
    /// Owner
    pub owner: Pubkey,
    /// WBTC Mint
    pub token_mint: Pubkey,
    /// Members Program
    pub members: Pubkey,
    /// Factory
    pub factory: Pubkey,

    /// Pause State
    pub paused: bool
}

#[error_code]
pub enum Errors {
    #[msg("sender not authorized for minting or burning.")]
    SenderNotAuthorized,
}