//! Generates tests based on failing pieces of code.
//!
//! Any piece of code that is wrapped in an [Executable](trait.Executable.html) can be passed to [execute](fn.execute.html). This function will wrap the resulting `Executable::Error` type in a [UnitTest](struct.UnitTest.html) type. This `UnitTest` can then be appended to a file.
//!
//! ```rust
//! extern crate exceptional;
//! extern crate serde;
//! extern crate serde_json;                // Needed for the actual generated test
//! #[macro_use]
//! extern crate serde_derive;
//!
//! #[derive(Serialize, Deserialize, Clone)]
//! pub struct SomeImportantAction {        // All structs must be Serializable and Deserializable to express them in the #[test]
//!     pub var_1: u32,
//!     pub var_2: u32,
//! }
//!
//! impl exceptional::Executable for SomeImportantAction {
//!     type Result = ();
//!     type Error = String;
//!     type Arguments = (u32, u32);
//!
//!     /// Get the full path for this type. This will be used to generate the unit test.
//!     fn full_path(&self) -> &'static str {
//!         "::SomeImportantAction"
//!     }
//!
//!     /// Get the description for this type to describe what it's function is, etc. This will show up in the description of the unit test
//!     fn description(&self) -> String {
//!         String::from("Executes some very important action!")
//!     }
//!
//!     /// Execute the logic. Call this with [execute](method.execute.html) to generate a Unit Test out of this error.
//!     fn execute(&mut self, args: &(u32, u32)) -> Result<(), String> {
//!         if args.1 == 3 {
//!             Err(String::from("Whoopsie"))
//!         } else {
//!             Ok(())
//!         }
//!     }
//! }
//!
//! fn main() {
//!     let mut action = SomeImportantAction {
//!         var_1: 0,
//!         var_2: 1,
//!     };
//!     let args = (2, 3);
//!
//!     // This will fail because args.1 == 3
//!     let result = exceptional::execute(&mut action, &args);
//!     if let Err(e) = result {
//!         e.append_to_file("src/test.rs")
//!             .expect("Could not write unit test");
//!         println!(
//!             "oh no we failed! Check src/test.rs for our newly generated unit test"
//!         );
//!         return;
//!     }
//! }
//! ```
#![deny(missing_docs)]

extern crate chrono;
extern crate serde;
extern crate serde_json;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::fs::OpenOptions;
use std::io::{self, Write};
use std::path::Path;

/// The trait that structs should implement to make them executable.
///
/// Note: this is cloned every time this is executed. Consider putting non-mutable values in `Arguments` rather than this struct.
///
/// Note: Any internally mutable values (like Rc<RefCell<T>>) will write the post-exception state to the test. It is assumed that these structs are not Serialize-safe.
pub trait Executable: Serialize + for<'a> Deserialize<'a> + Clone {
    /// The result of the execute action.
    type Result;

    /// The error of the execute action.
    type Error: Debug;

    /// The arguments that will be passed to the execute action
    type Arguments: Serialize;

    /// Get the full path for this type. This will be used to generate the unit test.
    fn full_path(&self) -> &'static str;

    /// Get the description for this type to describe what it's function is, etc. This will show up in the description of the unit test
    fn description(&self) -> String;

    /// Execute the logic. Call this with [execute](method.execute.html) to generate a Unit Test out of this error.
    fn execute(&mut self, args: &Self::Arguments) -> Result<Self::Result, Self::Error>;
}

/// Execute the given [Executable](trait.Executable.html). If the Executable fails, this struct will wrap the Error in a [UnitTest](struct.UnitTest.html) struct. This UnitTest struct can then be appended to a file.
///
/// Note: this always clones the given executable, because we need to store the state from before it failed. Make sure the `clone` impl is not too heavy.
pub fn execute<'a, E: Executable + 'a>(
    executable: &'a mut E,
    arguments: &'a E::Arguments,
) -> Result<E::Result, UnitTest<'a, E>> {
    // TODO: executable will be modified after the execute finishes.
    // Do we want to clone it every time? Alternatively we can make executable non-mut.
    let old = executable.clone();
    match executable.execute(arguments) {
        Ok(value) => Ok(value),
        Err(error) => Err(UnitTest {
            error,
            arguments,
            executable: old,
            time: Utc::now(),
        }),
    }
}

/// A unit-test-in-making. This wraps the Executable that failed, the arguments used, the actual error that was thrown, and when it happened.
pub struct UnitTest<'a, E: Executable + 'a> {
    /// The error that was thrown when the Executable failed
    pub error: E::Error,

    /// The arguments that were provided that caused the Executable to fail
    pub arguments: &'a E::Arguments,

    /// The executable that failed, with the state from before it failed.
    pub executable: E,

    /// The time at which this executable failed
    pub time: DateTime<Utc>,
}

impl<'a, E: Executable + 'a> UnitTest<'a, E> {
    /// Append this unit test to a file.
    pub fn append_to_file(self, file: impl AsRef<Path>) -> io::Result<()> {
        let mut file = OpenOptions::new().create(true).append(true).open(file)?;
        let text = self.to_string();
        file.write_all(text.as_bytes())?;
        Ok(())
    }
}

impl<'a, E: Executable + 'a> std::fmt::Display for UnitTest<'a, E> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(
            fmt,
            "/// Automatically generated unit test for Executable\n"
        )?;
        writeln!(fmt, "/// {}", self.executable.description())?;
        writeln!(fmt, "/// generated at {}", self.time.to_rfc2822())?;
        writeln!(fmt)?;
        writeln!(fmt, "/// exception was {:?}", self.error)?;
        writeln!(fmt, "#[test]")?;
        writeln!(fmt, "pub fn test_{}() {{", self.time.timestamp_millis())?;
        writeln!(fmt, "\tuse exceptional::Executable;")?;
        writeln!(
            fmt,
            "\tlet obj_json = r#\"{}\"#;",
            serde_json::to_string_pretty(&self.executable)
                .expect("Could not serialize the executable")
        )?;
        writeln!(fmt, "\tlet mut obj: {} = ::serde_json::from_str(obj_json).expect(\"Could not deserialize json\");", self.executable.full_path())?;
        writeln!(fmt, "\t")?;
        writeln!(
            fmt,
            "\tlet arg_json = r#\"{}\"#;",
            serde_json::to_string_pretty(&self.arguments).expect("Could not serialize arguments")
        )?;
        writeln!(
            fmt,
            "\tlet args = ::serde_json::from_str(arg_json).expect(\"Could not deserialize json\");"
        )?;
        writeln!(fmt)?;
        writeln!(fmt, "\tif let Err(e) = obj.execute(&args) {{")?;
        writeln!(
            fmt,
            "\t\tprintln!(\"Could not execute {{}}\", obj.description());"
        )?;
        writeln!(fmt, "\t\tprintln!(\"{{:?}}\", e);")?;
        writeln!(fmt, "\t\tpanic!();")?;
        writeln!(fmt, "\t}}")?;
        writeln!(fmt, "}}")?;

        Ok(())
    }
}
