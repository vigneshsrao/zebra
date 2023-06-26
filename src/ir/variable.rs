#[derive(Debug,Clone,Copy)]
pub struct Variable(pub u32);

impl Variable {
    pub fn print(&self) -> String {
        format!("v{}",self.0)
    }
}
