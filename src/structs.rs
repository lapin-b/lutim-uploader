use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub watch_path: String,
    pub lutim_url: String,
}

impl Default for Config {
    fn default() -> Self {
        let mut watch_path = std::env::current_dir().unwrap();
        watch_path.push("screenshots");

        Config {
            watch_path: String::from(watch_path.to_str().unwrap()),
            lutim_url: "Insert Lutim URL here".to_string()
        }
    }
}