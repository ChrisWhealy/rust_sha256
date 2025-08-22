// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// WASI interface definition
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// Matches WASI's __wasi_iovec_t
#[repr(C)]
pub struct Iovec {
    pub buf: *mut u8,
    pub buf_len: usize,
}

/// Matches WASI's __wasi_ciovec_t
#[repr(C)]
pub struct Ciovec {
    pub buf: *const u8,
    pub buf_len: usize,
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
#[link(wasm_import_module = "wasi_snapshot_preview1")]
unsafe extern "C" {
    pub fn args_sizes_get(argc: *mut usize, argv_buf_size: *mut usize) -> u16;
    pub fn args_get(argv: *mut *mut u8, argv_buf: *mut u8) -> u16;

    pub fn fd_seek(fd: u32, offset: i64, whence: u8, newoffset: *mut u64) -> u16;

    pub fn path_open(
        dir_fd: u32,
        dirflags: u32,
        path: *const u8,
        path_len: usize,
        oflags: u32,
        fs_rights_base: u64,
        fs_rights_inheriting: u64,
        fdflags: u32,
        fd_out: *mut u32,
    ) -> u16;

    pub fn fd_read(fd: u32, iovs: *const Iovec, iovs_len: usize, nread: *mut usize) -> u16;
    pub fn fd_write(fd: u32, iovs: *const Ciovec, iovs_len: usize, nwritten: *mut usize) -> u16;
}
