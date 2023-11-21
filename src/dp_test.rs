#[cfg(test)]
mod dp_test {
    use crate::protocol::leaky_kth_ranked_element;
    use crate::party::dp_client::{get_scale_fixed, get_scale_sigmoid, GetScaleFn, NoiseLevel};
    use crate::utils::plot::{draw_laplace, plot_bar, plot_scatter};
    use crate::test::init_logging;
    use crate::utils::protocol::{create_server_dp_clients, get_kth_element, KValue, sample_databases};

    const DATABASE_SIZE: usize = 1000;
    const NUM_PARTIES: usize = 10;
    const PRECISION: i32 = 100;

    const MIN_DB_VALUE: i32 = PRECISION;
    // Always to be defined as PRECISION (= 1).
    const MAX_DB_VALUE: i32 = PRECISION * 1000; // Set the multiplier to the highest value in the database.

    // How many runs per noise level.
    const NUM_RUNS: usize = 100;

    // The scale function used for the Laplacian noise.
    const NOISE_SCALE_FN: GetScaleFn = get_scale_sigmoid;

    const PLOTS_FOLDERNAME: &str = "accuracy_results";

    #[test]
    fn test_dp() {
        init_logging();
        deviation_histogram();
        scatter_plot();
    }

    fn deviation_histogram() {
        let databases = sample_databases::<i32>(DATABASE_SIZE, NUM_PARTIES, MIN_DB_VALUE, MAX_DB_VALUE);

        for level in vec![NoiseLevel::LOW, NoiseLevel::MEDIUM, NoiseLevel::HIGH] {
            for k in vec![KValue::Min, KValue::Median, KValue::Max] {
                let expected = get_kth_element(&databases, k.to_k(DATABASE_SIZE));
                let expected = conv_floats(expected);

                let (results, noises) = run_multi_party(databases.clone(), k, level);
                let results: Vec<f64> = results.iter().map(|&i| conv_floats(i)).collect();

                let folder_name = format!("{}/{} noise", PLOTS_FOLDERNAME, level);
                plot_bar(
                    format!("{}/bar_plot_{}.png", folder_name, k).as_str(),
                    results,
                    expected,
                    Some(format!("DP Results (noise: {}, k: {})", level, k).as_str()), // title
                    Some("Deviation from true kth element (in %)"), // x_label
                    None,  // y_label
                ).expect("Failed to plot bar results");

                // Print the average noise for min, median, and max. The sum should use the absolute value of the noise.
                let avg_noise: f64 = noises.iter().map(|&n| n.abs()).sum::<f64>() / noises.len() as f64;
                tracing::trace!("noise: {}, k: {}, Average noise: {}", level, k, avg_noise);

                draw_laplace(
                    format!("{}/noise_plot_{}.png", folder_name, k).as_str(),
                    noises,
                    None, // title
                    None, // x_label
                    None,  // y_label
                ).unwrap();
            }
        }
    }

    fn scatter_plot() {
        let databases = sample_databases::<i32>(DATABASE_SIZE, NUM_PARTIES, MIN_DB_VALUE, MAX_DB_VALUE);

        for k in vec![KValue::Min, KValue::Median, KValue::Max] {
            // Vector of vectors to hold each set of results for all noise levels.
            let mut results_matrix: Vec<Vec<i32>> = vec![];
            let expected = get_kth_element(&databases, k.to_k(DATABASE_SIZE));
            let expected = conv_floats(expected);

            for level in vec![NoiseLevel::LOW, NoiseLevel::MEDIUM, NoiseLevel::HIGH] {
                let (results, _) = run_multi_party(databases.clone(), k, level);
                results_matrix.push(results);
            }
            // Convert the results to floats, therefore interpreting the integers as floats based on
            // the PRECISION.
            let results_matrix: Vec<Vec<f64>> = results_matrix
                .iter()
                .map(|inner| {
                    inner
                        .iter()
                        .map(|&i| conv_floats(i))
                        .collect()
                })
                .collect();

            plot_scatter(
                format!("{}/scatter_plot_{}.png", PLOTS_FOLDERNAME, k).as_str(),
                &results_matrix,
                expected,
                None, // title
                None, // x_label
                None,  // y_label
            ).expect("Failed to create scatter plot");
        }
    }

    /// Runs the DP-protocol with the given databases, k value, and noise level and returns the
    /// results and the noises.
    ///
    /// # Arguments
    ///
    /// * `dbs` - A vector of vectors representing the databases.
    /// * `k` - The k value used for the DP-protocol.
    /// * `noise_level` - The level of noise to be used.
    ///
    /// # Returns
    ///
    /// Returns a tuple containing two vectors:
    /// * The first vector contains the results obtained from running the DP-protocol.
    /// * The second vector contains the noise values used by all parties.
    ///
    fn run_multi_party(dbs: Vec<Vec<i32>>, k: KValue, noise_level: NoiseLevel) -> (Vec<i32>, Vec<f64>) {
        let k = k.to_k(DATABASE_SIZE);
        let mut results = vec![];
        let mut noises: Vec<f64> = vec![];

        // Create the server and DP parties using the sigmoid scale function with the given level of DP.
        let (mut server, mut parties) = create_server_dp_clients(k, dbs.clone(), NOISE_SCALE_FN, noise_level);
        for _ in 0..NUM_RUNS {
            let result = leaky_kth_ranked_element(&mut server, &mut parties).unwrap_or_default();
            results.push(result);
        }

        // Append the used noise to the noise_array.
        parties.iter().for_each(|p| noises.append(&mut p.noise_array.clone()));

        (results, noises)
    }

    fn conv_floats(i: i32) -> f64 {
        i as f64 / PRECISION as f64
    }
}
