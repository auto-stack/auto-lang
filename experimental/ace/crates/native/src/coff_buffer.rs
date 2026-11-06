//! Exact COFF output reservation at object::write's real allocation boundary.
use crate::{config::MAX_COFF_BYTES, MnError};
use ace_source_query::resources::{Charge, OutputPermit, Reservation, Resources};
use cranelift_object::object::write::{Object, WritableBuffer};

struct Output<'a> {
    // Payload must be destroyed before its reservation on any failure path.
    bytes: Vec<u8>,
    reservation: Option<Reservation>,
    resources: Option<&'a Resources>,
    expected: Option<usize>,
    position: usize,
    failure: Option<MnError>,
    production: Option<OutputPermit>,
}
impl WritableBuffer for Output<'_> {
    fn len(&self) -> usize {
        self.position
    }
    fn reserve(&mut self, size: usize) -> Result<(), ()> {
        if self.expected.is_some() {
            self.failure = Some(MnError::Backend("COFF writer reserved twice".into()));
            return Err(());
        }
        self.expected = Some(size);
        if size > MAX_COFF_BYTES {
            self.failure = Some(MnError::Resource(
                "object-size",
                format!("COFF needs {size} bytes, above the frozen {MAX_COFF_BYTES} cap"),
            ));
            return Err(());
        }
        if let Some(resources) = self.resources {
            match resources.reserve_output(size) {
                Ok(ticket) => self.production = Some(ticket),
                Err(e) => {
                    self.failure = Some(MnError::Resource("request-output", e.to_string()));
                    return Err(());
                }
            }
            match resources.reserve(Charge::staging(size)) {
                Ok(ticket) => self.reservation = Some(ticket),
                Err(e) => {
                    self.failure = Some(MnError::Resource("managed-memory", e.to_string()));
                    return Err(());
                }
            }
        }
        if let Err(e) = self.bytes.try_reserve_exact(size) {
            self.failure = Some(MnError::Resource("object-allocation", e.to_string()));
            return Err(());
        }
        Ok(())
    }
    fn resize(&mut self, new_len: usize) {
        if self.failure.is_none() {
            if new_len < self.position || self.expected.is_none_or(|n| new_len > n) {
                self.failure = Some(MnError::Backend(
                    "COFF writer exceeded its reservation".into(),
                ));
            } else {
                self.bytes.resize(new_len, 0);
            }
        }
        self.position = new_len;
    }
    fn write_bytes(&mut self, bytes: &[u8]) {
        let end = self.position.checked_add(bytes.len());
        if self.failure.is_none() {
            if end.is_none_or(|end| self.expected.is_none_or(|n| end > n)) {
                self.failure = Some(MnError::Backend(
                    "COFF writer exceeded its reservation".into(),
                ));
            } else {
                self.bytes.extend_from_slice(bytes);
            }
        }
        self.position = end.unwrap_or(usize::MAX);
    }
}

pub(super) fn emit(
    object: &Object<'_>,
    resources: Option<&Resources>,
) -> Result<(Vec<u8>, Option<Reservation>), MnError> {
    let mut output = Output {
        bytes: Vec::new(),
        reservation: None,
        resources,
        expected: None,
        position: 0,
        failure: None,
        production: None,
    };
    let result = object.emit(&mut output);
    if let Some(failure) = output.failure.take() {
        return Err(failure);
    }
    result.map_err(|e| MnError::Backend(format!("emit object: {e}")))?;
    if output.expected != Some(output.bytes.len()) {
        return Err(MnError::Backend(
            "COFF writer did not fill its declared output".into(),
        ));
    }
    if let Some(production) = output.production.take() {
        production
            .complete(output.bytes.len())
            .map_err(|e| MnError::Resource("request-output", e.to_string()))?;
    }
    Ok((output.bytes, output.reservation))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ace_source_query::resources::Limits;
    use cranelift_object::object::{Architecture, BinaryFormat, Endianness, SectionKind};

    #[test]
    fn real_coff_output_is_byte_identical_and_reserved_before_materialization() {
        let mut object = Object::new(BinaryFormat::Coff, Architecture::X86_64, Endianness::Little);
        let section = object.add_section(Vec::new(), b".text".to_vec(), SectionKind::Text);
        object.section_mut(section).set_data(vec![0xcc; 4_096], 16);
        let expected = object.write().unwrap();
        let resources = Resources::default();
        let (bytes, reservation) = emit(&object, Some(&resources)).unwrap();
        assert_eq!(bytes, expected);
        assert_eq!(bytes.len(), bytes.capacity());
        assert_eq!(resources.stats().current.staging, bytes.capacity());
        drop(bytes);
        drop(reservation);
        assert_eq!(resources.stats().current.staging, 0);
        let tight = Resources::limited(Limits {
            staging: 4_096,
            ..Limits::default()
        })
        .unwrap();
        assert!(matches!(
            emit(&object, Some(&tight)),
            Err(MnError::Resource("managed-memory", _))
        ));
        assert_eq!(tight.stats().current.staging, 0);
        assert_eq!(tight.stats().rejections, 1);
        // The original hard cap also applies without a composing host account.
        let mut oversized =
            Object::new(BinaryFormat::Coff, Architecture::X86_64, Endianness::Little);
        let section = oversized.add_section(Vec::new(), b".text".to_vec(), SectionKind::Text);
        oversized
            .section_mut(section)
            .set_data(vec![0xcc; MAX_COFF_BYTES], 16);
        assert!(matches!(
            emit(&oversized, None),
            Err(MnError::Resource("object-size", _))
        ));
    }
}
