use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub port: u16,
    pub institutions_path: PathBuf,
}

pub fn load() -> AppConfig {
    AppConfig {
        port: std::env::var("PORT")
            .unwrap_or_else(|_| "4000".into())
            .parse()
            .expect("PORT must be a number"),
        institutions_path: std::env::var("INSTITUTIONS_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("institutions")),
    }
}
