use anyhow::{anyhow, Result};
use clap::Parser;
use serde_json;
use shape_runner::client::ShapeRunnerClientWrapper;
use shape_runner::codec::ShapeCodec;
use shape_runner::shape::{FeatureDesignInput, FeatureDesignOutput};
use std::io::{self, Read, Write};

#[derive(Parser)]
#[command(name = "shape-runner-cli")]
#[command(about = "CLI client for ShapeRunner gRPC service")]
struct Cli {
    /// Shape ID to execute (e.g., "FeatureDesign")
    #[arg(short, long, default_value = "FeatureDesign")]
    shape: String,

    /// Server address (e.g., "http://localhost:50051")
    #[arg(short, long, default_value = "http://localhost:50051")]
    server: String,

    /// Input file path (use "-" for stdin)
    #[arg(short, long, default_value = "-")]
    input: String,

    /// Output format: json or msgpack
    #[arg(short, long, default_value = "json")]
    format: String,

    /// Request timeout in seconds
    #[arg(short, long, default_value = "60")]
    timeout: u64,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Read input
    let input_json = if cli.input == "-" {
        let mut buffer = String::new();
        io::stdin()
            .read_to_string(&mut buffer)
            .map_err(|e| anyhow!("Failed to read from stdin: {e}"))?;
        buffer
    } else {
        std::fs::read_to_string(&cli.input)
            .map_err(|e| anyhow!("Failed to read input file {}: {e}", cli.input))?
    };

    // Parse input based on shape type
    let input: FeatureDesignInput = serde_json::from_str(&input_json)
        .map_err(|e| anyhow!("Failed to parse input JSON: {e}"))?;

    // Connect to server
    println!("Connecting to ShapeRunner server at {}...", cli.server);
    let mut client = ShapeRunnerClientWrapper::connect(cli.server.clone())
        .await
        .map_err(|e| anyhow!("Failed to connect: {e}"))?;

    println!("Running shape '{}'...", cli.shape);

    // Execute shape with timeout
    let timeout = std::time::Duration::from_secs(cli.timeout);
    let output: FeatureDesignOutput = client
        .run_shape_with_timeout(cli.shape.clone(), &input, timeout)
        .await
        .map_err(|e| anyhow!("Shape execution failed: {e}"))?;

    // Output result
    match cli.format.as_str() {
        "json" => {
            let json = serde_json::to_string_pretty(&output)
                .map_err(|e| anyhow!("Failed to serialize output: {e}"))?;
            println!("{}", json);
        }
        "msgpack" => {
            let codec = shape_runner::codec::MsgPackCodec;
            let bytes = codec
                .encode(&output)
                .map_err(|e| anyhow!("Failed to encode output: {e}"))?;
            io::stdout()
                .write_all(&bytes)
                .map_err(|e| anyhow!("Failed to write output: {e}"))?;
        }
        _ => {
            return Err(anyhow!("Unknown output format: {}", cli.format));
        }
    }

    Ok(())
}

