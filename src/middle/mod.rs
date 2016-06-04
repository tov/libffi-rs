//! The main idea is to wrap types `ffi_cif` and `ffi_closure` as `Cif` and
//! `Closure`, respectively, so that the resources are managed properly.
//! Calling a function via a CIF or closure is still unsafe.
use std::mem;
use std::os::raw::c_void;
use std::marker::PhantomData;

use low;

pub mod types;
use self::types::*;

/// A CIF (“Call InterFace”) describing the calling convention and types
/// for calling a function.
#[derive(Clone, Debug)]
pub struct Cif {
    cif:    low::ffi_cif,
    args:   TypeArray,
    result: Type,
}

/// When calling a function via a CIF, each argument must be passed
/// as a C `void*`. Wrapping the argument in the `Arg` struct
/// accomplishes the necessary coercion.
#[derive(Debug)]
pub struct Arg(*mut c_void);

impl Arg {
    /// Coerces an argument reference into the `Arg` types.
    pub fn new<T>(r: &T) -> Self {
        Arg(unsafe { mem::transmute(r as *const T) })
    }
}

/// Coerces an argument reference into the `Arg` types. (This is the same
/// as [`Arg::new`](struct.Arg.html#method.new)).
pub fn arg<T>(r: &T) -> Arg {
    Arg::new(r)
}

impl Cif {
    /// Creates a new CIF for the given argument and result types,
    /// using the default calling convention.
    pub fn new(args: Vec<Type>, result: Type) -> Self {
        Self::from_type_array(TypeArray::new(args), result)
    }

    /// Creates a new CIF for the given argument and result types,
    /// using the default calling convention.
    pub fn from_type_array(args: TypeArray, result: Type) -> Self {
        let mut cif: low::ffi_cif = Default::default();

        unsafe {
            low::prep_cif(&mut cif,
                          low::FFI_DEFAULT_ABI,
                          args.len(),
                          result.as_raw_ptr(),
                          args.as_raw_ptr())
        }.expect("low::prep_cif");

        // Note that cif retains references to args and result,
        // which is why we hold onto them here.
        Cif {
            cif:    cif,
            args:   args,
            result: result,
        }
    }

    /// Calls function `f` passing it the given arguments. Note that
    /// the funtion pointer is passed as a `usize`, which tends to be
    /// more convenient (and the types aren’t checked anyway).
    pub unsafe fn call<R>(&self, f: usize, values: &[Arg]) -> R {
        use std::mem;

        assert!(self.cif.nargs as usize == values.len());

        low::call::<R>(mem::transmute(&self.cif),
                       mem::transmute(f),
                       mem::transmute(values.as_ptr()))
    }

    /// Gets a raw pointer to the underlying
    /// [`ffi_cif`](../low/struct.ffi_cif.html). This can be used
    /// for passing the CIF to functions from the [`low`](../low/index.html)
    /// and [`raw`](../raw/index.html) modules.
    pub fn as_raw_ptr(&self) -> *mut low::ffi_cif {
        unsafe { mem::transmute(&self.cif) }
    }
}

/// Represents a closure, which captures a `void*` (user data) and
/// passes it to a callback when the code pointer (obtained via
/// [`code_ptr`](struct.Closure.html#method.code_ptr) is invoked.
pub struct Closure<'a> {
    _cif:    Box<Cif>,
    alloc:   *mut ::low::ffi_closure,
    code:    unsafe extern "C" fn(),
    _marker: PhantomData<&'a ()>,
}

impl<'a> Drop for Closure<'a> {
    fn drop(&mut self) {
        unsafe {
            low::closure_free(self.alloc);
        }
    }
}

pub use low::Callback;

impl<'a> Closure<'a> {
    /// Creates a new closure. The CIF describes the calling convention
    /// for the resulting C function. When called, the C function will
    /// call `callback`, passing along its arguments and the captured
    /// `userdata`.
    pub fn new<U, R>(cif:      Cif,
                     callback: Callback<U, R>,
                     userdata: &'a U) -> Self
    {
        let cif = Box::new(cif);
        let (alloc, code) = low::closure_alloc();

        unsafe {
            low::prep_closure_loc(alloc,
                                  cif.as_raw_ptr(),
                                  callback,
                                  // TODO: distinguish ClosureMut
                                  userdata as *const U as *mut U,
                                  code).unwrap();
        }

        Closure {
            _cif:    cif,
            alloc:   alloc,
            code:    code,
            _marker: PhantomData,
        }
    }

    /// Obtains the callable code pointer for a closure. The result
    /// needs to be transmuted to the correct type before it can be called.
    pub fn code_ptr(&self) -> &unsafe extern "C" fn() {
        &self.code
    }
}

#[cfg(test)]
mod test {
    use low;
    use super::*;
    use super::types::*;
    use std::mem;
    use std::os::raw::c_void;

    #[test]
    fn call() {
        let args = vec![Type::sint64(), Type::sint64()];
        let cif  = Cif::new(args, Type::sint64());
        let f    = |m: i64, n: i64| -> i64 {
            unsafe { cif.call(add_it as usize, &[arg(&m), arg(&n)]) }
        };

        assert_eq!(12, f(5, 7));
        assert_eq!(13, f(6, 7));
        assert_eq!(15, f(8, 7));
    }

    extern "C" fn add_it(n: i64, m: i64) -> i64 {
        return n + m;
    }

    #[test]
    fn closure() {
        let cif  = Cif::new(vec![Type::uint64()], Type::uint64());
        let env: u64 = 5;
        let closure = Closure::new(cif, callback, &env);

        unsafe {
            let fun: &unsafe extern "C" fn(u64) -> u64
                = mem::transmute(closure.code_ptr());

            assert_eq!(11, fun(6));
            assert_eq!(12, fun(7));
        }
    }

    unsafe extern "C" fn callback(_cif: &low::ffi_cif,
                                  result: &mut u64,
                                  args: *const *const c_void,
                                  userdata: &mut u64)
    {
        let args: *const &u64 = mem::transmute(args);
        *result = **args + *userdata;
    }

    #[test]
    fn rust_lambda() {
        let cif = Cif::new(vec![Type::uint64(), Type::uint64()],
                           Type::uint64());
        let env = |x: u64, y: u64| x + y;
        let closure = Closure::new(cif, callback2, &env);

        unsafe {
            let fun: &unsafe extern "C" fn (u64, u64) -> u64
                = mem::transmute(closure.code_ptr());

            assert_eq!(11, fun(5, 6));
        }
    }

    unsafe extern "C" fn callback2<F: Fn(u64, u64) -> u64>
        (_cif: &low::ffi_cif,
         result: &mut u64,
         args: *const *const c_void,
         userdata: &mut F)
    {
        let args: *const &u64 = mem::transmute(args);
        let arg1 = **args.offset(0);
        let arg2 = **args.offset(1);

        *result = userdata(arg1, arg2);
    }
}
