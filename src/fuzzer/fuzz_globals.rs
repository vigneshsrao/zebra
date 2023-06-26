//! This module is mainly for handling the data that will remain constant or is
//! common for all the Fuzzers. This includes the user preferences, JS Runtime
//! constants and statistics.

use std::sync::RwLock;
use std::thread;
use std::time::{Duration, Instant};

use crate::jsruntime::jsruntime::JSRuntime;
use crate::cmdlineoptions::CmdLineOptions;
use super::stats::Stats;

/// This holds the data that will not change during the fuzzing runs like the
/// user provided options, JS constants etc.
pub struct FuzzGlobals {
    pub program_name: String,
    pub cmdline:      CmdLineOptions,
    pub stats:        RwLock<Stats>,
    pub jsruntime:    JSRuntime,
}

impl FuzzGlobals {

    /// Initialize and create a new instance of the fuzzing global values.
    pub fn new(name: String, cmdline: CmdLineOptions, jsruntime: JSRuntime)
               -> Self {

        Self {
            program_name: name,
            cmdline:      cmdline,
            stats:        RwLock::new(Stats::default()),
            jsruntime:    jsruntime,
        }
    }

    /// Update the global store from the data that is collected by the worker
    /// threads. TODO: Make this thread safe by adding a RW lock
    pub fn update(&self, stats: &Stats ) {

        // All the updates are done in blocks of their own so that the lock is
        // dropped when the write is done.
        {
            let mut gstats = self.stats.write().expect("Lock Poisoned");
            gstats.update(stats);
        }

    }


    /// The loop that will run on the main thread. This loop only prints out the
    /// statistics to the screen once every second
    pub fn mainloop(&self, start: Instant) {

        loop {
            // The reporting is done once every second
            thread::sleep(Duration::from_millis(3000));

            // Print out the current statistics
            self.stats.read().unwrap().print(&start);
        }
    }

}
