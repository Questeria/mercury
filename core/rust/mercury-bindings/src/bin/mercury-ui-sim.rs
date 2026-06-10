use std::{env, process};

use mercury_bindings::{
    BACKEND_COMMANDS, PLATFORM_FIXTURES, PROTOTYPE_FIXTURES, PlatformBridgeOperation,
    PlatformBridgeRequest, backend_command_by_name, backend_command_value,
    platform_bridge_handle_json, platform_bridge_response_value, platform_fixture_by_name,
    platform_fixture_view, prototype_fixture_by_name, prototype_fixture_value,
};
use serde_json::{Map, Value, json};

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let result = match args.as_slice() {
        [] => {
            print_usage();
            Ok(())
        }
        [arg] if arg == "--help" || arg == "-h" => {
            print_usage();
            Ok(())
        }
        [arg] if arg == "--list" => {
            print_fixture_list();
            Ok(())
        }
        [arg] if arg == "--list-prototypes" => {
            print_prototype_fixture_list();
            Ok(())
        }
        [arg] if arg == "--list-commands" => {
            print_backend_command_list();
            Ok(())
        }
        [arg] if arg == "--all" => print_all_fixtures(),
        [arg] if arg == "--all-prototypes" => print_all_prototype_fixtures(),
        [arg] if arg == "--all-commands" => print_all_backend_commands(),
        [flag, name] if flag == "--scenario" => print_scenario(name),
        [flag, name] if flag == "--prototype" => print_prototype(name),
        [flag, name] if flag == "--command" => print_backend_command(name),
        [flag, operation, name] if flag == "--bridge" => print_bridge(operation, name),
        [flag, request] if flag == "--bridge-json" => print_bridge_json(request),
        [flag, name] if flag == "--sequence" => print_sequence(name),
        _ => Err("unknown arguments".to_owned()),
    };

    if let Err(message) = result {
        eprintln!("mercury-ui-sim: {message}");
        print_usage();
        process::exit(2);
    }
}

fn print_usage() {
    println!(
        "Usage:\n  mercury-ui-sim --list\n  mercury-ui-sim --all\n  mercury-ui-sim --scenario <name>\n  mercury-ui-sim --sequence <name>\n  mercury-ui-sim --list-prototypes\n  mercury-ui-sim --all-prototypes\n  mercury-ui-sim --prototype <name>\n  mercury-ui-sim --list-commands\n  mercury-ui-sim --all-commands\n  mercury-ui-sim --command <name>\n  mercury-ui-sim --bridge <operation> <name>\n  mercury-ui-sim --bridge-json <json>"
    );
}

fn print_fixture_list() {
    for descriptor in PLATFORM_FIXTURES {
        println!("{}", descriptor.name);
    }
}

fn print_backend_command_list() {
    for descriptor in BACKEND_COMMANDS {
        println!("{}", descriptor.name);
    }
}

fn print_prototype_fixture_list() {
    for descriptor in PROTOTYPE_FIXTURES {
        println!("{}", descriptor.name);
    }
}

fn print_all_fixtures() -> Result<(), String> {
    let mut fixtures = Map::new();
    for descriptor in PLATFORM_FIXTURES {
        fixtures.insert(
            descriptor.name.to_owned(),
            serde_json::to_value(platform_fixture_view(descriptor.fixture))
                .map_err(|err| err.to_string())?,
        );
    }

    print_json(Value::Object(fixtures))
}

fn print_all_backend_commands() -> Result<(), String> {
    let mut commands = Map::new();
    for descriptor in BACKEND_COMMANDS {
        commands.insert(
            descriptor.name.to_owned(),
            backend_command_value(descriptor),
        );
    }

    print_json(Value::Object(commands))
}

fn print_all_prototype_fixtures() -> Result<(), String> {
    let mut fixtures = Map::new();
    for descriptor in PROTOTYPE_FIXTURES {
        fixtures.insert(
            descriptor.name.to_owned(),
            prototype_fixture_value(descriptor.fixture),
        );
    }

    print_json(Value::Object(fixtures))
}

fn print_scenario(name: &str) -> Result<(), String> {
    let fixture =
        platform_fixture_by_name(name).ok_or_else(|| format!("unknown scenario: {name}"))?;

    print_json(serde_json::to_value(platform_fixture_view(fixture)).map_err(|err| err.to_string())?)
}

fn print_prototype(name: &str) -> Result<(), String> {
    let fixture =
        prototype_fixture_by_name(name).ok_or_else(|| format!("unknown prototype: {name}"))?;

    print_json(prototype_fixture_value(fixture))
}

fn print_backend_command(name: &str) -> Result<(), String> {
    let descriptor =
        backend_command_by_name(name).ok_or_else(|| format!("unknown command: {name}"))?;

    print_json(backend_command_value(descriptor))
}

fn print_bridge(operation: &str, name: &str) -> Result<(), String> {
    let operation = PlatformBridgeOperation::from_label(operation)
        .ok_or_else(|| format!("unknown bridge operation: {operation}"))?;
    print_json(platform_bridge_response_value(PlatformBridgeRequest::new(
        "0123456789abcdef0123456789abcdef",
        operation,
        name,
        0,
    )))
}

fn print_bridge_json(request: &str) -> Result<(), String> {
    let json = platform_bridge_handle_json(request).map_err(|err| err.to_string())?;
    println!("{json}");
    Ok(())
}

fn print_sequence(name: &str) -> Result<(), String> {
    let sequence = match name {
        "startup_ready" => &["bootstrap_accepted"][..],
        "sync_then_ready" => &["bootstrap_sync_incomplete", "bootstrap_accepted"][..],
        "recovery_then_ready" => &["bootstrap_recovery_required", "bootstrap_accepted"][..],
        "send_receive_happy" => &[
            "bootstrap_accepted",
            "outbound_send_accepted",
            "client_receive_accepted",
        ][..],
        "receive_gap_retry" => &[
            "bootstrap_accepted",
            "client_receive_ordering_gap",
            "client_receive_accepted",
        ][..],
        "ai_unavailable" => &["policy_ai_grant_rejected", "policy_ai_lifecycle_expired"][..],
        _ => return Err(format!("unknown sequence: {name}")),
    };

    let mut states = Vec::with_capacity(sequence.len());
    for scenario in sequence {
        let fixture = platform_fixture_by_name(scenario)
            .ok_or_else(|| format!("sequence references unknown scenario: {scenario}"))?;
        let view =
            serde_json::to_value(platform_fixture_view(fixture)).map_err(|err| err.to_string())?;
        states.push(json!({
            "scenario": scenario,
            "view": view,
        }));
    }

    print_json(Value::Array(states))
}

fn print_json(value: Value) -> Result<(), String> {
    println!(
        "{}",
        serde_json::to_string_pretty(&value).map_err(|err| err.to_string())?
    );
    Ok(())
}
