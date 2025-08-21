mod sha256;
mod wasi;

use sha256::*;
use wasi::*;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

const MAX_SIZE: u64 = 4 * 1024 * 1024 * 1024; // 4 GiB file size limit
const CHUNK_SIZE: usize = 2 * 1024 * 1024; // 2 MiB chunk size
const MSG_BLKS_PER_CHUNK: usize = CHUNK_SIZE >> 6;

static LINE_FEED: [u8; 1] = [0x0A];
static SPACES: &[u8; 2] = b"  ";
static HEX_CHARS: &[u8; 16] = b"0123456789abcdef";
static ERR_MSG_USAGE: &[u8] = "Usage: sha256 <filename>".as_bytes();
static ERR_MSG_FILE_TOO_LARGE: &[u8] = "Input file too large (>= 4Gb)".as_bytes();

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
fn main() -> Result<(), u16> {
    let mut msg_schedule = Box::new([0u32; 64]);

    // The hash values are initialised using the first 32 bits of the fractional part of the square roots of the first
    // 8 prime numbers (2..19)
    let mut hash_vals: [u32; 8] = [
        0x6A09E667, 0xBB67AE85, 0x3C6EF372, 0xA54FF53A, 0x510E527F, 0x9B05688C, 0x1F83D9AB,
        0x5BE0CD19,
    ];

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    // Get command line args
    let mut buf = [0u8; 256]; // buffer for cmd line args
    let mut args: [&str; 4] = [""; 4]; // slices into the buffer
    let argc = unsafe {
        match wasi_args_get(&mut buf, &mut args) {
            Ok(argc) => argc,
            Err(_) => return Err(1),
        }
    };

    if argc != 2 {
        let _ = unsafe { wasi_fd_write(2, &[ERR_MSG_USAGE]).unwrap() };
        return Err(1);
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    // Open file
    let filename = args[1];
    let fd = unsafe {
        match wasi_path_open(3, filename) {
            Ok(fd) => fd,
            Err(_) => return Err(1),
        }
    };

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    // Check file size is < 4Gb
    let file_size = unsafe {
        match fetch_file_size(fd) {
            Ok(bytes) => bytes,
            Err(_) => return Err(1),
        }
    };

    if file_size >= MAX_SIZE {
        let _ = unsafe { wasi_fd_write(2, &[ERR_MSG_FILE_TOO_LARGE]).unwrap() };
        return Err(1);
    }

    let file_size_bits = (file_size << 3).to_be_bytes();

    // Prepare empty msg block for edge case where file size is an exact integer-multiple of the buffer size
    let mut empty_msg_blk = [0u8; 64];
    empty_msg_blk[0] = 0x80;
    empty_msg_blk[56..].copy_from_slice(&file_size_bits);

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    // Open file
    let file_fd = match unsafe { wasi_path_open(3, filename) } {
        Ok(fd) => fd,
        Err(_) => return Err(1),
    };

    // Allocate buffer directly on the heap
    let mut buffer: Box<[u8]> = vec![0u8; CHUNK_SIZE].into_boxed_slice();
    let mut bytes_remaining = file_size;

    loop {
        let mut extra_blk = false;
        let bytes_read = match unsafe { wasi_fd_read(file_fd, &mut buffer) } {
            Ok(bytes_read) => bytes_read,
            Err(_e) => return Err(1),
        };

        if bytes_read == 0 {
            break; // EOF
        }

        bytes_remaining = bytes_remaining.saturating_sub(bytes_read as u64);

        // Assume the chunk is full
        let mut msg_blk_count = MSG_BLKS_PER_CHUNK;

        if bytes_read == CHUNK_SIZE {
            // Check for edge case
            if bytes_remaining == 0 {
                extra_blk = true; // file size is exact an integer-multiple of the chunk size
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
            phase_1(&buffer[blk_idx..blk_idx + 64], &mut msg_schedule);
            phase_2(&msg_schedule, &mut hash_vals);
        }

        // Potential extra message block for edge case
        if extra_blk {
            phase_1(&empty_msg_blk, &mut msg_schedule);
            phase_2(&msg_schedule, &mut hash_vals);
        }
    }

    // Convert the hex digits in the hash values to ASCII
    let mut hash_buf = [0u8; 64];

    for (i, &val) in hash_vals.iter().enumerate() {
        let offset = i << 3;
        let bytes = val.to_be_bytes();

        for j in 0..4 {
            hash_buf[offset + j * 2] = HEX_CHARS[(bytes[j] >> 4) as usize];
            hash_buf[offset + j * 2 + 1] = HEX_CHARS[(bytes[j] & 0x0F) as usize];
        }
    }

    let write_buf: [&[u8]; 4] = [&hash_buf, SPACES, filename.as_bytes(), &LINE_FEED];

    // io::stdout().write_vectored(bufs).unwrap();
    let _ = unsafe { wasi_fd_write(1, &write_buf).unwrap() };

    Ok(())
}
