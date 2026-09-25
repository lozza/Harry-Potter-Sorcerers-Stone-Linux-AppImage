#[derive(Debug, Clone, Copy, PartialEq)] pub enum DisplayProfile { Hd720, FullHd1080, SteamDeck, Ultrawide, Windowed }
impl DisplayProfile { pub fn all()->[Self;5]{[Self::Hd720,Self::FullHd1080,Self::SteamDeck,Self::Ultrawide,Self::Windowed]} pub fn id(self)->&'static str{match self{Self::Hd720=>"720p",Self::FullHd1080=>"1080p",Self::SteamDeck=>"deck",Self::Ultrawide=>"ultrawide",Self::Windowed=>"windowed"}} }
impl DisplayProfile {
    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            "720p" => Some(Self::Hd720),
            "1080p" => Some(Self::FullHd1080),
            "deck" => Some(Self::SteamDeck),
            "ultrawide" => Some(Self::Ultrawide),
            "windowed" => Some(Self::Windowed),
            _ => None,
        }
    }
}
#[derive(Debug, Clone, PartialEq)] pub struct GeneratedConfiguration { pub hp_ini:String, pub user_ini:String }
impl DisplayProfile { pub fn generate(self)->GeneratedConfiguration { let (width,height,fov,fullscreen)=match self { Self::Hd720=>(1280,720,90.0,true),Self::FullHd1080=>(1920,1080,106.26,true),Self::SteamDeck=>(1280,800,100.39,true),Self::Ultrawide=>(2560,1080,121.28,true),Self::Windowed=>(1280,720,90.0,false)}; GeneratedConfiguration {hp_ini:format!("[FirstRun]\nReconfig=0\n\n[Engine.Engine]\nGameRenderDevice=D3DDrv.D3DRenderDevice\nAudioDevice=Galaxy.GalaxyAudioSubsystem\n\n[WinDrv.WindowsClient]\nWindowedViewportX={width}\nWindowedViewportY={height}\nFullscreenViewportX={width}\nFullscreenViewportY={height}\nStartupFullscreen={fullscreen}\nUseJoystick=True\n"),user_ini:format!("[DefaultPlayer]\n\n[Engine.Input]\n\n[Harrypotter.Harry]\nDesiredFOV={fov}\nDefaultFOV={fov}\n")} } }
#[cfg(test)] mod tests { use super::*; #[test] fn deck_has_correct_size() {assert!(DisplayProfile::SteamDeck.generate().hp_ini.contains("FullscreenViewportY=800"));} }
