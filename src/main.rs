fn main() -> anyhow::Result<()> {
    fluent::run(&mut std::io::stdin(), &mut std::io::stdout())?;
    Ok(())
}
