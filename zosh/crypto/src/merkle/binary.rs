//! Binary merkle tree

/// Compute the root of a Merkle tree from chunks using Blake2b.
pub fn root(chunks: Vec<Vec<u8>>) -> [u8; 32] {
    hroot(chunks, crate::blake3)
}

/// Compute the root of a Merkle tree from hashes.
pub fn hroot(hashes: Vec<Vec<u8>>, hash: fn(&[u8]) -> [u8; 32]) -> [u8; 32] {
    if hashes.is_empty() {
        return [0u8; 32];
    }

    let tree = tree(hashes.to_vec(), hash);
    let mut root = [0u8; 32];
    root.copy_from_slice(&tree[tree.len() - 1][0]);
    root
}

/// Compute the Merkle tree.
pub fn tree(leaves: Vec<Vec<u8>>, hash: fn(&[u8]) -> [u8; 32]) -> Vec<Vec<Vec<u8>>> {
    if leaves.is_empty() {
        return vec![vec![vec![0u8; 32]]];
    }

    if leaves.len() == 1 {
        return vec![vec![hash(&leaves[0]).to_vec()]];
    }

    // pad leaves
    let mut tree = Vec::new();
    let mut current = leaves;

    // build layers until we reach the root.
    loop {
        let mut layer = Vec::new();
        for i in (0..current.len()).step_by(2) {
            let left = &current[i];
            if let Some(right) = current.get(i + 1) {
                layer.push(hash(&[b"node", left.as_slice(), right.as_slice()].concat()).to_vec());
            } else {
                layer.push(left.clone());
            }
        }

        tree.push(layer.clone());
        if layer.len() == 1 {
            break;
        }

        current = layer;
    }

    tree
}
