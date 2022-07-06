mod program_test;

use anchor_lang::prelude::Pubkey;
use program_test::gateway_voter_test::GatewayVoterTest;

use gpl_civic_gateway::error::GatewayError;
use solana_program::instruction::InstructionError;
use solana_program_test::*;
use solana_sdk::{signature::Keypair, signer::Signer, transport::TransportError};

use crate::program_test::governance_test::RealmCookie;
use crate::program_test::predecessor_plugin_test::PredecessorPluginTest;
use program_test::tools::{assert_anchor_err, assert_gateway_err, assert_ix_err};

#[tokio::test]
async fn test_update_registrar_new_gatekeeper_network() -> Result<(), TransportError> {
    // Arrange
    let mut gateway_voter_test = GatewayVoterTest::start_new().await;

    let (realm_cookie, registrar_cookie, _, _, _) = gateway_voter_test.setup(false).await?;

    let new_gateway_cookie = gateway_voter_test.with_gateway().await?;

    // Act
    gateway_voter_test
        .update_registrar(&realm_cookie, &registrar_cookie, &new_gateway_cookie, None)
        .await?;

    // Assert
    let registrar = gateway_voter_test
        .get_registrar_account(&registrar_cookie.address)
        .await;

    assert_eq!(
        registrar.gatekeeper_network,
        new_gateway_cookie.gatekeeper_network.pubkey()
    );

    Ok(())
}

#[tokio::test]
async fn test_update_registrar_new_predecessor() -> Result<(), TransportError> {
    // Arrange
    let mut gateway_voter_test = GatewayVoterTest::start_new().await;

    let (realm_cookie, registrar_cookie, gateway_cookie, _, _) =
        gateway_voter_test.setup(false).await?;

    // Act
    let predecessor_program_id = PredecessorPluginTest::program_id();
    gateway_voter_test
        .update_registrar(
            &realm_cookie,
            &registrar_cookie,
            &gateway_cookie,
            Some(predecessor_program_id),
        )
        .await?;

    // Assert
    let registrar = gateway_voter_test
        .get_registrar_account(&registrar_cookie.address)
        .await;

    assert_eq!(
        registrar.previous_voting_weight_plugin_program_id,
        Some(predecessor_program_id)
    );

    Ok(())
}

#[tokio::test]
async fn test_update_registrar_with_invalid_realm_authority_error() -> Result<(), TransportError> {
    // Arrange
    let mut gateway_voter_test = GatewayVoterTest::start_new().await;

    let (realm_cookie, registrar_cookie, gateway_cookie, _, _) =
        gateway_voter_test.setup(false).await?;

    let broken_realm_cookie = RealmCookie {
        realm_authority: Keypair::new(),
        ..realm_cookie
    };

    // Act
    let err = gateway_voter_test
        .update_registrar(
            &broken_realm_cookie,
            &registrar_cookie,
            &gateway_cookie,
            None,
        )
        .await
        .err()
        .unwrap();

    // Assert
    assert_gateway_err(err, GatewayError::InvalidRealmAuthority);

    Ok(())
}

#[tokio::test]
async fn test_update_registrar_with_realm_authority_must_sign_error() -> Result<(), TransportError>
{
    // Arrange
    let mut gateway_voter_test = GatewayVoterTest::start_new().await;

    let (realm_cookie, registrar_cookie, gateway_cookie, _, _) =
        gateway_voter_test.setup(false).await?;

    // Act
    let err = gateway_voter_test
        .update_registrar_using_ix(
            &realm_cookie,
            &registrar_cookie,
            &gateway_cookie,
            None,
            |i| i.accounts[3].is_signer = false, // realm_authority
            Some(&[]),
        )
        .await
        .err()
        .unwrap();

    // Assert
    assert_anchor_err(err, anchor_lang::error::ErrorCode::AccountNotSigner);

    Ok(())
}

#[tokio::test]
async fn test_update_registrar_with_invalid_spl_gov_program_id_error() -> Result<(), TransportError>
{
    // Arrange
    let mut gateway_voter_test = GatewayVoterTest::start_new().await;

    let (realm_cookie, registrar_cookie, gateway_cookie, _, _) =
        gateway_voter_test.setup(false).await?;

    // Try to use a different program id
    let governance_program_id = gateway_voter_test.program_id;

    // Act
    let err = gateway_voter_test
        .update_registrar_using_ix(
            &realm_cookie,
            &registrar_cookie,
            &gateway_cookie,
            None,
            |i| i.accounts[1].pubkey = governance_program_id, //governance_program_id
            None,
        )
        .await
        .err()
        .unwrap();

    // Assert
    assert_anchor_err(err, anchor_lang::error::ErrorCode::ConstraintOwner);

    Ok(())
}

#[tokio::test]
async fn test_update_registrar_with_invalid_realm_error() -> Result<(), TransportError> {
    // Arrange
    let mut gateway_voter_test = GatewayVoterTest::start_new().await;

    let (realm_cookie, registrar_cookie, gateway_cookie, _, _) =
        gateway_voter_test.setup(false).await?;

    // Act
    let err = gateway_voter_test
        .update_registrar_using_ix(
            &realm_cookie,
            &registrar_cookie,
            &gateway_cookie,
            None,
            |i| i.accounts[2].pubkey = Pubkey::new_unique(), // realm
            None,
        )
        .await
        .err()
        .unwrap();

    assert_ix_err(err, InstructionError::Custom(2004));

    Ok(())
}
