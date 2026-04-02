use ts_rs::TS;

#[derive(Debug, Clone, PartialEq, Eq, Hash, TS)]
#[ts(export)]
pub struct PsyNetIdTs {
    pub online_id: String,
    pub unknown1: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, TS)]
#[ts(export)]
pub struct SwitchIdTs {
    pub online_id: String,
    pub unknown1: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, TS)]
#[ts(export)]
pub struct Ps4IdTs {
    pub online_id: String,
    pub name: String,
    pub unknown1: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, TS)]
#[ts(export)]
pub enum RemoteIdTs {
    PlayStation(Ps4IdTs),
    PsyNet(PsyNetIdTs),
    SplitScreen(u32),
    Steam(String),
    Switch(SwitchIdTs),
    Xbox(String),
    QQ(String),
    Epic(String),
}

#[derive(Debug, Clone, PartialEq, TS)]
#[ts(export)]
pub enum HeaderPropTs {
    Array(Vec<Vec<(String, HeaderPropTs)>>),
    Bool(bool),
    Byte {
        kind: String,
        value: Option<String>,
    },
    Float(f32),
    Int(i32),
    Name(String),
    QWord(String),
    Str(String),
    Struct {
        name: String,
        fields: Vec<(String, HeaderPropTs)>,
    },
}
