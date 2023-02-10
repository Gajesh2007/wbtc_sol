use anchor_lang::prelude::*;
use controller::{self, Controller, MintCtx, BurnCtx};
use members::{self, Members, Merchant};
use anchor_lang::solana_program::clock::{self, Clock};
use anchor_spl::token::{self, TokenAccount, Mint, Token};

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

fn is_merchant(
    merchant: Pubkey,
    merchant_state: Account<Merchant>,
    member_state: Pubkey,
    members: Pubkey
) -> bool {
    let merchant_state_pubkey = Pubkey::find_program_address(&[
        b"merchant".as_ref(),
        member_state.key().as_ref(),
        merchant.key().as_ref()
    ], &members);

    if merchant_state.key() == merchant_state_pubkey.0 {
        return true
    } else {
        return false
    }
}

#[program]
pub mod factory {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let factory = &mut ctx.accounts.factory_state;
        factory.controller_program = ctx.accounts.controller_program.key();
        factory.controller_state = ctx.accounts.controller_state.key();
        factory.admin = ctx.accounts.controller_program.key();
        factory.mint_request_count = 0;
        factory.burn_request_count = 0;

        Ok(())
    }

    pub fn set_custodian_deposit_address(ctx: Context<SetCustodianDepositAddress>, deposit_address: String) -> Result<()> {
        let members_state = ctx.accounts.member_state.clone();

        if !is_merchant(ctx.accounts.merchant.key(), ctx.accounts.merchant_state.clone(), ctx.accounts.members.key(), ctx.accounts.member_state.key().clone()) {
            return Err(Errors::SenderNotAuthorized.into())
        }

        if members_state.custodian == ctx.accounts.signer.key() {
            return Err(Errors::SenderNotAuthorized.into())
        }

        if deposit_address.len() == 0 {
            return Err(Errors::InvalidDepositAddress.into())
        }
        
        let custodian_deposit_address = &mut ctx.accounts.custodian_deposit_address;
        custodian_deposit_address.address = deposit_address;

        Ok(())
    }

    pub fn set_merchant_deposit_address(ctx: Context<SetMerchantDepositAddress>, deposit_address: String) -> Result<()> {
        if !is_merchant(ctx.accounts.merchant.key(), ctx.accounts.merchant_state.clone(), ctx.accounts.members.key(), ctx.accounts.member_state.key().clone()) {
            return Err(Errors::SenderNotAuthorized.into())
        }

        if deposit_address.len() == 0 {
            return Err(Errors::InvalidDepositAddress.into())
        }
        
        let merchant_deposit_address = &mut ctx.accounts.merchant_deposit_address;
        merchant_deposit_address.address = deposit_address;

        Ok(())
    }

    pub fn add_mint_request(ctx: Context<AddMintRequest>, txid: String, deposit_address: String, amount: u64) -> Result<()> {
        let factory_state = &mut ctx.accounts.factory_state;

        if !is_merchant(ctx.accounts.merchant.key(), ctx.accounts.merchant_state.clone(), ctx.accounts.members.key(), ctx.accounts.member_state.key().clone()) {
            return Err(Errors::SenderNotAuthorized.into())
        }

        if deposit_address != ctx.accounts.custodian_deposit_address.address.to_string() {
            return Err(Errors::InvalidDepositAddress.into())
        }

        if txid.len() == 0 {
            return Err(Errors::InvalidTxid.into())
        }

        let timestamp: u64 = clock::Clock::get().unwrap().unix_timestamp.try_into().unwrap();

        let mint_request = &mut ctx.accounts.request;
        mint_request.requester = ctx.accounts.merchant.key();
        mint_request.amount = amount;
        mint_request.deposit_address = deposit_address;
        mint_request.txid = txid;
        mint_request.nonce = factory_state.mint_request_count + 1;
        factory_state.mint_request_count += 1;
        mint_request.timestamp = timestamp;
        mint_request.status = 0; // PENDING

        Ok(())
    }
    
    pub fn cancel_mint_request(ctx: Context<CancelMintRequest>, txid: String) -> Result<()> {
        if !is_merchant(ctx.accounts.merchant.key(), ctx.accounts.merchant_state.clone(), ctx.accounts.members.key(), ctx.accounts.member_state.key().clone()) {
            return Err(Errors::SenderNotAuthorized.into())
        }

        if txid.len() == 0 {
            return Err(Errors::InvalidTxid.into())
        }

        let mint_request = &mut ctx.accounts.request;
        mint_request.status = 1; // CANCELLED

        Ok(())
    }

    pub fn confirm_mint_request(ctx: Context<ConfirmMintRequest>, txid: String) -> Result<()> {
        if txid.len() == 0 {
            return Err(Errors::InvalidTxid.into())
        }

        let mint_request = &mut ctx.accounts.request;
        mint_request.status = 2; // APPROVED

        {
            let cpi_accounts = controller::cpi::accounts::MintCtx {
                controller_state: ctx.accounts.controller_state.to_account_info(),
                token_mint: ctx.accounts.token_mint.to_account_info(),
                to_token_account: ctx.accounts.token_account.to_account_info(),
                signer: ctx.accounts.factory_program.clone(),
                token_program: ctx.accounts.token_program.to_account_info(),
            };
    
            let cpi_ctx = CpiContext::new(
                ctx.accounts.factory_program.to_account_info(),
                cpi_accounts,
            );
            
            controller::cpi::mint(cpi_ctx, mint_request.amount);
        }

        Ok(())
    }

    pub fn reject_mint_request(ctx: Context<RejectMintRequest>, txid: String) -> Result<()> {
        if txid.len() == 0 {
            return Err(Errors::InvalidTxid.into())
        }

        let mint_request = &mut ctx.accounts.request;
        mint_request.status = 3; // REJECTED

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
    pub factory_state: Account<'info, FactoryState>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub controller_state: Account<'info, Controller>,

    pub system_program: Program<'info, System>,
    pub controller_program: UncheckedAccount<'info>
}


#[derive(Accounts)]
pub struct SetCustodianDepositAddress<'info> {
    #[account(
        mut,
    )]
    pub factory_state: Account<'info, FactoryState>,

    pub merchant: UncheckedAccount<'info>,
    pub merchant_state: Account<'info, Merchant>,
    pub members: UncheckedAccount<'info>,
    pub member_state: Account<'info, Members>,

    #[account(
        init_if_needed,
        seeds = [
            b"custodian_deposit".as_ref(),
            factory_state.key().as_ref(),
            merchant.key().as_ref()
        ],
        bump,
        payer = signer,
        space = 100
    )]
    pub custodian_deposit_address: Account<'info, DepositAddress>,

    #[account(mut)]
    pub signer: Signer<'info>,    

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SetMerchantDepositAddress<'info> {
    #[account(
        mut,
    )]
    pub factory_state: Account<'info, FactoryState>,

    pub merchant_state: Account<'info, Merchant>,
    pub members: UncheckedAccount<'info>,
    pub member_state: Account<'info, Members>,

    #[account(
        init_if_needed,
        seeds = [
            b"merchant_deposit".as_ref(),
            factory_state.key().as_ref(),
            merchant.key().as_ref()
        ],
        bump,
        payer = merchant,
        space = 100
    )]
    pub merchant_deposit_address: Account<'info, DepositAddress>,

    #[account(mut)]
    pub merchant: Signer<'info>,    

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(txid: String)]
pub struct AddMintRequest<'info> {
    #[account(
        mut,
    )]
    pub factory_state: Account<'info, FactoryState>,

    pub merchant_state: Account<'info, Merchant>,
    pub members: UncheckedAccount<'info>,
    pub member_state: Account<'info, Members>,

    #[account(
        init,
        seeds = [
            b"mint_request".as_ref(),
            factory_state.key().as_ref(),
            txid.as_ref()
        ],
        bump,
        payer = merchant,
        space = 1000
    )]
    pub request: Account<'info, Request>,

    #[account(
        seeds = [
            b"custodian_deposit".as_ref(),
            factory_state.key().as_ref(),
            merchant.key().as_ref()
        ],
        bump
    )]
    pub custodian_deposit_address: Account<'info, DepositAddress>,

    #[account(mut)]
    pub merchant: Signer<'info>,    

    pub system_program: Program<'info, System>,
}


#[derive(Accounts)]
#[instruction(txid: String)]
pub struct CancelMintRequest<'info> {
    #[account(
        mut,
    )]
    pub factory_state: Account<'info, FactoryState>,

    pub merchant_state: Account<'info, Merchant>,
    pub members: UncheckedAccount<'info>,
    pub member_state: Account<'info, Members>,

    #[account(
        seeds = [
            b"mint_request".as_ref(),
            factory_state.key().as_ref(),
            txid.as_ref()
        ],
        bump,
    )]
    pub request: Account<'info, Request>,

    #[account(mut)]
    pub merchant: Signer<'info>,    
}

#[derive(Accounts)]
#[instruction(txid: String)]
pub struct ConfirmMintRequest<'info> {
    #[account(
        mut,
        has_one = admin,
        has_one = controller_state,
        has_one = controller_program
    )]
    pub factory_state: Account<'info, FactoryState>,

    #[account(
        constraint = controller_state.factory == factory_program.key()
    )]
    pub controller_state: Account<'info, Controller>,
    pub controller_program: AccountInfo<'info>,

    pub token_mint: Account<'info, Mint>,
    #[account(
        mut,
        constraint = token_account.owner == request.requester
    )]
    pub token_account: Account<'info, TokenAccount>,

    #[account(
        seeds = [
            b"mint_request".as_ref(),
            factory_state.key().as_ref(),
            txid.as_ref()
        ],
        bump,
    )]
    pub request: Account<'info, Request>, 

    pub admin: Signer<'info>,
    pub factory_program: AccountInfo<'info>,

    pub token_program: Program<'info, Token>
}

#[derive(Accounts)]
#[instruction(txid: String)]
pub struct RejectMintRequest<'info> {
    #[account(
        has_one = admin
    )]
    pub factory_state: Account<'info, FactoryState>,

    #[account(
        seeds = [
            b"mint_request".as_ref(),
            factory_state.key().as_ref(),
            txid.as_ref()
        ],
        bump,
    )]
    pub request: Account<'info, Request>,

    #[account(mut)]
    pub admin: Signer<'info>,    
}

#[account]
pub struct FactoryState {
    /// Admin - Small DAO
    pub admin: Pubkey,

    /// Controller
    pub controller_state: Pubkey,
    pub controller_program: Pubkey,

    /// Mint Request Count
    pub mint_request_count: u128,

    /// Burn Request Count
    pub burn_request_count: u128,
}

#[account]
pub struct DepositAddress {
    pub address: String
}

#[account]
pub struct Request {
    /// Sender of the request
    pub requester: Pubkey,

    /// Amount of token to mint/burn
    pub amount: u64,

    /// custodian's asset address in mint, merchant's asset address in burn.
    pub deposit_address: String,
    
    /// asset txid for sending/redeeming asset in the mint/burn process.
    pub txid: String,

    /// serial number allocated for each request.
    pub nonce: u128,

    /// time of the request creation
    pub timestamp: u64,

    // Status of the request
    pub status: u8,

    pub bump: u8
}

#[error_code]
pub enum Errors {
    #[msg("sender not authorized for minting or burning.")]
    SenderNotAuthorized,
    #[msg("invalid asset deposit address")]
    InvalidDepositAddress,
    #[msg("invalid asset txid")]
    InvalidTxid,

}

// Request Status
// PENDING - 0
// CANCELLED - 1
// APPROVED - 2
// REJECTED - 3