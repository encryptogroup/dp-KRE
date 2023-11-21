import matplotlib.pyplot as plt
import os
import pandas as pd
import sys

import plot_utils


def plot_noise_histogram(input_filename: str, output_filename: str, title: str, x_label: str, y_label: str) -> None:
    """
    Plots a histogram of noise data.

    Args:
        input_filename (str): Name of the input CSV file.
        output_filename (str): Name of the output plot image file.
        title (str): Title of the plot.
        x_label (str): Label of the X-axis.
        y_label (str): Label of the Y-axis.

    Returns:
        None
    """

    # Read the CSV file into a DataFrame
    df = pd.read_csv(input_filename, header=None)

    # Remove outliers
    clean_data = plot_utils.remove_outliers(df[0])

    # Compute the number of bins using Freedman-Diaconis rule
    width = plot_utils.freedman_diaconis(clean_data)
    bins = int((clean_data.max() - clean_data.min()) / width)

    # Compute relevant statistics
    mean = clean_data.mean()
    std_dev = clean_data.std()
    skewness = clean_data.skew()
    kurtosis = clean_data.kurtosis()
    stats_text = f'Std Dev: {std_dev:.2f}\nSkewness: {skewness:.2f}\nKurtosis: {kurtosis:.2f}'

    # Plot histogram
    plt.hist(clean_data, bins=bins, edgecolor='black')
    plt.title(title)
    plt.xlabel(x_label)
    plt.ylabel(y_label)

    # Add text box with statistics
    plt.text(0.95, 0.95, stats_text, transform=plt.gca().transAxes, ha='right', va='top',
             bbox=dict(boxstyle='round', facecolor='white', alpha=0.5))

    output_folder = os.path.dirname(output_filename)
    # Create the folder if it doesn't exist
    os.makedirs(output_folder, exist_ok=True)
    # Save figure
    plt.savefig(output_filename)


# These are always expected
file_path = sys.argv[1]
image_path = sys.argv[2]

# These are optional
title = sys.argv[3] if len(sys.argv) > 3 else "Noise"
x_label = sys.argv[4] if len(sys.argv) > 4 else "Value"
y_label = sys.argv[5] if len(sys.argv) > 5 else "Frequency"

plot_noise_histogram(file_path, image_path, title, x_label, y_label)
