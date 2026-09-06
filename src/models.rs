//! The models known to work, with what the code cannot read from their files:
//! languages, size, and the prefixes some of them expect on queries and documents.

/// Prefixes an embedder expects, when it was trained with them.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Prompts {
    pub query: String,
    pub document: String,
}

pub struct KnownModel {
    pub repo: &'static str,
    pub alias: &'static str,
    pub family: &'static str,
    pub languages: &'static str,
    pub window: usize,
    pub params_m: usize,
    pub download_mb: usize,
    pub license: &'static str,
    pub prompts: Option<(&'static str, &'static str)>,
    pub note: &'static str,
}

pub const KNOWN: [KnownModel; 7] = [
    KnownModel {
        repo: "ibm-granite/granite-embedding-278m-multilingual",
        alias: "granite-multilingual",
        family: "XLM-RoBERTa",
        languages: "100+",
        window: 512,
        params_m: 278,
        download_mb: 556,
        license: "Apache-2.0",
        prompts: None,
        note: "default: notes in several languages, or a mix",
    },
    KnownModel {
        repo: "intfloat/multilingual-e5-small",
        alias: "e5-small",
        family: "BERT (MiniLM)",
        languages: "100+",
        window: 512,
        params_m: 118,
        download_mb: 471,
        license: "MIT",
        prompts: Some(("query: ", "passage: ")),
        note: "the small multilingual option, 384 dimensions",
    },
    KnownModel {
        repo: "intfloat/multilingual-e5-base",
        alias: "e5-base",
        family: "XLM-RoBERTa",
        languages: "100+",
        window: 512,
        params_m: 278,
        download_mb: 1112,
        license: "MIT",
        prompts: Some(("query: ", "passage: ")),
        note: "multilingual, mean pooling, query and passage prefixes",
    },
    KnownModel {
        repo: "intfloat/multilingual-e5-large",
        alias: "e5-large",
        family: "XLM-RoBERTa large",
        languages: "100+",
        window: 512,
        params_m: 560,
        download_mb: 2240,
        license: "MIT",
        prompts: Some(("query: ", "passage: ")),
        note: "multilingual, 1024 dimensions, the heaviest choice",
    },
    KnownModel {
        repo: "ibm-granite/granite-embedding-97m-multilingual-r2",
        alias: "granite-multilingual-r2",
        family: "ModernBERT",
        languages: "100+",
        window: 32768,
        params_m: 97,
        download_mb: 220,
        license: "Apache-2.0",
        prompts: None,
        note: "multilingual, whole notes in one vector, the lightest ModernBERT",
    },
    KnownModel {
        repo: "ibm-granite/granite-embedding-english-r2",
        alias: "granite-en",
        family: "ModernBERT",
        languages: "English",
        window: 8192,
        params_m: 149,
        download_mb: 298,
        license: "Apache-2.0",
        prompts: None,
        note: "English, long context, 768 dimensions",
    },
    KnownModel {
        repo: "Alibaba-NLP/gte-modernbert-base",
        alias: "gte-modernbert",
        family: "ModernBERT",
        languages: "English",
        window: 8192,
        params_m: 149,
        download_mb: 298,
        license: "Apache-2.0",
        prompts: None,
        note: "English, long context, strong on public retrieval benchmarks",
    },
];

/// The known model behind an alias, a repository name, or a directory name.
pub fn find(name: &str) -> Option<&'static KnownModel> {
    let name = name.trim_end_matches('/');
    let last = name.rsplit('/').next().unwrap_or(name);
    KNOWN.iter().find(|m| m.alias == name || m.repo == name || m.repo.rsplit('/').next() == Some(last))
}

/// Prefixes for a model directory: the known table first, then the
/// `config_sentence_transformers.json` prompts shipped with the model.
pub fn prompts_for(dir: &std::path::Path) -> Prompts {
    if let Some(m) = dir.file_name().and_then(|n| n.to_str()).and_then(find) {
        if let Some((q, d)) = m.prompts {
            return Prompts { query: q.to_string(), document: d.to_string() };
        }
    }
    let Ok(raw) = std::fs::read_to_string(dir.join("config_sentence_transformers.json")) else { return Prompts::default() };
    let Ok(v) = serde_json::from_str::<serde_json::Value>(&raw) else { return Prompts::default() };
    let p = &v["prompts"];
    let pick = |keys: &[&str]| keys.iter().find_map(|k| p[k].as_str().map(str::to_string)).unwrap_or_default();
    Prompts { query: pick(&["query", "search_query"]), document: pick(&["passage", "document", "search_document"]) }
}
