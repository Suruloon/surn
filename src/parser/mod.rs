use crate::CompilerOptions;

use self::context::ContextStore;

pub mod context;

/// The parser struct.
/// This contains the context of the AST as well as information
/// regarding errors and warnings with the source code.
pub struct Parser {
    options: CompilerOptions,
    contexts: ContextStore
}

impl Parser {
    
}