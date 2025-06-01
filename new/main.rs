use std::{io, process};

mod client;
mod map;
mod mapping_tool;
mod packet;
mod player;
mod server;

fn main() {
    println!("Welcome to Momentum!\nChoose an option:");
    println!("1. Start a server");
    println!("2. Play");
    println!("3. Make a map");
    println!("4. Exit");

    let mut buf = String::new();
    io::stdin().read_line(&mut buf).unwrap();
    let choice = match buf.trim().parse::<u8>() {
        Ok(choice) => choice,
        Err(_) => {
            println!("Invalid choice");
            return;
        }
    };

    match choice {
        1 => server::main(),
        2 => client::main(),
        3 => mapping_tool::main(),
        4 => process::exit(0),
        _ => println!("Invalid choice"),
    }
}
