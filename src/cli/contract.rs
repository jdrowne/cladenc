pub(super) const CONTRACT: &str = "\
Every container of a directed evolution is started from one image, the
image of the directed evolution. It is the image that --docker-image names,
or the image that cladenc builds when the directed evolution is started,
from the Dockerfile that --dockerfile names, as

    timeout -s KILL <SECONDS> docker build --tag <NAME> --file <FILE> <DIR>

  <SECONDS>  the value of --docker-build-timeout, and five seconds more
  <NAME>     the name cladenc gives the image: cladenc- and sixteen random
             hexadecimal digits
  <FILE>     a copy of the Dockerfile, kept in image in the outputs
             directory
  <DIR>      an empty directory, so the Dockerfile has no file to copy into
             the image

The build has the time limit that --docker-build-timeout gives, and image
in the outputs directory keeps what Docker printed. cladenc ends a build
that runs past its time limit, and the build ends itself five seconds later
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
there is its child. /var/log/cladenc/attempts holds the finished attempts
of the harness at the tasks that are not held out, to read: one directory
for each benchmark, and in it one for each task, holding what the attempt
of that task kept.

A harness that procreates is given more to read, and nothing more to
change: what was kept of each harness it descends from, back to the
primogenitor, and of each harness it made in an earlier procreation that
has ended, whether that one was created or not. It is given nothing of
any other harness.

  /var/log/cladenc/line/<ID>      a harness it descends from
  /var/log/cladenc/children/<ID>  a harness it made before

Each of these directories is named by the id of its harness, and holds

  harness      the source code of that harness
  attempts     its attempts at the tasks that are not held out, laid out
               as /var/log/cladenc/attempts is
  procreation  what the harness that made it printed, in stdout and
               stderr, with the id of that harness in the file named
               progenitor and one word in the file named outcome:
               created, failed, build_failed or timed_out
  build        what its build printed, in stdout and stderr

A directory that would hold nothing is not there: the primogenitor has no
procreation, a harness that was not created has no attempts, and a harness
that was not built has no build. Nothing of a holdout task is in any of
them.

The instruction asks the harness that procreates for a change to how its
child works on any task: one that answers something it saw in several of
its attempts, not in one, and that leaves in the source code nothing that
names a task, a file of a task or the answer to one. It says that taking
out what does not help is an improvement as well, that the child may be
asked in its turn to improve a copy of its own source code, so that making
the child better at that counts, and that the child is judged by tasks
the harness that procreates is not shown. An instruction is a request:
cladenc enforces none of it.

A harness procreates only once it has been benchmarked in full: it has
attempted every task of every benchmark of the directed evolution, so that
it finds every task that is not held out in /var/log/cladenc/attempts.
Where a procreation is due and no harness has been benchmarked in full
yet, an attempt is started in its place: the primogenitor is benchmarked
in full before the first procreation.

Every harness is benchmarked in full once it is created, whether it
procreates or not. cladenc chooses which harness procreates, partly by
chance. It does not choose which harness attempts next: the created harness
with the lowest id that has a task left attempts it.

--max-procreations gives the most procreations a directed evolution
starts. A procreation counts once it is started, whether or not it creates
its child. The directed evolution ends once that many have been started,
each of them has ended, and every created harness has been benchmarked in
full. With one benchmark of 5 tasks and --max-procreations 1, the
primogenitor is benchmarked on the 5 tasks and procreates, and its child is
benchmarked on the 5 tasks where the procreation created it.

--genealogy-growth says how soon a procreation is due, and never how many
are started: where it lets none start and nothing else is left to start,
a procreation is started all the same. At 0, each child is benchmarked in
full before the next procreation starts.

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
harnesses in the outputs directory, created or not. procreations in the
outputs directory keeps, in a directory named after the child, the
standard output and standard error of the harness that made it, the id of
that harness in the file named progenitor, and how the procreation ended
in the file named outcome.

Where --harness-build-command is given, a harness is built before it is
run for the first time, in a container of the same image, as

    sh -c <TIMER> <COMMAND> <SECONDS>

where <COMMAND> is the value of that flag and the timer is the same one,
with /var/lib/cladenc/harness as the working directory, which holds the
source code of the harness and is what CLADENC_HARNESS names. A build calls
no model, and is given none of the three variables of the model. What the
build makes stays there, and is part of every copy made of the harness. The
primogenitor is built before any procreation or attempt, within the time
limit of one procreation. A child is built as the last step of its
procreation, within what is left of the time limit of that procreation.

builds in the outputs directory keeps the standard output and standard error
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

  specification  a file of text, the task specification, which says what
                 is to be done
  workspace      a directory of the files the work starts from, which may
                 hold none
  grader         an executable file, the program that says by its exit
                 status whether the task was completed

and may hold more, such as tests that the grader runs. A file that is in
the benchmark and in no task is left alone. A benchmark that holds no task
is refused, and so is one with a task that lacks one of the three.

--benchmark is given once for each benchmark of the directed evolution. A
directed evolution that is given several has each harness benchmarked on
all of them, and looks for the one harness that completes the most of their
selection tasks. Every selection task counts the same, so a benchmark with
more of them weighs more. The name of a benchmark is the name of its
directory, made of the characters the id of a task is made of, and a task
of a directed evolution is known by the two together, as

    <BENCHMARK>/<TASK>

so two benchmarks may each hold a task of the same id. Two benchmarks whose
directories have the same name are refused. The tasks are copied into
benchmarks in the outputs directory when a directed evolution is started,
each into a directory named after its benchmark, so the directed evolution
keeps the tasks it started with, and `cladenc resume` does not read the
directories that --benchmark named.

--holdout-percentage gives the part of the tasks of each benchmark that is
held out, as so many in a hundred: a number above 0 and below 100, such as
60 or 12.5. The holdout tasks are the tasks that no progenitor reads
about. cladenc draws them at random when a directed evolution is started,
out of each benchmark by itself: the number of its tasks times the
percentage, divided by 100 and rounded to the nearest whole number, a half
rounded up. At 60, a benchmark of 500 tasks has 300 holdout tasks. A value
that leaves a benchmark no holdout task, or no task that is not held out,
is refused, and so is a benchmark of one task, which cannot be divided.
The draw is made once for a directed evolution and written in its journal,
and `cladenc resume` reads it there and draws nothing.

--holdout-corroboration-percentage divides the holdout tasks of each
benchmark in two. It gives the part of the holdout tasks that are
corroboration tasks, as so many in a hundred: a number above 0 and below
100. cladenc draws the corroboration tasks at random, once the holdout
tasks are drawn, out of the holdout tasks of each benchmark by itself: the
number of the holdout tasks of the benchmark times the percentage, divided
by 100 and rounded in the same way. The holdout tasks that are left are the
selection tasks. At 60 and 25, a benchmark of 500 tasks has 300 holdout
tasks: 75 corroboration tasks and 225 selection tasks. A value that leaves
a benchmark no corroboration task, or no selection task, is refused, so a
benchmark holds three tasks at the least. The second draw is written in the
journal beside the first.

Every harness is benchmarked on the holdout tasks as on the others,
selection tasks and corroboration tasks alike, and nothing a harness is
given in an attempt says which of the sets its task is in. A harness that
procreates is given its attempts at the tasks that are not held out, and
nothing of the holdout tasks: no task specification, no workspace, no
result, and no count of them.

Artificial selection goes by the selection tasks and by no others. cladenc
chooses which harness procreates by the selection tasks that the harness
and its descendants completed and did not complete, and it names as the
best harness the created harness, benchmarked in full, that completed the
most selection tasks: of several that completed as many, the one created
first. The tasks that are not held out are what a harness reads to find
what to change, and a harness that completes more of those alone is not
chosen for it.

cladenc counts for every harness the corroboration tasks it completed, and
chooses nothing by them: not the harness that procreates, and never the
best harness. A set of tasks gives a fair count only while nothing is
chosen by it.

The best harness is named among many, each benchmarked once, and a model
does not answer the same way twice. So the best of many harnesses that are
all alike has still completed more selection tasks than the primogenitor,
by luck. That luck does not reach the corroboration tasks: a harness that
was lucky on the selection tasks is no luckier on other tasks, and a change
that makes a harness complete more tasks shows on both sets. So how far the
best harness is ahead of the primogenitor is read in the columns of the
corroboration tasks of `cladenc report`. A lead there corroborates the lead
on the selection tasks: it came through a test that could have refuted it,
which is not to say that it is proven. One of two harnesses that are alike
is still ahead on the corroboration tasks by a few of them, and the fewer
the corroboration tasks, the more a few of them weigh.

How many more selection tasks the best of many harnesses that are all alike
completes is measured by a directed evolution in which no harness changes:
one carried out with the values of the directed evolution it is set beside,
and with a run command that ends at once with exit status 0 where the
harness is started for a procreation, as

    --harness-run-command \\
        '[ -d /var/log/cladenc/attempts ] && exit 0; <COMMAND>'

where <COMMAND> is the run command of the other. /var/log/cladenc/attempts
is there for a procreation and not for an attempt, so every procreation
creates a child that is the source code of the primogenitor, unchanged, and
calls no model. Such a directed evolution holds as many children as
--max-procreations gives, each of them benchmarked in full. In
`cladenc report` of it, how far apart the counts of the harnesses lie is
what luck does to the count of one harness, and how many more selection
tasks the best harness completed than the primogenitor is what luck and the
choosing give together, with nothing changed. The best harness of the other
directed evolution is ahead of its primogenitor on the selection tasks by a
change only as far as it is ahead by more than that. `cladenc report` also
says how many tasks at least one harness completed, of the selection tasks,
of the corroboration tasks and of the others. Where no harness changes,
that is what one more attempt at each task for each child gives the
primogenitor, and it is set beside the count of the best harness of the
other directed evolution, which had one attempt at each task.

An attempt is one harness working on one task, once, and cladenc carries it
out in two steps. First the harness is started, as it is for a procreation.
/var/lib/cladenc/work holds a copy of the workspace of the task, and
CLADENC_INSTRUCTION holds the task specification. The harness is given
nothing else of the task, so it can neither read nor change the grader.
When the harness has ended, with whatever exit status, the grader of the
task is run in a second container of the same image, as

    sh -c <TIMER> /var/lib/cladenc/task/grader <SECONDS>

  /var/lib/cladenc/work  holds what the harness left, and is the working
                         directory
  /var/lib/cladenc/task  holds the directory of the task, to read and not
                         to change
  CLADENC_TASK           /var/lib/cladenc/task

A grader calls no model, and is given none of the three variables of the
model. The two steps share the time limit of one attempt.

How an attempt ends decides its result:

  the grader exits with status 0   the harness completed the task
  the grader exits with another    the harness did not complete the task
  exit status 125 of either step   Docker did not start the container: the
                                   directed evolution stops, and `cladenc
                                   resume` carries out the attempt again
  the time limit                   the container is removed, and the
                                   harness did not complete the task,
                                   which is recorded as timed_out

A task that is recorded as completed means that the harness did the work
only as far as three things hold, and cladenc sees to none of them.

A harness reaches the network. Every container is started on the usual
network of Docker, and reaches the machine cladenc runs on and whatever
that machine reaches. Where a task was made from something public, its
answer may be one fetch away, and a harness can leave something outside
its container for a later harness to find, which holding tasks out does
not guard against. To keep every harness from reaching further than the
model, carry out the directed evolution on a machine that has no other
connection, with the model served on that machine.

A grader may trust what the harness left. The grader is started in the
workspace as the harness left it, and everything there was the harness's
to change: tests, build files, any program the grader starts from there.
A grader that runs the tests it finds in the workspace is passed by a
harness that changed those tests, on a holdout task as on any other. The
directory of the task is never given to a harness: a grader that keeps
what it relies on there, its tests among them, and takes nothing from the
workspace on trust, cannot be passed that way.

The model may know the task. A task taken from a public source may have
been among what the model was trained on. A harness then completes it from
what the model remembers, and a child that completes more of such tasks
may only be better at drawing that out.

attempts in the outputs directory keeps every attempt at a task that is not
held out, in a directory named after the harness, in it one named after the
benchmark, and in that one named after the task, which holds

  result         one word: completed, not_completed or timed_out
  specification  the task specification
  work           the workspace of the task as the harness left it
  stdout         the standard output of the harness
  stderr         the standard error of the harness
  grader         the standard output and the standard error of the grader

holdout in the outputs directory keeps the attempts at the holdout tasks,
selection tasks and corroboration tasks together, in the same way, apart
from the others, so that the directory a harness that procreates is given
holds none of them.";

pub(super) const RESUMING: &str = "\
A cladenc that is killed outright, as by `kill -9`, ends none of its
containers, and each goes on until its time limit has passed. Before it
does anything else, `cladenc resume` removes the containers that the
directed evolution left running, so that it never works beside one of them.

Every container of a directed evolution carries a Docker label, a name and
a value that Docker keeps with a container:

    cladenc.directed-evolution=<DIR>

where <DIR> is the path of the outputs directory, with every symbolic link
in it resolved. Once it has the outputs directory to itself,
`cladenc resume` asks Docker for the containers that carry the label of its
directed evolution, removes them, and says how many it removed. Where
Docker cannot be asked, it exits with status 2 and continues nothing.

Where the directed evolution is not resumed, its containers are removed by
hand.

    docker ps --filter label=cladenc.directed-evolution=<DIR>

lists them, their names begin cladenc-, and `docker kill` with a name ends
that container. Docker removes a container of cladenc once it has ended.";
