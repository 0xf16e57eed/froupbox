// Copyright (c) 2012-2022 John Nesky and contributing authors, distributed under the MIT license, see accompanying the LICENSE.md file.

//import { InstrumentType, Config } from "../synth/SynthConfig";
//import { NotePin, Note, makeNotePin, Pattern, Instrument, Channel, Song, Synth } from "../synth/synth";
//import { Preset, EditorConfig } from "./EditorConfig";
import { SongDocument } from "./SongDocument";
import { Prompt } from "./Prompt";
import { HTML } from "imperative-html/dist/esm/elements-strict";
//import { ChangeGroup } from "./Change";
import { /*removeDuplicatePatterns,*/ ChangeSong/*, ChangeReplacePatterns*/ } from "./changes";
//import { AnalogousDrum, analogousDrumMap, MidiChunkType, MidiFileFormat, MidiEventType, MidiControlEventMessage, MidiMetaEventMessage, MidiRegisteredParameterNumberMSB, MidiRegisteredParameterNumberLSB, midiVolumeToVolumeMult, midiExpressionToVolumeMult } from "./Midi";
//import { ArrayBufferReader } from "./ArrayBufferReader";

const { button, p, div, h2, input, select, option } = HTML;

export class ImportPrompt implements Prompt {
    private readonly _fileInput: HTMLInputElement = input({ type: "file", accept: ".json,application/json,audio/midi,audio/x-midi" });
    private readonly _cancelButton: HTMLButtonElement = button({ class: "cancelButton" });
    private readonly _modeImportSelect: HTMLSelectElement = select({ style: "width: 100%;" },
        option({ value: "auto" }, "Auto-detect mode (for json)"),
        option({ value: "BeepBox" }, "BeepBox"),
        option({ value: "ModBox" }, "ModBox"),
        option({ value: "JummBox" }, "JummBox"),
        option({ value: "SynthBox" }, "SynthBox"),
        option({ value: "GoldBox" }, "GoldBox"),
        option({ value: "PaandorasBox" }, "PaandorasBox"),
        // Currently this option is unnecessary (UB is handled the same as JB) but we're keeping it in case there's any future conflicts
        // There's also the situation where someone will see the "GoldBox" or "PaandorasBox" options and think they have to use one of those two
        option({ value: "UltraBox" }, "UltraBox"),
        option({ value: "slarmoosbox"}, "Slarmoo's Box"),
        option({ value: "froupbox"}, "froupbox")
    );

    public readonly container: HTMLDivElement = div({ class: "prompt noSelection", style: "width: 300px;" },
        h2("Import"),
        p({ style: "text-align: left; margin: 0.5em 0;" },
            "BeepBox songs can be exported and re-imported as .json files. You could also use other means to make .json files for BeepBox as long as they follow the same structure.",
        ),
        this._modeImportSelect,
        this._fileInput,
        this._cancelButton,
    );

    constructor(private _doc: SongDocument) {
        this._fileInput.select();
        setTimeout(() => this._fileInput.focus());

        this._fileInput.addEventListener("change", this._whenFileSelected);
        this._cancelButton.addEventListener("click", this._close);
    }

    private _close = (): void => {
        this._doc.undo();
    }

    public cleanUp = (): void => {
        this._fileInput.removeEventListener("change", this._whenFileSelected);
        this._cancelButton.removeEventListener("click", this._close);
    }

    private _whenFileSelected = (): void => {
        const file: File = this._fileInput.files![0];
        if (!file) return;

        const extension: string = file.name.slice((file.name.lastIndexOf(".") - 1 >>> 0) + 2).toLowerCase();
        if (extension == "json") {
            const reader: FileReader = new FileReader();
            reader.addEventListener("load", (event: Event): void => {
                this._doc.prompt = null;
                this._doc.goBackToStart();
                this._doc.record(new ChangeSong(this._doc, <string>reader.result, this._modeImportSelect.value), true, true);
            });
            reader.readAsText(file);
        } else {
            console.error("Unrecognized file extension.");
            this._close();
        }
    }
}