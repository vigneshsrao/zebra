/// Trait to hold the public functions of all the profiles
pub trait Profile {
    /// This will return the command line arguments for the profile selected
    fn get_args(&self) -> &Vec<&'static str>;
}

/// Types of Profiles allowed
#[derive(Debug)]
pub enum ProfileType {
    Spidermonkey,
    Jsc,
    V8,
}
