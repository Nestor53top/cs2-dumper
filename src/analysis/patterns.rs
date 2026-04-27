use pelite::pattern;
use pelite::pattern::Atom;
use phf::{Map, phf_map};

pub struct PatternSet {
    pub name: &'static str,
    pub patterns: &'static Map<&'static str, &'static [Atom]>,
}

pub const CLIENT_PATTERNS: Map<&'static str, &'static [Atom]> = phf_map! {
    "dwCSGOInput" => pattern!("488905${'} 0f57c0 0f1105"),
    "dwEntityList" => pattern!("48890d${'} e9${} cc"),
    "dwGameEntitySystem" => pattern!("488b1d${'} 48891d[4] 4c63b3"),
    "dwGameEntitySystem_highestEntityIndex" => pattern!("ff81u4 4885d2"),
    "dwGameRules" => pattern!("f6c1010f85${} 4c8b05${'} 4d85"),
    "dwGlobalVars" => pattern!("488915${'} 488942"),
    "dwGlowManager" => pattern!("488b05${'} c3 cccccccccccccccc 8b41"),
    "dwLocalPlayerController" => pattern!("488b05${'} 4189be"),
    "dwPlantedC4" => pattern!("488b15${'} 41ffc0 488d4c24? 448905[4]"),
    "dwPrediction" => pattern!("488d05${'} c3 cccccccccccccccc 405356 4154"),
    "dwSensitivity" => pattern!("488d0d${[8]'} 660f6ecd"),
    "dwSensitivity_sensitivity" => pattern!("488d7eu1 480fbae0? 72? 85d2 490f4fff"),
    "dwViewMatrix" => pattern!("488d0d${'} 48c1e006"),
    "dwViewRender" => pattern!("488905${'} 488bc8 4885c0"),
    "dwWeaponC4" => pattern!("488b15${'} 488b5c24? ffc0 8905${} 488bc6 488934ea 80be"),
};

pub const ENGINE2_PATTERNS: Map<&'static str, &'static [Atom]> = phf_map! {
    "dwBuildNumber" => pattern!("8905${'} 488d0d${} ff15${} 488b0d"),
    "dwNetworkGameClient" => pattern!("48893d${'} ff87"),
    "dwNetworkGameClient_clientTickCount" => pattern!("8b81u4 c3 cccccccccccccccccc 8b81${} c3 cccccccccccccccccc 83b9"),
    "dwNetworkGameClient_deltaTick" => pattern!("4c8db7u4 4c897c24"),
    "dwNetworkGameClient_isBackgroundMap" => pattern!("0fb681u4 c3 cccccccccccccccc 0fb681${} c3 cccccccccccccccc 4053"),
    "dwNetworkGameClient_localPlayer" => pattern!("428b94d3u4 5b 49ffe3 32c0 5b c3 cccccccccccccccc 4053"),
    "dwNetworkGameClient_maxClients" => pattern!("8b81u4 c3????????? 8b81[4] c3????????? 8b81"),
    "dwNetworkGameClient_serverTickCount" => pattern!("8b81u4 c3 cccccccccccccccccc 83b9"),
    "dwNetworkGameClient_signOnState" => pattern!("448b81u4 488d0d"),
    "dwWindowHeight" => pattern!("8b05${'} 8903"),
    "dwWindowWidth" => pattern!("8b05${'} 8907"),
};

pub const INPUT_SYSTEM_PATTERNS: Map<&'static str, &'static [Atom]> = phf_map! {
    "dwInputSystem" => pattern!("488905${'} 33c0"),
};

pub const MATCHMAKING_PATTERNS: Map<&'static str, &'static [Atom]> = phf_map! {
    "dwGameTypes" => pattern!("488d0d${'} ff90"),
};

pub const SOUNDSYSTEM_PATTERNS: Map<&'static str, &'static [Atom]> = phf_map! {
    "dwSoundSystem" => pattern!("488d05${'} c3 cccccccccccccccc 488915"),
    "dwSoundSystem_engineViewData" => pattern!("0f1147u1 0f104e? 0f118f"),
};

pub fn all_patterns() -> Vec<PatternSet> {
    vec![
        PatternSet {
            name: "client.dll",
            patterns: &CLIENT_PATTERNS,
        },
        PatternSet {
            name: "engine2.dll",
            patterns: &ENGINE2_PATTERNS,
        },
        PatternSet {
            name: "inputsystem.dll",
            patterns: &INPUT_SYSTEM_PATTERNS,
        },
        PatternSet {
            name: "matchmaking.dll",
            patterns: &MATCHMAKING_PATTERNS,
        },
        PatternSet {
            name: "soundsystem.dll",
            patterns: &SOUNDSYSTEM_PATTERNS,
        },
    ]
    }
