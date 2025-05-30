# Messaging in a CIM
CIM strives for simplicity.
Big things are made from many little things.
Little things are managed easily, big things, not so much.
If all the little things can only do ONE thing, then we have a balance that allows big things to not care about the whole all the time and just let the little things take care of themselves.

So goes it with Messaging in a CIM.

Everything derives from Msg.

We only have 3 types of messages, Command, Query and Event.
They are all Messages.

This does not mean they "inherit" message, they ARE a Message.

$$
(\text{Command}\langle T \rangle \lor \text{Query}\langle T \rangle) \xrightarrow{\text{produces}} \text{EventStream}\langle T \rangle
$$

**Key Components:**
- `Command⟨T⟩`: Typed command with payload structure `T`
- `Query⟨T⟩`: Typed query with payload structure `T` 
- `EventStream⟨T⟩`: Typed sequence of events sharing schema `T`
- `∨`: Logical OR (either operation can initiate the stream)
- `→`: Production relationship with explicit labeling

**Why This Works:**
1. **Type Safety:** The `⟨T⟩` parameterization ensures payload consistency across commands, queries, and resulting events
2. **Stream Semantics:** Explicit `EventStream` notation aligns with event-driven design using NATS
3. **Ad-Hoc Agent Compatibility:** Generic typing allows agents to process any `T`-structured stream through the NATS-based messagee system
4. **Domain Alignment:** payload types are first-class citizens in the formula to capture intent and preserve structure

This formalization directly supports the CIM architecture's goals of deterministic event flows and context-aware AI agent communication.

**Plain English:**  
A Command or Query produces a stream of Events.

ALL Messages have a Subject and a Payload

Msg says Content is anything you can put in a Byte Array.
Cmd says Content is a Command. (Execute a state change)
Qry says Content is a Query. (Observe the current state)
Evt says Content is an Event. (Declare that state changed)

If you need to change state we make a Command that can.
If you want to observe anything, that is a Query, whether it be Memory, Direct SQL or Logs.
If anything changes, an Event is published.

Adding more will certainly change the tight abstration we are creating here.
We firmly believe this covers and limits everything nicely in our distributed system.

## What is a Msg?
A Message is a way to communicate. Period.
```
Msg
  Id
  Correlation
  Causation
  Owner
  Content
```

We want this to be extremely small, but universally useful.

That is it.  Content is the only thing that we are constraining in the Msg sub types.

If I want to create a Cmd, I cannot put arbitrary information into the Content, it MUST be a Command.  A Command is a generic trait that allows state to change.

If I want to create a Qry, the content MUST be a Query.
Query is a generic trait that includes things like CYPHER, PetGraph, SQL, GraphQL, and other query systems.

If I want to create an Evt, then the Event is checked for validity.

You will see how constraining our communications will create streams of separated information we can monitor and control in different ways.

We also make placing a Cmd into the Query system impossible without breaking rules and having the compiler complain.

The way we deal with Commands vs. Queries vs. Events are distinctly different, but we can still abstract them to a common pattern.

### Id
The Id is a universally unique identifier.

### Correlation
This is an Optional Unique ID that is used to identify a collection of related messages.

### Causation
This is an Optional Unique ID that is used to identify another message that caused this one to be produced.

### Owner
This is a Required Unique ID that is used to identify the Actor (Person or Agent) who created the message.

### Content
The actually content of the Msg.
Content is EITHER a CID or a Byte Array
If Content is > MAX_SIZE it is put into the ObjectStore and replaced with a CID.'
This keeps large messages from clogging the main channels and partitions data efficiently.

By constraining to these parameters, we have a valid, usable messaging pattern.

#### Why Uuids?
We need something universal that is not dependent on a central node to dole out sequential integers. There are other forms and you may certainly opt for another solution, but we were hard-pressed to find one and settled on Uuid.

There is another ID for every Msg in the system, it is the ContentID or CID. CID is an Interplanetary Link Definition (IPLD) a known process for calculating aggregated blocks of content as it is built. This provides a verification chain for the EventStore as each message is added, it's current state is also held as a CID.

Every Message is saved. This is an Event Sourcing System. Events persist into an Event Store. We also have the idea of a Command Store and a Query Store. This provides even more insight into how the system is used.

The idea of what a Command or Query are will be wholly dependent on the needs of the organization for which the CIM is built. While a Command is a generic, it is also fairly restrictive in what it can do. This is all controllable by the CIM.

I can easily create specialized Queries, such as sqlx, an async, pure Rust SQL crate featuring compile-time checked queries without a DSL. These all get checked at compile time and fit nicely into our system.

Correlation and Causation are going to clarify how messages get grouped together and Owner allows us to see who is doing what.

By storing a ContentID, each Event in the Event Store participates in a Directed Acyclic Graph which represents everything it has ever done.

Finally, NATS itself has a message structure that is handling quite a few things for us, such a the Sequence of the EventStore, the ObjectStore, Security, and various Key/Value stores.

This is all Distilled into a collection of Conceptual Spaces.
We already design our Domain with defined Collections of Categories. Conceptual Spaces are a culmination of Geometric Definition and Categorized Computer Science Structures.

Conceptual Spaces are composed of Vectorized Quality Dimensions describing Concepts in a Category. We use these to define cognitive relationship mapping. This mapping serves not only as a definition of human language use, but also as a cohesive bridge to AI and LLM concepts that enhance context for the AI as it communicates to a human.
