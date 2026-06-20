use candle_core::{DType, Device, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::bert::{BertModel, Config};
use hf_hub::{Cache, Repo, api::sync::ApiBuilder};
use tokenizers::{Tokenizer, TruncationParams};

pub struct EmbeddingModel {
    model: BertModel,
    tokenizer: Tokenizer,
    device: Device,
}

impl EmbeddingModel {
    pub fn new(model_id: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let device = Device::Cpu;

        let cache = Cache::from_env();
        let cached = cache.repo(Repo::model(model_id.to_string()));
        let (model_file, config_file, tokenizer_file) = match (
            cached.get("model.safetensors"),
            cached.get("config.json"),
            cached.get("tokenizer.json"),
        ) {
            (Some(m), Some(c), Some(t)) => {
                println!("Using cached model files");
                (m, c, t)
            }
            _ => {
                println!("Downloading model files...");
                let api = ApiBuilder::from_env().with_progress(true).build()?;
                let repo = api.model(model_id.to_string());
                (
                    repo.get("model.safetensors")?,
                    repo.get("config.json")?,
                    repo.get("tokenizer.json")?,
                )
            }
        };

        // モデルと設定の読み込み
        let tensors = candle_core::safetensors::load(model_file, &device)?;
        let config: Config = serde_json::from_slice(&std::fs::read(config_file)?)?;
        let vb = VarBuilder::from_tensors(tensors, DType::F32, &device);
        let model = BertModel::load(vb, &config)?;

        // トークナイザーの読み込み（510トークンで切り詰め、[CLS]と[SEP]で+2）
        let mut tokenizer = Tokenizer::from_file(tokenizer_file)
            .map_err(|e| format!("Failed to load tokenizer: {e}"))?;
        tokenizer
            .with_truncation(Some(TruncationParams {
                max_length: 510,
                ..Default::default()
            }))
            .map_err(|e| format!("Failed to set truncation: {e}"))?;

        Ok(Self {
            model,
            tokenizer,
            device,
        })
    }

    pub fn embed(&self, text: &str) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
        // テキストをトークン化
        let tokens = self
            .tokenizer
            .encode(text, true)
            .map_err(|e| format!("Failed to tokenize: {e}"))?;

        let token_ids = Tensor::new(tokens.get_ids(), &self.device)?;
        let attention_mask = Tensor::new(tokens.get_attention_mask(), &self.device)?;
        let token_type_ids = token_ids.zeros_like()?;

        // モデルで推論実行
        let embeddings = self.model.forward(
            &token_ids.unsqueeze(0)?,
            &token_type_ids.unsqueeze(0)?,
            Some(&attention_mask.unsqueeze(0)?),
        )?;

        // 平均プーリングで文章埋め込みを生成
        let (_batch_size, seq_len, _hidden_size) = embeddings.dims3()?;
        let pooled = (embeddings.sum(1)? * (1.0 / seq_len as f64))?;

        // 正規化
        let normalized = pooled.broadcast_div(&pooled.sqr()?.sum_keepdim(1)?.sqrt()?)?;

        Ok(normalized.squeeze(0)?.to_vec1()?)
    }
}
