#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use bitflags::bitflags;
use std::ops::{BitOr, BitOrAssign};

// Shape is used to hold the information about what kind of an object the
// variable is in case the variable has the primitive type `Object`. This is
// analogous to `Shape` (spidermonkey), `Map` (v8) or `Structure` (JSC).
// Primitive types have a shape of `None`. The `Any` shape is used to signify
// that we don't care what type of an object this is, as long as its an object.
bitflags! {
    pub struct Shape: u64 {
        const None          = 0;
        const Static        = 1 << 0;
        const Object        = 1 << 1;
        const Array         = 1 << 2 | Shape::Object.bits;
        const ArrayBuffer   = 1 << 3 | Shape::Object.bits;
        const TypedArray    = 1 << 4 | Shape::Object.bits;
        const Reflect       = 1 << 5 | Shape::Object.bits;
        const Math          = 1 << 6 | Shape::Object.bits;
        const String        = 1 << 7 | Shape::Object.bits;
        const Custom        = 1 << 8 | Shape::Object.bits;
        const Any           = u64::MAX;
    }
}

impl Shape {
    pub fn fetch_clear_static(&mut self) -> bool {
        let ret = self.is_static();
        self.bits &= !Shape::Static.bits;
        ret
    }

    pub fn is_static(&self) -> bool {
        self.bits & Shape::Static.bits != 0
    }

    pub fn is_pure_object(&self) -> bool {
        (*self & !Shape::Object).bits == 0
    }
}

// A bitflag to hold the Primitive types that this typing system supports. This
// is analogous to a JSValue.
bitflags! {
    pub struct PType: u8 {
        const None      = 0;
        const Int       = 1 << 0;
        const Float     = 1 << 1;
        const String    = 1 << 2;
        const Bool      = 1 << 3;
        const Function  = 1 << 4;
        const Undefined = 1 << 5;
        const Unknown   = 1 << 6;
        const Object    = 1 << 7;
        const Any       = u8::MAX;
    }
}

/// A `Type` in this typing system consists of a Primitive type and a Shape. A
/// list of primitive types can be found in the `Ptypes` struct. The Shape is
/// only relevent in the case of an Object. This struct is analogous to the
/// `JSObject` in the popular js engines.
#[derive(Debug,Clone,Copy,PartialEq)]
pub struct Type {
    pub ptype: PType,
    pub shape: Shape,
}

impl Type {
    /// Create a new type given the primitive type and shape
    pub fn new(ptype: PType, shape: Shape) -> Self {
        Self {
            ptype: ptype,
            shape: shape
        }
    }

    /// Create an unknown type
    pub fn default() -> Self {
        Self {
            ptype: PType::Unknown,
            shape: Shape::None,
        }
    }

    /// Create a basic or primitive type. The shape is going to be none and we
    /// will only have a primitive type.
    pub fn basic(ptype: PType) -> Self {
        Self {
            ptype: ptype,
            shape: Shape::None,
        }
    }

    /// Create an object. The primitive type will be an `Object` and we will
    /// only have a shape
    pub fn obj(shape: Shape) -> Self {
        Self {
            ptype: PType::Object,
            shape: shape,
        }
    }

    /// Check if this type and another type, `rhs`, intersect, i.e if they
    /// contain a common type
    pub fn contains(&self, rhs: Type) -> bool {
        let result = (self.ptype & rhs.ptype).bits != 0;

        // If the rhs or lhs can't even be an object then we are searching for a
        // primitive type and we don't care about the shape.
        if !self.is_object() || !rhs.is_object() {
            return result;
        }

        // If the code reaches here then at this point we know that both types
        // can be objects.
        debug_assert!(self.is_object() && rhs.is_object(),
                      "Expected both types to be objects");

        // If the rhs and lhs have something else in common other than Object,
        // then we can just return true here, else we need to check the shapes
        // as well.
        let result = ((self.ptype & rhs.ptype) & !PType::Object).bits != 0;

        if result {return true;}

        // Now we are sure that they are matching on objects. If all the rhs
        // wants is an object and does not care about the shape, then we can
        // just retrun true here as we already know that this is an object.
        if rhs.shape == Shape::Any {
            return true;
        } else {
            // If rhs expects a specific shape and we are exclusively an
            // object, then we need to verify if we conform to what the rhs
            // expects.
            // return rhs.shape == self.shape;
            return (rhs.shape.bits & self.shape.bits) != 0;
        }
    }

    /// Create helpers to check for various types

    pub fn is_int(&self) -> bool {
        self.ptype.bits & PType::Int.bits == PType::Int.bits
    }

    pub fn is_float(&self) -> bool {
        self.ptype.bits & PType::Float.bits == PType::Float.bits
    }

    pub fn is_bool(&self) -> bool {
        self.ptype.bits & PType::Bool.bits == PType::Bool.bits
    }

    pub fn is_string(&self) -> bool {
        self.ptype.bits & PType::String.bits == PType::String.bits
    }

    pub fn is_function(&self) -> bool {
        self.ptype.bits & PType::Function.bits == PType::Function.bits
    }

    pub fn is_undefined(&self) -> bool {
        self.ptype.bits & PType::Undefined.bits == PType::Undefined.bits
    }

    pub fn is_unknown(&self) -> bool {
        self.ptype.bits & PType::Unknown.bits == PType::Unknown.bits
    }

    pub fn is_object(&self) -> bool {
        self.ptype.bits & PType::Object.bits == PType::Object.bits
    }

    pub fn is_numeric(&self) -> bool {
        self.ptype.bits == PType::Int.bits
            || self.ptype.bits == PType::Float.bits
            || self.ptype.bits == PType::Bool.bits
            || self.ptype.bits == PType::Undefined.bits
    }

    pub fn is_integer(&self) -> bool {
        self.is_int() || self.is_bool()
    }
}

impl BitOr for Type {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self {
        Self {
            ptype: self.ptype | rhs.ptype,
            shape: self.shape | rhs.shape,
        }
    }
}

impl BitOrAssign for Type {

    fn bitor_assign(&mut self, rhs: Self) {
        self.ptype |= rhs.ptype;
        self.shape |= rhs.shape
    }
}

///
/// A list of all the primitive types that will be used
///
pub const Int: Type = Type {
    ptype: PType::Int,
    shape: Shape::None
};
pub const Float: Type = Type {
    ptype: PType::Float,
    shape: Shape::None
};
pub const String: Type = Type {
    ptype: PType::String,
    shape: Shape::String
};
pub const Bool: Type = Type {
    ptype: PType::Bool,
    shape: Shape::None
};
pub const Function: Type = Type {
    ptype: PType::Function,
    shape: Shape::None
};
pub const Undefined: Type = Type {
    ptype: PType::Undefined,
    shape: Shape::None
};
pub const Unknown: Type = Type {
    ptype: PType::Unknown,
    shape: Shape::None
};
pub const Object: Type = Type {
    ptype: PType::Object,
    shape: Shape::Any
};
pub const Any: Type = Type {
    ptype: PType::Any,
    shape: Shape::Any
};
pub const Array: Type = Type {
    ptype: PType::Object,
    shape: Shape::Array
};
pub const TypedArray: Type = Type {
    ptype: PType::Object,
    shape: Shape::TypedArray
};

/// A FunctionSignature is used to hold all the data related to a function call.
#[derive(Debug,Clone)]
pub struct FunctionSignature {
    num_inputs:      u8,
    input_types:     Vec<Type>,
    is_constructing: bool,
    output_type:     Type,
    output_shape:    Option<Shape>,
}

impl FunctionSignature {
    pub fn new(num_inputs: u8) -> Self {
        Self {
            num_inputs:      num_inputs,
            input_types:     vec![Type::default(); num_inputs as usize],
            is_constructing: true,
            output_type:     Type::default(),
            output_shape:    None,
        }
    }

    pub fn is_constructing(&self) -> bool {
        self.is_constructing
    }

    pub fn args_count(&self) -> u8 {
        self.num_inputs
    }

    pub fn set_output_type(&mut self, output_type: Type) {
        self.output_type.ptype |= output_type.ptype;
        self.output_type.shape = output_type.shape;
    }

    pub fn get_output_type(&self) -> Type {
        self.output_type
    }

    pub fn set_is_constructing(&mut self) {
        self.is_constructing = true;
    }

    pub fn done_constructing(&mut self) {
        self.is_constructing = false;
    }

    pub fn _set_input_type_at(&mut self, idx: usize, itype: Type) {
        self.input_types[idx] = itype;
    }

    pub fn set_input_types(&mut self, input_types: Vec<Type>) {
        assert_eq!(self.num_inputs as usize,
                   input_types.len(),
                   "Incorrect num of input types for signature");

        self.input_types = input_types;
    }

    pub fn get_input_types(&self) -> &Vec<Type> {
        &self.input_types
    }
}

/// A constructor type represents the type of the constructor being used. There
/// are 2 types -
///
/// * Callable -    These are the normal constructors we would call in JS with
///                 the `new` keyword. They have a specific name and hence work
///                 like methods, with the difference that they don't operate on
///                 a `this`, so we just don't have a `this` object as the first
///                 arg. 
/// * NonCallable - These are not essentially constructors in JS sense. They are
///                 static types on which we can call methods. For eg, `Math`,
///                 `Reflect` etc. They don't have to be "constructed". Hence we
///                 only associate the name and the type on these types of
///                 constructors.
#[derive(Debug, Clone)]
pub enum ConstructorType {
    Callable(MethodSignature),
    NonCallable(String, Type)
}

impl ConstructorType {
    pub fn is_callable(&self) -> bool {
        match self {
            ConstructorType::Callable(_) => true,
            ConstructorType::NonCallable(_,_) => false
        }
    }

    pub fn is_non_callable(&self) -> bool {
        match self {
            ConstructorType::Callable(_) => false,
            ConstructorType::NonCallable(_,_) => true
        }
    }
}

/// Arguments for method calls. Some methods can have optional arguments and
/// some might even take infinite number of arguments. We wrap all that in this
/// enum. Type signifies that this is a plain type. Optional says that this
/// argument is optional and the Repeat struct defines the type and the amount
/// of times the arg repeats
#[derive(Debug,Clone)]
pub enum MethodArg {
    Type(Type),
    Optional(Type),
    Repeat(u8, Type),
}

/// A method signature has a task similar to a function signature but is used on
/// method calls instead. This means that we need some extra info in the
/// metadata like the name of the method, the type of the `this` input it works
/// on etc.
#[derive(Debug,Clone)]
pub struct MethodSignature {
    name:            String,
    this_type:       Type,
    input_types:     Vec<MethodArg>,
    output_type:     Type,
}

impl MethodSignature {

    pub fn new<T> (name: T, this_type: Type, input_types: Vec<MethodArg>,
                   output_type: Type) -> Self
        where T: AsRef<str> {

        Self {
            name:            name.as_ref().to_string(),
            this_type:       this_type,
            input_types:     input_types,
            output_type:     output_type,
        }
    }

    pub fn set_name(&mut self, name: &str) {
        self.name = name.to_string();
    }

    pub fn set_output_type(&mut self, output_type: Type) {
        self.output_type.ptype |= output_type.ptype;
        self.output_type.shape = output_type.shape;
    }

    // rough count of the number of args. The actual count can only be found at
    // runtime as we can have repeating and optional arguments
    pub fn min_args_count(&self) -> usize {
        self.input_types.len()
    }

    pub fn output_type(&self) -> Type {
        self.output_type
    }

    // pub fn _set_input_type_at(&mut self, idx: usize, itype: Type) {
    //     self.input_types[idx] = itype;
    // }

    pub fn input_type_at(&self, idx: usize) -> &MethodArg {
        &self.input_types[idx]
    }

    // pub fn set_input_types(&mut self, input_types: Vec<Type>) {
    //     assert_eq!(self.num_inputs as usize,
    //                input_types.len(),
    //                "Incorrect num of input types for signature");

    //     self.input_types = input_types;
    // }

    pub fn get_input_types(&self) -> &Vec<MethodArg> {
        &self.input_types
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }
}
