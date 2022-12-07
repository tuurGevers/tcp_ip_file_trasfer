use std::fs::{OpenOptions};
use std::io::{BufRead, BufReader, Lines, Read, Write};
use std::net::{Shutdown, TcpStream};
use std::str::from_utf8;
use std::{fs, net, thread};
use std::rc::Rc;
use std::sync::{Arc, Mutex};


fn main() {
    let listener = net::TcpListener::bind("127.0.0.1:80").unwrap(); //tcp listener aanmaken en ermee verbinden

    for stream in listener.incoming() {
        //connectei die binekomt checken en handelen
        match stream {
            Ok(stream) => {
                println!("new connection with {}", stream.peer_addr().unwrap());
                thread::spawn(move || {
                    //maak thread aan en verplaats ownership
                    handle_file_connection(stream)
                });
            }
            Err(e) => panic!("error {}, shutting down", e),
        }
    }

    drop(listener);
}

fn handle_file_connection(mut stream: TcpStream){
    let mut data = [0 as u8; 50]; //buffer aanmaken van 50 bytes
    let mut incomming = String::new();
    let incomming_clone = incomming.clone();

    'outer: while match stream.read(&mut data) {
        //inkomende data blijven lezen en checken op errors
        Ok(size) => {
            stream.write(&data[0..size]).unwrap(); // response naar client

            incomming = from_utf8(&data).unwrap().trim_matches(char::from(0)).to_string();
            let mut lines = incomming.lines().into_iter();
            let mut content:Arc<Mutex<Vec<&str>>> = Arc::new(Mutex::new(Vec::new())) ;
            for line in lines{
                content.lock().unwrap().push(line);
                content.lock().unwrap().push("\n");
            }

            let file_name = content.lock().unwrap()[0];
            content.lock().unwrap().remove(0);
            let mut file = OpenOptions::new().write(true).create(true).open(file_name).unwrap();

            match file.write_all(content.lock().unwrap().concat().as_ref()){
                Ok(_) => println!("ok"),
                Err(e) => println!("error: {}", e),
            };
            true
        }
        Err(e) => {
            println!(
                "error occured {}, shutting down connection with {}",
                e,
                stream.peer_addr().unwrap()
            );
            stream.shutdown(Shutdown::Both).unwrap(); //reading en writing uitschakelen
            false
        }
    } {}

}

fn handle_connection(mut stream: TcpStream) {
    let file = OpenOptions::new().write(true).read(true).create(true).open("test.txt").unwrap();
    let mut buffer = BufReader::new(file);
    let mut lines: Vec<String> = Vec::new();

    let mut data = [0 as u8; 50]; //buffer aanmaken van 50 bytes
    let mut prev_data = [0 as u8; 50];
    'outer: while match stream.read(&mut data) {
        //inkomende data blijven lezen en checken op errors
        Ok(size) => {
            if data == prev_data {
                drop(stream);
                break 'outer;
            }
            let res = from_utf8(&data).unwrap().trim_matches(char::from(0));
            println!("{}", &res);
            stream.write(&data[0..size]).unwrap(); // response naar client
            for line in buffer.by_ref().lines() {
                match line {
                    Ok(l) => lines.push(format!("{}\n", l)),
                    Err(e) => {
                        println!("error: {}", e)
                    }
                }
            }
            lines.push(res.parse().unwrap());
            for line in &lines{
                println!("{}",line);
            }

            fs::remove_file("test.txt").unwrap();
            let mut file2 = OpenOptions::new().write(true).create(true).open("test.txt").unwrap();

            match file2.write_all(lines.concat().as_ref()){
                Ok(_) => println!("ok"),
                Err(e) => println!("error: {}", e),
            };
            prev_data = data;
            true
        }
        Err(e) => {
            println!(
                "error occured {}, shutting down connection with {}",
                e,
                stream.peer_addr().unwrap()
            );
            stream.shutdown(Shutdown::Both).unwrap(); //reading en writing uitschakelen
            false
        }
    } {}
}
