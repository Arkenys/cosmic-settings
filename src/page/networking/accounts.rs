// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: GPL-3.0-only

use crate::page;

pub fn page() -> page::Meta {
    page::Meta::new("online-accounts", "goa-panel-symbolic")
        .title(fl!("online-accounts"))
        .description(fl!("online-accounts", "desc"))
}
