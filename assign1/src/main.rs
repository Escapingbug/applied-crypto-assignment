mod sm3;

use hex::{decode, encode};
use std::env;

fn main() {
    let args = env::args().collect::<Vec<String>>();
    if args.len() < 3 {
        eprintln!("usage: {} HASH EXTRA_MESSAGE", args[0]);
        return;
    }
    let hash = &args[1];
    let extra = &args[2];

    let hash_vec = decode(hash).expect("hash is not hex encoded");
    if hash_vec.len() != 32 {
        eprintln!("hash is not 256 bits, incorrect hash");
        return;
    }

    let new_hash = sm3::expansion_attack(&hash_vec, &extra.bytes().collect::<Vec<u8>>());
    println!("{}", encode(&new_hash));
}
