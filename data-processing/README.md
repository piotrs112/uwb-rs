# Data processing for uwb-rs
This is a collection of scripts and notebooks for processing ranging data from UWB tags.

Description:
- `accel.ipynb` - accelerometer data filtering \<WIP\>
- `movement.ipynb` - for csv files with movement \<WIP\>
    - UWB-only for now
- `tests.ipynb` - dane stacjonarne, own mean-squared-error-based trilateration algorithm \<WIP\>
    - not very accurate
- `preliminary_tests.ipynb` - stationary data trilateration using `easy_trilateration` library \<WIP\>
    - much better than tests.ipynb
- `kalman.ipynb` - Kalman filter for data processed through `movement.ipynb`

## Data collection
Use `collect_data.py` to collect data.
Parameters are set using environmental variables.
Plug in the tag's USB cable before starting the data collection.
Anchors and the tag do not have to be paired before starting data collection.

Example:

```bash
LABEL=<label> PROBE=</dev/XYZ> python3 collect_data.py
```

The default probe is `/dev/ttyACM0`. There is no default label (empty string).

## Tips
Start with `preliminary_tests.ipynb`, then test movement data with `movement.ipynb`. The notebbok outputs results into a CSV file into `data/triangulated/`. This data can then be processed through a Kalman filter with `kalman.ipynb`.