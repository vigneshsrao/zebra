use std::io;
use std::process;
use std::ffi::CString;
use std::thread::sleep;
use std::time::Duration;
use std::os::unix::process::CommandExt;
use std::os::unix::process::ExitStatusExt;

use super::execution::{ReturnCode, Execution};
use super::ffi::*;

const CRFD: i32 = 100;
const CWFD: i32 = 101;
const DRFD: i32 = 102;
const _DWFD: i32 = 103;

const MAX_SIZE: usize = 0x10000;

// Error to wrap around all the repl related errors
#[derive(Debug, Eq, PartialEq)]
enum ReplError {
    Timeout,
    Other(&'static str),
}

impl std::fmt::Display for ReplError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Timeout    => {
                write!(fmt, "[-] ReplError: Timed out")
            },
            Self::Other(msg) => {
                write!(fmt, "[-] ReplError: {}", msg)
            },
        }
    }
}

impl std::error::Error for ReplError {}

// type ReplResult<T> = Result<T, Box<dyn std::error::Error>>;
type ReplResult<T> = Result<T, ReplError>;

// Helper to create new CStrings from str's. We will fail hard on this as this
// should never fail unless the input str that has a `\0` in it in which case we
// must be doing something wrong
macro_rules! cstring {
    ($value: expr) => {
        CString::new($value).expect("Failed to create cstring")
    }
}

// Macro to check the return value of the ffi calls. In case of error, it will
// print out the error and terminate the entire process.
macro_rules! check {
    ($value: expr, $message: expr) => {
        {
            // This is so that if a function is passed to this macro, then it will
            // be evaluated here and we can just use the result elsewhere.
            let retval = $value;
            if retval == !0 {
                let message = format!("[!] Error: {}", $message);
                let msg     = cstring!(message);
                perror(msg.as_ptr());
                // std::process::exit(-1);
                // Err(Box::new(ReplError("FFI function failure")))
                // Err(ReplError::Other("FFI Function Failure"))
                Err(ReplError::Other($message))
            } else {
                Ok(retval)
            }
        }
    };
    ($value: expr) => {
        {
            // This is so that if a function is passed to this macro, then it will
            // be evaluated here and we can just use the result elsewhere.
            let retval = $value;
            if retval == !0 {
                Err(ReplError::Other("FFI function failure"))
            } else {
                Ok(retval)
            }
        }
    };
}


#[derive(Eq,PartialEq)]
enum CtrlCmd {
    Helo,
    Exec,
    Exit,
    Misc(i32)
}

impl From<i32> for CtrlCmd {

    fn from(val: i32) -> CtrlCmd {
        match val {
            0x4f4c4548 => CtrlCmd::Helo,
            0x63657865 => CtrlCmd::Exec,
            0x74697865 => CtrlCmd::Exit,
            _          => CtrlCmd::Misc(val),
        }
    }
}

impl From<&CtrlCmd> for CString {
    fn from(cmd: &CtrlCmd) -> CString {
        match cmd {
            CtrlCmd::Helo    => cstring!("HELO"),
            CtrlCmd::Exec    => cstring!("exec"),
            CtrlCmd::Exit    => cstring!("exit"),
            CtrlCmd::Misc(_) => {
                unimplemented!();
            }
        }
    }
}

/// A structure to store all the read-eval-print-loop connection related data.
pub struct ReplConnection {
    data_write_fd: Option<i32>,
    ctrl_write_fd: Option<Pipefd>,
    ctrl_read_fd:  Option<Pipefd>,
    mapping:       Option<*mut u8>,
    child:         Option<process::Child>,
    path:          Option<String>,
    args:          Option<Vec<&'static str>>,
    timeout:       Option<u32>,
}

impl Execution for ReplConnection {

    /// Wrapper function to call execute_impl. This function will check if
    /// execute_impl failed and if so try a second time. If both fail, then this
    /// function terminates the process
    fn execute(&mut self, input: &String) -> ReturnCode {
        match self.execute_impl(input) {
            Ok(code) => code,
            Err(_)   => {
                // For some reason, execution failed. Lets re-initialize the
                // child and try again.
                self.reset_connection();
                match self.execute_impl(input) {
                    Ok(code) => code,
                    Err(err) => {
                        // We failed yet again. Now lets stop trying.
                        println!("[-] Repl Execution Failure: {err}");
                        process::exit(-1);
                    }
                }
            }
        }
    }
}

impl ReplConnection {

    pub fn new(path: String, args: Vec<&'static str>, timeout: u32) -> Self {
        let mut replcon = Self::default();
        replcon.path    = Some(path);
        replcon.args    = Some(args);
        replcon.timeout = Some(timeout);
        if let Err(err) = replcon.init() {
                println!("[-] ReplConnection Initialization Failure! {err}");
                process::exit(-1);
        };
        replcon
    }

    /// Setup and initialize a new connection to a program at `path`
    fn init(&mut self) -> ReplResult<()> {

        // First reset the connection to clean up any existing resources
        self.reset_connection();

        let shmname = cstring!("SHMRegion");
        let mut ctrl_fd_read  = Pipefd::default();
        let mut ctrl_fd_write = Pipefd::default();

        let (address, fd);

        unsafe {
            fd = memfd_create(shmname.as_ptr(), MFD_CLOEXEC);
            check!(fd, "memfd_create")?;
            check!(ftruncate(fd, MAX_SIZE), "ftruncate")?;

            address = mmap(std::ptr::null_mut(), MAX_SIZE,
                        PROT_READ | PROT_WRITE, MAP_SHARED, fd, 0);
            check!(address as usize, "mmap")?;

            check!(pipe(&mut ctrl_fd_read),  "pipe, read")?;
            check!(pipe(&mut ctrl_fd_write), "pipe, write")?;
        }

        // This closure will be run in the forked child process. It will do the
        // necessary initialization of the fd's that the target process will
        // expect and close the unused fds.
        let pre_exec =  move || -> io::Result<()> {
            // Macro to wrap `check!` to return an io::Error.
            macro_rules! check_ioerr {
                ($ret: expr, $msg: expr) => {
                    if check!($ret, $msg).is_err() {
                        Err(io::Error::new(io::ErrorKind::Other, $msg))
                    } else {
                        Ok(())
                    }
                };
            }

            unsafe {

                // Duplicate the fd's for use in the spawned process
                check_ioerr!(dup2(fd, DRFD), "dup2")?;
                check_ioerr!(dup2(ctrl_fd_write.readfd, CRFD), "dup2")?;
                check_ioerr!(dup2(ctrl_fd_read.writefd, CWFD), "dup2")?;

                // Close the unused fd's of the pipe
                check_ioerr!(close(ctrl_fd_write.writefd), "close")?;
                check_ioerr!(close(ctrl_fd_read.readfd), "close")?;
            }

            Ok(())
        };

        // Execute the child. Its safe to unwrap path and args here as these
        // should be set when an instance of this struct is created.
        let child = unsafe {
            process::Command::new(self.path.as_ref().unwrap())
                .args(self.args.as_ref().unwrap())
                .pre_exec(pre_exec)
                .stdout(process::Stdio::null())
                .stderr(process::Stdio::null())
                .spawn()
                .map_err(|_|
                         ReplError::Other("Failed to execute target Process"))?
        };

        // Close the unused ends of the pipes
        ctrl_fd_write.close_read();
        ctrl_fd_read.close_write();

        self.data_write_fd = Some(fd);
        self.ctrl_write_fd = Some(ctrl_fd_write);
        self.ctrl_read_fd  = Some(ctrl_fd_read);
        self.mapping       = Some(address);
        self.child         = Some(child);

        // Receive the Helo message from the child to ensure that the connection
        // is successfully setup.
        let msg = self.recv_cmd()?;
        if msg == CtrlCmd::Helo {
            self.send_cmd(CtrlCmd::Helo)?;
        } else {
            return Err(ReplError::Other("Incorrect msg received"));
        }

        Ok(())
    }

    /// Close the connection to the child and clean up the resources. After this
    /// function is called, the ReplConnection will be in the `Default` state
    pub fn reset_connection(&mut self) {

        if self.child.is_some() {

            // Reset self.child to its default value and try to kill it if its
            // running. If kill fails, then the child was anyway not running. So
            // we don't care about the return value. Same goes for the wait.
            let mut child = std::mem::take(&mut self.child).unwrap();
            let _  = child.kill();
            let _  = child.wait();
        }

        // Close the data write fd
        if self.data_write_fd.is_some() {
            let ret = unsafe { close(self.dwfd()) };
            self.data_write_fd = None;
            if ret != 0 {
                println!("ERROR closing dwfd");
                unsafe {core::ptr::copy(0x414141usize as *const u8, 0x414141usize as *mut u8, 8)};
            }
        }

        // Close both of the control fd's. This will reset the value to the
        // default ones and automatically call the drop handlers on the old ones
        let _ = std::mem::take(&mut self.ctrl_read_fd);
        let _ = std::mem::take(&mut self.ctrl_write_fd);

        // Clean up the mapping.
        if self.mapping.is_some() {
            unsafe { munmap(self.mapping(), MAX_SIZE) };
            self.mapping = None;
        }
    }

    /// Send a message to the child process to tell it to operate on the input
    /// passed in `input` parameter. This will wait for the child to either
    /// finish executing the input and returning a return code, or for it to
    /// crash or up till the timeout is reached. It returns either the status
    /// returned by the child, the signal that terminated it or a timeout.
    fn execute_impl(&mut self, input: &str) -> ReplResult<ReturnCode> {

        // Check if the connection is already initialized. Initialize it if not.
        if !self.is_initialized() {
            // self.reset_connection();
            // self.init(self.path.as_ref())?;
            self.init()?;
        }

        // Reset the file descriptors of the backing buffer
        unsafe { check!(lseek(self.dwfd(), 0, SEEK_SET), "lseek")? };

        // Make sure that the size of the input does not go beyond the
        // `MAX_SIZE` and then copy the input over the to the backing shared
        // memory
        let size = std::cmp::min(input.len(), MAX_SIZE-1);
        unsafe { core::ptr::copy(input.as_ptr(), self.mapping(), size) };

        // Send the execute command to the child and then tell it the size of
        // the input
        self.send_cmd(CtrlCmd::Exec)?;
        self.send_u64(size as u64)?;

        // Now lets wait for the child to either finish execution, crash, or
        // timeout.
        let result = match self.recv_cmd() {
            Ok(CtrlCmd::Misc(ret)) => {
                // The read succeded which means the child successfully executed
                // the code and returned a status
                ReturnCode::Status(ret)
            },
            Ok(_)  => {
                // Invalid message received
                return Err(ReplError::Other("Invalid message received"));
            },
            Err(ReplError::Timeout) => {
                // The child timed out while trying to execute the input. We will
                // reset the connection now.
                self.reset_connection();
                ReturnCode::Timeout
            },
            Err(_) => {
                // The child probably crashed. Lets try_wait on it a few times
                // to see if we can get the return value. If we fail on the
                // try_wait, then we will just error out.
                let mut iters = 0;

                loop {
                    let ret = match self.child.as_mut().unwrap().try_wait() {
                        Ok(Some(status)) => {
                            // Child surely exited. Lets find if it crashed or
                            // normally returned
                            self.reset_connection();
                            if let Some(code) = status.code() {
                                // Normal exit: This should ideally never
                                // happen, but lets handle it in case it does.
                                ReturnCode::Status(code)
                            } else {
                                // Its definitely ternimated by a signal. Lets
                                // return the signal that terminated it.
                                ReturnCode::Crash(status.signal().unwrap())
                            }
                        },
                        Ok(None) => {
                            // The child is still running. This should never
                            // happen as if the child is running, then our read
                            // should never fail. Maybe the child is in the
                            // process of crashing and we need to try a few more
                            // times.
                            if iters >= 10 {
                                self.reset_connection();
                                return Err(ReplError::Other(
                                    "Poll succeded but read failed"));
                            }

                            iters += 1;
                            sleep(Duration::new(0, 10000));
                            continue;
                        },
                        Err(_)   => {
                            // Error while waiting. Just error out now
                            return Err(ReplError::Other("Error in try_wait"));
                        }
                    };

                    break ret;
                }
            }
        };

        Ok(result)
    }

    fn recv_cmd(&self) -> ReplResult<CtrlCmd> {
        let mut buf = [0i32; 1];
        let fd = self.crfd();

        // First poll for the child to either write to the control fd or change
        // state
        let mut pollfd = Pollfd {
            fd:         self.crfd(),
            events:     POLLIN,
            revents:    0,
        };

        let timeout = self.timeout.ok_or(ReplError::Other("Missing timeout"))?;
        let timeout = timeout * 1000;
        let result = unsafe {
            check!(poll(&mut pollfd as *mut Pollfd, 1, timeout as i32), "poll")?
        };

        // Check if we timed out on the poll. If so, then just return a Timeout
        // Error
        if result == 0 {
            return Err(ReplError::Timeout);
        }

        // Since we did not timeout, we definitely have something to read
        unsafe {
            let ret = read(fd, buf.as_mut_ptr() as *mut u8, 4);
            // This check should not printout the perror as failing here might
            // be valid if the child has crashed. Also, we should strictly check
            // this as read might succeded on crashed child and return 0
            let ret = if ret != 4 {
                -1
            } else {
                ret
            };
            check!(ret)?;
        }

        Ok(CtrlCmd::from(buf[0]))
    }

    fn send_cmd(&self, cmd: CtrlCmd) -> ReplResult<()> {

        unsafe {
            let cmd = CString::from(&cmd);
            let ret = write(self.cwfd(), cmd.as_ptr() as *const u8, 4);
            check!(ret, "write")?;
        }

        Ok(())
    }

    fn send_u64(&self, data: u64) -> ReplResult<()> {
        unsafe {
            let ret = write(self.cwfd(),
                            &data as *const u64 as *const u8, 8);
            check!(ret, "write")?;
        }

        Ok(())
    }

    /// Define getters for the fields. The unwarp here should not fail as they
    /// should only be called in a context where its verified that they exist.
    fn dwfd(&self) -> i32 {
        self.data_write_fd.unwrap()
    }

    fn crfd(&self) -> i32 {
        self.ctrl_read_fd.as_ref().unwrap().readfd
    }

    fn cwfd(&self) -> i32 {
        self.ctrl_write_fd.as_ref().unwrap().writefd
    }

    fn mapping(&self) -> *mut u8 {
        self.mapping.unwrap()
    }

    /// Check if the connection is initialized. This will also check if the
    /// child is running.
    fn is_initialized(&mut self) -> bool {
        if self.data_write_fd.is_none() ||
            self.ctrl_read_fd.is_none() ||
            self.ctrl_write_fd.is_none() ||
            self.mapping.is_none() ||
            self.child.is_none() {
                return false;
            }

        // // If we reach here, then the child exists else we would have returned
        // // false already
        if let Ok(None) = self.child.as_mut().unwrap().try_wait() {
            return true;
        }

        return false;
    }
}

impl Default for ReplConnection {
    fn default() -> Self {
        Self {
            data_write_fd: None,
            ctrl_write_fd: None,
            ctrl_read_fd:  None,
            mapping:       None,
            child:         None,
            path:          None,
            args:          None,
            timeout:       None,
        }
    }
}


impl Drop for ReplConnection {
    fn drop(&mut self) {
        println!("[+] ReplConnection Drop Handler");
        self.reset_connection();
    }
}
