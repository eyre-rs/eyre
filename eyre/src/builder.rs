use crate::{
    error::{context_downcast, context_downcast_mut, context_drop_rest, ContextError},
    vtable::{
        object_boxed, object_downcast, object_downcast_mut, object_drop, object_drop_front,
        object_mut, object_ref, ErrorVTable,
    },
    HandlerBacktraceCompat, HookParams, Report, StdError,
};
use std::fmt::Display;

#[derive(Debug, Default)]
/// Used for incrementally constructing reports
pub struct ReportBuilder {
    params: HookParams,
}

impl ReportBuilder {
    /// Creates a new report builder with default parameters
    pub fn new() -> Self {
        Self::default()
    }

    /// Use the given backtrace for the error
    pub fn with_backtrace(mut self, backtrace: impl Into<HandlerBacktraceCompat>) -> Self {
        self.params.backtrace = Some(backtrace.into());
        self
    }

    #[cfg_attr(track_caller, track_caller)]
    /// Creates a report from the given error message
    pub fn from_msg<M>(self, message: M) -> Report
    where
        M: Display + std::fmt::Debug + Send + Sync + 'static,
    {
        use crate::wrapper::MessageError;
        let error: MessageError<M> = MessageError(message);
        let vtable = &ErrorVTable {
            object_drop: object_drop::<MessageError<M>>,
            object_ref: object_ref::<MessageError<M>>,
            object_mut: object_mut::<MessageError<M>>,
            object_boxed: object_boxed::<MessageError<M>>,
            object_downcast: object_downcast::<M>,
            object_downcast_mut: object_downcast_mut::<M>,
            object_drop_rest: object_drop_front::<M>,
        };

        // Safety: MessageError is repr(transparent) so it is okay for the
        // vtable to allow casting the MessageError<M> to M.
        let handler = Some(crate::capture_handler(&error, self.params));

        unsafe { Report::construct(error, vtable, handler) }
    }

    #[cfg_attr(track_caller, track_caller)]
    /// Creates a report from the following error
    pub fn from_stderr<E>(self, error: E) -> Report
    where
        E: StdError + Send + Sync + 'static,
    {
        let vtable = &ErrorVTable {
            object_drop: object_drop::<E>,
            object_ref: object_ref::<E>,
            object_mut: object_mut::<E>,
            object_boxed: object_boxed::<E>,
            object_downcast: object_downcast::<E>,
            object_downcast_mut: object_downcast_mut::<E>,
            object_drop_rest: object_drop_front::<E>,
        };

        // Safety: passing vtable that operates on the right type E.
        let handler = Some(crate::capture_handler(&error, self.params));

        unsafe { Report::construct(error, vtable, handler) }
    }

    #[cfg_attr(track_caller, track_caller)]
    /// Creates a report from the following boxed error
    pub fn from_boxed(self, error: Box<dyn StdError + Send + Sync>) -> Report {
        use crate::wrapper::BoxedError;
        let error = BoxedError(error);
        let handler = Some(crate::capture_handler(&error, self.params));

        let vtable = &ErrorVTable {
            object_drop: object_drop::<BoxedError>,
            object_ref: object_ref::<BoxedError>,
            object_mut: object_mut::<BoxedError>,
            object_boxed: object_boxed::<BoxedError>,
            object_downcast: object_downcast::<Box<dyn StdError + Send + Sync>>,
            object_downcast_mut: object_downcast_mut::<Box<dyn StdError + Send + Sync>>,
            object_drop_rest: object_drop_front::<Box<dyn StdError + Send + Sync>>,
        };

        // Safety: BoxedError is repr(transparent) so it is okay for the vtable
        // to allow casting to Box<dyn StdError + Send + Sync>.
        unsafe { Report::construct(error, vtable, handler) }
    }

    #[cfg_attr(track_caller, track_caller)]
    /// Wraps a source error with a message
    pub fn wrap_with_msg<D, E>(self, msg: D, error: E) -> Report
    where
        D: Display + Send + Sync + 'static,
        E: StdError + Send + Sync + 'static,
    {
        let error: ContextError<D, E> = ContextError { msg, error };

        let vtable = &ErrorVTable {
            object_drop: object_drop::<ContextError<D, E>>,
            object_ref: object_ref::<ContextError<D, E>>,
            object_mut: object_mut::<ContextError<D, E>>,
            object_boxed: object_boxed::<ContextError<D, E>>,
            object_downcast: context_downcast::<D, E>,
            object_downcast_mut: context_downcast_mut::<D, E>,
            object_drop_rest: context_drop_rest::<D, E>,
        };

        // Safety: passing vtable that operates on the right type.
        let handler = Some(crate::capture_handler(&error, self.params));

        unsafe { Report::construct(error, vtable, handler) }
    }

    #[cfg_attr(track_caller, track_caller)]
    pub(crate) fn from_display<M>(self, message: M) -> Report
    where
        M: Display + Send + Sync + 'static,
    {
        use crate::wrapper::{DisplayError, NoneError};
        let error: DisplayError<M> = DisplayError(message);
        let vtable = &ErrorVTable {
            object_drop: object_drop::<DisplayError<M>>,
            object_ref: object_ref::<DisplayError<M>>,
            object_mut: object_mut::<DisplayError<M>>,
            object_boxed: object_boxed::<DisplayError<M>>,
            object_downcast: object_downcast::<M>,
            object_downcast_mut: object_downcast_mut::<M>,
            object_drop_rest: object_drop_front::<M>,
        };

        // Safety: DisplayError is repr(transparent) so it is okay for the
        // vtable to allow casting the DisplayError<M> to M.
        let handler = Some(crate::capture_handler(&NoneError, Default::default()));

        unsafe { Report::construct(error, vtable, handler) }
    }
}
