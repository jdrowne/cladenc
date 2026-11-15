pub(super) const CONTRACT: &str = "\
Every container of a search is started from one image, the image of the
search. It is the image that --docker-image names, or the image that
cladenc builds when the search is started, from the Dockerfile that
--dockerfile names, as

    docker build --tag <NAME> --file <FILE> <DIR>

  <NAME>  the name cladenc gives the image: cladenc- and sixteen random
          hexadecimal digits
  <FILE>  a copy of the Dockerfile, kept in image in the run directory
  <DIR>   an empty directory, so the Dockerfile has no file to copy into
          the image

The build has the time limit that --docker-build-timeout gives, and image
in the run directory keeps what Docker printed. cladenc tells Docker to
fetch nothing: a Dockerfile whose base image is on the machine, and whose
steps fetch nothing, is built with no connection to the internet. An image
that is not built stops `cladenc start` with exit status 2, before the
search has a journal. The image stays on the machine when the search ends:
`cladenc resume` starts its containers from it and builds nothing.

A harness is started in a container of the image of the search, as

    sh -c <COMMAND>

where <COMMAND> is the value of --harness-run-command. In the container,

  /harness             holds the source code of the harness, a copy made
                       for this job
  /work                is the directory the harness works in, and the
                       working directory
  CLADENC_HARNESS      /harness
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
reach. The key is read from its file when a search is started, and again
each time the search is resumed. cladenc keeps the key off every command
line, out of the journal and out of what it prints.

A procreation is carried out by the harness that procreates. /work holds a
second copy of its source code, and the instruction asks the harness to
improve that copy: what the harness leaves in /work is its child.
/evaluations holds the finished evaluations of the harness, to read: one
directory for each task, holding the outcome and what the evaluator left.

How the harness ends decides what is recorded:

  exit status 0     the procreation created a harness
  exit status 125   Docker did not start the container: the search stops, and
                    `cladenc resume` runs the job again
  any other status  the procreation failed
  the time limit    the container is removed, and the procreation is recorded
                    as timed out

A child that holds anything but regular files, directories and symbolic
links is a failed procreation. The source code of every child is kept under
harnesses in the run directory, created or not.

Where --harness-build-command is given, a harness is built before it is
run for the first time, in a container of the same image, as

    sh -c <COMMAND>

with /harness as the working directory, which holds the source code of the
harness and is what CLADENC_HARNESS names. A build calls no model, and is
given none of the three variables of the model. What the build makes stays
there, and is part of every copy made of the harness. The progenitor is
built before the first job, within the time limit of one procreation. A
child is built as the last step of its procreation, within what is left of
the time limit of that procreation. builds in the run directory keeps the
standard output and standard error of every build, in a directory named
after the harness.

How a build ends decides what is recorded:

  exit status 0     the harness is built
  exit status 125   Docker did not start the container: the search stops, and
                    `cladenc resume` builds again
  any other status  a child is recorded as build_failed, and is not created;
                    for the progenitor the search stops, and `cladenc resume`
                    builds it again
  the time limit    the container is removed, and the procreation of a child
                    is recorded as timed out; for the progenitor the search
                    stops

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
refused, and so is one with a task that lacks one of the three. The tasks
are copied into benchmark in the run directory when a search is started, so
the search keeps the tasks it started with, and `cladenc resume` does not
read the directory that --benchmark named.

The evaluator is run once for each evaluation, as

    EVALUATOR <harness> <task> <output>

  <harness>  a copy of the harness to try
  <task>     the id of the task, the name of its directory in the
             benchmark
  <output>   an empty directory for anything worth keeping, which is
             also the working directory

It runs in a process group of its own, with nothing on standard input, and
its standard output and standard error are kept beside <output>. It is
given CLADENC_MODEL, CLADENC_MODEL_URL and CLADENC_MODEL_KEY in its
environment, as a harness is.

How the evaluator ends decides what is recorded:

  exit status 0   the evaluation succeeded
  exit status 1   the evaluation failed
  the time limit  the evaluator's process group is killed, and the evaluation
                  is recorded as timed out
  anything else   the search stops, and `cladenc resume` runs the job again";
