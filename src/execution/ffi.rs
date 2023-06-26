use std::os::raw::c_char;

extern "C" {
    pub fn close(fd: i32) -> i32;
    pub fn perror(s: *const c_char);
    pub fn alarm(seconds: u32) -> u32;
    pub fn pipe(pipefd: *mut Pipefd) -> i32;
    pub fn dup2(oldfd: i32, newfd: i32) -> i32;
    pub fn ftruncate(fd: i32, length: usize) -> i32;
    pub fn munmap(addr: *mut u8, length: usize) -> i32;
    pub fn lseek(fd: i32, offset: i32, whence: i32) -> i32;
    pub fn read(fd: i32, buf: *mut u8, count: usize) -> i32;
    pub fn write(fd: i32, buf: *const u8, count: usize) -> i32;
    pub fn memfd_create(name: *const c_char, flags: u32) -> i32;
    pub fn poll(fds: *mut Pollfd, nfds_t: u64, timeout: i32) -> i32;
    pub fn mmap(addr: *mut u8, length: usize, prot: i32, flags: i32,
            fd: i32, offset: i32) -> *mut u8;
}


pub const MAP_SHARED:  i32 = 0x1;
pub const PROT_READ:   i32 = 0x1;
pub const PROT_WRITE:  i32 = 0x2;
pub const MFD_CLOEXEC: u32 = 0x1;
pub const SEEK_SET:    i32 = 0x0;
pub const POLLIN:      i16 = 0x1;

#[repr(C)]
#[derive(Debug)]
pub struct Pipefd {
    pub readfd:  i32,
    pub writefd: i32
}

impl Pipefd {

    /// Close the read end of the pipe and set it to -1
    pub fn close_read(&mut self) {

        // If its not open, then we have nothing to do
        if self.readfd == -1 {
            return;
        }

        // This function can be called from drop handlers, so we will not do a
        // hard fail if close returns an error as that would lead to an infinite
        // loop. At worse this would just mean that the fd is left open.
        unsafe { close(self.readfd) };

        self.readfd = -1;
    }

    /// Close the write end of the pipe and set it to -1
    pub fn close_write(&mut self) {

        if self.writefd == -1 {
            return;
        }

        // This function can be called from drop handlers, so we will not do a
        // hard fail if close returns an error as that would lead to an infinite
        // loop. At worse this would just mean that the fd is left open.
        unsafe { close(self.writefd) };

        self.writefd = -1;
    }

    /// Close both the fd's
    fn destroy(&mut self) {
        self.close_read();
        self.close_write();
    }

}

impl Default for Pipefd {

    /// Initialize the file descriptors to `-1`
    fn default() -> Self {
        Self {
            readfd: -1,
            writefd: -1,
        }
    }
}

impl Drop for Pipefd {

    fn drop(&mut self) {
        self.destroy();
    }

}

/// Struct to mirror the pollfd struct in C
#[repr(C)]
pub struct Pollfd {
    pub fd: i32,
    pub events: i16,
    pub revents: i16,
}
