extern crate rand;

static B_BITS: [u8; 64] = [
  6, 5, 5, 5, 5, 5, 5, 6,
  5, 5, 5, 5, 5, 5, 5, 5,
  5, 5, 7, 7, 7, 7, 5, 5,
  5, 5, 7, 9, 9, 7, 5, 5,
  5, 5, 7, 9, 9, 7, 5, 5,
  5, 5, 7, 7, 7, 7, 5, 5,
  5, 5, 5, 5, 5, 5, 5, 5,
  6, 5, 5, 5, 5, 5, 5, 6
];

static R_BITS: [u8; 64] = [
  12, 11, 11, 11, 11, 11, 11, 12,
  11, 10, 10, 10, 10, 10, 10, 11,
  11, 10, 10, 10, 10, 10, 10, 11,
  11, 10, 10, 10, 10, 10, 10, 11,
  11, 10, 10, 10, 10, 10, 10, 11,
  11, 10, 10, 10, 10, 10, 10, 11,
  11, 10, 10, 10, 10, 10, 10, 11,
  12, 11, 11, 11, 11, 11, 11, 12
];

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

fn get_offset(key: u64, magic: u64, shift: u8) -> usize {
    (key.overflowing_mul(magic).0 >> (64 - shift)) as usize
}

fn find_magic(square: u8, bishop: bool) -> u64 {

    let mut keys: [u64; 4096] = [0; 4096];
    let mut attacks: [u64; 4096] = [0; 4096];
    let mut used: [u64; 4096] = [0; 4096];

    let mask = if bishop {
        get_bishop_mask(square)
    } else {
        get_rook_mask(square)
    };

    let n = mask.count_ones();

    let m = if bishop {
        B_BITS[square as usize]
    } else {
        R_BITS[square as usize]
    };

    for i in 0..(1<<n) {
        keys[i] = index_to_u64(i, n, mask);
        attacks[i] = if bishop {
            get_bishop_attack(square, keys[i])
        } else {
            get_rook_attack(square, keys[i])
        }
    }

    // Testing 100 000 000 magic factors
    for _ in 0..100_000_000 {
        let magic = gen_magic();
        if (mask.overflowing_mul(magic).0 & 0xFF00000000000000).count_ones() < 6 {
            continue;
        }
        for i in 0..4096 {
            used[i] = 0;
        }

        let mut fail = false;
        for i in 0..(1 << n) {
            let j = get_offset(keys[i], magic, m);
            if used[j] == 0 {
                used[j] = attacks[i];
            } else if used[j] != attacks[i] {
                fail = true;
                break;
            }
        }
        if !fail {
            println!("{}", magic);
            return magic;
        }
    }

    println!("=== Failed ===");

    0
}

fn main() {

    println!("bishop");
    for square in 0..64 {
        println!("square {}", square);
        find_magic(square, true);
    }

    println!("rook");
    for square in 0..64 {
        println!("square {}", square);
        find_magic(square, false);
    }
}

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
