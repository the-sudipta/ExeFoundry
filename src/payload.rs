use anyhow::{Context, Result, bail};

pub const PACKAGE_MAGIC: &[u8; 8] = b"EXFDPK1\0";
pub const FOOTER_MAGIC: &[u8; 16] = b"EXEFOUNDRY_V1\0\0\0";
pub const FOOTER_LEN: usize = 24;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct PayloadHeader {
    pub flags: u32,
    pub bat_len: u64,
}

pub fn build_package(bat: &[u8], flags: u32) -> Vec<u8> {
    let mut out = Vec::with_capacity(PACKAGE_MAGIC.len() + 4 + 8 + bat.len());
    out.extend_from_slice(PACKAGE_MAGIC);
    out.extend_from_slice(&flags.to_le_bytes());
    out.extend_from_slice(&(bat.len() as u64).to_le_bytes());
    out.extend_from_slice(bat);
    out
}

pub fn append_package(exe: &mut Vec<u8>, package: &[u8]) {
    exe.extend_from_slice(package);
    exe.extend_from_slice(&(package.len() as u64).to_le_bytes());
    exe.extend_from_slice(FOOTER_MAGIC);
}

pub fn extract_package(exe: &[u8]) -> Result<PayloadHeaderAndBytes<'_>> {
    if exe.len() < FOOTER_LEN {
        bail!("this executable does not contain an ExeFoundry payload");
    }

    let footer_start = exe.len() - FOOTER_LEN;
    let (package_len_bytes, magic) = exe[footer_start..].split_at(8);
    if magic != FOOTER_MAGIC {
        bail!("this executable does not contain an ExeFoundry payload");
    }

    let package_len = u64::from_le_bytes(
        package_len_bytes
            .try_into()
            .context("invalid ExeFoundry footer")?,
    ) as usize;

    if package_len > footer_start {
        bail!("invalid ExeFoundry payload length");
    }

    let package_start = footer_start - package_len;
    let package = &exe[package_start..footer_start];
    if package.len() < PACKAGE_MAGIC.len() + 12 || &package[..PACKAGE_MAGIC.len()] != PACKAGE_MAGIC
    {
        bail!("invalid ExeFoundry payload package");
    }

    let flags_offset = PACKAGE_MAGIC.len();
    let bat_len_offset = flags_offset + 4;
    let bat_offset = bat_len_offset + 8;
    let flags = u32::from_le_bytes(
        package[flags_offset..bat_len_offset]
            .try_into()
            .context("invalid ExeFoundry payload flags")?,
    );
    let bat_len = u64::from_le_bytes(
        package[bat_len_offset..bat_offset]
            .try_into()
            .context("invalid ExeFoundry BAT length")?,
    ) as usize;

    if bat_offset + bat_len != package.len() {
        bail!("ExeFoundry payload size does not match header");
    }

    Ok(PayloadHeaderAndBytes {
        header: PayloadHeader {
            flags,
            bat_len: bat_len as u64,
        },
        bat: &package[bat_offset..],
    })
}

pub struct PayloadHeaderAndBytes<'a> {
    pub header: PayloadHeader,
    pub bat: &'a [u8],
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_round_trip() {
        let bat = b"@echo off\r\necho hello\r\n";
        let mut exe = b"MZ fake template".to_vec();
        let package = build_package(bat, 7);
        append_package(&mut exe, &package);

        let extracted = extract_package(&exe).unwrap();
        assert_eq!(extracted.header.flags, 7);
        assert_eq!(extracted.header.bat_len, bat.len() as u64);
        assert_eq!(extracted.bat, bat);
    }
}
