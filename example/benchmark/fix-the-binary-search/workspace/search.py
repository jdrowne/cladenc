def find(items, target):
    low, high = 0, len(items)
    while low < high:
        middle = (low + high) // 2
        if items[middle] == target:
            return middle
        if items[middle] < target:
            low = middle
        else:
            high = middle - 1
    return -1
