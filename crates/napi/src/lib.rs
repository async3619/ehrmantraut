#![deny(clippy::all)]

use ehrmantraut_js_lowering::Language;
use napi_derive::napi;
use rayon::prelude::*;

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

fn parse_internal(source: &str, language: &str) -> napi::Result<serde_json::Value> {
  let lang = parse_language(language)?;
  let cst = ehrmantraut_js_lowering::parse(source, lang)
    .map_err(|e| napi::Error::from_reason(format!("{e}")))?;
  serde_json::to_value(&cst)
    .map_err(|e| napi::Error::from_reason(format!("serialization error: {e}")))
}

fn lower_internal(source: &str, language: &str) -> napi::Result<serde_json::Value> {
  let lang = parse_language(language)?;
  let cst = ehrmantraut_js_lowering::parse(source, lang)
    .map_err(|e| napi::Error::from_reason(format!("{e}")))?;
  let ir = ehrmantraut_js_lowering::lower(&cst, lang, source)
    .map_err(|e| napi::Error::from_reason(format!("{e}")))?;
  serde_json::to_value(&ir)
    .map_err(|e| napi::Error::from_reason(format!("serialization error: {e}")))
}

#[napi(ts_return_type = "CstNode")]
pub fn parse(source: String, language: String) -> napi::Result<serde_json::Value> {
  parse_internal(&source, &language)
}

#[napi(ts_return_type = "IrModule")]
pub fn lower(source: String, language: String) -> napi::Result<serde_json::Value> {
  lower_internal(&source, &language)
}

#[napi(ts_return_type = "Promise<CstNode>")]
pub async fn parse_async(source: String, language: String) -> napi::Result<serde_json::Value> {
  parse_internal(&source, &language)
}

#[napi(ts_return_type = "Promise<IrModule>")]
pub async fn lower_async(source: String, language: String) -> napi::Result<serde_json::Value> {
  lower_internal(&source, &language)
}

#[napi(object)]
pub struct SourceEntry {
  pub source: String,
  pub language: String,
}

fn wrap_result(res: napi::Result<serde_json::Value>) -> serde_json::Value {
  match res {
    Ok(val) => serde_json::json!({ "success": true, "result": val }),
    Err(e) => serde_json::json!({ "success": false, "error": e.to_string() }),
  }
}

#[napi(ts_return_type = "Promise<Array<BatchParseResult>>")]
pub async fn parse_batch(entries: Vec<SourceEntry>) -> napi::Result<serde_json::Value> {
  let results: Vec<serde_json::Value> = entries
    .par_iter()
    .map(|entry| wrap_result(parse_internal(&entry.source, &entry.language)))
    .collect();
  Ok(serde_json::Value::Array(results))
}

#[napi(ts_return_type = "Promise<Array<BatchLowerResult>>")]
pub async fn lower_batch(entries: Vec<SourceEntry>) -> napi::Result<serde_json::Value> {
  let results: Vec<serde_json::Value> = entries
    .par_iter()
    .map(|entry| wrap_result(lower_internal(&entry.source, &entry.language)))
    .collect();
  Ok(serde_json::Value::Array(results))
}
