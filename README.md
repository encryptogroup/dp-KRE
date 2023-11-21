# Differentially-Private Kth Ranked Element Protocol

This is an implementation of the Leaky Kth Ranked Element Protocol (KRE) as described in [1] with
differential privacy.

## Getting Started
You first need to install the dependencies. This can be done by running the following command:
```shell
cargo build
```
Additionally, you need some Python packages to create the plots used for the accuracy tests.
These can be installed by running:
```shell
pip install -r requirements.txt
```
---
### Running the Protocol
Now, you can run the tests which execute the original kth-ranked element protocol.
The test file is located in `src/test.rs`, there you can adapt parameters like the database size or the number of parties.
To run the tests, execute the following command:
```shell   
cargo test --lib test::tests
```
Or, if you only want to run the multi-party test, use `--lib test::tests::test_multi_party` instead.


## Accuracy Tests
The accuracy tests for the original and the DP protocol can be run by executing the following command:

*Hint:* You can adapt all parameters such as the database size, the number of parties, number of runs, etc. in the 
`src/dp_test.rs` file.

```shell
cargo test --lib dp_test::dp_test::test_dp
```
The results for the different noise levels are saved as different plots in the `accuracy_results` directory.
```
accuracy_results/
├── high noise - The results for the high noise level.
│   ├── bar_plot_max ----> The histogram plot for searching for the maximum.
│   ├── bar_plot_median ----> The histogram plot for searching for the median.
│   ├── bar_plot_min ----> The histogram plot for searching for the minimum.
│   ├── noise_plot_max ----> The noise plot for searching for the maximum.
│   ├── noise_plot_median ----> The noise plot for searching for the median.
│   └── noise_plot_min ----> The noise plot for searching for the minimum.
│   
├── low noise - The results for the low noise level.
├── medium noise - The results for the low noise level.
│   
└───scatter - The scatter plots for the different noise levels.
    ├── scatter_plot_max ----> The scatter plot for searching for the maximum.
    ├── scatter_plot_median ----> The scatter plot for searching for the median.
    └── scatter_plot_min ----> The scatter plot for searching for the minimum.
```

## Benchmarks
The benchmarks for the original and the DP protocol can be run by executing the following command:
```shell
cargo bench
```
You will find the results in the `bench_results` directory:
```
bench_results/
├── Protocol DP ----> Contains the benchmarks for the DP protocol for the different noise levels.
├── Protocol Leakage ---> Contains the benchmarks for the original protocol.
```
To view the generated plots all in one, you can open the respective `report/index.html` file in your browser. 

---
### References
[1] Chandran, Gowri R., et al. "Comparison-based MPC in Star Topology." (2022). URL:
[https://encrypto.de/papers/CHHS22.pdf](https://encrypto.de/papers/CHHS22.pdf)