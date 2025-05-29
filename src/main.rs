use std::process;

mod client;
mod map;
mod mapping_tool;
mod packet;
mod player;
mod server;
pub fn main() {
    let args = std::env::args().collect::<Vec<String>>();

    if args.len() != 2 {
        eprintln!("Usage: {} <host/join>", args[0]);
        process::exit(1);
    }

    match args[1].as_str() {
        "host" => {
            server::main();
        }
        "mapping" => {
            mapping_tool::main();
        }
        "join" => {
            let _ = client::main();
        }
        _ => {
            eprintln!("Invalid mode");
            process::exit(1);
        }
    }
}
