#[cfg(not(feature = "cli"))]
fn main() {}

#[cfg(feature = "cli")]
use clap::Clap;
use ingreedy_rs::Ingredient;

#[cfg(feature = "cli")]
#[derive(Clap, Debug)]
#[clap(name = "ingreedy")]
struct Ingreedy {
    input: String,
}
#[cfg(feature = "cli")]
fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let ingreedy = Ingreedy::parse();
    let ingredient = Ingredient::parse(&ingreedy.input)?;
    println!("{}", serde_json::to_string_pretty(&ingredient)?);
    Ok(())
}
