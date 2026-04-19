//! minimal lsp stdio handler.
//!
//! tower-lsp + tokio brings in ~2MB of binary and the v0.14.x lsp surface is
//! small. so we speak the protocol by hand over stdin/stdout: content-length
//! framed json-rpc, only the methods we need.

use crate::config::Config;
use crate::dialect::Dialect;
use crate::parse::parse;
use crate::rules::{Registry, Severity};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};

pub fn run() -> anyhow::Result<()> {
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let mut reader = BufReader::new(stdin.lock());
    let mut out = stdout.lock();
    let mut server = Server::default();

    loop {
        let msg = match read_message(&mut reader)? {
            Some(m) => m,
            None => break,
        };
        let parsed: Value = serde_json::from_str(&msg)?;
        let method = parsed.get("method").and_then(|m| m.as_str()).unwrap_or("");
        let id = parsed.get("id").cloned();

        match method {
            "initialize" => {
                let resp = json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "capabilities": {
                            "textDocumentSync": 1,
                            "codeActionProvider": true
                        },
                        "serverInfo": { "name": "drift", "version": crate::VERSION }
                    }
                });
                write_message(&mut out, &resp.to_string())?;
            }
            "initialized" => {}
            "shutdown" => {
                write_message(
                    &mut out,
                    &json!({"jsonrpc":"2.0","id":id,"result":null}).to_string(),
                )?;
            }
            "exit" => break,
            "textDocument/didOpen" => {
                server.handle_open(&parsed);
                server.publish(&mut out)?;
            }
            "textDocument/didChange" => {
                server.handle_change(&parsed);
                server.publish(&mut out)?;
            }
            "textDocument/didSave" => {
                server.handle_save(&parsed);
                server.publish(&mut out)?;
            }
            _ => {}
        }
    }
    Ok(())
}

#[derive(Default)]
struct Server {
    docs: HashMap<String, String>,
    registry: Registry,
    config: Config,
}

#[derive(Deserialize)]
struct DidOpenParams {
    #[serde(rename = "textDocument")]
    text_document: TextDocumentItem,
}
#[derive(Deserialize)]
struct TextDocumentItem {
    uri: String,
    text: String,
}
#[derive(Deserialize)]
struct DidChangeParams {
    #[serde(rename = "textDocument")]
    text_document: VersionedDoc,
    #[serde(rename = "contentChanges")]
    content_changes: Vec<ChangeEvent>,
}
#[derive(Deserialize)]
struct VersionedDoc {
    uri: String,
}
#[derive(Deserialize)]
struct ChangeEvent {
    text: String,
}
#[derive(Serialize)]
struct Diag<'a> {
    range: Range,
    severity: u8,
    code: &'a str,
    source: &'a str,
    message: &'a str,
}
#[derive(Serialize)]
struct Range {
    start: Pos,
    end: Pos,
}
#[derive(Serialize, Clone, Copy)]
struct Pos {
    line: u32,
    character: u32,
}

impl Server {
    fn handle_open(&mut self, v: &Value) {
        if let Ok(p) = serde_json::from_value::<DidOpenParams>(v["params"].clone()) {
            self.docs.insert(p.text_document.uri, p.text_document.text);
        }
    }
    fn handle_change(&mut self, v: &Value) {
        if let Ok(p) = serde_json::from_value::<DidChangeParams>(v["params"].clone()) {
            if let Some(ch) = p.content_changes.into_iter().last() {
                self.docs.insert(p.text_document.uri, ch.text);
            }
        }
    }
    fn handle_save(&mut self, _v: &Value) {}

    fn publish(&self, out: &mut impl Write) -> anyhow::Result<()> {
        for (uri, src) in &self.docs {
            let parsed = parse(src, Dialect::Postgres);
            let viols = self.registry.run(&parsed, &self.config);
            let diags: Vec<Diag> = viols
                .iter()
                .map(|v| Diag {
                    range: Range {
                        start: Pos {
                            line: (v.line.saturating_sub(1)) as u32,
                            character: (v.col.saturating_sub(1)) as u32,
                        },
                        end: Pos {
                            line: (v.line.saturating_sub(1)) as u32,
                            character: (v.col.saturating_sub(1)) as u32 + 1,
                        },
                    },
                    severity: match v.severity {
                        Severity::Error => 1,
                        Severity::Warning => 2,
                        Severity::Info => 3,
                        Severity::Off => 4,
                    },
                    code: v.rule_id,
                    source: "drift",
                    message: &v.message,
                })
                .collect();
            let note = json!({
                "jsonrpc": "2.0",
                "method": "textDocument/publishDiagnostics",
                "params": {
                    "uri": uri,
                    "diagnostics": diags,
                }
            });
            write_message(out, &note.to_string())?;
        }
        Ok(())
    }
}

fn read_message<R: BufRead>(r: &mut R) -> anyhow::Result<Option<String>> {
    let mut content_len: Option<usize> = None;
    let mut line = String::new();
    loop {
        line.clear();
        let n = r.read_line(&mut line)?;
        if n == 0 {
            return Ok(None);
        }
        let trimmed = line.trim_end();
        if trimmed.is_empty() {
            break;
        }
        if let Some(rest) = trimmed.strip_prefix("Content-Length: ") {
            content_len = rest.parse().ok();
        }
    }
    let len = match content_len {
        Some(l) => l,
        None => return Ok(None),
    };
    let mut buf = vec![0u8; len];
    std::io::Read::read_exact(r, &mut buf)?;
    Ok(Some(String::from_utf8(buf)?))
}

fn write_message<W: Write>(w: &mut W, body: &str) -> anyhow::Result<()> {
    write!(w, "Content-Length: {}\r\n\r\n{}", body.len(), body)?;
    w.flush()?;
    Ok(())
}
