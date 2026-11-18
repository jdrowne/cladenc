use test_inputs::sample;

#[test]
fn a_sample_is_looked_for_among_the_samples_of_its_tier() {
    assert!(sample("unit", "Dockerfile").ends_with("test/unit/samples/Dockerfile"));
}

#[test]
fn a_sample_of_the_unit_tier_is_found() {
    assert!(sample("unit", "Dockerfile").is_file());
}

#[test]
fn a_sample_of_the_e2e_tier_is_found() {
    assert!(sample("e2e", "fake").is_dir());
}

#[test]
fn a_sample_that_no_tier_has_is_not_found() {
    assert!(!sample("unit", "missing").exists());
}
