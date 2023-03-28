import requests
import subprocess
import time
import os

peers = {
    "peer_2": {
        "number": "2",
        "ip": "54.167.6.101",
        "web_server_port": "56790",
        "libp2p_port": "56789",
        "key_file": "peer_2_nw_aws_rsa_key.pem",
        "id": "",
        "remote_peers_addresses": "",
    },
    "peer_3": {
        "number": "3",
        "ip": "18.231.166.74",
        "web_server_port": "56790",
        "libp2p_port": "56789",
        "key_file": "peer_3_sao_aws_rsa_key.pem",
        "id": "",
        "remote_peers_addresses": ""
    },
    "peer_4": {
        "number": "4",
        "ip": "54.206.18.218",
        "web_server_port": "56790",
        "libp2p_port": "56789",
        "key_file": "peer_4_syd_aws_rsa_key.pem",
        "id": "",
        "remote_peers_addresses": ""
    },
    "peer_5": {
        "number": "5",
        "ip": "3.124.187.44",
        "web_server_port": "56790",
        "libp2p_port": "56789",
        "key_file": "peer_5_frank_aws_rsa_key.pem",
        "id": "",
        "remote_peers_addresses": ""
    },
    "peer_6": {
        "number": "6",
        "ip": "13.51.79.234",
        "web_server_port": "56790",
        "libp2p_port": "56789",
        "key_file": "peer_6_stock_aws_rsa_key.pem",
        "id": "",
        "remote_peers_addresses": ""
    },
    "peer_7": {
        "number": "7",
        "ip": "54.183.235.91",
        "web_server_port": "56790",
        "libp2p_port": "56789",
        "key_file": "peer_7_cali_aws_rsa_key.pem",
        "id": "",
        "remote_peers_addresses": ""
    },
    "peer_1": {
        "number": "1",
        "ip": "13.212.122.154",
        "web_server_port": "56790",
        "libp2p_port": "56789",
        "key_file": "peer_1_sing_aws_rsa_key.pem",
        "id": "",
        "remote_peers_addresses": ""
    },
}

N = len(peers.keys())
F = 0
P = 0
T = 60
FICC = True

print("\nStarting subnet running " + ("FICC" if FICC else "ICC") + f" with n={N}, f={F} and p={P}")

for peer in peers.values():
    with open(".env.example", "r") as file:
        contents = file.readlines()
        contents[0] = "REPLICA_NUMBER="+peer["number"]+"\n"
        contents[1] = "TOTAL_REPLICA_NUMBER="+str(N)+"\n"
        contents[2] = "FAULTY_REPLICAS="+str(F)+"\n"
        contents[3] = "DISAGREEING_REPLICA="+str(P)+"\n"
        contents[4] = "EXECUTION_TIME="+str(T)+"\n"

    with open("./.env.example", "w") as file:
        file.writelines(contents)

    os.chmod("./keys/"+peer["key_file"], 0o400)
    set_params_cmd = f'scp -i ./keys/{peer["key_file"]} ./.env.example ubuntu@{peer["ip"]}:consensus_on_demand/.env'
    subprocess.run(set_params_cmd, shell=True)

print("\nReplicas parameters set")

processes = []
for peer in peers.values():
    start_replica_cmd = f'ssh -i ./keys/{peer["key_file"]} -t -q ubuntu@{peer["ip"]} "cd consensus_on_demand && docker compose up --build"'
    process = subprocess.Popen(start_replica_cmd, shell=True, stdout=subprocess.DEVNULL)
    processes.append(process)

print("\nReplicas started")

time.sleep(30)  # wait for docker containers to start

print("\nConnecting peers")

for peer in peers.values(): 
    response = requests.get("http://"+peer["ip"]+":"+peer["web_server_port"]+"/local_peer_id")
    if response.status_code == 200:
        peer["id"] = response.text[1:-1]

for i,peer in enumerate(peers.values()):
    remote_peers_addresses = ""
    for j,other_peer in enumerate(peers.values()):
        if i != j:
            remote_peers_addresses += "/ip4/"+other_peer["ip"]+"/tcp/"+other_peer["libp2p_port"]+"/p2p/"+other_peer["id"]+","
    peer["remote_peers_addresses"] = remote_peers_addresses[0:-1]

for peer in peers.values():
    requests.post("http://"+peer["ip"]+":"+peer["web_server_port"]+"/remote_peers_addresses", data=peer["remote_peers_addresses"])

print("\nProtocol started")

for p in processes:
    p.communicate() # waits for replica to finish

print("\nReplicas stopped")

for peer in peers.values():
    get_benchmark_results_cmd = f'scp -i ./keys/{peer["key_file"]} ubuntu@{peer["ip"]}:consensus_on_demand/benchmark/benchmark_results.json benchmark/benchmark_results_{peer["number"]}.json'
    subprocess.run(get_benchmark_results_cmd, shell=True, stdout=subprocess.DEVNULL)