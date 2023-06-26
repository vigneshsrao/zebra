//! Hold the code for the representation of a particular JS Builtin in a manner
//! that will be understood by the fuzzer

use crate::ir::codeanalysis::types::{MethodSignature, Shape, ConstructorType};

/// The JSBuiltin struct is used to hold the data related to a particular JS
/// Builtin function or object like the shape, constructors etc.
pub struct JSBuiltin {
    /// The shape of this builtin
    pub shape: Shape,

    /// The constructor for this builtin. Constructors can be of 2 types -
    /// 1. callable - which can be called like a function with the new keyword
    /// 2. non callable - don't have to instantiated, used more like an inbuilt
    /// function. Eg - Math, Reflect etc
    pub constructor: Vec<ConstructorType>,

    /// A list of properties that are present by default on an instance of this
    /// builtin
    pub properties: Vec<String>,

    /// The list of methods that can be called on this builtin
    pub methods: Option<Vec<MethodSignature>>,

    /// The list of methods that can be statically called, i.e called directly
    /// on the object instead of an instance, on this builtin
    pub static_methods: Option<Vec<MethodSignature>>,
}

