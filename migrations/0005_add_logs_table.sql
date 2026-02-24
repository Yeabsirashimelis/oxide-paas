CREATE TABLE logs (
    id BIGSERIAL PRIMARY KEY,
    app_id UUID NOT NULL REFERENCES apps(id) ON DELETE CASCADE,
    stream TEXT NOT NULL,  -- 'stdout' or 'stderr'
    message TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_logs_app_id ON logs(app_id);
CREATE INDEX idx_logs_created_at ON logs(created_at);
