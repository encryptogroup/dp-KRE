import matplotlib.pyplot as plt
import numpy as np
import os
import pandas as pd
import sys

import plot_utils


def plot_devi_histogram(input_filename: str, reference: str, output_filename: str, title: str, x_label: str,
                        y_label: str) -> None:
    """
    Plots a histogram of percent deviation data.

    Args:
        input_filename (str): Name of the input CSV file.
        reference (float): Reference value for calculating percent deviation.
        output_filename (str): Name of the output plot image file.
        title (str): Title of the plot.
        x_label (str): Label of the X-axis.
        y_label (str): Label of the Y-axis.

    Returns:
        None
    """

    # Read the CSV file into a DataFrame
    df = pd.read_csv(input_filename, header=None)

    # Calculate percent deviation
    deviation = ((df[0] - reference) / reference) * 100

    # Compute the mean absolute percent deviation
    mean_dev = np.mean(np.abs(deviation))

    # Compute the number of bins using Freedman-Diaconis rule (or take min bin width)
    min_width = 0.05
    width = max(plot_utils.freedman_diaconis(deviation), min_width)

    # If there is no deviation, just plot a single bin & add extra bins for margin
    if mean_dev == 0:
        width = min_width
        # Calculate a bin that covers [-1/2*width ... 1/2*width]
        min_value = -0.5 * width
        max_value = 0.5 * width
        bins = [min_value, max_value]

        for i in range(1, 8):
            max_value += width
            bins.append(max_value)
            min_value -= width
            bins = [min_value] + bins

    else:
        # Calculate a bin that covers [-1/2*width ... 1/2*width]
        min_value = -0.5 * width
        max_value = 0.5 * width
        bins = [min_value, max_value]

        # Add bins of length width in positive direction until the biggest value is captured
        while True:
            max_value += width
            bins.append(max_value)
            if not max_value < deviation.max():
                break

        # Add bins in negative direction until the smallest value is captured
        while True:
            min_value -= width
            bins = [min_value] + bins
            if not min_value > deviation.min():
                break

    # Plot histogram
    plt.hist(deviation, bins=bins, edgecolor='black')
    plt.title(title)
    plt.xlabel(x_label)
    plt.ylabel(y_label)

    # Add mean deviation label in a rectangle
    stats_text = f'Mean Abs Deviation: {mean_dev:.2f}%\n Plot Resolution: {width:.2f}   '
    plt.gca().annotate(stats_text, xy=(0.95, 0.95), xycoords='axes fraction',
                       fontsize=10, ha='right', va='top',
                       bbox=dict(boxstyle='round', facecolor='white', edgecolor='black'))

    output_folder = os.path.dirname(output_filename)
    # Create the folder if it doesn't exist
    os.makedirs(output_folder, exist_ok=True)
    # Save figure
    plt.savefig(output_filename)


# These are always expected
file_path = sys.argv[1]
reference = float(sys.argv[2])
image_path = sys.argv[3]

# These are optional, defaults set here
title = sys.argv[4] if len(sys.argv) > 4 else "DP Result Deviations"
x_label = sys.argv[5] if len(sys.argv) > 5 else "Deviation (%)"
y_label = sys.argv[6] if len(sys.argv) > 6 else "Frequency"

plot_devi_histogram(file_path, reference, image_path, title, x_label, y_label)
