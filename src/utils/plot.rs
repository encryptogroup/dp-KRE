use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::Command;

/// Generates a scatter plot by calling a Python script.
///
/// # Arguments
/// * `path` - A string slice that holds the path where the plot should be saved.
/// * `results_matrix` - A 2D vector of floating point numbers that represent the data to be plotted.
/// * `reference` - The reference value against which the data should be compared.
/// * `title` - Optional title for the plot.
/// * `x_label` - Optional label for the x-axis.
/// * `y_label` - Optional label for the y-axis.
///
/// # Returns
/// * A Result which is an Ok if the plot was successfully created, otherwise an Err.
pub fn plot_scatter(
    path: &str,
    results_matrix: &Vec<Vec<f64>>,
    reference: f64,
    title: Option<&str>,
    x_label: Option<&str>,
    y_label: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let results_filename = "results_scatter.csv";

    // Store points in csv
    let mut file = File::create(results_filename)?;
    for (_, results) in results_matrix.iter().enumerate() {
        let row: Vec<String> = results.iter().map(|&x| x.to_string()).collect();
        file.write_all(format!("{}\n", row.join(",")).as_bytes())?;
    }

    // Prepare the Python command
    let mut cmd = Command::new("python3");
    cmd.arg("src/utils/results_scatter.py"); // Python script name
    cmd.arg(results_filename);
    cmd.arg(path); // output image filename
    cmd.arg(reference.to_string()); // reference value

    // Append optional parameters if provided
    if let Some(t) = title { cmd.arg(t); }
    if let Some(x) = x_label { cmd.arg(x); }
    if let Some(y) = y_label { cmd.arg(y); }

    // Execute the Python command
    let output = cmd.output()?;

    // Check for errors in stdout/stderr
    if !output.stderr.is_empty() {
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
    }

    // Delete results.csv
    if Path::new(results_filename.clone()).exists() {
        std::fs::remove_file(results_filename)?;
    }

    Ok(())
}


/// Generates a bar plot by calling a Python script.
///
/// # Arguments
/// * `filename` - A string slice that holds the name of the file where the plot should be saved.
/// * `results` - A vector of floating point numbers that represent the data to be plotted.
/// * `reference` - The reference value against which the data should be compared.
/// * `title` - Optional title for the plot.
/// * `x_label` - Optional label for the x-axis.
/// * `y_label` - Optional label for the y-axis.
///
/// # Returns
/// * A Result which is an Ok if the plot was successfully created, otherwise an Err.
pub fn plot_bar(
    filename: &str,
    results: Vec<f64>,
    reference: f64,
    title: Option<&str>,
    x_label: Option<&str>,
    y_label: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let results_filename = "results.csv";

    // Store points in csv
    let mut file = File::create(results_filename)?;
    for point in results {
        file.write_all(format!("{},\n", point).as_bytes())?;
    }

    // Call python script
    let mut cmd = Command::new("python3");
    cmd.arg("src/utils/deviation_histogram.py") // Python script name
        .arg(results_filename)
        .arg(reference.to_string()) // reference value
        .arg(filename); // output image filename

    // Append optional parameters if provided
    if let Some(t) = title { cmd.arg(t); }
    if let Some(x) = x_label { cmd.arg(x); }
    if let Some(y) = y_label { cmd.arg(y); }

    // Execute the Python command
    let output = cmd.output()?;

    // Check for errors in stdout/stderr
    if !output.stderr.is_empty() {
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
    }

    // Delete points.csv
    if Path::new(results_filename).exists() {
        std::fs::remove_file(results_filename)?;
    }

    Ok(())
}


/// Generates a Laplace noise plot by calling a Python script.
///
/// # Arguments
/// * `filename` - A string slice that holds the name of the file where the plot should be saved.
/// * `points` - A vector of floating point numbers that represent the data to be plotted.
/// * `title` - Optional title for the plot.
/// * `x_label` - Optional label for the x-axis.
/// * `y_label` - Optional label for the y-axis.
///
/// # Returns
/// * A Result which is an Ok if the plot was successfully created, otherwise an Err.
pub fn draw_laplace(
    filename: &str,
    mut points: Vec<f64>,
    title: Option<&str>,
    x_label: Option<&str>,
    y_label: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let points_filename = "points.csv";

    // Sort points
    points.sort_by(|a, b| a.partial_cmp(b).unwrap());

    // Store points in csv for later use in python
    let mut file = File::create(points_filename)?;
    for point in &points {
        file.write_all(format!("{},\n", point).as_bytes())?;
    }

    // Call python script
    let mut cmd = Command::new("python3");
    cmd.arg("src/utils/noise_histogram.py") // Python script name
        .arg(points_filename)
        .arg(filename); // output image filename
    // Append optional parameters if provided
    if let Some(t) = title { cmd.arg(t); }
    if let Some(x) = x_label { cmd.arg(x); }
    if let Some(y) = y_label { cmd.arg(y); }

    // Execute the Python command
    let output = cmd.output()?;

    // Check for errors in stdout/stderr
    if !output.stderr.is_empty() {
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
    }

    // Delete points.csv
    if Path::new(points_filename).exists() {
        std::fs::remove_file(points_filename)?;
    }

    Ok(())
}



