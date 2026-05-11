use ansi_term::Colour::{Red, Green, Yellow};

fn main() {
    println!("{}", Red.paint("Error: something went wrong"));
    println!("{}", Green.paint("Success: operation completed"));
    println!("{}", Yellow.paint("Warning: check your input"));
}
