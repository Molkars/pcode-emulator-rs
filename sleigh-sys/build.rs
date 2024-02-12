use std::path::Path;

const SOURCE_FILES: &[&str] = &[
    "space.cc",
    "float.cc",
    "address.cc",
    "pcoderaw.cc",
    "translate.cc",
    "opcodes.cc",
    "globalcontext.cc",
    "capability.cc",
    "architecture.cc",
    "options.cc",
    "graph.cc",
    "cover.cc",
    "block.cc",
    "cast.cc",
    "typeop.cc",
    "database.cc",
    "cpool.cc",
    "comment.cc",
    "fspec.cc",
    "action.cc",
    "loadimage.cc",
    "varnode.cc",
    "op.cc",
    "type.cc",
    "variable.cc",
    "varmap.cc",
    "jumptable.cc",
    "emulate.cc",
    "emulateutil.cc",
    "flow.cc",
    "userop.cc",
    "funcdata.cc",
    "funcdata_block.cc",
    "funcdata_varnode.cc",
    "funcdata_op.cc",
    "pcodeinject.cc",
    "heritage.cc",
    "prefersplit.cc",
    "rangeutil.cc",
    "ruleaction.cc",
    "subflow.cc",
    "blockaction.cc",
    "merge.cc",
    "double.cc",
    "coreaction.cc",
    "condexe.cc",
    "override.cc",
    "dynamic.cc",
    "crc32.cc",
    "prettyprint.cc",
    "printlanguage.cc",
    "printc.cc",
    "printjava.cc",
    "memstate.cc",
    "opbehavior.cc",
    "paramid.cc",
    "transform.cc",
    "stringmanage.cc",
    //"string_ghidra.cc",
    //"ghidra_arch.cc",
    //"typegrp_ghidra.cc",
    //"cpool_ghidra.cc",
    "loadimage_ghidra.cc",
    //"inject_ghidra.cc",
    //"database_ghidra.cc",
    "inject_sleigh.cc",
    //"ghidra_translate.cc",
    //"ghidra_context.cc",
    //"comment_ghidra.cc",
    "sleigh_arch.cc",
    "sleigh.cc",
    // "filemanage.cc", // not-on-windows: unistd.h
    "semantics.cc",
    "slghsymbol.cc",
    "context.cc",
    "sleighbase.cc",
    "slghpatexpress.cc",
    "slghpattern.cc",
    "pcodecompile.cc",
    //"slgh_compile.cc",
    "slghscan.cc",
    "slghparse.cc",
    "xml.cc",
];

fn main() {
    println!("cargo:rerun-if-changed=bridge/");
    println!("cargo:rerun-if-changed=decompiler/");

    let mut builder = cxx_build::bridge("src/lib.rs");

    builder
        .cpp(true)
        .define("PACKAGE", "cppserver")
        .files(SOURCE_FILES.iter().map(|s| Path::new("decompiler").join(s)))
        .file("bridge/bridge.cc")
        .includes(["decompiler", "bridge"])
        .warnings(false)
        .flag_if_supported("-std=c++14");

    #[cfg(not(target_os = "windows"))]
    {
        builder.flag("-lbfd -lz");
    }

    builder.compile("sleigh");
}
