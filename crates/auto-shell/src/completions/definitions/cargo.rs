//! Cargo completion specification
//!
//! Covers the most commonly used cargo subcommands, flags, and
//! project-aware argument sources.

use ash_core::completions::spec::*;

pub fn spec() -> CompletionSpec {
    CompletionSpec::new("cargo")
        .desc("Rust package manager")
        // Global flags
        .flag(FlagSpec::long("quiet").desc("Don't print to stdout"))
        .flag(FlagSpec::both("v", "verbose").desc("Use verbose output"))
        .flag(FlagSpec::long("color").desc("Colorize output").takes_arg("when"))
        .flag(FlagSpec::long("config").desc("Override config values").takes_arg("key=val"))
        .flag(FlagSpec::both("Z", "unstable-features").desc("Unstable nightly-only flags").takes_arg("flag"))
        .flag(FlagSpec::both("h", "help").desc("Print help"))
        .flag(FlagSpec::both("V", "version").desc("Print version"))
        // Subcommands
        .subcommand(build_spec())
        .subcommand(check_spec())
        .subcommand(clean_spec())
        .subcommand(clippy_spec())
        .subcommand(doc_spec())
        .subcommand(fmt_spec())
        .subcommand(init_spec())
        .subcommand(install_spec())
        .subcommand(new_spec())
        .subcommand(publish_spec())
        .subcommand(run_spec())
        .subcommand(test_spec())
        .subcommand(update_spec())
}

fn build_spec() -> SubcommandSpec {
    SubcommandSpec::new("build")
        .desc("Compile the current package")
        .flag(FlagSpec::long("release").desc("Build in release mode"))
        .flag(FlagSpec::long("target").desc("Build for target triple").takes_arg("triple"))
        .flag(FlagSpec::long("target-dir").desc("Target directory").takes_arg("dir"))
        .flag(FlagSpec::both("p", "package").desc("Package to build").takes_arg("spec"))
        .flag(FlagSpec::long("workspace").desc("Build all packages in workspace"))
        .flag(FlagSpec::both("j", "jobs").desc("Number of parallel jobs").takes_arg("n"))
        .flag(FlagSpec::long("features").desc("Space-separated list of features").takes_arg("features"))
        .flag(FlagSpec::long("all-features").desc("Activate all features"))
        .flag(FlagSpec::long("no-default-features").desc("Do not activate default features"))
}

fn check_spec() -> SubcommandSpec {
    SubcommandSpec::new("check")
        .desc("Check for errors without building")
        .flag(FlagSpec::long("release").desc("Check in release mode"))
        .flag(FlagSpec::long("target").desc("Check for target triple").takes_arg("triple"))
        .flag(FlagSpec::both("p", "package").desc("Package to check").takes_arg("spec"))
        .flag(FlagSpec::long("workspace").desc("Check all packages"))
        .flag(FlagSpec::both("j", "jobs").desc("Number of parallel jobs").takes_arg("n"))
        .flag(FlagSpec::long("features").desc("Space-separated list of features").takes_arg("features"))
        .flag(FlagSpec::long("all-features").desc("Activate all features"))
        .flag(FlagSpec::long("no-default-features").desc("Do not activate default features"))
}

fn clean_spec() -> SubcommandSpec {
    SubcommandSpec::new("clean")
        .desc("Remove generated artifacts")
        .flag(FlagSpec::long("release").desc("Clean release artifacts"))
        .flag(FlagSpec::long("target").desc("Clean for target triple").takes_arg("triple"))
        .flag(FlagSpec::long("target-dir").desc("Target directory").takes_arg("dir"))
        .flag(FlagSpec::both("p", "package").desc("Package to clean").takes_arg("spec"))
}

fn clippy_spec() -> SubcommandSpec {
    SubcommandSpec::new("clippy")
        .desc("Run Clippy lints")
        .flag(FlagSpec::long("release").desc("Check in release mode"))
        .flag(FlagSpec::long("target").desc("Check for target triple").takes_arg("triple"))
        .flag(FlagSpec::both("p", "package").desc("Package to check").takes_arg("spec"))
        .flag(FlagSpec::long("workspace").desc("Check all packages"))
        .flag(FlagSpec::long("fix").desc("Automatically apply lint suggestions"))
        .flag(FlagSpec::long("features").desc("Space-separated list of features").takes_arg("features"))
        .flag(FlagSpec::long("all-features").desc("Activate all features"))
        .flag(FlagSpec::long("no-default-features").desc("Do not activate default features"))
}

fn doc_spec() -> SubcommandSpec {
    SubcommandSpec::new("doc")
        .desc("Build documentation")
        .flag(FlagSpec::long("open").desc("Open docs in browser after build"))
        .flag(FlagSpec::long("release").desc("Build docs in release mode"))
        .flag(FlagSpec::long("no-deps").desc("Don't build docs for dependencies"))
        .flag(FlagSpec::both("p", "package").desc("Package to document").takes_arg("spec"))
        .flag(FlagSpec::long("workspace").desc("Document all packages"))
}

fn fmt_spec() -> SubcommandSpec {
    SubcommandSpec::new("fmt")
        .desc("Format source code with rustfmt")
        .flag(FlagSpec::long("check").desc("Check formatting without changing files"))
        .flag(FlagSpec::long("all").desc("Format all packages"))
        .flag(FlagSpec::both("p", "package").desc("Package to format").takes_arg("spec"))
}

fn init_spec() -> SubcommandSpec {
    SubcommandSpec::new("init")
        .desc("Create a new Cargo package in current directory")
        .flag(FlagSpec::long("name").desc("Package name").takes_arg("name"))
        .flag(FlagSpec::long("edition").desc("Rust edition").takes_arg("year"))
        .flag(FlagSpec::long("vcs").desc("VCS to use (git/hg/pijul/fossil/none)").takes_arg("vcs"))
        .arg(ArgSpec::new(0).desc("Path").source(CompletionSource::Directories))
}

fn install_spec() -> SubcommandSpec {
    SubcommandSpec::new("install")
        .desc("Install a Rust binary")
        .flag(FlagSpec::long("version").desc("Specify version").takes_arg("version"))
        .flag(FlagSpec::long("git").desc("Git URL to install from").takes_arg("url"))
        .flag(FlagSpec::long("path").desc("Local path to install from").takes_arg("path"))
        .flag(FlagSpec::long("force").desc("Force overwrite existing install"))
        .flag(FlagSpec::long("list").desc("List all installed packages"))
        .arg(ArgSpec::new(0).desc("Crate name"))
}

fn new_spec() -> SubcommandSpec {
    SubcommandSpec::new("new")
        .desc("Create a new Cargo package")
        .flag(FlagSpec::long("name").desc("Package name").takes_arg("name"))
        .flag(FlagSpec::long("edition").desc("Rust edition").takes_arg("year"))
        .flag(FlagSpec::long("vcs").desc("VCS to use (git/hg/pijul/fossil/none)").takes_arg("vcs"))
        .flag(FlagSpec::long("lib").desc("Create a library package"))
        .arg(ArgSpec::new(0).desc("Path").source(CompletionSource::Directories))
}

fn publish_spec() -> SubcommandSpec {
    SubcommandSpec::new("publish")
        .desc("Upload a package to the registry")
        .flag(FlagSpec::long("dry-run").desc("Perform all checks without uploading"))
        .flag(FlagSpec::long("allow-dirty").desc("Allow working tree with uncommitted changes"))
        .flag(FlagSpec::both("p", "package").desc("Package to publish").takes_arg("spec"))
        .flag(FlagSpec::long("index").desc("Registry index to publish to").takes_arg("index"))
}

fn run_spec() -> SubcommandSpec {
    SubcommandSpec::new("run")
        .desc("Run the current package's binary")
        .flag(FlagSpec::long("release").desc("Run in release mode"))
        .flag(FlagSpec::long("target").desc("Run for target triple").takes_arg("triple"))
        .flag(FlagSpec::both("p", "package").desc("Package to run").takes_arg("spec"))
        .flag(FlagSpec::long("example").desc("Run an example").takes_arg("name"))
        .flag(FlagSpec::long("features").desc("Space-separated list of features").takes_arg("features"))
        .flag(FlagSpec::long("all-features").desc("Activate all features"))
        .flag(FlagSpec::long("no-default-features").desc("Do not activate default features"))
}

fn test_spec() -> SubcommandSpec {
    SubcommandSpec::new("test")
        .desc("Run tests")
        .flag(FlagSpec::long("release").desc("Test in release mode"))
        .flag(FlagSpec::long("target").desc("Test for target triple").takes_arg("triple"))
        .flag(FlagSpec::both("p", "package").desc("Package to test").takes_arg("spec"))
        .flag(FlagSpec::long("workspace").desc("Test all packages"))
        .flag(FlagSpec::both("j", "jobs").desc("Number of parallel jobs").takes_arg("n"))
        .flag(FlagSpec::long("features").desc("Space-separated list of features").takes_arg("features"))
        .flag(FlagSpec::long("all-features").desc("Activate all features"))
        .flag(FlagSpec::long("no-default-features").desc("Do not activate default features"))
        .flag(FlagSpec::long("no-run").desc("Compile but don't run tests"))
        .flag(FlagSpec::long("doc").desc("Test documentation"))
        .arg(ArgSpec::any().desc("Test filter"))
}

fn update_spec() -> SubcommandSpec {
    SubcommandSpec::new("update")
        .desc("Update dependencies")
        .flag(FlagSpec::both("p", "package").desc("Package to update").takes_arg("spec"))
        .flag(FlagSpec::long("precise").desc("Update to exact version").takes_arg("version"))
        .flag(FlagSpec::long("dry-run").desc("Don't write lockfile"))
        .flag(FlagSpec::long("workspace").desc("Update all packages"))
}
