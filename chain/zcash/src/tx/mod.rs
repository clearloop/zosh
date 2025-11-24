//! Transaction builder for zcash frost

use anyhow::Result;
use orchard::{
    bundle::Flags, keys::Scope, tree::MerklePath, value::NoteValue, Address, Anchor, Note,
};
use zcash_keys::keys::UnifiedFullViewingKey;
use zcash_primitives::transaction::{TransactionData, TxVersion, Unauthorized};
use zcash_protocol::consensus::{BlockHeight, BranchId};

/// Sign a transaction with a unified full viewing key
pub fn sign(
    ufvk: UnifiedFullViewingKey,
    anchor: Anchor,
    merkle_path: MerklePath,
    note: Note,
    output_addr: Address,
    output_value: u64,
) -> Result<TransactionData<Unauthorized>> {
    let fvk = ufvk
        .orchard()
        .ok_or(anyhow::anyhow!("Invalid orchard full viewing key"))?;

    let ovk = fvk.clone().to_ovk(Scope::External);
    let mut builder = orchard::builder::Builder::new(
        orchard::builder::BundleType::Transactional {
            flags: Flags::ENABLED,
            bundle_required: false,
        },
        anchor,
    );

    builder.add_spend(fvk.clone(), note, merkle_path)?;
    builder.add_output(
        Some(ovk.clone()),
        output_addr,
        NoteValue::from_raw(output_value),
        None,
    )?;

    let (bundle, _) = builder
        .build(rand_core::OsRng)?
        .ok_or(anyhow::anyhow!("Failed to build bundle"))?;

    // create the transaction data
    let branch = BranchId::Nu6;
    let tx = TransactionData::<Unauthorized>::from_parts(
        TxVersion::suggested_for_branch(branch),
        branch,
        0,
        BlockHeight::from_u32(0),
        None,
        None,
        None,
        Some(bundle),
    );
    Ok(tx)
}
