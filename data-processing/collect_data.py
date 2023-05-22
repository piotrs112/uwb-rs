import pandas as pd
from datetime import datetime
import serial
import os

start_dt = datetime.now().isoformat(sep="-")
data = []

with serial.Serial(os.getenv("PROBE") or "/dev/ttyACM0", baudrate=115200) as s:
    try:
        while line := s.readline().decode():
            try:
                address, distance, instant, acc_x, acc_y, acc_z = [
                    v.strip() for v in line.split(" ")
                ]
                address = hex(int(address))
                print(f"{address} {distance} {instant}, {acc_x}, {acc_y}, {acc_z}")
                data.append(
                    (
                        address,
                        distance,
                        int(instant),
                        datetime.now().isoformat(),
                        float(acc_x),
                        float(acc_y),
                        float(acc_z),
                    )
                )
            except ValueError:
                continue
    except KeyboardInterrupt:
        pass
    df = pd.DataFrame(
        data,
        columns=(
            "anchor",
            "distance",
            "instant",
            "timestamp",
            "acc_x",
            "acc_y",
            "acc_z",
        ),
    )
    # df["instant"] -= df["instant"][0]

    df.to_csv(f"{start_dt}{os.getenv('LABEL') or ''}.csv")
