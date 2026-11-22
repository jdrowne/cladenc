use test_inputs::{manual, manual_on_one_line, text_of_roff};

#[test]
fn a_line_of_text_is_kept_as_it_is() {
    assert_eq!(
        text_of_roff("A harness procreates."),
        "A harness procreates."
    );
}

#[test]
fn the_title_of_a_section_is_kept_without_its_request() {
    assert_eq!(text_of_roff(".SH FLAGS"), "FLAGS");
}

#[test]
fn the_title_of_a_subsection_is_kept_without_its_request() {
    assert_eq!(text_of_roff(".SS Tables"), "Tables");
}

#[test]
fn a_request_that_says_nothing_leaves_an_empty_line() {
    assert_eq!(text_of_roff("one\n.PP\ntwo"), "one\n\ntwo");
}

#[test]
fn an_escaped_hyphen_is_a_hyphen() {
    assert_eq!(text_of_roff("\\-\\-outputs\\-dir"), "--outputs-dir");
}

#[test]
fn an_escaped_apostrophe_is_an_apostrophe() {
    assert_eq!(text_of_roff("\\(aqfi\\(aq"), "'fi'");
}

#[test]
fn a_change_of_font_says_nothing() {
    assert_eq!(text_of_roff("\\fBcladenc\\fP report"), "cladenc report");
}

#[test]
fn the_guard_of_a_line_that_opens_with_a_dot_says_nothing() {
    assert_eq!(text_of_roff("\\&.gitignore"), ".gitignore");
}

#[test]
fn an_escaped_backslash_is_a_backslash() {
    assert_eq!(
        text_of_roff("a character after \\e"),
        "a character after \\"
    );
}

#[test]
fn an_escaped_backslash_before_an_escaped_hyphen_is_a_backslash_and_a_hyphen() {
    assert_eq!(text_of_roff("\\e\\-"), "\\-");
}

#[test]
fn the_manual_of_a_command_is_read_from_its_page() {
    assert!(manual("cladenc").contains("NAME\ncladenc - "));
}

#[test]
fn the_manual_keeps_the_lines_of_an_example() {
    assert!(manual("cladenc-start").contains(
        "--harness-run-command 'if [ -d /var/log/cladenc/attempts ]\n    then echo >> procreated; \
         else <COMMAND>; fi'"
    ));
}

#[test]
fn the_manual_on_one_line_holds_no_line_break() {
    assert!(!manual_on_one_line("cladenc").contains('\n'));
}

#[test]
fn the_manual_on_one_line_holds_no_two_spaces_in_a_row() {
    assert!(!manual_on_one_line("cladenc-start").contains("  "));
}

#[test]
fn the_manual_on_one_line_joins_what_two_lines_say_with_a_space() {
    assert!(manual_on_one_line("cladenc").contains("FLAGS -h, --help Print help."));
}
