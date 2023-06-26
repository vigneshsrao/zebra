use std::collections::HashMap;

use super::super::instruction::Instruction;
use super::super::variable::Variable;
use super::super::operators::*;
use super::super::operation::*;
use super::types::*;
use super::super::opcodes::Opcodes as op;

/// Type Analyzer
///
/// A basic typing system designed to be used by the fuzzer. In the current
/// state the typing system support type propogation and type inference in case
/// the initial type is unknown.

pub struct TypeAnalyzer {
   
    type_map:               HashMap<u32, Type>,
    function_stack:         Vec<(Vec<Variable>, Type)>,
    signature_map:          HashMap<u32, FunctionSignature>,

}

impl TypeAnalyzer {

    pub fn new() -> Self {
        Self {
            type_map:               HashMap::<u32, Type>::new(),
            function_stack:         Vec::<(Vec<Variable>, Type)>::new(),
            signature_map:          HashMap::<u32, FunctionSignature>::new(),
        }
    }

    pub fn set_type(&mut self, variable: &Variable, var_type: Type) {

        // If this variable already exists, then we just add the new type info
        // to the existing type otherwise we create a field for this variable.
        if let Some(cur_type) = self.type_map.get_mut(&variable.0) {
            cur_type.ptype |= var_type.ptype;
            if var_type.shape != Shape::None {
                cur_type.shape = var_type.shape;
            }
        } else {
            self.type_map.insert(variable.0, var_type);
        }
    }

    pub fn get_type(&self, variable: &Variable) -> Type {
        match self.type_map.get(&variable.0) {
            Some(vtype) => *vtype,
            None        => panic!("variable not found")
        }
    }

    pub fn get_signature_for(&self, func: Variable) -> &FunctionSignature {
        self.signature_map.get(&func.0).unwrap()
    }

    pub fn analyze(&mut self, inst: &mut Instruction) {
        match inst.operation.opcode() {

            // ignore if the instruction does not produce an output
            op::Nop         |
            op::EndIf       |
            op::Continue    |
            op::Break       |
            op::BeginElse   |
            op::EndFor => {},

            op::LoadInt       => self.set_type(&inst.output_at(0), Int),
            op::LoadFloat     => self.set_type(&inst.output_at(0), Float),
            op::LoadBool      => self.set_type(&inst.output_at(0), Bool),
            op::LoadString    => self.set_type(&inst.output_at(0), String),
            op::LoadUndefined => self.set_type(&inst.output_at(0), Undefined),

            op::BeginIf => {
                let arg = inst.input_at(0);
                if self.get_type(arg).is_unknown() {
                    self.set_type(arg, Bool | Unknown);
                }
            },

            op::Copy => {
                let t = self.get_type(&inst.input_at(1));
                self.set_type(inst.input_at(0), t);
            },

            op::BeginFor => {
                self.set_type(inst.temp_at(0), Int | Float | Bool);
            }

            // Refer https://tc39.es/ecma262/#sec-applystringornumericbinaryoperator
            op::BinaryOp => {
                let lhs = inst.input_at(0);
                let rhs = inst.input_at(1);
                if self.get_type(lhs).is_unknown() {
                    self.set_type(lhs, Int | Unknown);
                }
                if self.get_type(rhs).is_unknown() {
                    self.set_type(rhs, Int | Unknown);
                }

                let op = inst.cast_into::<BinaryOp>();
                let lhs_type = self.get_type(lhs);
                let rhs_type = self.get_type(rhs);
                let output = inst.output_at(0);
                match op.0 {
                    BinaryOperators::Add => {
                        if lhs_type.is_numeric() && rhs_type.is_numeric() {
                            if lhs_type.is_integer() && rhs_type.is_integer() {
                                self.set_type(&output, Int);
                            } else {
                                self.set_type(&output, Float);
                            }
                        } else {
                            self.set_type(&output, String);
                        }

                    },

                    BinaryOperators::Sub |
                    BinaryOperators::Mul => {
                        if lhs_type.is_integer() && rhs_type.is_integer() {
                            self.set_type(&output, Int);
                        } else {
                            self.set_type(&output, Float);
                        }
                    },

                    BinaryOperators::Div => self.set_type(&output, Float),

                    // all mods might not be ints but yolo it for now
                    BinaryOperators::Mod => self.set_type(&output, Int),

                    BinaryOperators::BitAnd   |
                    BinaryOperators::BitOr    |
                    BinaryOperators::Xor      |
                    BinaryOperators::LShift   |
                    BinaryOperators::RShift   => self.set_type(&output, Int),

                    BinaryOperators::LogicAnd |
                    BinaryOperators::LogicOr  => self.set_type(&output, Bool),
                };
            },

            op::UnaryOp => {
                let lhs = inst.input_at(0);
                if self.get_type(lhs).is_unknown() {
                    self.set_type(lhs, Int | Unknown);
                }
                let op = inst.cast_into::<UnaryOp>();
                let output = inst.output_at(0);
                let input_type = self.get_type(&inst.input_at(0));
                match op.0 {
                    UnaryOperators::Inc         |
                    UnaryOperators::Dec         |
                    UnaryOperators::BitwiseNot  => {
                        if input_type.is_int() || input_type.is_bool() {
                            self.set_type(&output, Int);
                        } else {
                            self.set_type(&output, Float);
                        }
                    },
                    UnaryOperators::LogicalNot  => self.set_type(&output, Bool),
                };
            },

            op::CompareOp => {
                let lhs = inst.input_at(0);
                let rhs = inst.input_at(1);
                if self.get_type(lhs).is_unknown() {
                    self.set_type(lhs, Int | Unknown);
                }
                if self.get_type(rhs).is_unknown() {
                    self.set_type(rhs, Int | Unknown);
                }

                let output = inst.output_at(0);
                self.set_type(&output, Bool);
            },

            op::BeginFunctionDefinition => {

                // When we start a function definition, we first need to find
                // the function name (variable) and signature and map them in
                // the signature map.
                // After that we also push the function name and inputs along
                // with the output type (unknown initially) onto the stack this
                // will be used in the end function definition

                let output_var = *inst.output_at(0);
                let mut inputs: Vec<Variable> = inst.temp().iter().copied()
                                                                  .collect();
                for v in &inputs {
                    self.set_type(&v, Unknown);
                }
                inputs.insert(0, output_var);

                let op = inst.cast_into_mut::<BeginFunctionDefinition>();
                op.0.set_is_constructing();
                let signature = op.0.clone();

                self.function_stack.push((inputs, op.0.get_output_type()));
                self.signature_map.insert(output_var.0, signature);

                self.set_type(&output_var, Function);

            },

            op::EndFunctionDefinition => {

                // When we encounter an EndFuctionDefinition, we need to do 3
                // things -
                // 1. Pop the return type of the stack and set it on the
                // function signature
                //
                // 2. Take the input variables from the stack, find their final
                // types and set them on the signature
                //
                // 3. Inform the signature that we are done constructing this
                // function and this can be called now

                let (mut func_vars, output_type) = self.function_stack.pop().unwrap();
                let func_name = func_vars[0];

                func_vars.remove(0);
                let mut input_types = Vec::<Type>::with_capacity(
                    inst.operation.num_inputs() as usize);


                for v in func_vars.iter() {
                    input_types.push(self.get_type(v));
                }

                let sig = self.signature_map.get_mut(&func_name.0).unwrap();
                sig.set_output_type(output_type);
                sig.set_input_types(input_types);
                sig.done_constructing();

            },

            op::Return => {

                let output_type = self.get_type(&inst.input_at(0));
                let current_type = self.function_stack.last_mut().unwrap();
                current_type.1 |= output_type;

            },

            op::FunctionCall => {

                let func_var = inst.input_at(0);
                let signature = self.signature_map.get(&func_var.0).unwrap();
                let output_type = signature.get_output_type();
                self.set_type(inst.output_at(0), output_type);

            },

            op::CreateArray => {
                let output = inst.output_at(0);
                self.set_type(output, Array);
            },

            op::LoadElement => {
                let output = inst.output_at(0);
                let input  = inst.input_at(0);
                let idx    = inst.input_at(1);
                if self.get_type(input).is_unknown() {
                    self.set_type(input, Array);
                }
                if self.get_type(idx).is_unknown() {
                    self.set_type(idx, Int);
                }
                self.set_type(output, Int | Float | Object);
            },

            op::StoreElement => {
                let array = inst.input_at(0);
                let index = inst.input_at(1);
                let value = inst.input_at(2);

                if self.get_type(array).is_unknown() {
                    self.set_type(array, Array);
                }

                if self.get_type(index).is_unknown() {
                    self.set_type(index, Int);
                }

                if self.get_type(value).is_unknown() {
                    self.set_type(array, Int | Float | Object);
                }
            },

            op::MethodCall => {
                let op = inst.cast_into::<MethodCall>();
                let output = inst.output_at(0);

                let signature = &op.0;
                for (idx, inp) in inst.inputs()[1..].iter().enumerate() {
                    if self.get_type(inp).is_unknown() {
                        let idx = idx % op.0.min_args_count();
                        let itype = match signature.input_type_at(idx) {
                            MethodArg::Type(itype) |
                            MethodArg::Optional(itype) |
                            MethodArg::Repeat(_ , itype) => *itype,
                        };
                        self.set_type(inp, itype);
                    }
                }

                let output_type = op.0.output_type();
                self.set_type(&output, output_type);
            },

            op::LoadProperty => {
                let input = inst.input_at(0);
                if self.get_type(input).is_unknown() {
                    self.set_type(input, Object);
                }
                self.set_type(inst.output_at(0), Float | Int | Object);
            },

            op::StoreProperty => {
                let input = inst.input_at(0);
                let value = inst.input_at(1);
                if self.get_type(input).is_unknown() {
                    self.set_type(input, Object);
                }
                if self.get_type(value).is_unknown() {
                    self.set_type(input, Float | Int | Object);
                }
            },

            op::LoadBuiltin => {
                let op = inst.cast_into::<LoadBuiltin>();
                let otype = match &op.0 {
                    ConstructorType::Callable(ms) => {
                        ms.output_type()
                    },
                    ConstructorType::NonCallable(_, otype) => *otype
                };

                self.set_type(inst.output_at(0), otype);
            }

            op::CreateObject => {
                let custom_type = Type {
                    ptype: PType::Object,
                    shape: Shape::Custom
                };
                self.set_type(inst.output_at(0), custom_type);
            },

            op::Delete => {
                let op = inst.cast_into::<Delete>();
                let is_indexed_prop = op.0;
                let object = inst.input_at(0);
                let prop   = inst.input_at(1);
                if is_indexed_prop && self.get_type(prop).is_unknown() {
                    self.set_type(prop, Int);
                }

                if self.get_type(object).is_unknown() {
                    let custom_type = Type {
                        ptype: PType::Object,
                        shape: Shape::Custom
                    };
                    self.set_type(object, custom_type);
                }
            },


            // op => assert!(false, "Unimplemented types for opcode {:?}", op),
        };

    }

    // #[cfg(debug_assertions)]
    pub fn _debug_print(&self) {
        for (v,t) in &self.type_map {
            println!("v{:?} => {:?}", v,t);
        }
    }

}
