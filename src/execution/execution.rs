/// This will be the status when the target finishes execution.
/// * Timeout: the target timed out
/// * Crash(code): The target crashed with the signal number `code`
/// * Status(code): The target successfully executed and returned `code`
pub enum ReturnCode {
    Timeout,
    Crash(i32),
    Status(i32)
}

pub trait Execution {
    // fn new(path: String, args: Vec<&'static str>, timeout: u32) -> Self;
    fn execute(&mut self, input: &String) -> ReturnCode;
}
