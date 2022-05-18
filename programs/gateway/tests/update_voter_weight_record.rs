use gpl_gateway::error::GatewayError;
use gpl_gateway::state::*;
use program_test::gateway_voter_test::GatewayVoterTest;
use program_test::tools::*;
use solana_program_test::*;
use solana_sdk::transport::TransportError;

mod program_test;

#[tokio::test]
async fn test_update_voter_weight_record() -> Result<(), TransportError> {
    // Arrange
    let mut gateway_voter_test = GatewayVoterTest::start_new().await;

    let realm_cookie = gateway_voter_test.governance.with_realm().await?;
    let gateway_cookie = gateway_voter_test.with_gateway().await?;
    let registrar_cookie = gateway_voter_test.with_registrar(&realm_cookie, &gateway_cookie).await?;

    gateway_voter_test
        .with_max_voter_weight_record(&registrar_cookie)
        .await?;

    let voter_cookie = gateway_voter_test.bench.with_wallet().await;
    let gateway_token_cookie = gateway_voter_test.with_gateway_token(&gateway_cookie, &voter_cookie).await?;

    let mut voter_weight_record_cookie = gateway_voter_test
        .with_voter_weight_record(&registrar_cookie, &voter_cookie)
        .await?;

    gateway_voter_test.bench.advance_clock().await;
    let clock = gateway_voter_test.bench.get_clock().await;

    // Act
    gateway_voter_test
        .update_voter_weight_record(
            &registrar_cookie,
            &mut voter_weight_record_cookie,
            &gateway_token_cookie,
            VoterWeightAction::CreateProposal,
        )
        .await?;

    // Assert

    let voter_weight_record = gateway_voter_test
        .get_voter_weight_record(&voter_weight_record_cookie.address)
        .await;

    assert_eq!(voter_weight_record.voter_weight, 1);
    assert_eq!(voter_weight_record.voter_weight_expiry, Some(clock.slot));
    assert_eq!(
        voter_weight_record.weight_action,
        Some(VoterWeightAction::CreateProposal.into())
    );
    assert_eq!(voter_weight_record.weight_action_target, None);

    Ok(())
}

#[tokio::test]
async fn test_update_voter_weight_with_cast_vote_not_allowed_error() -> Result<(), TransportError> {
    // Arrange
    let mut gateway_voter_test = GatewayVoterTest::start_new().await;

    let realm_cookie = gateway_voter_test.governance.with_realm().await?;
    let gateway_cookie = gateway_voter_test.with_gateway().await?;

    let registrar_cookie = gateway_voter_test.with_registrar(&realm_cookie, &gateway_cookie).await?;

    gateway_voter_test
        .with_max_voter_weight_record(&registrar_cookie)
        .await?;

    let voter_cookie = gateway_voter_test.bench.with_wallet().await;
    let gateway_token_cookie = gateway_voter_test.with_gateway_token(&gateway_cookie, &voter_cookie).await?;

    let mut voter_weight_record_cookie = gateway_voter_test
        .with_voter_weight_record(&registrar_cookie, &voter_cookie)
        .await?;

    // Act
    let err = gateway_voter_test
        .update_voter_weight_record(
            &registrar_cookie,
            &mut voter_weight_record_cookie,
            &gateway_token_cookie,
            VoterWeightAction::CastVote,
        )
        .await
        .err()
        .unwrap();

    // Assert
    assert_gateway_err(err, GatewayError::CastVoteIsNotAllowed);

    Ok(())
}