# ABN: A Better Notion

Made in rust.

Make sure you have rust installed for your specific platform (see: https://rustup.rs/)

Run client:

```
cargo run -p client
```

Run server:

```
cargo run -p server
```

to read the documentation of a crate, run:

```
cargo rustdoc -p <crate_name> --open
```

currently we have 3 crates: `client`, `common`, and `server`.

# Contributing

Pull requests should have 100% coverage for tests and should be formatted and have no warnings from linting.

Run clippy and generate coverage reports for tests:

```
zsh pr-checks.sh
```

Please also format your rust code before submitting a PR:

```
cargo fmt
```

# database

to create the database use ` docker-compose up` in the server/database directory

this will create the server in a docker container. from there you can access it like a normal postgres database running on localhost:5432

docker-compose should only be ran on the first instance to create the database, afterwards it should be started through docker normally.

# Bug Tracking Instructions
Bugs and issues are tracked with github issues. To view the list current bugs & issues, navigate to the github issues tab above. Issue show status, assignees, and open/closed. Github issues do not support adding priority and timeline to issues, but the issue tracker has a dropdown menu which allows issues to be sorted by number of comments and date created.

To submit a new issue, do the following:

- Navigate to the issues tab and click "New Issue"
- For the title, summarize the issue in brief
- For the description, give a detailed explanation of the issue and link relevant code
- Set the status of the issue using the labels menu on the right. Optionally, assign the issue to a developer
- Submit the issue.
