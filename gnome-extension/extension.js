// whiskerd top-bar indicator. Reads $XDG_RUNTIME_DIR/whiskerd.json (written by the whiskerd
// daemon) and re-renders only when the file changes (inotify); it never polls.

import GLib from 'gi://GLib';
import Gio from 'gi://Gio';
import St from 'gi://St';
import Clutter from 'gi://Clutter';

import * as Main from 'resource:///org/gnome/shell/ui/main.js';
import * as PanelMenu from 'resource:///org/gnome/shell/ui/panelMenu.js';
import * as PopupMenu from 'resource:///org/gnome/shell/ui/popupMenu.js';
import {Extension} from 'resource:///org/gnome/shell/extensions/extension.js';

const LOW = 20;
// Theme icons by device kind. Mice use the bundled mouse.png instead (see _icon).
const ICONS = {
    keyboard: 'input-keyboard-symbolic',
    device: 'battery-symbolic',
};

function ago(updated) {
    if (!updated)
        return '';
    const mins = Math.max(0, Math.round((Date.now() / 1000 - updated) / 60));
    if (mins < 1)
        return 'just now';
    if (mins < 60)
        return `${mins} min ago`;
    return `${Math.round(mins / 60)} h ago`;
}

export default class WhiskerdExtension extends Extension {
    enable() {
        this._button = new PanelMenu.Button(0.0, this.metadata.name, false);
        this._box = new St.BoxLayout({style_class: 'panel-status-menu-box'});
        this._button.add_child(this._box);
        Main.panel.addToStatusArea(this.uuid, this._button);

        this._mouseIcon = Gio.FileIcon.new(this.dir.get_child('mouse.png'));
        this._devices = [];
        this._menuId = this._button.menu.connect('open-state-changed', (_m, open) => {
            if (open)
                this._buildMenu();
        });

        const path = GLib.build_filenamev([GLib.get_user_runtime_dir(), 'whiskerd.json']);
        this._file = Gio.File.new_for_path(path);
        this._monitor = this._file.monitor_file(Gio.FileMonitorFlags.WATCH_MOVES, null);
        this._monitorId = this._monitor.connect('changed', () => this._reload());
        this._reload();
    }

    disable() {
        this._monitor?.disconnect(this._monitorId);
        this._monitor?.cancel();
        this._button?.menu.disconnect(this._menuId);
        this._button?.destroy();
        this._monitor = null;
        this._file = null;
        this._button = null;
        this._box = null;
        this._mouseIcon = null;
        this._devices = null;
    }

    _read() {
        try {
            const [, bytes] = this._file.load_contents(null);
            const data = JSON.parse(new TextDecoder().decode(bytes));
            return Array.isArray(data.devices) ? data.devices : [];
        } catch (_e) {
            return []; // daemon not running, or file being replaced
        }
    }

    _reload() {
        if (!this._button)
            return;
        this._devices = this._read();
        this._box.destroy_all_children();
        const shown = this._devices.filter(d => d.battery !== null && d.battery !== undefined);
        for (const d of shown) {
            this._box.add_child(this._icon(d.kind));
            const label = new St.Label({
                text: `${d.battery}%${d.charging ? ' ⚡' : ''}`,
                y_align: Clutter.ActorAlign.CENTER,
                style_class: 'whiskerd-label',
            });
            if (d.battery <= LOW && !d.charging)
                label.add_style_class_name('whiskerd-low');
            if (!d.connected)
                label.add_style_class_name('whiskerd-stale');
            this._box.add_child(label);
        }
        this._button.visible = shown.length > 0;
        if (this._button.menu.isOpen)
            this._buildMenu();
    }

    _icon(kind) {
        if (kind === 'mouse') {
            // Full-colour artwork: not recoloured by the shell theme like symbolic icons.
            return new St.Icon({gicon: this._mouseIcon, style_class: 'system-status-icon'});
        }
        return new St.Icon({
            icon_name: ICONS[kind] ?? ICONS.device,
            style_class: 'system-status-icon',
        });
    }

    _buildMenu() {
        const menu = this._button.menu;
        menu.removeAll();
        for (const d of this._devices) {
            let text = d.battery === null || d.battery === undefined
                ? `${d.name}: no reading yet`
                : `${d.name}: ${d.battery}%${d.charging ? ' (charging)' : ''}`;
            const when = ago(d.updated);
            if (when)
                text += `  ·  ${when}`;
            if (!d.connected)
                text += '  ·  asleep';
            const item = new PopupMenu.PopupMenuItem(text, {reactive: false});
            menu.addMenuItem(item);
        }
    }
}
