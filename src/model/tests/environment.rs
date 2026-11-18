use crate::model::Model;

fn environment_with(key: Option<&str>) -> Vec<(&str, &str)> {
    let model = Model {
        name: "a-model",
        url: "http://models.example/v1",
        key,
    };
    model.environment().collect()
}

#[test]
fn a_model_with_a_key_is_handed_on_in_three_variables() {
    assert_eq!(
        environment_with(Some("a-made-up-key")),
        [
            ("CLADENC_MODEL", "a-model"),
            ("CLADENC_MODEL_URL", "http://models.example/v1"),
            ("CLADENC_MODEL_KEY", "a-made-up-key"),
        ]
    );
}

#[test]
fn a_model_with_no_key_is_handed_on_in_two_variables() {
    assert_eq!(
        environment_with(None),
        [
            ("CLADENC_MODEL", "a-model"),
            ("CLADENC_MODEL_URL", "http://models.example/v1"),
        ]
    );
}
