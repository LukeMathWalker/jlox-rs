use crate::helpers::{execute, try_execute};
use insta::assert_display_snapshot;

#[test]
fn declare_and_invoke_function() {
    let source = r#"fun sayHi(first, last) {
  print "Hi, " + first + " " + last + "!";
}

sayHi("Dear", "Reader");"#;
    let output = execute(source);
    assert_display_snapshot!(output, @r###"
    Hi, Dear Reader!
    "###);
}

#[test]
fn function_scope_does_not_leak() {
    let source = r#"fun f() {
    var c = 1;
}

print c;"#;
    let error = try_execute(source).unwrap_err();
    assert_display_snapshot!(error, @"An error occurred at runtime. Undefined variable named c");
}
