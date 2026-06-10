//! Stage 6: epoch-batched publishing. Many device-key updates land in ONE epoch
//! (one tree update / one signed tree head), and every key in the batch is
//! independently provable at that single checkpoint.

use mercury_kt::{KeyTransparencyProofStatus as Status, KtDirectory, verify_inclusion};

#[tokio::test]
async fn batch_publish_puts_many_keys_in_one_epoch() {
    let mut dir = KtDirectory::new().await.unwrap();
    let updates = vec![
        ("device:alice:a1".to_string(), b"alice-key".to_vec()),
        ("device:bob:b2".to_string(), b"bob-key".to_vec()),
        ("device:carol:c3".to_string(), b"carol-key".to_vec()),
    ];

    let checkpoint = dir.register_batch(&updates).await.unwrap();
    assert_eq!(
        checkpoint.epoch(),
        1,
        "all three keys land in a single epoch"
    );

    // Every key in the batch is provable at that one checkpoint, against the
    // same pinned VRF key -- i.e. batching did not weaken per-key inclusion.
    let pk = dir.public_key().await.unwrap();
    for (label, key) in &updates {
        let (proof, ckpt) = dir.prove_inclusion(label).await.unwrap();
        assert_eq!(
            verify_inclusion(&pk, &ckpt, label, key, proof),
            Status::Verified,
            "key for {label} must verify at the batched checkpoint"
        );
    }
}

#[tokio::test]
async fn batched_keys_reject_substitution() {
    // Batching must not let a wrong key pass: a value the directory never
    // committed for a batched label is surfaced as Invalid.
    let mut dir = KtDirectory::new().await.unwrap();
    let updates = vec![
        ("device:alice:a1".to_string(), b"alice-key".to_vec()),
        ("device:bob:b2".to_string(), b"bob-key".to_vec()),
    ];
    dir.register_batch(&updates).await.unwrap();
    let pk = dir.public_key().await.unwrap();

    let (proof, ckpt) = dir.prove_inclusion("device:alice:a1").await.unwrap();
    assert_eq!(
        verify_inclusion(&pk, &ckpt, "device:alice:a1", b"attacker-key", proof),
        Status::Invalid,
    );
}

#[tokio::test]
async fn batches_advance_one_epoch_each() {
    // Two batches -> exactly two epochs (not one-per-key): the second batch's
    // checkpoint is epoch 2 regardless of how many keys each batch carried.
    let mut dir = KtDirectory::new().await.unwrap();
    let first = vec![
        ("device:a".to_string(), b"a1".to_vec()),
        ("device:b".to_string(), b"b1".to_vec()),
    ];
    let second = vec![
        ("device:c".to_string(), b"c1".to_vec()),
        ("device:d".to_string(), b"d1".to_vec()),
        ("device:e".to_string(), b"e1".to_vec()),
    ];
    let ep1 = dir.register_batch(&first).await.unwrap();
    let ep2 = dir.register_batch(&second).await.unwrap();
    assert_eq!((ep1.epoch(), ep2.epoch()), (1, 2));
}

#[tokio::test]
async fn empty_batch_is_rejected() {
    // An empty batch must NOT mint an epoch (no key updates -> no new epoch);
    // it fails closed rather than advancing the log with nothing to attest.
    let mut dir = KtDirectory::new().await.unwrap();
    assert!(dir.register_batch(&[]).await.is_err());
}
