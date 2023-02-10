use ructe::{Result, Ructe};

fn main() -> Result<()> {
    // Compile our templates
    ructe()?;

    Ok(())
}

fn ructe() -> Result<()> {
    // Compile our templates
    let mut ructe = Ructe::from_env().unwrap();
    let mut statics = ructe.statics().unwrap();

    statics.add_files("./dist").unwrap();
    ructe.compile_templates("./dist").unwrap();

    Ok(())
}
