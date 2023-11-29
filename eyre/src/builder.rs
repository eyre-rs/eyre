use crate::{
    backtrace::Backtrace,
    vtable::{
        object_boxed, object_downcast, object_downcast_mut, object_drop, object_drop_front,
        object_mut, object_ref, ErrorVTable,
    },
    Report, StdError,
};
use std::fmt::Display;

#[derive(Default)]
pub struct ReportBuilder {
    backtrace: Option<Backtrace>,
}

impl std::fmt::Debug for ReportBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl ReportBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Use the given backtrace for the error
    pub fn with_backtrace(mut self, backtrace: Option<Backtrace>) -> Self {
        self.backtrace = backtrace;
        self
    }

    #[cfg_attr(track_caller, track_caller)]
    /// Creates a report from the given error message
    pub fn msg<M>(self, message: M) -> Report
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
        let handler = Some(crate::capture_handler(&error));

        unsafe { Report::construct(error, vtable, handler) }
    }

    #[cfg_attr(track_caller, track_caller)]
    /// Creates a report from the following error
    pub fn report<E>(self, error: E) -> Report
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
        let handler = Some(crate::capture_handler(&error));

        unsafe { Report::construct(error, vtable, handler) }
    }
}
