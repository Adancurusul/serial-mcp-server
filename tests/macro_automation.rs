use clap::Parser;
use serial_mcp_server::automation::{
    plan_target, validate_pack, MacroExecutor, MacroPack, MacroTarget, SimulatedMacroTransport,
};
use serial_mcp_server::config::{Args, Command, MacroCommand};
use serial_mcp_server::tools::{
    MacroLoadArgs, MacroPlanArgs, MacroRunArgs, MacroRunInput, MacroTargetArgs,
};
use serial_mcp_server::{Config, SerialHandler};
use std::process::Command as CliCommand;

fn pack_json() -> &'static str {
    r#"{
        "schema_version": "0.3",
        "name": "ping-pack",
        "description": "serial handshake",
        "macros": [
            {
                "name": "ping",
                "steps": [
                    { "type": "send", "data": "PING\n", "encoding": "utf8" },
                    { "type": "delay", "ms": 5 },
                    { "type": "expect", "op": "contains", "data": "PONG", "encoding": "utf8", "timeout_ms": 1000 }
                ]
            },
            {
                "name": "status",
                "steps": [
                    { "type": "send", "data": "STATUS\n", "encoding": "utf8" },
                    { "type": "expect", "op": "equals", "data": "OK", "encoding": "utf8", "timeout_ms": 1000, "idle_ms": 10, "trim": true }
                ]
            }
        ],
        "assemblies": [
            {
                "name": "boot-check",
                "steps": [
                    { "type": "macro", "name": "ping" },
                    { "type": "macro", "name": "status" }
                ]
            }
        ]
    }"#
}

fn parse_pack(input: &str) -> MacroPack {
    serde_json::from_str(input).expect("pack JSON should deserialize")
}

#[test]
fn macro_dsl_valid_pack_lists_targets() {
    let pack = parse_pack(pack_json());
    let inventory = validate_pack(&pack).expect("pack should validate");

    assert_eq!(inventory.schema_version, "0.3");
    assert_eq!(inventory.name, "ping-pack");
    assert_eq!(inventory.macros, vec!["ping", "status"]);
    assert_eq!(inventory.assemblies, vec!["boot-check"]);
}

#[test]
fn macro_dsl_rejects_invalid_packs_before_execution() {
    let duplicate_names = r#"{
        "schema_version": "0.3",
        "name": "bad-pack",
        "macros": [
            { "name": "ping", "steps": [{ "type": "send", "data": "A" }] },
            { "name": "ping", "steps": [{ "type": "send", "data": "B" }] }
        ]
    }"#;
    let duplicate_pack = parse_pack(duplicate_names);
    assert!(validate_pack(&duplicate_pack).is_err());

    let unknown_reference = r#"{
        "schema_version": "0.3",
        "name": "bad-pack",
        "macros": [
            { "name": "ping", "steps": [{ "type": "send", "data": "A" }] }
        ],
        "assemblies": [
            { "name": "boot", "steps": [{ "type": "macro", "name": "missing" }] }
        ]
    }"#;
    let reference_pack = parse_pack(unknown_reference);
    assert!(validate_pack(&reference_pack).is_err());

    let unsupported_encoding = r#"{
        "schema_version": "0.3",
        "name": "bad-pack",
        "macros": [
            { "name": "ping", "steps": [{ "type": "send", "data": "A", "encoding": "latin1" }] }
        ]
    }"#;
    assert!(serde_json::from_str::<MacroPack>(unsupported_encoding).is_err());

    let unknown_field = r#"{
        "schema_version": "0.3",
        "name": "bad-pack",
        "extra": true,
        "macros": [
            { "name": "ping", "steps": [{ "type": "send", "data": "A" }] }
        ]
    }"#;
    assert!(serde_json::from_str::<MacroPack>(unknown_field).is_err());
}

#[test]
fn macro_planner_expands_targets_without_hardware() {
    let pack = parse_pack(pack_json());
    validate_pack(&pack).expect("pack should validate");

    let macro_plan =
        plan_target(&pack, MacroTarget::macro_named("ping")).expect("macro should plan");
    assert_eq!(macro_plan.target_name, "ping");
    assert_eq!(macro_plan.steps.len(), 3);

    let assembly_plan = plan_target(&pack, MacroTarget::assembly_named("boot-check"))
        .expect("assembly should plan");
    assert_eq!(assembly_plan.target_kind, "assembly");
    assert_eq!(assembly_plan.target_name, "boot-check");
    assert_eq!(assembly_plan.steps.len(), 5);
}

#[test]
fn macro_cli_dry_run_and_simulation_do_not_open_hardware() {
    let args = Args::parse_from([
        "serial-mcp-server",
        "macro",
        "run",
        "--file",
        "examples/macros/ping.json",
        "--macro",
        "ping",
        "--dry-run",
        "--json",
    ]);
    let Some(Command::Macro(MacroCommand::Run(run))) = args.command else {
        panic!("expected macro run command");
    };
    assert!(run.dry_run);
    assert_eq!(run.serial.port, None);
    assert_eq!(run.simulate_read, Vec::<String>::new());

    let args = Args::parse_from([
        "serial-mcp-server",
        "macro",
        "run",
        "--file",
        "examples/macros/ping.json",
        "--macro",
        "ping",
        "--simulate-read",
        "PONG",
        "--json",
    ]);
    let Some(Command::Macro(MacroCommand::Run(run))) = args.command else {
        panic!("expected macro run command");
    };
    assert_eq!(run.serial.port, None);
    assert_eq!(run.simulate_read, vec!["PONG"]);
}

#[tokio::test]
async fn macro_executor_runs_send_delay_expect_contains_equals() {
    let pack = parse_pack(pack_json());

    let ping_plan =
        plan_target(&pack, MacroTarget::macro_named("ping")).expect("macro should plan");
    let ping_transport = SimulatedMacroTransport::new(vec![b"noise PONG\r\n".to_vec()]);
    let ping_report = MacroExecutor::simulated()
        .run(ping_plan, ping_transport)
        .await
        .expect("contains plan should run");
    assert!(ping_report.success);
    assert_eq!(ping_report.writes[0].data, "PING\n");

    let status_plan =
        plan_target(&pack, MacroTarget::macro_named("status")).expect("macro should plan");
    let status_transport = SimulatedMacroTransport::new(vec![b"OK\n".to_vec()]);
    let status_report = MacroExecutor::simulated()
        .run(status_plan, status_transport)
        .await
        .expect("equals plan should run");
    assert!(status_report.success);
    assert_eq!(status_report.writes[0].data, "STATUS\n");
}

#[tokio::test]
async fn macro_mcp_registry_is_runtime_only() {
    let handler = SerialHandler::new(Config::default());
    let loaded = handler
        .macro_load_pack(MacroLoadArgs {
            pack_json: Some(pack_json().to_string()),
            path: None,
        })
        .await
        .expect("pack should load");
    assert_eq!(loaded.inventory.macros, vec!["ping", "status"]);

    let listed = handler
        .macro_list_packs(None)
        .await
        .expect("loaded pack should list");
    assert_eq!(listed.packs.len(), 1);

    let run = handler
        .macro_run_loaded(MacroRunArgs {
            pack_id: loaded.pack_id.clone(),
            target: MacroTargetArgs::macro_named("ping"),
            input: MacroRunInput::Simulation {
                reads: vec!["PONG".to_string()],
            },
        })
        .await
        .expect("loaded pack should run with simulation");
    assert!(run.success);

    handler
        .macro_unload_pack(&loaded.pack_id)
        .await
        .expect("pack should unload");
    let listed = handler
        .macro_list_packs(None)
        .await
        .expect("empty registry should list");
    assert!(listed.packs.is_empty());

    let fresh = SerialHandler::new(Config::default());
    let fresh_list = fresh
        .macro_list_packs(None)
        .await
        .expect("fresh server should list");
    assert!(fresh_list.packs.is_empty());

    let inline_plan = fresh
        .macro_plan_pack(MacroPlanArgs {
            pack_id: None,
            pack_json: Some(pack_json().to_string()),
            path: None,
            target: MacroTargetArgs::assembly_named("boot-check"),
        })
        .await
        .expect("inline pack should plan without loading");
    assert_eq!(inline_plan.target_kind, "assembly");
    assert_eq!(inline_plan.steps.len(), 5);

    let path_plan = fresh
        .macro_plan_pack(MacroPlanArgs {
            pack_id: None,
            pack_json: None,
            path: Some("examples/macros/ping.json".to_string()),
            target: MacroTargetArgs::macro_named("ping"),
        })
        .await
        .expect("path pack should plan without loading");
    assert_eq!(path_plan.target_kind, "macro");
    assert_eq!(path_plan.steps.len(), 2);
}

#[test]
fn macro_dsl_rejects_quick_scripting_and_rts_dtr() {
    for step_type in ["quick", "shell", "python", "javascript", "rts", "dtr"] {
        let pack = format!(
            r#"{{
                "schema_version": "0.3",
                "name": "bad-pack",
                "macros": [
                    {{ "name": "bad", "steps": [{{ "type": "{step_type}", "data": "A" }}] }}
                ]
            }}"#
        );
        assert!(serde_json::from_str::<MacroPack>(&pack).is_err());
    }
}

#[test]
fn macro_hardware_boundary_is_explicit() {
    let real_args = Args::parse_from([
        "serial-mcp-server",
        "macro",
        "run",
        "--file",
        "examples/macros/ping.json",
        "--macro",
        "ping",
        "--port",
        "COM19",
        "--baud",
        "115200",
        "--json",
    ]);
    let Some(Command::Macro(MacroCommand::Run(real_run))) = real_args.command else {
        panic!("expected macro run command");
    };
    assert_eq!(real_run.serial.port.as_deref(), Some("COM19"));
    assert!(!real_run.dry_run);
    assert!(real_run.simulate_read.is_empty());

    let simulation_args = Args::parse_from([
        "serial-mcp-server",
        "macro",
        "run",
        "--file",
        "examples/macros/ping.json",
        "--macro",
        "ping",
        "--simulate-read",
        "PONG",
        "--json",
    ]);
    let Some(Command::Macro(MacroCommand::Run(simulation_run))) = simulation_args.command else {
        panic!("expected macro run command");
    };
    assert_eq!(simulation_run.serial.port, None);
    assert_eq!(simulation_run.simulate_read, vec!["PONG"]);
}

#[test]
fn macro_cli_json_errors_are_structured() {
    let output = CliCommand::new(env!("CARGO_BIN_EXE_serial-mcp-server"))
        .args([
            "macro",
            "plan",
            "--file",
            "examples/macros/ping.json",
            "--json",
        ])
        .output()
        .expect("binary should run");
    assert!(!output.status.success());
    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("stdout should be JSON");
    assert_eq!(json["status"], "error");
    assert!(json["error"]
        .as_str()
        .expect("error should be text")
        .contains("Specify exactly one of --macro or --assembly"));

    let invalid_pack = tempfile::NamedTempFile::new().expect("temp file should create");
    std::fs::write(
        invalid_pack.path(),
        r#"{
            "schema_version": "0.3",
            "name": "bad-pack",
            "macros": [
                { "name": "ping", "steps": [{ "type": "send", "data": "A" }] },
                { "name": "ping", "steps": [{ "type": "send", "data": "B" }] }
            ]
        }"#,
    )
    .expect("temp file should write");

    let output = CliCommand::new(env!("CARGO_BIN_EXE_serial-mcp-server"))
        .args([
            "macro",
            "validate",
            "--file",
            invalid_pack
                .path()
                .to_str()
                .expect("temp path should be utf8"),
            "--json",
        ])
        .output()
        .expect("binary should run");
    assert!(!output.status.success());
    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("stdout should be JSON");
    assert_eq!(json["status"], "error");
    assert_eq!(json["field"], "macros[1].name");

    let output = CliCommand::new(env!("CARGO_BIN_EXE_serial-mcp-server"))
        .args([
            "macro",
            "plan",
            "--file",
            "examples/macros/ping.json",
            "--macro",
            "ping",
            "--assembly",
            "boot-check",
            "--json",
        ])
        .output()
        .expect("binary should run");
    assert!(!output.status.success());
    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("clap error stdout should be JSON");
    assert_eq!(json["status"], "error");
    assert!(json["error"]
        .as_str()
        .expect("error should be text")
        .contains("cannot be used with"));
}
