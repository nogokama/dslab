import matplotlib.pyplot as plt
import numpy as np

# Read data from the file
with open('load.txt', 'r') as file:
    data = file.read()

# Parse the data into separate lists
types, times, machine_ids, loads = zip(*(line.split(', ') for line in data.strip().split('\n')))

# Convert lists to appropriate data types
times = np.array([float(time) for time in times])
machine_ids = np.array([str(machine_id) for machine_id in machine_ids])
loads = np.array([float(load) for load in loads])

# Filter data based on type (e.g., cpu)
cpu_indices = [i for i, t in enumerate(types) if t == 'cpu']
mem_indices = [i for i, t in enumerate(types) if t == 'mem']

# Get unique machine IDs
unique_machine_ids = np.unique(machine_ids)

# Create a figure with a grid of subplots
num_rows = len(unique_machine_ids)
fig, axes = plt.subplots(num_rows, 2, figsize=(25, 5 * num_rows), sharex='col')

# Loop through each machine ID
for row, machine_id in enumerate(unique_machine_ids):
    # Plot CPU load
    cpu_indices_machine = np.where((machine_ids == machine_id) & np.isin(range(len(types)), cpu_indices))
    axes[row, 0].plot(times[cpu_indices_machine], loads[cpu_indices_machine], label=f'Machine {machine_id}', linestyle='-', marker='')
    axes[row, 0].set_ylabel('Load')
    axes[row, 0].set_title(f'Machine {machine_id} - CPU Load')
    axes[row, 0].legend()

    # Plot Memory load
    mem_indices_machine = np.where((machine_ids == machine_id) & np.isin(range(len(types)), mem_indices))
    axes[row, 1].plot(times[mem_indices_machine], loads[mem_indices_machine], label=f'Machine {machine_id}', linestyle='-', marker='')
    axes[row, 1].set_ylabel('Load')
    axes[row, 1].set_title(f'Machine {machine_id} - Memory Load')
    axes[row, 1].legend()

# Set common x-axis label
axes[-1, 0].set_xlabel('Time')
axes[-1, 1].set_xlabel('Time')

# Adjust layout
plt.tight_layout()

# Save the plot
plt.savefig('timeseries.png')
