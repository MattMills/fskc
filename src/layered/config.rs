use super::layer::{Layer, FractalLayer, SymmetricLayer};

/// Represents a sequence of encryption layers
pub type LayerSequence = Vec<Layer>;

/// Configuration for the layered encryption system
#[derive(Clone)]
pub struct LayerConfig {
    pub(crate) sequence: LayerSequence,
    pub(crate) fractal_depth: usize,
    pub(crate) chunk_size: usize,
    pub(crate) use_zippering: bool,
}

impl Default for LayerConfig {
    fn default() -> Self {
        Self {
            sequence: vec![
                Layer::Fractal(FractalLayer::new()),
                Layer::Symmetric(SymmetricLayer::Aes),
            ],
            fractal_depth: 3,
            chunk_size: 64,
            use_zippering: false,
        }
    }
}

impl LayerConfig {
    /// Creates a new builder for LayerConfig
    pub fn builder() -> Builder {
        Builder::default()
    }
}

/// Builder for configuring LayerConfig
#[derive(Default)]
pub struct Builder {
    sequence: LayerSequence,
    fractal_depth: Option<usize>,
    chunk_size: Option<usize>,
    use_zippering: bool,
}

impl Builder {
    /// Creates a new LayerConfig builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a fractal layer to the sequence
    pub fn add_fractal(mut self) -> Self {
        self.sequence.push(Layer::Fractal(FractalLayer::new()));
        self
    }

    /// Adds an AES layer to the sequence
    pub fn add_aes(mut self) -> Self {
        self.sequence.push(Layer::Symmetric(SymmetricLayer::Aes));
        self
    }

    /// Adds a ChaCha20 layer to the sequence
    pub fn add_chacha(mut self) -> Self {
        self.sequence.push(Layer::Symmetric(SymmetricLayer::ChaCha));
        self
    }

    /// Sets the fractal depth for all fractal layers
    pub fn fractal_depth(mut self, depth: usize) -> Self {
        self.fractal_depth = Some(depth);
        self
    }

    /// Sets the chunk size for fractal layers
    pub fn chunk_size(mut self, size: usize) -> Self {
        self.chunk_size = Some(size);
        self
    }

    /// Enables self-zippering
    pub fn enable_zippering(mut self) -> Self {
        self.use_zippering = true;
        self
    }

    /// Builds the LayerConfig
    pub fn build(self) -> LayerConfig {
        LayerConfig {
            sequence: if self.sequence.is_empty() {
                LayerConfig::default().sequence
            } else {
                self.sequence
            },
            fractal_depth: self.fractal_depth.unwrap_or(3),
            chunk_size: self.chunk_size.unwrap_or(64),
            use_zippering: self.use_zippering,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = LayerConfig::default();
        assert_eq!(config.sequence.len(), 2);
        assert_eq!(config.fractal_depth, 3);
        assert_eq!(config.chunk_size, 64);
        assert!(!config.use_zippering);
    }

    #[test]
    fn test_builder_pattern() {
        let config = LayerConfig::builder()
            .add_fractal()
            .add_aes()
            .add_chacha()
            .fractal_depth(2)
            .chunk_size(128)
            .enable_zippering()
            .build();

        assert_eq!(config.sequence.len(), 3);
        assert_eq!(config.fractal_depth, 2);
        assert_eq!(config.chunk_size, 128);
        assert!(config.use_zippering);
    }
}
