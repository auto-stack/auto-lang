//! Git completion specification
//!
//! Covers the most commonly used git subcommands, flags, and dynamic
//! argument sources (branches, remotes, modified files).

use ash_core::completions::spec::*;

pub fn spec() -> CompletionSpec {
    CompletionSpec::new("git")
        .desc("Git version control system")
        // ── Common subcommands ──────────────────────────────
        .subcommand(add_spec())
        .subcommand(branch_spec())
        .subcommand(checkout_spec())
        .subcommand(clone_spec())
        .subcommand(commit_spec())
        .subcommand(diff_spec())
        .subcommand(fetch_spec())
        .subcommand(log_spec())
        .subcommand(merge_spec())
        .subcommand(pull_spec())
        .subcommand(push_spec())
        .subcommand(rebase_spec())
        .subcommand(remote_spec())
        .subcommand(reset_spec())
        .subcommand(stash_spec())
        .subcommand(status_spec())
        .subcommand(switch_spec())
        .subcommand(tag_spec())
}

fn add_spec() -> SubcommandSpec {
    SubcommandSpec::new("add")
        .desc("Add file contents to the index")
        .flag(FlagSpec::both("A", "all").desc("Add changes from all tracked and untracked files"))
        .flag(FlagSpec::both("u", "update").desc("Update tracked files"))
        .flag(FlagSpec::both("f", "force").desc("Allow adding otherwise ignored files"))
        .flag(FlagSpec::both("n", "dry-run").desc("Don't actually add the file(s)"))
        .flag(FlagSpec::both("v", "verbose").desc("Be verbose"))
        .flag(FlagSpec::both("i", "interactive").desc("Interactive mode"))
        .flag(FlagSpec::both("p", "patch").desc("Interactive patch selection"))
        .arg(
            ArgSpec::any()
                .repeat()
                .desc("Files to stage")
                .source(CompletionSource::command_field("git status --porcelain", 1)),
        )
}

fn branch_spec() -> SubcommandSpec {
    SubcommandSpec::new("branch")
        .desc("List, create, or delete branches")
        .flag(FlagSpec::both("a", "all").desc("List both remote-tracking and local branches"))
        .flag(FlagSpec::both("d", "delete").desc("Delete a branch"))
        .flag(FlagSpec::long("delete-force").desc("Force delete a branch"))
        .flag(FlagSpec::both("m", "move").desc("Rename a branch"))
        .flag(FlagSpec::both("l", "list").desc("List branches"))
        .flag(FlagSpec::both("v", "verbose").desc("Show sha1 and commit subject"))
        .arg(
            ArgSpec::new(0)
                .desc("Branch name or pattern")
                .when(WhenCondition::flags_absent(&["d", "delete", "delete-force", "m", "move"]))
                .source(CompletionSource::command("git branch --list")),
        )
        .arg(
            ArgSpec::new(0)
                .desc("Branch to delete")
                .when(WhenCondition::flags_present(&["d", "delete"]))
                .source(CompletionSource::command("git branch --list")),
        )
}

fn checkout_spec() -> SubcommandSpec {
    SubcommandSpec::new("checkout")
        .desc("Switch branches or restore working tree files")
        .flag(FlagSpec::both("b", "branch").desc("Create a new branch").takes_arg("name"))
        .flag(FlagSpec::both("B", "branch-force").desc("Create/reset a new branch"))
        .flag(FlagSpec::both("f", "force").desc("Force checkout"))
        .flag(FlagSpec::both("t", "track").desc("Track a new branch"))
        .arg(
            ArgSpec::new(0)
                .desc("Branch to switch to")
                .when(WhenCondition::flags_absent(&["b", "B"]))
                .source(CompletionSource::command("git branch --list")),
        )
}

fn clone_spec() -> SubcommandSpec {
    SubcommandSpec::new("clone")
        .desc("Clone a repository into a new directory")
        .flag(FlagSpec::long("depth").desc("Create a shallow clone").takes_arg("depth"))
        .flag(FlagSpec::long("branch").desc("Checkout specific branch").takes_arg("branch"))
        .flag(FlagSpec::long("single-branch").desc("Clone only history leading to the tip"))
        .flag(FlagSpec::long("recurse-submodules").desc("Initialize submodules"))
        .flag(FlagSpec::both("q", "quiet").desc("Quiet mode"))
        .flag(FlagSpec::both("v", "verbose").desc("Verbose mode"))
        .arg(ArgSpec::new(0).desc("Repository URL"))
        .arg(ArgSpec::new(1).desc("Target directory").source(CompletionSource::Directories))
}

fn commit_spec() -> SubcommandSpec {
    SubcommandSpec::new("commit")
        .desc("Record changes to the repository")
        .flag(FlagSpec::both("m", "message").desc("Commit message").takes_arg("msg"))
        .flag(FlagSpec::both("a", "all").desc("Commit all changed files"))
        .flag(FlagSpec::both("e", "edit").desc("Edit commit message"))
        .flag(FlagSpec::both("n", "no-verify").desc("Bypass pre-commit and commit-msg hooks"))
        .flag(FlagSpec::long("amend").desc("Amend previous commit"))
        .flag(FlagSpec::long("no-edit").desc("Reuse existing commit message"))
        .flag(FlagSpec::both("s", "signoff").desc("Add Signed-off-by trailer"))
        .flag(FlagSpec::both("v", "verbose").desc("Show diff in commit message template"))
}

fn diff_spec() -> SubcommandSpec {
    SubcommandSpec::new("diff")
        .desc("Show changes between commits, commit and working tree, etc.")
        .flag(FlagSpec::both("a", "all").desc("Show diff of all files"))
        .flag(FlagSpec::long("cached").desc("Show diff of staged changes"))
        .flag(FlagSpec::long("staged").desc("Alias for --cached"))
        .flag(FlagSpec::both("s", "stat").desc("Show diffstat instead of patch"))
        .flag(FlagSpec::long("name-only").desc("Show only names of changed files"))
        .flag(FlagSpec::long("name-status").desc("Show names and status of changed files"))
        .arg(
            ArgSpec::new(0)
                .desc("Commit or branch to compare")
                .source(CompletionSource::command("git branch --list")),
        )
}

fn fetch_spec() -> SubcommandSpec {
    SubcommandSpec::new("fetch")
        .desc("Download objects and refs from another repository")
        .flag(FlagSpec::both("a", "all").desc("Fetch all remotes"))
        .flag(FlagSpec::long("prune").desc("Remove stale remote-tracking branches"))
        .flag(FlagSpec::long("dry-run").desc("Show what would be done"))
        .flag(FlagSpec::both("f", "force").desc("Force update"))
        .flag(FlagSpec::both("t", "tags").desc("Fetch all tags"))
        .arg(
            ArgSpec::new(0)
                .desc("Remote name")
                .source(CompletionSource::command("git remote")),
        )
}

fn log_spec() -> SubcommandSpec {
    SubcommandSpec::new("log")
        .desc("Show commit logs")
        .flag(FlagSpec::both("n", "number").desc("Limit number of commits").takes_arg("n"))
        .flag(FlagSpec::long("oneline").desc("Shorthand for --pretty=oneline --abbrev-commit"))
        .flag(FlagSpec::long("graph").desc("Show ASCII graph of branch history"))
        .flag(FlagSpec::both("p", "patch").desc("Show patch"))
        .flag(FlagSpec::long("decorate").desc("Decorate refs"))
        .flag(FlagSpec::long("follow").desc("Continue listing file history across renames"))
        .flag(FlagSpec::long("all").desc("Pretend as if all refs are listed"))
        .arg(
            ArgSpec::new(0)
                .desc("Revision or branch")
                .source(CompletionSource::command("git branch --list")),
        )
}

fn merge_spec() -> SubcommandSpec {
    SubcommandSpec::new("merge")
        .desc("Join two or more development histories together")
        .flag(FlagSpec::long("no-ff").desc("Create a merge commit even when fast-forward"))
        .flag(FlagSpec::long("ff-only").desc("Refuse to merge unless fast-forward"))
        .flag(FlagSpec::both("s", "strategy").desc("Merge strategy").takes_arg("strategy"))
        .flag(FlagSpec::long("abort").desc("Abort the current merge"))
        .flag(FlagSpec::long("continue").desc("Continue the current merge"))
        .arg(
            ArgSpec::new(0)
                .desc("Branch to merge")
                .source(CompletionSource::command("git branch --list")),
        )
}

fn pull_spec() -> SubcommandSpec {
    SubcommandSpec::new("pull")
        .desc("Fetch from and integrate with another repository")
        .flag(FlagSpec::long("rebase").desc("Rebase the current branch on top of upstream"))
        .flag(FlagSpec::long("no-rebase").desc("Do not rebase"))
        .flag(FlagSpec::long("ff-only").desc("Refuse to merge unless fast-forward"))
        .flag(FlagSpec::long("no-ff").desc("Create a merge commit"))
        .arg(
            ArgSpec::new(0)
                .desc("Remote name")
                .source(CompletionSource::command("git remote")),
        )
}

fn push_spec() -> SubcommandSpec {
    SubcommandSpec::new("push")
        .desc("Update remote refs along with associated objects")
        .flag(FlagSpec::both("f", "force").desc("Force push"))
        .flag(FlagSpec::long("force-with-lease").desc("Force push with safety check"))
        .flag(FlagSpec::both("u", "set-upstream").desc("Set upstream for the branch"))
        .flag(FlagSpec::long("dry-run").desc("Dry run"))
        .flag(FlagSpec::both("d", "delete").desc("Delete a remote branch"))
        .flag(FlagSpec::long("all").desc("Push all branches"))
        .flag(FlagSpec::long("tags").desc("Push tags"))
        .arg(
            ArgSpec::new(0)
                .desc("Remote name")
                .source(CompletionSource::command("git remote")),
        )
        .arg(
            ArgSpec::new(1)
                .desc("Branch name")
                .source(CompletionSource::command("git branch --list")),
        )
}

fn rebase_spec() -> SubcommandSpec {
    SubcommandSpec::new("rebase")
        .desc("Reapply commits on top of another base tip")
        .flag(FlagSpec::both("i", "interactive").desc("Interactive rebase"))
        .flag(FlagSpec::long("continue").desc("Continue after resolving conflicts"))
        .flag(FlagSpec::long("abort").desc("Abort the rebase"))
        .flag(FlagSpec::long("skip").desc("Skip current commit"))
        .flag(FlagSpec::long("onto").desc("Rebase onto specific commit").takes_arg("newbase"))
        .arg(
            ArgSpec::new(0)
                .desc("Upstream branch")
                .source(CompletionSource::command("git branch --list")),
        )
}

fn remote_spec() -> SubcommandSpec {
    SubcommandSpec::new("remote")
        .desc("Manage set of tracked repositories")
        .subcommand(
            SubcommandSpec::new("add")
                .desc("Add a remote")
                .flag(FlagSpec::both("f", "fetch").desc("Run git fetch after adding"))
                .flag(FlagSpec::long("tags").desc("Import tags"))
                .arg(ArgSpec::new(0).desc("Remote name"))
                .arg(ArgSpec::new(1).desc("Remote URL")),
        )
        .subcommand(
            SubcommandSpec::new("remove")
                .desc("Remove a remote")
                .arg(
                    ArgSpec::new(0)
                        .desc("Remote to remove")
                        .source(CompletionSource::command("git remote")),
                ),
        )
        .subcommand(
            SubcommandSpec::new("rename")
                .desc("Rename a remote")
                .arg(
                    ArgSpec::new(0)
                        .desc("Old name")
                        .source(CompletionSource::command("git remote")),
                )
                .arg(ArgSpec::new(1).desc("New name")),
        )
        .subcommand(
            SubcommandSpec::new("set-url")
                .desc("Change URL for a remote")
                .arg(
                    ArgSpec::new(0)
                        .desc("Remote name")
                        .source(CompletionSource::command("git remote")),
                )
                .flag(FlagSpec::long("push").desc("Manipulate push URLs instead of fetch"))
                .arg(ArgSpec::new(1).desc("New URL")),
        )
        .subcommand(
            SubcommandSpec::new("list")
                .desc("List remotes")
                .flag(FlagSpec::both("v", "verbose").desc("Show URLs")),
        )
}

fn reset_spec() -> SubcommandSpec {
    SubcommandSpec::new("reset")
        .desc("Reset current HEAD to the specified state")
        .flag(FlagSpec::long("soft").desc("Leave working tree and index untouched"))
        .flag(FlagSpec::long("mixed").desc("Reset index but leave working tree"))
        .flag(FlagSpec::long("hard").desc("Reset index and working tree"))
        .flag(FlagSpec::long("merge").desc("Reset for a failed merge"))
        .flag(FlagSpec::long("keep").desc("Reset but keep local changes"))
        .arg(
            ArgSpec::new(0)
                .desc("Commit or branch to reset to")
                .source(CompletionSource::command("git log --oneline -20")),
        )
}

fn stash_spec() -> SubcommandSpec {
    SubcommandSpec::new("stash")
        .desc("Stash the changes in a dirty working directory away")
        .subcommand(
            SubcommandSpec::new("list")
                .desc("List stashes"),
        )
        .subcommand(
            SubcommandSpec::new("show")
                .desc("Show stash contents")
                .flag(FlagSpec::both("p", "patch").desc("Show patch"))
                .arg(ArgSpec::new(0).desc("Stash index")),
        )
        .subcommand(
            SubcommandSpec::new("drop")
                .desc("Remove a stash")
                .arg(ArgSpec::new(0).desc("Stash index")),
        )
        .subcommand(
            SubcommandSpec::new("pop")
                .desc("Apply and remove a stash")
                .flag(FlagSpec::long("index").desc("Restore staged changes"))
                .arg(ArgSpec::new(0).desc("Stash index")),
        )
        .subcommand(
            SubcommandSpec::new("apply")
                .desc("Apply a stash")
                .flag(FlagSpec::long("index").desc("Restore staged changes"))
                .arg(ArgSpec::new(0).desc("Stash index")),
        )
        .subcommand(
            SubcommandSpec::new("branch")
                .desc("Create branch from stash")
                .arg(ArgSpec::new(0).desc("Branch name")),
        )
        .flag(FlagSpec::both("u", "include-untracked").desc("Include untracked files"))
        .flag(FlagSpec::both("a", "all").desc("Include ignored files"))
        .flag(FlagSpec::both("m", "message").desc("Stash message").takes_arg("msg"))
}

fn status_spec() -> SubcommandSpec {
    SubcommandSpec::new("status")
        .desc("Show the working tree status")
        .flag(FlagSpec::both("s", "short").desc("Give output in short format"))
        .flag(FlagSpec::long("branch").desc("Show branch info"))
        .flag(FlagSpec::long("porcelain").desc("Machine-readable output"))
        .flag(FlagSpec::both("v", "verbose").desc("Be verbose"))
        .flag(FlagSpec::long("ignored").desc("Show ignored files"))
}

fn switch_spec() -> SubcommandSpec {
    SubcommandSpec::new("switch")
        .desc("Switch to a branch")
        .flag(FlagSpec::both("c", "create").desc("Create a new branch").takes_arg("name"))
        .flag(FlagSpec::long("detach").desc("Switch to a commit for inspection"))
        .flag(FlagSpec::both("f", "force").desc("Force switch"))
        .flag(FlagSpec::both("t", "track").desc("Track a remote branch"))
        .arg(
            ArgSpec::new(0)
                .desc("Branch to switch to")
                .when(WhenCondition::flags_absent(&["c", "create"]))
                .source(CompletionSource::command("git branch --list")),
        )
}

fn tag_spec() -> SubcommandSpec {
    SubcommandSpec::new("tag")
        .desc("Create, list, delete or verify a tag object")
        .flag(FlagSpec::both("a", "annotate").desc("Annotated tag"))
        .flag(FlagSpec::both("d", "delete").desc("Delete a tag"))
        .flag(FlagSpec::both("l", "list").desc("List tags"))
        .flag(FlagSpec::both("m", "message").desc("Tag message").takes_arg("msg"))
        .flag(FlagSpec::both("f", "force").desc("Force tag creation"))
        .arg(
            ArgSpec::new(0)
                .desc("Tag name or pattern")
                .when(WhenCondition::flags_absent(&["d", "delete"]))
                .source(CompletionSource::command("git tag --list")),
        )
}
