//! Named outputs of one source-host compiler command. Input object copies are
//! deliberately absent. Reuse the contained executor's polling and final reap;
//! no second process runner, sampler thread, or persistent quota state.
use ace_source_query::resources::{Charge, ExternalOutputPermit, Reservation, Resources};
use std::{io, path::Path};

pub(crate) struct Outputs<'a> {
    files: Vec<(&'a Path, u64)>,
    permit: Option<ExternalOutputPermit>,
    _allocation: Reservation,
}
fn length(path: &Path) -> Result<Option<u64>, String> {
    let metadata = match std::fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(format!("compiler output metadata: {e}")),
    };
    use std::os::windows::fs::MetadataExt;
    if !metadata.is_file() || metadata.file_attributes() & 0x400 != 0 {
        return Err("compiler output is not a regular, non-reparse file".into());
    }
    Ok(Some(metadata.len()))
}
impl<'a> Outputs<'a> {
    /// These are the source host's mutable compiler output slots, not accepted
    /// cache artifacts. Clear stale slots before the producer so a failed
    /// command cannot count or consume a previous successful build's files.
    pub(crate) fn new(
        resources: &Resources,
        paths: impl ExactSizeIterator<Item = &'a Path> + Clone,
    ) -> Result<Self, String> {
        let path_units = paths
            .clone()
            .try_fold(0usize, |sum, path| sum.checked_add(path.as_os_str().len()))
            .ok_or("compiler output path size overflow")?;
        // Include native path conversion scratch used by metadata/removal,
        // before the first filesystem call, as well as the retained Vec.
        let allocation = resources
            .reserve(Charge::index(
                paths
                    .len()
                    .checked_mul(std::mem::size_of::<(&Path, u64)>() + 256)
                    .and_then(|n| path_units.checked_mul(8)?.checked_add(n))
                    .ok_or("compiler output index overflow")?,
            ))
            .map_err(|e| e.to_string())?;
        let permit = resources
            .reserve_external_output()
            .map_err(|e| e.to_string())?;
        let mut files = Vec::new();
        files
            .try_reserve_exact(paths.len())
            .map_err(|e| e.to_string())?;
        for path in paths {
            if length(path)?.is_some() {
                std::fs::remove_file(path).map_err(|e| format!("stale compiler output: {e}"))?;
            }
            files.push((path, 0));
        }
        Ok(Self {
            files,
            permit: Some(permit),
            _allocation: allocation,
        })
    }
    pub(crate) fn observe(&mut self) -> Result<(), String> {
        let mut error = None;
        let mut total = 0u64;
        let mut largest = 0;
        for (path, peak) in &mut self.files {
            match length(path) {
                Ok(Some(bytes)) => *peak = (*peak).max(bytes),
                Ok(None) => (),
                Err(message) => error = Some(message),
            }
            total = total.saturating_add(*peak);
            largest = largest.max(*peak);
        }
        let quota = self
            .permit
            .as_mut()
            .unwrap()
            .observe(total, largest)
            .map_err(|e| e.to_string());
        match error {
            Some(message) => Err(message),
            None => quota,
        }
    }
    /// Call after either success or contained failure, including stream caps
    /// and deadlines. Failed/discarded output remains cumulative request spend.
    pub(crate) fn finish(mut self) -> Result<(), String> {
        let observed = self.observe();
        let completed = self
            .permit
            .take()
            .unwrap()
            .finish()
            .map_err(|e| e.to_string());
        observed.and(completed)
    }
}
impl Drop for Outputs<'_> {
    fn drop(&mut self) {
        if self.permit.is_some() {
            let _ = self.observe();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ace_core::link::{CaptureCaps, SubprocError};
    use ace_source_query::resources::OutputLimits;
    use std::{
        process::Command,
        time::{Duration, Instant},
    };
    #[test]
    fn external_outputs_keep_deleted_spend_and_reap_quota_and_deadline_failures() {
        let root = std::path::PathBuf::from("D:/autostack/plan-artifacts/plan-938/tests")
            .join(format!("external-output-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let path = root.join("output.obj");
        let resources = Resources::default();
        let baseline = resources.stats().current;
        resources
            .enable_output_budget(OutputLimits {
                artifact: 32768,
                request: 65536,
            })
            .unwrap();
        let caps = CaptureCaps {
            stdout_max: 64,
            stderr_max: 64,
        };
        for count in 1..=2 {
            let mut outputs = Outputs::new(&resources, std::iter::once(path.as_path())).unwrap();
            let mut command = Command::new("python");
            command.args(["-c", "import pathlib,sys,time; p=pathlib.Path(sys.argv[1]); p.write_bytes(b'x'*32768); time.sleep(.25); p.unlink()"])
                .arg(&path);
            let run = crate::ctool_capture::run_observed(
                &mut command,
                Duration::from_secs(5),
                caps,
                Some(&resources),
                || outputs.observe(),
            )
            .unwrap();
            assert_eq!(run.value.exit_code, Some(0));
            assert!(!path.exists());
            outputs.finish().unwrap();
            drop(run);
            assert_eq!(
                resources.output_stats().unwrap().produced,
                count * 32768,
                "terminal-only observation lost deleted production"
            );
            assert_eq!(resources.output_stats().unwrap().pending, 0);
            assert_eq!(resources.stats().current, baseline);
        }
        assert!(Outputs::new(&resources, std::iter::once(path.as_path())).is_err());
        for quota in [false, true] {
            let resources = Resources::default();
            resources
                .enable_output_budget(OutputLimits {
                    artifact: if quota { 65535 } else { 65536 },
                    request: 131072,
                })
                .unwrap();
            let mut outputs = Outputs::new(&resources, std::iter::once(path.as_path())).unwrap();
            let mut command = Command::new("python");
            command.args(["-c", "import sys,time; f=open(sys.argv[1],'wb'); f.write(b'x'*65536); f.flush(); time.sleep(30)"]).arg(&path);
            let start = Instant::now();
            let run = crate::ctool_capture::run_observed(
                &mut command,
                Duration::from_secs(1),
                caps,
                Some(&resources),
                || outputs.observe(),
            );
            let finish = outputs.finish();
            if quota {
                assert!(matches!(
                    run,
                    Err(crate::ctool_capture::CaptureError::Resource(_))
                ));
                assert!(finish.is_err());
                assert!(resources.output_stats().unwrap().refused);
            } else {
                assert!(matches!(
                    run,
                    Err(crate::ctool_capture::CaptureError::Subprocess(
                        SubprocError::Deadline
                    ))
                ));
                finish.unwrap();
                assert!(!resources.output_stats().unwrap().refused);
            }
            assert!(start.elapsed() < Duration::from_secs(5));
            assert_eq!(resources.output_stats().unwrap().produced, 65536);
            assert_eq!(resources.output_stats().unwrap().pending, 0);
            // Python held this real file open through its 30-second sleep.
            // Successful delete proves the contained tree closed the handle.
            std::fs::remove_file(&path).unwrap();
            assert_eq!(resources.stats().current, baseline);
        }
        // Metadata admission precedes either Vec allocation or stale clearing.
        std::fs::write(&path, b"previous output").unwrap();
        let resources = Resources::default();
        let pressure = resources
            .reserve(Charge::index((8 << 20) - resources.stats().current.index))
            .unwrap();
        assert!(Outputs::new(&resources, std::iter::once(path.as_path())).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), b"previous output");
        drop(pressure);
    }
}
