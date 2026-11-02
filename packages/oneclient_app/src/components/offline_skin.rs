use bytes::Bytes;
use freya::prelude::*;
use freya::query::{QueriesStorage, Query, QueryCapability, UseQuery, use_query};
use freya::router::RouterContext;
use oneclient_auth::MinecraftAccount;
use oneclient_core::LauncherError;
use oneclient_core::local_skin::{self, LocalSkin};
use uuid::Uuid;

use super::{Button, PlayerModel, Segment, SegmentedControl};
use crate::i18n::t;
use crate::theme::colors;

#[derive(Clone, Copy, PartialEq)]
pub struct SkinDraft {
    pub skin: State<Option<LocalSkin>>,
    pub slim: State<bool>,
    pub busy: State<bool>,
    pub error: State<Option<String>>,
}

pub fn use_skin_draft() -> SkinDraft {
    SkinDraft {
        skin: use_state(|| None),
        slim: use_state(|| false),
        busy: use_state(|| false),
        error: use_state(|| None),
    }
}

impl SkinDraft {
    pub fn selection(&self) -> Option<LocalSkin> {
        self.skin.peek().clone().map(|mut skin| {
            skin.slim = *self.slim.peek();
            skin
        })
    }

    pub fn clear(mut self) {
        self.skin.set(None);
        self.slim.set(false);
        self.error.set(None);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct LocalSkinQuery;

impl QueryCapability for LocalSkinQuery {
    type Ok = Option<LocalSkin>;
    type Err = LauncherError;
    type Keys = Uuid;

    async fn run(&self, id: &Uuid) -> Result<Self::Ok, Self::Err> {
        if id.is_nil() {
            return Ok(None);
        }
        // A leftover file must never override a Microsoft identity.
        let account = crate::launcher::state()?.auth.get_account(*id).await;
        if !account.is_some_and(|account| account.is_offline()) {
            return Ok(None);
        }
        local_skin::load(*id).await
    }
}

pub fn use_local_skin(id: Uuid) -> UseQuery<LocalSkinQuery> {
    use_query(Query::new(id, LocalSkinQuery))
}

pub async fn invalidate_local_skin(id: Uuid) {
    QueriesStorage::<LocalSkinQuery>::invalidate_matching(id).await;
}

#[derive(PartialEq)]
pub struct OfflineSkinPicker {
    pub draft: SkinDraft,
    pub enabled: bool,
    pub uuid: String,
}

impl Component for OfflineSkinPicker {
    fn render(&self) -> impl IntoElement {
        let mut draft = self.draft;
        let enabled = self.enabled && !*draft.busy.read();
        let skin = draft.skin.read().clone();
        let slim = *draft.slim.read();
        let preview = Some((
            skin.as_ref().map_or_else(
                || {
                    crate::AppAssets::get_bytes(if slim { "alex.png" } else { "steve.png" })
                        .unwrap_or_default()
                },
                |skin| Bytes::from(skin.png.clone()),
            ),
            slim,
        ));
        rect()
            .vertical()
            .width(Size::fill())
            .spacing(8.)
            .child(
                label()
                    .text(t("oneclient.offline_skin.title"))
                    .font_size(12.),
            )
            .child(
                rect()
                    .horizontal()
                    .width(Size::fill())
                    .content(Content::Flex)
                    .spacing(12.)
                    .child(
                        PlayerModel::new(self.uuid.clone())
                            .skin(preview)
                            .width(Size::px(90.))
                            .height(Size::px(140.)),
                    )
                    .child(
                        rect()
                            .vertical()
                            .width(Size::flex(1.))
                            .spacing(8.)
                            .child(
                                Button::new()
                                    .secondary()
                                    .enabled(enabled)
                                    .text(if *draft.busy.read() {
                                        t("oneclient.offline_skin.reading")
                                    } else {
                                        t("oneclient.offline_skin.choose")
                                    })
                                    .on_press(move |_| {
                                        draft.busy.set(true);
                                        draft.error.set(None);
                                        spawn(async move {
                                            if let Some(file) = rfd::AsyncFileDialog::new()
                                                .set_title(t("oneclient.offline_skin.choose"))
                                                .add_filter("PNG", &["png"])
                                                .pick_file()
                                                .await
                                            {
                                                match LocalSkin::from_file(
                                                    file.path(),
                                                    *draft.slim.peek(),
                                                )
                                                .await
                                                {
                                                    Ok(skin) => draft.skin.set(Some(skin)),
                                                    Err(err) => {
                                                        draft.error.set(Some(err.to_string()))
                                                    }
                                                }
                                            }
                                            draft.busy.set(false);
                                        });
                                    }),
                            )
                            .child(
                                label()
                                    .text(skin.as_ref().map_or_else(
                                        || t("oneclient.offline_skin.none").to_string(),
                                        |skin| skin.file_name.clone(),
                                    ))
                                    .width(Size::fill())
                                    .font_size(11.)
                                    .color(colors::fg_secondary())
                                    .max_lines(2),
                            )
                            .child(
                                SegmentedControl::new(draft.slim)
                                    .disabled(!enabled)
                                    .segment(
                                        Segment::new(false)
                                            .label(t("oneclient.offline_skin.normal")),
                                    )
                                    .segment(
                                        Segment::new(true).label(t("oneclient.offline_skin.slim")),
                                    ),
                            )
                            .maybe_child(skin.is_some().then(|| {
                                Button::new()
                                    .ghost()
                                    .enabled(enabled)
                                    .text(t("oneclient.offline_skin.reset"))
                                    .on_press(move |_| draft.clear())
                                    .into_element()
                            })),
                    ),
            )
            .child(
                label()
                    .text(t("oneclient.offline_skin.hint"))
                    .width(Size::fill())
                    .font_size(11.)
                    .color(colors::fg_secondary()),
            )
            .maybe_child(draft.error.read().clone().map(|err| {
                label()
                    .text(err)
                    .width(Size::fill())
                    .font_size(12.)
                    .color(colors::danger())
                    .into_element()
            }))
            .into_element()
    }
}

pub struct OfflineSkinEditor {
    pub account: MinecraftAccount,
}

impl PartialEq for OfflineSkinEditor {
    fn eq(&self, other: &Self) -> bool {
        self.account.id == other.account.id && self.account.kind == other.account.kind
    }
}

impl Component for OfflineSkinEditor {
    fn render_key(&self) -> DiffKey {
        DiffKey::from(&self.account.id)
    }

    fn render(&self) -> impl IntoElement {
        let mut draft = use_skin_draft();
        let mut initialized = use_state(|| false);
        let mut saving = use_state(|| false);
        let mut saved = use_state(|| None::<LocalSkin>);
        let mut message = use_state(|| None::<String>);
        let account = self.account.clone();
        let username = account.username.clone();
        let id = account.id;
        use_hook(move || {
            spawn(async move {
                match local_skin::load(id).await {
                    Ok(skin) => {
                        draft.slim.set(skin.as_ref().is_some_and(|skin| skin.slim));
                        draft.skin.set(skin.clone());
                        saved.set(skin);
                        initialized.set(true);
                    }
                    Err(err) => {
                        draft.error.set(Some(err.to_string()));
                        initialized.set(true);
                    }
                }
            });
        });
        let changed = {
            let selection = draft.skin.read().clone().map(|mut skin| {
                skin.slim = *draft.slim.read();
                skin
            });
            selection != *saved.read()
        };
        rect()
            .vertical()
            .width(Size::fill())
            .spacing(8.)
            .child(OfflineSkinPicker {
                draft,
                enabled: *initialized.read() && !*saving.read(),
                uuid: id.to_string(),
            })
            .child(
                Button::new()
                    .primary()
                    .enabled(
                        *initialized.read() && changed && !*saving.read() && !*draft.busy.read(),
                    )
                    .text(t("oneclient.offline_skin.save"))
                    .on_press(move |_| {
                        saving.set(true);
                        message.set(None);
                        draft.error.set(None);
                        let account = account.clone();
                        let selection = draft.selection();
                        spawn(async move {
                            match local_skin::save(&account, selection.as_ref()).await {
                                Ok(()) => {
                                    saved.set(selection);
                                    invalidate_local_skin(id).await;
                                    message.set(Some(t("oneclient.offline_skin.saved").into()));
                                }
                                Err(err) => draft.error.set(Some(err.to_string())),
                            }
                            saving.set(false);
                        });
                    }),
            )
            .maybe_child(message.read().clone().filter(|_| !changed).map(|text| {
                label()
                    .text(text)
                    .font_size(12.)
                    .color(colors::fg_secondary())
                    .into_element()
            }))
            .child(OfflineSkinHelp { draft, username })
            .into_element()
    }
}

#[derive(PartialEq)]
struct OfflineSkinHelp {
    draft: SkinDraft,
    username: String,
}

impl Component for OfflineSkinHelp {
    fn render(&self) -> impl IntoElement {
        let draft = self.draft;
        let mut expanded = use_state(|| false);
        let mut exporting = use_state(|| false);
        let mut result = use_state(|| None::<String>);
        let clusters =
            crate::hooks::settled_or_loading(&crate::hooks::use_clusters()).unwrap_or_default();
        let selection = draft.selection();
        let filename = format!(
            "{}-{}.png",
            self.username,
            if selection.as_ref().is_some_and(|skin| skin.slim) {
                "slim"
            } else {
                "normal"
            }
        );
        rect()
            .vertical()
            .width(Size::fill())
            .spacing(8.)
            .child(
                Button::new()
                    .secondary()
                    .text(t("oneclient.offline_skin.game_support"))
                    .on_press(move |_| {
                        let next = !*expanded.peek();
                        expanded.set(next);
                    }),
            )
            .maybe_child((*expanded.read()).then(|| {
                rect()
                    .vertical()
                    .width(Size::fill())
                    .spacing(6.)
                    .child(
                        label()
                            .text(t("oneclient.offline_skin.game_help"))
                            .width(Size::fill())
                            .font_size(12.)
                            .color(colors::fg_secondary()),
                    )
                    .children(clusters.into_iter().map(|cluster| {
                        let id = cluster.id;
                        Button::new()
                            .secondary()
                            .small()
                            .text(format!("{} ({})", cluster.name, cluster.mc_version))
                            .on_press(move |_| {
                                let _ = RouterContext::get().push(crate::Route::BrowserPackage {
                                    cluster_id: id,
                                    package_type: "mod".into(),
                                    // Official CustomSkinLoader project on Modrinth.
                                    package_id: "0:idMHQ4n2".into(),
                                    add_to_cluster: true,
                                });
                            })
                            .into_element()
                    }))
                    .into_element()
            }))
            .child(
                label()
                    .text(t("oneclient.offline_skin.multiplayer_help"))
                    .width(Size::fill())
                    .font_size(12.)
                    .color(colors::fg_secondary()),
            )
            .child(
                Button::new()
                    .secondary()
                    .enabled(selection.is_some() && !*exporting.read() && !*draft.busy.read())
                    .text(t("oneclient.offline_skin.export"))
                    .on_press(move |_| {
                        let Some(skin) = draft.selection() else {
                            return;
                        };
                        exporting.set(true);
                        result.set(None);
                        let filename = filename.clone();
                        spawn(async move {
                            if let Some(file) = rfd::AsyncFileDialog::new()
                                .set_title(t("oneclient.offline_skin.export"))
                                .set_file_name(filename)
                                .add_filter("PNG", &["png"])
                                .save_file()
                                .await
                            {
                                let saved = async {
                                    skin.validate()?;
                                    polyio::write_atomic(file.path(), skin.png.as_slice()).await?;
                                    Ok::<(), LauncherError>(())
                                }
                                .await;
                                result.set(Some(match saved {
                                    Ok(()) => t("oneclient.offline_skin.exported").into(),
                                    Err(err) => err.to_string(),
                                }));
                            }
                            exporting.set(false);
                        });
                    }),
            )
            .child(
                Button::new()
                    .ghost()
                    .small()
                    .text(t("oneclient.offline_skin.server_guide"))
                    .on_press(|_| {
                        crate::platform::open_url(
                            "https://skinsrestorer.net/docs/features/skin-upload",
                        )
                    }),
            )
            .maybe_child(result.read().clone().map(|text| {
                label()
                    .text(text)
                    .width(Size::fill())
                    .font_size(12.)
                    .color(colors::fg_secondary())
                    .into_element()
            }))
            .into_element()
    }
}
