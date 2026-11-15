pub(super) const CONTRACT: &str = "\
The mutator is run once for each mutation, as

    MUTATOR <parent> <child> <evaluations> <output>

  <parent>       a copy of the harness being mutated, to read
  <child>        a second copy of it, to change into the new harness
  <evaluations>  the finished evaluations of the parent, to read: one
                 directory for each task, holding the outcome and what
                 the evaluator left
  <output>       an empty directory for anything worth keeping, which is
                 also the working directory

The evaluator is run once for each evaluation, as

    EVALUATOR <harness> <task> <output>

  <harness>  a copy of the harness to try
  <task>     the id of the task, a line of the task file
  <output>   an empty directory for anything worth keeping, which is
             also the working directory

Each program runs in a process group of its own, with nothing on standard
input, and its standard output and standard error are kept beside <output>.

How a program ends decides what is recorded:

  exit status 0   the mutation created a harness; the evaluation succeeded
  exit status 1   the mutation failed; the evaluation failed
  the time limit  the program's process group is killed, and the job is
                  recorded as timed out
  anything else   the search stops, and `cladenc resume` runs the job again

A child that holds anything but regular files, directories and symbolic
links is a failed mutation.";
