#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Score {
    pub id: u64,
    pub mode: i8,
    pub mods: i32,

    pub map_md5: String,
    pub pp: f32,
    pub acc: f32,

    pub max_combo: i32,
    pub ngeki: i32,
    pub n300: i32,
    pub nkatu: i32,
    pub n100: i32,
    pub n50: i32,
    pub nmiss: i32,

    pub userid: i32,
    pub map_id: i32,

    pub mods_json: Option<sqlx::types::Json<serde_json::Value>>,
    pub lazer: bool,
}

#[derive(Debug, sqlx::FromRow)]
pub struct BestScore {
    pub pp: f32,
    pub acc: f32,
}

// TODO: move this somewhere
#[derive(Debug, sqlx::FromRow)]
pub struct UserInfo {
    pub country: String,
    pub privs: i32,
}
