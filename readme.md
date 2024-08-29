# Alchemist

![The Alchemist](./alchemist.webp)
>"a person who transforms or creates something through a seemingly magical process."

This is an experiment for a User Interface and Projection system for a CIM.

The idea is that everything in the Information System is Identified.
It is composed of Entities (identifiable objects), Values (components), Behaviors (systems) and Events. 

These equate to our three base Models:
  - Applied Categories
  - Entity Component System (ECS)
  - Domain Driven Design (DDD)

## Mathematical Model
The Matematical Model is our definition of Mathematics and how we apply it.

### Applied Categories
These are actual Categories we define using Applied Category Theory.
Categories are Mathematical Objects which Model our Worlds.
There are known specifications for Categories using Category Theory.

## Observable Model
The Observable Model is how we observe the system, these are User Interfaces (UIs) and Applicatiopn Programming Interfaces (APIs).

### Entity Component System
We use an Entity Component System where:

#### Components
Components are Values
They are collections of data structures
No functionality is provided other than providing data

#### Entities
Identifiable Object with a Unique Identifier
Entities are composed from Components
An Entity is an Identified Collection of Values

#### Systems
Systems are behaviors and functionality which can be applied to Entities

## Domain Model
Domains are the boundaries we set on collections of ECS Worlds.

Domains define everything about a given collection of Values, Entities containing these values, and the systems that operate on them.

These are the definitions of meaning we apply to the ECS world.

[Initial Model](model.md)



