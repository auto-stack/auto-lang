//! Environment variable command completion specification (Plan 309 Task 1.2 P6).

use ash_core::completions::spec::*;

pub fn spec() -> CompletionSpec {
    CompletionSpec::new("env")
        .desc("Manage environment variables")
        .subcommand(SubcommandSpec::new("list").desc("List all environment variables"))
        // No subcommands — env takes flags or KEY=VALUE directly.
        // We can't easily complete variable names (they're dynamic), but
        // we can complete the known flags.
        .flag(FlagSpec::long("save").desc("Save a variable to ~/.config/ash/env.at").takes_arg("NAME"))
        .flag(FlagSpec::long("load").desc("Load persisted environment from env.at"))
        .flag(FlagSpec::long("rm").desc("Remove an environment variable").takes_arg("NAME"))
        .arg(ArgSpec::new(0).desc("KEY=VALUE to set, or KEY to query"))
}

pub fn path_spec() -> CompletionSpec {
    CompletionSpec::new("env.path")
        .desc("Manage PATH entries")
        .subcommand(SubcommandSpec::new("add").desc("Append to PATH").arg(ArgSpec::new(0).desc("Path to append").source(CompletionSource::Directories)))
        .subcommand(SubcommandSpec::new("pre").desc("Prepend to PATH").arg(ArgSpec::new(0).desc("Path to prepend").source(CompletionSource::Directories)))
        .subcommand(SubcommandSpec::new("rm").desc("Remove from PATH").arg(ArgSpec::new(0).desc("Path to remove").source(CompletionSource::Directories)))
        .subcommand(SubcommandSpec::new("rm-index").desc("Remove PATH entry by index").arg(ArgSpec::new(0).desc("Index (0-based)")))
        .subcommand(SubcommandSpec::new("move").desc("Move PATH entry").arg(ArgSpec::new(0).desc("From index")).arg(ArgSpec::new(1).desc("To index")))
        .subcommand(SubcommandSpec::new("dedup").desc("Remove duplicate PATH entries"))
        .subcommand(SubcommandSpec::new("clean").desc("Remove non-existent PATH entries"))
        .subcommand(SubcommandSpec::new("list").desc("List PATH entries"))
}
