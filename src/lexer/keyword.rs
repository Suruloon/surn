pub const MAX_KEYWORD_LENGTH: usize = 5;

#[derive(Debug, Clone, PartialEq)]
pub enum KeyWord {
    /// Namespace
    Namespace,
    /// Const
    Const,
    /// Var
    Var,
    /// Class
    Class,
    /// Function interface
    Interface,
    /// Type alias
    Type,
    /// `fn` - Function declaration.
    Function,
    /// `if` - Conditional statement.
    If,
    /// `else` - Conditional statement.
    Else,
    /// `pub` - Public visibility.
    Public,
    /// `priv` - Private visibility.
    Private,
    /// `prot` - Protected visibility.
    Protected,
}

impl KeyWord {
    pub fn from_string(v: &String) -> Option<Self> {
        match v.as_str() {
            "namespace" => Some(KeyWord::Namespace),
            "const" => Some(KeyWord::Const),
            "var" => Some(KeyWord::Var),
            "class" => Some(KeyWord::Class),
            "interface" => Some(KeyWord::Interface),
            "type" => Some(KeyWord::Type),
            "fn" => Some(KeyWord::Function),
            "if" => Some(KeyWord::If),
            "else" => Some(KeyWord::Else),
            "pub" => Some(KeyWord::Public),
            "priv" => Some(KeyWord::Private),
            "prot" => Some(KeyWord::Protected),
            _ => None,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            KeyWord::Namespace => "namespace".to_string(),
            KeyWord::Const => "const".to_string(),
            KeyWord::Var => "var".to_string(),
            KeyWord::Class => "class".to_string(),
            KeyWord::Interface => "interface".to_string(),
            KeyWord::Type => "type".to_string(),
            KeyWord::Function => "fn".to_string(),
            KeyWord::If => "if".to_string(),
            KeyWord::Else => "else".to_string(),
            KeyWord::Public => "pub".to_string(),
            KeyWord::Private => "priv".to_string(),
            KeyWord::Protected => "prot".to_string(),
        }
    }

    pub fn is_visibility(&self) -> bool {
        match self {
            KeyWord::Public | KeyWord::Private | KeyWord::Protected => true,
            _ => false,
        }
    }

    pub fn is_declarative(&self) -> bool {
        match self {
            KeyWord::Var | KeyWord::Const | KeyWord::Function | KeyWord::Class | KeyWord::Interface => true,
            _ => false,
        }
    }

    pub fn is_control(&self) -> bool {
        match self {
            KeyWord::If | KeyWord::Else => true,
            _ => false,
        }
    }
}
