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

def fill_missing_elements(iterations, metrics):
    range_ = max(iterations) - min(iterations) + 1
    filled_iterations = []
    filled_metrics = []
    for i in range(min(iterations), max(iterations) + 1):
        if i in iterations:
            index = iterations.index(i)
            filled_iterations.append(iterations[index])
            filled_metrics.append(metrics[index])
        else:
            filled_iterations.append(i)
            filled_metrics.append(-1)
    return filled_iterations, filled_metrics

def plotResults():
    plt.figure()
    for i, benchmark in enumerate(benchmarks):
        x = [int(iteration) for iteration in benchmark["results"].keys()]
        y = [metrics["latency"]["secs"]+metrics["latency"]["nanos"]*1e-9 for metrics in benchmark["results"].values()]
        if len(y) != 0:
            average = sum(y) / len(y)
            print("The average time for block finalization for replica", i+1, "is:", average)
        filled_iterations, filled_metrics = fill_missing_elements(x, y)
        plt.subplot(N, 1, i+1)
        plt.bar(filled_iterations, filled_metrics, width=1)
        y = [0.5 if metrics["fp_finalization"] == True else 0 for metrics in benchmark["results"].values()]
        total_fp_finalizations = y.count(0.5)
        total_ic_finalizations = y.count(0)
        print("The number of iterations in which the block was FP finalized:", total_fp_finalizations)
        print("The number of iterations in which the block was IC finalized:", total_ic_finalizations)
        _, filled_metrics = fill_missing_elements(x, y)
        total_non_finalizations = filled_metrics.count(-1)
        print("The number of iterations in which a block wasn't finalized:", total_non_finalizations)
        plt.subplot(N, 1, i+1)
        plt.bar(filled_iterations, filled_metrics, width=1)
    plt.show()



COD = True
N = 6
F = 1
P = 1
T = 300

print("Runnning " + ("Fast IC Consensus" if COD else "original IC Consensus"))

processes = startSubnet()

waitForSubnetTermination()

benchmarks = getBenchmarks()

plotResults()
