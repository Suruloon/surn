use crate::compiler::{ast::AstBody, CompilerOptions};

pub enum ApiVersion {
    V1,
}

pub struct Language {
    /// The name of the language.
    pub name: String,
    /// The description of the language.
    pub description: String,
    /// The version of the language.
    pub version: String,
    /// The api of the language.
    pub api: ApiVersion,
    /// The author of the language.
    pub author: String,
    /// The generator of the language.
    pub generator: Box<dyn Generator>,
}

// A trait that allows transformation of surn to another language.
pub trait Generator {
    /// Generates given ast body to a given language and returns the string.
    /// Useful for scripts.
    fn generate_to_string(&self, ast: AstBody, options: CompilerOptions) -> String;

    /// Generates a script from a path given in CLI.
    /// This CAN be a file or a directory.
    fn generate(&mut self, path: &str, options: CompilerOptions) -> Result<(), String>;
}
