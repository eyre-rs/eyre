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
        let final_report = eyre!("Program failure with {} errors!", errors.len());

        let err = errors
            .into_iter()
            .fold(final_report, |report, r| report.with_report(|| r));

        return Err(err);
    }

    Ok(())
}

fn do_stuff() -> Result<()> {
    let parameters = (1, 2, 3);
    let err = eyre!(
        "Some thing gets wrong with parameters: \n-x: {} \n-y: {} \n-z: {}",
        parameters.0,
        parameters.1,
        parameters.2,
    );

    Err(err)
}

fn do_other_stuff() -> Result<()> {
    let err = eyre!("Some other thing gets wrong");

    Err(err)
}
