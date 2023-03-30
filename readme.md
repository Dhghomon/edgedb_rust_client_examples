# EdgeDB minimum expression repo

This repo contains a minimal EdgeDB setup with a schema based on [the tutorial](https://www.edgedb.com/tutorial), and a few sample queries:

* Simple scalar queries
* Queries to return an Object and how to work with the Value enum
* Query returning json to then deserialize into a Rust struct
* Query using the Queryable derive macro, allowing deserializing into a Rust struct without needing intermediary json

First clone the repo, then:

* [Make sure you have EdgeDB installed](https://www.edgedb.com/install)
* Type `edgedb project init` and follow a few quick instructions. (Call the project `example` or whatever you like) You should have a running instance.
* (Optional if curious: type `edgedb instance list` to see it and then type `edgedb` if you want to play around with the REPL a bit. (You can also type `edgedb ui` if you want to work through the UI) The schema hasn't been applied yet, so leave the REPL with `\quit` and:)
* Type `edgedb migration create`. You should see a file called `00001.edgeql` show up in the `migrations` folder. Then type `edgedb migrate` to finish the migration.
* Then just type `cargo run` and see the output.