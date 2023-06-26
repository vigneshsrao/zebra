use super::emitter::Emitter;
use crate::ir::program::Program;
use crate::ir::instruction::{Instruction, Value};
use crate::ir::opcodes::Opcodes as op;
use crate::ir::operators::*;
use crate::ir::operation::*;
use crate::ir::codeanalysis::types::ConstructorType;
use crate::utils::probablity::Probablity;
use crate::utils::random::Random;

pub struct Lifter {
    emitter: Emitter,
    probablity: Probablity,
}

impl Lifter {
    pub fn new() -> Self {
        Self {
            emitter: Emitter::new(),
            probablity: Probablity::new(Random::new(0)),
        }
    }

    pub fn do_lifting(&mut self, program: Program ) {
        for i in program.buffer {
            self.lift(&i);
        }
    }

    pub fn get_code(&self) -> &String {
        self.emitter.get_code()
    }

    pub fn reset(&mut self) {
        self.emitter.reset();
    }

    pub fn finalize(&mut self) {
        self.emitter.finalize();
    }

    fn lift(&mut self, inst: &Instruction) {

        match inst.operation.opcode() {

            op::Nop => {},

            op::LoadInt    |
            op::LoadFloat  |
            op::LoadString |
            op::LoadBool   |
            op::LoadUndefined => {
                let mut code = "var ".to_owned();
                code += &inst.output_at(0).print();
                code += &" = ";
                let val = inst.getval();

                match val {
                    Value::Int(val) => code += &val.to_string(),
                    Value::Float(val) => code += &val.to_string(),
                    Value::Str(val) => {
                        code.push('"');
                        code +=  &val.to_string();
                        code.push('"');
                    },
                    Value::Bool(val) => code += &val.to_string(),
                    Value::Undefined => code += "undefined",
                    Value::None => assert!(false, "Incorrect value for: {:?}",
                                           inst.operation.opcode()),
                }

                code.push(';');
                self.emitter.add(code);
            },

            op::Copy => {
                let lhs = inst.input_at(0).print();
                let rhs = inst.input_at(1).print();
                let code = format!("var {} = {};", lhs, rhs);
                self.emitter.add(code);
            },

            op::BeginIf => {
                let mut code = "if (".to_owned();
                code += &inst.input_at(0).print();
                code += ") {";
                self.emitter.add(code);
                self.emitter.indent();
            },

            op::BeginElse => {
                self.emitter.unindent();
                self.emitter.add("} else {".to_owned());
                self.emitter.indent();
            },

            op::EndIf => {
                self.emitter.unindent();
                self.emitter.add("}".to_owned());
            },

            op::BeginFor => {

                let tmp = inst.temp_at(0);
                let op = inst.cast_into::<BeginFor>();

                let mut code = format!("for (var {} = {}; {} {} {}; {}{})",
                                   tmp.print(), inst.input_at(0).print(),
                                   tmp.print(), op.1.rep(),
                                   inst.input_at(1).print(),
                                   tmp.print(), op.0);

                code.push('{');
                self.emitter.add(code);
                self.emitter.indent();
            },

            op::EndFor => {
                self.emitter.unindent();
                self.emitter.add("}".to_owned());
            },

            op::Break => {
                self.emitter.add("break;".to_owned());
            },

            op::Continue => {
                self.emitter.add("continue;".to_owned());
            }

            op::BinaryOp => {
                let op = inst.cast_into::<BinaryOp>();
                let out = inst.output_at(0);
                let lhs = inst.input_at(0);
                let rhs = inst.input_at(1);
                self.emitter.add(format!("var {} = {} {} {};",
                                         out.print(), lhs.print(),
                                         op.0.rep(), rhs.print()
                ));
            }

            op::UnaryOp => {
                let op = inst.cast_into::<UnaryOp>();
                let out = inst.output_at(0);
                let lhs = inst.input_at(0);
                let code = match op.0 {
                    UnaryOperators::Inc | UnaryOperators::Dec => {
                        format!("var {} = {}{};", out.print(), lhs.print(),
                                op.0.rep())
                    },
                    _ => {
                        format!("var {} = {}{};", out.print(), op.0.rep(),
                                lhs.print())
                    }
                };

                self.emitter.add(code);
            },

            op::CompareOp => {
                let op = inst.cast_into::<CompareOp>();
                let out = inst.output_at(0);
                let lhs = inst.input_at(0);
                let rhs = inst.input_at(1);
                self.emitter.add(format!("var {} = {} {} {};",
                                         out.print(), lhs.print(),
                                         op.0.rep(), rhs.print()
                ));
            },

            op::BeginFunctionDefinition => {
                let mut code = format!("function {}(", inst.output_at(0).print());
                for v in inst.temp() {
                    code.push_str(&v.print());
                    code += ", ";
                }
                if inst.temp.len() != 0 {
                    code.remove(code.len()-2);
                }
                code += ") {";
                self.emitter.add(code);
                self.emitter.indent();

            },

            op::EndFunctionDefinition => {
                self.emitter.unindent();
                self.emitter.add("}".to_string());
            },

            op::Return => {
                let code = format!("return {};", inst.input_at(0).print());
                self.emitter.add(code);
            },

            op::FunctionCall => {

                let inputs = inst.inputs();
                let function_name = inst.input_at(0);
                let output = inst.output_at(0);

                let mut code = format!("var {} = {}(",
                                       output.print(),
                                       function_name.print());

                for v in &inputs[1..] {
                    code.push_str(&v.print());
                    code += ", ";
                }

                if inputs.len() > 1 {
                    code.remove(code.len()-2);
                }

                code += ");";

                self.emitter.add(code);

            },

            op::CreateArray => {
                let mut code = "var ".to_string() + &inst.output_at(0).print();
                let inputs = &inst.inputs().iter()
                                           .map(|x| x.print())
                                           .collect::<Vec<String>>().join(", ");
                if self.probablity.probablity(0.5) {
                    code += " = [";
                    code += inputs;
                    code += "];";
                } else {
                    code += " = Array(";
                    code += inputs;
                    code += ");";
                }
                self.emitter.add(code);
            },

            op::LoadElement => {
                let array  = inst.input_at(0).print();
                let index  = inst.input_at(1).print();
                let output = inst.output_at(0).print();
                let code   = format!("var {} = {}[{}];", output, array, index);
                self.emitter.add(code);
            },

            op::StoreElement => {
                let array = inst.input_at(0).print();
                let index = inst.input_at(1).print();
                let value = inst.input_at(2).print();
                let code  = format!("{}[{}] = {};", array, index, value);
                self.emitter.add(code);
            },

            op::MethodCall => {
                let op = inst.cast_into::<MethodCall>();
                let inps = &inst.inputs()[1..];
                let args = inps.iter().map(|x| x.print())
                                      .collect::<Vec<String>>().join(", ");

                let code = format!("var {} = {}.{}({});",
                                   inst.output_at(0).print(),
                                   inst.input_at(0).print(),
                                   op.0.get_name(), args);

                self.emitter.add(code);
            },

            op::LoadProperty => {
                let op = inst.cast_into::<LoadProperty>();
                let code = format!("var {} = {}.{}",
                                   inst.output_at(0).print(),
                                   inst.input_at(0).print(), op.0);
                self.emitter.add(code);
            },

            op::StoreProperty => {
                let op = inst.cast_into::<StoreProperty>();
                let obj = inst.input_at(0);
                let val = inst.input_at(1);
                let code = if self.probablity.probablity(0.7) {
                    format!("{}.{} = {}", obj.print(), op.0, val.print())
                } else {
                    format!("{}[\"{}\"] = {}", obj.print(), op.0, val.print())
                };

                self.emitter.add(code);
            }

            op::LoadBuiltin => {
                let op = inst.cast_into::<LoadBuiltin>();
                let output = inst.output_at(0).print();
                let code;
                match &op.0 {
                    ConstructorType::Callable(ms) => {
                        let inps = &inst.inputs();
                        let args = inps.iter().map(|x| x.print())
                                              .collect::<Vec<String>>()
                                              .join(", ");

                        code = format!("var {} = new {}({});",
                                        output, ms.get_name(), args);
                    },
                    ConstructorType::NonCallable(name, _) => {
                        code = format!("var {} = {}", output, name)
                    }
                };

                self.emitter.add(code);
            },

            op::CreateObject => {
                let op = inst.cast_into::<CreateObject>();
                let output = inst.output_at(0);
                let object = op.0.iter()
                                 .zip(inst.inputs())
                                 .map(|(prop, val)| format!("{}: {}", prop,
                                                            val.print()))
                                 .collect::<Vec<String>>().join(", ");

                let code = format!("var {} = {{{}}};", output.print(), object);
                self.emitter.add(code);


            },

            op::Delete => {
                let object = inst.input_at(0);
                let prop   = inst.input_at(1);
                let code = format!("delete {}[{}]",
                                   object.print(), prop.print());
                self.emitter.add(code);
            }

            // op => assert!(false, "Unimplemented opcode for lifting : {:?}", op),
        }
    }
}
