/// Builders for types in the [middle] layer.
use super::types::*;

/// Provides a builder-style API for constructing CIFs and closures.
///
/// Construct a builder using [`Builder::new`](#method.new).
#[derive(Clone, Debug)]
pub struct Builder {
    args: Vec<Type>,
    res: Type,
    abi: super::FfiAbi,
}

impl Builder {
    /// Constructs a `Builder`.
    pub fn new() -> Self {
        Builder {
            args: vec![],
            res: Type::void(),
            abi: super::FFI_DEFAULT_ABI,
        }
    }

    /// Adds a type to the argument type list.
    pub fn arg(&mut self, type_: Type) -> &mut Self {
        self.args.push(type_);
        self
    }

    /// Adds several types to the argument type list.
    pub fn args<I: Iterator<Item = Type>>(&mut self, types: I) -> &mut Self {
        self.args.extend(types);
        self
    }

    /// Sets the result type.
    pub fn res(&mut self, type_: Type) -> &mut Self {
        self.res = type_;
        self
    }

    /// Sets the calling convention.
    pub fn abi(&mut self, abi: super::FfiAbi) -> &mut Self {
        self.abi = abi;
        self
    }

    pub fn into_cif(self) -> super::Cif {
        let mut result = super::Cif::new(self.args.into_iter(), self.res);
        result.set_abi(self.abi);
        result
    }

    pub fn into_closure<'a, U, R>(
        self,
        callback: super::Callback<U, R>,
        userdata: &'a U)
        -> super::Closure<'a>
    {
        super::Closure::new(self.into_cif(), callback, userdata)
    }

    pub fn into_closure_mut<'a, U, R>(
        self,
        callback: super::CallbackMut<U, R>,
        userdata: &'a mut U)
        -> super::Closure<'a>
    {
        super::Closure::new_mut(self.into_cif(), callback, userdata)
    }
}
