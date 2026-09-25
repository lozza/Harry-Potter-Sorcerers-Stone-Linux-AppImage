#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildStage {
    ValidateArguments,
    IdentifyEdition,
    Plan,
    ExtractInstaller,
    PreparePayload,
    AssembleAppDir,
    PackageAppImage,
    ValidateOutput,
}

impl BuildStage {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ValidateArguments => "validate_arguments",
            Self::IdentifyEdition => "identify_edition",
            Self::Plan => "plan",
            Self::ExtractInstaller => "extract_installer",
            Self::PreparePayload => "prepare_payload",
            Self::AssembleAppDir => "assemble_appdir",
            Self::PackageAppImage => "package_appimage",
            Self::ValidateOutput => "validate_output",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildEvent { pub stage: BuildStage, pub kind: &'static str, pub message: String }
impl BuildEvent {
    pub fn started(stage: BuildStage) -> Self { Self { stage, kind: "started", message: String::new() } }
    pub fn completed(stage: BuildStage) -> Self { Self { stage, kind: "completed", message: String::new() } }
    pub fn message(stage: BuildStage, message: impl Into<String>) -> Self { Self { stage, kind: "message", message: message.into() } }
}
pub trait EventSink { fn emit(&mut self, event: BuildEvent); }
impl EventSink for Vec<BuildEvent> { fn emit(&mut self, event: BuildEvent) { self.push(event); } }
