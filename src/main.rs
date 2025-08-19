use std::{
    env,
    fs::File,
    io::{self, BufReader, Read},
    process,
};

const MAX_SIZE: u64 = 4 * 1024 * 1024 * 1024; // 4 GiB file size limit
const CHUNK_SIZE: usize = 2 * 1024 * 1024; // 2 MiB chunk size
const MSG_BLKS_PER_CHUNK: usize = CHUNK_SIZE >> 6;

// The first 32 bits of the fractional part of the cube roots of the first 64 primes 2..311
const CONSTANTS: [u32; 64] = [
    0x428A2F98, 0x71374491, 0xB5C0FBCF, 0xE9B5DBA5, 0x3956C25B, 0x59F111F1, 0x923F82A4, 0xAB1C5ED5,
    0xD807AA98, 0x12835B01, 0x243185BE, 0x550C7DC3, 0x72BE5D74, 0x80DEB1FE, 0x9BDC06A7, 0xC19BF174,
    0xE49B69C1, 0xEFBE4786, 0x0FC19DC6, 0x240CA1CC, 0x2DE92C6F, 0x4A7484AA, 0x5CB0A9DC, 0x76F988DA,
    0x983E5152, 0xA831C66D, 0xB00327C8, 0xBF597FC7, 0xC6E00BF3, 0xD5A79147, 0x06CA6351, 0x14292967,
    0x27B70A85, 0x2E1B2138, 0x4D2C6DFC, 0x53380D13, 0x650A7354, 0x766A0ABB, 0x81C2C92E, 0x92722C85,
    0xA2BFE8A1, 0xA81A664B, 0xC24B8B70, 0xC76C51A3, 0xD192E819, 0xD6990624, 0xF40E3585, 0x106AA070,
    0x19A4C116, 0x1E376C08, 0x2748774C, 0x34B0BCB5, 0x391C0CB3, 0x4ED8AA4A, 0x5B9CCA4F, 0x682E6FF3,
    0x748F82EE, 0x78A5636F, 0x84C87814, 0x8CC70208, 0x90BEFFFA, 0xA4506CEB, 0xBEF9A3F7, 0xC67178F2,
];

fn main() -> io::Result<()> {
    let mut msg_digest: [u32; 64] = [0; 64];

    // The first 32 bits of the fractional part of the square roots of the first 8 primes 2..19
    let mut hash_vals: [u32; 8] = [
        0x6A09E667, 0xBB67AE85, 0x3C6EF372, 0xA54FF53A, 0x510E527F, 0x9B05688C, 0x1F83D9AB,
        0x5BE0CD19,
    ];

    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <filename>", args[0]);
        process::exit(1);
    }
    let filename = &args[1];

    // Check file size first
    let metadata = std::fs::metadata(filename)?;
    let file_size = metadata.len();
    if file_size >= MAX_SIZE {
        eprintln!(
            "Error: file '{}' is too large ({} bytes, max allowed is < 4 GiB)",
            filename, file_size
        );
        process::exit(1);
    }

    let file_size_bits = (file_size << 3).to_be_bytes();

    // Prepare empty msg block for edge case where file size is an exact integer-multiple of the buffer size
    let mut empty_msg_blk = [0u8; 64];
    empty_msg_blk[0] = 0x80;
    empty_msg_blk[56..].copy_from_slice(&file_size_bits);

    let file = File::open(filename)?;
    let mut reader = BufReader::new(file);

    // Allocate buffer directly on the heap
    let mut buffer: Box<[u8]> = vec![0u8; CHUNK_SIZE].into_boxed_slice();
    let mut bytes_remaining = file_size;

    loop {
        let mut extra_blk = false;
        let bytes_read = reader.read(&mut buffer[..])?;
        bytes_remaining = bytes_remaining.saturating_sub(bytes_read as u64);

        if bytes_read == 0 {
            break; // EOF
        }

        // Assume the chunk is full
        let mut msg_blk_count = MSG_BLKS_PER_CHUNK;

        if bytes_read == CHUNK_SIZE {
            if bytes_remaining == 0 {
                // Edge case: file size is exact an integer-multiple of the chunk size
                extra_blk = true;
            }
        } else {
            // Final partial chunk - which will always contain at least one empty byte for the EOD marker
            let eod = bytes_read;
            buffer[eod] = 0x80;
            let used = eod + 1;

            // Avoid buffer overflow
            if CHUNK_SIZE - used < 8 {
                extra_blk = true;
                empty_msg_blk[0] = 0x00; // EOD marker already exists at the end of the previous block
                buffer[used..].fill(0);
            } else {
                let rem = used % 64;
                let pad_zeros = if rem <= 56 { 56 - rem } else { 56 + 64 - rem };
                let end = used + pad_zeros + 8; // total bytes to process from this chunk

                buffer[used..used + pad_zeros].fill(0);
                buffer[end - 8..end].copy_from_slice(&file_size_bits);

                msg_blk_count = end / 64;
            }
        }

        // Process available message blocks
        for blk in 0..msg_blk_count {
            let blk_idx = blk << 6;
            let msg_blk = &buffer[blk_idx..blk_idx + 64];
            phase_1(msg_blk, &mut msg_digest);
            phase_2(&msg_digest, &mut hash_vals);
        }

        // Potential extra message block for edge case
        if extra_blk {
            phase_1(&empty_msg_blk, &mut msg_digest);
            phase_2(&msg_digest, &mut hash_vals);
        }
    }

    let hex_str = hash_vals
        .iter()
        .map(|h| format!("{:08x}", h))
        .collect::<String>();
    println!("{hex_str}  {filename}");

    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Internal
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
#[inline(always)]
fn inner_sigma(v: u32, rotr1: u32, rotr2: u32) -> u32 {
    v.rotate_right(rotr1) ^ v.rotate_right(rotr2)
}

#[inline(always)]
fn sigma(v: u32, rotr1: u32, rotr2: u32, shr: u32) -> u32 {
    inner_sigma(v, rotr1, rotr2) ^ (v >> shr)
}

#[inline(always)]
fn big_sigma(v: u32, rotr1: u32, rotr2: u32, rotr3: u32) -> u32 {
    inner_sigma(v, rotr1, rotr2) ^ v.rotate_right(rotr3)
}

#[inline(always)]
fn choose(a: u32, b: u32, c: u32) -> u32 {
    (a & b) ^ ((!a) & c)
}

#[inline(always)]
fn majority(a: u32, b: u32, c: u32) -> u32 {
    (a & b) ^ (a & c) ^ (b & c)
}

#[inline(always)]
fn phase_1(msg_blk: &[u8], msg_digest: &mut [u32; 64]) {
    // words 0..15
    for i in 0..16 {
        let j = i * 4;
        msg_digest[i] =
            u32::from_be_bytes([msg_blk[j], msg_blk[j + 1], msg_blk[j + 2], msg_blk[j + 3]]);
    }

    // words 16..63
    for i in 16..64 {
        msg_digest[i] = msg_digest[i - 16]
            .wrapping_add(sigma(msg_digest[i - 15], 7, 18, 3))
            .wrapping_add(msg_digest[i - 7])
            .wrapping_add(sigma(msg_digest[i - 2], 17, 19, 10));
    }
}

#[inline(always)]
fn phase_2(msg_digest: &[u32; 64], hash_vals: &mut [u32; 8]) {
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
            .wrapping_add(msg_digest[i])
            .wrapping_add(choose(e, f, g));
        let t2 = big_sigma(a, 2, 13, 22).wrapping_add(majority(a, b, c));

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
