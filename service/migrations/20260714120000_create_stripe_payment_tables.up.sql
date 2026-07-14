CREATE TABLE payment_attempts (
    id SERIAL PRIMARY KEY,
    pid UUID NOT NULL UNIQUE DEFAULT gen_random_uuid(),
    order_id INTEGER NOT NULL REFERENCES orders (id) ON DELETE RESTRICT,

    provider TEXT NOT NULL DEFAULT 'stripe'
        CONSTRAINT payment_attempts_provider_check CHECK (provider IN ('stripe')),
    status TEXT NOT NULL DEFAULT 'initiated'
        CONSTRAINT payment_attempts_status_check CHECK (
            status IN (
                'initiated',
                'session_created',
                'processing',
                'succeeded',
                'failed',
                'expired',
                'cancelled',
                'refunded'
            )
        ),
    amount NUMERIC(12, 2) NOT NULL
        CONSTRAINT payment_attempts_amount_check CHECK (amount >= 0),
    currency VARCHAR(3) NOT NULL
        CONSTRAINT payment_attempts_currency_check CHECK (currency ~ '^[A-Z]{3}$'),

    stripe_checkout_session_id TEXT UNIQUE,
    stripe_payment_intent_id TEXT UNIQUE,
    checkout_url TEXT,
    expires_at TIMESTAMPTZ,
    failure_code TEXT,
    failure_message TEXT,

    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    completed_at TIMESTAMPTZ
);

CREATE TRIGGER payment_attempts_updated_at_trigger
BEFORE UPDATE ON payment_attempts
FOR EACH ROW
EXECUTE FUNCTION set_updated_at();

CREATE INDEX payment_attempts_order_created_idx
ON payment_attempts (order_id, created_at DESC);

CREATE UNIQUE INDEX one_active_payment_attempt_per_order
ON payment_attempts (order_id)
WHERE status IN ('initiated', 'session_created', 'processing');

CREATE TABLE stripe_webhook_events (
    stripe_event_id TEXT PRIMARY KEY,
    event_type TEXT NOT NULL,
    api_version TEXT,
    payload JSONB NOT NULL
        CONSTRAINT stripe_webhook_events_payload_check CHECK (jsonb_typeof(payload) = 'object'),
    processed_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX stripe_webhook_events_type_created_idx
ON stripe_webhook_events (event_type, created_at DESC);
