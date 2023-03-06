fn main() -> Result<(), Box<dyn std::error::Error>> {
    for arg in std::env::args().skip(1) {
        let code = std::fs::read_to_string(&arg)?;
        let expr = jrsonnet_parser::parse(
            &code,
            &jrsonnet_parser::ParserSettings {
                source: jrsonnet_parser::Source::new(
                    jrsonnet_parser::SourcePath::new(jrsonnet_parser::SourceFile::new(arg.into())),
                    "".into(),
                ),
            },
        )?;
        let unused_variables = jrsonnet_lint::find_unused_variables(&expr);
        for variable in unused_variables {
            println!(
                "{}:{}:{} is defined but unused",
                variable.path.display(),
                variable.begin_offset_line()?,
                variable.name
            );
        }
    }
    Ok(())
}
