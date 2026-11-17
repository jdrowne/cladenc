# CladeNC

CladeNC (recursive acronym for Clade Next Clade) is a cleanroom implementation of the [Huxley-Gödel Machine](https://arxiv.org/abs/2510.21614) for agentic coding harnesses. It works by directed evolution. A directed evolution starts from one harness, the primogenitor. Harnesses procreate, each making a changed copy of its own source code, and are benchmarked: each attempts every task of the benchmarks a person brings. Artificial selection chooses which harness procreates, and at the end names the best harness.

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
| `cladenc report` | Prints the state of a directed evolution without changing it. |
| `cladenc resume` | Continues a directed evolution that stopped before its end. |
| `cladenc start` | Starts a new directed evolution and prints the directory of its best harness. |

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

`cladenc report` reads the journal in the outputs directory and prints the state of the directed evolution on standard output, without changing it: the model, how many procreations have been started, the best harness so far, and for each harness its progenitor, whether it was created, how many tasks it completed and did not complete, and the same counts for its clade, which is the harness and all its descendants together.

### `cladenc resume`

```text
cladenc resume \
  --outputs-dir <DIR>
```

| Flag | Required | Description |
| --- | --- | --- |
| `--outputs-dir <DIR>` | Required | The outputs directory of the directed evolution to continue. |

`cladenc resume` continues a directed evolution that stopped before its end, with the values that `cladenc start` was given, and prints the directory of the best harness on standard output once the directed evolution has ended. It says on standard error what it is doing, and it is stopped in the way `cladenc start` is. Of a directed evolution that had ended already, it prints the directory of the best harness again and does nothing else.

1. It removes the containers that the directed evolution left running.
2. It starts again each procreation and each attempt that was under way when the directed evolution stopped.
3. It carries the directed evolution on to its end, as `cladenc start` does.

### `cladenc start`

```text
cladenc start \
  --attempt-timeout <SECONDS> \
  --benchmark <DIR> \
  --docker-build-timeout <SECONDS> \
  --docker-image <IMAGE> \
  --dockerfile <FILE> \
  --genealogy-growth <NUMBER> \
  --harness-build-command <COMMAND> \
  --harness-run-command <COMMAND> \
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
| `--benchmark <DIR>` | Required | The directory of a benchmark, which holds a directory for each task. The flag is given once for each benchmark of the directed evolution. `cladenc` fetches no benchmark: the tasks are read from the directory and copied into the outputs directory. |
| `--docker-build-timeout <SECONDS>` | Not Required | The time limit of the build of the image. It is given with `--dockerfile`, and not without it. |
| `--docker-image <IMAGE>` | Not Required | The Docker image every container is started from. Either this flag or `--dockerfile` is given, and not both. |
| `--dockerfile <FILE>` | Not Required | The Dockerfile that `cladenc` builds the image of every container from, in place of `--docker-image`. |
| `--genealogy-growth <NUMBER>` | Required | How fast the genealogy may grow, from 0 to 1. |
| `--harness-build-command <COMMAND>` | Not Required | The shell command that builds a harness in a container, where one has to be built. |
| `--harness-run-command <COMMAND>` | Required | The shell command that runs a harness in a container. |
| `--max-concurrent-containers <COUNT>` | Required | The most containers running at once. |
| `--max-procreations <COUNT>` | Required | The most procreations to start, 1 or more. A procreation is one harness making another, and it counts once it is started, whether or not it creates its child. |
| `--model <NAME>` | Required | The name of the model the harnesses call, as its server knows it. `cladenc` calls no model itself: it tells each harness the name, the address and the key. |
| `--model-key-file <FILE>` | Not Required | The file that holds the key the model is called with, on one line, and nothing else. It is given where the server of the model asks for a key. `cladenc resume` reads the file again. |
| `--model-url <URL>` | Required | The address the model is served at, by a provider or by a server of your own. In a container, `host.docker.internal` names the machine `cladenc` runs on. |
| `--outputs-dir <DIR>` | Required | The outputs directory, which `start` creates. |
| `--primogenitor <DIR>` | Required | The directory of source code that is copied to make the primogenitor, the harness the directed evolution starts from. |
| `--procreation-timeout <SECONDS>` | Required | The time limit of one procreation, which is one harness making another. |

`cladenc start` starts a new directed evolution, carries it to its end, and prints the directory of the best harness on standard output. It says on standard error what it is doing. Ctrl-C, or one of the signals `SIGINT`, `SIGTERM`, `SIGQUIT` and `SIGHUP`, stops it sooner: it removes the containers it has running and exits with status 1, and `cladenc resume` continues the directed evolution from there.

1. It creates the outputs directory, and copies into it the source code of the primogenitor and the tasks of each benchmark.
2. Where `--dockerfile` is given, it builds the image that every container is started from.
3. Where `--harness-build-command` is given, it builds the primogenitor.
4. It has the primogenitor attempt every task of every benchmark. In one container the harness works on the task, and in a second one the grader of the task says whether the harness completed it.
5. It starts as many procreations as `--max-procreations` gives. In each, a harness makes a changed copy of its own source code, its child, and each child that is created attempts every task as the primogenitor did. `cladenc start` chooses which harness procreates among those that have attempted every task, partly by chance, and more often one that completed, together with its descendants, a larger share of the tasks they attempted.
6. It names the best harness: the created harness that completed the most tasks.
