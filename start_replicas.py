import requests
import subprocess
import time
import os
import json
import matplotlib.pyplot as plt

peers = [
    {
        "number": "2",
        "ip": "18.207.122.55",
        "web_server_port": "56790",
        "libp2p_port": "56789",
        "key_file": "ficc_peer_b_aws_rsa_key.pem",
        "id": "",
        "remote_peers_addresses": "",
    },
    {
        "number": "3",
        "ip": "3.89.57.68",
        "web_server_port": "56790",
        "libp2p_port": "56789",
        "key_file": "ficc_peer_b_aws_rsa_key.pem",
        "id": "",
        "remote_peers_addresses": "",
    },
    {
        "number": "4",
        "ip": "3.90.64.216",
        "web_server_port": "56790",
        "libp2p_port": "56789",
        "key_file": "peer_2_nw_aws_rsa_key.pem",
        "id": "",
        "remote_peers_addresses": "",
    },
    {
        "number": "5",
        "ip": "54.90.237.53",
        "web_server_port": "56790",
        "libp2p_port": "56789",
        "key_file": "ficc_peer_bb_aws_rsa_key.pem",
        "id": "",
        "remote_peers_addresses": "",
    },{
        "number": "6",
        "ip": "54.221.129.22",
        "web_server_port": "56790",
        "libp2p_port": "56789",
        "key_file": "ficc_peer_bb_aws_rsa_key.pem",
        "id": "",
        "remote_peers_addresses": "",
    },{
        "number": "7",
        "ip": "3.80.25.119",
        "web_server_port": "56790",
        "libp2p_port": "56789",
        "key_file": "ficc_peer_bb_aws_rsa_key.pem",
        "id": "",
        "remote_peers_addresses": "",
    },{
        "number": "8",
        "ip": "34.204.49.128",
        "web_server_port": "56790",
        "libp2p_port": "56789",
        "key_file": "ficc_peer_bb_aws_rsa_key.pem",
        "id": "",
        "remote_peers_addresses": "",
    },{
        "number": "9",
        "ip": "54.242.231.70",
        "web_server_port": "56790",
        "libp2p_port": "56789",
        "key_file": "ficc_peer_bb_aws_rsa_key.pem",
        "id": "",
        "remote_peers_addresses": "",
    },
    {
        "number": "10",
        "ip": "54.176.162.128",
        "web_server_port": "56790",
        "libp2p_port": "56789",
        "key_file": "ficc_peer_a_aws_rsa_key.cer",
        "id": "",
        "remote_peers_addresses": "",
    },
    {
        "number": "11",
        "ip": "13.57.221.56",
        "web_server_port": "56790",
        "libp2p_port": "56789",
        "key_file": "ficc_peer_a_aws_rsa_key.cer",
        "id": "",
        "remote_peers_addresses": "",
    },
    {
        "number": "12",
        "ip": "3.101.26.91",
        "web_server_port": "56790",
        "libp2p_port": "56789",
        "key_file": "peer_7_cali_aws_rsa_key.pem",
        "id": "",
        "remote_peers_addresses": "",
    },
    {
        "number": "13",
        "ip": "13.52.163.27",
        "web_server_port": "56790",
        "libp2p_port": "56789",
        "key_file": "ficc_peer_a_aws_rsa_key.cer",
        "id": "",
        "remote_peers_addresses": "",
    },
    {
        "number": "14",
        "ip": "54.219.209.156",
        "web_server_port": "56790",
        "libp2p_port": "56789",
        "key_file": "ficc_peer_ab_aws_rsa_key.pem",
        "id": "",
        "remote_peers_addresses": "",
    },
    {
        "number": "15",
        "ip": "54.67.18.230",
        "web_server_port": "56790",
        "libp2p_port": "56789",
        "key_file": "ficc_peer_ab_aws_rsa_key.pem",
        "id": "",
        "remote_peers_addresses": "",
    },
    {
        "number": "16",
        "ip": "184.169.205.232",
        "web_server_port": "56790",
        "libp2p_port": "56789",
        "key_file": "ficc_peer_ab_aws_rsa_key.pem",
        "id": "",
        "remote_peers_addresses": "",
    },
    {
        "number": "1",
        "ip": "52.53.194.157",
        "web_server_port": "56790",
        "libp2p_port": "56789",
        "key_file": "ficc_peer_ab_aws_rsa_key.pem",
        "id": "",
        "remote_peers_addresses": "",
    },
]

N = len(peers)
F = 3
P = 3
T = 300
D = 3000
FICC = True

print("\nStarting subnet running " + ("FICC" if FICC else "ICC") + f" with n={N}, f={F} and p={P}")

for peer in peers:
    with open(".env.example", "r") as file:
        contents = file.readlines()
        contents[0] = "REPLICA_NUMBER="+peer["number"]+"\n"
        contents[1] = "TOTAL_REPLICA_NUMBER="+str(N)+"\n"
        contents[2] = "FAULTY_REPLICAS="+str(F)+"\n"
        contents[3] = "DISAGREEING_REPLICA="+str(P)+"\n"
        contents[4] = "EXECUTION_TIME="+str(T)+"\n"
        contents[5] = "NOTARIZATION_DELAY="+str(D)+"\n"

    with open("./.env.example", "w") as file:
        file.writelines(contents)

    os.chmod("./keys/"+peer["key_file"], 0o400)
    set_params_cmd = f'scp -i ./keys/{peer["key_file"]} ./.env.example ubuntu@{peer["ip"]}:consensus_on_demand/.env'
    subprocess.run(set_params_cmd, shell=True)

with open("docker-compose.yml", "r") as file:
    contents = file.readlines()
    if FICC:
        contents[8] = '    command: ["--cod", "--r", $REPLICA_NUMBER, "--n", $TOTAL_REPLICA_NUMBER, "--f", $FAULTY_REPLICAS, "--p", $DISAGREEING_REPLICA, "--t", $EXECUTION_TIME, "--d", $NOTARIZATION_DELAY, "--broadcast_interval", "$BROADCAST_INTERVAL", "--port", $PORT]\n'
    else:
        contents[8] = '    command: ["--r", $REPLICA_NUMBER, "--n", $TOTAL_REPLICA_NUMBER, "--f", $FAULTY_REPLICAS, "--p", $DISAGREEING_REPLICA, "--t", $EXECUTION_TIME, "--d", $NOTARIZATION_DELAY, "--broadcast_interval", "$BROADCAST_INTERVAL", "--port", $PORT]\n'

with open("docker-compose.yml", "w") as file:
    file.writelines(contents)

for peer in peers:
    set_protocol_cmd = f'scp -i ./keys/{peer["key_file"]} ./docker-compose.yml ubuntu@{peer["ip"]}:consensus_on_demand/docker-compose.yml'
    subprocess.run(set_protocol_cmd, shell=True)

print("\nReplicas parameters set")

processes = []
for peer in peers:
    start_replica_cmd = f'ssh -i ./keys/{peer["key_file"]} -t -q ubuntu@{peer["ip"]} "cd consensus_on_demand && docker compose up --build"'
    process = subprocess.Popen(start_replica_cmd, shell=True, stdout=subprocess.DEVNULL)
    processes.append(process)

print("\nReplicas started")

time.sleep(100)  # wait for docker containers to start

print("\nConnecting peers")

for peer in peers: 
    response = requests.get("http://"+peer["ip"]+":"+peer["web_server_port"]+"/local_peer_id")
    if response.status_code == 200:
        peer["id"] = response.text[1:-1]
    else:
        print("Peer " + peer["number"] + " not reachable")

for i,peer in enumerate(peers):
    remote_peers_addresses = ""
    for j,other_peer in enumerate(peers):
        if i != j:
            remote_peers_addresses += "/ip4/"+other_peer["ip"]+"/tcp/"+other_peer["libp2p_port"]+"/p2p/"+other_peer["id"]+","
    peer["remote_peers_addresses"] = remote_peers_addresses[0:-1]

for peer in peers:
    requests.post("http://"+peer["ip"]+":"+peer["web_server_port"]+"/remote_peers_addresses", data=peer["remote_peers_addresses"])

print(f"\nProtocol started for {T} seconds")

for p in processes:
    p.communicate() # waits for replica to finish

print("\nReplicas stopped")

now = int(time.time())
folder = f'{("FICC" if FICC else "ICC")}_{N}_{F}_{P}_{D}_{T}_{now}'
subprocess.run(f'cd benchmark && mkdir {folder}', shell=True, stdout=subprocess.DEVNULL)

for peer in peers:
    get_benchmark_results_cmd = f'scp -i ./keys/{peer["key_file"]} ubuntu@{peer["ip"]}:consensus_on_demand/benchmark/benchmark_results.json benchmark/{("FICC" if FICC else "ICC")}_{N}_{F}_{P}_{D}_{T}_{now}/benchmark_results_{peer["number"]}.json'
    subprocess.run(get_benchmark_results_cmd, shell=True, stdout=subprocess.DEVNULL)

print(f'\nResults written in folder benchmark/{folder}')

def getBenchmarks():
    with open(f'./benchmark/{folder}/benchmark_results_1.json', 'r') as f:
        return json.loads(f.read())

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
    average_latency,
    total_fp_finalizations,
    total_ic_finalizations,
    total_dk_finalizations,
    total_non_finalizations,
):
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
    total_dk_finalizations = filled_finalization_types.count("DK")
    total_non_finalizations = filled_finalization_types.count("-")

    return (
        average_latency,
        total_fp_finalizations,
        total_ic_finalizations,
        total_dk_finalizations,
        total_non_finalizations,
    )

def plotLatencies(ax, filled_iterations, filled_latencies, filled_finalization_types):
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
    plt.plot() 
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

    ax = plt.gca()
    ax.set_xlabel("Round")
    ax.set_ylabel("Latency [s]")
    plotLatencies(plt.gca(), filled_iterations, filled_latencies, filled_finalization_types)
    plt.show()

    printMetrics(
        average_latency,
        total_fp_finalizations,
        total_ic_finalizations,
        total_dk_finalizations,
        total_non_finalizations,
    )

print("Displaying results for replica 1")

benchmark = getBenchmarks()

getResults()
