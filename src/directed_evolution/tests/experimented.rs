use crate::benchmark::Benchmark;
use crate::containers::ContainerError;
use crate::containers::tests::double::{Answer, Double};
use crate::directed_evolution::DirectedEvolutionError;
use crate::directed_evolution::tests::given::{self, Evolved};
use crate::file_system::tests::double::Memory;
use crate::genealogy::CreationOutcome;
use crate::journal::{Record, Settings, Started};

const A_BASELINE: [&str; 2] = ["run a harness", "baseline-3"];
const THE_BASELINE: &str = "the creation of harness 3, the singleton control group harness, by \
                            the primogenitor from a second clone of harness 1";

fn twice_over_three_tasks(directory: &Memory, settings: &mut Settings) {
    let benchmark = test_inputs::benchmark_in(directory, directory.path(), &["t1", "t2", "t3"]);
    settings.benchmarks = vec![Benchmark::read(directory, &benchmark).unwrap()];
    settings.max_diversifications = 2;
    settings.selection_tasks_percentage = 33.5;
    settings.holdout_tasks = vec!["benchmark/t2".to_owned(), "benchmark/t3".to_owned()];
    settings.corroboration_tasks_percentage = 33.5;
    settings.corroboration_tasks = vec!["benchmark/t3".to_owned()];
    settings.seed = 1;
}

fn working() -> Double {
    Double::default()
        .changing()
        .verifying(|trial| trial == "trial-2-benchmark+t3")
}

fn experimented() -> Evolved {
    given::started(working(), twice_over_three_tasks)
}

fn stopped_at_its_baseline() -> Evolved {
    let docker = working().answering(&A_BASELINE, &[Answer::Exited(125)]);
    given::started(docker, twice_over_three_tasks)
}

fn resumed() -> Evolved {
    given::resumed(stopped_at_its_baseline(), working())
}

fn changing_nothing() -> Evolved {
    given::started(Double::default(), twice_over_three_tasks)
}

fn leaving_the_clone_of_the_baseline_as_it_was() -> Evolved {
    let docker = Double::default().changing_when(|asked| !asked.kept.ends_with("baseline-3"));
    given::started(docker, twice_over_three_tasks)
}

fn kept_of_the_baseline(evolved: &Evolved, file: &str) -> String {
    evolved
        .files()
        .read_to_string(evolved.outputs().join("baselines/3").join(file))
        .unwrap_or_default()
}

#[test]
fn a_baseline_is_established_at_the_diversification_of_a_harness_other_than_the_primogenitor_alone()
{
    assert_eq!(
        experimented().starts(),
        [
            "trial of 0",
            "trial of 0",
            "trial of 0",
            "diversification of 1 by 0",
            "trial of 1",
            "trial of 1",
            "trial of 1",
            "diversification of 2 by 1",
            "baseline of 3 for 2 from 1",
            "trial of 2",
            "trial of 2",
            "trial of 2",
            "trial of 3",
            "trial of 3",
            "trial of 3",
        ]
    );
}

#[test]
fn what_is_kept_of_a_baseline_names_the_progenitor_the_child_and_how_it_ended() {
    let evolved = experimented();
    assert_eq!(
        ["progenitor", "child", "outcome"].map(|file| kept_of_the_baseline(&evolved, file)),
        ["1\n", "2\n", "created\n"]
    );
}

#[test]
fn a_directed_evolution_says_that_it_starts_the_creation_of_the_singleton_control_group_harness() {
    let lines = experimented().lines();
    assert!(lines.contains(&format!("cladenc: started {THE_BASELINE}")));
}

#[test]
fn a_directed_evolution_says_how_a_baseline_ended_and_where_its_harness_is_kept() {
    let evolved = experimented();
    assert!(evolved.lines().contains(&format!(
        "cladenc: {THE_BASELINE} created harness 3, kept in {}",
        evolved.outputs().join("harnesses/3").display()
    )));
}

#[test]
fn a_directed_evolution_numbers_a_diversification_by_the_diversifications_started_before_it() {
    let lines = experimented().lines();
    assert!(
        lines.contains(
            &"cladenc: started the diversification of harness 2 by harness 1, diversification 2 of \
              2"
                .to_owned()
        )
    );
}

#[test]
fn a_directed_evolution_says_last_how_many_harnesses_it_holds_with_the_singleton_control_group_harness()
 {
    let evolved = experimented();
    assert_eq!(
        evolved.lines().last(),
        Some(&format!(
            "cladenc: the directed evolution finished with 12 trials started and 4 harnesses: \
             its best harness is harness 0, kept in {}",
            evolved.outputs().join("harnesses/0").display()
        ))
    );
}

#[test]
fn a_container_that_ends_with_status_125_at_a_baseline_stops_the_directed_evolution() {
    assert!(matches!(
        stopped_at_its_baseline().result,
        Err(DirectedEvolutionError::Action {
            action: Started::Baseline {
                harness: 3,
                child: 2,
                progenitor: 1
            },
            error: ContainerError::Docker(_),
        })
    ));
}

#[test]
fn a_directed_evolution_a_baseline_stopped_continues_to_the_journal_an_unbroken_one_writes() {
    assert_eq!(
        test_inputs::without_seconds(&resumed().journal()),
        test_inputs::without_seconds(&experimented().journal())
    );
}

#[test]
fn a_resumed_directed_evolution_says_that_it_starts_again_the_baseline_that_was_under_way() {
    let lines = resumed().lines();
    assert!(lines.contains(&format!("cladenc: started again {THE_BASELINE}")));
}

#[test]
fn a_resumed_directed_evolution_counts_the_singleton_control_group_harness_among_its_harnesses() {
    let resumed = resumed();
    assert_eq!(
        resumed.lines()[0],
        format!(
            "cladenc: continuing the directed evolution in {}, which has started 2 of 2 \
             diversifications and holds 4 harnesses",
            resumed.outputs().display()
        )
    );
}

#[test]
fn a_child_that_would_be_the_progenitor_s_clone_as_it_was_trials_no_task() {
    assert_eq!(
        changing_nothing().starts(),
        [
            "trial of 0",
            "trial of 0",
            "trial of 0",
            "diversification of 1 by 0",
            "diversification of 2 by 0",
        ]
    );
}

#[test]
fn a_diversification_that_left_the_progenitor_s_clone_as_it_was_is_journaled_as_unchanged() {
    let unchanged = changing_nothing()
        .records()
        .into_iter()
        .filter(|record| {
            matches!(
                record,
                Record::DiversificationFinished {
                    outcome: CreationOutcome::Unchanged,
                    ..
                }
            )
        })
        .count();
    assert_eq!(unchanged, 2);
}

#[test]
fn a_directed_evolution_says_that_a_diversification_left_the_clone_unchanged_and_created_no_harness()
 {
    let evolved = changing_nothing();
    assert!(evolved.lines().contains(&format!(
        "cladenc: the diversification of harness 1 by harness 0 left the clone of harness 0 \
         unchanged, and created no harness: what the harness printed is kept in {}",
        evolved.outputs().join("diversifications/1").display()
    )));
}

#[test]
fn what_a_diversification_that_changed_nothing_left_is_kept_among_the_harnesses() {
    let evolved = changing_nothing();
    let kept = evolved.outputs().join("harnesses/1/harness.sh");
    assert!(evolved.files().is_file(kept));
}

#[test]
fn a_singleton_control_group_harness_that_would_be_the_clone_as_it_was_trials_no_task() {
    assert_eq!(
        leaving_the_clone_of_the_baseline_as_it_was().starts(),
        [
            "trial of 0",
            "trial of 0",
            "trial of 0",
            "diversification of 1 by 0",
            "trial of 1",
            "trial of 1",
            "trial of 1",
            "diversification of 2 by 1",
            "baseline of 3 for 2 from 1",
            "trial of 2",
            "trial of 2",
            "trial of 2",
        ]
    );
}

#[test]
fn a_directed_evolution_says_that_a_baseline_left_the_clone_unchanged_and_created_no_harness() {
    let evolved = leaving_the_clone_of_the_baseline_as_it_was();
    assert!(evolved.lines().contains(&format!(
        "cladenc: {THE_BASELINE} left the clone of harness 1 unchanged, and \
         created no harness: what the primogenitor printed is kept in {}",
        evolved.outputs().join("baselines/3").display()
    )));
}
