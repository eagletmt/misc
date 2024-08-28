#[derive(Debug, clap::Parser)]
struct Args {
    dir: std::path::PathBuf,
}

type ImportedSet = std::cell::RefCell<std::collections::HashSet<std::path::PathBuf>>;

fn main() -> anyhow::Result<()> {
    use clap::Parser as _;
    let args = Args::parse();
    let cwd = std::env::current_dir()?;

    let imported = ImportedSet::default();
    let mut found_libsonnet = std::collections::HashSet::new();

    for entry in walkdir::WalkDir::new(args.dir) {
        let entry = entry?;
        if entry.file_type().is_file()
            && entry.path().extension().is_some_and(|ext| ext == "jsonnet")
        {
            let state = create_state(imported.clone());
            if let Err(e) = state
                .import(entry.path())
                .and_then(|val| val.manifest(jrsonnet_evaluator::manifest::JsonFormat::minify()))
            {
                anyhow::bail!("{}: {}", entry.path().display(), e);
            }
        } else if entry.file_type().is_file()
            && entry
                .path()
                .extension()
                .is_some_and(|ext| ext == "libsonnet")
        {
            found_libsonnet.insert(entry.path().to_path_buf());
        }
    }

    let imported = imported.into_inner();
    let mut c = 0;
    for relative_path in found_libsonnet {
        let absolute_path = cwd.join(&relative_path);
        if !imported.contains(&absolute_path) {
            println!("{}", relative_path.display());
            c += 1;
        }
    }

    if c > 0 {
        std::process::exit(c);
    }
    Ok(())
}

#[jrsonnet_evaluator::function::builtin]
fn provide_vault(_options: jrsonnet_evaluator::Val, _name: jrsonnet_evaluator::Val) -> String {
    "fake".to_owned()
}

fn create_state(imported: ImportedSet) -> jrsonnet_evaluator::State {
    let state = jrsonnet_evaluator::State::default();
    state.set_import_resolver(ImportResolver::new(imported));

    let context_initializer = jrsonnet_stdlib::ContextInitializer::new(
        state.clone(),
        jrsonnet_evaluator::trace::PathResolver::new_cwd_fallback(),
    );
    context_initializer.add_native("provide.vault", provide_vault::INST);
    context_initializer.add_ext_var("appId".into(), "fake".into());
    state.set_context_initializer(context_initializer);

    state
}

struct ImportResolver {
    inner: jrsonnet_evaluator::FileImportResolver,
    imported: ImportedSet,
}

impl ImportResolver {
    fn new(imported: ImportedSet) -> Self {
        Self {
            inner: jrsonnet_evaluator::FileImportResolver::default(),
            imported,
        }
    }
}

impl jrsonnet_gcmodule::Trace for ImportResolver {}

impl jrsonnet_evaluator::ImportResolver for ImportResolver {
    fn load_file_contents(
        &self,
        resolved: &jrsonnet_parser::SourcePath,
    ) -> jrsonnet_evaluator::Result<Vec<u8>> {
        if let Some(path) = resolved.path() {
            self.imported.borrow_mut().insert(path.to_path_buf());
        }
        self.inner.load_file_contents(resolved)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn resolve(
        &self,
        path: &std::path::Path,
    ) -> jrsonnet_evaluator::Result<jrsonnet_parser::SourcePath> {
        self.inner.resolve(path)
    }

    fn resolve_from(
        &self,
        from: &jrsonnet_parser::SourcePath,
        path: &str,
    ) -> jrsonnet_evaluator::Result<jrsonnet_parser::SourcePath> {
        self.inner.resolve_from(from, path)
    }
}
