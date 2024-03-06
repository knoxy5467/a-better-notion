CREATE DATABASE IF NOT EXISTS 'abn';
USE 'abn';
CREATE TABLE IF NOT EXISTS 'task' (
    'id' SERIAL,
    'title' varchar(255) NOT NULL,
    'completed' boolean NOT NULL DEFAULT false,
    'last_changed' timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
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
) PARTITION BY LIST (type);
CREATE TABLE IF NOT EXISTS 'task_string_property_partition' PARTITION OF 'task_property' FOR
VALUES IN ('string');
CREATE TABLE IF NOT EXISTS 'task_num_property_partition' PARTITION OF 'task_property' FOR
VALUES IN ('real');
CREATE TABLE IF NOT EXISTS 'task_date_property_partition' PARTITION OF 'task_property' FOR
VALUES IN ('timestamp');
CREATE TABLE IF NOT EXISTS 'task_bool_property_partition' PARTITION OF 'task_property' FOR
VALUES IN ('boolean');
VALUES IN ('string');
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
CREATE OR REPLACE FUNCTION check_cycle() RETURNS TRIGGER AS $$
DECLARE cycle BOOLEAN;
BEGIN WITH RECURSIVE cte AS (
    SELECT NEW.task_id,
        NEW.depends_on_id
    UNION
    SELECT cte.task_id,
        d.depends_on_id
    FROM cte
        JOIN dependency d ON cte.depends_on_id = d.task_id
)
SELECT EXISTS (
        SELECT 1
        FROM cte
        WHERE cte.task_id = cte.depends_on_id
    ) INTO cycle;
IF cycle THEN RAISE EXCEPTION 'Dependency cycle detected';
END IF;
RETURN NEW;
END;
$$ LANGUAGE plpgsql;
CREATE TRIGGER dependency_insert_update_trigger BEFORE
INSERT
    OR
UPDATE ON dependency FOR EACH ROW EXECUTE FUNCTION check_cycle();
CREATE INDEX tasks_depend_on_index ON dependency (depends_on_id);
CREATE INDEX task_date_property_value_index ON task_date_property (value);
CREATE INDEX task_num_property_value_index ON task_num_property (value);
CREATE INDEX task_string_property_value_index ON task_string_property (value);
CREATE INDEX task_bool_property_value_index ON task_bool_property (value);
---CREATE INDEX task_property_type_index on task_property (jsonb_typeof(value));
CREATE MATERIALIZED VIEW task_properties AS
SELECT task.id AS task_id,
    task_property.name AS name,
    task_property.type AS type,
    task_string_property.value AS string_value,
    task_num_property.value AS num_value,
    task_date_property.value AS date_value,
    task_bool_property.value AS bool_value
FROM task
    LEFT JOIN task_string_property ON task.id = task_string_property.task_id
    LEFT JOIN task_num_property ON task.id = task_num_property.task_id
    LEFT JOIN task_date_property ON task.id = task_date_property.task_id
    LEFT JOIN task_bool_property ON task.id = task_bool_property.task_id;
-- check if a property already exists in the task_property table
CREATE OR REPLACE FUNCTION check_property(t varchar(255)) RETURNS TRIGGER AS $$ BEGIN IF EXISTS (
        SELECT 1
        FROM task_property
        WHERE id = NEW.task_id
            AND name = NEW.name
    ) THEN RAISE EXCEPTION 'Primary key already exists in task_property table';
ELSE
INSERT INTO task_property(id, name, type)
VALUES (NEW.task_id, NEW.name, t);
RETURN NEW;
END IF;
END;
$$ LANGUAGE plpgsql;
CREATE TRIGGER string_property_trigger BEFORE
INSERT ON task_string_property FOR EACH ROW EXECUTE FUNCTION check__property('string');
CREATE TRIGGER num_property_trigger BEFORE
INSERT ON task_num_property FOR EACH ROW EXECUTE FUNCTION check__property('real');
CREATE TRIGGER date_property_trigger BEFORE
INSERT ON task_date_property FOR EACH ROW EXECUTE FUNCTION check__property('timestamp');
CREATE TRIGGER bool_property_trigger BEFORE
INSERT ON task_bool_property FOR EACH ROW EXECUTE FUNCTION check__property('boolean');