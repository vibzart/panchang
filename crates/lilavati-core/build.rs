fn main() {
    // Compile the vendored Swiss Ephemeris C library.
    // Source: https://github.com/aloistr/swisseph (v2.10.03)
    let swe_dir = "../../vendor/swisseph";

    cc::Build::new()
        .files([
            format!("{swe_dir}/swecl.c"),
            format!("{swe_dir}/swedate.c"),
            format!("{swe_dir}/swehel.c"),
            format!("{swe_dir}/swehouse.c"),
            format!("{swe_dir}/swejpl.c"),
            format!("{swe_dir}/swemmoon.c"),
            format!("{swe_dir}/swemplan.c"),
            format!("{swe_dir}/sweph.c"),
            format!("{swe_dir}/swephlib.c"),
        ])
        .include(swe_dir)
        .warnings(false) // Swiss Ephemeris C code generates many warnings
        .compile("swisseph");

    println!("cargo:rerun-if-changed={swe_dir}");
    println!("cargo:rerun-if-changed=build.rs");
}
