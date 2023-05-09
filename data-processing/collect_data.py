import pandas as pd
from datetime import datetime
import serial

start_dt = datetime.now().isoformat(sep="-")
data = []

with serial.Serial("/dev/ttyACM0", baudrate=115200) as s:
    try:
        while line := s.readline().decode():
            try:
                address, distance = [v.strip() for v in line.split(" - ")]
                address = hex(int(address))
                print(f"{address} - {distance}")
                data.append((address, distance, datetime.now().isoformat()))
            except ValueError:
                continue
    except KeyboardInterrupt:
        pass
    pd.DataFrame(data, columns=("anchor", "distance", "timestamp")).to_csv(
        f"{start_dt}.csv"
    )
