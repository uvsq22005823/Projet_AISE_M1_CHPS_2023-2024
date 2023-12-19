use std::net::TcpStream;
use std::io::Write;
use std::io::BufWriter;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::fs::OpenOptions;
use crate::redis_translate::{pair_to_redis, array_to_redis, array_to_redis_get};


/// Function handling ping command
///
///
/// # Example
///
/// (with &my_vec = [("BulkString", ["PING"])])
/// command_ping(&mut writer, &my_vec)
/// sends +PONG towards client's side
pub fn command_ping(writer: &mut BufWriter<&TcpStream>, string_array: &Vec<(String, Vec<String>)>) {
    match string_array.len()  // Nombre d'arguments
    {
        1 => {writer.write(b"+PONG\r\n").unwrap();}
        2 => {
            // Returning a BulkString
            let mut retour: String = String::new();
            retour.push_str(pair_to_redis(string_array[1].clone()).as_str());
            writer.write(retour.as_bytes()).unwrap();
        }
        taille => {
            // Returnin an array
            writer.write(array_to_redis(&string_array, taille).as_bytes()).unwrap();
        }
    }
}



/// Function handling set command
///
///
/// # Example
///
/// (with &my_vec = [("BulkString", ["SET"]), ("BulkString", ["a"]), ("BulkString", ["coucou"])]])
/// command_set(&mut writer, &my_vec)
/// sends +OK towards client's side
/// and add corresponding entry to dico
pub fn command_set(weird_array: &Vec<(String, Vec<String>)>,
                   dico: &Arc<RwLock<HashMap<String, Vec<(String, Vec<String>)>>>>,
                   writer: &mut BufWriter<&TcpStream>)
{
    match weird_array.len() {
        0 => {writer.write("-Error (Should not be reachable))\r\n".as_bytes()).unwrap();}
        1 => {writer.write("-ERROR: No key given\r\n".as_bytes()).unwrap();}
        2 => {writer.write(b"-ERROR: Key cannot be NULL\r\n").unwrap();} // Maybe it should be able to ?
        3 => {/* Type simple*/
            // Access shared structure
            let mut this_dico_val = dico
                .write()
                .expect("RwLock poisoned");
            this_dico_val.insert(weird_array[1].1[0].to_string(), vec![weird_array[2].clone()]);
        }
        taille => { // Array
            let mut this_dico_val = dico
                .write()
                .expect("RwLock poisoned");
            let mut vecteur: Vec<(String, Vec<String>)> = vec![];
            for i in 2..taille
            {
                vecteur.push(weird_array[i].clone());
            }
            this_dico_val.insert(weird_array[1].1[0].to_string(), vecteur);
        }
    }

    writer.write(b"+OK\r\n").unwrap();
}



/// Function handling get command
///
///
/// # Example
///
/// (with &my_vec = [("BulkString", ["GET"]), ("BulkString", ["a"]), ("BulkString", ["b"])]]
///     a having "coucou" for entry and b not having one
/// command_set(&mut writer, &my_vec)
/// sends "coucou" and Null towards client's side
pub fn command_get(string_array: &Vec<(String, Vec<String>)>, dico: &Arc<RwLock<HashMap<String, Vec<(String, Vec<String>)>>>>,
               writer: &mut BufWriter<&TcpStream>)
{
    match string_array.len() {
        0 => {writer.write("-Error (Should not be reachable))\r\n".as_bytes()).unwrap();}
        1 => {writer.write("-ERROR: No key given\r\n".as_bytes()).unwrap();}
        2 => {/* Type simple*/
            // Access shared structure
            let this_dico_val = dico
                .read()
                .expect("RwLock poisoned");
            match this_dico_val.get(&string_array[1].1[0].to_string())
            {
                Some(valeur) => {
                    match valeur.len() {
                        1 => {writer.write(pair_to_redis(valeur[0].clone()).as_bytes()).unwrap();}
                        taille => {writer.write(array_to_redis_get(valeur, taille).as_bytes()).unwrap();}
                    }
                }
                None => {writer.write(b"_\r\n").unwrap();}
            }
        }
        taille => { // Array
            let mut vec: Vec<(String, Vec<String>)> = vec![];
            let this_dico_val = dico
                .read()
                .expect("RwLock poisoned");
            for i in 0..taille  // Pour chaque clef
            {
                match this_dico_val.get(&string_array[i].1[0].to_string())
                {
                    Some(valeur) => {
                        for j in 0..valeur.len()  // If array of arrays (should not happen)
                        {
                            vec.push((valeur[j].0.clone(), valeur[j].1.clone()));
                        }
                    }
                    None => {vec.push(("Null".to_string(), vec![String::new()]));}
                }
            }
            writer.write(array_to_redis(&vec.clone(), vec.len()).as_bytes()).unwrap();
        }
    }
}


/// Function handling del command
///
///
/// # Example
///
/// (with &my_vec = [("BulkString", ["DEL"]), ("BulkString", ["a"]), ("BulkString", ["b"])]]
///     a having "coucou" for entry and b not having one
/// command_set(&mut writer, &my_vec)
/// sends (integer) 1 towards client and removes a's entry from dico
pub fn command_del(string_array: &Vec<(String, Vec<String>)>, dico: &Arc<RwLock<HashMap<String, Vec<(String, Vec<String>)>>>>,
               writer: &mut BufWriter<&TcpStream>)
{
    let mut elt_enleves = 0;
    match string_array.len() {
        0 => {writer.write("-Error (Should not be reachable))\r\n".as_bytes()).unwrap();}
        1 => {writer.write("-ERROR: No key given\r\n".as_bytes()).unwrap();}
        2 => {
            // Access shared structure
            let mut this_dico_val = dico
                .write()
                .expect("RwLock poisoned");
            match this_dico_val.remove(&string_array[1].1[0].to_string()) {
                Some(_value) => {elt_enleves = elt_enleves + 1;}
                None => {}
            }
        }
        taille => { // Array
            let mut this_dico_val = dico
                .write()
                .expect("RwLock poisoned");
            for i in 1..taille
            {
                match this_dico_val.remove(&string_array[i].1[0].to_string()) {
                Some(_value) => {elt_enleves = elt_enleves + 1;}
                None => {}
                }
            }
        }
    }

    writer.write(pair_to_redis(("SimpleInteger".to_string(), vec![elt_enleves.to_string()])).as_bytes()).unwrap();
}


/// This command saves dico on the disk at path
///
/// NOTE: The function writes on path and doesn't append, so it may be wise to use a temporary file when saving
pub fn command_save(path: String,
                    dico: &Arc<RwLock<HashMap<String, Vec<(String, Vec<String>)>>>>,
                    writer: &mut BufWriter<&TcpStream>)
{
    println!("Saving, don't shutdown the server!");
    let file = OpenOptions::new()
                    .read(true)
                    .write(true)
                    .append(false)
                    .open(path).unwrap();
    let this_dico_val = dico
            .read()
            .expect("RwLock poisoned");

    file.set_len(0).unwrap();

    match serde_json::to_writer(file, &*this_dico_val)
    {
        Ok(_ok) => {
            println!("Save complete!");
            writer.write(b"+OK\r\n").unwrap();
        }
        Err(_e) => {
            writer.write(b"-ERROR: couldn't save!\r\n").unwrap();
        }
    }
}


/// Sends an error
pub fn trash(writer: &mut BufWriter<&TcpStream>) {writer.write(b"-ERROR: This command is not handled atm.\r\n").unwrap();}




