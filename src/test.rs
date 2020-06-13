/// Automatically generated unit test for Executable
/// Executes some very important action!
/// generated at Wed, 17 Oct 2018 07:27:13 +0000
///
/// exception was "Whoopsie"
#[test]
pub fn test_1539761233543() {
    use exceptional::Executable;
    let obj_json = r#"{
  "var_1": 0,
  "var_2": 0
}"#;
    let mut obj: ::SomeImportantAction =
        ::serde_json::from_str(obj_json).expect("Could not deserialize json");

    let arg_json = r#"[
  2,
  0
]"#;
    let args = ::serde_json::from_str(arg_json).expect("Could not deserialize json");

    if let Err(e) = obj.execute(&args) {
        println!("Could not execute {}", obj.description());
        println!("{:?}", e);
        panic!();
    }
}
