use std::fs;
use std::thread;
use std::sync::Arc;
use std::time::Instant;

mod ir;
mod lifter;
mod utils;
mod fuzzer;
mod jsruntime;
mod cmdlineoptions;
mod execution;

use fuzzer::fuzzer::Fuzzer;
use cmdlineoptions::CmdLineOptions;
use jsruntime::jsruntime::JSRuntime;
use fuzzer::fuzz_globals::FuzzGlobals;

extern "C" {
    fn signal(signum: i32, handler: *const ());
}

fn handle() {
    println!("CTRL-C!");
    std::process::exit(-1);
}

/// The function that will create all the fuzzers and invoke them to start
/// fuzzing. This will only ever return out if this is a dry run fuzzing test.
fn fuzz(cmdline: CmdLineOptions) {

    let nthreads   = cmdline.threads;
    let is_dry_run = cmdline.dry_run;
    let start = Instant::now();

    let runtime: JSRuntime = JSRuntime::new();
    let globals = FuzzGlobals::new("test".to_string(), cmdline, runtime);

    let mut threads = vec![];

    let globals = Arc::new(globals);
    for i in 0..nthreads {
        let globals = globals.clone();
        let t = thread::spawn(move || {
            let mut fuzzer = Fuzzer::new(i, globals);
            fuzzer.fuzzloop();
        });

        threads.push(t);
    }


    if is_dry_run {
        for t in threads {
            let _ = t.join();
        }
        return;
    }

    unsafe {
        signal(2, handle as *const ());
    }

    globals.mainloop(start);
}

/// Creates the directories that will be used by the fuzzers for storing
/// testcases and crashes.
fn prepare_dir() -> std::io::Result<()> {
    fs::create_dir_all("./tests")?;
    fs::create_dir_all("./crashes")?;
    Ok(())
}


fn main() {

    let cmdline: Vec<String> = std::env::args().collect();

    let cmdline_options = match CmdLineOptions::parse(cmdline) {
        Ok(cmd)  => cmd,
        Err(err) => {
            println!("Invalid cmd line syntax found: {}", err);
            CmdLineOptions::help();
            return;
        }
    };

    match prepare_dir() {
        Ok(_)    => {},
        Err(err) => {
            println!("Error occured while creating the directories: {}", err);
            return;
        }
    };

    fuzz(cmdline_options);

}
