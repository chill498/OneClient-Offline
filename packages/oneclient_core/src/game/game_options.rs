//! The game's own `options.txt` is the source of truth for the settings the
//! launcher exposes, so the game's crash fallback can still rewrite them

use std::path::Path;

use oneclient_common::domain::GraphicsBackend;

use crate::LauncherResult;
use crate::game::GameError;

const OPTIONS_NAME: &str = "options.txt";
const GRAPHICS_BACKEND_KEY: &str = "preferredGraphicsBackend:";

/// `None` until a game that has the option saves the file
pub async fn read_graphics_backend(game_dir: &Path) -> Option<GraphicsBackend> {
    let options = polyio::read_to_string(game_dir.join(OPTIONS_NAME))
        .await
        .ok()?;
    serde_json::from_str(find_graphics_backend(&options)?).ok()
}

/// Only replaces a line the game wrote itself, the game resets the option
/// in any file it has not upgraded yet
pub async fn write_graphics_backend(
    game_dir: &Path,
    backend: GraphicsBackend,
) -> LauncherResult<()> {
    let path = game_dir.join(OPTIONS_NAME);
    let options = polyio::read_to_string(&path).await.unwrap_or_default();
    let Some(current) = find_graphics_backend(&options) else {
        return Err(GameError::OptionsOutdated.into());
    };

    let updated = options.replacen(
        &format!("{GRAPHICS_BACKEND_KEY}{current}"),
        &format!("{GRAPHICS_BACKEND_KEY}{}", serde_json::to_string(&backend)?),
        1,
    );
    polyio::write(&path, updated).await?;
    Ok(())
}

fn find_graphics_backend(options: &str) -> Option<&str> {
    options
        .lines()
        .find_map(|line| line.strip_prefix(GRAPHICS_BACKEND_KEY))
}

/// Only touch options supported and already saved by this Minecraft version.
/// A first launch with no options file is deliberately left to the game.
pub(crate) async fn apply_cursor_compatibility(game_dir: &Path) -> LauncherResult<bool> {
    let path = game_dir.join(OPTIONS_NAME);
    if !polyio::try_exists(&path).await? {
        return Ok(false);
    }
    let options = polyio::read_to_string(&path).await?;
    let updated = cursor_compatible_options(&options);
    if updated == options {
        return Ok(false);
    }
    polyio::write_atomic(&path, updated).await?;
    Ok(true)
}

fn cursor_compatible_options(options: &str) -> String {
    options
        .split_inclusive('\n')
        .map(|line| {
            let value = line.trim_end_matches(['\r', '\n']);
            let newline = &line[value.len()..];
            match value {
                "exclusiveFullscreen:true" => format!("exclusiveFullscreen:false{newline}"),
                "allowCursorChanges:true" => format!("allowCursorChanges:false{newline}"),
                _ => line.to_string(),
            }
        })
        .collect()
}

#[cfg(test)]
mod cursor_tests {
    use super::*;

    #[test]
    fn compatibility_preserves_fullscreen_input_and_unrelated_settings() {
        let original = "version:4903\r\nfullscreen:true\r\nexclusiveFullscreen:true\r\nrawMouseInput:true\r\nallowCursorChanges:true\r\nmusic:0.4";
        let expected = "version:4903\r\nfullscreen:true\r\nexclusiveFullscreen:false\r\nrawMouseInput:true\r\nallowCursorChanges:false\r\nmusic:0.4";
        assert_eq!(cursor_compatible_options(original), expected);
        assert_eq!(cursor_compatible_options(expected), expected);
    }

    #[test]
    fn old_or_absent_options_are_not_invented() {
        for original in [
            "",
            "version:1343\nfullscreen:false\n",
            "exclusiveFullscreen:future\ncustom:allowCursorChanges:true\n",
        ] {
            assert_eq!(cursor_compatible_options(original), original);
        }
    }
}
