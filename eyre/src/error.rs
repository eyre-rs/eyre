use crate::builder::ReportBuilder;
use crate::chain::Chain;
use crate::ptr::{MutPtr, OwnedPtr, RefPtr};
use crate::vtable::{
    header, header_mut, object_boxed, object_drop, object_mut, object_ref, ErrorHeader, ErrorImpl,
    ErrorVTable,
};
use crate::EyreHandler;
use crate::{Report, StdError};
use core::any::TypeId;
use core::fmt::{self, Debug, Display};
use core::mem::ManuallyDrop;
use core::ptr::{self, NonNull};

use core::ops::{Deref, DerefMut};

impl Report {
    /// Create a new error object from any error type.
    ///
    /// The error type must be threadsafe and `'static`, so that the `Report`
    /// will be as well.
    ///
    /// If the error type does not provide a backtrace, a backtrace will be
    /// created here to ensure that a backtrace exists.
    #[cfg_attr(track_caller, track_caller)]
    pub fn new<E>(error: E) -> Self
    where
        E: StdError + Send + Sync + 'static,
    {
        ReportBuilder::default().from_stderr(error)
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
    #[cfg_attr(track_caller, track_caller)]
    pub fn msg<M>(message: M) -> Self
    where
        M: Display + Debug + Send + Sync + 'static,
    {
        ReportBuilder::default().from_msg(message)
    }

    #[cfg_attr(track_caller, track_caller)]
    pub(crate) fn from_adhoc<M>(message: M) -> Self
    where
        M: Display + Debug + Send + Sync + 'static,
    {
        ReportBuilder::default().from_msg(message)
    }

    #[cfg_attr(track_caller, track_caller)]
    pub(crate) fn from_display<M>(message: M) -> Self
    where
        M: Display + Send + Sync + 'static,
    {
        ReportBuilder::default().from_display(message)
    }

    #[cfg_attr(track_caller, track_caller)]
    /// Wraps a source error with a message
    pub(crate) fn from_msg<D, E>(msg: D, error: E) -> Self
    where
        D: Display + Send + Sync + 'static,
        E: StdError + Send + Sync + 'static,
    {
        ReportBuilder::default().wrap_with_msg(msg, error)
    }

    // Takes backtrace as argument rather than capturing it here so that the
    // user sees one fewer layer of wrapping noise in the backtrace.
    //
    // Unsafe because the given vtable must have sensible behavior on the error
    // value of type E.
    pub(crate) unsafe fn construct<E>(
        error: E,
        vtable: &'static ErrorVTable,
        handler: Option<Box<dyn EyreHandler>>,
    ) -> Self
    where
        E: StdError + Send + Sync + 'static,
    {
        let inner = ErrorImpl {
            header: ErrorHeader { vtable, handler },
            _object: error,
        };

        // Construct a new owned allocation through a raw pointer
        //
        // This does not keep the allocation around as a `Box` which would invalidate an
        // references when moved
        let ptr = OwnedPtr::<ErrorImpl<E>>::new(inner);

        // Safety: the type
        let ptr = ptr.cast::<ErrorImpl<()>>();
        Report { inner: ptr }
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
        // Safety: this access a `ErrorImpl<unknown>` as a valid reference to a `ErrorImpl<()>`
        //
        // As the generic is at the end of the struct and the struct is `repr(C)` this reference
        // will be within bounds of the original pointer, and the field will have the same offset
        let handler = header_mut(self.inner.as_mut()).handler.take();
        let error: ContextError<D, Report> = ContextError { msg, error: self };

        let vtable = &ErrorVTable {
            object_drop: object_drop::<ContextError<D, Report>>,
            object_ref: object_ref::<ContextError<D, Report>>,
            object_mut: object_mut::<ContextError<D, Report>>,
            object_boxed: object_boxed::<ContextError<D, Report>>,
            object_downcast: context_chain_downcast::<D>,
            object_downcast_mut: context_chain_downcast_mut::<D>,
            object_drop_rest: context_chain_drop_rest::<D>,
        };

        // Safety: passing vtable that operates on the right type.
        unsafe { Report::construct(error, vtable, handler) }
    }

    /// Access the vtable for the current error object.
    fn vtable(&self) -> &'static ErrorVTable {
        header(self.inner.as_ref()).vtable
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
    pub fn chain(&self) -> Chain<'_> {
        ErrorImpl::chain(self.inner.as_ref())
    }

    /// The lowest level cause of this error &mdash; this error's cause's
    /// cause's cause etc.
    ///
    /// The root cause is the last error in the iterator produced by
    /// [`chain()`][Report::chain].
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
            let addr = match (self.vtable().object_downcast)(self.inner.as_ref(), target) {
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

            // Drop rest of the data structure outside of E.
            (outer.vtable().object_drop_rest)(inner, target);

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
    /// # #[cfg(not(feature = "auto-install"))]
    /// # eyre::set_hook(Box::new(eyre::DefaultHandler::default_with)).unwrap();
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
            let addr = (self.vtable().object_downcast)(self.inner.as_ref(), target)?;
            Some(addr.cast::<E>().as_ref())
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
            let addr = (self.vtable().object_downcast_mut)(self.inner.as_mut(), target)?;
            Some(addr.cast::<E>().as_mut())
        }
    }

    /// Get a reference to the Handler for this Report.
    pub fn handler(&self) -> &dyn EyreHandler {
        header(self.inner.as_ref())
            .handler
            .as_ref()
            .unwrap()
            .as_ref()
    }

    /// Get a mutable reference to the Handler for this Report.
    pub fn handler_mut(&mut self) -> &mut dyn EyreHandler {
        header_mut(self.inner.as_mut())
            .handler
            .as_mut()
            .unwrap()
            .as_mut()
    }

    /// Get a reference to the Handler for this Report.
    #[doc(hidden)]
    pub fn context(&self) -> &dyn EyreHandler {
        header(self.inner.as_ref())
            .handler
            .as_ref()
            .unwrap()
            .as_ref()
    }

    /// Get a mutable reference to the Handler for this Report.
    #[doc(hidden)]
    pub fn context_mut(&mut self) -> &mut dyn EyreHandler {
        header_mut(self.inner.as_mut())
            .handler
            .as_mut()
            .unwrap()
            .as_mut()
    }
}

impl<E> From<E> for Report
where
    E: StdError + Send + Sync + 'static,
{
    #[cfg_attr(track_caller, track_caller)]
    fn from(error: E) -> Self {
        ReportBuilder::default().from_stderr(error)
    }
}

impl Deref for Report {
    type Target = dyn StdError + Send + Sync + 'static;

    fn deref(&self) -> &Self::Target {
        ErrorImpl::error(self.inner.as_ref())
    }
}

impl DerefMut for Report {
    fn deref_mut(&mut self) -> &mut Self::Target {
        ErrorImpl::error_mut(self.inner.as_mut())
    }
}

impl Display for Report {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        ErrorImpl::display(self.inner.as_ref(), formatter)
    }
}

impl Debug for Report {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        ErrorImpl::debug(self.inner.as_ref(), formatter)
    }
}

impl Drop for Report {
    fn drop(&mut self) {
        unsafe {
            // Read Box<ErrorImpl<()>> from self.
            (self.vtable().object_drop)(self.inner);
        }
    }
}

/// # Safety
///
/// Requires layout of *e to match ErrorImpl<ContextError<D, E>>.
pub(crate) unsafe fn context_downcast<D, E>(
    e: RefPtr<'_, ErrorImpl<()>>,
    target: TypeId,
) -> Option<NonNull<()>>
where
    D: 'static,
    E: 'static,
{
    if TypeId::of::<D>() == target {
        let unerased = unsafe { e.cast::<ErrorImpl<ContextError<D, E>>>().as_ref() };
        let addr = NonNull::from(&unerased._object.msg).cast::<()>();
        Some(addr)
    } else if TypeId::of::<E>() == target {
        let unerased = unsafe { e.cast::<ErrorImpl<ContextError<D, E>>>().as_ref() };
        let addr = NonNull::from(&unerased._object.error).cast::<()>();
        Some(addr)
    } else {
        None
    }
}

/// # Safety
///
/// Requires layout of *e to match ErrorImpl<ContextError<D, E>>.
pub(crate) unsafe fn context_downcast_mut<D, E>(
    e: MutPtr<'_, ErrorImpl<()>>,
    target: TypeId,
) -> Option<NonNull<()>>
where
    D: 'static,
    E: 'static,
{
    if TypeId::of::<D>() == target {
        let unerased = unsafe { e.cast::<ErrorImpl<ContextError<D, E>>>().into_mut() };
        let addr = NonNull::from(&unerased._object.msg).cast::<()>();
        Some(addr)
    } else if TypeId::of::<E>() == target {
        let unerased = unsafe { e.cast::<ErrorImpl<ContextError<D, E>>>().into_mut() };
        let addr = NonNull::from(&mut unerased._object.error).cast::<()>();
        Some(addr)
    } else {
        None
    }
}
/// # Safety
///
/// Requires layout of *e to match ErrorImpl<ContextError<D, E>>.
pub(crate) unsafe fn context_drop_rest<D, E>(e: OwnedPtr<ErrorImpl<()>>, target: TypeId)
where
    D: 'static,
    E: 'static,
{
    // Called after downcasting by value to either the D or the E and doing a
    // ptr::read to take ownership of that value.
    if TypeId::of::<D>() == target {
        unsafe {
            e.cast::<ErrorImpl<ContextError<ManuallyDrop<E>, E>>>()
                .into_box()
        };
    } else {
        debug_assert_eq!(TypeId::of::<E>(), target);
        unsafe {
            e.cast::<ErrorImpl<ContextError<E, ManuallyDrop<E>>>>()
                .into_box()
        };
    }
}

/// # Safety
///
/// Requires layout of *e to match ErrorImpl<ContextError<D, Report>>.
unsafe fn context_chain_downcast<D>(
    e: RefPtr<'_, ErrorImpl<()>>,
    target: TypeId,
) -> Option<NonNull<()>>
where
    D: 'static,
{
    let unerased = unsafe { e.cast::<ErrorImpl<ContextError<D, Report>>>().as_ref() };
    if TypeId::of::<D>() == target {
        let addr = NonNull::from(&unerased._object.msg).cast::<()>();
        Some(addr)
    } else {
        // Recurse down the context chain per the inner error's vtable.
        let source = &unerased._object.error;
        unsafe { (source.vtable().object_downcast)(source.inner.as_ref(), target) }
    }
}

/// # Safety
///
/// Requires layout of *e to match ErrorImpl<ContextError<D, Report>>.
pub(crate) unsafe fn context_chain_downcast_mut<D>(
    e: MutPtr<'_, ErrorImpl<()>>,
    target: TypeId,
) -> Option<NonNull<()>>
where
    D: 'static,
{
    let unerased = unsafe { e.cast::<ErrorImpl<ContextError<D, Report>>>().into_mut() };
    if TypeId::of::<D>() == target {
        let addr = NonNull::from(&unerased._object.msg).cast::<()>();
        Some(addr)
    } else {
        // Recurse down the context chain per the inner error's vtable.
        let source = &mut unerased._object.error;
        unsafe { (source.vtable().object_downcast_mut)(source.inner.as_mut(), target) }
    }
}

/// # Safety
///
/// Requires layout of *e to match ErrorImpl<ContextError<D, Report>>.
pub(crate) unsafe fn context_chain_drop_rest<D>(e: OwnedPtr<ErrorImpl<()>>, target: TypeId)
where
    D: 'static,
{
    // Called after downcasting by value to either the D or one of the causes
    // and doing a ptr::read to take ownership of that value.
    if TypeId::of::<D>() == target {
        let unerased = unsafe {
            e.cast::<ErrorImpl<ContextError<ManuallyDrop<D>, Report>>>()
                .into_box()
        };
        // Drop the entire rest of the data structure rooted in the next Report.
        drop(unerased);
    } else {
        unsafe {
            let unerased = e
                .cast::<ErrorImpl<ContextError<D, ManuallyDrop<Report>>>>()
                .into_box();
            // Read out a ManuallyDrop<Box<ErrorImpl<()>>> from the next error.
            let inner = ptr::read(&unerased.as_ref()._object.error.inner);
            drop(unerased);
            // Recursively drop the next error using the same target typeid.
            (header(inner.as_ref()).vtable.object_drop_rest)(inner, target);
        }
    }
}

// repr C to ensure that ContextError<D, E> has the same layout as
// ContextError<ManuallyDrop<D>, E> and ContextError<D, ManuallyDrop<E>>.
#[repr(C)]
pub(crate) struct ContextError<D, E> {
    pub(crate) msg: D,
    pub(crate) error: E,
}

impl From<Report> for Box<dyn StdError + 'static> {
    fn from(error: Report) -> Self {
        Box::<dyn StdError + Send + Sync>::from(error)
    }
}

impl AsRef<dyn StdError + Send + Sync> for Report {
    fn as_ref(&self) -> &(dyn StdError + Send + Sync + 'static) {
        &**self
    }
}

impl AsRef<dyn StdError> for Report {
    fn as_ref(&self) -> &(dyn StdError + 'static) {
        &**self
    }
}

#[cfg(feature = "pyo3")]
mod pyo3_compat;
