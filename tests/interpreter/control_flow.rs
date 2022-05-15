use crate::helpers::execute;
use insta::assert_display_snapshot;

#[test]
fn two_branch_conditional_works() {
    let source = r#"if (3 > 5) {
    print true;
} else {
    print false;
}"#;
    let output = execute(source);
    assert_display_snapshot!(output, @r###"
    false
    "###);
}

#[test]
fn single_branch_conditional_works() {
    let source = r#"if (5 > 2) {
    print true;
}"#;
    let output = execute(source);
    assert_display_snapshot!(output, @r###"
    true
    "###);
}
