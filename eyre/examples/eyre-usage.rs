use eyre::{Report, ResultExt, report};

fn main() -> Result<(), Report> {
    let e: Report = report!("oh no this program is just bad!");

    Err(e).wrap_err("usage example successfully experienced a failure")
}
