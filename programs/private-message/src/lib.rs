use account_compression::program::AccountCompression;
use anchor_lang::prelude::*;
use anchor_lang::solana_program::program_pack::IsInitialized;
use light_system_program::program::LightSystemProgram;
use light_system_program::InstructionDataInvoke;

declare_id!("8UWBhz9UzdgJnMCMCJfR1H1UHwJZ7V9TMTwUi9nVWMF4");

#[program]
pub mod zk_private_message {

    use super::*;

    pub fn send<'info>(
        ctx: Context<'_,'_,'_,'info,SendCtx<'info>>,
        recipient: Pubkey,
        data: InstructionDataInvoke,
    ) -> Result<()> {
        let inbox = &mut ctx.accounts.inbox;
        if inbox.is_initialized {
            inbox.number_of_messages += 1;
        }else{
            inbox.bump = ctx.bumps.inbox;
            inbox.number_of_messages = 0;
            inbox.address = recipient;
            inbox.is_initialized = true;
        }

       if data.new_address_params.len() != 1 ||
       data.new_address_params[0].seed != Pubkey::find_program_address(&[recipient.as_ref(), inbox.number_of_messages.to_le_bytes().as_ref()],ctx.program_id).0.to_bytes() {
            return Err(CustomError::RecipientMismatch.into())
       }

       let mut bytes_data:Vec<u8> = Vec::new();
       data.serialize(&mut bytes_data).unwrap();

        light_system_program::cpi::invoke(
            CpiContext::new(
                ctx.accounts.light_system_program.to_account_info(),
                light_system_program::cpi::accounts::InvokeInstruction{ 
                    fee_payer: ctx.accounts.signer.to_account_info(), 
                    authority: ctx.accounts.signer.to_account_info(), 
                    registered_program_pda: ctx.accounts.registered_program_pda.to_account_info(), 
                    noop_program: ctx.accounts.noop_program.to_account_info(), 
                    account_compression_authority: ctx.accounts.account_compression_authority.to_account_info(), 
                    account_compression_program: ctx.accounts.account_compression.to_account_info(), 
                    compressed_sol_pda: None, 
                    compression_recipient: None, 
                    system_program: ctx.accounts.system_program.to_account_info()
                }
                ).with_remaining_accounts(ctx.remaining_accounts.to_vec()), bytes_data)?;
        
        Ok(())
    }
}

#[account]
pub struct Inbox {
    pub is_initialized: bool,
    pub address: Pubkey,
    pub bump: u8,
    pub number_of_messages: u64,
}
pub const INBOX_PREFIX: &str = "inbox";
pub const INBOX_SIZE: usize = std::mem::size_of::<Inbox>() + 8;

impl IsInitialized for Inbox {
    fn is_initialized(&self) -> bool {
       self.is_initialized
    }
}

/// ecc25519 keys [ecc25519_private_key, ecc25519_public_key] are created using ed25519 private keys as seed
/// We then generate a shared secret key using [ecc25519_private_key, ecc25519_public_key]
/// message is then encrypted using AES-GCM with shared_secret_key & iv 
#[account]
pub struct MessageData {
    pub sender_ecc25519_public_key: Pubkey,
    pub recipient_ecc25519_public_key: Pubkey,
    pub iv: [u8;12],
    pub encrypted_message: Vec<u8>,
}

#[derive(Accounts)]
#[instruction(recipient:Pubkey)]
pub struct SendCtx<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        init_if_needed, 
        payer=signer, 
        space=INBOX_SIZE, 
        seeds=[INBOX_PREFIX.as_bytes(),recipient.key().as_ref()] ,
        bump
    )]
    pub inbox: Account<'info, Inbox>,
    
    /// CHECK:
    pub registered_program_pda: AccountInfo<'info>,
    /// CHECK:
    pub noop_program: AccountInfo<'info>,
    /// CHECK:
    pub account_compression_authority: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
    pub light_system_program: Program<'info, LightSystemProgram>,
    pub account_compression: Program<'info, AccountCompression>
}

#[error_code]
pub enum CustomError {
    #[msg("Recipient account does not match message recipient")]
    RecipientMismatch,
}


