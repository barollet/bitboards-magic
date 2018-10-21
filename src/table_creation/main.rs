extern crate array_init;
extern crate find_folder;
extern crate rayon;
extern crate itertools;

use rayon::prelude::*;
use itertools::Itertools;

use find_folder::Search;

use std::env;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::PathBuf;

#[derive(Debug, Copy, Clone)]
struct MagicEntry {
    magic_factor: u64,
    min: u32,
    width: u32,
}

impl MagicEntry {
    fn new_from_line(line: &[&str]) -> MagicEntry {
        MagicEntry {
            magic_factor: line[3].parse::<u64>().unwrap(),
            min: match line[0].parse::<u32>() {
                Ok(n) => n,
                Err(_e) => 12, // mdr
            },
            width: line[2].parse::<u32>().unwrap(),
        }
    }

    fn shared_size(&self, other: &MagicEntry) -> u32 {
        let max = std::cmp::max(self.min + self.width, other.min + other.width);
        let min = std::cmp::min(self.min, other.min);

        max - min
    }
}

#[allow(dead_code)]
fn get_fixed_offset(key: u64, magic: u64) -> usize {
    (key.overflowing_mul(magic).0 >> (64 - 12)) as usize
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

    let mut predicted_size = 0;

    // Rook tables
    for square in (0..64).step_by(2) {
        if (square / 8) % 2 != 0 {
            continue;
        }
        let mut min_shared_size = 4096;
        let mut start_offset = 0;
        let mut magic1 = 0;
        let mut magic2 = 0;
        for magic_entry in &rook_tables[square as usize] {
            for other_entry in &rook_tables[(square + 9 as usize)] {
                let shared_size = magic_entry.shared_size(&other_entry);
                if shared_size < min_shared_size {
                    min_shared_size = shared_size;

                    start_offset = std::cmp::min(magic_entry.min, other_entry.min);
                    magic1 = magic_entry.magic_factor;
                    magic2 = other_entry.magic_factor;
                }
            }
        }
        println!(
            "{} size: {} start: {} magic1: {} magic2: {}",
            square, min_shared_size, start_offset, magic1, magic2
        );
        predicted_size += min_shared_size;
    }

    for square in (1..64).step_by(2) {
        if (square / 8) % 2 != 0 {
            continue;
        }
        let mut min_shared_size = 4096;
        let mut start_offset = 0;
        let mut magic1 = 0;
        let mut magic2 = 0;
        for magic_entry in &rook_tables[square as usize] {
            for other_entry in &rook_tables[(square + 7 as usize)] {
                let shared_size = magic_entry.shared_size(&other_entry);
                if shared_size < min_shared_size {
                    min_shared_size = shared_size;

                    start_offset = std::cmp::min(magic_entry.min, other_entry.min);
                    magic1 = magic_entry.magic_factor;
                    magic2 = other_entry.magic_factor;
                }
            }
        }
        println!(
            "{} size: {} start: {} magic1: {} magic2: {}",
            square, min_shared_size, start_offset, magic1, magic2
        );
        predicted_size += min_shared_size;
    }

    // Bishop tables
    let bishop_sharing: [usize; 64] = [
        0,  2,  4,  4,  4,  4, 12, 14,
        0,  2,  5,  5,  5,  5, 12, 14,
        0,  2,  6,  6,  6,  6, 12, 14,
        0,  2,  7,  7,  7,  7, 12, 14,
        1,  3,  8,  8,  8,  8, 13, 15,
        1,  3,  9,  9,  9,  9, 13, 15,
        1,  3, 10, 10, 10, 10, 13, 15,
        1,  3, 11, 11, 11, 11, 13, 15,
    ];
    let mut bishop_shared: [[usize; 4]; 16] = [[0; 4]; 16];
    let mut indexes: [usize; 16] = [0; 16];
    for square in 0..64 {
        let sharing = bishop_sharing[square];
        bishop_shared[sharing][indexes[usize::from(sharing)]] = square;
        indexes[usize::from(sharing)] += 1;
    }

    println!("{:?}", bishop_shared);

    let mut i = 0;
    for squares in &bishop_shared {
        let min = squares.into_iter().map(|sq| bishop_tables[*sq].iter()).multi_cartesian_product()
            .min_by_key(|magic_entries|
                        magic_entries.iter().map(|m| m.min + m.width).max().unwrap() - magic_entries.iter().map(|m| m.min).min().unwrap()
                        ).unwrap();
        let start = min.iter().map(|m| m.min).min().unwrap();
        let size = min.iter().map(|m| m.min + m.width).max().unwrap() - min.iter().map(|m| m.min).min().unwrap();
        println!("{} {:?} {:?} {:?} {} {} {} {}", i, squares, start, size, min[0].magic_factor, min[1].magic_factor, min[2].magic_factor, min[3].magic_factor);
        i += 1;
    }

    println!("Shared size found");

    println!(
        "Predicted offset {}, size {}",
        predicted_size,
        predicted_size * 8
    );
}

fn load_file_content_into_table(table: &mut [Vec<MagicEntry>; 64], path: &PathBuf, bishop: bool) {
    table.par_iter_mut().enumerate().for_each(
        |(square, magic_vec)| match load_file_from_type_square(square as u8, path, bishop) {
            Ok(file) => {
                let mut contents = String::new();
                let mut buf_reader = BufReader::new(file);
                buf_reader.read_to_string(&mut contents).is_ok();

                for line in contents.split('\n') {
                    let line_vec: Vec<_> = line.split_whitespace().collect();
                    if line_vec.len() > 3 {
                        magic_vec.push(MagicEntry::new_from_line(&line_vec));
                    }
                }
            }
            Err(e) => eprintln!(
                "Can't open file for {} on {} {}",
                if bishop { "bishop" } else { "rook" },
                get_square_name(square as u8),
                e
            ),
        },
    );
}

fn load_file_from_type_square(
    square: u8,
    path: &PathBuf,
    bishop: bool,
) -> Result<File, std::io::Error> {
    let mut name = String::with_capacity(4);
    name.push(if bishop { 'b' } else { 'r' });
    name.push('_');
    push_square_name(&mut name, square);

    let mut path = path.join(name);
    path.set_extension("csv");

    File::open(path)
}

fn push_square_name(name: &mut String, square: u8) {
    name.push((b'a' + (square % 8)) as char);
    name.push(std::char::from_digit(u32::from(square) / 8 + 1, 10).unwrap());
}

fn get_square_name(square: u8) -> String {
    let mut result = String::with_capacity(2);
    push_square_name(&mut result, square);
    result
}

fn print_help() {
    eprintln!("Usage: ./table_creation");
}
