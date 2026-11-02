//! Local, unauthenticated cosmetics. These files never become account credentials
//! or get uploaded to Microsoft. In-game use requires CustomSkinLoader.
use std::io::Cursor;
use std::path::{Path, PathBuf};

use image::{ImageFormat, ImageReader};
use oneclient_auth::MinecraftAccount;
use oneclient_common::paths;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::{LauncherError, LauncherResult};

const MAX_PNG_BYTES: usize = 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LocalSkin {
    pub png: Vec<u8>,
    pub slim: bool,
    pub file_name: String,
}

fn invalid(message: impl Into<String>) -> LauncherError {
    std::io::Error::new(std::io::ErrorKind::InvalidData, message.into()).into()
}

impl LocalSkin {
    pub fn validate(&self) -> LauncherResult<()> {
        if self.png.len() > MAX_PNG_BYTES {
            return Err(invalid("Choose a PNG skin smaller than 1 MB."));
        }
        let reader = ImageReader::with_format(Cursor::new(&self.png), ImageFormat::Png);
        let dimensions = reader
            .into_dimensions()
            .map_err(|_| invalid("Choose a valid PNG skin."))?;
        if dimensions != (64, 64) {
            return Err(invalid("Choose a 64 × 64 pixel PNG skin."));
        }
        image::load_from_memory_with_format(&self.png, ImageFormat::Png)
            .map_err(|_| invalid("This PNG skin is damaged. Choose another file."))?;
        Ok(())
    }

    pub async fn from_file(path: &Path, slim: bool) -> LauncherResult<Self> {
        // Bound the read as well as the decoder's dimensions.
        use tokio::io::AsyncReadExt;
        let file = tokio::fs::File::open(path).await?;
        let mut png = Vec::new();
        file.take((MAX_PNG_BYTES + 1) as u64)
            .read_to_end(&mut png)
            .await?;
        let skin = Self {
            png,
            slim,
            file_name: path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned(),
        };
        skin.validate()?;
        Ok(skin)
    }
}

fn skin_path(root: &Path, id: Uuid) -> PathBuf {
    root.join("offline-skins").join(format!("{id}.json"))
}

pub async fn load(id: Uuid) -> LauncherResult<Option<LocalSkin>> {
    load_from(paths::config_dir()?, id).await
}

async fn load_from(root: &Path, id: Uuid) -> LauncherResult<Option<LocalSkin>> {
    let path = skin_path(root, id);
    match tokio::fs::metadata(&path).await {
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(err) => return Err(err.into()),
        Ok(metadata) if metadata.len() > 5 * MAX_PNG_BYTES as u64 => {
            return Err(invalid("The saved skin file is too large."));
        }
        Ok(_) => {}
    }
    let skin: LocalSkin = polyio::read_json(&path).await?;
    skin.validate()?;
    Ok(Some(skin))
}

pub async fn save(account: &MinecraftAccount, skin: Option<&LocalSkin>) -> LauncherResult<()> {
    save_to(paths::config_dir()?, account, skin).await
}

async fn save_to(
    root: &Path,
    account: &MinecraftAccount,
    skin: Option<&LocalSkin>,
) -> LauncherResult<()> {
    if !account.is_offline() {
        return Err(invalid(
            "Local skins are only for offline accounts. Use the wardrobe for Microsoft accounts.",
        ));
    }
    let path = skin_path(root, account.id);
    match skin {
        Some(skin) => {
            skin.validate()?;
            polyio::create_dir_all(path.parent().unwrap()).await?;
            polyio::write_json_atomic(&path, skin).await?;
        }
        None => remove_if_exists(&path).await?,
    }
    Ok(())
}

async fn remove_if_exists(path: &Path) -> LauncherResult<()> {
    match tokio::fs::remove_file(path).await {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(err) => Err(err.into()),
    }
}

/// Export only our own files. CustomSkinLoader's ExtraList API prepends this
/// local provider without replacing its existing configuration or providers.
/// Each username resolves to its own profile instead of a fixed PNG that would
/// replace every player's skin.
pub async fn prepare_for_launch(game_dir: &Path, account: &MinecraftAccount) -> LauncherResult<()> {
    let skin = if account.is_offline() {
        load(account.id).await?
    } else {
        None
    };
    export(game_dir, account, skin.as_ref()).await
}

async fn export(
    game_dir: &Path,
    account: &MinecraftAccount,
    skin: Option<&LocalSkin>,
) -> LauncherResult<()> {
    // Validate even if a caller passes a manually constructed account.
    oneclient_auth::validate_offline_username(&account.username)?;
    let root = game_dir.join("CustomSkinLoader");
    let profile = root
        .join("OneClientOffline")
        .join(format!("{}.json", account.username));
    let Some(skin) = skin.filter(|_| account.is_offline()) else {
        // Also clear a stale local override when launching a Microsoft identity.
        return remove_if_exists(&profile).await;
    };
    skin.validate()?;
    let hash: String = Sha256::digest(&skin.png)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    let textures = root.join("OneClientOffline/textures");
    polyio::create_dir_all(&textures).await?;
    let texture_path = textures.join(&hash);
    tokio::fs::write(texture_path, &skin.png).await?;
    let model = if skin.slim { "slim" } else { "default" };
    polyio::write_json_atomic(
        &profile,
        &serde_json::json!({
            "username": account.username,
            "textures": {(model): hash},
        }),
    )
    .await?;
    let extra = root.join("ExtraList");
    polyio::create_dir_all(&extra).await?;
    polyio::write_json_atomic(
        extra.join("OneClientOffline.json"),
        &serde_json::json!({
            "name": "OneClient Offline Skins",
            "type": "CustomSkinAPI",
            "root": "OneClientOffline/",
        }),
    )
    .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(width: u32, height: u32) -> LocalSkin {
        let mut png = Cursor::new(Vec::new());
        image::DynamicImage::new_rgba8(width, height)
            .write_to(&mut png, ImageFormat::Png)
            .unwrap();
        LocalSkin {
            png: png.into_inner(),
            slim: false,
            file_name: "skin.png".into(),
        }
    }

    #[test]
    fn validates_png_data_and_dimensions() {
        assert!(sample(64, 64).validate().is_ok());
        assert!(sample(64, 32).validate().is_err());
        assert!(sample(128, 128).validate().is_err());
        let mut damaged = sample(64, 64);
        damaged.png.truncate(40);
        assert!(damaged.validate().is_err());
        damaged.png = vec![0; MAX_PNG_BYTES + 1];
        assert!(damaged.validate().is_err());
    }

    #[tokio::test]
    async fn saves_copied_skin_and_model_per_account_and_removes_it() {
        let root = std::env::temp_dir().join(format!("oneclient-skin-test-{}", Uuid::new_v4()));
        let first = oneclient_auth::offline_account("SkinTest".into());
        let other = oneclient_auth::offline_account("OtherPlayer".into());
        let mut skin = sample(64, 64);
        skin.slim = true;
        tokio::fs::create_dir_all(&root).await.unwrap();
        let original = root.join("chosen-skin.png");
        tokio::fs::write(&original, &skin.png).await.unwrap();
        skin = LocalSkin::from_file(&original, true).await.unwrap();
        save_to(&root, &first, Some(&skin)).await.unwrap();
        tokio::fs::remove_file(&original).await.unwrap();
        assert_eq!(
            load_from(&root, first.id).await.unwrap(),
            Some(skin.clone())
        );
        assert_eq!(load_from(&root, other.id).await.unwrap(), None);
        let mut broken = skin.clone();
        broken.png = vec![1, 2, 3];
        assert!(save_to(&root, &first, Some(&broken)).await.is_err());
        assert_eq!(
            load_from(&root, first.id).await.unwrap(),
            Some(skin.clone())
        );
        let mut online = first.clone();
        online.kind = oneclient_auth::AccountKind::Microsoft;
        assert!(save_to(&root, &online, Some(&skin)).await.is_err());
        assert_eq!(load_from(&root, first.id).await.unwrap(), Some(skin));
        save_to(&root, &first, None).await.unwrap();
        assert_eq!(load_from(&root, first.id).await.unwrap(), None);
        save_to(&root, &first, None).await.unwrap();
        tokio::fs::remove_dir_all(&root).await.unwrap();
    }

    #[tokio::test]
    async fn exports_username_scoped_skins_and_explicit_models_without_replacing_config() {
        let root = std::env::temp_dir().join(format!("oneclient-skin-export-{}", Uuid::new_v4()));
        let account = oneclient_auth::offline_account("SkinTest".into());
        let mut skin = sample(64, 64);
        let csl = root.join("CustomSkinLoader");
        tokio::fs::create_dir_all(&csl).await.unwrap();
        tokio::fs::write(csl.join("CustomSkinLoader.json"), "existing config")
            .await
            .unwrap();
        for slim in [false, true] {
            skin.slim = slim;
            export(&root, &account, Some(&skin)).await.unwrap();
            let profile: serde_json::Value =
                polyio::read_json(csl.join("OneClientOffline/SkinTest.json"))
                    .await
                    .unwrap();
            let hash = profile["textures"][if slim { "slim" } else { "default" }]
                .as_str()
                .unwrap();
            assert_eq!(
                tokio::fs::read(csl.join("OneClientOffline/textures").join(hash))
                    .await
                    .unwrap(),
                skin.png
            );
            assert!(!csl.join("OneClientOffline/OtherPlayer.json").exists());
            let provider: serde_json::Value =
                polyio::read_json(csl.join("ExtraList/OneClientOffline.json"))
                    .await
                    .unwrap();
            assert_eq!(provider["root"], "OneClientOffline/");
            assert_eq!(
                tokio::fs::read_to_string(csl.join("CustomSkinLoader.json"))
                    .await
                    .unwrap(),
                "existing config"
            );
        }
        export(&root, &account, None).await.unwrap();
        assert!(!csl.join("OneClientOffline/SkinTest.json").exists());
        export(&root, &account, Some(&skin)).await.unwrap();
        let mut online = account.clone();
        online.kind = oneclient_auth::AccountKind::Microsoft;
        export(&root, &online, Some(&skin)).await.unwrap();
        assert!(!csl.join("OneClientOffline/SkinTest.json").exists());
        let mut invalid = account;
        invalid.username = "../escape".into();
        assert!(export(&root, &invalid, Some(&skin)).await.is_err());
        tokio::fs::remove_dir_all(&root).await.unwrap();
    }
}
