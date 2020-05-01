use console::style;
use std::env;
use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader, ErrorKind};
use std::sync::atomic::{AtomicUsize, Ordering::SeqCst};
use tracing_error::SpanTrace;

pub fn colorize(span_trace: &SpanTrace) -> impl fmt::Display + '_ {
    ColorSpanTrace { span_trace }
}

struct ColorSpanTrace<'a> {
    span_trace: &'a SpanTrace,
}

macro_rules! try_bool {
    ($e:expr, $dest:ident) => {{
        let ret = $e.unwrap_or_else(|e| $dest = Err(e));

        if $dest.is_err() {
            return false;
        }

        ret
    }};
}

struct Frame<'a> {
    metadata: &'a tracing_core::Metadata<'static>,
    fields: &'a str,
}

fn enabled() -> bool {
    // Cache the result of reading the environment variables to make
    // backtrace captures speedy, because otherwise reading environment
    // variables every time can be somewhat slow.
    static ENABLED: AtomicUsize = AtomicUsize::new(0);
    match ENABLED.load(SeqCst) {
        0 => {}
        1 => return false,
        _ => return true,
    }
    let enabled = match env::var("RUST_LIB_BACKTRACE") {
        Ok(s) => s != "0",
        Err(_) => match env::var("RUST_BACKTRACE") {
            Ok(s) => s != "0",
            Err(_) => false,
        },
    };
    ENABLED.store(enabled as usize + 1, SeqCst);
    return enabled;
}

impl Frame<'_> {
    fn print(&self, i: u32, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.print_header(i, f)?;
        self.print_fields(f)?;
        self.print_source_location(f)?;
        Ok(())
    }

    fn print_header(&self, i: u32, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:>2}: {}",
            i,
            style(format_args!(
                "{}::{}",
                self.metadata.target(),
                self.metadata.name()
            ))
            .red()
        )
    }

    fn print_fields(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !self.fields.is_empty() {
            write!(f, " with {}", style(self.fields).cyan())?;
        }

        Ok(())
    }

    fn print_source_location(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(file) = self.metadata.file() {
            let lineno = self
                .metadata
                .line()
                .map_or("<unknown line>".to_owned(), |x| x.to_string());
            write!(f, "\n    at {}:{}", file, lineno)?;
        } else {
            write!(f, "\n    at <unknown source file>")?;
        }

        Ok(())
    }

    fn print_source_if_avail(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (lineno, filename) = match (self.metadata.line(), self.metadata.file()) {
            (Some(a), Some(b)) => (a, b),
            // Without a line number and file name, we can't sensibly proceed.
            _ => return Ok(()),
        };

        let file = match File::open(filename) {
            Ok(file) => file,
            Err(ref e) if e.kind() == ErrorKind::NotFound => return Ok(()),
            e @ Err(_) => e.unwrap(),
        };

        // Extract relevant lines.
        let reader = BufReader::new(file);
        let start_line = lineno - 2.min(lineno - 1);
        let surrounding_src = reader.lines().skip(start_line as usize - 1).take(5);
        for (line, cur_line_no) in surrounding_src.zip(start_line..) {
            if cur_line_no == lineno {
                write!(
                    f,
                    "\n{}",
                    style(format_args!("{:>8} > {}", cur_line_no, line.unwrap())).bold()
                )?;
            } else {
                write!(f, "\n{:>8} │ {}", cur_line_no, line.unwrap())?;
            }
        }

        Ok(())
    }
}

impl fmt::Display for ColorSpanTrace<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut err = Ok(());
        let mut span = 0;

        writeln!(f, "{:━^80}\n", " SPANTRACE ")?;
        self.span_trace.with_spans(|metadata, fields| {
            let frame = Frame { metadata, fields };

            if span > 0 {
                try_bool!(write!(f, "\n",), err);
            }

            try_bool!(frame.print(span, f), err);

            if enabled() {
                try_bool!(frame.print_source_if_avail(f), err);
            }

            span += 1;
            true
        });

        err
    }
}
