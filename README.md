# Redox
Redox is a toy redis implementation in Rust.

## Features
Largely incomplete (still a toy so far), but some cool stuff
### Partial RESP Parsing
Redox implements Partial RESP parsing, which as I understand is not the most common way of implementing the parser. I am not sure if the official implementations use partial parsing. This allows for a fixed size buffer to be consumed and parsed into a partial state, so that the buffer can now be cleared and filled again to a fixed size, rather than throwing away all the parsing work, growing the buffer, and attempting to parse again. This probably has some benefits if you are trying to store humungous stuff into redis, (not benchmarked, could be wrong), but probably has a negligible impact on day to day perfomance. (Just fill out the buffer completely).

This was my first combinator parser though, and I thought partial parsing would be cool to attempt, which is really the only reason I built it. Rust's type system is incredibly expressive and helps reason about the parsing very clearly.


### Other features
No real other notable features. Only supported actions right now are SET, GET, ECHO, PING, and also expiry for the SET/GET.
I plan to complete all stages, so will eventually add support for replication, persistence, and streams.

# Codecrafters Progress
(Codecrafters is pretty cool btw)
[![progress-banner](https://backend.codecrafters.io/progress/redis/d94ebcc3-a895-456f-8b97-c9ff31c6bf74)](https://app.codecrafters.io/users/codecrafters-bot?r=2qF)