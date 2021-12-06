use std::{collections::HashMap, fs::OpenOptions, io::Read, path::PathBuf};

use crate::ast::{AstBody, Statement};

/// Contains a map to the actual source for any context.
/// The contents of the source file are not kept in memory, but rather
/// their locations.
/// Each map entry will contain a id for the given session.
pub struct SourceMap {
    internal: HashMap<u64, SourceOrigin>,
    current: u64,
}

impl SourceMap {
    /// Creates a new source map.
    pub fn new() -> Self {
        Self {
            internal: HashMap::new(),
            current: 0,
        }
    }

    pub fn add(&mut self, source: SourceOrigin) -> bool {
        let search = match source.is_virtual() {
            true => source.path.as_ref().unwrap().to_str().unwrap().to_string(),
            false => source.name.as_ref().unwrap().to_string(),
        };

        if self.get_by_path_or_name(search).is_none() {
            self.current += 1;
            self.internal.insert(self.current, source.clone());
            return true;
        } else {
            return false;
        }
    }

    pub fn get(&self, session_id: u64) -> Option<&SourceOrigin> {
        self.internal.get(&session_id)
    }

    fn get_by_path_or_name(&self, search: String) -> Option<&SourceOrigin> {
        for (id, origin) in self.internal.iter() {
            if origin.is_virtual() {
                // check the name
                if origin.name.as_ref().unwrap().clone() == search {
                    return Some(origin);
                }
            } else {
                // check the path
                if origin.path.as_ref().unwrap().to_str().unwrap().to_string() == search {
                    return Some(origin);
                }
            }
        }

        return None;
    }
}

/// A struct used to help identify the origin of a source.
/// This is used to help return errors to the user.
#[derive(Debug, Clone)]
pub struct SourceOrigin {
    /// Optional, the path to the file. (if virtual, add a name to the source.)
    pub path: Option<PathBuf>,
    /// The name of the source (if no path is specified)
    pub name: Option<String>,
    /// The current cached source. (only if virtual)
    contents: Option<String>,
    /// Whether or not the source is virtual.
    is_virtual: bool,
}

impl SourceOrigin {
    /// Creates a new origin with the given path.
    pub fn new(path: PathBuf) -> Self {
        Self {
            path: Some(path),
            name: None,
            contents: None,
            is_virtual: false,
        }
    }

    /// Creates a new origin from a virtualized source (like a script).
    pub fn new_virtual(name: String, contents: String) -> Self {
        Self {
            path: None,
            name: Some(name),
            contents: Some(contents),
            is_virtual: true,
        }
    }

    pub fn get_contents(&self) -> Option<String> {
        if self.is_virtual {
            let string = self.contents.as_ref().unwrap();
            Some(string.clone())
        } else {
            // read the file.
            // the path is assumed to be absolute.
            let file = OpenOptions::new()
                .read(true)
                .open(self.path.as_ref().unwrap());

            if let Ok(mut f) = file {
                let mut string = String::new();
                f.read_to_string(&mut string).unwrap();
                return Some(string);
            }

            return None;
        }
    }

    pub fn is_virtual(&self) -> bool {
        self.is_virtual
    }
}

/// A context store is all contexts for the current run.
/// This **will** include standard libraries and user defined libraries.
/// This is used to resolve symbols.
pub struct ContextStore {
    contexts: HashMap<u64, Context>,
    /// Ids that provide the location of the global variables.
    globals: Vec<u64>,
    id: u64,
}

impl ContextStore {
    pub fn new() -> Self {
        Self {
            contexts: HashMap::new(),
            globals: Vec::new(),
            id: 0,
        }
    }

    pub fn add_context(&mut self, context: &mut Context) {
        self.id += 1;
        context.origin = self.id;
        self.contexts.insert(self.id, context.clone());
    }

    pub fn new_context(&mut self, source: SourceOrigin) -> &Context {
        self.id += 1;
        self.contexts.insert(self.id, Context::new(source, self.id));
        return self.contexts.get(&self.id).unwrap();
    }

    pub fn next_context_id(&self) -> u64 {
        self.id + 1
    }

    pub fn get_globals(&self) -> &Vec<u64> {
        &self.globals
    }
}

#[derive(Debug, Clone)]
pub struct Context {
    pub source: Option<SourceOrigin>,
    pub body: AstBody,
    origin: u64,
    local_id: u64,
}

impl Context {
    pub fn new(source: SourceOrigin, id: u64) -> Self {
        Self {
            source: Some(source),
            body: AstBody::new(),
            origin: id,
            local_id: 0
        }
    }

    pub fn get_next_local_id(&mut self) -> u64 {
        self.local_id += 1;
        return self.local_id;
    }
}
