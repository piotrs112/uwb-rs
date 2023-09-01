# Data processing for uwb-rs
This is a collection of scripts and notebooks for processing ranging data from UWB tags.

## Main notebooks
- `mlat_many_samples.ipynb` - full multilateration and Kalman filtering script. Probably the one you want to use
- `uwb_simulation` - generates simulated UWB distance data

## Test notebooks

These notebooks are used for testing different features. They may or may not work.
- `accel.ipynb` - accelerometer data filtering \<WIP\>
- `movement.ipynb` - for csv files with movement \<WIP\>
    - UWB-only for now
- `tests.ipynb` - stationary data, own mean-squared-error-based trilateration algorithm \<WIP\>
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


## Datasets
The directory `data/` should contain datasets. Each dataset should contain one or multiple CSV files with distance measurements and an `environment.json` file describing the test environment, for example:

```json
{
    "tag": {"P1": [1560, 1700], "P2": [2185, 2550], "P3": [1560, 2990], "P4": [775, 2130], "P5": [1560, 0], "P6": [1560, 840]},
    "anchors": {"0x6c0d": [0, 0], "0x5601": [3400, 0], "0x26bd": [3400, 4350], "0x5836": [-200, 4350]},
    "z": "",
    "acc_orientation": "+y",
    "REF_POINT": "P2"
}

```
where:

- `tag` contains a map of predefined reference point and their [`x`, `y`] coordinates,
- `anchors` contains a map of anchors and their positions. The key should match the anchors' values used in the `anchor` CSV column,
- `z` is the Z-axis height of the anchors. This is not used at the moment,
- `acc_orientation` is the direction of the accelerometer's +Z axis in the room's coordinate system. Valid values are: `+y`, `-y`, `+z`, `-z`, `+x` and `-x`.
- `REF_POINT`: the reference point name. Should match a key from the `tag` map.

These parameters can also be set in the `mlat_many_samples.ipynb` notebook. The presence of the `environment.json` file **will** override those manually set parameters.

## Tips
If you don't have the hardware modules, you can generate simulated data. Warning: the simualtion does not generate accelerometer data.

The data should be placed inside the `data/` directory in the project root. Using datasets is highly recommended.