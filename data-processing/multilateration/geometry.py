class Point2D:
    x: float
    y: float

    def __init__(self, x: float, y: float) -> None:
        self.x = x
        self.y = y

    def __getitem__(self, item) -> float:
        match item:
            case 0:
                return self.x
            case 1:
                return self.y
            case _:
                raise KeyError


class Point3D:
    x: float
    y: float
    z: float

    def __init__(self, x: float, y: float, z: float) -> None:
        self.x = x
        self.y = y
        self.z = z

    def __getitem__(self, item) -> float:
        match item:
            case 0:
                return self.x
            case 1:
                return self.y
            case 2:
                return self.z
            case _:
                raise KeyError


class Circle:
    center: Point2D
    radius: float

    def __init__(self, center: Point2D, radius: float) -> None:
        self.center = center
        self.radius = radius

    @classmethod
    def from_coords(cls, x, y, r) -> "Circle":
        return cls(Point2D(x, y), r)


class Sphere:
    center: Point3D
    radius: float

    def __init__(self, center: Point3D, radius: float) -> None:
        self.center = center
        self.radius = radius

    @classmethod
    def from_coords(cls, x, y, z, r) -> "Sphere":
        return cls(Point3D(x, y, z), r)
