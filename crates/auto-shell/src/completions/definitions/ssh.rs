//! SSH/SCP completion specification

use ash_core::completions::spec::*;

pub fn ssh_spec() -> CompletionSpec {
    CompletionSpec::new("ssh")
        .desc("OpenSSH remote login client")
        .flag(FlagSpec::both("p", "port").desc("Port to connect to").takes_arg("port"))
        .flag(FlagSpec::both("i", "identity").desc("Identity file").takes_arg("file"))
        .flag(FlagSpec::both("v", "verbose").desc("Verbose mode"))
        .flag(FlagSpec::both("q", "quiet").desc("Quiet mode"))
        .flag(FlagSpec::both("C", "compress").desc("Enable compression"))
        .flag(FlagSpec::long("config").desc("Config file").takes_arg("file"))
        .flag(FlagSpec::both("L", "local-forward").desc("Local port forwarding").takes_arg("forward"))
        .flag(FlagSpec::both("R", "remote-forward").desc("Remote port forwarding").takes_arg("forward"))
        .flag(FlagSpec::both("N", "no-command").desc("No remote command"))
        .flag(FlagSpec::both("T", "no-tty").desc("Disable pseudo-TTY allocation"))
        .flag(FlagSpec::both("t", "tty").desc("Force pseudo-TTY allocation"))
        .flag(FlagSpec::long("no StrictHostKeyChecking=no").desc("Skip host key check"))
        .arg(ArgSpec::new(0).desc("Destination (user@host)"))
}

pub fn scp_spec() -> CompletionSpec {
    CompletionSpec::new("scp")
        .desc("Secure copy")
        .flag(FlagSpec::both("r", "recursive").desc("Copy directories recursively"))
        .flag(FlagSpec::both("P", "port").desc("Port").takes_arg("port"))
        .flag(FlagSpec::both("i", "identity").desc("Identity file").takes_arg("file"))
        .flag(FlagSpec::both("v", "verbose").desc("Verbose mode"))
        .flag(FlagSpec::both("C", "compress").desc("Enable compression"))
        .flag(FlagSpec::both("q", "quiet").desc("Quiet mode"))
        .arg(ArgSpec::new(0).desc("Source"))
        .arg(ArgSpec::new(1).desc("Destination"))
}
