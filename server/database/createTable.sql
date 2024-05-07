CREATE SCHEMA IF NOT EXISTS task;
SET search_path TO task;
CREATE TABLE IF NOT EXISTS "task" (
    "id" SERIAL,
    "title" varchar(255) NOT NULL,
    "completed" boolean NOT NULL DEFAULT false,
    "last_edited" timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY ("id")
);
CREATE TABLE IF NOT EXISTS "dependency"(
    "task_id" INT NOT NULL,
    "depends_on_id" INT NOT NULL,
    FOREIGN KEY ("task_id") REFERENCES "task"("id") ON DELETE CASCADE,
    FOREIGN KEY ("depends_on_id") REFERENCES "task"("id") ON DELETE CASCADE,
    PRIMARY KEY ("task_id", "depends_on_id")
);
CREATE TABLE IF NOT EXISTS "task_property" (
    "task_id" INT NOT NULL,
    "name" varchar(255) NOT NULL,
    "type" TEXT NOT NULL,
    FOREIGN KEY ("task_id") REFERENCES "task"("id") ON DELETE CASCADE,
    PRIMARY KEY ("task_id", "name")
);
CREATE TABLE IF NOT EXISTS "task_string_property" (
    "task_id" INT NOT NULL,
    "task_property_name" varchar(255) NOT NULL,
    "value" TEXT NOT NULL,
    PRIMARY KEY ("task_id", "task_property_name"),
    FOREIGN KEY ("task_id", "task_property_name") REFERENCES "task_property"("task_id", "name") ON DELETE CASCADE
);
CREATE TABLE IF NOT EXISTS "task_num_property" (
    "task_id" INT NOT NULL,
    "task_property_name" varchar(255) NOT NULL,
    "value" NUMERIC NOT NULL,
    PRIMARY KEY ("task_id", "task_property_name"),
    FOREIGN KEY ("task_id", "task_property_name") REFERENCES "task_property"("task_id", "name") ON DELETE CASCADE
);
CREATE TABLE IF NOT EXISTS "task_date_property" (
    "task_id" INT NOT NULL,
    "task_property_name" varchar(255) NOT NULL,
    "value" timestamp NOT NULL,
    PRIMARY KEY ("task_id", "task_property_name"),
    FOREIGN KEY ("task_id", "task_property_name") REFERENCES "task_property"("task_id", "name") ON DELETE CASCADE
);
CREATE TABLE IF NOT EXISTS "task_bool_property" (
    "task_id" INT NOT NULL,
    "task_property_name" varchar(255) NOT NULL,
    "value" BOOLEAN NOT NULL,
    PRIMARY KEY ("task_id", "task_property_name"),
    FOREIGN KEY ("task_id", "task_property_name") REFERENCES "task_property"("task_id", "name") ON DELETE CASCADE
);
CREATE TABLE IF NOT EXISTS "scripts" (
    "id" SERIAL,
    "name" varchar(255) NOT NULL,
    "code" text NOT NULL,
    PRIMARY KEY ("id")
);
CREATE TABLE IF NOT EXISTS "task_scripts" (
    "task_id" INT NOT NULL REFERENCES "task"("id") ON DELETE CASCADE,
    "script_id" INT NOT NULL REFERENCES "scripts"("id") ON DELETE CASCADE,
    "event" varchar(255) NOT NULL,
    PRIMARY KEY ("task_id", "script_id")
);
CREATE TABLE IF NOT EXISTS "global_property" (
    "name" varchar(255) NOT NULL,
    "type" TEXT NOT NULL,
    PRIMARY KEY ("name")
);
CREATE TABLE IF NOT EXISTS "global_string_property" (
    "property_name" varchar(255) NOT NULL REFERENCES "global_property"("name") ON DELETE CASCADE,
    "value" TEXT NOT NULL,
    PRIMARY KEY ("property_name")
);
CREATE TABLE IF NOT EXISTS "global_num_property" (
    "property_name" varchar(255) NOT NULL REFERENCES "global_property"("name") ON DELETE CASCADE,
    "value" REAL NOT NULL,
    PRIMARY KEY ("property_name")
);
CREATE TABLE IF NOT EXISTS "global_date_property" (
    "property_name" varchar(255) NOT NULL REFERENCES "global_property"("name") ON DELETE CASCADE,
    "value" timestamp NOT NULL,
    PRIMARY KEY ("property_name")
);
CREATE TABLE IF NOT EXISTS "global_bool_property" (
    "property_name" varchar(255) NOT NULL REFERENCES "global_property"("name") ON DELETE CASCADE,
    "value" boolean NOT NULL,
    PRIMARY KEY ("property_name")
);
CREATE OR REPLACE FUNCTION check_cycle() RETURNS TRIGGER AS $$
DECLARE cycle BOOLEAN;
BEGIN WITH RECURSIVE cte AS (
    SELECT NEW.task_id,
        NEW.depends_on_id
    FROM NEW
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
DO $$ BEGIN IF NOT EXISTS (
    SELECT 1
    FROM pg_class c
        JOIN pg_namespace n ON n.oid = c.relnamespace
    WHERE c.relname = 'task_depend_on_index'
        AND n.nspname = 'public'
) THEN CREATE INDEX task_depend_on_index ON dependency (depends_on_id);
END IF;
END;
$$;
DO $$ BEGIN IF NOT EXISTS (
    SELECT 1
    FROM pg_class c
        JOIN pg_namespace n ON n.oid = c.relnamespace
    WHERE c.relname = 'task_date_property_value_index'
        AND n.nspname = 'public'
) THEN CREATE INDEX task_date_property_value_index ON task_date_property (value);
END IF;
END;
$$;
DO $$ BEGIN IF NOT EXISTS (
    SELECT 1
    FROM pg_class c
        JOIN pg_namespace n ON n.oid = c.relnamespace
    WHERE c.relname = 'task_num_property_value_index'
        AND n.nspname = 'public'
) THEN CREATE INDEX task_num_property_value_index ON task_num_property (value);
END IF;
END;
$$;
DO $$ BEGIN IF NOT EXISTS (
    SELECT 1
    FROM pg_class c
        JOIN pg_namespace n ON n.oid = c.relnamespace
    WHERE c.relname = 'task_string_property_value_index'
        AND n.nspname = 'public'
) THEN CREATE INDEX task_string_property_value_index ON task_string_property (value);
END IF;
END;
$$;
DO $$ BEGIN IF NOT EXISTS (
    SELECT 1
    FROM pg_class c
        JOIN pg_namespace n ON n.oid = c.relnamespace
    WHERE c.relname = 'task_bool_property_value_index'
        AND n.nspname = 'public'
) THEN CREATE INDEX task_bool_property_value_index ON task_bool_property (value);
END IF;
END;
$$;
---CREATE INDEX task_property_type_index on task_property (jsonb_typeof(value));
CREATE OR REPLACE FUNCTION check_property() RETURNS TRIGGER AS $$ BEGIN IF EXISTS (
        SELECT 1
        FROM task_property
        WHERE id = NEW.task_id
            AND name = NEW.name
    ) THEN RAISE EXCEPTION 'Primary key already exists in task_property table';
ELSE
INSERT INTO task_property(id, name, type)
VALUES (NEW.task_id, NEW.name, TG_ARGV [0]);
RETURN NEW;
END IF;
END;
$$ LANGUAGE plpgsql;
CREATE TRIGGER string_property_trigger BEFORE
INSERT ON task_string_property FOR EACH ROW EXECUTE FUNCTION check_property("string");
CREATE TRIGGER num_property_trigger BEFORE
INSERT ON task_num_property FOR EACH ROW EXECUTE FUNCTION check_property("real");
CREATE TRIGGER date_property_trigger BEFORE
INSERT ON task_date_property FOR EACH ROW EXECUTE FUNCTION check_property("timestamp");
CREATE TRIGGER bool_property_trigger BEFORE
INSERT ON task_bool_property FOR EACH ROW EXECUTE FUNCTION check_property("boolean");
CREATE OR REPLACE FUNCTION update_last_edited() RETURNS TRIGGER AS $$ BEGIN
UPDATE task
SET last_edited = NOW()
WHERE id = NEW.task_id;
RETURN NEW;
END;
$$ LANGUAGE plpgsql;
CREATE TRIGGER task_table_updated_trigger AFTER
UPDATE
ON task FOR EACH ROW EXECUTE FUNCTION update_last_edited();
CREATE OR REPLACE FUNCTION task_id_tables() RETURNS VOID AS $$
DECLARE tbl_name text;
BEGIN FOR tbl_name IN
SELECT table_name
FROM information_schema.columns
WHERE column_name = 'task_id' LOOP EXECUTE format(
        '
         CREATE TRIGGER update_last_edited_trigger
         AFTER INSERT OR UPDATE OR DELETE ON %I
         FOR EACH ROW EXECUTE PROCEDURE update_last_edited();
      ',
        tbl_name
    );
END LOOP;
END $$ LANGUAGE plpgsql;
SELECT task_id_tables();
INSERT INTO task (completed, title)
VALUES (
        true,
        'give ABN an A for their Alpha Release!'
    );
INSERT INTO task (completed, title)
VALUES (false, 'make dinner');
CREATE TABLE IF NOT EXISTS "view" (
    "id" SERIAL PRIMARY KEY,
    "name" text,
    "properties" text[]  NOT NULL,
    "filter" jsonb NOT NULL
);
