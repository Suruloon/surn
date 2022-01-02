use self::langs::Language;
use std::collections::HashMap;

mod defaults;
pub mod format;
pub mod langs;

pub struct Transpiler {
    registered: HashMap<&'static str, Language>,
}

impl Transpiler {
    pub fn new() -> Self {
        Transpiler {
            registered: HashMap::new(),
        }
    }

    pub fn register_defaults(&mut self) {
        self.registered.insert("php", defaults::php::new());
    }

    pub fn get(&self, lang: &str) -> Option<&Language> {
        self.registered.get(lang)
    }
}
