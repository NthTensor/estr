//! Shows how to match a runtime string against compile-time known strings,
//! so that at runtime, only a single hash comparison is needed.
use estr::*;

fn main() {
    // think of this as some runtime string that is not known at compile time
    let some_string = estr("bar");

    match &some_string.hash() {
        ehash!("foo") => {
            println!("got a foo!");
        }
        ehash!("bar") => {
            println!("got a bar!");
        }
        ehash!("baz") => {
            println!("got a baz!");
        }
        _ => {
            println!("got something else!");
        }
    }
}
