//! Docker completion specification

use ash_core::completions::spec::*;

pub fn spec() -> CompletionSpec {
    CompletionSpec::new("docker")
        .desc("Docker container management")
        .subcommand(run_spec())
        .subcommand(build_spec())
        .subcommand(ps_spec())
        .subcommand(images_spec())
        .subcommand(exec_spec())
        .subcommand(stop_spec())
        .subcommand(rm_spec())
        .subcommand(rmi_spec())
        .subcommand(compose_spec())
        .subcommand(logs_spec())
        .subcommand(pull_spec())
        .subcommand(push_spec())
}

fn run_spec() -> SubcommandSpec {
    SubcommandSpec::new("run")
        .desc("Run a command in a new container")
        .flag(FlagSpec::both("d", "detach").desc("Run in background"))
        .flag(FlagSpec::both("i", "interactive").desc("Keep STDIN open"))
        .flag(FlagSpec::both("t", "tty").desc("Allocate pseudo-TTY"))
        .flag(FlagSpec::long("name").desc("Container name").takes_arg("name"))
        .flag(FlagSpec::both("p", "publish").desc("Publish port (host:container)").takes_arg("port"))
        .flag(FlagSpec::both("v", "volume").desc("Bind mount volume").takes_arg("volume"))
        .flag(FlagSpec::both("e", "env").desc("Set environment variable").takes_arg("env"))
        .flag(FlagSpec::long("rm").desc("Auto-remove container on exit"))
        .flag(FlagSpec::long("network").desc("Network mode").takes_arg("network"))
        .flag(FlagSpec::both("w", "workdir").desc("Working directory").takes_arg("dir"))
        .flag(FlagSpec::long("restart").desc("Restart policy").takes_arg("policy"))
        .arg(ArgSpec::new(0).desc("Image name").source(CompletionSource::command("docker images --format '{{.Repository}}:{{.Tag}}'")))
}

fn build_spec() -> SubcommandSpec {
    SubcommandSpec::new("build")
        .desc("Build an image from a Dockerfile")
        .flag(FlagSpec::both("t", "tag").desc("Name and optionally tag").takes_arg("tag"))
        .flag(FlagSpec::long("file").desc("Dockerfile path").takes_arg("file"))
        .flag(FlagSpec::long("no-cache").desc("Build without cache"))
        .flag(FlagSpec::both("q", "quiet").desc("Suppress build output"))
        .flag(FlagSpec::long("build-arg").desc("Build-time variable").takes_arg("arg"))
        .flag(FlagSpec::long("target").desc("Target build stage").takes_arg("stage"))
        .arg(ArgSpec::new(0).desc("Build context path").source(CompletionSource::Directories))
}

fn ps_spec() -> SubcommandSpec {
    SubcommandSpec::new("ps")
        .desc("List containers")
        .flag(FlagSpec::both("a", "all").desc("Show all containers (default running only)"))
        .flag(FlagSpec::both("q", "quiet").desc("Only display IDs"))
        .flag(FlagSpec::long("format").desc("Format output").takes_arg("format"))
        .flag(FlagSpec::long("filter").desc("Filter output").takes_arg("filter"))
        .flag(FlagSpec::both("n", "last").desc("Show n last created containers").takes_arg("n"))
}

fn images_spec() -> SubcommandSpec {
    SubcommandSpec::new("images")
        .desc("List images")
        .flag(FlagSpec::both("a", "all").desc("Show all images (default hides intermediate)"))
        .flag(FlagSpec::both("q", "quiet").desc("Only display IDs"))
        .flag(FlagSpec::long("format").desc("Format output").takes_arg("format"))
        .flag(FlagSpec::long("filter").desc("Filter output").takes_arg("filter"))
}

fn exec_spec() -> SubcommandSpec {
    SubcommandSpec::new("exec")
        .desc("Run a command in a running container")
        .flag(FlagSpec::both("i", "interactive").desc("Keep STDIN open"))
        .flag(FlagSpec::both("t", "tty").desc("Allocate pseudo-TTY"))
        .flag(FlagSpec::both("d", "detach").desc("Run in background"))
        .flag(FlagSpec::both("e", "env").desc("Set environment variable").takes_arg("env"))
        .flag(FlagSpec::both("w", "workdir").desc("Working directory").takes_arg("dir"))
        .arg(ArgSpec::new(0).desc("Container").source(CompletionSource::command("docker ps --format '{{.Names}}'")))
        .arg(ArgSpec::new(1).desc("Command"))
}

fn stop_spec() -> SubcommandSpec {
    SubcommandSpec::new("stop")
        .desc("Stop one or more running containers")
        .flag(FlagSpec::both("t", "time").desc("Seconds to wait for stop").takes_arg("seconds"))
        .arg(ArgSpec::new(0).repeat().desc("Container").source(CompletionSource::command("docker ps --format '{{.Names}}'")))
}

fn rm_spec() -> SubcommandSpec {
    SubcommandSpec::new("rm")
        .desc("Remove one or more containers")
        .flag(FlagSpec::both("f", "force").desc("Force removal"))
        .flag(FlagSpec::long("volumes").desc("Remove anonymous volumes"))
        .arg(ArgSpec::new(0).repeat().desc("Container").source(CompletionSource::command("docker ps -a --format '{{.Names}}'")))
}

fn rmi_spec() -> SubcommandSpec {
    SubcommandSpec::new("rmi")
        .desc("Remove one or more images")
        .flag(FlagSpec::both("f", "force").desc("Force removal"))
        .arg(ArgSpec::new(0).repeat().desc("Image").source(CompletionSource::command("docker images --format '{{.Repository}}:{{.Tag}}'")))
}

fn compose_spec() -> SubcommandSpec {
    SubcommandSpec::new("compose")
        .desc("Docker Compose")
        .subcommand(SubcommandSpec::new("up").desc("Start services")
            .flag(FlagSpec::both("d", "detach").desc("Run in background"))
            .flag(FlagSpec::long("build").desc("Build images before starting"))
            .flag(FlagSpec::long("force-recreate").desc("Recreate containers")))
        .subcommand(SubcommandSpec::new("down").desc("Stop services")
            .flag(FlagSpec::both("v", "volumes").desc("Remove volumes")))
        .subcommand(SubcommandSpec::new("logs").desc("View output from containers")
            .flag(FlagSpec::both("f", "follow").desc("Follow log output"))
            .flag(FlagSpec::long("tail").desc("Number of lines").takes_arg("n")))
        .subcommand(SubcommandSpec::new("build").desc("Build services")
            .flag(FlagSpec::long("no-cache").desc("Build without cache")))
        .subcommand(SubcommandSpec::new("ps").desc("List containers"))
}

fn logs_spec() -> SubcommandSpec {
    SubcommandSpec::new("logs")
        .desc("Fetch logs of a container")
        .flag(FlagSpec::both("f", "follow").desc("Follow log output"))
        .flag(FlagSpec::long("tail").desc("Number of lines from end").takes_arg("n"))
        .flag(FlagSpec::long("since").desc("Show logs since timestamp").takes_arg("ts"))
        .flag(FlagSpec::long("until").desc("Show logs before timestamp").takes_arg("ts"))
        .arg(ArgSpec::new(0).desc("Container").source(CompletionSource::command("docker ps --format '{{.Names}}'")))
}

fn pull_spec() -> SubcommandSpec {
    SubcommandSpec::new("pull")
        .desc("Pull an image or repository from a registry")
        .flag(FlagSpec::both("a", "all-tags").desc("Pull all tagged images"))
        .arg(ArgSpec::new(0).desc("Image name"))
}

fn push_spec() -> SubcommandSpec {
    SubcommandSpec::new("push")
        .desc("Push an image to a registry")
        .arg(ArgSpec::new(0).desc("Image name").source(CompletionSource::command("docker images --format '{{.Repository}}:{{.Tag}}'")))
}
