use once_cell::sync::Lazy;
use std::fs;
use std::path::PathBuf;

pub(crate) fn get_site_uuid() -> &'static u8 {
    &SITE_UUID
}

static SITE_UUID: Lazy<u8> = Lazy::new(|| {

    let path = get_path_to_id();

    // create the parent folder if missing
    if let Some(parent) = &path.parent().filter(|p| !p.exists()) {
        fs::create_dir_all(parent)
            .expect("Failed creating site_id parent folder");
        
    }

    let string_id = 
        fs::read_to_string(&path)
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| {
            let new_id = String::from("0");
            fs::write(&path, &new_id)
                .expect("Failed writing site id in folder");
            new_id
        });

    string_id
        .parse::<u8>()
        .expect("Invalid site id")
});

fn get_path_to_id() -> PathBuf {

    #[cfg(debug_assertions)]
    {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("data/site_id.txt")
    }

    #[cfg(not(debug_assertions))]
    {
        dirs::data_dir()
            .expect("no data dir")
            .join("stormcrow")
            .join("site_id.txt");
    }
}