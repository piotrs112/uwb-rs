from collections import namedtuple
from math import sqrt
from scipy.optimize import minimize, least_squares

Point = namedtuple("Point", ["x", "y"])


class Anchor:
    name: str
    position: Point

    def __init__(self, name: str, position: Point) -> None:
        self.name = name
        self.position = position

    def __str__(self) -> str:
        return f"{self.name} @ x: {self.position.x}\ty: {self.position.y}"


def calculate_distance(a: Point, b: Point) -> float:
    return sqrt((b[0] - a[0]) ** 2 + (b[1] - a[1]) ** 2)


def mean_squared_error(point: Point, distances: list[float], anchors: list[Anchor]):
    mse = 0.0
    for d, a in zip(distances, anchors):
        dist = calculate_distance(point, a.position)
        mse += (dist - d) ** 2
    return mse / len(anchors)


# def trilateration(
#     distances: list[float], anchors: list[Anchor], initial_guess: Point = None
# ):
#     # Use closest anchor as initial guess by default
#     initial_guess = initial_guess or anchors[distances.index(min(distances))].position

#     return minimize(
#         mean_squared_error,
#         initial_guess,
#         args=(distances, anchors),
#         method="L-BFGS-B",
#         options={"ftol": 1e-5, "maxiter": 1e7},
#     )


def trilateration(
    distances: list[float], anchors: list[Anchor], initial_guess: Point = None
):
    initial_guess = initial_guess or anchors[distances.index(min(distances))].position
    return least_squares(mean_squared_error, initial_guess, args=(distances, anchors))
