pub(super) const VERSION_BACKGROUNDS: [&str; 7] = [
    "backgrounds/version-classic.png",
    "backgrounds/version-trials.png",
    "backgrounds/version-grove.png",
    "backgrounds/version-peaks.png",
    "backgrounds/version-jungle.png",
    "backgrounds/version-cavern.png",
    "backgrounds/version-wilderness.png",
];

pub(super) fn version_background(
    major: Option<u32>,
    key: Option<(u32, Option<u32>)>,
) -> Option<&'static str> {
    let index = match (major?, key.map(|key| key.0)) {
        (8, _) => 0,
        (21, Some(1)) => 1,
        (21, Some(10)) => 2,
        (21, Some(11)) => 3,
        (26, Some(1)) => 4,
        (26, Some(2)) => 5,
        (26, Some(3)) => 6,
        _ => return None,
    };
    Some(VERSION_BACKGROUNDS[index])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn installed_release_lines_have_distinct_art() {
        let releases = [
            (8, None),
            (21, Some(1)),
            (21, Some(10)),
            (21, Some(11)),
            (26, Some(1)),
            (26, Some(2)),
            (26, Some(3)),
        ];
        let art: std::collections::HashSet<_> = releases
            .into_iter()
            .map(|(major, minor)| {
                version_background(Some(major), minor.map(|v| (v, None))).unwrap()
            })
            .collect();
        assert_eq!(art.len(), 7);
    }

    #[test]
    fn patches_share_their_release_art() {
        assert_eq!(
            version_background(Some(26), Some((1, Some(2)))),
            version_background(Some(26), Some((1, None)))
        );
    }

    #[test]
    fn unknown_versions_keep_the_remote_fallback() {
        assert_eq!(version_background(Some(27), Some((1, None))), None);
        assert_eq!(version_background(None, None), None);
        assert_eq!(version_background(Some(26), None), None);
    }
}
