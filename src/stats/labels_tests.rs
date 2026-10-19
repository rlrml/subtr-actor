use super::*;

#[test]
fn repeated_label_deserialization_reuses_string_allocations() {
    let json = r#"{"key":"deserialized-label-key","value":"deserialized-label-value"}"#;
    let first: StatLabel = serde_json::from_str(json).unwrap();
    let second: StatLabel = serde_json::from_str(json).unwrap();
    assert_eq!(first, second);
    assert!(std::ptr::eq(first.key, second.key));
    assert!(std::ptr::eq(first.value, second.value));
    assert_eq!(serde_json::to_string(&first).unwrap(), json);
}

#[test]
fn concurrent_label_deserialization_shares_the_same_intern_pool() {
    let labels: Vec<_> = std::thread::scope(|scope| {
        let threads: Vec<_> = (0..8)
            .map(|_| {
                scope.spawn(|| {
                    serde_json::from_str::<StatLabel>(
                        r#"{"key":"concurrent-shared-label","value":"concurrent-shared-label"}"#,
                    )
                    .unwrap()
                })
            })
            .collect();
        threads
            .into_iter()
            .map(|thread| thread.join().unwrap())
            .collect()
    });
    for label in &labels {
        assert!(std::ptr::eq(label.key, labels[0].key));
        assert!(std::ptr::eq(label.key, label.value));
    }
}
