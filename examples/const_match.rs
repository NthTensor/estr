//! Shows how to match a runtime string against compile-time known strings,
//! so that at runtime, only a single hash comparison is needed.

use estr::{digest, estr};

fn main() {
    // think of this as a string that is not known at compile time
    let some_string = estr("bar");

    // these however *are* known at compile time
    const FOO: u64 = digest("foo").hash();
    const BAR: u64 = digest("bar").hash();
    const BAZ: u64 = digest("baz").hash();

    match some_string.digest().hash() {
        FOO => {
            println!("got a foo!");
        }
        BAR => {
            println!("got a bar!");
        }
        BAZ => {
            println!("got a baz!");
        }
        _ => {
            println!("got something else!");
        }
    }
}
