from easy_trilateration.model import Point
from math import sqrt


def distance(a: Point, b: Point):
    return sqrt((b.x - a.x) ** 2 + (b.y - a.y) ** 2)
