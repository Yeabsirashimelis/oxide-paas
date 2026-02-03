
CREATE TYPE app_status AS ENUM('PENDING', 'RUNNING', 'FAILED', 'STOPPED');

create table App (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    command TEXT NOT NULL,
    status app_status NOT NULL,
    port INTEGER
)