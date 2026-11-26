fn main() {
    #[cfg(feature = "gui")]
    {
        // Deterministic, plain fixture icon required by Windows resource generation.
        // This is not product branding. Generated output is excluded from Git.
        let mut ico = vec![0, 0, 1, 0, 1, 0, 32, 32, 0, 0, 1, 0, 32, 0];
        ico.extend_from_slice(&4264u32.to_le_bytes());
        ico.extend_from_slice(&22u32.to_le_bytes());
        ico.extend_from_slice(&40u32.to_le_bytes());
        ico.extend_from_slice(&32i32.to_le_bytes());
        ico.extend_from_slice(&64i32.to_le_bytes());
        ico.extend_from_slice(&1u16.to_le_bytes());
        ico.extend_from_slice(&32u16.to_le_bytes());
        ico.extend_from_slice(&[0; 24]);
        for _ in 0..1024 {
            ico.extend_from_slice(&[120, 90, 40, 255]);
        }
        ico.extend_from_slice(&[0; 128]);
        std::fs::create_dir_all("icons").expect("fixture icon directory");
        std::fs::write("icons/icon.ico", ico).expect("fixture icon");
        tauri_build::build();
    }
}
