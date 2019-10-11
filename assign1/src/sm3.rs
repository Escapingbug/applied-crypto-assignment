//! # sm3 implementation
//! Based on https://tools.ietf.org/id/draft-oscca-cfrg-sm3-01.html
use std::mem::transmute;

const T: [u32; 64] = [
    0x79cc4519, 0x79cc4519, 0x79cc4519, 0x79cc4519, 0x79cc4519, 0x79cc4519, 0x79cc4519, 0x79cc4519,
    0x79cc4519, 0x79cc4519, 0x79cc4519, 0x79cc4519, 0x79cc4519, 0x79cc4519, 0x79cc4519, 0x79cc4519,
    0x7a879d8a, 0x7a879d8a, 0x7a879d8a, 0x7a879d8a, 0x7a879d8a, 0x7a879d8a, 0x7a879d8a, 0x7a879d8a,
    0x7a879d8a, 0x7a879d8a, 0x7a879d8a, 0x7a879d8a, 0x7a879d8a, 0x7a879d8a, 0x7a879d8a, 0x7a879d8a,
    0x7a879d8a, 0x7a879d8a, 0x7a879d8a, 0x7a879d8a, 0x7a879d8a, 0x7a879d8a, 0x7a879d8a, 0x7a879d8a,
    0x7a879d8a, 0x7a879d8a, 0x7a879d8a, 0x7a879d8a, 0x7a879d8a, 0x7a879d8a, 0x7a879d8a, 0x7a879d8a,
    0x7a879d8a, 0x7a879d8a, 0x7a879d8a, 0x7a879d8a, 0x7a879d8a, 0x7a879d8a, 0x7a879d8a, 0x7a879d8a,
    0x7a879d8a, 0x7a879d8a, 0x7a879d8a, 0x7a879d8a, 0x7a879d8a, 0x7a879d8a, 0x7a879d8a, 0x7a879d8a,
];

/// convet u8 to u32. sm3 works on big-endian, we assume
/// here we are little-endian
pub fn sm3_u8_to_u32(bytes: &[u8]) -> Vec<u32> {
    let mut m = Vec::new();
    let mut used = 0;

    for b in bytes.iter() {
        if m.len() < used / 32 + 1 {
            m.push(0);
        }
        m[used / 32] <<= 8;
        m[used / 32] |= (*b) as u32;
        used += 8;
    }

    m
}

/// convertion with consideration of little-endian -> bit-endian
/// convert
pub fn sm3_u32_to_u8(m: &[u32]) -> Vec<u8> {
    let mut v = Vec::new();
    for each in m.iter() {
        let cur: [u8; 4] = unsafe { transmute((*each).to_be()) };
        v.extend_from_slice(&cur);
    }

    v
}

fn pad(m: &[u8]) -> [u32; 16] {
    let l = m.len() as u64;
    let mut m_prime = [0u32; 16];
    let mut used = 0;
    for message in m.iter() {
        m_prime[used / 32] <<= 8;
        m_prime[used / 32] |= (*message) as u32;
        used += 8;
    }

    if used % 32 == 0 {
        m_prime[used / 32] = 0x80000000;
    } else {
        m_prime[used / 32] <<= 32 - (used % 32);
        m_prime[used / 32] |= 1 << (31 - (used % 32));
    }

    let bits_len = l * 8;

    m_prime[14] = (bits_len >> 32) as u32;
    m_prime[15] = bits_len as u32;
    m_prime
}

fn p_0(x: u32) -> u32 {
    x ^ (x.rotate_left(9) ^ x.rotate_left(17))
}

fn p_1(x: u32) -> u32 {
    x ^ (x.rotate_left(15) ^ x.rotate_left(23))
}

fn message_expansion(original_w: &[u32]) -> [u32; 132] {
    let mut ws = [0u32; 132];
    for i in 0..original_w.len() {
        ws[i] = original_w[i]
    }
    for j in 16..=67 {
        ws[j] = p_1(ws[j - 16] ^ ws[j - 9] ^ (ws[j - 3].rotate_left(15)))
            ^ (ws[j - 13].rotate_left(7))
            ^ ws[j - 6];
    }

    let mut w_primes = [0u32; 64];
    for j in 0..=63 {
        w_primes[j] = ws[j] ^ ws[j + 4];
    }

    for j in 68..132 {
        ws[j] = w_primes[j - 68];
    }

    ws
}

fn ff(j: u32) -> Box<dyn FnOnce(u32, u32, u32) -> u32> {
    if j <= 15 {
        Box::new(|x: u32, y: u32, z: u32| x ^ y ^ z)
    } else {
        Box::new(|x: u32, y: u32, z: u32| (x & y) | (x & z) | (y & z))
    }
}

fn gg(j: u32) -> Box<dyn FnOnce(u32, u32, u32) -> u32> {
    if j <= 15 {
        Box::new(|x: u32, y: u32, z: u32| x ^ y ^ z)
    } else {
        Box::new(|x: u32, y: u32, z: u32| (x & y) | ((!x) & z))
    }
}

fn compression_function(v: &[u32], ei: &[u32]) -> [u32; 8] {
    let v_clone = v.clone();
    let mut a = v_clone[0];
    let mut b = v_clone[1];
    let mut c = v_clone[2];
    let mut d = v_clone[3];
    let mut e = v_clone[4];
    let mut f = v_clone[5];
    let mut g = v_clone[6];
    let mut h = v_clone[7];

    let mut w = [0u32; 68];
    let mut w_prime = [0u32; 64];
    for i in 0..68 {
        w[i] = ei[i];
    }

    for i in 0..64 {
        w_prime[i] = ei[i + 68];
    }

    for j in 0u32..=63 {
        let ss1 = a
            .rotate_left(12)
            .overflowing_add(e)
            .0
            .overflowing_add(T[j as usize].rotate_left(j % 32))
            .0
            .rotate_left(7);
        let ss2 = ss1 ^ a.rotate_left(12);
        let tt1 = ff(j)(a, b, c)
            .overflowing_add(d)
            .0
            .overflowing_add(ss2)
            .0
            .overflowing_add(w_prime[j as usize])
            .0;
        let tt2 = gg(j)(e, f, g)
            .overflowing_add(h)
            .0
            .overflowing_add(ss1)
            .0
            .overflowing_add(w[j as usize])
            .0;
        d = c;
        c = b.rotate_left(9);
        b = a;
        a = tt1;
        h = g;
        g = f.rotate_left(19);
        f = e;
        e = p_0(tt2);
    }

    let o_a = v[0];
    let o_b = v[1];
    let o_c = v[2];
    let o_d = v[3];
    let o_e = v[4];
    let o_f = v[5];
    let o_g = v[6];
    let o_h = v[7];

    [
        a ^ o_a,
        b ^ o_b,
        c ^ o_c,
        d ^ o_d,
        e ^ o_e,
        f ^ o_f,
        g ^ o_g,
        h ^ o_h,
    ]
}

fn split_to_blocks(m: &[u8]) -> Vec<[u32; 16]> {
    let l = m.len();
    let mut blocks = Vec::new();
    for i in 0..(l / 64) {
        let mut block = [0u32; 16];
        // A B C D E F G H ...
        // [A B C D] [E F G H] ...
        for j in 0..64 {
            block[j / 4] |= (m[i * 64 + j] as u32) << ((j % 4) * 8);
        }
        blocks.push(block);
    }
    if l % 64 != 0 {
        blocks.push(pad(&m[(l - (l % 64))..]));
    }
    blocks
}

#[cfg(test)]
fn sm3(m: &[u8]) -> [u32; 8] {
    let blocks = split_to_blocks(m);
    let n = blocks.len();
    let mut v = [
        0x7380166f, 0x4914b2b9, 0x172442d7, 0xda8a0600, 0xa96f30bc, 0x163138aa, 0xe38dee4d,
        0xb0fb0e4e,
    ];

    for i in 0..=(n - 1) {
        let e = message_expansion(&blocks[i]);
        v = compression_function(&v, &e);
    }

    v
}

pub fn expansion_attack(original_hash: &[u8], extra: &[u8]) -> Vec<u8> {
    let expand_messages = split_to_blocks(extra);
    let hash_blocks = sm3_u8_to_u32(original_hash);
    let mut v = [
        hash_blocks[0],
        hash_blocks[1],
        hash_blocks[2],
        hash_blocks[3],
        hash_blocks[4],
        hash_blocks[5],
        hash_blocks[6],
        hash_blocks[7],
    ];
    for message in expand_messages.iter() {
        let e_i = message_expansion(message);
        v = compression_function(&v, &e_i);
    }
    sm3_u32_to_u8(&v)
}

#[test]
fn test_sm3() {
    use hex::encode;
    let res = sm3_u32_to_u8(&sm3(&['a' as u8, 'b' as u8, 'c' as u8]));
    assert_eq!(
        encode(res),
        "66c7f0f462eeedd9d1f2d46bdc10e4e24167c4875cf2f7a2297da02b8f4ba8e0"
    );
    let mut v = Vec::new();
    for _ in 0..20 {
        v.push('a' as u8);
    }
    let res = sm3_u32_to_u8(&sm3(&v));
    assert_eq!(
        encode(&res),
        "a50f9a8c6db698761f811c7f0d0d40c3fe58d193fd7b9a91588ddd903f79e2bb"
    );
}

#[test]
fn test_attack() {
    use hex::encode;
    let mut v = Vec::new();
    for _ in 0..64 {
        v.push('a' as u8);
    }
    let orig_res = sm3_u32_to_u8(&sm3(&v));
    v.push('b' as u8);
    let real_res = sm3_u32_to_u8(&sm3(&v));
    let attack_res = expansion_attack(&orig_res, &['b' as u8]);
    assert_eq!(encode(real_res), encode(attack_res));
}
