import subprocess
import re
import matplotlib.pyplot as plt

# range from 100 to 1000, incrementing by 100
iterations = range(100, 4100, 100)
# proc_count = range(2000, 5000, 100)
# peers_count = range(1, 2000, 100)
elapsed_times = []

ord_elapsed = []

for i in iterations:
    command = f'cargo run --release -- --iterations {i} --peer-count 100 --proc-count 10000 --use-network'
    output = subprocess.check_output(command, shell=True, text=True)
    elapsed_time = float(re.findall(
        r'Processed \d+ iterations in (\d+\.\d+)s', output)[0])
    elapsed_times.append(elapsed_time)

    command = f'cargo run --package ping-pong --release -- --iterations {i} --peer-count 100 --proc-count 10000 --use-network'
    output = subprocess.check_output(command, shell=True, text=True)
    elapsed_time = float(re.findall(
        r'Processed \d+ iterations in (\d+\.\d+)s', output)[0])
    ord_elapsed.append(elapsed_time)

plt.plot(iterations, elapsed_times, label='async-ping-pong')
plt.plot(iterations, ord_elapsed, label='ping-pong')
plt.xlabel('Iterations count')
plt.ylabel('Elapsed time (s)')
plt.legend()
plt.show()
