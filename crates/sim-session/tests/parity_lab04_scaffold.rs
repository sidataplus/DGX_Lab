//! Native parity harness for Lab 04.

use sim_session::{digest_for_transcript, lab04_completes};

#[test]
fn canonical_sinfo_squeue_digest_is_stable() {
    let commands = ["sinfo", "squeue"];
    let first = digest_for_transcript("dgx-h200-8", 42, &commands).expect("first");
    let second = digest_for_transcript("dgx-h200-8", 42, &commands).expect("second");
    assert_eq!(first, second);
    assert_eq!(first.len(), 64, "sha256 hex digest");
}

#[test]
fn lab04_full_transcript_is_deterministic() {
    let (complete_a, percent_a, digest_a) = lab04_completes(42).expect("a");
    let (complete_b, percent_b, digest_b) = lab04_completes(42).expect("b");
    assert!(complete_a && complete_b);
    assert_eq!(percent_a, percent_b);
    assert!(percent_a >= 80);
    assert_eq!(digest_a, digest_b);
    assert_eq!(digest_a.len(), 64);
}
