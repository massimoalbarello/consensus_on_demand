import subprocess
import time
import json
import matplotlib.pyplot as plt



def startReplica(procs, i):
    shellCommand = 'cargo run --quiet -- --n ' + str(N) + ' --f ' + str(F) + ' --p ' + str(P) + ' --t ' + str(T) + ' --r ' + str(i) + (' --cod' if COD else '')
    procs.append(subprocess.Popen([shellCommand], shell=True, stderr=subprocess.DEVNULL))

def startSubnet():
    procs = []
    for i in range(2, N+1):
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

def plotResults():
    fig, ax = plt.subplots()
    for i, benchmark in enumerate(benchmarks):
        x = [iteration for iteration, _ in benchmark["results"].items()]
        y = [time["secs"]+time["nanos"]*1e-9 for _, time in benchmark["results"].items()]
        if len(y) != 0:
            average = sum(y) / len(y)
            print("The average time for block finalization for replica", i+1, "is:", average)
        ax.plot(x, y, label=str(i+1))
        ax.legend()
    plt.show()



COD = True
N = 6
F = 1
P = 1
T = 180

print("Runnning " + ("Fast IC Consensus" if COD else "original IC Consensus"))

processes = startSubnet()

waitForSubnetTermination()

benchmarks = getBenchmarks()

plotResults()
