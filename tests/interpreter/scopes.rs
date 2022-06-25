use crate::helpers::execute;
use insta::assert_display_snapshot;

#[test]
fn lexical_scopes_are_interpreted_correctly() {
    let source = r#"var a = "global a";
var b = "global b";
var c = "global c";
{
  var a = "outer a";
  var b = "outer b";
  {
    var a = "inner a";
    print a;
    print b;
    print c;
  }
  print a;
  print b;
  print c;
}
print a;
print b;
print c;"#;
    let output = execute(source);
    assert_display_snapshot!(output, @r###"
    inner a
    outer b
    global c
    outer a
    outer b
    global c
    global a
    global b
    global c
    "###);
}

#[test]
fn function_and_lexical_scopes() {
    let source = r#"var a = "global";
{
  fun showA() {
    print a;
  }
  
  showA();
  var a = "block";
  showA();
}"#;
    let output = execute(source);
    assert_display_snapshot!(output, @r###"
global
global"###);
}
