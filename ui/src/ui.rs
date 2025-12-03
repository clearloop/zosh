//! UI types for the Zosh UI

use crate::util;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use zcore::{ex::BridgeBundle, Block, Hash};

/// UI representation of a block
#[derive(Debug, Serialize, Deserialize)]
pub struct UIBlock {
    pub slot: u32,
    pub hash: String,
    pub parent: String,
    pub state: String,
    pub accumulator: String,
    pub extrinsic_root: String,
    pub votes: BTreeMap<String, String>,
    pub extrinsic: UIExtrinsic,
}

impl UIBlock {
    pub fn from_block(block: &Block) -> Self {
        let hash = block.header.hash();
        Self {
            slot: block.header.slot,
            hash: bs58::encode(hash).into_string(),
            parent: bs58::encode(block.header.parent).into_string(),
            state: bs58::encode(block.header.state).into_string(),
            accumulator: bs58::encode(block.header.accumulator).into_string(),
            extrinsic_root: bs58::encode(block.header.extrinsic).into_string(),
            votes: block
                .header
                .votes
                .iter()
                .map(|(k, v)| (bs58::encode(k).into_string(), bs58::encode(v).into_string()))
                .collect(),
            extrinsic: UIExtrinsic::from_extrinsic(
                &block.extrinsic.bridge,
                &block.extrinsic.receipts,
            ),
        }
    }
}

/// UI representation of extrinsic data
#[derive(Debug, Serialize, Deserialize)]
pub struct UIExtrinsic {
    pub bridge: Vec<UIBridgeBundle>,
    pub receipts: Vec<UIReceipt>,
}

impl UIExtrinsic {
    pub fn from_extrinsic(
        bridge: &BTreeMap<Hash, BridgeBundle>,
        receipts: &[zcore::ex::Receipt],
    ) -> Self {
        Self {
            bridge: bridge
                .iter()
                .map(|(hash, bundle)| UIBridgeBundle::from_bundle(hash, bundle))
                .collect(),
            receipts: receipts.iter().map(UIReceipt::from_receipt).collect(),
        }
    }
}

/// UI representation of a bridge bundle
#[derive(Debug, Serialize, Deserialize)]
pub struct UIBridgeBundle {
    pub hash: String,
    pub target: String,
    pub bridges: Vec<UIBridge>,
    pub data: String,
    pub signatures: Vec<String>,
}

impl UIBridgeBundle {
    pub fn from_bundle(hash: &Hash, bundle: &BridgeBundle) -> Self {
        Self {
            hash: bs58::encode(hash).into_string(),
            target: format!("{:?}", bundle.target),
            bridges: bundle.bridge.iter().map(UIBridge::from_bridge).collect(),
            data: bs58::encode(&bundle.data).into_string(),
            signatures: bundle
                .signatures
                .iter()
                .map(|s| bs58::encode(s).into_string())
                .collect(),
        }
    }
}

/// UI representation of a bridge transaction
#[derive(Debug, Serialize, Deserialize)]
pub struct UIBridge {
    pub coin: String,
    pub recipient: String,
    pub amount: u64,
    pub source: String,
    pub target: String,
    pub txid: String,
}

impl UIBridge {
    pub fn from_bridge(bridge: &zcore::ex::Bridge) -> Self {
        Self {
            coin: format!("{:?}", bridge.coin),
            recipient: util::encode_recipient(&bridge.recipient),
            amount: bridge.amount,
            source: format!("{:?}", bridge.source),
            target: format!("{:?}", bridge.target),
            txid: util::encode_txid(&bridge.txid),
        }
    }
}

/// UI representation of a receipt
#[derive(Debug, Serialize, Deserialize)]
pub struct UIReceipt {
    pub anchor: String,
    pub coin: String,
    pub txid: String,
    pub source: String,
    pub target: String,
}

impl UIReceipt {
    pub fn from_receipt(receipt: &zcore::ex::Receipt) -> Self {
        Self {
            anchor: util::encode_txid(&receipt.anchor),
            coin: format!("{:?}", receipt.coin),
            txid: util::encode_txid(&receipt.txid),
            source: format!("{:?}", receipt.source),
            target: format!("{:?}", receipt.target),
        }
    }
}

/// UI representation of a block head with transaction count
#[derive(Debug, Serialize, Deserialize)]
pub struct UIHead {
    pub slot: u32,
    pub hash: String,
    pub txns: u32,
}

/// UI representation of paginated blocks response
#[derive(Debug, Serialize, Deserialize)]
pub struct UIBlocksPage {
    pub blocks: Vec<UIHead>,
    pub total: u32,
    pub page: u32,
    pub row: u32,
}

/// UI representation of a transaction with optional receipt
#[derive(Debug, Serialize, Deserialize)]
pub struct UITxn {
    pub txid: String,
    pub coin: String,
    pub amount: u64,
    pub recipient: String,
    pub source: String,
    pub target: String,
    pub slot: u32,
    pub receipt: Option<UIReceipt>,
}

/// UI representation of paginated transactions response
#[derive(Debug, Serialize, Deserialize)]
pub struct UITxnsPage {
    pub txns: Vec<UITxn>,
    pub total: u32,
    pub page: u32,
    pub row: u32,
}
