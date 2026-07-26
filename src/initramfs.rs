use plaza_foundation::core::PlazaResult;

pub struct InitramfsGenerator;

impl InitramfsGenerator {
    pub fn new() -> Self {
        Self
    }
    
    pub async fn generate(&self, _output_path: &str) -> PlazaResult<()> {
        Ok(()) // DP1 Stub
    }
}
