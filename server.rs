use crate::packet;
use crate::player::Player;
use std::io::{Error, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

type ClientList = Arc<Mutex<Vec<TcpStream>>>;

pub fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    let clients: ClientList = Arc::new(Mutex::new(Vec::new()));
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let stream_clone = stream.try_clone().unwrap(); // Clone for broadcasting
                let clients = Arc::clone(&clients);
                clients.lock().unwrap().push(stream_clone);

                thread::spawn(move || {
                    handle_client(stream, clients.clone());
                });
            }
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }
    }
}

fn handle_client(mut stream: TcpStream, clients: ClientList) {
    loop {
        let packet = packet::receive_packet(&mut stream);
        if let Err(e) = packet {
            eprintln!("Error: {}", e);
            break;
        }
        println!("Received packet {:?}", packet);
        println!("Connected clients: {}", clients.lock().unwrap().len());
        // Broadcast packet to all clients
        let clients = clients.lock().unwrap();
        for client in clients.iter() {
            if client.peer_addr().unwrap() != stream.peer_addr().unwrap() {
                println!("Broadcasting to: {}", client.peer_addr().unwrap());
                if let Ok(player_packet) = &packet {
                    if let Err(e) =
                        packet::send_packet(&mut client.try_clone().unwrap(), player_packet)
                    {
                        eprintln!("Error broadcasting packet: {}", e);
                    }
                }
            }
        }
    }
}
