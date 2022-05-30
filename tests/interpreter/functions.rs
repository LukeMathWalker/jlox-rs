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

#[test]
fn closure_capture_works_as_expected() {
    let source = r#"
var a = "global";
{
  fun showA() {
    print a;
  }

  showA();
  var a = "block";
  showA();
}"#;
    let output = execute(source);
    assert_display_snapshot!(output, @"
global
global")
}

#[test]
fn closure_can_capture_local_env() {
    let source = r#"
fun returnClosure() {
    var a = 3;
    fun showA() {
        print a;
    }
    return showA;
}

var f = returnClosure();
f();
"#;
    let output = execute(source);
    assert_display_snapshot!(output, @"
3")
}

#[test]
fn local_function() {
    let source = r#"
fun makeCounter() {
  var i = 0;
  fun count() {
    i = i + 1;
    print i;
  }

  return count;
}

var counter = makeCounter();
counter();
counter();"#;
    let output = execute(source);
    assert_display_snapshot!(output, @"
1
2")
}
