use std::net::SocketAddr;

use anyhow::Result;
use shape_runner::codec::MsgPackCodec;
use shape_runner::llm::LlmClient;
use shape_runner::rpc::shaperunner::shape_runner_server::{ShapeRunner, ShapeRunnerServer};
use shape_runner::rpc::shaperunner::{RunRequest, RunResponse};
use shape_runner::shape::{feature_design_output_typedef, formation_output_typedef, FeatureDesignInput, FeatureDesignOutput, FormationInput, FormationOutput};
use tonic::{transport::Server, Request, Response, Status};

struct ShapeRunnerService<C> {
    codec: C,
    llm: LlmClient,
}

#[tonic::async_trait]
impl<C> ShapeRunner for ShapeRunnerService<C>
where
    C: shape_runner::codec::ShapeCodec + Send + Sync + 'static,
{
    async fn run(&self, request: Request<RunRequest>) -> Result<Response<RunResponse>, Status> {
        let inner = request.into_inner();

        match inner.shape_id.as_str() {
            "FeatureDesign" => {
                // Decode input bytes to FeatureDesignInput
                let input: FeatureDesignInput = self
                    .codec
                    .decode(&inner.input)
                    .map_err(|e| Status::invalid_argument(format!("decode input failed: {e}")))?;

                // Call LLM + validation
                let output: FeatureDesignOutput = self
                    .llm
                    .generate_feature_design(&input, &feature_design_output_typedef())
                    .await
                    .map_err(|e| Status::internal(format!("LLM error: {e}")))?;

                // Encode output to bytes
                let output_bytes = self
                    .codec
                    .encode(&output)
                    .map_err(|e| Status::internal(format!("encode output failed: {e}")))?;

                let resp = RunResponse {
                    output: output_bytes,
                    ok: true,
                    error: String::new(),
                };

                Ok(Response::new(resp))
            }
            "Formation" => {
                // Decode input bytes to FormationInput
                let input: FormationInput = self
                    .codec
                    .decode(&inner.input)
                    .map_err(|e| Status::invalid_argument(format!("decode input failed: {e}")))?;

                // Call LLM + validation
                let output: FormationOutput = self
                    .llm
                    .generate_formation(&input, &formation_output_typedef())
                    .await
                    .map_err(|e| Status::internal(format!("LLM error: {e}")))?;

                // Encode output to bytes
                let output_bytes = self
                    .codec
                    .encode(&output)
                    .map_err(|e| Status::internal(format!("encode output failed: {e}")))?;

                let resp = RunResponse {
                    output: output_bytes,
                    ok: true,
                    error: String::new(),
                };

                Ok(Response::new(resp))
            }
            _ => Err(Status::not_found(format!("unknown shape_id: {}", inner.shape_id))),
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Configure from env
    let addr: SocketAddr = "0.0.0.0:50051".parse().unwrap();
    let llm_base_url = std::env::var("LLM_BASE_URL").unwrap_or_else(|_| {
        // Default to Ollama if available, otherwise fall back to mock server
        "http://localhost:11434/api/generate".to_string()
    });
    let ollama_model = std::env::var("OLLAMA_MODEL").ok();

    println!("ShapeRunner listening on {addr}");
    println!("Using LLM endpoint: {}", llm_base_url);
    if let Some(ref model) = ollama_model {
        println!("Using Ollama model: {}", model);
    }

    let service = ShapeRunnerService {
        codec: MsgPackCodec,
        llm: LlmClient::new_with_model(llm_base_url, ollama_model),
    };

    Server::builder()
        .add_service(ShapeRunnerServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
