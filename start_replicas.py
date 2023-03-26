import requests

peers = {
    "peer_2": {
        "ip": "127.0.0.1",
        "web_server_port": "56790",
        "libp2p_port": "56789",
        "id": "",
        "remote_peers_addresses": ""
    },
    "peer_3": {
        "ip": "127.0.0.1",
        "web_server_port": "56796",
        "libp2p_port": "56795",
        "id": "",
        "remote_peers_addresses": ""
    },
    "peer_1": {
        "ip": "127.0.0.1",
        "web_server_port": "56781",
        "libp2p_port": "56780",
        "id": "",
        "remote_peers_addresses": ""
    },
}

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
    print(peer["remote_peers_addresses"])

for peer in peers.values():
    requests.post("http://"+peer["ip"]+":"+peer["web_server_port"]+"/remote_peers_addresses", data=peer["remote_peers_addresses"])