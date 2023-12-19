/// Transforms received buffer (bytes) into a String
fn buffer_to_string(buf: & Vec<u8>, debut: usize) -> String {
    let length = buf.len();
    let mut word = "".to_string();

    for i in debut..length {
        match buf[i] {
            13 => { // match for CRLF
                if buf[i + 1] == 10
                  {
                    return word;
                  }
                else
                {
                  // Maybe panic isn't the best way to handle the error,
                  // but that's a way I guess
                  panic!("Invalid argument!\n");
                }
            }
            0 => { // match for end of file
                if word != ""
                {
                    break;
                }
            }
            other => {
                // increase the word
                word.push(other as char);
                if i == length - 1 {
                }
            }
        }
    }
    word
}


/// Transforms received buffer (bytes) into a String in a different way
/// => Checks if sign is in first position or not in buffer
fn buffer_to_string_integer(buf: & Vec<u8>, debut: usize) -> String {
  let length = buf.len();
  let mut word = "".to_string();

  for i in debut..length {
      match buf[i] {
        b'+' => {
          if i == debut
          {word.push('+');}
          else
          {panic!("Invalid argument!\n")}
        }
        b'-' => {
          if i == debut
          {word.push('-');}
          else
          {panic!("Invalid argument!\n")}
        }

        13 => { // match for CRLF
            if buf[i + 1] == 10
              {return word;}
            else
            {panic!("Invalid argument!\n");}
        }
        0 => { // match for end of file
            if word != ""
            {break;}
        }
        other => {
            // increase the word
            word.push(other as char);
        }
      }
  }
  word
}


/// Calculate number of elements in buf before CRLF
fn size_to_crlf(buf: & Vec<u8>, debut: usize) -> usize {
  let length = buf.len();
  for i in debut..length {
    match buf[i] {
      13 => { // match for CRLF
        if buf[i + 1] == 10
          {return i + 2;}
        else
          {panic!("Invalid argument!\n");}
      }
      0 => { // match for end of file
        {panic!("Not what we want or buffer too small!\n");}
      }
      _other => {}
    }
  }
  panic!("Not what we want or buffer too small!\n");
}


/// Creates a pair for SimpleString
fn simple_string(buf: & Vec<u8>, i: usize) -> (String, Vec<String>)
{
  ("SimpleString".to_string(), vec![buffer_to_string(&buf, i)])
}


/// Creates a pair for SimpleError
fn simple_error(buf: & Vec<u8>, i: usize) -> (String, Vec<String>)
{
  ("SimpleError".to_string(), vec![buffer_to_string(&buf, i)])
}


/// Creates a pair for SimpleInteger
fn simple_integer(buf: & Vec<u8>, i: usize) -> (String, Vec<String>)
{
  ("SimpleInteger".to_string(), vec![buffer_to_string_integer(&buf, i)])
}


/// Creates a pair for BulkString
fn bulk_string(buf: & Vec<u8>, i: usize) -> (String, Vec<String>)
{
  let new_i = size_to_crlf(buf, i+1);
  ("BulkString".to_string(), vec![buffer_to_string(&buf, new_i)])
}


/// Converts Redis-complient protocol buffer to my-program-complient vector
///
///
/// My format is weeeeeird but hey, it works!
///
/// # Format:
///   [("Array", [(Element_type, [Element] ) ]
///
///   Basically it's an array of elements where each element is an array
///   If an element is not supposed to be an array, it will be said in Element_type
///     and the array will be of size 1.
///
///   This format was meant to make handling arrays easier but it's kinda hard to use lol.
pub fn redis_to_array(buf: & Vec<u8>) -> Vec<(String, Vec<(String, Vec<String>)>)> {

  let mut grand_vec: Vec<(String, Vec<(String, Vec<String>)>)> = vec![]; // Le vec de retour
  let mut vec: Vec<(String, Vec<String>)> = vec![];                      // Le vec dans celui de retour
  let length = buf.len();

  // Compteur pour savoir combien de choses sauter après avoir commencé un array
  let mut compteur = 0;

  for i in 0..length {
    match buf[i] {  // Detect type and create pair accordingly
      b'+' => {
        vec.push(simple_string(&buf, i));
        if compteur == 0
        {
          grand_vec.push(("Array".to_string(), vec.clone()));
          vec.clear();
        }
        compteur = compteur - 1;
      }

      b'-' => {
        vec.push(simple_error(&buf, i));
        if compteur == 0
        {
          grand_vec.push(("Array".to_string(), vec.clone()));
          vec.clear();
        }
        compteur = compteur - 1;
      }

      b':' => {
        vec.push(simple_integer(&buf, i));
        if compteur == 0
        {
          grand_vec.push(("Array".to_string(), vec.clone()));
          vec.clear();
        }
        compteur = compteur - 1;
      }

      b'$' => {
        vec.push(bulk_string(&buf, i));
        if compteur == 0
        {
          grand_vec.push(("Array".to_string(), vec.clone()));
          vec.clear();
        }
        compteur = compteur - 1;
      }

      b'_' => {
        if buf[i+1] == 13 && buf[i+2] == 10
        {
          vec.push(("Null".to_string(), vec!["".to_string()]));
          if compteur == 0
          {
            grand_vec.push(("Array".to_string(), vec.clone()));
            vec.clear();
          }
        }
        else
        {panic!("Not what we want or buffer too small!\n");}
      }

      b'#' => {
        vec.push(("Bool".to_string(), vec![buf[i+1].to_string()]));
        if compteur == 0
        {
          grand_vec.push(("Array".to_string(), vec.clone()));
          vec.clear();
        }
      }

      b'*' => {
        if compteur == 0
          // La valeur est en byte => conversion en string => conversion en entier
          {compteur = buf[i+1].to_string().parse::<usize>().unwrap();}
        else
          {panic!("Array of arrayw not supported!");}
      }

      13 => {
        if buf[i+1] == 10
        {}
        else {panic!();}
      }

      10 => {
        if buf[i-1] == 13
        {}
        else {panic!();}
      }

      0 => {break}

      _other => { /* We create words in other categories so here we don't want to do anything */ }
    }
  }
  if vec != vec![]
    {
      grand_vec.push(("Array".to_string(), vec));
    }
  return grand_vec;
}


/// Similar function than redis_to_array but for inline commands
///
/// NOTE: EXPERIMENTAL!!! Does not work properly yet. Only usable for PING
pub fn inline_redis(buf: & Vec<u8>) -> Vec<(String, Vec<(String, Vec<String>)>)> {
  let mut grand_vec: Vec<(String, Vec<(String, Vec<String>)>)> = vec![]; // Le vec de retour
  let mut vec: Vec<(String, Vec<String>)> = vec![];                      // Le vec dans celui de retour
  let length = buf.len();

  let mut string = String::from("");

  // On crée un String qu'on fout dans le vecteur
  // Quid des types ?

  for i in 0..length {
    match buf[i] {
      b' ' => {
        vec.push(("SimpleString".to_string(), vec![string]));
        string = String::from("");
      }

      13 => {
        break;
      }

      10 => {
        break
      }

      0 => {break;}

      other => {string.push(other as char);}
    }
  }
  // BulkString plutôt que Simple ? Ça serait probablement plus puissant. À voir.
  vec.push(("SimpleString".to_string(), vec![string]));
  if vec != vec![]
  {
    grand_vec.push(("Array".to_string(), vec));
  }
  return grand_vec;
}


/// Transforms a pair of my elements to a Redis-compliant String
///
/// # Example
///
/// in: ("Integer", ["25"])
/// out: ":25"
///
/// NOTE: I need to check if it works with negative values.
pub fn pair_to_redis(entry: (String, Vec<String>)) -> String {
  let crlf: &str = "\r\n";

  match entry.0.as_str() {

    // So like, maybe put + instead of SimpleString to make it quicker ?
    // Meh, less readable for me ^^
    "SimpleString" => {
      let mut retour: String = "+".to_owned();
      for i in 0..entry.1.len()
        {retour.push_str(entry.1[i].as_str());}
      retour.push_str(crlf);
      return retour;
    }

    "SimpleError" => {
      let mut retour: String = "-".to_owned();
      for i in 0..entry.1.len()
        {retour.push_str(entry.1[i].as_str());}
      retour.push_str(crlf);
      return retour;
    }

    "SimpleInteger" => {
      // WARNING: Hasn't been tested with negative values!
      let mut retour: String = ":".to_owned();
      for i in 0..entry.1.len()
        {retour.push_str(entry.1[i].as_str());}
      retour.push_str(crlf);
      return retour;
    }

    "BulkString" => {
      let mut retour: String = "$".to_owned();
      retour.push_str(entry.1[0].len().to_string().as_str());
      retour.push_str(crlf);
      retour.push_str(entry.1[0].as_str());
      retour.push_str(crlf);
      return retour;
    }

    "Null" => {
      return "_\r\n".to_string()
    }

    "Bool" => {
      let mut retour: String = "#".to_owned();
      for i in 0..entry.1.len()
        {retour.push_str(entry.1[i].as_str());}
      retour.push_str(crlf);
      return retour;
    }

    _other => {
      panic!("?????????????");
    }

  }
}


/// Transforms an array in a Redis-ready String
///
/// It's basically a bunch of pair_to_redis one after another.
pub fn array_to_redis(string_array: &Vec<(String, Vec<String>)>, taille: usize) -> String
{
  let mut retour: String = String::from("*");
  retour.push_str((taille-1).to_string().as_str());
  retour.push_str("\r\n");
  for i in 1..taille
  {
    // string_array: [(Type, <elt1, elt2...>), (Type, <elt1, elt2...>), ...]
    retour.push_str(pair_to_redis(string_array[i].clone()).as_str());
  }
  return retour;
}


/// Same as array_to_redis but for the get command (ranges are different)
pub fn array_to_redis_get(string_array: &Vec<(String, Vec<String>)>, taille: usize) -> String
{
  let mut retour: String = String::from("*");
  retour.push_str((taille).to_string().as_str());
  retour.push_str("\r\n");
  for i in 0..taille
  {
    // string_array: [(Type, <elt1, elt2...>), (Type, <elt1, elt2...>), ...]
    retour.push_str(pair_to_redis(string_array[i].clone()).as_str());
  }
  return retour;
}
