//! Contains the code associated with harnessing the target being fuzzed. This
//! offers two modes -
//!
//! * Repl - A read-eval-print-loop which means that the target will not be
//! spawned each time and will instead only be spawned when it times out or
//! crashes. This mode also supports memory mapped trasfer of JS files to the
//! target which will reduce the disk overhead. However for this mode to be
//! used, the target code needs to be modified with the Fuzzilli patch. If using
//! a engine which cannot be patched, use the disk mode.
//!
//! * Disk - The generated JS code is written to the disk and then the target is
//! invoked to run this program. This requires no modification of the target but
//! will also incur the overheads of disk usage.

pub mod repl;
pub mod execution;
pub mod ffi;
pub mod spawn;
