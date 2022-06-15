use crate::state::generic_voter_weight::GenericVoterWeight;
use anchor_lang::prelude::*;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use solana_program::program_pack::IsInitialized;

use crate::tools::anchor::{DISCRIMINATOR_SIZE, PUBKEY_SIZE};

/// The default vote weight matches the default decimal places of a governance token
/// so that a single vote using this plugin matches a single vote with a governance token
pub const DEFAULT_VOTE_WEIGHT: u64 = 1000000;

/// VoterWeightAction enum as defined in spl-governance-addin-api
/// It's redefined here for Anchor to export it to IDL
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone, Copy, PartialEq, FromPrimitive)]
pub enum VoterWeightAction {
    /// Cast vote for a proposal. Target: Proposal
    CastVote,

    /// Comment a proposal. Target: Proposal
    CommentProposal,

    /// Create Governance within a realm. Target: Realm
    CreateGovernance,

    /// Create a proposal for a governance. Target: Governance
    CreateProposal,

    /// Signs off a proposal for a governance. Target: Proposal
    /// Note: SignOffProposal is not supported in the current version
    SignOffProposal,
}

/// VoterWeightRecord account as defined in spl-governance-addin-api
/// It's redefined here without account_discriminator for Anchor to treat it as native account
///
/// The account is used as an api interface to provide voting power to the governance program from external addin contracts
#[account]
#[derive(Debug, PartialEq)]
pub struct VoterWeightRecord {
    /// The Realm the VoterWeightRecord belongs to
    pub realm: Pubkey,

    /// Governing Token Mint the VoterWeightRecord is associated with
    /// Note: The addin can take deposits of any tokens and is not restricted to the community or council tokens only
    // The mint here is to link the record to either community or council mint of the realm
    pub governing_token_mint: Pubkey,

    /// The owner of the governing token and voter
    /// This is the actual owner (voter) and corresponds to TokenOwnerRecord.governing_token_owner
    pub governing_token_owner: Pubkey,

    /// Voter's weight
    /// The weight of the voter provided by the addin for the given realm, governing_token_mint and governing_token_owner (voter)
    pub voter_weight: u64,

    /// The slot when the voting weight expires
    /// It should be set to None if the weight never expires
    /// If the voter weight decays with time, for example for time locked based weights, then the expiry must be set
    /// As a common pattern Revise instruction to update the weight should be invoked before governance instruction within the same transaction
    /// and the expiry set to the current slot to provide up to date weight
    pub voter_weight_expiry: Option<u64>,

    /// The governance action the voter's weight pertains to
    /// It allows to provided voter's weight specific to the particular action the weight is evaluated for
    /// When the action is provided then the governance program asserts the executing action is the same as specified by the addin
    pub weight_action: Option<VoterWeightAction>,

    /// The target the voter's weight action pertains to
    /// It allows to provided voter's weight specific to the target the weight is evaluated for
    /// For example when addin supplies weight to vote on a particular proposal then it must specify the proposal as the action target
    /// When the target is provided then the governance program asserts the target is the same as specified by the addin
    pub weight_action_target: Option<Pubkey>,

    /// Reserved space for future versions
    pub reserved: [u8; 8],
}

impl VoterWeightRecord {
    pub fn get_space() -> usize {
        DISCRIMINATOR_SIZE + PUBKEY_SIZE * 4 + 8 + 1 + 8 + 1 + 1 + 1 + 8
    }
}

impl Default for VoterWeightRecord {
    fn default() -> Self {
        Self {
            realm: Default::default(),
            governing_token_mint: Default::default(),
            governing_token_owner: Default::default(),
            voter_weight: Default::default(),
            voter_weight_expiry: Some(0),
            weight_action: Some(VoterWeightAction::CastVote),
            weight_action_target: Some(Default::default()),
            reserved: Default::default(),
        }
    }
}

impl IsInitialized for VoterWeightRecord {
    fn is_initialized(&self) -> bool {
        self.realm != Default::default()
            && self.governing_token_mint != Default::default()
            && self.governing_token_owner != Default::default()
    }
}

impl GenericVoterWeight for VoterWeightRecord {
    fn get_governing_token_mint(&self) -> Pubkey {
        self.governing_token_mint
    }

    fn get_governing_token_owner(&self) -> Pubkey {
        self.governing_token_owner
    }

    fn get_realm(&self) -> Pubkey {
        self.realm
    }

    fn get_voter_weight(&self) -> u64 {
        self.voter_weight
    }

    fn get_weight_action(&self) -> Option<VoterWeightAction> {
        self.weight_action.clone()
    }

    fn get_weight_action_target(&self) -> Option<Pubkey> {
        self.weight_action_target
    }

    fn get_vote_expiry(&self) -> Option<u64> {
        self.voter_weight_expiry
    }
}

// the "official" on-chain voter weight record has a discriminator field
// when a predecessor voter weight is provided, it uses this struct instead of the one defined above.
// We add the GenericVoterWeight trait here to hide this from the rest of the code.
impl GenericVoterWeight for spl_governance_addin_api::voter_weight::VoterWeightRecord {
    fn get_governing_token_mint(&self) -> Pubkey {
        self.governing_token_mint
    }

    fn get_governing_token_owner(&self) -> Pubkey {
        self.governing_token_owner
    }

    fn get_realm(&self) -> Pubkey {
        self.realm
    }

    fn get_voter_weight(&self) -> u64 {
        self.voter_weight
    }

    // The GenericVoterWeight interface expects a crate-defined VoterWeightAction.
    // This is identical to spl_governance_addin_api::voter_weight::VoterWeightAction, but added here
    // so that Anchor will create the mapping correctly in the IDL.
    // This function converts the spl_governance_addin_api::voter_weight::VoterWeightAction to the
    // crate-defined VoterWeightAction by mapping the enum values by integer.
    // Note - it is imperative that the two enums stay in sync to avoid errors here.
    fn get_weight_action(&self) -> Option<VoterWeightAction> {
        self.weight_action.clone().map(|x| FromPrimitive::from_u32(x as u32).unwrap() )
    }

    fn get_weight_action_target(&self) -> Option<Pubkey> {
        self.weight_action_target
    }

    fn get_vote_expiry(&self) -> Option<u64> {
        self.voter_weight_expiry
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_get_space() {
        // Arrange
        let expected_space = VoterWeightRecord::get_space();

        // Act
        let actual_space =
            DISCRIMINATOR_SIZE + VoterWeightRecord::default().try_to_vec().unwrap().len();

        // Assert
        assert_eq!(expected_space, actual_space);
    }
}
