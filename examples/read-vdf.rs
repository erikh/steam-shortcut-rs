use steam_shortcut::parser::Parser;

fn main() -> Result<(), std::io::Error> {
    let mut parser = Parser::new("shortcuts.vdf")?;
    while let Some(item) = parser.next() {
        println!("{:?}", item);
    }
    Ok(())
}
