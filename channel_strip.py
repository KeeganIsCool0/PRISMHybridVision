#!/usr/bin/env python3
"""Channel Strip — a GTK4/GStreamer offline-friendly audio channel strip."""
import math
import os
import sys
import threading
from pathlib import Path

import numpy as np
import gi
gi.require_version("Gtk", "4.0")
gi.require_version("Gst", "1.0")
from gi.repository import Gtk, Gio, GLib, Gst, Gdk

Gst.init(None)

CSS = b"""
window { background: #15171a; color: #ece7dc; }
headerbar { background: #20242a; color: #f5efe2; }
button { background: #30363d; color: #f5efe2; border-radius: 5px; border: 1px solid #4a535d; }
button:hover { background: #424b55; }
button.solo { background: #b47a2b; color: #18120a; font-weight: bold; }
.strip { background: linear-gradient(180deg, #303439, #202326); border: 1px solid #4f5455; border-radius: 8px; padding: 10px; }
.module-title { color: #f0bd55; font-size: 11px; font-weight: 800; letter-spacing: 1px; }
.module { border-top: 2px solid #be8a35; padding-top: 7px; margin-top: 5px; }
.readout { color: #b9c7cf; font-family: monospace; font-size: 10px; }
.knob-label { font-size: 9px; color: #d7d1c5; }
scale trough { min-width: 7px; min-height: 7px; background: #101214; border-radius: 5px; }
scale highlight { background: #d19a39; border-radius: 5px; }
scale slider { background: #e5c071; border: 1px solid #111; border-radius: 50%; min-width: 16px; min-height: 16px; }
.meter { background: #261b16; border: 1px solid #62452b; border-radius: 8px; padding: 5px; }
.meter.on { background: #e6a22b; color: #1d1205; }
progressbar trough { min-height: 7px; background: #30343a; border-radius: 4px; }
progressbar progress { background: #d19a39; border-radius: 4px; }
"""

class ChannelStrip(Gtk.Application):
    def __init__(self):
        super().__init__(application_id="com.channelstrip.gstreamer")
        self.pipeline = None
        self.exporting = False
        self.file_path = None
        self.duration = 0
        self.user_seeking = False
        self.position_updating = False
        self.controls = {}
        self.processors = {}
        self.playing = False
        self.analysis_lock = threading.Lock()
        self.analysis_samples = np.zeros(8192, dtype=np.float32)
        self.spectrogram = np.zeros((96, 128), dtype=np.float32)
        GLib.timeout_add(120, self._tick)
        GLib.timeout_add(45, self._redraw_analyzers)

    def do_activate(self):
        provider = Gtk.CssProvider()
        provider.load_from_data(CSS)
        Gtk.StyleContext.add_provider_for_display(Gdk.Display.get_default(), provider,
            Gtk.STYLE_PROVIDER_PRIORITY_APPLICATION)
        self.win = Gtk.ApplicationWindow(application=self, title="Channel Strip")
        self.win.set_default_size(1440, 760)
        self.win.connect("close-request", self._close)
        open_action = Gio.SimpleAction.new("open_audio", None)
        open_action.connect("activate", self._open_dialog)
        export_action = Gio.SimpleAction.new("export_audio", None)
        export_action.connect("activate", self._export_dialog)
        self.win.add_action(open_action)
        self.win.add_action(export_action)
        header = Gtk.HeaderBar()
        self.win.set_titlebar(header)
        file_menu = Gio.Menu()
        file_menu.append("Open Audio…", "win.open_audio")
        file_menu.append("Export Processed WAV…", "win.export_audio")
        header.pack_start(Gtk.MenuButton(label="File", menu_model=file_menu))
        self.status = Gtk.Label(label="Open an audio file to begin")
        header.set_title_widget(self.status)

        root = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=8)
        root.set_margin_top(10); root.set_margin_bottom(10)
        root.set_margin_start(12); root.set_margin_end(12)
        root.append(self._transport())
        scroll = Gtk.ScrolledWindow(vexpand=True, hscrollbar_policy=Gtk.PolicyType.AUTOMATIC)
        strip_row = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=9)
        strip_row.set_margin_start(4); strip_row.set_margin_end(4)
        for title, content in [
            ("INPUT / PREAMP", self._preamp()), ("FILTERS", self._filters()),
            ("GATE / EXPANDER", self._gate()), ("5-BAND EQ", self._eq5()),
            ("10-BAND GRAPHIC", self._eq10()), ("OPTO COMP", self._opto()),
            ("VCA COMP", self._vca()), ("OUTPUT", self._output())]:
            strip_row.append(self._module(title, content))
        scroll.set_child(strip_row)
        root.append(scroll)
        root.append(self._analyzers())
        self.win.set_child(root)
        self.win.present()

    def _module(self, title, child):
        box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=4, css_classes=["strip"])
        box.append(Gtk.Label(label=title, css_classes=["module-title"]))
        box.append(child)
        return box

    def _transport(self):
        box = Gtk.Box(spacing=8)
        self.play_btn = Gtk.Button(label="▶  Play", sensitive=False)
        self.pause_btn = Gtk.Button(label="❚❚  Pause", sensitive=False)
        self.play_btn.connect("clicked", lambda *_: self._set_playing(True))
        self.pause_btn.connect("clicked", lambda *_: self._set_playing(False))
        box.append(self.play_btn); box.append(self.pause_btn)
        self.position = Gtk.Scale.new_with_range(Gtk.Orientation.HORIZONTAL, 0, 100, .1)
        self.position.set_hexpand(True); self.position.set_draw_value(False)
        self.position.connect("value-changed", self._seek)
        box.append(self.position)
        self.time_label = Gtk.Label(label="00:00 / 00:00", css_classes=["readout"])
        box.append(self.time_label)
        return box

    def _analyzers(self):
        """Live visual monitors for the processed (post-fader) signal."""
        row = Gtk.Box(spacing=8)
        scope_box, self.scope = self._analyzer("OSCILLOSCOPE", self._draw_scope)
        wave_box, self.waveform = self._analyzer("WAVEFORM", self._draw_waveform)
        spectrum_box, self.spectrum = self._analyzer("SPECTRUM", self._draw_spectrum)
        spectro_box, self.spectro = self._analyzer("SPECTROGRAM", self._draw_spectrogram)
        for widget in (scope_box, wave_box, spectrum_box, spectro_box): row.append(widget)
        return row

    def _analyzer(self, title, renderer):
        frame = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=3, hexpand=True, css_classes=["strip"])
        frame.append(Gtk.Label(label=title, css_classes=["module-title"]))
        area = Gtk.DrawingArea(content_width=280, content_height=130, hexpand=True)
        area.set_draw_func(renderer)
        frame.append(area)
        return frame, area

    def _analysis_copy(self):
        with self.analysis_lock: return self.analysis_samples.copy(), self.spectrogram.copy()

    @staticmethod
    def _background(ctx, width, height):
        ctx.set_source_rgb(.045, .052, .055); ctx.rectangle(0, 0, width, height); ctx.fill()
        ctx.set_source_rgba(.62, .48, .20, .18); ctx.set_line_width(1)
        for x in range(0, int(width), max(1, int(width / 8))): ctx.move_to(x, 0); ctx.line_to(x, height)
        for y in range(0, int(height), max(1, int(height / 4))): ctx.move_to(0, y); ctx.line_to(width, y)
        ctx.stroke()

    def _draw_scope(self, area, ctx, width, height):
        samples, _ = self._analysis_copy(); self._background(ctx, width, height)
        data = samples[-1024:]
        ctx.set_source_rgb(.95, .64, .18); ctx.set_line_width(1.3)
        for x, value in enumerate(data):
            px = x * width / (len(data) - 1); py = height * .5 - value * height * .42
            ctx.move_to(px, py) if x == 0 else ctx.line_to(px, py)
        ctx.stroke()

    def _draw_waveform(self, area, ctx, width, height):
        samples, _ = self._analysis_copy(); self._background(ctx, width, height)
        columns = max(1, int(width)); blocks = np.array_split(samples, columns)
        ctx.set_source_rgba(.83, .58, .16, .75); ctx.set_line_width(1)
        for x, block in enumerate(blocks):
            top = height * .5 - float(np.max(block)) * height * .43
            bottom = height * .5 - float(np.min(block)) * height * .43
            ctx.move_to(x, top); ctx.line_to(x, bottom)
        ctx.stroke()

    def _draw_spectrum(self, area, ctx, width, height):
        samples, _ = self._analysis_copy(); self._background(ctx, width, height)
        data = samples[-2048:] * np.hanning(2048)
        magnitudes = 20 * np.log10(np.maximum(np.abs(np.fft.rfft(data)), 1e-7))
        magnitudes = np.clip((magnitudes + 72) / 72, 0, 1)
        # Log-ish sampling makes the low frequencies legible.
        bins = np.unique(np.geomspace(1, len(magnitudes) - 1, int(width)).astype(int))
        ctx.set_source_rgba(.90, .64, .18, .9); ctx.set_line_width(max(1, width / len(bins)))
        for i, index in enumerate(bins):
            x = i * width / max(1, len(bins) - 1); ctx.move_to(x, height); ctx.line_to(x, height * (1 - magnitudes[index]))
        ctx.stroke()

    def _draw_spectrogram(self, area, ctx, width, height):
        _, image = self._analysis_copy(); self._background(ctx, width, height)
        cell_w, cell_h = width / image.shape[0], height / image.shape[1]
        for x in range(image.shape[0]):
            for y in range(image.shape[1]):
                value = float(image[x, image.shape[1] - 1 - y])
                # Warm console palette: black → brown → amber → cream.
                ctx.set_source_rgb(min(1, value * 1.7), min(1, value * value * 1.25), value * .20)
                ctx.rectangle(x * cell_w, y * cell_h, cell_w + 1, cell_h + 1); ctx.fill()

    def _redraw_analyzers(self):
        for area in (getattr(self, "scope", None), getattr(self, "waveform", None), getattr(self, "spectrum", None), getattr(self, "spectro", None)):
            if area: area.queue_draw()
        return True

    def _preamp(self):
        box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=6)
        preamps = Gtk.Box(spacing=3)
        self.preamp = "API"
        self.preamp_buttons = []
        for name in ("API", "NEVE", "TUBE"):
            b = Gtk.ToggleButton(label=name, active=name == "API")
            b.connect("toggled", self._preamp_selected, name)
            preamps.append(b)
            self.preamp_buttons.append(b)
        box.append(preamps)
        box.append(Gtk.Label(label="CLEAN ↔ VINTAGE", css_classes=["knob-label"]))
        box.append(self._knob("character", "CHARACTER", 0, 100, 50, ""))
        invert = Gtk.Switch(); invert.connect("state-set", self._invert)
        row = Gtk.Box(spacing=5); row.append(Gtk.Label(label="Ø POLARITY", css_classes=["knob-label"])); row.append(invert)
        box.append(row); box.append(self._knob("preamp_gain", "GAIN", -18, 36, 0, "dB"))
        return box

    def _filters(self):
        box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=6)
        box.append(self._knob("hpf", "HIGH PASS", 20, 800, 20, "Hz"))
        box.append(self._knob("lpf", "LOW PASS", 1000, 20000, 20000, "Hz"))
        return box

    def _gate(self):
        box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=6)
        for key, label, lo, hi, default, unit in [
            ("gate_thresh", "THRESH", -70, 0, -50, "dB"), ("gate_depth", "DEPTH", 0, 30, 0, "dB"),
            ("gate_hold", "RELEASE / HOLD", 5, 500, 100, "ms")]: box.append(self._knob(key, label, lo, hi, default, unit))
        sw = Gtk.Switch(); sw.connect("state-set", self._fast_attack)
        row = Gtk.Box(spacing=5); row.append(Gtk.Label(label="FAST ATTACK", css_classes=["knob-label"])); row.append(sw); box.append(row)
        return box

    def _eq5(self):
        box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=4)
        for i, hz in enumerate((80, 250, 1000, 4000, 12000)): box.append(self._knob(f"eq5_{i}", f"{hz} Hz", -15, 15, 0, "dB"))
        return box

    def _eq10(self):
        box = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=2)
        for i, hz in enumerate((29, 59, 119, 237, 474, 947, 1889, 3770, 7523, 15011)):
            box.append(self._knob(f"eq10_{i}", self._freq_label(hz), -15, 15, 0, "dB", compact=True))
        return box

    @staticmethod
    def _freq_label(hz): return f"{hz / 1000:g}k" if hz >= 1000 else str(hz)

    def _opto(self):
        box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=6)
        for key, label, lo, hi, default, unit in [
            ("opto_thresh", "THRESH", -50, 0, -18, "dB"), ("opto_release", "RELEASE", 40, 1200, 350, "ms"),
            ("opto_attack", "ATTACK", 1, 150, 20, "ms"), ("opto_ratio", "RATIO", 1, 20, 3, ":1")]: box.append(self._knob(key, label, lo, hi, default, unit))
        self.opto_light = Gtk.Label(label="GAIN REDUCTION", css_classes=["meter"])
        box.append(self.opto_light)
        return box

    def _vca(self):
        box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=6)
        for key, label, lo, hi, default, unit in [
            ("vca_thresh", "THRESH", -50, 0, -12, "dB"), ("vca_release", "RELEASE", 20, 800, 150, "ms"),
            ("vca_attack", "ATTACK", 1, 100, 10, "ms"), ("vca_ratio", "RATIO", 1, 20, 4, ":1"),
            ("vca_gain", "MAKEUP GAIN", -12, 24, 0, "dB")]: box.append(self._knob(key, label, lo, hi, default, unit))
        return box

    def _output(self):
        box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=6)
        box.append(self._knob("fader", "FADER", -70, 12, 0, "dB"))
        return box

    def _knob(self, key, label, lo, hi, value, unit, compact=False):
        box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=1)
        box.append(Gtk.Label(label=label, css_classes=["knob-label"]))
        scale = Gtk.Scale.new_with_range(Gtk.Orientation.VERTICAL, lo, hi, .1 if hi-lo < 1000 else 1)
        scale.set_inverted(True); scale.set_value(value); scale.set_size_request(30 if compact else 54, 150 if not compact else 148)
        scale.set_draw_value(False); scale.connect("value-changed", self._control_changed, key)
        box.append(scale)
        readout = Gtk.Label(label=self._format(value, unit), css_classes=["readout"])
        box.append(readout); self.controls[key] = (scale, readout, unit)
        return box

    @staticmethod
    def _format(value, unit): return f"{value:.0f}{unit}" if unit != ":1" else f"{value:.1f}{unit}"

    def _value(self, key): return self.controls[key][0].get_value()
    def _control_changed(self, scale, key):
        _, readout, unit = self.controls[key]; readout.set_label(self._format(scale.get_value(), unit))
        self._apply_controls()

    def _preamp_selected(self, button, name):
        if button.get_active():
            self.preamp = name
            for sibling in self.preamp_buttons:
                if sibling is not button: sibling.set_active(False)
            self._apply_controls()

    def _invert(self, switch, state):
        if self.processors.get("invert"): self.processors["invert"].set_property("degree", 1.0 if state else 0.0)
        return False
    def _fast_attack(self, switch, state): self.fast_attack = state; self._apply_controls(); return False

    def _apply_controls(self):
        if not self.processors: return
        db = lambda x: math.pow(10, x / 20.0)
        color = {"API": 1.0, "NEVE": 1.10, "TUBE": 1.22}[self.preamp]
        character = self._value("character") / 100
        self.processors["input"].set_property("volume", db(self._value("preamp_gain")) * (1 + character * (color - 1)))
        self.processors["hpf"].set_property("cutoff", self._value("hpf"))
        self.processors["lpf"].set_property("cutoff", self._value("lpf"))
        self.processors["gate"].set_property("threshold", db(self._value("gate_thresh")))
        self.processors["gate"].set_property("ratio", 1 + self._value("gate_depth") / 4)
        for i in range(5): self.processors["eq5"].get_child_by_index(i).set_property("gain", self._value(f"eq5_{i}"))
        for i in range(10): self.processors["eq10"].set_property(f"band{i}", self._value(f"eq10_{i}"))
        self.processors["opto"].set_property("threshold", db(self._value("opto_thresh")))
        self.processors["opto"].set_property("ratio", self._value("opto_ratio"))
        self.processors["vca"].set_property("threshold", db(self._value("vca_thresh")))
        self.processors["vca"].set_property("ratio", self._value("vca_ratio"))
        self.processors["output"].set_property("volume", db(self._value("fader") + self._value("vca_gain")))

    def _new_pipeline(self, export_path=None):
        p = Gst.Pipeline.new("channel-strip")
        decode = Gst.ElementFactory.make("uridecodebin", "decoder")
        decode.set_property("uri", Gio.File.new_for_path(self.file_path).get_uri())
        conv1 = Gst.ElementFactory.make("audioconvert", None); resample = Gst.ElementFactory.make("audioresample", None)
        input_vol = Gst.ElementFactory.make("volume", "input"); invert = Gst.ElementFactory.make("audioinvert", "invert")
        hpf = Gst.ElementFactory.make("audiocheblimit", "hpf"); hpf.set_property("mode", 1)
        lpf = Gst.ElementFactory.make("audiocheblimit", "lpf"); lpf.set_property("mode", 0)
        gate = Gst.ElementFactory.make("audiodynamic", "gate"); gate.set_property("mode", 1)
        # This portable factory exposes five independently configurable bands.
        eq5 = Gst.ElementFactory.make("equalizer-nbands", "eq5")
        eq5.set_property("num-bands", 5)
        for index, frequency in enumerate((80, 250, 1000, 4000, 12000)):
            band = eq5.get_child_by_index(index)
            band.set_property("freq", frequency)
            band.set_property("bandwidth", frequency * .7)
        eq10 = Gst.ElementFactory.make("equalizer-10bands", "eq10")
        opto = Gst.ElementFactory.make("audiodynamic", "opto"); opto.set_property("characteristics", 1)
        vca = Gst.ElementFactory.make("audiodynamic", "vca")
        output = Gst.ElementFactory.make("volume", "output")
        tail = [Gst.ElementFactory.make("audioconvert", None), Gst.ElementFactory.make("audioresample", None)]
        if export_path:
            tail += [Gst.ElementFactory.make("wavenc", None), Gst.ElementFactory.make("filesink", None)]
            tail[-1].set_property("location", export_path)
            chain = [decode, conv1, resample, input_vol, invert, hpf, lpf, gate, eq5, eq10, opto, vca, output] + tail
            if not all(chain): raise RuntimeError("Required GStreamer element unavailable")
            p.add(*chain); Gst.Element.link_many(*chain[1:])
        else:
            # Tee the *processed* signal: one branch plays, the other supplies
            # mono floating-point samples to the four analysis monitors.
            tee = Gst.ElementFactory.make("tee", "analysis-tee")
            playback = [Gst.ElementFactory.make("queue", None)] + tail + [Gst.ElementFactory.make("autoaudiosink", None)]
            capture_queue = Gst.ElementFactory.make("queue", None)
            capture_convert = Gst.ElementFactory.make("audioconvert", None)
            caps = Gst.ElementFactory.make("capsfilter", None)
            caps.set_property("caps", Gst.Caps.from_string("audio/x-raw,format=F32LE,channels=1,layout=interleaved"))
            analyzer_sink = Gst.ElementFactory.make("appsink", "analysis-sink")
            analyzer_sink.set_property("emit-signals", True)
            analyzer_sink.set_property("sync", False)
            analyzer_sink.set_property("max-buffers", 4)
            analyzer_sink.set_property("drop", True)
            analyzer_sink.connect("new-sample", self._new_analysis_sample)
            core = [decode, conv1, resample, input_vol, invert, hpf, lpf, gate, eq5, eq10, opto, vca, output, tee]
            capture = [capture_queue, capture_convert, caps, analyzer_sink]
            chain = core + playback + capture
            if not all(chain): raise RuntimeError("Required GStreamer element unavailable")
            p.add(*chain); Gst.Element.link_many(*core[1:]); Gst.Element.link_many(*playback); Gst.Element.link_many(*capture)
            tee.link(playback[0]); tee.link(capture[0])
        decode.connect("pad-added", self._on_pad_added, conv1)
        self.processors = {"input": input_vol, "invert": invert, "hpf": hpf, "lpf": lpf, "gate": gate, "eq5": eq5, "eq10": eq10, "opto": opto, "vca": vca, "output": output}
        self._apply_controls()
        bus = p.get_bus(); bus.add_signal_watch(); bus.connect("message", self._bus_message)
        return p

    def _new_analysis_sample(self, sink):
        sample = sink.emit("pull-sample")
        if not sample: return Gst.FlowReturn.OK
        buffer = sample.get_buffer()
        ok, mapped = buffer.map(Gst.MapFlags.READ)
        if not ok: return Gst.FlowReturn.OK
        try:
            incoming = np.frombuffer(mapped.data, dtype="<f4").copy()
        finally:
            buffer.unmap(mapped)
        if not len(incoming): return Gst.FlowReturn.OK
        incoming = np.nan_to_num(incoming, nan=0.0, posinf=0.0, neginf=0.0)
        with self.analysis_lock:
            self.analysis_samples = np.concatenate((self.analysis_samples, incoming))[-8192:]
            window = self.analysis_samples[-1024:] * np.hanning(1024)
            values = np.abs(np.fft.rfft(window))[:128]
            values = np.clip((20 * np.log10(np.maximum(values, 1e-7)) + 72) / 72, 0, 1)
            self.spectrogram = np.roll(self.spectrogram, -1, axis=0)
            self.spectrogram[-1] = values
        return Gst.FlowReturn.OK

    @staticmethod
    def _on_pad_added(src, pad, target):
        sink = target.get_static_pad("sink")
        if not sink.is_linked() and pad.query_caps(None).to_string().startswith("audio/"): pad.link(sink)

    def _open_dialog(self, *_):
        dialog = Gtk.FileChooserNative(title="Open audio", transient_for=self.win, action=Gtk.FileChooserAction.OPEN, accept_label="Open")
        filt = Gtk.FileFilter(); filt.set_name("Audio files"); filt.add_mime_type("audio/*"); filt.add_pattern("*.wav"); filt.add_pattern("*.mp3"); filt.add_pattern("*.flac"); filt.add_pattern("*.ogg")
        dialog.add_filter(filt); dialog.connect("response", self._open_response); dialog.show()

    def _open_response(self, dialog, response):
        if response == Gtk.ResponseType.ACCEPT and (file := dialog.get_file()):
            self.file_path = file.get_path(); self.status.set_label(Path(self.file_path).name)
            self._build_preview()

    def _build_preview(self):
        self._stop_pipeline(); self.pipeline = self._new_pipeline(); self.play_btn.set_sensitive(True); self.pause_btn.set_sensitive(True)
        self._set_playing(True)

    def _set_playing(self, playing):
        if self.pipeline:
            self.pipeline.set_state(Gst.State.PLAYING if playing else Gst.State.PAUSED); self.playing = playing

    def _stop_pipeline(self):
        if self.pipeline: self.pipeline.set_state(Gst.State.NULL); self.pipeline = None

    def _seek(self, *_):
        if not self.position_updating and self.pipeline and self.duration:
            self.pipeline.seek_simple(Gst.Format.TIME, Gst.SeekFlags.FLUSH | Gst.SeekFlags.KEY_UNIT, int(self.position.get_value() / 100 * self.duration))

    def _tick(self):
        if self.pipeline:
            ok, duration = self.pipeline.query_duration(Gst.Format.TIME)
            if ok: self.duration = duration
            ok, position = self.pipeline.query_position(Gst.Format.TIME)
            if ok and self.duration:
                self.position_updating = True
                self.position.set_value(position / self.duration * 100)
                self.position_updating = False
            self.time_label.set_label(f"{self._clock(position if ok else 0)} / {self._clock(self.duration)}")
            reduction = max(0, self._value("opto_thresh") + 6)
            self.opto_light.set_css_classes(["meter", "on"] if reduction else ["meter"])
        return True
    @staticmethod
    def _clock(ns):
        sec = int(ns / Gst.SECOND); return f"{sec // 60:02}:{sec % 60:02}"

    def _export_dialog(self, *_):
        if not self.file_path: self.status.set_label("Open audio before exporting"); return
        dialog = Gtk.FileChooserNative(title="Export processed audio", transient_for=self.win, action=Gtk.FileChooserAction.SAVE, accept_label="Export")
        dialog.set_current_name(Path(self.file_path).stem + " - Channel Strip.wav")
        dialog.connect("response", self._export_response); dialog.show()
    def _export_response(self, dialog, response):
        if response == Gtk.ResponseType.ACCEPT and (file := dialog.get_file()):
            self._stop_pipeline(); self.exporting = True; self.status.set_label("Exporting processed WAV…")
            self.pipeline = self._new_pipeline(file.get_path()); self.pipeline.set_state(Gst.State.PLAYING)

    def _bus_message(self, bus, message):
        if message.type == Gst.MessageType.ERROR:
            err, debug = message.parse_error(); self.status.set_label(f"Error: {err.message}"); self._stop_pipeline()
        elif message.type == Gst.MessageType.EOS:
            if self.exporting:
                self.status.set_label("Export complete"); self.exporting = False; self._stop_pipeline(); self._build_preview()
            else: self._set_playing(False); self.pipeline.seek_simple(Gst.Format.TIME, Gst.SeekFlags.FLUSH, 0)
    def _close(self, *_): self._stop_pipeline(); return False

if __name__ == "__main__":
    raise SystemExit(ChannelStrip().run(sys.argv))
