extern crate exceptional;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

#[cfg(test)]
mod test;

fn main() {
    for i1 in 0..10 {
        for i2 in 0..10 {
            for i3 in 0..10 {
                for i4 in 0..10 {
                    let mut action = SomeImportantAction {
                        var_1: i1,
                        var_2: i2,
                    };
                    let args = (i3, i4);

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
            }
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SomeImportantAction {
    pub var_1: u32,
    pub var_2: u32,
}

impl exceptional::Executable for SomeImportantAction {
    type Result = ();
    type Error = String;
    type Arguments = (u32, u32);

    fn full_path(&self) -> &'static str {
        "::SomeImportantAction"
    }
    fn description(&self) -> String {
        String::from("Executes some very important action!")
    }

    fn execute(&mut self, args: &(u32, u32)) -> Result<(), String> {
        if args.0 == 3 {
            Err(String::from("Whoopsie"))
        } else {
            Ok(())
        }
    }
}
