import json
import matplotlib.pyplot as plt
import statistics

def getBenchmarks():
    results = []
    with open('./benchmark/benchmark_results.json', 'r') as f:
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
                "length": sequence_length if sequence_length != None else 0,
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

def printMetrics(
    i,
    average_latency,
    total_fp_finalizations,
    total_ic_finalizations,
    total_non_finalizations,
    sequences,
):
    print("\n### Replica", i+1, "###")
    print("The average time for block finalization is:", average_latency)
    print("The number of iterations in which the block is:")
    print("- FP finalized:", total_fp_finalizations)
    print("- IC finalized:", total_ic_finalizations)
    print("- not explicitly finalized:", total_non_finalizations)
    print("Found", len(sequences), "sequences:")
    for sequence in sequences:
        print("- starting at", sequence["IC_index"], "with length", sequence["length"])


def processResults(latencies, filled_iterations, filled_finalization_types, delays_info, proposals_timings):
    average_latency = None
    if len(latencies) != 0:
        average_latency = sum(latencies) / len(latencies)
    total_fp_finalizations = filled_finalization_types.count("FP")
    total_ic_finalizations = filled_finalization_types.count("IC")
    total_non_finalizations = filled_finalization_types.count("-")
    sequences = []
    sequences_length = []
    if COD:
        sequences = countFpSequences(filled_iterations[0], filled_finalization_types)
        for sequence in sequences:
            sequences_length.append(sequence["length"])

    for hash, timings in proposals_timings.items():
        if hash not in delays_info:
            delays_info[hash] = {
                "sent": None,
                "received": [],
            }
        if timings["sent"] != None:
            delays_info[hash]["sent"] = timings["sent"]
        elif timings["received"] != None:
            delays_info[hash]["received"].append(timings["received"])
        else:
            print("Replica didn't send nor receive block", hash)

    return (
        average_latency,
        total_fp_finalizations,
        total_ic_finalizations,
        total_non_finalizations,
        sequences,
        sequences_length
    )

def plotSequenceLengthDistribution(ax, arr):
    frequencies = {}
    for j in arr:
        if j in frequencies:
            frequencies[j] += 1
        else:
            frequencies[j] = 1

    ax.bar(frequencies.keys(), frequencies.values(), width=1, color='orange')

def plotLatencies(i, ax, filled_iterations, filled_latencies, filled_finalization_types):
    colors = ["green", "blue"]
    color_labels = {
        "green": "FP-finalized block",
        "blue": "IC-finalized block"
    }
    fp_bar = None
    ic_bar = None
    for j, type in enumerate(filled_finalization_types):
        if type == "FP":
            fp_bar = ax.bar(filled_iterations[j], filled_latencies[j], width=1, color=colors[0], label=color_labels[colors[0]])
        elif type == "IC":
            ic_bar = ax.bar(filled_iterations[j], filled_latencies[j], width=1, color=colors[1], label=color_labels[colors[1]])
    handles = [fp_bar, ic_bar]
    labels = ["FP-finalized block", "IC-finalized block"]
    ax.legend(handles, labels, loc="upper right")

def getResults():
    fig, axs = plt.subplots(N, 1)
    fig.suptitle("Block finalization latency using " + ("FICC" if COD else "ICC"))
    delays_info = {}
    for i, benchmark in enumerate(benchmarks):
        iterations = [int(iteration) for iteration in benchmark["finalization_times"].keys()]
        latencies = [metrics["latency"]["secs"]+metrics["latency"]["nanos"]*1e-9 for metrics in benchmark["finalization_times"].values()]
        filled_iterations, filled_latencies = fillMissingElements(iterations, latencies, 0)
        finalization_types = ["FP" if metrics["fp_finalization"] == True else "IC" for metrics in benchmark["finalization_times"].values()]
        _, filled_finalization_types = fillMissingElements(iterations, finalization_types, "-")

        (
            average_latency,
            total_fp_finalizations,
            total_ic_finalizations,
            total_non_finalizations,
            sequences,
            sequences_length
        ) = processResults(latencies, filled_iterations, filled_finalization_types, delays_info, benchmark["proposals_timings"])

        printMetrics(
            i,
            average_latency,
            total_fp_finalizations,
            total_ic_finalizations,
            total_non_finalizations,
            sequences,
        )

        ax_lat = axs[i]
        ax_lat.set_xlabel("Iteration")
        ax_lat.set_ylabel("Latency [secs]")

        plotLatencies(i, ax_lat, filled_iterations, filled_latencies, filled_finalization_types)
        if i == 0:
            xlim_lat = ax_lat.get_xlim()
        else:
            ax_lat.set_xlim(xlim_lat)

        # if COD:
        #     ax_distr = plt.subplot(2*N, 1, N+i+1)
        #     plotSequenceLengthDistribution(ax_distr, sequences_length)
        #     if i == 0:
        #         xlim_distr = ax_distr.get_xlim()
        #     else:
        #         ax_distr.set_xlim(xlim_distr)
    plt.show()



COD = True          # use FICC (True) or ICC (False)
ADVERSARY_TYPE = 0  # 0: no adversary, 1: passive adversary
N = 3               # total number of replicas
F = 0               # number of corrupt replicas
P = 0               # number of replicas that can disagree during fast-path finalization
T = 60              # subnet simulation time (seconds)
D = 500             # artifct delay for block proposals and notarization shares (milliseconds)

if N <= 3*F + 2*P or P > F:
    print("Wrong parameters: must satisfy: N > 3F + 2P and P <= F")
elif T < 20:
    print("Subnet must be run for at least 60 seconds")
elif D < 100:
    print("Artifact delay must be at least 100 milliseconds")
else: 
    print(
        "Runnning " + 
        ("Fast IC Consensus" if COD else "original IC Consensus") + 
        " with " + 
        ("honest " if ADVERSARY_TYPE == 0 else "passive ") + 
        " adversary\n"
    )

    benchmarks = getBenchmarks()

    getResults()
