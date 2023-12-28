use std::{
    collections::HashMap,
    os::raw::{c_uint, c_ulong, c_ushort},
};

use crate::cubin::NVInfoItem;

#[repr(C, align(8))]
#[derive(Debug)]
struct FatbinHeader {
    magic: c_uint,
    version: c_ushort,
    header_size: c_ushort,
    fat_size: c_ulong, // not including this header
}

// TODO: static assert size is 64 bytes
#[repr(C)]
#[derive(Debug)]
struct FatbinData {
    kind: c_ushort,
    version: c_ushort,
    header_size: c_uint,
    padded_payload_size: c_uint,
    unknown0: c_uint, // check if it's written into separately
    payload_size: c_uint,
    unknown1: c_uint,
    unknown2: c_uint,
    sm_version: c_uint,
    bit_width: c_uint,
    unknown3: c_uint,
    unknown4: c_ulong,
    unknown5: c_ulong,
    uncompressed_payload: c_ulong,
}

pub fn is_fatbin(fat_cubin: *const ::std::os::raw::c_void) -> bool {
    let header = unsafe { &*(fat_cubin as *const FatbinHeader) };

    // Check magic for the binary
    header.magic == 0xBA55ED50
}

pub fn get_fatbin_size(fat_cubin: *const ::std::os::raw::c_void) -> usize {
    let header = unsafe { &*(fat_cubin as *const FatbinHeader) };

    // Check magic for the binary
    assert!(header.magic == 0xBA55ED50);

    // Compute size bytes
    (header.header_size as usize) + (header.fat_size as usize)
}

// TODO: rewrite this to make parsing more safe
// This is directly ported from a parser I wrote in C++ so it's not great in terms
// of safety (and it's not idiomatic rust)
#[tracing::instrument]
pub unsafe fn parse(mut data_ptr: *const u8) -> HashMap<u32, HashMap<String, Vec<NVInfoItem>>> {
    // See https://gist.github.com/malfet/8990c577d61c1a46fa87e7d93b8dfdf8 for parsing

    let mut out: HashMap<u32, HashMap<String, Vec<NVInfoItem>>> = HashMap::new();

    // Get the header
    let header = &*(data_ptr as *const FatbinHeader);

    log::trace!("Got fatbinary header: {:#?}", header);
    assert!(header.magic == 0xBA55ED50 && header.version == 1);

    assert_eq!(std::mem::size_of::<FatbinData>(), 64);

    // Skip the rest of the header
    data_ptr = data_ptr.add(header.header_size as usize);

    // Compute the remaining size
    let mut remaining_size = header.fat_size;
    while remaining_size > 0 {
        let data = &*(data_ptr as *const FatbinData);

        log::trace!("NEW CUBIN: {:#?}", data);

        assert!(data.version == 0x0101 && (data.kind == 1 || data.kind == 2));
        log::trace!(
            "{}_{}",
            if data.kind == 1 { "ptx" } else { "sm" },
            data.sm_version
        );

        // Skip the rest of the data header
        data_ptr = data_ptr.add(data.header_size as usize);

        // Decompress if necessary
        let cubin = if data.uncompressed_payload != 0 {
            // TODO: make sure the payload is actually compressed
            let compressed_data = std::slice::from_raw_parts(data_ptr, data.payload_size as _);
            data_ptr = data_ptr.add(data.padded_payload_size as _);

            let out = lz4::block::decompress(compressed_data, Some(data.uncompressed_payload as _))
                .unwrap();

            if out.len() != data.uncompressed_payload as usize {
                panic!(
                    "Decompressed size {} does not match target {}",
                    out.len(),
                    data.uncompressed_payload
                );
            }

            out
        } else {
            let s = std::slice::from_raw_parts(data_ptr, data.padded_payload_size as _);

            data_ptr = data_ptr.add(data.padded_payload_size as _);

            // This isn't ideal...
            s.to_vec()
        };

        // TODO: otherwise compile PTX and then run this
        if data.kind == 2 {
            // Make sure the next few bytes are an elf header
            let elf_header = [0x7f, 'E' as _, 'L' as _, 'F' as _];
            assert_eq!(cubin[0..4], elf_header);

            // Parse the cubin and add it to our output map
            let parsed = crate::cubin::parse(&cubin[..]).unwrap();

            let to_fill = out.entry(data.sm_version).or_default();
            for (k, v) in parsed {
                to_fill.insert(k, v);
            }
        } else {
            log::trace!("Ignoring PTX...");
        }

        remaining_size -= data.header_size as u64 + data.padded_payload_size as u64;
    }

    out
}
