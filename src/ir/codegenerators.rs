use super::operators::*;
use super::program::{Program, Mode};
use super::variable::Variable;
// use super::codeanalysis::types::{Type, PType, Shape, FunctionSignature};
use super::codeanalysis::types::*;

use crate::jsruntime::constants::PROPERTIES;

pub struct CodeGenerators();

const DEBUG: bool = false;

impl CodeGenerators {

    pub fn integer_literal_generator(program: &mut Program) -> Option<()> {

        if DEBUG {
            println!("integer_literal_generato");
        }

        let int = program.getint();
        program.load_int(int);
        Some(())
    }

    pub fn float_literal_generator(program: &mut Program) -> Option<()> {

        if DEBUG {
            println!("float_literal_generator");
        }

        let float = program.getfloat();
        program.load_float(float);
        Some(())
    }

    pub fn string_literal_generator(program: &mut Program) -> Option<()> {

        if DEBUG {
            println!("string_literal_generator");
        }

        let string = program.getstring().to_string();
        program.load_string(string);
        Some(())
    }

    pub fn bool_literal_generator(program: &mut Program) -> Option<()> {

        if DEBUG {
            println!("bool_literal_generator");
        }

        let boolean = program.prob.probablity(0.5);
        program.load_bool(boolean);
        Some(())
    }

    pub fn undefined_literal_generator(program: &mut Program) -> Option<()> {

        if DEBUG {
            println!("undefined_literal_generator");
        }

        program.load_undefined();
        Some(())
    }

    pub fn nop_generator(program: &mut Program) -> Option<()> {

        if DEBUG {
            println!("nop_generator");
        }

        program.nop();
        Some(())
    }

    pub fn copy_generator(program: &mut Program) -> Option<()> {

        if DEBUG {
            println!("copy_generator");
        }


        let lhs = program.random_variable(Any);
        let rhs = program.random_variable(Any);
        program.copy(lhs, rhs);
        Some(())
    }

    pub fn if_condition_generator(program: &mut Program) -> Option<()> {

        if DEBUG {
            println!(" if_condition_generator");
        }

        let cond = program.random_variable(Bool);
        let var  = program.random_variable(Unknown);

        program.begin_if(cond);
        program.generate_random_insts(2);
        let tmp = program.random_variable(Any);
        program.copy(var, tmp);

        program.begin_else();
        program.generate_random_insts(2);
        let tmp = program.random_variable(Any);
        program.copy(var, tmp);
        program.end_if();

        Some(())
    }

    pub fn for_loop_generator(program: &mut Program) -> Option<()> {

        if DEBUG {
            println!("for_loop_generator");
        }

        let (start, end, step) = if program.prob.probablity(0.7) {
            let start = program.load_int(0);
            let end = program.load_int(0x500);
            let step = program.load_int(1);
            (start, end, step)
        } else {
            let start = program.random_variable(Int);
            let end = program.random_variable(Int);
            let step = program.random_variable(Int);
            (start, end, step)
        };

        let copy = program.random_variable(Any);

        program.begin_for(start, end, step,
                          "++".to_owned(), Comparators::LessThan);

        program.generate_random_insts(2);
        let tmp = program.random_variable(Any);
        program.copy(copy, tmp);
        program.end_for();
       
        Some(())
    }

    pub fn break_generator(program: &mut Program) -> Option<()> {

        if DEBUG {
            println!(" break_generator");
        }

        if program.is_in_loop() {
            program.insert_break();
            Some(())
        } else {
            None
        }
    }

    pub fn continue_generator(program: &mut Program) -> Option<()> {

        if DEBUG {
            println!("continue_generator");
        }

        if program.is_in_loop() {
            program.insert_continue();
            Some(())
        } else {
            None
        }
    }

    pub fn binary_op_generator(program: &mut Program) -> Option<()> {

        if DEBUG {
            println!("binary_op_generator");
        }

        let lhs = program.random_variable(Int | Float);
        let rhs = program.random_variable(Int | Float);

        let binary_op = BinaryOperators::all();
        let binary_op = program.rng.random_element(&binary_op);
        program.binary_op(lhs, rhs, *binary_op);

        Some(())
    }

    pub fn compare_op_generator(program: &mut Program) -> Option<()> {

        if DEBUG {
            println!("compare_op_generator");
        }

        let lhs = program.random_variable(Int | Float);
        let rhs = program.random_variable(Int | Float);

        let compare_op = Comparators::all();
        let compare_op = program.rng.random_element(&compare_op);
        program.compare_op(lhs, rhs, *compare_op);

        Some(())
    }

    pub fn unary_op_generator(program: &mut Program) -> Option<()> {

        if DEBUG {
            println!("unary_op_generator");
        }

        let lhs = program.random_variable(Int);

        let unary_op = UnaryOperators::all();
        let unary_op = program.rng.random_element(&unary_op);
        program.unary_op(lhs, *unary_op);

        Some(())
    }

    // TODO: Fix this crap. This will create super large size of programs if
    // unchecked.
    pub fn function_definition_generator(program: &mut Program) -> Option<()> {

        if DEBUG {
            println!("function_definition_generator");
        }


        let args_count = program.rng.rand_in_range(0, 5) as u8;
        let signature = FunctionSignature::new(args_count);
        let func = program.begin_function_definition(signature);
        program.generate_random_insts(3);
        let return_var = program.random_variable(Any);
        program.insert_return(return_var);
        program.end_function_definition();
        // println!("build function @@ {}", func.print());

        program.generate_random_insts(1);

        let inputs = program.generate_function_args(func);
        program.function_call(func, inputs);

        Some(())

    }

    pub fn function_call_generator(program: &mut Program) -> Option<()> {

        if DEBUG {
            println!("function_call_generator");
        }

        let func = program.random_variable_of_type(Function, Mode::Strict)?;
        let typ = program.get_type(&func);
        if typ != Function {
            return None;
        }

        // println!("type of the buggy variable = {:#?}", program.get_type(&func));

        let select_probablity = program.prob.probablity(0.9);
        let signature = program.get_signature_for(&func);

        if signature.is_constructing() && select_probablity {
            return None;
        }

        let inputs = program.generate_function_args(func);
        program.function_call(func, inputs);

        Some(())

    }

    pub fn function_return_generator(program: &mut Program) -> Option<()> {

        if DEBUG {
            println!("function_return_generator");
        }


        if program.is_in_function() {
            let r = program.random_variable(Any);
            program.insert_return(r);
            Some(())
        } else {
            None
        }

    }

    pub fn int_array_generator(program: &mut Program) -> Option<()> {

        if DEBUG {
            println!("int_array_generator");
        }


        let size = program.rng.rand_idx(30);
        let var = program.random_variable_of_type(Int, Mode::Strict);

        let variable = if let Some(var) = var {
            var
        } else {
            let v = program.getint();
            program.load_int(v)
        };

        let args = [variable].iter()
                             .cycle()
                             .take(size)
                             .copied()
                             .collect::<Vec<Variable>>();

        program.create_array(args);
        Some(())
    }

    pub fn float_array_generator(program: &mut Program) -> Option<()> {

        if DEBUG {
            println!("float_array_generator");
        }


        let size = program.rng.rand_idx(30);
        let var = program.random_variable_of_type(Float, Mode::Strict);

        let variable = if let Some(var) = var {
            var
        } else {
            let v = program.getfloat();
            program.load_float(v)
        };

        let args = [variable].iter()
                             .cycle()
                             .take(size)
                             .copied()
                             .collect::<Vec<Variable>>();

        program.create_array(args);
        Some(())
    }

    pub fn load_element_generator(program: &mut Program) -> Option<()> {

        if DEBUG {
            println!("load_element_generator");
        }


        let array = program.random_variable_of_type(Array | Unknown | String,
                                                    Mode::Strict);
        let array = array?;
        let idx = if program.prob.probablity(0.7) {
            program.random_variable(Int)
        } else {
            let idx = program.getint();
            program.load_int(idx)
        };

        program.load_element(array, idx);
        Some(())

    }

    pub fn store_element_generator(program: &mut Program) -> Option<()> {

        if DEBUG {
            println!("store_element_generator");
        }


        let array = program.random_variable_of_type(Array, Mode::Strict);
        let array = array?;
        let idx = if program.prob.probablity(0.7) {
            program.random_variable(Int)
        } else {
            let idx = program.getint();
            program.load_int(idx)
        };

        let value = program.random_variable(Any);
        program.store_element(array, idx, value);
        Some(())

    }

    pub fn method_call_generator(program: &mut Program) -> Option<()> {

        if DEBUG {
            println!("method_call_generator");
        }

        let object = program.random_variable_of_type(Object | Unknown,
                                                     Mode::Strict)?;

        let object_type = program.get_type(&object);

        // For now ignore if don't know the concrete shape. TODO: change this
        // later otherwise we will not call any custom defined methods. Its fine
        // now as we don't define custom methods yet
        // if object_type.shape == Shape::Any {return None;}

        // First select a random method from the possible methods
        let method = program.random_method_for_shape(object_type.shape)?;

        // Now generate arguments for the selected method.
        let inputs = program.generate_method_args(&method, Some(object));
        program.method_call(inputs, method);

        Some(())
    }

    pub fn load_property_generator(program: &mut Program) -> Option<()> {

        if DEBUG {
            println!("load_property_generator");
        }


        let object = program.random_variable_of_type(Object, Mode::Strict)?;

        let object_type = program.get_type(&object);

        // For now ignore if don't know the concrete shape. TODO: change this
        // later otherwise we will not load props on custom objs. Its fine
        // now as we don't define custom objects yet
        // if object_type.shape == Shape::Any {return None;}

        let prop = if program.prob.probablity(0.6) {
            program.rng.random_element(&PROPERTIES).to_string()
        } else {
            let prop = program.jsruntime.get_properties(object_type.shape)?;
            program.rng.random_element(&prop).clone()
        };

        program.load_property(prop.to_string(), object);
        Some(())
    }

    pub fn store_property_generator(program: &mut Program) -> Option<()> {

        let object   = program.random_variable_of_type(Object, Mode::Strict)?;
        let property = program.rng.random_element(&PROPERTIES).to_string();
        let value    = program.random_variable(Any);

        program.store_property(property, object, value);

        Some(())
    }

    pub fn load_builtin_generator(program: &mut Program) -> Option<()> {

        if DEBUG {
            println!("load_builtin_generator");
        }

        let constructor = program.jsruntime.get_constructors();
        let constructor = program.rng.random_element(&constructor);



        match constructor {
            ConstructorType::Callable(ms) => {

                let inputs = program.generate_method_args(&ms, None);
                program.load_builtin(constructor, Some(inputs));
            },
            ConstructorType::NonCallable(_,_) => {
                program.load_builtin(constructor, None);
            }
        }
        Some(())
    }

    pub fn create_object_generator(program: &mut Program) -> Option<()> {

        let num_props = program.rng.rand_in_range(0, PROPERTIES.len() as isize);
        let props = program.rng.get_n_random_elements(&PROPERTIES,
                                                          num_props as usize);
        let mut values = Vec::<Variable>::with_capacity(num_props as usize);
        for _ in 0..num_props {
            values.push(program.random_variable(Any));
        }

        let props = props.iter()
                         .map(|v| v.to_string())
                         .collect::<Vec<String>>();

        program.create_object(props, values);


        Some(())
    }

    pub fn delete_property_generator(program: &mut Program) -> Option<()> {

        let object = program.random_variable_of_type(Object, Mode::Strict)?;
        let mut is_indexed_property = false;
        let property = if program.prob.probablity(0.5) {
            is_indexed_property = true;
            program.random_variable(Int)
        } else {
            let prop = program.rng.random_element(&PROPERTIES);
            program.load_string(prop.to_string())
        };

        program.delete_property(object, property, is_indexed_property);
        Some(())
    }


    ///////////////////////
    /////////////////////// misc generators /////////////////////////////////
    ///////////////////////


    pub fn empty_loop_generator(program: &mut Program) -> Option<()> {

        if DEBUG {
            println!("empty_loop_generator");
        }


        if !program.is_in_function() {return None;}

        let (start, end, step) = if program.prob.probablity(0.7) {
            let start = program.load_int(0);
            let end = program.load_int(0x500);
            let step = program.load_int(1);
            (start, end, step)
        } else {
            let start = program.random_variable(Int);
            let end = program.random_variable(Int);
            let step = program.random_variable(Int);
            (start, end, step)
        };


        program.begin_for(start, end, step,
                          "++".to_owned(), Comparators::LessThan);
        program.end_for();

        Some(())

    }

    pub fn jit_function_generator(program: &mut Program) -> Option<()> {

        CodeGenerators::function_definition_generator(program);

        let func = program.random_variable_of_type(Function, Mode::Strict)?;
        let typ = program.get_type(&func);
        if typ != Function {
            // println!("bail due to incorrect variable {:?}", typ);
            return None;
        }

        let signature = program.get_signature_for(&func);

        if signature.is_constructing() {
            // println!("Still Constructing...");
            return None;
        }

        // println!("[INFO] JIT GENERATOR");

        let inputs = program.generate_function_args(func);

        let start = program.load_int(0);
        let end = program.rng.rand_in_range(0, 0x500);
        let end = program.load_int(end);
        let step = program.load_int(1);

        program.begin_for(start, end, step, "++".to_string(),
                          Comparators::LessThan);

        program.generate_random_insts(2);
        program.function_call(func, inputs);
        program.end_for();
        program.generate_random_insts(2);

        let inputs = program.generate_function_args(func);
        program.function_call(func, inputs);

        Some(())
    } 
}






