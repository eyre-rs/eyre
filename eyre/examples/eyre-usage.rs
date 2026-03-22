use eyre::{report, Report, WrapErr};

fn main() -> Result<(), Report> {
    let e: Report = report!("oh no this program is just bad!");

    Err(e).wrap_err("usage example successfully experienced a failure")
}
