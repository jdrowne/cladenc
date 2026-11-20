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
RETRIES = 3

SYSTEM = """You are an agentic coding harness working in the directory {work}.
You complete the instruction you are given by calling tools: list the files,
read them, change them, and run commands to check your work. Paths are taken
relative to {work}. You may read any file, but you write only under {work}.
Check your change by running it before you end. When the instruction is
complete, answer with a short summary and call no tool."""

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
        subdirectories[:] = sorted(d for d in subdirectories if d != ".git")
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


def size(messages):
    return sum(len(json.dumps(message)) for message in messages)


def shorten(messages):
    for message in messages[2:]:
        if size(messages) <= MAX_CONTEXT:
            return
        if message["role"] == "tool" and message["content"] != "[cut to save room]":
            message["content"] = "[cut to save room]"


def main():
    work = os.getcwd()
    messages = [
        {"role": "system", "content": SYSTEM.format(work=work)},
        {"role": "user", "content": os.environ["CLADENC_INSTRUCTION"]},
    ]
    for turn in range(1, MAX_TURNS + 1):
        shorten(messages)
        try:
            message = ask(messages)
        except Exception:
            return 1
        calls = message.get("tool_calls") or []
        messages.append(
            {
                "role": "assistant",
                "content": message.get("content") or "",
                **({"tool_calls": calls} if calls else {}),
            }
        )
        print(f"--- turn {turn}", flush=True)
        if message.get("content"):
            print(message["content"], flush=True)
        if not calls:
            return 0
        for call in calls:
            name = call["function"]["name"]
            arguments = call["function"].get("arguments") or "{}"
            print(f"> {name} {arguments}", flush=True)
            result = call_tool(work, name, arguments)
            print(result, flush=True)
            messages.append(
                {"role": "tool", "tool_call_id": call.get("id", name), "content": result}
            )
    print(f"--- stopped after {MAX_TURNS} turns", flush=True)
    return 0


if __name__ == "__main__":
    sys.exit(main())
