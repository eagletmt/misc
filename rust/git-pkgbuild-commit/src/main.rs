fn main() -> anyhow::Result<()> {
    let repo = gix::discover(".")?;
    let index = repo.index()?;
    let Some(entry) = index.entry_by_path(".SRCINFO".into()) else {
        anyhow::bail!(".SRCINFO does not exist");
    };
    let object = repo.find_object(entry.id)?;

    if !is_staged(&repo, &object)? {
        println!(".SRCINFO is not staged");
        return Ok(());
    }

    let blob = object.into_blob();
    let srcinfo = String::from_utf8_lossy(&blob.data);
    let Some(pkgver) = srcinfo.lines().find_map(|line| {
        let line = line.trim_start();
        const PKGVER_PREFIX: &str = "pkgver = ";
        line.starts_with(PKGVER_PREFIX)
            .then(|| &line[PKGVER_PREFIX.len()..])
    }) else {
        anyhow::bail!("Cannot find pkgver from .SRCINFO");
    };

    let status = std::process::Command::new("git")
        .args(["commit", "-m"])
        .arg(format!("Update to v{}", pkgver))
        .status()?;
    anyhow::ensure!(status.success(), "git-commit failed");

    Ok(())
}

fn is_staged(repo: &gix::Repository, object: &gix::Object) -> anyhow::Result<bool> {
    let tree = repo.head_commit()?.tree()?;
    if let Some(entry) = tree.find_entry(".SRCINFO") {
        Ok(object.id != entry.oid())
    } else {
        // .SRCINFO is new blob
        Ok(true)
    }
}
