use std::fmt::{self, Display};

pub(crate) struct HeaderWriter<'a, H, W> {
    pub(crate) inner: W,
    pub(crate) header: &'a H,
    pub(crate) started: bool,
}

pub(crate) struct ReadyHeaderWriter<'a, 'b, H, W>(&'b mut HeaderWriter<'a, H, W>);

impl<'a, H, W> HeaderWriter<'a, H, W> {
    pub(crate) fn ready(&mut self) -> ReadyHeaderWriter<'a, '_, H, W> {
        self.started = false;

        ReadyHeaderWriter(self)
    }
}

impl<H, W> fmt::Write for ReadyHeaderWriter<'_, '_, H, W>
where
    H: Display,
    W: fmt::Write,
{
    fn write_str(&mut self, s: &str) -> fmt::Result {
        if !self.0.started && !s.is_empty() {
            self.0.inner.write_fmt(format_args!("{}", self.0.header))?;
            self.0.started = true;
        }

        self.0.inner.write_str(s)
    }
}
