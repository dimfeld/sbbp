CREATE TABLE videos (
  id uuid NOT NULL PRIMARY KEY,
  organization_id uuid NOT NULL REFERENCES organizations (id) ON DELETE CASCADE,
  updated_at timestamptz NOT NULL DEFAULT now(),
  created_at timestamptz NOT NULL DEFAULT now(),
  processing_state text NOT NULL,
  url text,
  images jsonb,
  title text,
  duration integer,
  read boolean NOT NULL DEFAULT FALSE,
  progress integer NOT NULL DEFAULT 0,
  summary text,
  processed_path text
);

CREATE INDEX videos_organization_id ON videos (organization_id);

CREATE INDEX videos_read ON videos (organization_id, read);

CREATE INDEX videos_updated_at ON videos (organization_id, updated_at DESC);

CREATE INDEX videos_created_at ON videos (organization_id, created_at DESC);
