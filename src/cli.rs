use crate::automation::{
    plan_target, validate_pack, MacroExecutor, MacroPack, MacroTarget, SerialMacroTransport,
    SimulatedMacroTransport,
};
use crate::config::{
    CliDataFormat, Command, ControlLineLevel, MacroCommand, MacroFileCommand, MacroPlanCommand,
    MacroRunCommand, OptionalSerialPortArgs, ReadCommand, SerialPortArgs, SetControlLinesCommand,
    WriteCommand,
};
use crate::error::{Result, SerialError};
use crate::serial::{
    ConnectionConfig, DataBits, FlowControl, LocalSerialError, Parity, PortInfo, SerialConnection,
    StopBits,
};
use crate::utils::{DataConverter, DataFormat};
use crate::Config;
use serde::Serialize;
use std::sync::Arc;

pub async fn run(command: Command, config: &Config) -> Result<()> {
    match command {
        Command::ListPorts(args) => list_ports(args.json),
        Command::Probe(args) => probe(args, config).await,
        Command::Write(args) => write(args, config).await,
        Command::Read(args) => read(args, config).await,
        Command::SetControlLines(args) => set_control_lines(args, config).await,
        Command::Macro(args) => macro_command(args, config).await,
        Command::Serve
        | Command::GenerateConfig
        | Command::ValidateConfig
        | Command::ShowConfig => Err(SerialError::InternalError(
            "server/config command reached CLI command dispatcher".to_string(),
        )),
    }
}

async fn macro_command(args: MacroCommand, config: &Config) -> Result<()> {
    match args {
        MacroCommand::Validate(args) => macro_validate(args),
        MacroCommand::List(args) => macro_list(args),
        MacroCommand::Plan(args) => macro_plan(args),
        MacroCommand::Run(args) => macro_run(args, config).await,
    }
}

fn macro_validate(args: MacroFileCommand) -> Result<()> {
    let pack = load_macro_pack(&args.file)?;
    let inventory = validate_pack(&pack)?;
    if args.json {
        print_json(&MacroValidateOutput {
            status: "ok",
            inventory,
        })?;
    } else {
        println!("Macro pack {} is valid", pack.name);
    }
    Ok(())
}

fn macro_list(args: MacroFileCommand) -> Result<()> {
    let pack = load_macro_pack(&args.file)?;
    let inventory = validate_pack(&pack)?;
    if args.json {
        print_json(&inventory)?;
    } else {
        println!("Macros:");
        for name in inventory.macros {
            println!("- {}", name);
        }
        if !inventory.assemblies.is_empty() {
            println!("Assemblies:");
            for name in inventory.assemblies {
                println!("- {}", name);
            }
        }
    }
    Ok(())
}

fn macro_plan(args: MacroPlanCommand) -> Result<()> {
    let pack = load_macro_pack(&args.file)?;
    let target = macro_target(args.macro_name, args.assembly)?;
    let plan = plan_target(&pack, target)?;
    if args.json {
        print_json(&plan)?;
    } else {
        println!(
            "{} {} has {} step(s)",
            plan.target_kind,
            plan.target_name,
            plan.steps.len()
        );
    }
    Ok(())
}

async fn macro_run(args: MacroRunCommand, config: &Config) -> Result<()> {
    let pack = load_macro_pack(&args.file)?;
    let target = macro_target(args.macro_name, args.assembly)?;
    let plan = plan_target(&pack, target)?;

    if args.dry_run {
        let output = MacroDryRunOutput {
            mode: "dry_run",
            success: true,
            plan,
        };
        if args.json {
            print_json(&output)?;
        } else {
            println!(
                "Dry run {} {} with {} step(s)",
                output.plan.target_kind,
                output.plan.target_name,
                output.plan.steps.len()
            );
        }
        return Ok(());
    }

    if !args.simulate_read.is_empty() {
        let reads = args
            .simulate_read
            .iter()
            .map(|chunk| chunk.as_bytes().to_vec())
            .collect();
        let report = MacroExecutor::simulated()
            .run(plan, SimulatedMacroTransport::new(reads))
            .await?;
        if args.json {
            print_json(&report)?;
        } else {
            println!(
                "Macro {} {} completed in simulation",
                report.target_kind, report.target_name
            );
        }
        return Ok(());
    }

    let connection_config = macro_connection_config(&args.serial, config)?;
    let connection = Arc::new(open_connection(connection_config).await?);
    let report = MacroExecutor::real()
        .run(plan, SerialMacroTransport::new(connection))
        .await?;
    if args.json {
        print_json(&report)?;
    } else {
        println!(
            "Macro {} {} completed",
            report.target_kind, report.target_name
        );
    }
    Ok(())
}

fn load_macro_pack(path: &std::path::Path) -> Result<MacroPack> {
    let content = std::fs::read_to_string(path)?;
    let pack = serde_json::from_str::<MacroPack>(&content)?;
    validate_pack(&pack)?;
    Ok(pack)
}

fn macro_target(macro_name: Option<String>, assembly: Option<String>) -> Result<MacroTarget> {
    match (macro_name, assembly) {
        (Some(name), None) => Ok(MacroTarget::macro_named(name)),
        (None, Some(name)) => Ok(MacroTarget::assembly_named(name)),
        _ => Err(SerialError::InvalidConfig(
            "Specify exactly one of --macro or --assembly".to_string(),
        )),
    }
}

fn list_ports(json: bool) -> Result<()> {
    let ports = PortInfo::list_ports()?;
    if json {
        print_json(&PortsOutput { ports })?;
        return Ok(());
    }

    if ports.is_empty() {
        println!("No serial ports found");
        return Ok(());
    }

    for port in ports {
        if let Some(hardware_id) = port.hardware_id {
            println!("{}\t{}\t{}", port.name, port.description, hardware_id);
        } else {
            println!("{}\t{}", port.name, port.description);
        }
    }
    Ok(())
}

async fn probe(args: SerialPortArgs, app_config: &Config) -> Result<()> {
    let json = args.json;
    let config = connection_config(&args, app_config)?;
    let connection = open_connection(config.clone()).await?;
    let status = connection.status().await;
    let output = ProbeOutput {
        port: config.port,
        baud_rate: config.baud_rate,
        opened: true,
        connection_id: status.id,
    };

    if json {
        print_json(&output)?;
    } else {
        println!(
            "Opened {} at {} baud (connection {})",
            output.port, output.baud_rate, output.connection_id
        );
    }
    Ok(())
}

async fn write(args: WriteCommand, config: &Config) -> Result<()> {
    let json = args.serial.json;
    let connection_config = connection_config(&args.serial, config)?;
    let baud_rate = connection_config.baud_rate;
    let connection = open_connection(connection_config).await?;
    let format = args.format.as_data_format();
    let payload = DataConverter::decode(&args.data, format)?;
    let bytes_written = connection.write(&payload).await.map_err(map_serial_error)?;
    let read = if args.read {
        Some(
            read_from_connection(
                &connection,
                args.timeout_ms.unwrap_or(config.serial.default_timeout_ms),
                args.max_bytes,
                format,
            )
            .await?,
        )
    } else {
        None
    };
    let output = WriteOutput {
        port: args.serial.port,
        baud_rate,
        bytes_written,
        read,
    };

    if json {
        print_json(&output)?;
    } else if let Some(read) = output.read {
        println!(
            "Wrote {} bytes, read {} bytes: {}",
            output.bytes_written, read.bytes_read, read.data
        );
    } else {
        println!("Wrote {} bytes", output.bytes_written);
    }
    Ok(())
}

async fn read(args: ReadCommand, config: &Config) -> Result<()> {
    let json = args.serial.json;
    let connection_config = connection_config(&args.serial, config)?;
    let baud_rate = connection_config.baud_rate;
    let connection = open_connection(connection_config).await?;
    let read = read_from_connection(
        &connection,
        args.timeout_ms.unwrap_or(config.serial.default_timeout_ms),
        args.max_bytes,
        args.format.as_data_format(),
    )
    .await?;
    let output = ReadOutput {
        port: args.serial.port,
        baud_rate,
        read,
    };

    if json {
        print_json(&output)?;
    } else {
        println!("{}", output.read.data);
    }
    Ok(())
}

async fn set_control_lines(args: SetControlLinesCommand, app_config: &Config) -> Result<()> {
    if args.rts.is_none() && args.dtr.is_none() {
        return Err(SerialError::InvalidConfig(
            "At least one of --rts or --dtr must be specified".to_string(),
        ));
    }

    let json = args.serial.json;
    let connection_config = connection_config(&args.serial, app_config)?;
    let baud_rate = connection_config.baud_rate;
    let connection = open_connection(connection_config).await?;
    if let Some(rts) = args.rts {
        connection
            .set_rts(rts.as_bool())
            .await
            .map_err(map_serial_error)?;
    }
    if let Some(dtr) = args.dtr {
        connection
            .set_dtr(dtr.as_bool())
            .await
            .map_err(map_serial_error)?;
    }

    let output = ControlLinesOutput {
        port: args.serial.port,
        baud_rate,
        rts: args.rts.map(ControlLineLevel::as_str),
        dtr: args.dtr.map(ControlLineLevel::as_str),
    };

    if json {
        print_json(&output)?;
    } else {
        let mut parts = Vec::new();
        if let Some(rts) = output.rts {
            parts.push(format!("RTS {}", rts));
        }
        if let Some(dtr) = output.dtr {
            parts.push(format!("DTR {}", dtr));
        }
        println!("Updated {} on {}", parts.join(", "), output.port);
    }
    Ok(())
}

async fn read_from_connection(
    connection: &SerialConnection,
    timeout_ms: u64,
    max_bytes: usize,
    format: DataFormat,
) -> Result<ReadPayload> {
    let mut buffer = vec![0_u8; max_bytes];
    match connection.read(&mut buffer, Some(timeout_ms)).await {
        Ok(bytes_read) => {
            buffer.truncate(bytes_read);
            Ok(ReadPayload {
                bytes_read,
                data: DataConverter::encode(&buffer, format)?,
                status: "ok",
                timeout_ms,
            })
        }
        Err(LocalSerialError::ReadTimeout) => Ok(ReadPayload {
            bytes_read: 0,
            data: String::new(),
            status: "timeout",
            timeout_ms,
        }),
        Err(error) => Err(map_serial_error(error)),
    }
}

async fn open_connection(config: ConnectionConfig) -> Result<SerialConnection> {
    SerialConnection::new(config)
        .await
        .map_err(map_serial_error)
}

fn connection_config(args: &SerialPortArgs, config: &Config) -> Result<ConnectionConfig> {
    let stop_bits = args
        .stop_bits
        .as_deref()
        .unwrap_or(&config.serial.default_stop_bits);
    let parity = args
        .parity
        .as_deref()
        .unwrap_or(&config.serial.default_parity);
    let flow_control = args
        .flow_control
        .as_deref()
        .unwrap_or(&config.serial.default_flow_control);

    Ok(ConnectionConfig {
        port: args.port.clone(),
        baud_rate: args.baud.unwrap_or(config.serial.default_baud_rate),
        data_bits: parse_data_bits(args.data_bits.unwrap_or(config.serial.default_data_bits))?,
        stop_bits: parse_stop_bits(stop_bits)?,
        parity: parse_parity(parity)?,
        flow_control: parse_flow_control(flow_control)?,
    })
}

fn macro_connection_config(
    args: &OptionalSerialPortArgs,
    config: &Config,
) -> Result<ConnectionConfig> {
    let Some(port) = &args.port else {
        return Err(SerialError::InvalidConfig(
            "macro run requires --dry-run, --simulate-read, or --port".to_string(),
        ));
    };
    let stop_bits = args
        .stop_bits
        .as_deref()
        .unwrap_or(&config.serial.default_stop_bits);
    let parity = args
        .parity
        .as_deref()
        .unwrap_or(&config.serial.default_parity);
    let flow_control = args
        .flow_control
        .as_deref()
        .unwrap_or(&config.serial.default_flow_control);

    Ok(ConnectionConfig {
        port: port.clone(),
        baud_rate: args.baud.unwrap_or(config.serial.default_baud_rate),
        data_bits: parse_data_bits(args.data_bits.unwrap_or(config.serial.default_data_bits))?,
        stop_bits: parse_stop_bits(stop_bits)?,
        parity: parse_parity(parity)?,
        flow_control: parse_flow_control(flow_control)?,
    })
}

fn parse_data_bits(value: u8) -> Result<DataBits> {
    match value {
        5 => Ok(DataBits::Five),
        6 => Ok(DataBits::Six),
        7 => Ok(DataBits::Seven),
        8 => Ok(DataBits::Eight),
        _ => Err(SerialError::InvalidDataBits(value)),
    }
}

fn parse_stop_bits(value: &str) -> Result<StopBits> {
    match value.to_lowercase().as_str() {
        "one" | "1" => Ok(StopBits::One),
        "two" | "2" => Ok(StopBits::Two),
        _ => Err(SerialError::InvalidStopBits(value.to_string())),
    }
}

fn parse_parity(value: &str) -> Result<Parity> {
    match value.to_lowercase().as_str() {
        "none" => Ok(Parity::None),
        "odd" => Ok(Parity::Odd),
        "even" => Ok(Parity::Even),
        _ => Err(SerialError::InvalidParity(value.to_string())),
    }
}

fn parse_flow_control(value: &str) -> Result<FlowControl> {
    match value.to_lowercase().as_str() {
        "none" => Ok(FlowControl::None),
        "software" => Ok(FlowControl::Software),
        "hardware" => Ok(FlowControl::Hardware),
        _ => Err(SerialError::InvalidFlowControl(value.to_string())),
    }
}

fn map_serial_error(error: LocalSerialError) -> SerialError {
    match error {
        LocalSerialError::PortNotFound(port) => SerialError::PortNotFound(port),
        LocalSerialError::ConnectionFailed(reason) => SerialError::ConnectionFailed(reason),
        LocalSerialError::InvalidConnection(id) => SerialError::InvalidConnection(id),
        LocalSerialError::ConnectionExists(port) => SerialError::ConnectionExists(port),
        LocalSerialError::InvalidBaudRate(rate) => SerialError::InvalidBaudRate(rate),
        LocalSerialError::InvalidConfig(reason) => SerialError::InvalidConfig(reason),
        LocalSerialError::ReadTimeout => SerialError::ReadTimeout,
        LocalSerialError::WriteTimeout => SerialError::WriteTimeout,
        LocalSerialError::EncodingError(reason) => SerialError::EncodingError(reason),
        LocalSerialError::IoError(error) => SerialError::IoError(error),
        LocalSerialError::SerialPortError(error) => SerialError::SerialPortError(error),
        LocalSerialError::Utf8Error(error) => SerialError::Utf8Error(error),
    }
}

fn print_json<T: Serialize>(value: &T) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}

impl CliDataFormat {
    fn as_data_format(self) -> DataFormat {
        match self {
            CliDataFormat::Utf8 => DataFormat::Text,
            CliDataFormat::Hex => DataFormat::Hex,
            CliDataFormat::Base64 => DataFormat::Base64,
        }
    }
}

#[derive(Debug, Serialize)]
struct PortsOutput {
    ports: Vec<PortInfo>,
}

#[derive(Debug, Serialize)]
struct ProbeOutput {
    port: String,
    baud_rate: u32,
    opened: bool,
    connection_id: String,
}

#[derive(Debug, Serialize)]
struct WriteOutput {
    port: String,
    baud_rate: u32,
    bytes_written: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    read: Option<ReadPayload>,
}

#[derive(Debug, Serialize)]
struct ReadOutput {
    port: String,
    baud_rate: u32,
    read: ReadPayload,
}

#[derive(Debug, Serialize)]
struct ReadPayload {
    bytes_read: usize,
    data: String,
    status: &'static str,
    timeout_ms: u64,
}

#[derive(Debug, Serialize)]
struct ControlLinesOutput {
    port: String,
    baud_rate: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    rts: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    dtr: Option<&'static str>,
}

#[derive(Debug, Serialize)]
struct MacroValidateOutput {
    status: &'static str,
    inventory: crate::automation::MacroInventory,
}

#[derive(Debug, Serialize)]
struct MacroDryRunOutput {
    mode: &'static str,
    success: bool,
    plan: crate::automation::MacroPlan,
}
