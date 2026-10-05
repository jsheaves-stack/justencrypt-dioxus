//! Small cross-platform helpers shared by code that isn't itself request- or
//! UI-specific enough to belong under `requests` or `components`.

use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};
use std::{borrow::Cow, time::Duration};

#[cfg(not(target_arch = "wasm32"))]
use tokio::time;

/// Characters percent-encoded in a single URL path segment: everything URL
/// parsing would otherwise treat as syntax rather than as part of a file or
/// folder name - `?` (query), `#` (fragment), `%` (escape introducer), `/`
/// and `\` (segment separators; the URL standard treats `\` as `/` for
/// http(s) URLs) - plus space, quotes, and the other characters the URL
/// standard always escapes in paths. Controls and all non-ASCII bytes are
/// encoded too (`CONTROLS` and `utf8_percent_encode` cover those).
const PATH_SEGMENT: &AsciiSet = &CONTROLS
    .add(b' ')
    .add(b'"')
    .add(b'#')
    .add(b'%')
    .add(b'/')
    .add(b'<')
    .add(b'>')
    .add(b'?')
    .add(b'\\')
    .add(b'`')
    .add(b'{')
    .add(b'}');

/// Percent-encodes one file or folder name for use as a single URL path
/// segment, so e.g. `notes#1.txt` stays one segment named `notes#1.txt`
/// instead of being truncated to `notes` at the `#`.
///
/// Note this can't make the names `.` or `..` safe: the URL standard treats
/// their encoded forms (`%2e`, `%2e%2e`) as dot segments too, so those have
/// to be rejected before they ever reach a URL.
pub fn encode_segment(segment: &str) -> Cow<'_, str> {
    utf8_percent_encode(segment, PATH_SEGMENT).into()
}

/// Appends `/{segment}` to `url` for each folder in `current_path`, then for
/// `item_name` (skipped if `None` or empty), each percent-encoded via
/// `encode_segment`. The one place request and media URLs turn names into
/// path segments, so every URL escapes them the same way.
pub fn push_path_segments(url: &mut String, current_path: &[String], item_name: Option<&str>) {
    let name = item_name.filter(|name| !name.is_empty());
    for segment in current_path.iter().map(String::as_str).chain(name) {
        url.push('/');
        url.push_str(&encode_segment(segment));
    }
}

/// Sleeps for `duration` on either platform: `tokio::time::sleep` on
/// desktop, `gloo_timers` (backed by `setTimeout`) on wasm.
pub async fn sleep(duration: Duration) {
    #[cfg(target_arch = "wasm32")]
    gloo_timers::future::sleep(duration).await;

    #[cfg(not(target_arch = "wasm32"))]
    time::sleep(duration).await;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn path(segments: &[&str]) -> Vec<String> {
        segments.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn encode_segment_leaves_ordinary_names_untouched() {
        assert_eq!(encode_segment("report-2024_v1.txt"), "report-2024_v1.txt");
    }

    #[test]
    fn encode_segment_escapes_url_syntax_characters() {
        assert_eq!(encode_segment("notes#1.txt"), "notes%231.txt");
        assert_eq!(encode_segment("invoice?2024.pdf"), "invoice%3F2024.pdf");
        assert_eq!(encode_segment("100%41.txt"), "100%2541.txt");
        assert_eq!(encode_segment("a/b"), "a%2Fb");
        assert_eq!(encode_segment("a\\b"), "a%5Cb");
        assert_eq!(encode_segment("my file.txt"), "my%20file.txt");
    }

    #[test]
    fn encode_segment_escapes_non_ascii() {
        assert_eq!(encode_segment("café"), "caf%C3%A9");
    }

    #[test]
    fn push_path_segments_encodes_each_segment_separately() {
        let mut url = String::from("/file");
        push_path_segments(&mut url, &path(&["a b", "c#d"]), Some("e?f.txt"));
        assert_eq!(url, "/file/a%20b/c%23d/e%3Ff.txt");
    }

    #[test]
    fn push_path_segments_skips_missing_or_empty_name() {
        let mut url = String::from("/folder");
        push_path_segments(&mut url, &path(&["a"]), None);
        assert_eq!(url, "/folder/a");

        let mut url = String::from("/folder");
        push_path_segments(&mut url, &[], Some(""));
        assert_eq!(url, "/folder");
    }
}
