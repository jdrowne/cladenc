use crate::containers::ContainerError;
use crate::containers::tests::double::{Answer, Double};
use crate::directed_evolution::DirectedEvolutionError;
use crate::directed_evolution::tests::given::{self, Evolved};
use crate::file_system::tests::double::Memory;
use crate::journal::{self, Settings};

const ENVIRONMENTS_BUILT: [&str; 2] = [
    "docker build --tag cladenc-environments-0123456789abcdef:0 --label \
     cladenc.task=benchmark/t1 --label cladenc.directed-evolution=OUTPUTS \
     OUTPUTS/benchmarks/benchmark/t1/environment",
    "docker build --tag cladenc-environments-0123456789abcdef:1 --label \
     cladenc.task=benchmark/t2 --label cladenc.directed-evolution=OUTPUTS \
     OUTPUTS/benchmarks/benchmark/t2/environment",
];

const GRADER_BUILT: &str = "docker build --tag cladenc-environments-0123456789abcdef:0-grader \
                            --label cladenc.task=benchmark/t1 --label \
                            cladenc.directed-evolution=OUTPUTS \
                            OUTPUTS/benchmarks/benchmark/t1/tests";

const REFUSED: Answer = Answer::Refused("docker is not there");
const MACHINE: Answer = Answer::Printed("8 16777216000\n");
const IMAGE_GIVEN: &str = "[environment]\ndocker_image = \"an-image:given\"\n";
const TWO_CPUS: &str = "[environment]\ncpus = 2\n";
const GRADER_GIVEN: &str = "[verifier.environment]\ndocker_image = \"a-grader:given\"\n";
const SEPARATE: &str = "[verifier]\nenvironment_mode = \"separate\"\n";

fn of_t1(configuration: &'static str) -> impl FnOnce(&Memory, &mut Settings) {
    move |directory, _| given::configured(directory, "t1", configuration)
}

fn of_each_task(configuration: &'static str) -> impl FnOnce(&Memory, &mut Settings) {
    move |directory, _| {
        given::configured(directory, "t1", configuration);
        given::configured(directory, "t2", configuration);
    }
}

fn with_a_dockerfile_of_the_grader(directory: &Memory, _: &mut Settings) {
    given::configured(directory, "t1", SEPARATE);
    directory
        .write(
            directory.path().join("benchmark/t1/tests/Dockerfile"),
            "FROM scratch\n",
        )
        .unwrap();
}

fn made() -> Evolved {
    given::started(Double::default(), given::unchanged)
}

fn not_built() -> Evolved {
    let docker = Double::default().answering(&["build"], &[Answer::Exited(1)]);
    given::started(docker, given::unchanged)
}

fn on_a_machine(configuration: &'static str) -> Evolved {
    let docker = Double::default().answering(&["info"], &[MACHINE]);
    given::started(docker, of_t1(configuration))
}

fn grader_not_built() -> Evolved {
    let docker = Double::default().answering(&["build", "grader"], &[Answer::Exited(1)]);
    given::started(docker, with_a_dockerfile_of_the_grader)
}

#[test]
fn the_environment_of_each_task_is_built_before_anything_else() {
    assert_eq!(made().commands()[..2], ENVIRONMENTS_BUILT);
}

#[test]
fn the_environments_that_were_built_are_told() {
    let said = made().said;
    assert!(said.contains("built the environments of the 2 tasks"));
}

#[test]
fn an_environment_whose_build_fails_ends_the_directed_evolution_with_the_cause() {
    assert!(matches!(
        not_built().result,
        Err(DirectedEvolutionError::Environment {
            task,
            error: ContainerError::Exit(1),
        }) if task == "benchmark/t1"
    ));
}

#[test]
fn an_environment_whose_build_fails_is_told_with_the_cause() {
    let said = not_built().said;
    assert!(
        said.contains("the build of the environment of task benchmark/t1 exited with status 1")
    );
}

#[test]
fn the_images_of_the_environments_are_removed_when_one_of_them_cannot_be_built() {
    assert_eq!(not_built().count_of("docker image rm --force"), 1);
}

#[test]
fn a_directed_evolution_that_is_stopped_while_an_environment_is_built_has_not_started() {
    let docker = Double::default().answering(&["build"], &[Answer::Stopped]);
    let stopped = given::started(docker, given::unchanged);
    assert!(matches!(
        stopped.result,
        Err(DirectedEvolutionError::NotStarted)
    ));
}

#[test]
fn a_task_whose_configuration_is_not_valid_ends_the_directed_evolution() {
    let evolved = given::started(Double::default(), of_t1("not a table"));
    assert!(matches!(
        evolved.result,
        Err(DirectedEvolutionError::Configuration { task, .. }) if task == "benchmark/t1"
    ));
}

#[test]
fn an_image_that_a_task_names_as_its_environment_is_looked_for_on_the_machine() {
    let evolved = given::started(Double::default(), of_t1(IMAGE_GIVEN));
    assert_eq!(
        evolved.count_of("docker image inspect --format {{.Id}} an-image:given"),
        1
    );
}

#[test]
fn an_image_that_a_task_names_as_its_environment_is_told() {
    let said = given::started(Double::default(), of_t1(IMAGE_GIVEN)).said;
    assert!(
        said.contains("taking the image an-image:given as the environment of task benchmark/t1")
    );
}

#[test]
fn an_image_that_a_task_names_and_the_machine_lacks_ends_the_directed_evolution() {
    let docker = Double::default().answering(&["image", "inspect"], &[REFUSED]);
    let evolved = given::started(docker, of_t1(IMAGE_GIVEN));
    assert!(matches!(
        evolved.result,
        Err(DirectedEvolutionError::NotOnTheMachine { task, image, .. })
            if task == "benchmark/t1" && image == "an-image:given"
    ));
}

#[test]
fn no_environment_is_built_for_a_task_that_names_its_image() {
    let commands = given::started(Double::default(), of_t1(IMAGE_GIVEN)).commands();
    let built: Vec<&str> = commands
        .iter()
        .filter(|command| command.starts_with("docker build "))
        .map(String::as_str)
        .collect();
    assert_eq!(built, [ENVIRONMENTS_BUILT[1]]);
}

#[test]
fn no_environment_is_told_as_built_when_every_task_names_an_image() {
    let said = given::started(Double::default(), of_each_task(IMAGE_GIVEN)).said;
    assert!(!said.contains("built the environment"));
}

#[test]
fn the_machine_is_asked_once_what_it_has_whatever_the_number_of_tasks_that_ask_for_some() {
    let docker = Double::default().answering(&["info"], &[MACHINE]);
    let evolved = given::started(docker, of_each_task(TWO_CPUS));
    assert_eq!(evolved.count_of("docker info"), 1);
}

#[test]
fn a_task_that_asks_for_what_the_machine_has_is_attempted() {
    let best = on_a_machine(TWO_CPUS).result.ok();
    assert_eq!(best.map(|found| found.best_harness), Some(0));
}

#[test]
fn a_task_that_asks_for_more_processors_than_the_machine_has_ends_the_directed_evolution() {
    assert!(matches!(
        on_a_machine("[environment]\ncpus = 16\n").result,
        Err(DirectedEvolutionError::TooLarge { key: "cpus", .. })
    ));
}

#[test]
fn a_task_that_asks_for_more_memory_than_the_machine_has_ends_the_directed_evolution() {
    assert!(matches!(
        on_a_machine("[environment]\nmemory_mb = 32000\n").result,
        Err(DirectedEvolutionError::TooLarge {
            key: "memory_mb",
            ..
        })
    ));
}

#[test]
fn a_machine_that_cannot_be_asked_what_it_has_ends_the_directed_evolution() {
    let docker = Double::default().answering(&["info"], &[REFUSED]);
    let evolved = given::started(docker, of_t1(TWO_CPUS));
    assert!(matches!(
        evolved.result,
        Err(DirectedEvolutionError::Machine(_))
    ));
}

#[test]
fn an_image_that_a_task_names_as_the_environment_of_its_grader_is_told() {
    let said = given::started(Double::default(), of_t1(GRADER_GIVEN)).said;
    assert!(said.contains(
        "taking the image a-grader:given as the environment of the grader of task benchmark/t1"
    ));
}

#[test]
fn an_image_of_a_grader_that_the_machine_lacks_ends_the_directed_evolution() {
    let docker = Double::default().answering(&["inspect", "a-grader:given"], &[REFUSED]);
    let evolved = given::started(docker, of_t1(GRADER_GIVEN));
    assert!(matches!(
        evolved.result,
        Err(DirectedEvolutionError::GraderNotOnTheMachine { task, image, .. })
            if task == "benchmark/t1" && image == "a-grader:given"
    ));
}

#[test]
fn the_environment_of_a_grader_with_a_dockerfile_of_its_own_is_built_from_the_tests() {
    let evolved = given::started(Double::default(), with_a_dockerfile_of_the_grader);
    assert_eq!(evolved.count_of(GRADER_BUILT), 1);
}

#[test]
fn the_container_of_a_grader_built_from_the_tests_is_a_container_of_that_image() {
    let evolved = given::started(Double::default(), with_a_dockerfile_of_the_grader);
    assert_eq!(
        evolved.count_of(" --detach cladenc-environments-0123456789abcdef:0-grader sleep "),
        1
    );
}

#[test]
fn an_environment_of_a_grader_whose_build_fails_ends_the_directed_evolution_with_the_cause() {
    assert!(matches!(
        grader_not_built().result,
        Err(DirectedEvolutionError::GraderEnvironment {
            task,
            error: ContainerError::Exit(1),
        }) if task == "benchmark/t1"
    ));
}

#[test]
fn an_environment_of_a_grader_whose_build_fails_is_told_with_the_cause() {
    let said = grader_not_built().said;
    assert!(said.contains(
        "the build of the environment of the grader of task benchmark/t1 exited with status 1"
    ));
}

#[test]
fn a_grader_with_no_dockerfile_of_its_own_gets_a_container_of_the_image_of_its_task() {
    let evolved = given::started(Double::default(), of_t1(SEPARATE));
    assert_eq!(evolved.count_of("--detach"), 3);
}

#[test]
fn a_grader_that_asks_for_more_processors_than_the_machine_has_ends_the_directed_evolution() {
    assert!(matches!(
        on_a_machine("[verifier.environment]\ncpus = 16\n").result,
        Err(DirectedEvolutionError::TooLarge {
            key: "verifier.environment.cpus",
            ..
        })
    ));
}

fn lacking_the_image_of_t1() -> Evolved {
    let docker = Double::default().answering(&["image", "inspect"], &[REFUSED]);
    given::started(docker, of_t1(IMAGE_GIVEN))
}

fn built_for_too_long() -> Evolved {
    let docker = Double::default().answering(&["build"], &[Answer::TimedOut]);
    given::started(docker, given::unchanged)
}

fn holds_a_journal(evolved: &Evolved) -> bool {
    evolved.files().exists(journal::path(&evolved.outputs()))
}

fn places_of(evolved: &Evolved, starts: &[&str]) -> Vec<Option<usize>> {
    let lines = evolved.lines();
    starts
        .iter()
        .map(|start| lines.iter().position(|line| line.starts_with(start)))
        .collect()
}

#[test]
fn the_environment_of_each_task_is_told_to_be_built_with_the_image_it_is_built_as() {
    let building: Vec<String> = made()
        .lines()
        .into_iter()
        .filter(|line| line.starts_with("cladenc: building the environment of task "))
        .collect();
    assert_eq!(
        building,
        [
            "cladenc: building the environment of task benchmark/t1 as the image \
             cladenc-environments-0123456789abcdef:0",
            "cladenc: building the environment of task benchmark/t2 as the image \
             cladenc-environments-0123456789abcdef:1",
        ]
    );
}

#[test]
fn the_environments_are_built_after_the_tasks_are_held_out_and_before_the_first_action() {
    let starts = [
        "cladenc: holding out ",
        "cladenc: building the environment of task benchmark/t1 ",
        "cladenc: built the environments ",
        "cladenc: started the ",
    ];
    assert_eq!(
        places_of(&made(), &starts),
        [Some(2), Some(3), Some(5), Some(6)]
    );
}

#[test]
fn the_environments_built_for_some_of_the_tasks_are_told_with_how_many_of_how_many() {
    let evolved = given::started(Double::default(), of_t1(IMAGE_GIVEN));
    assert!(evolved.said.contains(&format!(
        "cladenc: built the environment of 1 of the 2 tasks: what Docker printed is kept in \
         {}\n",
        evolved.outputs().join("environments").display()
    )));
}

#[test]
fn an_environment_whose_build_fails_leaves_the_directed_evolution_no_journal() {
    assert!(!holds_a_journal(&not_built()));
}

#[test]
fn an_environment_whose_build_fails_leaves_the_environments_after_it_unbuilt() {
    assert_eq!(not_built().count_of("docker build "), 1);
}

#[test]
fn a_directed_evolution_stopped_while_an_environment_is_built_does_not_say_it_was_not_built() {
    let docker = Double::default().answering(&["build"], &[Answer::Stopped]);
    let said = given::started(docker, given::unchanged).said;
    assert!(!said.contains("cladenc: the build of the environment"));
}

#[test]
fn an_environment_whose_build_runs_past_the_time_limit_of_its_task_ends_the_directed_evolution() {
    assert!(matches!(
        built_for_too_long().result,
        Err(DirectedEvolutionError::Environment {
            task,
            error: ContainerError::TimeLimit,
        }) if task == "benchmark/t1"
    ));
}

#[test]
fn an_environment_whose_build_runs_past_its_time_limit_is_said_with_where_what_docker_printed_is_kept()
 {
    let evolved = built_for_too_long();
    assert!(evolved.said.contains(&format!(
        "cladenc: the build of the environment of task benchmark/t1 ran past its time limit: \
         what Docker printed is kept in {}\n",
        evolved
            .outputs()
            .join("environments/benchmark/t1")
            .display()
    )));
}

#[test]
fn an_image_that_a_task_names_and_the_machine_lacks_leaves_the_directed_evolution_no_journal() {
    assert!(!holds_a_journal(&lacking_the_image_of_t1()));
}

#[test]
fn an_image_that_a_task_names_and_the_machine_lacks_leaves_the_environments_after_it_unbuilt() {
    assert_eq!(lacking_the_image_of_t1().count_of("docker build "), 0);
}

#[test]
fn a_task_that_asks_for_more_than_the_machine_has_leaves_the_directed_evolution_no_journal() {
    assert!(!holds_a_journal(&on_a_machine(
        "[environment]\ncpus = 16\n"
    )));
}

#[test]
fn a_directed_evolution_that_runs_to_its_end_holds_a_journal() {
    assert!(holds_a_journal(&made()));
}

#[test]
fn the_environment_of_a_grader_is_told_to_be_built_with_the_image_it_is_built_as() {
    let said = given::started(Double::default(), with_a_dockerfile_of_the_grader).said;
    assert!(said.contains(
        "cladenc: building the environment of the grader of task benchmark/t1 as the image \
         cladenc-environments-0123456789abcdef:0-grader\n"
    ));
}

#[test]
fn the_environment_of_a_grader_is_built_after_that_of_its_task_and_before_that_of_the_next() {
    let evolved = given::started(Double::default(), with_a_dockerfile_of_the_grader);
    let starts = [
        "cladenc: building the environment of task benchmark/t1 ",
        "cladenc: building the environment of the grader of task benchmark/t1 ",
        "cladenc: building the environment of task benchmark/t2 ",
    ];
    assert_eq!(places_of(&evolved, &starts), [Some(3), Some(4), Some(5)]);
}

#[test]
fn a_grader_that_runs_in_a_copy_of_the_environment_of_its_task_has_no_environment_built() {
    let said = given::started(Double::default(), of_t1(SEPARATE)).said;
    assert!(!said.contains("environment of the grader"));
}

#[test]
fn a_grader_that_asks_for_more_memory_than_the_machine_has_ends_the_directed_evolution() {
    assert!(matches!(
        on_a_machine("[verifier.environment]\nmemory_mb = 32000\n").result,
        Err(DirectedEvolutionError::TooLarge {
            key: "verifier.environment.memory_mb",
            ..
        })
    ));
}
