use super::*;
use pretty_assertions::assert_eq;

#[tokio::test]
async fn greeting_setting_controls_provisional_and_configured_headers_without_hiding_help() {
    for show_greeting in [true, false] {
        let (tx_raw, mut events) = unbounded_channel::<AppEvent>();
        let (op_tx, _ops) = unbounded_channel::<Op>();
        let (codex_home, mut config) = test_config().await;
        config.show_greeting = show_greeting;
        config.show_tooltips = false;
        let model = "gpt-5.2".to_string();
        let cwd = config.cwd.clone();
        let common = ChatWidgetInit {
            requires_openai_auth: config.model_provider.requires_openai_auth,
            local_settings: crate::local_settings::LocalSettings::from(&config),
            frame_requester: FrameRequester::test_dummy(),
            app_event_tx: AppEventSender::new(tx_raw),
            config: config.clone(),
            workspace_command_runner: None,
            initial_user_message: None,
            enhanced_keys_supported: false,
            has_chatgpt_account: false,
            has_codex_backend_auth: false,
            model_catalog: test_model_catalog(&config),
            feedback: codex_feedback::CodexFeedback::new(),
            is_first_run: true,
            status_account_display: None,
            initial_plan_type: None,
            model: Some(model.clone()),
            startup_tooltip_override: None,
            status_line_invalid_items_warned: Arc::new(AtomicBool::new(false)),
            terminal_title_invalid_items_warned: Arc::new(AtomicBool::new(false)),
            session_telemetry: test_session_telemetry(&config, &model),
        };
        let mut chat = ChatWidget::new_with_op_target(common, CodexOpTarget::Direct(op_tx));
        chat.test_codex_home = Some(codex_home);
        // The constructor binds the header before a fresh thread chooses its phrase.
        chat.empty_state_animation.borrow_mut().start_fresh();
        let greeting = *chat
            .empty_state_animation
            .borrow()
            .greeting
            .get()
            .expect("fresh greeting");
        let provisional = chat
            .transcript
            .active_cell
            .as_ref()
            .expect("provisional header")
            .raw_lines()
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("\n");
        assert_eq!(provisional.contains(greeting.phrase), show_greeting);
        assert!(provisional.contains("OpenAI Codex"));

        chat.handle_thread_session(crate::session_state::ThreadSessionState {
            windows_sandbox_host: crate::app::WindowsSandboxHost::Local,
            thread_id: ThreadId::new(),
            forked_from_id: None,
            fork_parent_title: None,
            thread_name: None,
            model,
            model_provider_id: "openai".to_string(),
            service_tier: None,
            approval_policy: AskForApproval::Never,
            approvals_reviewer: ApprovalsReviewer::User,
            permission_profile: PermissionProfile::read_only(),
            active_permission_profile: None,
            cwd: cwd.clone(),
            runtime_workspace_roots: vec![cwd],
            instruction_source_paths: Vec::new(),
            reasoning_effort: None,
            collaboration_mode: None,
            personality: None,
            message_history: None,
            network_proxy: None,
            rollout_path: None,
        });
        let rendered = drain_insert_history_with(&mut events, HistoryCell::raw_lines)
            .into_iter()
            .flatten()
            .map(|line| line.to_string())
            .collect::<Vec<_>>()
            .join("\n");
        assert_eq!(rendered.contains(greeting.phrase), show_greeting);
        assert!(rendered.contains("OpenAI Codex"));
        assert!(rendered.contains("To get started, describe a task"));
    }
}
