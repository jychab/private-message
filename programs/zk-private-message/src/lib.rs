use anchor_lang::prelude::*;

declare_id!("8UWBhz9UzdgJnMCMCJfR1H1UHwJZ7V9TMTwUi9nVWMF4");

#[program]
pub mod zk_private_message {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
