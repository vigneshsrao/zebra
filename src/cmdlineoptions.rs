//! Module to hold all the commandline arguments related code.

#[derive(Debug)]
struct CmdLineError(&'static str);
impl std::fmt::Display for CmdLineError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "{}", self.0)
    }
}

impl std::error::Error for CmdLineError {}

type CmdLineResult<T> = Result<T, Box<dyn std::error::Error>>;

/// This structure hold the command line arguments that will be used in the
/// fuzzing process.
#[derive(Debug)]
pub struct CmdLineOptions {
    pub dry_run:  bool,
    pub threads:  u8,
    pub filename: String,
    pub timeout:  u8,
    pub disk:     bool,
}

impl Default for CmdLineOptions {
    fn default() -> Self {
        Self {
            dry_run: false,
            threads: 1,
            filename: "/home/vignesh/Documents/exploits/temp/webkit_new_source\
                       /WebKit/FuzzBuild/Debug/bin/jsc".to_string(),
            timeout: 5,
            disk:    false,
        }
    }
}

impl CmdLineOptions {

    /// Parse the command line arguments into a representation of the
    /// [CmdLineOptions](CmdLineOptions) struct
    pub fn parse(cmdline: Vec<String>) -> CmdLineResult<Self> {
        let mut arguments = Self::default();
        let mut skip = false;
        for (idx, value) in cmdline[1..].iter().enumerate() {

            if skip {
                skip = false;
                continue;
            }

            match value.as_str() {
               
                "--dry-run" => arguments.dry_run = true,

                "-d" |
                "--disk"    => arguments.disk = true,

                "-f" |
                "--file"    => {
                    arguments.filename =
                        if let Some(name) = cmdline.get(idx + 2) {
                            skip = true;
                            name.to_string()
                        } else {
                            return Err(Box::new(
                                CmdLineError("Please specify the filename")));
                        }
                },

                "-j" |
                "--jobs" => {
                    arguments.threads =
                        if let Some(jobs) = cmdline.get(idx + 2) {
                            if let Ok(jobs) = jobs.parse::<u8>() {
                                skip = true;
                                jobs
                            } else {
                                println!();
                                return Err(Box::new(CmdLineError(
                                    "Please specify a valid number for the no.\
                                     of jobs")));
                            }
                        } else {
                            return Err(Box::new(
                                CmdLineError("Please specify the\
                                              number of jobs")));
                        };
                },

                "-t" |
                "--timeout" => {
                    arguments.timeout =
                        if let Some(timeout) = cmdline.get(idx + 2) {
                            if let Ok(timeout) = timeout.parse::<u8>() {
                                skip = true;
                                timeout
                            } else {
                                return Err(Box::new(
                                    CmdLineError("Please specify a valid number\
                                                  for the timeout")));
                            }
                        } else {
                            return Err(Box::new(
                                CmdLineError("Please specify the timeout value\
                                              in seconds")));
                        };
                },

                "-h" |
                "--help" => {
                    CmdLineOptions::help();
                    std::process::exit(0);
                },

                others => println!("Invalid arg passed: {}", others),
            }
        }

        Ok(arguments)
    }

    /// Print out a help menu to the screen describing how to use this and the
    /// options that are available.
    pub fn help() {
        println!("
Usage: ./zebra [OPTIONS]

Options -

    -h, --help                     Print this help menu and exit

    --dry-run                      Just generate a program, print it out to stdout, execute it and exit
                                   This is false by default.

    -d, --disk                     Tell the fuzzer to save testcases into a file and then use those as args to the engine.
                                   This will result in lots of writes to disk.
                                   If this is not specified, then the fuzzer will pass the testcases via a memory mapped
                                   file however, this involves modifing the engine being fuzzed so it executes programs via a memory mapped file.
                                   This is false by default.

    -j, --jobs <nthreads>          No. of threads to use to run the fuzzer.
                                   Default value of 1 thread.

    -f, --file <path/to/jsengine>  The full path of the js engine to fuzz.

    -t, --timeout <timout in secs> The timeout that is to be applied for each run of jsc.
                                   Default value of 5 seconds.
    ");
    }
}
