use super::variable::Variable;
use super::operation::*;

/// If the opcode is a primitive opcode then it will have a value associated
/// with it. This enum is used to represent that value
#[derive(Debug, Clone)]
pub enum Value {
    Int(isize),
    Float(f64),
    Str(String),
    Bool(bool),
    Undefined,
    None,
}

/// A Zebra IR instruction. This will contain all the runtime data required for
/// the operation of a single Operation.
pub struct Instruction {
    pub idx:        u32,
    pub operation:  Box<dyn Operation>,
    pub inputs:     Vec<Variable>,
    pub outputs:    Vec<Variable>,
    pub temp:       Vec<Variable>,
}

impl Instruction {

    pub fn new(idx: u32, operation: Box<dyn Operation>, inputs: Vec<Variable>,
               outputs: Vec<Variable>, temp: Vec<Variable>,) -> Self {

        debug_assert!(inputs.len() == operation.num_inputs() as usize,
                      "Incorrect no. of inputs provided. Expected {}, got {}",
                      operation.num_inputs() ,inputs.len());

        Self {
            idx:       idx,
            operation: operation,
            inputs:    inputs,
            outputs:   outputs,
            temp:      temp,
        }
    }

    pub fn getval(&self) -> Value {

        // Assert that we are indeed calling the getval function on a valid
        // opcode type
        assert_eq!(self.operation.attributes() & Attributes::IS_PRIMITIVE,
                   Attributes::IS_PRIMITIVE,
                   "Invalid Opcode {:?} called getval() ",
                   self.operation.opcode());

        // First cast the operation to `Any` type
        let val = self.operation.as_any();

        // Now find the concrete type of the operation
        if let Some(val) = val.downcast_ref::<LoadInt>() {
            Value::Int(val.0)
        } else if let Some(val) = val.downcast_ref::<LoadFloat>() {
            Value::Float(val.0)
        } else if let Some(val) = val.downcast_ref::<LoadBool>() {
            Value::Bool(val.0)
        } else if let Some(val) = val.downcast_ref::<LoadString>() {
            Value::Str(val.0.clone())
        } else if let Some(_) = val.downcast_ref::<LoadUndefined>() {
            Value::Undefined
        } else {
            debug_assert!(false, "unreachable branch");
            Value::None
        }
    }

    /// Helper functions to get the value at the nth position of the
    /// input/output/temp vectors
   
    pub fn input_at(&self, idx: usize) -> &Variable {
        debug_assert!(idx < self.inputs.len(), "Invalid idx provided");
        &self.inputs[idx]
    }

    pub fn output_at(&self, idx: usize) -> &Variable {
        debug_assert!(idx < self.outputs.len(), "Invalid idx provided");
        &self.outputs[idx]
    }

    pub fn temp_at(&self, idx: usize) -> &Variable {
        debug_assert!(idx < self.temp.len(), "Invalid idx {} provided for len {}", idx, self.temp.len());
        &self.temp[idx]
    }

    /// Helper functions to get all the inputs and outputs of this instruction

    pub fn inputs(&self) -> &Vec<Variable> {
        &self.inputs
    }

    pub fn outputs(&self) -> &Vec<Variable> {
        &self.outputs
    }

    pub fn temp(&self) -> &Vec<Variable> {
        &self.temp
    }

    pub fn cast_into<T: Operation + 'static>(&self) -> &T {
        self.operation.as_any().downcast_ref::<T>().unwrap()
    }

    pub fn cast_into_mut<T: Operation + 'static>(&mut self) -> &mut T {
        self.operation.as_any_mut().downcast_mut::<T>().unwrap()
    }

    /// Display the instruction. Only valid for debugging
    #[cfg(debug_assertions)]
    pub fn _print(&self) -> String {
        use super::opcodes::Opcodes;

        let mut s: String = String::new();
        for i in &self.outputs {
            s.push_str(&i.print());
            s.push_str(&", ");
        }

        if self.operation.num_outputs() != 0 {
            s.remove(s.len() - 2);
            s.push_str(&"= ");
        }
        s.push_str(&format!("{:?}(", self.operation.opcode()));


        if self.operation.opcode() == Opcodes::BeginFor {

            let tmp =  self.temp_at(0);
            let op =  self.operation.as_any().downcast_ref::<BeginFor>().unwrap();
            let out = format!("{} = {}, {} {} {}, {}{}",
                        tmp.print(), self.input_at(0).print(),
                        tmp.print(), op.1.rep(), self.input_at(1).print(),
                        tmp.print(), op.0);
            s.push_str(&out);

        } else {

            if self.operation.is_primitive() {
                s.push_str(&format!("{:?}", self.getval()));
            }

            for i in &self.inputs {
                s.push_str(&i.print());
                s.push_str(&", ");
            }

            if self.operation.num_inputs() != 0 {
                s.remove(s.len() - 2);
                s.remove(s.len() - 1);
            }

        }

        s.push_str(&")");
       
        s
    }
}

//////////////////////////////////////////////
//////////////// TESTS ///////////////////////
//////////////////////////////////////////////

#[cfg(test)]
pub mod test {
    use super::*;
    use crate::operators::*;
    use crate::operation as ops;
    use crate::variable::Variable;

    pub fn _createinst() -> Vec<Instruction> {

        let uop = ops::UnaryOp(UnaryOperators::Inc);

        vec![
            Instruction::new(0, Box::new(ops::LoadInt(1337)),
                             vec![],
                             vec![Variable(1)],
                             vec![]),

            Instruction::new(0, Box::new(ops::LoadFloat(133.37)),
                             vec![],
                             vec![Variable(2)],
                             vec![]),

            Instruction::new(0, Box::new(ops::LoadUndefined()),
                             vec![],
                             vec![Variable(3)],
                             vec![]),

            Instruction::new(0, Box::new(ops::LoadBool(true)),
                             vec![],
                             vec![Variable(4)],
                             vec![]),

            Instruction::new(0, Box::new(ops::LoadString("Hello all".to_string())),
                             vec![],
                             vec![Variable(5)],
                             vec![]),

            Instruction::new(0, Box::new(ops::BeginFor("++".to_string(), Comparators::LessThan)),
                             vec![Variable(7), Variable(8), Variable(9)],
                             vec![],
                             vec![Variable(10)]),

            Instruction::new(0, Box::new(ops::BeginIf()),
                             vec![Variable(4)],
                             vec![],
                             vec![]),

            Instruction::new(0, Box::new(ops::BeginFor("++".to_string(), Comparators::LessThan)),
                             vec![Variable(7), Variable(8), Variable(9)],
                             vec![],
                             vec![Variable(10)]),

            Instruction::new(0, Box::new(ops::BinaryOp(BinaryOperators::Add)),
                             vec![Variable(1), Variable(2)],
                             vec![Variable(6)],
                             vec![]),

            Instruction::new(0, Box::new(ops::EndFor()),
                             vec![],
                             vec![],
                             vec![]),

            Instruction::new(0, Box::new(ops::BeginElse()),
                             vec![],
                             vec![],
                             vec![]),

            Instruction::new(0, Box::new(uop),
                             vec![Variable(1)],
                             vec![Variable(6)],
                             vec![]),

            Instruction::new(0, Box::new(ops::EndIf()),
                             vec![],
                             vec![],
                             vec![]),

            Instruction::new(0, Box::new(ops::EndFor()),
                             vec![],
                             vec![],
                             vec![]),

            Instruction::new(0, Box::new(ops::LoadBool(true)),
                             vec![],
                             vec![Variable(7)],
                             vec![]),
        ]

    }

    // #[test]
    // fn testinst() {

    //     for i in createinst() {
    //         println!("{}", i.print());
    //     }
    // }
}
