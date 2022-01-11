use uuid::Uuid;

pub fn template() -> String {
    let uuid = Uuid::new_v4().to_string();
    let template = format!(
        "---
uuid: {}
title:
description:
lang: ja
category: misc
---

## TL;DR
",
        uuid
    );

    template
}
