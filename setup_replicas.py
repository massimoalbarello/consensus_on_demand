import requests
import subprocess
import time
import os

peers = [
    {
        "number": "2",
        "ip": "44.212.45.29",
        "web_server_port": "56790",
        "libp2p_port": "56789",
        "key_file": "peer_2_nw_aws_rsa_key.pem",
        "id": "",
        "remote_peers_addresses": "",
    },
    {
        "number": "3",
        "ip": "54.233.213.217",
        "web_server_port": "56790",
        "libp2p_port": "56789",
        "key_file": "peer_3_sao_aws_rsa_key.pem",
        "id": "",
        "remote_peers_addresses": "",
    },
    {
        "number": "4",
        "ip": "3.27.137.153",
        "web_server_port": "56790",
        "libp2p_port": "56789",
        "key_file": "peer_4_syd_aws_rsa_key.pem",
        "id": "",
        "remote_peers_addresses": "",
    },
    {
        "number": "1",
        "ip": "13.214.145.252",
        "web_server_port": "56790",
        "libp2p_port": "56789",
        "key_file": "peer_1_sing_aws_rsa_key.pem",
        "id": "",
        "remote_peers_addresses": "",
    },
]

for peer in peers:
    print("\nInstalling docker for replica", peer["number"])
    os.chmod("./keys/"+peer["key_file"], 0o400)
    docker_installation_cmds = [
        "sudo apt-get update",
        "sudo apt-get install -y apt-transport-https ca-certificates curl gnupg lsb-release",
        "curl -fsSL https://download.docker.com/linux/ubuntu/gpg | sudo gpg --dearmor -o /usr/share/keyrings/docker-archive-keyring.gpg",
        'echo "deb [arch=amd64 signed-by=/usr/share/keyrings/docker-archive-keyring.gpg] https://download.docker.com/linux/ubuntu $(lsb_release -cs) stable" | sudo tee /etc/apt/sources.list.d/docker.list > /dev/null',
        "sudo apt-get update",
        "sudo apt-get install -y docker-ce docker-ce-cli containerd.io",
        "sudo usermod -aG docker $USER"
    ]
    for cmd in docker_installation_cmds:
        install_docker_cmd = f'ssh -i ./keys/{peer["key_file"]} -t -q ubuntu@{peer["ip"]} \'{cmd}\''
        process = subprocess.Popen(install_docker_cmd, shell=True)
        process.wait()

print("\nDocker installed on new replicas")

for peer in peers:
    print("\nCloning repo for replica", peer["number"])
    clone_repo_cmd = f'ssh -i ./keys/{peer["key_file"]} -t -q ubuntu@{peer["ip"]} \'git clone https://github.com/massimoalbarello/consensus_on_demand.git\''
    process = subprocess.Popen(clone_repo_cmd, shell=True)
    process.wait()

print("\nRepo cloned on new replicas")

processes = []
for peer in peers:
    print("\nBuilding container for replica", peer["number"])
    build_container_cmd = f'ssh -i ./keys/{peer["key_file"]} -t -q ubuntu@{peer["ip"]} \'cd consensus_on_demand && git checkout origin test-goodifier && docker compose build\''
    process = subprocess.Popen(build_container_cmd, shell=True)
    processes.append(process)

for p in processes:
    p.communicate() # waits for replica to finish

print("\nContainer built on new replicas")
