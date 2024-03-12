CREATE DATABASE IF NOT EXISTS 'abn';

USE 'abn';

CREATE SCHEMA IF NOT EXISTS task;

SET
    search_path TO task;

CREATE TABLE IF NOT EXISTS 'task' (
    'id' SERIAL,
    'title' varchar(255) NOT NULL,
    'completed' boolean NOT NULL DEFAULT false,
    'last_edited' timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY ('id')
);

CREATE TABLE IF NOT EXISTS 'dependency'(
    'task_id' INT NOT NULL,
    'depends_on_id' INT NOT NULL,
    FOREIGN KEY ('task_id') REFERENCES 'task'('id') ON DELETE CASCADE,
    FOREIGN KEY ('depends_on_id') REFERENCES 'task'('id') ON DELETE CASCADE,
    PRIMARY KEY ('task_id', 'depends_on_id')
);

CREATE TABLE IF NOT EXISTS 'task_property' (
    'task_id' INT NOT NULL,
    'name' varchar(255) NOT NULL,
    'type' TEXT NOT NULL,
    FOREIGN KEY ('task_id') REFERENCES 'task'('id') ON DELETE CASCADE,
    PRIMARY KEY ('task_id', 'name')
) PARTITION BY LIST (TYPE);

CREATE TABLE IF NOT EXISTS 'task_string_property_partition' PARTITION OF 'task_property' FOR
VALUES
    IN ('string');

CREATE TABLE IF NOT EXISTS 'task_num_property_partition' PARTITION OF 'task_property' FOR
VALUES
    IN ('real');

CREATE TABLE IF NOT EXISTS 'task_date_property_partition' PARTITION OF 'task_property' FOR
VALUES
    IN ('timestamp');

CREATE TABLE IF NOT EXISTS 'task_bool_property_partition' PARTITION OF 'task_property' FOR
VALUES
    IN ('boolean');

VALUES
    IN ('string');

CREATE TABLE IF NOT EXISTS 'task_string_property' (
    'task_id' INT NOT NULL REFERENCES 'task'('id') ON DELETE CASCADE,
    'task_property_name' varchar(255) NOT NULL REFERENCES 'task_property'('name'),
    'value' TEXT NOT NULL,
    PRIMARY KEY ('task_id', 'task_property_name')
);

CREATE TABLE IF NOT EXISTS 'task_num_property' (
    'task_id' INT NOT NULL REFERENCES 'task'('id') ON DELETE CASCADE,
    'task_property_name' varchar(255) NOT NULL REFERENCES 'task_property'('name'),
    'value' REAL NOT NULL,
    PRIMARY KEY ('task_id', 'task_property_name')
);

CREATE TABLE IF NOT EXISTS 'task_date_property' (
    'task_id' INT NOT NULL REFERENCES 'task'('id') ON DELETE CASCADE,
    'task_property_name' varchar(255) NOT NULL REFERENCES 'task_property'('name'),
    'value' timestamp NOT NULL,
    PRIMARY KEY ('task_id', 'task_property_name')
);

CREATE TABLE IF NOT EXISTS 'task_bool_property' (
    'task_id' INT NOT NULL REFERENCES 'task'('id') ON DELETE CASCADE,
    'task_property_name' varchar(255) NOT NULL REFERENCES 'task_property'('name'),
    'value' boolean NOT NULL,
    PRIMARY KEY ('task_id', 'task_property_name')
);

CREATE TABLE IF NOT EXISTS 'scripts' (
    'id' SERIAL,
    'name' varchar(255) NOT NULL,
    'code' text NOT NULL,
    PRIMARY KEY ('id')
);

CREATE TABLE IF NOT EXISTS 'task_scripts' (
    'task_id' INT NOT NULL REFERENCES 'task'('id') ON DELETE CASCADE,
    'script_id' INT NOT NULL REFERENCES 'scripts'('id') ON DELETE CASCADE,
    'event' varchar(255) NOT NULL,
    PRIMARY KEY ('task_id ', 'script_id')
);

CREATE TABLE IF NOT EXISTS 'global_property' (
    'name' varchar(255) NOT NULL,
    'type' TEXT NOT NULL,
    PRIMARY KEY ('name')
);

CREATE TABLE IF NOT EXISTS 'global_string_property' (
    'property_name' varchar(255) NOT NULL REFERENCES 'global_property'('name') ON DELETE CASCADE,
    'value' TEXT NOT NULL,
    PRIMARY KEY ('property_name')
);

CREATE TABLE IF NOT EXISTS 'global_num_property' (
    'property_name' varchar(255) NOT NULL REFERENCES 'global_property'('name') ON DELETE CASCADE,
    'value' REAL NOT NULL,
    PRIMARY KEY ('property_name')
);

CREATE TABLE IF NOT EXISTS 'global_date_property' (
    'property_name' varchar(255) NOT NULL REFERENCES 'global_property'('name') ON DELETE CASCADE,
    'value' timestamp NOT NULL,
    PRIMARY KEY ('property_name')
);

CREATE TABLE IF NOT EXISTS 'global_bool_property' (
    'property_name' varchar(255) NOT NULL REFERENCES 'global_property'('name') ON DELETE CASCADE,
    'value' boolean NOT NULL,
    PRIMARY KEY ('property_name')
);

CREATE
OR REPLACE FUNCTION check_cycle() RETURNS TRIGGER AS $ $ DECLARE cycle BOOLEAN;

BEGIN WITH RECURSIVE cte AS (
    SELECT
        NEW.task_id,
        NEW.depends_on_id
    UNION
    SELECT
        cte.task_id,
        d.depends_on_id
    FROM
        cte
        JOIN dependency d ON cte.depends_on_id = d.task_id
)
SELECT
    EXISTS (
        SELECT
            1
        FROM
            cte
        WHERE
            cte.task_id = cte.depends_on_id
    ) INTO cycle;

IF cycle THEN RAISE EXCEPTION 'Dependency cycle detected';

END IF;

RETURN NEW;

END;

$ $ LANGUAGE plpgsql;

CREATE TRIGGER dependency_insert_update_trigger BEFORE
INSERT
    OR
UPDATE
    ON dependency FOR EACH ROW EXECUTE FUNCTION check_cycle();

CREATE INDEX task_depend_on_index ON dependency (depends_on_id);

CREATE INDEX task_date_property_value_index ON task_date_property (value);

CREATE INDEX task_num_property_value_index ON task_num_property (value);

CREATE INDEX task_string_property_value_index ON task_string_property (value);

CREATE INDEX task_bool_property_value_index ON task_bool_property (value);

---CREATE INDEX task_property_type_index on task_property (jsonb_typeof(value));
CREATE MATERIALIZED VIEW task_properties AS
SELECT
    task.id AS task_id,
    task_property.name AS name,
    task_property.type AS TYPE,
    task_string_property.value AS string_value,
    task_num_property.value AS num_value,
    task_date_property.value AS date_value,
    task_bool_property.value AS bool_value
FROM
    task
    LEFT JOIN task_string_property ON task.id = task_string_property.task_id
    LEFT JOIN task_num_property ON task.id = task_num_property.task_id
    LEFT JOIN task_date_property ON task.id = task_date_property.task_id
    LEFT JOIN task_bool_property ON task.id = task_bool_property.task_id;

-- check if a property already exists in the task_property table
CREATE
OR REPLACE FUNCTION check_property(t varchar(255)) RETURNS TRIGGER AS $ $ BEGIN IF EXISTS (
    SELECT
        1
    FROM
        task_property
    WHERE
        id = NEW.task_id
        AND name = NEW.name
) THEN RAISE EXCEPTION 'Primary key already exists in task_property table';

ELSE
INSERT INTO
    task_property(id, name, TYPE)
VALUES
    (NEW.task_id, NEW.name, t);

RETURN NEW;

END IF;

END;

$ $ LANGUAGE plpgsql;

CREATE TRIGGER string_property_trigger BEFORE
INSERT
    ON task_string_property FOR EACH ROW EXECUTE FUNCTION check__property('string');

CREATE TRIGGER num_property_trigger BEFORE
INSERT
    ON task_num_property FOR EACH ROW EXECUTE FUNCTION check__property('real');

CREATE TRIGGER date_property_trigger BEFORE
INSERT
    ON task_date_property FOR EACH ROW EXECUTE FUNCTION check__property('timestamp');

CREATE TRIGGER bool_property_trigger BEFORE
INSERT
    ON task_bool_property FOR EACH ROW EXECUTE FUNCTION check__property('boolean');

CREATE
OR REPLACE FUNCTION update_last_edited() RETURNS TRIGGER AS $ $ BEGIN
UPDATE
    task
SET
    last_edited = NOW()
WHERE
    id = NEW.task_id;

RETURN NEW;

END;

$ $ LANGUAGE plpgsql;

CREATE
OR REPLACE FUNCTION create_update_last_edited_triggers() RETURNS VOID AS $ $ DECLARE table_name text;

BEGIN FOR table_name IN
SELECT
    table_name
FROM
    information_schema.tables
WHERE
    table_schema = 'task'
    AND column_name = 'task_id' LOOP EXECUTE format(
        'CREATE TRIGGER update_last_edited_trigger_%I
                        AFTER INSERT OR DELETE OR UPDATE ON %I
                        FOR EACH ROW EXECUTE FUNCTION update_last_edited();',
        table_name,
        table_name
    );

END LOOP;

END $ $ LANGUAGE plpgsql;

SELECT
    create_update_last_edited_triggers();

-- Generate the PlantUML code for the database schema
SELECT
    '@startuml\n\n' || '-- Create the database\n' || 'database "abn" as abn\n\n' || '-- Create the task schema\n' || 'package "task" {\n\n' || '-- Create the task table\n' || 'table "task" as task {\n' || '    +id: SERIAL (PK)\n' || '    title: varchar(255) NOT NULL\n' || '    completed: boolean NOT NULL DEFAULT false\n' || '    last_edited: timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP\n' || '}\n\n' || '-- Create the dependency table\n' || 'table "dependency" as dependency {\n' || '    +task_id: INT (FK: task.id)\n' || '    +depends_on_id: INT (FK: task.id)\n' || '}\n\n' || '-- Create the task_property table\n' || 'table "task_property" as task_property {\n' || '    +task_id: INT (FK: task.id)\n' || '    +name: varchar(255) NOT NULL\n' || '    +type: TEXT NOT NULL\n' || '}\n\n' || '-- Create the task_string_property table\n' || 'table "task_string_property" as task_string_property {\n' || '    +task_id: INT (FK: task.id)\n' || '    +task_property_name: varchar(255) (FK: task_property.name)\n' || '    +value: TEXT NOT NULL\n' || '}\n\n' || '-- Create the task_num_property table\n' || 'table "task_num_property" as task_num_property {\n' || '    +task_id: INT (FK: task.id)\n' || '    +task_property_name: varchar(255) (FK: task_property.name)\n' || '    +value: REAL NOT NULL\n' || '}\n\n' || '-- Create the task_date_property table\n' || 'table "task_date_property" as task_date_property {\n' || '    +task_id: INT (FK: task.id)\n' || '    +task_property_name: varchar(255) (FK: task_property.name)\n' || '    +value: timestamp NOT NULL\n' || '}\n\n' || '-- Create the task_bool_property table\n' || 'table "task_bool_property" as task_bool_property {\n' || '    +task_id: INT (FK: task.id)\n' || '    +task_property_name: varchar(255) (FK: task_property.name)\n' || '    +value: boolean NOT NULL\n' || '}\n\n' || '-- Create the scripts table\n' || 'table "scripts" as scripts {\n' || '    +id: SERIAL (PK)\n' || '    name: varchar(255) NOT NULL\n' || '    code: text NOT NULL\n' || '}\n\n' || '-- Create the task_scripts table\n' || 'table "task_scripts" as task_scripts {\n' || '    +task_id: INT (FK: task.id)\n' || '    +script_id: INT (FK: scripts.id)\n' || '    event: varchar(255) NOT NULL\n' || '}\n\n' || '-- Create the global_property table\n' || 'table "global_property" as global_property {\n' || '    +name: varchar(255) NOT NULL\n' || '    +type: TEXT NOT NULL\n' || '}\n\n' || '-- Create the global_string_property table\n' || 'table "global_string_property" as global_string_property {\n' || '    +property_name: varchar(255) (FK: global_property.name)\n' || '    +value: TEXT NOT NULL\n' || '}\n\n' || '-- Create the global_num_property table\n' || 'table "global_num_property" as global_num_property {\n' || '    +property_name: varchar(255) (FK: global_property.name)\n' || '    +value: REAL NOT NULL\n' || '}\n\n' || '-- Create the global_date_property table\n' || 'table "global_date_property" as global_date_property {\n' || '    +property_name: varchar(255) (FK: global_property.name)\n' || '    +value: timestamp NOT NULL\n' || '}\n\n' || '-- Create the global_bool_property table\n' || 'table "global_bool_property" as global_bool_property {\n' || '    +property_name: varchar(255) (FK: global_property.name)\n' || '    +value: boolean NOT NULL\n' || '}\n\n' || '}\n\n' || '-- Create the check_cycle trigger\n' || 'trigger "check_cycle" as check_cycle {\n' || '    -- Trigger code goes here\n' || '}\n\n' || '-- Create the dependency_insert_update_trigger trigger\n' || 'trigger "dependency_insert_update_trigger" as dependency_insert_update_trigger {\n' || '    -- Trigger code goes here\n' || '}\n\n' || '-- Create the task_depend_on_index index\n' || 'index "task_depend_on_index" as task_depend_on_index on dependency (depends_on_id)\n\n' || '-- Create the task_date_property_value_index index\n' || 'index "task_date_property_value_index" as task_date_property_value_index on task_date_property (value)\n\n' || '-- Create the task_num_property_value_index index\n' || 'index "task_num_property_value_index" as task_num_property_value_index on task_num_property (value)\n\n' || '-- Create the task_string_property_value_index index\n' || 'index "task_string_property_value_index" as task_string_property_value_index on task_string_property (value)\n\n' || '-- Create the task_bool_property_value_index index\n' || 'index "task_bool_property_value_index" as task_bool_property_value_index on task_bool_property (value)\n\n' || '-- Create the task_properties materialized view\n' || 'materialized view "task_properties" as task_properties {\n' || '    -- View code goes here\n' || '}\n\n' || '-- Create the check_property trigger\n' || 'trigger "check_property" as check_property {\n' || '    -- Trigger code goes here\n' || '}\n\n' || '-- Create the string_property_trigger trigger\n' || 'trigger "string_property_trigger" as string_property_trigger {\n' || '    -- Trigger code goes here\n' || '}\n\n' || '-- Create the num_property_trigger trigger\n' || 'trigger "num_property_trigger" as num_property_trigger {\n' || '    -- Trigger code goes here\n' || '}\n\n' || '-- Create the date_property_trigger trigger\n' || 'trigger "date_property_trigger" as date_property_trigger {\n' || '    -- Trigger code goes here\n' || '}\n\n' || '-- Create the bool_property_trigger trigger\n' || 'trigger "bool_property_trigger" as bool_property_trigger {\n' || '    -- Trigger code goes here\n' || '}\n\n' || '-- Create the update_last_edited trigger\n' || 'trigger "update_last_edited" as update_last_edited {\n' || '    -- Trigger code goes here\n' || '}\n\n' || '-- Create the create_update_last_edited_triggers function\n' || 'function "create_update_last_edited_triggers" as create_update_last_edited_triggers {\n' || '    -- Function code goes here\n' || '}\n\n' || '-- Call the create_update_last_edited_triggers function\n' || 'call create_update_last_edited_triggers()\n\n' || '@enduml' INTO OUTFILE '/path/to/output.puml';

CREATE schema IF NOT EXISTS user;

SET
    search_path TO user;

CREATE TABLE IF NOT EXISTS user (
    id SERIAL,
    username varchar(255) NOT NULL UNIQUE,
    PASSWORD TEXT NOT NULL,
    email TEXT NOT NULL,
    PRIMARY KEY (id)
);

CREATE TABLE IF NOT EXISTS VIEW (
    id SERIAL,
    properties TEXT [] NOT NULL,
    filter JASONB NOT NULL,
    PRIMARY KEY (id)
);

CREATE TABLE IF NOT EXISTS user_view (
    user_id INT NOT NULL REFERENCES user(id) ON DELETE CASCADE,
    view_id INT NOT NULL REFERENCES VIEW(id) ON DELETE CASCADE,
    PRIMARY KEY (user_id, view_id)
);

CREATE TABLE IF NOT EXISTS organization (
    id SERIAL,
    name TEXT NOT NULL UNIQUE,
    PRIMARY KEY (id)
);

CREATE TABLE IF NOT EXISTS user_organization (
    user_id INT NOT NULL REFERENCES user(id) ON DELETE CASCADE,
    organization_id INT NOT NULL REFERENCES organization(id) ON DELETE CASCADE,
    role TEXT NOT NULL,
    PRIMARY KEY (user_id, organization_id)
);

CREATE TABLE IF NOT EXISTS user_task (
    user_id INT NOT NULL REFERENCES user(id) ON DELETE CASCADE,
    task_id INT NOT NULL REFERENCES task.task(id) ON DELETE CASCADE,
    PRIMARY KEY (user_id, task_id)
);