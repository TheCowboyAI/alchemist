We need to clarify use cases... NOT EVERY EVENT IS A DOMAIN EVENT... We have many things going on in Bevy that do not need to dynamically get sent to NATS. Animations for one... They only happen in Bevy nd transferring all those changes are just noise to the Domain... AFTER we have done our manipulations, we send what we have done to the Domain. We may apply 15 animations, move the model 100 times and add or remove subgraphs, nodes and edges to our hearts content in Bevy, then we save our changes to the Domain.

When we created our initial test for the Event Store, it was just that... a test, not something we are carrying into production. For production, we will have a very different initial state, probably an empty model.

For development, we want to create a K7 and Render it. Then instead of persiting it every time, we can acknowledge, this is a K7 and we already know about a K7, we just need to know what you want to DO to a K7... We will have many models we can then apply know morphisms to in rendering animations and force-directed layout that are visual appealing as well as visually recognizable.

ALL our graphs should have a known model they can be represented as... K#, C#, Mealy State Machine, Moore Machine, etc. Not through anything like Inheritance, a K7 is a K7 and there is only one way to represent a K7, the rest is just filling in the component. When we create unkown Graph Models, we should name the model in DDD Terms. Just like an Address Value Object will describe an Address Component with an Address Graph. The whole point is to be able to generate the Address Component from the Address Graph.

We must Document this intent as well as apply it to our current code model.

We have extremely rich Domain Models which are all structure preserving. We also have many structures and systems which may operate on those injected models.
