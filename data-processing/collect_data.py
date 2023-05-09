import pandas as pd
from datetime import datetime
import serial

start_dt = datetime.now().isoformat(sep="-")
data = []

with serial.Serial("/dev/ttyACM0", baudrate=115200) as s:
    try:
        while line := s.readline().decode():
            try:
                address, distance, instant = [v.strip() for v in line.split(" ")]
                address = hex(int(address))
                print(f"{address} {distance} {instant}")
                data.append(
                    (address, distance, int(instant), datetime.now().isoformat())
                )
            except ValueError:
                continue
    except KeyboardInterrupt:
        pass
    df = pd.DataFrame(data, columns=("anchor", "distance", "instant", "timestamp"))
    df["instant"] -= df["instant"][0]

    df.to_csv(f"{start_dt}.csv")
