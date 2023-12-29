use std::fmt::Write;
use std::path::{Path, PathBuf};

use cuda_parsers::cubin::{NVInfoAttribute, NVInfoItem, NVInfoSvalValue, NVInfoValue};
use futures::StreamExt;

/// Display in the same format as cuobjdump
fn cuobjdump_fmt<T>(data: T) -> String
where
    T: IntoIterator<Item = (String, Vec<NVInfoItem>)>,
{
    let mut out = String::new();

    for (name, attrs) in data {
        writeln!(&mut out, "{name}").unwrap();

        for (i, attr) in attrs.iter().enumerate() {
            writeln!(&mut out, "\t<0x{}>", i + 1).unwrap();
            writeln!(&mut out, "\tAttribute:\t{:?}", attr.attribute).unwrap();
            writeln!(&mut out, "\tFormat:\t{:?}", attr.format).unwrap();

            match &attr.value {
                NVInfoValue::NoValue(_) => {}
                NVInfoValue::BVal(val) => writeln!(&mut out, "\tValue:\t{:#x}", val).unwrap(),
                NVInfoValue::HVal(val) => writeln!(&mut out, "\tValue:\t{:#x}", val).unwrap(),
                NVInfoValue::SVal(val) => match &val.value {
                    NVInfoSvalValue::KParamInfoValue {
                        index,
                        ordinal,
                        offset,
                        log_alignment,
                        space,
                        cbank,
                        is_cbank,
                        size_bytes,
                    } => {
                        let cbank_str = if *is_cbank { "CBANK" } else { "SMEM" };
                        writeln!(&mut out, "\tValue:\tIndex : {index:#x}\tOrdinal : {ordinal:#x}\tOffset  : {offset:#x}\tSize    : {size_bytes:#x}").unwrap();
                        writeln!(&mut out, "\t\tPointee's logAlignment : {log_alignment:#x}\tSpace : {space:#x}\tcbank : {cbank:#x}\tParameter Space : {cbank_str}\t").unwrap();
                    }
                    NVInfoSvalValue::ExternSValue { index, value } => {
                        writeln!(&mut out, "\tValue:\texterns:\t{value}({index:#x})\t").unwrap();
                    }
                    NVInfoSvalValue::Other { data }
                        if attr.attribute == NVInfoAttribute::EIATTR_ATOM16_EMUL_INSTR_REG_MAP =>
                    {
                        write!(&mut out, "\tValue:\t").unwrap();
                        assert!(data.len() % 2 == 0);

                        for item in data.chunks(2) {
                            write!(&mut out, "({:#x}, {})  ", item[0], item[1]).unwrap();
                        }
                        writeln!(&mut out).unwrap();
                    }
                    NVInfoSvalValue::Other { data } => {
                        write!(&mut out, "\tValue:\t").unwrap();
                        for item in data {
                            write!(&mut out, "{:#x} ", item).unwrap();
                        }
                        writeln!(&mut out).unwrap();
                    }
                },
            }
        }

        writeln!(&mut out).unwrap();
        writeln!(&mut out).unwrap();
    }

    out
}

/// Runs cuobjdump on a file
/// TODO: the majority of execution CPU usage is from running the external cuobjdump process.
/// We could cache the results of the external cuobjdump to make subsequent runs much faster
async fn run_cuobjdump(path: &str) -> String {
    let out = tokio::process::Command::new("/usr/local/cuda-10.2/bin/cuobjdump")
        .args(["-elf", path])
        .output()
        .await
        .unwrap()
        .stdout;

    String::from_utf8(out).unwrap()
}

/// Strips things that aren't part of .nv.info. sections from the cuobjdump output
fn strip_non_nvinfo(target: String) -> String {
    let mut out = String::new();
    let mut in_nv_info = false;
    for line in target.lines() {
        if in_nv_info && !(line.starts_with("\t") || line == "") {
            in_nv_info = false;
        }

        if line.starts_with(".nv.info.") {
            in_nv_info = true;
        }

        if in_nv_info {
            out.push_str(line);
            out.push_str("\n");
        }
    }

    out
}

/// Given a cubin path, run the real cuobjdump and our reimplementation
async fn test_cubin<P: AsRef<Path>>(cubin_path: P) {
    let cubin_path_str = cubin_path.as_ref().to_str().unwrap();

    log::info!("Testing {}... ", &cubin_path_str);

    let data = tokio::fs::read(&cubin_path).await.unwrap();
    let mut ours = tokio::task::spawn_blocking(move || {
        let parsed = cuda_parsers::cubin::parse(&data).unwrap();
        cuobjdump_fmt(parsed)
    })
    .await
    .unwrap();

    let target = run_cuobjdump(&cubin_path_str).await;

    let mut target = strip_non_nvinfo(target);
    while target.ends_with("\n") {
        target.pop();
    }

    while ours.ends_with("\n") {
        ours.pop();
    }

    pretty_assertions::assert_eq!(ours, target);
}

/// For each test file, compare the real cuobjdump output to the outupt of our reimplementation
#[tokio::test]
async fn test_validate_output() {
    let _ = env_logger::builder().is_test(true).try_init();
    // let cubin_path = Path::new("test_data/cubins/libtorch_cuda.3022.sm_70.cubin");
    // test_cubin(&cubin_path);

    let base_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let paths = std::fs::read_dir(base_path.join("test_data/cubins/")).unwrap();

    let _ = futures::stream::iter(paths)
        .map(|entry| {
            let cubin_path = entry.unwrap().path();
            test_cubin(cubin_path)
        })
        .buffer_unordered(20)
        .count()
        .await;
}
