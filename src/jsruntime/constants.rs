//! A crate to hold all the constants. Currently only contains the list of Typed
//! Arrays and a list of properties that can be modified by the fuzzer

pub const TYPED_ARRAY_NAMES: [&str; 10] = [
    "Array",
    "Int8Array",
    "Uint8Array",
    "Uint8ClampedArray",
    "Int16Array",
    "Uint16Array",
    "Int32Array",
    "Uint32Array",
    "Float32Array",
    "Float64Array",
    // "BigInt64Array",
    // "BigUint64Array",
];


pub const PROPERTIES: [&str; 8] = [
    "a", "b", "c", "d", "w", "x", "y", "z"
];
