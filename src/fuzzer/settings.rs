use crate::ir::codegenerators::CodeGenerators;
use crate::ir::program::Program;

pub const BASIC_GENERATORS: [fn(&mut Program) -> Option<()>; 5] = [
    CodeGenerators::undefined_literal_generator,
    CodeGenerators::string_literal_generator,
    CodeGenerators::bool_literal_generator,
    CodeGenerators::float_literal_generator,
    CodeGenerators::integer_literal_generator,
];

pub const GENERATORS: [(fn(&mut Program) -> Option<()>, u16); 29] = [
    (CodeGenerators::create_object_generator,       30),
    (CodeGenerators::jit_function_generator,        30),
    (CodeGenerators::load_builtin_generator,        50),
    (CodeGenerators::method_call_generator,         35),
    (CodeGenerators::store_property_generator,      45),
    (CodeGenerators::load_property_generator,       30),
    (CodeGenerators::function_call_generator,       40),
    (CodeGenerators::load_element_generator,        30),
    (CodeGenerators::int_array_generator,           30),
    (CodeGenerators::if_condition_generator,        10),
    (CodeGenerators::binary_op_generator,           30),
    (CodeGenerators::for_loop_generator,            15),
    (CodeGenerators::store_element_generator,       40),
    (CodeGenerators::unary_op_generator,            30),
    (CodeGenerators::compare_op_generator,          30),
    (CodeGenerators::delete_property_generator,     30),
    (CodeGenerators::function_return_generator,     10),
    (CodeGenerators::function_definition_generator, 30),
    (CodeGenerators::float_array_generator,         30),
    (CodeGenerators::empty_loop_generator,          20),
    (CodeGenerators::nop_generator,                 1),
    (CodeGenerators::copy_generator,                1),
    (CodeGenerators::break_generator,               5),
    (CodeGenerators::continue_generator,            5),
    (CodeGenerators::integer_literal_generator,     5),
    (CodeGenerators::float_literal_generator,       1),
    (CodeGenerators::string_literal_generator,      1),
    (CodeGenerators::bool_literal_generator,        1),
    (CodeGenerators::undefined_literal_generator,   1),
];
