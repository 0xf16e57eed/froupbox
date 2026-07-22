// Copyright (c) 2012-2022 John Nesky and contributing authors, distributed under the MIT license, see accompanying the LICENSE.md file.

import { Synth } from "../synth/synth";
import { ColorConfig } from "./ColorConfig";
import { SongDocument } from "./SongDocument";
import { Prompt } from "./Prompt";
import { HTML } from "imperative-html/dist/esm/elements-strict";

const { button, div, h2, input, select, option} = HTML;

function save(blob: Blob, name: string): void {
    if ((<any>navigator).msSaveOrOpenBlob) {
        (<any>navigator).msSaveOrOpenBlob(blob, name);
        return;
    }

    const anchor: HTMLAnchorElement = document.createElement("a");
    if (anchor.download != undefined) {
        const url: string = URL.createObjectURL(blob);
        setTimeout(function () { URL.revokeObjectURL(url); }, 60000);
        anchor.href = url;
        anchor.download = name;
        // Chrome bug regression: We need to delay dispatching the click
        // event. Seems to be related to going back in the browser history.
        // https://bugs.chromium.org/p/chromium/issues/detail?id=825100
        setTimeout(function () { anchor.dispatchEvent(new MouseEvent("click")); }, 0);
    } else {
        const url: string = URL.createObjectURL(blob);
        setTimeout(function () { URL.revokeObjectURL(url); }, 60000);
        if (!window.open(url, "_blank")) window.location.href = url;
    }
}

export class ExportPrompt implements Prompt {
    private synth: Synth;
    private thenExportTo: string;
    private recordedSamplesL: Float32Array;
    private recordedSamplesR: Float32Array;
    private sampleFrames: number;
    private totalChunks: number;
    private currentChunk: number;
    private samplesPerChunk: number;
    private outputStarted: boolean = false;
    private readonly _fileName: HTMLInputElement = input({ type: "text", style: "width: 10em;", value: "BeepBox-Song", maxlength: 250, "autofocus": "autofocus" });
    private readonly _computedSamplesLabel: HTMLDivElement = div({ style: "width: 10em;" }, new Text("0:00"));
    private readonly _enableIntro: HTMLInputElement = input({ type: "checkbox" });
    private readonly _loopDropDown: HTMLInputElement = input({ style: "width: 3em;", type: "number", min: "1", max: "16", step: "1" });
    private readonly _enableOutro: HTMLInputElement = input({ type: "checkbox" });
    private readonly _formatSelect: HTMLSelectElement = select({ style: "width: 100%;" },
        option({ value: "wav" }, "Export to .wav file."),
        option({ value: "mp3" }, "Export to .mp3 file."),
	    option({ value: "ogg" }, "Export to .ogg file."),
        option({ value: "opus" }, "Export to .opus file."),
        option({ value: "json" }, "Export to .json file."),
        option({ value: "html" }, "Export to .html file."),
    );
    private readonly _removeWhitespace: HTMLInputElement = input({ type: "checkbox" });
    private readonly _removeWhitespaceDiv: HTMLDivElement = div({ style: "vertical-align: middle; align-items: center; justify-content: space-between; margin-bottom: 14px;" },
    "Remove Whitespace: ", this._removeWhitespace);
    private readonly _cancelButton: HTMLButtonElement = button({ class: "cancelButton" });
    private readonly _exportButton: HTMLButtonElement = button({ class: "exportButton", style: "width:45%;" }, "Export");
    private readonly _outputProgressBar: HTMLDivElement = div({ style: `width: 0%; background: ${ColorConfig.loopAccent}; height: 100%; position: absolute; z-index: 2;` });
    private readonly _outputProgressLabel: HTMLDivElement = div({ style: `position: relative; top: -1px; z-index: 3;` }, "0%");
    private readonly _outputProgressContainer: HTMLDivElement = div({ style: `height: 12px; background: ${ColorConfig.uiWidgetBackground}; display: block; position: relative; z-index: 1; margin-bottom: 14px;` },
        this._outputProgressBar,
        this._outputProgressLabel,
    );

    public _exportPrompt: HTMLDivElement = div({},
        div({class:"promptTitle",style:"margin-bottom: 14px;"}, h2({class:"exportExt",style:"text-align: inherit;"}, ""), h2({class:"exportTitle"},"Export Options")),
        div({ style: "display: flex; flex-direction: row; align-items: center; justify-content: space-between; margin-bottom: 14px;" },
            "File name:",
            this._fileName,
        ),
        div({ style: "display: flex; flex-direction: row; align-items: center; justify-content: space-between; margin-bottom: 14px;" },
            "Length:",
            this._computedSamplesLabel,
        ),
        div({ style: "display: table; width: 100%; margin-bottom: 14px;" },
            div({ style: "display: table-row;" },
                div({ style: "display: table-cell;" }, "Intro:"),
                div({ style: "display: table-cell;" }, "Loop Count:"),
                div({ style: "display: table-cell;" }, "Outro:"),
            ),
            div({ style: "display: table-row; margin-bottom: 14px;" },
                div({ style: "display: table-cell; vertical-align: middle;" }, this._enableIntro),
                div({ style: "display: table-cell; vertical-align: middle;" }, this._loopDropDown),
                div({ style: "display: table-cell; vertical-align: middle;" }, this._enableOutro),
            ),
        ),
        this._removeWhitespaceDiv,
        div({ class: "selectContainer", style: "width: 100%; margin-bottom: 14px;" }, this._formatSelect),
        div({ style: "text-align: left; margin-bottom: 14px;" }, "Exporting can be slow. Reloading the page or clicking the X will cancel it. Please be patient."),
        this._outputProgressContainer,
        div({ style: "display: flex; flex-direction: row-reverse; justify-content: space-between; margin-bottom: 14px;" },
            this._exportButton,
        ),
        this._cancelButton,
    );

    public readonly container: HTMLDivElement = div({ class: "prompt noSelection", style: "width: 200px;" },
        this._exportPrompt,
    );

    constructor(private _doc: SongDocument) {
        this._loopDropDown.value = "1";

        if (this._doc.song.loopStart == 0) {
            this._enableIntro.checked = false;
            this._enableIntro.disabled = true;
        } else {
            this._enableIntro.checked = true;
            this._enableIntro.disabled = false;
        }
        if (this._doc.song.loopStart + this._doc.song.loopLength == this._doc.song.barCount) {
            this._enableOutro.checked = false;
            this._enableOutro.disabled = true;
        } else {
            this._enableOutro.checked = true;
            this._enableOutro.disabled = false;
        }

        const lastExportFormat: string | null = window.localStorage.getItem("exportFormat");
        if (lastExportFormat != null) {
            this._formatSelect.value = lastExportFormat;
        }

        const lastExportWhitespace: boolean = window.localStorage.getItem("exportWhitespace") == "true";
        if (lastExportWhitespace != null) {
            this._removeWhitespace.checked = lastExportWhitespace;
        }

        if (this._formatSelect.value == "json") {
            this._removeWhitespaceDiv.style.display = "block";
        } else {
            this._removeWhitespaceDiv.style.display = "none";
        }

        this._fileName.select();
        setTimeout(() => this._fileName.focus());

        this._fileName.addEventListener("input", ExportPrompt._validateFileName);
        this._loopDropDown.addEventListener("blur", ExportPrompt._validateNumber);
        this._exportButton.addEventListener("click", this._export);
        this._cancelButton.addEventListener("click", this._close);
        this._enableOutro.addEventListener("click", () => { (this._computedSamplesLabel.firstChild as Text).textContent = this._doc.samplesToTime(this._doc.synth.getTotalSamples(this._enableIntro.checked, this._enableOutro.checked, +this._loopDropDown.value - 1)); });
        this._enableIntro.addEventListener("click", () => { (this._computedSamplesLabel.firstChild as Text).textContent = this._doc.samplesToTime(this._doc.synth.getTotalSamples(this._enableIntro.checked, this._enableOutro.checked, +this._loopDropDown.value - 1)); });
        this._loopDropDown.addEventListener("change", () => { (this._computedSamplesLabel.firstChild as Text).textContent = this._doc.samplesToTime(this._doc.synth.getTotalSamples(this._enableIntro.checked, this._enableOutro.checked, +this._loopDropDown.value - 1)); });
        this._formatSelect.addEventListener("change", () => { if (this._formatSelect.value == "json") { this._removeWhitespaceDiv.style.display = "block"; } else {  this._removeWhitespaceDiv.style.display = "none"; } });
        this.container.addEventListener("keydown", this._whenKeyPressed);

        this._fileName.value = _doc.song.title;
        ExportPrompt._validateFileName(null, this._fileName);

        (this._computedSamplesLabel.firstChild as Text).textContent = this._doc.samplesToTime(this._doc.synth.getTotalSamples(this._enableIntro.checked, this._enableOutro.checked, +this._loopDropDown.value - 1));
    
        if (this._doc.prompt == "quickExport") {
            this._export();
        }
    }

    private _close = (): void => {
        if (this.synth != null)
            this.synth.renderingSong = false;
        this.outputStarted = false;
        this._doc.undo();
    }

    public changeFileName(newValue: string) {
        this._fileName.value = newValue;
    }

    public cleanUp = (): void => {
        this._fileName.removeEventListener("input", ExportPrompt._validateFileName);
        this._loopDropDown.removeEventListener("blur", ExportPrompt._validateNumber);
        this._exportButton.removeEventListener("click", this._export);
        this._cancelButton.removeEventListener("click", this._close);
        this.container.removeEventListener("keydown", this._whenKeyPressed);
    }

    private _whenKeyPressed = (event: KeyboardEvent): void => {
        if ((<Element>event.target).tagName != "BUTTON" && event.keyCode == 13) { // Enter key
            this._export();
        }
    }

    private static _validateFileName(event: Event | null, use?: HTMLInputElement): void {

        let input: HTMLInputElement;
        if (event != null) {
            input = <HTMLInputElement>event.target;
        } else if (use != undefined) {
            input = use;
        }
        else {
            return;
        }
        const deleteChars = /[\+\*\$\?\|\{\}\\\/<>#%!`&'"=:@]/gi;
        if (deleteChars.test(input.value)) {
            let cursorPos: number = <number>input.selectionStart;
            input.value = input.value.replace(deleteChars, "");
            cursorPos--;
            input.setSelectionRange(cursorPos, cursorPos);
        }
    }

    private static _validateNumber(event: Event): void {
        const input: HTMLInputElement = <HTMLInputElement>event.target;
        input.value = Math.floor(Math.max(Number(input.min), Math.min(Number(input.max), Number(input.value)))) + "";
    }

    private _export = (): void => {
        if (this.outputStarted == true)
            return;
        window.localStorage.setItem("exportFormat", this._formatSelect.value);
        window.localStorage.setItem("exportWhitespace", this._removeWhitespace.value);
        window.localStorage.setItem("exportFormat", this._formatSelect.value);
        switch (this._formatSelect.value) {
            case "wav":
                this.outputStarted = true;
                this._exportTo("wav");
                break;
            case "mp3":
                this.outputStarted = true;
                this._exportTo("mp3");
                break;
            case "ogg":
                this.outputStarted = true;
                this._exportTo("ogg");
                break;   
            case "opus":
                this.outputStarted = true;
                this._exportTo("opus");
                break;
            case "json":
                this.outputStarted = true;
                this._exportToJson();
                break;
            case "html":
                this._exportToHtml();
                break;
            default:
                throw new Error("Unhandled file export type.");
        }
    }

    private _synthesize(): void {
        //const timer: number = performance.now();

        // If output was stopped e.g. user clicked the close button, abort.
        if (this.outputStarted == false) {
            return;
        }

        const currentFrame: number = this.currentChunk * this.samplesPerChunk;

        const samplesInChunk: number = Math.min(this.samplesPerChunk, this.sampleFrames - currentFrame);
        const tempSamplesL = new Float32Array(samplesInChunk);
        const tempSamplesR = new Float32Array(samplesInChunk);

        this.synth.renderingSong = true;
        this.synth.synthesize(tempSamplesL, tempSamplesR, samplesInChunk);

        // Concatenate chunk data into final array
        this.recordedSamplesL.set(tempSamplesL, currentFrame);
        this.recordedSamplesR.set(tempSamplesR, currentFrame);

        // Update UI
        this._outputProgressBar.style.setProperty("width", Math.round((this.currentChunk + 1) / this.totalChunks * 100.0) + "%");
        this._outputProgressLabel.innerText = Math.round((this.currentChunk + 1) / this.totalChunks * 100.0) + "%";

        // Next call, synthesize the next chunk.
        this.currentChunk++;

        if (this.currentChunk >= this.totalChunks) {
            // Done, call final function
            this.synth.renderingSong = false;
            this._outputProgressLabel.innerText = "Encoding...";
            if (this.thenExportTo == "wav") {
                this._exportToWavFinish();
            }
            else if (this.thenExportTo == "mp3") {
                this._exportToMp3Finish();
            }
            else if (this.thenExportTo == "ogg") {
                this._exportToOggFinish();
            }
            else if (this.thenExportTo == "opus") {
                this._exportToOpusFinish();
            }
            else {
                throw new Error("Unrecognized file export type chosen!");
            }
        }
        else {
            // Continue batch export
            setTimeout(() => { this._synthesize(); });
        }

        //console.log("export timer", (performance.now() - timer) / 1000.0);
    }

    private _exportTo(type: string): void {
        // Batch the export operation
        this.thenExportTo = type;
        this.currentChunk = 0;
        this.synth = new Synth(this._doc.song);
        if (type == "wav") {
            this.synth.samplesPerSecond = 48000; // Use professional video editing standard sample rate for .wav file export.
        }
        else if (type == "mp3") {
            this.synth.samplesPerSecond = 44100; // Use consumer CD standard sample rate for .mp3 export.
        }
        else if (type == "ogg") {
            this.synth.samplesPerSecond = 48000; // Wikipedia says ogg typically uses 44.1 kHz.
        } 
        else if (type == "opus") {
            this.synth.samplesPerSecond = 48000; // Wikipedia says ogg typically uses 44.1 kHz.
        } 
        else {
            throw new Error("Unrecognized file export type chosen!");
        }

        this._outputProgressBar.style.setProperty("width", "0%");
        this._outputProgressLabel.innerText = "0%";

        this.synth.loopRepeatCount = Number(this._loopDropDown.value) - 1;
        if (!this._enableIntro.checked) {
            for (let introIter: number = 0; introIter < this._doc.song.loopStart; introIter++) {
                this.synth.goToNextBar();
            }
        }

        
        this.synth.initModFilters(this._doc.song);
        this.synth.computeLatestModValues();
	      this.synth.warmUpSynthesizer(this._doc.song);

        this.sampleFrames = this.synth.getTotalSamples(this._enableIntro.checked, this._enableOutro.checked, this.synth.loopRepeatCount);
        // Compute how many UI updates will need to run to determine how many 
        // Update progress bar UI once per 5 sec of exported data
        this.samplesPerChunk = this.synth.samplesPerSecond * 5;
        this.totalChunks = Math.ceil(this.sampleFrames / this.samplesPerChunk);
        this.recordedSamplesL = new Float32Array(this.sampleFrames);
        this.recordedSamplesR = new Float32Array(this.sampleFrames);

        // Batch the actual export
        setTimeout(() => { this._synthesize(); });
    }

    private _exportToWavFinish(): void {
        const sampleFrames: number = this.recordedSamplesL.length;
        const sampleRate: number = this.synth.samplesPerSecond;

        const wavChannelCount: number = 2;
        const bytesPerSample: number = 2;
        const bitsPerSample: number = 8 * bytesPerSample;
        const sampleCount: number = wavChannelCount * sampleFrames;

        const totalFileSize: number = 44 + sampleCount * bytesPerSample;

        let index: number = 0;
        const arrayBuffer: ArrayBuffer = new ArrayBuffer(totalFileSize);
        const data: DataView = new DataView(arrayBuffer);
        data.setUint32(index, 0x52494646, false); index += 4;
        data.setUint32(index, 36 + sampleCount * bytesPerSample, true); index += 4; // size of remaining file
        data.setUint32(index, 0x57415645, false); index += 4;
        data.setUint32(index, 0x666D7420, false); index += 4;
        data.setUint32(index, 0x00000010, true); index += 4; // size of following header
        data.setUint16(index, 0x0001, true); index += 2; // not compressed
        data.setUint16(index, wavChannelCount, true); index += 2; // channel count
        data.setUint32(index, sampleRate, true); index += 4; // sample rate
        data.setUint32(index, sampleRate * bytesPerSample * wavChannelCount, true); index += 4; // bytes per second
        data.setUint16(index, bytesPerSample * wavChannelCount, true); index += 2; // block align
        data.setUint16(index, bitsPerSample, true); index += 2; // bits per sample
        data.setUint32(index, 0x64617461, false); index += 4;
        data.setUint32(index, sampleCount * bytesPerSample, true); index += 4;

        if (bytesPerSample > 1) {
            // usually samples are signed. 
            const range: number = (1 << (bitsPerSample - 1)) - 1;
            for (let i: number = 0; i < sampleFrames; i++) {
                let valL: number = Math.floor(Math.max(-1, Math.min(1, this.recordedSamplesL[i])) * range);
                let valR: number = Math.floor(Math.max(-1, Math.min(1, this.recordedSamplesR[i])) * range);
                if (bytesPerSample == 2) {
                    data.setInt16(index, valL, true); index += 2;
                    data.setInt16(index, valR, true); index += 2;
                } else if (bytesPerSample == 4) {
                    data.setInt32(index, valL, true); index += 4;
                    data.setInt32(index, valR, true); index += 4;
                } else {
                    throw new Error("unsupported sample size");
                }
            }
        } else {
            // 8 bit samples are a special case: they are unsigned.
            for (let i: number = 0; i < sampleFrames; i++) {
                let valL: number = Math.floor(Math.max(-1, Math.min(1, this.recordedSamplesL[i])) * 127 + 128);
                let valR: number = Math.floor(Math.max(-1, Math.min(1, this.recordedSamplesR[i])) * 127 + 128);
                data.setUint8(index, valL > 255 ? 255 : (valL < 0 ? 0 : valL)); index++;
                data.setUint8(index, valR > 255 ? 255 : (valR < 0 ? 0 : valR)); index++;
            }
        }

        const blob: Blob = new Blob([arrayBuffer], { type: "audio/wav" });
        save(blob, this._fileName.value.trim() + ".wav");

        this._close();
    }

    private _exportToMp3Finish(): void {
        const whenEncoderIsAvailable = (): void => {

            const lamejs: any = (<any>window)["lamejs"];
            const channelCount: number = 2;
            const kbps: number = 192;
            const sampleBlockSize: number = 1152;
            const mp3encoder: any = new lamejs.Mp3Encoder(channelCount, this.synth.samplesPerSecond, kbps);
            const mp3Data: any[] = [];

            const left: Int16Array = new Int16Array(this.recordedSamplesL.length);
            const right: Int16Array = new Int16Array(this.recordedSamplesR.length);
            const range: number = (1 << 15) - 1;
            for (let i: number = 0; i < this.recordedSamplesL.length; i++) {
                left[i] = Math.floor(Math.max(-1, Math.min(1, this.recordedSamplesL[i])) * range);
                right[i] = Math.floor(Math.max(-1, Math.min(1, this.recordedSamplesR[i])) * range);
            }

            for (let i: number = 0; i < left.length; i += sampleBlockSize) {
                const leftChunk: Int16Array = left.subarray(i, i + sampleBlockSize);
                const rightChunk: Int16Array = right.subarray(i, i + sampleBlockSize);
                const mp3buf: any = mp3encoder.encodeBuffer(leftChunk, rightChunk);
                if (mp3buf.length > 0) mp3Data.push(mp3buf);
            }

            const mp3buf: any = mp3encoder.flush();
            if (mp3buf.length > 0) mp3Data.push(mp3buf);

            const blob: Blob = new Blob(mp3Data, { type: "audio/mp3" });
            save(blob, this._fileName.value.trim() + ".mp3");
            this._close();
        }

        if ("lamejs" in window) {
            whenEncoderIsAvailable();
        } else {
            var script = document.createElement("script");
            script.src = "https://cdn.jsdelivr.net/npm/lamejs@1.2.0/lame.min.js";
            script.onload = whenEncoderIsAvailable;
            document.head.appendChild(script);
        }
    }

    private _exportToOggFinish(): void {
        const scripts: string[] = [
            "https://unpkg.com/wasm-media-encoders/dist/umd/WasmMediaEncoder.min.js",
        ];
        let scriptsLoaded: number = 0;
        const scriptsToLoad: number = scripts.length;
        const whenEncoderIsAvailable = (): void => {
            scriptsLoaded++;
            if (scriptsLoaded < scriptsToLoad) return;
            const WasmMediaEncoder: any = (<any>window)["WasmMediaEncoder"];
            const channelCount: number = 2;
            const quality: number = 10;
            const sampleBlockSize: number = 4096;
            WasmMediaEncoder.createOggEncoder().then((oggEncoder: any) => {
                oggEncoder.configure({
                    channels: channelCount,
                    sampleRate: this.synth.samplesPerSecond,
                    vbrQuality: quality,
                });
                const left: Float32Array = this.recordedSamplesL;
                const right: Float32Array = this.recordedSamplesR;
                const parts: Uint8Array[] = [];
                let sampleIndex: number = 0;
                for (; sampleIndex < left.length; sampleIndex += sampleBlockSize) {
                    const leftChunk: Float32Array = left.subarray(sampleIndex, sampleIndex + sampleBlockSize);
                    const rightChunk: Float32Array = right.subarray(sampleIndex, sampleIndex + sampleBlockSize);
                    const frame: Float32Array[] = channelCount === 2 ? ([leftChunk, rightChunk]) : ([leftChunk]);
                    parts.push(oggEncoder.encode(frame).slice());
                }
                parts.push(oggEncoder.finalize().slice());
                const blob: Blob = new Blob(parts, { type: "audio/ogg" });
                save(blob, this._fileName.value.trim() + ".ogg");
                this._close();
            });
        }
        if ("WasmMediaEncoder" in window) {
            scriptsLoaded = scripts.length;
            whenEncoderIsAvailable();
        } else {
            scriptsLoaded = 0;
            for (const src of scripts) {
                const script = document.createElement("script");
                script.src = src;
                script.onload = whenEncoderIsAvailable;
                document.head.appendChild(script);
            }
        }
    }
    private _exportToOpusFinish(): void {
        const scripts: string[] = [
            "https://cdn.jsdelivr.net/gh/mmig/opus-encdec@e33ca40b92ddff8c168c7f5aca34b626c9acc08a/dist/libopus-encoder.js",
            "https://cdn.jsdelivr.net/gh/mmig/opus-encdec@e33ca40b92ddff8c168c7f5aca34b626c9acc08a/src/oggOpusEncoder.js"
        ];
        let scriptsLoaded: number = 0;
        const scriptsToLoad: number = scripts.length;
        const whenEncoderIsAvailable = (): void => {
            scriptsLoaded++;
            if (scriptsLoaded < scriptsToLoad) return;
            const OggOpusEncoder: any = (<any>window)["OggOpusEncoder"];
            const OpusEncoderLib: any = (<any>window)["OpusEncoderLib"];
            // @TODO: Very non-ideal.
            OggOpusEncoder.prototype.getOpusControl = function (control: number): number | null {
                let result: number | null = null;
                // Hack to defeat Terser's mangling. Alternatively, the
                // compilation scripts could be changed.
                const doNotMangle: string = Math.random() > 2 ? "" : "";
                const location: number = this["_" + doNotMangle + "malloc"](4);
                const outputLocation: number = this["_" + doNotMangle + "malloc"](4);
                this.HEAP32[location >> 2] = outputLocation;
                const returnCode: number = this["_" + doNotMangle + "opus_encoder_ctl"](this.encoder, control, location);
                if (returnCode === 0) {
                    result = this.HEAP32[outputLocation >> 2];
                }
                this["_" + doNotMangle + "free"](outputLocation);
                this["_" + doNotMangle + "free"](location);
                return result;
            };
            OggOpusEncoder.prototype.getLookahead = function (): number {
                return this.getOpusControl(4027) ?? 0;
            };
            OggOpusEncoder.prototype.setBitrate = function (value: number): void {
                this.setOpusControl(4002, value);
            };
            OggOpusEncoder.prototype.generateIdPage2 = function (lookahead: number): any {
                const segmentDataView: DataView = new DataView(this.segmentData.buffer);
                segmentDataView.setUint32(0, 1937076303, true); // Magic Signature 'Opus'
                segmentDataView.setUint32(4, 1684104520, true); // Magic Signature 'Head'
                segmentDataView.setUint8(8, 1); // Version
                segmentDataView.setUint8(9, this.config.numberOfChannels); // Channel count
                segmentDataView.setUint16(10, lookahead, true); // pre-skip (0ms)
                segmentDataView.setUint32(12, this.config.originalSampleRateOverride || this.config.originalSampleRate, true); // original sample rate
                segmentDataView.setUint16(16, 0, true); // output gain
                segmentDataView.setUint8(18, 0); // channel map 0 = mono or stereo
                this.segmentTableIndex = 1;
                this.segmentDataIndex = this.segmentTable[0] = 19;
                this.headerType = 2;
                return this.generatePage();
            };
            const channelCount: number = 2;
            const frameSizeInMilliseconds: number = 20;
            const frameSizeInSeconds: number = frameSizeInMilliseconds / 1000;
            const sampleBlockSize: number = Math.floor(this.synth.samplesPerSecond * frameSizeInSeconds);
            const oggEncoder: any = new OggOpusEncoder({
                numberOfChannels: channelCount,
                originalSampleRate: this.synth.samplesPerSecond,
                encoderSampleRate: this.synth.samplesPerSecond,
                bufferLength: sampleBlockSize,
                encoderApplication: 2049,
                encoderComplexity: 10,
                resampleQuality: 3, // [0, 10], but we're not using this.
            }, OpusEncoderLib);
            const parts: Uint8Array[] = [];
            const left: Float32Array = this.recordedSamplesL;
            const right: Float32Array = this.recordedSamplesR;
            oggEncoder.setBitrate(256_000); // bits per second
            parts.push(oggEncoder.generateIdPage2(oggEncoder.getLookahead()).page);
            parts.push(oggEncoder.generateCommentPage().page);
            let sampleIndex: number = 0;
            for (; sampleIndex < left.length; sampleIndex += sampleBlockSize) {
                const leftChunk: Float32Array = left.subarray(sampleIndex, sampleIndex + sampleBlockSize);
                const rightChunk: Float32Array = right.subarray(sampleIndex, sampleIndex + sampleBlockSize);
                const frame: Float32Array[] = channelCount === 2 ? ([leftChunk, rightChunk]) : ([leftChunk]);
                oggEncoder.encode(frame).forEach((page: any) => parts.push(page.page));
            }
            // @TODO: This padding matches FFmpeg... but is it correct?
            {
                const paddingSize: number = sampleIndex - left.length;
                const leftChunk: Float32Array = new Float32Array(paddingSize);
                const rightChunk: Float32Array = new Float32Array(paddingSize);
                const frame: Float32Array[] = channelCount === 2 ? ([leftChunk, rightChunk]) : ([leftChunk]);
                oggEncoder.encode(frame).forEach((page: any) => parts.push(page.page));
            }
            // const remaining: any = oggEncoder.flush();
            // if (remaining) parts.push(remaining.page);
            oggEncoder.encodeFinalFrame().forEach((page: any) => parts.push(page.page));
            oggEncoder.destroy();
            const blob: Blob = new Blob(parts, { type: "audio/opus" });
            save(blob, this._fileName.value.trim() + ".opus");
            this._close();
        }
        if (("OggOpusEncoder" in window) && ("OpusEncoderLib" in window)) {
            scriptsLoaded = scripts.length;
            whenEncoderIsAvailable();
        } else {
            scriptsLoaded = 0;
            for (const src of scripts) {
                const script = document.createElement("script");
                script.src = src;
                script.onload = whenEncoderIsAvailable;
                document.head.appendChild(script);
            }
        }
    }

    private _exportToJson(): void {
        const jsonObject: Object = this._doc.song.toJsonObject(this._enableIntro.checked, Number(this._loopDropDown.value), this._enableOutro.checked);
        let whiteSpaceParam: string | undefined = this._removeWhitespace.checked ? undefined : '\t';
        const jsonString: string = JSON.stringify(jsonObject, null, whiteSpaceParam);
        const blob: Blob = new Blob([jsonString], { type: "application/json" });
        save(blob, this._fileName.value.trim() + ".json");
        this._close();
    }

    private _exportToHtml(): void {
        const fileContents = `\
<!DOCTYPE html><meta charset="utf-8">

You should be redirected to the song at:<br /><br />

<a id="destination" href="${new URL("#" + this._doc.song.toBase64String(), location.href).href}"></a>

<style>
	:root {
		color: white;
		background: black;
		font-family:
		sans-serif;
	}
	a {
		color: #98f;
	}
	a[href]::before {
		content: attr(href);
	}
</style>

<script>
	location.assign(document.querySelector("a#destination").href);
</script>
`;
        const blob: Blob = new Blob([fileContents], { type: "text/html" });
        save(blob, this._fileName.value.trim() + ".html");
        this._close();
    }
}
