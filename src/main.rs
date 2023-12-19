mod commands;
mod spawning_pool;
mod redis_translate;

use std::{thread, time};
use std::net::{TcpListener, TcpStream, Shutdown};
use std::io::{BufReader, BufWriter, Read};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::fs::OpenOptions;
use crate::spawning_pool::ThreadPool;
use crate::redis_translate::{redis_to_array, inline_redis};
use crate::commands::{command_ping, command_set, command_get, command_del, command_save, trash};


/// Function handling a client, called in a dedicated thread.
fn handle_client(stream: TcpStream,
                 dico: &Arc<RwLock<HashMap<String, Vec<(String, Vec<String>)>>>>) {
    loop {
        let mut reader = BufReader::new(&stream);
        let mut writer = BufWriter::new(&stream);
        let mut data = vec![0; 256];  // Buffer of 256 Bytes
        let len = reader.read(&mut data).unwrap();

        // Handle client disconnecting.
        if len == 0
        {
            println!("{} has left the game.", stream.peer_addr().unwrap());
            stream.shutdown(Shutdown::Both).expect("shutdown call failed");
            return;
        }

        let mut entry_array: Vec<(String, Vec<(String, Vec<String>)>)> = redis_to_array(&data);

        // Handle inline commands
        // WARNING Probably only works with PING for now!!!
        if entry_array.len() == 0
            {entry_array = inline_redis(&data);}
        if entry_array.len() == 0
            {continue;}
        /*
         ex: [("Array", [("SimpleString", ["PING"])])]
         ex: [("Array", [("SimpleString", ["GET"]), ("SimpleString", ["a"])])]
         */

        match entry_array[0].0.as_str() {
            "Array" => {
                match entry_array[0].1[0].1[0].to_lowercase().as_str() {
                    "ping" => { command_ping(&mut writer, &entry_array[0].1); }
                    "stop" => {
                        // For client to disconnect without sending SIGINT to their program or something.
                        // Malheureusement j'ai pas eu le temps de faire se fermer les threads
                        println!("Disconnecting {}", stream.peer_addr().unwrap());
                        stream.shutdown(Shutdown::Both).expect("shutdown call failed");
                        break;
                    }
                    "set" => {  command_set (&entry_array[0].1, &dico, &mut writer); }
                    "get" => {  command_get (&entry_array[0].1, &dico, &mut writer); }
                    "del" => {  command_del (&entry_array[0].1, &dico, &mut writer); }
                    "save" => { command_save("storage.json".to_string(), &dico, &mut writer); }

                    _other => {
                        // Returns an error
                        trash(&mut writer);
                    }
                }
            }
            _other => {/* Ignore that */}
        }
    }
}


/// Saves data on path every 5 minutes.
///
/// Should be called by a dedicated thread.
fn auto_saver(path: String,
              dico: &Arc<RwLock<HashMap<String, Vec<(String, Vec<String>)>>>>)
{
    let five_minutes = time::Duration::from_secs(300);
    loop
    {
        thread::sleep(five_minutes);
        println!("Auto-saving, please do not shutdown server!");
        let file = OpenOptions::new()
                    .read(true)
                    .write(true)
                    .append(false)
                    .open(path.clone()).unwrap();
        let this_dico_val = dico
                .read()
                .expect("RwLock poisoned");

        file.set_len(0).unwrap();

        match serde_json::to_writer(file, &*this_dico_val)
        {
            Ok(_ok) => {
                println!("Auto-save complete!");
            }
            Err(_e) => {
                panic!();
            }
        }
    }
}




fn main() {
    // TODO Demander l'IP et le port à l'utilisateur (std::env)
    let listener = match TcpListener::bind("127.0.0.1:8080") {
        Ok(tcp) => {tcp}
        Err(_e) => {panic!("ERROR! Couldn't create TcpListener!");}
    };
    // accept connections and process them, spawning a new thread for each one
    println!("Server listening on port 8080");

    // TODO Demander le nom du fichier de stockage (ajouter option pour choisir)
    // Try to load stored data
    let chemin = String::from("storage.json");
    let file = match OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .open(&chemin)
    {
        Ok(f) => {f}
        Err(_e) => {panic!("ERROR! Couldn't open nor create storage file!")}
    };

    let hashmap: HashMap<String, Vec<(String, Vec<String>)>>;
    match serde_json::from_reader(file)
    {
        Ok(data) => {hashmap = data;
            println!("Old HashMap found, loading it.")
        }
        Err(_e) => {hashmap = HashMap::new();
            println!("No HashMap found, creating one.")
        }
    }

    let dico_val = Arc::new(RwLock::new(hashmap));
    let dico_val_clone = Arc::clone(&dico_val);
    let saver = thread::spawn(move || {
        auto_saver(chemin.to_string(), &dico_val_clone);
    });

    // TODO Let user choose thread number
    // Creating a pool to have a max number of threads
    // Do note that there is an additional thread handling auto-saves
    let spawning_pool = ThreadPool::new(500, saver);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("New connection: {}", stream.peer_addr().unwrap());
                let dico_val_clone = Arc::clone(&dico_val);
                // Starting a thread from the pool
                spawning_pool.execute(move || {
                    handle_client(stream, &dico_val_clone);
                });
            }
            Err(e) => {
                println!("Error: {}", e);
                // connection failed
            }
        }
    }
    // close the socket server
    drop(listener);
}
