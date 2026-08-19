use crate::api::BASE_URL;
use crate::models::Topic;

/// Incrementally scans a Discourse JSON response for the `topic_list.topics`
/// array and yields each complete topic object as a raw JSON string. This lets
/// topics be deserialized and rendered one-by-one as bytes arrive, rather than
/// waiting for the whole document.
pub struct TopicStreamParser {
    found_array: bool,
}

impl TopicStreamParser {
    pub fn new() -> Self {
        Self { found_array: false }
    }

    /// Feeds the accumulated `buffer` and returns every topic object that is
    /// now complete, consuming those bytes from `buffer`. Incomplete objects
    /// stay in `buffer` for the next call.
    pub fn poll(&mut self, buffer: &mut Vec<u8>) -> Vec<String> {
        let mut out = Vec::new();
        loop {
            if !self.found_array {
                // Match the key `"topics"` (with its trailing colon) so a value
                // that happens to equal `"topics"` elsewhere is not mistaken for
                // it.
                let Some(key) = find_subslice(buffer, b"\"topics\":") else {
                    break;
                };
                let Some(rel) = buffer[key..].iter().position(|&b| b == b'[') else {
                    break;
                };
                buffer.drain(..=key + rel);
                self.found_array = true;
                continue;
            }

            let Some(start) = buffer.iter().position(|&b| b == b'{') else {
                break;
            };
            let Some(end) = find_object_end(&buffer[start..]) else {
                break;
            };
            let end = start + end;
            let json = String::from_utf8(buffer[start..=end].to_vec()).unwrap_or_default();
            out.push(json);
            buffer.drain(..=end);
        }
        out
    }
}

/// Streams the topics of a Discourse path such as `latest.json` or
/// `c/{slug}/{id}.json`, invoking `on_topic` for each topic as soon as its
/// object is fully received.
pub async fn stream_topics(
    path: &str,
    mut on_topic: impl FnMut(Topic),
) -> Result<(), String> {
    let client = reqwest::Client::new();
    let url = format!("{BASE_URL}/{path}");
    let mut response = client.get(url).send().await.map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        // Discourse serves a 403 (with `error_type: "not_logged_in"`) for lists
        // that require a logged-in user, such as `new.json`. Surface that as a
        // friendly message instead of a bare HTTP status.
        return Err(if response.status() == reqwest::StatusCode::FORBIDDEN {
            "Required to Login".to_string()
        } else {
            format!("HTTP {}", response.status())
        });
    }

    let mut buffer: Vec<u8> = Vec::new();
    let mut parser = TopicStreamParser::new();

    while let Some(chunk) = response.chunk().await.map_err(|e| e.to_string())? {
        buffer.extend_from_slice(&chunk);
        for json in parser.poll(&mut buffer) {
            if let Ok(topic) = serde_json::from_str::<Topic>(&json) {
                on_topic(topic);
            }
        }
    }

    Ok(())
}

/// Finds `needle` in `haystack`, returning its start index.
fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || haystack.len() < needle.len() {
        return None;
    }
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

/// Given a slice whose first byte is `{`, returns the index of the matching
/// closing `}`, respecting nested objects/arrays and string escapes.
fn find_object_end(bytes: &[u8]) -> Option<usize> {
    let mut depth = 0_i32;
    let mut in_string = false;
    let mut escaped = false;

    for (i, &b) in bytes.iter().enumerate() {
        if in_string {
            if escaped {
                escaped = false;
            } else if b == b'\\' {
                escaped = true;
            } else if b == b'"' {
                in_string = false;
            }
            continue;
        }

        match b {
            b'"' => in_string = true,
            b'{' | b'[' => depth += 1,
            b'}' | b']' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn topics_object() -> Vec<String> {
        let mut parser = TopicStreamParser::new();
        let mut buffer = b"{\"users\":[],\"topic_list\":{\"topics\":[{\"id\":1,\"title\":\"a\",\"slug\":\"a\"},{\"id\":2,\"title\":\"b\",\"slug\":\"b\"}]}}".to_vec();
        let mut out = parser.poll(&mut buffer);
        // trailing `]}}` is ignored
        out.extend(parser.poll(&mut buffer));
        out
    }

    #[test]
    fn extracts_each_topic_object() {
        let topics = topics_object();
        assert_eq!(topics.len(), 2);
        assert!(topics[0].starts_with("{\"id\":1"));
        assert!(topics[1].starts_with("{\"id\":2"));
    }

    #[test]
    fn handles_topics_split_across_chunks() {
        let mut parser = TopicStreamParser::new();
        let mut buffer = Vec::new();

        buffer.extend_from_slice(b"{\"users\":[],\"topic_list\":{\"top");
        assert!(parser.poll(&mut buffer).is_empty());

        // The first topic object is still incomplete: no closing brace yet.
        buffer.extend_from_slice(b"ics\":[{\"id\":1,\"title\":\"a\",\"slug\":");
        assert!(parser.poll(&mut buffer).is_empty());

        buffer.extend_from_slice(b"\"a\"},{\"id\":2,\"title\":\"b\",\"slug\":\"b\"}]}}");
        let topics = parser.poll(&mut buffer);
        assert_eq!(topics.len(), 2);
    }

    #[test]
    fn ignores_a_topics_value_before_the_key() {
        let mut parser = TopicStreamParser::new();
        // A user whose username is literally "topics" must not be mistaken for
        // the `"topics"` key.
        let mut buffer =
            br#"{"users":[{"username":"topics"}],"topic_list":{"topics":[{"id":1,"title":"a","slug":"a"}]}}"#
                .to_vec();
        let topics = parser.poll(&mut buffer);
        assert_eq!(topics.len(), 1);
        assert!(topics[0].starts_with("{\"id\":1"));
    }

    #[test]
    fn ignores_braces_inside_strings() {
        let mut parser = TopicStreamParser::new();
        let mut buffer =
            b"{\"topic_list\":{\"topics\":[{\"id\":1,\"title\":\"a }\",\"slug\":\"a\"}]}}".to_vec();
        let topics = parser.poll(&mut buffer);
        assert_eq!(topics.len(), 1);
        assert_eq!(topics[0], "{\"id\":1,\"title\":\"a }\",\"slug\":\"a\"}");
    }

    #[test]
    fn find_object_end_matches_nested_arrays() {
        let json = br#"{"id":1,"posters":[{"x":1}],"nested":{"a":{}}}"#;
        assert_eq!(find_object_end(json), Some(json.len() - 1));
    }
}
