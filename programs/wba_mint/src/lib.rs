use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount, MintTo},
};

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod wba_mint {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, price_per_wba:u64) -> Result<()> {
        ctx.accounts.state.authority = *ctx.accounts.authority.key;
        ctx.accounts.state.price_per_wba = price_per_wba;
        Ok(())
    }

    pub fn mint(ctx: Context<MintWBA>, amount:u8) -> Result<()> {
        let cadet = ctx.accounts.cadet.to_account_info();
        let cadet_ata = ctx.accounts.cadet_ata.to_account_info();
        let vault = ctx.accounts.wba_vault.to_account_info();
        let mint = ctx.accounts.wba_mint.to_account_info();
        let auth = ctx.accounts.wba_auth.to_account_info();
        let token_program = ctx.accounts.token_program.to_account_info();
        let sys_program = ctx.accounts.system_program.to_account_info();

        let cpi_sol = CpiContext::new(sys_program, Transfer { from:cadet.clone(), to:vault });
        transfer(cpi_sol, u64::from(amount)*ctx.accounts.state.price_per_wba)?;

        let seeds = &[
            &b"wba_auth"[..],
            &[*ctx.bumps.get("wba_auth").unwrap()],
        ];

        let signer = &[&seeds[..]];

        let cpi_mint = CpiContext::new_with_signer(
            token_program,
            MintTo {
                mint:mint,
                to:cadet_ata,
                authority:auth,
            },
            signer,
        );

        anchor_spl::token::mint_to(cpi_mint, u64::from(amount)*1_000_000)?;
        Ok(())
    }

    pub fn claim(ctx: Context<Claim>) -> Result<()> {
        let sys_program = ctx.accounts.system_program.to_account_info();
        let from = ctx.accounts.wba_vault.to_account_info();
        let to = ctx.accounts.pay_to.to_account_info();

        let seeds = &[
            b"wba_vault",
            ctx.accounts.wba_mint.key.as_ref(),
            &[*ctx.bumps.get("wba_vault").unwrap()],
        ];

        let signer = &[&seeds[..]];

        let cpi = CpiContext::new_with_signer(sys_program, Transfer { from:from.clone(), to }, signer);
        transfer(cpi, from.lamports())?;
        Ok(())
    }
    
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        init, 
        seeds = [b"wba_mint"], 
        payer = authority, 
        bump, 
        mint::decimals = 6,
        mint::authority = wba_auth)]
    pub wba_mint: Account<'info, Mint>,
    /// CHECK: just signs,
    #[account(seeds = [b"wba_auth"], bump)]
    pub wba_auth: UncheckedAccount<'info>,
    #[account(init, seeds = [b"state"], payer = authority, bump, space = State::LEN )]
    pub state: Account<'info, State>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct MintWBA<'info> {
    #[account(mut)]
    pub cadet: Signer<'info>,
    #[account(mut, seeds = [b"wba_mint"], bump)]
    pub wba_mint: Account<'info, Mint>,
    #[account(
        init_if_needed,
        payer = cadet,
        associated_token::mint = wba_mint,
        associated_token::authority = cadet,
    )]
    pub cadet_ata: Account<'info, TokenAccount>,
    #[account(mut, seeds = [b"wba_vault", wba_mint.key().as_ref()], bump)]
    pub wba_vault: SystemAccount<'info>,
    /// CHECK: just signs,
    #[account(seeds = [b"wba_auth"], bump)]
    pub wba_auth: UncheckedAccount<'info>,
    pub state: Account<'info, State>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program:Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Claim<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(mut)]
    pub pay_to: SystemAccount<'info>,
    /// CHECK: no need to check this.
    #[account(seeds = [b"wba_mint"], bump)]
    pub wba_mint: UncheckedAccount<'info>,
    #[account(mut, seeds = [b"wba_vault", wba_mint.key().as_ref()], bump)]
    pub wba_vault: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
}


#[account]
pub struct State {
    pub authority: Pubkey,
    pub price_per_wba: u64,
}

impl State {
    const LEN: usize = 8 + 32 + 8;
}