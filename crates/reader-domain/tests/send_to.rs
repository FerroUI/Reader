use reader_domain::{
    validate_send_to, Action, AppState, SendToDestination, MAX_SEND_TO_DESTINATIONS,
};

fn destination() -> SendToDestination {
    SendToDestination {
        id: 1,
        name: "Notebook".into(),
        url_template:
            "https://save.example/add?url=${url}&title=${title}&source=${source}&again=${title}"
                .into(),
        enabled: true,
    }
}

#[test]
fn placeholders_encode_values_once_and_source_is_a_title() {
    let link = destination();
    let article = "https://news.example/article?a=1&b=two#part";
    let title = "Zażółć & 日本語 / ${source} + #?";
    let source = "My renamed Rust & Zig feed";
    let result = link.resolve(article, title, source).unwrap();
    let parsed = url::Url::parse(&result).unwrap();
    let values: std::collections::BTreeMap<_, _> = parsed.query_pairs().into_owned().collect();
    assert_eq!(values["url"], article);
    assert_eq!(values["title"], title);
    assert_eq!(values["again"], title);
    assert_eq!(values["source"], source);
    assert!(result.contains("%20"));
    assert!(!result.contains("${"));
    assert_eq!(parsed.host_str(), Some("save.example"));
}

#[test]
fn unsafe_or_mistyped_templates_are_rejected_before_saving() {
    for template in [
        "javascript:alert('${url}')",
        "data:text/html,${url}",
        "http://save.example/?u=${url}",
        "https://user:secret@save.example/?u=${url}",
        "https://${source}.example/?u=${url}",
        "https://save.example/",
        "https://save.example/?u={url}",
        "https://save.example/?u=${unknown}",
        "https://save.example/?u=${url",
        "https://save.example/?u=${url}\n&title=${title}",
    ] {
        let mut items = vec![SendToDestination {
            url_template: template.into(),
            ..destination()
        }];
        assert!(validate_send_to(&mut items).is_err(), "{template}");
    }
    let mut valid = vec![destination()];
    valid[0].name = "  My notebook  ".into();
    validate_send_to(&mut valid).unwrap();
    assert_eq!(valid[0].name, "My notebook");
}

#[test]
fn destination_identity_names_and_limits_are_validated() {
    let mut invalid = vec![destination(), destination()];
    assert!(validate_send_to(&mut invalid).is_err());
    invalid[1].id = 2;
    invalid[1].name = "notebook".into();
    assert!(validate_send_to(&mut invalid).is_err());
    for name in [
        "".into(),
        "\n".into(),
        "a".repeat(81),
        "Name\u{0000}".into(),
    ] {
        assert!(validate_send_to(&mut [SendToDestination {
            name,
            ..destination()
        }])
        .is_err());
    }
    let mut many: Vec<_> = (1..=MAX_SEND_TO_DESTINATIONS + 1)
        .map(|id| SendToDestination {
            id: id as u64,
            name: format!("Destination {id}"),
            ..destination()
        })
        .collect();
    assert!(validate_send_to(&mut many).is_err());
    many.pop();
    validate_send_to(&mut many).unwrap();
}

#[test]
fn old_backups_default_to_no_services_and_round_trip_preserves_custom_links() {
    let mut state = AppState::new();
    let mut legacy = serde_json::to_value(&state).unwrap();
    legacy["settings"]
        .as_object_mut()
        .unwrap()
        .remove("send_to");
    assert!(AppState::from_json(&legacy.to_string())
        .unwrap()
        .settings
        .send_to
        .is_empty());
    state.settings.send_to.push(destination());
    let restored = AppState::from_json(&state.to_json().unwrap()).unwrap();
    assert_eq!(restored.settings.send_to, state.settings.send_to);
}

#[test]
fn invalid_settings_do_not_replace_the_saved_document() {
    let mut state = AppState::new();
    state.settings.send_to.push(destination());
    let saved = state.to_json().unwrap();
    let mut settings = state.settings.clone();
    settings.send_to[0].url_template = "https://bad.example/?url=${secret}".into();
    assert!(state.apply(Action::SetSettings(settings), 1000).is_err());
    assert_eq!(state.to_json().unwrap(), saved);
}

#[test]
fn disabled_missing_and_excessive_article_values_never_produce_an_address() {
    assert!(SendToDestination {
        enabled: false,
        ..destination()
    }
    .resolve("https://news.example/", "Title", "Source")
    .is_err());
    assert!(destination().resolve("", "Title", "Source").is_err());
    assert!(destination()
        .resolve("https://news.example/", &"x".repeat(33000), "Source")
        .is_err());
}
