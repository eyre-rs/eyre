use std::{
    any::TypeId,
    fmt::{self, Debug, Display},
    mem::{self, ManuallyDrop},
    ptr::NonNull,
};

use crate::{
    ptr::{MutPtr, OwnedPtr, RefPtr},
    Chain, EyreHandler, Report, StdError,
};

pub(crate) struct ErrorVTable {
    pub(crate) object_drop: unsafe fn(OwnedPtr<ErrorImpl<()>>),
    pub(crate) object_ref:
        unsafe fn(RefPtr<'_, ErrorImpl<()>>) -> &(dyn StdError + Send + Sync + 'static),
    pub(crate) object_mut:
        unsafe fn(MutPtr<'_, ErrorImpl<()>>) -> &mut (dyn StdError + Send + Sync + 'static),
    #[allow(clippy::type_complexity)]
    pub(crate) object_boxed:
        unsafe fn(OwnedPtr<ErrorImpl<()>>) -> Box<dyn StdError + Send + Sync + 'static>,
    pub(crate) object_downcast: unsafe fn(RefPtr<'_, ErrorImpl<()>>, TypeId) -> Option<NonNull<()>>,
    pub(crate) object_downcast_mut:
        unsafe fn(MutPtr<'_, ErrorImpl<()>>, TypeId) -> Option<NonNull<()>>,
    pub(crate) object_drop_rest: unsafe fn(OwnedPtr<ErrorImpl<()>>, TypeId),
}

// repr C to ensure that E remains in the final position.
#[repr(C)]
pub(crate) struct ErrorImpl<E = ()> {
    pub(crate) header: ErrorHeader,
    // NOTE: Don't use directly. Use only through vtable. Erased type may have
    // different alignment.
    pub(crate) _object: E,
}

#[repr(C)]
pub(crate) struct ErrorHeader {
    pub(crate) vtable: &'static ErrorVTable,
    pub(crate) handler: Option<Box<dyn EyreHandler>>,
}
// Reads the header out of `p`. This is the same as `p.as_ref().header`, but
// avoids converting `p` into a reference of a shrunk provenance with a type different than the
// allocation.
pub(crate) fn header(p: RefPtr<'_, ErrorImpl<()>>) -> &'_ ErrorHeader {
    // Safety: `ErrorHeader` is the first field of repr(C) `ErrorImpl`
    unsafe { p.cast().as_ref() }
}

pub(crate) fn header_mut(p: MutPtr<'_, ErrorImpl<()>>) -> &mut ErrorHeader {
    // Safety: `ErrorHeader` is the first field of repr(C) `ErrorImpl`
    unsafe { p.cast().into_mut() }
}

impl<E> ErrorImpl<E> {
    /// Returns a type erased Error
    fn erase(&self) -> RefPtr<'_, ErrorImpl<()>> {
        // Erase the concrete type of E but preserve the vtable in self.vtable
        // for manipulating the resulting thin pointer. This is analogous to an
        // unsize coersion.
        RefPtr::new(self).cast()
    }
}

impl ErrorImpl<()> {
    pub(crate) fn error(this: RefPtr<'_, Self>) -> &(dyn StdError + Send + Sync + 'static) {
        // Use vtable to attach E's native StdError vtable for the right
        // original type E.
        unsafe { (header(this).vtable.object_ref)(this) }
    }

    pub(crate) fn error_mut(this: MutPtr<'_, Self>) -> &mut (dyn StdError + Send + Sync + 'static) {
        // Use vtable to attach E's native StdError vtable for the right
        // original type E.
        unsafe { (header_mut(this).vtable.object_mut)(this) }
    }

    pub(crate) fn chain(this: RefPtr<'_, Self>) -> Chain<'_> {
        Chain::new(Self::error(this))
    }

    pub(crate) fn header(this: RefPtr<'_, ErrorImpl>) -> &ErrorHeader {
        header(this)
    }
}

impl<E> StdError for ErrorImpl<E>
where
    E: StdError,
{
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        ErrorImpl::<()>::error(self.erase()).source()
    }
}

impl<E> Debug for ErrorImpl<E>
where
    E: Debug,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        ErrorImpl::debug(self.erase(), formatter)
    }
}

impl<E> Display for ErrorImpl<E>
where
    E: Display,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(ErrorImpl::error(self.erase()), formatter)
    }
}

impl From<Report> for Box<dyn StdError + Send + Sync + 'static> {
    fn from(error: Report) -> Self {
        let outer = ManuallyDrop::new(error);
        unsafe {
            // Read Box<ErrorImpl<()>> from error. Can't move it out because
            // Report has a Drop impl which we want to not run.
            // Use vtable to attach ErrorImpl<E>'s native StdError vtable for
            // the right original type E.
            (header(outer.inner.as_ref()).vtable.object_boxed)(outer.inner)
        }
    }
}

/// # Safety
///
/// Requires layout of *e to match ErrorImpl<E>.
pub(crate) unsafe fn object_drop<E>(e: OwnedPtr<ErrorImpl<()>>) {
    // Cast to a context type and drop the Box allocation.
    let unerased = unsafe { e.cast::<ErrorImpl<E>>().into_box() };
    drop(unerased);
}

/// # Safety
///
/// Requires layout of *e to match ErrorImpl<E>.
pub(crate) unsafe fn object_drop_front<E>(e: OwnedPtr<ErrorImpl<()>>, target: TypeId) {
    // Drop the fields of ErrorImpl other than E as well as the Box allocation,
    // without dropping E itself. This is used by downcast after doing a
    // ptr::read to take ownership of the E.
    let _ = target;
    // Note: This must not use `mem::transmute` because it tries to reborrow the `Unique`
    //   contained in `Box`, which must not be done. In practice this probably won't make any
    //   difference by now, but technically it's unsound.
    //   see: https://github.com/rust-lang/unsafe-code-guidelines/blob/master/wip/stacked-borrows.m
    let unerased = unsafe { e.cast::<ErrorImpl<E>>().into_box() };

    mem::forget(unerased._object)
}

/// # Safety
///
/// Requires layout of *e to match ErrorImpl<E>.
pub(crate) unsafe fn object_ref<E>(
    e: RefPtr<'_, ErrorImpl<()>>,
) -> &(dyn StdError + Send + Sync + 'static)
where
    E: StdError + Send + Sync + 'static,
{
    // Attach E's native StdError vtable onto a pointer to self._object.
    &unsafe { e.cast::<ErrorImpl<E>>().as_ref() }._object
}

/// # Safety
///
/// Requires layout of *e to match ErrorImpl<E>.
pub(crate) unsafe fn object_mut<E>(
    e: MutPtr<'_, ErrorImpl<()>>,
) -> &mut (dyn StdError + Send + Sync + 'static)
where
    E: StdError + Send + Sync + 'static,
{
    // Attach E's native StdError vtable onto a pointer to self._object.
    &mut unsafe { e.cast::<ErrorImpl<E>>().into_mut() }._object
}

/// # Safety
///
/// Requires layout of *e to match ErrorImpl<E>.
pub(crate) unsafe fn object_boxed<E>(
    e: OwnedPtr<ErrorImpl<()>>,
) -> Box<dyn StdError + Send + Sync + 'static>
where
    E: StdError + Send + Sync + 'static,
{
    // Attach ErrorImpl<E>'s native StdError vtable. The StdError impl is below.
    unsafe { e.cast::<ErrorImpl<E>>().into_box() }
}

/// # Safety
///
/// Requires layout of *e to match ErrorImpl<E>.
pub(crate) unsafe fn object_downcast<E>(
    e: RefPtr<'_, ErrorImpl<()>>,
    target: TypeId,
) -> Option<NonNull<()>>
where
    E: 'static,
{
    if TypeId::of::<E>() == target {
        // Caller is looking for an E pointer and e is ErrorImpl<E>, take a
        // pointer to its E field.
        let unerased = unsafe { e.cast::<ErrorImpl<E>>().as_ref() };
        Some(NonNull::from(&(unerased._object)).cast::<()>())
    } else {
        None
    }
}

/// # Safety
///
/// Requires layout of *e to match ErrorImpl<E>.
pub(crate) unsafe fn object_downcast_mut<E>(
    e: MutPtr<'_, ErrorImpl<()>>,
    target: TypeId,
) -> Option<NonNull<()>>
where
    E: 'static,
{
    if TypeId::of::<E>() == target {
        // Caller is looking for an E pointer and e is ErrorImpl<E>, take a
        // pointer to its E field.
        let unerased = unsafe { e.cast::<ErrorImpl<E>>().into_mut() };
        Some(NonNull::from(&mut (unerased._object)).cast::<()>())
    } else {
        None
    }
}
