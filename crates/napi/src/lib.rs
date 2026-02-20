#![deny(clippy::all)]

use ehrmantraut_js_lowering::Language;
use napi_derive::napi;

fn parse_language(language: &str) -> napi::Result<Language> {
  match language {
    "javascript" | "js" => Ok(Language::JavaScript),
    "typescript" | "ts" => Ok(Language::TypeScript),
    "tsx" => Ok(Language::Tsx),
    _ => Err(napi::Error::from_reason(format!(
      "unsupported language: {language}"
    ))),
  }
}

#[napi]
pub fn parse(source: String, language: String) -> napi::Result<serde_json::Value> {
  let lang = parse_language(&language)?;
  let cst = ehrmantraut_js_lowering::parse(&source, lang)
    .map_err(|e| napi::Error::from_reason(format!("{e}")))?;
  serde_json::to_value(&cst)
    .map_err(|e| napi::Error::from_reason(format!("serialization error: {e}")))
}

#[napi]
pub fn lower(source: String, language: String) -> napi::Result<serde_json::Value> {
  let lang = parse_language(&language)?;
  let cst = ehrmantraut_js_lowering::parse(&source, lang)
    .map_err(|e| napi::Error::from_reason(format!("{e}")))?;
  let ir = ehrmantraut_js_lowering::lower(&cst, lang)
    .map_err(|e| napi::Error::from_reason(format!("{e}")))?;
  serde_json::to_value(&ir)
    .map_err(|e| napi::Error::from_reason(format!("serialization error: {e}")))
}
