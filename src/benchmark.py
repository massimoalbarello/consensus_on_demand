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

def fillMissingElements(iterations, metrics, default_element):
    filled_iterations = []
    filled_metrics = []
    for i in range(min(iterations), max(iterations) + 1):
        if i in iterations:
            index = iterations.index(i)
            filled_iterations.append(iterations[index])
            filled_metrics.append(metrics[index])
        else:
            filled_iterations.append(i)
            filled_metrics.append(default_element)
    return filled_iterations, filled_metrics

def countFpSequences(first_index_offset, arr):
    sequences = []

    sequence_length = None
    not_finalized_sequence_len = 0
    starting_index = 0 # genesis block is IC finalized
    for i in range(len(arr)):
        if arr[i] == 'IC':
            # register previous sequence
            sequences.append({
                "length": sequence_length,
                "IC_index": starting_index,
            })
            # initialize new sequence
            starting_index = i+first_index_offset
            sequence_length = 0
        elif arr[i] == 'FP':
            if sequence_length == None: # sequence length can be "None" only for the first sequence
                sequence_length = i+first_index_offset
            else:
                sequence_length += 1
        elif arr[i] == '-':
            not_finalized_sequence_len += 1
            if i < len(arr)-1 and arr[i+1] == 'IC':
                not_finalized_sequence_len = 0
            elif i < len(arr)-1 and arr[i+1] == 'FP':
                sequence_length += not_finalized_sequence_len
                not_finalized_sequence_len = 0

    # register last sequence
    sequences.append({
        "length": sequence_length,
        "IC_index": starting_index, 
    })

    return sequences

def printStatistics(i, first_index_offset, latencies, filled_finalization_types):
    print("\n### Replica", i+1, "###")
    if len(latencies) != 0:
        average = sum(latencies) / len(latencies)
        print("The average time for block finalization is:", average)
    total_fp_finalizations = filled_finalization_types.count("FP")
    total_ic_finalizations = filled_finalization_types.count("IC")
    total_non_finalizations = filled_finalization_types.count("-")
    print("The number of iterations in which the block is:")
    print("- FP finalized:", total_fp_finalizations)
    print("- IC finalized:", total_ic_finalizations)
    print("- not explicitly finalized:", total_non_finalizations)
    if COD:
        sequences = countFpSequences(first_index_offset, filled_finalization_types)
        print("Found", len(sequences), "sequences:")
        for sequence in sequences:
            print("- starting at", sequence["IC_index"], "with length", sequence["length"])

def plotResults():
    plt.figure()
    for i, benchmark in enumerate(benchmarks):
        iterations = [int(iteration) for iteration in benchmark["results"].keys()]
        latencies = [metrics["latency"]["secs"]+metrics["latency"]["nanos"]*1e-9 for metrics in benchmark["results"].values()]
        filled_iterations, filled_latencies = fillMissingElements(iterations, latencies, 0)
        finalization_types = ["FP" if metrics["fp_finalization"] == True else "IC" for metrics in benchmark["results"].values()]
        _, filled_finalization_types = fill_missing_elements(iterations, finalization_types, "-")
        printStatistics(i, filled_iterations[0], latencies, filled_finalization_types)
        ax = plt.subplot(N, 1, i+1)
        for j, type in enumerate(filled_finalization_types):
            if type == "FP":
                ax.bar(filled_iterations[j], filled_latencies[j], width=1, color='green')
            elif type == "IC":
                ax.bar(filled_iterations[j], filled_latencies[j], width=1, color="blue" )
        if i == 0:
            xlim = ax.get_xlim()
        else:
            ax.set_xlim(xlim)
    plt.show()



COD = True
N = 6
F = 1
P = 1
T = 100

print("Runnning " + ("Fast IC Consensus" if COD else "original IC Consensus"))

processes = startSubnet()

waitForSubnetTermination()

benchmarks = getBenchmarks()

plotResults()
