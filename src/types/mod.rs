use std::collections::HashMap;

use crate::ast::Expression;

#[derive(Debug, Clone)]
pub enum TypeKind {
    BuiltIn(BuiltInType),
    Ref(TypeRef),
    Other(TypeRef),
}

#[derive(Debug, Clone)]
pub enum BuiltInType {
    Int,
    Float,
    Bool,
    String,
    Array(Box<TypeKind>),
    Map(Box<TypeKind>, Box<TypeKind>),
}

#[derive(Debug, Clone)]
pub struct TypeRef {
    /// The context in which the type is defined in
    pub context: u64,
    /// The id of the type
    pub id: u64,
}

#[derive(Debug, Clone)]
pub struct Type {
    /// The name of the type.
    pub name: String,
    /// The evaluation of the type.
    /// If this is a function, it will be used to validate declarations that use this type.
    /// It is important to note that if a function is used, it must return a boolean.
    pub kind: TypeKind,
    /// The id of the type.
    pub id: u64,
}

/// Storage for ALL types.
/// This is not included in context ids to avoid confusion
/// and to avoid conflicts with NodeId's and TypeId's.
/// This does act like a `Context` in that it is a list of types.
///
/// It is important to denote that each context contains it's own TypeStore.
#[derive(Debug, Clone)]
pub struct TypeStore {
    pub(crate) types: HashMap<u64, Type>,
    /// The id of the context the TypeStore is associated with.
    /// This can be 0, but is not recommended. As this is used
    /// to determine if a TypeStore is associated with a context.
    ctx_id: u64,
    /// The id of the next type to be added to the TypeStore.
    next_id: u64,
}

impl TypeStore {
    pub fn new(ctx_id: Option<u64>) -> Self {
        Self {
            types: HashMap::new(),
            ctx_id: ctx_id.unwrap_or(0),
            next_id: 0,
        }
    }

    /// Returns a type reference to the type with the name.
    pub fn find_type(&self, name: String) -> Option<&Type> {
        self.types.values().find(|t| t.name == name)
    }

    /// This function will create a TypeRef for a type.
    /// If the type name already exists, it will return the existing TypeRef.
    /// If the type name does not exist, it will create a new TypeRef.
    /// You should use `type_exists` to validate type declarations.
    pub fn make_type(&mut self, ty: Type) -> TypeRef {
        // validate the type name, check if its unique, if it is, we need to make a new one
        // other-wise we can just return the existing one.
        if !self.type_exists(ty.name.clone()) {
            // create the reference
            let type_ref = TypeRef {
                context: self.ctx_id,
                id: self.get_next_id(),
            };

            self.types.insert(type_ref.id, ty);
            return type_ref;
        } else {
            // find the existing type
            let existing_type = self.find_type(ty.name.clone()).unwrap();
            // check if the type is the same
            if existing_type.name == ty.name {
                // return the existing type
                return TypeRef {
                    context: self.ctx_id,
                    id: existing_type.id,
                };
            } else {
                // the type name is not the same, this is an error
                panic!("Type name is not unique");
            }
        }
    }

    /// This is a helper function to check if a type exists.
    /// This method is slower than `find_type` as it will search the entire TypeStore.
    /// However it should be used for validation.
    pub fn type_exists(&self, name: String) -> bool {
        for ty in self.types.values() {
            if ty.name == name {
                return true;
            }
        }
        return false;
    }

    pub fn get_next_id(&mut self) -> u64 {
        self.next_id += 1;
        return self.next_id;
    }
}
