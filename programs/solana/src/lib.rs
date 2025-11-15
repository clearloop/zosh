use anchor_lang::prelude::*;

declare_id!("2KwobV7wjmUzGRQfpd3G5HVRfCRUXfry9MoM3Hbks9dz");

#[program]
pub mod solana {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
