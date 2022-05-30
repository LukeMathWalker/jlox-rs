use crate::helpers::execute;
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
    let output = execute(source);
    assert_display_snapshot!(output, @r###"
    Hi, Dear Reader!
    "###);
}
