use test_inputs::sample;

#[test]
fn a_sample_is_looked_for_among_the_samples_of_its_tier() {
    assert!(sample("e2e", "fake").ends_with("tests/e2e/samples/fake"));
}

#[test]
fn a_sample_of_the_e2e_tier_is_found() {
    assert!(sample("e2e", "fake").is_dir());
}

#[test]
fn a_sample_that_its_tier_does_not_have_is_not_found() {
    assert!(!sample("e2e", "missing").exists());
}

#[test]
fn a_sample_of_a_tier_that_has_none_is_not_found() {
    assert!(!sample("integration", "fake").exists());
}
