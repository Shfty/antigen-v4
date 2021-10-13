use std::{collections::BTreeMap, path::Path};

/// A map of VID/PID to Report Descriptor bytes
///
/// Used to provide a report descriptor for known devices
#[derive(Debug, Default, Clone)]
pub struct ReportDescriptorData(BTreeMap<(u16, u16), Vec<u8>>);

impl ReportDescriptorData {
    pub fn insert(&mut self, vid: u16, pid: u16, report_desc: &[u8]) -> Option<Vec<u8>> {
        self.0
            .insert((vid, pid), report_desc.iter().copied().collect())
    }

    pub fn get(&self, vid: u16, pid: u16) -> Option<&Vec<u8>> {
        self.0.get(&(vid, pid))
    }

    pub fn dump(&self, path: &Path) -> std::io::Result<()> {
        for ((vid, pid), report) in &self.0 {
            std::fs::write(
                path.join(Path::new(&format!("VID_{:04x}_PID_{:04x}.hrd", vid, pid))),
                report,
            )?;
        }
        Ok(())
    }
}
