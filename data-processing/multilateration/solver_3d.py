from typing import Any
from scipy.optimize import least_squares

from .geometry import Point3D, Sphere


class Multilateration3D:
    anchors: list[Sphere]

    def __init__(self, anchors: list[Sphere]):
        self.anchors = anchors

    @classmethod
    def solve_history(
        cls,
        history: list["Multilateration3D"],
        initial_guess: Sphere = Sphere.from_coords(0, 0, 0, 0),
    ) -> list[Sphere]:
        guess: Sphere = initial_guess
        results: list[Sphere] = []
        for item in history:
            guess = cls.solve(item, guess)
            results.append(guess)
        return results

    @classmethod
    def solve(
        cls,
        multilateration: "Multilateration3D",
        guess: Sphere = Sphere.from_coords(0, 0, 0, 0),
    ) -> Sphere:
        result, _ = cls._solve_least_squares(multilateration.anchors, guess)
        return result

    @classmethod
    def _solve_least_squares(
        cls, spheres: list[Sphere], guess: Sphere = Sphere.from_coords(0, 0, 0, 0)
    ) -> tuple[Sphere, Any]:
        g = (guess.center.x, guess.center.y, guess.center.z, guess.radius)
        result = least_squares(cls.equations, g, args=[spheres])
        xf, yf, zf, rf = result.x

        return Sphere.from_coords(xf, yf, zf, rf), result

    @staticmethod
    def equations(guess, spheres: list[Sphere]):
        eqs = []
        x, y, z, r = guess
        for sphere in spheres:
            eqs.append(
                (
                    (x - sphere.center.x) ** 2
                    + (y - sphere.center.y) ** 2
                    + (z - sphere.center.z) ** 2
                    - (sphere.radius - r) ** 2
                )
            )
        return eqs
