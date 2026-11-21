# CladeNC

CladeNC (recursive acronym for Clade Next Clade) is a cleanroom implementation of the [Huxley-Gödel Machine](https://arxiv.org/abs/2510.21614) for agentic coding harnesses. It works by directed evolution. A directed evolution starts from one harness, the primogenitor. Harnesses procreate, each making a changed copy of its own source code, and are benchmarked: each attempts every task of the benchmarks a person brings. A part of the tasks of each benchmark is held out: no harness that procreates reads about them. The holdout tasks are divided in two, the selection tasks and the corroboration tasks. Artificial selection chooses which harness procreates, and at the end names the best harness, both by the selection tasks alone. Every harness is scored on the corroboration tasks and nothing is chosen by them, so they say how far the best harness is ahead of the primogenitor on tasks that had no part in the choosing. At each procreation of a harness other than the primogenitor, `cladenc` also conducts an experiment to test whether a progenitor is better at procreating better harnesses than the primogenitor. To conduct the experiment, `cladenc` establishes a baseline by having the primogenitor create a harness from the same source code that the progenitor uses to procreate the progenitor's child. The harness the primogenitor creates from the source code of the progenitor is the singleton control group harness.

- [`cladenc`](#cladenc-1)
  - [`cladenc report`](#cladenc-report)
  - [`cladenc resume`](#cladenc-resume)
  - [`cladenc start`](#cladenc-start)
  - [`cladenc status`](#cladenc-status)

## `cladenc`

```text
cladenc \
  --help \
  --version
```

| Flag | Required | Description |
| --- | --- | --- |
| `--help` | Not Required | Print the sub-commands and the flags of `cladenc`. |
| `--version` | Not Required | Print the version of `cladenc`. |

`cladenc` does its work through a sub-command. Called with neither a sub-command nor a flag, it prints what `--help` prints, on standard error, and exits with status 2.

| Sub-command | Description |
| --- | --- |
| `cladenc report` | Prints the report of a directed evolution that has ended, without changing it. |
| `cladenc resume` | Continues a directed evolution that stopped before its end. |
| `cladenc start` | Starts a new directed evolution and prints the directory of its best harness. |
| `cladenc status` | Prints the state so far of a directed evolution that has not ended, without changing it. |

`--help` after a sub-command prints its flags.

### `cladenc report`

```text
cladenc report \
  --json \
  --outputs-dir <DIR>
```

| Flag | Required | Description |
| --- | --- | --- |
| `--json` | Not Required | Print one JSON object in place of the table. |
| `--outputs-dir <DIR>` | Required | The outputs directory of the directed evolution to report on. |

`cladenc report` is for a directed evolution that has ended. It reads the journal in the outputs directory and prints the report of the directed evolution on standard output, without changing it: the model, how many procreations were started, the best harness, how many of the selection tasks, of the corroboration tasks and of the other tasks at least one harness completed, and for each harness its progenitor, whether it was created, which reads `unchanged` where its procreation left the copy of the source code as it was, how many of the selection tasks it completed and did not complete, how many of the corroboration tasks it completed and did not complete, how many of the other tasks it completed and did not complete, the counts of the selection tasks for its clade, which is the harness and all its descendants together, and how many seconds the harness ran in all its attempts together and how many its progenitor ran in the procreation that made it. Nothing is chosen by the corroboration tasks, so the counts of the corroboration tasks are where a person reads how far the best harness is ahead of the primogenitor. For the best harness against the primogenitor, `cladenc report` also prints how many corroboration tasks only the best harness completed, how many only the primogenitor completed, and how likely luck alone gives a lead at least as large where the two harnesses are equally good. Of the experiments that test whether a progenitor is better at procreating better harnesses than the primogenitor, `cladenc report` prints how many were conducted, in how many the progenitor's child was created, in how many the singleton control group harness was created, and in how many both were. Over the experiments where both were created, it prints how many corroboration tasks only the progenitor's child completed, how many only the singleton control group harness completed, and how likely luck alone gives a lead at least as large where the two are created equally well, counted as it is for the best harness against the primogenitor. Where no experiment created both the progenitor's child and the singleton control group harness, it says so in place of those counts and of the likelihood, and `--json` gives no likelihood for the experiments. A singleton control group harness has a row of its own in the table: the row names no progenitor, since the singleton control group harness is nobody's child, and names under `experiment` the progenitor's child of its experiment. The singleton control group harnesses are counted in no clade, and are left out of the count of the tasks at least one harness completed. Given a directed evolution that has not ended, `cladenc report` prints nothing on standard output: it says on standard error that the directed evolution has not ended and that `cladenc status` is the sub-command to call, and exits with status 2.

### `cladenc resume`

```text
cladenc resume \
  --outputs-dir <DIR>
```

| Flag | Required | Description |
| --- | --- | --- |
| `--outputs-dir <DIR>` | Required | The outputs directory of the directed evolution to continue. |

`cladenc resume` continues a directed evolution that stopped before its end, with the values that `cladenc start` was given, and prints the directory of the best harness on standard output once the directed evolution has ended, and nothing else there. It says on standard error what it is doing, and when the directed evolution ends it prints there what `cladenc report` prints, as `cladenc start` does. It is stopped in the way `cladenc start` is, and then prints no report. Of a directed evolution that had ended already, it prints the directory of the best harness and the report again and does nothing else.

1. It removes the containers that the directed evolution left running.
2. It starts again each procreation, each creation of a singleton control group harness and each attempt that was under way when the directed evolution stopped.
3. It carries the directed evolution on to its end, as `cladenc start` does.

### `cladenc start`

```text
cladenc start \
  --attempt-timeout <SECONDS> \
  --benchmark <DIR> \
  --docker-build-timeout <SECONDS> \
  --docker-image <IMAGE> \
  --dockerfile <FILE> \
  --harness-build-command <COMMAND> \
  --harness-run-command <COMMAND> \
  --holdout-corroboration-percentage <NUMBER> \
  --holdout-percentage <NUMBER> \
  --max-concurrent-containers <COUNT> \
  --max-procreations <COUNT> \
  --model <NAME> \
  --model-key-file <FILE> \
  --model-url <URL> \
  --outputs-dir <DIR> \
  --primogenitor <DIR> \
  --procreation-timeout <SECONDS>
```

| Flag | Required | Description |
| --- | --- | --- |
| `--attempt-timeout <SECONDS>` | Required | The time limit of one attempt, which is one harness working on one task, once. |
| `--benchmark <DIR>` | Required | The directory of a benchmark, which holds a directory for each task. The directory of a task holds its task specification in a file named `instruction.md`, its grader in an executable file named `test.sh` in a directory named `tests`, and the Dockerfile of its environment in a directory named `environment`, as in [Harbor's task layout](https://docs.harborframework.com/tasks/verifier). A task whose `environment` directory holds a Compose file, such as `docker-compose.yaml`, is refused, since an environment of more than one container cannot be built. A task's `solution` directory is never given to a container. The flag is given once for each benchmark of the directed evolution. `cladenc` fetches no benchmark: the tasks are read from the directory and copied into the outputs directory. |
| `--docker-build-timeout <SECONDS>` | Not Required | The time limit of the build of the image. It is given with `--dockerfile`, and not without it. |
| `--docker-image <IMAGE>` | Not Required | The Docker image every container is started from. Either this flag or `--dockerfile` is given, and not both. |
| `--dockerfile <FILE>` | Not Required | The Dockerfile that `cladenc` builds the image of every container from, in place of `--docker-image`. |
| `--harness-build-command <COMMAND>` | Not Required | The shell command that builds a harness in a container, where one has to be built. |
| `--harness-run-command <COMMAND>` | Required | The shell command that runs a harness in a container. |
| `--holdout-corroboration-percentage <NUMBER>` | Required | The percentage of the holdout tasks of each benchmark that are corroboration tasks, a number above 0 and below 100, such as 25 or 12.5. `cladenc` scores every harness on the corroboration tasks and chooses nothing by them. It draws them at random out of the holdout tasks of each benchmark, and the holdout tasks that are left are the selection tasks. It refuses a value that leaves a benchmark no corroboration task or no selection task, so a benchmark holds three tasks at the least. |
| `--holdout-percentage <NUMBER>` | Required | The percentage of the tasks of each benchmark that is held out, a number above 0 and below 100, such as 60 or 12.5. The holdout tasks are the tasks that no harness reads about when it procreates. `cladenc` draws them at random out of each benchmark, and refuses a value that leaves a benchmark no holdout task or no other task. `--holdout-corroboration-percentage` divides them in two, and `cladenc` chooses which harness procreates, and names the best harness, by the selection tasks alone. |
| `--max-concurrent-containers <COUNT>` | Required | The most containers running at once. |
| `--max-procreations <COUNT>` | Required | The most procreations to start, 1 or more. A procreation is one harness making another, and it counts once it is started, whether or not it creates its child. The container a singleton control group harness is created in is no procreation and is not counted. |
| `--model <NAME>` | Required | The name of the model the harnesses call, as its server knows it. `cladenc` calls no model itself: it tells each harness the name, the address and the key. |
| `--model-key-file <FILE>` | Not Required | The file that holds the key the model is called with, on one line, and nothing else. It is given where the server of the model asks for a key. `cladenc resume` reads the file again. |
| `--model-url <URL>` | Required | The address the model is served at, by a provider or by a server of your own. In a container, `host.docker.internal` names the machine `cladenc` runs on. |
| `--outputs-dir <DIR>` | Required | The outputs directory, which `start` creates. |
| `--primogenitor <DIR>` | Required | The directory of source code that is copied to make the primogenitor, the harness the directed evolution starts from. |
| `--procreation-timeout <SECONDS>` | Required | The time limit of one procreation, which is one harness making another. |

`cladenc start` starts a new directed evolution, carries it to its end, and prints the directory of the best harness on standard output, and nothing else there. It says on standard error what it is doing, and when the directed evolution ends it prints there what `cladenc report` prints: the table of the harnesses and the lines above the table, which say how far the best harness is ahead of the primogenitor on the corroboration tasks and how likely luck alone gives a lead at least as large. Ctrl-C, or one of the signals `SIGINT`, `SIGTERM`, `SIGQUIT` and `SIGHUP`, stops it sooner: it removes the containers it has running, prints no report and exits with status 1, and `cladenc resume` continues the directed evolution from there. While the directed evolution has not ended, `cladenc status` prints its state so far.

1. It draws the holdout tasks at random: of the tasks of each benchmark, the percentage that `--holdout-percentage` gives. Out of the holdout tasks of each benchmark it draws the corroboration tasks, the percentage that `--holdout-corroboration-percentage` gives, and the holdout tasks that are left are the selection tasks.
2. It creates the outputs directory, and copies into it the source code of the primogenitor and the tasks of each benchmark.
3. Where `--dockerfile` is given, it builds the image that every container is started from.
4. Where `--harness-build-command` is given, it builds the primogenitor.
5. It benchmarks the primogenitor. For each task of each benchmark, held out or not, the harness works on the task in a container. Once the harness has ended, the task's `tests` directory is copied into the same container at `/tests`, and the grader, `/tests/test.sh`, writes its score in `/logs/verifier/reward.txt`, or one score for each label in `/logs/verifier/reward.json`. The harness completed the task where every score is 1, and what the grader leaves in `/logs/verifier` is kept with the attempt.
6. It starts as many procreations as `--max-procreations` gives, each only once every harness created so far has been benchmarked in full, so that it chooses each harness that procreates with what every created harness completed in hand. In each, a harness reads its attempts at the tasks that are not held out, and what was kept of the harnesses it descends from and of the children it made before, and makes a changed copy of its own source code, its child. A procreation in which the harness leaves the copy of its own source code as it was creates no child, and is recorded as unchanged. Each child that is created is benchmarked as the primogenitor was. `cladenc start` chooses which harness procreates among those that have been benchmarked in full, partly by chance, and more often one that completed, together with its descendants, a larger share of the selection tasks they attempted. A harness whose procreation does not create its child is left out of the choice for the next procreation, whichever harness procreates in it. Each time a procreation of the same harness again does not create its child, the harness is left out for twice as many procreations as the time before: 2, then 4, then 8. Once a procreation of the harness creates its child, the harness is left out no longer, and the count starts again at 1. When every created harness is left out, `cladenc start` chooses among the harnesses that are left out for the fewest procreations still to come.
7. At each procreation of a harness other than the primogenitor, it conducts an experiment to test whether a progenitor is better at procreating better harnesses than the primogenitor. To conduct the experiment, it establishes a baseline: it brings up one more container, in which the primogenitor creates a harness from the same source code that the progenitor uses to procreate the progenitor's child. The source code that runs in the container is a copy of the source code of the primogenitor, and the source code that is changed is a second copy of the source code of the progenitor. What is there to read is what the progenitor is given in its procreation: the attempts of the progenitor at the tasks that are not held out, and what was kept of the harnesses the progenitor descends from and of the children the progenitor procreated before. The harness the primogenitor creates from the source code of the progenitor is the singleton control group harness. Where the primogenitor leaves the second copy of the source code of the progenitor as it was, no singleton control group harness is created. It is built and benchmarked in full, as the progenitor's child is, and it takes no other part in the directed evolution: it is never chosen to procreate, it is counted in no clade, it is not named the best harness, and the container it is created in does not count toward `--max-procreations`. Where the primogenitor procreates, no experiment is conducted: the progenitor's child and the singleton control group harness would be created in the same way. No flag leaves the experiment out. Each experiment has one more harness created and benchmarked in full, so a directed evolution can bring up about twice the containers, and take about twice as long, as its procreations and the attempts of the primogenitor and the children alone would.
8. It names the best harness: the created harness that completed the most selection tasks, a singleton control group harness left aside.
9. It prints on standard error what `cladenc report` prints, and on standard output the directory of the best harness.

### `cladenc status`

```text
cladenc status \
  --json \
  --outputs-dir <DIR>
```

| Flag | Required | Description |
| --- | --- | --- |
| `--json` | Not Required | Print one JSON object in place of the table. |
| `--outputs-dir <DIR>` | Required | The outputs directory of the directed evolution to print the state of. |

`cladenc status` is for a directed evolution that has not ended, whether the directed evolution is still going or stopped before its end. It reads the journal in the outputs directory and prints the state of the directed evolution so far on standard output, without changing it. It says first that the directed evolution has not ended, and then prints what `cladenc report` prints, with the harness that is ahead so far in place of the best harness. The harness that is ahead so far is the harness that would be named the best harness if the directed evolution ended now: the created harness, benchmarked in full, that completed the most selection tasks. For the harness that is ahead so far against the primogenitor, `cladenc status` prints how many corroboration tasks only the harness that is ahead so far completed, how many only the primogenitor completed, and how likely luck alone gives a lead at least as large where the two harnesses are equally good. What `cladenc status` prints is the state so far, which can change until the directed evolution ends: another harness can be ahead later, and a person who calls `cladenc status` many times reads many likelihoods, of which one is low by luck more often than one likelihood that is read once. Of the experiments, a corroboration task that the progenitor's child or the singleton control group harness has not attempted yet is left out. Given a directed evolution that has ended, `cladenc status` prints nothing on standard output: it says on standard error that the directed evolution has ended and that `cladenc report` is the sub-command to call, and exits with status 2.
