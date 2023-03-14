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

def printMetrics(
    i,
    average_latency,
    total_fp_finalizations,
    total_ic_finalizations,
    total_dk_finalizations,
    total_non_finalizations,
):
    print("\n### Replica", i+1, "###")
    print("The average time for block finalization is:", average_latency)
    print("The number of iterations in which the block is:")
    print("- FP finalized:", total_fp_finalizations)
    print("- IC finalized:", total_ic_finalizations)
    print("- DK finalized:", total_dk_finalizations)
    print("- not explicitly finalized:", total_non_finalizations)

def processResults(latencies, filled_iterations, filled_finalization_types):
    average_latency = None
    if len(latencies) != 0:
        average_latency = sum(latencies) / len(latencies)
    total_fp_finalizations = filled_finalization_types.count("FP")
    total_ic_finalizations = filled_finalization_types.count("IC")
    total_dk_finalizations = filled_finalization_types.count("Dk")
    total_non_finalizations = filled_finalization_types.count("-")

    return (
        average_latency,
        total_fp_finalizations,
        total_ic_finalizations,
        total_dk_finalizations,
        total_non_finalizations,
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
    colors = ["green", "blue", "grey"]
    color_labels = {
        "green": "FP-finalized block",
        "blue": "IC-finalized block",
        "grey": "finalization from peer"
    }
    fp_bar = None
    ic_bar = None
    for j, type in enumerate(filled_finalization_types):
        if type == "FP":
            fp_bar = ax.bar(filled_iterations[j], filled_latencies[j], width=1, color=colors[0], label=color_labels[colors[0]])
        elif type == "IC":
            ic_bar = ax.bar(filled_iterations[j], filled_latencies[j], width=1, color=colors[1], label=color_labels[colors[1]])
        elif type == "DK":
            ic_bar = ax.bar(filled_iterations[j], filled_latencies[j], width=1, color=colors[2], label=color_labels[colors[2]])
    handles = [fp_bar, ic_bar]
    labels = ["FP-finalized block", "IC-finalized block", "finalization from peer"]
    ax.legend(handles, labels, loc="upper right")

def getResults():
    fig, axs = plt.subplots(N, 1)
    fig.suptitle("Block finalization latency using " + ("FICC" if COD else "ICC"))
    delays_info = {}
    for i, benchmark in enumerate(benchmarks):
        iterations = [int(iteration) for iteration in benchmark["finalization_times"].keys()]
        latencies = [metrics["latency"]["secs"]+metrics["latency"]["nanos"]*1e-9 for metrics in benchmark["finalization_times"].values()]
        filled_iterations, filled_latencies = fillMissingElements(iterations, latencies, 0)
        finalization_types = [metrics["fp_finalization"] for metrics in benchmark["finalization_times"].values()]
        _, filled_finalization_types = fillMissingElements(iterations, finalization_types, "-")

        (
            average_latency,
            total_fp_finalizations,
            total_ic_finalizations,
            total_dk_finalizations,
            total_non_finalizations,
        ) = processResults(latencies, filled_iterations, filled_finalization_types)

        printMetrics(
            i,
            average_latency,
            total_fp_finalizations,
            total_ic_finalizations,
            total_dk_finalizations,
            total_non_finalizations,
        )

        ax_lat = axs[i]
        ax_lat.set_xlabel("Iteration")
        ax_lat.set_ylabel("Latency [secs]")

        plotLatencies(i, ax_lat, filled_iterations, filled_latencies, filled_finalization_types)
        if i == 0:
            xlim_lat = ax_lat.get_xlim()
        else:
            ax_lat.set_xlim(xlim_lat)
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
