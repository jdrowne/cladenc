use crate::file_system::tests::double::Memory;
use crate::journal::{self, JournalError};

pub(super) const RUN: &str = concat!(
    r#"{"record":"run","format":36,"primogenitor":"/abs/harness","model":"a-model","#,
    r#""model_url":"http://models.example/v1","model_key_file":null,"docker_image":"image","#,
    r#""dockerfile":null,"docker_build_timeout":null,"#,
    r#""environment_image_repository":"cladenc-environments-0123456789abcdef","#,
    r#""harness_build_command":null,"harness_run_command":"run a harness","#,
    r#""datasets":[{"directory":"/abs/dataset","tasks":["t1","t2","t3"]}],"#,
    r#""selection_tasks_percentage":25.0,"holdout_tasks":["dataset/t2","dataset/t3"],"#,
    r#""corroboration_tasks_percentage":25.0,"corroboration_tasks":["dataset/t3"],"#,
    r#""max_diversifications":800,"#,
    r#""max_concurrent_sandboxes":8,"#,
    r#""diversification_timeout":3600,"max_verifier_health_check_retries":3,"#,
    r#""max_task_health_check_retries":2,"seed":7}"#
);

pub(super) fn error_of_text(text: &str) -> JournalError {
    let directory = Memory::default();
    directory
        .write(journal::path(directory.path()), text)
        .unwrap();
    journal::read(&directory, directory.path()).unwrap_err()
}

pub(super) fn error_of(lines: &[&str]) -> JournalError {
    error_of_text(
        &lines
            .iter()
            .map(|line| format!("{line}\n"))
            .collect::<String>(),
    )
}
