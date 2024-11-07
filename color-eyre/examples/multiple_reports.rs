use color_eyre::Section;
use eyre::{eyre, Result};

fn main() -> Result<()> {
    color_eyre::install()?;

    let mut errors = vec![];

    if let Err(err) = do_stuff() {
        errors.push(err);
    }

    if let Err(err) = do_other_stuff() {
        errors.push(err);
    }

    if !errors.is_empty() {
        let mut err = eyre!("Program failure with {} errors!", errors.len());

        for error in errors {
            err = err.report(error);
        }

        return Err(err);
    }

    Ok(())
}

fn do_stuff() -> Result<()> {
    let err = eyre!("Some thing gets wrong");

    Err(err)
}

fn do_other_stuff() -> Result<()> {
    let err = eyre!("Some other thing gets wrong");

    Err(err)
}
