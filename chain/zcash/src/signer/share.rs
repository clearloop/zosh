//! Share of a ZypherBridge client

use reddsa::frost::redpallas::{
    keys::{PublicKeyPackage, SecretShare},
    Identifier,
};

/// Zcash signer of a ZypherBridge client
#[derive(Debug)]
pub struct ShareSigner {
    /// frost redjubjub identifier
    pub identifier: Identifier,

    /// frost redjubjub public key package
    pub rjpackage: PublicKeyPackage,

    /// frost redjubjub secret share
    pub rjshare: SecretShare,
}
