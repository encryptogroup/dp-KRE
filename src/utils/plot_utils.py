import numpy as np


def freedman_diaconis(data):
    """ Freedman-Diaconis rule to compute optimal histogram bin width. """
    data = np.asarray(data, dtype=np.float_)
    IQR = np.percentile(data, 75) - np.percentile(data, 25)
    n = len(data)
    return 2.0 * IQR / np.power(n, 1.0 / 3.0)


def remove_outliers(data, multiplier=1.5):
    """ Remove outliers using IQR method. """
    Q1 = data.quantile(0.25)
    Q3 = data.quantile(0.75)
    IQR = Q3 - Q1
    data_clean = data[~((data < (Q1 - multiplier * IQR)) | (data > (Q3 + multiplier * IQR)))]
    return data_clean
