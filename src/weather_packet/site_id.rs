use once_cell::sync::Lazy;
use std::fs;
use std::path::PathBuf;

pub(crate) fn get_site_char() -> char {
    *SITE_ID
}

static SITE_ID: Lazy<char> = Lazy::new(|| {

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
            // NOTE: override to new char when implementing in a unique site
            let new_id = String::from("A");
            fs::write(&path, &new_id)
                .expect("Failed writing site id in folder");
            new_id
        });

    parse_site_char(&string_id)
});

fn parse_site_char(s: &str) -> char {
    let mut chars = s.chars();
    let c = chars.next().expect("Empty site id");

    // confirm correct character
    if chars.next().is_some() {
        panic!("Single char required for site id");
    } else if !c.is_ascii_uppercase() {
        panic!("Uppercase site id required");
    }

    c
}

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