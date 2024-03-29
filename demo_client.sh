psql "postgres://abn:abn@localhost:5432/abn?options=-c%20search_path%3Dtask" -c "DELETE FROM task;"
psql "postgres://abn:abn@localhost:5432/abn?options=-c%20search_path%3Dtask" -c "INSERT INTO task (completed, title, id) VALUES (true, 'give ABN an A for their Alpha Release!', 1);"
psql "postgres://abn:abn@localhost:5432/abn?options=-c%20search_path%3Dtask" -c "INSERT INTO task (completed, title, id) VALUES (false, 'make dinner', 2);"
cargo run -p client