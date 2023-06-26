#[derive(Debug, PartialEq)]
pub enum Opcodes {
    Nop,
    LoadInt,
    LoadFloat,
    LoadString,
    LoadUndefined,
    LoadBool,
    Copy,
    BeginIf,
    EndIf,
    BeginElse,
    BeginFor,
    EndFor,
    Break,
    Continue,
    BinaryOp,
    UnaryOp,
    CompareOp,
    BeginFunctionDefinition,
    EndFunctionDefinition,
    Return,
    FunctionCall,
    CreateArray,
    LoadElement,
    StoreElement,
    MethodCall,
    LoadProperty,
    StoreProperty,
    LoadBuiltin,
    CreateObject,
    Delete,
}
