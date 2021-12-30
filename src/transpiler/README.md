# Transpiler

This is the home of the public API for surn.

### How do I support a new language?

You can add a new language by using our C/rust api.
Here's an example of how to add nano php:

```rust
use surn::transpiler::lang::{ Language, Version, map };
use surn::engines::type_engine::*;
use surn::transpiler::ast::extended::*;
use surn::compiler::ast::*;

pub fn register_language() -> Language {
    let php = Language::new("php", Version::from("8.x.x"));

    // Because there is no extension for php to enable this,
    // we need to disable it.
    php.allow_threading(false);

    // php is annoying, so we have to do this
    php.set_traversing_engine(traversing_engine::Traverser::new());

    php.set_configuration(configure_traverser);
    return php;
}


pub fn configure_traverser(traverser: &mut Traverser) {
    // to update how surn will transpiler this code, we need to update the to_source of all nodes
    // in the ast.
    traverser.update_node_translation(Node::ObjectStatement, |node| {
        // the node is an object
        // we can translate based on the custom settings we provide
        if traverser.custom_opts().get("objects.parse-as-class").is_true() {
            // Parse this object as a class.
            // create a new call to the class constructor
            let mut class = Class::new();
            class.name = node.get_name().unwrap_or("".to_string());
            class.extends = "\\surn\\std\\php\\Object".into();

            let props: Vec<ClassProperty> = Vec::new();

            for prop in node.properties.iter() {
                let type_of = prop.value().get_named_type();
                // create a new class property
                let p = ClassProperty::new(prop.name.clone(), Visibility::Public, type_of, prop.value.to_target());
                props.push(p);
            }

            class.body = ClassBody::new();
            class.body.properties = props;
            return class.to_target();
        }
    });
}
```