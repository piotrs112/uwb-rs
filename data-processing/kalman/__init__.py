import numpy as np


def kalman_step(x, P, measurement, R, Q, F, H):
    # Predict
    x = F * x
    P = F * P * F.T + Q

    # Update
    y = np.matrix(measurement).T - H * x
    S = H * P * H.T + R
    K = P * H.T * S.I
    x = x + K * y
    I = np.matrix(np.eye(F.shape[0]))
    P = (I - K * H) * P

    return x, P
