use super::profile::Profile;

pub struct JavaScriptCoreProfile {
    args: Vec<&'static str>,
}

impl Profile for JavaScriptCoreProfile {
    fn get_args(&self) -> &Vec<&'static str> {
        &self.args
    }
}

impl JavaScriptCoreProfile {
    pub fn new(repl: bool) -> Self {

        let mut args = vec![
                "--validateOptions=true",
                "--useConcurrentJIT=false",
                // No need to call functions thousands of times before they are JIT compiled
                "--thresholdForJITSoon=10",
                "--thresholdForJITAfterWarmUp=10",
                "--thresholdForOptimizeAfterWarmUp=100",
                "--thresholdForOptimizeAfterLongWarmUp=100",
                "--thresholdForOptimizeSoon=100",
                "--thresholdForFTLOptimizeAfterWarmUp=1000",
                "--thresholdForFTLOptimizeSoon=1000",
                // Enable bounds check elimination validation
                // "--validateBCE=true"
        ];

        if repl {
            args.push("--reprl");
        }

        JavaScriptCoreProfile {
            args
        }
    }
}
