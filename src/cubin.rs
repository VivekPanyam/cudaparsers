use deku::prelude::*;
use goblin::Object;

/// An item in an .nv.info.* section
#[derive(Debug, PartialEq, DekuRead)]
pub struct NVInfoItem {
    pub format: NVInfoFormat,
    pub attribute: NVInfoAttribute,
    #[deku(ctx = "*format, *attribute")]
    pub value: NVInfoValue,
}

#[derive(Debug, PartialEq, DekuRead)]
#[deku(
    ctx = "format: NVInfoFormat, attribute: NVInfoAttribute",
    id = "format"
)]
pub enum NVInfoValue {
    #[deku(id = "NVInfoFormat::EIFMT_NVAL")]
    NoValue(u16),

    #[deku(id = "NVInfoFormat::EIFMT_BVAL")]
    BVal(u16), // This should be u8 I think

    #[deku(id = "NVInfoFormat::EIFMT_HVAL")]
    HVal(u16),

    #[deku(id = "NVInfoFormat::EIFMT_SVAL")]
    SVal(#[deku(ctx = "attribute")] NVInfoSval),
}

#[derive(Debug, PartialEq, DekuRead)]
#[deku(ctx = "attribute: NVInfoAttribute")]
pub struct NVInfoSval {
    // The size in bytes
    value_size: u16,

    #[deku(ctx = "attribute, *value_size")]
    pub value: NVInfoSvalValue,
}

#[deku_derive(DekuRead)]
#[derive(Debug, PartialEq)]
#[deku(ctx = "attribute: NVInfoAttribute, value_size: u16", id = "attribute")]
pub enum NVInfoSvalValue {
    #[deku(id = "NVInfoAttribute::EIATTR_KPARAM_INFO")]
    KParamInfoValue {
        #[deku(assert = "value_size == 12")]
        index: u32,

        ordinal: u16,
        offset: u16,

        // Deku only reads bits in msb so this is a little hacky since the data
        // is lsb
        // Read into a temp variable and then extract the bits manually
        #[deku(temp)]
        tmp: u32,

        #[deku(skip, default = "(tmp & 0xff) as _")]
        log_alignment: u8,

        #[deku(skip, default = "((tmp >> 0x8) & 0xf) as _")]
        space: u8,

        #[deku(skip, default = "((tmp >> 0xc) & 0x1f) as _")]
        cbank: u8,

        #[deku(skip, default = "((tmp >> 0x10) & 2) == 0")]
        is_cbank: bool,

        #[deku(skip, default = "(((tmp >> 0x10) & 0xffff) >> 2) as _")]
        size_bytes: u16,
    },

    #[deku(id = "NVInfoAttribute::EIATTR_EXTERNS")]
    ExternSValue {
        #[deku(assert = "value_size == 4")]
        index: u32,

        #[deku(skip)]
        value: String,
    },

    #[deku(id_pat = "_")]
    Other {
        #[deku(bytes_read = "value_size")]
        data: Vec<u32>,
    },
}

#[derive(Debug, PartialEq, DekuRead, Clone, Copy)]
#[deku(type = "u8")]
#[allow(non_camel_case_types)]
pub enum NVInfoFormat {
    EIFMT_NVAL = 0x01,
    EIFMT_BVAL,
    EIFMT_HVAL,
    EIFMT_SVAL,
}

#[derive(Debug, PartialEq, DekuRead, Clone, Copy)]
#[deku(type = "u8")]
#[allow(non_camel_case_types)]
pub enum NVInfoAttribute {
    EIATTR_ERROR = 0x00,
    EIATTR_PAD,
    EIATTR_IMAGE_SLOT,
    EIATTR_JUMPTABLE_RELOCS,
    EIATTR_CTAIDZ_USED,
    EIATTR_MAX_THREADS,
    EIATTR_IMAGE_OFFSET,
    EIATTR_IMAGE_SIZE,
    EIATTR_TEXTURE_NORMALIZED,
    EIATTR_SAMPLER_INIT,
    EIATTR_PARAM_CBANK,
    EIATTR_SMEM_PARAM_OFFSETS,
    EIATTR_CBANK_PARAM_OFFSETS,
    EIATTR_SYNC_STACK,
    EIATTR_TEXID_SAMPID_MAP,
    EIATTR_EXTERNS,
    EIATTR_REQNTID,
    EIATTR_FRAME_SIZE,
    EIATTR_MIN_STACK_SIZE,
    EIATTR_SAMPLER_FORCE_UNNORMALIZED,
    EIATTR_BINDLESS_IMAGE_OFFSETS,
    EIATTR_BINDLESS_TEXTURE_BANK,
    EIATTR_BINDLESS_SURFACE_BANK,
    EIATTR_KPARAM_INFO,
    EIATTR_SMEM_PARAM_SIZE,
    EIATTR_CBANK_PARAM_SIZE,
    EIATTR_QUERY_NUMATTRIB,
    EIATTR_MAXREG_COUNT,
    EIATTR_EXIT_INSTR_OFFSETS,
    EIATTR_S2RCTAID_INSTR_OFFSETS,
    EIATTR_CRS_STACK_SIZE,
    EIATTR_NEED_CNP_WRAPPER,
    EIATTR_NEED_CNP_PATCH,
    EIATTR_EXPLICIT_CACHING,
    EIATTR_ISTYPEP_USED,
    EIATTR_MAX_STACK_SIZE,
    EIATTR_SUQ_USED,
    EIATTR_LD_CACHEMOD_INSTR_OFFSETS,
    EIATTR_LOAD_CACHE_REQUEST,
    EIATTR_ATOM_SYS_INSTR_OFFSETS,
    EIATTR_COOP_GROUP_INSTR_OFFSETS,
    EIATTR_COOP_GROUP_MAX_REGIDS,
    EIATTR_SW1850030_WAR,
    EIATTR_WMMA_USED,
    EIATTR_HAS_PRE_V10_OBJECT,
    EIATTR_ATOMF16_EMUL_INSTR_OFFSETS,
    EIATTR_ATOM16_EMUL_INSTR_REG_MAP,
    EIATTR_REGCOUNT,
    EIATTR_SW2393858_WAR,
    EIATTR_INT_WARP_WIDE_INSTR_OFFSETS,
    EIATTR_SHARED_SCRATCH,
    EIATTR_STATISTICS,

    // New between cuda 10.2 and 11.6
    EIATTR_INDIRECT_BRANCH_TARGETS,
    EIATTR_SW2861232_WAR,
    EIATTR_SW_WAR,
    EIATTR_CUDA_API_VERSION,
    EIATTR_NUM_MBARRIERS,
    EIATTR_MBARRIER_INSTR_OFFSETS,
    EIATTR_COROUTINE_RESUME_ID_OFFSETS,
    EIATTR_SAM_REGION_STACK_SIZE,
    EIATTR_PER_REG_TARGET_PERF_STATS,

    // New between cuda 11.6 and 11.8
    EIATTR_CTA_PER_CLUSTER,
    EIATTR_EXPLICIT_CLUSTER,
    EIATTR_MAX_CLUSTER_RANK,
    EIATTR_INSTR_REG_MAP,
}

pub fn parse(data: &[u8]) -> Result<Vec<(String, Vec<NVInfoItem>)>, deku::DekuError> {
    if let Object::Elf(elf) = Object::parse(data).unwrap() {
        let mut out: Vec<(String, Vec<NVInfoItem>)> = Vec::new();

        for section in elf.section_headers {
            let section_name = &elf.shdr_strtab[section.sh_name];

            if !section_name.starts_with(".nv.info.") {
                // Only looking at function info
                continue;
            }

            let frange = section.file_range().unwrap();
            let mut bit_offset = 0;
            let mut section_data = &data[frange.start..frange.end];

            log::trace!("SECTION: {}", section_name);

            let mut output_sections = Vec::new();
            while section_data.len() > 0 {
                let (a, mut parsed) = NVInfoItem::from_bytes((section_data, bit_offset))?;

                // Get symbol names if we need to
                if parsed.attribute == NVInfoAttribute::EIATTR_EXTERNS {
                    if let NVInfoValue::SVal(value) = &mut parsed.value {
                        if let NVInfoSvalValue::ExternSValue { index, value } = &mut value.value {
                            let symnameidx = elf.syms.get(*index as _).unwrap().st_name;

                            *value = elf.strtab[symnameidx].to_string();
                        } else {
                            panic!("Error in parsing - expected ExternSValue");
                        }
                    } else {
                        panic!("Error in parsing - expected SVal");
                    }
                }

                (section_data, bit_offset) = a;
                log::trace!("{:#?}", &parsed);
                output_sections.push(parsed);
            }

            out.push((section_name.to_string(), output_sections));
        }

        return Ok(out);
    }

    panic!("Failed parsing elf file");
}
