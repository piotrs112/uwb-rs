import pandas as pd
from datetime import datetime, timedelta
import serial
import os

start_dt = datetime.now()
start_dt_str = start_dt.isoformat(sep="-")
data = []
timeout = os.getenv("TIMEOUT")
with serial.Serial(os.getenv("PROBE") or "/dev/ttyACM0", baudrate=115200) as s:
    try:
        while line := s.readline().decode():
            if timeout is not None and datetime.now() - start_dt >= timedelta(
                seconds=int(timeout)
            ):
                break
            try:
                address, distance, instant, acc_x, acc_y, acc_z, los_confidence = [
                    v.strip() for v in line.split(" ")
                ]
                address = hex(int(address))
                print(
                    f"{address} {distance} {instant}, {acc_x}, {acc_y}, {acc_z}, {los_confidence}"
                )
                data.append(
                    (
                        address,
                        distance,
                        int(instant),
                        datetime.now().isoformat(),
                        float(acc_x),
                        float(acc_y),
                        float(acc_z),
                        float(los_confidence),
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
            "los_confidence",
        ),
    )
    # df["instant"] -= df["instant"][0]

    df.to_csv(f"{start_dt_str}{os.getenv('LABEL') or ''}.csv")
