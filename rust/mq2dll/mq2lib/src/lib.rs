#![allow(non_snake_case, non_upper_case_globals)]

use log::{info, LevelFilter};

use std::ffi::c_char;
use std::ffi::CString;
use std::format as f;
use std::time::{Duration, Instant};
use windows::{
    core::s,
    Win32::Foundation::*,
    Win32::{System::SystemServices::*, UI::WindowsAndMessaging::MessageBoxA},
};

// TO call using autocxx, it's in MQ2Main.dll MQ2Main.h
// MQLIB_API void WriteChatf(const char* Format, ...);

// MQ2Rust : Defines the entry point for the DLL application.
//

// PLUGIN_API is only to be used for callbacks.  All existing callbacks at this time
// are shown below. Remove the ones your plugin does not use.  Always use Initialize
// and Shutdown for setup and cleanup.

/**
 * Avoid Globals if at all possible, since they persist throughout your program.
 * But if you must have them, here is the place to put them.
*/
// bool ShowMQ2RustWindow = true;

/////////// TYPES
type DWORD = u32;
type FFIStringPtr = *const c_char;
static MQ2Version: &str = "0.1";
static mut PULSE_TIMER: Option<Instant> = None;
static mut DRAW_HUD_TIMER: Option<Instant> = None;

// example extern
#[no_mangle]
pub extern "C" fn add(left: usize, right: usize) -> usize {
    left + right
}

//#[no_mangle]
//pub extern "C" fn tryFindSpawn() -> bool {
//    left + right
//}

//////////////////////////// CPP FUNCTIONS
#[repr(C)]
struct PlayerClient; // Opaque type representing eqlib::PlayerClient

//type Spawn = *mut PlayerClient; // Equivalent to `eqlib::PlayerClient* Spawn;`
type Spawn = bool;

extern "C" {
    fn FindSpawnFromName(name: *const std::os::raw::c_char) -> Spawn;
}

pub fn find_spawn_from_name(name: &str) -> Option<Spawn> {
    let c_name = CString::new(name).expect("CString::new failed");
    unsafe {
        let spawn = FindSpawnFromName(c_name.as_ptr());
        Some(spawn)
        //if spawn {
        //    Some(spawn)
        //} else {
        //    None
        //}
    }
}

/////////////////////////// UTILS

fn DebugSpewAlways<T: AsRef<str>>(message: T) {
    info!("DebugSpew: {}", message.as_ref());
}

fn GetRustStringRefFromCharPointer(pchar: FFIStringPtr) -> &'static str {
    let c_str = unsafe {
        assert!(!pchar.is_null());

        std::ffi::CStr::from_ptr(pchar)
    };

    c_str.to_str().expect("Invalid UTF-8 String")
}

////////////////////////////////// MAIN
fn init_logging() {
    //let stdout = ConsoleAppender::builder().build();

    simple_logging::log_to_file("mq2rustlib.log", LevelFilter::Info).unwrap();
    info!("Logging initialized");
}

fn attach() {
    unsafe {
        // Create a message box
        // MessageBoxA(HWND(0), s!("ZOMG!"), s!("hello.dll"), Default::default());
    };
}

fn detach() {
    unsafe {
        // Create a message box
        // MessageBoxA(HWND(0), s!("GOODBYE!"), s!("hello.dll"), Default::default());
    };
}

#[no_mangle]
#[allow(non_snake_case, unused_variables)]
extern "system" fn DllMain(dll_module: HINSTANCE, call_reason: u32, _: *mut ()) -> bool {
    match call_reason {
        DLL_PROCESS_ATTACH => attach(),
        DLL_PROCESS_DETACH => detach(),
        _ => (),
    }

    true
}

///////////////////////////////////// pluginapi.h //////////////////////////////////////

//////////////////////////////////// PLUGIN INTERFACE /////////////////////////////////////////////////////
/**
 * @fn InitializePlugin
 *
 * This is called once on plugin initialization and can be considered the startup
 * routine for the plugin.
*/
#[no_mangle]
pub extern "C" fn InitializePlugin() -> bool {
    init_logging();

    DebugSpewAlways(f!("MQ2Rust::Initializing version {MQ2Version}"));
    let myspawn = find_spawn_from_name("Teleportation").unwrap();
    DebugSpewAlways(f!("Called cpp function from Rust and got spawn: {myspawn}"));
    myspawn

    // Examples:
    // AddCommand("/mycommand", MyCommand);
    // AddXMLFile("MQUI_MyXMLFile.xml");
    // AddMQ2Data("mytlo", MyTLOData);
}

/**
 * @fn ShutdownPlugin
 *
 * This is called once when the plugin has been asked to shutdown.  The plugin has
 * not actually shut down until this completes.
*/
#[no_mangle]
pub extern "C" fn ShutdownPlugin() {
    DebugSpewAlways("MQ2Rust::Shutting down");

    // Examples:
    // RemoveCommand("/mycommand");
    // RemoveXMLFile("MQUI_MyXMLFile.xml");
    // RemoveMQ2Data("mytlo");
}

/**
 * @fn OnCleanUI
 *
 * This is called once just before the shutdown of the UI system and each time the
 * game requests that the UI be cleaned.  Most commonly this happens when a
 * /loadskin command is issued, but it also occurs when reaching the character
 * select screen and when first entering the game.
 *
 * One purpose of this function is to allow you to destroy any custom windows that
 * you have created and cleanup any UI items that need to be removed.
*/
#[no_mangle]
pub extern "C" fn OnCleanUI() {
    DebugSpewAlways("MQ2Rust::OnCleanUI()");
}

/**
 * @fn OnReloadUI
 *
 * This is called once just after the UI system is loaded. Most commonly this
 * happens when a /loadskin command is issued, but it also occurs when first
 * entering the game.
 *
 * One purpose of this function is to allow you to recreate any custom windows
 * that you have setup.
*/
#[no_mangle]
pub extern "C" fn OnReloadUI() {
    DebugSpewAlways("MQ2Rust::OnReloadUI()");
}

/**
 * @fn OnDrawHUD
 *
 * This is called each time the Heads Up Display (HUD) is drawn.  The HUD is
 * responsible for the net status and packet loss bar.
 *
 * Note that this is not called at all if the HUD is not shown (default F11 to
 * toggle).
 *
 * Because the net status is updated frequently, it is recommended to have a
 * timer or counter at the start of this call to limit the amount of times the
 * code in this section is executed.
*/
#[no_mangle]
pub extern "C" fn OnDrawHUD() {
    unsafe {
        if DRAW_HUD_TIMER.is_none() {
            DRAW_HUD_TIMER = Some(Instant::now());
        }

        if let Some(timer) = DRAW_HUD_TIMER {
            if Instant::now() > timer {
                DRAW_HUD_TIMER = Some(Instant::now() + Duration::from_millis(500));
                DebugSpewAlways("MQ2Rust::OnDrawHUD()");
            }
        }
    }
    /*
        static std::chrono::steady_clock::time_point DrawHUDTimer = std::chrono::steady_clock::now();
        // Run only after timer is up
        if (std::chrono::steady_clock::now() > DrawHUDTimer)
        {
            // Wait half a second before running again
            DrawHUDTimer = std::chrono::steady_clock::now() + std::chrono::milliseconds(500);
            DebugSpewAlways("MQ2Rust::OnDrawHUD()");
        }
    */
}

/**
 * @fn SetGameState
 *
 * This is called when the GameState changes.  It is also called once after the
 * plugin is initialized.
 *
 * For a list of known GameState values, see the constants that begin with
 * GAMESTATE_.  The most commonly used of these is GAMESTATE_INGAME.
 *
 * When zoning, this is called once after @ref OnBeginZone @ref OnRemoveSpawn
 * and @ref OnRemoveGroundItem are all done and then called once again after
 * @ref OnEndZone and @ref OnAddSpawn are done but prior to @ref OnAddGroundItem
 * and @ref OnZoned
 *
 * @param GameState int - The value of GameState at the time of the call
*/
#[no_mangle]
pub extern "C" fn SetGameState(gameState: i32) {
    DebugSpewAlways(f!("MQ2Rust::SetGameState({gameState})"));
}

/**
 * @fn OnPulse
 *
 * This is called each time MQ2 goes through its heartbeat (pulse) function.
 *
 * Because this happens very frequently, it is recommended to have a timer or
 * counter at the start of this call to limit the amount of times the code in
 * this section is executed.
*/
#[no_mangle]
pub extern "C" fn OnPulse() {
    unsafe {
        if PULSE_TIMER.is_none() {
            PULSE_TIMER = Some(Instant::now());
        }

        if PULSE_TIMER.unwrap() <= Instant::now() {
            PULSE_TIMER = Some(Instant::now() + Duration::new(5, 0));
            DebugSpewAlways("MQ2Rust::OnPulse()");
        }
    }
    /*
        static std::chrono::steady_clock::time_point PulseTimer = std::chrono::steady_clock::now();
        // Run only after timer is up
        if (std::chrono::steady_clock::now() > PulseTimer)
        {
            // Wait 5 seconds before running again
            PulseTimer = std::chrono::steady_clock::now() + std::chrono::seconds(5);
            DebugSpewAlways("MQ2Rust::OnPulse()");
        }
    */
}

/**
 * @fn OnWriteChatColor
 *
 * This is called each time WriteChatColor is called (whether by MQ2Main or by any
 * plugin).  This can be considered the "when outputting text from MQ" callback.
 *
 * This ignores filters on display, so if they are needed either implement them in
 * this section or see @ref OnIncomingChat where filters are already handled.
 *
 * If CEverQuest::dsp_chat is not called, and events are required, they'll need to
 * be implemented here as well.  Otherwise, see @ref OnIncomingChat where that is
 * already handled.
 *
 * For a list of Color values, see the constants for USERCOLOR_.  The default is
 * USERCOLOR_DEFAULT.
 *
 * @param Line const char* - The line that was passed to WriteChatColor
 * @param Color int - The type of chat text this is to be sent as
 * @param Filter int - (default 0)
*/
#[no_mangle]
pub extern "C" fn OnWriteChatColor(Line: FFIStringPtr, Color: i32, Filter: i32) {
    let line_str = GetRustStringRefFromCharPointer(Line);
    DebugSpewAlways(f!(
        "MQ2Rust::OnWriteChatColor({line_str}, {Color}, {Filter})"
    ));
}

/**
 * @fn OnIncomingChat
 *
 * This is called each time a line of chat is shown.  It occurs after MQ filters
 * and chat events have been handled.  If you need to know when MQ2 has sent chat,
 * consider using @ref OnWriteChatColor instead.
 *
 * For a list of Color values, see the constants for USERCOLOR_. The default is
 * USERCOLOR_DEFAULT.
 *
 * @param Line const char* - The line of text that was shown
 * @param Color int - The type of chat text this was sent as
 *
 * @return bool - Whether to filter this chat from display
*/
#[no_mangle]
pub extern "C" fn OnIncomingChat(Line: FFIStringPtr, Color: DWORD) -> bool {
    let line_str = GetRustStringRefFromCharPointer(Line);
    DebugSpewAlways(f!("MQ2Rust::OnIncomingChat({line_str}, {Color})"));
    return false;
}

/**
 * @fn OnAddSpawn
 *
 * This is called each time a spawn is added to a zone (ie, something spawns). It is
 * also called for each existing spawn when a plugin first initializes.
 *
 * When zoning, this is called for all spawns in the zone after @ref OnEndZone is
 * called and before @ref OnZoned is called.
 *
 * @param pNewSpawn PSPAWNINFO - The spawn that was added
*/
//#[no_mangle]
//pub extern fn OnAddSpawn(PSPAWNINFO pNewSpawn)
//{
//	// DebugSpewAlways("MQ2Rust::OnAddSpawn(%s)", pNewSpawn->Name);
//}

/**
 * @fn OnRemoveSpawn
 *
 * This is called each time a spawn is removed from a zone (ie, something despawns
 * or is killed).  It is NOT called when a plugin shuts down.
 *
 * When zoning, this is called for all spawns in the zone after @ref OnBeginZone is
 * called.
 *
 * @param pSpawn PSPAWNINFO - The spawn that was removed
*/
//#[no_mangle]
//pub extern fn OnRemoveSpawn(PSPAWNINFO pSpawn)
//{
//	// DebugSpewAlways("MQ2Rust::OnRemoveSpawn(%s)", pSpawn->Name);
//}

/**
 * @fn OnAddGroundItem
 *
 * This is called each time a ground item is added to a zone (ie, something spawns).
 * It is also called for each existing ground item when a plugin first initializes.
 *
 * When zoning, this is called for all ground items in the zone after @ref OnEndZone
 * is called and before @ref OnZoned is called.
 *
 * @param pNewGroundItem PGROUNDITEM - The ground item that was added
*/
//#[no_mangle]
//pub extern fn OnAddGroundItem(PGROUNDITEM pNewGroundItem)
//{
//	// DebugSpewAlways("MQ2Rust::OnAddGroundItem(%d)", pNewGroundItem->DropID);
//}

/**
 * @fn OnRemoveGroundItem
 *
 * This is called each time a ground item is removed from a zone (ie, something
 * despawns or is picked up).  It is NOT called when a plugin shuts down.
 *
 * When zoning, this is called for all ground items in the zone after
 * @ref OnBeginZone is called.
 *
 * @param pGroundItem PGROUNDITEM - The ground item that was removed
*/
//#[no_mangle]
//pub extern fn OnRemoveGroundItem(PGROUNDITEM pGroundItem)
//{
//	// DebugSpewAlways("MQ2Rust::OnRemoveGroundItem(%d)", pGroundItem->DropID);
//}

/**
 * @fn OnBeginZone
 *
 * This is called just after entering a zone line and as the loading screen appears.
*/
//#[no_mangle]
//pub extern fn OnBeginZone()
//{
//	// DebugSpewAlways("MQ2Rust::OnBeginZone()");
//}

/**
 * @fn OnEndZone
 *
 * This is called just after the loading screen, but prior to the zone being fully
 * loaded.
 *
 * This should occur before @ref OnAddSpawn and @ref OnAddGroundItem are called. It
 * always occurs before @ref OnZoned is called.
*/
//#[no_mangle]
//pub extern fn OnEndZone()
//{
//	// DebugSpewAlways("MQ2Rust::OnEndZone()");
//}

/**
 * @fn OnZoned
 *
 * This is called after entering a new zone and the zone is considered "loaded."
 *
 * It occurs after @ref OnEndZone @ref OnAddSpawn and @ref OnAddGroundItem have
 * been called.
*/
//#[no_mangle]
//pub extern fn OnZoned()
//{
//	// DebugSpewAlways("MQ2Rust::OnZoned()");
//}

/**
 * @fn OnUpdateImGui
 *
 * This is called each time that the ImGui Overlay is rendered. Use this to render
 * and update plugin specific widgets.
 *
 * Because this happens extremely frequently, it is recommended to move any actual
 * work to a separate call and use this only for updating the display.
*/
//#[no_mangle]
//pub extern fn OnUpdateImGui()
//{
///*
//	if (GetGameState() == GAMESTATE_INGAME)
//	{
//		if (ShowMQ2RustWindow)
//		{
//			if (ImGui::Begin("MQ2Rust", &ShowMQ2RustWindow, ImGuiWindowFlags_MenuBar))
//			{
//				if (ImGui::BeginMenuBar())
//				{
//					ImGui::Text("MQ2Rust is loaded!");
//					ImGui::EndMenuBar();
//				}
//			}
//			ImGui::End();
//		}
//	}
//*/
//}

/**
 * @fn OnMacroStart
 *
 * This is called each time a macro starts (ex: /mac somemacro.mac), prior to
 * launching the macro.
 *
 * @param Name const char* - The name of the macro that was launched
*/
//#[no_mangle]
//pub extern fn OnMacroStart(const char* Name)
//{
//	// DebugSpewAlways("MQ2Rust::OnMacroStart(%s)", Name);
//}

/**
 * @fn OnMacroStop
 *
 * This is called each time a macro stops (ex: /endmac), after the macro has ended.
 *
 * @param Name const char* - The name of the macro that was stopped.
*/
//#[no_mangle]
//pub extern fn OnMacroStop(const char* Name)
//{
//	// DebugSpewAlways("MQ2Rust::OnMacroStop(%s)", Name);
//}

/**
 * @fn OnLoadPlugin
 *
 * This is called each time a plugin is loaded (ex: /plugin someplugin), after the
 * plugin has been loaded and any associated -AutoExec.cfg file has been launched.
 * This means it will be executed after the plugin's @ref InitializePlugin callback.
 *
 * This is also called when THIS plugin is loaded, but initialization tasks should
 * still be done in @ref InitializePlugin.
 *
 * @param Name const char* - The name of the plugin that was loaded
*/
#[no_mangle]
pub extern "C" fn OnLoadPlugin(name: FFIStringPtr) {
    let name_str = GetRustStringRefFromCharPointer(name);
    DebugSpewAlways(f!("MQ2Rust::OnLoadPlugin({name_str})"));
}

/**
 * @fn OnUnloadPlugin
 *
 * This is called each time a plugin is unloaded (ex: /plugin someplugin unload),
 * just prior to the plugin unloading.  This means it will be executed prior to that
 * plugin's @ref ShutdownPlugin callback.
 *
 * This is also called when THIS plugin is unloaded, but shutdown tasks should still
 * be done in @ref ShutdownPlugin.
 *
 * @param Name const char* - The name of the plugin that is to be unloaded
*/
#[no_mangle]
pub extern "C" fn OnUnloadPlugin(name: FFIStringPtr) {
    let c_str = unsafe {
        assert!(!name.is_null());

        std::ffi::CStr::from_ptr(name)
    };

    let name_str = c_str.to_str().expect("Invalid UTF-8 String");

    DebugSpewAlways(f!("MQ2Rust::OnUnloadPlugin({name_str})"));
}
