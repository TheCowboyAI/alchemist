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

# Parts
We are going to call these "parts" for now. "parts" are all the different things in our Domain.
I mean the structures themselves. Many of these are terms cross associated with Domain Driven Design, ECS and FRP.

## Values
Values are either 
  ### Primitives: (a.k.a. Rust Primitives)
  -  Single Values of a specified Type:
      - Boolean
      - Integer
      - Float
      - String

  ### Components: (a.k.a. Value Objects)
  - Sets of Invariant Values:
    - Shipping Address
    - GeoLocation
    - Telephone Number
    - Invoice
    - Line Item
    - Network Address
    - Country
    - Product
    - Person Name
    - Organization Name
    - Email Address
  
These are specific Values that carry special meaning. If any part of the Value Change, the Whole Value Changes at the Entity Level.

## Entities
Entites are Sets of Components with an ID.

It is that simple, you are collecting a set of Values and naming it.

  - Sets of Components:
    - Person
      - PersonUid
      - Person Name
      - Birthdate
      - Sex
      - Gender
      - Home Address
      - Cellphone

Of course this can be any set of Components required... 
Employee would be a completely different Entity:

  - Employee
    - EmployeeUid
    - Person
    - VOIP Number
    - Supervisor
    - IT Account

Notice we point to Person, that link automatically dictates that we have a Link between EmployeeUid and PersonUid somewhere, but we do not need to Specify it because it Uid is a required Component of any Entity and the name of the field doesn't matter, it is the responsibility of the Component to give it to me.

You may be use to OOP and Inheritance, this works in a similar way, but it operates more like a Dependent Type for us.
If the underlying Type of Uid changes from say UUIDv4 to UUIDv7, we don't care, we just ask Employee for it's ID and it gives us the right one.

This is also radically different from Relational Database that would use EmployeedUid as a ForiegnKey.
The problem with this approach is that it is Type Bound for one. If you change UUidv4 to v7 you have to repopulate your database.

It also start making some relationships really difficult, such as Many to Many relationships.

We have completely avoided this dilemna by using an Event Sourced change system.

This includes in-line updates to data structures and multiple pathways that may depend on versioned type structures.

Entities have the following abstracted structure:
  Name
  ID
  [Collection of Values]

Where ID is identified as some Type
and Values are a Collection of Components

# Systems
Systems are any behavior or functionality that operates on Components.

Systems usually operate by Query, such as Entities with Addresses or Employees with Supervisor equal to John.

This allows us to operate on Components no matter what Entity they are attached to in an abstract way, such as "Turn all Borders for Persons with Favorite Color Yellow to Yellow."

If some Component is attached to them somewhere, such as PersonProfile, then this will get matched by the Query.

# Events
Signal a Change in State
Observe State at a specified sequence

Domain Events are stored in a sequential Event Store.
This may be a partitioned set of stores.
Stores are recommended to be configured for auto-replication
Stores are recomended to be a Clustered Quorum 

# Observers
Observe the current state

These are Components that are the target of Events
When Events are received, they are projected to an Observer that may be read like a Resource.

# Signals
aka Triggers...
aka EventHandlers...

