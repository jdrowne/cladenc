pub(super) const CONTRACT: &str = "\
Every procreation, and every build of a harness, is carried out in a
container of one image, the image of the directed evolution. The creation
of a singleton control group harness is carried out in a container of that
image as well, and no attempt is: an attempt is carried out in the
environment of its task. The image of the directed evolution is the image
that --procreation-docker-image names, or the image that cladenc builds
when the directed evolution is started, from the Dockerfile that
--procreation-dockerfile names, as

    timeout -s KILL <SECONDS> docker build --tag <NAME> --file <FILE> <DIR>

  <SECONDS>  the value of --procreation-docker-image-build-timeout, and
             five seconds more
  <NAME>     the name cladenc gives the image: cladenc- and sixteen random
             hexadecimal digits
  <FILE>     a copy of the Dockerfile, kept in image in the outputs
             directory
  <DIR>      an empty directory, so the Dockerfile has no file to copy into
             the image

The build has the time limit that --procreation-docker-image-build-timeout
gives, and image in the outputs directory keeps what Docker printed.
cladenc ends a build that runs past its time limit, and the build ends
itself five seconds later where cladenc was killed outright, as by `kill
-9`, and is not there to end it. The machine cladenc runs on has to hold
timeout, which the package coreutils brings. An image that is not built
stops `cladenc start` with exit status 2, before the directed evolution has
a journal. The image stays on the machine when the directed evolution ends:
`cladenc resume` carries out its procreations and builds in it, and does
not build it again.

cladenc fetches no image, and has no flag for a registry, the server that
Docker fetches an image from. The name of an image says which registry it
is of, as registry.example:5000/tools/harness:1 does, and a name that says
none is of Docker Hub. The image that --procreation-docker-image names has
to be on the machine before the directed evolution: fetched there with
`docker pull`, carried there as a file with `docker save` and `docker
load`, or built there. cladenc tells Docker never to fetch the image of a
container, so a directed evolution whose image is not on the machine stops
at the first container of that image, and `cladenc resume` continues it
once the image is there. A Dockerfile whose base image is on the machine,
and whose steps fetch nothing, is built with no connection to the internet.
cladenc does not ask Docker to fetch a base image and cannot forbid it:
Docker fetches a base image that is not on the machine, from the registry
of its name.

cladenc builds the environment of every task of every benchmark when the
directed evolution is started, and again each time it is resumed, before
it brings up any container, as

    docker build --tag <NAME> --label cladenc.task=<TASK> \\
        --label cladenc.directed-evolution=<PATH> <DIR>

  <NAME>  the name cladenc gives the environment: cladenc-environments-
          and sixteen random hexadecimal digits, the same for every task
          of the directed evolution, then a colon and the place of the
          task among the tasks of the directed evolution, counted from 0
  <TASK>  the task, as <BENCHMARK>/<TASK>
  <PATH>  the path of the outputs directory, with every symbolic link in
          it resolved
  <DIR>   the environment directory of the copy of the task in the
          outputs directory, the build context of its Dockerfile

The two labels are Docker labels, each a name and a value that Docker
keeps with an image. The second is the label that every container of the
directed evolution carries. It makes the environments of two directed
evolutions two images, even where both are built from one task of one
benchmark, so that removing those of one leaves those of the other on the
machine.

The build has the time limit that the task.toml of the task gives, as
build_timeout_sec under environment, or 600 seconds where it gives none.
environments/<BENCHMARK>/<TASK> in the outputs directory keeps what Docker
printed. A task whose environment is
not built stops the directed evolution with exit status 2, and
`cladenc start` leaves it no journal. A resumed directed evolution builds
each environment again, under the same name. The image that the Dockerfile
of a task starts from has to be on the machine: cladenc does not ask
Docker to fetch it and cannot forbid it, so Docker fetches a base image
that is not on the machine, from the registry of its name.

cladenc removes the environments it built once `cladenc start` or
`cladenc resume` ends, whether the directed evolution finished, stopped or
could not go on, as

    docker image rm --force <NAME>...

so no directed evolution leaves them on the machine. An image that the
task.toml of a task names is not one cladenc built, and is left alone. A
cladenc that was killed outright removes nothing: `cladenc resume` builds
the environments again, and removes them when it ends.

A stop while the image of the directed evolution or an environment is
built comes before the directed evolution has a journal. `cladenc start`
then exits with status 2 and says that the directed evolution was stopped
before it was started: `cladenc resume` has nothing to continue, and the
outputs directory is removed by hand before it is named to `cladenc start`
again.

A task whose task.toml names an image, as docker_image under environment,
has that image as its environment: cladenc builds nothing for the task, and
the task needs no environment directory. The image has to be on the machine
before the directed evolution, as the image that --procreation-docker-image
names has to be. cladenc looks for it when the directed evolution is
started, and again each time it is resumed, as

    docker image inspect --format {{.Id}} <IMAGE>

and a task whose image is not on the machine stops the directed evolution
with exit status 2. `cladenc start` then leaves it no journal.

A harness is started in a container, of the image of the directed
evolution for a procreation and of the environment of its task for an
attempt, as

    sh -c <TIMER> <COMMAND> <SECONDS>

  <TIMER>    sh -c \"$0\" & run=$!;
             (sleep \"$1\"; kill -9 $run) > /dev/null 2>&1 & wait $run
  <COMMAND>  the value of --harness-run-command
  <SECONDS>  what is left of the time limit of the command when it is
             started, in whole seconds, and five seconds more

The timer is a line of shell that starts the command, ends it once the
seconds have passed, and hands on its exit status. Every container of a
directed evolution is started under it, or, for an attempt, under sleep, so
that a container ends itself once its time limit has passed, also when
cladenc was killed outright, as by `kill -9`, and is not there to remove
it. The image of the directed evolution has to hold sh and sleep, and the
environment of every task sh, sleep, rm, mkdir, chown and ln.

Until then a container that was left goes on. `cladenc resume` removes the
containers that the directed evolution left running before it does anything
else, and its long help says how they are found, and how they are removed
by hand where the directed evolution is not resumed.

In the container of a harness,

  /var/opt/cladenc/harness  holds the source code of the harness, a copy
                            made for this container
  /var/opt/cladenc/work     in a procreation, the directory the harness
                            works in, and the working directory

  CLADENC_HARNESS      /var/opt/cladenc/harness
  CLADENC_INSTRUCTION  what the harness is asked to do
  CLADENC_MODEL        the name of the model, the value of --model
  CLADENC_MODEL_URL    the address the model is served at, the value of
                       --model-url
  CLADENC_MODEL_KEY    the key the model is called with, read from the file
                       that --model-key-file names, and not set where that
                       flag is not given

In a procreation the command runs as the user cladenc runs as, and in an
attempt as the user of the environment of the task, or as the user that
the task.toml of the task names. It is given nothing on standard input,
and its standard output and standard error are kept.

cladenc calls no model. A harness calls the model itself, at the address it
is told. In every container, host.docker.internal names the machine cladenc
runs on, so a model served on that machine at port 11434 is named by

    --model-url http://host.docker.internal:11434

and is reached as long as its server listens on an address a container can
reach. The key is read from its file when a directed evolution is started,
and again each time the directed evolution is resumed. cladenc keeps the
key off every command line, out of the journal and out of what it prints.

A procreation is carried out by the harness that procreates.
/var/opt/cladenc/work holds a second copy of its source code, and the
instruction asks the harness to improve that copy: what the harness leaves
there is its child. /var/log/cladenc/attempts holds the finished attempts
of the harness at the non-holdout tasks, to read: one directory
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
  attempts     its attempts at the non-holdout tasks, laid out
               as /var/log/cladenc/attempts is
  procreation  what the harness that made it printed, in stdout and
               stderr, with the id of that harness in the file named
               progenitor and one word in the file named outcome:
               created, unchanged, failed, build_failed or timed_out
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

A harness procreates only once it has been benchmarked in full, on every
task of every benchmark of the directed evolution, so that it finds every
non-holdout task in /var/log/cladenc/attempts. The primogenitor is
benchmarked in full before the first procreation.

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

A procreation starts only once no attempt is left to start, no attempt is
running and no other procreation is under way: every harness created so far
has been benchmarked in full, the child of the procreation before among
them. So cladenc chooses each harness that procreates with what every
created harness completed in hand. A singleton control group harness is not
waited for, since it never procreates: its creation and its attempts go on
beside a procreation. While a procreation runs, no container of another
child is started, so fewer containers run than --max-concurrent-containers
allows.

How the harness ends decides what is recorded:

  exit status 0     the procreation created a harness, or is recorded as
                    unchanged where the harness changed nothing
  exit status 125   Docker did not start the container: the directed
                    evolution stops, and `cladenc resume` carries out the
                    procreation again
  any other status  the procreation failed
  the time limit    the container is removed, and the procreation is
                    recorded as timed out

A procreation whose harness leaves /var/opt/cladenc/work as it was creates
no child, and is recorded as unchanged. cladenc compares what the harness
left with the source code of that harness, as it is kept under harnesses in
the outputs directory. The two are the same where they hold the same files,
directories and symbolic links, each file with the same contents and the
same permission bits, and each symbolic link with the same target. Such a
child would be its progenitor over again, and a model does not answer the
same way twice, so what the child completed would differ from what its
progenitor completed by luck alone. It is not built and not benchmarked, and
it is never chosen to procreate.

A child that holds anything but regular files, directories and symbolic
links is a failed procreation. The source code of every child is kept under
harnesses in the outputs directory, created or not. procreations in the
outputs directory keeps, in a directory named after the child, the
standard output and standard error of the harness that made it, the id of
that harness in the file named progenitor, and how the procreation ended
in the file named outcome.

At each procreation of a harness other than the primogenitor, cladenc
conducts an experiment to test whether a progenitor is better at
procreating better harnesses than the primogenitor. To conduct the
experiment, cladenc establishes a baseline: it has the primogenitor create
a harness from the same source code that the progenitor uses to procreate
the progenitor's child. The harness the primogenitor creates from the
source code of the progenitor is the singleton control group harness.
cladenc brings up one container more for it, in which

  /var/opt/cladenc/harness  holds a copy of the source code of the
                            primogenitor, which is what runs
  /var/opt/cladenc/work     holds a second copy of the source code of the
                            progenitor, which is what is changed

The primogenitor is given the instruction the progenitor is given, and what
the progenitor is given to read: under /var/log/cladenc/attempts the
attempts of the progenitor at the non-holdout tasks, under
/var/log/cladenc/line the harnesses the progenitor descends from, and
under /var/log/cladenc/children the children the progenitor procreated
before the progenitor's child of the experiment. What the primogenitor
leaves in /var/opt/cladenc/work is the singleton control group harness.
Where the primogenitor leaves /var/opt/cladenc/work as it was, no singleton
control group harness is created, and the creation is recorded as unchanged.

The singleton control group harness is built and benchmarked in full, as
the progenitor's child is, and it takes no other part in the directed
evolution: it is never chosen to procreate, it is counted in no clade, it
is not named the best harness, and the container it is created in does not
count toward --max-procreations. It is nobody's child, so no harness that
procreates is given anything of it. Where the primogenitor procreates, no
experiment is conducted: the progenitor's child and the singleton control
group harness would be created in the same way.

The source code of every singleton control group harness is kept under
harnesses in the outputs directory, created or not. baselines in the
outputs directory keeps, in a directory named after the singleton control
group harness, the standard output and standard error of the primogenitor,
the id of the progenitor in the file named progenitor, the id of the
progenitor's child in the file named child, and how the creation ended in
the file named outcome.

Where --harness-build-command is given, a harness is built before it is
run for the first time, in a container of the same image, as

    sh -c <TIMER> <COMMAND> <SECONDS>

where <COMMAND> is the value of that flag and the timer is the same one,
with /var/opt/cladenc/harness as the working directory, which holds the
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

  instruction.md  a file of text, the task specification, which says what
                  is to be done
  task.toml       a file of text, the task configuration of Harbor, which
                  says how the task is to be attempted
  tests           a directory that holds test.sh, an executable file, the
                  grader of the task, and whatever else the grader uses
  environment     a directory that holds a file named Dockerfile, which
                  describes the environment of the task, the image its
                  attempts are carried out in with the files the work
                  starts from, and whatever else the Dockerfile uses

as in Harbor's task layout, and may hold more, such as solution, which no
container is ever given. A file that is in the benchmark and in no task is
left alone. A benchmark that holds no task is refused, and so is one with
a task that lacks one of the four, or whose environment directory holds a
Compose file, such as docker-compose.yaml: an environment of more than one
container cannot be built. A task whose task.toml names an image needs
neither the Dockerfile nor the environment directory.

A task whose grader runs in a container of its own, as said below, is
refused as well for a Compose file in its tests directory. Where the image
of that container holds the grader, the task needs no test.sh.

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

--selection-tasks-percentage gives the part of the tasks of each benchmark
that are selection tasks, as so many in a hundred: a number above 0 and
below 100, such as 45 or 12.5. --corroboration-tasks-percentage gives the
part that are corroboration tasks, in the same way. The selection tasks and
the corroboration tasks together are the holdout tasks. The holdout tasks
are the tasks that no progenitor reads about, and the tasks that are not
held out are the non-holdout tasks. cladenc draws the selection tasks and
the corroboration tasks at random when a directed evolution is started, out
of each benchmark by itself: the number of its tasks times the percentage,
divided by 100 and rounded to the nearest whole number, a half rounded up.
At 45 and 15, a benchmark of 500 tasks has 225 selection tasks, 75
corroboration tasks and 200 non-holdout tasks.

A --selection-tasks-percentage that leaves a benchmark no selection task is
refused, and so is a --corroboration-tasks-percentage that leaves a
benchmark no corroboration task, and so are two values that together leave
a benchmark no non-holdout task. So a benchmark holds three tasks at the
least, and a benchmark of fewer is refused. The draw is made once for a
directed evolution and written in its journal, and `cladenc resume` reads
it there and draws nothing.

Every harness is benchmarked on the holdout tasks as on the non-holdout
tasks, selection tasks and corroboration tasks alike, and nothing a harness
is given in an attempt says which of the sets its task is in. A harness
that procreates is given its attempts at the non-holdout tasks, and nothing
of the holdout tasks: no task specification, no work, no result, and no
count of them.

Artificial selection goes by the selection tasks and by no others. cladenc
chooses which harness procreates by the selection tasks that the harness
and its descendants completed and did not complete, and it names as the
best harness the created harness, benchmarked in full, that completed the
most selection tasks: of several that completed as many, the one created
first. The non-holdout tasks are what a harness reads to find
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

`cladenc report` is for a directed evolution that has ended, and
`cladenc status` is for one that has not ended. Each refuses the other
kind, and says which of the two to call. When a directed evolution ends,
`cladenc start` prints on standard error what `cladenc report` prints,
after the line that says that the directed evolution finished, and prints
on standard output the directory of the best harness and nothing else.

A directed evolution that has not ended has no best harness. In its place
`cladenc status` names the harness that is ahead so far: the harness that
would be named the best harness if the directed evolution ended now. What
`cladenc status` prints is the state so far, which can change until the
directed evolution ends: another harness can be ahead later, and the lead
of the harness that is ahead so far can grow or shrink.

`cladenc report` says three things of the best harness and the
primogenitor, counted on the corroboration tasks: how many of them only the
best harness completed, how many only the primogenitor completed, and how
likely luck alone gives a lead at least as large. A corroboration task that
both harnesses completed, or that neither completed, says nothing about
which of the two is ahead, and is left out. Where two harnesses are equally
good, a task that only one of them completed is as likely the one's as the
other's, like the toss of a fair coin. So the likelihood is that of as many
heads or more in as many tosses as there are tasks that tell the two apart:
with 14 tasks that only the best harness completed and 6 that only the
primogenitor completed, 14 heads or more in 20 tosses, which is 0.058,
about 6 times in a hundred. Where no task tells the two apart the
likelihood is 1, and where the best harness is the primogenitor there are
no two harnesses to compare. cladenc draws no line through the likelihood:
what counts as more than luck is for the person who reads it to say. The
likelihood is shown for the best harness alone, since among many harnesses
one or two show a low likelihood by luck.

`cladenc status` says the same three things of the harness that is ahead so
far and the primogenitor. A person who calls `cladenc status` many times
reads many likelihoods, and one of many likelihoods is low by luck more
often than one that is read once. Each of them is the state so far, and not
the result of the directed evolution.

`cladenc report` and `cladenc status` say of the experiments how many were
conducted, in how many the progenitor's child was created, in how many the
singleton control group harness was created, and in how many both were.
Over the experiments where both were created, each counts the corroboration
tasks that only the progenitor's child completed and those that only the
singleton control group harness completed, and says how likely luck alone
gives a lead at least as large where the two are created equally well,
counted as it is for the best harness and the primogenitor. A corroboration
task that one of the two has not attempted yet is left out. In the table,
the row of a singleton control group harness names no progenitor, and names
under experiment the progenitor's child of its experiment. The singleton
control group harnesses are left out of the count of the tasks that at
least one harness completed. Where no experiment created both the
progenitor's child and the singleton control group harness, each says so in
the place of those counts and of the likelihood, and --json gives no
likelihood for the experiments.

How many more selection tasks the best of many harnesses that are all alike
completes is measured by a directed evolution in which every harness works
as the primogenitor does: one carried out with the values of the directed
evolution it is set beside, and with a run command that, where the harness
is started for a procreation, calls no model and adds one line to a file
that no harness reads, as

    --harness-run-command 'if [ -d /var/log/cladenc/attempts ]
        then echo >> procreated; else <COMMAND>; fi'

where <COMMAND> is the run command of the other, and procreated is a name
that no file of the primogenitor has. /var/log/cladenc/attempts is there for
a procreation and not for an attempt, so every procreation calls no model,
and creates a child that is the source code of its progenitor with one line
more in the file named procreated. The line is added because a procreation
that changes nothing creates no child. Such a directed evolution holds as
many children as --max-procreations gives, each of them benchmarked in full.
In `cladenc report` of it, how far apart the counts of the harnesses lie is
what luck does to the count of one harness, and how many more selection
tasks the best harness completed than the primogenitor is what luck and the
choosing give together, with nothing changed in how a harness works. The
best harness of the other directed evolution is ahead of its primogenitor on
the selection tasks by a change only as far as it is ahead by more than
that. `cladenc report` also says how many tasks at least one harness
completed, of the selection tasks, of the corroboration tasks and of the
non-holdout tasks, and `cladenc status` says how many so far. Where every
harness works as the primogenitor does, that is what one more attempt at
each task for each child gives the primogenitor, and it is set beside the
count of the best harness of the other directed evolution, which had one
attempt at each task.

An attempt is one harness working on one task, once, and cladenc carries it
out in two steps, in one container of the environment of the task, started
as

    sleep <SECONDS>

  <SECONDS>  the time limit of the harness and the time limit of the
             grader together, the longest the health check of the task
             can take, and ten minutes more, for what cladenc copies in
             and out

First the source code of the harness is copied into that container, at
/var/opt/cladenc/harness, and the harness is started there, as it is for a
procreation, in the working directory of the environment and as its user:
the user and the working directory its Dockerfile names, and root and /
where it names none. CLADENC_INSTRUCTION holds the task specification. The
harness is given nothing else of the task, so it can neither read nor
change the grader. Before the harness is started, cladenc makes

  /logs            a symbolic link to /var/log/cladenc/harbor
  /logs/artifacts  an empty directory, for what the harness saves to be
                   kept with the attempt

When the harness has ended, with whatever exit status, the tests directory
of the task is copied into the same container, and its grader is run there,
in the same working directory and as the same user, unless the task.toml of
the task names a user for the grader, as

    sh -c <TIMER> /tests/test.sh <SECONDS>

  /tests          a symbolic link to /var/opt/cladenc/harbor/tests, which
                  holds a copy of the tests directory of the task, and
                  nothing that the harness left there
  /logs/verifier  an empty directory, for the score of the grader

The grader finds the container as the harness left it: the files the
harness left outside /tests and /logs/verifier, and the processes it left
running, are there as well. A grader calls no model, and is given none of
the three variables of the model. The container is removed once the attempt
has ended.

/logs and /tests are the two names that Harbor's task layout fixes at the
top of the filesystem. cladenc makes no directory there, as the Filesystem
Hierarchy Standard asks of an application: each of the two is a symbolic
link. /logs is a link to /var/log/cladenc/harbor, since /var/log/cladenc
holds the records cladenc keeps, and /tests is a link to
/var/opt/cladenc/harbor/tests, since /var/opt/cladenc holds what cladenc
gives a container to work with. So /logs/artifacts is
/var/log/cladenc/harbor/artifacts, and /logs/verifier is
/var/log/cladenc/harbor/verifier. While the harness runs, the container
holds neither /tests nor the directory that /tests is a link to. Where the
image of the environment already holds something at /logs, cladenc makes no
link and leaves what is there, and makes /logs/artifacts and /logs/verifier
inside it. What the image or the harness left at /tests is removed before
the link is made.

The harness has the time limit that the task.toml of the task gives, as
timeout_sec under agent, and no time limit where it gives none, as in
Harbor. A harness that runs past its time limit is stopped, and every
process it left running is ended, as by `kill -9 -1`. What it left is
judged all the same. The grader has a time limit of its own, timeout_sec
under verifier, or 600 seconds where the task gives none, counted from the
moment the harness ended or was stopped. No flag gives a time limit of an
attempt. Where a task gives its harness no time limit, the container of an
attempt does not end itself: a cladenc that was killed outright leaves it
running until `cladenc resume` removes it, or a person does.

Where the task.toml of a task gives a health check, as command under
environment.healthcheck, cladenc runs that command in the container before
it copies the source code of the harness in, under the same timer, as the
user of the environment and in its working directory. The harness is
started only once the command has exited with status 0. A run of the
command has the time limit timeout_sec, or 30 seconds where the task gives
none. A run that exits with another status, or runs past its time limit,
has failed, and the command is run again after interval_sec, or 5 seconds
where the task gives none. The health check has not passed once the
command has failed as many times as retries says, or 3 times where the
task gives none. A failure within start_period_sec of the first run, or 0
seconds where the task gives none, is not counted, and the command is then
run again after start_interval_sec, or 5 seconds where the task gives none.
Where the health check does not pass, the harness is not started and the
grader is not run: the harness did not complete the task, and cladenc says
that the environment of the task did not pass its health check.

Where the task.toml of a task lists paths under artifacts, cladenc keeps
each with the attempt, a file or a directory with all it holds, as the
harness left it: it copies them out of the container once the harness has
ended or was stopped, before the tests directory is copied in. An entry of
the list is a path in the container, or a table that names the path as
source. The path is kept in artifacts in the directory of the attempt, at
the place the table gives as destination, and at the path itself without its
leading / where the table gives none or the entry is a path alone:
/app/out/report.json is kept as artifacts/app/out/report.json. A symbolic
link at the path is followed. A path that is not in the container keeps
nothing, and the attempt goes on. Where two entries would be kept at the
same place, or one inside the other, the first is kept and the other is not.

Of a directory, what the table lists as exclude is left out. Each is a
pattern as `tar --exclude` takes one: it leaves out a file or a directory
whose name it matches, with all the directory holds, and a pattern with a /
in it is matched against the path of the file from the directory that is
kept, or from any directory inside it, so that logs/run.txt leaves out
run.txt in every directory named logs. In a pattern, * stands for any run of
characters, / among them, ? for any one character, the characters between [
and ] for any one of them, and a character after \\ for itself.

What a harness left in /logs/artifacts is kept as a path that the task lists
is, as artifacts/logs/artifacts, though the task lists no such path, as in
Harbor, and nothing is kept where the harness left nothing there. cladenc
makes the directory before the harness is started, as said above, and it
belongs to the user the harness runs as, so that a harness that does not run
as root saves a file there as well. A task that lists /logs/artifacts
itself, or a path that is kept at the same place, has that entry obeyed in
its place.

Where the task.toml of a task lists commands under verifier.collect, cladenc
runs each in the container of the harness once the harness has ended or was
stopped, before it keeps any path, in the order the task lists them. A
command is run as the harness is, under the same timer and in the working
directory of the environment, as the user its entry names as user, or the
user of the environment where it names none. It has the time limit that its
entry gives as timeout_sec, or 60 seconds where it gives none. A command
that exits with another status than 0, or runs past its time limit, does not
end the attempt: the next command is run, and the paths are kept as they
then are. collect in the directory of the attempt keeps what each command
printed, in a directory named after the place of the command in the list,
counted from 1.

Where the task.toml of a task says that its grader runs separately, cladenc
runs the grader in a container of its own, which holds nothing of the
container of the harness but the paths the task lists under artifacts. A
task says so with environment_mode under verifier, written as separate, or
with a table named environment under verifier, which says so by being there.
A task that gives neither, or writes environment_mode as shared, has its
grader run in the container of the harness, as said above.

The container of the grader is a container of the first of these that the
task gives:

  1. the image that docker_image under verifier.environment names, which
     has to be on the machine, as an image that a task names for its
     environment has to be
  2. an image that cladenc builds from the Dockerfile in the tests
     directory of the task, with that directory as the build context
  3. the environment of the task, as it is before a harness works in it

An image of 1 or 2 holds the grader, at /tests/test.sh: cladenc copies no
tests directory into its container and makes nothing at /tests, and the task
needs no test.sh. Into a container of 3 the tests directory of the task is
copied, and /tests is a symbolic link to the copy, as said above. cladenc
builds an image of 2 when it builds the environments of the tasks, and as it
builds those, under the name of the environment of the task with -grader
after it, and removes it when it removes them.
environments/<BENCHMARK>/<TASK>/grader in the outputs directory keeps what
Docker printed. An image of 1 that is not on the machine, and an image of 2
that is not built, stop the directed evolution with exit status 2.

Where the task.toml holds the table environment under verifier, the
container of the grader follows that table and nothing of environment:
workdir, env, cpus, memory_mb, healthcheck and build_timeout_sec are obeyed
there as they are under environment, and a setting the table leaves out is
left out. Where the task.toml holds no such table, the container of the
grader follows environment. What is refused under environment is refused
under verifier.environment as well.

Once the harness has ended or was stopped, and the paths under artifacts are
kept, cladenc removes the container of the harness and brings up that of the
grader, so that an attempt is one container at a time. Where the settings of
the grader give a health check, cladenc runs it there as it runs that of the
task, and nothing else is done in the container until the health check has
passed. /logs is then made the symbolic link that it is in the
container of the harness, and each path that was kept is copied to the same
path in the container of the grader, the path of its entry and not
destination, without what exclude left out. A directory takes the place of
what the image holds at that path.

Where the health check of the container of the grader does not pass, no
grader has judged what the harness left, so cladenc does not record that the
harness did not complete the task. It removes the container, waits, brings
up a new container of the grader and runs the health check there again, as
many times as --max-grader-health-check-retries gives. Before the first
retry it waits interval_sec of the health check, or 5 seconds where the task
gives none, and before each retry after that twice as long as before the
last: 5 seconds, then 10, then 20. It says each retry on standard error.
Where the health check passes in no container, the directed evolution stops
with exit status 1, and no result is recorded for the attempt. cladenc then
names the harness and the task, says that the health check of the container
of the grader did not pass and in how many tries, and names the directory
that holds what the health check printed the last time it ran. That
directory is under running in the outputs directory, and stays there until
`cladenc resume` carries out the attempt again.

What is copied in is not trusted. No file keeps a setuid or a setgid bit. A
file that is not a regular file, a directory or a symbolic link is left out,
and so is a symbolic link that points outside the path it was kept with.
What is copied in belongs to the user the grader runs as.

Then the tests directory is copied in where the image holds no grader,
/logs/verifier is emptied, and the grader is run as said above, in the
working directory and as the user of its own container, unless the task.toml
names a user for the grader. Its time limit is counted from then. The
container of the grader is started as

    sleep <SECONDS>

  <SECONDS>  the time limit of the grader, the longest the health check
             of the grader can take, and ten minutes more

and is removed once the grader has ended. The image of a grader has to hold
what the environment of a task has to: sh, sleep, rm, mkdir, chown and ln.

The container of an attempt is given no more CPUs and no more memory than
its task asks for, as --cpus and --memory of `docker run`, so the harness
and the grader have them between them. A task that asks for neither is
given what Docker gives any container. The older spelling of the memory,
memory under environment, a number and one of G, M and K, such as 4G, is
read as Harbor reads it, and a task that gives both spellings with sizes
that differ is refused. cladenc asks Docker what the machine has, as

    docker info --format '{{.NCPU}} {{.MemTotal}}'

and a task that asks for more CPUs or more memory than that stops the
directed evolution with exit status 2, when it is started and each time it
is resumed, with the task named.

The task.toml of a task is the task configuration of Harbor. cladenc obeys
what it says of the environment of the task:

  docker_image under environment  the image that is the environment, in
                                  place of one built from the Dockerfile
  workdir under environment       the working directory of the harness
                                  and of the grader, in place of the one
                                  the image names
  env under environment           variables that the container is given,
                                  so the harness and the grader alike
  user under agent                the user the harness runs as, a name or
                                  a number, in place of the user the
                                  image names
  user under verifier             the user the grader runs as, in the
                                  same way
  env under verifier              variables that the grader alone is
                                  given
  timeout_sec under agent         the time limit of the harness, in
                                  seconds
  timeout_sec under verifier      the time limit of the grader, in
                                  seconds
  environment_mode under          separate where the grader runs in a
  verifier                        container of its own, and shared where
                                  it runs in that of the harness
  environment under verifier      what environment says of the container
                                  of the harness, said of the container
                                  of the grader
  collect under verifier          commands that are run in the container
                                  once the harness has ended, before the
                                  paths under artifacts are kept
  build_timeout_sec under         the time limit of the build of the
  environment                     environment, in seconds
  cpus under environment          the most CPUs the container of an
                                  attempt is given, a whole number or a
                                  part of one
  memory_mb under environment     the most memory that container is
                                  given, in megabytes
  healthcheck under environment   a command that has to pass before the
                                  harness is started, and how often it
                                  is run
  artifacts                       paths in the container that are kept
                                  with the attempt once the harness has
                                  ended

cladenc obeys nothing else of a task.toml. A setting that is left out, or
written with nothing between its quotation marks, changes nothing. The
source code of the harness and /logs/artifacts belong to the user the
harness runs as, and the copy of the tests directory and /logs/verifier
belong to the user the grader runs as.

In the value of a variable, ${NAME} stands for the value that the variable
NAME has where cladenc runs, and ${NAME:-TEXT} for that value, or for TEXT
where NAME is not set there. A variable that cladenc sets for a harness,
such as CLADENC_MODEL, keeps the value cladenc gives it. cladenc reads the
values when the directed evolution is started, and again each time it is
resumed, and writes none of them in the journal. It keeps them off every
command line: Docker reads them from its standard input, one line for each
variable, so the name of a variable is made of letters, digits, dots,
underscores and hyphens, and its value is one line of text.

A task is refused when the directed evolution is started where it holds no
task.toml, or where its task.toml is not valid as the task configuration of
Harbor. A task.toml is not valid where it is not TOML, where it gives a
setting a value of another kind than the setting takes, or a value that is
not one of those the setting takes, where a table lacks a key it has to
hold, such as command under environment.healthcheck, or where it says under
verifier that the grader shares the environment of the harness, as
environment_mode, and gives the grader an environment of its own, as
environment. cladenc checks every setting that Harbor's documentation of the
task configuration names, those it does not obey among them, and lets
through a setting that the documentation does not name.

A task is also refused where its task.toml names a variable otherwise, or
takes a value from a variable that is not set where cladenc runs and gives
no text in its place. A time limit that is not a number of seconds above 0
is refused in the same way, and so are CPUs or megabytes of memory that are
not a number above 0. Of a health check, timeout_sec is refused where it is
not a number of seconds above 0, retries where they are not a number above
0, and interval_sec, start_period_sec and start_interval_sec where they are
not a number of seconds, 0 or more. Of an entry under artifacts, the path is
refused where it holds .., and destination where it begins with /, holds ..,
or names no file or directory.

A task is refused as well where its task.toml asks for what cladenc has no
way to give: several steps, as [[steps]]; a GPU, as gpus under environment
above 0, or a TPU, as tpu; Windows, as os; MCP servers, as mcp_servers; a
skills directory, as skills_dir; a path or a command of a container other
than that of the harness, as service of an entry under artifacts or under
verifier.collect that is not main; or the whole container of the harness in
the container of the grader, as an entry under artifacts whose path names no
file or directory, where the grader runs separately. The refusal names the
task and what it asks for.

The grader writes its score in /logs/verifier/reward.txt, a single number,
or in /logs/verifier/reward.json, an object of labelled numbers, which is
read in place of reward.txt where both are there. /tests and
/logs/verifier are the paths of Harbor's task layout, which a grader
expects.

How an attempt ends decides its result:

  a score of 1, or 1 for every     the harness completed the task
  label of reward.json
  any other score, a file that     the harness did not complete the task
  cannot be read, or no score
  a health check of the task that  the harness is not started, and did not
  does not pass                    complete the task
  a health check of the container  no result is recorded: the directed
  of the grader that does not      evolution stops, and `cladenc resume`
  pass in any of its tries         carries out the attempt again
  exit status 125 of either step   Docker did not start the container: the
  or of the health check           directed evolution stops, and `cladenc
                                   resume` carries out the attempt again
  the time limit of the grader     the container is removed, and the
                                   harness did not complete the task,
                                   which is recorded as timed_out

The exit status of the grader decides nothing more.

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
container as the harness left it, and everything there was the harness's
to change: tests, build files, any program the grader starts from the
working directory, and the processes the harness left running. A grader
that runs the tests it finds in the working directory is passed by a
harness that changed those tests, on a holdout task as on any other. The
tests directory of the task is never given to a harness: a grader that
keeps what it relies on there, and takes nothing from the working
directory on trust, can be passed that way only by a harness that left a
process running to change the copy of that directory, or the score, once
they are made.

The model may know the task. A task taken from a public source may have
been among what the model was trained on. A harness then completes it from
what the model remembers, and a child that completes more of such tasks
may only be better at drawing that out.

attempts in the outputs directory keeps every attempt at a task that is not
held out, in a directory named after the harness, in it one named after the
benchmark, and in that one named after the task, which holds

  result         one word: completed, not_completed or timed_out
  ending         one word: ended, stopped where the harness ran past its
                 time limit, or not_started where the environment did
                 not pass its health check
  specification  the task specification
  work           what the harness left in the working directory of the
                 environment, not kept where that directory is /
  changes        every path the harness added, changed or removed in the
                 container, one to a line after A, C or D
  artifacts      what the harness left at the paths the task lists under
                 artifacts, where it lists any, and in /logs/artifacts
  collect        what each command under verifier.collect printed, in a
                 directory named after its place in the list, where the
                 task lists any
  stdout         the standard output of the harness
  stderr         the standard error of the harness
  grader         the standard output and the standard error of the grader,
                 in stdout and stderr, and in logs/verifier what the grader
                 left in /logs/verifier, its score among them; where the
                 grader ran in a container of its own with a health check,
                 what the last run of that printed, in health-check
  health-check   the standard output and the standard error of the last
                 run of the health check, in stdout and stderr, where the
                 task gives one

An attempt whose harness was not started keeps result, ending,
specification and health-check, and nothing else.

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
that container. Docker removes a container of cladenc once it has ended.

Once the containers are removed, `cladenc resume` builds the environment
of every task again, from the copy of the task in the outputs directory
and under the name it was given when the directed evolution was started,
before it brings up any container. The image that the Dockerfile of each
task starts from has to be on the machine: Docker fetches one that is
not, from the registry of its name. A task whose environment is not built
stops `cladenc resume` with exit status 2, and
environments/<BENCHMARK>/<TASK> in the outputs directory keeps what Docker
printed.

`cladenc resume` reads the task.toml of the copy of each task again, and
the variables it names as they are set where `cladenc resume` runs. For a
task whose task.toml names an image it builds nothing, and looks for the
image on the machine. A task whose image is not there, or which takes a
value from a variable that is not set, stops `cladenc resume` with exit
status 2.

When the directed evolution that it continued ends, `cladenc resume` prints
on standard error what `cladenc report` prints, after the line that says
that the directed evolution finished, and prints on standard output the
directory of the best harness and nothing else. Of a directed evolution
that had ended already, it prints the same after a line that says that the
directed evolution had finished.";
