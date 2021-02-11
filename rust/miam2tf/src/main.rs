fn main() -> Result<(), anyhow::Error> {
    let miam = miam2tf::loader::load_miam("IAMfile")?;
    miam2tf::printer::print_as_hcl2(&mut std::io::stdout().lock(), &miam)?;
    Ok(())
}
