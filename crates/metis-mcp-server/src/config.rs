use anyhow::Result;

#[derive(Debug, Clone, Default)]
pub struct MetisServerConfig {
    // Configuration is now minimal since we use direct paths
    // Could add logging level, port bindings, etc. in future
}

impl MetisServerConfig {
    pub fn from_env() -> Result<Self> {
        // No environment variables needed for stateless operation
        Ok(Self {})
    }

    pub fn new() -> Self {
        Self {}
    }
}
