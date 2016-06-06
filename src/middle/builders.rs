/// Builders for types in the [middle] layer.
use super::types::*;

#[derive(Clone, Debug)]
pub struct Builder {
    args: Vec<Type>,
    res: Type,
    abi: super::FfiAbi,
}

impl Builder {
    pub fn new() -> Self {
        Builder {
            args: vec![],
            res: Type::void(),
            abi: super::FFI_DEFAULT_ABI,
        }
    }

    pub fn arg(mut self, type_: Type) -> Self {
        self.args.push(type_);
        self
    }

    pub fn args<I: Iterator<Item = Type>>(mut self, types: I) -> Self {
        self.args.extend(types);
        self
    }

    pub fn res(mut self, type_: Type) -> Self {
        self.res = type_;
        self
    }

    pub fn abi(mut self, abi: super::FfiAbi) -> Self {
        self.abi = abi;
        self
    }

    pub fn into_cif(self) -> super::Cif {
        let mut result = super::Cif::new(self.args, self.res);
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


