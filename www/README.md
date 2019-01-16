# The parsley homepage

this directory contains the source code for the parsley webpage. It builds using
the local version of the library, and deploys automatically when the master
branch is built on Travis.

## As a testing tool

To spin it up, install [cargo-web](https://github.com/koute/cargo-web) with
`cargo install cargo-web`, then navigate to this directory and run
`cargo web start`. Since the library dependency is specified as a path, any
changes you make in the parent directory will be recompiled. Go to the default
address (`localhost:8000`) and refresh each time you make a change.
