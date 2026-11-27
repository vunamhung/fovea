use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::error::{CoreError, CoreResult};
use crate::schema::{ProjectDoc, SCHEMA_VERSION};
use crate::telemetry::{TelemetryEvent, TelemetryReadReport};

#[derive(Deserialize)]
struct SchemaProbe {
    schema_version: u32,
}

#[derive(Debug, Clone)]
pub struct FoveaBundle {
    root: PathBuf,
    project: ProjectDoc,
}

impl FoveaBundle {
    pub fn create(
        root: impl AsRef<Path>,
        title: impl Into<String>,
        width: u32,
        height: u32,
        fps: u32,
    ) -> CoreResult<Self> {
        let root = root.as_ref().to_path_buf();
        fs::create_dir_all(root.join("voices"))?;
        let project = ProjectDoc::new(title, width, height, fps);
        let bundle = Self { root, project };
        bundle.save_project()?;
        if !bundle.telemetry_path().exists() {
            File::create(bundle.telemetry_path())?;
        }
        Ok(bundle)
    }

    pub fn open(root: impl AsRef<Path>) -> CoreResult<Self> {
        let root = root.as_ref().to_path_buf();
        let json_path = root.join("project.json");
        let raw = fs::read_to_string(&json_path)?;
        let probe: SchemaProbe = serde_json::from_str(&raw)?;
        if probe.schema_version != SCHEMA_VERSION {
            return Err(CoreError::UnsupportedSchemaVersion(probe.schema_version));
        }
        let project: ProjectDoc = serde_json::from_str(&raw)?;
        Ok(Self { root, project })
    }

    pub fn save_project(&self) -> CoreResult<()> {
        let target = self.project_json_path();
        let tmp = self.root.join("project.json.tmp");
        let body = serde_json::to_string_pretty(&self.project)?;
        fs::write(&tmp, body)?;
        fs::rename(&tmp, &target)?;
        Ok(())
    }

    pub fn project(&self) -> &ProjectDoc {
        &self.project
    }

    pub fn project_mut(&mut self) -> &mut ProjectDoc {
        &mut self.project
    }

    pub fn set_project(&mut self, project: ProjectDoc) {
        self.project = project;
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn project_json_path(&self) -> PathBuf {
        self.root.join("project.json")
    }

    pub fn telemetry_path(&self) -> PathBuf {
        self.root.join("telemetry.jsonl")
    }

    pub fn screen_video_path(&self) -> PathBuf {
        self.root.join(&self.project.source.screen_video)
    }

    pub fn webcam_video_path(&self) -> PathBuf {
        self.root.join("webcam.mov")
    }

    pub fn system_audio_path(&self) -> PathBuf {
        self.root.join("system.caf")
    }

    pub fn mic_audio_path(&self) -> PathBuf {
        self.root.join("mic.caf")
    }

    pub fn voices_dir(&self) -> PathBuf {
        self.root.join("voices")
    }

    pub fn append_telemetry(&self, event: &TelemetryEvent) -> CoreResult<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.telemetry_path())?;
        let line = serde_json::to_string(event)?;
        writeln!(file, "{line}")?;
        Ok(())
    }

    pub fn read_telemetry_lenient(&self) -> CoreResult<TelemetryReadReport> {
        let path = self.telemetry_path();
        if !path.exists() {
            return Ok(TelemetryReadReport {
                events: Vec::new(),
                skipped_lines: 0,
            });
        }
        let reader = BufReader::new(File::open(path)?);
        let mut events = Vec::new();
        let mut skipped_lines = 0;
        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            match serde_json::from_str::<TelemetryEvent>(&line) {
                Ok(ev) => events.push(ev),
                Err(_) => skipped_lines += 1,
            }
        }
        Ok(TelemetryReadReport {
            events,
            skipped_lines,
        })
    }
}
