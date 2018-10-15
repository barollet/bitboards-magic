extern crate find_folder;
extern crate rayon;

use rayon::prelude::*;

use find_folder::Search;

use std::io::{Read, BufReader};
use std::env;
use std::path::PathBuf;
use std::fs::File;

fn main() {

    // Reading arguments
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        eprintln!("Too many arguments.");
        print_help();
        std::process::exit(1);
    }

    // Creating the magic folder if it doesn't exist
    let magic_path = match Search::Parents(3).for_folder("magic") {
        Ok(path) => path,
        Err(_) => {
            println!("No magic folder found, create one with ./gen_magic n");
            std::process::exit(1);
        }
    };

    // bishop magic
    let vec: Vec<u8> = (0..64).collect();
    vec.par_iter().for_each(|square| {
        match load_file_from_type_square(*square, &magic_path, true) {
            Ok(file) => {
                let mut contents = String::new();
                let mut buf_reader = BufReader::new(file);
                buf_reader.read_to_string(&mut contents).is_ok();

                let min_array_size: u32 = contents.split('\n').fold(u32::max_value(), |min, line| {
                    let vec: Vec<_> = line.split_whitespace().collect();
                    if vec.len() > 2 {
                        std::cmp::min(min, vec[2].parse::<u32>().unwrap())
                    } else {
                        min
                    }
                });
                println!("bishop {} done. Array size: {}", get_square_name(*square), min_array_size);
            },
            Err(e) => eprintln!("Can't open file for {} on {} {}", "bishop", get_square_name(*square), e),
        }
    });

    // rook magic
    let vec: Vec<u8> = (0..64).collect();
    vec.par_iter().for_each(|square| {
        match load_file_from_type_square(*square, &magic_path, false) {
            Ok(file) => {
                let mut contents = String::new();
                let mut buf_reader = BufReader::new(file);
                buf_reader.read_to_string(&mut contents).is_ok();

                let min_array_size: u32 = contents.split('\n').fold(u32::max_value(), |min, line| {
                    let vec: Vec<_> = line.split_whitespace().collect();
                    if vec.len() > 2 {
                        std::cmp::min(min, vec[2].parse::<u32>().unwrap())
                    } else {
                        min
                    }
                });
                println!("rook {} done. Array size: {}", get_square_name(*square), min_array_size);
            },
            Err(e) => eprintln!("Can't open file for {} on {} {}", "rook", get_square_name(*square), e),
        }
    });
}

fn load_file_from_type_square(square: u8, path: &PathBuf, bishop: bool) -> Result<File, std::io::Error> {

    let mut name = String::with_capacity(4);
    name.push(if bishop {'b'} else {'r'});
    name.push('_');
    push_square_name(&mut name, square);

    let mut path = path.join(name);
    path.set_extension("csv");

    File::open(path)
}

fn push_square_name(name: &mut String, square: u8) {
    name.push((('a' as u8) + (square % 8)) as char);
    name.push(std::char::from_digit(square as u32 / 8 + 1, 10).unwrap());
}

fn get_square_name(square: u8) -> String {
    let mut result = String::with_capacity(2);
    push_square_name(&mut result, square);
    result
}

fn print_help() {
    eprintln!("Usage: ./magic_management");
}
