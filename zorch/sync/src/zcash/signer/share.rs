//! Share of a ZorchBridge client

use reddsa::frost::redpallas::{
    keys::{PublicKeyPackage, SecretShare},
    Identifier,
};
use serde::{Deserialize, Serialize};

/// Zcash signer of a ZorchBridge client
///
/// TODO: add random sk here
#[derive(Debug, Serialize, Deserialize)]
pub struct ShareSigner {
    /// frost redjubjub identifier
    pub identifier: Identifier,

    /// frost redjubjub public key package
    pub rjpackage: PublicKeyPackage,

    /// frost redjubjub secret share
    pub rjshare: SecretShare,
}
