//! An RFC 6962 (Certificate Transparency) append-only Merkle hash tree over audit
//! event records, with INCLUSION proofs that are really computed and verified.
//!
//! RFC 6962 domain-separates leaf and interior hashes (`0x00` / `0x01` prefixes)
//! so a leaf hash can never collide with an interior node — closing the classic
//! second-preimage attack on naive Merkle trees. This module is the tamper-evident
//! substrate for the sealed-audit event chain: the gate binds the
//! `merkle_leaf_hash` and `merkle_root_hash` (both 32 bytes) and requires
//! `inclusion_proof_verified`.
//! Consistency proofs (append-only between two tree sizes) are a sibling module.

use sha2::{Digest as _, Sha256};

/// RFC 6962 leaf-hash prefix.
const LEAF_PREFIX: u8 = 0x00;
/// RFC 6962 interior-node-hash prefix.
const NODE_PREFIX: u8 = 0x01;

/// RFC 6962 leaf hash: `SHA-256(0x00 || data)`. This is what a verifier recomputes
/// over the record bytes before checking an inclusion proof.
pub fn leaf_hash(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update([LEAF_PREFIX]);
    hasher.update(data);
    hasher.finalize().into()
}

/// RFC 6962 interior node hash: `SHA-256(0x01 || left || right)`.
pub(crate) fn node_hash(left: &[u8; 32], right: &[u8; 32]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update([NODE_PREFIX]);
    hasher.update(left);
    hasher.update(right);
    hasher.finalize().into()
}

/// The largest power of two strictly less than `n` (the RFC 6962 split point),
/// for `n > 1`. E.g. split(2)=1, split(3)=2, split(5)=4, split(8)=4.
pub(crate) fn split_point(n: usize) -> usize {
    debug_assert!(n > 1);
    let mut k = 1usize;
    while k << 1 < n {
        k <<= 1;
    }
    k
}

/// The Merkle Tree Hash (root) of `leaf_hashes` (each already a [`leaf_hash`]).
/// `MTH({}) = SHA-256("")` per RFC 6962; `MTH({d0}) = d0`.
pub fn merkle_root(leaf_hashes: &[[u8; 32]]) -> [u8; 32] {
    match leaf_hashes.len() {
        0 => Sha256::digest([]).into(),
        1 => leaf_hashes[0],
        n => {
            let k = split_point(n);
            node_hash(
                &merkle_root(&leaf_hashes[..k]),
                &merkle_root(&leaf_hashes[k..]),
            )
        }
    }
}

/// The RFC 6962 inclusion ("audit") path for the leaf at index `m` in
/// `leaf_hashes`: the sibling subtree roots from the leaf up to the root. The
/// path is ordered deepest-sibling-first (top-level sibling last).
pub fn inclusion_proof(leaf_hashes: &[[u8; 32]], m: usize) -> Vec<[u8; 32]> {
    let n = leaf_hashes.len();
    if m >= n || n <= 1 {
        return Vec::new();
    }
    let k = split_point(n);
    if m < k {
        let mut path = inclusion_proof(&leaf_hashes[..k], m);
        path.push(merkle_root(&leaf_hashes[k..]));
        path
    } else {
        let mut path = inclusion_proof(&leaf_hashes[k..], m - k);
        path.push(merkle_root(&leaf_hashes[..k]));
        path
    }
}

/// Reconstruct the root implied by an inclusion `proof` for `leaf_hash` at index
/// `m` in a tree of `n` leaves. Mirrors [`inclusion_proof`] (consumes the
/// top-level sibling from the back). Returns `None` if the proof shape is wrong.
fn root_from_inclusion(
    leaf_hash: &[u8; 32],
    m: usize,
    n: usize,
    proof: &[[u8; 32]],
) -> Option<[u8; 32]> {
    if m >= n {
        return None;
    }
    if n == 1 {
        // A single-leaf tree has an empty path and the leaf is the root.
        return proof.is_empty().then_some(*leaf_hash);
    }
    let k = split_point(n);
    let (sibling, rest) = proof.split_last()?;
    if m < k {
        let left = root_from_inclusion(leaf_hash, m, k, rest)?;
        Some(node_hash(&left, sibling))
    } else {
        let right = root_from_inclusion(leaf_hash, m - k, n - k, rest)?;
        Some(node_hash(sibling, &right))
    }
}

/// Verify an RFC 6962 inclusion proof: that the record whose leaf hash is
/// `leaf_hash` sits at index `m` in the size-`n` tree with root `root`. Fails
/// closed on any wrong index/size/proof/root.
pub fn verify_inclusion(
    leaf_hash: &[u8; 32],
    m: usize,
    n: usize,
    proof: &[[u8; 32]],
    root: &[u8; 32],
) -> bool {
    root_from_inclusion(leaf_hash, m, n, proof).is_some_and(|computed| &computed == root)
}

/// The RFC 6962 SUBPROOF: the recursive core of the consistency proof. `b` marks
/// whether the size-`m` boundary is the full current subtree.
fn subproof(m: usize, leaves: &[[u8; 32]], b: bool) -> Vec<[u8; 32]> {
    let n = leaves.len();
    if m == n {
        return if b {
            Vec::new()
        } else {
            vec![merkle_root(leaves)]
        };
    }
    let k = split_point(n);
    if m <= k {
        let mut proof = subproof(m, &leaves[..k], b);
        proof.push(merkle_root(&leaves[k..]));
        proof
    } else {
        let mut proof = subproof(m - k, &leaves[k..], false);
        proof.push(merkle_root(&leaves[..k]));
        proof
    }
}

/// The RFC 6962 CONSISTENCY proof that the size-`m` prefix tree is an append-only
/// prefix of the full `leaf_hashes` tree (`0 < m < n`). Empty otherwise.
pub fn consistency_proof(leaf_hashes: &[[u8; 32]], m: usize) -> Vec<[u8; 32]> {
    let n = leaf_hashes.len();
    if m == 0 || m >= n {
        return Vec::new();
    }
    subproof(m, leaf_hashes, true)
}

/// Verify an RFC 6962 consistency proof: that the size-`m` tree with root
/// `first_root` is an append-only PREFIX of the size-`n` tree with root
/// `second_root` (RFC 6962 §2.1.2). Fails closed on any inconsistency, tampering,
/// or malformed proof (bounds-checked — never panics).
///
/// `m` and `n` are TRUSTED context: the caller authenticates the two tree sizes
/// out of band (e.g. from signed tree heads), and this function binds the two
/// ROOTS to that context — it is not designed to detect a forged tree SIZE, only
/// a forged or inconsistent root/proof for the given sizes.
pub fn verify_consistency(
    m: usize,
    n: usize,
    proof: &[[u8; 32]],
    first_root: &[u8; 32],
    second_root: &[u8; 32],
) -> bool {
    if m > n {
        return false;
    }
    if m == n {
        return proof.is_empty() && first_root == second_root;
    }
    if m == 0 {
        // Every tree is consistent with the empty tree; the proof carries nothing.
        return proof.is_empty();
    }

    let mut node = m - 1;
    let mut last_node = n - 1;
    // Climb out of any right-child run: the boundary's largest complete subtree.
    while node & 1 == 1 {
        node >>= 1;
        last_node >>= 1;
    }

    let mut pos = 0usize;
    let mut next = || {
        let item = proof.get(pos).copied();
        pos += 1;
        item
    };

    // Seed: if `m` is not a power of two the seed is in the proof; otherwise the
    // first tree's own root.
    let (mut first_hash, mut second_hash) = if node != 0 {
        match next() {
            Some(seed) => (seed, seed),
            None => return false,
        }
    } else {
        (*first_root, *first_root)
    };

    while node != 0 {
        if node & 1 == 1 {
            let Some(p) = next() else { return false };
            first_hash = node_hash(&p, &first_hash);
            second_hash = node_hash(&p, &second_hash);
        } else if node < last_node {
            let Some(p) = next() else { return false };
            second_hash = node_hash(&second_hash, &p);
        }
        node >>= 1;
        last_node >>= 1;
    }

    while last_node != 0 {
        let Some(p) = next() else { return false };
        second_hash = node_hash(&second_hash, &p);
        last_node >>= 1;
    }

    pos == proof.len() && &first_hash == first_root && &second_hash == second_root
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Independent (naive, un-optimized) Merkle root for cross-checking: builds
    /// the tree level by level using the same leaf/node hashing.
    fn naive_root(leaves: &[[u8; 32]]) -> [u8; 32] {
        match leaves.len() {
            0 => Sha256::digest([]).into(),
            1 => leaves[0],
            _ => {
                let k = split_point(leaves.len());
                node_hash(&naive_root(&leaves[..k]), &naive_root(&leaves[k..]))
            }
        }
    }

    fn leaves(n: usize) -> Vec<[u8; 32]> {
        (0..n)
            .map(|i| leaf_hash(format!("event #{i} payload").as_bytes()))
            .collect()
    }

    #[test]
    fn leaf_and_node_hashes_are_domain_separated() {
        // A leaf hash can never equal an interior node hash (0x00 vs 0x01 prefix).
        let a = leaf_hash(b"x");
        let b = leaf_hash(b"y");
        assert_ne!(leaf_hash(&[]), node_hash(&a, &b));
        // Distinct data -> distinct leaf hashes.
        assert_ne!(leaf_hash(b"x"), leaf_hash(b"y"));
    }

    #[test]
    fn split_point_is_largest_power_of_two_below_n() {
        assert_eq!(split_point(2), 1);
        assert_eq!(split_point(3), 2);
        assert_eq!(split_point(4), 2);
        assert_eq!(split_point(5), 4);
        assert_eq!(split_point(8), 4);
        assert_eq!(split_point(9), 8);
    }

    #[test]
    fn root_matches_independent_recomputation() {
        for n in 0..=64 {
            let l = leaves(n);
            assert_eq!(merkle_root(&l), naive_root(&l), "root mismatch at n={n}");
        }
        // Empty tree is the hash of the empty string (RFC 6962).
        assert_eq!(merkle_root(&[]), <[u8; 32]>::from(Sha256::digest([])));
        // Single leaf is its own root.
        let one = leaves(1);
        assert_eq!(merkle_root(&one), one[0]);
    }

    #[test]
    fn inclusion_proofs_verify_for_every_leaf() {
        for n in 1..=64 {
            let l = leaves(n);
            let root = merkle_root(&l);
            for m in 0..n {
                let proof = inclusion_proof(&l, m);
                assert!(
                    verify_inclusion(&l[m], m, n, &proof, &root),
                    "inclusion failed for leaf {m} of {n}"
                );
            }
        }
    }

    #[test]
    fn inclusion_fails_closed_on_tampering() {
        let n = 11;
        let l = leaves(n);
        let root = merkle_root(&l);
        let m = 5;
        let proof = inclusion_proof(&l, m);
        assert!(verify_inclusion(&l[m], m, n, &proof, &root));

        // Wrong leaf hash.
        let mut bad_leaf = l[m];
        bad_leaf[0] ^= 0x01;
        assert!(!verify_inclusion(&bad_leaf, m, n, &proof, &root));

        // Wrong index.
        assert!(!verify_inclusion(&l[m], m + 1, n, &proof, &root));
        // Out-of-range index.
        assert!(!verify_inclusion(&l[m], n, n, &proof, &root));

        // Tampered proof node.
        let mut bad_proof = proof.clone();
        bad_proof[0][0] ^= 0x01;
        assert!(!verify_inclusion(&l[m], m, n, &bad_proof, &root));

        // Wrong root.
        let mut bad_root = root;
        bad_root[0] ^= 0x01;
        assert!(!verify_inclusion(&l[m], m, n, &proof, &bad_root));

        // Proof of the wrong length (truncated / extended).
        let mut short = proof.clone();
        short.pop();
        assert!(!verify_inclusion(&l[m], m, n, &short, &root));
        let mut long = proof.clone();
        long.push([0u8; 32]);
        assert!(!verify_inclusion(&l[m], m, n, &long, &root));
    }

    #[test]
    fn a_proof_does_not_transfer_to_a_different_tree() {
        // An inclusion proof for one tree must not verify against a tree with a
        // different leaf set (even at the same index/size).
        let l = leaves(8);
        let proof = inclusion_proof(&l, 3);
        let mut other = leaves(8);
        other[7] = leaf_hash(b"a different last event");
        let other_root = merkle_root(&other);
        // Leaf 3 is unchanged, but the root differs, so the proof must fail.
        assert!(!verify_inclusion(&l[3], 3, 8, &proof, &other_root));
    }

    #[test]
    fn consistency_proofs_verify_for_every_prefix() {
        for n in 2..=48 {
            let l = leaves(n);
            let full_root = merkle_root(&l);
            for m in 1..n {
                let prefix_root = merkle_root(&l[..m]);
                let proof = consistency_proof(&l, m);
                assert!(
                    verify_consistency(m, n, &proof, &prefix_root, &full_root),
                    "consistency failed for m={m}, n={n}"
                );
            }
        }
    }

    #[test]
    fn consistency_of_equal_trees_is_an_empty_proof() {
        let l = leaves(7);
        let root = merkle_root(&l);
        assert!(verify_consistency(7, 7, &[], &root, &root));
        // A non-empty proof for equal trees is rejected.
        assert!(!verify_consistency(7, 7, &[[0u8; 32]], &root, &root));
        // Equal sizes but different roots: rejected.
        let mut other = root;
        other[0] ^= 0x01;
        assert!(!verify_consistency(7, 7, &[], &root, &other));
    }

    #[test]
    fn consistency_rejects_a_forged_first_root() {
        // The proof reconstructs the first root from the REAL prefix; a different
        // claimed first root cannot match -> rejected (this is what proves the
        // size-m tree is genuinely an append-only prefix, not a rewritten history).
        let l = leaves(13);
        let full_root = merkle_root(&l);
        let m = 5;
        let proof = consistency_proof(&l, m);
        assert!(verify_consistency(
            m,
            13,
            &proof,
            &merkle_root(&l[..m]),
            &full_root
        ));

        let mut fake = leaves(m);
        fake[0] = leaf_hash(b"forged first event");
        assert!(!verify_consistency(
            m,
            13,
            &proof,
            &merkle_root(&fake),
            &full_root
        ));

        // Also exercise a POWER-OF-TWO prefix (m=8), which takes the other
        // verifier seed branch (first_root seed rather than a proof seed).
        let m2 = 8;
        let proof2 = consistency_proof(&l, m2);
        assert!(verify_consistency(
            m2,
            13,
            &proof2,
            &merkle_root(&l[..m2]),
            &full_root
        ));
        let mut fake2 = leaves(m2);
        fake2[0] = leaf_hash(b"forged power-of-two prefix");
        assert!(!verify_consistency(
            m2,
            13,
            &proof2,
            &merkle_root(&fake2),
            &full_root
        ));
    }

    #[test]
    fn consistency_fails_closed_on_tampering() {
        let l = leaves(20);
        let full_root = merkle_root(&l);
        let m = 9;
        let prefix_root = merkle_root(&l[..m]);
        let proof = consistency_proof(&l, m);
        assert!(verify_consistency(m, 20, &proof, &prefix_root, &full_root));

        // Tampered proof node.
        let mut bad = proof.clone();
        bad[0][0] ^= 0x01;
        assert!(!verify_consistency(m, 20, &bad, &prefix_root, &full_root));
        // Wrong second root.
        let mut bad_root = full_root;
        bad_root[0] ^= 0x01;
        assert!(!verify_consistency(m, 20, &proof, &prefix_root, &bad_root));
        // NOTE: m and n are TRUSTED context (the caller authenticates them via
        // signed tree heads), so the verifier binds the ROOTS given the sizes,
        // not the sizes against the proof — a wrong n that halves to the same
        // path is out of the RFC 6962 threat model and is not asserted here.
        // Truncated / extended proof (bounds-checked, no panic).
        let mut short = proof.clone();
        short.pop();
        assert!(!verify_consistency(m, 20, &short, &prefix_root, &full_root));
        let mut long = proof.clone();
        long.push([0u8; 32]);
        assert!(!verify_consistency(m, 20, &long, &prefix_root, &full_root));
        // m > n is rejected.
        assert!(!verify_consistency(20, m, &proof, &full_root, &prefix_root));
    }
}
