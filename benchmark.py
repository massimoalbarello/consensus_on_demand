import os
import json
import matplotlib.pyplot as plt

results = [
    {
        "folder": "./benchmark/ICC_16_5_0_3000_300_1680773781",
        "label": "ICC n=16 f=5"
    },
    {
        "folder": "./benchmark/FICC_16_5_0_3000_300_1680774307",
        "label": "FICC n=16 f=5 p=0"
    },
    {
        "folder": "./benchmark/FICC_16_3_3_3000_300_1680774842",
        "label": "FICC n=16 f=3 p=3"
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
plt.ylabel('Latency (seconds)')
plt.show()