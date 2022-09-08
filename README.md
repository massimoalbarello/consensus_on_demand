# consensus_on_demand

Use mDNS to find local peers and floodsub protocol to broadcast a message.

Using different terminal windows, start multiple instances. If your local network allows mDNS, they will automatically connect.
Type a message in either terminal and hit return: the message is sent and printed in the other terminals. Close with Ctrl-c.
Dialing any of the other peers will propagate the new participant to all members and everyone will receive all messages.

If the nodes don't automatically connect, take note of the listening addresses of one instance and start another with one of the addresses as the first argument. 
In a terminal window, run:
```sh
cargo run
```
It will print the PeerId and the listening addresses, e.g. `Listening on "/ip4/0.0.0.0/tcp/24915"`.

In another other terminal window, start a new instance with:
```sh
cargo run -- /ip4/127.0.0.1/tcp/24915
```