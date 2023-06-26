use std::fs::File;
use std::sync::Arc;
use std::io::{self, Write};

use crate::ir::program::Program;
use crate::lifter::lifter::Lifter;
use crate::execution::execution::{ReturnCode, Execution};
use crate::execution::repl::ReplConnection;
use crate::execution::spawn::Spawn;

use super::stats::Stats;
use super::fuzz_globals::FuzzGlobals;

/// The amount of iterations after which we should update the statistics of each
/// thread on to the `Globals` stat
const REPORT_INTERVEL: u64 = 10;

/// Create `filename` and write `data` to it
pub fn write_file(filename: &str, data: &String) -> io::Result<()> {
    let mut file = File::create(filename)?;
    file.write(data.as_bytes())?;
    Ok(())
}

pub struct Fuzzer {
    id:         u8,
    stats:      Stats,
    lifter:     Lifter,
    globals:    Arc<FuzzGlobals>,
    exec:       Box<dyn Execution>,
}

impl Fuzzer {
    pub fn new(id: u8, globals: Arc<FuzzGlobals>) -> Self {

        let mut args = vec![
                "--baseline-warmup-threshold=10",
                "--ion-warmup-threshold=100",
                "--ion-check-range-analysis",
                "--ion-extra-checks",
                "--fuzzing-safe",
        ];

        if !globals.cmdline.disk {
            args.push("--reprl");
        }

        let exec: Box<dyn Execution> = if globals.cmdline.disk {
            Box::new(Spawn::new(globals.cmdline.filename.to_string(),
                                args, globals.cmdline.timeout as u32))
        } else {
            Box::new(ReplConnection::new(globals.cmdline.filename.to_string(),
                                         args, globals.cmdline.timeout as u32))
        };

        Self {
            id:         id,
            stats:      Stats::default(),
            lifter:     Lifter::new(),
            globals:    globals,
            exec:       exec,
        }
    }

    /// The fuzzing front end that will call the fuzz_one function and update
    /// the global data
    pub fn fuzzloop(&mut self) {
        loop {

            for _ in 0..REPORT_INTERVEL {

                // Perform one round of fuzzing
                self.fuzz_one();

                // If this is a dry run then just exit here
                if self.globals.cmdline.dry_run {
                    return;
                }

            }

            // Update the stats of this thread to the global pool
            self.globals.update(&self.stats);

            // Reset the thread local stats
            self.stats.reset();
        }
    }

    /// The core fuzzing logic. This function performs one round of fuzzing on
    /// the target binary.
    fn fuzz_one(&mut self) {

        let mut program = Program::new(&self.globals.jsruntime);
        self.lifter.reset();

        // Create an IR with at least 10 instructions
        program.generate_random_insts(5);

        // Now lift that IR into JavaScript
        self.lifter.do_lifting(program);

        // Finalize the JS code. No more additions to the code will be done
        self.lifter.finalize();

        // Execute the program and handle how it returns
        self.execute();

        // Update the stats
        self.stats.iter += 1;

    }

    /// Executes the JS program passed. Returns true if the program crashed,
    /// else returns false
    fn execute(&mut self) {


        let program = self.lifter.get_code();

        // if self.globals.cmdline.disk {

        //     // Write out the program to disk file
        //     _write_file(&self.pname, program).expect("Error when writting out to file");
        // } else {

        //     // Memcpy the program into the target address
        //     let size = std::cmp::min(program.len(), 0x1000-1);
        //     unsafe {
        //         core::ptr::copy(program.as_ptr(),
        //                         self.address.unwrap() as *mut u8, size);
        //     };
        // }

        if self.globals.cmdline.dry_run {
            println!("{}", self.lifter.get_code());
        }



        // JSC specific args for properly fuzzing jsc. TODO: make a profile like
        // fuzzilli and add these things there. Right now only add these args if
        // fuzzing in memfd mode and not in disk mode.
        // let cmdargs = if self.globals.cmdline.disk {
        //     Vec::<&'static str>::new()
        // } else {
            // vec![
            //     "-repl",
            //     "--validateOptions=true",
            //     "--useConcurrentJIT=false",
            //     // No need to call functions thousands of times before they are JIT compiled
            //     "--thresholdForJITSoon=10",
            //     "--thresholdForJITAfterWarmUp=10",
            //     "--thresholdForOptimizeAfterWarmUp=100",
            //     "--thresholdForOptimizeAfterLongWarmUp=100",
            //     "--thresholdForOptimizeSoon=100",
            //     "--thresholdForFTLOptimizeAfterWarmUp=1000",
            //     "--thresholdForFTLOptimizeSoon=1000",
            //     // Enable bounds check elimination validation
            //     // "--validateBCE=true"
            // ]

        // };

        let return_code = self.exec.execute(program);

        match return_code {
            ReturnCode::Timeout => {
                self.stats.timeouts += 1;
            },
            ReturnCode::Status(code) => {
                if code != 0 {
                    self.stats.incorrect += 1;
                }
            },
            ReturnCode::Crash(signal) => {
                self.save(signal);
                self.stats.crashes += 1;
            }
        }
    }

    fn save(&self, signal: i32) {
        let rand = unsafe { std::arch::x86_64::_rdtsc() };
        let filename = format!("crashes/crash.{}.{}.{}.js",
                                self.id, self.stats.iter, rand);
        let tosave = format!("{}\n\n// Crash with Signal: {}\n",
                             self.lifter.get_code(), signal);
        write_file(&filename, &tosave)
            .expect("Failed to write crash to file");
    }
}
