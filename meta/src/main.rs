mod config;
mod editor;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use config::MetaConfig;
use editor::CrateEditor;
use glob::glob;
use semver::Version;
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use toml_edit::DocumentMut;

#[derive(Parser)]
#[command(name = "meta")]
#[command(about = "Manage a meta-workspace of Rust crates", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Bump the version of all crates in the meta-workspace
    Bump {
        /// The new version to set (e.g. "0.2.0")
        version: Version,
    },
    /// Initialize a new Meta.toml by scanning the current directory
    Init,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Bump { version } => bump_all(version),
        Commands::Init => generate_meta(),
    }
}

fn generate_meta() -> Result<()> {
    let current_dir = std::env::current_dir()?;
    generate_meta_at(&current_dir)
}

fn generate_meta_at(current_dir: &Path) -> Result<()> {
    // 1. Scan subdirectories
    let mut members = Vec::new();

    println!("Scanning {} for crates...", current_dir.display());

    for entry in fs::read_dir(current_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            let cargo_toml_path = path.join("Cargo.toml");
            if cargo_toml_path.exists() {
                process_crate_or_workspace(&mut members, current_dir, &path, &cargo_toml_path)?;
            }
        }
    }

    // sort members
    members.sort();

    // dedup members
    members.dedup();

    println!("Found {} members: {:?}", members.len(), members);

    if members.is_empty() {
        println!("No crates found. Exiting.");
        return Ok(());
    }

    // 2. Write Meta.toml
    let meta_path = current_dir.join("Meta.toml");
    if meta_path.exists() {
        // For safety, let's not overwrite if it exists without asking (or just fail for now)
        // User requested "generate an initial version", usually implies fresh start.
        // I will fail if exists to be safe.
        anyhow::bail!(
            "Meta.toml already exists. Please delete it or rename it before running init."
        );
    }

    // Create config structure manually or just write toml string
    let mut doc = DocumentMut::new();
    doc["workspace"] = toml_edit::table();

    let mut members_array = toml_edit::Array::new();
    for member in members {
        members_array.push(member);
    }

    doc["workspace"]["members"] = toml_edit::value(members_array);

    fs::write(meta_path, doc.to_string())?;
    println!("Generated Meta.toml successfully.");

    Ok(())
}

fn process_crate_or_workspace(
    members: &mut Vec<String>,
    root_path: &Path,
    dir_path: &Path,
    cargo_toml_path: &Path,
) -> Result<()> {
    let content = fs::read_to_string(cargo_toml_path)?;
    let doc = content.parse::<DocumentMut>()?;

    // Check if it is a workspace
    if let Some(workspace) = doc.get("workspace") {
        if let Some(ws_members) = workspace.get("members").and_then(|m| m.as_array()) {
            for member in ws_members {
                if let Some(member_str) = member.as_str() {
                    // Resolve glob
                    let pattern = dir_path.join(member_str);
                    let pattern_str = pattern.to_string_lossy();

                    for entry in glob(&pattern_str)? {
                        match entry {
                            Ok(p) => {
                                // verify it has a Cargo.toml
                                if p.join("Cargo.toml").exists() {
                                    // Add relative path from root_path
                                    if let Ok(rel) = p.strip_prefix(root_path) {
                                        members.push(rel.to_string_lossy().replace("\\", "/"));
                                    }
                                }
                            }
                            Err(e) => eprintln!("Glob error: {:?}", e),
                        }
                    }
                }
            }
        }
    } else if doc.get("package").is_some() {
        // It's a single crate
        if let Ok(rel) = dir_path.strip_prefix(root_path) {
            members.push(rel.to_string_lossy().replace("\\", "/"));
        }
    }

    Ok(())
}

fn bump_all(new_version: &Version) -> Result<()> {
    let config = MetaConfig::load()?;
    let mut editors = Vec::new();

    println!("Loading workspace members...");
    for member_path in &config.workspace.members {
        let path = Path::new(member_path);
        let editor = CrateEditor::new(path)
            .with_context(|| format!("Failed to load member at {}", member_path))?;
        editors.push(editor);
    }

    // Collect all package names to know which dependencies to update
    let member_names: HashSet<String> = editors
        .iter()
        .filter_map(|e| e.get_package_name())
        .collect();

    println!("Found {} members: {:?}", member_names.len(), member_names);

    for editor in &mut editors {
        let name = editor.get_package_name().unwrap_or_default();
        println!("Updating {}...", name);

        editor.bump_version(new_version)?;

        // Convert HashSet to Vec for the API I designed in editor.rs (oops, I designed it as slice, so strict ref is okay)
        // Actually editor.rs takes &[String]. HashSet doesn't blindly turn into slice.
        // I should update editor.rs or just collect here.
        // Let's collect to a sorted vec for stability or just iterate.
        let member_names_vec: Vec<String> = member_names.iter().cloned().collect();
        editor.update_dependencies(&member_names_vec, new_version)?;

        editor.save()?;
    }

    println!("Successfully bumped all crates to {}", new_version);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_workspace_integration() -> Result<()> {
        let temp_dir = tempdir()?;
        let workspace_root = temp_dir.path();

        // Create workspace structure programmatically
        fs::write(
            workspace_root.join("Meta.toml"),
            r#"[workspace]
members = [
    "crate_a",
    "crate_b"
]
"#,
        )?;

        fs::create_dir(workspace_root.join("crate_a"))?;
        fs::write(
            workspace_root.join("crate_a/Cargo.toml"),
            r#"[package]
name = "crate_a"
version = "0.1.0"
edition = "2021"

[dependencies]
"#,
        )?;

        fs::create_dir(workspace_root.join("crate_b"))?;
        fs::write(
            workspace_root.join("crate_b/Cargo.toml"),
            r#"[package]
name = "crate_b"
version = "0.1.0"
edition = "2021"

[dependencies]
crate_a = { git = "https://github.com/foo/crate_a", tag = "v0.1.0" }
start-up = "1.0"
"#,
        )?;

        // Change current directory to temp_dir so MetaConfig::load() finds Meta.toml
        // But changing CWD in test is dangerous for parallel tests.
        // Instead, we should probably refactor `MetaConfig::load` to accept a path?
        // Or refactor `bump_all` to take a config?

        // Let's refactor `MetaConfig::load` to take an optional path or just make a private load_from_path

        // Actually, for this test, let's just use `bump_all` logic inline or modify `bump_all`.
        // `bump_all` calls `MetaConfig::load()`.

        // Refactoring `MetaConfig::load` is the cleanest way.
        // But for now, to avoid changing too much code, I can manually verify the steps in the test
        // by loading config manually and calling editors.

        let config_path = workspace_root.join("Meta.toml");
        let content = fs::read_to_string(&config_path)?;
        let config: MetaConfig = toml_edit::de::from_str(&content)?;

        let new_version = Version::parse("0.2.0")?;
        let mut editors = Vec::new();

        for member_path in &config.workspace.members {
            let path = workspace_root.join(member_path);
            let editor = CrateEditor::new(&path)?;
            editors.push(editor);
        }

        let member_names: HashSet<String> = editors
            .iter()
            .filter_map(|e| e.get_package_name())
            .collect();

        for editor in &mut editors {
            editor.bump_version(&new_version)?;
            let member_names_vec: Vec<String> = member_names.iter().cloned().collect();
            editor.update_dependencies(&member_names_vec, &new_version)?;
            editor.save()?;
        }

        // Verify crate_a
        let crate_a_toml = fs::read_to_string(workspace_root.join("crate_a/Cargo.toml"))?;
        assert!(crate_a_toml.contains(r#"version = "0.2.0""#));

        // Verify crate_b
        let crate_b_toml = fs::read_to_string(workspace_root.join("crate_b/Cargo.toml"))?;
        assert!(crate_b_toml.contains(r#"version = "0.2.0""#));
        // Verify dependency update
        assert!(
            crate_b_toml.contains(
                r#"crate_a = { git = "https://github.com/foo/crate_a", tag = "v0.2.0" }"#
            )
        );

        Ok(())
    }

    #[test]
    fn test_init_command() -> Result<()> {
        let temp_dir = tempdir()?;
        let workspace_root = temp_dir.path();

        // Create dummy crates
        fs::create_dir(workspace_root.join("crate_x"))?;
        fs::write(
            workspace_root.join("crate_x/Cargo.toml"),
            r#"[package]
name = "crate_x"
version = "0.1.0"
"#,
        )?;

        fs::create_dir(workspace_root.join("crate_y"))?;
        fs::write(
            workspace_root.join("crate_y/Cargo.toml"),
            r#"[package]
name = "crate_y"
version = "0.1.0"
"#,
        )?;

        // We can't really call `generate_meta` directly because it relies on `std::env::current_dir()`.
        // To test it, we either need to change CWD (unsafe in multithreaded tests)
        // or refactor `generate_meta` to take a path.
        // Given I already wrote `generate_meta` to use `current_dir`, I should refactor it slightly to perform the core logic on a path.
        // But for time being, I can't easily change CWD.
        // Let's refactor `generate_meta` to `generate_meta_at(path: &Path)`.

        generate_meta_at(workspace_root)?;

        let meta_toml_path = workspace_root.join("Meta.toml");
        assert!(meta_toml_path.exists());

        let content = fs::read_to_string(meta_toml_path)?;
        assert!(content.contains(r#""crate_x""#));
        assert!(content.contains(r#""crate_y""#));

        Ok(())
    }
    #[test]
    #[ignore]
    fn generate_manual_workspace() -> Result<()> {
        let root = std::env::current_dir()?.join("tests_workspace");
        if root.exists() {
            fs::remove_dir_all(&root)?;
        }
        fs::create_dir(&root)?;

        fs::write(
            root.join("Meta.toml"),
            r#"[workspace]
members = [
    "crate_a",
    "crate_b"
]
"#,
        )?;

        fs::create_dir(root.join("crate_a"))?;
        fs::write(
            root.join("crate_a/Cargo.toml"),
            r#"[package]
name = "crate_a"
version = "0.1.0"
edition = "2021"

[dependencies]
"#,
        )?;

        fs::create_dir(root.join("crate_b"))?;
        fs::write(
            root.join("crate_b/Cargo.toml"),
            r#"[package]
name = "crate_b"
version = "0.1.0"
edition = "2021"

[dependencies]
crate_a = {  git = "https://github.com/foo/crate_a", tag = "v0.1.0" }
start-up = "1.0"
"#,
        )?;

        println!("Created tests_workspace at {}", root.display());
        Ok(())
    }
}
