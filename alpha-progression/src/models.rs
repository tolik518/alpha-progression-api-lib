use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub uuid: String,
    pub email: String,
    pub token: String,
}

// ---------------------------------------------------------------------------
// Workout
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkoutDoc {
    #[serde(rename = "_id")]
    pub id: String,
    #[serde(rename = "_rev", skip_serializing_if = "Option::is_none")]
    pub rev: Option<String>,
    pub name: String,
    #[serde(default)]
    pub exercises: Vec<Exercise>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Exercise {
    #[serde(rename = "_id")]
    pub id: String,
    #[serde(rename = "exerciseId")]
    pub exercise_id: String,
    #[serde(rename = "repsTarget", skip_serializing_if = "Option::is_none")]
    pub reps_target: Option<u32>,
    #[serde(default)]
    pub sets: Vec<ExerciseSet>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExerciseSet {
    pub reps: u32,
    pub weight: f64,
    #[serde(rename = "weightUnit")]
    pub weight_unit: String,
    pub completed: i64,
}

// ---------------------------------------------------------------------------
// Measurements
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MeasurementKind {
    Bodyweight,
    Bodyfat,
    Neck,
    Shoulders,
    Chest,
    Waist,
    UpperArmsLeft,
    UpperArmsRight,
    ForeArmsLeft,
    ForeArmsRight,
    CalvesLeft,
    CalvesRight,
}

impl MeasurementKind {
    pub fn doc_id(self) -> &'static str {
        match self {
            Self::Bodyweight    => "measurement-bodyweight",
            Self::Bodyfat       => "measurement-bodyfat",
            Self::Neck          => "measurement-neck",
            Self::Shoulders     => "measurement-shoulders",
            Self::Chest         => "measurement-chest",
            Self::Waist         => "measurement-waist",
            Self::UpperArmsLeft  => "measurement-upper-arms-left",
            Self::UpperArmsRight => "measurement-upper-arms-right",
            Self::ForeArmsLeft   => "measurement-forearms-left",
            Self::ForeArmsRight  => "measurement-forearms-right",
            Self::CalvesLeft    => "measurement-calves-left",
            Self::CalvesRight   => "measurement-calves-right",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeasurementDoc {
    #[serde(rename = "_id")]
    pub id: String,
    #[serde(rename = "_rev", skip_serializing_if = "Option::is_none")]
    pub rev: Option<String>,
    #[serde(default)]
    pub entries: Vec<MeasurementEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeasurementEntry {
    pub value: f64,
    pub unit: String,
    pub timestamp: i64,
}

// ---------------------------------------------------------------------------
// Cloudant / CouchDB
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbInfo {
    pub db_name: String,
    pub doc_count: u64,
    pub doc_del_count: u64,
    pub update_seq: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangesResponse {
    pub results: Vec<ChangeEntry>,
    pub last_seq: String,
    pub pending: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeEntry {
    pub id: String,
    pub seq: String,
    pub changes: Vec<RevInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevInfo {
    pub rev: String,
}
