import argparse

parser = argparse.ArgumentParser()
parser.add_argument("celsius", type=float)
args = parser.parse_args()
print(f"{args.celsius + 273.15:.1f}")
