#![allow(dead_code)]

use super::super::instruction::Instruction;
use super::super::variable::Variable;

/// This will be used to tell the current context of the instruction and if its
/// in a Loop context or not.
pub struct ContextAnalyzer {
    context: Vec<u8>,
}

impl ContextAnalyzer {

    const NONE:             u8 = 0;
    const GLOBAL_CONTEXT:   u8 = 1 << 0;
    const LOOP_CONTEXT:     u8 = 1 << 1;
    const FUNCTION_CONTEXT: u8 = 1 << 2;

    pub fn new() -> Self {
        Self {
            context: vec![ContextAnalyzer::GLOBAL_CONTEXT],
        }
    }

    pub fn analyze(&mut self, inst: &Instruction) {

        // If the current inst is a loop start instruction, then first we need
        // to check if we are already in a Loop context. If we are not then we
        // just have to set the current context as loop. However if we are
        // already in a loop context we push a new LOOP_CONTEXT entry into the
        // context stack.
        if inst.operation.is_loop_start() {
            if self.in_loop() {
                self.context.push(ContextAnalyzer::LOOP_CONTEXT);
            } else if let Some(last) = self.context.last_mut() {
                *last |= ContextAnalyzer::LOOP_CONTEXT;
            } else {
                unreachable!();
            }
        }

        if inst.operation.is_loop_end() {
            // First we verify that we are indeed inside a loop context
            debug_assert!(self.in_loop(), "Loop end without a loop start");

            // Now we delete the loop context info from the current context
            if let Some(last) = self.context.last_mut() {
                *last &= !ContextAnalyzer::LOOP_CONTEXT;
            } else {
                unreachable!();
            }

            // Finally we need to check that there are no more context infos
            // in the current context and pop the current context off the
            // context stack if this is so
            if self.cur_context() == ContextAnalyzer::NONE {
                self.context.pop();
            }
        }

        // When dealing with a function we always want to push a new entry onto
        // the context stack. If the old context was already a func, then we
        // would have pushed anyway. If it was a loop then we don't need to
        // preserve the loop context inside of the function definition
        if inst.operation.is_function_start() {
            self.context.push(ContextAnalyzer::FUNCTION_CONTEXT);
        }

        // Similar to function context creation, when we see a function end, we
        // just pop off the stack.
        if inst.operation.is_function_end() {

            // First we verify that we are indeed inside a function context
            debug_assert!(self.in_function(),
                          "function end without a function start");

            // Now verify that the context is exclusively function context. We
            // should not have a loop or global context or'ed here
            debug_assert!(self.cur_context() == ContextAnalyzer::FUNCTION_CONTEXT,
                          "Context was not exclusively function at func end");

            // Now we delete the loop context info from the current context
            self.context.pop();
        }


        // #[cfg(debug_assertions)]
        // self.debug_print();
        // if cfg!(debug_assertions) {
        // }
    }

    #[cfg(debug_assertions)]
    fn debug_print(&self) {
        let mut s = "Current Context = ".to_string();
        if self.in_global() {
            s.push_str(&"GLOBAL_CONTEXT ");
        }

        if self.in_loop() {
            s.push_str(&"LOOP_CONTEXT ")
        }

        if self.in_function() {
            s.push_str(&"Function_CONTEXT ")
        }

        println!("{}",s);
    }

    /// Helper function to get the current context. This should not be
    /// accessible from outside world. The context stack should never be empty
    /// as we would be in Global context at least. So its safe to unwrap here.
    fn cur_context(&self) -> u8 {
        *self.context.last().unwrap()
    }

    pub fn in_loop(&self) -> bool {
        (self.cur_context() & ContextAnalyzer::LOOP_CONTEXT)
            == ContextAnalyzer::LOOP_CONTEXT
    }

    pub fn in_function(&self) -> bool {
        (self.cur_context() & ContextAnalyzer::FUNCTION_CONTEXT)
            == ContextAnalyzer::FUNCTION_CONTEXT
    }

    pub fn in_global(&self) -> bool {
        (self.cur_context() & ContextAnalyzer::GLOBAL_CONTEXT)
            == ContextAnalyzer::GLOBAL_CONTEXT
    }
}

/// Used to track the scopes of each of the variables that are being used. This
/// information will be later used to find the appropriate variable to use in
/// the code generators as we don't wan't to use a variable that has gone out of
/// scope or is not yet declared.
pub struct ScopeAnalyzer<> {
    scope: Vec<Vec<Variable>>,
}

impl ScopeAnalyzer{
    pub fn new() -> Self {
        let mut scope = Vec::<Vec<Variable>>::new();
        scope.push(Vec::<Variable>::new());
        Self {
            scope: scope,
        }
    }

    pub fn analyze(&mut self, inst: &Instruction) {

        // For all instructions we need to append the variables that they create
        // into the current scope so that they can be tracked.
        let scope = self.scope.last_mut().unwrap();
        scope.extend(inst.outputs());

        if inst.operation.is_block_end() {

            // If the instruction is a block end then we just pop off the
            // topmost scope in the scopes stack.

            debug_assert!(self.scope.len() > 1, "trying to pop global scope");
            self.scope.pop();

        }

        if inst.operation.is_block_start() {

            // If the instruction is a block start then we have to create a new
            // scope and push all the temp variables of this scope, any onto the
            // current scope

            let mut scope = Vec::<Variable>::new();
            scope.extend(inst.temp());
            self.scope.push(scope);

        }


        // #[cfg(debug_assertions)]
        // self.debug_print();

    }

    /// Get access to the variables from the current scope
    pub fn get_inner_variables(&self) -> &Vec<Variable> {
        self.scope.last().unwrap()
    }

    /// Get access to variables from all the outer scopes
    pub fn get_outer_variables(&self) -> &[Vec<Variable>] {
        &self.scope[..self.scope.len()-1]
    }

    /// A list of all variables that can be accessed from the current scope.
    pub fn get_visible_variables(&self) -> Vec<Variable> {
        self.scope.clone().into_iter().flatten().collect::<Vec<Variable>>()
    }

    /// Getter for scopes
    pub fn get_all_scopes(&self) -> &Vec<Vec<Variable>> {
        &self.scope
    }

    #[cfg(debug_assertions)]
    fn debug_print(&self) {
        println!("scopes = {:?}", self.get_visible_variables());
    }
}


#[cfg(test)]
mod test {
    // use crate::instruction::test::createinst;
    // use crate::random::Random;
    // use super::*;

    // #[test]
    // fn test_analyzers() {
    //     let prog = createinst();
    //     let mut sa = ScopeAnalyzer::new();
    //     let mut ca = ContextAnalyzer::new();
    //     for i in prog {
    //         println!("{}", i.print());
    //         sa.analyze(&i);
    //         ca.analyze(&i);
    //         println!("--------------------------------------------------------------------");
    //     }
    //     // let mut r = Random::new(0);
    //     // println!("{}", r.float_in_range(-1000,1000));
    // }
}
