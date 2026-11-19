add_subdirectory(common)

corrosion_import_crate(
    MANIFEST_PATH riti/Cargo.toml
    CRATE_TYPES staticlib
    CRATES riti
)

add_subdirectory(ibus)
