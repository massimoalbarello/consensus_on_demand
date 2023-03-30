import os
import json
import statistics
import matplotlib.pyplot as plt

results = [
    {
        "folder": "./benchmark/ICC_16_0_0_3000_60_1680033093",
        "label": "ICC n=16 f=0"
    },
    {
        "folder": "./benchmark/res_16_2_0_3000_60_1680020111",
        "label": "FICC n=16 f=2 p=0"
    },
    {
        "folder": "./benchmark/res_16_0_0_3000_300_1680028847",
        "label": "FICC n=16 f=p=0"
    },
]

box_plots = []
labels = []

for res in results:
    subnet_latencies = []
    for filename in os.listdir(res["folder"]):
        if filename.endswith(".json"):
            with open(os.path.join(res["folder"], filename)) as f:
                data = json.load(f)
            # extract the latency values from the data
            subnet_latencies.extend([value["latency"]["secs"] + value["latency"]["nanos"] / 1000000000 for value in data['finalization_times'].values()])
    box_plots.append(subnet_latencies)
    labels.append(res["label"])

# create the boxplot
plt.boxplot(box_plots, labels=labels, showfliers=True, flierprops=dict(marker='o', markerfacecolor='gray', markersize=4, linestyle='none', markeredgecolor='gray'))
plt.title('Latency Distribution')
plt.ylabel('Latency (seconds)')
plt.show()