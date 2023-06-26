use std::time::Instant;

#[derive(Clone, Debug, Default)]
pub struct Stats {
    pub iter:       u64,
    pub crashes:    u64,
    pub timeouts:   u64,
    pub incorrect:  u64,
}

impl Stats {
    pub fn reset(&mut self) {
        self.iter      = 0;
        self.crashes   = 0;
        self.timeouts  = 0;
        self.incorrect = 0;
    }

    pub fn print(&self, start: &Instant) {
        let esc = 27 as char;
        // let esc = 61 as char;
        let elapsed = start.elapsed();
        let total_samples = self.iter;
        let total_crashes = self.crashes;
        let correctness = 100.0 - (((self.incorrect + self.timeouts) as f64 /(total_samples as f64))*100.0);
        let fcps = total_samples as f64 / ((elapsed.as_micros()) as f64 / 1000000 as f64);
        // println!("{}[2J{}[1;1H\
        println!("
-----------------------
fcps            = {:.0}/s
Timeouts        = {}
Crashes         = {}
Incorrect Cases = {}
Correctness     = {:.2}%
Runtime         = {} seconds
Total Cases     = {}",
                 // esc,
                 // esc,
                 fcps,
                 self.timeouts,
                 total_crashes,
                 self.incorrect,
                 correctness,
                 elapsed.as_secs(),
                 total_samples
        );
    }

    pub fn update(&mut self, other: &Stats) {
        self.iter      += other.iter;
        self.crashes   += other.crashes;
        self.timeouts  += other.timeouts;
        self.incorrect += other.incorrect;
    }
}
