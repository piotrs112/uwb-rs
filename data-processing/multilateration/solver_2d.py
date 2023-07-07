from typing import Any
from scipy.optimize import least_squares, minimize

from .geometry import Point2D, Circle


class Multilateration2D:
    anchors: list[Circle]

    def __init__(self, anchors: list[Circle]):
        self.anchors = anchors

    @classmethod
    def solve_history(
        cls,
        history: list["Multilateration2D"],
        initial_guess: Circle = Circle.from_coords(0, 0, 0),
    ) -> list[Circle]:
        guess: Circle = initial_guess
        results: list[Circle] = []
        for item in history:
            guess = cls.solve(item, guess)
            results.append(guess)
        return results

    @classmethod
    def solve(
        cls,
        multilateration: "Multilateration2D",
        guess: Circle = Circle.from_coords(0, 0, 0),
    ) -> Circle:
        result, _ = cls._solve_least_squares(multilateration.anchors, guess)
        return result

    @classmethod
    def _solve_least_squares(
        cls, circles: list[Circle], guess: Circle = Circle.from_coords(0, 0, 0)
    ) -> tuple[Circle, Any]:
        g = (guess.center.x, guess.center.y, guess.radius)
        result = least_squares(cls.equations, g, args=[circles])
        xf, yf, rf = result.x

        return Circle.from_coords(xf, yf, rf), result

    @staticmethod
    def equations(guess, circles: list[Circle]):
        eqs: list[float] = []
        x, y, r = guess
        for sphere in circles:
            eqs.append(
                (
                    (x - sphere.center.x) ** 2
                    + (y - sphere.center.y) ** 2
                    - (sphere.radius - r) ** 2
                )
            )
        return eqs
