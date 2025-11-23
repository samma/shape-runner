use anyhow::{anyhow, Result};
use crate::codec::{MsgPackCodec, ShapeCodec};
use crate::rpc::shaperunner::shape_runner_client::ShapeRunnerClient;
use crate::rpc::shaperunner::{RunRequest, RunResponse};
use serde::{de::DeserializeOwned, Serialize};
use std::time::Duration;
use tonic::transport::Channel;

pub struct ShapeRunnerClientWrapper {
    client: ShapeRunnerClient<Channel>,
    codec: MsgPackCodec,
}

impl ShapeRunnerClientWrapper {
    pub async fn connect(addr: String) -> Result<Self> {
        let client = ShapeRunnerClient::connect(addr)
            .await
            .map_err(|e| anyhow!("Failed to connect to ShapeRunner server: {e}"))?;

        Ok(Self {
            client,
            codec: MsgPackCodec,
        })
    }

    pub async fn run_shape<I, O>(&mut self, shape_id: String, input: &I) -> Result<O>
    where
        I: Serialize,
        O: DeserializeOwned,
    {
        // Encode input
        let input_bytes = self
            .codec
            .encode(input)
            .map_err(|e| anyhow!("Failed to encode input: {e}"))?;

        // Make gRPC call
        let request = tonic::Request::new(RunRequest {
            shape_id,
            input: input_bytes,
        });

        let response = self
            .client
            .run(request)
            .await
            .map_err(|e| anyhow!("gRPC call failed: {e}"))?;

        let RunResponse { output, ok, error } = response.into_inner();

        if !ok {
            return Err(anyhow!("Shape execution failed: {}", error));
        }

        // Decode output
        let result: O = self
            .codec
            .decode(&output)
            .map_err(|e| anyhow!("Failed to decode output: {e}"))?;

        Ok(result)
    }

    pub async fn run_shape_with_timeout<I, O>(
        &mut self,
        shape_id: String,
        input: &I,
        timeout: Duration,
    ) -> Result<O>
    where
        I: Serialize,
        O: DeserializeOwned,
    {
        // Encode input
        let input_bytes = self
            .codec
            .encode(input)
            .map_err(|e| anyhow!("Failed to encode input: {e}"))?;

        // Make gRPC call with timeout
        let request = tonic::Request::new(RunRequest {
            shape_id,
            input: input_bytes,
        });

        let response = tokio::time::timeout(timeout, self.client.run(request))
            .await
            .map_err(|_| anyhow!("Request timed out after {:?}", timeout))?
            .map_err(|e| anyhow!("gRPC call failed: {e}"))?;

        let RunResponse { output, ok, error } = response.into_inner();

        if !ok {
            return Err(anyhow!("Shape execution failed: {}", error));
        }

        // Decode output
        let result: O = self
            .codec
            .decode(&output)
            .map_err(|e| anyhow!("Failed to decode output: {e}"))?;

        Ok(result)
    }
}

