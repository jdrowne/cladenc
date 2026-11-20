import unittest

from calc import average, divide


class CalcTest(unittest.TestCase):
    def test_divide(self):
        self.assertEqual(divide(7, 2), 3.5)

    def test_average(self):
        self.assertEqual(average([1, 2]), 1.5)


if __name__ == "__main__":
    unittest.main()
