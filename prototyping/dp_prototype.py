import math
import multiprocessing
import random
import threading
from typing import List, Tuple
import numpy as np
from matplotlib import pyplot as plt


def count_elements(database: List[int], m: int) -> Tuple[int, int]:
    """
    Count elements in the database which are lesser than or greater than m.

    Args:
        database: The list of integers to analyze.
        m: The pivot element to compare others against.

    Returns:
        A tuple (L, G) where L is the count of elements less than m and G is the count of elements greater than m.
    """
    L = sum(i < m for i in database)
    G = sum(i > m for i in database)
    return L, G


class CentralParty:
    """
    A class representing the central party responsible for finding the minimum.

    Attributes:
        databases: List of lists, where each list is a database of integers.
        k: An integer to define the element to find.
        a: The minimum element among all databases.
        b: The maximum element among all databases.
        N: The total number of elements across all databases.
        i: A counter to track the number of steps.
        use_laplace: A boolean indicating whether to use Laplace noise. If false, a uniform noise is used.
        max_laplace_scale: The maximum scale for the Laplace noise.
    """

    def __init__(self, databases: List[List[int]], k: int, use_laplace: bool = False, max_laplace_scale: float = 1.0):
        self.databases = databases
        self.k = k
        self.a = min(min(db) for db in databases)
        self.b = max(max(db) for db in databases)
        self.N = len(databases[0]) * len(databases)
        self.i = 0
        self.use_laplace = use_laplace
        self.max_laplace_scale = max_laplace_scale

    def find_k(self, noise_prob):
        """
        Find the kth element across all databases.

        Args:
            noise_prob: Probability to add noise.
            use_laplace: If we want to use laplace or uniform

        Returns:
            A tuple (m, i) where m is the minimum value and i is the number of steps taken to find it.
        """
        while True:
            m = math.floor((self.a + self.b) / 2)
            self.i += 1

            L, G = 0, 0
            for db in self.databases:
                l, g = count_elements(db, m)

                if self.use_laplace:
                    l, g = count_elements(db, m)
                    # Draw a sample from the Laplace distribution
                    laplace_noise = np.random.laplace(scale=noise_prob * self.max_laplace_scale)
                    l = max(int(l + laplace_noise), 0)
                    g = max(int(g - laplace_noise), 0)
                else:
                    if not (random.random() > noise_prob):
                        l = max(l + random.randint(-1, 1), 0)
                        g = max(g - random.randint(-1, 1), 0)
                L += l
                G += g

            if L < self.k and G <= self.N - self.k:
                return m, self.i
            elif L >= self.k:
                self.b = m - 1
            else:  # L + big_E < k
                self.a = m + 1


def kth_element(databases: List[List[int]], k: int) -> int:
    """
    Determine the kth element in the databases. Used to get the reference value.

    Args:
        databases: List of lists, where each list is a database of integers.
        k: An integer to define the element to find.

    Returns:
        The kth element across all databases.
    """
    # Flatten the list of databases into a single list
    flat_list = [item for sublist in databases for item in sublist]

    # Sort the list
    sorted_list = sorted(flat_list)

    # Find and return the kth element
    return sorted_list[k - 1]


def plot_data(k, iterations_per_probability, resolution, databases, use_laplace, laplace_scale):
    """
    Plot the data for average deviation and average iterations needed.

    Args:
        k: An integer to define the element to find.
        iterations_per_probability: The number of iterations for each probability.
        resolution: The resolution for the range of probabilities.
        databases: List of lists, where each list is a database of integers.
        use_laplace: If to use laplace for noise
        laplace_scale: Scale for laplace
    """
    expected_result = kth_element(databases, k)
    avg_deviation = []
    avg_steps_needed = []

    for i in range(0, resolution):
        min_values = []
        steps_needed = []
        for _ in range(iterations_per_probability):
            # Initialize CentralParty object
            cp = CentralParty(databases, k, use_laplace=use_laplace, max_laplace_scale=laplace_scale)
            # Find minimum
            min_val, steps = cp.find_k(i / (resolution - 1))
            min_values.append(min_val)
            steps_needed.append(steps)

        # Calculate average deviation and average steps
        avg_deviation.append(np.mean(np.abs(np.array(min_values) - expected_result)))
        avg_steps_needed.append(np.mean(steps_needed))

    if use_laplace:
        probabilities = [(i / resolution) * laplace_scale for i in range(0, resolution)]
    else:
        probabilities = [(i / resolution) for i in range(0, resolution)]

    # Creating a figure with two y-axes
    fig, ax1 = plt.subplots()

    if use_laplace:
        plt.title(f"Laplace Noise, k = {k}")
    else:
        plt.title(f"Uniform Probability, k = {k}")

    color = 'tab:red'

    if use_laplace:
        ax1.set_xlabel('Laplace Scale')
    else:
        ax1.set_xlabel('Probability to add noise')

    ax1.set_ylabel('Average Deviation', color=color)
    ax1.plot(probabilities, avg_deviation, color=color)
    ax1.tick_params(axis='y', labelcolor=color)

    ax2 = ax1.twinx()
    color = 'tab:blue'
    ax2.set_ylabel('Average Iterations Needed', color=color)
    ax2.plot(probabilities, avg_steps_needed, color=color)
    ax2.tick_params(axis='y', labelcolor=color)

    fig.tight_layout()

    # Save the plot
    if use_laplace:
        fig.savefig(f"k_{k}_laplace.png")
    else:
        fig.savefig(f"k_{k}_uniform.png")

    # Show the plot
    plt.show()


def create_databases(amount, db_size, seed):
    random.seed(seed)
    databases = []
    for _ in range(amount):
        databases.append(sorted([random.randint(1, 100) for _ in range(db_size)]))
    return databases


def worker(k, databases, use_laplace):
    plot_data(k=k, iterations_per_probability=1000, resolution=100, databases=databases, use_laplace=use_laplace,
              laplace_scale=2)


if __name__ == '__main__':
    PARTIES = 3  # Amount of parties
    DB_SIZE = 100  # Number of elements per party

    # Databases will be the same for a fixed seed
    databases = create_databases(amount=PARTIES, db_size=DB_SIZE, seed="some_seed_to_fix_db")

    # We are looking for [min, med, max]
    k_values = [1, math.floor((PARTIES * DB_SIZE) / 2), PARTIES * DB_SIZE]

    # The computation is done in parallel to get results faster (especially useful for large data sets)
    jobs = []
    for use_laplace in [False, True]:
        for k in k_values:
            process = multiprocessing.Process(target=worker, args=(k, databases, use_laplace,))
            jobs.append(process)
            process.start()

    # Ensure all of the processes have finished
    for job in jobs:
        job.join()
