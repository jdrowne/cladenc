# CladeNC

CladeNC (recursive acronym for Clade Next Clade) is a cleanroom implementation of the [Huxley-Gödel Machine](https://arxiv.org/abs/2510.21614) for agentic coding harnesses.

## Sub-commands

| Sub-command | Description |
| --- | --- |
| `cladenc start` | Starts a new search and prints the directory of its best harness. |
| `cladenc resume` | Continues a search that stopped before its end. |
| `cladenc report` | Prints the state of a search without changing it. |

`cladenc --help` lists the sub-commands, and `cladenc --version` prints the version. `--help` after a sub-command prints its flags.

A search goes on until its budget is spent. To stop it sooner, press Ctrl-C, or send `cladenc` one of the signals `SIGINT`, `SIGTERM`, `SIGQUIT` and `SIGHUP`, as `kill` does. `cladenc` then ends the jobs it has running and exits with status 1, and `cladenc resume` carries the search on from where it stopped.

### `cladenc start`

```text
cladenc start \
  --budget <COUNT> \
  --confidence <PERCENT> \
  --docker-image <IMAGE> \
  --evaluation-timeout <SECONDS> \
  --evaluator <FILE> \
  --genealogy-growth <NUMBER> \
  --harness-build-command <COMMAND> \
  --harness-run-command <COMMAND> \
  --initial-procreations <COUNT> \
  --procreation-timeout <SECONDS> \
  --progenitor <DIR> \
  --quiet \
  --run-dir <DIR> \
  --seed <INTEGER> \
  --tasks <FILE> \
  --workers <COUNT>
```

| Flag | Required | Description |
| --- | --- | --- |
| `--budget <COUNT>` | Required | The number of evaluations to start. |
| `--confidence <PERCENT>` | Required | The probability behind the lower bound that picks the best harness. |
| `--docker-image <IMAGE>` | Required | The Docker image every container is started from. |
| `--evaluation-timeout <SECONDS>` | Required | The time limit of one evaluation. |
| `--evaluator <FILE>` | Required | The program that carries out an evaluation, which is one harness tried on one task. |
| `--genealogy-growth <NUMBER>` | Required | How fast the genealogy may grow, from 0 to 1. |
| `--harness-build-command <COMMAND>` | Not Required | The shell command that builds a harness in a container, where one has to be built. |
| `--harness-run-command <COMMAND>` | Required | The shell command that runs a harness in a container. |
| `--initial-procreations <COUNT>` | Required | The number of procreations of the progenitor made before the first evaluation. A procreation is one harness making another. |
| `--procreation-timeout <SECONDS>` | Required | The time limit of one procreation. |
| `--progenitor <DIR>` | Required | The directory of source code that is copied to make the progenitor, the harness the search starts from. |
| `--quiet` | Not Required | Print nothing on standard error. |
| `--run-dir <DIR>` | Required | The run directory, which `start` creates. |
| `--seed <INTEGER>` | Required | The seed of every random draw. |
| `--tasks <FILE>` | Required | A file of task ids, one on each line. |
| `--workers <COUNT>` | Required | The most jobs running at once. |

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
