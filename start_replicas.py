import requests
import subprocess
import time
import os

peers = {
    "peer_2": {
        "number": "2",
        "ip": "3.83.120.112",
        "web_server_port": "56790",
        "libp2p_port": "56789",
        "key_file": "peer_2_nw_aws_rsa_key.pem",
        "id": "",
        "remote_peers_addresses": "",
    },
    "peer_3": {
        "number": "3",
        "ip": "18.231.76.120",
        "web_server_port": "56790",
        "libp2p_port": "56789",
        "key_file": "peer_3_sao_aws_rsa_key.pem",
        "id": "",
        "remote_peers_addresses": ""
    },
    "peer_4": {
        "number": "4",
        "ip": "3.106.137.254",
        "web_server_port": "56790",
        "libp2p_port": "56789",
        "key_file": "peer_4_syd_aws_rsa_key.pem",
        "id": "",
        "remote_peers_addresses": ""
    },
    "peer_5": {
        "number": "5",
        "ip": "3.122.59.55",
        "web_server_port": "56790",
        "libp2p_port": "56789",
        "key_file": "peer_5_frank_aws_rsa_key.pem",
        "id": "",
        "remote_peers_addresses": ""
    },
    "peer_6": {
        "number": "6",
        "ip": "13.53.131.223",
        "web_server_port": "56790",
        "libp2p_port": "56789",
        "key_file": "peer_6_stock_aws_rsa_key.pem",
        "id": "",
        "remote_peers_addresses": ""
    },
    "peer_7": {
        "number": "7",
        "ip": "54.67.51.84",
        "web_server_port": "56790",
        "libp2p_port": "56789",
        "key_file": "peer_7_cali_aws_rsa_key.pem",
        "id": "",
        "remote_peers_addresses": ""
    },
    "peer_8": {
        "number": "8",
        "ip": "3.144.187.147",
        "web_server_port": "56790",
        "libp2p_port": "56789",
        "key_file": "peer_8_ohio_aws_rsa_key.pem",
        "id": "",
        "remote_peers_addresses": "",
    },
    "peer_9": {
        "number": "9",
        "ip": "35.92.16.162",
        "web_server_port": "56790",
        "libp2p_port": "56789",
        "key_file": "peer_9_oreg_aws_rsa_key.pem",
        "id": "",
        "remote_peers_addresses": "",
    },
    "peer_10": {
        "number": "10",
        "ip": "3.110.122.184",
        "web_server_port": "56790",
        "libp2p_port": "56789",
        "key_file": "peer_10_mum_aws_rsa_key.pem",
        "id": "",
        "remote_peers_addresses": "",
    },
    "peer_11": {
        "number": "11",
        "ip": "13.208.190.65",
        "web_server_port": "56790",
        "libp2p_port": "56789",
        "key_file": "peer_11_osaka_aws_rsa_key.pem",
        "id": "",
        "remote_peers_addresses": "",
    },
    "peer_12": {
        "number": "12",
        "ip": "3.36.113.9",
        "web_server_port": "56790",
        "libp2p_port": "56789",
        "key_file": "peer_12_seo_aws_rsa_key.pem",
        "id": "",
        "remote_peers_addresses": "",
    },
    "peer_13": {
        "number": "13",
        "ip": "13.114.250.88",
        "web_server_port": "56790",
        "libp2p_port": "56789",
        "key_file": "peer_13_tok_aws_rsa_key.pem",
        "id": "",
        "remote_peers_addresses": "",
    },
    "peer_14": {
        "number": "14",
        "ip": "3.99.214.250",
        "web_server_port": "56790",
        "libp2p_port": "56789",
        "key_file": "peer_14_can_aws_rsa_key.pem",
        "id": "",
        "remote_peers_addresses": "",
    },
    "peer_15": {
        "number": "15",
        "ip": "54.220.184.107",
        "web_server_port": "56790",
        "libp2p_port": "56789",
        "key_file": "peer_15_ire_aws_rsa_key.pem",
        "id": "",
        "remote_peers_addresses": "",
    },
    "peer_16": {
        "number": "16",
        "ip": "13.37.57.131",
        "web_server_port": "56790",
        "libp2p_port": "56789",
        "key_file": "peer_16_par_aws_rsa_key.pem",
        "id": "",
        "remote_peers_addresses": "",
    },
    "peer_1": {
        "number": "1",
        "ip": "13.214.156.3",
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
subprocess.run(f'cd benchmark && mkdir res_{N}_{F}_{P}_{D}_{T}_{now}', shell=True, stdout=subprocess.DEVNULL)

for peer in peers.values():
    get_benchmark_results_cmd = f'scp -i ./keys/{peer["key_file"]} ubuntu@{peer["ip"]}:consensus_on_demand/benchmark/benchmark_results.json benchmark/res_{N}_{F}_{P}_{D}_{T}_{now}/benchmark_results_{peer["number"]}.json'
    subprocess.run(get_benchmark_results_cmd, shell=True, stdout=subprocess.DEVNULL)

print(f"\nResults written in folder benchmark/res_{N}_{F}_{P}_{D}_{T}_{now}")