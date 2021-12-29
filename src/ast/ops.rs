#[derive(Debug, Clone)]
pub enum AnyOperation {
    BinOp(BinOp),
    UnaryOp(UnaryOp),
    LogicalOp(LogicalOp),
    ComparisonOp(ComparisonOp),
    AssignmentOp(AssignmentOp),
}

impl AnyOperation {
    pub fn from_string(value: String) -> Option<AnyOperation> {
        return match value.as_str() {
            "=" => Some(AnyOperation::AssignmentOp(AssignmentOp::Eq)),
            "+=" => Some(AnyOperation::AssignmentOp(AssignmentOp::Add)),
            "-=" => Some(AnyOperation::AssignmentOp(AssignmentOp::Sub)),
            "*=" => Some(AnyOperation::AssignmentOp(AssignmentOp::Mul)),
            "/=" => Some(AnyOperation::AssignmentOp(AssignmentOp::Div)),
            "%=" => Some(AnyOperation::AssignmentOp(AssignmentOp::Rem)),
            "&=" => Some(AnyOperation::AssignmentOp(AssignmentOp::BitAnd)),
            "|=" => Some(AnyOperation::AssignmentOp(AssignmentOp::BitOr)),
            "^=" => Some(AnyOperation::AssignmentOp(AssignmentOp::BitXor)),
            "<<=" => Some(AnyOperation::AssignmentOp(AssignmentOp::BitSh1)),
            ">>=" => Some(AnyOperation::AssignmentOp(AssignmentOp::BitShr)),
            "==" => Some(AnyOperation::ComparisonOp(ComparisonOp::Eq)),
            "!=" => Some(AnyOperation::ComparisonOp(ComparisonOp::NotEq)),
            "<" => Some(AnyOperation::ComparisonOp(ComparisonOp::LessThan)),
            ">" => Some(AnyOperation::ComparisonOp(ComparisonOp::GreaterThan)),
            "<=" => Some(AnyOperation::ComparisonOp(ComparisonOp::LessThanOrEqual)),
            ">=" => Some(AnyOperation::ComparisonOp(ComparisonOp::GreaterThanOrEqual)),
            "<<" => Some(AnyOperation::BinOp(BinOp::Shl)),
            ">>" => Some(AnyOperation::BinOp(BinOp::Shr)),
            "&" => Some(AnyOperation::BinOp(BinOp::And)),
            "|" => Some(AnyOperation::BinOp(BinOp::Or)),
            "^" => Some(AnyOperation::BinOp(BinOp::Caret)),
            "&&" => Some(AnyOperation::LogicalOp(LogicalOp::And)),
            "||" => Some(AnyOperation::LogicalOp(LogicalOp::Or)),
            "!" => Some(AnyOperation::BinOp(BinOp::Not)),
            "~" => Some(AnyOperation::BinOp(BinOp::Flip)),
            "-" => Some(AnyOperation::BinOp(BinOp::Minus)),
            "+" => Some(AnyOperation::BinOp(BinOp::Plus)),
            "*" => Some(AnyOperation::BinOp(BinOp::Star)),
            "/" => Some(AnyOperation::BinOp(BinOp::Slash)),
            "%" => Some(AnyOperation::BinOp(BinOp::Percent)),
            _ => None,
        };
    }
}

// Binary Operators
#[derive(Clone, PartialEq, Debug)]
pub enum BinOp {
    // +
    Plus,

    // -
    Minus,

    // *
    Star,

    // /
    Slash,

    // %
    Percent,

    // ^
    Caret,

    // !
    Not,

    // Complementary
    Flip,

    // &
    And,

    // |
    Or,

    // <<
    Shl,

    // >>
    Shr,

    // >>>
    UShr,
}

#[derive(Clone, PartialEq, Debug)]
pub enum UnaryOp {
    // ++x
    IncP,

    // x++
    Inc,

    // --x
    DecP,

    // x--
    Dec,

    // -x
    Neg,

    // +x
    Pos,

    // !x
    Not,

    // experimental delete x
    Delete,

    // A syntax sugar for x = {}
    Object,
}

#[derive(Clone, PartialEq, Debug)]
pub enum LogicalOp {
    // x && y
    And,

    // x || y
    Or,

    // x ?? y
    Coalasce,
}

#[derive(Clone, PartialEq, Debug)]
pub enum ComparisonOp {
    Eq,

    NotEq,

    GreaterThan,

    GreaterThanOrEqual,

    LessThan,

    LessThanOrEqual,

    Contains,

    In,

    InstanceOf,
}

#[derive(Clone, PartialEq, Debug)]
pub enum AssignmentOp {
    Eq,

    // x += y
    Add,

    // x -= y
    Sub,

    // x *= y
    Mul,

    // x /= y
    Div,

    // x %= y
    Rem,

    BitAnd,

    BitOr,

    BitXor,

    BitSh1,

    BitShr,

    BitUshr,

    // [EXPERIMENT] x &&= y
    BoolAnd,

    // [EXPERIMENT] x ||= y
    BoolOr,

    // [EXPERIMENT] x ??= y : Support may not be in future versions
    Coalesce,
}
