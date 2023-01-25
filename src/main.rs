// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: GPL-3.0-only

#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_lossless)]

pub mod app;
pub use app::{Message, SettingsApp};

#[macro_use]
pub mod localize;

pub mod widget;

pub mod page;

use cosmic::{iced::Application, settings};
use i18n_embed::DesktopLanguageRequester;

/// # Errors
///
/// Returns error if iced fails to run the application.
pub fn main() -> cosmic::iced::Result {
    let localizer = crate::localize::localizer();
    let requested_languages = DesktopLanguageRequester::requested_languages();

    if let Err(error) = localizer.select(&requested_languages) {
        eprintln!("error while loading fluent localizations: {}", error);
    }

    settings::set_default_icon_theme("Pop");
    let mut settings = settings();
    settings.window.min_size = Some((600, 300));
    SettingsApp::run(settings)
}