#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TypeId(pub u32);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Unknown,
    Unit,
    Bool,
    Number,
    String,
    Symbol,
    Universe,
    Signal,
    Scalar(Box<Type>),
    Series(Box<Type>),
    Maybe(Box<Type>),
    List(Box<Type>),
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TypeArena {
    types: Vec<Type>,
}

impl TypeArena {
    pub fn intern(&mut self, ty: Type) -> TypeId {
        if let Some(index) = self.types.iter().position(|existing| existing == &ty) {
            return TypeId(index as u32);
        }
        let index = self.types.len() as u32;
        self.types.push(ty);
        TypeId(index)
    }

    pub fn get(&self, type_id: TypeId) -> &Type {
        &self.types[type_id.0 as usize]
    }

    pub fn unknown(&mut self) -> TypeId {
        self.intern(Type::Unknown)
    }

    pub fn unit(&mut self) -> TypeId {
        self.intern(Type::Unit)
    }

    pub fn bool(&mut self) -> TypeId {
        self.intern(Type::Bool)
    }

    pub fn number(&mut self) -> TypeId {
        self.intern(Type::Number)
    }

    pub fn string(&mut self) -> TypeId {
        self.intern(Type::String)
    }

    pub fn symbol(&mut self) -> TypeId {
        self.intern(Type::Symbol)
    }

    pub fn universe(&mut self) -> TypeId {
        self.intern(Type::Universe)
    }

    pub fn signal(&mut self) -> TypeId {
        self.intern(Type::Signal)
    }

    pub fn scalar(&mut self, inner: TypeId) -> TypeId {
        let inner = self.get(inner).clone();
        self.intern(Type::Scalar(Box::new(inner)))
    }

    pub fn series(&mut self, inner: TypeId) -> TypeId {
        let inner = self.get(inner).clone();
        self.intern(Type::Series(Box::new(inner)))
    }

    pub fn maybe(&mut self, inner: TypeId) -> TypeId {
        let inner = self.get(inner).clone();
        self.intern(Type::Maybe(Box::new(inner)))
    }

    pub fn list(&mut self, inner: TypeId) -> TypeId {
        let inner = self.get(inner).clone();
        self.intern(Type::List(Box::new(inner)))
    }
}

pub fn parse_type_annotation(input: &str) -> Result<Type, String> {
    let input = input.trim();
    if input.is_empty() {
        return Err("type annotation cannot be empty".into());
    }

    match input {
        "Unknown" => Ok(Type::Unknown),
        "Unit" => Ok(Type::Unit),
        "Bool" | "bool" => Ok(Type::Bool),
        "Number" | "number" => Ok(Type::Number),
        "String" | "string" => Ok(Type::String),
        "Symbol" | "symbol" => Ok(Type::Symbol),
        "Universe" | "universe" => Ok(Type::Universe),
        "Signal" | "signal" => Ok(Type::Signal),
        _ => {
            if let Some(inner) = parse_wrapped_type(input, "Scalar") {
                return parse_type_annotation(inner).map(|ty| Type::Scalar(Box::new(ty)));
            }
            if let Some(inner) = parse_wrapped_type(input, "Series") {
                return parse_type_annotation(inner).map(|ty| Type::Series(Box::new(ty)));
            }
            if let Some(inner) = parse_wrapped_type(input, "Maybe") {
                return parse_type_annotation(inner).map(|ty| Type::Maybe(Box::new(ty)));
            }
            if let Some(inner) = parse_wrapped_type(input, "List") {
                return parse_type_annotation(inner).map(|ty| Type::List(Box::new(ty)));
            }

            Err(format!("unsupported type annotation: {input}"))
        }
    }
}

fn parse_wrapped_type<'a>(input: &'a str, outer: &str) -> Option<&'a str> {
    let prefix = format!("{outer}<");
    input
        .strip_prefix(&prefix)
        .and_then(|rest| rest.strip_suffix('>'))
        .map(str::trim)
}
