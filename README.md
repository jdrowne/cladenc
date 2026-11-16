# CladeNC

CladeNC (recursive acronym for Clade Next Clade) is a cleanroom implementation of the [Huxley-Gödel Machine](https://arxiv.org/abs/2510.21614) for agentic coding harnesses.

- [`cladenc`](#cladenc-1)
  - [`cladenc report`](#cladenc-report)
  - [`cladenc resume`](#cladenc-resume)
  - [`cladenc start`](#cladenc-start)

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
| `cladenc report` | Prints the state of a search without changing it. |
| `cladenc resume` | Continues a search that stopped before its end. |
| `cladenc start` | Starts a new search and prints the directory of its best harness. |

`--help` after a sub-command prints its flags.

### `cladenc report`

```text
cladenc report \
  --json \
  --run-dir <DIR>
```

| Flag | Required | Description |
| --- | --- | --- |
| `--json` | Not Required | Print one JSON object in place of the table. |
| `--run-dir <DIR>` | Required | The run directory of the search to report on. |

### `cladenc resume`

```text
cladenc resume \
  --run-dir <DIR>
```

| Flag | Required | Description |
| --- | --- | --- |
| `--run-dir <DIR>` | Required | The run directory of the search to continue. |

`cladenc resume` says on standard error what it is doing, on lines that begin `cladenc:`. It says how many containers it removed, where the search left any running, how far the search had got and each procreation and evaluation it starts again, and from there what `cladenc start` says. Of a search that had finished it says that it had, and prints the directory of its best harness again. The shell sends standard error elsewhere, as it does for `cladenc start`: `2>/dev/null` after the command sends it nowhere.

The search goes on until its budget is spent. To stop it sooner, press Ctrl-C, or send `cladenc` one of the signals `SIGINT`, `SIGTERM`, `SIGQUIT` and `SIGHUP`, as `kill` does. `cladenc` then removes the containers it has running and exits with status 1, and a search that stopped again can be continued again.

A `cladenc` that was killed outright, by `SIGKILL`, ended none of its containers, and each goes on until its time limit has passed. Before it does anything else, `cladenc resume` removes the containers that the search left running, so that it never works beside one of them. Every container of a search carries the Docker label `cladenc.search`, whose value is the path of the run directory, and `cladenc resume` removes each container that carries the label of its search, once it has the run directory to itself. Where Docker cannot be asked, `cladenc resume` exits with status 2 and continues nothing.

Where the search is not resumed, its containers are removed by hand: `docker ps` lists them under names that begin `cladenc-`, and `docker kill` with a name ends that container. Docker removes a container of `cladenc` by itself once the container has ended.

A `cladenc resume` that is killed outright removes nothing either, and the next `cladenc resume` removes what it left.

### `cladenc start`

```text
cladenc start \
  --benchmark <DIR> \
  --budget <COUNT> \
  --confidence <PERCENT> \
  --docker-build-timeout <SECONDS> \
  --docker-image <IMAGE> \
  --dockerfile <FILE> \
  --evaluation-timeout <SECONDS> \
  --genealogy-growth <NUMBER> \
  --harness-build-command <COMMAND> \
  --harness-run-command <COMMAND> \
  --initial-procreations <COUNT> \
  --model <NAME> \
  --model-key-file <FILE> \
  --model-url <URL> \
  --procreation-timeout <SECONDS> \
  --progenitor <DIR> \
  --run-dir <DIR> \
  --seed <INTEGER> \
  --workers <COUNT>
```

| Flag | Required | Description |
| --- | --- | --- |
| `--benchmark <DIR>` | Required | The directory of a benchmark, which holds a directory for each task. The flag is given once for each benchmark of the search. `cladenc` fetches no benchmark: the tasks are read from the directory and copied into the run directory. |
| `--budget <COUNT>` | Required | The number of evaluations to start. |
| `--confidence <PERCENT>` | Required | The probability behind the lower bound that picks the best harness. |
| `--docker-build-timeout <SECONDS>` | Not Required | The time limit of the build of the image. It is given with `--dockerfile`, and not without it. |
| `--docker-image <IMAGE>` | Not Required | The Docker image every container is started from. Either this flag or `--dockerfile` is given, and not both. |
| `--dockerfile <FILE>` | Not Required | The Dockerfile that `cladenc` builds the image of every container from, in place of `--docker-image`. |
| `--evaluation-timeout <SECONDS>` | Required | The time limit of one evaluation, which is one harness tried on one task. |
| `--genealogy-growth <NUMBER>` | Required | How fast the genealogy may grow, from 0 to 1. |
| `--harness-build-command <COMMAND>` | Not Required | The shell command that builds a harness in a container, where one has to be built. |
| `--harness-run-command <COMMAND>` | Required | The shell command that runs a harness in a container. |
| `--initial-procreations <COUNT>` | Required | The number of procreations of the progenitor made before the first evaluation. A procreation is one harness making another. |
| `--model <NAME>` | Required | The name of the model the harnesses call, as its server knows it. `cladenc` calls no model itself: it tells each harness the name, the address and the key. |
| `--model-key-file <FILE>` | Not Required | The file that holds the key the model is called with, on one line, and nothing else. It is given where the server of the model asks for a key. `cladenc resume` reads the file again. |
| `--model-url <URL>` | Required | The address the model is served at, by a provider or by a server of your own. In a container, `host.docker.internal` names the machine `cladenc` runs on. |
| `--procreation-timeout <SECONDS>` | Required | The time limit of one procreation. |
| `--progenitor <DIR>` | Required | The directory of source code that is copied to make the progenitor, the harness the search starts from. |
| `--run-dir <DIR>` | Required | The run directory, which `start` creates. |
| `--seed <INTEGER>` | Required | The seed of every random draw. |
| `--workers <COUNT>` | Required | The most containers running at once. |

A benchmark is a set of tasks, and a task is one piece of coding work that a harness is tried on. `--benchmark` names a whole benchmark: a directory that holds one directory for each task. A benchmark of three tasks, in a directory named `parsing`:

```text
parsing/
  add-a-flag/
    check
    files/
    statement
  fix-the-parser/
    check
    files/
    statement
  rename-a-module/
    check
    files/
    statement
```

The name of a task's directory is the id of the task, and is made of letters, digits, dots, underscores and hyphens. In it, `statement` is a file of text that says what is to be done, `files` is a directory of the files the work starts from, and `check` is an executable file: the program that says by its exit status whether the work passes. The directory of a task may hold more, such as tests that the check runs.

`cladenc start` refuses a benchmark that holds no task, and one with a task that lacks its statement, its files or its check, and it says which benchmark and which task.

`--benchmark` is given once for each benchmark of the search. A search that is given `--benchmark parsing` and `--benchmark refactoring` tries its harnesses on the tasks of both, and returns one harness: the one it believes passes the largest share of all those tasks. Every task counts the same, so a benchmark with more tasks weighs more.

The name of a benchmark is the name of its directory, made of letters, digits, dots, underscores and hyphens. A task of a search is known by the name of its benchmark and its id together, as `parsing/fix-the-parser`, so two benchmarks may each hold a task of the same id. `cladenc start` refuses a benchmark that is given twice, and two benchmarks whose directories have the same name.

`cladenc start` copies the tasks into `benchmarks` in the run directory, each into a directory named after its benchmark, so a search keeps the tasks it started with, and `cladenc resume` does not read the directories that `--benchmark` named.

An evaluation is one harness tried on one task. `cladenc` starts the harness in a container, on a copy of the task's `files`, with the text of `statement` as what the harness is asked to do. The harness is given nothing else of the task, so it can neither read nor change the check. When the harness has ended, `cladenc` runs the task's `check` in a second container: there `/work`, the working directory, holds what the harness left, and `/task` holds the directory of the task, to read. A check that exits with status 0 is a success, and one that exits with any other status is a failure. `evaluations` in the run directory keeps what each evaluation left, in a directory named after the harness, the benchmark and the task, as `evaluations/3/parsing/fix-the-parser`.

Every container of a search is started from one image. `--docker-image` names an image that Docker already holds. `--dockerfile` names a Dockerfile, the text file Docker builds an image from, and `cladenc` builds the image itself, once, when the search is started, within the time limit that `--docker-build-timeout` gives.

`cladenc` fetches no image, and it has no flag for a registry, the server that Docker fetches an image from. The name of an image says which registry it is of, as `registry.example:5000/tools/harness:1` does, and a name that says none is of Docker Hub. The image that `--docker-image` names has to be on the machine before the search: fetched there with `docker pull`, carried there as a file with `docker save` and `docker load`, or built there. `cladenc` tells Docker never to fetch the image of a container, so a search whose image is not on the machine stops at its first container, with what Docker said kept in the run directory, and `cladenc resume` continues it once the image is there.

The Dockerfile is all that Docker is given. It can start from an image and run commands, and it has no file to copy into the image: a Dockerfile that copies files is built by hand, and its image is given with `--docker-image`. A Dockerfile whose base image is on the machine, and whose steps fetch nothing, is built with no connection to the internet. `cladenc` does not ask Docker to fetch a base image and cannot forbid it: Docker fetches a base image that is not on the machine, from the registry of its name.

`cladenc` names the image `cladenc-` and sixteen random hexadecimal digits, and says the name. `image` in the run directory keeps a copy of the Dockerfile and what Docker printed. An image that is not built stops `cladenc start` with exit status 2. The image stays on the machine when the search ends, and `cladenc resume` starts its containers from it and builds nothing.

The build of the image ends itself once its time limit has passed, as a container does, also when `cladenc` was killed outright and is not there to end it. `cladenc` starts `docker build` under `timeout`, with five seconds more than the time limit, so that `cladenc` comes first for as long as it is there. `timeout` is a program the machine has to hold, and on Debian and Ubuntu the package `coreutils` brings it. On a machine that does not hold it no image is built, and `cladenc start` stops with exit status 2.

`cladenc start` says on standard error what it is doing, each thing as it starts and as it ends, on lines that begin `cladenc:`. It says the run directory being made, the image being built where a Dockerfile is given, the progenitor being built where it has to be, every procreation and every evaluation with how it ended and where what it left is kept, and at the end the best harness. A procreation or an evaluation that stops the search is said when it happens, and so is the search being asked to stop. The shell sends standard error elsewhere: `2>/dev/null` after the command sends it nowhere, and `2>start.log` keeps it in the file `start.log`. The directory of the best harness is printed on standard output, which neither changes.

A search goes on until its budget is spent, and it is stopped sooner in the same way as one that `cladenc resume` continues: by Ctrl-C, or by one of the signals `SIGINT`, `SIGTERM`, `SIGQUIT` and `SIGHUP`. `cladenc` then exits with status 1, and `cladenc resume` carries the search on from where it stopped.

`cladenc` cannot listen for `SIGKILL`, the signal that `kill -9` sends, and that Linux sends a program when the machine runs out of memory. A `cladenc` that is ended by it is killed outright: it removes no container, and Docker goes on running each one. So that none outlives its time limit, `cladenc` starts the command of every container under a timer, a line of shell that ends the command once what was left of the time limit has passed, and five seconds more. A container therefore ends itself, whether `cladenc` is still there or not, and Docker removes it. The timer is made of `sh` and `sleep`, which the image has to hold. Until its time limit has passed, a container that was left goes on, unless `cladenc resume` continues the search: it removes the containers that the search left running before it does anything else.
