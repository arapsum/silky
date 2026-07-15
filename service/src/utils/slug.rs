/// Converts a display name into a lowercase, URL-safe slug.
///
/// Consecutive non-ASCII-alphanumeric characters collapse into a single
/// hyphen. The supplied fallback is returned when the value contains no
/// characters that can form a slug.
#[must_use]
pub fn slugify(value: &str, fallback: &str) -> String {
    let slug = value
        .to_lowercase()
        .trim()
        .split(|character: char| !character.is_ascii_alphanumeric())
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-");

    if slug.is_empty() {
        fallback.to_string()
    } else {
        slug
    }
}
