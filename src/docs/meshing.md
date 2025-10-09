# fixing the problem
> Problem: in a poorly configured mesh network, a relay could send the same message in a continuous loop because it would keep being relayed it's messages

# Rules
- Each node has **ONE** parent
- Each node can have an unlimited number of children
- Messages get a header of [origin] and [last-leap], [last-leap] comes from the last node that relayed the message to the current one
- When recieving a message, don't send it to the child matching [last-leap]
- every time a message is relayed, it **MUST** be sent to all children (except for [last-leap]) and then the parent ***in this specific order***

## Security
- all children **MUST** authenticate with a short lived JWT (~1 hour)
- the node has a blacklist for all revoked users during the token's lifetime

# Handling inboxes in this
- Each message gets a UUID
- When the client recieves a telegram message, it will tell it's inbox server to drop the one with the assosiated UUID because it's stored on the user's local machine already 

