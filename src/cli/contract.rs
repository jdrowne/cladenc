pub(super) const CONTRACT: &str = "\
Every container of a directed evolution is started from one image, the
image of the directed evolution. It is the image that --docker-image names,
or the image that cladenc builds when the directed evolution is started,
from the Dockerfile that --dockerfile names, as

    timeout -s KILL <SECONDS> docker build --tag <NAME> --file <FILE> <DIR>

  <SECONDS>  the value of --docker-build-timeout, and five seconds more
  <NAME>     the name cladenc gives the image: cladenc- and sixteen random
             hexadecimal digits
  <FILE>     a copy of the Dockerfile, kept in image in the run directory
  <DIR>      an empty directory, so the Dockerfile has no file to copy into
             the image

The build has the time limit that --docker-build-timeout gives, and image
in the run directory keeps what Docker printed. cladenc ends a build that
runs past its time limit, and the build ends itself five seconds later
where cladenc was killed outright, as by `kill -9`, and is not there to end
it. The machine cladenc runs on has to hold timeout, which the package
coreutils brings. An image that is not built stops `cladenc start` with
exit status 2, before the directed evolution has a journal. The image stays
on the machine when the directed evolution ends: `cladenc resume` starts
its containers from it and builds nothing.

cladenc fetches no image, and has no flag for a registry, the server that
Docker fetches an image from. The name of an image says which registry it
is of, as registry.example:5000/tools/harness:1 does, and a name that says
none is of Docker Hub. The image that --docker-image names has to be on the
machine before the directed evolution: fetched there with `docker pull`,
carried there as a file with `docker save` and `docker load`, or built
there. cladenc tells Docker never to fetch the image of a container, so a
directed evolution whose image is not on the machine stops at its first
container, and `cladenc resume` continues it once the image is there. A
Dockerfile whose base image is on the machine, and whose steps fetch
nothing, is built with no connection to the internet. cladenc does not ask
Docker to fetch a base image and cannot forbid it: Docker fetches a base
image that is not on the machine, from the registry of its name.

A harness is started in a container of the image of the directed evolution,
as

    sh -c <TIMER> <COMMAND> <SECONDS>

  <TIMER>    sh -c \"$0\" & run=$!; (sleep \"$1\"; kill -9 $run) & wait $run
  <COMMAND>  the value of --harness-run-command
  <SECONDS>  what is left of the time limit when the container is started,
             in whole seconds, and five seconds more

The timer is a line of shell that starts the command, ends it once the
seconds have passed, and hands on its exit status. Every container of a
directed evolution is started under it, so that a container ends itself
once its time limit has passed, also when cladenc was killed outright, as
by `kill -9`, and is not there to remove it. The image of the directed
evolution has to hold sh and sleep.

Until then a container that was left goes on. `cladenc resume` removes the
containers that the directed evolution left running before it does anything
else, and its long help says how they are found, and how they are removed
by hand where the directed evolution is not resumed.

In the container of a harness,

  /var/lib/cladenc/harness  holds the source code of the harness, a copy
                            made for this container
  /var/lib/cladenc/work     is the directory the harness works in, and the
                            working directory

  CLADENC_HARNESS      /var/lib/cladenc/harness
  CLADENC_INSTRUCTION  what the harness is asked to do
  CLADENC_MODEL        the name of the model, the value of --model
  CLADENC_MODEL_URL    the address the model is served at, the value of
                       --model-url
  CLADENC_MODEL_KEY    the key the model is called with, read from the file
                       that --model-key-file names, and not set where that
                       flag is not given

The command runs as the user cladenc runs as, with nothing on standard
input, and its standard output and standard error are kept.

cladenc calls no model. A harness calls the model itself, at the address it
is told. In every container, host.docker.internal names the machine cladenc
runs on, so a model served on that machine at port 11434 is named by

    --model-url http://host.docker.internal:11434

and is reached as long as its server listens on an address a container can
reach. The key is read from its file when a directed evolution is started,
and again each time the directed evolution is resumed. cladenc keeps the
key off every command line, out of the journal and out of what it prints.

A procreation is carried out by the harness that procreates.
/var/lib/cladenc/work holds a second copy of its source code, and the
instruction asks the harness to improve that copy: what the harness leaves
there is its child. /var/log/cladenc/assessments holds the finished
assessments of the harness, to read: one directory for each benchmark, and
in it one for each task, holding what the assessment of that task kept.

How the harness ends decides what is recorded:

  exit status 0     the procreation created a harness
  exit status 125   Docker did not start the container: the directed
                    evolution stops, and `cladenc resume` carries out the
                    procreation again
  any other status  the procreation failed
  the time limit    the container is removed, and the procreation is
                    recorded as timed out

A child that holds anything but regular files, directories and symbolic
links is a failed procreation. The source code of every child is kept under
harnesses in the run directory, created or not.

Where --harness-build-command is given, a harness is built before it is
run for the first time, in a container of the same image, as

    sh -c <TIMER> <COMMAND> <SECONDS>

where <COMMAND> is the value of that flag and the timer is the same one,
with /var/lib/cladenc/harness as the working directory, which holds the
source code of the harness and is what CLADENC_HARNESS names. A build calls
no model, and is given none of the three variables of the model. What the
build makes stays there, and is part of every copy made of the harness. The
primogenitor is built before any procreation or assessment, within the time
limit of one procreation. A child is built as the last step of its
procreation, within what is left of the time limit of that procreation.

builds in the run directory keeps the standard output and standard error
of every build, in a directory named after the harness.

How a build ends decides what is recorded:

  exit status 0     the harness is built
  exit status 125   Docker did not start the container: the directed
                    evolution stops, and `cladenc resume` builds again
  any other status  a child is recorded as build_failed, and is not
                    created; for the primogenitor the directed evolution
                    stops, and `cladenc resume` builds it again
  the time limit    the container is removed, and the procreation of a
                    child is recorded as timed out; for the primogenitor the
                    directed evolution stops

A build that leaves anything but regular files, directories and symbolic
links has failed.

A benchmark is a directory that holds one directory for each task. The name
of a task's directory is the id of the task, made of letters, digits, dots,
underscores and hyphens, and the directory holds

  statement  a file of text that says what is to be done
  files      a directory of the files the work starts from, which may hold
             none
  check      an executable file, the program that says by its exit status
             whether the work passes

and may hold more, such as tests that the check runs. A file that is in the
benchmark and in no task is left alone. A benchmark that holds no task is
refused, and so is one with a task that lacks one of the three.

--benchmark is given once for each benchmark of the directed evolution. A
directed evolution that is given several tries its harnesses on the tasks
of all of them, and looks for the one harness that passes the largest share
of those tasks. Every task counts the same, so a benchmark with more tasks
weighs more. The name of a benchmark is the name of its directory, made of
the characters the id of a task is made of, and a task of a directed
evolution is known by the two together, as

    <BENCHMARK>/<TASK>

so two benchmarks may each hold a task of the same id. Two benchmarks whose
directories have the same name are refused. The tasks are copied into
benchmarks in the run directory when a directed evolution is started, each
into a directory named after its benchmark, so the directed evolution keeps
the tasks it started with, and `cladenc resume` does not read the
directories that --benchmark named.

An assessment is one harness tried on one task, and cladenc carries it out
in two steps. First the harness is started, as it is for a procreation.
/var/lib/cladenc/work holds a copy of the files of the task, and
CLADENC_INSTRUCTION holds the statement of the task. The harness is given
nothing else of the task, so it can neither read nor change the check. When
the harness has ended, with whatever exit status, the check of the task is
run in a second container of the same image, as

    sh -c <TIMER> /var/lib/cladenc/task/check <SECONDS>

  /var/lib/cladenc/work  holds what the harness left, and is the working
                         directory
  /var/lib/cladenc/task  holds the directory of the task, to read and not
                         to change
  CLADENC_TASK           /var/lib/cladenc/task

A check calls no model, and is given none of the three variables of the
model. The two steps share the time limit of one assessment.

How an assessment ends decides how the harness is scored:

  the check exits with status 0    the harness is scored a success
  the check exits with another     the harness is scored a failure
  exit status 125 of either step   Docker did not start the container: the
                                   directed evolution stops, and `cladenc
                                   resume` carries out the assessment again
  the time limit                   the container is removed, and the
                                   harness is scored a failure, which is
                                   recorded as timed_out

assessments in the run directory keeps every assessment, in a directory
named after the harness, in it one named after the benchmark, and in that
one named after the task, which holds

  score      one word: success, failure or timed_out
  statement  the statement of the task
  work       the files of the task as the harness left them
  stdout     the standard output of the harness
  stderr     the standard error of the harness
  check      the standard output and the standard error of the check";

pub(super) const RESUMING: &str = "\
A cladenc that is killed outright, as by `kill -9`, ends none of its
containers, and each goes on until its time limit has passed. Before it
does anything else, `cladenc resume` removes the containers that the
directed evolution left running, so that it never works beside one of them.

Every container of a directed evolution carries a Docker label, a name and
a value that Docker keeps with a container:

    cladenc.directed-evolution=<DIR>

where <DIR> is the path of the run directory, with every symbolic link in
it resolved. Once it has the run directory to itself, `cladenc resume` asks
Docker for the containers that carry the label of its directed evolution,
removes them, and says how many it removed. Where Docker cannot be asked,
it exits with status 2 and continues nothing.

Where the directed evolution is not resumed, its containers are removed by
hand.

    docker ps --filter label=cladenc.directed-evolution=<DIR>

lists them, their names begin cladenc-, and `docker kill` with a name ends
that container. Docker removes a container of cladenc once it has ended.";
