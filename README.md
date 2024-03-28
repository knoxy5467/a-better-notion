# ABN: A Better Notion

Made in rust.

Make sure you have rust installed for your specific platform (see: https://rustup.rs/)

Run database:
```
cd server/database
docker-compose up
```
This requires `docker`/`podman` and `docker-compose`/`podman-compose`. It will initialize a postgres database running on `localhost:5432`. (make sure to run in a dedicated terminal!)

`docker-compose` should only be run on the first instance to create the database, afterwards it should be started through docker normally.

Run server:

```
cargo run -p server
```

Run client:

```
cargo run -p client
```

### Docs

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

# Bug Tracking Instructions

Bugs and issues are tracked with github issues. To view the list current bugs & issues, navigate to the github issues tab above. Issue show status, assignees, and open/closed. Github issues do not support adding priority and timeline to issues, but the issue tracker has a dropdown menu which allows issues to be sorted by number of comments and date created.

To submit a new issue, do the following:

Navigate to the issues tab and click "New Issue"
For the title, summarize the issue in brief
For the description, give a detailed explanation of the issue and link relevant code
Set the status of the issue using the labels menu on the right. Optionally, assign the issue to a developer
Submit the issue.

# Querying Database
Once you have the database running in either Docker or on your local intallation you can interact with it in 1 of 2 ways.

## psql (the standard postgres interface)
In order to run normal Postgresql queries you will need to install postgres on your computer.

when postgres is installed on your computer you can connect to the database with `psql "postgres://abn:abn@localhost:5432/abn?options=-c%20search_path%3Dtask"`

an example query to create a task would be 
`INSERT INTO task (completed, title)
VALUES (false, "my task title");`

and to retrieve value from this table you would use `SELECT * FROM task;`


