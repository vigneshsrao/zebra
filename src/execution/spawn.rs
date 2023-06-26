use std::process;
use std::fs::File;
use std::io::{self, Write};
use std::os::unix::process::CommandExt;
use std::os::unix::process::ExitStatusExt;

use super::execution::{ReturnCode, Execution};
use super::ffi::alarm;

/// Create `filename` and write `data` to it
pub fn write_file(filename: &str, data: &String) -> io::Result<()> {
    let mut file = File::create(filename)?;
    file.write(data.as_bytes())?;
    Ok(())
}

pub struct Spawn {
    path:          String,
    args:          Vec<&'static str>,
    timeout:       u32,
    pname:         String,
}

impl Spawn {

    pub fn new(path: String, args: Vec<&'static str>, timeout: u32) -> Self {

        let rand  = unsafe { std::arch::x86_64::_rdtsc() };
        let pname = format!("tests/testfile_{}.js", rand);

        Spawn {
            path:    path,
            args:    args,
            timeout: timeout,
            pname:   pname,
        }
    }
}

impl Execution for Spawn {

    fn execute(&mut self, input: &String) -> ReturnCode {

        write_file(&self.pname, input)
            .expect("Error when writting out to file");

        let timeout = self.timeout;
        let child_pre_exec = move || -> io::Result<()> {

            unsafe {
                alarm(timeout);
            }

            Ok(())
        };

        let status = unsafe {
            process::Command::new(&self.path)
                    .pre_exec(child_pre_exec)
                    .args(&self.args)
                    .arg(&self.pname)
                    .stdout(process::Stdio::null())
                    .stderr(process::Stdio::null())
                    .status()
                    .expect("Failed to exe proc")
        };

        match status.code() {
            Some(code) => {
                ReturnCode::Status(code)
            },
            None => {
                let signal = status.signal().unwrap_or(0);

                if signal == 14 {
                    ReturnCode::Timeout
                } else {
                    ReturnCode::Crash(signal)
                }

            }
        }
    }
}

