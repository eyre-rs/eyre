use eyre::Report;

fn fail(fail: bool) -> Result<(), Report> {
    let e: Report = eyre!("Internal error message");
    if fail {
        Err(e).wrap_err("External error message")
    } else {
        Ok(())
    }
}

fn main() {
    fail(true);
}
