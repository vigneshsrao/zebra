//! This crate holds all the JS runtime, more specifically the syntax of the
//! builtins like what is the name of a method or contructor, what are the
//! number and type of args etc. One of the default code generators can pick up
//! a method or constructor to use from this information. Note that there will
//! be some syntax that we just can't represent properly and/or generically in
//! this format. For example, for creating a Proxy in JavaScript, we need to
//! pass in an object that implements some of the proxy handlers like `get`,
//! `set` etc. This is not possible to represent here generically. To do this,
//! it is necessary that we implement a dedicated code generator, specifically
//! designed to create the proxy handlers and then call the Proxy constructor.

use super::jsbuiltin::JSBuiltin;
use crate::ir::codeanalysis::types::MethodSignature as MS;
use crate::ir::codeanalysis::types::MethodArg as MA;
use crate::ir::codeanalysis::types::*;

/// A stucture to represent the js runtime so we can keep track of all the
/// builtin objects and functions. This struct will be created only once in the
/// life of the fuzzer and should be shared among all the workers
pub struct JSRuntime {

    /// A list of all the builtins that the runtime holds
    builtins: Vec<JSBuiltin>,

    /// A list of all possible constructors that are present in the runtime.
    /// Essentially this is redundant information as this can be found from the
    /// builtins, but the runtime is only created once for the life of the
    /// fuzzer, so we don't have to worry about the memory overhead here. This
    /// will provide some speedup while fetching the constructors as we would
    /// not have to iterate over all possible builtins each time.
    constructors: Vec<ConstructorType>,
}

impl JSRuntime {
    pub fn new() -> Self {

        let mut runtime = Self {
            builtins: Vec::<JSBuiltin>::new(),
            constructors: Vec::<ConstructorType>::new(),
        };

        runtime.register_array();
        runtime.register_math();
        runtime.register_string();
        runtime.register_object();
        runtime.register_arraybuffer();
        runtime.register_typedarray();

        runtime.init_constructors();

        runtime
    }

    /// Find and fill in all the constructors accessible from this runtime.
    pub fn init_constructors(&mut self) {
        for builtin in &self.builtins {
            for cons in &builtin.constructor {
                self.constructors.push(cons.clone());
            }
        }
    }

    /// Get a list of methods for an object with the shape `shape`
    pub fn get_methods(&self, mut shape: Shape) -> Option<Vec<MS>> {

        // Rip the static type out of the shape.
        let is_static = shape.fetch_clear_static();

        // A dirty hack to ensure that objects like static `Array` are not
        // called with static object functions as all objects shape have the
        // object bit set. We see if the shape provided is static. If it is
        // static we check if the shape has any other bits other than Object
        // set. If this is also true, then get rid of the object bit.
        if !shape.is_pure_object() {
            shape = shape & !Shape::Object;
        }

        // Iterate through all the builtins and collect the ones that contain
        // this shape
        let candidates = self.builtins.iter().filter(|b| {
            let mut cshape = b.shape;
            //// TODO: Implement this `contains` ourselves instead on relying on
            //// the library method.
            if shape.contains(Shape::Object) && !b.shape.is_pure_object() {
                cshape = cshape & !Shape::Object;
            }
            cshape.contains(shape)
        }).collect::<Vec<&JSBuiltin>>();

        // If we don't have method for this shape, then just return None
        if candidates.is_empty() {
            return None;
        }

        let mut ret = Vec::new();

        // Now iterate through all the candidates, collecting the possible
        // methods as we go.
        for candidate in candidates {

            // If the shape that is being passed is a static shape, then we will
            // only return static types else we will go on to instance methods.
            if is_static {
                ret.extend_from_slice(&candidate.static_methods.as_ref()?[..]);
            } else {
                ret.extend_from_slice(&candidate.methods.as_ref()?[..]);

            }
        }

        return Some(ret);

    }

    /// Get access to all the properties that might be present on a instance of
    /// an object with the shape `shape`
    pub fn get_properties(&self, shape: Shape) -> Option<Vec<String>> {

        let mut ret = Vec::new();

        // Iterate through all the builtins and collect the ones that contain
        // this shape
        let candidates = self.builtins.iter().filter(|b| {
            b.shape.contains(shape)
        }).collect::<Vec<&JSBuiltin>>();

        // If we don't have property for this shape, then just return None
        if candidates.is_empty() {
            return None;
        }

        // Iterate over the candidate builtins collecting the properties as we
        // go.
        for candidate in candidates {
            ret.extend_from_slice(&candidate.properties[..]);
        }

        Some(ret)

    }

    /// Get access to a list of constructors that are present on this runtime.
    pub fn get_constructors(&self) -> &Vec<ConstructorType> {
        &self.constructors
    }


    ////
    //// Define JSBuiltins from here
    ////

    pub fn register_object(&mut self) {

        let static_obj =  Type::obj(Shape::Object | Shape::Static);

        let constructor = MS::new("Object", Object, vec![], Object);
        let constructor = vec![
            ConstructorType::Callable(constructor),
            ConstructorType::NonCallable(String::from("Object"), static_obj),
        ];

        let properties = vec![
            String::from("constructor"),
            String::from("__proto__"),
        ];

        let static_methods = vec![
            MS::new("assign", static_obj, vec![MA::Type(Object), MA::Optional(Object)], Object),
            MS::new("create", static_obj, vec![MA::Type(Object)], Object),
            MS::new("defineProperty", static_obj, vec![MA::Type(Object), MA::Type(String), MA::Type(Object)], Object),
            MS::new("freeze", static_obj, vec![MA::Type(Object)], Undefined),
            MS::new("getOwnPropertyDescriptor", static_obj, vec![MA::Type(Object), MA::Type(String)], Object),
            MS::new("getOwnPropertyDescriptors", static_obj, vec![MA::Type(Object)], Object),
            MS::new("getOwnPropertyNames", static_obj, vec![MA::Type(Object)], Array),
            MS::new("getOwnPropertySymbols", static_obj, vec![MA::Type(Object)], Array),
            MS::new("getPrototypeOf", static_obj, vec![MA::Type(Object)], Object),
            MS::new("is", static_obj, vec![MA::Type(Any)], Bool),
            MS::new("isExtensible", static_obj, vec![MA::Type(Any)], Bool),
            MS::new("isFrozen", static_obj, vec![MA::Type(Any)], Bool),
            MS::new("isSealed", static_obj, vec![MA::Type(Any)], Bool),
            MS::new("keys", static_obj, vec![MA::Type(Object)], Array),
            MS::new("preventExtensions", static_obj, vec![MA::Type(Object)], Object),
            MS::new("seal", static_obj, vec![MA::Type(Object)], Object),
            MS::new("setPrototypeOf", static_obj, vec![MA::Type(Object), MA::Type(Object)], Object),
            MS::new("setPrototypeOf", static_obj, vec![MA::Type(Object), MA::Type(Object)], Object),
            MS::new("values", static_obj, vec![MA::Type(Object)], String),
        ];

        self.builtins.push(JSBuiltin {
            shape:          Shape::Object,
            constructor:    constructor,
            properties:     properties,
            methods:        None,
            static_methods: Some(static_methods),
        });
    }
    fn register_array(&mut self) {

        let static_array_type = Type::obj(Shape::Array | Shape::Static);

        let constructor = MS::new(String::from("Array"), Array,
                                  vec![MA::Type(Int)], Array);
        let constructor = vec![
            ConstructorType::Callable(constructor),
            ConstructorType::NonCallable(String::from("Array"),
                                         static_array_type),
        ];

        let properties = vec![String::from("length")];

        let methods = vec![
            MS::new("push",    Array, vec![MA::Type(Any)], Int),
            MS::new("pop",     Array, vec![], Any),
            MS::new("shift",   Array, vec![], Any),
            MS::new("sort",    Array, vec![], Array),
            MS::new("join",    Array, vec![], String),
            MS::new("concat",  Array, vec![MA::Repeat(10, Any)], Array),
            MS::new("unshift", Array, vec![MA::Repeat(10, Any)], Int),
            MS::new("fill",    Array, vec![MA::Type(Int), MA::Repeat(2, Int)], Array),
            MS::new("lastIndexOf", Array, vec![MA::Type(Any)], Any),
            MS::new("includes",    Array, vec![MA::Type(Any)], Bool),
            MS::new("slice",       Array, vec![MA::Type(Int), MA::Optional(Int)], Array),
            MS::new("copyWithin",  Array, vec![MA::Type(Int), MA::Repeat(2, Int)], Array),
            MS::new("splice", Array, vec![MA::Type(Int), MA::Optional(Int), MA::Repeat(10, Any)], Undefined),
        ];

        let static_methods = vec![
            MS::new("from", Array, vec![MA::Type(Array | String)], Array),
            MS::new("from", Array, vec![MA::Type(Any)], Bool),
            MS::new("of", Array, vec![MA::Repeat(100, Any)], Array),
        ];

        self.builtins.push(JSBuiltin {
            shape:          Shape::Array,
            constructor:    constructor,
            properties:     properties,
            methods:        Some(methods),
            static_methods: Some(static_methods),
        });
    }

    fn register_string(&mut self) {
        let static_string = Type::obj(Shape::String | Shape::Static);
        let constructor = MS::new("String", String, vec![], String);
        let constructor = vec![
            ConstructorType::Callable(constructor),
            ConstructorType::NonCallable(String::from("String"), static_string),
        ];

        let properties = vec![
            String::from("length"),
        ];

        let static_methods = vec![
            MS::new("fromCharCode", static_string, vec![MA::Repeat(20, Int)],
                    String),
            MS::new("fromCodePoint", static_string, vec![MA::Repeat(20, Int)],
                    String),

        ];

        let methods = vec![
            MS::new("at", String, vec![MA::Type(Int)], String),
            MS::new("charAt", String, vec![MA::Type(Int)], String),
            MS::new("charCodeAt", String, vec![MA::Type(Int)], Int),
            MS::new("codePointAt", String, vec![MA::Type(Int)], Int),
            MS::new("codePointAt", String, vec![MA::Type(Int)], Int),
            MS::new("concat", String, vec![MA::Repeat(20, String)], String),
            MS::new("includes", String, vec![MA::Type(String), MA::Optional(Int)], Bool),
            MS::new("endsWith", String, vec![MA::Type(String), MA::Optional(Int)], Bool),
            MS::new("startsWith", String, vec![MA::Type(String), MA::Optional(Int)], Bool),
            MS::new("indexOf", String, vec![MA::Type(String), MA::Optional(Int)], Int),
            MS::new("indexOf", String, vec![MA::Type(String), MA::Optional(Int)], Int),
            MS::new("lastIndexOf", String, vec![MA::Type(String), MA::Optional(Int)], Int),
            MS::new("localeCompare", String, vec![MA::Type(String), MA::Optional(String), MA::Optional(Object)], Int),
            MS::new("padEnd", String, vec![MA::Type(Int), MA::Optional(String)], String),
            MS::new("padStart", String, vec![MA::Type(Int), MA::Optional(String)], Int),
            MS::new("repeat", String, vec![MA::Type(Int)], String),
            MS::new("replace", String, vec![MA::Type(String), MA::Type(String)], String),
            MS::new("replaceAll", String, vec![MA::Type(String), MA::Type(String)], String),
            MS::new("slice", String, vec![MA::Type(Int), MA::Optional(Int)], Bool),
            MS::new("split", String, vec![MA::Optional(String), MA::Optional(Int)], Array),
            MS::new("substring", String, vec![MA::Optional(Int), MA::Optional(Int)], String),
            MS::new("toLowerCase", String, vec![], String),
            MS::new("toUpperCase", String, vec![], String),
            MS::new("trim", String, vec![], String),
            MS::new("toString", String, vec![], String),
            MS::new("trimStart", String, vec![], String),
            MS::new("trimEnd", String, vec![], String),
            MS::new("valueOf", String, vec![], String),
        ];

        self.builtins.push(JSBuiltin {
            shape:          Shape::String,
            constructor:    constructor,
            properties:     properties,
            methods:        Some(methods),
            static_methods: Some(static_methods),
        });

    }

    fn register_math(&mut self) {
        let math =  Type::obj(Shape::Math | Shape::Static);
        let numeric = Int | Float;
        let constructor = vec![
            ConstructorType::NonCallable(String::from("Math"), math)
        ];

        let properties = vec![
            String::from("E"),
            String::from("LN2"),
            String::from("LN10"),
            String::from("LOG2E"),
            String::from("LOG10E"),
            String::from("PI"),
            String::from("SQRT_2"),
            String::from("SQRT2"),
        ];

        let methods = vec![
            MS::new("random",math, vec![], Float),
            MS::new("abs",   math, vec![MA::Type(numeric)], Float),
            MS::new("acos",  math, vec![MA::Type(numeric)], Float),
            MS::new("asin",  math, vec![MA::Type(numeric)], Float),
            MS::new("asinh", math, vec![MA::Type(numeric)], Float),
            MS::new("asinh", math, vec![MA::Type(numeric)], Float),
            MS::new("atan",  math, vec![MA::Type(numeric)], Float),
            MS::new("atanh", math, vec![MA::Type(numeric)], Float),
            MS::new("atan2", math, vec![MA::Type(numeric)], Float),
            MS::new("cbrt",  math, vec![MA::Type(numeric)], Float),
            MS::new("ceil",  math, vec![MA::Type(numeric)], Float),
            MS::new("clz32", math, vec![MA::Type(numeric)], Float),
            MS::new("cos",   math, vec![MA::Type(numeric)], Float),
            MS::new("cosh",  math, vec![MA::Type(numeric)], Float),
            MS::new("exp",   math, vec![MA::Type(numeric)], Float),
            MS::new("expm1", math, vec![MA::Type(numeric)], Float),
            MS::new("floor", math, vec![MA::Type(numeric)], Float),
            MS::new("fround",math, vec![MA::Type(numeric)], Float),
            MS::new("log",   math, vec![MA::Type(numeric)], Float),
            MS::new("log1p", math, vec![MA::Type(numeric)], Float),
            MS::new("log10", math, vec![MA::Type(numeric)], Float),
            MS::new("log2",  math, vec![MA::Type(numeric)], Float),
            MS::new("round", math, vec![MA::Type(numeric)], Float),
            MS::new("sign",  math, vec![MA::Type(numeric)], Float),
            MS::new("sin",   math, vec![MA::Type(numeric)], Float),
            MS::new("sinh",  math, vec![MA::Type(numeric)], Float),
            MS::new("sqrt",  math, vec![MA::Type(numeric)], Float),
            MS::new("tan",   math, vec![MA::Type(numeric)], Float),
            MS::new("tanh",  math, vec![MA::Type(numeric)], Float),
            MS::new("trunc", math, vec![MA::Type(numeric)], Float),
            MS::new("pow",   math, vec![MA::Type(numeric), MA::Type(numeric)], Float),
            MS::new("imul",  math, vec![MA::Type(numeric), MA::Type(numeric)], Float),
            MS::new("max",   math, vec![MA::Type(numeric), MA::Repeat(4, numeric)], Float),
            MS::new("min",   math, vec![MA::Type(numeric), MA::Repeat(4, numeric)], Float),
            MS::new("hypot", math, vec![MA::Type(numeric), MA::Repeat(4, numeric)], Float),
        ];

        self.builtins.push(JSBuiltin {
            shape:          Shape::Math,
            constructor:    constructor,
            properties:     properties,
            methods:        None,
            static_methods: Some(methods)
        });

    }

    fn register_arraybuffer(&mut self) {

        let arraybuf = Type::obj(Shape::ArrayBuffer);
        let arraybuf_static = Type::obj(Shape::ArrayBuffer | Shape::Static);
        let constructor = MS::new("ArrayBuffer", arraybuf, vec![MA::Type(Int)], arraybuf);
        let constructor = vec![
            ConstructorType::Callable(constructor),
            ConstructorType::NonCallable(String::from("ArrayBuffer"), arraybuf_static)
        ];

        let properties = vec![String::from("byteLength")];

        let static_methods = vec![
            MS::new("isView", arraybuf_static, vec![MA::Type(Any)], Bool),
        ];

        let methods = vec![
            MS::new("slice", arraybuf, vec![MA::Type(Int), MA::Optional(Int)], arraybuf),
        ];

        self.builtins.push(JSBuiltin {
            shape: Shape::ArrayBuffer,
            constructor:        constructor,
            properties:         properties,
            methods:            Some(methods),
            static_methods:     Some(static_methods)

        });

    }

    fn register_typedarray(&mut self) {

        let typed_array = Type::obj(Shape::TypedArray);
        let array_buffer = Type::obj(Shape::ArrayBuffer);
        let typed_array_static = Type::obj(Shape::TypedArray | Shape::Static);

        let constructor1 = MS::new("TypedArray", typed_array,
                                 vec![MA::Optional(Int | typed_array | Object)],
                                 typed_array);

        let constructor2 = MS::new("TypedArray", typed_array,
                                   vec![MA::Type(array_buffer),
                                        MA::Optional(Int), MA::Optional(Int)],
                                   typed_array);

        let constructor = vec![
            ConstructorType::Callable(constructor1),
            ConstructorType::Callable(constructor2),
            ConstructorType::NonCallable(String::from("TypedArray"), typed_array_static),
        ];

        let properties = vec![
            String::from("buffer"),
            String::from("byteLength"),
            String::from("byteOffset"),
            String::from("length")
        ];

        let static_methods = vec![
            MS::new("from", typed_array_static, vec![MA::Type(Array)], typed_array),
            MS::new("of", typed_array_static, vec![MA::Repeat(10, Int)], typed_array)
        ];

        let methods = vec![
            MS::new("at", typed_array, vec![MA::Type(Int)], Int | Float),
            MS::new("copyWithin", typed_array, vec![MA::Type(Int), MA::Optional(Int), MA::Optional(Int)], typed_array),
            MS::new("copyWithin", typed_array, vec![MA::Type(Int), MA::Optional(Int), MA::Optional(Int)], typed_array),
            MS::new("fill", typed_array, vec![MA::Type(Int | Float), MA::Optional(Int), MA::Optional(Int)], typed_array),
            MS::new("includes", typed_array, vec![MA::Type(Int | Float), MA::Optional(Int)], Bool),
            MS::new("indexOf", typed_array, vec![MA::Type(Int | Float), MA::Optional(Int)], Int),
            MS::new("join", typed_array, vec![], String),
            MS::new("lastIndexOf", typed_array, vec![MA::Type(Int | Float), MA::Optional(Int)], Int),
            MS::new("reverse", typed_array, vec![], Int),
            MS::new("set", typed_array, vec![MA::Type(Array | typed_array), MA::Optional(Int)], Undefined),
            MS::new("slice", typed_array, vec![MA::Optional(Int), MA::Optional(Int)], Undefined),
            MS::new("sort", typed_array, vec![], typed_array),
            MS::new("subarray", typed_array, vec![MA::Optional(Int), MA::Optional(Int)], typed_array),
            MS::new("toLocaleString", typed_array, vec![], String),
        ];

        self.builtins.push(JSBuiltin {
            shape:          Shape::TypedArray,
            constructor:    constructor,
            properties:     properties,
            methods:        Some(methods),
            static_methods: Some(static_methods),
        })

    }

}
