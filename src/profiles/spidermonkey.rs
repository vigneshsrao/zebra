use super::profile::Profile;

pub struct SpidermonkeyProfile {
    args: Vec<&'static str>,
}

impl Profile for SpidermonkeyProfile {
    fn get_args(&self) -> &Vec<&'static str> {
        &self.args
    }
}

impl SpidermonkeyProfile {
    pub fn new(repl: bool) -> Self {
        let mut args = vec![
                "--baseline-warmup-threshold=10",
                "--ion-warmup-threshold=100",
                "--ion-check-range-analysis",
                "--ion-extra-checks",
                "--fuzzing-safe",
        ];

        if repl {
            args.push("--reprl");
        }

        SpidermonkeyProfile {
            args
        }
    }
}
