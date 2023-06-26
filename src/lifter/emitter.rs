const INDENT_SPACES: u8 = 3;

pub struct Emitter {
    code: String,
    indent_level: u8,
}

impl Emitter {
    pub fn new() -> Self {
        Self {
            code: String::new(),
            indent_level: 0,
        }
    }

    pub fn indent(&mut self) {
        self.indent_level += INDENT_SPACES;
    }

    pub fn unindent(&mut self) {
        debug_assert!(self.indent_level > 0, "Unbalanced unindent");
        self.indent_level -= INDENT_SPACES;
    }

    pub fn add(&mut self, code: String) {
        self.code += &" ".repeat(self.indent_level as usize);
        self.code += &code;
        self.code.push('\n');
    }

    pub fn get_code(&self) -> &String {
        &self.code
    }

    pub fn finalize(&mut self) {
        // self.code += "gc();\ngc();\n\0";
        // self.code += "\0";
    }

    pub fn reset(&mut self) {
        self.code.clear();
    }
}
