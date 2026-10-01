#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    // TODO: Uncomment the code below to pass the first stage
    print!("$ ");
    let mut command = String::new();
    io::stdin().read_line(&mut command).unwrap();
    eprintln!("{command}: command not found");
    io::stdout().flush().unwrap();
}
