# Architecture

[source](https://rust-stack.com/articles/featured/20260623-clean-architecture-rust-guide-to-be-project-structure/)

- src/
  - main.rs
  - lib.rs
  - domain/
    - mod.rs
    - error.rs
    - data_models
      - mod.rs
      - ds1.rs
      - ds2.rs
    - ports/
      - mod.rs
      - port1.rs
      - port2.rs
  - application/
    - mod.rs
    - error.rs (not needed. Use cases will be done with anyhow)
    - uc1.rs
    - uc2.rs
  - infrastructure/
    - mod.rs
    - error.rs
    - docker/
      - mod.rs
    - git/
      - mod.rs

## Layers

### domain

Contains data structures and the rules that govern them. Does not contain external dependencies, drivers or frameworks (exception is thiserror for Domain Errors).

### application

Contains use cases, orchastrates flow of data to and from the domain entities. With ports we define how we interact with the outside world using traits.

### infrastructure

Defines interactions with external sources. Implements traits defined in the application layer, wire up web server, docker ...