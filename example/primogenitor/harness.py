import ast
import builtins
import json
import os
import subprocess
import sys
import time
import urllib.error
import urllib.request

MAX_TURNS = 40
MAX_OUTPUT = 8000
MAX_CONTEXT = 60000
COMMAND_TIMEOUT = 120
REQUEST_TIMEOUT = 600
START_TIMEOUT = 30
RETRIES = 3
SKIPPED = {".git", "__pycache__"}

SYSTEM = """You are an agentic coding harness working in the directory {work}.
You complete the instruction you are given by calling tools: list the files,
read them, change them, and run commands to check your work. Paths are taken
relative to {work}. You may read any file, but you write only under {work}.
A file changes only when you call write_file or replace_in_file: code that
you only print in an answer changes nothing. Check your change by running it
before you end. When the instruction is complete, answer with a short summary
and call no tool."""

NOTHING_WRITTEN = """Nothing was written: no file under {work} has changed
since you started. A file changes only when you call write_file or
replace_in_file, and code or a plan that you only print changes nothing. Where
the instruction asks for a change, read the files it is about, make the change
with a tool call, and then answer with a short summary and call no tool."""

NOT_FINISHED = """The files you changed were checked, and the work is not
finished:

{faults}

Mend this with the tools, then answer with a short summary and call no tool."""

TOOLS = [
    {
        "type": "function",
        "function": {
            "name": "list_files",
            "description": "List the files under a directory, recursively.",
            "parameters": {
                "type": "object",
                "properties": {"path": {"type": "string"}},
                "required": ["path"],
            },
        },
    },
    {
        "type": "function",
        "function": {
            "name": "read_file",
            "description": "Read a file of text, with its lines numbered.",
            "parameters": {
                "type": "object",
                "properties": {"path": {"type": "string"}},
                "required": ["path"],
            },
        },
    },
    {
        "type": "function",
        "function": {
            "name": "write_file",
            "description": "Write a whole file, replacing what it held.",
            "parameters": {
                "type": "object",
                "properties": {
                    "path": {"type": "string"},
                    "content": {"type": "string"},
                },
                "required": ["path", "content"],
            },
        },
    },
    {
        "type": "function",
        "function": {
            "name": "replace_in_file",
            "description": "Replace text that occurs exactly once in a file.",
            "parameters": {
                "type": "object",
                "properties": {
                    "path": {"type": "string"},
                    "old": {"type": "string"},
                    "new": {"type": "string"},
                },
                "required": ["path", "old", "new"],
            },
        },
    },
    {
        "type": "function",
        "function": {
            "name": "run",
            "description": "Run a shell command in the working directory "
            "and return what it printed and its exit status.",
            "parameters": {
                "type": "object",
                "properties": {"command": {"type": "string"}},
                "required": ["command"],
            },
        },
    },
]


def cut(text):
    if len(text) <= MAX_OUTPUT:
        return text
    half = MAX_OUTPUT // 2
    return text[:half] + "\n[... cut ...]\n" + text[-half:]


def resolve(work, path):
    return os.path.normpath(os.path.join(work, path))


def writable(work, path):
    full = resolve(work, path)
    if full != work and not full.startswith(work + os.sep):
        raise ValueError(f"{path} is outside {work}, where nothing is written")
    return full


def list_files(work, path):
    root = resolve(work, path)
    found = []
    for directory, subdirectories, files in os.walk(root):
        subdirectories[:] = sorted(d for d in subdirectories if d not in SKIPPED)
        for name in sorted(files):
            found.append(os.path.relpath(os.path.join(directory, name), root))
    return "\n".join(found) or "(no files)"


def read_file(work, path):
    with open(resolve(work, path), encoding="utf-8", errors="replace") as file:
        lines = file.read().splitlines()
    return "\n".join(f"{number:4} {line}" for number, line in enumerate(lines, 1))


def write_file(work, path, content):
    full = writable(work, path)
    os.makedirs(os.path.dirname(full), exist_ok=True)
    with open(full, "w", encoding="utf-8") as file:
        file.write(content)
    return f"wrote {len(content)} characters to {path}"


def replace_in_file(work, path, old, new):
    full = writable(work, path)
    with open(full, encoding="utf-8") as file:
        text = file.read()
    count = text.count(old)
    if count != 1:
        return f"the text occurs {count} times in {path}, not once; nothing changed"
    with open(full, "w", encoding="utf-8") as file:
        file.write(text.replace(old, new, 1))
    return f"replaced the text in {path}"


def run(work, command):
    try:
        done = subprocess.run(
            command,
            shell=True,
            cwd=work,
            capture_output=True,
            text=True,
            errors="replace",
            timeout=COMMAND_TIMEOUT,
        )
    except subprocess.TimeoutExpired:
        return f"the command ran past {COMMAND_TIMEOUT} seconds and was stopped"
    return f"exit status {done.returncode}\n{done.stdout}{done.stderr}"


ACTIONS = {
    "list_files": list_files,
    "read_file": read_file,
    "write_file": write_file,
    "replace_in_file": replace_in_file,
    "run": run,
}


def call_tool(work, name, arguments):
    action = ACTIONS.get(name)
    if action is None:
        return f"there is no tool named {name}"
    try:
        given = json.loads(arguments) if isinstance(arguments, str) else arguments
        return cut(action(work, **given))
    except Exception as error:
        return f"{type(error).__name__}: {error}"


def ask(messages):
    url = os.environ["CLADENC_MODEL_URL"].rstrip("/") + "/chat/completions"
    body = json.dumps(
        {"model": os.environ["CLADENC_MODEL"], "messages": messages, "tools": TOOLS}
    ).encode()
    headers = {"Content-Type": "application/json"}
    key = os.environ.get("CLADENC_MODEL_KEY")
    if key:
        headers["Authorization"] = f"Bearer {key}"
    for attempt in range(1, RETRIES + 1):
        request = urllib.request.Request(url, data=body, headers=headers)
        try:
            with urllib.request.urlopen(request, timeout=REQUEST_TIMEOUT) as response:
                return json.load(response)["choices"][0]["message"]
        except (urllib.error.URLError, TimeoutError, KeyError, ValueError) as error:
            print(f"the model could not be asked ({error}), try {attempt}", flush=True)
            if attempt == RETRIES:
                raise
            time.sleep(5 * attempt)


def snapshot(work):
    seen = {}
    for directory, subdirectories, files in os.walk(work):
        subdirectories[:] = [d for d in subdirectories if d not in SKIPPED]
        for name in files:
            path = os.path.join(directory, name)
            try:
                status = os.lstat(path)
            except OSError:
                continue
            seen[path] = (status.st_mtime_ns, status.st_size)
    return seen


def changed_since(before, after):
    return sorted(path for path in after if before.get(path) != after[path])


def unknown_names(tree):
    defined = set(dir(builtins)) | {"__file__", "__builtins__", "__class__"}
    used = set()
    for node in ast.walk(tree):
        if isinstance(node, ast.Name) and isinstance(node.ctx, ast.Load):
            used.add((node.lineno, node.id))
        elif isinstance(node, ast.Name):
            defined.add(node.id)
        elif isinstance(node, ast.alias):
            defined.add((node.asname or node.name).split(".")[0])
        elif isinstance(node, (ast.Global, ast.Nonlocal)):
            defined.update(node.names)
        else:
            for field in ("name", "arg", "rest"):
                if isinstance(getattr(node, field, None), str):
                    defined.add(getattr(node, field))
    if "*" in defined:
        return []
    return sorted(use for use in used if use[1] not in defined)


def faults_of(work, path):
    name = os.path.relpath(path, work)
    try:
        with open(path, encoding="utf-8") as file:
            tree = ast.parse(file.read(), name)
        compile(tree, name, "exec")
    except SyntaxError as error:
        return [f"{name} does not compile: line {error.lineno}: {error.msg}"]
    except (OSError, ValueError) as error:
        return [f"{name} cannot be read as Python: {error}"]
    return [
        f"{name} line {line} uses the name {unknown}, which nothing in the file defines"
        for line, unknown in unknown_names(tree)
    ]


def started_wrongly(work, script):
    name = os.path.basename(script)
    asked = f"{name}, started with CLADENC_INSTRUCTION not set,"
    expected = "is expected to print a message and exit with status 2 at once"
    environment = {
        key: value for key, value in os.environ.items() if key != "CLADENC_INSTRUCTION"
    }
    try:
        done = subprocess.run(
            [sys.executable, "-B", script],
            cwd=work,
            env=environment,
            stdin=subprocess.DEVNULL,
            capture_output=True,
            text=True,
            errors="replace",
            timeout=START_TIMEOUT,
        )
    except subprocess.TimeoutExpired:
        return [f"{asked} {expected}; it was still running after {START_TIMEOUT} seconds"]
    if done.returncode == 2 and "Traceback" not in done.stderr:
        return []
    printed = cut(done.stdout + done.stderr)
    return [
        f"{asked} {expected}, with no traceback; it exited with status "
        f"{done.returncode} and printed:\n{printed}"
    ]


def faults(work, changed):
    found = []
    for path in changed:
        if path.endswith(".py"):
            found.extend(faults_of(work, path))
    script = os.path.join(work, os.path.basename(__file__))
    if changed and not found and os.path.isfile(script):
        found.extend(started_wrongly(work, script))
    return found


def size(messages):
    return sum(len(json.dumps(message)) for message in messages)


def shorten(messages):
    for message in messages[2:]:
        if size(messages) <= MAX_CONTEXT:
            return
        if message["role"] == "tool" and message["content"] != "[cut to save room]":
            message["content"] = "[cut to save room]"


def main():
    instruction = os.environ.get("CLADENC_INSTRUCTION")
    if not instruction:
        print("CLADENC_INSTRUCTION is not set, so there is nothing to do", file=sys.stderr)
        return 2
    work = os.getcwd()
    before = snapshot(work)
    told = False
    messages = [
        {"role": "system", "content": SYSTEM.format(work=work)},
        {"role": "user", "content": instruction},
    ]
    for turn in range(1, MAX_TURNS + 1):
        shorten(messages)
        try:
            message = ask(messages)
        except Exception:
            return 1
        calls = message.get("tool_calls") or []
        content = message.get("content") or ""
        messages.append(
            {
                "role": "assistant",
                "content": content,
                **({"tool_calls": calls} if calls else {}),
            }
        )
        print(f"--- turn {turn}", flush=True)
        if content:
            print(content, flush=True)
        for call in calls:
            name = call["function"]["name"]
            arguments = call["function"].get("arguments") or "{}"
            print(f"> {name} {arguments}", flush=True)
            result = call_tool(work, name, arguments)
            print(result, flush=True)
            messages.append(
                {"role": "tool", "tool_call_id": call.get("id", name), "content": result}
            )
        if calls:
            continue
        after = snapshot(work)
        if after == before and not told:
            told = True
            said = NOTHING_WRITTEN.format(work=work)
        else:
            found = faults(work, changed_since(before, after))
            if not found:
                return 0
            said = NOT_FINISHED.format(faults="\n".join(found))
        print(f"--- check\n{said}", flush=True)
        messages.append({"role": "user", "content": said})
    print(f"--- stopped after {MAX_TURNS} turns", flush=True)
    return 1 if faults(work, changed_since(before, snapshot(work))) else 0


if __name__ == "__main__":
    sys.exit(main())
