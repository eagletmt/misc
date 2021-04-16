# agent-proto
Sample implementation of ssh-agent(1) style daemon/client with gRPC.

## Usage
```
% cargo build
% eval $(target/debug/agent)
% echo $AGENT_PROTO_SOCK
/tmp/agent-proto-gngcgl/agent.22569.sock
% echo $AGENT_PROTO_PID
22570
% target/debug/client
total = 1
% target/debug/client
total = 2
% target/debug/client
total = 3
% kill $AGENT_PROTO_PID
```
