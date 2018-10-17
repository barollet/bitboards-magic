extern crate array_init;
extern crate find_folder;
extern crate rayon;

use rayon::prelude::*;

use find_folder::Search;

use std::io::{Read, BufReader};
use std::env;
use std::path::PathBuf;
use std::fs::File;

struct MagicEntry {
    magic_entry: u64,
    width: u32,
    holes: Vec<u32>,
}

impl MagicEntry {
    fn new_from_line(line: Vec<&str>) -> MagicEntry {
        MagicEntry {
            magic_entry: line[3].parse::<u64>().unwrap(),
            width: line[2].parse::<u32>().unwrap(),
            holes: Vec::new(),
        }
    }
}

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

    let mut rook_tables: [Vec<MagicEntry>; 64] = array_init::array_init(|_i| Vec::new());
    let mut bishop_tables: [Vec<MagicEntry>; 64] = array_init::array_init(|_i| Vec::new());

    load_file_content_into_table(&mut bishop_tables, &magic_path, true);
    load_file_content_into_table(&mut rook_tables, &magic_path, false);

    println!("Table loaded, start generating the final table");

}

fn load_file_content_into_table(table: &mut [Vec<MagicEntry>; 64], path: &PathBuf, bishop: bool) {
    table.par_iter_mut().enumerate().for_each(|(square, magic_vec)| {
        match load_file_from_type_square(square as u8, path, bishop) {
            Ok(file) => {
                let mut contents = String::new();
                let mut buf_reader = BufReader::new(file);
                buf_reader.read_to_string(&mut contents).is_ok();

                for line in contents.split('\n') {
                    let line_vec: Vec<_> = line.split_whitespace().collect();
                    if line_vec.len() > 3 {
                        magic_vec.push(MagicEntry::new_from_line(line_vec));
                    }
                }
            },
            Err(e) => eprintln!("Can't open file for {} on {} {}", if bishop {"bishop"} else {"rook"}, get_square_name(square as u8), e),
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
    eprintln!("Usage: ./table_creation");
}
