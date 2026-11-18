use std::process::{self, Command};

use test_inputs::{IMAGE, image};

fn tag(name: &str) {
    Command::new("docker")
        .args(["tag", IMAGE, name])
        .output()
        .expect("docker can be called");
}

fn is_there(name: &str) -> bool {
    Command::new("docker")
        .args(["image", "inspect", name])
        .output()
        .expect("docker can be called")
        .status
        .success()
}

#[test]
fn an_image_is_named_for_the_tests_of_cladenc_and_for_the_process_that_asks() {
    let named = format!("cladenc-test-{}-", process::id());
    assert!(image().name.starts_with(&named));
}

#[test]
fn two_images_have_two_names() {
    assert_ne!(image().name, image().name);
}

#[test]
fn an_image_is_there_for_as_long_as_it_is_held() {
    let held = image();
    tag(&held.name);
    assert!(is_there(&held.name));
}

#[test]
fn an_image_is_removed_once_it_is_let_go() {
    let held = image();
    let name = held.name.clone();
    tag(&name);
    drop(held);
    assert!(!is_there(&name));
}
