pub mod ast;
pub mod error;
pub mod lexer;
pub mod parser;
pub mod types;

pub const CURRENT_VERSION: &'static str = "0.0.1-alpha.rc.1";
pub const NIGHTLY_VERSION: &'static str = "0.0.1-alpha.rc.1";
pub const BETA_VERSION: &'static str = "0.0.1-alpha.rc.1";

pub struct CompilerOptions {
    /// The version of the compiler to compile with, by default,
    /// this is the most recent version.
    pub version: &'static str,
    /// Whether or not to perform a fast check before parsing.
    /// This is a pre-parse check that checks for things like:
    /// - Unclosed strings
    /// - Keyword placement
    /// - Numbers with valid characters
    /// - etc.
    pub semantic_checks: bool,
    /// Whether or not to optimize the code.
    /// This is done after parsing and before code generation.
    pub optimize: bool,
    /// Whether or not to dump the ast to a `surn-ast.bin` file
    /// in the projects current working directory.
    pub dump_ast: bool,
    /// Whether or not to perform after-parse semantic checks.
    /// This is a post-parse check that checks for things like:
    /// - Unused variables
    /// - Unused functions
    /// - Runtime error prevention
    pub post_semantic_checks: bool,
    /// Whether or not to stop compiling after the ast is complete.
    /// This is useful for debugging / testing.
    pub ast_only: bool,
    // / The target php version to compile for.
    // pub target_php_version: &'static str,
    pub detect_bleeding_declarations: bool,
}

impl CompilerOptions {
    pub fn default() -> Self {
        Self {
            version: NIGHTLY_VERSION,
            semantic_checks: true,
            optimize: true,
            dump_ast: false,
            post_semantic_checks: true,
            ast_only: false,
            detect_bleeding_declarations: false,
        }
    }

    pub fn dev() -> Self {
        Self {
            version: CURRENT_VERSION,
            semantic_checks: true,
            optimize: true,
            dump_ast: true,
            post_semantic_checks: false,
            ast_only: false,
            detect_bleeding_declarations: false,
        }
    }
}
