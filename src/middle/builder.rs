/// Builders for types in the [middle] layer.
use super::types::*;

/// Provides a builder-style API for constructing CIFs and closures.
///
/// To use a builder, first construct it using [`Builder::new`](#method.new).
/// The default calling convention is `FFI_DEFAULT_ABI`, and the default
/// function type is `extern "C" fn()` (or in C, `void(*)()`). Add
/// argument types to the function type with the [`arg`](#method.arg)
/// and [`args`](#method.args) methods. Set the result type with
/// [`res`](#method.res). Change the calling convention, if necessary,
/// with [`abi`](#method.abi).
///
/// Once the builder is configured, construct a `Cif` with
/// [`into_cif`](#method.into_cif) or a closure with
/// [`into_closure`](#method.into_closure) or
/// [`into_closure_mut`](#method.into_closure_mut).
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

    /// Builds a CIF.
    pub fn into_cif(self) -> super::Cif {
        let mut result = super::Cif::new(self.args.into_iter(), self.res);
        result.set_abi(self.abi);
        result
    }

    /// Builds an immutable closure.
    ///
    /// # Arguments
    ///
    /// - `callback` — the function to call when the closure is invoked
    /// - `userdata` — the pointer to pass to `callback` along with the
    ///   arguments when the closure is called
    ///
    /// # Result
    ///
    /// The new closure.
    pub fn into_closure<'a, U, R>(
        self,
        callback: super::Callback<U, R>,
        userdata: &'a U)
        -> super::Closure<'a>
    {
        super::Closure::new(self.into_cif(), callback, userdata)
    }

    /// Builds a mutable closure.
    ///
    /// # Arguments
    ///
    /// - `callback` — the function to call when the closure is invoked
    /// - `userdata` — the pointer to pass to `callback` along with the
    ///   arguments when the closure is called
    ///
    /// # Result
    ///
    /// The new closure.
    pub fn into_closure_mut<'a, U, R>(
        self,
        callback: super::CallbackMut<U, R>,
        userdata: &'a mut U)
        -> super::Closure<'a>
    {
        super::Closure::new_mut(self.into_cif(), callback, userdata)
    }
}
