pub mod ops;
pub mod types;

use crate::compiler::{
    lexer::{keyword::KeyWord, token::Token},
};

use self::types::{TypeDefinition, TypeKind};
use self::ops::AnyOperation;

// Expressions {{

/// An expression is any thing that can be evaluated to a value.
/// For example:
///  - `x + 1`
///  - `some_function()`
#[derive(Debug, Clone)]
pub enum Expression {
    /// An Awaited expression.
    /// For example:
    /// ```ts
    /// await something();
    /// ```
    Await(Box<Expression>),
    /// A regular function call.
    ///
    /// For example:
    /// - `some_function()`
    Call(Call),
    /// A method call.
    ///
    /// For example:
    /// - `x.method()`
    MethodCall(MethodCall),
    /// A new instance of a type.
    /// For example:
    /// - `new SomeType()`
    New(NewCall),
    /// An array literal.
    ///
    /// For example:
    /// - `[1, 2, 3]`
    /// - `[1; 10]`
    Array(Array),
    /// An Object Literal.
    /// For example:
    /// - `{ key: "value" }`
    Object(Object),
    /// An Operation.
    ///
    /// For example:
    /// - `1 + 2`
    /// - `1 - 2`
    Operation(Operation),
    /// A statement
    Statement(Box<Statement>),
    /// A member expression
    ///
    /// For example:
    /// - `x.y`
    /// - `x[y]`
    /// - `x.y.z`
    /// - `x[y].z`
    /// - `x.y[z]`
    Member(MemberListNode),
    /// A literal value.
    ///
    /// For example:
    /// - `1`
    /// - `"hello"`
    /// - `true`
    /// - `false`
    Literal(Literal),
    /// A end of statement,
    ///
    /// For example:
    /// - `;`
    EndOfLine,
}

#[derive(Debug, Clone)]
pub struct Literal {
    pub value: String,
    /// The type of the literal assumed by the compiler
    pub ty: Option<TypeKind>,
}

impl Literal {
    pub fn new(value: String, ty: Option<TypeKind>) -> Self {
        Self { value, ty }
    }
}

#[derive(Debug, Clone)]
pub enum MemberLookup {
    /// A Static member lookup.
    /// For example:
    /// - `SomeType::static_member`
    Static,
    /// A Member lookup.
    /// For example:
    /// - `x.member`
    Dynamic,
    /// An Index lookup.
    /// While this is technically a member lookup, it is not a member of the
    /// object, but rather an index lookup.
    ///
    /// > This can only be used for array indexing.
    /// For example:
    /// - `x[y]`
    /// - `x[y][z]`
    Index,
}

/// A member list is a list of members.
/// For example:
/// - `x.y`
#[derive(Debug, Clone)]
pub struct MemberListNode {
    /// The `name` is the value of the last member or the "property" being accessed. eg: `y` in `x.y`
    pub name: Box<Expression>,
    /// The `origin` is the value that the prop is coming from or the "name" of the initial eg: `x` in `x.y`.
    pub origin: Token,
    /// The `lookup` is the type of access it is, eg whether or not it's a static or dynamic access.
    pub lookup: MemberLookup,
}

impl MemberListNode {
    pub fn new(name: Expression, origin: Token, lookup: MemberLookup) -> MemberListNode {
        MemberListNode {
            name: Box::new(name),
            origin,
            lookup,
        }
    }
}

/// An array literal. This represents an array of values.
/// The values in the array are validated after parsing.
/// For example:
/// - `[1, 2, 3]`
/// - `[1; 10]`
#[derive(Debug, Clone)]
pub struct Array {
    pub values: Vec<Expression>,
    pub ty: Option<TypeKind>,
}

impl Array {
    pub fn new(values: Vec<Expression>, ty: Option<TypeKind>) -> Array {
        Array { values, ty }
    }
}

#[derive(Debug, Clone)]
pub struct Object {
    /// The properties of the object.
    pub properties: Vec<ObjectProperty>,
    /// The type of the object.
    /// This is used to validate the object.
    /// However it can be None if the object is annonymous.
    pub ty: Option<TypeKind>,
}

impl Object {
    pub fn new(properties: Vec<ObjectProperty>, ty: Option<TypeKind>) -> Object {
        Object { properties, ty }
    }

    pub fn empty() -> Object {
        Object {
            properties: Vec::new(),
            ty: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ObjectProperty {
    /// The name of the property.
    pub name: String,
    /// The value of the property.
    pub value: Expression,
}

impl ObjectProperty {
    pub fn new(name: String, value: Expression) -> ObjectProperty {
        ObjectProperty { name, value }
    }
}
#[derive(Debug, Clone)]
pub struct Operation {
    pub left: Box<Expression>,
    pub right: Box<Expression>,
    pub op: AnyOperation,
}

impl Operation {
    pub fn new(left: Expression, op: AnyOperation, right: Expression) -> Operation {
        Operation {
            left: Box::new(left),
            right: Box::new(right),
            op,
        }
    }
}
// }}

// Statements {{
/// A statement is anything that can be executed.
/// For example:
/// - `x = 1`
/// - `return 1`
/// - `if (x > 1) { return 1; }`
/// - `class Foo {}`
/// - `type Foo = int;`
/// - `interface Foo {}`
#[derive(Debug, Clone)]
pub enum Statement {
    /// A var statement.
    Var(Variable),
    /// A const statement.
    Const(Variable),
    /// A static statement.
    Static(Static),
    /// A function declaration.
    Function(Function),
    /// A class declaration.
    Class(Class),
    /// A block statment
    Block(Vec<Expression>),
    /// A import statement.
    /// For example:
    /// - `use foo;`
    /// - `use foo::bar;`
    /// - `use foo::bar::baz;`
    /// - `use foo::bar as baz;`
    /// - `use foo::bar::{ baz, bat };`
    Import(Path),
    /// A namespace statement.
    /// For example:
    /// - `namespace Foo;`
    /// - `namespace Foo { code }`
    Namespace(Namespace),
    /// A type statement / alias.
    /// For example:
    /// - `type Foo = int;`
    /// - `type Foo = Bar;`
    TypeDef(TypeDefinition),
    /// A return statement.
    ///
    /// For example:
    /// - `return 1`
    Return(Return),
    /// A macro invocation.
    /// For example:
    /// - `php!( "hello" )`
    /// - `php! { public function foo() { return "hello"; } }`
    MacroInvocation(CompilerMacro),
}

impl Statement {
    pub fn get_block(&self) -> Option<Vec<Expression>> {
        match self {
            Statement::Block(v) => Some(v.clone()),
            _ => None,
        }
    }

    pub fn get_type_definition(&self) -> Option<TypeDefinition> {
        match self {
            Statement::TypeDef(t) => Some(t.clone()),
            _ => None,
        }
    }

    pub fn get_function(&self) -> Option<Function> {
        match self {
            Statement::Function(f) => Some(f.clone()),
            _ => None,
        }
    }

    pub fn get_class(&self) -> Option<Class> {
        match self {
            Statement::Class(c) => Some(c.clone()),
            _ => None,
        }
    }

    pub fn get_import(&self) -> Option<Path> {
        match self {
            Statement::Import(p) => Some(p.clone()),
            _ => None,
        }
    }

    pub fn get_macro_invocation(&self) -> Option<CompilerMacro> {
        match self {
            Statement::MacroInvocation(m) => Some(m.clone()),
            _ => None,
        }
    }

    pub fn get_mutable(&self) -> Option<Variable> {
        match self {
            Statement::Var(v) => Some(v.clone()),
            _ => None,
        }
    }

    pub fn get_immutable(&self) -> Option<Variable> {
        match self {
            Statement::Const(v) => Some(v.clone()),
            _ => None,
        }
    }

    pub fn get_static(&self) -> Option<Static> {
        match self {
            Statement::Static(s) => Some(s.clone()),
            _ => None,
        }
    }

    pub fn is_block(&self) -> bool {
        match self {
            Statement::Block(_) => true,
            _ => false,
        }
    }

    pub fn is_class(&self) -> bool {
        match self {
            Statement::Class(_) => true,
            _ => false,
        }
    }

    pub fn is_function(&self) -> bool {
        match self {
            Statement::Function(_) => true,
            _ => false,
        }
    }

    pub fn is_import(&self) -> bool {
        match self {
            Statement::Import(_) => true,
            _ => false,
        }
    }

    pub fn is_macro_invocation(&self) -> bool {
        match self {
            Statement::MacroInvocation(_) => true,
            _ => false,
        }
    }

    pub fn is_mutable(&self) -> bool {
        match self {
            Statement::Var(_) => true,
            _ => false,
        }
    }

    pub fn is_immutable(&self) -> bool {
        match self {
            Statement::Const(_) => true,
            _ => false,
        }
    }

    pub fn is_type(&self) -> bool {
        match self {
            Statement::TypeDef(_) => true,
            _ => false,
        }
    }

    pub fn is_static(&self) -> bool {
        match self {
            Statement::Static(_) => true,
            _ => false,
        }
    }
}
// }}

// Visibility {{
#[derive(Debug, Clone)]
pub enum Visibility {
    /// Public visibility. Every module can see this.
    Public,
    /// Private visibility. Only the current **scope** can see this.
    Private,
    /// Protected, only the current **scope** and its children can see this.
    Protected,
    /// Internal visibility. Only the current **module** can see this.
    /// This is the default visibility.
    /// This is not userdefined.
    Module,
}

impl Visibility {
    pub fn from_keyword(keyword: KeyWord) -> Self {
        match keyword {
            KeyWord::Public => Visibility::Public,
            KeyWord::Private => Visibility::Private,
            KeyWord::Protected => Visibility::Protected,
            _ => Visibility::Private,
        }
    }
}
//}}

#[derive(Debug, Clone)]
pub struct Static {
    pub visibility: Visibility,
    pub statement: Box<Statement>,
}

impl Static {
    pub fn new(visibility: Visibility, statement: Statement) -> Static {
        Static {
            visibility,
            statement: Box::new(statement),
        }
    }
}

// Classes {{
#[derive(Debug, Clone)]
pub struct Class {
    pub name: String,
    pub extends: Option<String>,
    pub implements: Option<Vec<String>>,
    pub body: ClassBody,
    pub node_id: u64,
}

impl Class {
    pub fn new() -> Self {
        Class {
            name: String::new(),
            extends: None,
            implements: None,
            body: ClassBody::new(),
            node_id: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ClassProperty {
    pub name: String,
    pub visibility: Visibility,
    pub ty: Option<TypeKind>,
    pub assignment: Option<Expression>,
}

impl ClassProperty {
    pub fn new(
        name: String,
        visibility: Visibility,
        ty: Option<TypeKind>,
        assignment: Option<Expression>,
    ) -> Self {
        ClassProperty {
            name,
            visibility,
            ty,
            assignment,
        }
    }
}

/// Unlike the Statement enum, this contains a special list of statements.
/// destructured and categorized by the parser.
#[derive(Debug, Clone)]
pub struct ClassBody {
    pub properties: Vec<ClassProperty>,
    pub methods: Vec<Function>,
    pub other: Vec<ClassAllowedStatement>,
}

impl ClassBody {
    pub fn new() -> Self {
        ClassBody {
            properties: Vec::new(),
            methods: Vec::new(),
            other: Vec::new(),
        }
    }
}

/// Class bodies ares special because they can contain certain statements,
/// eg circular classes etc.
#[derive(Debug, Clone)]
pub enum ClassAllowedStatement {
    Property(ClassProperty),
    Method(Function),
    Macro(CompilerMacro),
    Import(Path),
    Static(Box<ClassAllowedStatement>),
}

impl ClassAllowedStatement {
    pub fn new_static(s: ClassAllowedStatement) -> Self {
        ClassAllowedStatement::Static(Box::new(s))
    }
}

#[derive(Debug, Clone)]
pub struct Return {
    pub expression: Option<Expression>,
}

impl Return {
    pub fn new(expression: Option<Expression>) -> Self {
        Return { expression }
    }
}
// }}

// Functions {{
/// A function call or method call.
#[derive(Debug, Clone)]
pub struct Function {
    /// The name of the function.
    pub name: Option<String>,
    /// The arguments to the function.
    pub inputs: Vec<FunctionInput>,
    /// The body of the function,
    pub body: Box<Statement>,
    /// The return type of the function.
    pub outputs: Option<TypeKind>,
    /// The visibilty of the function.
    pub visibility: Visibility,
    /// The id for the given function.
    pub node_id: u64,
}

#[derive(Debug, Clone)]
pub struct FunctionInput {
    pub name: String,
    pub ty: Option<TypeKind>,
}

impl FunctionInput {
    pub fn new(name: String, ty: Option<TypeKind>) -> Self {
        FunctionInput { name, ty }
    }
}

/// A function call or method call.
/// This is calling a specific function.
/// For example:
/// - `foo()`
#[derive(Debug, Clone)]
pub struct Call {
    /// The name of the function being called.
    /// This is the name of the function, not the name of the variable.
    pub name: String,
    /// The arugments being passed to the function.
    pub arguments: Vec<Expression>,
}

impl Call {
    pub fn new(name: String, arguments: Vec<Expression>) -> Self {
        Call { name, arguments }
    }
}

/// A `new` call.
/// This is calling a constructor.
/// For example:
/// - `new Foo()`
#[derive(Debug, Clone)]
pub struct NewCall {
    /// The name of the class being constructed.
    pub name: String,
    /// The arugments being passed to the constructor.
    pub arguments: Vec<Expression>,
}

impl NewCall {
    pub fn new(name: String, arguments: Vec<Expression>) -> Self {
        NewCall { name, arguments }
    }
}

/// A method call.
/// For example:
/// - `foo.bar()`
#[derive(Debug, Clone)]
pub struct MethodCall {
    /// The name of the function being called.
    /// This is the name of the function, not the name of the variable.
    pub name: String,
    /// The arugments being passed to the function.
    pub arguments: Vec<Expression>,
    /// The callee of the method call.
    pub callee: Box<Expression>,
}
// }}

// Variables & Types {{
#[derive(Debug, Clone)]
pub struct Variable {
    pub name: String,
    pub node_id: u64,
    pub ty: Option<TypeKind>,
    pub visibility: Visibility,
    pub assignment: Option<Expression>,
}

impl Variable {
    pub fn new(
        name: String,
        ty: Option<TypeKind>,
        visibility: Visibility,
        assignment: Option<Expression>,
    ) -> Self {
        Self {
            name,
            node_id: 0,
            ty,
            visibility,
            assignment,
        }
    }

    pub fn is_uninit(&self) -> bool {
        self.assignment.is_none()
    }
}

#[derive(Debug, Clone)]
pub struct Path {
    /// The module to import.
    /// For example:
    /// - `foo`
    /// - `std` in `std::io` etc.
    pub name: String,
    /// The parts of the import
    /// For example:
    /// - `foo` in `bar::foo`
    /// - `bar, baz` in `foo::{bar, baz}`
    pub parts: Vec<Path>,
}

impl Path {
    pub fn new(name: String) -> Self {
        Self {
            name,
            parts: Vec::new(),
        }
    }

    pub fn from(name: String, parts: Vec<String>) -> Self {
        let mut path = Path {
            name,
            parts: Vec::new(),
        };
        for part in parts {
            path.parts.push(Path {
                name: part,
                parts: Vec::new(),
            });
        }
        path
    }
}

#[derive(Debug, Clone)]
pub struct Namespace {
    /// The path of the namespace.
    /// For example:
    /// - `foo`
    /// - `std\io` etc.
    pub path: Path,
    /// The code of the namespace.
    /// If code does not surround the namespace with `{}`, then it is automatically,
    /// assumed to be within this namespace.
    pub body: Option<Box<Statement>>,
}

impl Namespace {
    pub fn new(path: Path) -> Self {
        Namespace { path, body: None }
    }
}
// }}

// Macros {{
#[derive(Debug, Clone)]
pub struct CompilerMacro {
    /// The name of the macro to invoke,
    /// e.g. `php!`
    /// or `php! { ... }`
    /// Note that these can not be user-defined.
    pub name: String,
    /// The body of the macro, this will be traversed
    /// during macro invokations.
    pub body: String,
}
// }}

// AST {{
#[derive(Debug, Clone)]
pub struct AstBody {
    // todo: Compiler flags
    flags: u64,
    program: Vec<Expression>,
}

impl AstBody {
    pub fn new() -> Self {
        AstBody {
            flags: 0,
            program: Vec::new(),
        }
    }

    pub fn push_statement(&mut self, statement: Statement) {
        self.program
            .push(Expression::Statement(Box::new(statement)));
    }

    pub fn push_expression(&mut self, expression: Expression) {
        self.program.push(expression);
    }

    pub fn get_program(&self) -> &Vec<Expression> {
        &self.program
    }
}
//}}
