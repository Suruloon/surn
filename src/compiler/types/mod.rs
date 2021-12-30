use std::collections::HashMap;

use crate::compiler::ast::{Expression, Literal};

/// This is all the different kind of types that may exist.

#[derive(Debug, Clone)]
pub enum TypeKind {
    /// A union type.
    /// This is a type that can be any of the types in the union.
    ///
    /// For example:
    /// ```ts
    /// type Dog = Animal | Mammal
    /// ```
    Union(Box<TypeUnion>),
    /// A type reference.
    /// This is a type that is defined by an alias.
    ///
    /// For Example:
    /// ```ts
    /// type Dog = Animal
    /// ```
    /// Where `Animal` is defined as `type Animal = number`
    Reference(TypeReference),
    /// A runtime type.
    /// This is a type that can be evaluated at runtime, and is not defined by an alias.
    ///
    /// For Example:
    /// ```ts
    /// type AnyNumber(x) = std::isFloat(x) || std::isInt(x);
    /// ```
    ///  > **⚠️This is not implemented yet!**
    RuntimeType(RuntimeType),
    /// A built in type.
    /// This is a type that is defined by the language.
    /// For example:
    /// ```ts
    /// type AnyNumber = string;
    /// ```
    BuiltIn(BuiltInType),
}

impl TypeKind {
    pub fn union(types: Vec<TypeKind>) -> Self {
        TypeKind::Union(Box::new(TypeUnion::new(types)))
    }

    pub fn reference(context: String, params: Option<Vec<TypeParam>>) -> Self {
        TypeKind::Reference(TypeReference::new(context, params))
    }

    pub fn runtime_type(name: String) -> Self {
        TypeKind::RuntimeType(RuntimeType::empty())
    }

    pub fn built_in(name: String) -> Self {
        TypeKind::BuiltIn(BuiltInType::from_string(name).expect("Built in type not found."))
    }
}

/// A single type parameter
/// For example:
/// ```ts
/// caller<T>(x: T)
/// ```
/// Where the paramater is `T`
#[derive(Debug, Clone)]
pub struct TypeParam {
    pub name: Option<String>,
    pub kind: TypeKind,
}

impl TypeParam {
    pub fn new(kind: TypeKind) -> Self {
        TypeParam { name: None, kind }
    }
}

/// A type union.
/// This is a type that can be any of the types in the union.
///
/// For example:
/// ```ts
/// type Dog = Animal | Mammal
/// ```
#[derive(Debug, Clone)]
pub struct TypeUnion {
    pub types: Vec<TypeKind>,
}

impl TypeUnion {
    pub fn empty() -> Self {
        TypeUnion { types: vec![] }
    }

    pub fn new(types: Vec<TypeKind>) -> Self {
        TypeUnion { types }
    }
}

/// A type that is defined by an alias.
///
/// For Example:
/// ```ts
/// type Dog = Animal
/// ```
/// Where `Animal` is defined as `type Animal = number`
#[derive(Debug, Clone)]
pub struct TypeReference {
    pub name: String,
    pub params: Option<Vec<TypeParam>>,
}

impl TypeReference {
    pub fn new(name: String, params: Option<Vec<TypeParam>>) -> Self {
        TypeReference { name, params }
    }
}

/// A runtime type.
/// Similar to a type definition, but is evaluated at runtime.
///
/// For Example:
/// ```ts
/// type AnyNumber(x) = std::isFloat(x) || std::isInt(x);
/// ```
#[derive(Debug, Clone)]
pub struct RuntimeType {
    pub params: Option<Vec<TypeParam>>,
    pub body: Box<Expression>,
}

impl RuntimeType {
    pub fn new(params: Option<Vec<TypeParam>>, body: Expression) -> Self {
        RuntimeType {
            params,
            body: Box::new(body),
        }
    }

    pub fn empty() -> Self {
        RuntimeType {
            params: None,
            body: Box::new(Expression::Literal(Literal::new("None".to_string(), None))),
        }
    }
}

/// A built in type.
/// This is a type that is defined by the language.
///
/// For example:
/// ```ts
/// type AnyNumber = string;
/// ```
#[derive(Debug, Clone)]
pub enum BuiltInType {
    /// A strict type, this is a collection of strict types.
    Strict(StrictBuiltInType),
    /// A single byte (8 bits)
    Byte,
    /// A number within 16 bits or 4 bytes.
    Short,
    /// A number within the range of `i8` to `i64`.
    /// This does not include `f32` or `f64`.
    Int,
    /// A number that is within the range of `i64` to `i128`.
    /// This does not include `f32` or `f64` and should not be used.
    Long,
    /// A number within the range of `f32`.
    Float,
    /// A number within the range of a `f64`.
    Double,
    /// A boolean.
    Bool,
    /// Any string, this is a heap allocated string.
    String,
    /// An array of a type.
    Array(Box<TypeKind>),
    /// Any type, this is disabled in strict mode.
    Any,
}

impl BuiltInType {
    pub fn from_string(s: String) -> Option<Self> {
        match s.as_str() {
            "byte" => Some(BuiltInType::Byte),
            "short" => Some(BuiltInType::Short),
            "int" => Some(BuiltInType::Int),
            "long" => Some(BuiltInType::Long),
            "float" => Some(BuiltInType::Float),
            "double" => Some(BuiltInType::Double),
            "bool" => Some(BuiltInType::Bool),
            "string" => Some(BuiltInType::String),
            "array" => Some(BuiltInType::Array(Box::new(TypeKind::built_in(
                "any".to_string(),
            )))),
            "any" => Some(BuiltInType::Any),
            "u8" => Some(BuiltInType::Strict(StrictBuiltInType::U8)),
            "u16" => Some(BuiltInType::Strict(StrictBuiltInType::U16)),
            "u32" => Some(BuiltInType::Strict(StrictBuiltInType::U32)),
            "u64" => Some(BuiltInType::Strict(StrictBuiltInType::U64)),
            "u128" => Some(BuiltInType::Strict(StrictBuiltInType::U128)),
            "i8" => Some(BuiltInType::Strict(StrictBuiltInType::I8)),
            "i16" => Some(BuiltInType::Strict(StrictBuiltInType::I16)),
            "i32" => Some(BuiltInType::Strict(StrictBuiltInType::I32)),
            "i64" => Some(BuiltInType::Strict(StrictBuiltInType::I64)),
            "i128" => Some(BuiltInType::Strict(StrictBuiltInType::I128)),
            "f32" => Some(BuiltInType::Strict(StrictBuiltInType::F32)),
            "f64" => Some(BuiltInType::Strict(StrictBuiltInType::F64)),
            _ => None,
        }
    }
}

/// A strict built in type.
/// This is a type that is defined when the "strict-types" compiler flag is enabled.
///
/// For example:
/// ```ts
/// type byte = u8;
/// type short = u16;
/// ```
#[derive(Debug, Clone)]
pub enum StrictBuiltInType {
    U8,
    U16,
    U32,
    U64,
    U128,
    I8,
    I16,
    I32,
    I64,
    I128,
    F32,
    F64,
}

/// A literal type statement.
/// This is a type that is defined by a anything.
///
/// For Example:
/// ```ts
/// type Foo<K, V> = Map<K, V>;
/// ```
#[derive(Debug, Clone)]
pub struct TypeDefinition {
    /// The name of the type.
    ///
    /// For example:
    /// `foo` in `type foo = int`
    pub name: String,
    /// The generic parameters of the type.
    ///
    /// For Example:
    /// `T` in `type foo<T> = `
    pub params: Option<Vec<TypeParam>>,
    /// The kind of the type.
    ///
    /// For example:
    /// `int` in `type foo = int`
    pub kind: TypeKind,
}

impl TypeDefinition {
    pub fn new(name: String, params: Option<Vec<TypeParam>>, kind: TypeKind) -> Self {
        TypeDefinition { name, params, kind }
    }
}

/// This is an AST type, it holds information about a type relative to the AST.
/// This will never be used during parsing.
pub struct TypeRef {
    pub context: u64,
    pub node: u64,
}

impl TypeRef {
    pub fn new(context: u64, node: u64) -> Self {
        TypeRef { context, node }
    }
}

/// This is a store that holds all the types for a given context.
/// This is used to resolve types when they are outside of the current scope.
#[derive(Debug, Clone)]
pub struct TypeStore {
    pub types: HashMap<u64, TypeDefinition>,
    next_id: u64,
}

impl TypeStore {
    pub fn new() -> Self {
        Self {
            types: HashMap::new(),
            next_id: 0,
        }
    }

    pub fn add_type(
        &mut self,
        name: String,
        params: Option<Vec<TypeParam>>,
        kind: TypeKind,
    ) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.types.insert(id, TypeDefinition { name, params, kind });
        id
    }

    pub fn get_type(&self, id: u64) -> Option<&TypeDefinition> {
        self.types.get(&id)
    }
}
