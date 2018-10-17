# Exceptional
Generates tests based on failing pieces of code.

Any piece of code that is wrapped in an [Executable](trait.Executable.html) can be passed to [execute](fn.execute.html). This function will wrap the resulting `Executable::Error` type in a [UnitTest](struct.UnitTest.html) type. This `UnitTest` can then be appended to a file.

```rust
extern crate exceptional;
extern crate serde;
extern crate serde_json;                // Needed for the actual generated test
#[macro_use]
extern crate serde_derive;

#[derive(Serialize, Deserialize)]
pub struct SomeImportantAction {        // All structs must be Serializable and Deserializable to express them in the #[test]
    pub var_1: u32,
    pub var_2: u32,
}

impl exceptional::Executable for SomeImportantAction {
    type Result = ();
     type Error = String;
    type Arguments = (u32, u32);

    /// Get the full path for this type. This will be used to generate the unit test.
    fn full_path(&self) -> &'static str {
        "::SomeImportantAction"
    }

    /// Get the description for this type to describe what it's function is, etc. This will show up in the description of the unit test
    fn description(&self) -> String {
        String::from("Executes some very important action!")
    }

    /// Execute the logic. Call this with [execute](method.execute.html) to generate a Unit Test out of this error.
    fn execute(&mut self, args: &(u32, u32)) -> Result<(), String> {
        if args.1 == 3 {
            Err(String::from("Whoopsie"))
        } else {
            Ok(())
        }
    }
}

fn main() {
    let mut action = SomeImportantAction {
        var_1: 0,
        var_2: 1,
    };
    let args = (2, 3);

    // This will fail because args.1 == 3
    let result = exceptional::execute(&mut action, &args);
    if let Err(e) = result {
        e.append_to_file("src/test.rs")
            .expect("Could not write unit test");
        println!(
            "oh no we failed! Check src/test.rs for our newly generated unit test"
        );
        return;
    }
}
```
