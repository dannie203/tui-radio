#!/usr/bin/env python3
import sys
import json
import os
import signal
from typing import Dict, Any

try:
    import gi
    gi.require_version('Gio', '2.0')
    gi.require_version('GLib', '2.0')
    from gi.repository import Gio, GLib
except ImportError:
    sys.stderr.write("PyGObject (gi.repository) is not available. Tray/MPRIS2 disabled.\n")
    sys.exit(0)

MPRIS_INTROSPECTION = """
<node>
  <interface name="org.mpris.MediaPlayer2">
    <method name="Raise"/>
    <method name="Quit"/>
    <property name="CanQuit" type="b" access="read"/>
    <property name="CanRaise" type="b" access="read"/>
    <property name="HasTrackList" type="b" access="read"/>
    <property name="Identity" type="s" access="read"/>
    <property name="SupportedUriSchemes" type="as" access="read"/>
    <property name="SupportedMimeTypes" type="as" access="read"/>
  </interface>
  <interface name="org.mpris.MediaPlayer2.Player">
    <method name="Next"/>
    <method name="Previous"/>
    <method name="Pause"/>
    <method name="PlayPause"/>
    <method name="Stop"/>
    <method name="Play"/>
    <method name="Seek">
      <arg direction="in" name="Offset" type="x"/>
    </method>
    <method name="SetPosition">
      <arg direction="in" name="TrackId" type="o"/>
      <arg direction="in" name="Position" type="x"/>
    </method>
    <method name="OpenUri">
      <arg direction="in" name="Uri" type="s"/>
    </method>
    <property name="PlaybackStatus" type="s" access="read"/>
    <property name="LoopStatus" type="s" access="readwrite"/>
    <property name="Rate" type="d" access="readwrite"/>
    <property name="Shuffle" type="b" access="readwrite"/>
    <property name="Metadata" type="a{sv}" access="read"/>
    <property name="Volume" type="d" access="readwrite"/>
    <property name="Position" type="x" access="read"/>
    <property name="MinimumRate" type="d" access="read"/>
    <property name="MaximumRate" type="d" access="read"/>
    <property name="CanControl" type="b" access="read"/>
    <property name="CanPlay" type="b" access="read"/>
    <property name="CanPause" type="b" access="read"/>
    <property name="CanSeek" type="b" access="read"/>
    <property name="CanGoNext" type="b" access="read"/>
    <property name="CanGoPrevious" type="b" access="read"/>
  </interface>
</node>
"""

SNI_INTROSPECTION = """
<node>
  <interface name="org.kde.StatusNotifierItem">
    <property name="Category" type="s" access="read"/>
    <property name="Id" type="s" access="read"/>
    <property name="Title" type="s" access="read"/>
    <property name="Status" type="s" access="read"/>
    <property name="WindowId" type="i" access="read"/>
    <property name="IconName" type="s" access="read"/>
    <property name="IconPixmap" type="a(iiay)" access="read"/>
    <property name="OverlayIconName" type="s" access="read"/>
    <property name="OverlayIconPixmap" type="a(iiay)" access="read"/>
    <property name="AttentionIconName" type="s" access="read"/>
    <property name="AttentionIconPixmap" type="a(iiay)" access="read"/>
    <property name="AttentionMovieName" type="s" access="read"/>
    <property name="ToolTip" type="(sa(iiay)ss)" access="read"/>
    <property name="ItemIsMenu" type="b" access="read"/>
    <property name="Menu" type="o" access="read"/>
    <method name="ContextMenu">
      <arg direction="in" name="x" type="i"/>
      <arg direction="in" name="y" type="i"/>
    </method>
    <method name="Activate">
      <arg direction="in" name="x" type="i"/>
      <arg direction="in" name="y" type="i"/>
    </method>
    <method name="SecondaryActivate">
      <arg direction="in" name="x" type="i"/>
      <arg direction="in" name="y" type="i"/>
    </method>
    <method name="Scroll">
      <arg direction="in" name="delta" type="i"/>
      <arg direction="in" name="orientation" type="s"/>
    </method>
    <signal name="NewTitle"/>
    <signal name="NewIcon"/>
    <signal name="NewAttentionIcon"/>
    <signal name="NewOverlayIcon"/>
    <signal name="NewToolTip"/>
    <signal name="NewStatus">
      <arg name="status" type="s"/>
    </signal>
  </interface>
</node>
"""

DBUSMENU_INTROSPECTION = """
<node>
  <interface name="com.canonical.dbusmenu">
    <property name="Version" type="u" access="read"/>
    <property name="TextDirection" type="s" access="read"/>
    <property name="Status" type="s" access="read"/>
    <property name="IconThemePath" type="as" access="read"/>
    <method name="GetLayout">
      <arg direction="in" name="parentId" type="i"/>
      <arg direction="in" name="recursionDepth" type="i"/>
      <arg direction="in" name="propertyNames" type="as"/>
      <arg direction="out" name="revision" type="u"/>
      <arg direction="out" name="layout" type="(ia{sv}av)"/>
    </method>
    <method name="GetGroupProperties">
      <arg direction="in" name="ids" type="ai"/>
      <arg direction="in" name="propertyNames" type="as"/>
      <arg direction="out" name="properties" type="a(ia{sv})"/>
    </method>
    <method name="GetProperty">
      <arg direction="in" name="id" type="i"/>
      <arg direction="in" name="name" type="s"/>
      <arg direction="out" name="value" type="v"/>
    </method>
    <method name="Event">
      <arg direction="in" name="id" type="i"/>
      <arg direction="in" name="eventId" type="s"/>
      <arg direction="in" name="data" type="v"/>
      <arg direction="in" name="timestamp" type="u"/>
    </method>
    <method name="EventGroup">
      <arg direction="in" name="events" type="a(isvu)"/>
      <arg direction="out" name="idErrors" type="ai"/>
    </method>
    <method name="AboutToShow">
      <arg direction="in" name="id" type="i"/>
      <arg direction="out" name="needUpdate" type="b"/>
    </method>
    <signal name="ItemsPropertiesUpdated">
      <arg name="updatedProps" type="a(ia{sv})"/>
      <arg name="removedProps" type="a(ias)"/>
    </signal>
    <signal name="LayoutUpdated">
      <arg name="revision" type="u"/>
      <arg name="parent" type="i"/>
    </signal>
  </interface>
</node>
"""

class DesktopService:
    def __init__(self):
        self.state = {
            "title": "Nothing playing",
            "artist": "BOOMBOX RX-505",
            "album": "Radio / Library",
            "playing": False,
            "paused": False,
            "volume": 80,
            "timePos": 0,
            "duration": 0,
            "stereoMode": "STEREO",
            "bassBoost": False,
            "dolbyMode": "DOLBY-B"
        }
        self.con = None
        self.menu_revision = 1

    def send_action(self, action: str, data: Any = None):
        msg = {"action": action}
        if data is not None:
            msg["data"] = data
        sys.stdout.write(json.dumps(msg) + "\n")
        sys.stdout.flush()

    def get_mpris_metadata(self):
        dur_us = int(float(self.state.get("duration", 0)) * 1000000)
        meta = {
            "mpris:trackid": GLib.Variant("o", "/org/mpris/MediaPlayer2/track/0"),
            "xesam:title": GLib.Variant("s", str(self.state.get("title", "Nothing playing"))),
            "xesam:artist": GLib.Variant("as", [str(self.state.get("artist", "BOOMBOX"))]),
            "xesam:album": GLib.Variant("s", str(self.state.get("album", ""))),
            "mpris:length": GLib.Variant("x", dur_us)
        }
        return GLib.Variant("a{sv}", meta)

    def update_state(self, new_state: Dict[str, Any]):
        self.state.update(new_state)
        self.menu_revision += 1
        if self.con:
            try:
                is_playing = self.state.get("playing", False) and not self.state.get("paused", False)
                status_str = "Playing" if is_playing else ("Paused" if self.state.get("paused") else "Stopped")
                
                changed_props = {
                    "PlaybackStatus": GLib.Variant("s", status_str),
                    "Volume": GLib.Variant("d", float(self.state.get("volume", 80)) / 100.0),
                    "Metadata": self.get_mpris_metadata()
                }

                prop_signal = GLib.Variant("(sa{sv}as)", (
                    "org.mpris.MediaPlayer2.Player",
                    changed_props,
                    []
                ))
                self.con.emit_signal(None, "/org/mpris/MediaPlayer2", "org.freedesktop.DBus.Properties", "PropertiesChanged", prop_signal)
            except Exception:
                pass

            try:
                self.con.emit_signal(None, "/StatusNotifierItem", "org.kde.StatusNotifierItem", "NewToolTip", None)
                self.con.emit_signal(None, "/StatusNotifierItem", "org.kde.StatusNotifierItem", "NewIcon", None)
                self.con.emit_signal(None, "/MenuBar", "com.canonical.dbusmenu", "LayoutUpdated", GLib.Variant("(ui)", (self.menu_revision, 0)))
            except Exception:
                pass

    def get_menu_items(self):
        title = str(self.state.get("title", "Ready"))
        artist = str(self.state.get("artist", "BOOMBOX RX-505"))
        playing = self.state.get("playing", False) and not self.state.get("paused", False)
        stereo = str(self.state.get("stereoMode", "STEREO")).upper().strip()
        is_3d = stereo in ("3D WIDE", "3D", "WIDE", "STEREO-3D")
        is_stereo = stereo == "STEREO"
        is_mono = stereo == "MONO"
        bass = bool(self.state.get("bassBoost", False))
        dolby = str(self.state.get("dolbyMode", "OFF")).upper()
        has_dolby = dolby != "OFF"
        vol = int(self.state.get("volume", 80))

        return [
            # 1. Header / App Title
            (1, {"label": GLib.Variant("s", "📻 BOOMBOX RX-505 Retro Audio"), "enabled": GLib.Variant("b", False)}),
            (2, {"type": GLib.Variant("s", "separator")}),

            # 2. Soundstage DSP Presets (Radios with checkmark ✓ like EasyEffects output presets)
            (3, {
                "label": GLib.Variant("s", "✦ 3D WIDE (Open-Air Matrix)"),
                "toggle-type": GLib.Variant("s", "radio"),
                "toggle-state": GLib.Variant("i", 1 if is_3d else 0),
                "action": "set_stereo_3d"
            }),
            (4, {
                "label": GLib.Variant("s", "● STEREO (Dry Studio)"),
                "toggle-type": GLib.Variant("s", "radio"),
                "toggle-state": GLib.Variant("i", 1 if is_stereo else 0),
                "action": "set_stereo_stereo"
            }),
            (5, {
                "label": GLib.Variant("s", "◉ MONO (Vintage Broadcast)"),
                "toggle-type": GLib.Variant("s", "radio"),
                "toggle-state": GLib.Variant("i", 1 if is_mono else 0),
                "action": "set_stereo_mono"
            }),
            (6, {"type": GLib.Variant("s", "separator")}),

            # 3. Hardware FX Enhancements (Checkable toggles with checkmark ✓ like EasyEffects Active)
            (7, {
                "label": GLib.Variant("s", "🔊 Mega Bass Boost (+7dB)"),
                "toggle-type": GLib.Variant("s", "checkmark"),
                "toggle-state": GLib.Variant("i", 1 if bass else 0),
                "action": "toggle_bass"
            }),
            (8, {
                "label": GLib.Variant("s", f"🎚 Dolby NR Tape Bias [{dolby}]"),
                "toggle-type": GLib.Variant("s", "checkmark"),
                "toggle-state": GLib.Variant("i", 1 if has_dolby else 0),
                "action": "cycle_dolby"
            }),
            (9, {"type": GLib.Variant("s", "separator")}),

            # 4. Now Playing Status & Device Info (like EasyEffects Device section)
            (10, {"label": GLib.Variant("s", f"🎵 {title}"), "enabled": GLib.Variant("b", False)}),
            (11, {"label": GLib.Variant("s", f"🎙️ {artist}"), "enabled": GLib.Variant("b", False)}),
            (12, {"type": GLib.Variant("s", "separator")}),

            # 5. Playback & Volume Actions
            (13, {
                "label": GLib.Variant("s", "✓ Active Playback" if playing else "⏸ Paused (Click to Play)"),
                "toggle-type": GLib.Variant("s", "checkmark"),
                "toggle-state": GLib.Variant("i", 1 if playing else 0),
                "action": "play_pause"
            }),
            (14, {"label": GLib.Variant("s", "⏭ Next Track"), "action": "next"}),
            (15, {"label": GLib.Variant("s", "⏮ Previous Track"), "action": "prev"}),
            (16, {"label": GLib.Variant("s", f"🔊 Volume: {vol}% (+5%)"), "action": "volume_up"}),
            (17, {"label": GLib.Variant("s", f"🔉 Volume: {vol}% (-5%)"), "action": "volume_down"}),
            (18, {"type": GLib.Variant("s", "separator")}),

            # 6. Shortcuts & Manual (like EasyEffects Shortcuts & Manual)
            (19, {"label": GLib.Variant("s", "📟 Open / Focus Boombox TUI"), "action": "open_tui"}),
            (20, {"label": GLib.Variant("s", "⌨️ Keypad Shortcuts"), "action": "open_tui"}),
            (21, {"type": GLib.Variant("s", "separator")}),

            # 7. Quit
            (22, {"label": GLib.Variant("s", "✕ Quit"), "action": "quit"})
        ]

    # --- MPRIS2 Callbacks ---
    def handle_mpris_method(self, connection, sender, path, iface, method, params, invocation):
        if iface == "org.mpris.MediaPlayer2":
            if method == "Raise":
                self.send_action("open_tui")
                invocation.return_value(None)
            elif method == "Quit":
                self.send_action("quit")
                invocation.return_value(None)
        elif iface == "org.mpris.MediaPlayer2.Player":
            if method == "PlayPause":
                self.send_action("play_pause")
                invocation.return_value(None)
            elif method == "Play":
                self.send_action("play")
                invocation.return_value(None)
            elif method == "Pause":
                self.send_action("pause")
                invocation.return_value(None)
            elif method == "Stop":
                self.send_action("stop")
                invocation.return_value(None)
            elif method == "Next":
                self.send_action("next")
                invocation.return_value(None)
            elif method == "Previous":
                self.send_action("prev")
                invocation.return_value(None)
            elif method == "Seek":
                offset_us = params[0]
                self.send_action("seek", offset_us / 1000000.0)
                invocation.return_value(None)
            elif method == "SetPosition":
                pos_us = params[1]
                self.send_action("set_position", pos_us / 1000000.0)
                invocation.return_value(None)
            else:
                invocation.return_value(None)

    def handle_mpris_get_prop(self, connection, sender, path, iface, prop):
        if iface == "org.mpris.MediaPlayer2":
            if prop == "CanQuit": return GLib.Variant("b", True)
            if prop == "CanRaise": return GLib.Variant("b", True)
            if prop == "HasTrackList": return GLib.Variant("b", False)
            if prop == "Identity": return GLib.Variant("s", "BOOMBOX RX-505 Retro Audio Player")
            if prop == "SupportedUriSchemes": return GLib.Variant("as", ["file", "http", "https"])
            if prop == "SupportedMimeTypes": return GLib.Variant("as", ["audio/mpeg", "audio/flac", "audio/ogg", "audio/opus"])
        elif iface == "org.mpris.MediaPlayer2.Player":
            if prop == "PlaybackStatus":
                is_playing = self.state.get("playing", False) and not self.state.get("paused", False)
                return GLib.Variant("s", "Playing" if is_playing else ("Paused" if self.state.get("paused") else "Stopped"))
            if prop == "LoopStatus": return GLib.Variant("s", "None")
            if prop == "Rate": return GLib.Variant("d", 1.0)
            if prop == "Shuffle": return GLib.Variant("b", False)
            if prop == "Volume": return GLib.Variant("d", float(self.state.get("volume", 80)) / 100.0)
            if prop == "Position": return GLib.Variant("x", int(float(self.state.get("timePos", 0)) * 1000000))
            if prop == "CanControl": return GLib.Variant("b", True)
            if prop == "CanPlay": return GLib.Variant("b", True)
            if prop == "CanPause": return GLib.Variant("b", True)
            if prop == "CanSeek": return GLib.Variant("b", True)
            if prop == "CanGoNext": return GLib.Variant("b", True)
            if prop == "CanGoPrevious": return GLib.Variant("b", True)
            if prop == "Metadata": return self.get_mpris_metadata()
        return None

    # --- SNI Tray Callbacks ---
    def handle_sni_method(self, connection, sender, path, iface, method, params, invocation):
        if method == "Activate":
            # Left-click on Tray Icon: Toggle/Reopen the TUI window (EasyEffects-style)
            self.send_action("open_tui")
            invocation.return_value(None)
        elif method == "SecondaryActivate":
            self.send_action("open_tui")
            invocation.return_value(None)
        elif method == "Scroll":
            delta, orientation = params
            if delta > 0: self.send_action("volume_down")
            else: self.send_action("volume_up")
            invocation.return_value(None)
        elif method == "ContextMenu":
            invocation.return_value(None)

    def handle_sni_get_prop(self, connection, sender, path, iface, prop):
        if prop == "Category": return GLib.Variant("s", "ApplicationStatus")
        if prop == "Id": return GLib.Variant("s", "hiphop-radio-tui")
        if prop == "Title": return GLib.Variant("s", "BOOMBOX RX-505")
        if prop == "Status": return GLib.Variant("s", "Active")
        if prop == "WindowId": return GLib.Variant("i", 0)
        if prop == "IconName":
            is_playing = self.state.get("playing", False) and not self.state.get("paused", False)
            return GLib.Variant("s", "media-playback-start" if is_playing else "audio-player")
        if prop == "IconPixmap": return GLib.Variant("a(iiay)", [])
        if prop == "OverlayIconName": return GLib.Variant("s", "")
        if prop == "OverlayIconPixmap": return GLib.Variant("a(iiay)", [])
        if prop == "AttentionIconName": return GLib.Variant("s", "")
        if prop == "AttentionIconPixmap": return GLib.Variant("a(iiay)", [])
        if prop == "AttentionMovieName": return GLib.Variant("s", "")
        if prop == "ItemIsMenu": return GLib.Variant("b", False)
        if prop == "Menu": return GLib.Variant("o", "/MenuBar")
        if prop == "ToolTip":
            title = str(self.state.get("title", "BOOMBOX RX-505"))
            artist = str(self.state.get("artist", "Ready"))
            is_playing = self.state.get("playing", False) and not self.state.get("paused", False)
            status = "Playing" if is_playing else "Paused / Standby"
            return GLib.Variant("(sa(iiay)ss)", ("audio-player", [], f"BOOMBOX RX-505 [{status}]", f"{title}\n{artist}"))
        return None

    # --- DBusMenu Callbacks ---
    def handle_menu_method(self, connection, sender, path, iface, method, params, invocation):
        if method == "GetLayout":
            parent_id, depth, prop_names = params
            menu_items = self.get_menu_items()
            items_by_id = dict((item[0], item[1]) for item in menu_items)
            
            if parent_id == 0:
                child_variants = []
                for item_id, props in menu_items:
                    prop_dict = {}
                    for k, v in props.items():
                        if k != "action":
                            if not prop_names or k in prop_names:
                                prop_dict[k] = v
                    child_node = GLib.Variant("(ia{sv}av)", (item_id, prop_dict, []))
                    child_variants.append(GLib.Variant("v", child_node))
                
                root_props = {"children-display": GLib.Variant("s", "submenu")}
                root_layout = (0, root_props, child_variants)
                ret = GLib.Variant("(u(ia{sv}av))", (self.menu_revision, root_layout))
                invocation.return_value(ret)
            elif parent_id in items_by_id:
                prop_dict = {}
                for k, v in items_by_id[parent_id].items():
                    if k != "action":
                        if not prop_names or k in prop_names:
                            prop_dict[k] = v
                leaf_layout = (parent_id, prop_dict, [])
                ret = GLib.Variant("(u(ia{sv}av))", (self.menu_revision, leaf_layout))
                invocation.return_value(ret)
            else:
                empty_layout = (parent_id, {}, [])
                ret = GLib.Variant("(u(ia{sv}av))", (self.menu_revision, empty_layout))
                invocation.return_value(ret)
        elif method == "GetGroupProperties":
            ids, prop_names = params
            menu_items = dict((item[0], item[1]) for item in self.get_menu_items())
            result = []
            for item_id in ids:
                if item_id == 0:
                    result.append((0, {"children-display": GLib.Variant("s", "submenu")}))
                elif item_id in menu_items:
                    prop_dict = {}
                    for k, v in menu_items[item_id].items():
                        if k != "action":
                            if not prop_names or k in prop_names:
                                prop_dict[k] = v
                    result.append((item_id, prop_dict))
            invocation.return_value(GLib.Variant("(a(ia{sv}))", (result,)))
        elif method == "GetProperty":
            item_id, name = params
            if item_id == 0:
                if name == "children-display":
                    invocation.return_value(GLib.Variant("(v)", (GLib.Variant("s", "submenu"),)))
                else:
                    invocation.return_value(GLib.Variant("(v)", (GLib.Variant("s", ""),)))
            else:
                menu_items = dict((item[0], item[1]) for item in self.get_menu_items())
                if item_id in menu_items and name in menu_items[item_id]:
                    invocation.return_value(GLib.Variant("(v)", (menu_items[item_id][name],)))
                else:
                    invocation.return_value(GLib.Variant("(v)", (GLib.Variant("s", ""),)))
        elif method == "Event":
            item_id, event_id, data, ts = params
            menu_items = dict((item[0], item[1]) for item in self.get_menu_items())
            if item_id in menu_items and "action" in menu_items[item_id]:
                self.send_action(menu_items[item_id]["action"])
            invocation.return_value(None)
        elif method == "EventGroup":
            events = params[0]
            menu_items = dict((item[0], item[1]) for item in self.get_menu_items())
            for item_id, event_id, data, ts in events:
                if item_id in menu_items and "action" in menu_items[item_id]:
                    self.send_action(menu_items[item_id]["action"])
            invocation.return_value(GLib.Variant("(ai)", ([],)))
        elif method == "AboutToShow":
            invocation.return_value(GLib.Variant("(b)", (False,)))
        else:
            invocation.return_value(None)

    def handle_menu_get_prop(self, connection, sender, path, iface, prop):
        if prop == "Version": return GLib.Variant("u", 3)
        if prop == "TextDirection": return GLib.Variant("s", "ltr")
        if prop == "Status": return GLib.Variant("s", "normal")
        if prop == "IconThemePath": return GLib.Variant("as", [])
        return None

    def on_bus_acquired(self, connection, name):
        self.con = connection
        
        # 1. Register MPRIS2
        mpris_node = Gio.DBusNodeInfo.new_for_xml(MPRIS_INTROSPECTION)
        for iface_info in mpris_node.interfaces:
            connection.register_object(
                "/org/mpris/MediaPlayer2",
                iface_info,
                self.handle_mpris_method,
                self.handle_mpris_get_prop,
                None
            )

        # 2. Register SNI
        sni_node = Gio.DBusNodeInfo.new_for_xml(SNI_INTROSPECTION)
        connection.register_object(
            "/StatusNotifierItem",
            sni_node.interfaces[0],
            self.handle_sni_method,
            self.handle_sni_get_prop,
            None
        )

        # 3. Register DBusMenu
        menu_node = Gio.DBusNodeInfo.new_for_xml(DBUSMENU_INTROSPECTION)
        connection.register_object(
            "/MenuBar",
            menu_node.interfaces[0],
            self.handle_menu_method,
            self.handle_menu_get_prop,
            None
        )

        # 4. Register with StatusNotifierWatcher if available
        try:
            connection.call(
                "org.kde.StatusNotifierWatcher",
                "/StatusNotifierWatcher",
                "org.kde.StatusNotifierWatcher",
                "RegisterStatusNotifierItem",
                GLib.Variant("(s)", ("org.mpris.MediaPlayer2.hiphop_radio",)),
                None,
                Gio.DBusCallFlags.NONE,
                1000,
                None,
                None,
                None
            )
        except Exception:
            pass

    def run(self):
        Gio.bus_own_name(
            Gio.BusType.SESSION,
            "org.mpris.MediaPlayer2.hiphop_radio",
            Gio.BusNameOwnerFlags.NONE,
            self.on_bus_acquired,
            None,
            None
        )

        loop = GLib.MainLoop()

        def on_stdin_read(channel, condition):
            if condition & GLib.IOCondition.IN:
                line = sys.stdin.readline()
                if not line:
                    loop.quit()
                    return False
                try:
                    data = json.loads(line.strip())
                    if data.get("type") == "UPDATE":
                        self.update_state(data.get("state", {}))
                    elif data.get("type") == "QUIT":
                        loop.quit()
                        return False
                except Exception:
                    pass
                return True
            return False

        io_channel = GLib.IOChannel.unix_new(sys.stdin.fileno())
        GLib.io_add_watch(io_channel, GLib.PRIORITY_DEFAULT, GLib.IOCondition.IN | GLib.IOCondition.HUP, on_stdin_read)

        signal.signal(signal.SIGINT, lambda s, f: loop.quit())
        signal.signal(signal.SIGTERM, lambda s, f: loop.quit())

        loop.run()

if __name__ == '__main__':
    service = DesktopService()
    service.run()
