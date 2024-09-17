use crate::data_loader::sierra_program::{GetDebugInfos, SierraProgram};
use anyhow::{Context, Result};
use cairo_lang_sierra::debug_info::DebugInfo;
use cairo_lang_sierra_to_casm::compiler::CairoProgramDebugInfo;
use camino::Utf8PathBuf;
use derived_deref::Deref;
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use std::fs;
use trace_data::{CallTrace, CallTraceNode, CasmLevelInfo};

#[derive(Deref)]
pub struct LoadedDataMap(HashMap<Utf8PathBuf, LoadedData>);

pub struct LoadedData {
    pub debug_info: DebugInfo,
    pub casm_level_infos: Vec<CasmLevelInfo>,
    pub casm_debug_info: CairoProgramDebugInfo,
}

impl LoadedDataMap {
    pub fn load(call_trace_paths: &[Utf8PathBuf]) -> Result<Self> {
        let execution_infos = call_trace_paths
            .iter()
            .map(read_and_deserialize)
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .flat_map(load_nested_traces)
            .filter_map(|call_trace| call_trace.cairo_execution_info)
            .collect::<Vec<_>>();

        // OPTIMIZATION:
        // Group execution info by source Sierra path
        // so that the same Sierra program does not need to be deserialized multiple times.
        let execution_infos_by_sierra_path = execution_infos.into_iter().fold(
            HashMap::new(),
            |mut acc: HashMap<_, Vec<_>>, execution_info| {
                acc.entry(execution_info.source_sierra_path)
                    .or_default()
                    .push(execution_info.casm_level_info);
                acc
            },
        );

        Ok(Self(
            execution_infos_by_sierra_path
                .into_iter()
                .map(|(source_sierra_path, casm_level_infos)| {
                    read_and_deserialize::<SierraProgram>(&source_sierra_path)?
                        .compile_and_get_debug_infos()
                        .map(|(debug_info, casm_debug_info)| LoadedData {
                            debug_info,
                            casm_level_infos,
                            casm_debug_info,
                        })
                        .context(format!(
                            "Error occurred while loading program from: {source_sierra_path}"
                        ))
                        .map(|loaded_data| (source_sierra_path, loaded_data))
                })
                .collect::<Result<_>>()?,
        ))
    }
}

fn load_nested_traces(call_trace: CallTrace) -> Vec<CallTrace> {
    fn load_recursively(call_trace: CallTrace, acc: &mut Vec<CallTrace>) {
        acc.push(call_trace.clone());
        for call_trace_node in call_trace.nested_calls {
            if let CallTraceNode::EntryPointCall(nested_call_trace) = call_trace_node {
                load_recursively(nested_call_trace, acc);
            }
        }
    }

    let mut call_traces = Vec::new();
    load_recursively(call_trace, &mut call_traces);
    call_traces
}

fn read_and_deserialize<T: DeserializeOwned>(file_path: &Utf8PathBuf) -> Result<T> {
    fs::read_to_string(file_path)
        .context(format!("Failed to read file at path: {file_path}"))
        .and_then(|content| {
            serde_json::from_str(&content).context(format!(
                "Failed to deserialize JSON content from file at path: {file_path}"
            ))
        })
}
