//! Granular file extraction submodules.

pub mod globals;
pub mod types;

use rs_teststand::sequence::SequenceFile;

use self::globals::{extract_file_globals, extract_station_globals};
use self::types::extract_file_types;
use crate::data::{ExtractorConfig, FileData, SequenceData};
use crate::extraction::sequence::extract_sequence;
use crate::rendering::markdown::display_name;

struct FileMetadata {
    file_version: String,
    model_file: String,
    load_opt: String,
    unload_opt: String,
    requirements: Vec<String>,
}

fn extract_file_metadata(seq_file: &SequenceFile) -> FileMetadata {
    let mut file_version = String::new();
    let mut model_file = String::new();
    let mut load_opt = String::new();
    let mut unload_opt = String::new();
    let mut requirements = Vec::new();

    if let Ok(po_file) = seq_file.as_property_object_file() {
        if let Ok(v) = po_file.version() {
            file_version = v;
        }
        if let Ok(po_data) = po_file.data() {
            if file_version.is_empty() {
                if let Ok(v) = po_data.get_val_string("Version", 0x1) {
                    file_version = v;
                }
            }
            if let Ok(mf) = po_data.get_val_string("ModelFile", 0x1) {
                model_file = mf;
            }
            if let Ok(lo) = po_data.get_val_string("LoadOpt", 0x1) {
                load_opt = lo;
            }
            if let Ok(uo) = po_data.get_val_string("UnloadOpt", 0x1) {
                unload_opt = uo;
            }
            if let Ok(req_po) = po_data.get_property_object("Requirements.Links", 0x1) {
                let count = req_po.get_num_elements().unwrap_or(0);
                for i in 0..count {
                    if let Ok(elem) = req_po.get_property_object_by_offset(i, 0) {
                        if let Ok(req) = elem.get_val_string("Requirement", 0) {
                            if !req.is_empty() {
                                requirements.push(req);
                            }
                        }
                    }
                }
            }
        }
    }

    FileMetadata {
        file_version,
        model_file,
        load_opt,
        unload_opt,
        requirements,
    }
}

fn compute_file_delay(sequences: &[SequenceData]) -> Option<f64> {
    let mut total_delay = 0.0;
    let mut found_delay = false;
    for seq in sequences {
        if let Some(delay) = seq.estimated_software_delay {
            total_delay += delay;
            found_delay = true;
        }
    }
    if found_delay { Some(total_delay) } else { None }
}

/// Extracts complete `FileData` from a live `SequenceFile` object.
#[must_use]
pub fn extract_file(
    seq_file: &SequenceFile,
    depth: usize,
    config: &ExtractorConfig,
    engine: Option<&rs_teststand::Engine>,
) -> FileData {
    let path = seq_file.path().unwrap_or_default();
    let name = if path.is_empty() {
        "Untitled.seq".to_owned()
    } else {
        display_name(&path)
    };

    let num_seqs = seq_file.num_sequences().unwrap_or(0);
    let cap = usize::try_from(num_seqs.max(0)).unwrap_or_default();
    let mut sequences = Vec::with_capacity(cap);

    for idx in 0..num_seqs {
        if let Ok(seq) = seq_file.get_sequence(idx) {
            if let Ok(seq_data) = extract_sequence(&seq, config) {
                sequences.push(seq_data);
            }
        }
    }

    let file_globals = extract_file_globals(seq_file);
    let station_globals = engine.map_or_else(Vec::new, extract_station_globals);
    let custom_data_types = extract_file_types(seq_file);
    let estimated_software_delay = compute_file_delay(&sequences);
    let meta = extract_file_metadata(seq_file);

    FileData {
        name,
        path,
        sequences,
        depth,
        file_globals,
        station_globals,
        file_version: meta.file_version,
        model_file: meta.model_file,
        load_opt: meta.load_opt,
        unload_opt: meta.unload_opt,
        requirements: meta.requirements,
        custom_data_types,
        estimated_software_delay,
    }
}
