//! Check first-frame editor and screen settings using the real client configuration loader.

use super::*;
use clap::Parser;
use crossterm::event::KeyCode;
use pretty_assertions::assert_eq;

#[tokio::test]
async fn selected_profile_controls_submit_before_the_first_frame() -> anyhow::Result<()> {
    let home = tempfile::tempdir()?;
    std::fs::write(
        home.path().join("config.toml"),
        "disable_paste_burst = false\n",
    )?;
    let selected = AbsolutePathBuf::from_absolute_path(home.path().join("work.config.toml"))?;
    let cli = Cli::try_parse_from(["codex"])?;
    for (submit, expected_bindings) in [
        ("[]", Vec::new()),
        ("\"f12\"", vec![crate::key_hint::plain(KeyCode::F(12))]),
    ] {
        std::fs::write(
            &selected,
            format!("disable_paste_burst = true\n[tui.keymap.composer]\nsubmit = {submit}\n"),
        )?;
        let presentation = load(
            &cli,
            home.path(),
            LoaderOverrides {
                user_config_path: Some(selected.clone()),
                user_config_profile: Some("work".parse()?),
                ignore_project_config: true,
                ..LoaderOverrides::without_managed_config_for_tests()
            },
            Vec::new(),
            /*config_cwd*/ None,
        )
        .await?;
        assert_eq!(
            (
                presentation.screen.keymap.composer.submit,
                presentation.screen.disable_paste_burst,
            ),
            (expected_bindings, true),
        );
    }
    Ok(())
}

#[tokio::test]
async fn first_frame_respects_screen_and_status_line_overrides() -> anyhow::Result<()> {
    let home = tempfile::tempdir()?;
    for (config, args, expected) in [
        ("", vec!["codex"], (true, true, true)),
        ("[tui]\n", vec!["codex"], (true, true, true)),
        ("", vec!["codex", "--no-alt-screen"], (false, false, true)),
        (
            "[tui]\nfullscreen_transcript = true\n",
            vec!["codex"],
            (true, true, true),
        ),
        (
            "[tui]\nfullscreen_transcript = true\n",
            vec!["codex", "--no-alt-screen"],
            (false, false, true),
        ),
        (
            "[tui]\nfullscreen_transcript = false\n",
            vec!["codex"],
            (true, false, true),
        ),
        (
            "[tui]\nfullscreen_transcript = true\nalternate_screen = \"never\"\nstatus_line = []\n",
            vec!["codex"],
            (false, false, false),
        ),
    ] {
        std::fs::write(home.path().join("config.toml"), config)?;
        let cli = Cli::try_parse_from(args)?;
        let presentation = load(
            &cli,
            home.path(),
            LoaderOverrides {
                ignore_project_config: true,
                ..LoaderOverrides::without_managed_config_for_tests()
            },
            Vec::new(),
            /*config_cwd*/ None,
        )
        .await?;
        assert_eq!(
            (
                presentation.screen.use_alt_screen,
                presentation.screen.transcript_mode.is_owned(),
                presentation.screen.status_line_enabled
            ),
            expected,
        );
    }
    Ok(())
}

#[tokio::test]
async fn greeting_preference_applies_before_first_paint_and_accepts_cli_overrides()
-> anyhow::Result<()> {
    let home = tempfile::tempdir()?;
    let cli = Cli::try_parse_from(["codex"])?;
    for (settings, cli_override, expected) in [
        ("", None, true),
        ("[tui]\n", None, true),
        ("[tui]\nshow_greeting = false\n", None, false),
        ("[tui]\nshow_greeting = false\n", Some(true), true),
        ("[tui]\nshow_greeting = true\n", Some(false), false),
        (
            "[tui]\nanimations = false\nshow_tooltips = false\n",
            None,
            true,
        ),
    ] {
        std::fs::write(home.path().join("config.toml"), settings)?;
        let presentation = load(
            &cli,
            home.path(),
            LoaderOverrides {
                ignore_project_config: true,
                ..LoaderOverrides::without_managed_config_for_tests()
            },
            cli_override
                .map(|value| ("tui.show_greeting".into(), TomlValue::Boolean(value)))
                .into_iter()
                .collect(),
            /*config_cwd*/ None,
        )
        .await?;
        assert_eq!(presentation.screen.show_greeting, expected);
    }
    Ok(())
}
