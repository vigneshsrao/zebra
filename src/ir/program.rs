use crate::utils::random::Random;
use crate::utils::probablity::Probablity;
use crate::fuzzer::settings::{GENERATORS, BASIC_GENERATORS};
use crate::jsruntime::jsruntime::JSRuntime;
use crate::jsruntime::constants::{TYPED_ARRAY_NAMES};
use crate::fuzzer::interesting::INTERESTING_INTS;

use super::operation::*;
use super::operators::*;
use super::variable::Variable;
use super::instruction::Instruction;
use super::codeanalysis::typeanalyzer::TypeAnalyzer;
use super::codeanalysis::types::{PType, Type, FunctionSignature, MethodArg};
use super::codeanalysis::types::{self, MethodSignature, Shape, ConstructorType};
use super::codeanalysis::analyzers::{ContextAnalyzer, ScopeAnalyzer};

/// A Mode to help in type of selection that we want to use in the random number
/// selector of the program. See the comment before
/// [random_variable_of_type](Program::random_variable_of_type) function for
/// more details.
#[derive(Debug, PartialEq)]
pub enum Mode {
    Free,
    Strict,
}

/// This is the IR Program that is being generated or mutated.
pub struct Program<'a> {
    /// The program buffer. This is the vector that is going to hold all the
    /// instructions in this program.
    pub buffer:                 Vec<Instruction>,

    /// A count of the total number of instructions in this program
    pub num_instr:              u32,

    /// The context analyzer instance for this program
    pub context_analyzer:       ContextAnalyzer,

    /// The scope analyzer instance for this program
    pub scope_analyzer:         ScopeAnalyzer,

    /// The type tracking instance for this program.
    pub type_analyzer:          TypeAnalyzer,

    /// The id of the next free variable available for use in this program
    pub next_free_variable_id:  u32,

    /// A reference to the jsruntime which will be global to the fuzzer. This is
    /// going to hold all the information related to JavaScript Builtins
    pub jsruntime:              &'a JSRuntime,

    /// A list of all the integers generated over the course of this program
    pub seen_ints:              Vec<isize>,

    /// A list of all the floats generated over the course of this program
    pub seen_floats:            Vec<f64>,

    /// A list of all the strings generated over the course of this program
    pub seen_strings:           Vec<String>,

    /// A random number generator instance for this program
    pub rng:                    Random,

    /// A probablity instance to calcutate the probablity
    pub prob:                   Probablity,
}

impl<'a> Program<'a> {

    /// Build the context for program generation. It expects a reference to the
    /// JSRuntime as an arg. Note that the runtime should live the life of the
    /// program
    pub fn new(jsruntime: &'a JSRuntime) -> Self {
        Self {
            buffer:                 Vec::<Instruction>::new(),
            num_instr:              0,
            context_analyzer:       ContextAnalyzer::new(),
            scope_analyzer:         ScopeAnalyzer::new(),
            type_analyzer:          TypeAnalyzer::new(),
            next_free_variable_id:  0,
            jsruntime:              jsruntime,
            seen_ints:              vec![],
            seen_floats:            vec![],
            seen_strings:           vec![],
            rng:                    Random::new(0),
            prob:                   Probablity::new(Random::new(0)),
        }
    }

    fn next_free_variable(&mut self) -> Variable {
        let id = self.next_free_variable_id;
        self.next_free_variable_id += 1;
        Variable(id)
    }

    /// This function is responsible for creating and adding new instructions to
    /// the program buffer. It creates the output and temp variables as
    /// necessary, analyzes the instruction and returns a reference to the
    /// output variables back to the caller
    fn insert<T: Operation + 'static>(&mut self, ops: T,
                                      inputs: Vec<Variable>) -> &Vec<Variable> {

        let ops = Box::new(ops);

        // First create the output and temp variables for this instructions
        let num_outputs = ops.num_outputs();
        let num_temp = ops.num_temp();
        let mut outputs = Vec::<Variable>::with_capacity(num_outputs.into());
        let mut temp = Vec::<Variable>::with_capacity(num_temp.into());

        for _ in 0..num_outputs {
            outputs.push(self.next_free_variable());
        }

        for _ in 0..num_temp {
            temp.push(self.next_free_variable());
        }

        // Create the instruction itself
        let mut inst = Instruction::new(self.num_instr, ops, inputs, outputs, temp);

        // Analyze the instruction now
        self.scope_analyzer.analyze(&inst);
        self.context_analyzer.analyze(&inst);
        self.type_analyzer.analyze(&mut inst);

        // Finally add it to the program buffer
        self.buffer.push(inst);
        self.num_instr += 1;

        self.buffer.last().unwrap().outputs()

    }

    /// Helper functions for accessing anazyzer data
    pub fn is_in_loop(&self) -> bool {
        self.context_analyzer.in_loop()
    }

    pub fn is_in_function(&self) -> bool {
        self.context_analyzer.in_function()
    }

    /// Generate random values for primitive types

    pub fn getint(&mut self) -> isize {
        let val = if self.prob.probablity(0.3)  {
            *self.rng.random_element(&INTERESTING_INTS)
        } else if self.prob.probablity(0.5) && self.seen_ints.len() >= 4 {
            *self.rng.random_element(&self.seen_ints)
        } else {
            // let tmp = self.rng.rand_in_range(-0x100000000, 0x100000000);
            let tmp = if self.prob.probablity(0.8) {
                self.rng.rand_in_range(0, 0x10000)
            } else {
                self.rng.rand_in_range(-0x1000, 0x1000)
            };
           
            self.seen_ints.push(tmp);
            tmp
        };

        val
    }

    pub fn getfloat(&mut self) -> f64 {
        let val = if self.prob.probablity(0.5) && self.seen_floats.len() >= 4 {
            *self.rng.random_element(&self.seen_floats)
        } else {
            let tmp = self.rng.float_in_range(-0x1000, 0x1000);
            self.seen_floats.push(tmp);
            tmp
        };

        val
    }

    pub fn getstring(&mut self) -> &String {
        let val = if self.prob.probablity(0.5) && !self.seen_strings.is_empty() {
            self.rng.random_element(&self.seen_strings)
        } else {
            let len = self.rng.rand_in_range(0, 100) as u64;
            let tmp = self.rng.random_string(len);
            self.seen_strings.push(tmp);
            self.seen_strings.last().unwrap()
        };

        val
    }

    /// Helper function to fetch a random variable of type `rtype` in the free
    /// mode and unwrap the result to return a variable. Note that it is safe to
    /// unwrap as we know that the selection cannot fail in free mode.
    pub fn random_variable(&mut self, rtype: Type) -> Variable {
        self.random_variable_of_type(rtype, Mode::Free).unwrap()
    }

    /// The core logic of selecting a random variable from this program. There
    /// are 2 modes - Free and Strict.
    ///
    /// 1. Free -   The rtype that is provided is a hint as to what type of
    ///             variable we are interested in. So search for that type (or
    ///             unknown type), however if that type is not found then return
    ///             a variable of any type.
    /// 2. Strict - The variable that is required is strictly of the type
    ///             `rtype`. Hence search for that type and return None if that
    ///             is not found.
    pub fn random_variable_of_type(&mut self,
                                   mut rtype: Type,
                                   mode: Mode) -> Option<Variable> {

        // Get a list of all available scopes and the variables they contain
        let list = self.scope_analyzer.get_all_scopes();

        // Choose a random scope from the scope list, giving preference to the
        // ones that were created more recently.
        let scope = self.prob.choose_biased(&list, 1.2);

        // If the mode is free, then we can search for either unknown or a fixed
        // type
        if mode == Mode::Free {
            rtype.ptype |= PType::Unknown;
        }

        // We define the searching closure. We will basically iterate over all
        // the variables in the current list and filter based on whether the
        // variable contains the required type.
        let filter = |x: &&Variable| -> bool {
            self.get_type(x).contains(rtype)
        };

        // Let the current set of candidates be the variables from the selected
        // scope
        let candidates = scope;

        // Now filter the candidates based on the filter closure that we defined
        // above
        let mut candidates = candidates.iter()
                                       .filter(&filter)
                                       .collect::<Vec<&Variable>>();

        let list = &self.scope_analyzer.get_visible_variables();

        // If we did not find any candidate variables in the current scope that
        // satisfy the required type, then we will set the new candidate list as
        // all the varibles that are visible from this part of the program and
        // also satisfy the filter function.
        if candidates.is_empty() {
            candidates = list.iter()
                             .filter(&filter)
                             .collect();
        }

        // If candidates is still empty then we have failed to find a variable
        // that satisfies the required type, so we just return `None` in case
        // the mode is strict. If mode is free then we set the candidates list
        // as all the variables visible from here. Else we just pick a random
        // variable from the current candidates list.
        if candidates.is_empty() {
            if mode == Mode::Strict {
                return None;
            } else {
                candidates = list.iter().map(|x| x).collect();
            }
        }

        Some(**self.rng.random_element(&candidates))
    }

    /// Get a random method for the shape that is passed in as an arg
    pub fn random_method_for_shape(&mut self, shape: Shape)
                                   -> Option<MethodSignature> {

        let list = self.jsruntime.get_methods(shape)?;
        Some(self.rng.random_element(&list).clone())

    } 

    /// Helper function to call into the type analyzer for fetching types
    pub fn get_type(&self, variable: &Variable) -> Type {
       self.type_analyzer.get_type(variable)
    }

    /// Get the function signature for the variable that is passed in as the
    /// arg. This will crash if the variable passed in does not have a valid
    /// function signature, so its current the job of the caller to verify that
    /// this variable is a valid function.
    pub fn get_signature_for(&self, variable: &Variable) -> &FunctionSignature {
       self.type_analyzer.get_signature_for(*variable)
    }

    /// Generate random instructions by calling random code generators
    pub fn generate_random_insts(&mut self, count: u8) {

        // TODO: Optimize this. It might be too expensive to create a vec for
        // each new instruction that is too be created. It might be better to
        // keep a separate list of visible variables on the analyzer itself.
        if self.scope_analyzer.get_visible_variables().is_empty() {
            for _ in 0..3 {
                let generator = self.prob.choose_biased(&BASIC_GENERATORS, 1.2);
                generator(self);
            }
        }

        let mut cnt = 0;
        loop {
            let generator = self.prob.choose_weighted_baised(&GENERATORS);
            if generator(self).is_some() {
               cnt += 1;
            }

            if cnt == count {
                break;
            }
        }
    }

    /// Generate random arguments for the function signature that is passed in
    /// as the argument. Returns a vector of the generated arguments.
    pub fn generate_function_args(&mut self, function: Variable)
                                  -> Vec<Variable> {

        let signature = self.get_signature_for(&function);
        let mut inputs = Vec::<Variable>
            ::with_capacity(signature.args_count() as usize);

        let input_types = signature.get_input_types().clone();

        for t in input_types {
            let v = self.random_variable(t);
            inputs.push(v);
        }

        inputs

    }

    /// Generate random arguments for a method signature that is passed in as an
    /// argument. The function also accepts an optional `this` argument. This
    /// is passed in case the method is an instance method and not a static one.
    /// Thus we would need to set the `this` value as the first arg in the
    /// args array that is to be returned.
    pub fn generate_method_args(&mut self, method: &MethodSignature,
                                this: Option<Variable>) -> Vec<Variable> {

        let mut inputs = Vec::<Variable>
            ::with_capacity(method.min_args_count() + 1);

        // If this is a method and not a constructor, then the first input is
        // always the object on which this method is to be called.
        if let Some(this) = this {
            inputs.push(this);
        }

        // Generate the arguments for this method
        let len = method.min_args_count();
        for i in 0..len {
            let itype = method.input_type_at(i as usize);
            let var = match itype {
                // If the arg is of required type, then fetch a variable for
                // that type
                MethodArg::Type(itype) => self.random_variable(*itype),

                // If the arg is an optional arg then we generate the argument
                // with a 50% probablity
                MethodArg::Optional(itype) => {
                    if self.prob.probablity(0.5) {
                        self.random_variable(*itype)
                    } else {
                        continue;
                    }
                },

                // If we can repeat the argument any number of times, then we
                // first generate the amount of arguements that we want to
                // provide and then create those args
                MethodArg::Repeat(times, itype) => {
                    let itype = *itype;
                    let cnt = self.rng.rand_idx(*times as usize);

                    // If the count is zero, then just continue as this would
                    // underflow otherwise.
                    if cnt == 0 {
                        continue;
                    }
                    for _ in 0..cnt-1 {
                        let v = self.random_variable(itype);
                        inputs.push(v);
                    }
                    self.random_variable(itype)
                }
            };

            inputs.push(var);

        }
        inputs
    }

    /// Create each of the opcodes in a way that can be used by the code
    /// generators.

    pub fn nop(&mut self) {
        self.insert(Nop(), vec![]);
    }

    pub fn load_int(&mut self, val: isize) -> Variable {
        self.insert(LoadInt(val), vec![])[0]
    }

    pub fn load_float(&mut self, val: f64) -> Variable {
        self.insert(LoadFloat(val), vec![])[0]
    }

    pub fn load_bool(&mut self, val: bool) -> Variable {
        self.insert(LoadBool(val), vec![])[0]
    }

    pub fn load_string(&mut self, val: String) -> Variable {
        self.insert(LoadString(val), vec![])[0]
    }

    pub fn load_undefined(&mut self) -> Variable {
        self.insert(LoadUndefined(), vec![])[0]
    }

    pub fn copy(&mut self, lhs: Variable, rhs: Variable) {
        self.insert(Copy(), vec![lhs, rhs]);
    }

    pub fn begin_if(&mut self, var: Variable) {
        self.insert(BeginIf(), vec![var]);
    }

    pub fn end_if(&mut self) {
        self.insert(EndIf(), vec![]);
    }

    pub fn begin_else(&mut self) {
        self.insert(BeginElse(), vec![]);
    }

    pub fn begin_for(&mut self, start:Variable, end: Variable, step: Variable,
                     op: String, comparator: Comparators) {
        let ops = BeginFor(op, comparator);
        let inputs = vec![start, end, step];
        self.insert(ops, inputs);
    }

    pub fn end_for(&mut self) {
        self.insert(EndFor(), vec![]);
    }

    pub fn insert_break(&mut self) {
        self.insert(Break(), vec![]);
    }

    pub fn insert_continue(&mut self) {
        self.insert(Continue(), vec![]);
    }

    pub fn binary_op(&mut self, lhs: Variable, rhs: Variable,
                     op: BinaryOperators) -> Variable {
        self.insert(BinaryOp(op), vec![lhs, rhs])[0]
    }

    pub fn compare_op(&mut self, lhs: Variable, rhs: Variable,
                     op: Comparators) -> Variable {
        self.insert(CompareOp(op), vec![lhs, rhs])[0]
    }

    pub fn unary_op(&mut self, operand: Variable,
                     op: UnaryOperators) -> Variable {
        self.insert(UnaryOp(op), vec![operand])[0]
    }

    pub fn begin_function_definition(&mut self,
                                     signature: FunctionSignature) -> Variable {
        self.insert(BeginFunctionDefinition(signature), vec![])[0]
    }

    pub fn end_function_definition(&mut self) {
        self.insert(EndFunctionDefinition(), vec![]);
    }

    pub fn insert_return(&mut self, inp: Variable) {
        self.insert(Return(), vec![inp]);
    }

    pub fn function_call(&mut self, func: Variable, args: Vec<Variable>) -> Variable {
        let mut inputs = vec![func];
        let len = args.len() as u8;
        inputs.extend(args);
        self.insert(FunctionCall(len), inputs)[0]
    }

    pub fn create_array(&mut self, inputs: Vec<Variable>) -> Variable {
        self.insert(CreateArray(inputs.len() as u8), inputs)[0]
    }

    pub fn load_element(&mut self, array: Variable, idx: Variable) -> Variable {
        self.insert(LoadElement(), vec![array, idx])[0]
    }

    pub fn store_element(&mut self, array: Variable,
                         idx: Variable, value: Variable) {

        self.insert(StoreElement(), vec![array, idx, value]);
    }

    pub fn method_call(&mut self,
                       args: Vec<Variable>, ms: MethodSignature) -> Variable {
        let len = (args.len() - 1) as u8;
        self.insert(MethodCall(ms, len), args)[0]
    }

    pub fn load_property(&mut self, prop: String, object: Variable) -> Variable {
        self.insert(LoadProperty(prop), vec![object])[0]
    }

    pub fn store_property(&mut self, prop: String, object: Variable,
                          value: Variable) {
        self.insert(StoreProperty(prop), vec![object, value]);
    }

    pub fn create_object(&mut self, prop: Vec<String>, values:
                         Vec<Variable>) -> Variable {

        debug_assert!(prop.len() == values.len(),
                      "Fatal: No. of properties != No. of values");
        self.insert(CreateObject(prop), values)[0]
    }

    pub fn delete_property(&mut self, object: Variable, prop: Variable,
                           is_indexed_prop: bool) {
        self.insert(Delete(is_indexed_prop), vec![object, prop]);
    }

    pub fn load_builtin(&mut self, ctype: &ConstructorType,
                        args: Option<Vec<Variable>>) -> Variable {
        let mut ctype = ctype.clone();

        // If this is a typed array constructor, then we need to solidy which
        // typed array we are going to use. We do this by randomly selecting a
        // typed array type and then replacing the string in the constructor
        // with the typed array name.
        match ctype {
            ConstructorType::Callable(ref mut ms) => {
                if ms.output_type() == types::TypedArray {
                    let typed_array_name =
                            self.rng.random_element(&TYPED_ARRAY_NAMES);
                    ms.set_name(typed_array_name);

                }
            },

            ConstructorType::NonCallable(ref mut name, this_type) => {
                if this_type == Type::obj(Shape::Static | Shape::TypedArray) {
                    let typed_array_name =
                            self.rng.random_element(&TYPED_ARRAY_NAMES);
                    *name = typed_array_name.to_string();
                }
            }

        }

        if args.is_none() {
            debug_assert!(ctype.is_non_callable(), "Constructor Type Mismatch");
            self.insert(LoadBuiltin(ctype, 0), vec![])[0]
        } else {
            debug_assert!(ctype.is_callable(), "Constructor Type Mismatch");
            let args = args.unwrap();
            self.insert(LoadBuiltin(ctype, args.len() as u8), args)[0]
        }
    }

}

#[cfg(test)]
mod test {
    use super::*;
    use crate::lifter::Lifter;

    #[test]
    fn test_prog() {
        let mut p = Program::new();
        p.generate_random_insts(10);
        // for i in p.buffer {
        //     println!("{}", i.print());
        // }
        // p.type_analyzer.debug_print();

        let mut lifter = Lifter::new();
        lifter.do_lifting(p);
        println!("{}", lifter.get_code());
    }
}
