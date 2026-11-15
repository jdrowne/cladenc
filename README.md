# CladeNC

CladeNC (recursive acronym for Clade Next Clade) is a cleanroom implementation of the [Huxley-Gödel Machine](https://arxiv.org/abs/2510.21614) for agentic coding harnesses.

## Sub-commands

| Sub-command | What it does |
| --- | --- |
| `cladenc run` | Starts a new search and prints the directory of its best harness. |
| `cladenc resume` | Continues a search that stopped before its end. |
| `cladenc report` | Prints the state of a search without changing it. |

`cladenc --help` lists the sub-commands, and `cladenc --version` prints the version. `--help` after a sub-command prints its options.

### `cladenc run`

| Option | Required | What it is |
| --- | --- | --- |
| `--progenitor <DIR>` | yes | The directory of source code that is copied to make the progenitor, the harness the search starts from. |
| `--docker-image <IMAGE>` | yes | The Docker image every container is started from. |
| `--harness-build-command <COMMAND>` | no | The shell command that builds a harness in a container, where one has to be built. |
| `--harness-run-command <COMMAND>` | yes | The shell command that runs a harness in a container. |
| `--tasks <FILE>` | yes | A file of task ids, one on each line. |
| `--evaluator <FILE>` | yes | The program that carries out an evaluation, which is one harness tried on one task. |
| `--run-dir <DIR>` | yes | The run directory, which `run` creates. |
| `--budget <COUNT>` | yes | The number of evaluations to start. |
| `--genealogy-growth <NUMBER>` | yes | How fast the genealogy may grow, from 0 to 1. |
| `--confidence <PERCENT>` | yes | The probability behind the lower bound that picks the best harness. |
| `--initial-procreations <COUNT>` | yes | The number of procreations of the progenitor made before the first evaluation. A procreation is one harness making another. |
| `--workers <COUNT>` | yes | The most jobs running at once. |
| `--procreation-timeout <SECONDS>` | yes | The time limit of one procreation. |
| `--evaluation-timeout <SECONDS>` | yes | The time limit of one evaluation. |
| `--seed <INTEGER>` | yes | The seed of every random draw. |
| `--quiet` | no | Print nothing on standard error. |

### `cladenc resume`

| Option | Required | What it is |
| --- | --- | --- |
| `--run-dir <DIR>` | yes | The run directory of the search to continue. |
| `--quiet` | no | Print nothing on standard error. |

### `cladenc report`

| Option | Required | What it is |
| --- | --- | --- |
| `--run-dir <DIR>` | yes | The run directory of the search to report on. |
| `--json` | no | Print one JSON object in place of the table. |
