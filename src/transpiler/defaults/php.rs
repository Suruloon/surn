use crate::{
    compiler::{
        ast::{AstBody, Expression, Node, NodeKind, Statement, Variable},
        CompilerOptions,
    },
    transpiler::{
        format::FormatOptions,
        langs::{ApiVersion, Generator, Language},
    },
};

pub fn new() -> Language {
    Language {
        name: "php".to_string(),
        description: "PHP".to_string(),
        version: "8.x.x".to_string(),
        api: ApiVersion::V1,
        author: "Suruloon Studios".to_string(),
        generator: Box::new(PhpGenerator::new(AstBody::new())),
    }
}

pub struct PhpGenerator {
    formatting: FormatOptions,
}

impl PhpGenerator {
    pub fn new(body: AstBody) -> Self {
        PhpGenerator {
            formatting: FormatOptions::psr_4(),
        }
    }

    pub fn process_node(&self, node: Node) -> String {
        let kind = node.inner();

        match kind {
            NodeKind::Expression(expr) => self.process_expression(expr),
            NodeKind::Statement(stmt) => self.process_statement(stmt),
        }
    }

    fn process_expression(&self, expr: Expression) -> String {
        match expr {
            _ => "".to_string(),
        }
    }

    fn process_statement(&self, stmt: Statement) -> String {
        match stmt {
            Statement::Var(var) => self.process_variable(var),
            Statement::Const(var) => self.process_const(var),
            _ => "".to_string(),
        }
    }

    fn process_variable(&self, var: Variable) -> String {
        format!(
            "${} = {};",
            var.name,
            self.process_expression(var.assignment.unwrap())
        )
    }

    fn process_const(&self, var: Variable) -> String {
        format!(
            "static {} = {};",
            var.name,
            self.process_expression(var.assignment.unwrap())
        )
    }
}

impl Generator for PhpGenerator {
    fn generate_to_string(&self, ast: AstBody, options: CompilerOptions) -> String {
        let mut output = String::new();
        for node in ast.get_program() {
            output.push_str(&self.process_node(node.clone()));
        }
        return output;
    }

    fn generate(&mut self, _path: &str, _options: CompilerOptions) -> Result<(), String> {
        unimplemented!()
    }
}
