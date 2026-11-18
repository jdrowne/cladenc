use crate::holdout;

#[test]
fn sixty_in_a_hundred_of_500_tasks_is_300_tasks() {
    assert_eq!(holdout::count(500, 60.0), 300);
}

#[test]
fn a_half_of_a_task_is_rounded_up() {
    assert_eq!(holdout::count(5, 30.0), 2);
}

#[test]
fn less_than_a_half_of_a_task_is_rounded_down() {
    assert_eq!(holdout::count(5, 29.0), 1);
}

#[test]
fn more_than_a_half_of_a_task_is_rounded_up() {
    assert_eq!(holdout::count(3, 60.0), 2);
}

#[test]
fn a_percentage_with_a_decimal_point_is_counted_as_it_is_given() {
    assert_eq!(holdout::count(8, 12.5), 1);
}

#[test]
fn a_small_percentage_of_a_few_tasks_is_no_task() {
    assert_eq!(holdout::count(3, 10.0), 0);
}

#[test]
fn a_large_percentage_of_a_few_tasks_is_every_one_of_them() {
    assert_eq!(holdout::count(3, 90.0), 3);
}
