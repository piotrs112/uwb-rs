from abc import ABC, abstractmethod
from typing import Literal

import pandas as pd

from .solver_2d import Multilateration2D
from .solver_3d import Multilateration3D
from .geometry import Point2D, Point3D, Sphere, Circle


def get_point_class(method):
    match method:
        case "2d":
            return Point2D
        case "3d":
            return Point3D


def get_shape_class(method):
    match method:
        case "2d":
            return Circle
        case "3d":
            return Sphere


def get_solver_class(method):
    match method:
        case "2d":
            return Multilateration2D
        case "3d":
            return Multilateration3D


def solve(df, anchors, method):
    used_anchors = df["anchor"].unique()
    anchors = {k: v for k, v in anchors.items() if k in used_anchors}
    df = _prepare_data(df)
    Multilateration = get_solver_class(method)
    history = []

    for timestamp, row in df.iterrows():
        match method:
            case "2d":
                s = [
                    Circle.from_coords(*anchors[anchor], row[anchor])
                    for anchor in anchors
                ]
                history.append(Multilateration2D(s))
            case "3d":
                s = [
                    Sphere.from_coords(*anchors[anchor], 0, row[anchor])
                    for anchor in anchors
                ]
                history.append(Multilateration3D(s))

    results = Multilateration.solve_history(history)

    data = {"x": [], "y": [], "z": [], "radius": [], "timestamp": df.index.to_list()}
    for t in results:
        t: Sphere | Circle
        data["x"].append(t.center.x)
        data["y"].append(t.center.y)
        if method == "3d":
            data["z"].append(t.center.z)
        else:
            data["z"].append(0)
        data["radius"].append(t.radius)

    return pd.DataFrame(data)


def _prepare_data(df):
    n_anchors = df["anchor"].nunique()

    # Create column for every anchor
    df = (
        df.pivot(columns="anchor", values="distance")
        .fillna(0)
        .rolling(window=n_anchors, step=n_anchors)
        .sum()
        .dropna()
    )
    return df
