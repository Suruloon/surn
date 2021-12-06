use crate::ast::Expression;

#[derive(Debug, Clone)]
pub enum TypeKind {
    BuiltIn(BuiltInType),
    Ref(TypeRef),
    Other(Type)
}

#[derive(Debug, Clone)]
pub enum BuiltInType {
    Int,
    Float,
    Bool,
    String,
    Array(Type),
    Map(Box<Type>, Box<Type>),
}

#[derive(Debug, Clone)]
pub struct TypeRef {
    pub context: u64,
    pub node: u64
}

#[derive(Debug, Clone)]
pub struct Type {
    /// The name of the type.
    pub name: String,
    /// The evaluation of the type.
    /// If this is a function, it will be used to validate declarations that use this type.
    /// It is important to note that if a function is used, it must return a boolean.
    pub expression: Expression,
    /// The id of the type.
    pub id: u64,
}