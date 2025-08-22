use sha256::sha256::*;

use std::{
    env,
    fs::File,
    io::{self, BufReader, Read},
    process,
};

const MAX_SIZE: u64 = 4 * 1024 * 1024 * 1024; // 4 GiB file size limit
const CHUNK_SIZE: usize = 2 * 1024 * 1024; // 2 MiB chunk size
const MSG_BLKS_PER_CHUNK: usize = CHUNK_SIZE >> 6;

static HEX_CHARS: [char; 16] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f',
];

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
fn main() -> io::Result<()> {
    let mut msg_digest = Box::new([0u32; 64]);

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
            phase_1(&buffer[blk_idx..blk_idx + 64], &mut msg_digest);
            phase_2(&msg_digest, &mut hash_vals);
        }

        // Potential extra message block for edge case
        if extra_blk {
            phase_1(&empty_msg_blk, &mut msg_digest);
            phase_2(&msg_digest, &mut hash_vals);
        }
    }

    let mut hex_str = String::with_capacity(64);
    for &h in &hash_vals {
        for i in (0..8).rev() {
            hex_str.push(HEX_CHARS[((h >> (i * 4)) & 0xF) as usize]);
        }
    }
    println!("{hex_str}  {filename}");

    Ok(())
}
