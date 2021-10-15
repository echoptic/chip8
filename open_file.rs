use std::{
    fs::{self, File},
    io::Write,
};

fn main() {
    let mut file = File::create("test").unwrap();
    file.write(&[1, 0xff]).unwrap();
    write!(file, "ا");
    let contents = fs::read_to_string("test").unwrap();
    println!("file contents: {:?}", contents);
}
