import subprocess
import time

procs = []
for i in range(2, 7):
    procs.append(subprocess.Popen(['cargo run -- --cod --t 10 --r ' + str(i)], shell=True))

# replica number 1 must be start last
time.sleep(5)
procs.append(subprocess.Popen(['cargo run -- --cod --t 5 --r 1'], shell=True))

data = []
for i, p in enumerate(procs):
    p.communicate()
    with open('benchmark_result_' + str(i+1) + '.json', 'r') as f:
        data.append(f.read())

print("\n", data)