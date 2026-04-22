# Actor Concurrency

Auto builds concurrent systems on the Actor model, making concurrent programming safe by design.

## Actor Model

Actors are independent units of computation that communicate exclusively through message passing. Each actor has:
- A private state that cannot be accessed directly from outside
- A mailbox for receiving messages
- The ability to spawn new actors

## Async Types

Auto introduces the `~T` type notation for asynchronous values. This makes it explicit in the type system when a value may be computed concurrently.

```auto
fn process(actor: ~Actor) -> ~Result {
    actor.send(message)
}
```

## Safety Guarantees

- No shared mutable state between actors
- Message passing is the only communication channel
- Data races are impossible by design
