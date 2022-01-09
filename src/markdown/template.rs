use uuid::Uuid;

pub fn template() -> String {
    let uuid = Uuid::new_v4().to_string();
    let template = format!(
        "---
id: {}
title:
description:
category: misc
lang: ja
---

## TL;DR
",
        uuid
    );

    template
}
