# Core
cancel-button = Cancel
save-button = Save
add-button = Add
delete-button = Delete
install-button = Install
enable-button = Enable
ok-button = OK
ok-dont-show-again-button = OK, don't show again

# Login
login-button = Log In
logout-button = Logout
register-button = Register
remember-me-label = Remember Me
username-label = Username
password-label = Password
server-select-label = Server Selection

login-error = Error logging in: { $error }
register-error = Error registering: { $error }
server-select-error = Error connecting to "{ $server }": { $error }

# Server edit
server-edit-label = Edit Server
server-add-label = Add Server
server-add-dialog-label = Add a Server
offline-add-server-hint = Add or Select a server to get started!
server-name-label = Name
server-host-label = Host
server-launcher-port-label = Launcher Port
server-game-port-label = Game Port
server-game-folder-label = Server Game Path
server-game-version-label = Version

# Messages
announcements-label = Announcements
news-label = News
none-label = None
player-name-label = Player
mini-banner-label = Mini Banner / Campaign
external-link-open-title = Open external link
external-link-open-message = This will open in your browser. Continue?

# Settings
settings-button = Settings
settings-label = Settings
settings-info-toggle-label = Toggle settings info
settings-general-title = Launcher
settings-game-title = Version
settings-advanced-title = Advanced
settings-troubleshooting-title = Troubleshooting
font-style-label = Font
launcher-resolution-label = Resolution
launcher-resolution-help-label = Resizable launcher styles remember the last size you used.
launcher-resolution-reset-label = Reset
font-default-label = Default
font-classic-label = Classic
font-new-label = New
preload-controller-dlls-label = Preload controller DLLs (Windows)
go-back-button = Go back
style-label = Theme
classic-style = Classic Style
modern-style = Modern Style
online-style = Online Style
ps4-style = PS4 Style
game-folder-label = Location
current-folder-label = Current Folder
custom-folder-label = Custom Folder
game-folder-current-caption = Launcher will follow the currently detected install folder.
game-folder-custom-caption = Launcher will use this custom install folder until you switch back.
game-folder-panel-title-label = Location
game-folder-panel-description = Defaults to launcher directory unless set by the Hunter.
browse-folder-label = Browse Folder
edit-folder-label = Edit Path
apply-folder-label = Apply
game-folder-empty-label = No folder selected
offline-images-label = Offline-Images
wine-prefix-label = Wine Prefix
wine-prefix-portable-label = Portable (Mezeporta)
wine-prefix-system-label = System Wine Prefix
wine-prefix-proton-label = Proton
wine-prefix-custom-label = Custom Prefix
wine-prefix-custom-path-label = Custom Prefix Path
wine-prefix-browse-label = Browse
wine-prefix-help-label = Portable keeps Wine state inside Mezeporta/WinePrefix for this install.
wine-prefix-custom-help-label = Accepts absolute paths or ~/ paths on Linux.
wine-prefix-custom-path-required-label = Choose or enter a Wine prefix folder before launching.
wine-prefix-status-checking-label = Checking Mezeporta prefix...
wine-prefix-status-ready-label = Mezeporta prefix ready
wine-prefix-status-missing-label = Mezeporta prefix not found
wine-prefix-status-missing-tools-label = Missing runtime tools
linux-prefix-install-label = Portable Prefix Install
linux-prefix-install-confirmation = Set up the portable Mezeporta Wine prefix for this install?<br><br>This will verify <strong>wine</strong>, <strong>wineserver</strong>, and <strong>winetricks</strong>, create or reuse <strong>Mezeporta/WinePrefix</strong>, run <strong>wineboot -u</strong>, install <strong>d3dcompiler_47</strong>, <strong>dxvk</strong>, and <strong>vcrun2022</strong>, and apply the Linux controller DLL overrides if R-Analog Patch is enabled.
linux-prefix-install-progress = Installing the portable Mezeporta Wine prefix for this game folder...
linux-prefix-install-success = Portable Mezeporta Wine prefix is ready.
linux-prefix-install-missing-tools = Missing Linux runtime tools: { $tools }. Run the bundled <strong>mezeporta-setup-ubuntu.sh</strong> or <strong>mezeporta-setup-arch.sh</strong> script first.
linux-audio-runtime-missing-label = Linux launcher audio is unavailable on this host. Install the missing runtime components
linux-sfx-runtime-missing = Linux launcher audio is unavailable on this host. Install the missing runtime components: { $components }
game-launch-stage-prepping-wine = Prepping Wine...
game-launch-stage-booting-wine = Booting Wine...
game-launch-stage-launching-dependencies = Launching Butter Dependencies...
game-launch-stage-starting-frontier = Starting Monster Hunter Frontier...
game-launch-dialog-label = Launching Game
game-launch-loading-label = Starting game. Please wait...
hd-version-label = HD Mode
fullscreen-label = Fullscreen
window-resolution-label = Resolution
fullscreen-resolution-label = Resolution
list-remote-servers-label = List Remote Servers
list-remote-messages-label = List Global Messages
serverlist-url-label = Serverlist URL
settings-error = Failed to write settings to 'mhf.ini'

# Characters page
create-character-label = Create New Character
create-character-error = Error creating new character
options-character-label = Character Options
delete-character-label = Delete Character
delete-character-confirmation = Are you sure you want to delete '{ $character_name }'?
delete-character-error = Error deleting character: { $error }
export-character-label = Export Character Save
export-character-success = Exported save to "{ $location }"
export-character-failed = Error exporting save data: { $error }
copy-cid-label = Copy Character ID
character-gender-label = Gender
character-gender-female = Female
character-gender-male = Male
weapon-label = Weapon
last-online-label = Last Online
start-game-label = START GAME

# Patcher
updating-label = Updating...
patcher-updates-label = Server Updates
patcher-updates-confirmation =
    A new patch has been found that needs to be installed before you can play, would you like to install it?<br>
    <span class="warning">You can restore your files back to the original in the settings under Maintenance.</span>
patcher-checking = Checking updates...
patcher-queue = Waiting for patch slot... Queue position: { $position }
patcher-percentage = [{ $percentage }%]Downloading files...
patcher-progress = { $current } out of { $total } files downloaded
patcher-patching = Finishing...
patcher-restoring = Restoring original files...
patcher-installing = Installing patch files...
patcher-done = Done.

# Weapons
greatsword-label = Greatsword
heavy-bowgun-label = Heavy Bowgun
hammer-label = Hammer
lance-label = Lance
sword-and-shield-label = Sword and Shield
light-bowgun-label = Light Bowgun
dual-swords-label = Dual Swords
longsword-label = Longsword
hunting-horn-label = Hunting Horn
gunlance-label = Gunlance
bow-label = Bow
tonfa-label = Tonfa
switch-axe-label = Switch Axe
magnet-spike-label = Magnet Spike

# Launcher
endpoint-name-empty = Server name must not be empty
endpoint-host-empty = Server host must not be empty
endpoint-unique = Server names must be unique
file-error = Failed to manage files
path-folder-error = Path must be a directory
path-exists-error = The specified game folder does not exist
current-endpoint-error = Unable to fetch data from selected server
remote-endpoint-error = Unable to fetch remote servers
remote-messages-error = Unable to fetch global messages
launcher-network-error = Launcher failed to connect to launcher server
patcher-network-error = Patcher failed to connect to patcher server
patcher-file-error = Patcher failed to manage files in game folder
internal-error = Launcher error, check logs

# Remote
username-error = Username does not exist
password-error = Your password is incorrect
username-exists-error = Username already exists
username-password-empty-error = Username and password must not be empty


reset-patch-label = Maintenance
reset-button-label = Reset patched files
reset-patch-confirmation = Restore all patched files back to original for this game folder?
reset-patch-success-confirmation = Patched files were reset successfully.
resetting-label = Resetting...
server-switch-label = Switch Server Files
switch-button = Switch
server-switch-confirmation =
    Switch local patch files from "{ $active }" to "{ $target }"?
    This will reuse available local files when possible.
server-switch-existing-confirmation =
    Switch local patch files from "{ $active }" to "{ $target }"?
    Cached files for the target server were found and will be used.



launcher-sfx-label = Sound Effects
launcher-controller-label = Controller
launcher-hardware-acceleration-label = Hardware Acceleration
sfx-volume-label = SFX Volume
game-version-label = Version
friend-signature-label = Signature
friend-signature-none-label = None / Disabled
friend-signature-auto-detect-label = Auto detect (pattern)
disable-xinput-preload-label = R-Analog Patch
controller-dlls-not-found-tooltip = dll's not found
lite-mode-label = Lite mode
id-short-label = ID
hr-short-label = HR
gr-short-label = GR
logging-in-label = Logging in { $username }
guild-responded-label = Guild responded!
fetching-hunter-status-label = Fetching Hunter Status...
hunters-ready-label = Hunters are ready to depart!
sizzle-label = Sizzle..Sizzle..
so-tasty-label = So Tasty!



hdGraphicPlLightShadowAttenuation-label = Player light shadow attenuation
hdGraphicDofFarBlurSize-label = DOF far blur size
match-monitor-resolution-label = Match monitor resolution
hdGraphicShadowmapColor-label = Shadowmap color
hdGraphicShadowLobby-label = Lobby shadows
settings-settings-title = Settings
settings-screen-title = Screen
max-char-display-label = Max character display
hdGraphicBloomColor-label = Bloom color
hdGraphicAntiAliasingWeightScale-label = Anti-aliasing weight scale
unfocused-volume-label = Unfocused volume
sound-frequency-label = Sample rate
texture-compression-label = Texture compression
hdGraphicSsaoDensity-label = SSAO density
windowed-label = Windowed
settings-version-title = Version
game-bgm-volume-label = BGM volume
disable-sound-output-label = Disable sound output
general-volume-label = General volume
settings-launcher-title = Launcher
hdGraphicBgLightShadowAttenuation-label = Background light shadow attenuation
brightness-label = Brightness
settings-info-nav-launcher-body = Launcher theme, font, UI sounds, and controller input.
settings-info-nav-version-body = Game branch, client version, HD mode, signature, and install location.
settings-info-nav-settings-body = In-game brightness, display mode, and resolution.
settings-info-nav-graphics-body = In-game graphics options and texture compression.
settings-info-nav-audio-body = In-game volume, output, sample rate, and buffer settings.
settings-info-nav-controls-body = In-game controller options.
settings-info-nav-advanced-body = Extra launcher tools, Linux options, and patch reset.
settings-info-launcher-theme-body = Switch between Classic and PS4 launcher layouts.
settings-info-font-selection-body = Launcher font only. Custom fonts go in Mezeporta/fonts/custom.
settings-info-launcher-resolution-body = Window size for the launcher only.
settings-info-launcher-custom-resolution-body = Type a custom launcher width and height for the current style.
settings-info-launcher-resolution-reset-body = Reset the saved launcher size for the current style back to its default resolution.
settings-info-launcher-controller-body = Controller navigation for the launcher UI. Work in progress.
settings-info-launcher-hardware-acceleration-body = Linux GPU rendering. Turn off only for display issues. Restart required.
settings-info-launcher-sfx-body = UI hover, select, and login sound effects.
settings-info-launcher-sfx-volume-body = Volume for launcher UI sound effects. Default: 30%.
settings-info-game-branch-body = Select the Monster Hunter Frontier generation.
settings-info-game-branch-online-body = Monster Hunter Frontier Online [Season]. Supports Season 6 (JP) and Season 7 (KR).
settings-info-game-branch-forward-body = Monster Hunter Frontier Online [Forward]. Supports F4 (JP) and F5 (JP).
settings-info-game-branch-g-body = Monster Hunter Frontier G. Supports G1 through G10.1 (JP).
settings-info-game-branch-z-body = Monster Hunter Frontier Z. Supports Z1 (JP) and ZZ (JP).
settings-info-game-version-body = Select the exact client version to launch.
settings-info-game-version-s6-body = Season 6 game version.
settings-info-game-version-s7k-body = Season 7 Korean game version.
settings-info-game-version-f4-body = Forward 4 game version.
settings-info-game-version-f5-body = Forward 5 game version.
settings-info-game-version-g1-body = G1 game version.
settings-info-game-version-g2-body = G2 game version.
settings-info-game-version-g3-body = G3 game version.
settings-info-game-version-g3-1-body = G3.1 game version.
settings-info-game-version-g3-2-body = G3.2 game version.
settings-info-game-version-gg-body = GG game version.
settings-info-game-version-g5-body = G5 game version.
settings-info-game-version-g5-1-body = G5.1 game version.
settings-info-game-version-g5-2-body = G5.2 game version.
settings-info-game-version-g6-body = G6 game version.
settings-info-game-version-g7-body = G7 game version.
settings-info-game-version-g9-1-body = G9.1 game version.
settings-info-game-version-g10-1-body = G10.1 game version.
settings-info-game-version-z1-body = Z1 game version.
settings-info-game-version-z2-body = Z2 game version.
settings-info-game-version-z2t-body = Z2TW game version.
settings-info-game-version-zz-body = ZZ game version.
settings-info-friend-signature-body = Client signature used for friend list and save data lookup.
settings-info-hd-version-body = HD client files for supported versions.
settings-info-game-folder-body = Game root folder. This is the folder that contains /dat.
settings-info-game-folder-browse-body = Browse to select the game location.
settings-info-game-folder-edit-body = Type the game location manually.
settings-info-brightness-body = In-game brightness level.
settings-info-display-mode-body = Windowed or fullscreen in-game display mode.
settings-info-window-resolution-body = Windowed-mode resolution.
settings-info-window-custom-resolution-body = Type a custom windowed-mode resolution.
settings-info-fullscreen-resolution-body = Fullscreen-mode resolution.
settings-info-fullscreen-custom-resolution-body = Type a custom fullscreen resolution.
settings-info-match-monitor-resolution-body = Match the primary display resolution.
settings-info-texture-compression-body = Texture compression for supported clients.
settings-info-max-char-display-body = Maximum number of other hunters visible at once.
settings-info-hd-advanced-toggle-body = Shows extra HD graphics settings.
settings-info-hd-graphics-toggle-body = Standard HD graphics options.
settings-info-hd-graphics-numeric-body = Advanced numeric HD graphics settings.
settings-info-disable-sound-output-body = Mutes all in-game audio.
settings-info-general-volume-body = Main in-game volume while the game is focused.
settings-info-unfocused-volume-body = In-game volume while the window is unfocused.
settings-info-minimized-volume-body = In-game volume while the window is minimized.
settings-info-game-bgm-volume-body = In-game background music volume.
settings-info-game-se-volume-body = In-game sound effects volume.
settings-info-sound-frequency-body = In-game audio sample rate.
settings-info-sound-buffer-num-body = In-game audio buffer size.
settings-info-controller-vibration-body = In-game controller vibration.
settings-info-controller-fix-body = Controller DLL handling for the R-Analog patch.
settings-info-controller-fix-windows-body = Uses controller DLLs from the game folder for the R-Analog patch.
settings-info-controller-fix-linux-body = Applies Wine DLL overrides for xinput1_3, dinput, and dinput8.
settings-info-dev-mode-body = Show incomplete or experimental launcher features.
settings-info-offline-images-body = Launcher-side offline image overrides from Mezeporta/Offline-Images.
settings-info-wine-prefix-mode-body = Choose the Wine prefix mode used by Linux launches.
settings-info-wine-prefix-portable-body = Uses Mezeporta/WinePrefix inside this game folder.
settings-info-wine-prefix-system-body = Uses your default Wine prefix.
settings-info-wine-prefix-proton-body = For launching through Steam as a non-Steam game. Steam manages Proton and its compat data.
settings-info-wine-prefix-custom-body = Uses a Wine prefix folder you choose.
settings-info-wine-prefix-custom-path-body = Folder used when Custom Prefix is selected.
settings-info-wine-prefix-custom-browse-body = Browse to an existing Wine prefix folder.
settings-info-reset-patch-body = Restore patched files back to the original game state for the active server.
settings-session-lock-tooltip = Log-out to adjust
controller-vibration-label = Vibration
settings-audio-title = Audio
game-se-volume-label = SFX volume
hdGraphicDof-label = Depth of field
hdGraphicGodray-label = Godray
hdGraphicBloomDispersion-label = Bloom dispersion
settings-controls-title = Controls
sound-buffer-num-label = Buffer size
hdGraphicSsao-label = SSAO
hdGraphicBloomThreshold-label = Bloom threshold
hdGraphicAntiAliasing-label = Anti-aliasing
hdGraphicGaussianBlurDispersion-label = Gaussian blur dispersion
minimized-volume-label = Minimized volume
hdGraphicShadowQuest-label = Quest shadows
hdGraphicBloom-label = Bloom
hdGraphicGaussianBlurBlendRate-label = Gaussian blur blend rate
hdGraphicSoftParticle-label = Soft particles


settings-graphics-title = Graphics
hd-version-settings-label = HD Mode Settings
hd-version-settings-disabled-label = HD Mode Settings (Disabled)

advance-settings-label = Advance Settings
custom-resolution-label = Custom Size

game-branch-label = Branch
branch-online-label = Online
branch-forward-label = Forward
branch-g-label = G
branch-z-label = Z
dev-mode-label = Dev Mode

version-switch-label = Version Check
version-switch-message = Server is set to { $version } game version, would you like to switch to this version?
version-signature-selection-message = Select the client data signature for { $version }. This is used to populate launcher data like friends list/data. If you are not sure, choose <strong>I don't know</strong> to leave it disabled.
version-signature-unknown-label = I don't know
version-switch-dont-ask-again = No, and dont ask again
version-switch-stay-label = Stay on current version
Yes = Yes
Open external link = Open external link
This will open in your browser. Continue? = This will open in your browser. Continue?
OK, don't show again = OK, don't show again
