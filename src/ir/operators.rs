///! This crate holds the list of all binary, unary operators and comparators
///! that the IL is going to use. If a new operator is to be added, then make
///! changes to the appropriate enum in this crate


/// List of the known Binary Operators that we will be using
#[derive(Debug,Clone,Copy)]
pub enum BinaryOperators {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    BitAnd,
    BitOr,
    LogicAnd,
    LogicOr,
    Xor,
    LShift,
    RShift,
}

impl BinaryOperators {
    pub fn rep(&self) -> &str {
        match *self {
            BinaryOperators::Add      =>  "+",
            BinaryOperators::Sub      =>  "-",
            BinaryOperators::Mul      =>  "*",
            BinaryOperators::Div      =>  "/",
            BinaryOperators::Mod      =>  "%",
            BinaryOperators::BitAnd   =>  "&",
            BinaryOperators::BitOr    =>  "|",
            BinaryOperators::LogicAnd =>  "&&",
            BinaryOperators::LogicOr  =>  "||",
            BinaryOperators::Xor      =>  "^",
            BinaryOperators::LShift   =>  "<<",
            BinaryOperators::RShift   =>  ">>",
        }
    }

    pub fn all() -> [BinaryOperators; 12] {
        [
            BinaryOperators::Add,
            BinaryOperators::Sub,
            BinaryOperators::Mul,
            BinaryOperators::Div,
            BinaryOperators::Mod,
            BinaryOperators::BitAnd,
            BinaryOperators::BitOr,
            BinaryOperators::LogicAnd,
            BinaryOperators::LogicOr,
            BinaryOperators::Xor,
            BinaryOperators::LShift,
            BinaryOperators::RShift,
        ]
    }
}


/// List of the known Unary Operators that we will be using
#[derive(Debug,Clone,Copy, PartialEq)]
pub enum UnaryOperators {
    Inc,
    Dec,
    LogicalNot,
    BitwiseNot,
}

impl UnaryOperators {
    pub fn rep(&self) -> &str {
        match *self {
            UnaryOperators::Inc         => "++",
            UnaryOperators::Dec         => "--",
            UnaryOperators::LogicalNot  => "!",
            UnaryOperators::BitwiseNot  => "~",
        }
    }

    pub fn all() -> [UnaryOperators; 4] {
        [
            UnaryOperators::Inc,
            UnaryOperators::Dec,
            UnaryOperators::LogicalNot,
            UnaryOperators::BitwiseNot,
        ]
    }
}


/// List of the known Unary Operators that we will be using
#[derive(Debug,Clone,Copy, PartialEq)]
pub enum Comparators {
    Equal,
    StrictEqual,
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
}

impl Comparators {
    pub fn rep(&self) -> &str {
        match *self {
            Comparators::Equal               => "==",
            Comparators::StrictEqual         => "===",
            Comparators::NotEqual            => "!=",
            Comparators::LessThan            => "<",
            Comparators::LessThanOrEqual     => "<=",
            Comparators::GreaterThan         => ">",
            Comparators::GreaterThanOrEqual  => ">=",
        }
    }

    pub fn all() -> [Comparators; 7] {
        [
            Comparators::Equal,
            Comparators::StrictEqual,
            Comparators::NotEqual,
            Comparators::LessThan,
            Comparators::LessThanOrEqual,
            Comparators::GreaterThan,
            Comparators::GreaterThanOrEqual,
        ]
    }
}

