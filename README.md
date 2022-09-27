# consensus_on_demand

Use mDNS to find local peers and floodsub protocol to broadcast a message.

Using different terminal windows, start four instances and pass each of them a different number in [1,4]. This will identify the node number of each peer. If your local network allows mDNS, they will automatically connect.

```sh
cargo run 1
```

Repeat the same command in thre other terminals, each time passing a different number (2, 3, 4) as  a parameter. Once all four instances have started (each diplays the addresses it is listening on) press Enter in the instance started with parameter 1.