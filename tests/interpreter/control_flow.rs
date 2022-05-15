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

#[test]
fn ambiguous_if_else_is_execute_correctly() {
    // The else binds to the closest if, `if (true)`, therefore
    // it's not executed.
    let source = r#"if (false)
    if (true)
        print "if";
    else
        print "else";"#;
    let output = execute(source);
    assert_display_snapshot!(output, @r###"
    "###);
}
