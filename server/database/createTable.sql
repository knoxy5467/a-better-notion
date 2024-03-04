CREATE DATABASE IF NOT EXISTS 'abn';
USE 'abn';
CREATE TABLE IF NOT EXISTS 'task' (
    'id' SERIAL,
    'title' varchar(255) NOT NULL,
    'completed' boolean NOT NULL DEFAULT false,
    PRIMARY KEY ('id')
);
CREATE TABLE IF NOT EXISTS 'dependency'(
    'task_id' INT NOT NULL,
    'depends_on_id' INT NOT NULL,
    FOREIGN KEY ('task_id') REFERENCES 'task'('id'),
    FOREIGN KEY ('depends_on_id') REFERENCES 'task'('id'),
    PRIMARY KEY ('task_id', 'depends_on_id')
);
CREATE TABLE IF NOT EXISTS 'task_property' (
    'task_id' INT NOT NULL,
    'name' varchar(255) NOT NULL,
    'type' varchar(255) NOT NULL,
    FOREIGN KEY ('task_id') REFERENCES 'task'('id'),
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
    'task_id' INT NOT NULL REFERENCES 'task'('id'),
    'task_property_name' varchar(255) NOT NULL REFERENCES 'task_property'('name'),
    'value' varchar(255) NOT NULL,
    PRIMARY KEY ('task_id', 'task_property_name')
);
CREATE TABLE IF NOT EXISTS 'task_num_property' (
    'task_id' INT NOT NULL REFERENCES 'task'('id'),
    'task_property_name' varchar(255) NOT NULL REFERENCES 'task_property'('name'),
    'value' REAL NOT NULL,
    PRIMARY KEY ('task_id', 'task_property_name')
);
CREATE TABLE IF NOT EXISTS 'task_date_property' (
    'task_id' INT NOT NULL REFERENCES 'task'('id'),
    'task_property_name' varchar(255) NOT NULL REFERENCES 'task_property'('name'),
    'value' timestamp NOT NULL,
    PRIMARY KEY ('task_id', 'task_property_name')
);
CREATE TABLE IF NOT EXISTS 'task_bool_property' (
    'task_id' INT NOT NULL REFERENCES 'task'('id'),
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
    'task_id' INT NOT NULL REFERENCES 'task'('id'),
    'script_id' INT NOT NULL REFERENCES 'scripts'('id'),
    'event' varchar(255) NOT NULL,
    PRIMARY KEY ('task_id ', 'script_id')
);
CREATE TABLE IF NOT EXISTS 'global_property' (
    'name' varchar(255) NOT NULL,
    'type' varchar(255) NOT NULL,
    PRIMARY KEY ('name')
);
CREATE TABLE IF NOT EXISTS 'global_string_property' (
    'property_name' varchar(255) NOT NULL REFERENCES 'global_property'('name'),
    'value' varchar(255) NOT NULL,
    PRIMARY KEY ('property_name')
);
CREATE TABLE IF NOT EXISTS 'global_num_property' (
    'property_name' varchar(255) NOT NULL REFERENCES 'global_property'('name'),
    'value' REAL NOT NULL,
    PRIMARY KEY ('property_name')
);
CREATE TABLE IF NOT EXISTS 'global_date_property' (
    'property_name' varchar(255) NOT NULL REFERENCES 'global_property'('name'),
    'value' timestamp NOT NULL,
    PRIMARY KEY ('property_name')
);
CREATE TABLE IF NOT EXISTS 'global_bool_property' (
    'property_name' varchar(255) NOT NULL REFERENCES 'global_property'('name'),
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
CREATE MATERIALIZED VIEW task_properties AS
SELECT task.id AS task_id,
    task_property.name AS name,
    task_property.type AS type,
    task_string_property.value AS string_value,
    task_num_property.value AS num_value,
    task_date_property.value AS date_value,
    task_bool_property.value AS bool_value
FROM task
    LEFT JOIN task_property ON task.id = task_property.task_id
    LEFT JOIN task_string_property ON task.id = task_string_property.task_id
    LEFT JOIN task_num_property ON task.id = task_num_property.task_id
    LEFT JOIN task_date_property ON task.id = task_date_property.task_id
    LEFT JOIN task_bool_property ON task.id = task_bool_property.task_id;