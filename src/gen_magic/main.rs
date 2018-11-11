extern crate rand;
extern crate find_folder;
extern crate rayon;

use rayon::prelude::*;

use find_folder::Search;

use std::io::Write;
use std::env;
use std::path::PathBuf;
use std::fs::{OpenOptions, File, DirBuilder};

fn gen_magic() -> u64 {
    let mut magic = u64::max_value();
    for _ in 0..3 {
        magic &= rand::random::<u64>();
    }
    magic
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
    get_bishop_key(square, 0)
}

fn get_bishop_key(square: u8, blockers: u64) -> u64 {
    let mut result: u64 = 0;
    let ij = (square / 8, square % 8);

    direction_blockers_mask(&mut result, blockers, ij, |(k, l)| (k+1, l+1), |(k, l)| k < 6 && l < 6);
    direction_blockers_mask(&mut result, blockers, ij, |(k, l)| (k+1, l-1), |(k, l)| k < 6 && l > 1);
    direction_blockers_mask(&mut result, blockers, ij, |(k, l)| (k-1, l+1), |(k, l)| k > 1 && l < 6);
    direction_blockers_mask(&mut result, blockers, ij, |(k, l)| (k-1, l-1), |(k, l)| k > 1 && l > 1);

    result
}

fn get_bishop_attack(square: u8, blockers: u64) -> u64 {
    let mut result: u64 = 0;
    let ij = (square / 8, square % 8);

    direction_blockers_mask(&mut result, blockers, ij, |(k, l)| (k+1, l+1), |(k, l)| k < 7 && l < 7);
    direction_blockers_mask(&mut result, blockers, ij, |(k, l)| (k+1, l-1), |(k, l)| k < 7 && l > 0);
    direction_blockers_mask(&mut result, blockers, ij, |(k, l)| (k-1, l+1), |(k, l)| k > 0 && l < 7);
    direction_blockers_mask(&mut result, blockers, ij, |(k, l)| (k-1, l-1), |(k, l)| k > 0 && l > 0);

    result
}

fn get_rook_mask(square: u8) -> u64 {
    get_rook_key(square, 0)
}

fn get_rook_key(square: u8, blockers: u64) -> u64 {
    let mut result: u64 = 0;
    let ij = (square / 8, square % 8);

    direction_blockers_mask(&mut result, blockers, ij, |(k, l)| (k+1, l), |(k, _l)| k < 6);
    direction_blockers_mask(&mut result, blockers, ij, |(k, l)| (k-1, l), |(k, _l)| k > 1);
    direction_blockers_mask(&mut result, blockers, ij, |(k, l)| (k, l+1), |(_k, l)| l < 6);
    direction_blockers_mask(&mut result, blockers, ij, |(k, l)| (k, l-1), |(_k, l)| l > 1);

    result
}

fn get_rook_attack(square: u8, blockers: u64) -> u64 {
    let mut result: u64 = 0;
    let ij = (square / 8, square % 8);

    direction_blockers_mask(&mut result, blockers, ij, |(k, l)| (k+1, l), |(k, _l)| k < 7);
    direction_blockers_mask(&mut result, blockers, ij, |(k, l)| (k-1, l), |(k, _l)| k > 0);
    direction_blockers_mask(&mut result, blockers, ij, |(k, l)| (k, l+1), |(_k, l)| l < 7);
    direction_blockers_mask(&mut result, blockers, ij, |(k, l)| (k, l-1), |(_k, l)| l > 0);

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

#[allow(dead_code)]
fn get_offset(key: u64, magic: u64, shift: u32) -> usize {
    (key.overflowing_mul(magic).0 >> (64 - shift)) as usize
}

fn write_magic_to_file(file: &mut File, square: u8, bishop: bool) -> Result<u64, String> {

    let mut keys: [u64; 4096] = [0; 4096];
    let mut attacks: [u64; 4096] = [0; 4096];
    let mut used: [u64; 4096] = [0; 4096];

    let mask = if bishop {
        get_bishop_mask(square)
    } else {
        get_rook_mask(square)
    };

    let n = mask.count_ones();

    for i in 0..(1<<n) {
        keys[i] = index_to_u64(i, n, mask) | !mask;
        attacks[i] = if bishop {
            get_bishop_attack(square, keys[i])
        } else {
            get_rook_attack(square, keys[i])
        }
    }

    // Testing 100 000 000 magic factors
    for _ in 0..100_000_000{
        let magic = gen_magic();
        if (mask.overflowing_mul(magic).0 & 0xFF00000000000000).count_ones() < 6 {
            continue;
        }
        for i in 0..4096 {
            used[i] = 0;
        }

        let mut fail = false;
        let mut max_j = 0;
        let mut min_j = 4096;
        for i in 0..(1 << n) {
            //let j = get_offset(keys[i], magic, n);
            let j = get_fixed_offset(keys[i], magic);
            max_j = std::cmp::max(j, max_j);
            min_j = std::cmp::min(j, min_j);
            if used[j] == 0 {
                used[j] = attacks[i];
            } else if used[j] != attacks[i] {
                fail = true;
                break;
            }
        }
        /*
        if max_j - min_j > 2000 {
            continue
        }
        */
        if !fail {
            file.write(format!("{} {} {} {}\n", min_j, max_j, max_j-min_j, magic).as_bytes()).is_ok();
            return Ok(magic);
        }
    }

    Err(format!("=== Failed === for {} on {}",
                if bishop {"bishop"} else {"rook"},
                get_square_name(square)))
}

fn main() {

    // Reading arguments
    let args: Vec<String> = env::args().collect();
    if args.len() > 3 {
        eprintln!("Too many arguments.");
        print_help();
        std::process::exit(1);
    }

    let magic_size = match args[1].parse::<u64>() {
        Ok(n) => n,
        Err(e) => {
            eprintln!("{}", e);
            print_help();
            std::process::exit(1)
        }
    };

    // Creating the magic folder if it doesn't exist
    let magic_path = match Search::Parents(3).for_folder("magic") {
        Ok(path) => path,
        Err(_) => {
            println!("Created magic folder in the current directory");
            DirBuilder::new().create("magic").unwrap();
            Search::Parents(3).for_folder("magic").unwrap()
        }
    };

    if args.len() == 2 {
        // Searching magic for every square and type

        // Bishop magic
        let vec: Vec<u8> = (0..64).collect();
        /*
        vec.par_iter().for_each(|square| {
            let mut f = load_file_from_type_square(*square, &magic_path, true);
            for _ in 0..magic_size {
                write_magic_to_file(&mut f, *square, true).is_ok();
            }
            println!("bishop {} done", get_square_name(*square));
        });
        */

        // Rook magic
        vec.par_iter().for_each(|square| {
            let mut f = load_file_from_type_square(*square, &magic_path, false);
            for _ in 0..magic_size {
                write_magic_to_file(&mut f, *square, false).is_ok();
            }
            println!("rook {} done", get_square_name(*square));
        });

    } else {
        let square = square_from_name(&args[2]);
        // Hardcoded bishop magic
        let vec: Vec<u64> = (0..magic_size).collect();
        vec.par_iter().for_each(|_| {
            let mut f = load_file_from_type_square(square, &magic_path, true);
            write_magic_to_file(&mut f, square, true).is_ok();
        });
    }
}

fn load_file_from_type_square(square: u8, path: &PathBuf, bishop: bool) -> File {

    let mut name = String::with_capacity(4);
    name.push(if bishop {'b'} else {'r'});
    name.push('_');
    push_square_name(&mut name, square);

    let mut path = path.join(name);
    path.set_extension("csv");

    let file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(path)
        .unwrap();

    file
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

fn square_from_name(name: &str) -> u8 {
    let mut chars = name.chars();
    let mut square = 0;

    if let Some(letter) = chars.next() {
        square += (letter as u8) - ('a' as u8);
    }
    if let Some(digit) = chars.next() {
        square += 8*((digit as u8) - ('1' as u8));
    }
    square
}

fn print_help() {
    eprintln!("Usage: ./gen_magic number_of_magics (specific square)");
    eprintln!("Examples: ./gen_magic 1000");
    eprintln!("          ./gen_magic 1000 a2");
}

#[allow(dead_code)]
fn print_bitboard_mask(u: u64) {
    for i in (0..8).rev() {
        let line: u8 = ((u >> (8*i)) & 0xff) as u8;
        for j in 0..8 {
            print!("{}", (line >> j) & 0x1);
        }
        println!("");
    }
    println!("");
}
