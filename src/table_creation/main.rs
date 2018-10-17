extern crate array_init;
extern crate find_folder;
extern crate rayon;

use rayon::prelude::*;

use find_folder::Search;

use std::io::{Read, BufReader};
use std::env;
use std::path::PathBuf;
use std::fs::File;

#[derive(Debug)]
struct MagicEntry {
    magic_factor: u64,
    min: u32,
    width: u32,
    holes: Vec<Hole>,
}

#[derive(Debug)]
struct Hole {
    size: usize,
    position: usize,
}

impl Hole {
    fn new(size: usize, position: usize) -> Hole {
        Hole {
            size,
            position,
        }
    }
}

impl MagicEntry {
    fn new_from_line(line: Vec<&str>) -> MagicEntry {
        MagicEntry {
            magic_factor: line[3].parse::<u64>().unwrap(),
            min: match line[0].parse::<u32>() {
                Ok(n) => n,
                Err(e) => 12, // mdr
            },
            width: line[2].parse::<u32>().unwrap(),
            holes: Vec::new(),
        }
    }

    fn similarity(&self, other: &MagicEntry) -> f64 {
        0
    }

    fn find_holes(&mut self, bishop: bool, square: u8) {
        
        let mask = if bishop {
            get_bishop_mask(square)
        } else {
            get_rook_mask(square)
        };

        let n = mask.count_ones();
        let mut offsets: [bool; 4096] = [false; 4096];

        // finding offsets
        for i in 0..1<<n {
            let key = index_to_u64(i, n, mask) | !mask;
            let offset = get_fixed_offset(key, self.magic_factor);

            offsets[offset] = true;
        }

        // finding holes
        let mut i = 0;

        while i < 4096 {
            let hole_start = i;
            while i < 4096 && !offsets[i] {
                i += 1;
            }
            if i - hole_start > 50 {
                self.holes.push(Hole::new(i - hole_start, i));
            }
            while i < 4096 && offsets[i] {
                i += 1;
            }
        }
    }
}

fn direction_blockers_mask<F, G>(result: &mut u64, blockers: u64, mut kl: (u8, u8), update: F, check_bounds: G) 
    where F: Fn((u8, u8)) -> (u8, u8),
          G: Fn((u8, u8)) -> bool,
{
    while check_bounds(kl) {
        kl = update(kl);
        *result |= 1 << (8*kl.0 + kl.1);
        if blockers & 1 << (8*kl.0 + kl.1) != 0 {
            break
        }
    }
}

fn get_bishop_mask(square: u8) -> u64 {
    get_bishop_attack(square, 0)
}

fn get_bishop_attack(square: u8, blockers: u64) -> u64 {
    let mut result: u64 = 0;
    let ij = (square / 8, square % 8);

    direction_blockers_mask(&mut result, blockers, ij, |(k, l)| (k+1, l+1), |(k, l)| k < 6 && l < 6);
    direction_blockers_mask(&mut result, blockers, ij, |(k, l)| (k+1, l-1), |(k, l)| k < 6 && l > 1);
    direction_blockers_mask(&mut result, blockers, ij, |(k, l)| (k-1, l+1), |(k, l)| k > 1 && l < 6);
    direction_blockers_mask(&mut result, blockers, ij, |(k, l)| (k-1, l-1), |(k, l)| k > 1 && l > 1);

    result
}

fn get_rook_mask(square: u8) -> u64 {
    get_rook_attack(square, 0)
}

fn get_rook_attack(square: u8, blockers: u64) -> u64 {
    let mut result: u64 = 0;
    let ij = (square / 8, square % 8);

    direction_blockers_mask(&mut result, blockers, ij, |(k, l)| (k+1, l), |(k, _l)| k < 6);
    direction_blockers_mask(&mut result, blockers, ij, |(k, l)| (k-1, l), |(k, _l)| k > 1);
    direction_blockers_mask(&mut result, blockers, ij, |(k, l)| (k, l+1), |(_k, l)| l < 6);
    direction_blockers_mask(&mut result, blockers, ij, |(k, l)| (k, l-1), |(_k, l)| l > 1);

    result
}
fn pop_1st_bit(mask: &mut u64) -> u32 {
    let j = mask.trailing_zeros();
    *mask &= *mask -1;
    j
}

fn index_to_u64(index: usize, bits: u32, mut mask: u64) -> u64 {
    let mut result = 0;
    for i in 0..bits {
        let j = pop_1st_bit(&mut mask);
        if index & (1 << i) != 0 {
            result |= 1u64 << j;
        }
    }
    result
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

    /*
    for square in 0..64 {
        for entry in rook_tables[square].iter_mut() {
            entry.find_holes(false, square as u8);
        }
        for entry in bishop_tables[square].iter_mut() {
            entry.find_holes(true, square as u8);
        }
        println!("{} done", square);
    }
    */

    println!("Holes found");

    println!("{:?}", rook_tables[0][0]);

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
