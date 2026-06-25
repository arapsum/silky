use std::sync::OnceLock;

static CLEANUP_UUID: OnceLock<Vec<(&'static str, &'static str)>> = OnceLock::new();
static CLEANUP_DATE: OnceLock<Vec<(&'static str, &'static str)>> = OnceLock::new();
static CLEANUP_PASSWORD: OnceLock<Vec<(&'static str, &'static str)>> = OnceLock::new();
static CLEANUP_JWT: OnceLock<Vec<(&'static str, &'static str)>> = OnceLock::new();
static CLEANUP_HEADERS: OnceLock<Vec<(&'static str, &'static str)>> = OnceLock::new();
static CLEANUP_VERIFICATION_TOKEN: OnceLock<Vec<(&'static str, &'static str)>> = OnceLock::new();
static CLEANUP_HASHED_TOKEN: OnceLock<Vec<(&'static str, &'static str)>> = OnceLock::new();
static CLEANUP_ID: OnceLock<Vec<(&'static str, &'static str)>> = OnceLock::new();

pub fn cleanup_id() -> &'static Vec<(&'static str, &'static str)> {
    CLEANUP_ID.get_or_init(|| vec![(r"id:\s*\d+", "id: ID")])
}

pub fn cleanup_uuid() -> &'static Vec<(&'static str, &'static str)> {
    CLEANUP_UUID.get_or_init(|| {
        vec![(
            r"([0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12})",
            "PID",
        )]
    })
}

pub fn cleanup_date() -> &'static Vec<(&'static str, &'static str)> {
    CLEANUP_DATE.get_or_init(|| {
        vec![
            (
                r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(\.\d+)?\+\d{2}:\d{2}",
                "DATE",
            ), // with tz
            (r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d+", "DATE"),
            (r"(\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2})", "DATE"),
            (r"(\d{2})-(\d{2})-(\d{4})\s+(\d{2}):(\d{2}):(\d{2})", "DATE"),
            (r#"\d{2}-\d{2}-\d{4} \d{2}:\d{2}"#, "DATE"),
            (r"\d{4}[-/]\d{2}[-/]\d{2}", "DATE"),
        ]
    })
}

pub fn cleanup_password() -> &'static Vec<(&'static str, &'static str)> {
    CLEANUP_PASSWORD
        .get_or_init(|| vec![(r"password_hash: (.*{60}),", "password_hash: \"PASSWORD\",")])
}

pub fn cleanup_verification_token() -> &'static Vec<(&'static str, &'static str)> {
    CLEANUP_VERIFICATION_TOKEN.get_or_init(|| vec![(r#"\b[a-f0-9]{64}\b"#, "TOKEN")])
}

pub fn cleanup_hashed_token() -> &'static Vec<(&'static str, &'static str)> {
    CLEANUP_HASHED_TOKEN.get_or_init(|| vec![(r#"\b[a-f0-9]{64}\b"#, "TOKEN")])
}

pub fn cleanup_jwt() -> &'static Vec<(&'static str, &'static str)> {
    CLEANUP_JWT.get_or_init(|| vec![(r"[A-Za-z0-9-_]+\.[A-Za-z0-9-_]+\.[A-Za-z0-9-_]+", "JWT")])
}

pub fn cleanup_headers() -> &'static Vec<(&'static str, &'static str)> {
    CLEANUP_HEADERS.get_or_init(|| {
        vec![(
            r#""content-length":\s*"\d+""#,
            r#""content-length": "NUMBER""#,
        )]
    })
}
