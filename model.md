# Model
A Model is a digital twin of something in the real world.
The "universe" is just that, everything, but a world is something we define within the global "universe".

- We have several notions of "models".
- Models are collected into worlds.
- Models can be shared or copied between worlds.
- Models interact with each other.
- Worlds can interact with each other.
- Models are reactive, they react to Events.

What is important to distinguish is that we want a clear separation between "data" and "functionality".

## Data
Data is the value of a thing.
Data has a structure. 
The structure is separate from the data itself.

## Functionality
This is any behavior or morphism applied to Data.
These are separated because we may apply the same functionality across different data structures.
Functionality is either a Command, or a Query.
  - A Query will OBSERVE State
    - Observed state is an Event
  - A Command will CHANGE State
    - Change to State is an Event

# Interface
Our Interface is a set of Values, composed into Entities, that emit Events when Functionality is applied.

These Events are messages that pass between Entites and may identify State Changes or Queries.

We obtain Events through Subscriptions
We Emit Events through either Requests or through Publishing

Events may be Local, Leaf, or Domain level. This affects where they are published and who may subscribe.

Local Events are the Events in the running Interface, usually a screen or a stream.
Leaf Events apply to the running World and are a stream.
Domain Events apply to the running Domain and are a stream.

There is also a notion of an Operator Level with Tenants, but we consider this a "pull-back" or something derived from a collection of Domains, which becomes a Domain itself.

