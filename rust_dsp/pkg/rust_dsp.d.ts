/* tslint:disable */
/* eslint-disable */

export class CompressorInstance {
    free(): void;
    [Symbol.dispose](): void;
    constructor();
    process(buffer: DspBuffer): void;
    end: CompressorInstanceParams;
    start: CompressorInstanceParams;
}

export class CompressorInstanceParams {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    attack: number;
    decay: number;
    freq_lo_mid: number;
    freq_mid_hi: number;
    hi_gain: number;
    lo_gain: number;
    mid_gain: number;
    ratio_down: number;
    ratio_up: number;
    threshold: number;
}

export class CompressorParams {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    attack: number;
    decay: number;
    knee: number;
    ratio_down: number;
    ratio_up: number;
    threshold: number;
}

export class DspBuffer {
    free(): void;
    [Symbol.dispose](): void;
    constructor(frame_size: number);
    readonly buffer: Float32Array;
    run_length: number;
    sample_rate: number;
}

export class FlangerInstance {
    free(): void;
    [Symbol.dispose](): void;
    constructor();
    process(buffer: DspBuffer): void;
    end: FlangerInstanceParams;
    start: FlangerInstanceParams;
}

export class FlangerInstanceParams {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    delay: number;
    feedmix: number;
    mix: number;
    panning: number;
}

export class PhaserInstance {
    free(): void;
    [Symbol.dispose](): void;
    begin(sample_rate: number, run_length: number): void;
    constructor(frame_size: number);
    process(sample: number): number;
    disperse: boolean;
    end: PhaserInstanceParams;
    frame_size: number;
    start: PhaserInstanceParams;
    set legacy_behavior(value: boolean);
    set num_stages(value: number);
}

export class PhaserInstanceParams {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    feedback: number;
    freq: number;
    mix: number;
}

export function start(): void;
