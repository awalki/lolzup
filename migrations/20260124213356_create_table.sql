CREATE TABLE tasks (
    id INTEGER PRIMARY KEY,
    thread_id INTEGER NOT NULL UNIQUE,
    run_at TIMESTAMPTZ NOT NULL
);

CREATE INDEX idx_tasks_run_at ON tasks (run_at);