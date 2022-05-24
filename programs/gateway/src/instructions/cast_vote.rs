use crate::error::GatewayError;
use crate::{state::*};
use anchor_lang::prelude::*;
use anchor_lang::Accounts;
use solana_gateway::Gateway;

/// Casts a vote using the voter weight record.
/// This instruction updates VoterWeightRecord which is valid for the current Slot and the target Proposal only
/// and hance the instruction has to be executed inside the same transaction as spl-gov.CastVote
///
/// CastVote is accumulative and can be invoked using several transactions
/// In this scenario only the last CastVote should be bundled with spl-gov.CastVote in the same transaction
/// 
/// NOTE - Gateway: All implementations of this gateway should prevent multiple voting
/// with the same tokens - this is not added by the gateway because it is use-case-specific
/// 
/// CastVote instruction is not directional. It does not record vote choice (ex Yes/No)
/// VoteChoice is recorded by spl-gov in VoteRecord
///
#[derive(Accounts)]
#[instruction(proposal: Pubkey)]
pub struct CastVote<'info> {
    /// The voting registrar
    pub registrar: Account<'info, Registrar>,

    #[account(
    mut,
    constraint = voter_weight_record.realm == registrar.realm
    @ GatewayError::InvalidVoterWeightRecordRealm,

    constraint = voter_weight_record.governing_token_mint == registrar.governing_token_mint
    @ GatewayError::InvalidVoterWeightRecordMint,
    )]
    pub voter_weight_record: Account<'info, VoterWeightRecord>,

    /// The token owner who casts the vote
    #[account(
    address = voter_weight_record.governing_token_owner @ GatewayError::InvalidTokenOwnerForVoterWeightRecord
    )]
    pub governing_token_owner: Signer<'info>,

    /// A gateway token from the gatekeeper network in the registrar.
    /// Proves that the holder is permitted to take an action.
    /// CHECK: Checked in the gateway library.
    #[account()]
    pub gateway_token: UncheckedAccount<'info>,

    /// The account which pays for the transaction
    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

/// Casts vote using a dummy voter weight of 1
pub fn cast_vote<'a, 'b, 'c, 'info>(
    ctx: Context<'a, 'b, 'c, 'info, CastVote<'info>>,
    proposal: Pubkey,
) -> Result<()> {
    // Gateway: Check if the voter has a valid gateway token and fail if not
    Gateway::verify_gateway_token_account_info(
        &ctx.accounts.gateway_token.to_account_info(),
        &ctx.accounts.voter_weight_record.governing_token_owner,
        &ctx.accounts.registrar.gatekeeper_network,
        None
    ).or(Err(error!(GatewayError::InvalidGatewayToken)))?;
    
    let voter_weight = DEFAULT_VOTE_WEIGHT;
    let voter_weight_record = &mut ctx.accounts.voter_weight_record;

    if voter_weight_record.weight_action_target == Some(proposal)
        && voter_weight_record.weight_action == Some(VoterWeightAction::CastVote)
    {
        // If cast_vote is called for the same proposal then we keep accumulating the weight
        // this way cast_vote can be called multiple times in different transactions
        // NOTE - Gateway: All implementations of this gateway should prevent multiple voting
        // with the same tokens - this is not added by the gateway because it is use-case-specific 
        voter_weight_record.voter_weight = voter_weight_record
            .voter_weight
            .checked_add(voter_weight)
            .unwrap();
    } else {
        voter_weight_record.voter_weight = voter_weight;
    }

    // The record is only valid as of the current slot
    voter_weight_record.voter_weight_expiry = Some(Clock::get()?.slot);

    // The record is only valid for casting vote on the given Proposal
    voter_weight_record.weight_action = Some(VoterWeightAction::CastVote);
    voter_weight_record.weight_action_target = Some(proposal);

    Ok(())
}
