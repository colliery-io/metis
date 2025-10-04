-- Add configuration table for flight level settings
CREATE TABLE configuration (
    key TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL,
    updated_at REAL NOT NULL
);

-- Insert default flight level configuration
INSERT INTO configuration (key, value, updated_at) VALUES 
    ('flight_levels', '{"strategies_enabled":true,"initiatives_enabled":true}', strftime('%s', 'now'));