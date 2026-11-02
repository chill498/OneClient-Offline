use freya::prelude::*;
use freya::query::MutationStateData;
use freya::router::RouterContext;
use oneclient_auth::MinecraftAccount;

use crate::components::{
    Avatar, Button, Icon, IconType, OfflineSkinEditor, OfflineSkinPicker, ScrollArea, TextInput,
    use_microsoft_login, use_skin_draft,
};
use crate::hooks::{
    AddOfflineAccountKeys, mutation_is_running, try_default_account, use_add_offline_account,
    use_current_account,
};
use crate::i18n::t;
use crate::routes::Route;
use crate::theme::colors;
use crate::view::onboarding::{
    onboarding_illustration, onboarding_nav_action, onboarding_page, step_heading,
};

#[derive(PartialEq)]
pub struct OnboardingAccount;

impl Component for OnboardingAccount {
    fn render(&self) -> impl IntoElement {
        let account_query = use_current_account();
        let msa = use_microsoft_login();
        let add_offline = use_add_offline_account();
        let username = use_state(String::new);
        let skin_draft = use_skin_draft();
        let mut submitting = use_state(|| false);
        let pending = *submitting.read() || mutation_is_running(&add_offline) || msa.pending;
        let offline_error = match &*add_offline.read().state() {
            MutationStateData::Settled { res: Err(err), .. } => Some(err.to_string()),
            _ => None,
        };
        let name = username.read().trim().to_string();
        let validation = oneclient_auth::validate_offline_username(&name);
        let validation_error = if name.is_empty() {
            None
        } else {
            validation.as_ref().err().map(ToString::to_string)
        };
        let name_valid = validation.is_ok();
        let account = try_default_account(&account_query);
        let has_account = account.is_some();
        let picker_busy = *skin_draft.busy.read();

        // Next owns creation and navigation. Typing a valid username must not
        // require finding a second account-creation button below the skin form.
        let continue_account = EventHandler::new(move |()| {
            if pending || *submitting.peek() || *skin_draft.busy.peek() {
                return;
            }
            if has_account {
                let _ = RouterContext::get().replace(Route::OnboardingBundles {});
                return;
            }
            let name = username.peek().trim().to_string();
            if oneclient_auth::validate_offline_username(&name).is_err() {
                return;
            }
            submitting.set(true);
            let skin = skin_draft.selection();
            spawn(async move {
                let result = add_offline
                    .mutate_async(AddOfflineAccountKeys {
                        username: name,
                        skin,
                    })
                    .await;
                let succeeded = matches!(
                    &*result.state(),
                    MutationStateData::Settled { res: Ok(_), .. }
                );
                submitting.set(false);
                if succeeded {
                    let _ = RouterContext::get().replace(Route::OnboardingBundles {});
                }
            });
        });

        let submit_account = continue_account.clone();
        let content = rect().vertical().width(Size::fill()).spacing(20.)
            .child(step_heading(t("oneclient.onboarding.account.title"), t("oneclient.onboarding.account.subtitle")))
            .child(match &account {
                Some(account) => rect().vertical().width(Size::fill()).spacing(16.)
                    .child(account_preview(account))
                    .maybe_child(account.is_offline().then(|| OfflineSkinEditor { account: account.clone() }.into_element()))
                    .into_element(),
                None => {
                    let start = msa.clone();
                    rect().vertical().width(Size::fill()).spacing(16.)
                        .child(rect().vertical().width(Size::fill()).spacing(8.)
                            .child(label().text(t("oneclient.settings.accounts.offline.username")).font_size(12.))
                            .child(TextInput::new(username).enabled(!pending)
                                .on_submit(move |_| submit_account.call(()))
                                .placeholder(t("oneclient.settings.accounts.offline.username_placeholder")))
                            .maybe_child(validation_error.map(|message| label().text(message).width(Size::fill()).font_size(12.).color(colors::danger()).into_element()))
                            .child(OfflineSkinPicker { draft: skin_draft, enabled: !pending, uuid: oneclient_auth::offline_uuid(&name).to_string() })
                            .child(label().text(t("oneclient.offline_skin.skip")).width(Size::fill()).font_size(12.).color(colors::fg_secondary()))
                            .child(label().text(t("oneclient.settings.accounts.offline.description")).width(Size::fill()).font_size(12.).color(colors::fg_secondary()))
                            .maybe_child(offline_error.map(|message| label().text(message).width(Size::fill()).font_size(12.).color(colors::danger()).into_element())))
                        .child(sign_in_card(msa.pending, !pending && !picker_busy, msa.error.clone(), move |_| start.start()))
                        .into_element()
                }
            });
        let page = onboarding_page(
            onboarding_illustration(IconType::OnboardingAccount),
            ScrollArea::new()
                .width(Size::fill())
                .height(Size::fill())
                .child(content),
            onboarding_nav_action(
                Some(Route::OnboardingLanguage {}),
                if pending {
                    t("oneclient.offline_skin.continuing")
                } else {
                    t("oneclient.common.next")
                },
                (has_account || name_valid) && !pending && !picker_busy,
                move |()| continue_account.call(()),
            ),
        );
        rect()
            .width(Size::fill())
            .height(Size::fill())
            .child(page)
            .maybe_child(msa.popup())
    }
}

fn account_preview(account: &MinecraftAccount) -> impl IntoElement {
    rect()
        .horizontal()
        .width(Size::fill())
        .spacing(24.)
        .child(
            rect()
                .horizontal()
                .spacing(12.)
                .cross_align(Alignment::Center)
                .child(
                    Avatar::new(account.id.to_string())
                        .width(Size::px(48.))
                        .height(Size::px(48.)),
                )
                .child(
                    rect()
                        .vertical()
                        .spacing(4.)
                        .child(
                            label()
                                .text(if account.is_offline() {
                                    t("oneclient.settings.accounts.kind.offline")
                                } else {
                                    "Microsoft"
                                })
                                .font_size(12.)
                                .color(colors::fg_secondary()),
                        )
                        .child(
                            label()
                                .text(account.username.clone())
                                .font_size(16.)
                                .font_weight(FontWeight::SEMI_BOLD)
                                .color(colors::fg_primary()),
                        )
                        .child(
                            label()
                                .text(account.id.to_string())
                                .font_size(12.)
                                .color(colors::fg_secondary()),
                        ),
                ),
        )
        .into_element()
}

fn sign_in_card(
    pending: bool,
    enabled: bool,
    error: Option<String>,
    on_add: impl FnMut(Event<PressEventData>) + 'static,
) -> impl IntoElement {
    rect()
        .vertical()
        .spacing(12.)
        .cross_align(Alignment::Start)
        .child(
            Button::new()
                .secondary()
                .large()
                .enabled(enabled && !pending)
                .on_press(on_add)
                .child(Icon::new(IconType::Globe01).size(16.))
                .text(if pending {
                    t("oneclient.onboarding.account.signing_in")
                } else {
                    t("oneclient.onboarding.account.add")
                }),
        )
        .maybe_child(error.map(|message| {
            rect()
                .horizontal()
                .cross_align(Alignment::Center)
                .spacing(6.)
                .child(
                    Icon::new(IconType::AlertTriangle)
                        .size(13.)
                        .color(colors::danger()),
                )
                .child(label().text(message).font_size(12.).color(colors::danger()))
                .into_element()
        }))
        .into_element()
}
