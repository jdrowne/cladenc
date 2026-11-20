import json
import os
import sys

FILE = "todo.json"


def read():
    if not os.path.exists(FILE):
        return []
    with open(FILE) as file:
        return json.load(file)


def write(items):
    with open(FILE, "w") as file:
        json.dump(items, file)


def main(args):
    items = read()
    if args[0] == "add":
        items.append({"text": " ".join(args[1:]), "done": False})
        write(items)
    elif args[0] == "list":
        for number, item in enumerate(items, 1):
            mark = "x" if item["done"] else " "
            print(f"{number}. [{mark}] {item['text']}")


if __name__ == "__main__":
    main(sys.argv[1:])
