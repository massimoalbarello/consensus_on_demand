import subprocess
import time
import json

def startReplica(procs, i):
    if COD:
        procs.append(subprocess.Popen(['cargo run --quiet -- --cod --n ' + str(N) + ' --f ' + str(F) + ' --p ' + str(P) + ' --t ' + str(T) + ' --r ' + str(i)], shell=True, stderr=subprocess.DEVNULL))
    else:
        procs.append(subprocess.Popen(['cargo run --quiet -- --n ' + str(N) + ' --f ' + str(F) + ' --p ' + str(P) + ' --t ' + str(T) + ' --r ' + str(i)], shell=True, stderr=subprocess.DEVNULL))

def startSubnet():
    procs = []
    for i in range(2, 7):
        startReplica(procs, i)
    # replica number 1 must be start last
    time.sleep(5)
    startReplica(procs, 1)
    return procs

def waitForSubnetTermination():
    for p in processes:
        p.communicate() # waits for replica to finish and return result (none)


def getBenchmarks():
    results = []
    for i, _ in enumerate(processes):
        with open('benchmark_result_' + str(i+1) + '.json', 'r') as f:
            results.append(json.loads(f.read()))
    return results

COD = True
N = 6
F = 1
P = 1
T = 2000

processes = startSubnet()

waitForSubnetTermination()

benchmarks = getBenchmarks()

for benchmark in benchmarks:
    print("\n", benchmark["results"])
