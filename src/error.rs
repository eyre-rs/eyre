use crate::alloc::Box;
use crate::chain::Chain;
use crate::EyreContext;
use crate::{Report, StdError};
use core::any::Any;
use core::any::TypeId;
use core::fmt::{self, Debug, Display};
use core::mem::{self, ManuallyDrop};
use core::ptr::{self, NonNull};

#[cfg(backtrace)]
use crate::backtrace::Backtrace;

#[cfg(feature = "std")]
use core::ops::{Deref, DerefMut};

impl<C> Report<C>
where
    C: EyreContext,
{
    /// Create a new error object from any error type.
    ///
    /// The error type must be threadsafe and `'static`, so that the `Report`
    /// will be as well.
    ///
    /// If the error type does not provide a backtrace, a backtrace will be
    /// created here to ensure that a backtrace exists.
    #[cfg(feature = "std")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "std")))]
    pub fn new<E>(error: E) -> Self
    where
        E: StdError + Send + Sync + 'static,
    {
        Report::from_std(error)
    }

    /// Create a new error object from a printable error message.
    ///
    /// If the argument implements std::error::Error, prefer `Report::new`
    /// instead which preserves the underlying error's cause chain and
    /// backtrace. If the argument may or may not implement std::error::Error
    /// now or in the future, use `eyre!(err)` which handles either way
    /// correctly.
    ///
    /// `Report::msg("...")` is equivalent to `eyre!("...")` but occasionally
    /// convenient in places where a function is preferable over a macro, such
    /// as iterator or stream combinators:
    ///
    /// ```
    /// # mod ffi {
    /// #     pub struct Input;
    /// #     pub struct Output;
    /// #     pub async fn do_some_work(_: Input) -> Result<Output, &'static str> {
    /// #         unimplemented!()
    /// #     }
    /// # }
    /// #
    /// # use ffi::{Input, Output};
    /// #
    /// use eyre::{Report, Result};
    /// use futures::stream::{Stream, StreamExt, TryStreamExt};
    ///
    /// async fn demo<S>(stream: S) -> Result<Vec<Output>>
    /// where
    ///     S: Stream<Item = Input>,
    /// {
    ///     stream
    ///         .then(ffi::do_some_work) // returns Result<Output, &str>
    ///         .map_err(Report::msg)
    ///         .try_collect()
    ///         .await
    /// }
    /// ```
    pub fn msg<M>(message: M) -> Self
    where
        M: Display + Debug + Send + Sync + 'static,
    {
        Report::from_adhoc(message)
    }

    #[cfg(feature = "std")]
    pub(crate) fn from_std<E>(error: E) -> Self
    where
        E: StdError + Send + Sync + 'static,
    {
        let vtable = &ErrorVTable {
            object_drop: object_drop::<E, C>,
            object_ref: object_ref::<E, C>,
            #[cfg(feature = "std")]
            object_mut: object_mut::<E, C>,
            object_boxed: object_boxed::<E, C>,
            object_downcast: object_downcast::<E, C>,
            object_drop_rest: object_drop_front::<E, C>,
        };

        // Safety: passing vtable that operates on the right type E.
        let context = Some(C::default(&error));
        unsafe { Report::construct(error, vtable, context) }
    }

    pub(crate) fn from_adhoc<M>(message: M) -> Self
    where
        M: Display + Debug + Send + Sync + 'static,
    {
        use crate::wrapper::MessageError;
        let error: MessageError<M> = MessageError(message);
        let vtable = &ErrorVTable {
            object_drop: object_drop::<MessageError<M>, C>,
            object_ref: object_ref::<MessageError<M>, C>,
            #[cfg(feature = "std")]
            object_mut: object_mut::<MessageError<M>, C>,
            object_boxed: object_boxed::<MessageError<M>, C>,
            object_downcast: object_downcast::<M, C>,
            object_drop_rest: object_drop_front::<M, C>,
        };

        // Safety: MessageError is repr(transparent) so it is okay for the
        // vtable to allow casting the MessageError<M> to M.
        let context = Some(C::default(&error));
        unsafe { Report::construct(error, vtable, context) }
    }

    #[cfg(feature = "std")]
    pub(crate) fn from_msg<D, E>(msg: D, error: E) -> Self
    where
        D: Display + Send + Sync + 'static,
        E: StdError + Send + Sync + 'static,
    {
        let error: ContextError<D, E> = ContextError { msg, error };

        let vtable = &ErrorVTable {
            object_drop: object_drop::<ContextError<D, E>, C>,
            object_ref: object_ref::<ContextError<D, E>, C>,
            #[cfg(feature = "std")]
            object_mut: object_mut::<ContextError<D, E>, C>,
            object_boxed: object_boxed::<ContextError<D, E>, C>,
            object_downcast: context_downcast::<D, E, C>,
            object_drop_rest: context_drop_rest::<D, E, C>,
        };

        // Safety: passing vtable that operates on the right type.
        let context = Some(C::default(&error));
        unsafe { Report::construct(error, vtable, context) }
    }

    #[cfg(feature = "std")]
    pub(crate) fn from_boxed(error: Box<dyn StdError + Send + Sync>) -> Self {
        use crate::wrapper::BoxedError;
        let error = BoxedError(error);
        let context = Some(C::default(&error));
        let vtable = &ErrorVTable {
            object_drop: object_drop::<BoxedError, C>,
            object_ref: object_ref::<BoxedError, C>,
            #[cfg(feature = "std")]
            object_mut: object_mut::<BoxedError, C>,
            object_boxed: object_boxed::<BoxedError, C>,
            object_downcast: object_downcast::<Box<dyn StdError + Send + Sync>, C>,
            object_drop_rest: object_drop_front::<Box<dyn StdError + Send + Sync>, C>,
        };

        // Safety: BoxedError is repr(transparent) so it is okay for the vtable
        // to allow casting to Box<dyn StdError + Send + Sync>.
        unsafe { Report::construct(error, vtable, context) }
    }

    // Takes backtrace as argument rather than capturing it here so that the
    // user sees one fewer layer of wrapping noise in the backtrace.
    //
    // Unsafe because the given vtable must have sensible behavior on the error
    // value of type E.
    unsafe fn construct<E>(error: E, vtable: &'static ErrorVTable<C>, context: Option<C>) -> Self
    where
        E: StdError + Send + Sync + 'static,
    {
        let inner = Box::new(ErrorImpl {
            vtable,
            context,
            _object: error,
        });
        // Erase the concrete type of E from the compile-time type system. This
        // is equivalent to the safe unsize coersion from Box<ErrorImpl<E>> to
        // Box<ErrorImpl<dyn StdError + Send + Sync + 'static>> except that the
        // result is a thin pointer. The necessary behavior for manipulating the
        // underlying ErrorImpl<E> is preserved in the vtable provided by the
        // caller rather than a builtin fat pointer vtable.
        let erased = mem::transmute::<Box<ErrorImpl<E, C>>, Box<ErrorImpl<(), C>>>(inner);
        let inner = ManuallyDrop::new(erased);
        Report { inner }
    }

    /// Create a new error from an error message to wrap the existing error.
    ///
    /// For attaching a higher level error message to a `Result` as it is propagated, the
    /// [`WrapErr`][crate::WrapErr] extension trait may be more convenient than this function.
    ///
    /// The primary reason to use `error.wrap_err(...)` instead of `result.wrap_err(...)` via the
    /// `WrapErr` trait would be if the message needs to depend on some data held by the underlying
    /// error:
    ///
    /// ```
    /// # use std::fmt::{self, Debug, Display};
    /// #
    /// # type T = ();
    /// #
    /// # impl std::error::Error for ParseError {}
    /// # impl Debug for ParseError {
    /// #     fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
    /// #         unimplemented!()
    /// #     }
    /// # }
    /// # impl Display for ParseError {
    /// #     fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
    /// #         unimplemented!()
    /// #     }
    /// # }
    /// #
    /// use eyre::Result;
    /// use std::fs::File;
    /// use std::path::Path;
    ///
    /// struct ParseError {
    ///     line: usize,
    ///     column: usize,
    /// }
    ///
    /// fn parse_impl(file: File) -> Result<T, ParseError> {
    ///     # const IGNORE: &str = stringify! {
    ///     ...
    ///     # };
    ///     # unimplemented!()
    /// }
    ///
    /// pub fn parse(path: impl AsRef<Path>) -> Result<T> {
    ///     let file = File::open(&path)?;
    ///     parse_impl(file).map_err(|error| {
    ///         let message = format!(
    ///             "only the first {} lines of {} are valid",
    ///             error.line, path.as_ref().display(),
    ///         );
    ///         eyre::Report::new(error).wrap_err(message)
    ///     })
    /// }
    /// ```
    pub fn wrap_err<D>(mut self, msg: D) -> Self
    where
        D: Display + Send + Sync + 'static,
    {
        let context = self.inner.context.take();
        let error: ContextError<D, Report<C>> = ContextError { msg, error: self };

        let vtable = &ErrorVTable {
            object_drop: object_drop::<ContextError<D, Report<C>>, C>,
            object_ref: object_ref::<ContextError<D, Report<C>>, C>,
            #[cfg(feature = "std")]
            object_mut: object_mut::<ContextError<D, Report<C>>, C>,
            object_boxed: object_boxed::<ContextError<D, Report<C>>, C>,
            object_downcast: context_chain_downcast::<D, C>,
            object_drop_rest: context_chain_drop_rest::<D, C>,
        };

        // Safety: passing vtable that operates on the right type.
        unsafe { Report::construct(error, vtable, context) }
    }

    /// Get the backtrace for this Report.
    ///
    /// Backtraces are only available on the nightly channel. Tracking issue:
    /// [rust-lang/rust#53487][tracking].
    ///
    /// In order for the backtrace to be meaningful, one of the two environment
    /// variables `RUST_LIB_BACKTRACE=1` or `RUST_BACKTRACE=1` must be defined
    /// and `RUST_LIB_BACKTRACE` must not be `0`. Backtraces are somewhat
    /// expensive to capture in Rust, so we don't necessarily want to be
    /// capturing them all over the place all the time.
    ///
    /// - If you want panics and errors to both have backtraces, set
    ///   `RUST_BACKTRACE=1`;
    /// - If you want only errors to have backtraces, set
    ///   `RUST_LIB_BACKTRACE=1`;
    /// - If you want only panics to have backtraces, set `RUST_BACKTRACE=1` and
    ///   `RUST_LIB_BACKTRACE=0`.
    ///
    /// [tracking]: https://github.com/rust-lang/rust/issues/53487
    #[cfg(backtrace)]
    pub fn backtrace(&self) -> &Backtrace {
        self.inner.backtrace()
    }

    /// An iterator of the chain of source errors contained by this Report.
    ///
    /// This iterator will visit every error in the cause chain of this error
    /// object, beginning with the error that this error object was created
    /// from.
    ///
    /// # Example
    ///
    /// ```
    /// use eyre::Report;
    /// use std::io;
    ///
    /// pub fn underlying_io_error_kind(error: &Report) -> Option<io::ErrorKind> {
    ///     for cause in error.chain() {
    ///         if let Some(io_error) = cause.downcast_ref::<io::Error>() {
    ///             return Some(io_error.kind());
    ///         }
    ///     }
    ///     None
    /// }
    /// ```
    #[cfg(feature = "std")]
    pub fn chain(&self) -> Chain {
        self.inner.chain()
    }

    /// The lowest level cause of this error &mdash; this error's cause's
    /// cause's cause etc.
    ///
    /// The root cause is the last error in the iterator produced by
    /// [`chain()`][Report::chain].
    #[cfg(feature = "std")]
    pub fn root_cause(&self) -> &(dyn StdError + 'static) {
        let mut chain = self.chain();
        let mut root_cause = chain.next().unwrap();
        for cause in chain {
            root_cause = cause;
        }
        root_cause
    }

    /// Returns true if `E` is the type held by this error object.
    ///
    /// For errors constructed from messages, this method returns true if `E` matches the type of
    /// the message `D` **or** the type of the error on which the message has been attached. For
    /// details about the interaction between message and downcasting, [see here].
    ///
    /// [see here]: trait.WrapErr.html#effect-on-downcasting
    pub fn is<E>(&self) -> bool
    where
        E: Display + Debug + Send + Sync + 'static,
    {
        self.downcast_ref::<E>().is_some()
    }

    /// Attempt to downcast the error object to a concrete type.
    pub fn downcast<E>(self) -> Result<E, Self>
    where
        E: Display + Debug + Send + Sync + 'static,
    {
        let target = TypeId::of::<E>();
        unsafe {
            // Use vtable to find NonNull<()> which points to a value of type E
            // somewhere inside the data structure.
            let addr = match (self.inner.vtable.object_downcast)(&self.inner, target) {
                Some(addr) => addr,
                None => return Err(self),
            };

            // Prepare to read E out of the data structure. We'll drop the rest
            // of the data structure separately so that E is not dropped.
            let outer = ManuallyDrop::new(self);

            // Read E from where the vtable found it.
            let error = ptr::read(addr.cast::<E>().as_ptr());

            // Read Box<ErrorImpl<()>> from self. Can't move it out because
            // Report has a Drop impl which we want to not run.
            let inner = ptr::read(&outer.inner);
            let erased = ManuallyDrop::into_inner(inner);

            // Drop rest of the data structure outside of E.
            (erased.vtable.object_drop_rest)(erased, target);

            Ok(error)
        }
    }

    /// Downcast this error object by reference.
    ///
    /// # Example
    ///
    /// ```
    /// # use eyre::{Report, eyre};
    /// # use std::fmt::{self, Display};
    /// # use std::task::Poll;
    /// #
    /// # #[derive(Debug)]
    /// # enum DataStoreError {
    /// #     Censored(()),
    /// # }
    /// #
    /// # impl Display for DataStoreError {
    /// #     fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
    /// #         unimplemented!()
    /// #     }
    /// # }
    /// #
    /// # impl std::error::Error for DataStoreError {}
    /// #
    /// # const REDACTED_CONTENT: () = ();
    /// #
    /// # let error: Report = eyre!("...");
    /// # let root_cause = &error;
    /// #
    /// # let ret =
    /// // If the error was caused by redaction, then return a tombstone instead
    /// // of the content.
    /// match root_cause.downcast_ref::<DataStoreError>() {
    ///     Some(DataStoreError::Censored(_)) => Ok(Poll::Ready(REDACTED_CONTENT)),
    ///     None => Err(error),
    /// }
    /// # ;
    /// ```
    pub fn downcast_ref<E>(&self) -> Option<&E>
    where
        E: Display + Debug + Send + Sync + 'static,
    {
        let target = TypeId::of::<E>();
        unsafe {
            // Use vtable to find NonNull<()> which points to a value of type E
            // somewhere inside the data structure.
            let addr = (self.inner.vtable.object_downcast)(&self.inner, target)?;
            Some(&*addr.cast::<E>().as_ptr())
        }
    }

    /// Downcast this error object by mutable reference.
    pub fn downcast_mut<E>(&mut self) -> Option<&mut E>
    where
        E: Display + Debug + Send + Sync + 'static,
    {
        let target = TypeId::of::<E>();
        unsafe {
            // Use vtable to find NonNull<()> which points to a value of type E
            // somewhere inside the data structure.
            let addr = (self.inner.vtable.object_downcast)(&self.inner, target)?;
            Some(&mut *addr.cast::<E>().as_ptr())
        }
    }

    pub fn member_ref<T: Any>(&self) -> Option<&T> {
        self.inner.member_ref()
    }

    pub fn member_mut<T: Any>(&mut self) -> Option<&mut T> {
        self.inner.member_mut()
    }

    pub fn context(&self) -> &C {
        self.inner.context.as_ref().unwrap()
    }

    pub fn context_mut(&mut self) -> &mut C {
        self.inner.context.as_mut().unwrap()
    }
}

#[cfg(feature = "std")]
impl<E, C> From<E> for Report<C>
where
    C: EyreContext,
    E: StdError + Send + Sync + 'static,
{
    fn from(error: E) -> Self {
        Report::from_std(error)
    }
}

#[cfg(feature = "std")]
impl<C: EyreContext> Deref for Report<C> {
    type Target = dyn StdError + Send + Sync + 'static;

    fn deref(&self) -> &Self::Target {
        self.inner.error()
    }
}

#[cfg(feature = "std")]
impl<C: EyreContext> DerefMut for Report<C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner.error_mut()
    }
}

impl<C: EyreContext> Display for Report<C> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        self.inner.display(formatter)
    }
}

impl<C: EyreContext> Debug for Report<C> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        self.inner.debug(formatter)
    }
}

impl<C> Drop for Report<C>
where
    C: EyreContext,
{
    fn drop(&mut self) {
        unsafe {
            // Read Box<ErrorImpl<()>> from self.
            let inner = ptr::read(&self.inner);
            let erased = ManuallyDrop::into_inner(inner);

            // Invoke the vtable's drop behavior.
            (erased.vtable.object_drop)(erased);
        }
    }
}

struct ErrorVTable<C>
where
    C: EyreContext,
{
    object_drop: unsafe fn(Box<ErrorImpl<(), C>>),
    object_ref: unsafe fn(&ErrorImpl<(), C>) -> &(dyn StdError + Send + Sync + 'static),
    #[cfg(feature = "std")]
    object_mut: unsafe fn(&mut ErrorImpl<(), C>) -> &mut (dyn StdError + Send + Sync + 'static),
    #[allow(clippy::type_complexity)]
    object_boxed: unsafe fn(Box<ErrorImpl<(), C>>) -> Box<dyn StdError + Send + Sync + 'static>,
    object_downcast: unsafe fn(&ErrorImpl<(), C>, TypeId) -> Option<NonNull<()>>,
    object_drop_rest: unsafe fn(Box<ErrorImpl<(), C>>, TypeId),
}

// Safety: requires layout of *e to match ErrorImpl<E>.
unsafe fn object_drop<E, C>(e: Box<ErrorImpl<(), C>>)
where
    C: EyreContext,
{
    // Cast back to ErrorImpl<E> so that the allocator receives the correct
    // Layout to deallocate the Box's memory.
    let unerased = mem::transmute::<Box<ErrorImpl<(), C>>, Box<ErrorImpl<E, C>>>(e);
    drop(unerased);
}

// Safety: requires layout of *e to match ErrorImpl<E>.
unsafe fn object_drop_front<E, C>(e: Box<ErrorImpl<(), C>>, target: TypeId)
where
    C: EyreContext,
{
    // Drop the fields of ErrorImpl other than E as well as the Box allocation,
    // without dropping E itself. This is used by downcast after doing a
    // ptr::read to take ownership of the E.
    let _ = target;
    let unerased = mem::transmute::<Box<ErrorImpl<(), C>>, Box<ErrorImpl<ManuallyDrop<E>, C>>>(e);
    drop(unerased);
}

// Safety: requires layout of *e to match ErrorImpl<E>.
unsafe fn object_ref<E, C>(e: &ErrorImpl<(), C>) -> &(dyn StdError + Send + Sync + 'static)
where
    C: EyreContext,
    E: StdError + Send + Sync + 'static,
{
    // Attach E's native StdError vtable onto a pointer to self._object.
    &(*(e as *const ErrorImpl<(), C> as *const ErrorImpl<E, C>))._object
}

// Safety: requires layout of *e to match ErrorImpl<E>.
#[cfg(feature = "std")]
unsafe fn object_mut<E, C>(e: &mut ErrorImpl<(), C>) -> &mut (dyn StdError + Send + Sync + 'static)
where
    C: EyreContext,
    E: StdError + Send + Sync + 'static,
{
    // Attach E's native StdError vtable onto a pointer to self._object.
    &mut (*(e as *mut ErrorImpl<(), C> as *mut ErrorImpl<E, C>))._object
}

// Safety: requires layout of *e to match ErrorImpl<E>.
unsafe fn object_boxed<E, C>(e: Box<ErrorImpl<(), C>>) -> Box<dyn StdError + Send + Sync + 'static>
where
    C: EyreContext,
    E: StdError + Send + Sync + 'static,
    C: Send + Sync + 'static,
{
    // Attach ErrorImpl<E>'s native StdError vtable. The StdError impl is below.
    mem::transmute::<Box<ErrorImpl<(), C>>, Box<ErrorImpl<E, C>>>(e)
}

// Safety: requires layout of *e to match ErrorImpl<E>.
unsafe fn object_downcast<E, C>(e: &ErrorImpl<(), C>, target: TypeId) -> Option<NonNull<()>>
where
    C: EyreContext,
    E: 'static,
{
    if TypeId::of::<E>() == target {
        // Caller is looking for an E pointer and e is ErrorImpl<E>, take a
        // pointer to its E field.
        let unerased = e as *const ErrorImpl<(), C> as *const ErrorImpl<E, C>;
        let addr = &(*unerased)._object as *const E as *mut ();
        Some(NonNull::new_unchecked(addr))
    } else {
        None
    }
}

// Safety: requires layout of *e to match ErrorImpl<ContextError<D, E>>.
#[cfg(feature = "std")]
unsafe fn context_downcast<D, E, C>(e: &ErrorImpl<(), C>, target: TypeId) -> Option<NonNull<()>>
where
    C: EyreContext,
    D: 'static,
    E: 'static,
{
    if TypeId::of::<D>() == target {
        let unerased = e as *const ErrorImpl<(), C> as *const ErrorImpl<ContextError<D, E>, C>;
        let addr = &(*unerased)._object.msg as *const D as *mut ();
        Some(NonNull::new_unchecked(addr))
    } else if TypeId::of::<E>() == target {
        let unerased = e as *const ErrorImpl<(), C> as *const ErrorImpl<ContextError<D, E>, C>;
        let addr = &(*unerased)._object.error as *const E as *mut ();
        Some(NonNull::new_unchecked(addr))
    } else {
        None
    }
}

// Safety: requires layout of *e to match ErrorImpl<ContextError<D, E>>.
#[cfg(feature = "std")]
unsafe fn context_drop_rest<D, E, C>(e: Box<ErrorImpl<(), C>>, target: TypeId)
where
    C: EyreContext,
    D: 'static,
    E: 'static,
{
    // Called after downcasting by value to either the D or the E and doing a
    // ptr::read to take ownership of that value.
    if TypeId::of::<D>() == target {
        let unerased = mem::transmute::<
            Box<ErrorImpl<(), C>>,
            Box<ErrorImpl<ContextError<ManuallyDrop<D>, E>, C>>,
        >(e);
        drop(unerased);
    } else {
        let unerased = mem::transmute::<
            Box<ErrorImpl<(), C>>,
            Box<ErrorImpl<ContextError<D, ManuallyDrop<E>>, C>>,
        >(e);
        drop(unerased);
    }
}

// Safety: requires layout of *e to match ErrorImpl<ContextError<D, Report>>.
unsafe fn context_chain_downcast<D, C>(e: &ErrorImpl<(), C>, target: TypeId) -> Option<NonNull<()>>
where
    C: EyreContext,
    D: 'static,
{
    if TypeId::of::<D>() == target {
        let unerased =
            e as *const ErrorImpl<(), C> as *const ErrorImpl<ContextError<D, Report<C>>, C>;
        let addr = &(*unerased)._object.msg as *const D as *mut ();
        Some(NonNull::new_unchecked(addr))
    } else {
        // Recurse down the context chain per the inner error's vtable.
        let unerased =
            e as *const ErrorImpl<(), C> as *const ErrorImpl<ContextError<D, Report<C>>, C>;
        let source = &(*unerased)._object.error;
        (source.inner.vtable.object_downcast)(&source.inner, target)
    }
}

// Safety: requires layout of *e to match ErrorImpl<ContextError<D, Report>>.
unsafe fn context_chain_drop_rest<D, C>(e: Box<ErrorImpl<(), C>>, target: TypeId)
where
    C: EyreContext,
    D: 'static,
{
    // Called after downcasting by value to either the D or one of the causes
    // and doing a ptr::read to take ownership of that value.
    if TypeId::of::<D>() == target {
        let unerased = mem::transmute::<
            Box<ErrorImpl<(), C>>,
            Box<ErrorImpl<ContextError<ManuallyDrop<D>, Report<C>>, C>>,
        >(e);
        // Drop the entire rest of the data structure rooted in the next Report.
        drop(unerased);
    } else {
        let unerased = mem::transmute::<
            Box<ErrorImpl<(), C>>,
            Box<ErrorImpl<ContextError<D, ManuallyDrop<Report<C>>>, C>>,
        >(e);
        // Read out a ManuallyDrop<Box<ErrorImpl<()>>> from the next error.
        let inner = ptr::read(&unerased._object.error.inner);
        drop(unerased);
        let erased = ManuallyDrop::into_inner(inner);
        // Recursively drop the next error using the same target typeid.
        (erased.vtable.object_drop_rest)(erased, target);
    }
}

// repr C to ensure that E remains in the final position.
#[repr(C)]
pub(crate) struct ErrorImpl<E, C>
where
    C: EyreContext,
{
    vtable: &'static ErrorVTable<C>,
    pub(crate) context: Option<C>,
    // NOTE: Don't use directly. Use only through vtable. Erased type may have
    // different alignment.
    _object: E,
}

// repr C to ensure that ContextError<D, E> has the same layout as
// ContextError<ManuallyDrop<D>, E> and ContextError<D, ManuallyDrop<E>>.
#[repr(C)]
pub(crate) struct ContextError<D, E> {
    pub msg: D,
    pub error: E,
}

impl<E, C> ErrorImpl<E, C>
where
    C: EyreContext,
{
    fn erase(&self) -> &ErrorImpl<(), C> {
        // Erase the concrete type of E but preserve the vtable in self.vtable
        // for manipulating the resulting thin pointer. This is analogous to an
        // unsize coersion.
        unsafe { &*(self as *const ErrorImpl<E, C> as *const ErrorImpl<(), C>) }
    }
}

impl<C> ErrorImpl<(), C>
where
    C: EyreContext,
{
    pub(crate) fn error(&self) -> &(dyn StdError + Send + Sync + 'static) {
        // Use vtable to attach E's native StdError vtable for the right
        // original type E.
        unsafe { &*(self.vtable.object_ref)(self) }
    }

    #[cfg(feature = "std")]
    pub(crate) fn error_mut(&mut self) -> &mut (dyn StdError + Send + Sync + 'static) {
        // Use vtable to attach E's native StdError vtable for the right
        // original type E.
        unsafe { &mut *(self.vtable.object_mut)(self) }
    }

    pub fn member_ref<T: Any>(&self) -> Option<&T> {
        self.context
            .as_ref()
            .unwrap()
            .member_ref(TypeId::of::<T>())?
            .downcast_ref::<T>()
    }

    pub fn member_mut<T: Any>(&mut self) -> Option<&mut T> {
        self.context
            .as_mut()
            .unwrap()
            .member_mut(TypeId::of::<T>())?
            .downcast_mut::<T>()
    }

    #[cfg(backtrace)]
    pub(crate) fn backtrace(&self) -> &Backtrace {
        // This unwrap can only panic if the underlying error's backtrace method
        // is nondeterministic, which would only happen in maliciously
        // constructed code.
        self.member_ref()
            .or_else(|| self.error().backtrace())
            .expect("backtrace capture failed")
    }

    pub(crate) fn chain(&self) -> Chain {
        Chain::new(self.error())
    }
}

impl<E, C> StdError for ErrorImpl<E, C>
where
    C: EyreContext,
    E: StdError,
{
    #[cfg(backtrace)]
    fn backtrace(&self) -> Option<&Backtrace> {
        Some(self.erase().backtrace())
    }

    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.erase().error().source()
    }
}

impl<E, C> Debug for ErrorImpl<E, C>
where
    C: EyreContext,
    E: Debug,
{
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        self.erase().debug(formatter)
    }
}

impl<E, C> Display for ErrorImpl<E, C>
where
    C: EyreContext,
    E: Display,
{
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.erase().error(), formatter)
    }
}

impl<C: EyreContext> From<Report<C>> for Box<dyn StdError + Send + Sync + 'static> {
    fn from(error: Report<C>) -> Self {
        let outer = ManuallyDrop::new(error);
        unsafe {
            // Read Box<ErrorImpl<()>> from error. Can't move it out because
            // Report has a Drop impl which we want to not run.
            let inner = ptr::read(&outer.inner);
            let erased = ManuallyDrop::into_inner(inner);

            // Use vtable to attach ErrorImpl<E>'s native StdError vtable for
            // the right original type E.
            (erased.vtable.object_boxed)(erased)
        }
    }
}

impl<C: EyreContext> From<Report<C>> for Box<dyn StdError + 'static> {
    fn from(error: Report<C>) -> Self {
        Box::<dyn StdError + Send + Sync>::from(error)
    }
}

#[cfg(feature = "std")]
impl<C: EyreContext> AsRef<dyn StdError + Send + Sync> for Report<C> {
    fn as_ref(&self) -> &(dyn StdError + Send + Sync + 'static) {
        &**self
    }
}

#[cfg(feature = "std")]
impl<C: EyreContext> AsRef<dyn StdError> for Report<C> {
    fn as_ref(&self) -> &(dyn StdError + 'static) {
        &**self
    }
}
