import json

DEFAULTS = {"host": "localhost", "port": 8080, "debug": False}


def load(path):
    with open(path) as file:
        data = json.load(file)
    return {"host": data["host"], "port": data["port"], "debug": data["debug"]}
