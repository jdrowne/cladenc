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
  --quiet \
  --run-dir <DIR>
```

| Flag | Required | Description |
| --- | --- | --- |
| `--quiet` | Not Required | Print nothing on standard error. |
| `--run-dir <DIR>` | Required | The run directory of the search to continue. |

The search goes on until its budget is spent. To stop it sooner, press Ctrl-C, or send `cladenc` one of the signals `SIGINT`, `SIGTERM`, `SIGQUIT` and `SIGHUP`, as `kill` does. `cladenc` then ends the jobs it has running and exits with status 1, and a search that stopped again can be continued again.

### `cladenc start`

```text
cladenc start \
  --benchmark <DIR> \
  --budget <COUNT> \
  --confidence <PERCENT> \
  --docker-image <IMAGE> \
  --evaluation-timeout <SECONDS> \
  --evaluator <FILE> \
  --genealogy-growth <NUMBER> \
  --harness-build-command <COMMAND> \
  --harness-run-command <COMMAND> \
  --initial-procreations <COUNT> \
  --model <NAME> \
  --model-key-file <FILE> \
  --model-url <URL> \
  --procreation-timeout <SECONDS> \
  --progenitor <DIR> \
  --quiet \
  --run-dir <DIR> \
  --seed <INTEGER> \
  --workers <COUNT>
```

| Flag | Required | Description |
| --- | --- | --- |
| `--benchmark <DIR>` | Required | The directory of the benchmark, which holds a directory for each task. `cladenc` fetches no benchmark: the tasks are read from this directory and copied into the run directory. |
| `--budget <COUNT>` | Required | The number of evaluations to start. |
| `--confidence <PERCENT>` | Required | The probability behind the lower bound that picks the best harness. |
| `--docker-image <IMAGE>` | Required | The Docker image every container is started from. |
| `--evaluation-timeout <SECONDS>` | Required | The time limit of one evaluation. |
| `--evaluator <FILE>` | Required | The program that carries out an evaluation, which is one harness tried on one task. |
| `--genealogy-growth <NUMBER>` | Required | How fast the genealogy may grow, from 0 to 1. |
| `--harness-build-command <COMMAND>` | Not Required | The shell command that builds a harness in a container, where one has to be built. |
| `--harness-run-command <COMMAND>` | Required | The shell command that runs a harness in a container. |
| `--initial-procreations <COUNT>` | Required | The number of procreations of the progenitor made before the first evaluation. A procreation is one harness making another. |
| `--model <NAME>` | Required | The name of the model the harnesses call, as its server knows it. `cladenc` calls no model itself: it tells each harness the name, the address and the key. |
| `--model-key-file <FILE>` | Not Required | The file that holds the key the model is called with, on one line, and nothing else. It is given where the server of the model asks for a key. `cladenc resume` reads the file again. |
| `--model-url <URL>` | Required | The address the model is served at, by a provider or by a server of your own. In a container, `host.docker.internal` names the machine `cladenc` runs on. |
| `--procreation-timeout <SECONDS>` | Required | The time limit of one procreation. |
| `--progenitor <DIR>` | Required | The directory of source code that is copied to make the progenitor, the harness the search starts from. |
| `--quiet` | Not Required | Print nothing on standard error. |
| `--run-dir <DIR>` | Required | The run directory, which `start` creates. |
| `--seed <INTEGER>` | Required | The seed of every random draw. |
| `--workers <COUNT>` | Required | The most jobs running at once. |

A benchmark is a set of tasks, and a task is one piece of coding work that a harness is tried on. `--benchmark` names the whole benchmark: a directory that holds one directory for each task. A benchmark of three tasks, in a directory named `parsing`:

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

`cladenc start` refuses a benchmark that holds no task, and one with a task that lacks its statement, its files or its check, and it says which task. It copies the tasks into `benchmark` in the run directory, so a search keeps the tasks it started with, and `cladenc resume` does not read the directory that `--benchmark` named.

A search goes on until its budget is spent, and it is stopped sooner in the same way as one that `cladenc resume` continues: by Ctrl-C, or by one of the signals `SIGINT`, `SIGTERM`, `SIGQUIT` and `SIGHUP`. `cladenc` then exits with status 1, and `cladenc resume` carries the search on from where it stopped.
