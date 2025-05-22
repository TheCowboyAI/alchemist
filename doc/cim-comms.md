# Comms
A NixOS utility module allowing you to communicate messages to a CIM using nats, without dealing with nats.

While the nats interface is robust, it is a lot of boilerplate we can abstract away to make working in a CIM much easier.

So I create create a Command or Query and then it would be really nice if I could just send it and not worry at all about the backend system and how it talks. I just get back an EventStream<T>.

Something along the lines of Command<T>.send().await?
That will await an acknowledgement that the Command was sent.
This is all very procedural and not quite what we want, we need a reactive system.
Command<T>.sub(&Self.channel) && Command<T>.pub(&Self.channel, T)

It doesn't wait for a response, anytime the subscription receives an Event in the EventStream, it reacts.

Of course, this a great workflow for Clients and exactly what the `comms` module does.

We abstract everything a CIM needs into CIM terms and you can stop worrying about how to make it happen.  Issue a Command or Request a Query and just get it fulfilled.

Commands and Queries are simply ways of changing the Domain State or Asking about the current Domain State.

Events are how this all happens.

>(CMD | QRY) --> [EVT]

>Execute, Observe, or Declare

Either a Command or Query is issued and a Stream of Events is produced.

DomainState is Nothing
Command changes DomainState AND issues Events describing the Change
EventStore -> Nothing -> CommandExecuted -> Current DomainState -> CommandExecuted -> Current DomainState
Query can always respond to current DomainState and watch for changes.
Commands are how we create and modify the system.

You can either Measure or Move, but not both at the same time.

Aggregates are a target of Command, meaning I am sending this Command to an Aggregate and it figures out all the Entities, Values,  Policies, and People that go along with this transaction.

This seems weird at first, but a CIM starts with nothing and tracks everything you do inside to create a sequence of immutable, replicatable Events that when added together in the sequence created, always obtain the current State of the CIM.  Think of this as an Integral in Calculus... We are adding up all the little changes in order to arrive at a current state. An integral serves as the continuous counterpart to a sum, utilized for calculation purposes. we start somewhere, add up all the little changes and arrive at a state which represents that value at a fixed point in time. When the SAME Inputs produce EXACTLY the SAME Outputs, then they are "pure" and we know this will ALWAYS calculate exactly the same way. 

The Event knows what caused it, either a Command or Query and it also knows what transaction it is contained within (the Aggregate). These are the causation and correlation properties in every message. Message also has a Sequence ID that is tied to it's comms server.  If this is a Leaf Node, it may be a local message promoted to the Domain... If it is Cluster or SuperCluster, it is a Domain message and published to the connected leafs. 

As a developer, we don't have to worry about this most of the time, comms handles it with Subject Based Messaging for us. We simply send and receive messages. It either works or it doesn't and when it doesn't we get an error we can look at.

Simple enough.

When dealing with a Messaging system we MUST be reactive... meaning I need to be able to respond to things coming in and alter behaviors based on those results. I can't ask a question and block the entire network until I get a response, so transactions work a little bit differently.

We don't want to tie everything to request/response behaviors because they become bottlenecks.  When we are reactive, we can just listen for things and react to them.

I didn't tell it to do that, it just happened... this is the sort of response we usually get. Actually we do tell it to do that, it's just kicked over to the background and we forget about it until it is needed.

## Subjects

Subjects are part of your critical infrastructure. They have a Taxonomy and Ontology.
We have built a system that operates on human language and Subjects are no different.
Subjects are a stored as a Hypergraph of the CIM. It represents the control flow of messages like plumbing, they are the router.

Subjects are the way we discover information.

We already have correlation and causation to help with this, but we need to have something between all Events and a collection of corresponding messages.

This is where Subjects come into play. Subjects let us group things into collections of messages that are interesting to certain responders.  For example if I listen for Commands, do I listen for ALL Commands and then filter that? Or should I listen for a filtered set of Commands... The answer as always is: it depends. I may just listen for all Commands, but that is a whole lot of traffic... I may only need Commands that are for a specific application, or a specific resource, and that is a Partitioned Subject.

How we organize these Subjects is a very important part of a CIM.
We work in Bounded Contexts and Domains as our top-level structures and the Subject Model should reflect this.

While nats makes this a simple `List<String>` joined by a `.`. We elevate these to taxonomies and ontologies that become a known part of the CIM's Domain.

In normal pub/sub, you can just make up a subject and the publisher and subscriber filter their communications through that subject... this is all fine and dandy until someone else wants to use that same Subject and now you both have unexpected noise.

We mitigate this by only allowing previously known subjects.
nats call this `subject transformation`

At the JetStream level you can define subject transforms as part of the stream configuration, which will apply to any message published to that stream, and you can also use subject transforms as part of the mirror, source configurations, and for message republishing. This is exactly what we need because messages and objects related to those messages move around the system, but we don't want to need to rely on some central firehose of Events.

This means we can tell the server only to allow certain subjects and we don't have to worry about collisions, noise or routing from other connected servers or someone trying to hack our system and use their own subjects (as strings).

We already have a good way to incorporate subjects based on CommandType and QueryType, so we have implemented this into our CIM so that relationships here become automated and we don't have to assign them, but we also know that they are qualified into the system for us.

What does comms do that nats doesn't?

comms handles shuffling around IDs and Subject Names automatically based on Domain Type... We control the Context of Messaging. This means that when you create an Event comms already knows this is going to be a Message and you will be sending it over nats. I don't need to know anything about nats or how it's configured, or where it is, I just send and it works... when it doesn't I have an immediate way to recover and make decisions.

We turn all these communications into State Machines... What State is the current Message in and the same for expected responses, etc.  Names are very specific. The more specific and related to other Names, the better.

Example:

Command<LoginCredentials>

Inuitively, what does this say?
It says a Command containing LoginCredentials
nice...
We can do better.
Command<LoginCredentialsForSage>
Now it says a Command containing LoginCredentials for Sage (an application)
This is better, because now deliver is basically automatic. Here is a Command that contains the credential necessary to login to Sage... Hmmm, I wonder where to send that... Sage of course. No Other Application needs to see that and my system knows how to intuitively make that happen.

This isn't the Only way to represent this:
Command<LoginToSage<Credentials>>

Here we represent this with nested generics, why???
I have 5 ways to login to Sage: JWT, NKey, User/Pass, Certificate and Token
If Credentials is a choice of those 5, then one thing can handle them all.
I now have a way to implement a whole bunch of wrappers basically.

I start with something, in this case a Credential, then I wrap it in an Application Specific Command, LoginToSage, then I wrap it again in a Message and it can control my Personal interaction with Sage from anywhere that has an internet connection and I don't have to care where it is.
