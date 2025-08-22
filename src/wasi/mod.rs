mod interface;

use interface::*;
use std::ffi::CStr;

static ERR_MSG_CMD_ARGS: &[u8] = "Unable to fetch command line arguments: 0x".as_bytes();
static ERR_MSG_CMD_ARGS_TOO_LONG: &[u8] = "Command line arguments too long (>256 chars)".as_bytes();
static ERR_MSG_FD_SEEK: &[u8] = "Error reading file size: 0x".as_bytes();
static ERR_MSG_PATH_OPEN: &[u8] = "Unable to open file: 0x".as_bytes();
static ERR_MSG_BAD_FD: &[u8] = "Bad file descriptor".as_bytes();
static ERR_MSG_NOENT: &[u8] = "No such file or directory".as_bytes();
static ERR_MSG_NOT_DIR_SYMLINK: &[u8] =
    "Neither a directory nor a symlink to a directory".as_bytes();
static ERR_MSG_NOT_PERMITTED: &[u8] = "Operation not permitted".as_bytes();
static ERR_MSG_FD_READ: &[u8] = "Error reading file: 0x".as_bytes();

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Rust <--> WASI interface
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub unsafe fn wasi_args_get<'a>(
    buf: &'a mut [u8],
    argv: &mut [&'a str; 4], // max 4 cmd line args
) -> Result<usize, u16> {
    let mut argc: usize = 0;
    let mut argv_buf_size: usize = 0;
    let ret = unsafe { args_sizes_get(&mut argc as *mut usize, &mut argv_buf_size as *mut usize) };

    if ret != 0 {
        let _ = unsafe { wasi_fd_write(2, &[ERR_MSG_CMD_ARGS, &ret.to_le_bytes()]).unwrap()};
        return Err(ret);
    }

    // Avoid buffer overflow
    if argv_buf_size > buf.len() {
        let _ = unsafe { wasi_fd_write(2, &[ERR_MSG_CMD_ARGS_TOO_LONG]).unwrap() };
        return Err(1);
    }

    let mut raw_ptrs: [*mut u8; 4] = [core::ptr::null_mut(); 4];

    let ret = unsafe { args_get(raw_ptrs.as_mut_ptr(), buf.as_mut_ptr()) };
    if ret != 0 {
        let _ = unsafe { wasi_fd_write(2, &[ERR_MSG_CMD_ARGS, &ret.to_le_bytes()]).unwrap()};
        return Err(1);
    }

    for i in 0..argc.min(argv.len()) {
        let arg_bytes = unsafe { CStr::from_ptr(raw_ptrs[i] as *const _).to_bytes() };
        argv[i] = str::from_utf8(arg_bytes).unwrap_or_default();
    }

    Ok(argc)
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub unsafe fn fetch_file_size(fd: u32) -> Result<u64, u16> {
    let mut new_offset: u64 = 0;

    // Seek to the end of the file
    let file_size_bytes: u64 = match unsafe { fd_seek(fd, 0, 2, &mut new_offset) } {
        0 => new_offset,
        e => {
            let _ = unsafe { wasi_fd_write(2, &[ERR_MSG_FD_SEEK, &e.to_le_bytes()]).unwrap()};
            return Err(1);
        }
    };

    // Reset seek pointer
    match unsafe { fd_seek(fd, 0, 0, &mut new_offset) } {
        0 => Ok(file_size_bytes),
        e => {
            let _ = unsafe { wasi_fd_write(2, &[ERR_MSG_FD_SEEK, &e.to_le_bytes()]).unwrap()};
            Err(1)
        }
    }
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub unsafe fn wasi_path_open(dir_fd: u32, path: &str) -> Result<u32, u16> {
    let mut new_fd: u32 = 0;

    let ret = unsafe { path_open(
        dir_fd,
        0, // Don't follow symlinks
        path.as_ptr(),
        path.len(),
        0, // 0 = read only
        6, // 6 = fd_seek (4) + fd_read (2)
        0, // inheriting rights
        0, // fdflags
        &mut new_fd,
    )};

    if ret == 0 {
        Ok(new_fd)
    } else {
        let _ = match ret {
            0x08 => unsafe { wasi_fd_write(2, &[ERR_MSG_BAD_FD]).unwrap() },
            0x2C => unsafe { wasi_fd_write(2, &[ERR_MSG_NOENT]).unwrap() },
            0x36 => unsafe { wasi_fd_write(2, &[ERR_MSG_NOT_DIR_SYMLINK]).unwrap() },
            0x3F => unsafe { wasi_fd_write(2, &[ERR_MSG_NOT_PERMITTED]).unwrap() },
            _ => unsafe { wasi_fd_write(2, &[ERR_MSG_PATH_OPEN]).unwrap() },
        };

        Err(ret)
    }
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub unsafe fn wasi_fd_read(fd: u32, buf: &mut [u8]) -> Result<usize, u16> {
    let iov = Iovec {
        buf: buf.as_mut_ptr(),
        buf_len: buf.len(),
    };
    let mut nread: usize = 0;

    let ret = unsafe { fd_read(fd, &iov, 1, &mut nread) };

    if ret == 0 {
        Ok(nread)
    } else {
        let _ = unsafe { wasi_fd_write(2, &[ERR_MSG_FD_READ, &ret.to_le_bytes()]).unwrap() };
        Err(ret)
    }
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub unsafe fn wasi_fd_write(fd: u32, bufs: &[&[u8]]) -> Result<usize, u16> {
    let iovs: Vec<Ciovec> = bufs
        .iter()
        .map(|b| Ciovec {
            buf: b.as_ptr(),
            buf_len: b.len(),
        })
        .collect();
    let mut bytes_written: usize = 0;

    let ret = unsafe { fd_write(fd, iovs.as_ptr(), iovs.len(), &mut bytes_written) };

    if ret == 0 {
        Ok(bytes_written)
    } else {
        Err(ret)
    }
}
