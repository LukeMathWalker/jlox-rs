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
    String("inner a")
    String("outer b")
    String("global c")
    String("outer a")
    String("outer b")
    String("global c")
    String("global a")
    String("global b")
    String("global c")
    "###);
}
