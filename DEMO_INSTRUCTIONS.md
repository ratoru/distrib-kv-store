## Instructions for running the DEMO

1. First open a terminal and run the admin program to launch the 3 test clusters:

```
cargo run --bin admin
```

2. Next, run the client_repl bin file:

```
cargo run --bin client_repl
```

This uses the rust interfaces to write or read from the clusters including the client side carp logic.

Use this to do some writes and reads and note from which nodes these transactions are happening.

3. In another terminal, run demo_repl.sh from the main directory

```
./scripts/demo_repl.sh
```

This repl allows for direct reading and writing from individual nodes.

The interface is:

write (key) (value) (cluster_num) (node_num)

Note write will only work if you write to the leader of the cluster.

read (key) (cluster_num) (node_num)

Read can happen from any nodes in a cluster. So we can note which leader and cluster a write was sent to and then read from a follower of that leader and should get the right value.

Questions:

1. Should we also kill leaders then read from followers? This should work fine but then it might beg the question, can we write while leader is killed? And the code doesn't work for that.