import matplotlib.pyplot as plt
import os
import pandas as pd
import sys
from collections import Counter
from matplotlib.colors import LinearSegmentedColormap
from matplotlib.ticker import MaxNLocator


def plot_scatter(input_filename: str, output_filename: str, ref_value: float, title: str,
                 x_label: str, y_label: str) -> None:
    """
    Plots a scatter plot of the input data.

    Args:
        input_filename (str): Name of the input CSV file.
        output_filename (str): Name of the output plot image file.
        ref_value (float): Reference value for the vertical line.
        title (str): Title of the plot.
        x_label (str): Label of the X-axis.
        y_label (str): Label of the Y-axis.

    Returns:
        None
    """

    # Read the CSV file into a DataFrame
    df = pd.read_csv(input_filename, header=None)

    # Initialize variables
    x_values = []
    y_values = []
    sizes = []

    # Iterate over each row in the DataFrame
    for index, row in df.iterrows():
        # Skip empty lines
        if row.isnull().all():
            continue

        # Get the counts of each x-value in the row
        row_counts = Counter(row)

        # Loop through each unique value and its count in the row
        for value, count in row_counts.items():
            x_values.append(value)
            y_values.append(index)
            sizes.append(count)  # Adjusted size

    # Create a color map from light blue to red
    colors = ["lightblue", "red"]
    cmap = LinearSegmentedColormap.from_list("", colors)

    # Adapt size of scatter points
    size = 200
    if df.shape[0] > 10:
        size = (10 / df.shape[0]) * 200

    # Create the scatter plot
    scatter = plt.scatter(x=x_values, y=y_values, c=sizes, cmap=cmap, s=size, alpha=0.6)

    # Add a vertical line for the reference value
    ref_value_float = float(ref_value)
    plt.axvline(x=ref_value_float, color='green', linestyle='--')

    # Set plot title and labels
    plt.title(title)
    plt.xlabel(x_label)
    plt.ylabel(y_label)

    # Fix the y-axis ticks
    if df.shape[0] > 20:
        step = 5
    else:
        step = 1
    plt.yticks(range(0, df.shape[0], step))

    # Create colorbar
    cbar = plt.colorbar(scatter)
    cbar.locator = MaxNLocator(integer=True)
    cbar.update_ticks()

    # Add label for the reference value
    plt.text(0.95, 0.95, f'Ref: {ref_value_float}',
             transform=plt.gca().transAxes, color='green',
             ha='right', va='top',
             bbox=dict(facecolor='white', alpha=0.6, edgecolor='green'))

    output_folder = os.path.dirname(output_filename)
    # Create the folder if it doesn't exist
    os.makedirs(output_folder, exist_ok=True)
    # Save the plot to the output filename
    plt.savefig(output_filename)


# These are always expected
file_path = sys.argv[1]
image_path = sys.argv[2]
reference_value = float(sys.argv[3])

# These are optional, defaults set here
title = sys.argv[4] if len(sys.argv) > 4 else "Scatter Plot"
x_label = sys.argv[5] if len(sys.argv) > 5 else "Deviation"
y_label = sys.argv[6] if len(sys.argv) > 6 else "(0=Low, 1=Medium, 2=High) Noise Level"

plot_scatter(file_path, image_path, reference_value, title, x_label, y_label)
