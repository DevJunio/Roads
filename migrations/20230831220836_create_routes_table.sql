-- Add migration script here

CREATE TABLE routes(
  id uuid NOT NULL,
  PRIMARY KEY (id),
  route TEXT NOT NULL,
  redirect_to TEXT NOT NULL
);
