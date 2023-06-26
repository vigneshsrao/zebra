use bitflags::bitflags;
use std::any::Any;
use super::opcodes::Opcodes;
use super::operators::*;
use super::codeanalysis::types::{FunctionSignature, MethodSignature};
use super::codeanalysis::types::ConstructorType;

// These flags represent the specific property of an opcode/Operation. These
// will mostly be used in the analysis phases
bitflags! {
    pub struct Attributes: u8 {
        const NONE           = 0;
        const IS_BLOCK_START = 1 << 0;
        const IS_BLOCK_END   = 1 << 1;
        const IS_LOOP_START  = Attributes::IS_BLOCK_START.bits | 1 << 2;
        const IS_LOOP_END    = Attributes::IS_BLOCK_END.bits   | 1 << 3;
        const IS_PRIMITIVE   = 1 << 4;
        const IS_FUNCTION_START  = Attributes::IS_BLOCK_START.bits | 1 << 5;
        const IS_FUNCTION_END    = Attributes::IS_BLOCK_END.bits   | 1 << 6;
    }
}


/// This trait implements functions that all Operations must satisfy. Over-ride
/// these functions in the respective struct implementations so as to return the
/// properties of that particular Operation.
pub trait Operation {
    fn opcode(&self) -> Opcodes;
    fn attributes(&self) -> Attributes {
        Attributes::NONE
    }
    fn num_inputs(&self) -> u8 {
        0
    }
    fn num_outputs(&self) -> u8 {
        0
    }
    fn num_temp(&self) -> u8 {
        0
    }
    fn is_loop_start(&self) -> bool {
        if (self.attributes() & Attributes::IS_LOOP_START) ==
                Attributes::IS_LOOP_START {
            true
        } else {
            false
        }
    }

    fn is_loop_end(&self) -> bool {
        if (self.attributes() & Attributes::IS_LOOP_END) ==
                Attributes::IS_LOOP_END {
            true
        } else {
            false
        }
    }

    fn is_block_start(&self) -> bool {
        if (self.attributes() & Attributes::IS_BLOCK_START) ==
                Attributes::IS_BLOCK_START {
            true
        } else {
            false
        }
    }

    fn is_block_end(&self) -> bool {
        if (self.attributes() & Attributes::IS_BLOCK_END) ==
                Attributes::IS_BLOCK_END {
            true
        } else {
            false
        }
    }

    fn is_function_start(&self) -> bool {
        if (self.attributes() & Attributes::IS_FUNCTION_START) ==
                Attributes::IS_FUNCTION_START {
            true
        } else {
            false
        }
    }

    fn is_function_end(&self) -> bool {
        if (self.attributes() & Attributes::IS_FUNCTION_END) ==
                Attributes::IS_FUNCTION_END {
            true
        } else {
            false
        }
    }


    fn is_primitive(&self) -> bool {
        if (self.attributes() & Attributes::IS_PRIMITIVE) ==
                Attributes::IS_PRIMITIVE {
            true
        } else {
            false
        }
    }

    // a function to help cast the object back to the concrete type
    // https://stackoverflow.com/a/33687996
    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any {
        panic!("Should not mut access");
    }
}

macro_rules! define_impl {
    ($opcode: ident, $attr: ident,
     $inputs: stmt, $outputs: stmt) => {
        impl Operation for $opcode {
            fn opcode(&self) -> Opcodes {
                Opcodes::$opcode
            }

            fn attributes(&self) -> Attributes {
                Attributes::$attr
            }

            fn num_outputs(&self) -> u8 {
                $outputs
            }

            fn num_inputs(&self) -> u8 {
                $inputs
            }

            fn as_any(&self) -> &dyn Any {
                self
            }
        }
    };
}

macro_rules! define {
    ($opcode: ident, $attr: ident, $type: ty,
     $inputs: literal, $outputs: literal) => {

        #[derive(Debug)]
        pub struct $opcode(pub $type);
        define_impl!($opcode, $attr, $inputs, $outputs);
    };

    ($opcode: ident, $attr: ident,
     $inputs: literal, $outputs: literal) => {

        #[derive(Debug)]
        pub struct $opcode();
        define_impl!($opcode, $attr, $inputs, $outputs);
    };
}

//
//
// From here on is the definition of each of the opcodes
//
//

define!(LoadInt,       IS_PRIMITIVE, isize,           0, 1);
define!(LoadFloat,     IS_PRIMITIVE, f64,             0, 1);
define!(LoadBool,      IS_PRIMITIVE, bool,            0, 1);
define!(LoadString,    IS_PRIMITIVE, String,          0, 1);
define!(BinaryOp,      NONE,         BinaryOperators, 2, 1);
define!(UnaryOp,       NONE,         UnaryOperators,  1, 1);
define!(CompareOp,     NONE,         Comparators,     2, 1);
define!(LoadProperty,  NONE,         String,          1, 1);
define!(StoreProperty, NONE,         String,          2, 0);
define!(Delete,        NONE,         bool,            2, 0);

define!(Nop,                    NONE,            0, 0);
define!(Copy,                   NONE,            2, 0);
define!(BeginIf,                IS_BLOCK_START,  1, 0);
define!(EndIf,                  IS_BLOCK_END,    0, 0);
define!(EndFor,                 IS_LOOP_END,     0, 0);
define!(Break,                  NONE,            0, 0);
define!(Continue,               NONE,            0, 0);
define!(LoadUndefined,          IS_PRIMITIVE,    0, 1);
define!(EndFunctionDefinition,  IS_FUNCTION_END, 0, 0);
define!(Return,                 NONE,            1, 0);
define!(LoadElement,            NONE,            2, 1);
define!(StoreElement,           NONE,            3, 0);

//
// Define opcodes with more complex functionality
//

pub struct BeginElse();
impl Operation for BeginElse {
    fn opcode(&self) -> Opcodes {
        Opcodes::BeginElse
    }

    fn attributes(&self) -> Attributes {
        Attributes::IS_BLOCK_START | Attributes::IS_BLOCK_END
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub struct BeginFor (
    // The operation that is used to step, eg - ++, --, += etc...
    pub String,
    // The comparator that is used to test the end condition. Eg, < , > etc
    pub Comparators,
);
impl Operation for BeginFor {
    fn opcode(&self) -> Opcodes {
        Opcodes::BeginFor
    }

    fn attributes(&self) -> Attributes {
        Attributes::IS_LOOP_START
    }

    fn num_inputs(&self) -> u8 {
        3
    }

    fn num_temp(&self) -> u8 {
        1
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Debug,Clone)]
pub struct BeginFunctionDefinition (pub FunctionSignature);
impl Operation for BeginFunctionDefinition {
    fn opcode(&self) -> Opcodes {
        Opcodes::BeginFunctionDefinition
    }

    fn attributes(&self) -> Attributes {
        Attributes::IS_FUNCTION_START
    }

    fn num_temp(&self) -> u8 {
        self.0.args_count()
    }

    fn num_outputs(&self) -> u8 {
        1
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

pub struct FunctionCall(pub u8);
impl Operation for FunctionCall {

    fn opcode(&self) -> Opcodes {
        Opcodes::FunctionCall
    }

    fn num_inputs(&self) -> u8 {
        self.0 + 1
    }

    fn num_outputs(&self) -> u8 {
        1
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub struct CreateArray(pub u8);
impl Operation for CreateArray {
    fn opcode(&self) -> Opcodes {
        Opcodes::CreateArray
    }

    fn num_inputs(&self) -> u8 {
        self.0
    }

    fn num_outputs(&self) -> u8 {
        1
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// A method call. The method signature is assumed to be a reference. Note that
/// this was built with builtin methods only, might need to clone the method sig
/// in future. Its assumed that the signature will live forever, which is true
/// for js builtin methods
pub struct MethodCall(pub MethodSignature, pub u8);
impl Operation for MethodCall {

    fn opcode(&self) -> Opcodes {
        Opcodes::MethodCall
    }

    fn num_inputs(&self) -> u8 {
        self.1 + 1
    }

    fn num_outputs(&self) -> u8 {
        1
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub struct LoadBuiltin(pub ConstructorType, pub u8);

impl Operation for LoadBuiltin {

    fn opcode(&self) -> Opcodes {
        Opcodes::LoadBuiltin
    }

    fn num_inputs(&self) -> u8 {
        self.1
    }

    fn num_outputs(&self) -> u8 {
        1
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub struct CreateObject(pub Vec<String>);
impl Operation for CreateObject {

    fn opcode(&self) -> Opcodes {
        Opcodes::CreateObject
    }

    fn num_inputs(&self) -> u8 {
        self.0.len() as u8
    }

    fn num_outputs(&self) -> u8 {
        1
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
