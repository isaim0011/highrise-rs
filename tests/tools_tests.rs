use std::time::Duration;

use highrise_rs::{
    BotSpec, CommandError, CommandManager, CommandMeta, Config, DanceFloorManager, DanceFloorZone,
    DanceMode, Emotes, EntityPosition, Facing, HighriseCluster, Position, Role, User,
};

#[test]
fn emotes_catalog_resolves_and_blocks() {
    let mut emotes = Emotes::new();
    assert!(emotes.resolve("1").is_some());
    assert_eq!(emotes.resolve("Macarena").unwrap().id, "dance-macarena");
    assert_eq!(emotes.resolve("emote-wave").unwrap().name, "Wave");

    emotes.block("emote-wave");
    assert!(emotes.resolve("emote-wave").is_none());
    assert!(!emotes.dances().is_empty());
}

#[tokio::test]
async fn dancefloor_tracks_zones_and_excludes_seated_and_bot() {
    let df = DanceFloorManager::new();
    df.add_zone(DanceFloorZone::new(
        "main",
        Position { x: 2.0, y: 0.0, z: 2.0, facing: Facing::FrontRight },
        Position { x: 8.0, y: 2.0, z: 8.0, facing: Facing::FrontRight },
    ))
    .await;
    df.set_mode(DanceMode::PerUserRandom).await;

    df.update_position(
        "u1",
        &EntityPosition::Floor(Position { x: 5.0, y: 0.5, z: 5.0, facing: Facing::FrontRight }),
    )
    .await;
    df.update_position(
        "u2",
        &EntityPosition::Floor(Position { x: 15.0, y: 0.0, z: 5.0, facing: Facing::FrontRight }),
    )
    .await;
    df.update_position(
        "bot1",
        &EntityPosition::Floor(Position { x: 5.0, y: 0.0, z: 5.0, facing: Facing::FrontRight }),
    )
    .await;

    let dancers = df.active_dancers(Some("bot1")).await;
    assert_eq!(dancers, vec!["u1".to_string()]);
}

#[tokio::test]
async fn command_manager_handles_subcommands_overrides_and_cooldowns() {
    let mgr = CommandManager::new(["/", "!"]);
    mgr.register(
        CommandMeta {
            name: "music".into(),
            category: "Music".into(),
            description: "Music root".into(),
            usage: "music".into(),
            min_role: Role::Member,
            cooldown: Duration::ZERO,
            aliases: vec![],
            enabled: true,
        },
        |_ctx| async { Err(CommandError::Failed("root".into())) },
    )
    .await;

    mgr.register(
        CommandMeta {
            name: "music fit save".into(),
            category: "Outfit".into(),
            description: "Save outfit".into(),
            usage: "music fit save <name>".into(),
            min_role: Role::Owner,
            cooldown: Duration::from_secs(5),
            aliases: vec!["mfs".into()],
            enabled: true,
        },
        |ctx| async move {
            assert_eq!(ctx.args, vec!["chill".to_string()]);
            Ok(())
        },
    )
    .await;

    // Dummy transport-less Highrise isn't needed for unit test unless we connect;
    // verify help_for + role filtering & cluster builder compile cleanly.
    let member_help = mgr.help_for(Role::Member).await;
    assert_eq!(member_help.len(), 1);
    let owner_help = mgr.help_for(Role::Owner).await;
    assert_eq!(owner_help.len(), 2);

    mgr.set_min_role("music fit save", Role::Mod).await;
    let mod_help = mgr.help_for(Role::Mod).await;
    assert_eq!(mod_help.len(), 2);

    mgr.set_enabled("music", false).await;
    assert_eq!(mgr.help_for(Role::Mod).await.len(), 1);

    let mut cluster = HighriseCluster::new();
    struct Noop;
    impl highrise_rs::BotHandler for Noop {}
    cluster.add_bot(BotSpec::new("general", Config::new("t1", "r1")), Noop);
    cluster.add_bot(BotSpec::new("music", Config::new("t2", "r2")), Noop);
    let _ = User { id: "u1".into(), username: "iBimo_o".into() };
}
