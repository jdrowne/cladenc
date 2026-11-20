import sys

total = 0
with open(sys.argv[1]) as file:
    for line in file:
        total += float(line.split(",")[2])
print(total)
