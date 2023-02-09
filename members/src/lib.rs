use anchor_lang::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod members {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let member = &mut ctx.accounts.member_state;

        member.admin = ctx.accounts.admin.key();

        Ok(())
    }

    pub fn set_custodian(ctx: Context<SetCustodian>) -> Result<()> {
        let member = &mut ctx.accounts.member_state;

        member.custodian = ctx.accounts.custodian.key();

        Ok(())
    }

    pub fn add_merchant(ctx: Context<AddMerchant>) -> Result<()> {
        let merchant = &mut ctx.accounts.merchant_state;

        merchant.merchant = ctx.accounts.merchant.key();
        merchant.active = true;
        merchant.bump = *ctx.bumps.get("merchant").unwrap();

        Ok(())
    }

    pub fn remove_merchant(ctx: Context<RemoveMerchant>) -> Result<()> {
        let merchant = &mut ctx.accounts.merchant_state;

        merchant.active = false;

        Ok(())
    }
    
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        space = 1000,
        payer = payer
    )]
    pub member_state: Account<'info, Members>,

    #[account(mut)]
    pub payer: Signer<'info>,

    /// CHECK: TODO. Probably Small DAO
    pub admin: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
pub struct SetCustodian<'info> {
    #[account(
        mut,
        has_one = admin
    )]
    pub member_state: Account<'info, Members>,

    #[account(mut)]
    pub admin: Signer<'info>,

    pub custodian: UncheckedAccount<'info>
}

#[derive(Accounts)]
pub struct AddMerchant<'info> {
    #[account(
        mut,
        has_one = admin
    )]
    pub member_state: Account<'info, Members>,

    #[account(mut)]
    pub admin: Signer<'info>,

    pub merchant: UncheckedAccount<'info>,

    #[account(
        init_if_needed,
        seeds = [
            b"merchant".as_ref(),
            member_state.key().as_ref(),
            merchant.key().as_ref()
        ],
        space = 100,
        payer = admin,
        bump,
    )]
    pub merchant_state: Account<'info, Merchant>,

    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
pub struct RemoveMerchant<'info> {
    #[account(
        mut,
        has_one = admin
    )]
    pub member_state: Account<'info, Members>,

    #[account(mut)]
    pub admin: Signer<'info>,

    pub merchant: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [
            b"merchant".as_ref(),
            member_state.key().as_ref(),
            merchant.key().as_ref()
        ],
        bump = merchant_state.bump,
    )]
    pub merchant_state: Account<'info, Merchant>,
}

#[account]
pub struct Members {
    /// Admin
    pub admin: Pubkey,
    /// Custodian
    pub custodian: Pubkey,
}

#[account]
pub struct Merchant {
    /// Merchant
    pub merchant: Pubkey,

    /// Merchant State
    pub active: bool,

    /// Seed Bump
    pub bump: u8
}