// The first 32 bits of the fractional part of the cube roots of the first 64 primes 2..311
static CONSTANTS: [u32; 64] = [
    0x428A2F98, 0x71374491, 0xB5C0FBCF, 0xE9B5DBA5, 0x3956C25B, 0x59F111F1, 0x923F82A4, 0xAB1C5ED5,
    0xD807AA98, 0x12835B01, 0x243185BE, 0x550C7DC3, 0x72BE5D74, 0x80DEB1FE, 0x9BDC06A7, 0xC19BF174,
    0xE49B69C1, 0xEFBE4786, 0x0FC19DC6, 0x240CA1CC, 0x2DE92C6F, 0x4A7484AA, 0x5CB0A9DC, 0x76F988DA,
    0x983E5152, 0xA831C66D, 0xB00327C8, 0xBF597FC7, 0xC6E00BF3, 0xD5A79147, 0x06CA6351, 0x14292967,
    0x27B70A85, 0x2E1B2138, 0x4D2C6DFC, 0x53380D13, 0x650A7354, 0x766A0ABB, 0x81C2C92E, 0x92722C85,
    0xA2BFE8A1, 0xA81A664B, 0xC24B8B70, 0xC76C51A3, 0xD192E819, 0xD6990624, 0xF40E3585, 0x106AA070,
    0x19A4C116, 0x1E376C08, 0x2748774C, 0x34B0BCB5, 0x391C0CB3, 0x4ED8AA4A, 0x5B9CCA4F, 0x682E6FF3,
    0x748F82EE, 0x78A5636F, 0x84C87814, 0x8CC70208, 0x90BEFFFA, 0xA4506CEB, 0xBEF9A3F7, 0xC67178F2,
];

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Internal SHA256 machinery
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
fn inner_sigma(v: u32, rotr1: u32, rotr2: u32) -> u32 {
    v.rotate_right(rotr1) ^ v.rotate_right(rotr2)
}

fn sigma(v: u32, rotr1: u32, rotr2: u32, shr: u32) -> u32 {
    inner_sigma(v, rotr1, rotr2) ^ (v >> shr)
}

fn big_sigma(v: u32, rotr1: u32, rotr2: u32, rotr3: u32) -> u32 {
    inner_sigma(v, rotr1, rotr2) ^ v.rotate_right(rotr3)
}

fn choose(a: u32, b: u32, c: u32) -> u32 {
    (a & b) ^ ((!a) & c)
}

fn majority(a: u32, b: u32, c: u32) -> u32 {
    (a & b) ^ (a & c) ^ (b & c)
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Transfer the current message block to the first 16 words of the 64-word message schedule,
/// then populate the remaining 48 words with scrambled versions of the first 16 words
pub fn phase_1(msg_blk: &[u8], msg_schedule: &mut [u32; 64]) {
    // words 0..15
    for i in 0..16 {
        let j = i * 4;
        msg_schedule[i] =
            u32::from_be_bytes([msg_blk[j], msg_blk[j + 1], msg_blk[j + 2], msg_blk[j + 3]]);
    }

    // words 16..63
    for i in 16..64 {
        msg_schedule[i] = msg_schedule[i - 16]
            .wrapping_add(sigma(msg_schedule[i - 15], 7, 18, 3))
            .wrapping_add(msg_schedule[i - 7])
            .wrapping_add(sigma(msg_schedule[i - 2], 17, 19, 10));
    }
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Generate the hash values based on the contents of the message schedule
pub fn phase_2(msg_schedule: &[u32; 64], hash_vals: &mut [u32; 8]) {
    let mut a = hash_vals[0];
    let mut b = hash_vals[1];
    let mut c = hash_vals[2];
    let mut d = hash_vals[3];
    let mut e = hash_vals[4];
    let mut f = hash_vals[5];
    let mut g = hash_vals[6];
    let mut h = hash_vals[7];

    for i in 0..64 {
        let t1 = h
            .wrapping_add(big_sigma(e, 6, 11, 25))
            .wrapping_add(CONSTANTS[i])
            .wrapping_add(msg_schedule[i])
            .wrapping_add(choose(e, f, g));
        let t2 = big_sigma(a, 2, 13, 22).wrapping_add(majority(a, b, c));

        // Shunt working copies of the hash values
        h = g;
        g = f;
        f = e;
        e = d.wrapping_add(t1);
        d = c;
        c = b;
        b = a;
        a = t1.wrapping_add(t2);
    }

    hash_vals[0] = hash_vals[0].wrapping_add(a);
    hash_vals[1] = hash_vals[1].wrapping_add(b);
    hash_vals[2] = hash_vals[2].wrapping_add(c);
    hash_vals[3] = hash_vals[3].wrapping_add(d);
    hash_vals[4] = hash_vals[4].wrapping_add(e);
    hash_vals[5] = hash_vals[5].wrapping_add(f);
    hash_vals[6] = hash_vals[6].wrapping_add(g);
    hash_vals[7] = hash_vals[7].wrapping_add(h);
}

#[cfg(test)]
mod unit_tests;
