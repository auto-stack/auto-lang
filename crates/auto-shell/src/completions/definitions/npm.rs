//! npm completion specification

use ash_core::completions::spec::*;

pub fn spec() -> CompletionSpec {
    CompletionSpec::new("npm")
        .desc("Node.js package manager")
        .subcommand(install_spec())
        .subcommand(run_spec())
        .subcommand(init_spec())
        .subcommand(test_spec())
        .subcommand(publish_spec())
        .subcommand(update_spec())
        .subcommand(uninstall_spec())
        .subcommand(list_spec())
        .subcommand(start_spec())
        .subcommand(build_spec())
}

fn install_spec() -> SubcommandSpec {
    SubcommandSpec::new("install")
        .desc("Install dependencies")
        .flag(FlagSpec::long("save-dev").desc("Save to devDependencies"))
        .flag(FlagSpec::long("save-prod").desc("Save to dependencies"))
        .flag(FlagSpec::long("global").desc("Install globally"))
        .flag(FlagSpec::long("force").desc("Force install"))
        .flag(FlagSpec::long("dry-run").desc("Report without installing"))
        .arg(ArgSpec::new(0).repeat().desc("Package name"))
}

fn run_spec() -> SubcommandSpec {
    SubcommandSpec::new("run")
        .desc("Run a script defined in package.json")
        .arg(ArgSpec::new(0).desc("Script name").source(CompletionSource::command("node -e \"const p=require('./package.json');console.log(Object.keys(p.scripts||{}).join('\\n'))\"")))
}

fn init_spec() -> SubcommandSpec {
    SubcommandSpec::new("init")
        .desc("Create a package.json file")
        .flag(FlagSpec::both("y", "yes").desc("Use defaults"))
        .flag(FlagSpec::long("scope").desc("Scope for package").takes_arg("scope"))
}

fn test_spec() -> SubcommandSpec {
    SubcommandSpec::new("test")
        .desc("Run tests")
        .flag(FlagSpec::long("ignore-scripts").desc("Skip scripts"))
}

fn publish_spec() -> SubcommandSpec {
    SubcommandSpec::new("publish")
        .desc("Publish a package to the registry")
        .flag(FlagSpec::long("access").desc("Access level (public/restricted)").takes_arg("level"))
        .flag(FlagSpec::long("dry-run").desc("Report without publishing"))
        .flag(FlagSpec::long("tag").desc("Tag for the version").takes_arg("tag"))
}

fn update_spec() -> SubcommandSpec {
    SubcommandSpec::new("update")
        .desc("Update packages")
        .flag(FlagSpec::long("global").desc("Update global packages"))
        .flag(FlagSpec::long("save-dev").desc("Update devDependencies"))
        .arg(ArgSpec::new(0).repeat().desc("Package name"))
}

fn uninstall_spec() -> SubcommandSpec {
    SubcommandSpec::new("uninstall")
        .desc("Remove a package")
        .flag(FlagSpec::long("save-dev").desc("Remove from devDependencies"))
        .flag(FlagSpec::long("global").desc("Remove global package"))
        .arg(ArgSpec::new(0).repeat().desc("Package name"))
}

fn list_spec() -> SubcommandSpec {
    SubcommandSpec::new("list")
        .desc("List installed packages")
        .flag(FlagSpec::long("depth").desc("Max depth").takes_arg("n"))
        .flag(FlagSpec::long("global").desc("List global packages"))
        .flag(FlagSpec::both("l", "long").desc("Show extended info"))
}

fn start_spec() -> SubcommandSpec {
    SubcommandSpec::new("start")
        .desc("Start a package")
}

fn build_spec() -> SubcommandSpec {
    SubcommandSpec::new("run-build")
        .desc("Run the build script")
        .flag(FlagSpec::long("ignore-scripts").desc("Skip scripts"))
}
