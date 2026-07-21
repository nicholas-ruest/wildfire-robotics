-- Prompt 04 / ADR-018, ADR-024, ADR-028, ADR-041.
-- Expand-only initial migration. A context may install these primitives in its
-- database without acquiring an unbounded table lock.
SET lock_timeout = '5s';
SET statement_timeout = '30s';

CREATE SCHEMA IF NOT EXISTS wr_infra;

CREATE TABLE IF NOT EXISTS wr_infra.aggregate_state (
    tenant_id uuid NOT NULL,
    aggregate_type text NOT NULL CHECK (length(aggregate_type) BETWEEN 1 AND 128),
    aggregate_id uuid NOT NULL,
    version bigint NOT NULL CHECK (version >= 1),
    state jsonb NOT NULL,
    updated_at timestamptz NOT NULL DEFAULT transaction_timestamp(),
    PRIMARY KEY (tenant_id, aggregate_type, aggregate_id)
);

CREATE TABLE IF NOT EXISTS wr_infra.outbox (
    tenant_id uuid NOT NULL,
    message_id uuid NOT NULL,
    aggregate_type text NOT NULL CHECK (length(aggregate_type) BETWEEN 1 AND 128),
    aggregate_id uuid NOT NULL,
    aggregate_version bigint NOT NULL CHECK (aggregate_version >= 1),
    subject text NOT NULL CHECK (length(subject) BETWEEN 1 AND 512),
    content_type text NOT NULL CHECK (length(content_type) BETWEEN 1 AND 255),
    classification text NOT NULL CHECK (classification IN ('PUBLIC','INTERNAL','CONFIDENTIAL','RESTRICTED')),
    payload bytea NOT NULL CHECK (octet_length(payload) <= 1048576),
    payload_sha256 char(64) NOT NULL CHECK (payload_sha256 ~ '^[0-9a-f]{64}$'),
    occurred_at timestamptz NOT NULL,
    recorded_at timestamptz NOT NULL DEFAULT transaction_timestamp(),
    available_at timestamptz NOT NULL DEFAULT transaction_timestamp(),
    attempts integer NOT NULL DEFAULT 0 CHECK (attempts >= 0),
    claimed_by text,
    claimed_at timestamptz,
    published_at timestamptz,
    quarantined_at timestamptz,
    last_error text,
    PRIMARY KEY (tenant_id, message_id),
    UNIQUE (tenant_id, aggregate_type, aggregate_id, aggregate_version)
);
CREATE INDEX IF NOT EXISTS outbox_relay_ready_idx
    ON wr_infra.outbox (available_at, recorded_at)
    WHERE published_at IS NULL AND quarantined_at IS NULL;

CREATE TABLE IF NOT EXISTS wr_infra.inbox (
    tenant_id uuid NOT NULL,
    consumer text NOT NULL CHECK (length(consumer) BETWEEN 1 AND 128),
    message_id uuid NOT NULL,
    business_key text,
    processed_at timestamptz NOT NULL DEFAULT transaction_timestamp(),
    payload_sha256 char(64) NOT NULL CHECK (payload_sha256 ~ '^[0-9a-f]{64}$'),
    PRIMARY KEY (tenant_id, consumer, message_id)
);
CREATE UNIQUE INDEX IF NOT EXISTS inbox_business_key_idx
    ON wr_infra.inbox (tenant_id, consumer, business_key)
    WHERE business_key IS NOT NULL;

CREATE TABLE IF NOT EXISTS wr_infra.poison_message (
    tenant_id uuid NOT NULL,
    message_id uuid NOT NULL,
    consumer_or_relay text NOT NULL,
    payload bytea NOT NULL CHECK (octet_length(payload) <= 1048576),
    payload_sha256 char(64) NOT NULL CHECK (payload_sha256 ~ '^[0-9a-f]{64}$'),
    failure_count integer NOT NULL CHECK (failure_count > 0),
    reason text NOT NULL CHECK (length(reason) BETWEEN 1 AND 4096),
    quarantined_at timestamptz NOT NULL DEFAULT transaction_timestamp(),
    repaired_at timestamptz,
    repair_actor text,
    PRIMARY KEY (tenant_id, message_id, consumer_or_relay)
);

CREATE TABLE IF NOT EXISTS wr_infra.lease (
    tenant_id uuid NOT NULL,
    resource_key text NOT NULL CHECK (length(resource_key) BETWEEN 1 AND 256),
    holder_id text NOT NULL CHECK (length(holder_id) BETWEEN 1 AND 256),
    fencing_token bigint NOT NULL CHECK (fencing_token > 0),
    acquired_at timestamptz NOT NULL DEFAULT transaction_timestamp(),
    expires_at timestamptz NOT NULL,
    CHECK (expires_at > acquired_at),
    PRIMARY KEY (tenant_id, resource_key)
);

COMMENT ON SCHEMA wr_infra IS 'Reusable technical persistence primitives; no business aggregates or cross-context tables';
