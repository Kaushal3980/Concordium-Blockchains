use crate::crypto_common::base16_encode_string;
use crate::types::*;
use concordium_rust_sdk::{
    common::{self as crypto_common, types::KeyPair},
    id::{
        constants::{ArCurve, AttributeKind},
        id_proof_types::Statement,
        types::{AccountAddress, AccountCredentialWithoutProofs},
    },
    v2::BlockIdentifier,
};
use log::warn;
use rand::Rng;
use std::convert::Infallible;
use std::time::SystemTime;
use warp::{http::StatusCode, Rejection};

static CHALLENGE_EXPIRY_SECONDS: u64 = 600;
static CLEAN_INTERVAL_SECONDS: u64 = 600;

pub async fn handle_get_challenge(
    state: Server,
    address: AccountAddress,
) -> Result<impl warp::Reply, Rejection> {
    let state = state.clone();
    log::debug!("Parsed statement. Generating challenge");
    match get_challenge_worker(state, address).await {
        Ok(r) => Ok(warp::reply::json(&r)),
        Err(e) => {
            warn!("Request is invalid {:#?}.", e);
            Err(warp::reject::custom(e))
        }
    }
}

/// A common function that produces a challenge and adds it to the state.
async fn get_challenge_worker(
    state: Server,
    address: AccountAddress,
) -> Result<ChallengeResponse, InjectStatementError> {
    let mut challenge = [0u8; 32];
    rand::thread_rng().fill(&mut challenge[..]);
    let mut sm = state
        .challenges
        .lock()
        .map_err(|_| InjectStatementError::LockingError)?;
    log::debug!("Generated challenge: {:?}", challenge);
    let challenge = Challenge(challenge);

    sm.insert(
        base16_encode_string(&challenge.0),
        ChallengeStatus {
            address,
            created_at: SystemTime::now(),
        },
    );
    Ok(ChallengeResponse { challenge })
}


pub async fn handle_provide_proof(
    client: concordium_rust_sdk::v2::Client,
    state: Server,
    statement: Statement<ArCurve, AttributeKind>,
    request: ChallengedProof,
    key_pair: KeyPair,
) -> Result<impl warp::Reply, Rejection> {
    let client = client.clone();
    let state = state.clone();
    let statement = statement.clone();
    match check_proof_worker(client, state, request, statement, key_pair).await {
        Ok(r) => Ok(warp::reply::json(&r)),
        Err(e) => {
            warn!("Request is invalid {:#?}.", e);
            Err(warp::reject::custom(e))
        }
    }
}

/// A common function that validates the cryptographic proofs in the request.
async fn check_proof_worker(
    mut client: concordium_rust_sdk::v2::Client,
    state: Server,
    request: ChallengedProof,
    statement: Statement<ArCurve, AttributeKind>,
    key_pair: KeyPair,
) -> Result<String, InjectStatementError> {
    let status = {
        let challenges = state
            .challenges
            .lock()
            .map_err(|_| InjectStatementError::LockingError)?;

        challenges
            .get(&base16_encode_string(&request.challenge.0))
            .ok_or(InjectStatementError::UnknownSession)?
            .clone()
    };

    let cred_id = request.proof.credential;
    let acc_info = client
        .get_account_info(&status.address.into(), BlockIdentifier::LastFinal)
        .await?;

    // TODO Check remaining credentials
    let credential = acc_info
        .response
        .account_credentials
        .get(&0.into())
        .ok_or(InjectStatementError::Credential)?;

        //The line below  makes sure that the credential sent by the user is the same, as the one that the account has.
    if crypto_common::to_bytes(credential.value.cred_id()) != crypto_common::to_bytes(&cred_id) {
        return Err(InjectStatementError::Credential);
    }

    let commitments = match &credential.value {
        AccountCredentialWithoutProofs::Initial { icdv: _, .. } => {
            return Err(InjectStatementError::NotAllowed);
        }
        AccountCredentialWithoutProofs::Normal { commitments, .. } => commitments,
    };

    let mut challenges = state
        .challenges
        .lock()
        .map_err(|_| InjectStatementError::LockingError)?;

        // we verify the proof with this part and respond back with the result which is the signature
    if statement.verify(
        &request.challenge.0,
        &state.global_context,
        cred_id.as_ref(),
        commitments,
        &request.proof.proof.value, // TODO: Check version.
    ) {
        challenges.remove(&base16_encode_string(&request.challenge.0));
        let sig = key_pair.sign(&acc_info.response.account_address.0);
        Ok(hex::encode_upper(sig.sig))
    } else {
        Err(InjectStatementError::InvalidProofs)
    }
}
