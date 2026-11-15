pub(super) const CONTRACT: &str = "\
A harness is started in a Docker container of the image that --docker-image
names, as

    sh -c <COMMAND>

where <COMMAND> is the value of --harness-run-command. In the container,

  /harness             holds the source code of the harness, a copy made
                       for this job
  /work                is the directory the harness works in, and the
                       working directory
  CLADENC_HARNESS      /harness
  CLADENC_INSTRUCTION  what the harness is asked to do

The command runs as the user cladenc runs as, with nothing on standard
input, and its standard output and standard error are kept.

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

The evaluator is run once for each evaluation, as

    EVALUATOR <harness> <task> <output>

  <harness>  a copy of the harness to try
  <task>     the id of the task, a line of the task file
  <output>   an empty directory for anything worth keeping, which is
             also the working directory

It runs in a process group of its own, with nothing on standard input, and
its standard output and standard error are kept beside <output>.

How the evaluator ends decides what is recorded:

  exit status 0   the evaluation succeeded
  exit status 1   the evaluation failed
  the time limit  the evaluator's process group is killed, and the evaluation
                  is recorded as timed out
  anything else   the search stops, and `cladenc resume` runs the job again";
