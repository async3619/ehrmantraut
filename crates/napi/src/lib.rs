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

fn wrap_result_json(res: napi::Result<serde_json::Value>) -> String {
  match res {
    Ok(val) => {
      let inner = serde_json::to_string(&val).unwrap_or_default();
      format!(r#"{{"success":true,"result":{inner}}}"#)
    }
    Err(e) => {
      let escaped = e.to_string().replace('\\', "\\\\").replace('"', "\\\"");
      format!(r#"{{"success":false,"error":"{escaped}"}}"#)
    }
  }
}

fn batch_to_json_strings(
  entries: &[SourceEntry],
  process: impl Fn(&str, &str) -> napi::Result<serde_json::Value> + Sync,
) -> Vec<String> {
  entries
    .par_iter()
    .map(|entry| wrap_result_json(process(&entry.source, &entry.language)))
    .collect()
}

#[napi(js_name = "parseBatchRaw")]
pub async fn parse_batch(entries: Vec<SourceEntry>) -> napi::Result<Vec<String>> {
  Ok(batch_to_json_strings(&entries, parse_internal))
}

#[napi(js_name = "lowerBatchRaw")]
pub async fn lower_batch(entries: Vec<SourceEntry>) -> napi::Result<Vec<String>> {
  Ok(batch_to_json_strings(&entries, lower_internal))
}
