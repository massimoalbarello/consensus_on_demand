import requests
import subprocess
import time
import os

peers = {
    "peer_2": {
        "number": "2",
        "ip": "3.84.220.66",
        "web_server_port": "56790",
        "libp2p_port": "56789",
        "key_file": "peer_2_nw_aws_rsa_key.pem",
        "id": "",
        "remote_peers_addresses": "",
    },
    "peer_3": {
        "number": "3",
        "ip": "15.228.190.23",
        "web_server_port": "56790",
        "libp2p_port": "56789",
        "key_file": "peer_3_sao_aws_rsa_key.pem",
        "id": "",
        "remote_peers_addresses": ""
    },
    "peer_4": {
        "number": "4",
        "ip": "13.239.15.224",
        "web_server_port": "56790",
        "libp2p_port": "56789",
        "key_file": "peer_4_syd_aws_rsa_key.pem",
        "id": "",
        "remote_peers_addresses": ""
    },
    "peer_5": {
        "number": "5",
        "ip": "18.198.2.4",
        "web_server_port": "56790",
        "libp2p_port": "56789",
        "key_file": "peer_5_frank_aws_rsa_key.pem",
        "id": "",
        "remote_peers_addresses": ""
    },
    "peer_6": {
        "number": "6",
        "ip": "16.171.6.33",
        "web_server_port": "56790",
        "libp2p_port": "56789",
        "key_file": "peer_6_stock_aws_rsa_key.pem",
        "id": "",
        "remote_peers_addresses": ""
    },
    "peer_7": {
        "number": "7",
        "ip": "54.153.46.116",
        "web_server_port": "56790",
        "libp2p_port": "56789",
        "key_file": "peer_7_cali_aws_rsa_key.pem",
        "id": "",
        "remote_peers_addresses": ""
    },
    "peer_8": {
        "number": "8",
        "ip": "3.20.225.7",
        "web_server_port": "56790",
        "libp2p_port": "56789",
        "key_file": "peer_8_ohio_aws_rsa_key.pem",
        "id": "",
        "remote_peers_addresses": "",
    },
    "peer_9": {
        "number": "9",
        "ip": "35.93.0.69",
        "web_server_port": "56790",
        "libp2p_port": "56789",
        "key_file": "peer_9_oreg_aws_rsa_key.pem",
        "id": "",
        "remote_peers_addresses": "",
    },
    "peer_10": {
        "number": "10",
        "ip": "3.111.51.88",
        "web_server_port": "56790",
        "libp2p_port": "56789",
        "key_file": "peer_10_mum_aws_rsa_key.pem",
        "id": "",
        "remote_peers_addresses": "",
    },
    "peer_11": {
        "number": "11",
        "ip": "13.208.168.106",
        "web_server_port": "56790",
        "libp2p_port": "56789",
        "key_file": "peer_11_osaka_aws_rsa_key.pem",
        "id": "",
        "remote_peers_addresses": "",
    },
    "peer_12": {
        "number": "12",
        "ip": "3.36.69.67",
        "web_server_port": "56790",
        "libp2p_port": "56789",
        "key_file": "peer_12_seo_aws_rsa_key.pem",
        "id": "",
        "remote_peers_addresses": "",
    },
    "peer_13": {
        "number": "13",
        "ip": "52.194.251.20",
        "web_server_port": "56790",
        "libp2p_port": "56789",
        "key_file": "peer_13_tok_aws_rsa_key.pem",
        "id": "",
        "remote_peers_addresses": "",
    },
    "peer_14": {
        "number": "14",
        "ip": "15.223.47.218",
        "web_server_port": "56790",
        "libp2p_port": "56789",
        "key_file": "peer_14_can_aws_rsa_key.pem",
        "id": "",
        "remote_peers_addresses": "",
    },
    "peer_15": {
        "number": "15",
        "ip": "3.250.174.210",
        "web_server_port": "56790",
        "libp2p_port": "56789",
        "key_file": "peer_15_ire_aws_rsa_key.pem",
        "id": "",
        "remote_peers_addresses": "",
    },
    "peer_16": {
        "number": "16",
        "ip": "35.180.196.144",
        "web_server_port": "56790",
        "libp2p_port": "56789",
        "key_file": "peer_16_par_aws_rsa_key.pem",
        "id": "",
        "remote_peers_addresses": "",
    },
    "peer_1": {
        "number": "1",
        "ip": "18.140.244.236",
        "web_server_port": "56790",
        "libp2p_port": "56789",
        "key_file": "peer_1_sing_aws_rsa_key.pem",
        "id": "",
        "remote_peers_addresses": ""
    },
}

N = len(peers.keys())
F = 2
P = 0
T = 60
D = 3000
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

for peer in peers.values():
    set_protocol_cmd = f'scp -i ./keys/{peer["key_file"]} ./docker-compose.yml ubuntu@{peer["ip"]}:consensus_on_demand/docker-compose.yml'
    subprocess.run(set_protocol_cmd, shell=True)

print("\nReplicas parameters set")

processes = []
for peer in peers.values():
    start_replica_cmd = f'ssh -i ./keys/{peer["key_file"]} -t -q ubuntu@{peer["ip"]} "cd consensus_on_demand && docker compose up --build"'
    process = subprocess.Popen(start_replica_cmd, shell=True, stdout=subprocess.DEVNULL)
    processes.append(process)

print("\nReplicas started")

time.sleep(15)  # wait for docker containers to start

print("\nConnecting peers")

for peer in peers.values(): 
    response = requests.get("http://"+peer["ip"]+":"+peer["web_server_port"]+"/local_peer_id")
    if response.status_code == 200:
        peer["id"] = response.text[1:-1]
    else:
        print("Peer " + peer["number"] + " not reachable")

for i,peer in enumerate(peers.values()):
    remote_peers_addresses = ""
    for j,other_peer in enumerate(peers.values()):
        if i != j:
            remote_peers_addresses += "/ip4/"+other_peer["ip"]+"/tcp/"+other_peer["libp2p_port"]+"/p2p/"+other_peer["id"]+","
    peer["remote_peers_addresses"] = remote_peers_addresses[0:-1]

for peer in peers.values():
    requests.post("http://"+peer["ip"]+":"+peer["web_server_port"]+"/remote_peers_addresses", data=peer["remote_peers_addresses"])

print(f"\nProtocol started for {T} seconds")

for p in processes:
    p.communicate() # waits for replica to finish

print("\nReplicas stopped")

now = int(time.time())
subprocess.run(f'cd benchmark && mkdir {("FICC" if FICC else "ICC")}_{N}_{F}_{P}_{D}_{T}_{now}', shell=True, stdout=subprocess.DEVNULL)

for peer in peers.values():
    get_benchmark_results_cmd = f'scp -i ./keys/{peer["key_file"]} ubuntu@{peer["ip"]}:consensus_on_demand/benchmark/benchmark_results.json benchmark/{("FICC" if FICC else "ICC")}_{N}_{F}_{P}_{D}_{T}_{now}/benchmark_results_{peer["number"]}.json'
    subprocess.run(get_benchmark_results_cmd, shell=True, stdout=subprocess.DEVNULL)

print(f'\nResults written in folder benchmark/{("FICC" if FICC else "ICC")}_{N}_{F}_{P}_{D}_{T}_{now}')